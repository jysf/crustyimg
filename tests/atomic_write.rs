//! A file write is all or nothing: it either replaces the destination whole or
//! leaves it exactly as it was, and any failure exits non-zero.
//!
//! Every row drives the real binary in a temp dir. Where a write must fail, the
//! row runs the binary under `ulimit -f` with SIGXFSZ ignored, so the kernel
//! returns `EFBIG` instead of killing the process. Assertions run in the order
//! exit code, old bytes, staged file, then message, so a row that goes red names
//! the damage before the wording. Expected messages are literals here, never the
//! code's own `Display`.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// The 29-byte stand-in for a file the user already had.
const OLD: &[u8] = b"OLD CONTENT, DO NOT LOSE ME!!";

// ── Helpers ────────────────────────────────────────────────────────────────────

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/classify/photo_entropy_floor.png")
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .current_dir(dir)
        .output()
        .expect("binary should run")
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).replace("\r\n", "\n")
}

/// A 40x40 PNG (about 1.3 KB: under the 8 KiB buffer the old code used).
fn small_png(dir: &Path, name: &str) {
    if let Some(parent) = dir.join(name).parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let out = run(
        dir,
        &[
            "resize",
            fixture().to_str().unwrap(),
            "--max",
            "40",
            "-o",
            name,
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "small fixture: {}",
        stderr_of(&out)
    );
    let len = std::fs::metadata(dir.join(name)).unwrap().len();
    assert!(
        (1_025..8_000).contains(&len),
        "the small fixture must sit between the 1 KiB limit and the 8 KiB buffer, got {len}"
    );
}

/// The 512x512 fixture, copied (about 82 KB: well past the 8 KiB buffer).
fn big_png(dir: &Path, name: &str) {
    if let Some(parent) = dir.join(name).parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::copy(fixture(), dir.join(name)).unwrap();
}

/// Names in `dir` that contain `.crustyimg-` (a staged file).
fn staged_left(dir: &Path) -> Vec<String> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    rd.filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains(".crustyimg-"))
        .collect()
}

/// Assert `path` holds exactly `want`, without dumping the bytes on failure.
fn assert_holds(path: &Path, want: &[u8], what: &str) {
    let got = std::fs::read(path).unwrap_or_default();
    assert!(
        got == want,
        "{what}: {} holds {} bytes, expected {} bytes",
        path.display(),
        got.len(),
        want.len()
    );
}

fn single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Run the binary under `sh -c "[trap '' XFSZ; ]ulimit -f <blocks>; exec BIN args"`.
/// `ulimit -f` counts 512-byte blocks under `sh`; every output used here is
/// larger than 1,024 B, so either unit stops it. Stderr is captured through a
/// pipe by `output()`: a stderr redirect inside the shell would be capped by the
/// same limit.
fn limited(dir: &Path, ignore_xfsz: bool, blocks: u32, args: &str) -> Output {
    let trap = if ignore_xfsz { "trap '' XFSZ; " } else { "" };
    let script = format!(
        "{trap}ulimit -f {blocks}; exec {} {args}",
        single_quote(BIN)
    );
    Command::new("sh")
        .arg("-c")
        .arg(script)
        .current_dir(dir)
        .output()
        .expect("sh should run")
}

fn write_file(dir: &Path, rel: &str, bytes: &[u8]) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, bytes).unwrap();
}

// ── Unix rows ──────────────────────────────────────────────────────────────────

#[cfg(unix)]
mod unix {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::process::ExitStatusExt;

    fn set_mode(path: &Path, mode: u32) {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    }

