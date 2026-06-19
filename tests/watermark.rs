//! Integration tests for the `watermark` command (SPEC-029, DEC-031).
//!
//! These drive the REAL compiled binary via `std::process::Command` and assert
//! exit codes + output bytes end-to-end. The overlay is loaded once at the IO
//! boundary (`run_watermark`); the op composites already-decoded pixels.
//!
//! Fixtures are generated NATIVELY (solid-color PNGs via the `image` crate) —
//! no committed binary files, no ImageMagick (AGENTS.md §12). `.unwrap()` here
//! is idiomatic test setup (the `no-unwrap` constraint is scoped to `src/**`).

use std::path::Path;
use std::process::Command;

use image::{DynamicImage, ImageFormat, RgbaImage};
use tempfile::TempDir;

/// Path to the compiled `crustyimg` binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// Write a solid-color RGBA PNG fixture and return its path.
fn write_png(dir: &Path, name: &str, w: u32, h: u32, rgba: [u8; 4]) -> std::path::PathBuf {
    let img = RgbaImage::from_pixel(w, h, image::Rgba(rgba));
    let path = dir.join(name);
    DynamicImage::ImageRgba8(img)
        .save_with_format(&path, ImageFormat::Png)
        .unwrap();
    path
}

#[test]
fn watermark_writes_composited_output() {
    let dir = TempDir::new().unwrap();
    let base = write_png(dir.path(), "base.png", 20, 20, [255, 0, 0, 255]);
    let logo = write_png(dir.path(), "logo.png", 4, 4, [0, 0, 255, 255]);
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "watermark",
            base.to_str().unwrap(),
            "--image",
            logo.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run watermark");

    assert_eq!(output.status.code(), Some(0), "watermark should exit 0");

    // The output decodes and differs from the base (the overlay was composited).
    let out_img = image::open(&out).expect("output should decode").to_rgba8();
    let base_img = image::open(&base).unwrap().to_rgba8();
    assert_eq!(out_img.dimensions(), base_img.dimensions());
    assert_ne!(
        out_img.into_raw(),
        base_img.into_raw(),
        "composited output should differ from the base"
    );
}

#[test]
fn watermark_missing_image_exits_3() {
    let dir = TempDir::new().unwrap();
    let base = write_png(dir.path(), "base.png", 20, 20, [255, 0, 0, 255]);
    let missing = dir.path().join("nonexistent.png");
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "watermark",
            base.to_str().unwrap(),
            "--image",
            missing.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run watermark");

    assert_eq!(
        output.status.code(),
        Some(3),
        "a missing --image is a load error (exit 3)"
    );
}

#[test]
fn watermark_bad_opacity_exits_2() {
    let dir = TempDir::new().unwrap();
    let base = write_png(dir.path(), "base.png", 20, 20, [255, 0, 0, 255]);
    let logo = write_png(dir.path(), "logo.png", 4, 4, [0, 0, 255, 255]);
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "watermark",
            base.to_str().unwrap(),
            "--image",
            logo.to_str().unwrap(),
            "--opacity",
            "2.0",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run watermark");

    assert_eq!(
        output.status.code(),
        Some(2),
        "an out-of-range --opacity is a usage error (exit 2)"
    );
}

#[test]
fn watermark_unknown_gravity_exits_2() {
    let dir = TempDir::new().unwrap();
    let base = write_png(dir.path(), "base.png", 20, 20, [255, 0, 0, 255]);
    let logo = write_png(dir.path(), "logo.png", 4, 4, [0, 0, 255, 255]);
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "watermark",
            base.to_str().unwrap(),
            "--image",
            logo.to_str().unwrap(),
            "--gravity",
            "sideways",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run watermark");

    assert_eq!(
        output.status.code(),
        Some(2),
        "an unknown --gravity is a usage error (exit 2)"
    );
}

#[test]
fn watermark_multi_input_fanout() {
    let dir = TempDir::new().unwrap();
    let base_a = write_png(dir.path(), "a.png", 20, 20, [255, 0, 0, 255]);
    let base_b = write_png(dir.path(), "b.png", 20, 20, [0, 255, 0, 255]);
    let logo = write_png(dir.path(), "logo.png", 4, 4, [0, 0, 255, 255]);
    let out_dir = dir.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let output = Command::new(BIN)
        .args([
            "watermark",
            base_a.to_str().unwrap(),
            base_b.to_str().unwrap(),
            "--image",
            logo.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run watermark");

    assert_eq!(output.status.code(), Some(0), "multi-input fan-out exits 0");

    // Two composited outputs were written.
    let out_a = out_dir.join("a.png");
    let out_b = out_dir.join("b.png");
    assert!(out_a.exists(), "output for a.png should exist");
    assert!(out_b.exists(), "output for b.png should exist");
    assert!(image::open(&out_a).is_ok(), "a output decodes");
    assert!(image::open(&out_b).is_ok(), "b output decodes");
}
