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
    wrap_with_orientation_app1(&gradient_jpeg(w, h), orientation)
}

/// A bimodal, near-gray, low-entropy "scan": a light page with darker
/// text-like bars. Shaped to satisfy the Document rule's conjunction
/// (`bimodality >= DOC_BIMODALITY`, `gray_ratio >= DOC_GRAY_RATIO`,
/// `entropy < DOC_ENTROPY_MAX`) rather than to look like anything.
///
/// Exists for SPEC-108's EXIF finding: the flip it records only reproduces on
/// document-class content, and every other JPEG generator here is a gradient
/// or a detailed pattern whose entropy already clears `PHOTO_ENTROPY_STRONG`.
pub fn scan_jpeg(w: u32, h: u32) -> Vec<u8> {
    let mut img = RgbImage::new(w, h);
    // Two luma levels only — a light page (paper) and dark bars (glyph rows) —
    // so the histogram is genuinely bimodal and the entropy stays low.
    const PAPER: [u8; 3] = [235, 235, 233];
    const INK: [u8; 3] = [58, 58, 60];
    let band = (h / 24).max(1);
    for (x, y, px) in img.enumerate_pixels_mut() {
        // Text-like bars: every third band, broken up along x so the rows are
        // not solid rectangles (a solid fill reads as flat-graphic, not scan).
        let in_line = (y / band) % 3 == 1;
        let in_glyph = (x / band.max(1)) % 4 != 3;
        *px = image::Rgb(if in_line && in_glyph { INK } else { PAPER });
    }
    encode(DynamicImage::ImageRgb8(img), ImageFormat::Jpeg)
}

