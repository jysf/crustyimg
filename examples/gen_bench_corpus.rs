//! Regenerate the **synthetic** part of the committed benchmark corpus (SPEC-088).
//!
//! Writes four synthetic, license-clean images (a smooth gradient and a flat-block
//! graphic, each small/large) into `bench/corpus/` (or an output dir passed as the
//! first argument). The content is generated deterministically from pure math — no
//! camera capture, no EXIF, no private data — so it can be committed and re-derived
//! by anyone (see `bench/corpus/README.md`).
//!
//! This does NOT write `photo_forest_cc0.jpg`: that is a real CC0 photograph (the
//! only row whose pixels reach the photographic/AVIF branch — synthetic math does
//! not), so it is committed as-is rather than generated. Existing files are
//! overwritten by name; nothing else in the directory is touched.
//!
//! Run with: `cargo run --example gen_bench_corpus` (from the repo root).

use std::path::PathBuf;

use image::{DynamicImage, ImageFormat, RgbImage};

/// A smooth colour gradient plus low-frequency sinusoidal structure, stored as a
/// JPEG (a lossy-family *source*, whatever the engine later decides).
///
/// **This is not a photograph, and the engine agrees:** the gradient is smooth
/// enough that `classify` measures a high flat-region ratio and routes it to
/// `graphic-logo` (lossless family) — the same verdict as the flat-block graphics
/// below. It was named `photo_*` before this fix; the tool contradicted the label,
/// so the name now describes what it is. A real photograph
/// (`photo_forest_cc0.jpg`) is what exercises the photographic/AVIF branch.
///
/// As a JPEG this is deliberately *already near-optimal*: `web`/`optimize`
/// correctly **pass it through** (never-bigger), which is a real path worth
/// exercising. Baking photographic detail into a committed synthetic file would
/// bloat the repo with incompressible noise — and still not classify as a photo.
fn gradient(w: u32, h: u32) -> RgbImage {
    let mut img = RgbImage::new(w, h);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let fx = x as f32 / w as f32;
        let fy = y as f32 / h as f32;
        let r = (fx * 235.0 + (fy * 7.0).sin() * 18.0) as i32;
        let g = (fy * 220.0 + (fx * 9.42).cos() * 22.0) as i32;
        let b =
            (((fx * 6.0).sin() * 0.5 + 0.5) * 190.0 + ((fy * 4.0).cos() * 0.5 + 0.5) * 55.0) as i32;
        *px = image::Rgb([
            r.clamp(0, 255) as u8,
            g.clamp(0, 255) as u8,
            b.clamp(0, 255) as u8,
        ]);
    }
    img
}

/// A graphic-like frame: a small palette of flat colour blocks (few unique
/// colours, hard edges) → the lossless/graphic family, where lossless WebP/PNG
/// wins and AVIF is (correctly) refused.
fn graphic(w: u32, h: u32) -> RgbImage {
    let palette = [
        [20u8, 30, 40],
        [220, 50, 47],
        [38, 139, 210],
        [133, 153, 0],
        [245, 245, 245],
    ];
    let mut img = RgbImage::new(w, h);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let bx = (x * 4 / w.max(1)) as usize;
        let by = (y * 3 / h.max(1)) as usize;
        *px = image::Rgb(palette[(bx + by) % palette.len()]);
    }
    img
}

fn main() {
    let out_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("bench/corpus"));
    std::fs::create_dir_all(&out_dir).expect("create corpus dir");

    // (name, image, format). Gradients are JPEG (a lossy source); graphics are PNG
    // (a lossless source). Small = 256², large = 512² (a 4× pixel step). Both kinds
    // classify `graphic-logo` — see `gradient`'s note and `bench/corpus/README.md`.
    let items: [(&str, RgbImage, ImageFormat); 4] = [
        ("gradient_small.jpg", gradient(256, 256), ImageFormat::Jpeg),
        ("gradient_large.jpg", gradient(512, 512), ImageFormat::Jpeg),
        ("graphic_small.png", graphic(256, 256), ImageFormat::Png),
        ("graphic_large.png", graphic(512, 512), ImageFormat::Png),
    ];

    for (name, img, fmt) in items {
        let path = out_dir.join(name);
        DynamicImage::ImageRgb8(img)
            .save_with_format(&path, fmt)
            .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
        println!("wrote {}", path.display());
    }
}
