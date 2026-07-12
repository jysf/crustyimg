//! HEIC/HEIF decode behind the off-by-default `heic` feature (SPEC-062, DEC-056).
//!
//! HEIC is the one common modern format crustyimg cannot put on the default
//! path. **DEC-052** pins two independent blockers, either of which alone forces
//! the gate: the mature pure-Rust HEIC decoders are AGPL (`no-agpl-default-deps`,
//! DEC-018), and HEVC is patent-encumbered (Access Advance pool) on *every*
//! decode path regardless of code license. So a permissive pure-Rust decoder
//! would not un-gate it, and the feature must never appear in a distributed
//! artifact.
//!
//! What that means for this module:
//!
//! - [`is_heic`] is compiled into **both** builds. Detection is what lets the
//!   default build answer a `.heic` with [`ImageError::CodecNotBuilt`] → exit 4
//!   ("rebuild with --features heic") instead of a vague "unsupported format".
//! - `decode_heic` exists only under `--features heic`, where it decodes via
//!   `libheif-rs` → the **system** libheif C library (decode-only) into the
//!   canonical [`crate::image::Image`]. libheif is a codec feeding the pixel
//!   core, not a second pixel library (the AVIF / webp-lossy precedent).
//!
//! HEIC and AVIF are ISOBMFF siblings, so this mirrors [`super::avif`]: brand
//! detection first, caps before allocation, typed errors throughout. AVIF is
//! dispatched *first* in `decode_with_limits` so an AVIF-in-HEIF container
//! (which also carries `mif1`) routes to the pure-Rust AVIF path.
//!
//! ## Security (untrusted-input-hardening)
//!
//! HEIC is hostile binary input handed to a **C** decoder with a long CVE
//! history, so the Rust side does not rely on libheif for bounds:
//!
//! - Dimensions are capped from the image *handle* (container metadata) **before**
//!   `decode` allocates any pixel plane (DEC-034), and re-checked on the decoded
//!   plane as defense in depth.
//! - The interleaved plane is row-padded; every row is copied through a checked
//!   slice honoring `stride`, never a `width * channels` assumption.
//! - The plane's storage depth is validated against the requested chroma before
//!   any pixel is read.
//! - Every libheif failure is a typed [`ImageError`] — no `unwrap`/`expect`/
//!   `panic!` on these paths. `fuzz/fuzz_targets/heic_decode.rs` exercises the
//!   container parse and the decode/copy path together.
//!
//! libheif ≥ 1.19 additionally applies its own internal security limits by
//! default. We pin the `v1_17` API floor for distro compatibility (DEC-056), so
//! the binding does not expose `set_security_limits` to tighten them further;
//! the DEC-034 pre-check is the load-bearing bound either way.

#[cfg(feature = "heic")]
use ::image::{DynamicImage, Limits, RgbImage, RgbaImage};

#[cfg(feature = "heic")]
use crate::error::{ImageError, Result};

/// Whether `bytes` is an ISOBMFF file whose `ftyp` box advertises an HEVC-coded
/// HEIF brand.
///
/// Compiled into **both** builds: the default (no-`heic`) build uses it to
/// return a precise `CodecNotBuilt` rather than a generic decode failure.
///
/// Only HEVC-specific brands count. The generic `mif1`/`msf1` structural brands
/// are deliberately NOT matched — AVIF files carry `mif1` too, and matching it
/// would steal them from the pure-Rust AVIF path (and, in the default build,
/// turn a working `.avif` into an exit 4).
pub(crate) fn is_heic(bytes: &[u8]) -> bool {
    // ftyp box: [size:u32][b"ftyp"][major:4][minor:4][compatible brands: 4*n].
    if bytes.len() < 12 || &bytes[4..8] != b"ftyp" {
        return false;
    }
    let box_size = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    // Clamp the brand scan to the declared box size and the actual buffer.
    let end = box_size.clamp(8, bytes.len());
    // Major brand at 8..12, then compatible brands every 4 bytes from 16.
    if is_heic_brand(&bytes[8..12]) {
        return true;
    }
    let mut i = 16;
    while i + 4 <= end {
        if is_heic_brand(&bytes[i..i + 4]) {
            return true;
        }
        i += 4;
    }
    false
}

