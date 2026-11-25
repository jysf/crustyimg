//! Integration tests for `build` destinations (every `out` resolves to one
//! identity before anything is written).
//!
//! Each test drives the real compiled binary with a temp project as its working
//! directory. A refused build must exit 2, print its message, and leave every
//! file exactly as it was: the `refused` helper snapshots the whole tree before
//! and after (links recorded as links, never followed).
//!
//! Expected messages are literals here, never computed by the code under test.

use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use image::{DynamicImage, ImageFormat, RgbImage};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// Scale every source to exactly 1x1, so a source that is overwritten by its own
/// output is visible in its dimensions.
const RECIPE: &str =
    "version = \"1\"\n\n[[step]]\nop = \"resize\"\nmode = \"exact\"\nwidth = 1\nheight = 1\n";

/// A recipe whose only step reads a watermark image.
const IMAGE_RECIPE: &str =
    "version = \"1\"\n\n[[step]]\nop = \"watermark\"\nimage = \"logo.png\"\n\
     gravity = \"southeast\"\nopacity = 1.0\nmargin = 0\ntile = false\n";

/// A recipe whose only step reads a watermark font.
const FONT_RECIPE: &str =
    "version = \"1\"\n\n[[step]]\nop = \"watermark\"\ntext = \"x\"\nfont = \"go.ttf\"\n";

/// A parseable font, so the font row builds on `main` instead of failing early.
const GO_FONT: &[u8] = include_bytes!("../assets/fonts/Go-Regular.ttf");

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn write_file(dir: &Path, rel: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Write a solid-color RGB JPEG at `dir/rel`.
fn write_jpeg(dir: &Path, rel: &str, w: u32, h: u32) -> PathBuf {
    let img = RgbImage::from_pixel(w, h, image::Rgb([200, 30, 30]));
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Jpeg)
        .unwrap();
    write_file(dir, rel, &buf.into_inner())
}

/// Write a solid-color RGB PNG at `dir/rel`.
fn write_png(dir: &Path, rel: &str, w: u32, h: u32) -> PathBuf {
    let img = RgbImage::from_pixel(w, h, image::Rgb([200, 30, 30]));
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .unwrap();
    write_file(dir, rel, &buf.into_inner())
}

/// One `[[target]]` block: `(source, out, name)`.
type T<'a> = (&'a str, &'a str, Option<&'a str>);

fn manifest(targets: &[T]) -> String {
    let mut text = String::from("version = 1\n");
    for (source, out, name) in targets {
        text.push_str(&format!(
            "\n[[target]]\nsource = \"{source}\"\nrecipe = \"r.toml\"\nout = \"{out}\"\n"
        ));
        if let Some(name) = name {
            text.push_str(&format!("name = \"{name}\"\n"));
        }
    }
    text
}

/// A project: the 1x1 recipe, a 2x2 `src/a.png`, and `targets` as the manifest.
fn project(targets: &[T]) -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "r.toml", RECIPE.as_bytes());
    write_png(root, "src/a.png", 2, 2);
    write_file(root, "crustyimg.build.toml", manifest(targets).as_bytes());
    dir
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .arg("build")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("binary should run")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn read_lock(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("crustyimg.build.lock")).expect("a lockfile")
}

// ── The refusal helper ────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
enum Node {
    File(Vec<u8>),
    Dir,
    Link(PathBuf),
}

