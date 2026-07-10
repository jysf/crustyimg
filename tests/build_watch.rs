//! Integration tests for `crustyimg build --watch` (SPEC-067, DEC-060).
//!
//! These drive the REAL compiled binary as a blocking child process against a temp
//! project, then observe the filesystem — because green exit-code tests miss the
//! defect class this feature is most exposed to: a self-trigger loop, a swallowed
//! failure, a rewritten lockfile. The unit tests in `src/build/watch.rs` prove the
//! pure logic deterministically; these prove the wiring end to end.
//!
//! Timing: file-watch is inherently asynchronous (FSEvents/inotify latency), so
//! every observation is a **poll-until** with a generous deadline, never a fixed
//! sleep-then-assert. The child's stderr is redirected to a log file kept OUTSIDE
//! the watched project tree, so reading it never perturbs the watcher.
//!
//! Gated on the `watch` feature: the lean `--no-default-features` binary has no
//! watch loop, so there is nothing to drive there.
#![cfg(feature = "watch")]

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use image::{DynamicImage, ImageFormat, RgbImage};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// The committed lockfile a default build writes at its working-dir root.
const LOCK: &str = "crustyimg.build.lock";

/// Resize every source to max 16px — small, fast, visible in the output dims.
const RESIZE_16: &str =
    "version = \"1\"\n\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 16\n";

/// The same recipe at a different size — a semantic change a rebuild must reflect.
const RESIZE_8: &str = "version = \"1\"\n\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 8\n";

/// A syntactically broken recipe — a build cycle over it fails (and, mid-watch,
/// must NOT kill the loop).
const BROKEN_RECIPE: &str = "this is not valid recipe toml {{{\n";

// ── Fixtures ────────────────────────────────────────────────────────────────────

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

/// A one-target project: two sources under `src/`, a resize recipe at the root,
/// outputs to `dist/`.
fn project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "r.toml", RESIZE_16.as_bytes());
    write_png(root, "src/a.png", 32, 32, [200, 30, 30]);
    write_png(root, "src/b.png", 48, 48, [30, 30, 200]);
    write_file(
        root,
        "crustyimg.build.toml",
        b"version = 1\n\n[[target]]\nsource = \"src/*.png\"\nrecipe = \"r.toml\"\nout = \"dist\"\n",
    );
    dir
}

// ── Child-process helpers ────────────────────────────────────────────────────────

/// Kills the child on drop, so a panicking test never leaks a watching process.
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Spawn `crustyimg build --watch [extra]` in `dir`, stderr → a log file kept in a
/// SEPARATE temp dir (never under the watched project, or the log write would wake
/// the watcher). Returns the guard, the log-holding TempDir, and the log path.
fn spawn_watch(dir: &Path, extra: &[&str]) -> (ChildGuard, TempDir, PathBuf) {
    let logdir = TempDir::new().unwrap();
    let logpath = logdir.path().join("watch.stderr.log");
    let errfile = std::fs::File::create(&logpath).unwrap();
    let child = Command::new(BIN)
        .arg("build")
        .arg("--watch")
        .args(extra)
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::from(errfile))
        .spawn()
        .expect("binary should spawn");
    (ChildGuard(child), logdir, logpath)
}

/// Run a plain (non-watch) `crustyimg build` to completion in `dir`.
fn build_once(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(BIN)
        .arg("build")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("binary should run")
}

// ── Poll helpers (generous deadlines; no fixed sleeps before asserting) ──────────

const DEADLINE: Duration = Duration::from_secs(45);
const TICK: Duration = Duration::from_millis(50);
/// How often `nudge_until` re-issues the mutation while waiting.
const NUDGE_EVERY: Duration = Duration::from_secs(2);

/// Poll `f` until it returns `true` or the deadline elapses. Returns whether it
/// became true in time.
fn poll_until(mut f: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < DEADLINE {
        if f() {
            return true;
        }
        std::thread::sleep(TICK);
    }
    f()
}