/// The HEVC-coded HEIF brands (ISO/IEC 23008-12): still images (`heic`/`heix`),
/// image sequences (`hevc`/`hevx`), and the `heim`/`heis` multiview/scalable
/// profiles.
fn is_heic_brand(brand: &[u8]) -> bool {
    matches!(
        brand,
        b"heic" | b"heix" | b"heim" | b"heis" | b"hevc" | b"hevx"
    )
}

/// Decode a HEIC/HEIF byte stream's primary image to an 8-bit RGB(A)
/// [`DynamicImage`], enforcing the decode caps in `limits` (DEC-034) before any
/// pixel plane is allocated.
///
/// Only the **primary** image is decoded: HEIF image sequences and multi-image
/// collections are out of scope (SPEC-062). 10/12-bit HDR inputs are
/// down-converted to 8-bit RGB(A) by libheif, mirroring the AVIF decision.
#[cfg(feature = "heic")]
pub(crate) fn decode_heic(bytes: &[u8], limits: &Limits) -> Result<DynamicImage> {
    use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};

    let ctx = HeifContext::read_from_bytes(bytes).map_err(map_heif_err)?;
    let handle = ctx.primary_image_handle().map_err(map_heif_err)?;

    // Cap from the container metadata BEFORE decoding, so a decompression-bomb
    // header is rejected without libheif ever allocating a plane (DEC-034).
    check_caps(handle.width(), handle.height(), limits)?;

    let has_alpha = handle.has_alpha_channel();
    let chroma = if has_alpha {
        RgbChroma::Rgba
    } else {
        RgbChroma::Rgb
    };
    let channels = if has_alpha { 4usize } else { 3 };

    let lib = LibHeif::new();
    let img = lib
        .decode(&handle, ColorSpace::Rgb(chroma), None)
        .map_err(map_heif_err)?;

    let planes = img.planes();
    let plane = planes
        .interleaved
        .ok_or_else(|| ImageError::Decode("heic: no interleaved plane".into()))?;

    // Defense in depth: the decoded plane must also satisfy the caps, and must
    // carry exactly the 8-bit-per-channel storage we asked for.
    check_caps(plane.width, plane.height, limits)?;
    if plane.storage_bits_per_pixel as usize != channels * 8 {
        return Err(ImageError::Decode(format!(
            "heic: expected {}-bit interleaved storage, got {}",
            channels * 8,
            plane.storage_bits_per_pixel
        )));
    }

    let (w, h) = (plane.width, plane.height);
    let row_bytes = (w as usize) * channels;
    if plane.stride < row_bytes {
        return Err(ImageError::Decode(format!(
            "heic: plane stride {} shorter than row {row_bytes}",
            plane.stride
        )));
    }

    // Rows are padded to `stride`; copy each row's leading `row_bytes` through a
    // CHECKED slice into a tightly-packed buffer.
    let mut packed = Vec::with_capacity(row_bytes * (h as usize));
    for y in 0..(h as usize) {
        let start = y * plane.stride;
        let row = plane
            .data
            .get(start..start + row_bytes)
            .ok_or_else(|| ImageError::Decode("heic: plane data shorter than declared".into()))?;
        packed.extend_from_slice(row);
    }

    if has_alpha {
        let buf = RgbaImage::from_raw(w, h, packed)
            .ok_or_else(|| ImageError::Decode("heic: rgba buffer size mismatch".into()))?;
        Ok(DynamicImage::ImageRgba8(buf))
    } else {
        let buf = RgbImage::from_raw(w, h, packed)
            .ok_or_else(|| ImageError::Decode("heic: rgb buffer size mismatch".into()))?;
        Ok(DynamicImage::ImageRgb8(buf))
    }
}