    fn mode_of(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    /// One of the eight EFBIG rows: `size` input, pixel or byte lane, to `-o` or
    /// `--out-dir`. The old destination holds `OLD`.
    fn efbig(big: bool, pixel: bool, to_dir: bool) {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let input = "in.png";
        if big {
            big_png(root, input);
        } else {
            small_png(root, input);
        }
        let verb = if pixel {
            "convert in.png --format png"
        } else {
            "meta strip in.png"
        };
        let (dest, args) = if to_dir {
            write_file(root, "od/in.png", OLD);
            ("od/in.png", format!("{verb} --out-dir od -y"))
        } else {
            write_file(root, "old.png", OLD);
            ("old.png", format!("{verb} -o old.png -y"))
        };
        let out = limited(root, true, 1, &args);
        // Exit code first, then the old bytes, then the staged file, then the words.
        assert_eq!(out.status.code(), Some(5), "stderr: {}", stderr_of(&out));
        assert_holds(&root.join(dest), OLD, "bytes kept");
        assert_eq!(staged_left(root), Vec::<String>::new());
        assert_eq!(staged_left(&root.join("od")), Vec::<String>::new());
        let err = stderr_of(&out);
        assert!(err.contains("could not write output"), "stderr: {err}");
        assert!(err.contains("File too large"), "stderr: {err}");
    }

    // FT-E1…E4: small outputs (the silent-success half).
    #[test]
    fn efbig_small_pixel_file() {
        efbig(false, true, false);
    }
    #[test]
    fn efbig_small_pixel_dir() {
        efbig(false, true, true);
    }
    #[test]
    fn efbig_small_bytes_file() {
        efbig(false, false, false);
    }
    #[test]
    fn efbig_small_bytes_dir() {
        efbig(false, false, true);
    }
    // FT-E5…E8: big outputs (the destroyed-old-file half).
    #[test]
    fn efbig_big_pixel_file() {
        efbig(true, true, false);
    }
    #[test]
    fn efbig_big_pixel_dir() {
        efbig(true, true, true);
    }
    #[test]
    fn efbig_big_bytes_file() {
        efbig(true, false, false);
    }
    #[test]
    fn efbig_big_bytes_dir() {
        efbig(true, false, true);
    }

    // FT-E9
    #[test]
    fn sigxfsz_kill_keeps_the_old_file() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        big_png(root, "in.png");
        write_file(root, "old.png", OLD);
        let out = limited(root, false, 1, "convert in.png --format png -o old.png -y");
        assert_eq!(out.status.signal(), Some(25), "killed by SIGXFSZ");
        assert_holds(&root.join("old.png"), OLD, "bytes kept");
        let left = staged_left(root);
        assert!(left.len() <= 1, "at most one staged file, got {left:?}");
    }

    // FT-E10: a pin; a helper that chmods new files or stages at 0600 reds it.
    #[test]
    fn a_new_file_follows_the_umask() {
        for (umask, want) in [("022", 0o644), ("077", 0o600)] {
            let dir = TempDir::new().unwrap();
            let root = dir.path();
            small_png(root, "in.png");
            let script =
                format!("umask {umask}; exec '{BIN}' convert in.png --format png -o new.png");
            let out = Command::new("sh")
                .arg("-c")
                .arg(script)
                .current_dir(root)
                .output()
                .unwrap();
            assert_eq!(out.status.code(), Some(0), "stderr: {}", stderr_of(&out));
            assert_eq!(mode_of(&root.join("new.png")), want, "umask {umask}");
            assert_eq!(staged_left(root), Vec::<String>::new());
        }
    }

    /// A directory the current user cannot create files in, with `old.png`
    /// (writable) inside. Asserts the precondition.
    fn read_only_dir_with_old(root: &Path) {
        write_file(root, "ro/old.png", OLD);
        set_mode(&root.join("ro"), 0o555);
        let probe = std::fs::File::create(root.join("ro/probe"));
        if probe.is_ok() {
            set_mode(&root.join("ro"), 0o755);
            panic!("precondition: running as root? a read-only directory cannot be made here");
        }
    }

