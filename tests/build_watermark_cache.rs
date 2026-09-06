//! Integration tests for SPEC-129 — `build`'s cache key hashes a watermark
//! asset's own file CONTENT, not just the recipe's `to_toml()` (the path).
//!
//! Reproduces the SPEC-128 verify's exact bug as the RED baseline: editing an
//! overlay/font's bytes on disk (same path, same name in the manifest) must no
//! longer be served a stale cache hit. `apply --recipe` is out of scope by
//! construction (it has no cache, DEC-058) so every test here drives `build`.
//!
//! Conventions follow `tests/build_cache.rs`: the real compiled binary, a temp
//! project as CWD (`.crustyimg/cache/` and every manifest path resolve against
//! it, DEC-057), fixtures synthesized in memory — no committed binary files.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use image::{DynamicImage, ImageFormat, RgbImage};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Fixture helpers (duplicated from `tests/build_cache.rs` per the spec's
// own note: "reuse `write_png` ... or duplicate it") ─────────────────────────

/// Write raw bytes to `dir/rel`, creating parent dirs. Returns the path.
fn write_file(dir: &Path, rel: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Write a solid-color RGB PNG at `dir/rel`.
fn write_png(dir: &Path, rel: &str, w: u32, h: u32, rgb: [u8; 3]) -> PathBuf {
    let img = RgbImage::from_pixel(w, h, image::Rgb(rgb));
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .unwrap();
    write_file(dir, rel, &buf.into_inner())
}

/// Run `crustyimg build [args]` with `dir` as the working directory.
fn build(dir: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .arg("build")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("binary should run")
}

/// Run a build, assert it exited 0, and return `(cached, rebuilt)` from its summary.
fn build_ok(dir: &Path, args: &[&str]) -> (usize, usize) {
    let out = build(dir, args);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "build should exit 0, got {:?}\nstderr: {stderr}",
        out.status.code()
    );
    assert!(!stderr.contains("panicked"), "must not panic: {stderr}");
    parse_counts(&stderr)
}

/// Pull `(C, R)` out of a `... (C cached, R rebuilt)` summary line.
fn parse_counts(stderr: &str) -> (usize, usize) {
    let open = stderr
        .rfind(" (")
        .unwrap_or_else(|| panic!("no summary counts in stderr: {stderr}"));
    let close = stderr[open..]
        .find(')')
        .unwrap_or_else(|| panic!("unterminated summary counts: {stderr}"));
    let inner = &stderr[open + 2..open + close];
    let (c, r) = inner
        .split_once(", ")
        .unwrap_or_else(|| panic!("malformed summary counts {inner:?}"));
    let num = |s: &str| -> usize {
        s.split_whitespace()
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or_else(|| panic!("malformed count {s:?}"))
    };
    assert!(c.ends_with(" cached"), "expected 'N cached', got {c:?}");
    assert!(r.ends_with(" rebuilt"), "expected 'N rebuilt', got {r:?}");
    (num(c), num(r))
}

