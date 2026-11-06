//! HEIC/HEIF decode behind the off-by-default `heic` feature (SPEC-062, DEC-056).
//!
//! HEIC is the one common modern format crustyimg cannot put on the default
//! path. **DEC-052** pinned two independent blockers: the mature pure-Rust HEIC
//! decoders were AGPL (`no-agpl-default-deps`, DEC-018), and HEVC is
//! patent-encumbered (Access Advance pool) on *every* decode path regardless of
//! code license.
//!
//! Decoding now goes through [`heifer`](https://crates.io/crates/heifer), a
//! permissive (MIT OR Apache-2.0), pure-Rust HEIC decoder with
//! `#![forbid(unsafe_code)]` and no C dependency. That resolves the first
//! blocker — there is no AGPL and no system libheif — but the second, the HEVC
//! patents, is unaffected, so `heic` must still never appear in a distributed
//! artifact and stays off by default.
//!
//! What that means for this module:
//!
//! - [`is_heic`] is compiled into **both** builds. Detection is what lets the
//!   default build answer a `.heic` with [`ImageError::CodecNotBuilt`] → exit 4
//!   ("rebuild with --features heic") instead of a vague "unsupported format".
//! - `decode_heic` exists only under `--features heic`, where it decodes via
//!   `heifer` into the canonical [`crate::image::Image`]. heifer is a codec
//!   feeding the pixel core, not a second pixel library (the AVIF precedent).
//!
//! HEIC and AVIF are ISOBMFF siblings, so this mirrors [`super::avif`]: brand
//! detection first, caps before allocation, typed errors throughout. AVIF is
//! dispatched *first* in `decode_with_limits` so an AVIF-in-HEIF container
//! (which also carries `mif1`) routes to the pure-Rust AVIF path.
//!
//! ## Security (untrusted-input-hardening)
//!
//! HEIC is hostile binary input, so it is treated as untrusted even though the
//! decoder is now memory-safe Rust:
//!
//! - Dimensions are capped from the container metadata (`ispe`) **before** any
//!   pixel plane is allocated (DEC-034), and re-checked on the decoded image as
//!   defense in depth. A primary image whose size the container does not give is
//!   refused before decode, never decoded uncapped.
//! - heifer itself bounds every decoded and assembled image with
//!   `Options::max_pixels` (a 2^28-pixel default) and is `#![forbid(unsafe_code)]`,
//!   so a malformed file cannot read out of bounds.
//! - Every heifer failure is a typed [`ImageError`] — no `unwrap`/`expect`/
//!   `panic!` on these paths. `fuzz/fuzz_targets/heic_decode.rs` exercises the
//!   container parse and the decode path together.
//!
//! ## Colour
//!
//! heifer converts to sRGB from the image's ICC profile or its `nclx` info, as
//! libheif's `ColorSpace::Rgb` target did. RGB input profiles are handled, both
//! matrix/TRC (`Display P3`, `Adobe RGB`, sRGB) and LUT-based (`A2B0`/`A2B1`:
//! `lut8`, `lut16`, `lutAtoB`). Two cases are refused with a typed
//! [`ImageError::Decode`] instead of converted:
//!
//! - an ICC profile whose input space is not RGB (CMYK, Lab, XYZ…);
//! - a base image coded with a PQ or HLG transfer (`nclx` transfer 16 or 18).
//!   The common HDR HEIC is a gain map over an SDR base, and is supported.

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
/// down-converted to 8-bit RGB(A) by heifer, mirroring the AVIF decision.
#[cfg(feature = "heic")]
pub(crate) fn decode_heic(bytes: &[u8], limits: &Limits) -> Result<DynamicImage> {
    use heifer::heifer_isobmff::HeifFile;

    let file = HeifFile::parse(bytes).map_err(map_heif_err)?;
    let id = file.primary_id;

    // Cap from the container metadata BEFORE decoding, so a decompression-bomb
    // header is rejected without heifer ever allocating a plane (DEC-034). A
    // primary image with no known size (no `ispe`, or a derived image whose size
    // cannot be read) is refused rather than decoded uncapped.
    let (w, h) = file
        .image_size(id)
        .map_err(map_heif_err)?
        .ok_or_else(|| ImageError::Decode("heic: primary image has no size (ispe)".into()))?;
    check_caps(w, h, limits)?;

    // heifer packs interleaved RGBA (stride == width * 4) and converts to sRGB
    // from the image's ICC profile or its `nclx` info, matching libheif's
    // `ColorSpace::Rgb` target. Ask for RGBA unconditionally, then drop the alpha
    // channel when the image is opaque.
    let image =
        heifer::decode_srgb_rgba8(bytes, &heifer::Options::default()).map_err(map_heif_err)?;

    // Defense in depth: the decoded image must also satisfy the caps.
    check_caps(image.width, image.height, limits)?;

    let (w, h) = (image.width, image.height);
    if image.has_alpha {
        let buf = RgbaImage::from_raw(w, h, image.data)
            .ok_or_else(|| ImageError::Decode("heic: rgba buffer size mismatch".into()))?;
        Ok(DynamicImage::ImageRgba8(buf))
    } else {
        let buf = RgbImage::from_raw(w, h, image.into_rgb8())
            .ok_or_else(|| ImageError::Decode("heic: rgb buffer size mismatch".into()))?;
        Ok(DynamicImage::ImageRgb8(buf))
    }
}

