//! Integration tests for the declared `crustyimg build` path (SPEC-063).
//!
//! All tests drive the real compiled binary via `env!("CARGO_BIN_EXE_crustyimg")`
//! with the temp project as the working directory — manifest paths (`source`,
//! `recipe`, `out`) resolve relative to the CWD (DEC-057). Fixtures are
//! synthesized in memory with the `image` crate — no committed binary files.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use image::{DynamicImage, ImageFormat, RgbImage};
use tempfile::TempDir;

mod common;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// A minimal recipe that resizes to max 16 (tiny, fast, verifiable).
const RESIZE_RECIPE: &str = r#"
version = "1"

[[step]]
op = "resize"
mode = "max"
width = 16
"#;

// ── Fixture helpers ───────────────────────────────────────────────────────────

/// Generate a solid-color RGB PNG at `dir/rel` (creating parent dirs).
fn write_png(dir: &Path, rel: &str, w: u32, h: u32) -> PathBuf {
    let img = RgbImage::from_pixel(w, h, image::Rgb([42u8, 100u8, 200u8]));
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .unwrap();
    write_file(dir, rel, &buf.into_inner())
}

/// Write raw bytes to `dir/rel` (creating parent dirs). Returns the path.
fn write_file(dir: &Path, rel: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Run `crustyimg build [args]` with `dir` as the working directory.
fn run_build(dir: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .arg("build")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("binary should run")
}

/// Assert a written output exists and has the expected dimensions.
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

/// A two-target project: one `source` glob string, one `source` list (with a
/// `name` template). Returns the temp dir.
fn two_target_project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "r.toml", RESIZE_RECIPE.as_bytes());
    write_png(root, "src_a/a1.png", 32, 32);
    write_png(root, "src_a/a2.png", 32, 32);
    write_png(root, "src_b/b1.png", 64, 64);
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src_a/*.png"
recipe = "r.toml"
out = "dist/a"

[[target]]
source = ["src_b/b1.png"]
recipe = "r.toml"
out = "dist/b"
name = "{stem}_web.{ext}"
"#,
    );
    dir
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// A valid manifest runs EVERY target: both targets' outputs land in their own
/// `out` dir, under their own name template, with the recipe applied.
#[test]
fn build_runs_all_targets() {
    let dir = two_target_project();
    let root = dir.path();

    let out = run_build(root, &[]);
    assert!(
        out.status.success(),
        "build should exit 0, got {:?}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );

    // Target 1: default template `{stem}.{ext}`, source format preserved (png).
    assert_dims(&root.join("dist/a/a1.png"), 16, 16);
    assert_dims(&root.join("dist/a/a2.png"), 16, 16);
    // Target 2: `{stem}_web.{ext}` template.
    assert_dims(&root.join("dist/b/b1_web.png"), 16, 16);
}

/// `crustyimg build` with no arg discovers `./crustyimg.build.toml`; an explicit
/// FILE path also works; a missing default manifest is a clear typed error (exit 3).
#[test]
fn build_discovers_default_manifest() {
    let dir = two_target_project();
    let root = dir.path();

    // No arg → discovers ./crustyimg.build.toml.
    assert!(run_build(root, &[]).status.success());
    assert!(root.join("dist/a/a1.png").exists());

    // Explicit FILE path → same manifest under a different name.
    std::fs::rename(
        root.join("crustyimg.build.toml"),
        root.join("custom.build.toml"),
    )
    .unwrap();
    let out = run_build(root, &["custom.build.toml"]);
    assert!(
        out.status.success(),
        "explicit FILE should work, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Missing default manifest → typed error, exit 3, no panic.
    let empty = TempDir::new().unwrap();
    let out = run_build(empty.path(), &[]);
    assert_eq!(out.status.code(), Some(3), "missing manifest should exit 3");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("crustyimg.build.toml"),
        "error should name the manifest it looked for, got: {stderr}"
    );
    assert!(!stderr.contains("panicked"), "must not panic: {stderr}");
}

/// A manifest with an unknown field or an unsupported version is rejected with a
/// typed error (exit 2) BEFORE any input is touched.
#[test]
fn build_rejects_bad_manifest_before_touching_inputs() {
    for (manifest, needle) in [
        (
            r#"
version = 1

[[target]]
source = "src_a/*.png"
recipe = "r.toml"
out = "dist/a"
bogus = 1
"#,
            "bogus",
        ),
        (
            r#"
version = 999

[[target]]
source = "src_a/*.png"
recipe = "r.toml"
out = "dist/a"
"#,
            "version",
        ),
    ] {
        let dir = two_target_project();
        let root = dir.path();
        write_file(root, "crustyimg.build.toml", manifest.as_bytes());

        let out = run_build(root, &[]);
        assert_eq!(
            out.status.code(),
            Some(2),
            "a malformed manifest should exit 2 (usage)"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(needle),
            "error should mention {needle}, got: {stderr}"
        );
        assert!(
            !root.join("dist").exists(),
            "no output may be written for a rejected manifest"
        );
    }
}

/// A target referencing a nonexistent recipe fails before ANY target writes.
#[test]
fn build_missing_recipe_fails_before_writing() {
    let dir = two_target_project();
    let root = dir.path();
    // Target 1 is fine; target 2's recipe does not exist. Nothing may be written.
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src_a/*.png"
recipe = "r.toml"
out = "dist/a"

[[target]]
source = "src_b/*.png"
recipe = "nope.toml"
out = "dist/b"
"#,
    );

    let out = run_build(root, &[]);
    assert_eq!(
        out.status.code(),
        Some(3),
        "a missing recipe file is an unreadable input (exit 3)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("nope.toml"),
        "error should name the missing recipe, got: {stderr}"
    );
    assert!(
        !root.join("dist").exists(),
        "no target may write when another target's recipe is unusable"
    );
}

/// `build` owns its declared outputs: a re-run overwrites them without `--yes`.
#[test]
fn build_reruns_idempotently_without_yes() {
    let dir = two_target_project();
    let root = dir.path();

    for run in 1..=2 {
        let out = run_build(root, &[]);
        assert!(
            out.status.success(),
            "run {run} should exit 0 (no --yes), stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_dims(&root.join("dist/a/a1.png"), 16, 16);
        assert_dims(&root.join("dist/b/b1_web.png"), 16, 16);
    }
}

/// A successful build prints a summary naming the targets and the output count.
#[test]
fn build_reports_summary() {
    let dir = two_target_project();
    let out = run_build(dir.path(), &[]);
    assert!(out.status.success());

    // Diagnostics go to stderr; stdout stays clean.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("dist/a") && stderr.contains("dist/b"),
        "summary should name each target's out dir, got: {stderr}"
    );
    assert!(
        stderr.contains("2 targets") && stderr.contains("3 outputs"),
        "summary should report targets run + outputs written, got: {stderr}"
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).is_empty(),
        "build must not write diagnostics to stdout"
    );

    // `--quiet` suppresses the summary.
    let quiet = run_build(dir.path(), &["--quiet"]);
    assert!(quiet.status.success());
    assert!(
        String::from_utf8_lossy(&quiet.stderr).is_empty(),
        "--quiet should suppress the summary"
    );
}

/// One undecodable source is a partial-batch failure (exit 6, DEC-015): the good
/// outputs are still written and the bad one is reported.
#[test]
fn build_partial_failure_is_exit_6() {
    let dir = two_target_project();
    let root = dir.path();
    write_file(root, "src_a/corrupt.png", b"this is not a PNG");

    let out = run_build(root, &[]);
    assert_eq!(
        out.status.code(),
        Some(6),
        "a per-output failure is a partial batch (exit 6)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("corrupt.png"),
        "the failing output should be reported, got: {stderr}"
    );
    // The good outputs of both targets are still written.
    assert_dims(&root.join("dist/a/a1.png"), 16, 16);
    assert_dims(&root.join("dist/a/a2.png"), 16, 16);
    assert_dims(&root.join("dist/b/b1_web.png"), 16, 16);
}

/// Write the minimal project fixtures (recipe + one source PNG) into `root`.
fn populate_min_project(root: &Path) {
    write_file(root, "r.toml", RESIZE_RECIPE.as_bytes());
    write_png(root, "src_a/a1.png", 32, 32);
    write_png(root, "src_a/a2.png", 32, 32);
}

/// A hostile manifest whose target `out` escapes the build tree — via `..` or an
/// absolute path — is rejected at manifest validation (exit 2, SPEC-068) BEFORE
/// any filesystem write, and NOTHING lands outside the tree. A legit relative
/// `out` still builds. Drives the real binary with a hostile FILE, not a
/// constructed struct — this is the ship-blocker the threat-model review found.
#[test]
fn build_rejects_out_directory_escape() {
    // The project is NESTED inside a dedicated outer temp dir, so a `..` escape
    // lands in a location this test fully OWNS (`<outer>/ESCAPE/...`) — not a
    // shared system-temp parent where a leftover would make the check flaky.
    // ── 1) Relative `..` escape ──────────────────────────────────────────────
    {
        let outer = TempDir::new().unwrap();
        let root = outer.path().join("proj");
        std::fs::create_dir_all(&root).unwrap();
        populate_min_project(&root);
        // `../ESCAPE/planted` climbs out of <outer>/proj into <outer>/ESCAPE.
        // Without the clamp this writes re-encoded bytes there at exit 0; with
        // it, exit 2 and nothing is written.
        write_file(
            &root,
            "crustyimg.build.toml",
            b"version = 1\n[[target]]\nsource = \"src_a/*.png\"\nrecipe = \"r.toml\"\nout = \"../ESCAPE/planted\"\n",
        );

        let out = run_build(&root, &[]);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(2),
            "a `..` out-escape must be rejected at validation (exit 2), stderr: {stderr}"
        );
        assert!(
            stderr.contains("out") && stderr.contains("escapes the build tree"),
            "error should name the escaping `out`, got: {stderr}"
        );
        assert!(!stderr.contains("panicked"), "must not panic: {stderr}");
        // Nothing may be written at the would-be escape target (owned by `outer`).
        assert!(
            !outer.path().join("ESCAPE").exists(),
            "no bytes may be written outside the build tree"
        );
    }

    // ── 2) Absolute escape ───────────────────────────────────────────────────
    // An absolute `out` pointing at a sibling temp dir OUTSIDE the project.
    {
        let escape_root = TempDir::new().unwrap();
        let planted = escape_root.path().join("planted");
        // TOML basic strings treat `\` as an escape; double it so a Windows path
        // (C:\...) round-trips (no-op on Unix).
        let abs = planted.to_string_lossy().replace('\\', "\\\\");

        let dir = TempDir::new().unwrap();
        let root = dir.path();
        populate_min_project(root);
        write_file(
            root,
            "crustyimg.build.toml",
            format!(
                "version = 1\n[[target]]\nsource = \"src_a/*.png\"\nrecipe = \"r.toml\"\nout = \"{abs}\"\n"
            )
            .as_bytes(),
        );

        let out = run_build(root, &[]);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(2),
            "an absolute out-escape must be rejected (exit 2), stderr: {stderr}"
        );
        assert!(
            !planted.exists(),
            "no bytes may be written at the absolute escape target"
        );
    }

    // ── 3) A legit contained `out` still builds ──────────────────────────────
    {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        populate_min_project(root);
        write_file(
            root,
            "crustyimg.build.toml",
            br#"
version = 1

[[target]]
source = "src_a/*.png"
recipe = "r.toml"
out = "dist"

[[target]]
source = "src_a/a1.png"
recipe = "r.toml"
out = "build/thumbs"
name = "{stem}_t.{ext}"
"#,
        );
        let out = run_build(root, &[]);
        assert!(
            out.status.success(),
            "a contained `out` (dist, build/thumbs) must still build, stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_dims(&root.join("dist/a1.png"), 16, 16);
        assert_dims(&root.join("dist/a2.png"), 16, 16);
        assert_dims(&root.join("build/thumbs/a1_t.png"), 16, 16);
    }
}

/// An empty glob / missing source path is a hard source error, not a silent no-op.
#[test]
fn build_empty_source_is_an_error() {
    let dir = two_target_project();
    let root = dir.path();
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src_a/*.jpg"
recipe = "r.toml"
out = "dist/a"
"#,
    );

    let out = run_build(root, &[]);
    assert_eq!(
        out.status.code(),
        Some(3),
        "an empty glob should be an input-not-found error"
    );
    assert!(!root.join("dist").exists(), "nothing may be written");
}

// ── SPEC-111: `build` runs bundled recipes ──────────────────────────────────

/// AC-1: `build` completes and writes output for a target bound to EACH
/// bundled recipe, by NAME. Fails today (`unknown operation 'optimize'`) for
/// all three.
#[test]
fn build_runs_each_bundled_recipe_by_name() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_png(root, "src/photo.png", 64, 64);
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/photo.png"
recipe = "web"
out = "dist/web"

[[target]]
source = "src/photo.png"
recipe = "gallery"
out = "dist/gallery"

[[target]]
source = "src/photo.png"
recipe = "product"
out = "dist/product"
"#,
    );

    let out = run_build(root, &[]);
    assert!(
        out.status.success(),
        "build must run every bundled recipe by name, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    for name in ["web", "gallery", "product"] {
        let written: Vec<_> = std::fs::read_dir(root.join("dist").join(name))
            .unwrap_or_else(|e| panic!("dist/{name} should exist: {e}"))
            .collect();
        assert_eq!(
            written.len(),
            1,
            "bundled recipe {name} should write exactly one output"
        );
    }
}

/// AC-1: a bundled recipe bound by FILE PATH (not just by name) also runs —
/// today's failure is identical either way.
#[test]
fn build_runs_a_bundled_recipe_by_path() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_png(root, "src/photo.png", 64, 64);
    write_file(root, "web.toml", include_bytes!("../recipes/web.toml"));
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/photo.png"
recipe = "web.toml"
out = "dist"
"#,
    );

    let out = run_build(root, &[]);
    assert!(
        out.status.success(),
        "build must run a bundled recipe bound by PATH, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let written: Vec<_> = std::fs::read_dir(root.join("dist")).unwrap().collect();
    assert_eq!(written.len(), 1, "should write exactly one output");
}

/// AC-2: the output is the format the DECISION chose, not the source format.
/// On a photographic PNG source through the bundled `web` recipe, the bytes
/// are AVIF (asserted via `image::guess_format`, not the extension alone —
/// AVIF-in-a-`.png` would pass an extension-only check) and the file is named
/// `.avif`, matching what `apply --recipe web` produces from the SAME input.
///
/// Gated to `avif` AND NOT `webp-lossy`: AVIF is the only LOSSY candidate the
/// fast decision has on that leg, so it reliably wins for photographic
/// content. Once `webp-lossy` is also built, lossy WebP becomes a second
/// competing lossy candidate and the byte-race winner is a measured, not
/// assumed, outcome (mirrors `web_equals_apply_recipe_web` in `tests/cli.rs`)
/// — [`build_decided_format_matches_apply_on_every_feature_leg`] below covers
/// AC-2's "one rule" requirement on every leg without assuming a winner.
#[cfg(all(feature = "avif", not(feature = "webp-lossy")))]
#[test]
fn build_writes_the_decided_format_not_the_source_format() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "src/photo.png", &common::detailed_png(256, 256));
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/photo.png"
recipe = "web"
out = "dist"
"#,
    );

    let out = run_build(root, &[]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let avif_path = root.join("dist/photo.avif");
    assert!(
        avif_path.exists(),
        "the decided AVIF format must name the output .avif"
    );
    let build_bytes = std::fs::read(&avif_path).unwrap();
    assert_eq!(
        image::guess_format(&build_bytes).ok(),
        Some(ImageFormat::Avif),
        "the output bytes must really be AVIF, not source-format bytes under a .avif name"
    );

    // Matches `apply --recipe web` on the same input — decision 1's "one rule"
    // requirement, checked directly rather than assumed.
    let apply_dir = root.join("apply_out");
    let apply = Command::new(BIN)
        .args(["apply", "--recipe", "web", "src/photo.png", "--out-dir"])
        .arg(&apply_dir)
        .current_dir(root)
        .output()
        .expect("failed to run apply");
    assert!(
        apply.status.success(),
        "apply stderr: {}",
        String::from_utf8_lossy(&apply.stderr)
    );
    let apply_bytes = std::fs::read(apply_dir.join("photo.avif")).unwrap();
    assert_eq!(
        build_bytes, apply_bytes,
        "build must match apply --recipe web byte-for-byte on the same input"
    );
}

/// AC-2's "one rule" requirement (decision 1), on EVERY feature leg — unlike
/// the test above, this does not assume which candidate the fast decision
/// picks (that varies once `webp-lossy` adds a second competing lossy
/// candidate). It asserts what must hold regardless: the written file's
/// extension matches its REAL decoded bytes (not source-format bytes under a
/// modernized name), and `build` matches `apply --recipe web` byte-for-byte
/// on the same input.
#[test]
fn build_decided_format_matches_apply_on_every_feature_leg() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "src/photo.png", &common::detailed_png(256, 256));
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/photo.png"
recipe = "web"
out = "dist"
"#,
    );

    let out = run_build(root, &[]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let written: Vec<String> = std::fs::read_dir(root.join("dist"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(written.len(), 1, "exactly one output written");
    let written_name = &written[0];
    let build_bytes = std::fs::read(root.join("dist").join(written_name)).unwrap();
    let guessed =
        image::guess_format(&build_bytes).expect("output must be a real, decodable image");
    let guessed_ext = match guessed {
        ImageFormat::Avif => "avif",
        ImageFormat::WebP => "webp",
        ImageFormat::Png => "png",
        // The lean leg (no avif, no webp-lossy) has no built lossy codec of
        // its own, so the never-bigger fallback reaches for a baseline JPEG
        // (`fast_fallback_lossy_entry`, `src/cli/optimize.rs`).
        ImageFormat::Jpeg => "jpg",
        other => panic!("unexpected decided format for a photographic source: {other:?}"),
    };
    assert!(
        written_name.ends_with(&format!(".{guessed_ext}")),
        "the written file's extension must match its REAL bytes: {written_name} vs decoded {guessed_ext}"
    );

    let apply_dir = root.join("apply_out");
    let apply = Command::new(BIN)
        .args(["apply", "--recipe", "web", "src/photo.png", "--out-dir"])
        .arg(&apply_dir)
        .current_dir(root)
        .output()
        .expect("failed to run apply");
    assert!(
        apply.status.success(),
        "apply stderr: {}",
        String::from_utf8_lossy(&apply.stderr)
    );
    let apply_written: Vec<String> = std::fs::read_dir(&apply_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        apply_written,
        vec![written_name.clone()],
        "build and apply --recipe web must agree on the decided FORMAT for the same input"
    );
    let apply_bytes = std::fs::read(apply_dir.join(written_name)).unwrap();
    assert_eq!(
        build_bytes, apply_bytes,
        "build must match apply --recipe web byte-for-byte on the same input"
    );
}

/// AC-3: a target whose template names a LITERAL extension (`{stem}.png`)
/// pins the format: a real PNG, decision skipped — the `build` twin of
/// `apply --recipe web -o hero.png`. Assert the bytes are really PNG.
#[test]
fn build_honours_a_literal_extension_template_as_a_format_pin() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "src/photo.png", &common::detailed_png(256, 256));
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/photo.png"
recipe = "web"
out = "dist"
name = "{stem}.png"
"#,
    );

    let out = run_build(root, &[]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let path = root.join("dist/photo.png");
    assert!(
        path.exists(),
        "the pinned literal extension must be honored"
    );
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(
        image::guess_format(&bytes).ok(),
        Some(ImageFormat::Png),
        "a literal-extension template must pin a REAL png, not AVIF-in-a-.png"
    );
}

/// AC-4 negative control: a recipe whose terminal step is a genuinely UNKNOWN
/// op (not `optimize`) still fails with `UnknownOperation` (exit 1). The
/// strip must key on the reserved name, not "drop whatever is last".
#[test]
fn build_still_rejects_an_unknown_terminal_op() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_png(root, "src/a.png", 32, 32);
    write_file(
        root,
        "bad.toml",
        b"version = \"1\"\n\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 16\n\n\
          [[step]]\nop = \"bogus\"\n",
    );
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/a.png"
recipe = "bad.toml"
out = "dist"
"#,
    );

    let out = run_build(root, &[]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "an unknown terminal op must still fail, stderr: {stderr}"
    );
    assert!(
        stderr.contains("bogus"),
        "error should name the unknown op: {stderr}"
    );
    assert!(!root.join("dist").exists(), "nothing may be written");
}

/// AC-5: an `optimize` step ANYWHERE BUT LAST still surfaces as
/// `UnknownOperation` — the existing documented behavior of
/// `split_terminal_optimize`, which must not regress.
#[test]
fn build_still_rejects_optimize_not_last() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_png(root, "src/a.png", 32, 32);
    write_file(
        root,
        "bad.toml",
        b"version = \"1\"\n\n[[step]]\nop = \"optimize\"\n\n\
          [[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 16\n",
    );
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/a.png"
recipe = "bad.toml"
out = "dist"
"#,
    );

    let out = run_build(root, &[]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "optimize anywhere but last must still fail, stderr: {stderr}"
    );
    assert!(
        stderr.contains("optimize"),
        "error should name the offending op: {stderr}"
    );
    assert!(!root.join("dist").exists(), "nothing may be written");
}

/// AC-6: a plain pixel recipe (no terminal `optimize`) through `build` is
/// UNCHANGED — byte-identical to `apply` on the same recipe + input (both
/// share `encode_one`'s `Preserve` path). The did-not-break-the-working-path
/// guard.
#[test]
fn build_plain_pixel_recipe_output_is_unchanged() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_png(root, "src/a.png", 32, 32);
    write_file(root, "r.toml", RESIZE_RECIPE.as_bytes());
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/a.png"
recipe = "r.toml"
out = "dist"
"#,
    );

    let out = run_build(root, &[]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let build_bytes = std::fs::read(root.join("dist/a.png")).unwrap();
    assert_eq!(
        image::guess_format(&build_bytes).ok(),
        Some(ImageFormat::Png),
        "source format (png) must be preserved, unchanged"
    );

    let apply_out = root.join("apply.png");
    let apply = Command::new(BIN)
        .args(["apply", "--recipe", "r.toml", "src/a.png", "-o"])
        .arg(&apply_out)
        .current_dir(root)
        .output()
        .expect("failed to run apply");
    assert!(
        apply.status.success(),
        "apply stderr: {}",
        String::from_utf8_lossy(&apply.stderr)
    );
    let apply_bytes = std::fs::read(&apply_out).unwrap();

    assert_eq!(
        build_bytes, apply_bytes,
        "a plain-pixel-recipe build must be byte-identical to apply on the same input"
    );
}

/// AC-7: the lockfile and cache name the file that was ACTUALLY written, with
/// the REAL decided extension — `lock_output_path` already takes `ext`, so
/// this confirms the decided format reaches it. A cache HIT must reproduce
/// the same output path as the miss that filled it.
#[test]
fn build_lock_entry_names_the_decided_extension() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "src/photo.png", &common::detailed_png(128, 128));
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/photo.png"
recipe = "web"
out = "dist"
"#,
    );

    // Miss: writes the output and the lockfile.
    let miss = run_build(root, &[]);
    assert!(
        miss.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&miss.stderr)
    );

    let written: Vec<String> = std::fs::read_dir(root.join("dist"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(written.len(), 1, "exactly one output written");
    let written_name = &written[0];

    // The lock/cache extension must match what the bytes REALLY are — not an
    // assumption about which feature-gated codec won (that varies across the
    // lean/default/webp-lossy legs; only the self-consistency is asserted).
    let written_bytes = std::fs::read(root.join("dist").join(written_name)).unwrap();
    let guessed =
        image::guess_format(&written_bytes).expect("output must be a real, decodable image");
    let guessed_ext = match guessed {
        ImageFormat::Avif => "avif",
        ImageFormat::WebP => "webp",
        ImageFormat::Png => "png",
        // The lean leg (no avif, no webp-lossy) has no built lossy codec of
        // its own, so the never-bigger fallback reaches for a baseline JPEG
        // (`fast_fallback_lossy_entry`, `src/cli/optimize.rs`).
        ImageFormat::Jpeg => "jpg",
        other => panic!("unexpected decided format for a photographic source: {other:?}"),
    };
    assert!(
        written_name.ends_with(&format!(".{guessed_ext}")),
        "the written file's extension must match its REAL bytes: {written_name} vs decoded {guessed_ext}"
    );

    let lock_text = std::fs::read_to_string(root.join("crustyimg.build.lock")).unwrap();
    assert!(
        lock_text.contains(written_name.as_str()),
        "lockfile must name the actually-written file {written_name}, got: {lock_text}"
    );
    assert!(
        !lock_text.contains("{ext}"),
        "lockfile must never contain the ext sentinel, got: {lock_text}"
    );

    // Hit: delete the written output so a re-run has to actually MATERIALIZE
    // from the cache rather than trivially agreeing with a file already
    // there, then re-run for real (not `--check`, which never writes —
    // `--check`/`--frozen`/`--locked` on a clean tree is `build_check_
    // frozen_locked_all_pass_with_a_decided_extension`'s job, not this one's).
    // A real hit must reproduce the SAME path with the SAME bytes, and the
    // summary line must actually SAY "cached", not just exit 0 — exit 0 alone
    // would also be true of a silent rebuild.
    std::fs::remove_file(root.join("dist").join(written_name)).unwrap();
    let hit = run_build(root, &[]);
    assert!(
        hit.status.success(),
        "a cache hit must reproduce the deleted output; stderr: {}",
        String::from_utf8_lossy(&hit.stderr)
    );
    let hit_stderr = String::from_utf8_lossy(&hit.stderr);
    assert!(
        hit_stderr.contains("(1 cached, 0 rebuilt)"),
        "re-running after deleting the output must be a cache HIT, not a rebuild: {hit_stderr}"
    );
    let written_after_hit: Vec<String> = std::fs::read_dir(root.join("dist"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        written_after_hit,
        vec![written_name.clone()],
        "the cache hit must materialize under the exact same name as the miss"
    );
    let bytes_after_hit = std::fs::read(root.join("dist").join(written_name)).unwrap();
    assert_eq!(
        bytes_after_hit, written_bytes,
        "the cache hit must materialize the exact same bytes as the miss"
    );
}

/// AC-10: `--frozen`/`--locked` (clap aliases of `--check`, one field —
/// `cli/mod.rs`) also still behave on a terminal-`optimize` target, whose
/// output extension now varies with content.
#[test]
fn build_check_frozen_locked_all_pass_with_a_decided_extension() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "src/photo.png", &common::detailed_png(128, 128));
    write_file(
        root,
        "crustyimg.build.toml",
        br#"
version = 1

[[target]]
source = "src/photo.png"
recipe = "web"
out = "dist"
"#,
    );
    let seed = run_build(root, &[]);
    assert!(
        seed.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&seed.stderr)
    );

    for flag in ["--check", "--frozen", "--locked"] {
        let out = run_build(root, &[flag]);
        assert!(
            out.status.success(),
            "{flag} must pass on a clean tree with a decided extension, stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
