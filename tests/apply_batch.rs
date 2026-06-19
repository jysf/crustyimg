//! Integration tests for the parallel batch `apply --recipe` path (SPEC-031).
//!
//! All tests drive the real compiled binary via `env!("CARGO_BIN_EXE_crustyimg")`.
//! Fixtures are synthesized in memory with the `image` crate — no committed
//! binary files, no ImageMagick. Recipes are written as inline TOML to a tempdir.

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

/// Write a recipe TOML string to `dir/name`. Returns the path.
fn write_recipe(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

/// A minimal recipe that resizes to max 16 (tiny, fast, verifiable).
const RESIZE_RECIPE: &str = r#"
version = "1"

[[step]]
op = "resize"
mode = "max"
width = 16
"#;

/// A minimal no-op identity recipe.
const IDENTITY_RECIPE: &str = r#"
version = "1"

[[step]]
op = "identity"
"#;

// ── Tests ─────────────────────────────────────────────────────────────────────

/// `apply --recipe r.toml a.png b.png c.png --out-dir out/ -y` writes 3 outputs; exit 0.
#[test]
fn apply_batch_writes_all_outputs() {
    let dir = TempDir::new().unwrap();
    let recipe = write_recipe(&dir, "r.toml", RESIZE_RECIPE);
    let a = write_png(&dir, "a.png", 32, 32);
    let b = write_png(&dir, "b.png", 32, 32);
    let c = write_png(&dir, "c.png", 32, 32);
    let out_dir = dir.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            c.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply batch");

    assert!(
        output.status.success(),
        "exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out_dir.join("a.png").exists(), "a.png must be created");
    assert!(out_dir.join("b.png").exists(), "b.png must be created");
    assert!(out_dir.join("c.png").exists(), "c.png must be created");
}

/// `apply --recipe r.toml a.png -o out.png -y` exits 0 and `out.png` exists.
/// Single-input behavior is preserved unchanged.
#[test]
fn apply_single_input_unchanged() {
    let dir = TempDir::new().unwrap();
    let recipe = write_recipe(&dir, "r.toml", IDENTITY_RECIPE);
    let a = write_png(&dir, "a.png", 20, 20);
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            a.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply single");

    assert!(
        output.status.success(),
        "exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out.exists(), "output file must exist");
}

/// 2 good PNGs + 1 bogus (non-image) input → 2 outputs written, exit 6.
#[test]
fn apply_batch_partial_failure_exits_6() {
    let dir = TempDir::new().unwrap();
    let recipe = write_recipe(&dir, "r.toml", RESIZE_RECIPE);
    let a = write_png(&dir, "a.png", 32, 32);
    let b = write_png(&dir, "b.png", 32, 32);
    // A text file that is NOT a valid image.
    let bad = dir.path().join("bad.png");
    std::fs::write(&bad, b"this is not an image").unwrap();
    let out_dir = dir.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            bad.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply partial");

    assert_eq!(
        output.status.code(),
        Some(6),
        "partial failure must exit 6; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out_dir.join("a.png").exists(), "a.png must be written");
    assert!(out_dir.join("b.png").exists(), "b.png must be written");
}

/// 2 inputs with no `--out-dir` → exit 2.
#[test]
fn apply_batch_multi_without_out_dir_exits_2() {
    let dir = TempDir::new().unwrap();
    let recipe = write_recipe(&dir, "r.toml", IDENTITY_RECIPE);
    let a = write_png(&dir, "a.png", 16, 16);
    let b = write_png(&dir, "b.png", 16, 16);

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            a.to_str().unwrap(),
            b.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run apply no-out-dir");

    assert_eq!(
        output.status.code(),
        Some(2),
        "missing --out-dir must exit 2; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Same recipe + inputs with `-j 1` and `-j 4` produce identical output dimensions.
#[test]
fn apply_batch_jobs_one_and_four_agree() {
    let dir = TempDir::new().unwrap();
    let recipe = write_recipe(&dir, "r.toml", RESIZE_RECIPE);
    let a = write_png(&dir, "a.png", 32, 32);
    let b = write_png(&dir, "b.png", 32, 32);

    // Run with -j 1
    let out1 = dir.path().join("out1");
    std::fs::create_dir_all(&out1).unwrap();
    let status1 = Command::new(BIN)
        .args([
            "-j",
            "1",
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--out-dir",
            out1.to_str().unwrap(),
            "-y",
        ])
        .status()
        .expect("failed to run -j 1");
    assert!(status1.success(), "-j 1 must exit 0");

    // Run with -j 4
    let out4 = dir.path().join("out4");
    std::fs::create_dir_all(&out4).unwrap();
    let status4 = Command::new(BIN)
        .args([
            "-j",
            "4",
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--out-dir",
            out4.to_str().unwrap(),
            "-y",
        ])
        .status()
        .expect("failed to run -j 4");
    assert!(status4.success(), "-j 4 must exit 0");

    // Both outputs must have the same dimensions.
    for name in ["a.png", "b.png"] {
        let img1 = image::open(out1.join(name)).unwrap();
        let img4 = image::open(out4.join(name)).unwrap();
        assert_eq!(
            (img1.width(), img1.height()),
            (img4.width(), img4.height()),
            "{name}: -j1 and -j4 dimensions must agree"
        );
    }
}

/// `--name-template {stem}_web.{ext}` → outputs named `*_web.png`.
#[test]
fn apply_batch_name_template_honored() {
    let dir = TempDir::new().unwrap();
    let recipe = write_recipe(&dir, "r.toml", IDENTITY_RECIPE);
    let a = write_png(&dir, "photo.png", 16, 16);
    let b = write_png(&dir, "logo.png", 16, 16);
    let out_dir = dir.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let output = Command::new(BIN)
        .args([
            "--name-template",
            "{stem}_web.{ext}",
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply with name-template");

    assert!(
        output.status.success(),
        "exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        out_dir.join("photo_web.png").exists(),
        "photo_web.png must exist"
    );
    assert!(
        out_dir.join("logo_web.png").exists(),
        "logo_web.png must exist"
    );
}

/// A recipe naming an unknown op → exit 1.
#[test]
fn apply_batch_unknown_op_exits_1() {
    let dir = TempDir::new().unwrap();
    let bad_recipe = write_recipe(
        &dir,
        "bad.toml",
        r#"
version = "1"

[[step]]
op = "no_such_op_ever"
"#,
    );
    let a = write_png(&dir, "a.png", 16, 16);
    let out_dir = dir.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            bad_recipe.to_str().unwrap(),
            a.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run apply unknown-op");

    assert_eq!(
        output.status.code(),
        Some(1),
        "unknown op must exit 1; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `--quiet` run → stdout empty (progress only on stderr / hidden).
#[test]
fn apply_batch_quiet_clean_stdout() {
    let dir = TempDir::new().unwrap();
    let recipe = write_recipe(&dir, "r.toml", IDENTITY_RECIPE);
    let a = write_png(&dir, "a.png", 16, 16);
    let b = write_png(&dir, "b.png", 16, 16);
    let out_dir = dir.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let output = Command::new(BIN)
        .args([
            "-Q",
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply --quiet");

    assert!(
        output.status.success(),
        "exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "stdout must be empty with --quiet; got: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
}