/// Apply `mutate`, then poll `cond` — re-applying `mutate` every `NUDGE_EVERY`
/// until `cond` holds or the deadline elapses. A single OS-watch event can be
/// dropped under startup churn (notably Linux inotify when the whole project tree
/// is watched while the cache dir is being populated); re-issuing the edit defeats
/// that race without weakening the assertion — the assertion still requires the
/// edit to *cause* the observable rebuild. Re-writing identical bytes is a cache
/// hit, so the observed output is unchanged once the first real rebuild lands.
fn nudge_until(mut mutate: impl FnMut(), mut cond: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    mutate();
    let mut last_nudge = Instant::now();
    while start.elapsed() < DEADLINE {
        if cond() {
            return true;
        }
        if last_nudge.elapsed() >= NUDGE_EVERY {
            mutate();
            last_nudge = Instant::now();
        }
        std::thread::sleep(TICK);
    }
    cond()
}

/// The max dimension of a PNG, or `None` if it does not exist / can't be read yet.
fn max_dim(path: &Path) -> Option<u32> {
    image::image_dimensions(path).ok().map(|(w, h)| w.max(h))
}

/// Read a file's bytes, or `None` if absent.
fn bytes(path: &Path) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

// ── Tests ────────────────────────────────────────────────────────────────────────

#[test]
fn watch_rebuilds_on_source_change() {
    let dir = project();
    let root = dir.path().to_path_buf();
    let out_a = root.join("dist/a.png");
    let out_b = root.join("dist/b.png");

    let (_guard, _logdir, _log) = spawn_watch(&root, &[]);

    // Initial build settles: both outputs exist at max-dim 16.
    assert!(
        poll_until(|| max_dim(&out_a) == Some(16) && max_dim(&out_b) == Some(16)),
        "initial build should produce both outputs at 16px"
    );
    let b_before = bytes(&out_b).expect("b exists");
    let a_before = bytes(&out_a).expect("a exists");

    // Edit ONE source to a new size + color; the watcher must rebuild it (its
    // bytes change). Re-issued until observed, to defeat a dropped startup event.
    assert!(
        nudge_until(
            || {
                write_png(&root, "src/a.png", 64, 64, [10, 220, 10]);
            },
            || bytes(&out_a).as_deref() != Some(a_before.as_slice()),
        ),
        "editing src/a.png should rebuild dist/a.png"
    );
    assert_eq!(max_dim(&out_a), Some(16), "a stays clamped to 16px");

    // b.png is untouched by an a.png edit (the cache prunes it) — its bytes are
    // byte-identical to before.
    assert_eq!(
        bytes(&out_b).as_deref(),
        Some(b_before.as_slice()),
        "dist/b.png must be unchanged when only src/a.png changed"
    );
}

#[test]
fn watch_does_not_self_trigger() {
    let dir = project();
    let root = dir.path().to_path_buf();
    let out_a = root.join("dist/a.png");

    let (_guard, _logdir, _log) = spawn_watch(&root, &[]);

    assert!(
        poll_until(|| max_dim(&out_a) == Some(16)),
        "initial build should settle"
    );
    // Snapshot the output once the initial build has settled.
    let settle = bytes(&out_a).expect("a exists");

    // Observe a quiet period WITHOUT touching any source. If the build's own writes
    // to dist/ / .crustyimg / the lockfile woke the watcher, the output would be
    // rewritten. It must not be.
    std::thread::sleep(Duration::from_secs(3));

    assert_eq!(
        bytes(&out_a).as_deref(),
        Some(settle.as_slice()),
        "the build must not rebuild itself during a quiet period (no self-trigger)"
    );
}

