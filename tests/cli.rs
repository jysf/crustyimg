//! Integration tests for the `crustyimg` binary CLI (SPEC-007).
//!
//! All tests drive the real compiled binary via `env!("CARGO_BIN_EXE_crustyimg")`
//! and `std::process::Command`. Fixtures are generated in-memory with the `image`
//! crate — no committed binary files, no ImageMagick.
//!
//! Stdout is trimmed before string assertions to handle Windows `\r\n` line endings.

use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;

use image::{DynamicImage, ImageFormat, RgbImage};
use tempfile::TempDir;

mod common;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Fixture helpers ───────────────────────────────────────────────────────────

/// Generate a tiny solid-color RGB PNG in memory and write it to `dir/name`.
/// Returns the full path.
fn write_test_png(dir: &TempDir, name: &str, w: u32, h: u32) -> PathBuf {
    let img = RgbImage::from_pixel(w, h, image::Rgb([42u8, 100u8, 200u8]));
    let dyn_img = DynamicImage::ImageRgb8(img);
    let mut buf = Cursor::new(Vec::new());
    dyn_img.write_to(&mut buf, ImageFormat::Png).unwrap();
    let path = dir.path().join(name);
    std::fs::write(&path, buf.into_inner()).unwrap();
    path
}

/// Write a minimal valid recipe TOML to `dir/name`.
fn write_recipe(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

/// Trim stdout bytes to a `String`, stripping leading/trailing whitespace
/// (handles Windows `\r\n`).
fn stdout_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// Trim stderr bytes to a `String`.
fn stderr_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_owned()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// `--help` exits 0 and lists every MVP subcommand name.
#[test]
fn help_lists_all_subcommands() {
    let output = Command::new(BIN)
        .arg("--help")
        .output()
        .expect("failed to run crustyimg --help");

    assert!(output.status.success(), "--help should exit 0");
    let stdout = stdout_str(&output);

    // Every subcommand name must appear in the help output.
    let expected = [
        "view",
        "info",
        "diff",
        "resize",
        "thumbnail",
        "shrink",
        "convert",
        "optimize",
        "responsive",
        "auto-orient",
        "watermark",
        "strip",
        "clean",
        "set",
        "copy-metadata",
        "edit",
        "apply",
    ];
    for name in expected {
        assert!(
            stdout.contains(name),
            "--help output should list subcommand '{name}', got:\n{stdout}"
        );
    }
}

/// `--version` exits 0 and contains the crate semver.
#[test]
fn version_prints_semver() {
    let output = Command::new(BIN)
        .arg("--version")
        .output()
        .expect("failed to run crustyimg --version");

    assert!(output.status.success(), "--version should exit 0");
    let stdout = stdout_str(&output);
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "--version output {:?} should contain package version {}",
        stdout,
        env!("CARGO_PKG_VERSION")
    );
}

/// An unknown subcommand exits with code 2 (clap usage error).
#[test]
fn unknown_subcommand_is_usage_error() {
    let output = Command::new(BIN)
        .arg("frobnicate")
        .arg("x.png")
        .output()
        .expect("failed to run crustyimg frobnicate");

    assert_eq!(
        output.status.code(),
        Some(2),
        "unknown subcommand should exit 2"
    );
}

/// Every documented subcommand accepts `--help` and exits 0.
///
/// This proves each variant and its args are declared in clap and parse cleanly.
#[test]
fn each_subcommand_help_parses() {
    let subcommands = [
        "view",
        "info",
        "diff",
        "resize",
        "thumbnail",
        "shrink",
        "convert",
        "optimize",
        "responsive",
        "auto-orient",
        "watermark",
        "strip",
        "clean",
        "set",
        "copy-metadata",
        "edit",
        "apply",
    ];

    for cmd in subcommands {
        let output = Command::new(BIN)
            .arg(cmd)
            .arg("--help")
            .output()
            .unwrap_or_else(|e| panic!("failed to run crustyimg {cmd} --help: {e}"));

        assert!(
            output.status.success(),
            "crustyimg {cmd} --help should exit 0; stderr: {}",
            stderr_str(&output)
        );
    }
}

/// `apply --recipe r.toml in.png -o out.png` runs end-to-end:
/// exits 0, writes a non-empty decodable output image.
#[test]
fn apply_recipe_runs_end_to_end() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 4, 4);
    let recipe_path = write_recipe(
        &dir,
        "r.toml",
        "version = \"1\"\n[[step]]\nop = \"invert\"\n",
    );
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe_path.to_str().unwrap(),
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run apply");

    assert_eq!(
        output.status.code(),
        Some(0),
        "apply should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should have been created");
    let metadata = std::fs::metadata(&out_path).unwrap();
    assert!(metadata.len() > 0, "output file should be non-empty");

    // The output must be a decodable image with the same dimensions.
    let decoded = image::open(&out_path).expect("output should be a decodable image");
    assert_eq!(decoded.width(), 4, "output width should match input");
    assert_eq!(decoded.height(), 4, "output height should match input");
}

/// `apply --recipe r.toml in.png -o -` writes only encoded bytes to stdout
/// (no diagnostics mixed in); stdout decodes as a valid image.
#[test]
fn apply_to_stdout_keeps_stdout_clean() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 4, 4);
    let recipe_path = write_recipe(
        &dir,
        "r.toml",
        "version = \"1\"\n[[step]]\nop = \"invert\"\n",
    );

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe_path.to_str().unwrap(),
            in_path.to_str().unwrap(),
            "-o",
            "-",
            "--format",
            "png",
        ])
        .output()
        .expect("failed to run apply stdout");

    assert_eq!(
        output.status.code(),
        Some(0),
        "apply -o - should exit 0; stderr: {}",
        stderr_str(&output)
    );

    // stdout must be ONLY the encoded image bytes — decodable as PNG.
    let decoded = image::load_from_memory(&output.stdout)
        .expect("stdout bytes should decode as a valid image");
    assert_eq!(decoded.width(), 4);
    assert_eq!(decoded.height(), 4);
}

/// `edit` without any op flag exits 2 (usage error: "requires at least one
/// operation flag"). SPEC-032 wired `edit`; no commands remain as stubs.
///
/// (resize is now real — SPEC-011; thumbnail is now real — SPEC-012;
/// shrink is now real — SPEC-013; convert is now real — SPEC-014;
/// auto-orient is now real — SPEC-015; watermark is now real — SPEC-029;
/// edit is now real — SPEC-032.)
#[test]
fn stub_command_returns_not_implemented() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 4, 4);

    let output = Command::new(BIN)
        .args(["edit", in_path.to_str().unwrap()])
        .output()
        .expect("failed to run edit");

    assert_eq!(
        output.status.code(),
        Some(2),
        "edit without op flags must exit 2 (usage error)"
    );
    let stderr = stderr_str(&output);
    assert!(
        stderr.contains("requires at least one operation flag"),
        "stderr should contain 'requires at least one operation flag', got: {stderr}"
    );
}

// ── SPEC-015 auto-orient integration tests ────────────────────────────────────

/// `auto-orient` on a JPEG with Orientation=6 rotates the pixels (4×2 →
/// 2×4) and the re-encoded output carries no EXIF (`info --json` reports
/// `"has_exif":false`).
#[test]
fn auto_orient_cli_rotates_and_clears_tag() {
    let dir = tempfile::tempdir().expect("tempdir");

    // Write a 4×2 JPEG with Orientation=6 (Rotate90).
    let jpg_bytes = common::jpeg_with_orientation(4, 2, 6);
    let in_path = dir.path().join("in.jpg");
    std::fs::write(&in_path, &jpg_bytes).unwrap();

    let out_path = dir.path().join("out.jpg");

    // Run auto-orient.
    let output = Command::new(BIN)
        .args([
            "auto-orient",
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run auto-orient");

    assert_eq!(
        output.status.code(),
        Some(0),
        "auto-orient should exit 0; stderr: {}",
        stderr_str(&output)
    );

    // Verify output dimensions are rotated: 4×2 → 2×4.
    let decoded = image::open(&out_path).expect("output should be a decodable JPEG");
    assert_eq!(
        decoded.width(),
        2,
        "auto-orient rotate90: width should be 2"
    );
    assert_eq!(
        decoded.height(),
        4,
        "auto-orient rotate90: height should be 4"
    );

    // Run `info --json` on the output and assert has_exif:false.
    let info_output = Command::new(BIN)
        .args(["info", out_path.to_str().unwrap(), "--json"])
        .output()
        .expect("failed to run info");

    let info_stdout = stdout_str(&info_output);
    assert!(
        info_stdout.contains("\"has_exif\":false"),
        "output JPEG should have no EXIF after auto-orient; got: {info_stdout}"
    );
}

/// `auto-orient` on a plain PNG (no EXIF) exits 0 with unchanged dimensions.
#[test]
fn auto_orient_cli_noop_without_exif() {
    let dir = tempfile::tempdir().expect("tempdir");

    let png_bytes = common::solid_png(8, 4, [100, 150, 200]);
    let in_path = dir.path().join("in.png");
    std::fs::write(&in_path, &png_bytes).unwrap();

    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "auto-orient",
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run auto-orient noop");

    assert_eq!(
        output.status.code(),
        Some(0),
        "auto-orient on no-EXIF PNG should exit 0; stderr: {}",
        stderr_str(&output)
    );

    let decoded = image::open(&out_path).expect("output should be decodable");
    assert_eq!(decoded.width(), 8, "no-op auto-orient: width unchanged");
    assert_eq!(decoded.height(), 4, "no-op auto-orient: height unchanged");
}

/// Multi-input `auto-orient` with `--out-dir` rotates all inputs and writes
/// them as JPEG files to the output directory.
#[test]
fn auto_orient_cli_multi_input_fan_out() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out_dir = dir.path().join("out");
    std::fs::create_dir(&out_dir).unwrap();

    // Two 4×2 JPEGs with Orientation=6.
    let jpg_bytes = common::jpeg_with_orientation(4, 2, 6);
    let in_a = dir.path().join("a.jpg");
    let in_b = dir.path().join("b.jpg");
    std::fs::write(&in_a, &jpg_bytes).unwrap();
    std::fs::write(&in_b, &jpg_bytes).unwrap();

    let output = Command::new(BIN)
        .args([
            "auto-orient",
            in_a.to_str().unwrap(),
            in_b.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run auto-orient multi");

    assert_eq!(
        output.status.code(),
        Some(0),
        "auto-orient multi-input should exit 0; stderr: {}",
        stderr_str(&output)
    );

    // Both outputs should be 2×4 JPEG.
    for name in &["a.jpg", "b.jpg"] {
        let out_path = out_dir.join(name);
        assert!(out_path.exists(), "{name} should exist in out-dir");
        let decoded = image::open(&out_path).expect("output should be decodable");
        assert_eq!(
            decoded.width(),
            2,
            "{name}: width should be 2 after rotate90"
        );
        assert_eq!(
            decoded.height(),
            4,
            "{name}: height should be 4 after rotate90"
        );
    }
}

/// `auto-orient` with a missing input file exits 3.
#[test]
fn auto_orient_cli_missing_input_exits_3() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("nope.jpg");
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "auto-orient",
            missing.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run auto-orient with missing input");

    assert_eq!(
        output.status.code(),
        Some(3),
        "auto-orient with missing input should exit 3"
    );
}

/// `auto-orient` with multiple inputs but no `--out-dir` exits 2 and the
/// stderr mentions `out-dir`.
#[test]
fn auto_orient_cli_multi_without_out_dir_is_usage_error() {
    let dir = tempfile::tempdir().expect("tempdir");

    let jpg_bytes = common::jpeg_with_orientation(4, 2, 6);
    let in_a = dir.path().join("a.jpg");
    let in_b = dir.path().join("b.jpg");
    std::fs::write(&in_a, &jpg_bytes).unwrap();
    std::fs::write(&in_b, &jpg_bytes).unwrap();

    let output = Command::new(BIN)
        .args([
            "auto-orient",
            in_a.to_str().unwrap(),
            in_b.to_str().unwrap(),
            // No --out-dir
        ])
        .output()
        .expect("failed to run auto-orient without out-dir");

    assert_eq!(
        output.status.code(),
        Some(2),
        "auto-orient multi without --out-dir should exit 2"
    );
    let stderr = stderr_str(&output);
    assert!(
        stderr.to_ascii_lowercase().contains("out-dir"),
        "stderr should mention 'out-dir'; got: {stderr}"
    );
}

/// `apply` with a missing input file exits 3 (input not found).
#[test]
fn apply_missing_input_exits_3() {
    let dir = tempfile::tempdir().expect("tempdir");
    let recipe_path = write_recipe(
        &dir,
        "r.toml",
        "version = \"1\"\n[[step]]\nop = \"invert\"\n",
    );
    let out_path = dir.path().join("out.png");
    let missing = dir.path().join("nope.png");

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe_path.to_str().unwrap(),
            missing.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run apply");

    assert_eq!(
        output.status.code(),
        Some(3),
        "missing input should exit 3; stderr: {}",
        stderr_str(&output)
    );
}

// ── SPEC-008 view tests ───────────────────────────────────────────────────────

/// `view <png>` with a piped (non-tty) stdout exits 5 and reports a
/// terminal/tty requirement on stderr; stdout must be empty (no image bytes).
#[test]
fn view_non_tty_refuses_exit_5() {
    let dir = tempfile::tempdir().expect("tempdir");
    let png = write_test_png(&dir, "view_input.png", 4, 4);

    let output = Command::new(BIN)
        .args(["view", png.to_str().unwrap()])
        .output()
        .expect("failed to run view");

    assert_eq!(
        output.status.code(),
        Some(5),
        "view on non-tty should exit 5; stderr: {}",
        stderr_str(&output)
    );
    let stderr = stderr_str(&output).to_ascii_lowercase();
    assert!(
        stderr.contains("tty") || stderr.contains("terminal"),
        "stderr should mention tty or terminal requirement, got: {stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "stdout must be empty — no image bytes should leak on non-tty"
    );
}

/// `view <missing>` exits 3 (input not found).
#[test]
fn view_missing_input_exits_3() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("nope.png");

    let output = Command::new(BIN)
        .args(["view", missing.to_str().unwrap()])
        .output()
        .expect("failed to run view missing");

    assert_eq!(
        output.status.code(),
        Some(3),
        "view of missing file should exit 3; stderr: {}",
        stderr_str(&output)
    );
}

/// `view <dir>` resolves the first image in the directory and reaches the
/// non-tty refusal (exit 5). This pins the MVP "display the first resolved
/// input" decision and confirms no panic / usage error on a directory input.
#[test]
fn view_directory_uses_first_input() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Write exactly one PNG into the dir so source::resolve finds it.
    write_test_png(&dir, "only.png", 4, 4);

    let output = Command::new(BIN)
        .args(["view", dir.path().to_str().unwrap()])
        .output()
        .expect("failed to run view on directory");

    assert_eq!(
        output.status.code(),
        Some(5),
        "view on directory should resolve first image and exit 5 on non-tty; \
         stderr: {}",
        stderr_str(&output)
    );
}

