//! Shared, native-generated image fixtures for the integration tests.
//!
//! Fixtures are synthesized in memory with the `image` crate's pure-Rust
//! encoders — no ImageMagick, no committed binary fixtures (AGENTS.md §12).
//! `.unwrap()` here is idiomatic test setup (the `no-unwrap` constraint is
//! scoped to `src/**`).
//!
//! `#![allow(dead_code)]`: this module is included via `mod common;` by
//! multiple integration-test crates, and not every crate uses every fixture
//! (e.g. `tests/info_exif.rs` uses only `jpeg_with_exif`). Each crate's
//! dead-code analysis runs independently, so unused-in-that-crate helpers
//! would otherwise warn under `--all-targets`.
#![allow(dead_code)]

use std::io::Cursor;

use image::{DynamicImage, ImageFormat, RgbImage, RgbaImage};

/// Encode a solid-color `RgbImage` to PNG bytes.
pub fn solid_png(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
    let img = RgbImage::from_pixel(w, h, image::Rgb(rgb));
    encode(DynamicImage::ImageRgb8(img), ImageFormat::Png)
}

/// Encode a horizontal-gradient `RgbImage` to JPEG bytes.
pub fn gradient_jpeg(w: u32, h: u32) -> Vec<u8> {
    let mut img = RgbImage::new(w, h);
    for (x, _y, px) in img.enumerate_pixels_mut() {
        let v = if w > 1 {
            ((x * 255) / (w - 1)) as u8
        } else {
            0
        };
        *px = image::Rgb([v, v, v]);
    }
    encode(DynamicImage::ImageRgb8(img), ImageFormat::Jpeg)
}

/// Encode an `RgbaImage` (with an alpha channel) to PNG bytes.
pub fn rgba_png(w: u32, h: u32) -> Vec<u8> {
    let img = RgbaImage::from_pixel(w, h, image::Rgba([10, 20, 30, 128]));
    encode(DynamicImage::ImageRgba8(img), ImageFormat::Png)
}

/// Produce JPEG bytes carrying a minimal, valid EXIF APP1 segment.
///
/// Starts from a generated gradient JPEG and splices an APP1 segment
/// (`0xFF 0xE1`, 2-byte big-endian length, `Exif\0\0`, then a tiny
/// little-endian TIFF header with a zero-entry IFD) right after SOI
/// (`0xFF 0xD8`). The capture path only needs to *detect and record* the
/// `Exif\0\0` segment; the EXIF contents are not asserted.
pub fn jpeg_with_exif(w: u32, h: u32) -> Vec<u8> {
    let base = gradient_jpeg(w, h);
    // base[0..2] is SOI (FF D8).
    assert_eq!(
        &base[0..2],
        &[0xFF, 0xD8],
        "generated JPEG must start with SOI"
    );

    // EXIF payload: "Exif\0\0" + minimal little-endian TIFF (II*\0, IFD at
    // offset 8, zero entries, next-IFD offset 0).
    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(b"Exif\0\0");
    payload.extend_from_slice(b"II"); // little-endian
    payload.extend_from_slice(&[0x2A, 0x00]); // 42
    payload.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // IFD offset = 8
    payload.extend_from_slice(&[0x00, 0x00]); // 0 IFD entries
    payload.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // next IFD = 0

    // APP1 segment length includes the 2 length bytes themselves.
    let seg_len = (payload.len() + 2) as u16;

    let mut out: Vec<u8> = Vec::with_capacity(base.len() + payload.len() + 4);
    out.extend_from_slice(&base[0..2]); // SOI
    out.push(0xFF);
    out.push(0xE1); // APP1 marker
    out.extend_from_slice(&seg_len.to_be_bytes());
    out.extend_from_slice(&payload);
    out.extend_from_slice(&base[2..]); // rest of the JPEG
    out
}