#[test]
fn watch_survives_a_failing_cycle() {
    let dir = project();
    let root = dir.path().to_path_buf();
    let out_a = root.join("dist/a.png");

    let (mut guard, _logdir, _log) = spawn_watch(&root, &[]);

    assert!(
        poll_until(|| max_dim(&out_a) == Some(16)),
        "initial build should settle at 16px"
    );

    // Mid-watch: break the recipe. The cycle fails; the loop must stay alive.
    write_file(&root, "r.toml", BROKEN_RECIPE.as_bytes());
    std::thread::sleep(Duration::from_secs(2));
    assert!(
        guard.0.try_wait().unwrap().is_none(),
        "a failing build cycle must NOT exit the watch loop"
    );

    // Now a valid edit (a different size): the loop recovers and rebuilds to 8px.
    // Re-issued until observed, to defeat a dropped watch event.
    assert!(
        nudge_until(
            || {
                write_file(&root, "r.toml", RESIZE_8.as_bytes());
            },
            || max_dim(&out_a) == Some(8),
        ),
        "after fixing the recipe, the loop should rebuild (now clamped to 8px)"
    );
}

#[test]
fn watch_does_not_write_the_lockfile() {
    let dir = project();
    let root = dir.path().to_path_buf();
    let out_a = root.join("dist/a.png");
    let lock = root.join(LOCK);

    // A normal build first, to COMMIT a lockfile.
    let out = build_once(&root, &[]);
    assert!(out.status.success(), "seed build should succeed");
    let lock_before = bytes(&lock).expect("a normal build writes a lockfile");

    // Now watch, and trigger a real rebuild.
    let (_guard, _logdir, _log) = spawn_watch(&root, &[]);
    assert!(
        poll_until(|| max_dim(&out_a) == Some(16)),
        "watch initial build should settle"
    );
    let a_before = bytes(&out_a).expect("a exists");
    assert!(
        nudge_until(
            || {
                write_png(&root, "src/a.png", 64, 64, [10, 220, 10]);
            },
            || bytes(&out_a).as_deref() != Some(a_before.as_slice()),
        ),
        "the source edit should rebuild the output"
    );

    // The committed lockfile is byte-identical: a watch cycle suppresses the write.
    assert_eq!(
        bytes(&lock).as_deref(),
        Some(lock_before.as_slice()),
        "a --watch cycle must NOT rewrite the committed lockfile"
    );
}

#[test]
fn watch_rejects_verify_modes() {
    let dir = project();
    let root = dir.path();

    // `--watch --check` (and its `--frozen` alias) is a usage error, exit 2, and
    // returns immediately — no watching begins, so no child to kill.
    for verify in [["--check"], ["--frozen"]] {
        let out = build_once(root, &[&["--watch"][..], &verify[..]].concat());
        assert_eq!(
            out.status.code(),
            Some(2),
            "build --watch {verify:?} must be a usage error (exit 2)"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("--watch cannot be combined"),
            "the message should explain the incompatibility, got: {stderr}"
        );
    }
}

#[test]
fn watch_starts_despite_a_broken_initial_build() {
    let dir = project();
    let root = dir.path().to_path_buf();
    let out_a = root.join("dist/a.png");

    // Break the recipe BEFORE starting, so the very first build cycle fails.
    write_file(&root, "r.toml", BROKEN_RECIPE.as_bytes());

    let (mut guard, _logdir, _log) = spawn_watch(&root, &[]);

    // A broken *initial build* is NOT a hard exit: the loop is entered so the user
    // can fix it and watch it recover. Give it a moment, then confirm it's alive.
    std::thread::sleep(Duration::from_secs(2));
    assert!(
        guard.0.try_wait().unwrap().is_none(),
        "a broken initial build must enter the watch loop, not hard-exit"
    );

    // Fixing the recipe rebuilds (re-issued until observed).
    assert!(
        nudge_until(
            || {
                write_file(&root, "r.toml", RESIZE_16.as_bytes());
            },
            || max_dim(&out_a) == Some(16),
        ),
        "after fixing the recipe, the first successful build should appear"
    );
}

#[test]
fn watch_hard_exits_on_a_missing_manifest() {
    // A MISSING manifest, by contrast to a broken build, IS a hard exit: there is
    // nothing to derive a watch set from. This returns immediately (exit 3).
    let dir = TempDir::new().unwrap();
    let out = build_once(dir.path(), &["--watch"]);
    assert_eq!(
        out.status.code(),
        Some(3),
        "build --watch with no manifest must hard-exit (input not found)"
    );
}