    // FT-E11
    #[test]
    fn a_read_only_directory_is_refused_by_name() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        small_png(root, "in.png");
        read_only_dir_with_old(root);
        let out = run(
            root,
            &[
                "convert",
                "in.png",
                "--format",
                "png",
                "-o",
                "ro/old.png",
                "-y",
            ],
        );
        set_mode(&root.join("ro"), 0o755);
        assert_eq!(out.status.code(), Some(5), "stderr: {}", stderr_of(&out));
        assert_holds(&root.join("ro/old.png"), OLD, "bytes kept");
        assert_eq!(staged_left(&root.join("ro")), Vec::<String>::new());
        assert_eq!(staged_left(root), Vec::<String>::new());
        let err = stderr_of(&out);
        assert!(
            err.contains(
                "error: could not write output ro/old.png: cannot create a new file in ro: \
                 Permission denied"
            ),
            "stderr: {err}"
        );
    }

    // FT-E12: a pin; the guards run before staging.
    #[test]
    fn a_read_only_directory_without_yes_says_already_exists() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        small_png(root, "in.png");
        read_only_dir_with_old(root);
        let out = run(
            root,
            &["convert", "in.png", "--format", "png", "-o", "ro/old.png"],
        );
        set_mode(&root.join("ro"), 0o755);
        assert_eq!(out.status.code(), Some(5), "stderr: {}", stderr_of(&out));
        assert_holds(&root.join("ro/old.png"), OLD, "bytes kept");
        let err = stderr_of(&out);
        assert!(
            err.contains("output file already exists (use --yes to overwrite): ro/old.png"),
            "stderr: {err}"
        );
    }

    // FT-E13: two non-default modes, so a skipped copy shows.
    #[test]
    fn yes_keeps_a_non_default_mode() {
        for mode in [0o600, 0o640] {
            let dir = TempDir::new().unwrap();
            let root = dir.path();
            small_png(root, "in.png");
            write_file(root, "old.png", OLD);
            set_mode(&root.join("old.png"), mode);
            let out = run(
                root,
                &[
                    "convert", "in.png", "--format", "png", "-o", "old.png", "-y",
                ],
            );
            assert_eq!(out.status.code(), Some(0), "stderr: {}", stderr_of(&out));
            assert_ne!(std::fs::read(root.join("old.png")).unwrap(), OLD);
            assert_eq!(mode_of(&root.join("old.png")), mode);
            assert_eq!(staged_left(root), Vec::<String>::new());
        }
    }

    // FT-E15
    #[test]
    fn yes_over_a_read_only_file_is_refused() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        small_png(root, "in.png");
        write_file(root, "old.png", OLD);
        set_mode(&root.join("old.png"), 0o444);
        if std::fs::OpenOptions::new()
            .write(true)
            .open(root.join("old.png"))
            .is_ok()
        {
            panic!("precondition: running as root? a read-only file cannot be made here");
        }
        let out = run(
            root,
            &[
                "convert", "in.png", "--format", "png", "-o", "old.png", "-y",
            ],
        );
        set_mode(&root.join("old.png"), 0o644);
        assert_eq!(out.status.code(), Some(5), "stderr: {}", stderr_of(&out));
        assert_holds(&root.join("old.png"), OLD, "bytes kept");
        assert_eq!(staged_left(root), Vec::<String>::new());
        let err = stderr_of(&out);
        assert!(
            err.contains("error: could not write output old.png: Permission denied"),
            "stderr: {err}"
        );
    }

    // FT-E19
    #[test]
    fn a_batch_with_one_refused_destination_keeps_it_and_writes_the_rest() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        small_png(root, "in/a.png");
        small_png(root, "in/b.png");
        write_file(root, "od/a.png", OLD);
        set_mode(&root.join("od/a.png"), 0o444);
        if std::fs::OpenOptions::new()
            .write(true)
            .open(root.join("od/a.png"))
            .is_ok()
        {
            panic!("precondition: running as root? a read-only file cannot be made here");
        }
        let out = run(
            root,
            &["convert", "in", "--format", "png", "--out-dir", "od", "-y"],
        );
        set_mode(&root.join("od/a.png"), 0o644);
        assert_eq!(out.status.code(), Some(6), "stderr: {}", stderr_of(&out));
        assert_holds(&root.join("od/a.png"), OLD, "bytes kept");
        let b = std::fs::read(root.join("od/b.png")).expect("b.png is written");
        assert!(b.starts_with(b"\x89PNG"), "b.png is a PNG");
        assert_eq!(staged_left(&root.join("od")), Vec::<String>::new());
        let err = stderr_of(&out);
        assert!(err.contains("od/a.png: Permission denied"), "stderr: {err}");
    }

    // FT-E21
    #[test]
    fn a_failed_lockfile_write_keeps_the_old_lockfile() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        small_png(root, "in/a.png");
        write_file(
            root,
            "recipes/r.toml",
            b"version = \"1\"\n\n[[step]]\nop = \"auto-orient\"\n",
        );
        write_file(
            root,
            "crustyimg.build.toml",
            b"version = 1\n\n[[target]]\nsource = \"in/*.png\"\nrecipe = \"recipes/r.toml\"\nout = \"dist\"\n",
        );
        std::fs::create_dir(root.join("dist")).unwrap();
        write_file(root, "crustyimg.build.lock", b"OLD LOCK");
        set_mode(root, 0o555);
        let probe = std::fs::File::create(root.join("probe"));
        if probe.is_ok() {
            set_mode(root, 0o755);
            panic!("precondition: running as root? a read-only directory cannot be made here");
        }
        let out = run(root, &["build", "--no-cache"]);
        set_mode(root, 0o755);
        assert_eq!(out.status.code(), Some(5), "stderr: {}", stderr_of(&out));
        assert_holds(
            &root.join("crustyimg.build.lock"),
            b"OLD LOCK",
            "bytes kept",
        );
        assert_eq!(staged_left(root), Vec::<String>::new());
        let err = stderr_of(&out);
        assert!(
            err.contains(
                "error: could not write build lockfile crustyimg.build.lock: Permission denied"
            ),
            "stderr: {err}"
        );
    }

    // FT-E22: stderr must reach the test through a pipe, not a file inside the
    // shell, which `ulimit -f 0` would cap and turn into a panic (exit 101).
    #[test]
    fn save_recipe_failed_write_keeps_the_old_recipe() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        small_png(root, "small.png");
        write_file(root, "r.toml", b"PRECIOUS");
        let out = limited(
            root,
            true,
            0,
            "edit small.png --invert -o - -y --save-recipe r.toml >/dev/null",
        );
        assert_eq!(out.status.code(), Some(5), "stderr: {}", stderr_of(&out));
        assert_holds(&root.join("r.toml"), b"PRECIOUS", "bytes kept");
        assert_eq!(staged_left(root), Vec::<String>::new());
        let err = stderr_of(&out);
        assert!(
            err.contains("could not write output r.toml: File too large"),
            "stderr: {err}"
        );
    }

    // FT-E23
    #[test]
    fn a_batch_where_every_write_fails_exits_6_and_keeps_every_file() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        small_png(root, "in/a.png");
        small_png(root, "in/b.png");
        write_file(root, "od/a.png", OLD);
        write_file(root, "od/b.png", OLD);
        let out = limited(root, true, 1, "convert in --format png --out-dir od -y");
        assert_eq!(out.status.code(), Some(6), "stderr: {}", stderr_of(&out));
        assert_holds(&root.join("od/a.png"), OLD, "bytes kept");
        assert_holds(&root.join("od/b.png"), OLD, "bytes kept");
        assert_eq!(staged_left(&root.join("od")), Vec::<String>::new());
    }
}