/// `view --width 80 <png>` parses --width and, under non-tty, still exits 5.
/// Proves the flag is wired into the Sink without changing the tty-refusal behavior.
#[test]
fn view_width_flag_still_refuses_non_tty() {
    let dir = tempfile::tempdir().expect("tempdir");
    let png = write_test_png(&dir, "sized.png", 4, 4);

    let output = Command::new(BIN)
        .args(["view", "--width", "80", png.to_str().unwrap()])
        .output()
        .expect("failed to run view --width");

    assert_eq!(
        output.status.code(),
        Some(5),
        "view --width on non-tty should exit 5; stderr: {}",
        stderr_str(&output)
    );
    let stderr = stderr_str(&output).to_ascii_lowercase();
    assert!(
        stderr.contains("tty") || stderr.contains("terminal"),
        "stderr should mention tty or terminal requirement, got: {stderr}"
    );
}

/// `apply` with a recipe whose version is not "1" exits 1 (generic runtime error).
#[test]
fn apply_bad_recipe_version_exits_1() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 4, 4);
    let recipe_path = write_recipe(&dir, "bad.toml", "version = \"999\"\n");
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe_path.to_str().unwrap(),
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run apply");

    assert_eq!(
        output.status.code(),
        Some(1),
        "bad recipe version should exit 1; stderr: {}",
        stderr_str(&output)
    );
}

// ── SPEC-009 info tests ───────────────────────────────────────────────────────

/// `info <png>` exits 0 and reports core facts on stdout (human output).
///
/// AC1: dimensions (8x8), format label (png), color-type label (rgb8), and
/// both an `icc` and an `exif` presence line appear on stdout; stderr is empty.
#[test]
fn info_human_output_reports_core_facts() {
    let dir = tempfile::tempdir().expect("tempdir");
    let png = write_test_png(&dir, "info_human.png", 8, 8);

    let output = Command::new(BIN)
        .args(["info", png.to_str().unwrap()])
        .output()
        .expect("failed to run info");

    assert_eq!(
        output.status.code(),
        Some(0),
        "info should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let stdout = stdout_str(&output).to_ascii_lowercase();
    assert!(
        stdout.contains("8x8"),
        "stdout should contain '8x8': {stdout}"
    );
    assert!(
        stdout.contains("png"),
        "stdout should contain 'png': {stdout}"
    );
    assert!(
        stdout.contains("rgb8"),
        "stdout should contain 'rgb8': {stdout}"
    );
    assert!(
        stdout.contains("icc"),
        "stdout should contain 'icc': {stdout}"
    );
    assert!(
        stdout.contains("exif"),
        "stdout should contain 'exif': {stdout}"
    );
    assert!(
        output.stderr.is_empty(),
        "stderr must be empty on success, got: {}",
        stderr_str(&output)
    );
}

/// `info --json <png>` exits 0; stdout is valid JSON with all documented fields.
///
/// AC2: single JSON object; width/height/format/color_type/bit_depth/has_alpha/
/// has_icc/has_exif are correct; file_size_bytes > 0; decoded_bytes > 0;
/// the `exif` key is absent (no --exif); stderr is empty.
#[test]
fn info_json_is_parseable_and_complete() {
    let dir = tempfile::tempdir().expect("tempdir");
    let png = write_test_png(&dir, "info_json.png", 8, 8);

    let output = Command::new(BIN)
        .args(["info", "--json", png.to_str().unwrap()])
        .output()
        .expect("failed to run info --json");

    assert_eq!(
        output.status.code(),
        Some(0),
        "info --json should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(
        output.stderr.is_empty(),
        "stderr must be empty on --json success, got: {}",
        stderr_str(&output)
    );

    let obj: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout must parse as JSON");
    assert!(obj.is_object(), "JSON output must be an object");
    assert_eq!(obj["width"], 8u64, "width must be 8");
    assert_eq!(obj["height"], 8u64, "height must be 8");
    assert_eq!(obj["format"], "png", "format must be 'png'");
    assert_eq!(obj["color_type"], "rgb8", "color_type must be 'rgb8'");
    assert_eq!(obj["bit_depth"], 8u64, "bit_depth must be 8");
    assert_eq!(obj["has_alpha"], false, "has_alpha must be false");
    assert_eq!(obj["has_icc"], false, "has_icc must be false");
    assert_eq!(obj["has_exif"], false, "has_exif must be false");
    assert!(
        obj["file_size_bytes"].as_u64().unwrap_or(0) > 0,
        "file_size_bytes must be > 0"
    );
    assert!(
        obj["decoded_bytes"].as_u64().unwrap_or(0) > 0,
        "decoded_bytes must be > 0"
    );
    assert!(
        obj.get("exif").is_none(),
        "exif key must be absent when --exif not passed"
    );
}

/// `info --json --exif <plain png>` exits 0; `exif` is an empty array.
///
/// AC3: no EXIF in the PNG → empty array (not an error); has_exif is false.
#[test]
fn info_json_exif_empty_array_on_plain_png() {
    let dir = tempfile::tempdir().expect("tempdir");
    let png = write_test_png(&dir, "info_json_exif.png", 8, 8);

    let output = Command::new(BIN)
        .args(["info", "--json", "--exif", png.to_str().unwrap()])
        .output()
        .expect("failed to run info --json --exif");

    assert_eq!(
        output.status.code(),
        Some(0),
        "info --json --exif on plain PNG should exit 0; stderr: {}",
        stderr_str(&output)
    );

    let obj: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout must parse as JSON");
    assert!(
        obj["exif"].as_array().unwrap().is_empty(),
        "exif must be an empty array for a plain PNG"
    );
    assert_eq!(
        obj["has_exif"], false,
        "has_exif must be false for plain PNG"
    );
}

/// `info --exif <plain png>` (no --json) exits 0 and reports no EXIF gracefully.
///
/// AC5: stdout contains "exif" and indicates absence ("no" or "(none)").
#[test]
fn info_exif_on_plain_png_reports_none() {
    let dir = tempfile::tempdir().expect("tempdir");
    let png = write_test_png(&dir, "info_exif_none.png", 8, 8);

    let output = Command::new(BIN)
        .args(["info", "--exif", png.to_str().unwrap()])
        .output()
        .expect("failed to run info --exif on plain PNG");

    assert_eq!(
        output.status.code(),
        Some(0),
        "info --exif on plain PNG should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let stdout = stdout_str(&output).to_ascii_lowercase();
    assert!(
        stdout.contains("exif"),
        "stdout should contain 'exif': {stdout}"
    );
    // Must indicate absence: "exif: no" line OR "(none)" in the tag block.
    let has_no = stdout.contains("exif:       no") || stdout.contains("(none)");
    assert!(
        has_no,
        "stdout should indicate no EXIF ('no' on exif line or '(none)'): {stdout}"
    );
}

// ── SPEC-011 resize integration tests ────────────────────────────────────────

/// Generate a small gradient JPEG in memory and write it to `dir/name`.
/// Returns the full path. Mirrors `write_test_png` for JPEG fixtures.
fn write_test_jpeg(dir: &TempDir, name: &str, w: u32, h: u32) -> PathBuf {
    use image::RgbImage;

    // Simple horizontal gradient so the image is non-trivial.
    let img = RgbImage::from_fn(w, h, |x, _y| {
        image::Rgb([(x * 255 / w.max(1)) as u8, 100u8, 150u8])
    });
    let dyn_img = image::DynamicImage::ImageRgb8(img);
    let mut buf = std::io::Cursor::new(Vec::new());
    dyn_img.write_to(&mut buf, ImageFormat::Jpeg).unwrap();
    let path = dir.path().join(name);
    std::fs::write(&path, buf.into_inner()).unwrap();
    path
}

/// `resize <png> --max 20 -o out.png` exits 0; output decodes to 20×10
/// (long edge == 20, aspect preserved from a 100×50 source). (AC1)
#[test]
fn resize_max_single_input_writes_scaled() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 100, 50);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "resize",
            in_path.to_str().unwrap(),
            "--max",
            "20",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run resize --max");

    assert_eq!(
        output.status.code(),
        Some(0),
        "resize --max should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");
    let decoded = image::open(&out_path).expect("output should be decodable");
    // Long edge must be 20; short edge must be 10 (aspect preserved).
    assert_eq!(decoded.width(), 20, "width should be 20 (long edge)");
    assert_eq!(
        decoded.height(),
        10,
        "height should be 10 (aspect preserved)"
    );
}

/// `resize <png> --exact 33x77 -o out.png` exits 0; output is exactly 33×77. (AC2)
#[test]
fn resize_exact_single_input_exact_dims() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 100, 50);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "resize",
            in_path.to_str().unwrap(),
            "--exact",
            "33x77",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run resize --exact");

    assert_eq!(
        output.status.code(),
        Some(0),
        "resize --exact should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");
    let decoded = image::open(&out_path).expect("output should be decodable");
    assert_eq!(decoded.width(), 33, "width should be exactly 33");
    assert_eq!(decoded.height(), 77, "height should be exactly 77");
}

/// Multi-input `resize a.png b.jpg --max 20 --out-dir D` exits 0; each output
/// exists in D scaled to the expected dims AND preserves the source format
/// (a.png stays PNG, b.jpg stays JPEG). (AC3, DEC-015)
#[test]
fn resize_multi_input_fan_out_preserves_format() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out_dir = tempfile::tempdir().expect("out tempdir");

    let png_path = write_test_png(&dir, "a.png", 100, 50);
    let jpg_path = write_test_jpeg(&dir, "b.jpg", 100, 50);

    let output = Command::new(BIN)
        .args([
            "resize",
            png_path.to_str().unwrap(),
            jpg_path.to_str().unwrap(),
            "--max",
            "20",
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to run resize multi-input");

    assert_eq!(
        output.status.code(),
        Some(0),
        "resize multi-input should exit 0; stderr: {}",
        stderr_str(&output)
    );

    // a.png → out_dir/a.png, scaled 20×10, and must decode as PNG.
    let out_png = out_dir.path().join("a.png");
    assert!(out_png.exists(), "a.png output should exist in out-dir");
    let decoded_png = image::open(&out_png).expect("a.png output should be decodable");
    assert_eq!(decoded_png.width(), 20, "a.png output width should be 20");
    assert_eq!(decoded_png.height(), 10, "a.png output height should be 10");
    // Verify format is actually PNG by reading the magic bytes.
    let png_bytes = std::fs::read(&out_png).unwrap();
    assert_eq!(
        &png_bytes[..4],
        b"\x89PNG",
        "a.png output should be PNG format"
    );

    // b.jpg → out_dir/b.jpg, scaled 20×10, and must decode as JPEG.
    let out_jpg = out_dir.path().join("b.jpg");
    assert!(out_jpg.exists(), "b.jpg output should exist in out-dir");
    let decoded_jpg = image::open(&out_jpg).expect("b.jpg output should be decodable");
    assert_eq!(decoded_jpg.width(), 20, "b.jpg output width should be 20");
    assert_eq!(decoded_jpg.height(), 10, "b.jpg output height should be 10");
    // JPEG magic: starts with FF D8.
    let jpg_bytes = std::fs::read(&out_jpg).unwrap();
    assert_eq!(
        &jpg_bytes[..2],
        b"\xFF\xD8",
        "b.jpg output should be JPEG format"
    );
}

/// `resize <jpg> --max 20 --format png -o out.png` exits 0; output is PNG
/// (--format override wins over source JPEG format). (AC11)
#[test]
fn resize_format_override_changes_format() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_jpeg(&dir, "in.jpg", 100, 50);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "resize",
            in_path.to_str().unwrap(),
            "--max",
            "20",
            "--format",
            "png",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run resize --format png");

    assert_eq!(
        output.status.code(),
        Some(0),
        "resize --format png should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");
    // Verify format is PNG by magic bytes.
    let bytes = std::fs::read(&out_path).unwrap();
    assert_eq!(&bytes[..4], b"\x89PNG", "output should be PNG format");
}

