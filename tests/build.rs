//! Integration tests for the declared `crustyimg build` path (SPEC-063).
//!
//! All tests drive the real compiled binary via `env!("CARGO_BIN_EXE_crustyimg")`
//! with the temp project as the working directory — manifest paths (`source`,
//! `recipe`, `out`) resolve relative to the CWD (DEC-057). Fixtures are
//! synthesized in memory with the `image` crate — no committed binary files.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use image::{DynamicImage, ImageFormat, RgbImage};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// A minimal recipe that resizes to max 16 (tiny, fast, verifiable).
const RESIZE_RECIPE: &str = r#"
version = "1"

[[step]]
op = "resize"
mode = "max"
width = 16
"#;

// ── Fixture helpers ───────────────────────────────────────────────────────────

/// Generate a solid-color RGB PNG at `dir/rel` (creating parent dirs).
fn write_png(dir: &Path, rel: &str, w: u32, h: u32) -> PathBuf {
    let img = RgbImage::from_pixel(w, h, image::Rgb([42u8, 100u8, 200u8]));
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .unwrap();
    write_file(dir, rel, &buf.into_inner())
}

/// Write raw bytes to `dir/rel` (creating parent dirs). Returns the path.
fn write_file(dir: &Path, rel: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Run `crustyimg build [args]` with `dir` as the working directory.
fn run_build(dir: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .arg("build")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("binary should run")
}

/// Assert a written output exists and has the expected dimensions.
fn assert_dims(path: &Path, w: u32, h: u32) {
    assert!(path.exists(), "expected output at {}", path.display());
    let img = image::open(path).unwrap_or_else(|e| panic!("{} should decode: {e}", path.display()));
    assert_eq!(
        (img.width(), img.height()),
        (w, h),
        "unexpected dimensions for {}",
        path.display()
    );
}

/// A two-target project: one `source` glob string, one `source` list (with a
/// `name` template). Returns the temp dir.
fn two_target_project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "r.toml", RESIZE_RECIPE.as_bytes());
    write_png(root, "src_a/a1.png", 32, 32);
    write_png(root, "src_a/a2.png", 32, 32);
    write_png(root, "src_b/b1.png", 64, 64);
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src_a/*.png"
recipe = "r.toml"
out = "dist/a"

[[target]]
source = ["src_b/b1.png"]
recipe = "r.toml"
out = "dist/b"
name = "{stem}_web.{ext}"
"#,
    );
    dir
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// A valid manifest runs EVERY target: both targets' outputs land in their own
/// `out` dir, under their own name template, with the recipe applied.
#[test]
fn build_runs_all_targets() {
    let dir = two_target_project();
    let root = dir.path();

    let out = run_build(root, &[]);
    assert!(
        out.status.success(),
        "build should exit 0, got {:?}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );

    // Target 1: default template `{stem}.{ext}`, source format preserved (png).
    assert_dims(&root.join("dist/a/a1.png"), 16, 16);
    assert_dims(&root.join("dist/a/a2.png"), 16, 16);
    // Target 2: `{stem}_web.{ext}` template.
    assert_dims(&root.join("dist/b/b1_web.png"), 16, 16);
}

/// `crustyimg build` with no arg discovers `./crustyimg.build.toml`; an explicit
/// FILE path also works; a missing default manifest is a clear typed error (exit 3).
#[test]
fn build_discovers_default_manifest() {
    let dir = two_target_project();
    let root = dir.path();

    // No arg → discovers ./crustyimg.build.toml.
    assert!(run_build(root, &[]).status.success());
    assert!(root.join("dist/a/a1.png").exists());

    // Explicit FILE path → same manifest under a different name.
    std::fs::rename(
        root.join("crustyimg.build.toml"),
        root.join("custom.build.toml"),
    )
    .unwrap();
    let out = run_build(root, &["custom.build.toml"]);
    assert!(
        out.status.success(),
        "explicit FILE should work, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Missing default manifest → typed error, exit 3, no panic.
    let empty = TempDir::new().unwrap();
    let out = run_build(empty.path(), &[]);
    assert_eq!(out.status.code(), Some(3), "missing manifest should exit 3");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("crustyimg.build.toml"),
        "error should name the manifest it looked for, got: {stderr}"
    );
    assert!(!stderr.contains("panicked"), "must not panic: {stderr}");
}

/// A manifest with an unknown field or an unsupported version is rejected with a
/// typed error (exit 2) BEFORE any input is touched.
#[test]
fn build_rejects_bad_manifest_before_touching_inputs() {
    for (manifest, needle) in [
        (
            r#"
version = 1

[[target]]
source = "src_a/*.png"
recipe = "r.toml"
out = "dist/a"
bogus = 1
"#,
            "bogus",
        ),
        (
            r#"
version = 999

[[target]]
source = "src_a/*.png"
recipe = "r.toml"
out = "dist/a"
"#,
            "version",
        ),
    ] {
        let dir = two_target_project();
        let root = dir.path();
        write_file(root, "crustyimg.build.toml", manifest.as_bytes());

        let out = run_build(root, &[]);
        assert_eq!(
            out.status.code(),
            Some(2),
            "a malformed manifest should exit 2 (usage)"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(needle),
            "error should mention {needle}, got: {stderr}"
        );
        assert!(
            !root.join("dist").exists(),
            "no output may be written for a rejected manifest"
        );
    }
}

/// A target referencing a nonexistent recipe fails before ANY target writes.
#[test]
fn build_missing_recipe_fails_before_writing() {
    let dir = two_target_project();
    let root = dir.path();
    // Target 1 is fine; target 2's recipe does not exist. Nothing may be written.
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src_a/*.png"
recipe = "r.toml"
out = "dist/a"

[[target]]
source = "src_b/*.png"
recipe = "nope.toml"
out = "dist/b"
"#,
    );

    let out = run_build(root, &[]);
    assert_eq!(
        out.status.code(),
        Some(3),
        "a missing recipe file is an unreadable input (exit 3)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("nope.toml"),
        "error should name the missing recipe, got: {stderr}"
    );
    assert!(
        !root.join("dist").exists(),
        "no target may write when another target's recipe is unusable"
    );
}

/// `build` owns its declared outputs: a re-run overwrites them without `--yes`.
#[test]
fn build_reruns_idempotently_without_yes() {
    let dir = two_target_project();
    let root = dir.path();

    for run in 1..=2 {
        let out = run_build(root, &[]);
        assert!(
            out.status.success(),
            "run {run} should exit 0 (no --yes), stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_dims(&root.join("dist/a/a1.png"), 16, 16);
        assert_dims(&root.join("dist/b/b1_web.png"), 16, 16);
    }
}

/// A successful build prints a summary naming the targets and the output count.
#[test]
fn build_reports_summary() {
    let dir = two_target_project();
    let out = run_build(dir.path(), &[]);
    assert!(out.status.success());

    // Diagnostics go to stderr; stdout stays clean.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("dist/a") && stderr.contains("dist/b"),
        "summary should name each target's out dir, got: {stderr}"
    );
    assert!(
        stderr.contains("2 targets") && stderr.contains("3 outputs"),
        "summary should report targets run + outputs written, got: {stderr}"
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).is_empty(),
        "build must not write diagnostics to stdout"
    );

    // `--quiet` suppresses the summary.
    let quiet = run_build(dir.path(), &["--quiet"]);
    assert!(quiet.status.success());
    assert!(
        String::from_utf8_lossy(&quiet.stderr).is_empty(),
        "--quiet should suppress the summary"
    );
}

/// One undecodable source is a partial-batch failure (exit 6, DEC-015): the good
/// outputs are still written and the bad one is reported.
#[test]
fn build_partial_failure_is_exit_6() {
    let dir = two_target_project();
    let root = dir.path();
    write_file(root, "src_a/corrupt.png", b"this is not a PNG");

    let out = run_build(root, &[]);
    assert_eq!(
        out.status.code(),
        Some(6),
        "a per-output failure is a partial batch (exit 6)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("corrupt.png"),
        "the failing output should be reported, got: {stderr}"
    );
    // The good outputs of both targets are still written.
    assert_dims(&root.join("dist/a/a1.png"), 16, 16);
    assert_dims(&root.join("dist/a/a2.png"), 16, 16);
    assert_dims(&root.join("dist/b/b1_web.png"), 16, 16);
}

/// Write the minimal project fixtures (recipe + one source PNG) into `root`.
fn populate_min_project(root: &Path) {
    write_file(root, "r.toml", RESIZE_RECIPE.as_bytes());
    write_png(root, "src_a/a1.png", 32, 32);
    write_png(root, "src_a/a2.png", 32, 32);
}

/// A hostile manifest whose target `out` escapes the build tree — via `..` or an
/// absolute path — is rejected at manifest validation (exit 2, SPEC-068) BEFORE
/// any filesystem write, and NOTHING lands outside the tree. A legit relative
/// `out` still builds. Drives the real binary with a hostile FILE, not a
/// constructed struct — this is the ship-blocker the threat-model review found.
#[test]
fn build_rejects_out_directory_escape() {
    // The project is NESTED inside a dedicated outer temp dir, so a `..` escape
    // lands in a location this test fully OWNS (`<outer>/ESCAPE/...`) — not a
    // shared system-temp parent where a leftover would make the check flaky.
    // ── 1) Relative `..` escape ──────────────────────────────────────────────
    {
        let outer = TempDir::new().unwrap();
        let root = outer.path().join("proj");
        std::fs::create_dir_all(&root).unwrap();
        populate_min_project(&root);
        // `../ESCAPE/planted` climbs out of <outer>/proj into <outer>/ESCAPE.
        // Without the clamp this writes re-encoded bytes there at exit 0; with
        // it, exit 2 and nothing is written.
        write_file(
            &root,
            "crustyimg.build.toml",
            b"version = 1\n[[target]]\nsource = \"src_a/*.png\"\nrecipe = \"r.toml\"\nout = \"../ESCAPE/planted\"\n",
        );

        let out = run_build(&root, &[]);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(2),
            "a `..` out-escape must be rejected at validation (exit 2), stderr: {stderr}"
        );
        assert!(
            stderr.contains("out") && stderr.contains("escapes the build tree"),
            "error should name the escaping `out`, got: {stderr}"
        );
        assert!(!stderr.contains("panicked"), "must not panic: {stderr}");
        // Nothing may be written at the would-be escape target (owned by `outer`).
        assert!(
            !outer.path().join("ESCAPE").exists(),
            "no bytes may be written outside the build tree"
        );
    }

    // ── 2) Absolute escape ───────────────────────────────────────────────────
    // An absolute `out` pointing at a sibling temp dir OUTSIDE the project.
    {
        let escape_root = TempDir::new().unwrap();
        let planted = escape_root.path().join("planted");
        // TOML basic strings treat `\` as an escape; double it so a Windows path
        // (C:\...) round-trips (no-op on Unix).
        let abs = planted.to_string_lossy().replace('\\', "\\\\");

        let dir = TempDir::new().unwrap();
        let root = dir.path();
        populate_min_project(root);
        write_file(
            root,
            "crustyimg.build.toml",
            format!(
                "version = 1\n[[target]]\nsource = \"src_a/*.png\"\nrecipe = \"r.toml\"\nout = \"{abs}\"\n"
            )
            .as_bytes(),
        );

        let out = run_build(root, &[]);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(2),
            "an absolute out-escape must be rejected (exit 2), stderr: {stderr}"
        );
        assert!(
            !planted.exists(),
            "no bytes may be written at the absolute escape target"
        );
    }

    // ── 3) A legit contained `out` still builds ──────────────────────────────
    {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        populate_min_project(root);
        write_file(
            root,
            "crustyimg.build.toml",
            br#"
version = 1

[[target]]
source = "src_a/*.png"
recipe = "r.toml"
out = "dist"

[[target]]
source = "src_a/a1.png"
recipe = "r.toml"
out = "build/thumbs"
name = "{stem}_t.{ext}"
"#,
        );
        let out = run_build(root, &[]);
        assert!(
            out.status.success(),
            "a contained `out` (dist, build/thumbs) must still build, stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_dims(&root.join("dist/a1.png"), 16, 16);
        assert_dims(&root.join("dist/a2.png"), 16, 16);
        assert_dims(&root.join("build/thumbs/a1_t.png"), 16, 16);
    }
}

/// An empty glob / missing source path is a hard source error, not a silent no-op.
#[test]
fn build_empty_source_is_an_error() {
    let dir = two_target_project();
    let root = dir.path();
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src_a/*.jpg"
recipe = "r.toml"
out = "dist/a"
"#,
    );

    let out = run_build(root, &[]);
    assert_eq!(
        out.status.code(),
        Some(3),
        "an empty glob should be an input-not-found error"
    );
    assert!(!root.join("dist").exists(), "nothing may be written");
}
