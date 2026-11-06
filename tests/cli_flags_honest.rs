//! SPEC-149: no global flag is accepted and then ignored. Each flag is honoured or
//! refused (exit 2) with a reason, and `-v` shows dependency warnings.
//!
//! These drive the REAL compiled binary in a temp dir with relative paths, and compare
//! a snapshot of every file under it (path -> SHA-256) before and after, because an
//! exit code alone cannot show that a refusal wrote nothing. The expected messages are
//! copied here as literals on purpose: a test that imported the code's own constants
//! could not catch a changed message.

use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use clap::CommandFactory;
use image::{DynamicImage, ImageFormat, RgbImage};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── The refusal lines (R-table), `error: ` prefix and newline included ──────────

const R1_WATCH: &str = "error: --watch is only valid with `build`: there is no rebuild loop to run for this subcommand\n";
const R2_CHECK: &str = "error: --check (or --frozen/--locked) is only valid with `build`: there is no build lockfile to check for this subcommand\n";
const R3_STRICT: &str = "error: --strict is only valid with `build --check`: there is no build lockfile to check for this subcommand\n";
const R4_STRICT_ALONE: &str =
    "error: --strict only applies with --check: run `build --check --strict`, or drop --strict\n";
const R5_NO_CACHE: &str =
    "error: --no-cache is only valid with `build`: there is no build cache for this subcommand\n";
const R6_KEEP_GPS: &str = "error: --keep-gps is not supported yet: pixel-lane encodes drop all metadata, GPS included; to keep it, copy the metadata back after the encode with `crustyimg meta copy --from <in> --to <out> -y` (JPEG and PNG)\n";
const R7_NAME_TEMPLATE: &str = "error: responsive does not take --name-template: its output names are fixed as {stem}-{w}w.{ext}\n";
const R8_FORMAT: &str = "error: responsive does not take --format: choose its output formats with --formats (e.g. --formats webp,jpeg)\n";

const FONT_PREFIX: &str = "error: could not read font file \"";
const OVERLAY_PREFIX: &str = "error: could not read watermark image \"";
const OLD_MESSAGE: &str = "could not read image input";

/// The `-v` fixture's single usvg warning, and its `-vv` debug line.
const W: &str = "warning: [usvg::parser::image] Image lacks the 'xlink:href' attribute. Skipped.";
const D: &str = "debug: [usvg::parser::text] Found font-variation-settings: ''wght' 700'";

const DEADLINE: Duration = Duration::from_secs(20);
const TICK: Duration = Duration::from_millis(50);

/// How many leaf commands the CLI has today. A new verb must be added to
/// `minimal_args` AND to this number.
const LEAF_COUNT: usize = 20;

// ── Helpers ─────────────────────────────────────────────────────────────────────

fn write_file(dir: &Path, rel: &str, bytes: &[u8]) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, bytes).unwrap();
}

fn png_bytes() -> Vec<u8> {
    let img = RgbImage::from_pixel(32, 32, image::Rgb([200, 30, 30]));
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .unwrap();
    buf.into_inner()
}

/// A temp dir holding `in.png` (32x32), `t.png` (a copy, for `meta copy --to`), `r.toml`
/// (resize max 16) and a one-target build manifest.
fn project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "in.png", &png_bytes());
    write_file(root, "t.png", &png_bytes());
    write_file(
        root,
        "r.toml",
        b"version = \"1\"\n\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 16\n",
    );
    write_file(
        root,
        "crustyimg.build.toml",
        b"version = 1\n\n[[target]]\nsource = \"in.png\"\nrecipe = \"r.toml\"\nout = \"bo\"\n",
    );
    dir
}

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

fn stderr_of(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Number of stderr lines exactly equal to `line`.
fn count_line(stderr: &str, line: &str) -> usize {
    stderr.lines().filter(|l| *l == line).count()
}

/// Assert a run exited 2 with exactly `expected` on stderr.
fn assert_refused(out: &std::process::Output, expected: &str, what: &str) {
    let stderr = stderr_of(out);
    assert_eq!(
        out.status.code(),
        Some(2),
        "{what}: exit code; stderr:\n{stderr}"
    );
    assert_eq!(stderr, expected, "{what}: stderr");
}

/// Kills the child on drop, so a panicking test never leaks a watching process.
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Run `crustyimg args` under [`DEADLINE`]; a run still alive at the deadline is killed
/// and the test fails as "watcher started".
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
            "`crustyimg {}` is still running after {DEADLINE:?}: watcher started",
            args.join(" ")
        );
        std::thread::sleep(TICK);
    };
    (status.code(), std::fs::read_to_string(&logpath).unwrap())
}

