//! `build` takes its outputs from the manifest, so a global flag that would set an
//! output's path, name or format is refused (exit 2) instead of silently ignored.
//!
//! These drive the REAL compiled binary against a temp project and compare a
//! snapshot of every file under it (path -> SHA-256, `.crustyimg/` and the lockfile
//! included) before and after, because an exit code alone cannot show that nothing
//! was written. `build_still_honours_its_own_flags` proves the snapshot can see a
//! write, so the "unchanged" assertions elsewhere are not vacuous.

use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use image::{DynamicImage, ImageFormat, RgbImage};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// The PNG signature: formats below are read from magic bytes, never from extensions.
const PNG_MAGIC: &[u8] = b"\x89PNG";

/// Resize every source to max 32px.
const RESIZE_32: &str =
    "version = \"1\"\n\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 32\n";

/// The same recipe at a different size: a semantic change that makes a lockfile stale.
const RESIZE_16: &str =
    "version = \"1\"\n\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 16\n";

/// A v2 recipe whose own `format` pins the output.
const RESIZE_32_GIF: &str =
    "version = \"2\"\nformat = \"gif\"\n\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 32\n";

/// Three targets: the format pinned by the name's extension, pinned by the recipe,
/// and not pinned at all (the output follows the source).
const MANIFEST: &str = "version = 1\n\n\
[[target]]\nsource = \"src/*.png\"\nrecipe = \"v1.toml\"\nout = \"out/ext\"\nname = \"{stem}.webp\"\n\n\
[[target]]\nsource = \"src/*.png\"\nrecipe = \"v2gif.toml\"\nout = \"out/recipefmt\"\n\n\
[[target]]\nsource = \"src/*.png\"\nrecipe = \"v1.toml\"\nout = \"out/neither\"\n";

/// The exact stderr line each refused flag prints, newline included.
const LINE_OUTPUT: &str = "error: build does not take -o/--output: the manifest says where a build writes; set the target's out and name instead\n";
const LINE_OUT_DIR: &str = "error: build does not take --out-dir: the manifest says where a build writes; set the target's out instead\n";
const LINE_NAME_TEMPLATE: &str = "error: build does not take --name-template: the manifest names a build's outputs; set the target's name instead\n";
const LINE_FORMAT: &str = "error: build does not take --format: the manifest sets a build's output format; give the target's name an extension (name = \"{stem}.png\") or set format in its recipe instead\n";

/// How long a run that should refuse immediately may take before the test gives up.
const DEADLINE: Duration = Duration::from_secs(20);
const TICK: Duration = Duration::from_millis(50);

// ── Fixtures ────────────────────────────────────────────────────────────────────

fn write_file(dir: &Path, rel: &str, bytes: &[u8]) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, bytes).unwrap();
}

fn encode(format: ImageFormat) -> Vec<u8> {
    let img = RgbImage::from_pixel(48, 36, image::Rgb([200, 30, 30]));
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, format)
        .unwrap();
    buf.into_inner()
}

/// The three-target project, with the manifest and recipes but nothing built yet.
fn project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "v1.toml", RESIZE_32.as_bytes());
    write_file(root, "v2gif.toml", RESIZE_32_GIF.as_bytes());
    write_file(root, "src/a.png", &encode(ImageFormat::Png));
    write_file(root, "crustyimg.build.toml", MANIFEST.as_bytes());
    dir
}

/// Path -> SHA-256 of every file under `root`.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, [u8; 32]> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, [u8; 32]>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(base, &path, out);
            } else {
                let digest: [u8; 32] = Sha256::digest(std::fs::read(&path).unwrap()).into();
                out.insert(path.strip_prefix(base).unwrap().to_path_buf(), digest);
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(BIN)
        .args(args)
        .current_dir(dir)
        .output()
        .expect("binary should run")
}

/// Kills the child on drop, so a panicking test never leaks a watching process.
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Run `crustyimg args` to completion under [`DEADLINE`], returning its exit code
/// and stderr. `.output()` would wait forever on a child that starts watching, so
/// this polls `try_wait` and fails the test, killing the child, on timeout. Stderr
/// goes to a file outside the project so reading it never perturbs a watcher.
fn run_with_deadline(dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    let logdir = TempDir::new().unwrap();
    let logpath = logdir.path().join("stderr.log");
    let errfile = std::fs::File::create(&logpath).unwrap();
    let child = Command::new(BIN)
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::from(errfile))
        .spawn()
        .expect("binary should spawn");
    let mut guard = ChildGuard(child);
    let start = Instant::now();
    let status = loop {
        if let Some(status) = guard.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            start.elapsed() < DEADLINE,
            "`crustyimg {}` is still running after {DEADLINE:?}: it started watching",
            args.join(" ")
        );
        std::thread::sleep(TICK);
    };
    (status.code(), std::fs::read_to_string(&logpath).unwrap())
}