/// `resize <png>` (no mode flag) → exit 2 (clap ArgGroup required). (AC4)
#[test]
fn resize_no_mode_is_usage_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 4, 4);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "resize",
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run resize no-mode");

    assert_eq!(
        output.status.code(),
        Some(2),
        "resize with no mode flag should exit 2"
    );
    assert!(
        !out_path.exists(),
        "output must not be created on usage error"
    );
}

/// `resize <png> --max 20 --exact 10x10` → exit 2 (two mode flags conflict). (AC5)
#[test]
fn resize_two_modes_is_usage_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 4, 4);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "resize",
            in_path.to_str().unwrap(),
            "--max",
            "20",
            "--exact",
            "10x10",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run resize two-modes");

    assert_eq!(
        output.status.code(),
        Some(2),
        "resize with two mode flags should exit 2"
    );
}

/// `resize <png> --exact abc` and `resize <png> --exact 800x` → exit 2
/// (malformed WxH string). (AC6)
#[test]
fn resize_bad_wxh_is_usage_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 4, 4);
    let out_path = dir.path().join("out.png");

    // --exact abc: no 'x' separator.
    let output = Command::new(BIN)
        .args([
            "resize",
            in_path.to_str().unwrap(),
            "--exact",
            "abc",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run resize --exact abc");

    assert_eq!(
        output.status.code(),
        Some(2),
        "resize --exact abc should exit 2"
    );

    // --exact 800x: missing height.
    let output2 = Command::new(BIN)
        .args([
            "resize",
            in_path.to_str().unwrap(),
            "--exact",
            "800x",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run resize --exact 800x");

    assert_eq!(
        output2.status.code(),
        Some(2),
        "resize --exact 800x should exit 2"
    );
}

/// `resize <missing.png> --max 20 -o out.png` → exit 3. (AC7)
#[test]
fn resize_missing_input_exits_3() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("missing.png");
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "resize",
            missing.to_str().unwrap(),
            "--max",
            "20",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run resize missing");

    assert_eq!(
        output.status.code(),
        Some(3),
        "resize of missing file should exit 3; stderr: {}",
        stderr_str(&output)
    );
}

/// Batch of one valid PNG + one `.png` file with garbage bytes → `--max 20
/// --out-dir D` → exit 6; the valid input's output IS written and decodes;
/// stderr mentions the failing file. (AC8)
#[test]
fn resize_partial_batch_exits_6() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out_dir = tempfile::tempdir().expect("out tempdir");

    let good_path = write_test_png(&dir, "good.png", 100, 50);
    // Write garbage bytes to a .png path (undecodable).
    let bad_path = dir.path().join("bad.png");
    std::fs::write(&bad_path, b"this is not an image at all").unwrap();

    let output = Command::new(BIN)
        .args([
            "resize",
            good_path.to_str().unwrap(),
            bad_path.to_str().unwrap(),
            "--max",
            "20",
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to run resize partial batch");

    assert_eq!(
        output.status.code(),
        Some(6),
        "partial batch should exit 6; stderr: {}",
        stderr_str(&output)
    );

    // The valid input's output must exist and decode.
    let good_out = out_dir.path().join("good.png");
    assert!(
        good_out.exists(),
        "valid input's output should still be written on partial batch failure"
    );
    let decoded = image::open(&good_out).expect("good output should be decodable");
    assert_eq!(decoded.width(), 20, "good output width should be 20");
    assert_eq!(decoded.height(), 10, "good output height should be 10");

    // stderr must mention the failing file.
    let stderr = stderr_str(&output);
    assert!(
        stderr.contains("bad.png"),
        "stderr should mention the failing file 'bad.png'; got: {stderr}"
    );
}

/// `resize <png> --max 20 -o -` exits 0; stdout is ONLY the encoded image bytes
/// (decodes to 20×10); stderr is empty. A known PNG source preserves PNG on `-o -`.
/// (AC9)
#[test]
fn resize_stdout_keeps_stdout_clean() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 100, 50);

    let output = Command::new(BIN)
        .args([
            "resize",
            in_path.to_str().unwrap(),
            "--max",
            "20",
            "-o",
            "-",
            "--format",
            "png",
        ])
        .output()
        .expect("failed to run resize stdout");

    assert_eq!(
        output.status.code(),
        Some(0),
        "resize -o - should exit 0; stderr: {}",
        stderr_str(&output)
    );
    // stdout must be ONLY the encoded image bytes — decodable.
    let decoded = image::load_from_memory(&output.stdout)
        .expect("stdout bytes should decode as a valid image");
    assert_eq!(decoded.width(), 20, "stdout image width should be 20");
    assert_eq!(decoded.height(), 10, "stdout image height should be 10");
    // stderr must be empty.
    assert!(
        output.stderr.is_empty(),
        "stderr must be empty on clean stdout run, got: {}",
        stderr_str(&output)
    );
}

/// Two PNG inputs with no `--out-dir` (and no `-o`) → exit 2 (usage error);
/// stderr mentions `--out-dir`. (AC10)
#[test]
fn resize_multi_without_out_dir_is_usage_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in1 = write_test_png(&dir, "a.png", 4, 4);
    let in2 = write_test_png(&dir, "b.png", 4, 4);

    let output = Command::new(BIN)
        .args([
            "resize",
            in1.to_str().unwrap(),
            in2.to_str().unwrap(),
            "--max",
            "2",
        ])
        .output()
        .expect("failed to run resize multi-no-out-dir");

    assert_eq!(
        output.status.code(),
        Some(2),
        "multi-input without --out-dir should exit 2; stderr: {}",
        stderr_str(&output)
    );
    let stderr = stderr_str(&output);
    assert!(
        stderr.contains("out-dir") || stderr.contains("out_dir"),
        "stderr should mention --out-dir; got: {stderr}"
    );
}

/// `info <missing>` exits 3 (input not found).
///
/// AC6: non-existent file → exit code 3.
#[test]
fn info_missing_input_exits_3() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("nope.png");

    let output = Command::new(BIN)
        .args(["info", missing.to_str().unwrap()])
        .output()
        .expect("failed to run info on missing file");

    assert_eq!(
        output.status.code(),
        Some(3),
        "info on missing file should exit 3; stderr: {}",
        stderr_str(&output)
    );
}

// ── SPEC-012 thumbnail integration tests ─────────────────────────────────────

/// `thumbnail <png>` (no `--size`) exits 0; the long edge == 256 (default),
/// aspect preserved. Source: 1000×500 → 256×128. (AC1)
#[test]
fn thumbnail_default_size_bounds_long_edge() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 1000, 500);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "thumbnail",
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run thumbnail default size");

    assert_eq!(
        output.status.code(),
        Some(0),
        "thumbnail (no --size) should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");
    let decoded = image::open(&out_path).expect("output should be decodable");
    assert_eq!(decoded.width(), 256, "long edge should be 256 (default)");
    assert_eq!(
        decoded.height(),
        128,
        "short edge should be 128 (aspect preserved)"
    );
}

/// `thumbnail <png> --size 64` exits 0; long edge == 64, aspect preserved.
/// Source: 100×50 → 64×32. (AC2)
#[test]
fn thumbnail_size_bounds_long_edge() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 100, 50);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "thumbnail",
            in_path.to_str().unwrap(),
            "--size",
            "64",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run thumbnail --size 64");

    assert_eq!(
        output.status.code(),
        Some(0),
        "thumbnail --size 64 should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");
    let decoded = image::open(&out_path).expect("output should be decodable");
    assert_eq!(decoded.width(), 64, "long edge should be 64");
    assert_eq!(
        decoded.height(),
        32,
        "short edge should be 32 (aspect preserved)"
    );
}

/// `thumbnail <png> --size 64 --square` exits 0; output is exactly 64×64
/// (cover + center-crop). Source: 100×50. (AC3)
#[test]
fn thumbnail_square_is_exact_square() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 100, 50);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "thumbnail",
            in_path.to_str().unwrap(),
            "--size",
            "64",
            "--square",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run thumbnail --square");

    assert_eq!(
        output.status.code(),
        Some(0),
        "thumbnail --size 64 --square should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");
    let decoded = image::open(&out_path).expect("output should be decodable");
    assert_eq!(decoded.width(), 64, "square output must be exactly 64 wide");
    assert_eq!(
        decoded.height(),
        64,
        "square output must be exactly 64 tall"
    );
}

/// `thumbnail` does NOT upscale: a 40×30 source with `--size 64` stays 40×30. (AC4)
#[test]
fn thumbnail_does_not_upscale() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 40, 30);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "thumbnail",
            in_path.to_str().unwrap(),
            "--size",
            "64",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run thumbnail no-upscale");

    assert_eq!(
        output.status.code(),
        Some(0),
        "thumbnail (no upscale) should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");
    let decoded = image::open(&out_path).expect("output should be decodable");
    assert_eq!(decoded.width(), 40, "width must stay 40 (no upscale)");
    assert_eq!(decoded.height(), 30, "height must stay 30 (no upscale)");
}

/// Multi-input fan-out: `thumbnail a.png b.jpg --size 64 --out-dir D` exits 0;
/// a.png stays PNG, b.jpg stays JPEG (format preserved, DEC-015). (AC5)
#[test]
fn thumbnail_multi_input_fan_out_preserves_format() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out_dir = tempfile::tempdir().expect("out tempdir");

    let png_path = write_test_png(&dir, "a.png", 100, 50);
    let jpg_path = write_test_jpeg(&dir, "b.jpg", 100, 50);

    let output = Command::new(BIN)
        .args([
            "thumbnail",
            png_path.to_str().unwrap(),
            jpg_path.to_str().unwrap(),
            "--size",
            "64",
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to run thumbnail multi-input");

    assert_eq!(
        output.status.code(),
        Some(0),
        "thumbnail multi-input should exit 0; stderr: {}",
        stderr_str(&output)
    );

    // a.png → out_dir/a.png, scaled 64×32, must be PNG.
    let out_png = out_dir.path().join("a.png");
    assert!(out_png.exists(), "a.png output should exist in out-dir");
    let decoded_png = image::open(&out_png).expect("a.png output should be decodable");
    assert_eq!(decoded_png.width(), 64, "a.png output width should be 64");
    assert_eq!(decoded_png.height(), 32, "a.png output height should be 32");
    let png_bytes = std::fs::read(&out_png).unwrap();
    assert_eq!(
        &png_bytes[..4],
        b"\x89PNG",
        "a.png output should be PNG format"
    );

    // b.jpg → out_dir/b.jpg, scaled 64×32, must be JPEG.
    let out_jpg = out_dir.path().join("b.jpg");
    assert!(out_jpg.exists(), "b.jpg output should exist in out-dir");
    let decoded_jpg = image::open(&out_jpg).expect("b.jpg output should be decodable");
    assert_eq!(decoded_jpg.width(), 64, "b.jpg output width should be 64");
    assert_eq!(decoded_jpg.height(), 32, "b.jpg output height should be 32");
    let jpg_bytes = std::fs::read(&out_jpg).unwrap();
    assert_eq!(
        &jpg_bytes[..2],
        b"\xFF\xD8",
        "b.jpg output should be JPEG format"
    );
}

/// `thumbnail <missing.png> --size 64 -o out` → exit 3. (AC6)
#[test]
fn thumbnail_missing_input_exits_3() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("missing.png");
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "thumbnail",
            missing.to_str().unwrap(),
            "--size",
            "64",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run thumbnail missing");

    assert_eq!(
        output.status.code(),
        Some(3),
        "thumbnail of missing file should exit 3; stderr: {}",
        stderr_str(&output)
    );
    assert!(!out_path.exists(), "no output should be created");
}

/// Two PNG inputs with no `--out-dir` → exit 2; stderr mentions `--out-dir`. (AC7)
#[test]
fn thumbnail_multi_without_out_dir_is_usage_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in1 = write_test_png(&dir, "a.png", 4, 4);
    let in2 = write_test_png(&dir, "b.png", 4, 4);

    let output = Command::new(BIN)
        .args([
            "thumbnail",
            in1.to_str().unwrap(),
            in2.to_str().unwrap(),
            "--size",
            "2",
        ])
        .output()
        .expect("failed to run thumbnail multi-no-out-dir");

    assert_eq!(
        output.status.code(),
        Some(2),
        "thumbnail multi without --out-dir should exit 2; stderr: {}",
        stderr_str(&output)
    );
    let stderr = stderr_str(&output);
    assert!(
        stderr.contains("out-dir") || stderr.contains("out_dir"),
        "stderr should mention --out-dir; got: {stderr}"
    );
}

/// `thumbnail <png> --size 64 -o -` exits 0; stdout is ONLY encoded image
/// bytes (decodes, long edge == 64); stderr is empty. (AC8)
#[test]
fn thumbnail_stdout_keeps_stdout_clean() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 100, 50);

    let output = Command::new(BIN)
        .args([
            "thumbnail",
            in_path.to_str().unwrap(),
            "--size",
            "64",
            "-o",
            "-",
            "--format",
            "png",
        ])
        .output()
        .expect("failed to run thumbnail stdout");

    assert_eq!(
        output.status.code(),
        Some(0),
        "thumbnail -o - should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let decoded = image::load_from_memory(&output.stdout)
        .expect("stdout bytes should decode as a valid image");
    assert_eq!(decoded.width(), 64, "stdout image width should be 64");
    assert_eq!(decoded.height(), 32, "stdout image height should be 32");
    assert!(
        output.stderr.is_empty(),
        "stderr must be empty on clean stdout run, got: {}",
        stderr_str(&output)
    );
}