// ── Rows for every OS ──────────────────────────────────────────────────────────

// FT-E14: pins completeness and cleanup, not name uniqueness (the unit test does).
#[test]
fn a_parallel_build_into_one_directory_leaves_no_staged_file() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    small_png(root, "seed.png");
    let seed = std::fs::read(root.join("seed.png")).unwrap();
    for i in 0..60 {
        write_file(root, &format!("src/{i:02}.png"), &seed);
    }
    write_file(
        root,
        "r.toml",
        b"version = \"1\"\n\n[[step]]\nop = \"auto-orient\"\n",
    );
    let mut manifest = String::from("version = 1\n");
    for t in 0..4 {
        manifest.push_str(&format!(
            "\n[[target]]\nsource = \"src/*.png\"\nrecipe = \"r.toml\"\nout = \"dist\"\nname = \"{{stem}}_{t}.{{ext}}\"\n"
        ));
    }
    write_file(root, "crustyimg.build.toml", manifest.as_bytes());
    let out = run(root, &["build", "-j", "8", "--no-cache"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", stderr_of(&out));
    let outputs = std::fs::read_dir(root.join("dist")).unwrap().count();
    assert_eq!(outputs, 240, "four targets over 60 inputs");
    assert_eq!(staged_left(&root.join("dist")), Vec::<String>::new());
    assert_eq!(staged_left(root), Vec::<String>::new());
}

// FT-E16
#[test]
fn save_recipe_without_yes_refuses_an_existing_recipe() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    small_png(root, "small.png");
    write_file(root, "r.toml", b"PRECIOUS");
    let out = run(
        root,
        &[
            "edit",
            "small.png",
            "--invert",
            "-o",
            "e.png",
            "--save-recipe",
            "r.toml",
        ],
    );
    assert_eq!(out.status.code(), Some(5), "stderr: {}", stderr_of(&out));
    assert_holds(&root.join("r.toml"), b"PRECIOUS", "bytes kept");
    assert!(
        !root.join("e.png").exists(),
        "the image output is not written"
    );
    assert_eq!(staged_left(root), Vec::<String>::new());
    let err = stderr_of(&out);
    assert!(
        err.contains("error: output file already exists (use --yes to overwrite): r.toml"),
        "stderr: {err}"
    );
}

