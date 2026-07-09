//! Integration tests for the injective source→output guarantee (SPEC-065).
//!
//! A build's `source → output` mapping must be a function: two inputs that would
//! be written to one output path race the rayon fan-out under `Overwrite::Allow`,
//! and STAGE-022's lockfile cannot pin a path two inputs fight over (DEC-057).
//!
//! Each test drives the real compiled binary with a temp project as its working
//! directory (manifest paths are cwd-relative, DEC-057) and asserts the shape the
//! spec promises: **exit 2, before any write** — no outputs, no `.crustyimg/`.
//! Fixtures are synthesized in memory with the `image` crate.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use image::{DynamicImage, ImageFormat, RgbImage};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// The cache root a build creates under its working directory — must NOT appear
/// when a collision is rejected (the check runs before `Cache::open`).
const CACHE_DIR: &str = ".crustyimg";

/// Resize every source to max 16px — tiny and fast.
const RECIPE: &str = r#"
version = "1"

[[step]]
op = "resize"
mode = "max"
width = 16
"#;

// ── Fixture helpers ───────────────────────────────────────────────────────────

fn write_file(dir: &Path, rel: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Write a solid-color RGB PNG at `dir/rel`.
fn write_png(dir: &Path, rel: &str, w: u32, h: u32, rgb: [u8; 3]) {
    let img = RgbImage::from_pixel(w, h, image::Rgb(rgb));
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .unwrap();
    write_file(dir, rel, &buf.into_inner());
}

/// A project with the shared recipe and `manifest` as `crustyimg.build.toml`.
fn project(manifest: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    write_file(dir.path(), "r.toml", RECIPE.as_bytes());
    write_file(dir.path(), "crustyimg.build.toml", manifest.as_bytes());
    dir
}

/// The two-same-stem project: `a/logo.png` + `b/logo.png` in ONE target.
fn same_stem_project(name_template: &str) -> TempDir {
    let dir = project(&format!(
        r#"
version = 1

[[target]]
source = ["a/*.png", "b/*.png"]
recipe = "r.toml"
out = "dist"
name = "{name_template}"
"#
    ));
    write_png(dir.path(), "a/logo.png", 32, 32, [200, 30, 30]);
    write_png(dir.path(), "b/logo.png", 48, 48, [30, 30, 200]);
    dir
}

fn build(dir: &Path) -> Output {
    Command::new(BIN)
        .arg("build")
        .current_dir(dir)
        .output()
        .expect("binary should run")
}

/// Assert the build was rejected as a collision: exit 2, a stderr message naming
/// the shared output and both sources, and **nothing written**.
///
/// `sources` are `/`-separated for readability; a source label is a `Path`
/// display, so each is re-joined natively before matching (Windows prints `\`).
fn assert_rejected_before_write(dir: &Path, out_dir: &str, sources: &[&str]) {
    let out = build(dir);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert_eq!(
        out.status.code(),
        Some(2),
        "a non-injective build must exit 2\nstderr: {stderr}"
    );
    assert!(!stderr.contains("panicked"), "must not panic: {stderr}");
    assert!(
        stderr.contains("output collision"),
        "stderr must name the collision: {stderr}"
    );
    for src in sources {
        let native: PathBuf = src.split('/').collect();
        let native = native.display().to_string();
        assert!(
            stderr.contains(&native),
            "stderr must name {native:?}: {stderr}"
        );
    }

    // Fail-before-write: no outputs, and the cache store was never opened.
    let out_path = dir.join(out_dir);
    if out_path.exists() {
        let entries: Vec<_> = std::fs::read_dir(&out_path).unwrap().collect();
        assert!(
            entries.is_empty(),
            "no output may be written before the collision is rejected: {entries:?}"
        );
    }
    assert!(
        !dir.join(CACHE_DIR).exists(),
        "the cache store must not be created by a rejected build"
    );
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn colliding_stems_in_one_target_are_rejected() {
    let dir = same_stem_project("{stem}.{ext}");
    assert_rejected_before_write(dir.path(), "dist", &["a/logo.png", "b/logo.png"]);
}

#[test]
fn disambiguating_template_builds_cleanly() {
    let dir = same_stem_project("{parent}_{stem}.{ext}");
    let out = build(dir.path());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "a disambiguated build must exit 0\nstderr: {stderr}"
    );
    // Both inputs kept their own output.
    assert!(dir.path().join("dist/a_logo.png").exists(), "{stderr}");
    assert!(dir.path().join("dist/b_logo.png").exists(), "{stderr}");
}

#[test]
fn collision_across_two_targets_is_rejected() {
    // Two targets, two recipes' worth of sources, one `out` and one `name`: the
    // shared stem collides ACROSS targets, which a per-target check would miss.
    let dir = project(
        r#"
version = 1

[[target]]
source = "a/*.png"
recipe = "r.toml"
out = "dist"

[[target]]
source = "b/*.png"
recipe = "r.toml"
out = "dist"
"#,
    );
    write_png(dir.path(), "a/logo.png", 32, 32, [200, 30, 30]);
    write_png(dir.path(), "b/logo.png", 48, 48, [30, 30, 200]);

    assert_rejected_before_write(dir.path(), "dist", &["a/logo.png", "b/logo.png"]);
}

#[test]
fn non_colliding_build_unaffected() {
    // The regression guard: the check must add no false positives to a plain
    // multi-input, multi-target build with distinct stems.
    let dir = project(
        r#"
version = 1

[[target]]
source = "src/*.png"
recipe = "r.toml"
out = "dist"

[[target]]
source = "src/*.png"
recipe = "r.toml"
out = "thumbs"
name = "{stem}_t.{ext}"
"#,
    );
    write_png(dir.path(), "src/one.png", 32, 32, [200, 30, 30]);
    write_png(dir.path(), "src/two.png", 48, 48, [30, 30, 200]);

    let out = build(dir.path());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "must exit 0\nstderr: {stderr}");
    for rel in [
        "dist/one.png",
        "dist/two.png",
        "thumbs/one_t.png",
        "thumbs/two_t.png",
    ] {
        assert!(dir.path().join(rel).exists(), "missing {rel}\n{stderr}");
    }
}