/// Partial batch: one valid PNG + one garbage-bytes `.png` → `--size 64
/// --out-dir D` → exit 6; valid output written; stderr names the failure. (AC9)
#[test]
fn thumbnail_partial_batch_exits_6() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out_dir = tempfile::tempdir().expect("out tempdir");

    let good_path = write_test_png(&dir, "good.png", 100, 50);
    let bad_path = dir.path().join("bad.png");
    std::fs::write(&bad_path, b"this is not an image at all").unwrap();

    let output = Command::new(BIN)
        .args([
            "thumbnail",
            good_path.to_str().unwrap(),
            bad_path.to_str().unwrap(),
            "--size",
            "64",
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to run thumbnail partial batch");

    assert_eq!(
        output.status.code(),
        Some(6),
        "partial batch should exit 6; stderr: {}",
        stderr_str(&output)
    );

    // Valid input's output must exist and decode.
    let good_out = out_dir.path().join("good.png");
    assert!(
        good_out.exists(),
        "valid input's output should still be written on partial batch failure"
    );
    let decoded = image::open(&good_out).expect("good output should be decodable");
    assert_eq!(decoded.width(), 64, "good output width should be 64");
    assert_eq!(decoded.height(), 32, "good output height should be 32");

    let stderr = stderr_str(&output);
    assert!(
        stderr.contains("bad.png"),
        "stderr should mention the failing file 'bad.png'; got: {stderr}"
    );
}

/// `thumbnail <png> --size 0` → exit 2 (op rejects width 0 → `Usage`). (AC10)
#[test]
fn thumbnail_size_zero_is_usage_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 100, 50);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "thumbnail",
            in_path.to_str().unwrap(),
            "--size",
            "0",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run thumbnail --size 0");

    assert_eq!(
        output.status.code(),
        Some(2),
        "thumbnail --size 0 should exit 2; stderr: {}",
        stderr_str(&output)
    );
    assert!(
        !out_path.exists(),
        "no output should be created on usage error"
    );
}

// ── SPEC-013 shrink integration tests ─────────────────────────────────────────

/// `shrink <jpg>` (no flags) resizes long edge to DEFAULT_SHRINK_MAX (1600) and
/// writes a file smaller than the original.
///
/// Source: 2000×1000 JPEG → long edge == 1600, smaller file. (AC1)
#[test]
fn shrink_defaults_bound_long_edge_and_shrink_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_jpeg(&dir, "in.jpg", 2000, 1000);
    let out_path = dir.path().join("out.jpg");

    let in_size = std::fs::metadata(&in_path).unwrap().len();

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink");

    assert_eq!(
        output.status.code(),
        Some(0),
        "shrink (no flags) should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");

    let decoded = image::open(&out_path).expect("output should be decodable");
    // Long edge (width) must be exactly 1600.
    assert_eq!(
        decoded.width(),
        1600,
        "long edge should be 1600 (DEFAULT_SHRINK_MAX)"
    );
    assert_eq!(
        decoded.height(),
        800,
        "short edge should be 800 (aspect preserved)"
    );

    let out_size = std::fs::metadata(&out_path).unwrap().len();
    assert!(
        out_size < in_size,
        "output ({out_size} bytes) should be smaller than input ({in_size} bytes)"
    );
}

/// `shrink <jpg> --max 100` on a 200×100 JPEG → long edge == 100, short edge == 50. (AC2)
#[test]
fn shrink_max_bounds_long_edge() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_jpeg(&dir, "in.jpg", 200, 100);
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--max",
            "100",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink --max");

    assert_eq!(
        output.status.code(),
        Some(0),
        "shrink --max 100 should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");

    let decoded = image::open(&out_path).expect("output should be decodable");
    assert_eq!(decoded.width(), 100, "long edge should be 100");
    assert_eq!(
        decoded.height(),
        50,
        "short edge should be 50 (aspect preserved)"
    );
}

/// Same JPEG at `-q 20` vs `-q 90`: low-quality output smaller; same dims. (AC3)
#[test]
fn shrink_quality_lower_is_smaller() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_jpeg(&dir, "in.jpg", 400, 200);
    let out_lo = dir.path().join("lo.jpg");
    let out_hi = dir.path().join("hi.jpg");

    // Low quality.
    let lo_out = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--max",
            "200",
            "-q",
            "20",
            "-o",
            out_lo.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink -q 20");

    assert_eq!(
        lo_out.status.code(),
        Some(0),
        "shrink -q 20 should exit 0; stderr: {}",
        stderr_str(&lo_out)
    );

    // High quality.
    let hi_out = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--max",
            "200",
            "-q",
            "90",
            "-o",
            out_hi.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink -q 90");

    assert_eq!(
        hi_out.status.code(),
        Some(0),
        "shrink -q 90 should exit 0; stderr: {}",
        stderr_str(&hi_out)
    );

    let lo_size = std::fs::metadata(&out_lo).unwrap().len();
    let hi_size = std::fs::metadata(&out_hi).unwrap().len();
    assert!(
        lo_size < hi_size,
        "low quality ({lo_size} bytes) should be smaller than high quality ({hi_size} bytes)"
    );

    // Both must decode at the same dimensions.
    let lo_img = image::open(&out_lo).expect("lo output should be decodable");
    let hi_img = image::open(&out_hi).expect("hi output should be decodable");
    assert_eq!(lo_img.width(), hi_img.width(), "widths must match");
    assert_eq!(lo_img.height(), hi_img.height(), "heights must match");
}

/// `shrink <png> --max 100 -q 10` → PNG output, exit 0, quality ignored. (AC4)
#[test]
fn shrink_png_preserves_format_quality_ignored() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 200, 100);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--max",
            "100",
            "-q",
            "10",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink on PNG");

    assert_eq!(
        output.status.code(),
        Some(0),
        "shrink on PNG should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");

    // Output must be PNG (magic bytes).
    let bytes = std::fs::read(&out_path).unwrap();
    assert_eq!(&bytes[..4], b"\x89PNG", "output should be PNG format");

    let decoded = image::open(&out_path).expect("output should be decodable");
    assert_eq!(decoded.width(), 100, "long edge should be 100");
}

/// `shrink a.png b.jpg --max 64 --out-dir D` → exit 0; each keeps source format. (AC5)
#[test]
fn shrink_multi_input_fan_out_preserves_format() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out_dir = tempfile::tempdir().expect("out tempdir");

    let png_path = write_test_png(&dir, "a.png", 200, 100);
    let jpg_path = write_test_jpeg(&dir, "b.jpg", 200, 100);

    let output = Command::new(BIN)
        .args([
            "shrink",
            png_path.to_str().unwrap(),
            jpg_path.to_str().unwrap(),
            "--max",
            "64",
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink multi-input");

    assert_eq!(
        output.status.code(),
        Some(0),
        "shrink multi-input should exit 0; stderr: {}",
        stderr_str(&output)
    );

    // a.png → out_dir/a.png, scaled to ≤ 64 on long edge, must be PNG.
    let out_png = out_dir.path().join("a.png");
    assert!(out_png.exists(), "a.png output should exist in out-dir");
    let png_bytes = std::fs::read(&out_png).unwrap();
    assert_eq!(
        &png_bytes[..4],
        b"\x89PNG",
        "a.png output should be PNG format"
    );
    let decoded_png = image::open(&out_png).expect("a.png output should be decodable");
    assert!(decoded_png.width() <= 64, "a.png long edge should be ≤ 64");

    // b.jpg → out_dir/b.jpg, scaled to ≤ 64 on long edge, must be JPEG.
    let out_jpg = out_dir.path().join("b.jpg");
    assert!(out_jpg.exists(), "b.jpg output should exist in out-dir");
    let jpg_bytes = std::fs::read(&out_jpg).unwrap();
    assert_eq!(
        &jpg_bytes[..2],
        b"\xFF\xD8",
        "b.jpg output should be JPEG format"
    );
    let decoded_jpg = image::open(&out_jpg).expect("b.jpg output should be decodable");
    assert!(decoded_jpg.width() <= 64, "b.jpg long edge should be ≤ 64");
}

/// `shrink <missing>` → exit 3. (AC6)
#[test]
fn shrink_missing_input_exits_3() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("nope.jpg");

    let output = Command::new(BIN)
        .args(["shrink", missing.to_str().unwrap()])
        .output()
        .expect("failed to run shrink missing");

    assert_eq!(
        output.status.code(),
        Some(3),
        "shrink of missing file should exit 3; stderr: {}",
        stderr_str(&output)
    );
}

/// Two inputs with no `--out-dir` → exit 2; stderr mentions `--out-dir`. (AC7)
#[test]
fn shrink_multi_without_out_dir_is_usage_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in1 = write_test_jpeg(&dir, "a.jpg", 4, 4);
    let in2 = write_test_jpeg(&dir, "b.jpg", 4, 4);

    let output = Command::new(BIN)
        .args(["shrink", in1.to_str().unwrap(), in2.to_str().unwrap()])
        .output()
        .expect("failed to run shrink multi-no-out-dir");

    assert_eq!(
        output.status.code(),
        Some(2),
        "shrink multi without --out-dir should exit 2; stderr: {}",
        stderr_str(&output)
    );
    let stderr = stderr_str(&output);
    assert!(
        stderr.contains("out-dir") || stderr.contains("out_dir"),
        "stderr should mention --out-dir; got: {stderr}"
    );
}

/// `shrink <jpg> --max 64 -o -` → exit 0; stdout is the encoded image; stderr empty. (AC8)
#[test]
fn shrink_stdout_keeps_stdout_clean() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_jpeg(&dir, "in.jpg", 200, 100);

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--max",
            "64",
            "-o",
            "-",
        ])
        .output()
        .expect("failed to run shrink stdout");

    assert_eq!(
        output.status.code(),
        Some(0),
        "shrink -o - should exit 0; stderr: {}",
        stderr_str(&output)
    );

    // stdout must be ONLY the encoded image bytes — decodable.
    let decoded = image::load_from_memory(&output.stdout)
        .expect("stdout bytes should decode as a valid image");
    assert!(
        decoded.width() <= 64,
        "stdout image width should be ≤ 64, got {}",
        decoded.width()
    );

    // stderr must be empty.
    assert!(
        output.stderr.is_empty(),
        "stderr must be empty on clean stdout run, got: {}",
        stderr_str(&output)
    );
}

// ── SPEC-014 convert integration tests ───────────────────────────────────────

/// `convert <png> --format jpg -o out.jpg` → exit 0; output is JPEG (FF D8 magic),
/// decodes, and dims are preserved. (AC1)
#[test]
fn convert_png_to_jpeg_changes_format() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 40, 30);
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "jpg",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert png→jpg");

    assert_eq!(
        output.status.code(),
        Some(0),
        "convert png→jpg should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");
    let bytes = std::fs::read(&out_path).unwrap();
    assert_eq!(
        &bytes[..2],
        b"\xFF\xD8",
        "output should be JPEG (FF D8 magic)"
    );
    let decoded = image::load_from_memory(&bytes).expect("output should be decodable");
    assert_eq!(decoded.width(), 40, "width must be preserved (40)");
    assert_eq!(decoded.height(), 30, "height must be preserved (30)");
}

/// `convert <jpg> --format png -o out.png` → exit 0; output is PNG, decodes at
/// original dims. (AC2)
#[test]
fn convert_jpeg_to_png_changes_format() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_jpeg(&dir, "in.jpg", 32, 16);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "png",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert jpg→png");

    assert_eq!(
        output.status.code(),
        Some(0),
        "convert jpg→png should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");
    let bytes = std::fs::read(&out_path).unwrap();
    assert_eq!(
        &bytes[..8],
        b"\x89PNG\r\n\x1a\n",
        "output should be PNG (8-byte PNG signature)"
    );
    let decoded = image::load_from_memory(&bytes).expect("output should be decodable");
    assert_eq!(decoded.width(), 32, "width must be 32");
    assert_eq!(decoded.height(), 16, "height must be 16");
}

/// `convert <png> --format gif -o out.png` → exit 0; output is actually GIF
/// (`GIF8` magic) even though the `-o` extension is `.png`. Forced `--format`
/// overrides the output extension. (AC3)
#[test]
fn convert_format_overrides_output_extension() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 8, 8);
    // Intentionally use a `.png` output path but request GIF format.
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "gif",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --format gif -o out.png");

    assert_eq!(
        output.status.code(),
        Some(0),
        "convert --format gif should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should exist");
    let bytes = std::fs::read(&out_path).unwrap();
    // GIF magic: "GIF8" (either GIF87a or GIF89a).
    assert_eq!(
        &bytes[..4],
        b"GIF8",
        "--format gif must win over .png extension (output must be GIF)"
    );
}

/// `convert <png> --format avif` → exit 4 (codec not built, DEC-004) without the
/// feature. (WebP is no longer an unbuilt codec — it is a pure-Rust default since
/// SPEC-019; see `convert_to_webp_produces_lossless_webp`.) (AC4)
#[test]
fn convert_unbuilt_codec_exits_4() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 4, 4);
    let out_path = dir.path().join("out.avif");

    // AVIF — not built.
    let avif_out = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "avif",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --format avif");

    // AVIF exits 4 only when the codec is NOT built; with `--features avif` the
    // same convert succeeds (exit 0). The dedicated cfg(avif) tests below assert
    // the success path's AVIF output.
    let expected_avif = if cfg!(feature = "avif") { 0 } else { 4 };
    assert_eq!(
        avif_out.status.code(),
        Some(expected_avif),
        "convert --format avif expected exit {expected_avif}; stderr: {}",
        stderr_str(&avif_out)
    );
}

