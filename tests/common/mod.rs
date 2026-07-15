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

/// Produce JPEG bytes carrying an EXIF APP1 segment with a GPS sub-IFD.
///
/// Mirrors `jpeg_with_orientation`, but the TIFF holds a single IFD0 entry — a
/// GPSInfo pointer (tag 0x8825) to a GPS IFD with one `GPSLatitudeRef` ("N")
/// entry. Enough for the `kamadak-exif` read side to surface a `Context::Gps`
/// field, which the `privacy/gps-metadata-leak` rule keys off. The exact
/// little-endian TIFF bytes:
///
/// ```text
/// 49 49 2A 00                 // "II", 42
/// 08 00 00 00                 // IFD0 offset = 8
/// 01 00                       // IFD0 entry count = 1
/// 25 88 04 00 01 00 00 00     // tag 0x8825 GPSInfo, type LONG, count 1,
/// 1A 00 00 00                 //   value = GPS-IFD offset (26)
/// 00 00 00 00                 // next-IFD offset = 0
/// 01 00                       // GPS IFD entry count = 1
/// 01 00 02 00 02 00 00 00     // tag 0x0001 GPSLatitudeRef, ASCII, count 2,
/// 4E 00 00 00                 //   value "N\0" inline
/// 00 00 00 00                 // next-IFD offset = 0
/// ```
pub fn jpeg_with_gps(w: u32, h: u32) -> Vec<u8> {
    wrap_with_gps_app1(&gradient_jpeg(w, h))
}

/// A structured [`detailed_jpeg`] carrying an APP2 `ICC_PROFILE` segment but **no
/// EXIF**. The detailed content (flat_ratio ≈ 0.69, near-zero edges) classifies as a
/// GraphicLogo → LosslessFlat bucket — but *only* because there is no EXIF (an EXIF
/// camera prior would force Photograph). This is the metadata-forced fallback trigger
/// (SPEC-084): a lossy JPEG source, in a bucket that offers ONLY lossless candidates,
/// with metadata (the ICC) that forbids a raw passthrough. `optimize` must ship a
/// compact lossy re-encode (≈ source), never a lossless blow-up several times the
/// source size, and must strip the ICC.
pub fn detailed_jpeg_with_icc(w: u32, h: u32) -> Vec<u8> {
    let base = detailed_jpeg(w, h);
    assert_eq!(
        &base[0..2],
        &[0xFF, 0xD8],
        "generated JPEG must start with SOI"
    );

    // APP2 ICC_PROFILE segment: the marker, a 1/1 chunk header, then filler profile
    // bytes. Enough for the decoder to surface `has_icc = true`.
    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(b"ICC_PROFILE\0");
    payload.push(1); // chunk sequence number
    payload.push(1); // chunk count
    payload.extend(std::iter::repeat_n(0xABu8, 128)); // profile bytes (filler)
    let seg_len = (payload.len() + 2) as u16;

    let mut out: Vec<u8> = Vec::with_capacity(base.len() + payload.len() + 4);
    out.extend_from_slice(&base[0..2]); // SOI
    out.push(0xFF);
    out.push(0xE2); // APP2 marker
    out.extend_from_slice(&seg_len.to_be_bytes());
    out.extend_from_slice(&payload);
    out.extend_from_slice(&base[2..]); // rest of the JPEG
    out
}

/// Wrap arbitrary base JPEG bytes with an EXIF APP1 segment carrying a GPS sub-IFD
/// (see [`jpeg_with_gps`] for the exact TIFF layout).
fn wrap_with_gps_app1(base: &[u8]) -> Vec<u8> {
    assert_eq!(
        &base[0..2],
        &[0xFF, 0xD8],
        "generated JPEG must start with SOI"
    );

    let mut tiff: Vec<u8> = Vec::new();
    tiff.extend_from_slice(&[0x49, 0x49, 0x2A, 0x00]); // "II", 42
    tiff.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // IFD0 offset = 8
    tiff.extend_from_slice(&[0x01, 0x00]); // IFD0 entry count = 1
    tiff.extend_from_slice(&[0x25, 0x88]); // tag 0x8825 (GPSInfo pointer)
    tiff.extend_from_slice(&[0x04, 0x00]); // type LONG
    tiff.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // count = 1
    tiff.extend_from_slice(&[0x1A, 0x00, 0x00, 0x00]); // value = offset 26
    tiff.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // next-IFD offset = 0
    tiff.extend_from_slice(&[0x01, 0x00]); // GPS IFD entry count = 1
    tiff.extend_from_slice(&[0x01, 0x00]); // tag 0x0001 (GPSLatitudeRef)
    tiff.extend_from_slice(&[0x02, 0x00]); // type ASCII
    tiff.extend_from_slice(&[0x02, 0x00, 0x00, 0x00]); // count = 2
    tiff.extend_from_slice(&[0x4E, 0x00, 0x00, 0x00]); // "N\0" inline
    tiff.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // next-IFD offset = 0

    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(b"Exif\0\0");
    payload.extend_from_slice(&tiff);

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

/// Encode a solid 16-bit RGB PNG (needless high bit depth for the web —
/// SPEC-053 `color/wrong-colorspace` fixture).
pub fn png_16bit(w: u32, h: u32) -> Vec<u8> {
    use image::ImageBuffer;
    let img: ImageBuffer<image::Rgb<u16>, Vec<u16>> =
        ImageBuffer::from_pixel(w, h, image::Rgb([40000u16, 20000, 10000]));
    encode(DynamicImage::ImageRgb16(img), ImageFormat::Png)
}

/// Encode a 2-frame animated GIF (SPEC-053 `format/animated-gif` fixture).
pub fn animated_gif(w: u32, h: u32) -> Vec<u8> {
    use image::codecs::gif::GifEncoder;
    use image::{Frame, RgbaImage};
    let mut buf = Vec::new();
    {
        let mut enc = GifEncoder::new(&mut buf);
        let f1 = Frame::new(RgbaImage::from_pixel(w, h, image::Rgba([255, 0, 0, 255])));
        let f2 = Frame::new(RgbaImage::from_pixel(w, h, image::Rgba([0, 255, 0, 255])));
        enc.encode_frames(vec![f1, f2]).unwrap();
    }
    buf
}

fn encode(img: DynamicImage, format: ImageFormat) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, format).unwrap();
    out.into_inner()
}
