//! Integration tests for `edit` + `--save-recipe` (SPEC-032).
//!
//! All tests drive the real compiled binary via `env!("CARGO_BIN_EXE_crustyimg")`.
//! Fixtures are synthesized in memory with the `image` crate — no committed
//! binary files. Outputs and recipes are written to a tempdir.

use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;

use image::{DynamicImage, ImageFormat, RgbImage};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Fixture helpers ───────────────────────────────────────────────────────────

/// Generate a tiny solid-color RGB PNG and write it to `dir/name`. Returns the path.
fn write_png(dir: &TempDir, name: &str, w: u32, h: u32) -> PathBuf {
    let img = RgbImage::from_pixel(w, h, image::Rgb([42u8, 100u8, 200u8]));
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .unwrap();
    let path = dir.path().join(name);
    std::fs::write(&path, buf.into_inner()).unwrap();
    path
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// `edit in.png --resize-max 8 -o out.png` → out.png exists, max edge == 8; exit 0.
#[test]
fn edit_resize_writes_output() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 32, 32);
    let out = dir.path().join("out.png");

    let status = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--resize-max",
            "8",
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .status()
        .expect("failed to run edit");

    assert!(
        status.success(),
        "edit --resize-max 8 must exit 0; got: {:?}",
        status.code()
    );
    assert!(out.exists(), "out.png must be created");

    let img = image::open(&out).unwrap();
    assert!(
        img.width() <= 8 && img.height() <= 8,
        "max edge must be ≤ 8, got {}×{}",
        img.width(),
        img.height()
    );
}

/// `edit in.png -o out.png` with NO op flag → exit 2.
#[test]
fn edit_no_ops_exits_2() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 16, 16);
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args(["edit", input.to_str().unwrap(), "-o", out.to_str().unwrap()])
        .output()
        .expect("failed to run edit no-ops");

    assert_eq!(
        output.status.code(),
        Some(2),
        "no op flags must exit 2; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `edit nope.png --invert -o out.png` (nonexistent input) → exit 3.
#[test]
fn edit_missing_input_exits_3() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args(["edit", "nope.png", "--invert", "-o", out.to_str().unwrap()])
        .output()
        .expect("failed to run edit missing-input");

    assert_eq!(
        output.status.code(),
        Some(3),
        "missing input must exit 3; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `edit in.png --resize-max 8 --save-recipe r.toml -o out.png` writes both
/// `out.png` and a valid `r.toml` containing `version = "1"`, `op = "resize"`,
/// and `width = 8`.
#[test]
fn edit_save_recipe_writes_parseable_toml() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 32, 32);
    let out = dir.path().join("out.png");
    let recipe = dir.path().join("r.toml");

    let status = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--resize-max",
            "8",
            "--save-recipe",
            recipe.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .status()
        .expect("failed to run edit --save-recipe");

    assert!(
        status.success(),
        "edit --save-recipe must exit 0; got: {:?}",
        status.code()
    );
    assert!(out.exists(), "out.png must be created");
    assert!(recipe.exists(), "r.toml must be created");

    let toml_text = std::fs::read_to_string(&recipe).unwrap();
    assert!(
        toml_text.contains("version = \"1\""),
        "recipe must have version = \"1\"; got: {toml_text}"
    );
    assert!(
        toml_text.contains("op = \"resize\""),
        "recipe must have op = \"resize\"; got: {toml_text}"
    );
    assert!(
        toml_text.contains("width = 8"),
        "recipe must have width = 8; got: {toml_text}"
    );
}

/// Round-trip: `edit in.png --auto-orient --resize-max 8 --invert --save-recipe r.toml
/// -o edit_out.png` then `apply --recipe r.toml in.png -o apply_out.png` produces
/// the same bytes.
#[test]
fn edit_save_recipe_round_trips_through_apply() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 32, 32);
    let edit_out = dir.path().join("edit_out.png");
    let apply_out = dir.path().join("apply_out.png");
    let recipe = dir.path().join("r.toml");

    // Run edit with all three ops and save the recipe.
    let status = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--auto-orient",
            "--resize-max",
            "8",
            "--invert",
            "--save-recipe",
            recipe.to_str().unwrap(),
            "-o",
            edit_out.to_str().unwrap(),
            "-y",
        ])
        .status()
        .expect("failed to run edit round-trip");

    assert!(
        status.success(),
        "edit (round-trip) must exit 0; got: {:?}",
        status.code()
    );

    // Apply the saved recipe to the same input.
    let status2 = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            input.to_str().unwrap(),
            "-o",
            apply_out.to_str().unwrap(),
            "-y",
        ])
        .status()
        .expect("failed to run apply round-trip");

    assert!(
        status2.success(),
        "apply (round-trip) must exit 0; got: {:?}",
        status2.code()
    );

    // Byte-for-byte equality: same ops, same order, same input → same output.
    let edit_bytes = std::fs::read(&edit_out).unwrap();
    let apply_bytes = std::fs::read(&apply_out).unwrap();
    assert_eq!(
        edit_bytes, apply_bytes,
        "edit output and apply-of-saved-recipe output must be byte-identical"
    );
}

/// Flag order is independent: `--invert --resize-max 8` and `--resize-max 8 --invert`
/// produce identical output bytes (canonical order is positional-independent).
#[test]
fn edit_flag_order_independent() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 32, 32);
    let a = dir.path().join("a.png");
    let b = dir.path().join("b.png");

    // Order A: --invert first, then --resize-max.
    let status_a = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--invert",
            "--resize-max",
            "8",
            "-o",
            a.to_str().unwrap(),
            "-y",
        ])
        .status()
        .expect("failed to run edit order A");

    assert!(status_a.success(), "edit order A must exit 0");

    // Order B: --resize-max first, then --invert.
    let status_b = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--resize-max",
            "8",
            "--invert",
            "-o",
            b.to_str().unwrap(),
            "-y",
        ])
        .status()
        .expect("failed to run edit order B");

    assert!(status_b.success(), "edit order B must exit 0");

    // Both outputs must be byte-identical (canonical ordering enforced by build_edit_ops).
    let bytes_a = std::fs::read(&a).unwrap();
    let bytes_b = std::fs::read(&b).unwrap();
    assert_eq!(bytes_a, bytes_b, "flag order must not affect output bytes");
}

/// `--save-recipe` to an unwritable path (nonexistent directory) → exit 5.
#[test]
fn edit_save_recipe_unwritable_exits_5() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 16, 16);
    let out = dir.path().join("out.png");
    // Point save-recipe at a path whose parent directory does not exist.
    let bad_recipe = dir.path().join("no_such_dir").join("r.toml");

    let output = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--resize-max",
            "8",
            "--save-recipe",
            bad_recipe.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run edit unwritable recipe");

    assert_eq!(
        output.status.code(),
        Some(5),
        "unwritable recipe path must exit 5; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
