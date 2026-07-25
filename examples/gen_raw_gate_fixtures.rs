//! Regenerate the committed RAW fixtures the demo's pixel-gate smoke tests use
//! (SPEC-103).
//!
//! Produces two hand-built RAW-shaped blobs, natively from the `image` crate's
//! own JPEG encoder (no camera, no ImageMagick, per AGENTS §12) — the same
//! primitives `examples/gen_raw_fixture.rs` (SPEC-061) and `src/image/raw.rs`'s
//! own unit tests use:
//!
//! - `tests/fixtures/raw/oversize_preview.dng` — a TIFF header plus ONE embedded
//!   JPEG whose SOF0 header DECLARES 8000×7800 (62.4 Mpix — over the wasm demo's
//!   60 Mpix gate, `MAX_RAW_PREVIEW_MEGAPIXELS` in `src/wasm.rs` (retuned 40→60,
//!   SPEC-104), but under the native 64 Mpix DEC-063 decode budget) while
//!   carrying only a real 16×12
//!   image's worth of entropy data (JPEG has no header checksum, so the
//!   dimension fields patch in place after encoding — the same F-RAW-1 bomb
//!   shape `raw.rs` uses). The whole file stays under 1 KB: nothing this small
//!   could feed a genuine multi-hundred-MB decode, so a correct rejection can
//!   only have come from the pre-decode header peek.
//! - `tests/fixtures/raw/no_preview.cr2` — a TIFF header plus non-JPEG noise: no
//!   embedded preview decodes at all, the OTHER honest failure mode the demo
//!   must show a distinct message for.
//!
//! The `.dng`/`.cr2` extensions are what route the demo to `rawPreview` at all
//! (RAW is routed by file EXTENSION, not content — DEC-055).
//!
//! Regen (from the repo root):
//!
//! ```sh
//! cargo run --example gen_raw_gate_fixtures
//! ```

use std::io::Cursor;

use image::{DynamicImage, ImageFormat, Rgb, RgbImage};

/// Little-endian TIFF header: `II` + magic 42 + first-IFD offset (unused). Never
/// parsed — RAW routing is by file extension (DEC-055) — but a realistic header
/// keeps the synthetic blob honest.
fn tiff_header() -> Vec<u8> {
    vec![0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00]
}

/// Encode a solid-color `w`×`h` JPEG in memory.
fn jpeg(w: u32, h: u32) -> Vec<u8> {
    let img = RgbImage::from_pixel(w, h, Rgb([120, 90, 60]));
    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut out, ImageFormat::Jpeg)
        .expect("encode jpeg");
    out.into_inner()
}

/// A JPEG whose SOF0 header DECLARES `w`×`h` while carrying only a real 16×12
/// image's worth of entropy data.
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

fn main() {
    std::fs::create_dir_all("tests/fixtures/raw").expect("create fixture dir");

    let mut oversize = tiff_header();
    oversize.extend_from_slice(&jpeg_declaring(8000, 7800)); // 62.4 Mpix declared
    assert!(oversize.len() < 1024, "bomb fixture must stay tiny");
    let oversize_path = "tests/fixtures/raw/oversize_preview.dng";
    std::fs::write(oversize_path, oversize).expect("write oversize fixture");
    eprintln!("wrote {oversize_path}");

    let mut no_preview = tiff_header();
    no_preview.extend_from_slice(&[0x13, 0x37, 0x00, 0xFE, 0xED, 0xBE, 0xEF, 0x42]); // no JPEG SOI at all
    let no_preview_path = "tests/fixtures/raw/no_preview.cr2";
    std::fs::write(no_preview_path, no_preview).expect("write no-preview fixture");
    eprintln!("wrote {no_preview_path}");
}
