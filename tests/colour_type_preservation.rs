//! Integration tests for SPEC-121 — ops preserve colour type and bit depth.
//!
//! `Invert`, `Resize` (reached by `resize`/`thumbnail`/`web`) and `Watermark`
//! used to widen every image to RGBA8 and never narrow back (docs/backlog.md,
//! "Live defect — ops widen to RGBA and never narrow back"). These drive the
//! REAL compiled binary end-to-end (matching `tests/watermark.rs`'s
//! convention) and read the output's `ColorType` back via
//! `crustyimg::image::Image::load` — a structural assertion on the decoded
//! IHDR, not a byte-size guess.
//!
//! **One `#[test]` fn per op body per claim, deliberately not bundled**
//! (AC-9): reverting one op body's fix must turn red only the tests below
//! that call it, while the other tests — independent functions — actually
//! run and pass, not merely go unreached by an early panic in a shared fn.

mod common;

use std::path::Path;
use std::process::Command;

use ::image::{ColorType, DynamicImage, ImageFormat, Rgba, RgbaImage};
use crustyimg::image::Image;

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Fixture helpers (native, no ImageMagick — AGENTS §12) ──────────────────

fn write_fixture(dir: &Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// A solid-color opaque RGBA8 PNG — used as a watermark overlay that should
/// narrow the composite back to RGB (Call 2's "fully opaque" direction).
fn write_opaque_overlay(dir: &Path, name: &str) -> std::path::PathBuf {
    let img = RgbaImage::from_pixel(6, 6, Rgba([10, 20, 30, 255]));
    let path = dir.join(name);
    DynamicImage::ImageRgba8(img)
        .save_with_format(&path, ImageFormat::Png)
        .unwrap();
    path
}

/// A solid-color translucent RGBA8 PNG — used as a watermark overlay that
/// genuinely contributes alpha (Call 2's "stays RGBA" direction).
fn write_translucent_overlay(dir: &Path, name: &str) -> std::path::PathBuf {
    let img = RgbaImage::from_pixel(6, 6, Rgba([10, 20, 30, 128]));
    let path = dir.join(name);
    DynamicImage::ImageRgba8(img)
        .save_with_format(&path, ImageFormat::Png)
        .unwrap();
    path
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(BIN).args(args).output().expect("run binary")
}

fn load_color(path: &Path) -> ColorType {
    Image::load(path)
        .unwrap_or_else(|e| panic!("output {path:?} should decode: {e}"))
        .pixels()
        .color()
}

// ── AC-1: an RGB (colour_type=2) source stays RGB through every op verb ────
// rgb_png_stays_rgb_through_resize_thumbnail_edit_and_web

#[test]
fn resize_max_preserves_rgb_colour_type() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(
        dir.path(),
        "src.png",
        &common::solid_png(32, 32, [200, 100, 50]),
    );
    let out = dir.path().join("out.png");
    let output = run(&[
        "resize",
        src.to_str().unwrap(),
        "--max",
        "16",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgb8);
}

#[test]
fn thumbnail_preserves_rgb_colour_type() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(
        dir.path(),
        "src.png",
        &common::solid_png(32, 32, [200, 100, 50]),
    );
    let out = dir.path().join("out.png");
    let output = run(&[
        "thumbnail",
        src.to_str().unwrap(),
        "--size",
        "16",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgb8);
}

#[test]
fn edit_invert_preserves_rgb_colour_type() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(
        dir.path(),
        "src.png",
        &common::solid_png(32, 32, [200, 100, 50]),
    );
    let out = dir.path().join("out.png");
    let output = run(&[
        "edit",
        src.to_str().unwrap(),
        "--invert",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgb8);
}

/// The flagship verb: `web` is `auto-orient → resize → optimize` (docs/backlog.md),
/// so fixing `Resize` fixes `web`.
#[test]
fn web_preserves_rgb_colour_type() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(
        dir.path(),
        "src.png",
        &common::solid_png(32, 32, [200, 100, 50]),
    );
    let out = dir.path().join("out.bin");
    let output = run(&["web", src.to_str().unwrap(), "-o", out.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgb8);
}

// ── AC-2: a 16-bit RGB source stays 16-bit ──────────────────────────────────
// sixteen_bit_png_stays_sixteen_bit

#[test]
fn resize_max_preserves_sixteen_bit() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "src16.png", &common::png_16bit(32, 32));
    let out = dir.path().join("out.png");
    let output = run(&[
        "resize",
        src.to_str().unwrap(),
        "--max",
        "16",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgb16);
}

