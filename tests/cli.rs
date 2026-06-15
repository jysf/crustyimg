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