/// Reject dimensions that exceed the `limits` (dimension or total allocation) or
/// the shared peak-memory pixel budget (DEC-063).
///
/// The allocation estimate uses the 8-bit RGBA buffer (`w * h * 4`), the largest
/// buffer this module packs. The pixel budget is the tighter, uniform bound: the
/// `heic` feature is off by default, but it decodes through a **C** library, so it
/// gets the same pre-decode peak bound as the pure-Rust paths.
#[cfg(feature = "heic")]
fn check_caps(w: u32, h: u32, limits: &Limits) -> Result<()> {
    super::check_pixel_budget(w, h)?;
    if let Some(max_w) = limits.max_image_width {
        if w > max_w {
            return Err(ImageError::LimitsExceeded(format!(
                "heic width {w} exceeds cap {max_w}"
            )));
        }
    }
    if let Some(max_h) = limits.max_image_height {
        if h > max_h {
            return Err(ImageError::LimitsExceeded(format!(
                "heic height {h} exceeds cap {max_h}"
            )));
        }
    }
    if let Some(max_alloc) = limits.max_alloc {
        let bytes = (w as u64) * (h as u64) * 4;
        if bytes > max_alloc {
            return Err(ImageError::LimitsExceeded(format!(
                "heic buffer {bytes} bytes exceeds alloc cap {max_alloc}"
            )));
        }
    }
    Ok(())
}

