//! Integration tests for the `crustyimg build` reproducibility lockfile (SPEC-066).
//!
//! Each test drives the real compiled binary with a temp project as the working
//! directory, so `crustyimg.build.lock` — like every manifest path — resolves
//! against the CWD (DEC-057). Fixtures are synthesized in memory with the
//! `image` crate; no committed binary files.
//!
//! Note what is NOT here: a **real** cross-environment build. `[env].target` is
//! `"{ARCH}-{OS}"` of the running binary, so one test binary can never observe
//! two targets — exactly as it can never observe two `CARGO_PKG_VERSION`s
//! (SPEC-064). The cross-env *policy* is proven in the `diff` unit tests; the
//! test below proves the CLI honors it, by hand-editing the committed lockfile's
//! `[env].target` to a foreign arch.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use image::{DynamicImage, ImageFormat, RgbImage};
use tempfile::TempDir;

use crustyimg::build::lock::{BuildLock, DEFAULT_LOCK_FILE};

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// Resize every source to max 16px — tiny, fast, deterministic.
const RESIZE_16: &str = r#"
version = "1"

[[step]]
op = "resize"
mode = "max"
width = 16
"#;

const MANIFEST: &[u8] = br#"
version = 1

[[target]]
source = "src/*.png"
recipe = "r.toml"
out = "dist"
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
    write_file(root, "crustyimg.build.toml", MANIFEST);
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

/// Run a build and assert its exit code, returning stderr.
fn build_expect(dir: &Path, args: &[&str], code: i32) -> String {
    let out = build(dir, args);
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(!stderr.contains("panicked"), "must not panic: {stderr}");
    assert_eq!(
        out.status.code(),
        Some(code),
        "expected exit {code} for `build {args:?}`\nstderr: {stderr}"
    );
    stderr
}

fn lock_path(dir: &Path) -> PathBuf {
    dir.join(DEFAULT_LOCK_FILE)
}

fn read_lock_text(dir: &Path) -> String {
    std::fs::read_to_string(lock_path(dir)).expect("lockfile should exist")
}

fn read_lock(dir: &Path) -> BuildLock {
    BuildLock::from_toml(&read_lock_text(dir)).expect("a written lockfile must parse")
}