/// Splice a one-entry Orientation IFD onto arbitrary JPEG bytes.
///
/// Factored out of [`jpeg_with_orientation`] so a caller can attach a *real*
/// orientation tag to content other than a gradient — which SPEC-108's EXIF
/// fixture needs, and which the zero-entry [`jpeg_with_exif`] cannot express.
pub fn wrap_with_orientation_app1(base: &[u8], orientation: u8) -> Vec<u8> {
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
/// EXIF**. Since SPEC-105 the detailed content classifies as a **Photograph** →
/// `Lossy` bucket (its luma entropy is 7.77, well over the strong-entropy floor); the
/// ICC is what forbids a raw passthrough, so `optimize` must ship a compact lossy
/// re-encode and strip the ICC, never a lossless blow-up.
///
/// It no longer reaches the *metadata-forced fallback* branch it was written for —
/// that needs a source in a lossless-only bucket. Use [`jpeg_with_icc`] over a real
/// graphic for that (see `spec_084_metadata_forced_fallback_is_reached`).
pub fn detailed_jpeg_with_icc(w: u32, h: u32) -> Vec<u8> {
    jpeg_with_icc(&detailed_jpeg(w, h))
}

/// Splice an APP2 `ICC_PROFILE` segment into arbitrary JPEG bytes, right after SOI.
///
/// The profile body is filler — the decode path only needs to surface
/// `has_icc = true`, and the container lane never interprets the bytes. Metadata that
/// must be stripped is what forces a re-encode, so this is the knob that turns a
/// passthrough-eligible source into one that has to go through the decision engine.
pub fn jpeg_with_icc(base: &[u8]) -> Vec<u8> {
    assert_eq!(&base[0..2], &[0xFF, 0xD8], "JPEG must start with SOI");

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

/// The `detailed_rgb` pattern encoded to JPEG at an explicit quality (SPEC-113
/// fixture): `optimize`'s pinned path re-encodes JPEG at the encoder DEFAULT
/// quality (no `-q`, no auto-quality search on the `Fast` decision —
/// `resolve_effective_quality`'s `AutoQuality::Fast` arm returns `quality: None`).
/// A source pinned at a HIGHER quality than that default re-encodes smaller,
/// which is what SPEC-113's AC-6 needs: proof the never-bigger guard does not
/// fire when a same-format re-encode genuinely wins.
pub fn detailed_jpeg_at_quality(w: u32, h: u32, quality: u8) -> Vec<u8> {
    use image::codecs::jpeg::JpegEncoder;
    let mut out = Cursor::new(Vec::new());
    let encoder = JpegEncoder::new_with_quality(&mut out, quality);
    DynamicImage::ImageRgb8(detailed_rgb(w, h))
        .write_with_encoder(encoder)
        .unwrap();
    out.into_inner()
}

/// A flat six-colour banded graphic as PNG bytes.
///
/// Deliberately a `LosslessFlat`-bucket source: every verb shortlists the same
/// codec-independent lossless candidates, so the report shape carries no dependence
/// on which codecs the build has. That is what lets
/// `json_shape_consistent_across_verbs` run on every feature leg.
pub fn flat_graphic_png(w: u32, h: u32) -> Vec<u8> {
    const BANDS: [[u8; 3]; 6] = [
        [200, 30, 30],
        [30, 200, 30],
        [30, 30, 200],
        [200, 200, 30],
        [200, 30, 200],
        [30, 200, 200],
    ];
    let mut img = RgbImage::new(w, h);
    for (_x, y, px) in img.enumerate_pixels_mut() {
        *px = image::Rgb(BANDS[(y * 6 / h.max(1)).min(5) as usize]);
    }
    encode(DynamicImage::ImageRgb8(img), ImageFormat::Png)
}

/// Encode the structured `detailed_rgb` pattern to PNG bytes (SPEC-016 fixture).
pub fn detailed_png(w: u32, h: u32) -> Vec<u8> {
    encode(
        DynamicImage::ImageRgb8(detailed_rgb(w, h)),
        ImageFormat::Png,
    )
}

/// The `detailed_rgb` pattern with a fully-opaque alpha channel, as PNG bytes
/// (SPEC-108, AC-7 fixture). Same high-entropy content as `detailed_png` — it
/// classifies `photograph` — but `RgbaImage`'s colour type reports `has_alpha:
/// true` (a structural property of the container, not per-pixel transparency),
/// which is what routes it through the `OptBucket::Lossy` + alpha shortlist arm.
pub fn detailed_rgba_png(w: u32, h: u32) -> Vec<u8> {
    let rgba = DynamicImage::ImageRgb8(detailed_rgb(w, h)).to_rgba8();
    encode(DynamicImage::ImageRgba8(rgba), ImageFormat::Png)
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

/// A single-frame (static) GIF — the SPEC-119/SPEC-053 did-not-break-it
/// control: `format/animated-gif` and the animated-input warning must both
/// stay silent on this.
pub fn static_gif(w: u32, h: u32) -> Vec<u8> {
    let img = RgbImage::from_pixel(w, h, image::Rgb([9, 9, 9]));
    encode(DynamicImage::ImageRgb8(img), ImageFormat::Gif)
}

/// The standard PNG/zlib CRC-32 (bit-by-bit, no lookup table — these fixtures
/// are tiny so the table's setup cost isn't worth it). Shared by
/// [`png_header_declaring`] and the SPEC-119 APNG fixture below.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in bytes {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// One PNG chunk: `[u32 be length][fourcc][payload][u32 be crc32(fourcc ++ payload)]`.
fn png_chunk(fourcc: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut out = (payload.len() as u32).to_be_bytes().to_vec();
    out.extend_from_slice(fourcc);
    out.extend_from_slice(payload);
    let mut crc_input = fourcc.to_vec();
    crc_input.extend_from_slice(payload);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
    out
}

/// A PNG whose IHDR *declares* `w`×`h` and carries no real pixel data — the
/// classic decompression-bomb shape: under 100 bytes claiming billions of
/// pixels. Every chunk's CRC is computed for real, and an empty `IDAT` +
/// `IEND` follow the header: `image` 0.25's PNG decoder needs to see those
/// chunk boundaries to finish reading the header info (a bare `IHDR` alone
/// makes `into_dimensions()` fail with a generic "unexpected end of file"
/// *before* the declared size is ever compared against the decode budget —
/// confirmed empirically while building this fixture, SPEC-107). Mirrors the
/// shape of `wasm_roundtrip.rs`'s private `png_header_declaring` (that copy's
/// caller only asserts `Err(_)` generically, so it never needed the trailing
/// chunks); duplicated rather than shared across the wasm/native test
/// targets, which do not otherwise depend on each other.
pub fn png_header_declaring(w: u32, h: u32) -> Vec<u8> {
    let mut ihdr_payload = w.to_be_bytes().to_vec();
    ihdr_payload.extend_from_slice(&h.to_be_bytes());
    ihdr_payload.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit, truecolour, no interlace

    let mut png = Vec::from(*b"\x89PNG\r\n\x1a\n");
    png.extend_from_slice(&png_chunk(b"IHDR", &ihdr_payload));
    png.extend_from_slice(&png_chunk(b"IDAT", &[])); // empty — never actually decoded
    png.extend_from_slice(&png_chunk(b"IEND", &[]));
    png
}

/// Concatenate the data of every chunk in `bytes` (a PNG byte stream) whose
/// type is `fourcc` — correct PNG behavior for a multi-`IDAT`/`fdAT` stream,
/// and the general case a single matching chunk is a special case of.
fn find_png_chunks(bytes: &[u8], fourcc: &[u8; 4]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 8; // past the 8-byte PNG signature
    while i + 8 <= bytes.len() {
        let len = u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
        if &bytes[i + 4..i + 8] == fourcc {
            out.extend_from_slice(&bytes[i + 8..i + 8 + len]);
        }
        i += 12 + len; // length(4) + type(4) + data(len) + crc(4)
    }
    out
}

/// Encode a 2-frame Animated PNG (APNG) fixture (SPEC-119): hand-assembles
/// the `acTL`/`fcTL`/`fdAT` chunks the APNG extension adds on top of a plain
/// PNG — `image` 0.25 only DECODES APNG (`PngDecoder::apng`/`ApngDecoder`),
/// it has no APNG-encode API. Each frame's compressed pixel data is
/// extracted byte-for-byte from `image`'s own single-image PNG encoder
/// output via [`find_png_chunks`], so the only hand-rolled bytes are the
/// chunk framing (length/type/CRC) and the `acTL`/`fcTL` control chunks the
/// APNG spec defines — never any DEFLATE/pixel compression.
pub fn animated_apng(w: u32, h: u32) -> Vec<u8> {
    let frame1_idat = find_png_chunks(&solid_rgba_png(w, h, [255, 0, 0, 255]), b"IDAT");
    let frame2_idat = find_png_chunks(&solid_rgba_png(w, h, [0, 255, 0, 255]), b"IDAT");

    let mut ihdr = w.to_be_bytes().to_vec();
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8-bit depth, RGBA (colour type 6)

    let mut actl = 2u32.to_be_bytes().to_vec(); // num_frames
    actl.extend_from_slice(&0u32.to_be_bytes()); // num_plays: loop forever

    let fctl = |seq: u32| -> Vec<u8> {
        let mut p = seq.to_be_bytes().to_vec();
        p.extend_from_slice(&w.to_be_bytes());
        p.extend_from_slice(&h.to_be_bytes());
        p.extend_from_slice(&0u32.to_be_bytes()); // x_offset
        p.extend_from_slice(&0u32.to_be_bytes()); // y_offset
        p.extend_from_slice(&1u16.to_be_bytes()); // delay_num
        p.extend_from_slice(&10u16.to_be_bytes()); // delay_den → 100ms/frame
        p.push(0); // dispose_op: NONE
        p.push(0); // blend_op: SOURCE
        p
    };

    let mut fdat2 = 2u32.to_be_bytes().to_vec(); // sequence_number (0=fcTL,1=fcTL,2=fdAT)
    fdat2.extend_from_slice(&frame2_idat);

    let mut out = Vec::from(*b"\x89PNG\r\n\x1a\n");
    out.extend_from_slice(&png_chunk(b"IHDR", &ihdr));
    out.extend_from_slice(&png_chunk(b"acTL", &actl));
    out.extend_from_slice(&png_chunk(b"fcTL", &fctl(0)));
    out.extend_from_slice(&png_chunk(b"IDAT", &frame1_idat));
    out.extend_from_slice(&png_chunk(b"fcTL", &fctl(1)));
    out.extend_from_slice(&png_chunk(b"fdAT", &fdat2));
    out.extend_from_slice(&png_chunk(b"IEND", &[]));
    out
}

fn solid_rgba_png(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    let img = RgbaImage::from_pixel(w, h, image::Rgba(rgba));
    encode(DynamicImage::ImageRgba8(img), ImageFormat::Png)
}

/// One RIFF chunk: `[fourcc][u32 le length][payload][pad byte if length is odd]`.
fn riff_chunk(fourcc: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut out = fourcc.to_vec();
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    if payload.len() % 2 == 1 {
        out.push(0);
    }
    out
}

/// Encode a 2-frame animated WebP fixture (SPEC-119): hand-assembles the
/// minimal `VP8X`/`ANIM`/`ANMF` container `image-webp` 0.2.4's decoder
/// requires (verified against its source: `decoder.rs`'s `read_data` needs
/// both an `ANIM` and at least one `ANMF` chunk when `VP8X`'s animation flag
/// is set, or it errors `ChunkMissing`) — `image`'s own `WebPEncoder` has no
/// animation-encode API (single-image `VP8L` only). Each frame's `VP8L`
/// bitstream chunk is extracted byte-for-byte from `image`'s own lossless
/// single-image WebP encoder output (which writes a bare
/// `RIFF`/`WEBP`/`VP8L` container when no metadata is set — `image-webp`
/// 0.2.4 `encoder.rs`'s "simple" branch), so the only hand-rolled bytes are
/// the RIFF chunk framing and the `VP8X`/`ANIM`/`ANMF` control chunks, never
/// any bitstream compression.
pub fn animated_webp(w: u32, h: u32) -> Vec<u8> {
    let vp8l_chunk = |rgba: [u8; 4]| -> Vec<u8> {
        let single = encode(
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, image::Rgba(rgba))),
            ImageFormat::WebP,
        );
        // Past "RIFF"(4) + size(4) + "WEBP"(4): the `VP8L` chunk itself.
        single[12..].to_vec()
    };

    let anmf_payload = |frame: &[u8]| -> Vec<u8> {
        let mut p = vec![0, 0, 0]; // frame X (2px units)
        p.extend_from_slice(&[0, 0, 0]); // frame Y
        p.extend_from_slice(&(w - 1).to_le_bytes()[..3]);
        p.extend_from_slice(&(h - 1).to_le_bytes()[..3]);
        p.extend_from_slice(&[10, 0, 0]); // duration: 10ms (24-bit LE)
        p.push(0); // flags: reserved/blend/dispose all 0
        p.extend_from_slice(frame);
        p
    };

    let anmf1 = riff_chunk(b"ANMF", &anmf_payload(&vp8l_chunk([255, 0, 0, 255])));
    let anmf2 = riff_chunk(b"ANMF", &anmf_payload(&vp8l_chunk([0, 255, 0, 255])));

    let mut anim_payload = vec![0, 0, 0, 0]; // background color
    anim_payload.extend_from_slice(&0u16.to_le_bytes()); // loop count: forever
    let anim = riff_chunk(b"ANIM", &anim_payload);

    let mut vp8x_payload = vec![0b0001_0010]; // flags: alpha(0x10) | animation(0x02)
    vp8x_payload.extend_from_slice(&[0, 0, 0]); // reserved
    vp8x_payload.extend_from_slice(&(w - 1).to_le_bytes()[..3]);
    vp8x_payload.extend_from_slice(&(h - 1).to_le_bytes()[..3]);
    let vp8x = riff_chunk(b"VP8X", &vp8x_payload);

    let mut body = vp8x;
    body.extend_from_slice(&anim);
    body.extend_from_slice(&anmf1);
    body.extend_from_slice(&anmf2);

    let mut out = Vec::from(*b"RIFF");
    out.extend_from_slice(&((4 + body.len()) as u32).to_le_bytes()); // "WEBP" + body
    out.extend_from_slice(b"WEBP");
    out.extend_from_slice(&body);
    out
}

fn encode(img: DynamicImage, format: ImageFormat) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, format).unwrap();
    out.into_inner()
}