// ── FT-1 ────────────────────────────────────────────────────────────────────────

/// One valid, minimal argv per leaf command, run in [`project`]'s directory. `None`
/// for a leaf nobody wrote an argv for, which fails the test by name.
fn minimal_args(path: &[&str]) -> Option<Vec<&'static str>> {
    let args: &[&'static str] = match path {
        ["view"] => &["view", "in.png"],
        ["info"] => &["info", "in.png"],
        ["diff"] => &["diff", "in.png", "in.png"],
        ["resize"] => &["resize", "in.png", "--max", "8", "-o", "o.png"],
        ["thumbnail"] => &["thumbnail", "in.png", "-o", "o.png"],
        ["convert"] => &["convert", "in.png", "--format", "jpeg", "-o", "o.jpg"],
        ["web"] => &["web", "in.png", "-o", "o.png"],
        ["optimize"] => &["optimize", "in.png", "-o", "o.png"],
        ["responsive"] => &["responsive", "in.png", "--widths", "8", "--out-dir", "rd"],
        ["auto-orient"] => &["auto-orient", "in.png", "-o", "o.png"],
        // `--size 8`: at the default size the mark covers 27% of a 32x32 image and the
        // 5% limit refuses it (exit 2), which would make the baseline a usage error.
        ["watermark"] => &[
            "watermark",
            "in.png",
            "--text",
            "hi",
            "--size",
            "8",
            "-o",
            "o.png",
        ],
        ["edit"] => &["edit", "in.png", "--invert", "-o", "o.png"],
        ["apply"] => &["apply", "--recipe", "r.toml", "in.png", "-o", "o.png"],
        ["build"] => &["build"],
        ["lint"] => &["lint", "in.png"],
        ["completions"] => &["completions", "zsh"],
        ["meta", "strip"] => &["meta", "strip", "in.png", "-o", "m.png"],
        ["meta", "clean"] => &["meta", "clean", "in.png", "--gps", "-o", "m.png"],
        ["meta", "copy"] => &["meta", "copy", "--from", "in.png", "--to", "t.png"],
        ["meta", "set"] => &["meta", "set", "in.png", "--artist", "a", "-o", "m.png"],
        _ => return None,
    };
    Some(args.to_vec())
}

