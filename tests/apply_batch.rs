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

/// Generate a tiny gradient RGB JPEG and write it to `dir/name`. Returns the
/// path. Mirrors `write_png` for JPEG fixtures (SPEC-126: the format-agreement
/// tests need a source format distinct from PNG, so preserving it can't be
/// mistaken for defaulting to PNG).
fn write_jpeg(dir: &TempDir, name: &str, w: u32, h: u32) -> PathBuf {
    let img = RgbImage::from_fn(w, h, |x, _y| {
        image::Rgb([(x * 255 / w.max(1)) as u8, 100u8, 150u8])
    });
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Jpeg)
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

// ─── SPEC-035: recipe resource-limit integration tests ───────────────────────

/// A recipe file larger than `RECIPE_MAX_BYTES` must cause the CLI to exit 1
/// (the CLI pre-read metadata guard fires before `read_to_string`) and write a
/// non-empty error message to stderr.
#[test]
fn apply_oversized_recipe_file_exits_1() {
    use crustyimg::recipe::RECIPE_MAX_BYTES;

    let dir = TempDir::new().unwrap();
    // Build a recipe file that exceeds the cap by 1 byte.
    // '#' makes it TOML-comment content so it would otherwise be valid.
    let oversized_content = "#".repeat(RECIPE_MAX_BYTES + 1);
    let recipe_path = write_recipe(&dir, "big.toml", &oversized_content);
    let img = write_png(&dir, "img.png", 8, 8);
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe_path.to_str().unwrap(),
            img.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run apply with oversized recipe");

    assert_eq!(
        output.status.code(),
        Some(1),
        "oversized recipe must exit 1; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.stderr.is_empty(),
        "stderr must be non-empty for oversized recipe error"
    );
}