/// Every entry under `root` by relative path. A link is recorded as a link and
/// never followed, so a link out of the tree is not walked.
fn snapshot(root: &Path) -> BTreeMap<String, Node> {
    fn walk(root: &Path, dir: &Path, into: &mut BTreeMap<String, Node>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let meta = std::fs::symlink_metadata(&path).unwrap();
            if meta.file_type().is_symlink() {
                into.insert(rel, Node::Link(std::fs::read_link(&path).unwrap()));
            } else if meta.is_dir() {
                into.insert(rel, Node::Dir);
                walk(root, &path, into);
            } else {
                into.insert(rel, Node::File(std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut into = BTreeMap::new();
    walk(root, root, &mut into);
    into
}

/// Run `crustyimg build <args>` in `dir`; assert exit 2, the exact message line,
/// and that the tree is byte-identical afterwards.
fn refused(dir: &Path, args: &[&str], message: &str) {
    let before = snapshot(dir);
    let out = run(dir, args);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(2), "exit code; stderr:\n{err}");
    assert!(
        err.contains(&format!("error: {message}\n")),
        "expected `error: {message}`; stderr:\n{err}"
    );
    assert_eq!(
        before,
        snapshot(dir),
        "a refused build must leave every file as it was"
    );
}

/// The collision message for `output`, first and second source.
fn collision(output: &str, first: &str, second: &str) -> String {
    format!(
        "output collision: \"{output}\" written by both \"{first}\" and \"{second}\" \
         — two inputs map to one output (disambiguate the name template, e.g. {{parent}}_{{stem}})"
    )
}

// ── Overlap with what the build reads ─────────────────────────────────────────

#[test]
fn out_is_the_source_folder_is_refused() {
    let dir = project(&[("src/a.png", "src", None)]);
    refused(
        dir.path(),
        &[],
        "target #0 would overwrite \"src/a.png\", which target #0 reads as a source",
    );
    assert!(!dir.path().join("crustyimg.build.lock").exists());
    assert!(!dir.path().join(".crustyimg").exists());
}

#[test]
fn out_dot_beside_its_source_is_refused() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "r.toml", RECIPE.as_bytes());
    write_png(root, "a.png", 2, 2);
    write_file(
        root,
        "crustyimg.build.toml",
        manifest(&[("a.png", ".", None)]).as_bytes(),
    );
    refused(
        root,
        &[],
        "target #0 would overwrite \"a.png\", which target #0 reads as a source",
    );
}

// ── Aliased directories ───────────────────────────────────────────────────────

#[test]
fn dotdot_alias_without_the_alias_dir_is_a_collision() {
    let dir = project(&[
        ("src/a.png", "dist", None),
        ("src2/a.png", "alias/../dist", None),
    ]);
    write_png(dir.path(), "src2/a.png", 2, 2);
    refused(
        dir.path(),
        &[],
        &collision("dist/a.{ext}", "src/a.png", "src2/a.png"),
    );
    assert!(!dir.path().join("alias").exists(), "no alias/ is created");
}

#[test]
fn dotdot_alias_with_the_alias_dir_is_a_collision() {
    let dir = project(&[
        ("src/a.png", "dist", None),
        ("src2/a.png", "alias/../dist", None),
    ]);
    write_png(dir.path(), "src2/a.png", 2, 2);
    std::fs::create_dir(dir.path().join("alias")).unwrap();
    refused(
        dir.path(),
        &[],
        &collision("dist/a.{ext}", "src/a.png", "src2/a.png"),
    );
}

#[test]
fn cur_dir_spellings_still_collide() {
    for spelling in ["./dist", "dist/", "dist/.", "dist//"] {
        let dir = project(&[("src/a.png", "dist", None), ("src2/a.png", spelling, None)]);
        write_png(dir.path(), "src2/a.png", 2, 2);
        refused(
            dir.path(),
            &[],
            &collision("dist/a.{ext}", "src/a.png", "src2/a.png"),
        );
    }
}

#[test]
fn out_dirs_differing_only_in_case_collide() {
    let dir = project(&[("src/a.png", "dist", None), ("src2/a.png", "DIST", None)]);
    write_png(dir.path(), "src2/a.png", 2, 2);
    refused(
        dir.path(),
        &[],
        &collision("dist/a.{ext}", "src/a.png", "src2/a.png"),
    );
}

#[test]
fn file_names_differing_only_in_case_collide() {
    let dir = project(&[("src/a.png", "dist", None), ("src2/A.png", "dist", None)]);
    write_png(dir.path(), "src2/A.png", 2, 2);
    refused(
        dir.path(),
        &[],
        &collision("dist/a.{ext}", "src/a.png", "src2/A.png"),
    );
}

#[test]
fn non_ascii_names_differing_only_in_case_collide() {
    // U+00E9 and U+00C9: lower- and upper-case e-acute.
    let dir = project(&[
        ("s1/\u{e9}.png", "dist", None),
        ("s2/\u{c9}.png", "dist", None),
    ]);
    write_png(dir.path(), "s1/\u{e9}.png", 2, 2);
    write_png(dir.path(), "s2/\u{c9}.png", 2, 2);
    refused(
        dir.path(),
        &[],
        &collision("dist/\u{e9}.{ext}", "s1/\u{e9}.png", "s2/\u{c9}.png"),
    );
}

// ── Chaining ──────────────────────────────────────────────────────────────────

const CHAINING: &str =
    "target #0 would overwrite \"dist/a.png\", which target #1 reads as a source; \
     one target's output cannot be another target's source";

#[test]
fn another_targets_source_is_refused() {
    let dir = project(&[("src/a.png", "dist", None), ("dist/a.png", "out2", None)]);
    write_png(dir.path(), "dist/a.png", 2, 2);
    refused(dir.path(), &[], CHAINING);
}

#[test]
fn another_targets_glob_source_is_refused() {
    let dir = project(&[("src/a.png", "dist", None), ("dist/*.png", "out2", None)]);
    write_png(dir.path(), "dist/a.png", 2, 2);
    refused(dir.path(), &[], CHAINING);
}

// ── Recipe, watermark assets, manifest, lockfile, cache dir ───────────────────

#[test]
fn the_recipe_file_is_refused() {
    let dir = project(&[("src/a.png", ".", Some("r.toml"))]);
    refused(
        dir.path(),
        &[],
        "target #0 would overwrite \"r.toml\", which is the recipe of target #0",
    );
}

#[test]
fn a_watermark_image_is_refused() {
    let dir = project(&[("src/a.png", ".", Some("logo.{ext}"))]);
    write_file(dir.path(), "r.toml", IMAGE_RECIPE.as_bytes());
    write_png(dir.path(), "logo.png", 4, 4);
    write_png(dir.path(), "src/a.png", 32, 32);
    refused(
        dir.path(),
        &[],
        "target #0 would overwrite \"logo.png\", which target #0's recipe reads as its watermark image",
    );
}

#[test]
fn a_watermark_font_is_refused() {
    let dir = project(&[("src/a.png", ".", Some("go.ttf"))]);
    write_file(dir.path(), "r.toml", FONT_RECIPE.as_bytes());
    write_file(dir.path(), "go.ttf", GO_FONT);
    write_png(dir.path(), "src/a.png", 64, 64);
    refused(
        dir.path(),
        &[],
        "target #0 would overwrite \"go.ttf\", which target #0's recipe reads as its watermark font",
    );
}

#[test]
fn the_manifest_is_refused() {
    let dir = project(&[("src/a.png", ".", Some("crustyimg.build.toml"))]);
    refused(
        dir.path(),
        &[],
        "target #0 would overwrite \"crustyimg.build.toml\", which is the build manifest",
    );
}

#[test]
fn the_lockfile_is_refused() {
    let dir = project(&[("src/a.png", ".", Some("crustyimg.build.lock"))]);
    refused(
        dir.path(),
        &[],
        "target #0 would overwrite \"crustyimg.build.lock\", which is the build lockfile",
    );
}

#[test]
fn the_cache_dir_is_refused() {
    let dir = project(&[("src/a.png", ".crustyimg/x", None)]);
    refused(
        dir.path(),
        &[],
        "target #0 would write \".crustyimg/x/a.{ext}\" inside .crustyimg/, the build's cache directory",
    );
}

// ── A refused build changes nothing ───────────────────────────────────────────

const SRC_OVERLAP: &str =
    "target #0 would overwrite \"src/a.png\", which target #0 reads as a source";

#[test]
fn a_refused_build_keeps_the_lockfile_and_cache_as_they_were() {
    let dir = project(&[("src/a.png", "dist", None)]);
    let first = run(dir.path(), &[]);
    assert_eq!(first.status.code(), Some(0), "{}", stderr(&first));
    assert!(dir.path().join("crustyimg.build.lock").exists());
    assert!(dir.path().join(".crustyimg").exists());

    write_file(
        dir.path(),
        "crustyimg.build.toml",
        manifest(&[("src/a.png", "src", None)]).as_bytes(),
    );
    refused(dir.path(), &[], SRC_OVERLAP);
}

#[test]
fn check_refuses_the_same_way_and_writes_nothing() {
    let dir = project(&[("src/a.png", "dist", None)]);
    let first = run(dir.path(), &[]);
    assert_eq!(first.status.code(), Some(0), "{}", stderr(&first));

    write_file(
        dir.path(),
        "crustyimg.build.toml",
        manifest(&[("src/a.png", "src", None)]).as_bytes(),
    );
    std::fs::remove_dir_all(dir.path().join("dist")).unwrap();
    std::fs::remove_dir_all(dir.path().join(".crustyimg")).unwrap();

    refused(dir.path(), &["--check"], SRC_OVERLAP);
    assert!(!dir.path().join("dist").exists());
    assert!(!dir.path().join(".crustyimg").exists());
}

// ── Extension, cache-dir and manifest-path cells ──────────────────────────────

#[test]
fn a_non_png_extension_overlap_is_refused_and_a_pinned_other_one_is_not() {
    // `{ext}` is checked under every extension the sink can write, not just png.
    let dir = project(&[("src/a.jpg", "src", None)]);
    write_jpeg(dir.path(), "src/a.jpg", 2, 2);
    refused(
        dir.path(),
        &[],
        "target #0 would overwrite \"src/a.jpg\", which target #0 reads as a source",
    );

    // A pinned `.png` target reads `a.jpg` and writes `a.png`: no overlap.
    let pinned = project(&[("src/a.jpg", "src", Some("{stem}.png"))]);
    write_jpeg(pinned.path(), "src/a.jpg", 2, 2);
    let out = run(pinned.path(), &[]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(pinned.path().join("src/a.png").exists());
}

#[test]
fn the_cache_dir_is_refused_in_any_letter_case() {
    let dir = project(&[("src/a.png", ".CRUSTYIMG/x", None)]);
    refused(
        dir.path(),
        &[],
        "target #0 would write \".CRUSTYIMG/x/a.{ext}\" inside .crustyimg/, the build's cache directory",
    );
}

#[test]
fn the_manifest_is_refused_when_given_by_a_non_default_path() {
    let dir = project(&[("src/a.png", "dist", None)]);
    write_file(
        dir.path(),
        "alt.toml",
        manifest(&[("src/a.png", ".", Some("alt.toml"))]).as_bytes(),
    );
    refused(
        dir.path(),
        &["alt.toml"],
        "target #0 would overwrite \"alt.toml\", which is the build manifest",
    );
}

#[test]
fn an_existing_lockfile_is_refused() {
    let dir = project(&[("src/a.png", "dist", None)]);
    let first = run(dir.path(), &[]);
    assert_eq!(first.status.code(), Some(0), "{}", stderr(&first));
    assert!(dir.path().join("crustyimg.build.lock").exists());

    write_file(
        dir.path(),
        "crustyimg.build.toml",
        manifest(&[("src/a.png", ".", Some("crustyimg.build.lock"))]).as_bytes(),
    );
    refused(
        dir.path(),
        &[],
        "target #0 would overwrite \"crustyimg.build.lock\", which is the build lockfile",
    );
}

#[test]
fn an_out_reached_as_x_dotdot_src_overlaps_the_source() {
    let dir = project(&[("src/a.png", "x/../src", None)]);
    refused(dir.path(), &[], SRC_OVERLAP);
    assert!(!dir.path().join("x").exists());
}

#[test]
fn an_out_in_another_letter_case_overlaps_the_source() {
    // Only a filesystem that merges `src` and `SRC` can overlap; elsewhere
    // `SRC` is another folder and the build is allowed.
    let probe = TempDir::new().unwrap();
    std::fs::create_dir(probe.path().join("probe")).unwrap();
    if !probe.path().join("PROBE").exists() {
        eprintln!("skipped: this filesystem keeps `probe` and `PROBE` apart");
        return;
    }
    let dir = project(&[("src/a.png", "SRC", None)]);
    refused(dir.path(), &[], SRC_OVERLAP);
}

// ── Names the filesystem merges but a lower-cased string does not ─────────────

/// Whether the temp filesystem treats `a` and `b` as one name, tested by
/// creating one and looking for the other. Locally a filesystem that keeps them
/// apart skips the test with the reason printed. Under `CI` (GitHub Actions sets
/// `CI=true`, and plain `cargo test` hides a passing test's output) it fails
/// instead, so a green run means the test ran.
#[cfg(target_os = "macos")]
fn fs_merges(a: &str, b: &str) -> bool {
    let probe = TempDir::new().unwrap();
    std::fs::write(probe.path().join(a), b"x").unwrap();
    let merged = probe.path().join(b).exists();
    if !merged {
        let why = format!("this filesystem keeps {a:?} and {b:?} apart");
        assert!(
            std::env::var_os("CI").is_none(),
            "{why}; on CI this test must run, not skip, or it proves nothing"
        );
        eprintln!("skipped: {why}");
    }
    merged
}

#[cfg(target_os = "macos")]
#[test]
fn macos_sharp_s_over_its_own_source_is_refused() {
    if !fs_merges("\u{df}-probe", "SS-probe") {
        return;
    }
    let dir = project(&[("src/\u{df}.png", "src", Some("SS.{ext}"))]);
    write_png(dir.path(), "src/\u{df}.png", 2, 2);
    refused(
        dir.path(),
        &[],
        "target #0 would overwrite \"src/SS.png\", which target #0 reads as a source",
    );
}

#[cfg(target_os = "macos")]
#[test]
fn macos_nfd_name_over_an_nfc_source_is_refused() {
    if !fs_merges("\u{e9}-probe", "e\u{301}-probe") {
        return;
    }
    let dir = project(&[("src/\u{e9}.png", "src", Some("e\u{301}.{ext}"))]);
    write_png(dir.path(), "src/\u{e9}.png", 2, 2);
    refused(
        dir.path(),
        &[],
        "target #0 would overwrite \"src/e\u{301}.png\", which target #0 reads as a source",
    );
}

// ── A source that would read its own output on the next build ─────────────────

#[test]
fn a_glob_source_that_would_read_its_output_is_refused_on_the_first_build() {
    let dir = project(&[("src/*.png", "src", Some("{stem}_w.{ext}"))]);
    refused(
        dir.path(),
        &[],
        "target #0 would overwrite \"src/a_w.png\", which target #0's source pattern \
         `src/*.png` would read on the next build",
    );
    assert!(!dir.path().join("crustyimg.build.lock").exists());
}

#[test]
fn a_folder_source_that_would_read_its_output_is_refused_on_the_first_build() {
    let dir = project(&[("src", "src", Some("{stem}_w.{ext}"))]);
    refused(
        dir.path(),
        &[],
        "target #0 would overwrite \"src/a_w.png\", which target #0's source folder \
         `src` would read on the next build",
    );
}

#[test]
fn every_spelling_of_a_listing_source_and_an_out_is_refused() {
    // Source spellings (glob and folder) against `out` spellings of one folder.
    // `x/../src/*.png` is absent: its `x` does not exist, so the glob lists
    // nothing and the build stops earlier ("no images matched").
    for source in ["src/*.png", "./src/*.png", "src/**/*.png", "src", "src/"] {
        for out in ["src", "./src", "src/", "a/../src"] {
            let dir = project(&[(source, out, Some("{stem}_w.{ext}"))]);
            let before = snapshot(dir.path());
            let result = run(dir.path(), &[]);
            let err = stderr(&result);
            assert_eq!(result.status.code(), Some(2), "{source} / {out}: {err}");
            assert!(
                err.contains("would overwrite \"src/a_w.png\"")
                    && err.contains(&format!("`{source}` would read on the next build")),
                "{source} / {out}: {err}"
            );
            assert_eq!(before, snapshot(dir.path()), "{source} / {out}");
        }
    }
}

#[test]
fn a_glob_that_does_not_match_its_output_still_builds_again() {
    let dir = project(&[("src/*.png", "src", Some("{stem}.webp"))]);
    for _ in 0..2 {
        let out = run(dir.path(), &[]);
        assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    }
    assert!(dir.path().join("src/a.webp").exists());
}

// ── Order of errors ───────────────────────────────────────────────────────────

#[test]
fn a_collision_is_reported_before_an_overlap() {
    // Both targets collide on `src/a.{ext}` and overlap `src/a.png`.
    let dir = project(&[("src2/a.png", "src", None), ("src/a.png", "src", None)]);
    write_png(dir.path(), "src2/a.png", 2, 2);
    refused(
        dir.path(),
        &[],
        &collision("src/a.{ext}", "src2/a.png", "src/a.png"),
    );
}

#[test]
fn a_dotdot_escape_is_reported_before_an_overlap() {
    let dir = project(&[("src/a.png", "src", None), ("src/a.png", "../x", None)]);
    let before = snapshot(dir.path());
    let out = run(dir.path(), &[]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(2), "{err}");
    assert!(
        err.contains("`out` escapes the build tree via `..`: ../x"),
        "{err}"
    );
    assert_eq!(before, snapshot(dir.path()));
}

// ── The lockfile path ─────────────────────────────────────────────────────────

#[test]
fn the_lockfile_names_the_root_relative_path() {
    let dir = project(&[
        ("src/a.png", "./dist/", None),
        ("src/b.png", "a/../dist", None),
    ]);
    write_png(dir.path(), "src/b.png", 2, 2);
    let out = run(dir.path(), &[]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let lock = read_lock(dir.path());
    assert!(lock.contains("path = \"dist/a.png\""), "{lock}");
    assert!(lock.contains("path = \"dist/b.png\""), "{lock}");
    assert!(
        !lock.contains("path = \"a/../"),
        "no `..` in a path: {lock}"
    );
}

#[test]
fn a_lockfile_with_an_old_spelling_drifts_once_on_paths() {
    let dir = project(&[("src/a.png", "a/../dist", None)]);
    let first = run(dir.path(), &[]);
    assert_eq!(first.status.code(), Some(0), "{}", stderr(&first));
    let fresh = read_lock(dir.path());

    // What the old code wrote for this `out`.
    let old = fresh.replace("path = \"dist/a.png\"", "path = \"a/../dist/a.png\"");
    assert_ne!(old, fresh, "the rewrite must change the lockfile");
    write_file(dir.path(), "crustyimg.build.lock", old.as_bytes());

    let check = run(dir.path(), &["--check"]);
    let err = stderr(&check);
    assert_eq!(check.status.code(), Some(7), "{err}");
    assert!(
        err.contains("drift: \"dist/a.png\": not in the lockfile"),
        "{err}"
    );
    assert!(
        err.contains("drift: \"a/../dist/a.png\": in the lockfile, not built"),
        "{err}"
    );

    let rebuild = run(dir.path(), &[]);
    assert_eq!(rebuild.status.code(), Some(0), "{}", stderr(&rebuild));
    assert_eq!(read_lock(dir.path()), fresh, "one build rewrites it");

    let again = run(dir.path(), &["--check"]);
    assert_eq!(again.status.code(), Some(0), "{}", stderr(&again));
}

// ── Links (Unix) ──────────────────────────────────────────────────────────────

#[cfg(unix)]
mod links {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn an_in_tree_link_alias_is_a_collision() {
        let dir = project(&[("src/a.png", "dist", None), ("src2/a.png", "link", None)]);
        write_png(dir.path(), "src2/a.png", 2, 2);
        std::fs::create_dir(dir.path().join("dist")).unwrap();
        symlink("dist", dir.path().join("link")).unwrap();
        refused(
            dir.path(),
            &[],
            &collision("dist/a.{ext}", "src/a.png", "src2/a.png"),
        );
    }

    #[test]
    fn dotdot_past_a_link_is_a_collision() {
        // `link/..` is `deep`, not the root: the kernel's answer, not a lexical one.
        let dir = project(&[
            ("src/a.png", "link/../dist", None),
            ("src2/a.png", "deep/dist", None),
        ]);
        write_png(dir.path(), "src2/a.png", 2, 2);
        std::fs::create_dir_all(dir.path().join("deep/a")).unwrap();
        symlink("deep/a", dir.path().join("link")).unwrap();
        refused(
            dir.path(),
            &[],
            &collision("deep/dist/a.{ext}", "src/a.png", "src2/a.png"),
        );
    }

    #[test]
    fn a_link_out_of_the_tree_is_refused_and_nothing_is_written() {
        let outside = TempDir::new().unwrap();
        let dir = project(&[("src/a.png", "link/x", None)]);
        symlink(outside.path(), dir.path().join("link")).unwrap();
        refused(
            dir.path(),
            &[],
            "target #0's out \"link/x\" resolves outside the build tree through the link \"link\"; \
             build inside the tree and copy the outputs where you need them",
        );
        assert_eq!(
            std::fs::read_dir(outside.path()).unwrap().count(),
            0,
            "nothing is written outside the tree"
        );
    }

    #[test]
    fn a_hard_link_to_a_source_at_the_output_name_is_refused() {
        // The same file under two names: no string comparison sees it.
        let dir = project(&[("src/a.png", "src", Some("x.png"))]);
        std::fs::hard_link(dir.path().join("src/a.png"), dir.path().join("src/x.png")).unwrap();
        refused(
            dir.path(),
            &[],
            "target #0 would overwrite \"src/x.png\", which target #0 reads as a source",
        );
    }

    #[test]
    fn an_out_that_is_itself_a_link_out_of_the_tree_is_refused() {
        let outside = TempDir::new().unwrap();
        let dir = project(&[("src/a.png", "link", None)]);
        symlink(outside.path(), dir.path().join("link")).unwrap();
        refused(
            dir.path(),
            &[],
            "target #0's out \"link\" resolves outside the build tree through the link \"link\"; \
             build inside the tree and copy the outputs where you need them",
        );
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
    }

    #[test]
    fn a_glob_through_a_link_and_an_out_through_a_link_are_refused() {
        for (source, out) in [
            ("lnk/*.png", "src"),
            ("src/*.png", "lnk"),
            ("lnk/*.png", "lnk"),
        ] {
            let dir = project(&[(source, out, Some("{stem}_w.{ext}"))]);
            symlink("src", dir.path().join("lnk")).unwrap();
            refused(
                dir.path(),
                &[],
                &format!(
                    "target #0 would overwrite \"src/a_w.png\", which target #0's source \
                     pattern `{source}` would read on the next build"
                ),
            );
        }
    }

    #[test]
    fn an_out_that_is_a_link_to_the_source_folder_overlaps_the_source() {
        let dir = project(&[("src/a.png", "lnk", None)]);
        symlink("src", dir.path().join("lnk")).unwrap();
        refused(dir.path(), &[], SRC_OVERLAP);
    }

    #[test]
    fn a_link_planted_at_an_output_file_makes_build_exit_6_and_is_not_followed() {
        // The sink's link refusal exits 5 on `apply`; `build` aggregates per-input
        // failures, so it exits 6 for the same refusal. (The folder-moved race is
        // not deterministic to drive end to end; the unit row beside
        // `write_planned` pins that one.)
        let victim = TempDir::new().unwrap();
        let target = victim.path().join("keep.png");
        std::fs::write(&target, b"untouched").unwrap();
        let dir = project(&[("src/a.png", "dist", None)]);
        std::fs::create_dir(dir.path().join("dist")).unwrap();
        symlink(&target, dir.path().join("dist/a.png")).unwrap();

        let out = run(dir.path(), &[]);
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(6), "{err}");
        assert!(err.contains("1 of 1 inputs failed"), "{err}");
        assert_eq!(std::fs::read(&target).unwrap(), b"untouched");
    }

    #[test]
    fn an_in_tree_link_builds_and_locks_the_real_path() {
        let dir = project(&[("src/a.png", "dist", None)]);
        std::fs::create_dir_all(dir.path().join("build/dist")).unwrap();
        symlink("build/dist", dir.path().join("dist")).unwrap();
        let out = run(dir.path(), &[]);
        assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
        assert!(dir.path().join("build/dist/a.png").exists());
        assert!(read_lock(dir.path()).contains("path = \"build/dist/a.png\""));
    }

    #[test]
    fn a_build_started_through_a_linked_path_locks_root_relative_paths() {
        let real = project(&[("src/a.png", "dist", None)]);
        let holder = TempDir::new().unwrap();
        let via = holder.path().join("proj");
        symlink(real.path(), &via).unwrap();
        let out = run(&via, &[]);
        assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
        assert!(read_lock(real.path()).contains("path = \"dist/a.png\""));
    }
}