/// Every leaf command path of the real clap tree (`meta` recurses). Derived from
/// `Cli::command()`, never from a hand list, so a verb added later is seen.
fn leaf_paths() -> Vec<Vec<String>> {
    fn walk(cmd: &clap::Command, prefix: &[String], out: &mut Vec<Vec<String>>) {
        for sub in cmd.get_subcommands() {
            if sub.get_name() == "help" {
                continue;
            }
            let mut path = prefix.to_vec();
            path.push(sub.get_name().to_owned());
            if sub.has_subcommands() {
                walk(sub, &path, out);
            } else {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(&crustyimg::cli::Cli::command(), &[], &mut out);
    out
}

#[test]
fn every_leaf_command_refuses_build_only_flags_and_keep_gps() {
    let leaves = leaf_paths();
    println!("saw {} leaf commands", leaves.len());
    assert_eq!(
        leaves.len(),
        LEAF_COUNT,
        "saw {} leaf commands, expected {LEAF_COUNT}: {leaves:?}",
        leaves.len()
    );

    let r_lines = [
        R1_WATCH,
        R2_CHECK,
        R3_STRICT,
        R4_STRICT_ALONE,
        R5_NO_CACHE,
        R6_KEEP_GPS,
        R7_NAME_TEMPLATE,
        R8_FORMAT,
    ];
    // (flag, the line it must print off `build`)
    let flag_rows: [(&str, &str); 7] = [
        ("--check", R2_CHECK),
        ("--frozen", R2_CHECK),
        ("--locked", R2_CHECK),
        ("--strict", R3_STRICT),
        ("--no-cache", R5_NO_CACHE),
        ("--watch", R1_WATCH),
        ("--keep-gps", R6_KEEP_GPS),
    ];
    // Leaves whose baseline run must write something, proving the snapshot can see a write.
    let writers = [
        "resize",
        "convert",
        "web",
        "optimize",
        "watermark",
        "edit",
        "apply",
        "thumbnail",
        "auto-orient",
    ];

    let mut rows = 0;
    for path in &leaves {
        let names: Vec<&str> = path.iter().map(String::as_str).collect();
        let joined = names.join(" ");
        let base = minimal_args(&names)
            .unwrap_or_else(|| panic!("no test argv for `{joined}`: add one to minimal_args"));

        // Baseline: the argv is valid (not a usage error) and prints no R-line.
        let dir = project();
        let before = snapshot(dir.path());
        let out = run(dir.path(), &base);
        let stderr = stderr_of(&out);
        assert_ne!(
            out.status.code(),
            Some(2),
            "baseline `{joined}` is a usage error, so minimal_args is wrong:\n{stderr}"
        );
        for line in r_lines {
            assert!(
                !stderr.contains(line.trim_end()),
                "baseline `{joined}` printed a refusal line:\n{stderr}"
            );
        }
        if writers.contains(&names[0])
            || ["strip", "clean", "set"].contains(&names[names.len() - 1])
        {
            assert_ne!(
                snapshot(dir.path()),
                before,
                "baseline `{joined}` wrote nothing, so the snapshot cannot see a write"
            );
        }

        for (flag, line) in flag_rows {
            if names == ["build"] {
                // `build` is never given --watch here (that starts a watcher: FT-4),
                // and only --keep-gps is refused on it among these seven.
                if flag != "--keep-gps" {
                    continue;
                }
            }
            let dir = project();
            let before = snapshot(dir.path());
            let mut args = base.clone();
            args.push(flag);
            let out = run(dir.path(), &args);
            assert_refused(&out, line, &format!("{joined} {flag}"));
            assert_eq!(snapshot(dir.path()), before, "{joined} {flag}: wrote files");
            rows += 1;
        }
    }
    // 19 non-build leaves x 7 flags + `build --keep-gps`.
    assert_eq!(rows, 19 * 7 + 1, "refusal rows driven");
}

// ── FT-3 ────────────────────────────────────────────────────────────────────────

#[test]
fn keep_gps_is_hidden_from_help_but_still_parsed() {
    let dir = project();
    for args in [&["--help"][..], &["resize", "--help"]] {
        let out = run(dir.path(), args);
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        assert_eq!(out.status.code(), Some(0), "{args:?}");
        assert!(
            !stdout.contains("--keep-gps"),
            "`{}` lists --keep-gps:\n{stdout}",
            args.join(" ")
        );
    }
    let out = run(
        dir.path(),
        &[
            "resize",
            "in.png",
            "--max",
            "8",
            "-o",
            "o.png",
            "--keep-gps",
        ],
    );
    assert_refused(&out, R6_KEEP_GPS, "resize --keep-gps");
}

// ── FT-4 ────────────────────────────────────────────────────────────────────────

#[test]
fn build_refuses_strict_without_check_and_keeps_its_own_flags() {
    let dir = project();
    let before = snapshot(dir.path());
    let out = run(dir.path(), &["build", "--strict"]);
    assert_refused(&out, R4_STRICT_ALONE, "build --strict");
    assert_eq!(snapshot(dir.path()), before, "build --strict wrote files");
    assert!(!dir.path().join("crustyimg.build.lock").exists());
    assert!(!dir.path().join(".crustyimg").exists());
    assert!(!dir.path().join("bo").exists());

    let (code, stderr) = run_with_deadline(dir.path(), &["build", "--watch", "--strict"]);
    assert_eq!(code, Some(2), "build --watch --strict; stderr:\n{stderr}");
    assert_eq!(stderr, R4_STRICT_ALONE, "build --watch --strict: stderr");
    assert_eq!(
        snapshot(dir.path()),
        before,
        "build --watch --strict wrote files"
    );

    // Positive controls: the flags `build` reads still run.
    let out = run(dir.path(), &["build"]);
    assert_eq!(out.status.code(), Some(0), "build: {}", stderr_of(&out));
    assert!(dir.path().join("crustyimg.build.lock").exists());
    let out = run(dir.path(), &["build", "--check", "--strict"]);
    let stderr = stderr_of(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "build --check --strict: {stderr}"
    );
    assert!(!stderr.contains("only applies with --check"), "{stderr}");
    let out = run(dir.path(), &["build", "--no-cache"]);
    let stderr = stderr_of(&out);
    assert_eq!(out.status.code(), Some(0), "build --no-cache: {stderr}");
    assert!(!stderr.contains("only valid with `build`"), "{stderr}");
}

// ── FT-5 ────────────────────────────────────────────────────────────────────────

#[test]
fn responsive_refuses_name_template_and_format() {
    let base = ["responsive", "in.png", "--widths", "8", "--out-dir", "rd"];
    let tn = ["--name-template", "{stem}.png"];
    let fmt = ["--format", "webp"];
    let rows: [(Vec<&str>, String); 3] = [
        ([&base[..], &tn[..]].concat(), R7_NAME_TEMPLATE.to_owned()),
        ([&base[..], &fmt[..]].concat(), R8_FORMAT.to_owned()),
        (
            [&base[..], &tn[..], &fmt[..]].concat(),
            format!("{R7_NAME_TEMPLATE}{R8_FORMAT}"),
        ),
    ];
    for (args, expected) in rows {
        let dir = project();
        let before = snapshot(dir.path());
        let out = run(dir.path(), &args);
        assert_refused(&out, &expected, &args.join(" "));
        assert_eq!(
            snapshot(dir.path()),
            before,
            "{}: wrote files",
            args.join(" ")
        );
        assert!(
            !dir.path().join("rd").exists(),
            "{}: created rd/",
            args.join(" ")
        );
    }

    // Control: --formats is read and writes.
    let dir = project();
    let out = run(
        dir.path(),
        &[
            "responsive",
            "in.png",
            "--widths",
            "8",
            "--out-dir",
            "rd",
            "--formats",
            "webp",
        ],
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr_of(&out));
    let webp = std::fs::read(dir.path().join("rd/in-8w.webp")).expect("rd/in-8w.webp written");
    assert_eq!(&webp[..4], b"RIFF");
}

// ── FT-6 ────────────────────────────────────────────────────────────────────────

#[test]
fn flags_a_verb_reads_are_not_refused() {
    let rows: [(&[&str], &str); 7] = [
        (
            &["--out-dir", "d", "--name-template", "{stem}_s.png"],
            "d/in_s.png",
        ),
        (&["--format", "jpeg", "-o", "o.jpg"], "o.jpg"),
        (&["-q", "40", "--format", "jpeg", "-o", "q.jpg"], "q.jpg"),
        (&["-j", "2", "--out-dir", "j"], "j/in.png"),
        (&["-y", "-o", "in2.png"], "in2.png"),
        (&["-Q", "-o", "qq.png"], "qq.png"),
        (&["-v", "-o", "v.png"], "v.png"),
    ];
    for (extra, expected_file) in rows {
        let dir = project();
        // `-y -o in2.png` overwrites an existing file.
        write_file(dir.path(), "in2.png", b"old");
        let mut args = vec!["resize", "in.png", "--max", "8"];
        args.extend_from_slice(extra);
        let out = run(dir.path(), &args);
        let stderr = stderr_of(&out);
        assert_eq!(out.status.code(), Some(0), "{}: {stderr}", args.join(" "));
        assert!(
            dir.path().join(expected_file).exists(),
            "{}: {expected_file} missing",
            args.join(" ")
        );
        for line in [
            R1_WATCH,
            R2_CHECK,
            R3_STRICT,
            R5_NO_CACHE,
            R6_KEEP_GPS,
            R7_NAME_TEMPLATE,
            R8_FORMAT,
        ] {
            assert!(
                !stderr.contains(line.trim_end()),
                "{}: {stderr}",
                args.join(" ")
            );
        }
    }
}

// ── FT-7 ────────────────────────────────────────────────────────────────────────

#[cfg(feature = "svg")]
#[test]
fn verbose_shows_dependency_warnings() {
    let fixture = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/svg/log_levels_40x30.svg"),
    )
    .unwrap();
    let dir = TempDir::new().unwrap();
    write_file(dir.path(), "n.svg", &fixture);
    write_file(dir.path(), "m.svg", &fixture);
    let convert = |out_name: &'static str, flags: &[&str]| {
        let mut args: Vec<&str> = flags.to_vec();
        args.extend(["convert", "n.svg", "--format", "png", "-o", out_name]);
        run(dir.path(), &args)
    };

    let out = convert("a.png", &[]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stderr_of(&out), "", "no -v: stderr must be empty");

    let out = convert("b.png", &["-v"]);
    let stderr = stderr_of(&out);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert_eq!(count_line(&stderr, W), 1, "-v: stderr:\n{stderr}");
    assert_eq!(
        count_line(&stderr, D),
        0,
        "-v shows no debug line:\n{stderr}"
    );
    assert_eq!(
        std::fs::read(dir.path().join("a.png")).unwrap(),
        std::fs::read(dir.path().join("b.png")).unwrap(),
        "-v changed the output bytes"
    );

    for (name, flags) in [("c.png", vec!["-vv"]), ("d.png", vec!["-v", "-v"])] {
        let out = convert(name, &flags);
        let stderr = stderr_of(&out);
        assert_eq!(count_line(&stderr, W), 1, "{flags:?}: stderr:\n{stderr}");
        assert_eq!(count_line(&stderr, D), 1, "{flags:?}: stderr:\n{stderr}");
    }

    for (name, flags) in [
        ("e.png", vec!["-v", "-Q"]),
        ("f.png", vec!["-Q", "-v"]),
        ("g.png", vec!["-vv", "-Q"]),
    ] {
        let out = convert(name, &flags);
        assert_eq!(stderr_of(&out), "", "{flags:?}: -Q must win");
    }

    // `diff` decodes both inputs: two records.
    let out = run(dir.path(), &["-v", "diff", "n.svg", "n.svg"]);
    let stderr = stderr_of(&out);
    assert_eq!(count_line(&stderr, W), 2, "diff: stderr:\n{stderr}");

    // Worker threads log through the same global logger: one record per input.
    let out = run(
        dir.path(),
        &[
            "-v",
            "-j",
            "4",
            "resize",
            "n.svg",
            "m.svg",
            "--max",
            "20",
            "--out-dir",
            "jd",
        ],
    );
    let stderr = stderr_of(&out);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert_eq!(count_line(&stderr, W), 2, "resize -j 4: stderr:\n{stderr}");
}

// ── FT-8 ────────────────────────────────────────────────────────────────────────

#[test]
fn very_verbose_reaches_the_avif_parser() {
    let avif = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/avif/photo_128.avif"),
    )
    .unwrap();
    let dir = TempDir::new().unwrap();
    write_file(dir.path(), "p.avif", &avif);

    for flags in [&[][..], &["-v"]] {
        let mut args = flags.to_vec();
        args.extend(["info", "p.avif"]);
        let out = run(dir.path(), &args);
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(stderr_of(&out), "", "{flags:?}: stderr must be empty");
    }
    let out = run(dir.path(), &["-vv", "info", "p.avif"]);
    let stderr = stderr_of(&out);
    assert!(
        stderr
            .lines()
            .any(|l| l.starts_with("debug: [avif_parse] ")),
        "-vv: no avif_parse debug line; stderr:\n{stderr}"
    );
    let out = run(dir.path(), &["-vv", "-Q", "info", "p.avif"]);
    assert_eq!(stderr_of(&out), "", "-vv -Q: -Q must win");
}