/// Multi-input convert to an unbuilt codec → exit 4 (NOT 6), because the target
/// codec is resolved up front before the per-input fan-out. (AC5)
#[test]
fn convert_unbuilt_codec_multi_input_exits_4_not_6() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out_dir = tempfile::tempdir().expect("out tempdir");

    let a = write_test_png(&dir, "a.png", 4, 4);
    let b = write_test_png(&dir, "b.png", 4, 4);

    let output = Command::new(BIN)
        .args([
            "convert",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--format",
            "avif",
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert multi-input --format avif");

    // Without the feature: exit 4 (codec failure resolved UP FRONT via
    // ensure_codec_built), NOT 6 (partial batch). With `--features avif`: both
    // inputs convert successfully → exit 0. Either way it must NEVER be 6.
    let expected = if cfg!(feature = "avif") { 0 } else { 4 };
    assert_eq!(
        output.status.code(),
        Some(expected),
        "multi-input convert to avif expected exit {expected} (never partial-batch 6); stderr: {}",
        stderr_str(&output)
    );
}

// ── SPEC-019: WebP (lossless output + decode) ───────────────────────────────────

/// `convert <png> --format webp -o out.webp` → exit 0; output magic-detects as
/// WebP AND round-trips losslessly (decoded pixels exactly equal the source).
#[test]
fn convert_to_webp_produces_lossless_webp() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Solid-color source so a lossless round-trip is bit-exact.
    let in_path = write_test_png(&dir, "in.png", 8, 6);
    let out_path = dir.path().join("out.webp");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "webp",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --format webp");

    assert_eq!(
        output.status.code(),
        Some(0),
        "convert --format webp should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("WebP output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::WebP,
        "output should be WebP"
    );
    // Lossless: decode the output and confirm dims + exact pixels vs the source.
    let decoded = image::load_from_memory(&bytes)
        .expect("decode webp")
        .to_rgb8();
    assert_eq!(decoded.dimensions(), (8, 6), "dims must be preserved");
    let src = image::open(&in_path).expect("open source").to_rgb8();
    assert_eq!(
        decoded.as_raw(),
        src.as_raw(),
        "lossless WebP must round-trip pixels exactly"
    );
}

/// `.webp` is a readable INPUT: convert a lossless-WebP fixture to PNG → exit 0;
/// output is PNG at the source dimensions. (Proves WebP decode works by default.)
#[test]
fn webp_input_decodes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.webp");
    std::fs::write(&in_path, common::webp_lossless(10, 7)).unwrap();
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "png",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert <webp> --format png");

    assert_eq!(
        output.status.code(),
        Some(0),
        "convert from .webp input should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("PNG output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Png,
        "output should be PNG"
    );
    let decoded = image::load_from_memory(&bytes).expect("decode png");
    assert_eq!(
        (decoded.width(), decoded.height()),
        (10, 7),
        "dims from the .webp input must be preserved"
    );
}

/// `shrink <jpg> -o out.webp` → exit 0; output is WebP (lossless; the `-o`
/// extension drives the format, DEC-015).
#[test]
fn shrink_to_webp_output() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.jpg");
    std::fs::write(&in_path, common::detailed_jpeg(48, 32)).unwrap();
    let out_path = dir.path().join("out.webp");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink -o out.webp");

    assert_eq!(
        output.status.code(),
        Some(0),
        "shrink to .webp should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("WebP output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::WebP,
        "output should be WebP"
    );
}

/// `-q` is ignored for (lossless) WebP output, like PNG (DEC-016): the command
/// still succeeds and produces WebP — the quality value has no effect.
#[test]
fn webp_quality_is_ignored() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 8, 8);
    let out_path = dir.path().join("out.webp");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "webp",
            "-q",
            "50",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --format webp -q 50");

    assert_eq!(
        output.status.code(),
        Some(0),
        "-q on lossless WebP should be ignored, not an error; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("WebP output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::WebP,
        "output should be WebP"
    );
}

// ── SPEC-020: lossy WebP (feature-gated webp-lossy) ─────────────────────────────

/// FEATURE build: `convert <detailed png> --format webp -q 20` is LOSSY and
/// smaller than the same source as lossless WebP (`convert --format webp`, no -q).
#[cfg(feature = "webp-lossy")]
#[test]
fn convert_to_lossy_webp_is_smaller() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.png");
    std::fs::write(&in_path, common::detailed_png(96, 96)).unwrap();
    let lossy_path = dir.path().join("lossy.webp");
    let lossless_path = dir.path().join("lossless.webp");

    // Lossy (with -q).
    let lossy = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "webp",
            "-q",
            "20",
            "-o",
            lossy_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --format webp -q 20");
    assert_eq!(
        lossy.status.code(),
        Some(0),
        "lossy convert should exit 0; stderr: {}",
        stderr_str(&lossy)
    );

    // Lossless (no -q) — the SPEC-019 default path.
    let lossless = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "webp",
            "-o",
            lossless_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --format webp (lossless)");
    assert_eq!(
        lossless.status.code(),
        Some(0),
        "lossless convert should exit 0"
    );

    let lossy_bytes = std::fs::read(&lossy_path).unwrap();
    let lossless_bytes = std::fs::read(&lossless_path).unwrap();
    assert_eq!(
        image::guess_format(&lossy_bytes).expect("guess format"),
        ImageFormat::WebP,
        "lossy output should be WebP"
    );
    assert_eq!(
        image::guess_format(&lossless_bytes).expect("guess format"),
        ImageFormat::WebP,
        "lossless output should be WebP"
    );
    assert!(
        lossy_bytes.len() < lossless_bytes.len(),
        "lossy q20 ({}) should be smaller than lossless ({})",
        lossy_bytes.len(),
        lossless_bytes.len()
    );
}

/// FEATURE build: the PERCEPTUAL search drives WebP — `shrink --target high -o
/// out.webp` → exit 0, valid WebP, and (unlike AVIF) NO "needs a decoder" warning.
#[cfg(feature = "webp-lossy")]
#[test]
fn webp_target_high() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.png");
    std::fs::write(&in_path, common::detailed_png(64, 64)).unwrap();
    let out_path = dir.path().join("out.webp");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--target",
            "high",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink --target high -o out.webp");

    assert_eq!(
        output.status.code(),
        Some(0),
        "perceptual WebP should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("WebP output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::WebP,
        "output should be WebP"
    );
    let stderr = stderr_str(&output);
    assert!(
        !stderr.contains("decoder"),
        "perceptual WebP must NOT warn about a missing decoder (the AVIF contrast); got: {stderr}"
    );
}

/// FEATURE build: `convert <detailed png> --format webp --max-size 4KB` → exit 0;
/// the byte budget drives the lossy WebP quality and the output fits (≤ 4000).
#[cfg(feature = "webp-lossy")]
#[test]
fn webp_max_size_fits() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.png");
    std::fs::write(&in_path, common::detailed_png(96, 96)).unwrap();
    let out_path = dir.path().join("out.webp");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "webp",
            "--max-size",
            "4KB",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --format webp --max-size 4KB");

    assert_eq!(
        output.status.code(),
        Some(0),
        "convert --format webp --max-size 4KB should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("WebP output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::WebP,
        "output should be WebP"
    );
    assert!(
        bytes.len() <= 4000,
        "WebP output should fit the 4KB budget, got {} bytes",
        bytes.len()
    );
}

// ── SPEC-018: AVIF output ──────────────────────────────────────────────────────

/// DEFAULT build: `convert <png> --format avif -o out.avif` → exit 4 with a
/// "rebuild with --features avif" hint (codec recognized but not built, DEC-004).
/// Gated `not(feature = "avif")`: in the feature build the same convert succeeds,
/// which the cfg(avif) tests below cover.
#[cfg(not(feature = "avif"))]
#[test]
fn convert_avif_without_feature_exits_4() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 8, 8);
    let out_path = dir.path().join("out.avif");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "avif",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --format avif");

    assert_eq!(
        output.status.code(),
        Some(4),
        "convert --format avif (no feature) should exit 4; stderr: {}",
        stderr_str(&output)
    );
    let stderr = stderr_str(&output);
    assert!(
        stderr.contains("avif"),
        "stderr should mention the avif codec; got: {stderr}"
    );
    assert!(
        stderr.contains("--features avif"),
        "stderr should hint `--features avif`; got: {stderr}"
    );
    assert!(
        !out_path.exists(),
        "no AVIF file should be written on exit 4"
    );
}

/// FEATURE build: `convert <png> --format avif -o out.avif` → exit 0; the output
/// magic-detects as AVIF (decode is NOT built, so we use `guess_format`).
#[cfg(feature = "avif")]
#[test]
fn convert_to_avif_produces_avif() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.png");
    std::fs::write(&in_path, common::detailed_png(64, 64)).unwrap();
    let out_path = dir.path().join("out.avif");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "avif",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --format avif");

    assert_eq!(
        output.status.code(),
        Some(0),
        "convert --format avif (feature) should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("AVIF output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Avif,
        "output should be AVIF"
    );
}

/// FEATURE build: `shrink <jpg> -o out.avif -q 50` → exit 0; output is AVIF.
#[cfg(feature = "avif")]
#[test]
fn shrink_to_avif_output() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.jpg");
    std::fs::write(&in_path, common::detailed_jpeg(64, 64)).unwrap();
    let out_path = dir.path().join("out.avif");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "-q",
            "50",
        ])
        .output()
        .expect("failed to run shrink -o out.avif");

    assert_eq!(
        output.status.code(),
        Some(0),
        "shrink to .avif (feature) should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("AVIF output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Avif,
        "output should be AVIF"
    );
}

/// FEATURE build: a perceptual target (`--target`) on AVIF degrades GRACEFULLY.
/// The SSIMULACRA2 search must DECODE each candidate to score it, but AVIF decode
/// is not built (output-only v1, DEC-020) — so the run still succeeds (exit 0),
/// writes valid AVIF at the encoder default, and warns that perceptual targeting
/// needs a decoder. (Byte-budget AVIF, which is encode-only, works — see
/// `avif_max_size_fits`.) Uses `shrink` because `--target`/`--ssim` are
/// shrink-only flags (SPEC-016); `convert` carries only `--max-size`.
#[cfg(feature = "avif")]
#[test]
fn avif_target_high() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.png");
    std::fs::write(&in_path, common::detailed_png(64, 64)).unwrap();
    let out_path = dir.path().join("out.avif");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--target",
            "high",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink --target high -o out.avif");

    assert_eq!(
        output.status.code(),
        Some(0),
        "shrink --target high to .avif (feature) should exit 0 (graceful fallback); stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("AVIF output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Avif,
        "output should be AVIF (written at encoder default)"
    );
    let stderr = stderr_str(&output);
    assert!(
        stderr.contains("decoder") || stderr.contains("--max-size"),
        "stderr should warn that perceptual AVIF needs a decoder; got: {stderr}"
    );
}

/// FEATURE build: `convert <detailed png> --format avif --max-size 4KB` → exit 0;
/// the byte budget drives the AVIF quality and the output fits (≤ 4000 bytes).
#[cfg(feature = "avif")]
#[test]
fn avif_max_size_fits() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.png");
    std::fs::write(&in_path, common::detailed_png(96, 96)).unwrap();
    let out_path = dir.path().join("out.avif");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "avif",
            "--max-size",
            "4KB",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --format avif --max-size 4KB");

    assert_eq!(
        output.status.code(),
        Some(0),
        "convert --format avif --max-size 4KB should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("AVIF output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Avif,
        "output should be AVIF"
    );
    assert!(
        bytes.len() <= 4000,
        "AVIF output should fit the 4KB budget, got {} bytes",
        bytes.len()
    );
}

/// `convert a.png b.png --format jpg --out-dir D` → exit 0; D/a.jpg and D/b.jpg
/// both exist and are JPEG. (AC6)
#[test]
fn convert_multi_input_fan_out() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out_dir = tempfile::tempdir().expect("out tempdir");

    let a = write_test_png(&dir, "a.png", 20, 10);
    let b = write_test_png(&dir, "b.png", 20, 10);

    let output = Command::new(BIN)
        .args([
            "convert",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--format",
            "jpg",
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert multi-input fan-out");

    assert_eq!(
        output.status.code(),
        Some(0),
        "convert multi-input should exit 0; stderr: {}",
        stderr_str(&output)
    );

    // D/a.jpg must exist and be JPEG.
    let out_a = out_dir.path().join("a.jpg");
    assert!(out_a.exists(), "D/a.jpg should exist");
    let a_bytes = std::fs::read(&out_a).unwrap();
    assert_eq!(&a_bytes[..2], b"\xFF\xD8", "D/a.jpg should be JPEG format");

    // D/b.jpg must exist and be JPEG.
    let out_b = out_dir.path().join("b.jpg");
    assert!(out_b.exists(), "D/b.jpg should exist");
    let b_bytes = std::fs::read(&out_b).unwrap();
    assert_eq!(&b_bytes[..2], b"\xFF\xD8", "D/b.jpg should be JPEG format");
}

/// Same gradient source at `-q 20` vs `-q 90`: low-quality JPEG output is smaller;
/// both decode to the same dims. Uses a gradient source so quality affects size. (AC7)
#[test]
fn convert_quality_lower_is_smaller() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Use the gradient JPEG fixture (multi-color detail so quality matters).
    let in_path = write_test_jpeg(&dir, "in.jpg", 200, 100);
    let out_lo = dir.path().join("lo.jpg");
    let out_hi = dir.path().join("hi.jpg");

    // Low quality.
    let lo_out = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "jpg",
            "-q",
            "20",
            "-o",
            out_lo.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert -q 20");

    assert_eq!(
        lo_out.status.code(),
        Some(0),
        "convert -q 20 should exit 0; stderr: {}",
        stderr_str(&lo_out)
    );

    // High quality.
    let hi_out = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "jpg",
            "-q",
            "90",
            "-o",
            out_hi.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert -q 90");

    assert_eq!(
        hi_out.status.code(),
        Some(0),
        "convert -q 90 should exit 0; stderr: {}",
        stderr_str(&hi_out)
    );

    let lo_size = std::fs::metadata(&out_lo).unwrap().len();
    let hi_size = std::fs::metadata(&out_hi).unwrap().len();
    assert!(
        lo_size < hi_size,
        "low quality ({lo_size} bytes) should be smaller than high quality ({hi_size} bytes)"
    );

    // Both must decode at the same dims.
    let lo_img = image::open(&out_lo).expect("lo output should be decodable");
    let hi_img = image::open(&out_hi).expect("hi output should be decodable");
    assert_eq!(lo_img.width(), hi_img.width(), "widths must match");
    assert_eq!(lo_img.height(), hi_img.height(), "heights must match");
}