// FT-E17
#[test]
fn save_recipe_with_yes_replaces_an_existing_recipe() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    small_png(root, "small.png");
    write_file(root, "r.toml", b"PRECIOUS");
    let out = run(
        root,
        &[
            "edit",
            "small.png",
            "--invert",
            "-o",
            "e.png",
            "--save-recipe",
            "r.toml",
            "-y",
        ],
    );
    assert_eq!(out.status.code(), Some(0), "stderr: {}", stderr_of(&out));
    let recipe = std::fs::read_to_string(root.join("r.toml")).unwrap();
    assert!(recipe.starts_with("version = \"1\""), "recipe: {recipe}");
    assert_eq!(staged_left(root), Vec::<String>::new());
}

// FT-E18: the file a sink writes is the stdout bytes (stdout does not use the
// helper). The 1,328 B `convert` case fits under 4,096 B; the big `meta strip`
// case does not, so a short-write bug shows in one of the two.
#[test]
fn a_file_output_holds_the_same_bytes_as_stdout() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    small_png(root, "small.png");
    big_png(root, "big.png");
    for args in [
        &["convert", "small.png", "--format", "png"][..],
        &["meta", "strip", "big.png"][..],
    ] {
        let mut to_file: Vec<&str> = args.to_vec();
        to_file.extend(["-o", "f.png"]);
        let mut to_stdout: Vec<&str> = args.to_vec();
        to_stdout.extend(["-o", "-"]);
        let a = run(root, &to_file);
        assert_eq!(a.status.code(), Some(0), "stderr: {}", stderr_of(&a));
        let b = run(root, &to_stdout);
        assert_eq!(b.status.code(), Some(0), "stderr: {}", stderr_of(&b));
        assert!(b.stdout.len() > 1_000, "stdout carries the image");
        let file = std::fs::read(root.join("f.png")).unwrap();
        assert!(file == b.stdout, "{args:?}: file and stdout bytes differ");
        std::fs::remove_file(root.join("f.png")).unwrap();
    }
}