// ── FT-9 / FT-10 ────────────────────────────────────────────────────────────────

fn assert_asset_error(args: &[&str], prefix: &str, shown_path: &str) {
    let dir = project();
    std::fs::create_dir(dir.path().join("fontdir")).unwrap();
    std::fs::create_dir(dir.path().join("logodir")).unwrap();
    let before = snapshot(dir.path());
    let out = run(dir.path(), args);
    let stderr = stderr_of(&out);
    assert_eq!(out.status.code(), Some(3), "{}: {stderr}", args.join(" "));
    let want = format!("{prefix}{shown_path}\": ");
    assert!(
        stderr.starts_with(&want),
        "{}: stderr:\n{stderr}",
        args.join(" ")
    );
    assert!(
        !stderr.contains(OLD_MESSAGE),
        "{}: {stderr}",
        args.join(" ")
    );
    assert_eq!(
        snapshot(dir.path()),
        before,
        "{}: wrote files",
        args.join(" ")
    );
}

#[test]
fn missing_font_names_the_file() {
    for font in ["no-font.ttf", "fontdir"] {
        assert_asset_error(
            &[
                "watermark",
                "in.png",
                "--text",
                "hi",
                "--font",
                font,
                "-o",
                "o.png",
            ],
            FONT_PREFIX,
            font,
        );
    }
}

#[test]
fn missing_overlay_names_the_file() {
    for image in ["no-logo.png", "logodir"] {
        assert_asset_error(
            &["watermark", "in.png", "--image", image, "-o", "o.png"],
            OVERLAY_PREFIX,
            image,
        );
    }
    // Control: a readable non-image keeps its own error and exit code.
    let dir = project();
    write_file(dir.path(), "bad.png", b"junk\n");
    let out = run(
        dir.path(),
        &["watermark", "in.png", "--image", "bad.png", "-o", "o.png"],
    );
    assert_refused_code(&out, 4, "error: unsupported or undetectable image format\n");
}

fn assert_refused_code(out: &std::process::Output, code: i32, expected: &str) {
    assert_eq!(out.status.code(), Some(code), "stderr: {}", stderr_of(out));
    assert_eq!(stderr_of(out), expected);
}