/// A two-input project whose one target watermarks with an image overlay at
/// `logo.png`. Outputs land at `dist/a.png` and `dist/b.png`.
fn image_watermark_project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_png(root, "logo.png", 8, 8, [10, 20, 30]);
    write_file(
        root,
        "r.toml",
        br#"
version = "1"

[[step]]
op = "watermark"
image = "logo.png"
gravity = "southeast"
opacity = 1.0
margin = 0
tile = false
"#,
    );
    write_png(root, "src/a.png", 32, 32, [200, 30, 30]);
    write_png(root, "src/b.png", 48, 48, [30, 30, 200]);
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/*.png"
recipe = "r.toml"
out = "dist"
"#,
    );
    dir
}

/// A two-input project whose one target watermarks with TEXT and an explicit
/// `font` key pointing at `font.ttf`.
fn font_watermark_project(font_bytes: &[u8]) -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "font.ttf", font_bytes);
    write_file(
        root,
        "r.toml",
        br#"
version = "1"

[[step]]
op = "watermark"
text = "mark"
font = "font.ttf"
"#,
    );
    write_png(root, "src/a.png", 32, 32, [200, 30, 30]);
    write_png(root, "src/b.png", 48, 48, [30, 30, 200]);
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/*.png"
recipe = "r.toml"
out = "dist"
"#,
    );
    dir
}

// ── AC-1: overlay bytes changing on disk must miss and rebuild ──────────────

/// The SPEC-128 verify's exact reproduction: mutate the overlay's bytes (same
/// path, same manifest, same recipe TOML), rebuild, and require a MISS — not
/// the stale byte-identical output `main` serves.
#[test]
fn build_rebuilds_when_overlay_bytes_change() {
    let dir = image_watermark_project();
    let root = dir.path();

    assert_eq!(build_ok(root, &[]), (0, 2), "a cold build rebuilds everything");
    let a_before = std::fs::read(root.join("dist/a.png")).unwrap();
    let b_before = std::fs::read(root.join("dist/b.png")).unwrap();

    // Same path, genuinely different pixels (not a resize — a content change).
    write_png(root, "logo.png", 8, 8, [250, 5, 5]);

    assert_eq!(
        build_ok(root, &[]),
        (0, 2),
        "an on-disk overlay edit must miss and rebuild both outputs sharing it \
         (main's bug: this reports (2, 0) and serves stale bytes)"
    );
    let a_after = std::fs::read(root.join("dist/a.png")).unwrap();
    let b_after = std::fs::read(root.join("dist/b.png")).unwrap();
    assert_ne!(
        a_after, a_before,
        "the rewritten output must reflect the new overlay bytes"
    );
    assert_ne!(
        b_after, b_before,
        "the rewritten output must reflect the new overlay bytes"
    );
}

// ── AC-2: positive control — unchanged overlay must still be a full hit ─────

/// Guards against over-invalidation: if the fix folded in something that
/// changes between runs even when the overlay is untouched (e.g. re-reading
/// nondeterministic metadata), this would flip from `(2, 0)` and break the
/// "no-change re-run is a full hit" headline (DEC-058).
#[test]
fn build_hits_when_overlay_bytes_unchanged() {
    let dir = image_watermark_project();
    let root = dir.path();

    build_ok(root, &[]);
    let a_before = std::fs::read(root.join("dist/a.png")).unwrap();
    let b_before = std::fs::read(root.join("dist/b.png")).unwrap();

    assert_eq!(
        build_ok(root, &[]),
        (2, 0),
        "an unchanged overlay must be a full cache hit"
    );
    assert_eq!(std::fs::read(root.join("dist/a.png")).unwrap(), a_before);
    assert_eq!(std::fs::read(root.join("dist/b.png")).unwrap(), b_before);
}

// ── AC-3: the `font` half of the mechanism ──────────────────────────────────

/// Same shape as AC-1, on the `font` asset key rather than `image`. Appending
/// one byte at EOF is a content change that a well-formed sfnt font (offset +
/// length addressed tables, never "read to EOF") still parses successfully —
/// so the rebuild succeeds while the content hash genuinely differs.
#[test]
fn build_rebuilds_when_font_bytes_change() {
    let font_v1 = crustyimg::text::DEFAULT_FONT;
    let dir = font_watermark_project(font_v1);
    let root = dir.path();

    assert_eq!(build_ok(root, &[]), (0, 2), "a cold build rebuilds everything");

    let mut font_v2 = font_v1.to_vec();
    font_v2.push(0);
    write_file(root, "font.ttf", &font_v2);

    assert_eq!(
        build_ok(root, &[]),
        (0, 2),
        "an on-disk font edit must miss and rebuild both outputs sharing it"
    );
}

/// AC-3's no-`font`-key subcase: a text watermark using the bundled default
/// font has nothing to resolve and nothing to hash, so it must keep hitting
/// exactly like any other watermark-free-of-assets recipe.
#[test]
fn build_hits_when_bundled_font_is_used() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(
        root,
        "r.toml",
        br#"
version = "1"

[[step]]
op = "watermark"
text = "mark"
"#,
    );
    write_png(root, "src/a.png", 32, 32, [200, 30, 30]);
    write_png(root, "src/b.png", 48, 48, [30, 30, 200]);
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/*.png"
recipe = "r.toml"
out = "dist"
"#,
    );

    build_ok(root, &[]);
    let a_before = std::fs::read(root.join("dist/a.png")).unwrap();
    assert_eq!(
        build_ok(root, &[]),
        (2, 0),
        "a bundled-default-font text watermark has nothing to resolve, so it \
         must hit exactly like a watermark-free recipe"
    );
    assert_eq!(std::fs::read(root.join("dist/a.png")).unwrap(), a_before);
}

// ── AC-5: missing-asset behavior is unchanged (a pin, not a new fix) ────────

/// `build` must still fail before touching any input when a step names an
/// unreadable asset — the same guarantee `prepare_target`'s pre-existing
/// `resolve_recipe_assets` call already provides (Call 6), asserted again
/// here so a future refactor cannot slip a hash computation into the failure
/// path.
#[test]
fn missing_overlay_still_fails_before_hashing() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(
        root,
        "r.toml",
        br#"
version = "1"

[[step]]
op = "watermark"
image = "does/not/exist.png"
"#,
    );
    write_png(root, "src/a.png", 16, 16, [1, 2, 3]);
    write_png(root, "src/b.png", 16, 16, [4, 5, 6]);
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/*.png"
recipe = "r.toml"
out = "dist"
"#,
    );

    let out = build(root, &[]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a missing recipe asset must exit 1 (bad recipe), not 3/6; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !root.join("dist").exists(),
        "no output may exist when the recipe's asset could not be resolved"
    );
    assert!(
        !root.join(".crustyimg").exists(),
        "the cache store must not even open before the recipe-level failure"
    );
}