// FT-E20: the `.tmp` suffix keeps a staged file out of folder and glob inputs.
#[test]
fn a_staged_file_in_an_input_folder_is_not_an_input() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    small_png(root, "src/a.png");
    std::fs::copy(
        root.join("src/a.png"),
        root.join("src/.a.png.crustyimg-1-0.tmp"),
    )
    .unwrap();
    for (input, od) in [("src", "od1"), ("src/*", "od2")] {
        let out = run(
            root,
            &["convert", input, "--format", "png", "--out-dir", od],
        );
        assert_eq!(out.status.code(), Some(0), "{input}: {}", stderr_of(&out));
        let n = std::fs::read_dir(root.join(od)).unwrap().count();
        assert_eq!(n, 1, "{input} gives one output");
    }
}

// ── Windows rows (CI only) ─────────────────────────────────────────────────────

#[cfg(windows)]
mod windows {
    use super::*;
    use std::os::windows::fs::OpenOptionsExt;

    // FT-W1
    #[test]
    fn windows_yes_replaces_an_existing_file() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        small_png(root, "in.png");
        write_file(root, "old.png", OLD);
        let out = run(
            root,
            &[
                "convert", "in.png", "--format", "png", "-o", "old.png", "-y",
            ],
        );
        assert_eq!(out.status.code(), Some(0), "stderr: {}", stderr_of(&out));
        assert_ne!(std::fs::read(root.join("old.png")).unwrap(), OLD);
        assert_eq!(staged_left(root), Vec::<String>::new());
    }

    // FT-W2: Rust's own `File::open` shares delete, so the holder sets share_mode.
    #[test]
    fn windows_a_destination_held_open_without_share_delete_is_kept() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        small_png(root, "in.png");
        write_file(root, "old.png", OLD);
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0x1 | 0x2)
            .open(root.join("old.png"))
            .unwrap();
        let out = run(
            root,
            &[
                "convert", "in.png", "--format", "png", "-o", "old.png", "-y",
            ],
        );
        drop(held);
        assert_eq!(out.status.code(), Some(5), "stderr: {}", stderr_of(&out));
        assert_holds(&root.join("old.png"), OLD, "bytes kept");
        assert_eq!(staged_left(root), Vec::<String>::new());
        let err = stderr_of(&out);
        assert!(
            err.contains("could not write output old.png:"),
            "stderr: {err}"
        );
    }

    // FT-W3
    #[test]
    fn windows_a_read_only_destination_is_refused_and_kept() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        small_png(root, "in.png");
        write_file(root, "old.png", OLD);
        let mut perm = std::fs::metadata(root.join("old.png"))
            .unwrap()
            .permissions();
        perm.set_readonly(true);
        std::fs::set_permissions(root.join("old.png"), perm).unwrap();
        let out = run(
            root,
            &[
                "convert", "in.png", "--format", "png", "-o", "old.png", "-y",
            ],
        );
        let mut perm = std::fs::metadata(root.join("old.png"))
            .unwrap()
            .permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perm.set_readonly(false);
        std::fs::set_permissions(root.join("old.png"), perm).unwrap();
        assert_eq!(out.status.code(), Some(5), "stderr: {}", stderr_of(&out));
        assert_holds(&root.join("old.png"), OLD, "bytes kept");
        assert_eq!(staged_left(root), Vec::<String>::new());
    }

    // FT-W4
    #[test]
    fn windows_yes_over_the_input_itself_replaces_it() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        small_png(root, "a.png");
        let out = run(
            root,
            &["convert", "a.png", "--format", "png", "-o", "a.png", "-y"],
        );
        assert_eq!(out.status.code(), Some(0), "stderr: {}", stderr_of(&out));
        assert!(image::open(root.join("a.png")).is_ok(), "the file decodes");
        assert_eq!(staged_left(root), Vec::<String>::new());
    }
}
