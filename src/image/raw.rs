//! Tier-1 RAW embedded-preview extraction (SPEC-061, DEC-055).
//!
//! crustyimg does not *develop* camera RAW (demosaic + white-balance +
//! color-matrix — that is Tier-2, LGPL `rawler` / a multi-month effort, out of
//! scope per DEC-018 / `no-agpl-default-deps`). But nearly every RAW file embeds
//! a **full-res JPEG preview** (what the camera's screen shows), and extracting
//! *that* is permissive, pure-Rust, patent-free, and needs no RAW codec. This
//! module turns a `.nef`/`.cr2`/`.cr3`/`.arw`/`.dng`/… into the canonical
//! [`crate::image::Image`] by scanning for embedded JPEG streams and decoding the
//! largest one through the existing (capped) `image` JPEG decoder — the third
//! default input of PROJ-009, mirroring [`crate::image::avif`] and
//! [`crate::image::svg`].
//!
//! ## Why a byte scan (not IFD/box parsing)
//!
//! RAW containers are a zoo: TIFF-based (`.nef/.cr2/.arw/.dng/.rw2/.orf/…`),
//! ISOBMFF (`.cr3`), and bespoke (`.raf`). A design-time probe (2026-07-08,
//! against the pinned `image` =0.25.10) established that `image`'s JPEG decoder
//! **tolerates trailing bytes after a JPEG's EOI**, so we do NOT need to find
//! each preview's exact end or walk any vendor's SubIFDs. Scanning the whole file
//! for JPEG start-of-image markers (`FF D8 FF`), decoding *from* each, and
//! keeping the **largest that decodes** extracts the full-res preview (skipping
//! the small thumbnail) and covers TIFF-RAW **and** CR3 **and** RAF in one path
//! with no per-vendor parsing and no new dependency.
//!
//! ## Security (untrusted-input-hardening)
//!
//! RAW is hostile, untrusted binary input. Three bounds apply:
//!
//! - **Every candidate decode routes through the DEC-034 [`Limits`]** (the same
//!   `image` decoder + caps as the generic path), so a decompression-bomb preview
//!   is rejected before allocation — never an uncapped `load_from_memory`.
//! - **The number of full decode attempts is bounded** by
//!   [`MAX_PREVIEW_CANDIDATES`]: a file stuffed with fake `FF D8 FF` markers
//!   cannot cause unbounded decode work.
//! - **Candidates are pruned cheaply** before decoding — a plausible JPEG marker
//!   byte must follow `FF D8 FF` — so false SOI matches in compressed data are
//!   skipped without a decode. A false SOI that slips the prune is simply a
//!   failed decode that is skipped; it does not error the whole file.
//!
//! Every recoverable outcome is a typed [`ImageError`]: an oversize-only preview
//! is [`ImageError::LimitsExceeded`], no decodable preview is
//! [`ImageError::Decode`] — no `unwrap`/`expect`/`panic!`. Rust slicing keeps the
//! scan in bounds; no maker-supplied offset is ever trusted (we scan, we do not
//! seek by IFD offset). A `cargo-fuzz` target (`fuzz/fuzz_targets/raw_preview.rs`)
//! exercises the scan + decode path.

use std::io::Cursor;
use std::path::Path;

use ::image::{DynamicImage, ImageFormat, ImageReader, Limits};

use super::map_image_decode_error;
use crate::error::{ImageError, Result};

/// RAW file extensions routed to embedded-preview extraction (case-insensitive).
///
/// Covers the common prosumer/camera RAW families. `.x3f` (Sigma Foveon) is
/// deliberately omitted — it carries no standard baseline-JPEG preview, so it
/// would only yield the typed "no preview" error if named directly.
const RAW_EXTENSIONS: &[&str] = &[
    "nef", "nrw", // Nikon
    "cr2", "cr3", // Canon
    "arw", "srf", "sr2", // Sony
    "dng", // Adobe / Leica / Google Pixel / …
    "raf", // Fujifilm
    "rw2", // Panasonic
    "orf", // Olympus
    "pef", // Pentax
    "srw", // Samsung
    "rwl", // Leica
    "raw", // generic
];