#[test]
fn edit_invert_preserves_sixteen_bit() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "src16.png", &common::png_16bit(32, 32));
    let out = dir.path().join("out.png");
    let output = run(&[
        "edit",
        src.to_str().unwrap(),
        "--invert",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgb16);
}

// `web` is deliberately NOT covered here for the 16-bit case (unlike AC-1's
// RGB case above): `web`'s final step is `optimize`'s smallest-candidate
// search, which — independently of this spec — may pick WebP/JPEG/AVIF over
// PNG, and (measured) `image`'s own *lossless* WebP encoder has no 16-bit
// mode either, so it silently downgrades via the same "automatically
// convert" behavior Call 3 documents for JPEG/lossy-WebP. That is a real,
// separate silent-downgrade site Call 3's settled scope does not cover
// (JPEG + lossy WebP only) — filed in docs/backlog.md rather than folded
// into this spec's fix, matching AC-3's own boundary discipline. The
// backlog's own 16-bit measurement never claimed `web` either — only
// `resize`/`edit --invert` — so this test suite doesn't overclaim it.

// ── AC-3: an RGBA input with genuine translucency keeps its alpha ──────────
// translucent_rgba_input_keeps_its_alpha — the control that stops "always
// narrow" from destroying real transparency. Passes today; must keep passing.

#[test]
fn resize_keeps_translucent_alpha() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "src.png", &common::rgba_png(32, 32));
    let out = dir.path().join("out.png");
    let output = run(&[
        "resize",
        src.to_str().unwrap(),
        "--max",
        "16",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgba8);
}

#[test]
fn edit_invert_keeps_translucent_alpha() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "src.png", &common::rgba_png(32, 32));
    let out = dir.path().join("out.png");
    let output = run(&[
        "edit",
        src.to_str().unwrap(),
        "--invert",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgba8);
}

// ── AC-4: `Watermark` decides in code, not by exemption (Call 2) ───────────

/// watermark_narrows_when_the_composite_is_opaque
#[test]
fn watermark_narrows_when_composite_is_opaque() {
    let dir = tempfile::tempdir().unwrap();
    let base = write_fixture(
        dir.path(),
        "base.png",
        &common::solid_png(32, 32, [200, 100, 50]),
    );
    let overlay = write_opaque_overlay(dir.path(), "overlay.png");
    let out = dir.path().join("out.png");
    let output = run(&[
        "watermark",
        base.to_str().unwrap(),
        "--image",
        overlay.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        load_color(&out),
        ColorType::Rgb8,
        "a fully opaque overlay over an RGB base must narrow like the other ops"
    );
}

/// watermark_keeps_alpha_when_the_overlay_is_translucent
#[test]
fn watermark_keeps_alpha_when_overlay_is_translucent() {
    let dir = tempfile::tempdir().unwrap();
    let base = write_fixture(
        dir.path(),
        "base.png",
        &common::solid_png(32, 32, [200, 100, 50]),
    );
    let overlay = write_translucent_overlay(dir.path(), "overlay.png");
    let out = dir.path().join("out.png");
    let output = run(&[
        "watermark",
        base.to_str().unwrap(),
        "--image",
        overlay.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        load_color(&out),
        ColorType::Rgba8,
        "a translucent overlay genuinely produces alpha and must stay RGBA"
    );
}

// ── AC-6: the byte win is measured, not assumed ─────────────────────────────
// rgb_output_is_smaller_than_rgba_for_the_same_pixels

#[test]
fn rgb_output_is_smaller_than_rgba_for_the_same_pixels() {
    use ::image::{Rgb, RgbImage};

    let (w, h) = (512, 512);
    let rgb = DynamicImage::ImageRgb8(RgbImage::from_fn(w, h, |x, y| {
        Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8])
    }));
    let rgba = DynamicImage::ImageRgba8(RgbaImage::from_fn(w, h, |x, y| {
        Rgba([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8, 255])
    }));

    let rgb_image = Image::from_parts(rgb, ImageFormat::Png, None);
    let rgba_image = Image::from_parts(rgba, ImageFormat::Png, None);

    let rgb_bytes = crustyimg::sink::encode_to_bytes(&rgb_image, ImageFormat::Png, None).unwrap();
    let rgba_bytes = crustyimg::sink::encode_to_bytes(&rgba_image, ImageFormat::Png, None).unwrap();

    assert!(
        rgb_bytes.len() < rgba_bytes.len(),
        "RGB ({} B) should be smaller than RGBA ({} B) for identical pixels",
        rgb_bytes.len(),
        rgba_bytes.len()
    );
}