/// `convert <missing> --format png` → exit 3 (input not found). (AC8)
#[test]
fn convert_missing_input_exits_3() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("no_such.png");
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "convert",
            missing.to_str().unwrap(),
            "--format",
            "png",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert missing");

    assert_eq!(
        output.status.code(),
        Some(3),
        "convert with missing input should exit 3; stderr: {}",
        stderr_str(&output)
    );
}

/// Two inputs with no `--out-dir` and `--format png` → exit 2; stderr mentions
/// `--out-dir`. (AC9)
#[test]
fn convert_multi_without_out_dir_is_usage_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in1 = write_test_png(&dir, "a.png", 4, 4);
    let in2 = write_test_png(&dir, "b.png", 4, 4);

    let output = Command::new(BIN)
        .args([
            "convert",
            in1.to_str().unwrap(),
            in2.to_str().unwrap(),
            "--format",
            "png",
        ])
        .output()
        .expect("failed to run convert multi without --out-dir");

    assert_eq!(
        output.status.code(),
        Some(2),
        "convert multi without --out-dir should exit 2; stderr: {}",
        stderr_str(&output)
    );
    let stderr = stderr_str(&output);
    assert!(
        stderr.contains("out-dir") || stderr.contains("out_dir"),
        "stderr should mention --out-dir; got: {stderr}"
    );
}

/// `convert <png>` with NO `--format` → exit 2 (clap required-arg error); stderr
/// mentions `--format`. (AC10)
#[test]
fn convert_requires_format_flag() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 4, 4);

    let output = Command::new(BIN)
        .args(["convert", in_path.to_str().unwrap()])
        .output()
        .expect("failed to run convert without --format");

    assert_eq!(
        output.status.code(),
        Some(2),
        "convert without --format should exit 2 (clap required); stderr: {}",
        stderr_str(&output)
    );
    let stderr = stderr_str(&output);
    assert!(
        stderr.contains("--format") || stderr.contains("format"),
        "stderr should mention --format; got: {stderr}"
    );
}

/// `convert <png> --format jpg -o -` → exit 0; stdout decodes as JPEG; stderr
/// empty. (AC11)
#[test]
fn convert_stdout_keeps_stdout_clean() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 20, 10);

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "jpg",
            "-o",
            "-",
        ])
        .output()
        .expect("failed to run convert to stdout");

    assert_eq!(
        output.status.code(),
        Some(0),
        "convert -o - should exit 0; stderr: {}",
        stderr_str(&output)
    );
    // stdout must be ONLY the encoded JPEG bytes.
    assert_eq!(
        &output.stdout[..2],
        b"\xFF\xD8",
        "stdout must begin with JPEG magic (FF D8)"
    );
    let decoded = image::load_from_memory(&output.stdout)
        .expect("stdout bytes should decode as a valid JPEG image");
    assert_eq!(decoded.width(), 20, "decoded width should be 20");
    assert_eq!(decoded.height(), 10, "decoded height should be 10");
    // stderr must be empty.
    assert!(
        output.stderr.is_empty(),
        "stderr must be empty on clean stdout run, got: {}",
        stderr_str(&output)
    );
}

// ── SPEC-016: perceptual auto-quality (`shrink --target` / `--ssim`) ───────────

/// Write raw fixture bytes to `dir/name` and return the path.
fn write_bytes(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// `shrink --target visually-lossless` produces a valid JPEG at the input dims.
#[test]
fn shrink_target_visually_lossless_produces_valid_jpeg() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(160, 160));
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--target",
            "visually-lossless",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink --target");

    assert_eq!(
        output.status.code(),
        Some(0),
        "shrink --target should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("output file should exist");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Jpeg,
        "output should be a JPEG"
    );
    let decoded = image::load_from_memory(&bytes).expect("output should decode");
    // Default --max is 1600, so a 160×160 input is not downscaled.
    assert_eq!(decoded.width(), 160, "width preserved");
    assert_eq!(decoded.height(), 160, "height preserved");
}

/// A lower `--ssim` target yields a smaller file than a higher one (same input).
#[test]
fn shrink_lower_ssim_target_is_smaller_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(160, 160));
    let lo_path = dir.path().join("lo.jpg");
    let hi_path = dir.path().join("hi.jpg");

    for (score, out) in [("50", &lo_path), ("95", &hi_path)] {
        let output = Command::new(BIN)
            .args([
                "shrink",
                in_path.to_str().unwrap(),
                "--ssim",
                score,
                "-o",
                out.to_str().unwrap(),
            ])
            .output()
            .expect("failed to run shrink --ssim");
        assert_eq!(
            output.status.code(),
            Some(0),
            "shrink --ssim {score} should exit 0; stderr: {}",
            stderr_str(&output)
        );
    }

    let lo_len = std::fs::metadata(&lo_path).unwrap().len();
    let hi_len = std::fs::metadata(&hi_path).unwrap().len();
    assert!(
        lo_len < hi_len,
        "--ssim 50 file ({lo_len} bytes) should be smaller than --ssim 95 file ({hi_len} bytes)"
    );
}

/// `--target` and `--ssim` together is a usage error (clap conflict) → exit 2.
#[test]
fn shrink_target_and_ssim_conflict_exits_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(128, 128));
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--target",
            "high",
            "--ssim",
            "80",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink");

    assert_eq!(
        output.status.code(),
        Some(2),
        "--target + --ssim should exit 2; stderr: {}",
        stderr_str(&output)
    );
}

/// `--ssim` outside 0..=100 is a usage error → exit 2.
#[test]
fn shrink_ssim_out_of_range_exits_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(128, 128));
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--ssim",
            "150",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink");

    assert_eq!(
        output.status.code(),
        Some(2),
        "--ssim 150 should exit 2; stderr: {}",
        stderr_str(&output)
    );
    assert!(
        stderr_str(&output).to_lowercase().contains("ssim"),
        "stderr should mention ssim, got: {}",
        stderr_str(&output)
    );
}

/// `-q` combined with an auto target is a usage error → exit 2.
#[test]
fn shrink_quality_and_target_conflict_exits_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(128, 128));
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "-q",
            "80",
            "--target",
            "high",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink");

    assert_eq!(
        output.status.code(),
        Some(2),
        "-q + --target should exit 2; stderr: {}",
        stderr_str(&output)
    );
    assert!(
        stderr_str(&output).to_lowercase().contains("quality")
            || stderr_str(&output).to_lowercase().contains("target"),
        "stderr should mention the quality/target conflict, got: {}",
        stderr_str(&output)
    );
}

/// A non-JPEG output ignores `--target` (no error, output stays PNG).
#[test]
fn shrink_target_non_jpeg_is_ignored() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.png", &common::detailed_png(120, 120));
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--target",
            "visually-lossless",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink --target on png");

    assert_eq!(
        output.status.code(),
        Some(0),
        "shrink --target on a PNG should exit 0 (target ignored); stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("output file should exist");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Png,
        "output should still be a PNG"
    );
}

/// `--target` runs the per-image search across a multi-input fan-out.
#[test]
fn shrink_target_multi_input_fan_out() {
    let dir = tempfile::tempdir().expect("tempdir");
    let a = write_bytes(&dir, "a.jpg", &common::detailed_jpeg(128, 128));
    let b = write_bytes(&dir, "b.jpg", &common::detailed_jpeg(144, 112));
    let out_dir = dir.path().join("out");
    std::fs::create_dir(&out_dir).unwrap();

    let output = Command::new(BIN)
        .args([
            "shrink",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--target",
            "high",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink --target multi");

    assert_eq!(
        output.status.code(),
        Some(0),
        "multi-input shrink --target should exit 0; stderr: {}",
        stderr_str(&output)
    );
    for name in ["a.jpg", "b.jpg"] {
        let bytes = std::fs::read(out_dir.join(name)).expect("output should exist");
        assert_eq!(
            image::guess_format(&bytes).expect("guess format"),
            ImageFormat::Jpeg,
            "{name} should be a JPEG"
        );
    }
}

/// A deterministic high-frequency pseudo-noise JPEG. Unlike the smooth
/// `detailed_*` fixtures, JPEG cannot reproduce this even at quality 100, so an
/// SSIMULACRA2 target near 100 is genuinely unreachable — exactly what the
/// best-effort/warning path needs.
fn noisy_jpeg(w: u32, h: u32) -> Vec<u8> {
    let mut img = RgbImage::new(w, h);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let n = x
            .wrapping_mul(2_654_435_761)
            .wrapping_add(y.wrapping_mul(40_503))
            ^ x.wrapping_mul(y).wrapping_add(0x9E37_79B9);
        *px = image::Rgb([
            (n & 0xFF) as u8,
            ((n >> 8) & 0xFF) as u8,
            ((n >> 16) & 0xFF) as u8,
        ]);
    }
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Jpeg)
        .unwrap();
    buf.into_inner()
}

/// An unreachable perceptual target (`--ssim 100` on a high-frequency image,
/// which a lossy JPEG round-trip cannot reach) still succeeds (best-effort) but
/// warns on stderr.
#[test]
fn shrink_unreachable_target_warns_best_effort() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &noisy_jpeg(96, 96));
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--ssim",
            "100",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink --ssim 100");

    // Best-effort: the command still succeeds and writes a file.
    assert_eq!(
        output.status.code(),
        Some(0),
        "unreachable target should still exit 0 (best effort); stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output file should still be written");
    // ...but it must warn that the target was not met.
    let err = stderr_str(&output).to_lowercase();
    assert!(
        err.contains("could not reach") || err.contains("best effort"),
        "expected an unmet-target warning on stderr, got: {}",
        stderr_str(&output)
    );
}

// ── SPEC-021: --max-size dimension-reduction fallback ───────────────────────────

/// `convert <big detailed png> --format png --max-size 8KB` DOWNSCALES to fit —
/// lossless PNG has no quality knob, so dimensions are the only lever (the new
/// capability). Output is PNG, ≤ the budget, smaller than the source, and warns.
#[test]
fn convert_png_max_size_downscales() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.png", &common::detailed_png(256, 256));
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "png",
            "--max-size",
            "8KB",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --format png --max-size 8KB");

    assert_eq!(
        output.status.code(),
        Some(0),
        "lossless --max-size should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("PNG output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Png,
        "output should be PNG"
    );
    assert!(
        bytes.len() <= 8000,
        "PNG output should fit the 8KB budget, got {} bytes",
        bytes.len()
    );
    let decoded = image::load_from_memory(&bytes).expect("decode png");
    assert!(
        decoded.width() < 256 && decoded.height() < 256,
        "output should be downscaled from 256x256, got {}x{}",
        decoded.width(),
        decoded.height()
    );
    assert!(
        stderr_str(&output).contains("scal"),
        "a downscale should warn; stderr: {}",
        stderr_str(&output)
    );
}

/// A budget the full-size output already fits → no downscale (dimensions kept).
#[test]
fn max_size_keeps_dims_when_it_fits() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 32, 32);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "png",
            "--max-size",
            "1MB",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --format png --max-size 1MB");

    assert_eq!(
        output.status.code(),
        Some(0),
        "feasible --max-size should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("PNG output file");
    let decoded = image::load_from_memory(&bytes).expect("decode png");
    assert_eq!(
        (decoded.width(), decoded.height()),
        (32, 32),
        "a met budget must not resize"
    );
}

// ── SPEC-017: --max-size byte budget (shrink + convert) ────────────────────────

/// `shrink --max-size <feasible>` produces a JPEG within the budget.
#[test]
fn shrink_max_size_fits_budget() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(160, 160));
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--max-size",
            "6KB",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run shrink --max-size");

    assert_eq!(
        output.status.code(),
        Some(0),
        "shrink --max-size should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("output should exist");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Jpeg,
        "output should be a JPEG"
    );
    assert!(
        bytes.len() as u64 <= 6000,
        "output ({} bytes) should fit the 6KB budget",
        bytes.len()
    );
}