/// Maximum number of candidate JPEG streams we will attempt to fully decode.
///
/// Real RAW files carry 1–3 genuine embedded previews; the plausible-marker
/// prune keeps the count low anyway. This is the hard backstop against a hostile
/// file stuffed with `FF D8 FF` sequences (`untrusted-input-hardening`).
const MAX_PREVIEW_CANDIDATES: usize = 16;

/// Whether `path`'s extension names a RAW format handled by [`extract_preview`].
///
/// Detection is by **extension**, not content: TIFF-based RAW starts with the
/// TIFF magic (`II*\0`/`MM\0*`), byte-indistinguishable from a plain `.tif`, so a
/// content sniff would risk mis-routing legitimate TIFFs. `Image::load` has the
/// `Path` and routes here before the generic byte decoder (DEC-055).
pub(crate) fn is_raw_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            RAW_EXTENSIONS
                .iter()
                .any(|&raw| e.eq_ignore_ascii_case(raw))
        })
        .unwrap_or(false)
}

/// Extract the largest embedded JPEG preview from a RAW byte stream as a
/// [`DynamicImage`], enforcing the decode caps in `limits` (DEC-034) on every
/// candidate.
///
/// Returns [`ImageError::LimitsExceeded`] when the only embedded preview(s)
/// exceed the caps, and [`ImageError::Decode`] when no embedded JPEG decodes at
/// all. Never panics on malformed input.
pub(crate) fn extract_preview(bytes: &[u8], limits: &Limits) -> Result<DynamicImage> {
    let (best, oversize, _attempts) = scan_for_preview(bytes, limits);
    match (best, oversize) {
        (Some(img), _) => Ok(img),
        // Carry the rejecting candidate's own reason: "exceeds decode caps" alone
        // tells the user nothing about WHICH cap (a 160 Mpix declared frame and a
        // 70 000-px-wide one are very different problems).
        (None, Some(reason)) => Err(ImageError::LimitsExceeded(format!(
            "raw: embedded preview exceeds decode caps: {reason}"
        ))),
        (None, None) => Err(ImageError::Decode(
            "raw: no decodable embedded JPEG preview".into(),
        )),
    }
}

/// Walk `bytes` for embedded JPEG SOIs, decode each pruned candidate under
/// `limits`, and keep the largest by pixel count. Bounded by
/// [`MAX_PREVIEW_CANDIDATES`] full decode attempts.
///
/// Returns `(best, oversize, attempts)`: the largest decoded preview, the reason
/// reported by a candidate rejected purely for exceeding the caps (so the caller
/// can say *which* cap), and how many full decodes were attempted (the last is a
/// test hook for the bounded-work guarantee).
fn scan_for_preview(
    bytes: &[u8],
    limits: &Limits,
) -> (Option<DynamicImage>, Option<String>, usize) {
    let mut best: Option<DynamicImage> = None;
    let mut best_px = 0usize;
    let mut oversize: Option<String> = None;
    let mut attempts = 0usize;

    let mut i = 0usize;
    // `i + 3 <= len` guarantees bytes[i..i+3] are in bounds; the 4th (marker)
    // byte is read via `.get` so the final SOI at the very end is still pruned.
    while i + 3 <= bytes.len() {
        if bytes[i] == 0xFF
            && bytes[i + 1] == 0xD8
            && bytes[i + 2] == 0xFF
            && is_plausible_jpeg_marker(bytes.get(i + 3).copied())
        {
            if attempts >= MAX_PREVIEW_CANDIDATES {
                break;
            }
            attempts += 1;
            match decode_jpeg_with_limits(&bytes[i..], limits) {
                Ok(img) => {
                    let px = (img.width() as usize) * (img.height() as usize);
                    if px > best_px {
                        best_px = px;
                        best = Some(img);
                    }
                }
                // A candidate rejected for exceeding the caps is remembered (with
                // its reason) so a RAW whose only preview is oversize surfaces
                // LimitsExceeded and names the cap it hit.
                Err(ImageError::LimitsExceeded(why)) => oversize = Some(why),
                // A false SOI / non-JPEG candidate is skipped, not fatal.
                Err(_) => {}
            }
            // Advance past this SOI; a genuine nested SOI is rare and the next
            // real preview is found by continuing the scan.
            i += 3;
        } else {
            i += 1;
        }
    }

    (best, oversize, attempts)
}