/// Assert a run exited 2 with exactly `expected` on stderr.
fn assert_refused(out: &std::process::Output, expected: &str, what: &str) {
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(2),
        "{what}: exit code; stderr:\n{stderr}"
    );
    assert_eq!(stderr, expected, "{what}: stderr");
}

/// Run `args` in a fresh project and assert it is refused with `expected` and that
/// no file under the project changed.
fn assert_refused_and_untouched(args: &[&str], expected: &str) {
    let dir = project();
    let before = snapshot(dir.path());
    let out = run(dir.path(), args);
    assert_refused(&out, expected, &args.join(" "));
    assert_eq!(
        snapshot(dir.path()),
        before,
        "{}: wrote files",
        args.join(" ")
    );
}

// ── Refusals ────────────────────────────────────────────────────────────────────

#[test]
fn build_refuses_format_and_writes_nothing() {
    assert_refused_and_untouched(&["build", "--format", "png"], LINE_FORMAT);
}

#[test]
fn build_refuses_output_in_every_spelling() {
    for args in [
        &["build", "-o", "x.png"][..],
        &["build", "--output", "x.png"],
        &["build", "--output=x.png"],
        &["build", "-ox.png"],
    ] {
        let dir = project();
        let before = snapshot(dir.path());
        let out = run(dir.path(), args);
        assert_refused(&out, LINE_OUTPUT, &args.join(" "));
        assert_eq!(
            snapshot(dir.path()),
            before,
            "{}: wrote files",
            args.join(" ")
        );
        assert!(
            !dir.path().join("x.png").exists(),
            "{}: wrote x.png",
            args.join(" ")
        );
    }
}

#[test]
fn build_refuses_out_dir() {
    let dir = project();
    let before = snapshot(dir.path());
    let out = run(dir.path(), &["build", "--out-dir", "other"]);
    assert_refused(&out, LINE_OUT_DIR, "--out-dir other");
    assert_eq!(snapshot(dir.path()), before);
    assert!(!dir.path().join("other").exists(), "created other/");
}

#[test]
fn build_refuses_name_template() {
    assert_refused_and_untouched(
        &["build", "--name-template", "{stem}.png"],
        LINE_NAME_TEMPLATE,
    );
}

#[test]
fn build_refuses_a_flag_given_its_default_its_own_or_an_empty_value() {
    for (args, line) in [
        (
            &["build", "--name-template", "{stem}.{ext}"][..],
            LINE_NAME_TEMPLATE,
        ),
        (&["build", "-o", ""], LINE_OUTPUT),
        (&["build", "--out-dir", "."], LINE_OUT_DIR),
        (&["build", "--format", "jpeg"], LINE_FORMAT),
        (&["build", "--format", ""], LINE_FORMAT),
        (&["build", "--format", "bogus"], LINE_FORMAT),
    ] {
        assert_refused_and_untouched(args, line);
    }
}

#[test]
fn build_refuses_a_flag_given_before_the_verb() {
    for (args, line) in [
        (&["--format", "png", "build"][..], LINE_FORMAT),
        (&["--out-dir", "other", "build"], LINE_OUT_DIR),
    ] {
        assert_refused_and_untouched(args, line);
    }
}

#[test]
fn build_names_every_refused_flag_in_a_fixed_order() {
    // Given in the order --format, --name-template, -o, --out-dir; reported in the
    // order -o, --out-dir, --name-template, --format.
    let all = [LINE_OUTPUT, LINE_OUT_DIR, LINE_NAME_TEMPLATE, LINE_FORMAT].concat();
    assert_refused_and_untouched(
        &[
            "build",
            "--format",
            "png",
            "--name-template",
            "{stem}.png",
            "-o",
            "x.png",
            "--out-dir",
            "other",
        ],
        &all,
    );
    let two = [LINE_OUT_DIR, LINE_FORMAT].concat();
    assert_refused_and_untouched(&["build", "--format", "png", "--out-dir", "other"], &two);
}

// ── Error order ─────────────────────────────────────────────────────────────────

#[test]
fn the_refusal_comes_before_a_missing_manifest() {
    let dir = TempDir::new().unwrap();
    let out = run(dir.path(), &["build", "--format", "png"]);
    assert_refused(&out, LINE_FORMAT, "no manifest");
}

#[test]
fn the_refusal_comes_before_a_malformed_manifest() {
    let dir = TempDir::new().unwrap();
    write_file(
        dir.path(),
        "crustyimg.build.toml",
        b"version = 1\nbogus = 3\n",
    );
    let out = run(dir.path(), &["build", "--out-dir", "other"]);
    assert_refused(&out, LINE_OUT_DIR, "malformed manifest");
    assert!(!String::from_utf8_lossy(&out.stderr).contains("could not parse build manifest"));
}