/// Produce JPEG bytes carrying a one-entry EXIF IFD with the Orientation tag
/// set to `orientation` (1–8).
///
/// Mirrors `jpeg_with_exif` but the APP1 payload is `b"Exif\0\0"` followed by
/// a single-entry IFD for tag 0x0112 (Orientation). The exact little-endian
/// TIFF bytes (per the SPEC-015 Notes):
///
/// ```text
/// 49 49 2A 00            // "II", 42  (little-endian TIFF magic)
/// 08 00 00 00            // IFD offset = 8
/// 01 00                  // entry count = 1
/// 12 01                  // tag 0x0112 (Orientation)
/// 03 00                  // type 3 (SHORT)
/// 01 00 00 00            // count = 1
/// <orientation> 00       // value = orientation byte
/// 00 00                  // value padding
/// 00 00 00 00            // next-IFD offset = 0
/// ```
pub fn jpeg_with_orientation(w: u32, h: u32, orientation: u8) -> Vec<u8> {
    let base = gradient_jpeg(w, h);
    assert_eq!(
        &base[0..2],
        &[0xFF, 0xD8],
        "generated JPEG must start with SOI"
    );

    // Build the APP1 payload: "Exif\0\0" + one-entry TIFF IFD.
    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(b"Exif\0\0");
    // Little-endian TIFF header.
    payload.extend_from_slice(&[0x49, 0x49]); // "II"
    payload.extend_from_slice(&[0x2A, 0x00]); // TIFF magic = 42
    payload.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // IFD offset = 8
    payload.extend_from_slice(&[0x01, 0x00]); // entry count = 1
    payload.extend_from_slice(&[0x12, 0x01]); // tag 0x0112
    payload.extend_from_slice(&[0x03, 0x00]); // type SHORT
    payload.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // count = 1
    payload.push(orientation);
    payload.push(0x00); // value padding (low byte already written)
    payload.extend_from_slice(&[0x00, 0x00]); // remaining value padding
    payload.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // next-IFD offset = 0

    // APP1 segment length includes the 2 length bytes themselves.
    let seg_len = (payload.len() + 2) as u16;

    let mut out: Vec<u8> = Vec::with_capacity(base.len() + payload.len() + 4);
    out.extend_from_slice(&base[0..2]); // SOI
    out.push(0xFF);
    out.push(0xE1); // APP1 marker
    out.extend_from_slice(&seg_len.to_be_bytes());
    out.extend_from_slice(&payload);
    out.extend_from_slice(&base[2..]); // rest of the JPEG
    out
}

/// Build a DETERMINISTIC, STRUCTURED RGB image: a smooth gradient plus a mild
/// 8px checker texture (SPEC-016 / DEC-019 auto-quality fixture).
///
/// The structure is deliberate. A flat gradient or solid color JPEG-compresses
/// near-losslessly (score ~100 at every quality), so a perceptual search would
/// always pick the minimum quality. Pure high-frequency noise is the opposite
/// failure — JPEG can't reach a high score on it even at quality 100, so distinct
/// targets collapse to the same output. The gradient-dominated image with a mild
/// checker degrades cleanly at low quality yet reaches a high score at high
/// quality, giving the search real, monotone signal.
fn detailed_rgb(w: u32, h: u32) -> RgbImage {
    let mut img = RgbImage::new(w, h);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let gx = (x * 255 / w.max(1)) as i32;
        let gy = (y * 255 / h.max(1)) as i32;
        let tex = if ((x / 8) + (y / 8)) % 2 == 0 { 30 } else { 0 };
        let r = (gx + tex).clamp(0, 255) as u8;
        let g = (gy + tex).clamp(0, 255) as u8;
        let b = ((gx + gy) / 2).clamp(0, 255) as u8;
        *px = image::Rgb([r, g, b]);
    }
    img
}

/// Encode the structured `detailed_rgb` pattern to JPEG bytes (SPEC-016 fixture).
pub fn detailed_jpeg(w: u32, h: u32) -> Vec<u8> {
    encode(
        DynamicImage::ImageRgb8(detailed_rgb(w, h)),
        ImageFormat::Jpeg,
    )
}

/// Encode the structured `detailed_rgb` pattern to PNG bytes (SPEC-016 fixture).
pub fn detailed_png(w: u32, h: u32) -> Vec<u8> {
    encode(
        DynamicImage::ImageRgb8(detailed_rgb(w, h)),
        ImageFormat::Png,
    )
}

/// Encode a small solid-color `RgbImage` to LOSSLESS WebP bytes (SPEC-019
/// fixture). WebP is a default format; `write_to(_, WebP)` uses the pure-Rust
/// lossless encoder. Used to exercise the `.webp` decode (INPUT) path.
pub fn webp_lossless(w: u32, h: u32) -> Vec<u8> {
    let img = RgbImage::from_pixel(w, h, image::Rgb([20, 130, 200]));
    encode(DynamicImage::ImageRgb8(img), ImageFormat::WebP)
}

fn encode(img: DynamicImage, format: ImageFormat) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, format).unwrap();
    out.into_inner()
}
