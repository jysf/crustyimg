//! Integration tests for the `crustyimg build` content-addressed cache (SPEC-064).
//!
//! Each test drives the real compiled binary with a temp project as the working
//! directory, so `.crustyimg/cache/` — like every manifest path — resolves
//! against the CWD (DEC-057). Fixtures are synthesized in memory with the
//! `image` crate; no committed binary files.
//!
//! Note what is NOT here: **version invalidation**. The shipped key folds in
//! `env!("CARGO_PKG_VERSION")`, a compile-time const, so one test binary can
//! never observe two crustyimg versions. That criterion is proven where it can
//! be — the `compute_key` unit test in `src/build/cache.rs`.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use image::{DynamicImage, ImageFormat, RgbImage};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// The cache root a build creates under its working directory.
const CACHE_DIR: &str = ".crustyimg/cache";

/// Resize every source to max 16px — tiny, fast, and visible in the output dims.
const RESIZE_16: &str = r#"
version = "1"

[[step]]
op = "resize"
mode = "max"
width = 16
"#;

/// The same recipe at a different size — a semantic param change (a cache miss).
const RESIZE_8: &str = r#"
version = "1"

[[step]]
op = "resize"
mode = "max"
width = 8
"#;

// ── Fixture helpers ───────────────────────────────────────────────────────────

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

/// A one-target project with two distinct source PNGs and a resize recipe.
/// Outputs land at `dist/a.png` and `dist/b.png`.
fn project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "r.toml", RESIZE_16.as_bytes());
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

/// Assert an output exists with the expected dimensions.
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