/// A larger budget yields a larger-or-equal file than a smaller budget.
#[test]
fn shrink_max_size_larger_budget_not_smaller() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(160, 160));
    let small = dir.path().join("small.jpg");
    let large = dir.path().join("large.jpg");

    for (budget, out) in [("4KB", &small), ("12KB", &large)] {
        let output = Command::new(BIN)
            .args([
                "shrink",
                in_path.to_str().unwrap(),
                "--max-size",
                budget,
                "-o",
                out.to_str().unwrap(),
            ])
            .output()
            .expect("failed to run shrink --max-size");
        assert_eq!(
            output.status.code(),
            Some(0),
            "shrink --max-size {budget} should exit 0; stderr: {}",
            stderr_str(&output)
        );
    }

    let small_len = std::fs::metadata(&small).unwrap().len();
    let large_len = std::fs::metadata(&large).unwrap().len();
    assert!(
        small_len <= large_len,
        "4KB-budget output ({small_len}) should be <= 12KB-budget output ({large_len})"
    );
}

/// `--max-size` conflicts with `--target` → exit 2 (clap).
#[test]
fn shrink_max_size_conflicts_with_target_exits_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(128, 128));
    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--max-size",
            "5KB",
            "--target",
            "high",
            "-o",
            dir.path().join("o.jpg").to_str().unwrap(),
        ])
        .output()
        .expect("run");
    assert_eq!(
        output.status.code(),
        Some(2),
        "stderr: {}",
        stderr_str(&output)
    );
}

/// `--max-size` conflicts with `--ssim` → exit 2 (clap).
#[test]
fn shrink_max_size_conflicts_with_ssim_exits_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(128, 128));
    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--max-size",
            "5KB",
            "--ssim",
            "90",
            "-o",
            dir.path().join("o.jpg").to_str().unwrap(),
        ])
        .output()
        .expect("run");
    assert_eq!(
        output.status.code(),
        Some(2),
        "stderr: {}",
        stderr_str(&output)
    );
}

/// `--max-size` conflicts with `-q` → exit 2 (runtime usage).
#[test]
fn shrink_max_size_conflicts_with_quality_exits_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(128, 128));
    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--max-size",
            "5KB",
            "-q",
            "80",
            "-o",
            dir.path().join("o.jpg").to_str().unwrap(),
        ])
        .output()
        .expect("run");
    assert_eq!(
        output.status.code(),
        Some(2),
        "stderr: {}",
        stderr_str(&output)
    );
}

/// An infeasibly tiny budget still succeeds (best-effort smallest) but warns.
#[test]
fn shrink_max_size_infeasible_warns() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(160, 160));
    let out_path = dir.path().join("out.jpg");
    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--max-size",
            "200", // 200 bytes — below even quality-1
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("run");
    assert_eq!(
        output.status.code(),
        Some(0),
        "infeasible budget should still exit 0 (best effort); stderr: {}",
        stderr_str(&output)
    );
    assert!(out_path.exists(), "output should still be written");
    let err = stderr_str(&output).to_lowercase();
    assert!(
        err.contains("budget") || err.contains("could not"),
        "expected an unmet-budget warning, got: {}",
        stderr_str(&output)
    );
}

/// `--max-size` on a lossless (PNG) output now DOWNSCALES to fit (SPEC-021,
/// DEC-023) — previously it warned and left the image at full size. With a tiny
/// budget the 120x120 source is scaled down; output stays PNG and exits 0.
#[test]
fn shrink_max_size_lossless_downscales() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.png", &common::detailed_png(120, 120));
    let out_path = dir.path().join("out.png");
    let output = Command::new(BIN)
        .args([
            "shrink",
            in_path.to_str().unwrap(),
            "--max-size",
            "1KB",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("run");
    assert_eq!(
        output.status.code(),
        Some(0),
        "shrink --max-size on a PNG should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("output should exist");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Png,
        "output should still be PNG"
    );
    // A 120x120 detailed PNG is far over 1KB, so the fallback must downscale it.
    let decoded = image::load_from_memory(&bytes).expect("decode png");
    assert!(
        decoded.width() < 120 && decoded.height() < 120,
        "lossless --max-size should downscale (got {}x{})",
        decoded.width(),
        decoded.height()
    );
    let err = stderr_str(&output).to_lowercase();
    assert!(
        err.contains("scal") || err.contains("budget"),
        "expected a scaled/budget warning, got: {}",
        stderr_str(&output)
    );
}

/// `convert --format jpeg --max-size <feasible>` fits the budget.
#[test]
fn convert_max_size_to_jpeg_fits() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.png", &common::detailed_png(160, 160));
    let out_path = dir.path().join("out.jpg");
    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "jpeg",
            "--max-size",
            "6KB",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("run");
    assert_eq!(
        output.status.code(),
        Some(0),
        "convert --max-size should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("output should exist");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Jpeg,
        "output should be a JPEG"
    );
    assert!(
        bytes.len() as u64 <= 6000,
        "output ({} bytes) should fit the 6KB budget",
        bytes.len()
    );
}

/// `convert --max-size … -q …` → exit 2 (runtime usage).
#[test]
fn convert_max_size_conflicts_with_quality_exits_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.png", &common::detailed_png(128, 128));
    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "jpeg",
            "--max-size",
            "6KB",
            "-q",
            "80",
            "-o",
            dir.path().join("o.jpg").to_str().unwrap(),
        ])
        .output()
        .expect("run");
    assert_eq!(
        output.status.code(),
        Some(2),
        "stderr: {}",
        stderr_str(&output)
    );
}

// ── SPEC-022: optimize (one-button web-good command) ───────────────────────────

/// `optimize` bakes EXIF orientation into pixels (dims swap on a 90° rotate) AND
/// strips metadata — the correctness + privacy half of the command.
#[test]
fn optimize_reorients_and_strips_metadata() {
    let dir = tempfile::tempdir().expect("tempdir");
    // 128×96 JPEG with Orientation=6 (Rotate90) → output should be 96×128.
    let in_path = write_bytes(&dir, "in.jpg", &common::jpeg_with_orientation(128, 96, 6));
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "optimize",
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run optimize");
    assert_eq!(
        output.status.code(),
        Some(0),
        "optimize should exit 0; stderr: {}",
        stderr_str(&output)
    );

    let decoded = image::open(&out_path).expect("output should decode as JPEG");
    assert_eq!(
        decoded.width(),
        96,
        "rotate90: width should be input height"
    );
    assert_eq!(
        decoded.height(),
        128,
        "rotate90: height should be input width"
    );

    // Metadata stripped: info --json reports has_exif:false.
    let info = Command::new(BIN)
        .args(["info", out_path.to_str().unwrap(), "--json"])
        .output()
        .expect("failed to run info");
    assert!(
        stdout_str(&info).contains("\"has_exif\":false"),
        "optimize output should carry no EXIF; got: {}",
        stdout_str(&info)
    );
}

/// `optimize` with no flags defaults to a visually-lossless re-encode: a valid
/// JPEG, smaller than a max-quality encode, with dimensions preserved.
#[test]
fn optimize_default_is_smaller_valid_jpeg() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = common::detailed_jpeg(96, 96);
    let in_path = write_bytes(&dir, "in.jpg", &src);
    let out_path = dir.path().join("out.jpg");

    // Baseline: the same decoded pixels re-encoded at quality 100.
    let baseline_len = {
        let img = image::load_from_memory(&src).expect("decode src");
        let mut c = Cursor::new(Vec::new());
        let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut c, 100);
        img.write_with_encoder(enc).expect("encode q100");
        c.into_inner().len()
    };

    let output = Command::new(BIN)
        .args([
            "optimize",
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run optimize");
    assert_eq!(
        output.status.code(),
        Some(0),
        "optimize should exit 0; stderr: {}",
        stderr_str(&output)
    );

    let bytes = std::fs::read(&out_path).expect("output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Jpeg,
        "output should be a JPEG"
    );
    assert!(
        bytes.len() < baseline_len,
        "visually-lossless output ({} bytes) should be smaller than a q100 encode ({} bytes)",
        bytes.len(),
        baseline_len
    );
    let decoded = image::load_from_memory(&bytes).expect("decode output");
    assert_eq!(
        (decoded.width(), decoded.height()),
        (96, 96),
        "dims preserved"
    );
}

/// `optimize` preserves the input format and dimensions by default (no resize,
/// no container change).
#[test]
fn optimize_preserves_format_and_dims_by_default() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(128, 96));
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "optimize",
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run optimize");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        stderr_str(&output)
    );

    let bytes = std::fs::read(&out_path).expect("output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Jpeg,
        "format preserved (JPEG)"
    );
    let decoded = image::load_from_memory(&bytes).expect("decode output");
    assert_eq!(
        (decoded.width(), decoded.height()),
        (128, 96),
        "no default resize"
    );
}

/// `optimize --max N` bounds the long edge (the only way it resizes).
#[test]
fn optimize_max_bounds_long_edge() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(256, 192));
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "optimize",
            in_path.to_str().unwrap(),
            "--max",
            "128",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run optimize --max");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        stderr_str(&output)
    );

    let decoded = image::open(&out_path).expect("decode output");
    assert!(
        decoded.width().max(decoded.height()) <= 128,
        "long edge should be ≤ 128, got {}x{}",
        decoded.width(),
        decoded.height()
    );
    assert_eq!(
        (decoded.width(), decoded.height()),
        (128, 96),
        "256x192 bounded to 128 long-edge → 128x96"
    );
}

/// `optimize -o out.png` honors the output format; the perceptual target is
/// silently ignored for the lossless format (no error).
#[test]
fn optimize_format_change_to_png() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(128, 96));
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "optimize",
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run optimize -o png");
    assert_eq!(
        output.status.code(),
        Some(0),
        "optimize to PNG should exit 0 (target ignored for lossless); stderr: {}",
        stderr_str(&output)
    );
    let bytes = std::fs::read(&out_path).expect("output file");
    assert_eq!(
        image::guess_format(&bytes).expect("guess format"),
        ImageFormat::Png,
        "output should be PNG"
    );
}

/// `optimize --max-size <budget>` fits the byte budget (reuses the SPEC-017/021 fit).
#[test]
fn optimize_max_size_fits_budget() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = common::detailed_jpeg(128, 128);
    let in_path = write_bytes(&dir, "in.jpg", &src);
    let out_path = dir.path().join("out.jpg");

    // A budget a third of the q100 size: feasible by lowering quality alone.
    let baseline_len = {
        let img = image::load_from_memory(&src).expect("decode src");
        let mut c = Cursor::new(Vec::new());
        let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut c, 100);
        img.write_with_encoder(enc).expect("encode q100");
        c.into_inner().len()
    };
    let budget = (baseline_len / 3).max(1);

    let output = Command::new(BIN)
        .args([
            "optimize",
            in_path.to_str().unwrap(),
            "--max-size",
            &format!("{budget}B"),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run optimize --max-size");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        stderr_str(&output)
    );

    let bytes = std::fs::read(&out_path).expect("output file");
    assert!(
        bytes.len() <= budget,
        "output ({} bytes) should fit the {budget}B budget",
        bytes.len()
    );
}

/// A fixed `-q` with `optimize` is a usage error (optimize always auto-tunes).
#[test]
fn optimize_quality_flag_is_usage_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(96, 96));
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "optimize",
            in_path.to_str().unwrap(),
            "-q",
            "80",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run optimize -q");
    assert_eq!(
        output.status.code(),
        Some(2),
        "-q with optimize should exit 2; stderr: {}",
        stderr_str(&output)
    );
}

/// Multi-input `optimize` with `--out-dir` writes every output.
#[test]
fn optimize_multi_input_fan_out() {
    let dir = tempfile::tempdir().expect("tempdir");
    let a = write_bytes(&dir, "a.jpg", &common::detailed_jpeg(96, 96));
    let b = write_bytes(&dir, "b.jpg", &common::detailed_jpeg(112, 96));
    let out_dir = dir.path().join("out");
    std::fs::create_dir(&out_dir).unwrap();

    let output = Command::new(BIN)
        .args([
            "optimize",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run optimize multi");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        stderr_str(&output)
    );

    for name in &["a.jpg", "b.jpg"] {
        assert!(
            out_dir.join(name).exists(),
            "{name} should exist in out-dir"
        );
    }
}

/// `optimize` with a missing input exits 3.
#[test]
fn optimize_missing_input_exits_3() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("nope.jpg");
    let out_path = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "optimize",
            missing.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run optimize missing");
    assert_eq!(output.status.code(), Some(3), "missing input should exit 3");
}

/// Multi-input `optimize` without `--out-dir` is a usage error (exit 2).
#[test]
fn optimize_multi_without_out_dir_is_usage_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let a = write_bytes(&dir, "a.jpg", &common::detailed_jpeg(96, 96));
    let b = write_bytes(&dir, "b.jpg", &common::detailed_jpeg(96, 96));

    let output = Command::new(BIN)
        .args(["optimize", a.to_str().unwrap(), b.to_str().unwrap()])
        .output()
        .expect("failed to run optimize multi no out-dir");
    assert_eq!(
        output.status.code(),
        Some(2),
        "multi-input without --out-dir should exit 2"
    );
}

// ── SPEC-023: diff (perceptual comparison + CI gate) ───────────────────────────

/// Encode the decoded pixels of `src` to a JPEG at the given quality (test helper
/// for building a degraded copy of the same dimensions).
fn jpeg_at_quality(src: &[u8], q: u8) -> Vec<u8> {
    let img = image::load_from_memory(src).expect("decode src");
    let mut c = Cursor::new(Vec::new());
    let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut c, q);
    img.write_with_encoder(enc).expect("encode jpeg");
    c.into_inner()
}

