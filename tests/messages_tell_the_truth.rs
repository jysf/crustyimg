//! Messages that used to say something false now say what is true:
//!
//! - a readable folder with no image at its top level says so, and how to reach
//!   subfolders, instead of "input not found or unreadable";
//! - a glob that matches no image says so;
//! - a missing path, or a folder that cannot be listed, keeps the not-found line;
//! - a one-file verb given a folder says it is a folder;
//! - `-q` names the conflict the user actually has;
//! - the `meta` format messages read as sentences;
//! - a path holding `"` is quoted so the reader can see where it ends.
//!
//! Every row drives the real binary in a temp dir with relative paths, assigns the
//! output first, then checks the exit code, then the exact stderr. The expected lines
//! are literals here on purpose: a test that imported the code's own text could not
//! catch a changed message. Fixtures are made with `image` or written as literal
//! bytes, never by crustyimg itself.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use image::{DynamicImage, ImageFormat, RgbImage};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// The three folder shapes with no image at the top level.
const SHAPES: [&str; 3] = ["sub", "txt", "empty"];

const NOT_FOUND_NOPE: &str = "error: input not found or unreadable: nope\n";
const OLD_READ_LINE: &str = "error: could not read image input\n";

// ── Fixtures ────────────────────────────────────────────────────────────────────

fn write_file(dir: &Path, rel: &str, bytes: &[u8]) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, bytes).unwrap();
}

fn encoded(format: ImageFormat) -> Vec<u8> {
    let img = RgbImage::from_pixel(16, 16, image::Rgb([200, 30, 30]));
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, format)
        .unwrap();
    buf.into_inner()
}

/// A 1x1 GIF, as literal bytes.
const GIF: &[u8] = b"GIF89a\x01\x00\x01\x00\x80\x00\x00\x00\x00\x00\xff\xff\xff,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02D\x01\x00;";

/// WebP magic bytes plus padding. The metadata lane sniffs magic bytes only, so no
/// WebP codec is needed to reach its message (the lean build included).
const WEBP: &[u8] =
    b"RIFF\x1a\x00\x00\x00WEBPVP8L\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

/// A temp dir holding:
/// - `sub/inner/a.png` (a folder whose only image sits one level down),
/// - `txt/notes.txt` (a folder of non-images), `empty/`,
/// - `img/a.png` (a folder that does hold an image),
/// - `a.png`, `a.jpg`, `b.jpg`, `a.gif`, `a.webp`, `junk.jpg` (not an image),
/// - `r.toml` (one `invert` step).
fn project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let png = encoded(ImageFormat::Png);
    let jpeg = encoded(ImageFormat::Jpeg);
    write_file(root, "sub/inner/a.png", &png);
    write_file(root, "txt/notes.txt", b"notes\n");
    std::fs::create_dir_all(root.join("empty")).unwrap();
    write_file(root, "img/a.png", &png);
    write_file(root, "a.png", &png);
    write_file(root, "a.jpg", &jpeg);
    write_file(root, "b.jpg", &jpeg);
    write_file(root, "a.gif", GIF);
    write_file(root, "a.webp", WEBP);
    write_file(root, "junk.jpg", b"not an image at all");
    write_file(
        root,
        "r.toml",
        b"version = \"1\"\n\n[[step]]\nop = \"invert\"\n",
    );
    dir
}

/// Every file under `root` (path -> SHA-256). Equal snapshots before and after a run
/// show it wrote nothing; an exit code alone cannot.
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
    String::from_utf8_lossy(&out.stderr).replace("\r\n", "\n")
}

/// The line a folder with no top-level image prints, newline included.
fn folder_line(name: &str) -> String {
    format!(
        "error: no images in folder '{name}': only its top level is read; to include subfolders, use a glob such as '{name}/**/*'\n"
    )
}

/// Run `args`, then assert: exit `code`, stderr exactly `expected`.
fn assert_exact(dir: &Path, args: &[&str], code: i32, expected: &str) -> std::process::Output {
    let out = run(dir, args);
    let stderr = stderr_of(&out);
    assert_eq!(
        out.status.code(),
        Some(code),
        "`{}`: want exit {code}, got {:?}; stderr: {stderr}",
        args.join(" "),
        out.status.code()
    );
    assert_eq!(stderr, expected, "`{}`: wrong stderr", args.join(" "));
    out
}