/// The lockfile's bytes + mtime — the pair `--check` must leave alone.
fn lock_fingerprint(dir: &Path) -> (String, std::time::SystemTime) {
    let text = read_lock_text(dir);
    let mtime = std::fs::metadata(lock_path(dir))
        .unwrap()
        .modified()
        .unwrap();
    (text, mtime)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn build_writes_lockfile() {
    let dir = project();
    let root = dir.path();
    build_expect(root, &[], 0);

    // The outputs themselves.
    assert!(root.join("dist/a.png").exists());
    assert!(root.join("dist/b.png").exists());

    let lock = read_lock(root);
    assert_eq!(lock.version, crustyimg::build::lock::SUPPORTED_LOCK_VERSION);

    // The `[env]` the hashes were observed under.
    assert_eq!(lock.env.crustyimg_version, crustyimg::version());
    assert_eq!(lock.env.target, crustyimg::build::lock::current_target());

    // One `[[output]]` per output, sorted by path, fully populated.
    let paths: Vec<&str> = lock.output.iter().map(|o| o.path.as_str()).collect();
    assert_eq!(paths, ["dist/a.png", "dist/b.png"]);

    for out in &lock.output {
        assert_eq!(out.recipe, "r.toml");
        assert!(out.source.starts_with("src/"), "{}", out.source);
        assert_eq!(out.key.len(), 64, "the pinned cache key is a sha256 hex");
        assert_eq!(out.hash.len(), 64, "the observed output hash likewise");
        assert!(out.key.chars().all(|c| c.is_ascii_hexdigit()));

        // `hash`/`bytes` describe the file that was actually written.
        let bytes = std::fs::read(root.join(&out.path)).expect("the named output exists");
        assert_eq!(out.bytes, bytes.len() as u64);
    }

    // The two outputs differ, so their keys and hashes must too.
    assert_ne!(lock.output[0].key, lock.output[1].key);
    assert_ne!(lock.output[0].hash, lock.output[1].hash);
}

#[test]
fn lockfile_is_deterministic() {
    let dir = project();
    let root = dir.path();

    build_expect(root, &[], 0);
    let first = read_lock_text(root);

    // A second build (fully cached) and a third that bypasses the cache entirely
    // must both reproduce the same lockfile, byte for byte: outputs are sorted,
    // and the key is computed with or without the store.
    build_expect(root, &[], 0);
    assert_eq!(
        read_lock_text(root),
        first,
        "a warm re-run must not churn it"
    );

    build_expect(root, &["--no-cache"], 0);
    assert_eq!(
        read_lock_text(root),
        first,
        "--no-cache must lock the same build"
    );
}

#[test]
fn check_passes_on_matching_tree() {
    let dir = project();
    let root = dir.path();
    build_expect(root, &[], 0);

    let before = lock_fingerprint(root);
    let stderr = build_expect(root, &["--check"], 0);
    assert!(stderr.contains("up to date"), "{stderr}");
    assert_eq!(lock_fingerprint(root), before, "--check must not write");
}

#[test]
fn check_fails_on_edited_source() {
    let dir = project();
    let root = dir.path();
    build_expect(root, &[], 0);
    let before = lock_fingerprint(root);

    // A different source → a different content hash → a different cache key.
    write_png(root, "src/a.png", 32, 32, [7, 7, 7]);

    let stderr = build_expect(root, &["--check"], 7);
    assert!(
        stderr.contains("dist/a.png"),
        "names the drifted output: {stderr}"
    );
    assert!(stderr.contains("inputs changed"), "{stderr}");
    assert!(!stderr.contains("dist/b.png"), "b did not drift: {stderr}");
    assert_eq!(
        lock_fingerprint(root),
        before,
        "a failing --check must not rewrite the lockfile"
    );

    // A plain build accepts the change and refreshes the lock; --check then passes.
    build_expect(root, &[], 0);
    assert_ne!(read_lock_text(root), before.0);
    build_expect(root, &["--check"], 0);
}

#[test]
fn check_fails_on_added_or_removed_output() {
    let dir = project();
    let root = dir.path();
    build_expect(root, &[], 0);
    let pinned = lock_fingerprint(root);

    // A new source under the same glob → an output the lockfile does not pin.
    write_png(root, "src/c.png", 20, 20, [9, 90, 9]);
    let stderr = build_expect(root, &["--check"], 7);
    assert!(stderr.contains("dist/c.png"), "{stderr}");
    assert!(stderr.contains("not in the lockfile"), "{stderr}");
    assert_eq!(lock_fingerprint(root), pinned, "--check must not write");

    // Re-pin with a build, then remove a source: the lockfile pins an output the
    // build no longer produces.
    build_expect(root, &[], 0);
    std::fs::remove_file(root.join("src/c.png")).unwrap();
    let stderr = build_expect(root, &["--check"], 7);
    assert!(stderr.contains("dist/c.png"), "{stderr}");
    assert!(stderr.contains("not built"), "{stderr}");
}

#[test]
fn frozen_without_lockfile_fails() {
    let dir = project();
    let root = dir.path();
    assert!(!lock_path(root).exists());

    let stderr = build_expect(root, &["--frozen"], 7);
    assert!(stderr.contains("no lockfile"), "{stderr}");
    assert!(
        stderr.contains("run `crustyimg build` to create one"),
        "the message must be actionable: {stderr}"
    );

    // Fail-before-write: a missing lockfile is caught in the prepare phase.
    assert!(!lock_path(root).exists(), "--frozen must never create one");
    assert!(
        !root.join("dist").exists(),
        "no output before the check fails"
    );

    // `--locked` is the same mode under a different name.
    let stderr = build_expect(root, &["--locked"], 7);
    assert!(stderr.contains("no lockfile"), "{stderr}");
}

#[test]
fn frozen_matching_passes_and_does_not_write() {
    let dir = project();
    let root = dir.path();
    build_expect(root, &[], 0);
    let before = lock_fingerprint(root);

    build_expect(root, &["--frozen"], 0);
    assert_eq!(lock_fingerprint(root), before, "--frozen must not write");

    // A drifting tree: --frozen fails, and STILL leaves the lockfile untouched.
    write_png(root, "src/b.png", 48, 48, [1, 2, 3]);
    build_expect(root, &["--frozen"], 7);
    assert_eq!(
        lock_fingerprint(root),
        before,
        "a failing --frozen must not update the lockfile"
    );
}

#[test]
fn cross_env_hash_tolerated_unless_strict() {
    let dir = project();
    let root = dir.path();
    build_expect(root, &[], 0);

    // Hand-edit the committed lock: claim it was recorded on a foreign arch, and
    // corrupt one output hash. The keys still match (the inputs did not change),
    // so this is exactly the cross-environment encoder variance the lockfile
    // records rather than promises.
    let lock = read_lock(root);
    let text = read_lock_text(root)
        .replace(&lock.env.target, "foreignarch-foreignos")
        .replace(&lock.output[0].hash, &"0".repeat(64));
    std::fs::write(lock_path(root), &text).unwrap();
    let before = lock_fingerprint(root);

    // Default: informational. Reported, but exit 0 — CI does not fail on bytes an
    // encoder never promised across arches.
    let stderr = build_expect(root, &["--check"], 0);
    assert!(
        stderr.contains("note: "),
        "informational, not drift: {stderr}"
    );
    assert!(stderr.contains("dist/a.png"), "{stderr}");
    assert!(stderr.contains("differ across environments"), "{stderr}");
    assert!(
        stderr.contains("--strict"),
        "the note points at the escape: {stderr}"
    );

    // `--strict` promotes exactly that change to a failure.
    let stderr = build_expect(root, &["--check", "--strict"], 7);
    assert!(stderr.contains("drift: "), "{stderr}");
    assert!(stderr.contains("dist/a.png"), "{stderr}");

    assert_eq!(lock_fingerprint(root), before, "neither mode may write");
}

#[test]
fn same_env_hash_drift_fails_check() {
    let dir = project();
    let root = dir.path();
    build_expect(root, &[], 0);

    // Same env, same key, a different recorded output hash: a real regression on
    // this machine. (Simulated by editing the lock — the encoder is deterministic
    // within a machine, so a build cannot produce this on its own.)
    let lock = read_lock(root);
    let text = read_lock_text(root).replace(&lock.output[0].hash, &"0".repeat(64));
    std::fs::write(lock_path(root), text).unwrap();

    let stderr = build_expect(root, &["--check"], 7);
    assert!(stderr.contains("drift: "), "{stderr}");
    assert!(
        stderr.contains("different output bytes on this machine"),
        "{stderr}"
    );
}

/// The lockfile is the second line of defense against a non-injective build.
///
/// SPEC-065's prepare-phase check runs *before* any decode, so it cannot expand
/// `{ext}`: a target naming a **literal** extension (`{stem}.png`) and one naming
/// `{stem}.{ext}` have different collision keys, yet both resolve to `dist/logo.png`
/// once the real extension is known. Here it is known — so the build refuses to pin
/// two outputs to one path (exit 2, no lockfile), rather than locking an ambiguity.
#[test]
fn literal_ext_collision_is_caught_before_the_lock_is_written() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "r.toml", RESIZE_16.as_bytes());
    write_png(root, "s1/logo.png", 32, 32, [200, 0, 0]);
    write_png(root, "s2/logo.png", 32, 32, [0, 0, 200]);
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "s1/logo.png"
recipe = "r.toml"
out = "dist"
name = "{stem}.png"