/// Peek the declared pixel count of the **largest** candidate embedded JPEG in
/// `bytes`, from each candidate's header alone — no pixel decode of any of them.
/// Returns `None` when the file carries no plausible JPEG candidate at all (the
/// same case in which [`extract_preview`] would return [`ImageError::Decode`],
/// never [`ImageError::LimitsExceeded`]).
///
/// This exists for the wasm demo's pre-decode pixel gate (SPEC-103, DEC-082): it
/// reuses the exact SOI scan and candidate-marker prune [`scan_for_preview`] uses,
/// bounded by the same [`MAX_PREVIEW_CANDIDATES`], but reads only each
/// candidate's JPEG header (a metadata parse) instead of decoding its pixels — so
/// the caller can reject an oversize preview BEFORE paying for the allocation
/// [`extract_preview`] would make, mirroring the DEC-063 SOF peek at a
/// caller-chosen ceiling instead of the native decode cap.
///
/// `cfg`'d to `wasm32` (its one real caller, `src/wasm.rs`) `+ test` (this
/// module's own unit tests below) rather than left unconditional: a native,
/// non-test build has no caller for it at all, and an unconditional definition
/// would be reported as dead code on every native `cargo build`/`clippy`.
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn largest_declared_preview_pixels(bytes: &[u8]) -> Option<u64> {
    let mut best: Option<u64> = None;
    let mut attempts = 0usize;

    let mut i = 0usize;
    while i + 3 <= bytes.len() {
        if bytes[i] == 0xFF
            && bytes[i + 1] == 0xD8
            && bytes[i + 2] == 0xFF
            && is_plausible_jpeg_marker(bytes.get(i + 3).copied())
        {
            if attempts >= MAX_PREVIEW_CANDIDATES {
                break;
            }
            attempts += 1;
            if let Some((w, h)) = peek_jpeg_dimensions(&bytes[i..]) {
                let px = (w as u64) * (h as u64);
                best = Some(best.map_or(px, |b| b.max(px)));
            }
            i += 3;
        } else {
            i += 1;
        }
    }

    best
}

/// Read a JPEG candidate's declared width/height from its header alone (no pixel
/// decode), forcing the JPEG format exactly as [`decode_jpeg_with_limits`] does
/// for its own internal peek. `None` for a candidate that doesn't parse as a
/// JPEG header at all (a false SOI match that slipped the marker prune) — the
/// caller treats that as "no data from this candidate", not an error.
#[cfg(any(target_arch = "wasm32", test))]
fn peek_jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let mut peek = ImageReader::new(Cursor::new(bytes));
    peek.set_format(ImageFormat::Jpeg);
    peek.into_dimensions().ok()
}

/// Whether `next` (the byte after `FF D8 FF`) is a plausible JPEG segment marker.
///
/// A genuine JPEG's third byte after SOI opens a marker segment: an application
/// segment (`APPn` = `E0..EF`), a quantization table (`DQT` = `DB`), a
/// start-of-frame (`SOFn`/`DHT`/`DAC` = `C0..CF`), or a comment (`COM` = `FE`).
/// Requiring one of these prunes most random `FF D8 FF` byte coincidences in
/// compressed data before paying for a decode. `None` (SOI at end of buffer)
/// cannot be a JPEG.
fn is_plausible_jpeg_marker(next: Option<u8>) -> bool {
    matches!(next, Some(0xE0..=0xEF | 0xDB | 0xC0..=0xCF | 0xFE))
}