/// Parse the score from a `ssimulacra2: <N>` stdout line.
fn parse_score(stdout: &str) -> f64 {
    stdout
        .lines()
        .find_map(|l| l.strip_prefix("ssimulacra2:"))
        .unwrap_or_else(|| panic!("no ssimulacra2 line in stdout:\n{stdout}"))
        .trim()
        .parse()
        .expect("score should parse as f64")
}

/// `diff a a` (identical) scores near 100 and exits 0.
#[test]
fn diff_identical_scores_high() {
    let dir = tempfile::tempdir().expect("tempdir");
    let a = write_bytes(&dir, "a.png", &common::detailed_png(96, 96));

    let output = Command::new(BIN)
        .args(["diff", a.to_str().unwrap(), a.to_str().unwrap()])
        .output()
        .expect("failed to run diff");
    assert_eq!(
        output.status.code(),
        Some(0),
        "diff identical should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let score = parse_score(&stdout_str(&output));
    assert!(
        score >= 90.0,
        "identical images should score ≥ 90, got {score}"
    );
}

/// `diff` of an image vs a heavily-degraded copy scores below the identical score
/// (and below 90), exit 0.
#[test]
fn diff_degraded_scores_lower() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = common::detailed_png(96, 96);
    let a = write_bytes(&dir, "a.png", &src);
    let b = write_bytes(&dir, "b.jpg", &jpeg_at_quality(&src, 5));

    let output = Command::new(BIN)
        .args(["diff", a.to_str().unwrap(), b.to_str().unwrap()])
        .output()
        .expect("failed to run diff");
    assert_eq!(
        output.status.code(),
        Some(0),
        "diff should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let score = parse_score(&stdout_str(&output));
    assert!(
        score < 90.0,
        "a quality-5 degraded copy should score below 90, got {score}"
    );
}

/// `--fail-under 90` on a below-90 pair exits 7, with the score still on stdout.
#[test]
fn diff_fail_under_gate_fails() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = common::detailed_png(96, 96);
    let a = write_bytes(&dir, "a.png", &src);
    let b = write_bytes(&dir, "b.jpg", &jpeg_at_quality(&src, 5));

    let output = Command::new(BIN)
        .args([
            "diff",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--fail-under",
            "90",
        ])
        .output()
        .expect("failed to run diff --fail-under");
    assert_eq!(
        output.status.code(),
        Some(7),
        "below-threshold gate should exit 7; stderr: {}",
        stderr_str(&output)
    );
    assert!(
        stdout_str(&output).contains("ssimulacra2:"),
        "the score line should still be printed; stdout: {}",
        stdout_str(&output)
    );
}

/// `--fail-under 90` on an at/above-90 pair (identical) exits 0.
#[test]
fn diff_fail_under_gate_passes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let a = write_bytes(&dir, "a.png", &common::detailed_png(96, 96));

    let output = Command::new(BIN)
        .args([
            "diff",
            a.to_str().unwrap(),
            a.to_str().unwrap(),
            "--fail-under",
            "90",
        ])
        .output()
        .expect("failed to run diff --fail-under");
    assert_eq!(
        output.status.code(),
        Some(0),
        "identical pair clears the gate; stderr: {}",
        stderr_str(&output)
    );
}

/// `--fail-under` outside 0..=100 is a usage error (exit 2).
#[test]
fn diff_fail_under_out_of_range_exits_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let a = write_bytes(&dir, "a.png", &common::detailed_png(96, 96));

    let output = Command::new(BIN)
        .args([
            "diff",
            a.to_str().unwrap(),
            a.to_str().unwrap(),
            "--fail-under",
            "150",
        ])
        .output()
        .expect("failed to run diff");
    assert_eq!(
        output.status.code(),
        Some(2),
        "--fail-under 150 should exit 2"
    );
}

/// Comparing images of different dimensions is a usage error (exit 2).
#[test]
fn diff_dimension_mismatch_exits_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let a = write_bytes(&dir, "a.png", &common::detailed_png(64, 64));
    let b = write_bytes(&dir, "b.png", &common::detailed_png(32, 32));

    let output = Command::new(BIN)
        .args(["diff", a.to_str().unwrap(), b.to_str().unwrap()])
        .output()
        .expect("failed to run diff mismatch");
    assert_eq!(
        output.status.code(),
        Some(2),
        "dimension mismatch should exit 2; stderr: {}",
        stderr_str(&output)
    );
}

/// `--json` emits a machine-readable object with score + passed.
#[test]
fn diff_json_output() {
    let dir = tempfile::tempdir().expect("tempdir");
    let a = write_bytes(&dir, "a.png", &common::detailed_png(96, 96));

    let output = Command::new(BIN)
        .args(["diff", a.to_str().unwrap(), a.to_str().unwrap(), "--json"])
        .output()
        .expect("failed to run diff --json");
    assert_eq!(
        output.status.code(),
        Some(0),
        "diff --json identical should exit 0; stderr: {}",
        stderr_str(&output)
    );
    let stdout = stdout_str(&output);
    assert!(
        stdout.contains("\"score\":"),
        "json should carry score; got: {stdout}"
    );
    assert!(
        stdout.contains("\"passed\":true"),
        "identical with no gate should be passed:true; got: {stdout}"
    );
}

/// A missing input path exits 3.
#[test]
fn diff_missing_input_exits_3() {
    let dir = tempfile::tempdir().expect("tempdir");
    let a = write_bytes(&dir, "a.png", &common::detailed_png(96, 96));
    let missing = dir.path().join("missing.png");

    let output = Command::new(BIN)
        .args(["diff", missing.to_str().unwrap(), a.to_str().unwrap()])
        .output()
        .expect("failed to run diff missing");
    assert_eq!(output.status.code(), Some(3), "missing input should exit 3");
}

// ── SPEC-024: responsive (<picture>/srcset set generator) ──────────────────────

/// `responsive --widths 320,640` writes width-scaled JPEG variants + an <img srcset>.
#[test]
fn responsive_writes_width_variants() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(800, 600));
    let out_dir = dir.path().join("dist");

    let output = Command::new(BIN)
        .args([
            "responsive",
            in_path.to_str().unwrap(),
            "--widths",
            "320,640",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run responsive");
    assert_eq!(
        output.status.code(),
        Some(0),
        "responsive should exit 0; stderr: {}",
        stderr_str(&output)
    );
    for (name, want_w) in [("in-320w.jpg", 320), ("in-640w.jpg", 640)] {
        let p = out_dir.join(name);
        assert!(p.exists(), "{name} should exist");
        let decoded = image::open(&p).expect("variant should decode");
        assert_eq!(decoded.width(), want_w, "{name} width");
    }
    let stdout = stdout_str(&output);
    assert!(
        stdout.contains("srcset="),
        "snippet should have srcset; got: {stdout}"
    );
    assert!(
        stdout.contains("320w") && stdout.contains("640w"),
        "descriptors: {stdout}"
    );
}

/// `--formats webp,jpeg` writes both per width and emits a <picture> block.
#[test]
fn responsive_multi_format_emits_picture() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "hero.jpg", &common::detailed_jpeg(800, 600));
    let out_dir = dir.path().join("dist");

    let output = Command::new(BIN)
        .args([
            "responsive",
            in_path.to_str().unwrap(),
            "--widths",
            "320,640",
            "--formats",
            "webp,jpeg",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run responsive multi-format");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        stderr_str(&output)
    );

    for name in [
        "hero-320w.webp",
        "hero-640w.webp",
        "hero-320w.jpg",
        "hero-640w.jpg",
    ] {
        assert!(out_dir.join(name).exists(), "{name} should exist");
    }
    let stdout = stdout_str(&output);
    assert!(
        stdout.contains("<picture>"),
        "should emit <picture>: {stdout}"
    );
    assert!(
        stdout.contains("type=\"image/webp\""),
        "webp source: {stdout}"
    );
    assert!(
        stdout.contains("type=\"image/jpeg\""),
        "jpeg source: {stdout}"
    );
    assert!(stdout.contains("<img "), "img fallback: {stdout}");
}

/// A width greater than the source width is skipped (no upscaling), with a warning.
#[test]
fn responsive_no_upscale_skips_wide() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(400, 300));
    let out_dir = dir.path().join("dist");

    let output = Command::new(BIN)
        .args([
            "responsive",
            in_path.to_str().unwrap(),
            "--widths",
            "320,800",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run responsive");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        stderr_str(&output)
    );
    assert!(
        out_dir.join("in-320w.jpg").exists(),
        "320 variant should exist"
    );
    assert!(
        !out_dir.join("in-800w.jpg").exists(),
        "800 must be skipped (no upscale)"
    );
    assert!(
        stderr_str(&output).contains("800") && stderr_str(&output).to_lowercase().contains("skip"),
        "should warn about skipping 800; stderr: {}",
        stderr_str(&output)
    );
}

/// If every requested width exceeds the source width → exit 2.
#[test]
fn responsive_all_widths_exceed_source_exits_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(200, 150));
    let out_dir = dir.path().join("dist");

    let output = Command::new(BIN)
        .args([
            "responsive",
            in_path.to_str().unwrap(),
            "--widths",
            "320,640",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run responsive");
    assert_eq!(
        output.status.code(),
        Some(2),
        "all-too-wide should exit 2; stderr: {}",
        stderr_str(&output)
    );
}

/// `--formats avif` without the feature exits 4 before writing anything.
#[cfg(not(feature = "avif"))]
#[test]
fn responsive_avif_without_feature_exits_4() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(800, 600));
    let out_dir = dir.path().join("dist");

    let output = Command::new(BIN)
        .args([
            "responsive",
            in_path.to_str().unwrap(),
            "--widths",
            "320",
            "--formats",
            "avif",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run responsive avif");
    assert_eq!(
        output.status.code(),
        Some(4),
        "avif without feature should exit 4; stderr: {}",
        stderr_str(&output)
    );
    // Nothing should have been written.
    assert!(
        !out_dir.join("in-320w.avif").exists(),
        "no file should be written on the up-front codec failure"
    );
}

/// `--no-snippet` suppresses the HTML on stdout but still writes the files.
#[test]
fn responsive_no_snippet_suppresses_html() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(800, 600));
    let out_dir = dir.path().join("dist");

    let output = Command::new(BIN)
        .args([
            "responsive",
            in_path.to_str().unwrap(),
            "--widths",
            "320",
            "--no-snippet",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run responsive --no-snippet");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        stderr_str(&output)
    );
    assert!(
        out_dir.join("in-320w.jpg").exists(),
        "variant still written"
    );
    assert!(
        !stdout_str(&output).contains("srcset"),
        "stdout should be empty of HTML; got: {}",
        stdout_str(&output)
    );
}

/// `--out-dir` is created if it does not exist.
#[test]
fn responsive_creates_out_dir() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(800, 600));
    let out_dir = dir.path().join("a/b/dist"); // nested, not pre-created

    let output = Command::new(BIN)
        .args([
            "responsive",
            in_path.to_str().unwrap(),
            "--widths",
            "320",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run responsive");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        stderr_str(&output)
    );
    assert!(
        out_dir.join("in-320w.jpg").exists(),
        "out-dir should be created"
    );
}

/// Malformed `--widths` is a usage error (exit 2).
#[test]
fn responsive_malformed_widths_exits_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_bytes(&dir, "in.jpg", &common::detailed_jpeg(800, 600));
    let out_dir = dir.path().join("dist");

    for bad in ["0", "abc"] {
        let output = Command::new(BIN)
            .args([
                "responsive",
                in_path.to_str().unwrap(),
                "--widths",
                bad,
                "--out-dir",
                out_dir.to_str().unwrap(),
            ])
            .output()
            .expect("failed to run responsive");
        assert_eq!(
            output.status.code(),
            Some(2),
            "--widths {bad} should exit 2; stderr: {}",
            stderr_str(&output)
        );
    }
}

// ── SPEC-033 decode resource limits integration test ──────────────────────────

/// Running `crustyimg info` on a 70 000×1 PNG (width > MAX_IMAGE_DIMENSION=65535)
/// must exit 1 and print a non-empty error to stderr — not panic, not hang, not
/// OOM, not exit 4 (format) or exit 3 (io). (SPEC-033, DEC-034)
#[test]
fn info_on_oversized_image_exits_1_not_panic() {
    use image::{DynamicImage, RgbImage};

    // Build a real 70 000×1 PNG (~210 KB encoded). `RgbImage::new` creates all
    // zero pixels; encoding succeeds. The decoder hits MAX_IMAGE_DIMENSION at the
    // IHDR check before any pixel data is read, so this fixture never OOMs.
    let img = RgbImage::new(70_000, 1);
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .expect("encode 70_000×1 PNG");

    let dir = tempfile::tempdir().expect("tempdir");
    let bomb_path = dir.path().join("bomb.png");
    std::fs::write(&bomb_path, buf.into_inner()).expect("write bomb.png");

    let output = Command::new(BIN)
        .args(["info", bomb_path.to_str().unwrap()])
        .output()
        .expect("failed to run crustyimg info bomb.png");

    assert_eq!(
        output.status.code(),
        Some(1),
        "oversized PNG must exit 1 (LimitsExceeded); stderr: {}",
        stderr_str(&output)
    );
    let stderr = stderr_str(&output);
    assert!(
        !stderr.is_empty(),
        "stderr must be non-empty (error message expected)"
    );
}