// ── FT-M1 ───────────────────────────────────────────────────────────────────────

/// The verbs that resolve inputs, each with the flags it needs. The folder argument
/// is appended last.
const VERB_ROWS: [&[&str]; 17] = [
    &["view"],
    &["resize", "--max", "8", "--out-dir", "od"],
    &["resize", "--max", "8"],
    &["convert", "--format", "jpeg", "--out-dir", "od"],
    &["thumbnail", "--out-dir", "od"],
    &["auto-orient", "--out-dir", "od"],
    &["edit", "--invert", "--out-dir", "od"],
    &["watermark", "--text", "hi", "--out-dir", "od"],
    &["optimize", "--format", "png", "--out-dir", "od"],
    &["optimize", "--out-dir", "od"],
    &["web", "--out-dir", "od"],
    &["apply", "--recipe", "r.toml", "--out-dir", "od"],
    &["meta", "strip", "--out-dir", "od"],
    &["meta", "clean", "--gps", "--out-dir", "od"],
    &["meta", "set", "--artist", "x", "--out-dir", "od"],
    &["info"],
    &["lint"],
];

#[test]
fn a_folder_with_no_top_level_image_says_so_on_every_verb() {
    let dir = project();
    let root = dir.path();
    let mut failures = Vec::new();
    let mut rows = 0;
    for verb in VERB_ROWS {
        for shape in SHAPES {
            rows += 1;
            let mut args: Vec<&str> = verb.to_vec();
            args.push(shape);
            let before = snapshot(root);
            let out = run(root, &args);
            let stderr = stderr_of(&out);
            let after = snapshot(root);
            let want = folder_line(shape);
            let mut wrong = Vec::new();
            if out.status.code() != Some(3) {
                wrong.push(format!("exit {:?}", out.status.code()));
            }
            if stderr != want {
                wrong.push("stderr".to_owned());
            }
            if !out.stdout.is_empty() {
                wrong.push(format!("{} bytes on stdout", out.stdout.len()));
            }
            if before != after || root.join("od").exists() || root.join("o.png").exists() {
                wrong.push("wrote something".to_owned());
            }
            if !wrong.is_empty() {
                failures.push(format!(
                    "`{}` ({shape}): {}; stderr: {stderr:?}",
                    args.join(" "),
                    wrong.join(", ")
                ));
            }
        }
    }
    assert_eq!(rows, 51, "the table should drive 17 verbs x 3 shapes");
    assert!(
        failures.is_empty(),
        "{} of {rows} rows wrong:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ── FT-M2 ───────────────────────────────────────────────────────────────────────

#[test]
fn a_build_target_whose_source_is_a_folder_with_no_images_says_so_and_writes_nothing() {
    let dir = project();
    let root = dir.path();
    for shape in SHAPES {
        let manifest = format!(
            "version = 1\n\n[[target]]\nsource = \"{shape}\"\nrecipe = \"r.toml\"\nout = \"bo\"\n"
        );
        write_file(root, "b.toml", manifest.as_bytes());
        let want = format!(
            "error: target #0 (recipe r.toml, out bo)\n{}",
            folder_line(shape)
        );
        assert_exact(root, &["build", "b.toml"], 3, &want);
        assert!(!root.join("bo").exists(), "build ({shape}) created bo/");
    }

    // Target 0 has an image, target 1 does not: every target is checked before any
    // write, so neither output folder appears.
    write_file(
        root,
        "b2.toml",
        b"version = 1\n\n[[target]]\nsource = \"img\"\nrecipe = \"r.toml\"\nout = \"bo0\"\n\n[[target]]\nsource = \"sub\"\nrecipe = \"r.toml\"\nout = \"bo1\"\n",
    );
    let want = format!(
        "error: target #1 (recipe r.toml, out bo1)\n{}",
        folder_line("sub")
    );
    assert_exact(root, &["build", "b2.toml"], 3, &want);
    assert!(!root.join("bo0").exists(), "two-target build created bo0/");
    assert!(!root.join("bo1").exists(), "two-target build created bo1/");
}

// ── FT-M3 ───────────────────────────────────────────────────────────────────────

#[test]
fn the_folder_name_is_derived_from_the_argument() {
    let dir = project();
    let root = dir.path();
    let out = assert_exact(
        root,
        &["resize", "--max", "8", "--out-dir", "od", "sub/"],
        3,
        &folder_line("sub"),
    );
    let stderr = stderr_of(&out);
    assert!(!stderr.contains("'sub/'"), "echoed the argument: {stderr}");
    assert!(!stderr.contains("sub//"), "doubled the separator: {stderr}");

    // The hint works: the suggested glob reaches the subfolder's image.
    let out = run(
        root,
        &["resize", "sub/**/*", "--max", "8", "--out-dir", "gl"],
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "the suggested glob failed: {}",
        stderr_of(&out)
    );
    assert!(root.join("gl/a.png").is_file(), "gl/a.png was not written");
}

// ── FT-M4 ───────────────────────────────────────────────────────────────────────

#[test]
fn a_glob_that_matches_no_image_says_so() {
    let dir = project();
    let root = dir.path();
    for glob in ["txt/*", "nope/*"] {
        let want = format!("error: no images matched '{glob}'\n");
        assert_exact(
            root,
            &["resize", "--max", "8", "--out-dir", "od", glob],
            3,
            &want,
        );
    }
    assert_exact(
        root,
        &["lint", "txt/*"],
        3,
        "error: no images matched 'txt/*'\n",
    );
}

// ── FT-M5 ───────────────────────────────────────────────────────────────────────

#[test]
fn a_missing_path_keeps_the_not_found_message() {
    let dir = project();
    let root = dir.path();
    for args in [
        &["resize", "--max", "8", "--out-dir", "od", "nope"][..],
        &["lint", "nope"][..],
        &["info", "nope"][..],
    ] {
        assert_exact(root, args, 3, NOT_FOUND_NOPE);
    }
}

#[cfg(unix)]
#[test]
fn an_unreadable_folder_keeps_the_not_found_message() {
    use std::os::unix::fs::PermissionsExt;

    let dir = project();
    let root = dir.path();
    let locked = root.join("locked");
    std::fs::create_dir(&locked).unwrap();
    write_file(&locked, "a.png", &encoded(ImageFormat::Png));
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
    assert!(
        std::fs::read_dir(&locked).is_err(),
        "precondition: running as root? an unreadable folder cannot be made here"
    );

    let mut got = Vec::new();
    for args in [
        &["resize", "--max", "8", "--out-dir", "od", "locked"][..],
        &["lint", "locked"][..],
        &["lint", "locked", "sub"][..],
    ] {
        let out = run(root, args);
        got.push((args.join(" "), out.status.code(), stderr_of(&out)));
    }
    // Restore the mode before asserting, so the temp dir can be removed.
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();

    for (args, code, stderr) in got {
        assert_eq!(code, Some(3), "`{args}`: stderr {stderr}");
        assert_eq!(
            stderr, "error: input not found or unreadable: locked\n",
            "`{args}`: wrong stderr"
        );
    }
}

// ── FT-M6 ───────────────────────────────────────────────────────────────────────

#[test]
fn several_arguments_keep_their_behaviour_and_name_the_first_cause() {
    let dir = project();
    let root = dir.path();

    let out = run(root, &["resize", "--max", "8", "a.png", "empty"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "resize a.png empty: {}",
        stderr_of(&out)
    );
    assert!(!out.stdout.is_empty(), "resize a.png empty wrote no image");

    let out = run(root, &["lint", "txt/*", "a.png"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "lint 'txt/*' a.png: {}",
        stderr_of(&out)
    );

    assert_exact(
        root,
        &["resize", "--max", "8", "--out-dir", "od", "sub", "empty"],
        3,
        &folder_line("sub"),
    );
    assert_exact(root, &["lint", "nope", "sub"], 3, NOT_FOUND_NOPE);
    assert_exact(root, &["lint", "sub", "nope"], 3, &folder_line("sub"));
}

// ── FT-M7 ───────────────────────────────────────────────────────────────────────

#[test]
fn a_single_file_verb_given_a_folder_says_it_is_a_folder() {
    let dir = project();
    let root = dir.path();
    let folder = "error: 'img' is a folder, not an image file\n";

    let before = snapshot(root);
    assert_exact(
        root,
        &["responsive", "--widths", "8", "--out-dir", "od", "img"],
        3,
        folder,
    );
    assert_eq!(before, snapshot(root), "responsive wrote something");
    assert!(!root.join("od").exists(), "responsive created od/");

    assert_exact(root, &["diff", "img", "a.png"], 3, folder);
    assert_exact(root, &["diff", "a.png", "img"], 3, folder);
    assert_exact(
        root,
        &[
            "meta", "copy", "--from", "img", "--to", "a.png", "-o", "o.jpg",
        ],
        3,
        folder,
    );
    assert_exact(
        root,
        &["meta", "copy", "--from", "a.jpg", "--to", "img"],
        3,
        folder,
    );

    // A missing file keeps the old line.
    assert_exact(
        root,
        &["responsive", "--widths", "8", "--out-dir", "od", "nope.png"],
        3,
        OLD_READ_LINE,
    );
    assert_exact(root, &["diff", "nope.png", "a.png"], 3, OLD_READ_LINE);
}

// ── FT-M8 ───────────────────────────────────────────────────────────────────────

#[test]
fn quality_flag_names_the_conflict_the_user_actually_has() {
    let dir = project();
    let root = dir.path();
    let used_with = |verb: &str| {
        format!(
            "error: -q/--quality cannot be used with {verb}, which chooses the quality itself; to encode at a fixed quality, use `crustyimg convert -q N`\n"
        )
    };
    let search = "error: -q/--quality cannot be combined with --target or --ssim, which choose the quality to reach a score\n";
    let budget = "error: -q/--quality cannot be combined with --max-size, which chooses the quality to fit the size\n";

    assert_exact(
        root,
        &["optimize", "-q", "10", "a.png", "-o", "o.png"],
        2,
        &used_with("optimize"),
    );
    assert_exact(
        root,
        &["web", "-q", "10", "a.png", "-o", "o.png"],
        2,
        &used_with("web"),
    );
    assert_exact(
        root,
        &[
            "optimize", "-q", "10", "--target", "high", "a.png", "-o", "o.png",
        ],
        2,
        search,
    );
    assert_exact(
        root,
        &[
            "optimize", "-q", "10", "--ssim", "80", "a.png", "-o", "o.png",
        ],
        2,
        search,
    );
    assert_exact(
        root,
        &[
            "optimize",
            "-q",
            "10",
            "--max-size",
            "2k",
            "a.png",
            "-o",
            "o.png",
        ],
        2,
        budget,
    );
    assert_exact(
        root,
        &[
            "convert",
            "--format",
            "jpeg",
            "-q",
            "10",
            "--max-size",
            "2k",
            "a.png",
            "-o",
            "x.jpg",
        ],
        2,
        budget,
    );
    assert!(!root.join("o.png").exists() && !root.join("x.jpg").exists());

    // The advice works.
    let out = run(
        root,
        &[
            "convert",
            "--format",
            "jpeg",
            "-q",
            "10",
            "a.png",
            "-o",
            "fixed.jpg",
        ],
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr_of(&out));
    let bytes = std::fs::read(root.join("fixed.jpg")).unwrap();
    assert!(bytes.starts_with(&[0xFF, 0xD8]), "fixed.jpg is not a JPEG");
}

// ── FT-M9 ───────────────────────────────────────────────────────────────────────

#[test]
fn meta_format_messages_are_sentences() {
    let dir = project();
    let root = dir.path();
    let lane = "error: the meta commands support JPEG and PNG only, and this file";
    let rows: [(&[&str], String); 6] = [
        (
            &["meta", "strip", "-o", "o.gif", "a.gif"],
            format!("{lane} is GIF\n"),
        ),
        (
            &["meta", "clean", "--gps", "-o", "o.webp", "a.webp"],
            format!("{lane} is WEBP\n"),
        ),
        (
            &["meta", "set", "--artist", "x", "junk.jpg"],
            format!("{lane}'s format is not recognised\n"),
        ),
        (
            &[
                "meta", "copy", "--from", "a.png", "--to", "b.jpg", "-o", "o.jpg",
            ],
            "error: meta copy supports JPEG only, and the --from file is PNG\n".to_owned(),
        ),
        (
            &[
                "meta", "copy", "--from", "b.jpg", "--to", "a.gif", "-o", "o.gif",
            ],
            "error: meta copy supports JPEG only, and the --to file is GIF\n".to_owned(),
        ),
        (
            &[
                "meta", "copy", "--from", "junk.jpg", "--to", "b.jpg", "-o", "o.jpg",
            ],
            "error: meta copy supports JPEG only, and the --from file's format is not recognised\n"
                .to_owned(),
        ),
    ];
    for (args, want) in rows {
        let out = assert_exact(root, args, 4, &want);
        let stderr = stderr_of(&out);
        for bad in ["does not support", " yet", "in v1", "Gif", "WebP"] {
            assert!(
                !stderr.contains(bad),
                "`{}`: contains {bad:?}: {stderr}",
                args.join(" ")
            );
        }
    }

    // In a batch the per-input line keeps its path prefix and the run exits 6.
    let out = run(
        root,
        &["meta", "strip", "--out-dir", "od", "a.gif", "a.png"],
    );
    let stderr = stderr_of(&out);
    assert_eq!(out.status.code(), Some(6), "batch: {stderr}");
    assert_eq!(
        stderr.lines().next(),
        Some("error: a.gif: the meta commands support JPEG and PNG only, and this file is GIF"),
        "batch: wrong first line"
    );
}

// ── FT-M10 ──────────────────────────────────────────────────────────────────────

/// Run `args`, assert exit `code` and that stderr starts with `prefix`.
fn assert_prefix(dir: &Path, args: &[&str], code: i32, prefix: &str) -> String {
    let out = run(dir, args);
    let stderr = stderr_of(&out);
    assert_eq!(
        out.status.code(),
        Some(code),
        "`{}`: stderr {stderr}",
        args.join(" ")
    );
    assert!(
        stderr.starts_with(prefix),
        "`{}`: want prefix {prefix:?}, got {stderr:?}",
        args.join(" ")
    );
    stderr
}

#[test]
fn a_path_with_a_double_quote_is_quoted_unambiguously() {
    // None of these files exist, so no file name holding `"` is ever created (Windows
    // does not allow one).
    let dir = project();
    let root = dir.path();
    assert_prefix(
        root,
        &[
            "watermark",
            "--text",
            "hi",
            "--font",
            "a\"b.ttf",
            "a.png",
            "-o",
            "o.png",
        ],
        3,
        "error: could not read font file \"a\\\"b.ttf\": ",
    );
    assert_prefix(
        root,
        &[
            "watermark",
            "--image",
            "nope\"x.png",
            "a.png",
            "-o",
            "o.png",
        ],
        3,
        "error: could not read watermark image \"nope\\\"x.png\": ",
    );
    write_file(
        root,
        "wq.toml",
        b"version = \"1\"\n\n[[step]]\nop = \"watermark\"\nimage = 'm\"q.png'\n",
    );
    assert_prefix(
        root,
        &["apply", "--recipe", "wq.toml", "a.png", "-o", "o.png"],
        1,
        "error: recipe step 0 ('watermark'): could not read 'image' file \"m\\\"q.png\": ",
    );

    // A backslash is shown as typed, so a Windows path stays readable.
    let stderr = assert_prefix(
        root,
        &[
            "watermark",
            "--text",
            "hi",
            "--font",
            r"C:\dir\f.ttf",
            "a.png",
            "-o",
            "o.png",
        ],
        3,
        "error: could not read font file ",
    );
    assert!(
        stderr.contains(r#""C:\dir\f.ttf""#),
        "the backslashes changed: {stderr}"
    );
}

// ── FT-M11 ──────────────────────────────────────────────────────────────────────

#[test]
fn help_text_that_changed_is_true() {
    let dir = project();
    let root = dir.path();

    let out = run(root, &["lint", "--help"]);
    assert_eq!(out.status.code(), Some(0));
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(help.contains("where one exists"), "lint --help: {help}");
    assert!(!help.contains("each with"), "lint --help: {help}");

    let out = run(root, &["build", "--help"]);
    assert_eq!(out.status.code(), Some(0));
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(help.contains("different platform"), "build --help: {help}");
}
