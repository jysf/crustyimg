//! A watermark that would not fit inside an image, or would cover too much of
//! it, is refused instead of drawn; a mark that passes draws exactly as before.
//!
//! The library half builds `Watermark` ops in memory. Bases are generated
//! natively as gradients (not solid colours), so "the output equals the input"
//! and the golden hashes both depend on every pixel. `.unwrap()` is idiomatic
//! test setup here (the `no-unwrap` constraint is scoped to `src/**`).

use std::path::{Path, PathBuf};

use crustyimg::image::Image;
use crustyimg::operation::{Gravity, Operation, Watermark};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba, RgbaImage};
use sha2::{Digest, Sha256};

// ─── Fixtures ────────────────────────────────────────────────────────────────

/// An opaque RGBA gradient base, `w×h`.
fn gradient(w: u32, h: u32) -> RgbaImage {
    RgbaImage::from_fn(w, h, |x, y| {
        Rgba([
            (x * 255 / w.max(1)) as u8,
            (y * 255 / h.max(1)) as u8,
            ((x + y) * 7 % 256) as u8,
            255,
        ])
    })
}

fn image_of(pixels: DynamicImage) -> Image {
    Image::from_parts(pixels, ImageFormat::Png, None)
}

/// An overlay whose every pixel is visible, with varied colour and alpha so a
/// change in compositing would show in the hash.
fn opaque_logo(w: u32, h: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(RgbaImage::from_fn(w, h, |x, y| {
        Rgba([
            (200 - (x * 3 % 200)) as u8,
            (y * 5 % 256) as u8,
            ((x * y) % 256) as u8,
            255,
        ])
    }))
}

fn image_mark(overlay: DynamicImage, opacity: f32, scale: Option<f32>, tile: bool) -> Watermark {
    Watermark::new_image(
        overlay,
        "logo.png".to_owned(),
        Gravity::SouthEast,
        opacity,
        scale,
        0,
        tile,
    )
}

/// The mark `watermark --text "© crustyimg"` draws with every default.
fn default_text_mark() -> Watermark {
    let color = [255, 255, 255, 255];
    let rendered =
        crustyimg::text::render_text(crustyimg::text::DEFAULT_FONT, "© crustyimg", 32.0, color)
            .unwrap();
    Watermark::new_text(
        DynamicImage::ImageRgba8(rendered),
        "© crustyimg".to_owned(),
        None,
        32.0,
        color,
        Gravity::SouthEast,
        1.0,
        None,
        0,
        false,
    )
}

// ─── Byte identity of a mark that passes ─────────────────────────────────────
//
// Six marks the damage rule lets through. Their decoded output is hashed and
// compared with goldens captured from code that predates the rule, so a
// change to placement, scaling, opacity or narrowing fails here. Regenerate
// only with `CRUSTYIMG_BLESS=1`, and only on the code the goldens describe.

fn golden_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/watermark/passing_marks.golden")
}

/// `(case name, op, base)` for every passing case.
fn passing_cases() -> Vec<(&'static str, Watermark, Image)> {
    // A 16-bit base whose low bytes vary, with a part-transparent mark.
    let base16: ImageBuffer<Rgba<u16>, Vec<u16>> = ImageBuffer::from_fn(100, 100, |x, y| {
        Rgba([
            (x * 655 + 17) as u16,
            (y * 641 + 3) as u16,
            ((x * y) % 65536) as u16,
            65535,
        ])
    });
    let soft_logo = DynamicImage::ImageRgba8(RgbaImage::from_fn(10, 10, |x, y| {
        Rgba([250, (x * 20) as u8, (y * 20) as u8, (x * 25 + y) as u8])
    }));
    vec![
        (
            "text_default_on_256",
            default_text_mark(),
            image_of(DynamicImage::ImageRgba8(gradient(256, 256))),
        ),
        (
            "text_default_on_800x600",
            default_text_mark(),
            image_of(DynamicImage::ImageRgba8(gradient(800, 600))),
        ),
        (
            "opaque_64_on_800x600_half_opacity",
            image_mark(opaque_logo(64, 64), 0.5, None, false),
            image_of(DynamicImage::ImageRgba8(gradient(800, 600))),
        ),
        (
            "opaque_64_tiled_on_256",
            image_mark(opaque_logo(64, 64), 1.0, None, true),
            image_of(DynamicImage::ImageRgba8(gradient(256, 256))),
        ),
        (
            "overlay_256_scaled_half_on_24",
            image_mark(opaque_logo(256, 256), 1.0, Some(0.5), false),
            image_of(DynamicImage::ImageRgba8(gradient(24, 24))),
        ),
        (
            "soft_10_on_16_bit_100",
            image_mark(soft_logo, 1.0, None, false),
            image_of(DynamicImage::ImageRgba16(base16)),
        ),
    ]
}

/// SHA-256 over the output's size, colour type and raw samples.
fn pixel_hash(img: &Image) -> String {
    let px = img.pixels();
    let mut h = Sha256::new();
    h.update(px.width().to_le_bytes());
    h.update(px.height().to_le_bytes());
    h.update(format!("{:?}", px.color()).as_bytes());
    h.update(px.as_bytes());
    h.finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
}

/// `name hash` lines, one per case, in case order.
fn current_hashes() -> String {
    passing_cases()
        .into_iter()
        .map(|(name, op, base)| {
            let out = op
                .apply(base)
                .unwrap_or_else(|e| panic!("{name}: a passing mark must apply: {e}"));
            format!("{name} {}\n", pixel_hash(&out))
        })
        .collect()
}

#[test]
fn passing_marks_are_byte_identical_to_main() {
    let golden = std::fs::read_to_string(golden_path())
        .expect("golden file")
        .replace("\r\n", "\n");
    assert_eq!(golden.lines().count(), 6, "six goldens");
    assert_eq!(current_hashes(), golden);
}

/// Writes the golden file. Not an assertion: run it with
/// `CRUSTYIMG_BLESS=1 cargo test --test watermark_damage -- --ignored bless`
/// only when the goldens should describe the current code.
#[test]
#[ignore]
fn bless_watermark_goldens() {
    if std::env::var_os("CRUSTYIMG_BLESS").is_none() {
        eprintln!("CRUSTYIMG_BLESS is not set; not writing goldens");
        return;
    }
    let path = golden_path();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, current_hashes()).unwrap();
}