/// Map a [`libheif_rs::HeifError`] to a typed [`ImageError`]. Malformed, truncated,
/// and unsupported-but-valid containers all surface as `Decode` with libheif's
/// message — never a panic.
#[cfg(feature = "heic")]
fn map_heif_err(e: libheif_rs::HeifError) -> ImageError {
    ImageError::Decode(format!("heic: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The committed 64×48 solid HEIC fixture. Regen (macOS, OS encoder):
    /// `sips -s format heic solid.png --out tests/fixtures/heic/solid_64x48.heic`
    /// — HEIC encode needs x265/GPL, so this is a committed static asset rather
    /// than a natively-generated fixture (AGENTS §12's deterministic-encoder rule
    /// cannot apply; the file is test data, not code).
    const FIXTURE: &[u8] = include_bytes!("../../tests/fixtures/heic/solid_64x48.heic");

    /// Build a minimal `ftyp` box: size + "ftyp" + major + minor + compat brands.
    fn ftyp(major: &[u8; 4], compat: &[&[u8; 4]]) -> Vec<u8> {
        let size = (16 + 4 * compat.len()) as u32;
        let mut b = Vec::new();
        b.extend_from_slice(&size.to_be_bytes());
        b.extend_from_slice(b"ftyp");
        b.extend_from_slice(major);
        b.extend_from_slice(&0u32.to_be_bytes());
        for c in compat {
            b.extend_from_slice(*c);
        }
        b
    }

    /// Every HEVC brand is detected, as a major brand or a compatible brand.
    #[test]
    fn is_heic_detects_hevc_brands() {
        for brand in [b"heic", b"heix", b"heim", b"heis", b"hevc", b"hevx"] {
            assert!(
                is_heic(&ftyp(brand, &[b"mif1"])),
                "major brand {:?} should be HEIC",
                std::str::from_utf8(brand)
            );
            assert!(
                is_heic(&ftyp(b"mif1", &[b"mif1", brand])),
                "compatible brand {:?} should be HEIC",
                std::str::from_utf8(brand)
            );
        }
    }

    /// AVIF, generic-`mif1`-only HEIF, PNG, and truncated input are NOT HEIC.
    /// The `mif1` case is the important one: AVIF carries it, so matching it here
    /// would hijack the pure-Rust AVIF path.
    #[test]
    fn is_heic_rejects_avif_mif1_and_non_isobmff() {
        assert!(!is_heic(&ftyp(b"avif", &[b"avif", b"mif1", b"miaf"])));
        assert!(!is_heic(&ftyp(b"avis", &[b"avis", b"msf1"])));
        assert!(!is_heic(&ftyp(b"mif1", &[b"mif1", b"miaf"])));
        assert!(!is_heic(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]));
        assert!(!is_heic(b"ftyp")); // too short
        assert!(!is_heic(&[]));
    }

    /// The real fixture (`ftyp` major brand `heic`) is detected in BOTH builds —
    /// this is what drives the default build's exit-4 message.
    #[test]
    fn is_heic_detects_real_fixture() {
        assert!(is_heic(FIXTURE));
    }

    /// A declared box size larger than the buffer must not read out of bounds.
    #[test]
    fn is_heic_clamps_oversized_box_size() {
        let mut b = ftyp(b"mif1", &[b"heic"]);
        b[..4].copy_from_slice(&0xFFFF_FFFFu32.to_be_bytes());
        assert!(is_heic(&b)); // found within the buffer, no panic
        let short = &b[..14];
        assert!(!is_heic(short)); // brand truncated away, still no panic
    }

    // ── `--features heic` only (need the system libheif) ──────────────────────

    /// The committed 64×48 fixture decodes with correct dimensions and pixels.
    #[cfg(feature = "heic")]
    #[test]
    fn decode_heic_solid_dimensions() {
        let img = decode_heic(FIXTURE, &Limits::default()).expect("decode heic fixture");
        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 48);
        // The fixture is a solid #C86432; HEVC is lossy, so allow a tolerance.
        let px = img.to_rgb8().get_pixel(32, 24).0;
        for (got, want) in px.iter().zip([200u8, 100, 50]) {
            assert!(
                (*got as i32 - want as i32).abs() <= 6,
                "pixel {px:?} too far from [200,100,50]"
            );
        }
    }

    /// A dimension cap below the fixture is enforced from the HANDLE dims, before
    /// decode — `LimitsExceeded`, not an OOM or a panic.
    #[cfg(feature = "heic")]
    #[test]
    fn heic_respects_dimension_cap() {
        let mut limits = Limits::default();
        limits.max_image_width = Some(32);
        limits.max_image_height = Some(32);
        let result = decode_heic(FIXTURE, &limits);
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    /// An allocation cap below the fixture's buffer is enforced too.
    #[cfg(feature = "heic")]
    #[test]
    fn heic_respects_alloc_cap() {
        let mut limits = Limits::default();
        limits.max_alloc = Some(16);
        let result = decode_heic(FIXTURE, &limits);
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    /// A truncated fixture, and brand-shaped junk that passes `is_heic` but holds
    /// no image, are typed decode errors — never panics.
    #[cfg(feature = "heic")]
    #[test]
    fn corrupt_heic_is_decode_error_not_panic() {
        let truncated = &FIXTURE[..64.min(FIXTURE.len())];
        let result = decode_heic(truncated, &Limits::default());
        assert!(
            matches!(result, Err(ImageError::Decode(_))),
            "expected Decode, got {result:?}"
        );

        let junk = ftyp(b"heic", &[b"mif1"]);
        let result = decode_heic(&junk, &Limits::default());
        assert!(
            matches!(result, Err(ImageError::Decode(_))),
            "expected Decode, got {result:?}"
        );
    }

    #[cfg(feature = "heic")]
    #[test]
    fn check_caps_rejects_oversize() {
        let mut limits = Limits::default();
        limits.max_image_width = Some(10);
        assert!(matches!(
            check_caps(16, 16, &limits),
            Err(ImageError::LimitsExceeded(_))
        ));

        let mut height = Limits::default();
        height.max_image_height = Some(10);
        assert!(matches!(
            check_caps(4, 16, &height),
            Err(ImageError::LimitsExceeded(_))
        ));

        let mut alloc = Limits::default();
        alloc.max_alloc = Some(16);
        assert!(matches!(
            check_caps(16, 16, &alloc),
            Err(ImageError::LimitsExceeded(_))
        ));

        assert!(check_caps(16, 16, &Limits::default()).is_ok());
    }
}