[[target]]
source = "s2/logo.png"
recipe = "r.toml"
out = "dist"
name = "{stem}.{ext}"
"#,
    );

    let stderr = build_expect(root, &[], 2);
    assert!(stderr.contains("output collision"), "{stderr}");
    assert!(stderr.contains("dist/logo.png"), "{stderr}");
    assert!(
        stderr.contains("s1/logo.png") && stderr.contains("s2/logo.png"),
        "{stderr}"
    );
    assert!(
        !lock_path(root).exists(),
        "an ambiguous build must not be pinned"
    );
}

#[test]
fn malformed_lockfile_is_exit_2() {
    let dir = project();
    let root = dir.path();

    // A well-formed lockfile at a given version, so each fixture below isolates
    // exactly one defect.
    let well_formed = |version: u32, extra: &str| {
        format!(
            "version = {version}\n{extra}\n[env]\ncrustyimg_version = \"{}\"\ntarget = \"{}\"\nfeatures = \"\"\n",
            crustyimg::version(),
            crustyimg::build::lock::current_target()
        )
    };

    // An unknown field: `deny_unknown_fields`, like the manifest and recipes.
    let bad = well_formed(1, "bogus = 1\n");
    std::fs::write(lock_path(root), &bad).unwrap();

    let stderr = build_expect(root, &["--check"], 2);
    assert!(
        stderr.contains("bogus"),
        "names the unknown field: {stderr}"
    );

    // Fail-before-write: the committed lockfile is parsed in the prepare phase.
    assert!(
        !root.join("dist").exists(),
        "no output before the parse fails"
    );
    assert_eq!(read_lock_text(root), bad, "and the lockfile is untouched");

    // An unsupported version is likewise a content error, not a failed check.
    // (Unknown fields are rejected during the parse, before the version gate, so
    // this fixture must be otherwise well-formed to reach it.)
    std::fs::write(lock_path(root), well_formed(999, "")).unwrap();
    let stderr = build_expect(root, &["--check"], 2);
    assert!(stderr.contains("unsupported"), "{stderr}");

    // A plain build owns the file: it regenerates a malformed lockfile rather
    // than refusing to build (it never reads one).
    build_expect(root, &[], 0);
    read_lock(root);
}

#[test]
fn non_hex_digest_in_lock_is_exit_2_not_a_panic() {
    // Regression: `key`/`hash` arrive from a hand-editable committed file. A digest
    // with a multi-byte char used to reach `short()` and panic on a mid-char byte
    // slice (exit 101). It must instead be a typed content error (exit 2), because
    // the realistic vector is a PR that edits crustyimg.build.lock — a CI gate must
    // not turn into a Rust panic trace.
    let dir = project();
    let root = dir.path();
    build_expect(root, &[], 0);

    let lock = read_lock(root);
    let text =
        read_lock_text(root).replace(&lock.output[0].key, "a\u{20ac}\u{20ac}\u{20ac}\u{20ac}");
    std::fs::write(lock_path(root), &text).unwrap();

    let stderr = build_expect(root, &["--check"], 2);
    assert!(
        stderr.contains("hex"),
        "the non-hex digest is named, not a panic: {stderr}"
    );
    assert!(
        !stderr.contains("panicked"),
        "must be a typed error, not a panic: {stderr}"
    );
}