/// Reject dimensions that exceed the `limits` (dimension or total allocation) or
/// the shared peak-memory pixel budget (DEC-063).
///
/// The allocation estimate uses the 8-bit RGBA buffer (`w * h * 4`), the largest
/// buffer this module packs. The pixel budget is the tighter, uniform bound: the
/// `heic` feature is off by default, so it is not on the untrusted default path,
/// but it gets the same pre-decode peak bound as the pure-Rust paths.
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

/// Map a heifer container or bitstream error to a typed [`ImageError`].
/// Malformed, truncated, and unsupported-but-valid containers all surface as
/// `Decode` with the underlying message — never a panic.
#[cfg(feature = "heic")]
fn map_heif_err(e: impl std::fmt::Display) -> ImageError {
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

    // ── `--features heic` only (need the `heifer` decoder) ────────────────────

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

    /// A 16×16 HEIC whose ICC profile has a CMYK input space, written by
    /// `heifer = "=0.4.0"`: `heifer::encode_with_metadata` with
    /// `Metadata { icc_profile: Some(header) }`, where `header` is a bare 132-byte
    /// ICC v4.3 header (`prtr`, colour space `CMYK`, PCS `Lab `, `acsp`, no tags).
    #[cfg(feature = "heic")]
    const CMYK_ICC: &[u8] = include_bytes!("../../tests/fixtures/heic/cmyk_icc_16x16.heic");

    /// The fixture with its `nclx` transfer characteristics set to `transfer`.
    #[cfg(feature = "heic")]
    fn with_transfer(transfer: u16) -> Vec<u8> {
        let mut bytes = FIXTURE.to_vec();
        let at = bytes
            .windows(8)
            .position(|w| w == b"colrnclx")
            .expect("fixture has an nclx colr box");
        // colr(4) nclx(4) primaries(2) transfer(2) matrix(2) flags(1).
        bytes[at + 10..at + 12].copy_from_slice(&transfer.to_be_bytes());
        bytes
    }

    /// The patch above really reaches the decoder: an sRGB transfer (13) still
    /// decodes, so the refusals below come from the PQ/HLG value alone.
    #[cfg(feature = "heic")]
    #[test]
    fn patched_srgb_transfer_still_decodes() {
        let img = decode_heic(&with_transfer(13), &Limits::default()).expect("decode");
        assert_eq!((img.width(), img.height()), (64, 48));
    }

    /// A PQ (16) or HLG (18) base image is a typed decode error, not a wrong
    /// conversion or a panic.
    #[cfg(feature = "heic")]
    #[test]
    fn pq_and_hlg_base_images_are_refused() {
        for transfer in [16, 18] {
            let result = decode_heic(&with_transfer(transfer), &Limits::default());
            match result {
                Err(ImageError::Decode(msg)) => assert!(
                    msg.contains("unsupported"),
                    "transfer {transfer}: unexpected message {msg:?}"
                ),
                other => panic!("transfer {transfer}: expected Decode, got {other:?}"),
            }
        }
    }

    /// An ICC profile with a non-RGB (CMYK) input space is a typed decode error.
    #[cfg(feature = "heic")]
    #[test]
    fn non_rgb_icc_profile_is_refused() {
        assert!(is_heic(CMYK_ICC));
        let result = decode_heic(CMYK_ICC, &Limits::default());
        match result {
            Err(ImageError::Decode(msg)) => {
                assert!(
                    msg.contains("not an RGB profile"),
                    "unexpected message {msg:?}"
                )
            }
            other => panic!("expected Decode, got {other:?}"),
        }
    }

    /// A primary image without an `ispe` size is refused before decode (DEC-034):
    /// with no size there is nothing to cap, so it must not be decoded uncapped.
    #[cfg(feature = "heic")]
    #[test]
    fn missing_image_size_is_refused_before_decode() {
        let mut bytes = FIXTURE.to_vec();
        let at = bytes
            .windows(4)
            .position(|w| w == b"ispe")
            .expect("fixture has an ispe box");
        // Rename the property box: heifer skips unknown properties, so the image
        // keeps decodable data but loses its declared size.
        bytes[at..at + 4].copy_from_slice(b"xspe");
        let result = decode_heic(&bytes, &Limits::default());
        match result {
            Err(ImageError::Decode(msg)) => {
                assert!(msg.contains("no size"), "unexpected message {msg:?}")
            }
            other => panic!("expected Decode, got {other:?}"),
        }
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
