//! Regenerate the committed synthetic RAW-preview fixture (SPEC-061).
//!
//! Produces `tests/fixtures/raw/synthetic_preview.nef`: a hand-built RAW-shaped
//! blob — a little-endian TIFF header, a small (16×12) embedded JPEG thumbnail,
//! filler container bytes, then a larger (64×48) embedded JPEG preview. It is
//! built natively from the `image` crate's OWN JPEG encoder (no camera, no
//! ImageMagick, per AGENTS §12), so the extraction test can assert the scan
//! keeps the LARGER preview over the thumbnail.
//!
//! The `.nef` extension is what routes `Image::load` to preview extraction; the
//! TIFF header bytes are never parsed (routing is by extension, DEC-055).
//!
//! Regen (from the repo root):
//!
//! ```sh
//! cargo run --example gen_raw_fixture
//! ```

use std::io::Cursor;

use image::{DynamicImage, ImageFormat, Rgb, RgbImage};

/// Encode a solid-color `w`×`h` JPEG in memory.
fn jpeg(w: u32, h: u32) -> Vec<u8> {
    let img = RgbImage::from_pixel(w, h, Rgb([120, 90, 60]));
    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut out, ImageFormat::Jpeg)
        .expect("encode jpeg");
    out.into_inner()
}

fn main() {
    // Little-endian TIFF header: `II` + magic 42 + first-IFD offset (unused).
    let mut blob: Vec<u8> = vec![0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00];
    blob.extend_from_slice(&jpeg(16, 12)); // embedded thumbnail
    blob.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]); // filler container bytes
    blob.extend_from_slice(&jpeg(64, 48)); // embedded full-res preview
    blob.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // trailing junk

    let path = "tests/fixtures/raw/synthetic_preview.nef";
    std::fs::create_dir_all("tests/fixtures/raw").expect("create fixture dir");
    std::fs::write(path, blob).expect("write fixture");
    eprintln!("wrote {path}");
}
