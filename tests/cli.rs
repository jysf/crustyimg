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
        "resize",
        "thumbnail",
        "shrink",
        "convert",
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
        "resize",
        "thumbnail",
        "shrink",
        "convert",
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

/// A stub command (here `resize`) exits 1 and writes "not yet implemented"
/// to stderr; no output file is created.
#[test]
fn stub_command_returns_not_implemented() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = write_test_png(&dir, "in.png", 4, 4);
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "resize",
            in_path.to_str().unwrap(),
            "--max",
            "800",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run resize");

    assert_eq!(output.status.code(), Some(1), "stub command should exit 1");
    assert!(
        !out_path.exists(),
        "stub command must not create an output file"
    );
    let stderr = stderr_str(&output);
    assert!(
        stderr.to_ascii_lowercase().contains("not yet implemented"),
        "stderr should contain 'not yet implemented', got: {stderr}"
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