/// Decode a JPEG byte stream under `limits`, forcing the JPEG format (the caller
/// found an SOI, so we do not re-sniff) and mapping a caps rejection to
/// [`ImageError::LimitsExceeded`] via the shared [`map_image_decode_error`].
///
/// Routes through the SAME `image` decoder + `Limits` as the generic path, so a
/// bomb preview is rejected before allocation (DEC-034) — never an uncapped
/// `load_from_memory`.
///
/// The candidate's SOF dimensions are peeked and checked against the peak-memory
/// budget (DEC-063) BEFORE the full decode: the DEC-034 caps alone let a preview
/// declaring 16384×9776 through (each side < 65535, the RGB output < 512 MiB
/// `max_alloc`) while the decode peaks at ~1.9 GB — this was SPEC-069's F-RAW-1,
/// and this peek is what closes it. `into_dimensions()` consumes its reader, so the
/// peek uses a throwaway one over the same (already in-memory) bytes; the peek is a
/// JPEG *header* parse, so an oversize candidate costs a few hundred bytes of work
/// instead of gigabytes.
fn decode_jpeg_with_limits(bytes: &[u8], limits: &Limits) -> Result<DynamicImage> {
    let mut peek = ImageReader::new(Cursor::new(bytes));
    peek.set_format(ImageFormat::Jpeg);
    peek.limits(limits.clone());
    let (w, h) = peek.into_dimensions().map_err(map_image_decode_error)?;
    super::check_pixel_budget(w, h)?;

    let mut reader = ImageReader::new(Cursor::new(bytes));
    reader.set_format(ImageFormat::Jpeg);
    reader.limits(limits.clone());
    reader.decode().map_err(map_image_decode_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::image::{DynamicImage, Rgb, RgbImage};

    /// Encode a solid-color `w`×`h` JPEG in memory (the fixture primitive — no
    /// camera, no ImageMagick, per AGENTS §12).
    fn jpeg(w: u32, h: u32) -> Vec<u8> {
        let img = RgbImage::from_pixel(w, h, Rgb([120, 90, 60]));
        let mut out = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut out, ImageFormat::Jpeg)
            .expect("encode jpeg fixture");
        out.into_inner()
    }

    /// A minimal little-endian TIFF header (`II*\0` + a first-IFD offset). The
    /// bytes are never parsed — routing is by extension — but a realistic header
    /// keeps the synthetic blob honest.
    fn tiff_header() -> Vec<u8> {
        vec![0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00]
    }

    /// Assemble `[TIFF hdr][thumb jpeg][junk][preview jpeg][junk]` — the shape of
    /// a real RAW: a small embedded thumbnail followed by the larger full-res
    /// preview, surrounded by container bytes.
    fn raw_blob(thumb: (u32, u32), preview: (u32, u32)) -> Vec<u8> {
        let mut b = tiff_header();
        b.extend_from_slice(&jpeg(thumb.0, thumb.1));
        b.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]); // junk
        b.extend_from_slice(&jpeg(preview.0, preview.1));
        b.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // trailing junk
        b
    }

    /// A JPEG whose SOF0 header **declares** `w`×`h` while carrying only the 16×12
    /// image's entropy data — the F-RAW-1 bomb shape (JPEG has no header checksum,
    /// so the dimensions patch in place).
    fn jpeg_declaring(w: u16, h: u16) -> Vec<u8> {
        let mut j = jpeg(16, 12);
        let sof = j
            .windows(2)
            .position(|m| m == [0xFF, 0xC0])
            .expect("encoded JPEG carries an SOF0 marker");
        // SOF0: FF C0 [len:2] [precision:1] [height:2] [width:2]
        j[sof + 5..sof + 7].copy_from_slice(&h.to_be_bytes());
        j[sof + 7..sof + 9].copy_from_slice(&w.to_be_bytes());
        j
    }

    fn generous() -> Limits {
        // Mirror decode_limits() shape without depending on its private consts.
        // NOTE: the DEC-063 pixel budget is deliberately NOT part of `Limits` (the
        // `image` crate has no such field) — it is the module-level
        // `super::MAX_IMAGE_PIXELS`, enforced by `check_pixel_budget` on every
        // path, so it applies here too and this mirror cannot drift from it.
        let mut l = Limits::default();
        l.max_image_width = Some(65_535);
        l.max_image_height = Some(65_535);
        l.max_alloc = Some(512 * 1024 * 1024);
        l
    }

    #[test]
    fn is_raw_extension_matches_known_exts() {
        for ext in [
            "nef", "cr2", "cr3", "arw", "dng", "raf", "rw2", "orf", "pef", "srw",
        ] {
            let lower = format!("photo.{ext}");
            let upper = format!("PHOTO.{}", ext.to_uppercase());
            assert!(is_raw_extension(Path::new(&lower)), "{lower} should be raw");
            assert!(is_raw_extension(Path::new(&upper)), "{upper} should be raw");
        }
        for ext in ["jpg", "png", "tif", "tiff", "svg", "avif", "webp"] {
            let name = format!("photo.{ext}");
            assert!(
                !is_raw_extension(Path::new(&name)),
                "{name} should NOT be raw"
            );
        }
        // No extension → not raw.
        assert!(!is_raw_extension(Path::new("photo")));
    }

    #[test]
    fn extract_preview_picks_largest_decodable_jpeg() {
        let blob = raw_blob((16, 12), (64, 48));
        let img = extract_preview(&blob, &generous()).expect("extract preview");
        assert_eq!(
            img.width(),
            64,
            "should pick the full preview, not the thumb"
        );
        assert_eq!(img.height(), 48);
    }

    #[test]
    fn extract_preview_skips_false_soi_matches() {
        // Same blob, but with extra `FF D8 FF xx` junk sequences that are not
        // valid JPEGs interleaved around the real previews.
        let mut blob = tiff_header();
        blob.extend_from_slice(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x01]); // false SOI (truncated)
        blob.extend_from_slice(&jpeg(16, 12));
        blob.extend_from_slice(&[0xFF, 0xD8, 0xFF, 0xDB, 0x99, 0x88]); // false SOI (garbage)
        blob.extend_from_slice(&jpeg(64, 48));
        blob.extend_from_slice(&[0xFF, 0xD8, 0xFF, 0xC0, 0x00]); // false SOI (truncated)

        let img = extract_preview(&blob, &generous()).expect("extract despite false SOIs");
        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 48);
    }

    #[test]
    fn extract_preview_bounds_decode_attempts() {
        // A blob with FAR more plausible fake SOIs than the cap, plus one real
        // preview at the front so the prune passes on the fakes.
        let mut blob = tiff_header();
        blob.extend_from_slice(&jpeg(32, 24)); // one real preview
        for _ in 0..(MAX_PREVIEW_CANDIDATES * 4) {
            // `FF D8 FF E0` passes the plausible-marker prune but is not a JPEG.
            blob.extend_from_slice(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x04, 0x00, 0x00]);
        }
        let (best, _oversize, attempts) = scan_for_preview(&blob, &generous());
        assert!(
            attempts <= MAX_PREVIEW_CANDIDATES,
            "attempts {attempts} must be bounded by {MAX_PREVIEW_CANDIDATES}"
        );
        // The real preview (decoded within the first attempts) still wins.
        let img = best.expect("valid preview found before the cap");
        assert_eq!(img.width(), 32);
        assert_eq!(img.height(), 24);
    }

    #[test]
    fn oversize_only_preview_is_limits_exceeded() {
        // A single embedded 64×48 JPEG against a tiny dimension cap → the only
        // candidate is rejected for exceeding the caps.
        let mut blob = tiff_header();
        blob.extend_from_slice(&jpeg(64, 48));
        let mut tiny = Limits::default();
        tiny.max_image_width = Some(8);
        tiny.max_image_height = Some(8);
        let result = extract_preview(&blob, &tiny);
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    /// SPEC-070 / F-RAW-1: a RAW whose embedded preview DECLARES 160 Mpix
    /// (16384×9776) is rejected at the SOF peek, before the ~1.9 GB decode. The
    /// whole blob is well under a kilobyte, so it cannot hold those pixels — the
    /// rejection can only be pre-decode. Under the production caps, not a tiny test
    /// seam: this passes every DEC-034 cap and is caught only by the DEC-063 budget.
    #[test]
    fn raw_preview_rejects_oversize_embedded_jpeg_before_decode() {
        let mut blob = tiff_header();
        blob.extend_from_slice(&jpeg_declaring(16384, 9776));
        assert!(blob.len() < 1024, "bomb fixture must stay tiny");

        let result = extract_preview(&blob, &generous());
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    /// The other side of the boundary: a RAW with a normal full-res preview
    /// (2000×1500 = 3 Mpix) still extracts under the same production caps — the
    /// budget rejects the bomb without rejecting real previews.
    #[test]
    fn raw_preview_within_budget_still_extracts() {
        let blob = raw_blob((160, 120), (2000, 1500));
        let img = extract_preview(&blob, &generous()).expect("normal preview must still extract");
        assert_eq!(img.width(), 2000);
        assert_eq!(img.height(), 1500);
    }

    #[test]
    fn no_embedded_jpeg_is_typed_error_not_panic() {
        // A TIFF header + noise with no decodable JPEG.
        let mut blob = tiff_header();
        blob.extend_from_slice(&[0x13, 0x37, 0x00, 0xFE, 0xED, 0xBE, 0xEF, 0x42]);
        let result = extract_preview(&blob, &generous());
        assert!(
            matches!(result, Err(ImageError::Decode(_))),
            "expected Decode, got {result:?}"
        );
    }

    /// [`largest_declared_preview_pixels`] reads the header only: a bomb declaring
    /// 160 Mpix while carrying 16×12 real entropy is reported at its DECLARED size
    /// (proving the peek never touches the entropy-coded scan data), and a normal
    /// real preview is reported at its true size.
    #[test]
    fn largest_declared_preview_pixels_reads_headers_not_entropy() {
        let mut bomb = tiff_header();
        bomb.extend_from_slice(&jpeg_declaring(16384, 9776));
        assert_eq!(
            largest_declared_preview_pixels(&bomb),
            Some(16384u64 * 9776),
            "the declared (not real) dimensions drive the peek"
        );

        let real = raw_blob((160, 120), (2000, 1500));
        assert_eq!(
            largest_declared_preview_pixels(&real),
            Some(2000u64 * 1500),
            "the LARGEST candidate wins, same as scan_for_preview's own selection"
        );
    }

    /// The boundary a demo-specific ceiling (SPEC-103's `MAX_RAW_PREVIEW_MEGAPIXELS`)
    /// would gate on: a declared size just at/over a 40 Mpix line is reported
    /// correctly distinct from one just under it, so the gate this function backs
    /// is not vacuous in either direction.
    #[test]
    fn largest_declared_preview_pixels_straddles_a_40mp_boundary() {
        const FORTY_MP: u64 = 40_000_000;

        let mut under = tiff_header();
        under.extend_from_slice(&jpeg_declaring(6300, 6349)); // 39,998,700 px
        let under_px = largest_declared_preview_pixels(&under).expect("candidate found");
        assert!(under_px < FORTY_MP, "{under_px} should be under 40 Mpix");

        let mut over = tiff_header();
        over.extend_from_slice(&jpeg_declaring(6300, 6351)); // 40,011,300 px
        let over_px = largest_declared_preview_pixels(&over).expect("candidate found");
        assert!(over_px > FORTY_MP, "{over_px} should be over 40 Mpix");
    }

    /// No plausible JPEG candidate at all → `None`, the same case
    /// [`extract_preview`] reports as [`ImageError::Decode`] rather than
    /// [`ImageError::LimitsExceeded`] — the peek and the real scan agree on which
    /// case this is.
    #[test]
    fn largest_declared_preview_pixels_none_when_no_candidate() {
        let mut blob = tiff_header();
        blob.extend_from_slice(&[0x13, 0x37, 0x00, 0xFE, 0xED, 0xBE, 0xEF, 0x42]);
        assert_eq!(largest_declared_preview_pixels(&blob), None);
    }

    #[test]
    fn is_plausible_jpeg_marker_prunes_junk() {
        assert!(is_plausible_jpeg_marker(Some(0xE0))); // APP0 (JFIF)
        assert!(is_plausible_jpeg_marker(Some(0xE1))); // APP1 (Exif)
        assert!(is_plausible_jpeg_marker(Some(0xDB))); // DQT
        assert!(is_plausible_jpeg_marker(Some(0xC0))); // SOF0
        assert!(is_plausible_jpeg_marker(Some(0xFE))); // COM
        assert!(!is_plausible_jpeg_marker(Some(0x00)));
        assert!(!is_plausible_jpeg_marker(Some(0xD8)));
        assert!(!is_plausible_jpeg_marker(None));
    }
}