/// A normal recipe file (well within `RECIPE_MAX_BYTES`) must still apply
/// successfully — regression guard for the pre-read change.
#[test]
fn apply_normal_recipe_still_works() {
    let dir = TempDir::new().unwrap();
    let recipe = write_recipe(&dir, "r.toml", IDENTITY_RECIPE);
    let img = write_png(&dir, "img.png", 8, 8);
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            img.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply with normal recipe");

    assert!(
        output.status.success(),
        "normal recipe must exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out.exists(), "output file must exist for normal recipe");
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

// ── SPEC-037: resize output byte cap via recipe path (DEC-038) ───────────────

/// `apply --recipe` with a recipe containing `resize exact 50000x50000` exits 1.
///
/// The resize output cap (DEC-038) must fire through the recipe path as well as
/// the CLI path. A tiny input (4×4) + huge target dims ensure the guard rejects
/// before the resize backend allocates (the test stays cheap / cannot OOM).
#[test]
fn apply_recipe_with_oversized_resize_exits_1() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 4, 4);
    let out_dir = dir.path().join("out");
    std::fs::create_dir(&out_dir).unwrap();

    // Recipe with a resize step whose output exceeds 512 MiB (DEC-038).
    // 50000 × 50000 × 4 bytes = 10 GB >> 512 MiB.
    let recipe = write_recipe(
        &dir,
        "oversized.toml",
        r#"
version = "1"

[[step]]
op = "resize"
mode = "exact"
width = 50000
height = 50000
"#,
    );

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            input.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply --recipe with oversized resize");

    assert_eq!(
        output.status.code(),
        Some(1),
        "oversized resize via recipe must exit 1; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ─── SPEC-126: apply/build output-format agreement ───────────────────────────

/// `apply --format X` must be honoured identically at 1 input and at N inputs,
/// for at least two target formats — so this cannot pass by coincidence of the
/// source format (AC-1). Before SPEC-126 the N-input (`--out-dir`) fan-out did
/// no format resolution at all and silently ignored `--format`.
///
/// Asserted on the written BYTES sniffed via `image::guess_format` (AGENTS
/// §15 / Call 4's own reasoning) — the filename extension is produced by the
/// same resolution this test exists to check, so it cannot be independent
/// evidence.
#[test]
fn apply_honours_format_at_every_arity() {
    let dir = TempDir::new().unwrap();
    let recipe = write_recipe(&dir, "r.toml", IDENTITY_RECIPE);

    // Target format 1 (png), driven at 1 input and at 2 inputs, from a JPEG source.
    let one_jpeg = write_jpeg(&dir, "one.jpg", 16, 16);
    let out_one_png = dir.path().join("out_one_png");
    std::fs::create_dir_all(&out_one_png).unwrap();
    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            one_jpeg.to_str().unwrap(),
            "--format",
            "png",
            "--out-dir",
            out_one_png.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply 1-input --format png");
    assert!(
        output.status.success(),
        "1 input --format png: exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = std::fs::read(out_one_png.join("one.png")).expect("one.png must be written");
    assert_eq!(
        image::guess_format(&bytes).expect("must sniff a known format"),
        ImageFormat::Png,
        "1 input: --format png must be honoured"
    );

    let two_a = write_jpeg(&dir, "two_a.jpg", 16, 16);
    let two_b = write_jpeg(&dir, "two_b.jpg", 16, 16);
    let out_two_png = dir.path().join("out_two_png");
    std::fs::create_dir_all(&out_two_png).unwrap();
    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            two_a.to_str().unwrap(),
            two_b.to_str().unwrap(),
            "--format",
            "png",
            "--out-dir",
            out_two_png.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply 2-input --format png");
    assert!(
        output.status.success(),
        "N inputs --format png: exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for name in ["two_a.png", "two_b.png"] {
        let bytes = std::fs::read(out_two_png.join(name))
            .unwrap_or_else(|e| panic!("{name} must be written: {e}"));
        assert_eq!(
            image::guess_format(&bytes).expect("must sniff a known format"),
            ImageFormat::Png,
            "N inputs: --format png must be honoured for {name} \
             (this is the multi-input arm that silently ignored --format before SPEC-126)"
        );
    }

    // Target format 2 (jpeg), driven at 1 input and at 2 inputs, from a PNG source.
    let one_png = write_png(&dir, "one.png", 16, 16);
    let out_one_jpg = dir.path().join("out_one_jpg");
    std::fs::create_dir_all(&out_one_jpg).unwrap();
    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            one_png.to_str().unwrap(),
            "--format",
            "jpeg",
            "--out-dir",
            out_one_jpg.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply 1-input --format jpeg");
    assert!(
        output.status.success(),
        "1 input --format jpeg: exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = std::fs::read(out_one_jpg.join("one.jpg")).expect("one.jpg must be written");
    assert_eq!(
        image::guess_format(&bytes).expect("must sniff a known format"),
        ImageFormat::Jpeg,
        "1 input: --format jpeg must be honoured"
    );

    let two_c = write_png(&dir, "two_c.png", 16, 16);
    let two_d = write_png(&dir, "two_d.png", 16, 16);
    let out_two_jpg = dir.path().join("out_two_jpg");
    std::fs::create_dir_all(&out_two_jpg).unwrap();
    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            two_c.to_str().unwrap(),
            two_d.to_str().unwrap(),
            "--format",
            "jpeg",
            "--out-dir",
            out_two_jpg.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply 2-input --format jpeg");
    assert!(
        output.status.success(),
        "N inputs --format jpeg: exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for name in ["two_c.jpg", "two_d.jpg"] {
        let bytes = std::fs::read(out_two_jpg.join(name))
            .unwrap_or_else(|e| panic!("{name} must be written: {e}"));
        assert_eq!(
            image::guess_format(&bytes).expect("must sniff a known format"),
            ImageFormat::Jpeg,
            "N inputs: --format jpeg must be honoured for {name}"
        );
    }
}

/// With no `--format`, `apply` must preserve the source format at 1 input and
/// at N inputs — driven on a JPEG source AND a PNG source (AC-2), so
/// "preserved" cannot be indistinguishable from "always PNG" (the exact
/// defect this spec fixes: before SPEC-126, 1 input with no `--format`
/// defaulted to PNG regardless of the source).
#[test]
fn apply_preserves_source_format_at_every_arity() {
    let dir = TempDir::new().unwrap();
    let recipe = write_recipe(&dir, "r.toml", IDENTITY_RECIPE);

    // JPEG source, 1 input.
    let j1 = write_jpeg(&dir, "j1.jpg", 16, 16);
    let out_j1 = dir.path().join("out_j1");
    std::fs::create_dir_all(&out_j1).unwrap();
    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            j1.to_str().unwrap(),
            "--out-dir",
            out_j1.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply 1-input jpeg no-format");
    assert!(output.status.success(), "exit 0 expected");
    let bytes = std::fs::read(out_j1.join("j1.jpg")).expect("j1.jpg must be written");
    assert_eq!(
        image::guess_format(&bytes).expect("must sniff a known format"),
        ImageFormat::Jpeg,
        "1 JPEG input, no --format: source format must be preserved, not defaulted to PNG"
    );

    // JPEG source, 2 inputs.
    let j2a = write_jpeg(&dir, "j2a.jpg", 16, 16);
    let j2b = write_jpeg(&dir, "j2b.jpg", 16, 16);
    let out_j2 = dir.path().join("out_j2");
    std::fs::create_dir_all(&out_j2).unwrap();
    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            j2a.to_str().unwrap(),
            j2b.to_str().unwrap(),
            "--out-dir",
            out_j2.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply 2-input jpeg no-format");
    assert!(output.status.success(), "exit 0 expected");
    for name in ["j2a.jpg", "j2b.jpg"] {
        let bytes = std::fs::read(out_j2.join(name))
            .unwrap_or_else(|e| panic!("{name} must be written: {e}"));
        assert_eq!(
            image::guess_format(&bytes).expect("must sniff a known format"),
            ImageFormat::Jpeg,
            "N JPEG inputs, no --format: source format must be preserved for {name}"
        );
    }

    // PNG source, 1 input.
    let p1 = write_png(&dir, "p1.png", 16, 16);
    let out_p1 = dir.path().join("out_p1");
    std::fs::create_dir_all(&out_p1).unwrap();
    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            p1.to_str().unwrap(),
            "--out-dir",
            out_p1.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply 1-input png no-format");
    assert!(output.status.success(), "exit 0 expected");
    let bytes = std::fs::read(out_p1.join("p1.png")).expect("p1.png must be written");
    assert_eq!(
        image::guess_format(&bytes).expect("must sniff a known format"),
        ImageFormat::Png,
        "1 PNG input, no --format: source format must be preserved"
    );

    // PNG source, 2 inputs.
    let p2a = write_png(&dir, "p2a.png", 16, 16);
    let p2b = write_png(&dir, "p2b.png", 16, 16);
    let out_p2 = dir.path().join("out_p2");
    std::fs::create_dir_all(&out_p2).unwrap();
    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            p2a.to_str().unwrap(),
            p2b.to_str().unwrap(),
            "--out-dir",
            out_p2.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply 2-input png no-format");
    assert!(output.status.success(), "exit 0 expected");
    for name in ["p2a.png", "p2b.png"] {
        let bytes = std::fs::read(out_p2.join(name))
            .unwrap_or_else(|e| panic!("{name} must be written: {e}"));
        assert_eq!(
            image::guess_format(&bytes).expect("must sniff a known format"),
            ImageFormat::Png,
            "N PNG inputs, no --format: source format must be preserved for {name}"
        );
    }
}

/// `apply --recipe` and `build` (same recipe, same input, no `--format`
/// anywhere) must produce BYTE-IDENTICAL output (AC-3). Asserted on the raw
/// bytes, not the extension or the summary line (Call 4) — a test pinning
/// `apply` to a format string would go green again the day a good-faith
/// default change let the two paths silently diverge; comparing bytes catches
/// that.
#[test]
fn apply_and_build_agree_byte_for_byte() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    std::fs::write(root.join("r.toml"), RESIZE_RECIPE).unwrap();
    // JPEG source (the spec's own repro format) at the manifest root, so
    // `build`'s relative `source` resolves it directly (DEC-057).
    write_jpeg_at(root, "in.jpg", 32, 32);

    let apply_out = root.join("apply_out");
    std::fs::create_dir_all(&apply_out).unwrap();
    let apply_output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            "r.toml",
            "in.jpg",
            "--out-dir",
            "apply_out",
            "-y",
        ])
        .current_dir(root)
        .output()
        .expect("failed to run apply");
    assert!(
        apply_output.status.success(),
        "apply exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&apply_output.stderr)
    );

    std::fs::write(
        root.join("crustyimg.build.toml"),
        br#"
version = 1

[[target]]
source = "in.jpg"
recipe = "r.toml"
out = "build_out"
"#,
    )
    .unwrap();
    let build_output = Command::new(BIN)
        .arg("build")
        .current_dir(root)
        .output()
        .expect("failed to run build");
    assert!(
        build_output.status.success(),
        "build exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let apply_bytes = std::fs::read(apply_out.join("in.jpg")).expect("apply output must exist");
    let build_bytes =
        std::fs::read(root.join("build_out").join("in.jpg")).expect("build output must exist");
    assert_eq!(
        apply_bytes, build_bytes,
        "apply and build must produce byte-identical output for the same \
         recipe, input, and settings"
    );
}

/// `-o PATH` and `--out-dir DIR` must agree byte-for-byte for the same `apply`
/// invocation (AC-4, Call 4's sibling assertion) — same recipe, same JPEG
/// input, no `--format`.
#[test]
fn apply_output_flags_agree() {
    let dir = TempDir::new().unwrap();
    let recipe = write_recipe(&dir, "r.toml", RESIZE_RECIPE);
    let src = write_jpeg(&dir, "in.jpg", 32, 32);

    let o_path = dir.path().join("o_out.jpg");
    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            src.to_str().unwrap(),
            "-o",
            o_path.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply -o");
    assert!(
        output.status.success(),
        "-o run: exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let dir_out = dir.path().join("dir_out");
    std::fs::create_dir_all(&dir_out).unwrap();
    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            src.to_str().unwrap(),
            "--out-dir",
            dir_out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply --out-dir");
    assert!(
        output.status.success(),
        "--out-dir run: exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let o_bytes = std::fs::read(&o_path).expect("-o output must exist");
    let dir_bytes = std::fs::read(dir_out.join("in.jpg")).expect("--out-dir output must exist");
    assert_eq!(
        o_bytes, dir_bytes,
        "-o and --out-dir must agree byte-for-byte for the same apply invocation \
         (before SPEC-126, --out-dir defaulted to PNG while -o preserved JPEG)"
    );
}

/// Write a tiny gradient RGB JPEG directly to `root/name` (no tempdir
/// wrapper) — used by tests that need `build`'s relative-path resolution
/// (DEC-057), which requires the CWD to be the manifest root.
fn write_jpeg_at(root: &std::path::Path, name: &str, w: u32, h: u32) -> PathBuf {
    let img = RgbImage::from_fn(w, h, |x, _y| {
        image::Rgb([(x * 255 / w.max(1)) as u8, 100u8, 150u8])
    });
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Jpeg)
        .unwrap();
    let path = root.join(name);
    std::fs::write(&path, buf.into_inner()).unwrap();
    path
}