#[test]
fn the_refusal_comes_before_a_stale_lockfile_check() {
    let dir = project();
    let built = run(dir.path(), &["build", "-Q"]);
    assert_eq!(built.status.code(), Some(0), "initial build");
    write_file(dir.path(), "v1.toml", RESIZE_16.as_bytes());
    // Control: the lockfile really is stale, so the 2 below is the guard's, not 7's.
    let stale = run(dir.path(), &["build", "--check"]);
    assert_eq!(
        stale.status.code(),
        Some(7),
        "plain --check on a stale lockfile"
    );
    let before = snapshot(dir.path());
    for (args, line) in [
        (&["build", "--check", "--format", "png"][..], LINE_FORMAT),
        (&["build", "--frozen", "-o", "x.png"], LINE_OUTPUT),
    ] {
        let out = run(dir.path(), args);
        assert_refused(&out, line, &args.join(" "));
        assert_eq!(
            snapshot(dir.path()),
            before,
            "{}: wrote files",
            args.join(" ")
        );
    }
}

// ── --watch ─────────────────────────────────────────────────────────────────────

#[test]
fn build_watch_refuses_and_never_starts_watching() {
    for (flag, value, line) in [
        ("--format", "png", LINE_FORMAT),
        ("--name-template", "{stem}.png", LINE_NAME_TEMPLATE),
    ] {
        let dir = project();
        let before = snapshot(dir.path());
        let (code, stderr) = run_with_deadline(dir.path(), &["build", "--watch", flag, value]);
        assert_eq!(
            code,
            Some(2),
            "build --watch {flag}: exit code; stderr:\n{stderr}"
        );
        assert_eq!(stderr, line, "build --watch {flag}: stderr");
        assert_eq!(
            snapshot(dir.path()),
            before,
            "build --watch {flag}: wrote files"
        );
    }
}

#[test]
fn the_refusal_comes_before_the_watch_check_conflict() {
    let dir = project();
    let (code, stderr) =
        run_with_deadline(dir.path(), &["build", "--watch", "--check", "-o", "x.png"]);
    assert_eq!(code, Some(2), "exit code; stderr:\n{stderr}");
    assert_eq!(stderr, LINE_OUTPUT);
    assert!(!stderr.contains("--watch cannot be combined"));
}

// ── Controls (green before and after) ───────────────────────────────────────────

#[test]
fn build_still_honours_its_own_flags() {
    let dir = project();
    let before = snapshot(dir.path());
    let out = run(
        dir.path(),
        &["build", "-q", "50", "-j", "2", "-Q", "--no-cache", "-y"],
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    for rel in ["out/ext/a.webp", "out/recipefmt/a.gif", "out/neither/a.png"] {
        assert!(dir.path().join(rel).is_file(), "missing {rel}");
    }
    // The snapshot must see a write, or every "unchanged" assertion is vacuous.
    assert_ne!(
        snapshot(dir.path()),
        before,
        "snapshot did not see the build's writes"
    );

    let strict = run(dir.path(), &["build", "--strict"]);
    assert_eq!(strict.status.code(), Some(0), "build --strict");
    let check = run(dir.path(), &["build", "--check"]);
    assert_eq!(check.status.code(), Some(0), "build --check");
}

#[test]
fn apply_still_honours_the_four_output_flags() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    // A JPEG source, so no expected output format equals the source's.
    write_file(root, "src/a.jpg", &encode(ImageFormat::Jpeg));
    write_file(root, "v1.toml", RESIZE_32.as_bytes());

    let apply = |extra: &[&str]| {
        let mut args = vec!["apply", "--recipe", "v1.toml", "src/a.jpg"];
        args.extend_from_slice(extra);
        let out = run(root, &args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "apply {extra:?}; stderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    let is_png = |rel: &str| {
        std::fs::read(root.join(rel))
            .unwrap()
            .starts_with(PNG_MAGIC)
    };

    apply(&["-o", "x.png"]);
    assert!(is_png("x.png"), "-o x.png is not a PNG");

    // `.out` pins nothing, so `--format` is what decided.
    apply(&["-o", "f.out", "--format", "png"]);
    assert!(is_png("f.out"), "-o f.out --format png is not a PNG");

    apply(&["--out-dir", "d"]);
    assert!(
        root.join("d/a.jpg").is_file(),
        "--out-dir d wrote no d/a.jpg"
    );

    apply(&["--out-dir", "t", "--name-template", "{stem}_w.png"]);
    assert!(is_png("t/a_w.png"), "t/a_w.png is not a PNG");
}