/// Every committed cache entry, i.e. every file under a shard dir (not `tmp/`).
fn cache_entries(root: &Path) -> Vec<PathBuf> {
    let cache = root.join(CACHE_DIR);
    let mut entries = Vec::new();
    let Ok(shards) = std::fs::read_dir(&cache) else {
        return entries;
    };
    for shard in shards.flatten() {
        if !shard.path().is_dir() || shard.file_name() == "tmp" {
            continue;
        }
        for entry in std::fs::read_dir(shard.path())
            .into_iter()
            .flatten()
            .flatten()
        {
            entries.push(entry.path());
        }
    }
    entries.sort();
    entries
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// The headline: a second run with no changes is a FULL cache hit — every output
/// reported cached, zero rebuilt, and the outputs are byte-identical.
#[test]
fn second_run_is_all_cache_hits() {
    let dir = project();
    let root = dir.path();

    assert_eq!(
        build_ok(root, &[]),
        (0, 2),
        "a cold build rebuilds everything"
    );
    let first_a = std::fs::read(root.join("dist/a.png")).unwrap();
    let first_b = std::fs::read(root.join("dist/b.png")).unwrap();

    assert_eq!(
        build_ok(root, &[]),
        (2, 0),
        "an unchanged re-run must do zero work"
    );
    assert_eq!(std::fs::read(root.join("dist/a.png")).unwrap(), first_a);
    assert_eq!(std::fs::read(root.join("dist/b.png")).unwrap(), first_b);
    assert_dims(&root.join("dist/a.png"), 16, 16);
}

/// Changing ONE source's bytes rebuilds only that output; the rest stay hits and
/// stay byte-identical.
#[test]
fn changing_one_input_rebuilds_only_that_output() {
    let dir = project();
    let root = dir.path();

    build_ok(root, &[]);
    let untouched_before = std::fs::read(root.join("dist/b.png")).unwrap();
    let changed_before = std::fs::read(root.join("dist/a.png")).unwrap();

    // Same dimensions, different pixels → different content hash.
    write_png(root, "src/a.png", 32, 32, [10, 220, 90]);

    assert_eq!(
        build_ok(root, &[]),
        (1, 1),
        "exactly the changed input rebuilds"
    );
    assert_ne!(
        std::fs::read(root.join("dist/a.png")).unwrap(),
        changed_before,
        "the changed source's output must be rewritten"
    );
    assert_eq!(
        std::fs::read(root.join("dist/b.png")).unwrap(),
        untouched_before,
        "an untouched source's output must be byte-identical"
    );
}

/// Changing a recipe PARAM misses and rebuilds every output that recipe
/// produces, and the new outputs reflect the new recipe (not cached old bytes).
#[test]
fn changing_recipe_param_forces_rebuild() {
    let dir = project();
    let root = dir.path();

    build_ok(root, &[]);
    assert_dims(&root.join("dist/a.png"), 16, 16);

    write_file(root, "r.toml", RESIZE_8.as_bytes());

    assert_eq!(build_ok(root, &[]), (0, 2), "a recipe change misses");
    assert_dims(&root.join("dist/a.png"), 8, 8);
    assert_dims(&root.join("dist/b.png"), 8, 8);

    // The recipe hash is over the canonical PARSED recipe, so a comment-only
    // edit is still a hit — that is the point of hashing the parsed form (DEC-005).
    let cosmetic = format!("# just a comment\n{RESIZE_8}\n\n");
    write_file(root, "r.toml", cosmetic.as_bytes());
    assert_eq!(
        build_ok(root, &[]),
        (2, 0),
        "a cosmetic recipe edit must not bust the cache"
    );
}

/// Changing `--quality` misses — safe over-invalidation, even for a format like
/// PNG that ignores the quality knob entirely.
#[test]
fn changing_quality_forces_rebuild() {
    let dir = project();
    let root = dir.path();

    build_ok(root, &[]);
    assert_eq!(
        build_ok(root, &["-q", "60"]),
        (0, 2),
        "a new quality misses"
    );
    // ... and the new quality is itself keyed, so it hits on its own re-run.
    assert_eq!(build_ok(root, &["-q", "60"]), (2, 0));
    // Dropping back to no `-q` is a distinct key again — `None` is not `Some(q)`.
    assert_eq!(build_ok(root, &[]), (2, 0), "the original key still hits");
}

/// A hit materializes a byte-correct output: delete an output, keep the cache,
/// and the re-run restores it byte-for-byte rather than leaving it missing.
#[test]
fn hit_materializes_byte_correct_output() {
    let dir = project();
    let root = dir.path();

    build_ok(root, &[]);
    let original = std::fs::read(root.join("dist/a.png")).unwrap();

    std::fs::remove_file(root.join("dist/a.png")).unwrap();
    assert!(!root.join("dist/a.png").exists());

    assert_eq!(
        build_ok(root, &[]),
        (2, 0),
        "a deleted output is still a hit"
    );
    assert_eq!(
        std::fs::read(root.join("dist/a.png")).unwrap(),
        original,
        "the restored output must be byte-identical to the built one"
    );
}

/// A corrupt cache entry falls back to a clean rebuild: correct output, exit 0,
/// no panic. Never a stale or garbage artifact served from a bad entry.
#[test]
fn corrupt_cache_entry_triggers_clean_rebuild() {
    let dir = project();
    let root = dir.path();

    build_ok(root, &[]);
    let good_output = std::fs::read(root.join("dist/a.png")).unwrap();

    let entries = cache_entries(root);
    assert_eq!(entries.len(), 2, "one entry per distinct input");

    // Flip the last byte of every entry's payload — the recorded output-hash no
    // longer matches, so verify-on-read must reject each one.
    for entry in &entries {
        let mut bytes = std::fs::read(entry).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;
        std::fs::write(entry, &bytes).unwrap();
    }
    // Remove the outputs so a stale entry cannot hide behind an existing file.
    std::fs::remove_dir_all(root.join("dist")).unwrap();

    assert_eq!(
        build_ok(root, &[]),
        (0, 2),
        "corrupt entries must all miss and rebuild"
    );
    assert_eq!(
        std::fs::read(root.join("dist/a.png")).unwrap(),
        good_output,
        "the rebuilt output must be correct, not the corrupted bytes"
    );

    // The rebuild also repaired the store, so the next run hits again.
    assert_eq!(build_ok(root, &[]), (2, 0));
}

/// A truncated entry, and an entry replaced by junk, are misses too — the store
/// is treated as untrusted input, and a malformed frame never panics.
#[test]
fn malformed_cache_entries_are_misses_not_panics() {
    let dir = project();
    let root = dir.path();
    build_ok(root, &[]);

    for entry in cache_entries(root) {
        let bytes = std::fs::read(&entry).unwrap();
        std::fs::write(&entry, &bytes[..bytes.len() / 2]).unwrap(); // truncated
    }
    assert_eq!(build_ok(root, &[]), (0, 2), "truncated entries must miss");

    for entry in cache_entries(root) {
        std::fs::write(&entry, b"not a cache entry at all").unwrap();
    }
    assert_eq!(build_ok(root, &[]), (0, 2), "junk entries must miss");
}

/// `--no-cache` bypasses the store in both directions: nothing is written under
/// `.crustyimg/`, and every input rebuilds on every run.
#[test]
fn no_cache_flag_bypasses_store() {
    let dir = project();
    let root = dir.path();

    assert_eq!(build_ok(root, &["--no-cache"]), (0, 2));
    assert!(
        !root.join(".crustyimg").exists(),
        "--no-cache must not create the store"
    );

    assert_eq!(
        build_ok(root, &["--no-cache"]),
        (0, 2),
        "--no-cache rebuilds every input on a repeat run"
    );
    assert!(!root.join(".crustyimg").exists());
    assert_dims(&root.join("dist/a.png"), 16, 16);

    // A cached build populates the store; --no-cache then ignores it (rebuilds)
    // without disturbing it.
    assert_eq!(build_ok(root, &[]), (0, 2));
    let before = cache_entries(root);
    assert_eq!(before.len(), 2);
    assert_eq!(
        build_ok(root, &["--no-cache"]),
        (0, 2),
        "a populated store is bypassed"
    );
    assert_eq!(cache_entries(root), before, "--no-cache writes no entries");
}

/// The store is a local directory under the project — `.crustyimg/cache/`,
/// sharded, hex-named. There is no remote/networked code path to exercise.
#[test]
fn cache_store_is_local_under_project() {
    let dir = project();
    let root = dir.path();
    build_ok(root, &[]);

    let cache = root.join(CACHE_DIR);
    assert!(cache.is_dir(), "the store lives under the project dir");

    let entries = cache_entries(root);
    assert_eq!(
        entries.len(),
        2,
        "the store is populated, one entry per input"
    );

    for entry in &entries {
        // <cache>/<2 hex>/<64 hex> — no user-controlled component anywhere.
        let name = entry.file_name().unwrap().to_str().unwrap();
        let shard = entry
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(name.len(), 64, "entry names are full hex keys");
        assert_eq!(shard.len(), 2, "entries are sharded by a 2-hex prefix");
        assert!(name.starts_with(shard));
        assert!(
            name.bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
            "entry name must be lowercase hex: {name}"
        );
        assert!(entry.starts_with(&cache));
    }
}

/// `--quiet` suppresses the cache summary, and the summary keeps the shape the
/// SPEC-063 tests rely on (`built N targets, M outputs`) with the counts appended.
#[test]
fn summary_reports_cached_and_rebuilt_and_quiet_suppresses_it() {
    let dir = project();
    let root = dir.path();

    let out = build(root, &[]);
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("built 1 target, 2 outputs (0 cached, 2 rebuilt)"),
        "unexpected cold-build summary: {stderr}"
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).is_empty(),
        "build must not write diagnostics to stdout"
    );

    let out = build(root, &[]);
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr)
            .contains("built 1 target, 2 outputs (2 cached, 0 rebuilt)"),
        "unexpected warm-build summary"
    );

    let quiet = build(root, &["--quiet"]);
    assert!(quiet.status.success());
    assert!(
        String::from_utf8_lossy(&quiet.stderr).is_empty(),
        "--quiet must suppress the summary"
    );
}
