//! Integration tests for `crustyimg lint` (SPEC-050/051, DEC-050).
//!
//! Drives the real compiled binary via `env!("CARGO_BIN_EXE_crustyimg")`.
//! Fixtures are generated in-memory (see `tests/common`) — no committed binary
//! files, no ImageMagick. Exit codes: `0` clean · `7` ≥1 error (or warns over
//! `--max-warnings`) · `2` usage/bad config · `3` no inputs.

use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

mod common;
use common::{
    animated_apng, animated_gif, animated_webp, detailed_jpeg_at_quality, detailed_png,
    gradient_jpeg, jpeg_with_orientation, png_16bit, solid_png, static_gif, webp_lossless,
};

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// Write `bytes` to `dir/name`.
fn write(dir: &TempDir, name: &str, bytes: &[u8]) {
    std::fs::write(dir.path().join(name), bytes).unwrap();
}

/// Write `bytes` to `dir/name` and return the written path (SPEC-119: for
/// tests that need to `lint` a single file directly, bypassing the
/// directory walk's extension allow-list).
fn write_path(dir: &TempDir, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Run `crustyimg lint <path>` and return (exit code, stdout).
fn lint(path: &Path) -> (i32, String) {
    lint_args(&[path.as_os_str()])
}

/// Run `crustyimg lint <args…>` and return (exit code, stdout).
fn lint_args<S: AsRef<OsStr>>(args: &[S]) -> (i32, String) {
    let output = Command::new(BIN)
        .arg("lint")
        .args(args)
        .output()
        .expect("failed to run crustyimg lint");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (code, stdout)
}

/// Write a `.crustyimg-lint.toml` into `dir` (discovered when linting `dir`).
fn write_config(dir: &TempDir, body: &str) {
    std::fs::write(dir.path().join(".crustyimg-lint.toml"), body).unwrap();
}

#[test]
fn lint_on_a_clean_dir_exits_0_with_no_findings() {
    let dir = TempDir::new().unwrap();
    write(&dir, "a.png", &solid_png(4, 4, [10, 20, 30]));
    write(&dir, "b.png", &solid_png(8, 8, [200, 100, 50]));

    let (code, stdout) = lint(dir.path());
    assert_eq!(code, 0, "clean dir should exit 0; stdout:\n{stdout}");
    assert!(stdout.contains("0 error"), "summary should report 0 errors");
    assert!(
        stdout.contains("2 scanned"),
        "summary should count both files"
    );
}

#[test]
fn lint_on_a_gps_tagged_jpeg_exits_7_and_prints_the_finding_and_fix() {
    let dir = TempDir::new().unwrap();
    write(&dir, "leak.jpg", &jpeg_with_gps(16, 16));

    let (code, stdout) = lint(dir.path());
    assert_eq!(
        code, 7,
        "a GPS leak is an error → exit 7; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("privacy/gps-metadata-leak"),
        "must name the rule; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("meta clean --gps"),
        "must print the runnable fix; stdout:\n{stdout}"
    );
    assert!(stdout.contains("leak.jpg"), "must name the file");
}

#[test]
fn lint_on_a_truncated_file_exits_7_and_still_lints_a_sibling() {
    let dir = TempDir::new().unwrap();
    // A .png that is actually truncated garbage → decode fails → a finding.
    write(
        &dir,
        "broken.png",
        &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00],
    );
    // A sibling clean image the run must still process.
    write(&dir, "ok.png", &solid_png(4, 4, [1, 2, 3]));

    let (code, stdout) = lint(dir.path());
    assert_eq!(
        code, 7,
        "a corrupt file is an error → exit 7; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("size/truncated-or-corrupt"),
        "must name the corrupt rule; stdout:\n{stdout}"
    );
    assert!(stdout.contains("broken.png"), "must name the broken file");
    assert!(
        stdout.contains("2 scanned"),
        "the sibling must still be scanned; stdout:\n{stdout}"
    );
}

#[test]
fn non_image_files_in_the_tree_are_skipped() {
    let dir = TempDir::new().unwrap();
    write(&dir, "notes.txt", b"this is not an image");
    write(&dir, "README.md", b"# docs");
    write(&dir, "photo.png", &solid_png(4, 4, [9, 9, 9]));

    let (code, stdout) = lint(dir.path());
    assert_eq!(
        code, 0,
        "non-images must not become findings; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("1 scanned"),
        "only the one image is scanned; stdout:\n{stdout}"
    );
}

#[test]
fn lint_with_no_resolvable_inputs_exits_3() {
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join("does-not-exist");

    let output = Command::new(BIN)
        .arg("lint")
        .arg(&missing)
        .output()
        .expect("failed to run crustyimg lint");
    assert_eq!(
        output.status.code().unwrap_or(-1),
        3,
        "no inputs resolved → exit 3"
    );
}

// ── SPEC-051: config surface ────────────────────────────────────────────────

#[test]
fn config_turning_a_rule_off_makes_a_gps_tree_pass_and_no_config_ignores_it() {
    let dir = TempDir::new().unwrap();
    write(&dir, "leak.jpg", &jpeg_with_gps(16, 16));
    // A discovered config that disables the GPS rule.
    write_config(
        &dir,
        "[severity]\n\"privacy/gps-metadata-leak\" = \"off\"\n",
    );

    // With the config discovered, the GPS finding is suppressed → exit 0.
    let (code, stdout) = lint(dir.path());
    assert_eq!(code, 0, "off rule → clean; stdout:\n{stdout}");
    assert!(!stdout.contains("privacy/gps-metadata-leak"));

    // `--no-config` ignores the discovered config → the GPS leak fails again.
    let (code, stdout) = lint_args(&[dir.path().as_os_str(), OsStr::new("--no-config")]);
    assert_eq!(
        code, 7,
        "--no-config restores the default rule; stdout:\n{stdout}"
    );
    assert!(stdout.contains("privacy/gps-metadata-leak"));
}

#[test]
fn cli_ignore_flag_filters_a_rule() {
    let dir = TempDir::new().unwrap();
    write(&dir, "leak.jpg", &jpeg_with_gps(16, 16));

    let (code, _) = lint_args(&[
        dir.path().as_os_str(),
        OsStr::new("--ignore"),
        OsStr::new("privacy"),
    ]);
    assert_eq!(code, 0, "ignoring privacy silences the GPS error");
}

#[test]
fn per_rule_severity_downgrade_and_max_warnings_gate() {
    let dir = TempDir::new().unwrap();
    write(&dir, "leak.jpg", &jpeg_with_gps(16, 16));
    // Downgrade the GPS error to a warning.
    write_config(
        &dir,
        "[severity]\n\"privacy/gps-metadata-leak\" = \"warn\"\n",
    );

    // A lone warning does not fail without --max-warnings.
    let (code, stdout) = lint(dir.path());
    assert_eq!(code, 0, "warn alone does not fail; stdout:\n{stdout}");
    assert!(stdout.contains("warn privacy/gps-metadata-leak"));

    // --max-warnings 0 makes any warning fail (exit 7).
    let (code, _) = lint_args(&[
        dir.path().as_os_str(),
        OsStr::new("--max-warnings"),
        OsStr::new("0"),
    ]);
    assert_eq!(code, 7, "1 warn > --max-warnings 0 → exit 7");
}

#[test]
fn an_unknown_rule_id_in_select_is_a_usage_error() {
    let dir = TempDir::new().unwrap();
    write(&dir, "a.png", &solid_png(4, 4, [1, 2, 3]));

    let (code, _) = lint_args(&[
        dir.path().as_os_str(),
        OsStr::new("--select"),
        OsStr::new("bogus/nope"),
    ]);
    assert_eq!(code, 2, "an unknown rule id is a usage error (exit 2)");
}

#[test]
fn a_malformed_config_is_a_usage_error_not_a_panic() {
    let dir = TempDir::new().unwrap();
    write(&dir, "a.png", &solid_png(4, 4, [1, 2, 3]));
    write_config(&dir, "this is = = not toml\n");

    let (code, _) = lint(dir.path());
    assert_eq!(code, 2, "a malformed config is a usage error (exit 2)");
}

// ── SPEC-052: JSON report + human polish ────────────────────────────────────

#[test]
fn json_format_on_a_gps_tree_emits_the_finding_and_passed_false() {
    let dir = TempDir::new().unwrap();
    write(&dir, "leak.jpg", &jpeg_with_gps(16, 16));

    let (code, stdout) = lint_args(&[
        dir.path().as_os_str(),
        OsStr::new("--format"),
        OsStr::new("json"),
    ]);
    assert_eq!(code, 7, "GPS leak still fails the gate; stdout:\n{stdout}");
    assert!(stdout.contains("\"schema\":\"crustyimg.lint/v1\""));
    assert!(stdout.contains("\"rule\":\"privacy/gps-metadata-leak\""));
    assert!(
        stdout.contains("meta clean --gps"),
        "fix command present; stdout:\n{stdout}"
    );
    assert!(stdout.contains("\"passed\":false"));
    // The output format must not change the exit code.
    assert!(stdout.contains("\"errors\":1"));
}

#[test]
fn human_and_json_produce_the_same_exit_code() {
    let dir = TempDir::new().unwrap();
    write(&dir, "leak.jpg", &jpeg_with_gps(16, 16));

    let (human_code, _) = lint_args(&[
        dir.path().as_os_str(),
        OsStr::new("--format"),
        OsStr::new("human"),
    ]);
    let (json_code, _) = lint_args(&[
        dir.path().as_os_str(),
        OsStr::new("--format"),
        OsStr::new("json"),
    ]);
    assert_eq!(
        human_code, json_code,
        "format must not change the exit code"
    );
    assert_eq!(human_code, 7);
}

#[test]
fn an_invalid_format_value_is_a_usage_error() {
    let dir = TempDir::new().unwrap();
    write(&dir, "a.png", &solid_png(4, 4, [1, 2, 3]));

    let (code, _) = lint_args(&[
        dir.path().as_os_str(),
        OsStr::new("--format"),
        OsStr::new("xml"),
    ]);
    assert_eq!(
        code, 2,
        "an unknown --format value is a usage error (exit 2)"
    );
}

// ── SPEC-056: SARIF report ──────────────────────────────────────────────────

#[test]
fn sarif_format_on_a_gps_tree_emits_a_sarif_result() {
    let dir = TempDir::new().unwrap();
    write(&dir, "leak.jpg", &jpeg_with_gps(16, 16));

    let (code, stdout) = lint_args(&[
        dir.path().as_os_str(),
        OsStr::new("--format"),
        OsStr::new("sarif"),
    ]);
    assert_eq!(code, 7, "GPS leak still fails the gate; stdout:\n{stdout}");
    assert!(
        stdout.contains(r#""version":"2.1.0""#),
        "SARIF 2.1.0; {stdout}"
    );
    assert!(stdout.contains(r#""name":"crustyimg""#), "tool driver");
    assert!(
        stdout.contains(r#""ruleId":"privacy/gps-metadata-leak""#),
        "the GPS result; {stdout}"
    );
    assert!(stdout.contains(r#""level":"error""#), "error level");
    // A file location is present (relativization-to-cwd is unit-tested; here the
    // tempdir is outside the cwd so the uri stays absolute — still references it).
    assert!(stdout.contains("leak.jpg"), "a file location; {stdout}");
    assert!(
        stdout.contains(r#""artifactLocation""#),
        "SARIF physicalLocation; {stdout}"
    );
}

#[test]
fn sarif_and_human_produce_the_same_exit_code() {
    let dir = TempDir::new().unwrap();
    write(&dir, "leak.jpg", &jpeg_with_gps(16, 16));

    let (human, _) = lint_args(&[
        dir.path().as_os_str(),
        OsStr::new("--format"),
        OsStr::new("human"),
    ]);
    let (sarif, _) = lint_args(&[
        dir.path().as_os_str(),
        OsStr::new("--format"),
        OsStr::new("sarif"),
    ]);
    assert_eq!(human, sarif, "format must not change the exit code");
    assert_eq!(human, 7);
}

// ── SPEC-053: shipped-capability rules ──────────────────────────────────────

#[test]
fn a_mixed_tree_yields_grouped_findings_across_rules_with_the_right_exit_code() {
    let dir = TempDir::new().unwrap();
    write(&dir, "leak.jpg", &jpeg_with_gps(16, 16)); // error: GPS
    write(&dir, "rotated.jpg", &jpeg_with_orientation(16, 16, 6)); // warn: orientation
    write(&dir, "deep.png", &png_16bit(8, 8)); // warn: 16-bit colorspace
    write(&dir, "clean.png", &solid_png(4, 4, [1, 2, 3])); // clean

    let (code, stdout) = lint(dir.path());
    assert_eq!(code, 7, "the GPS error fails the gate; stdout:\n{stdout}");
    assert!(
        stdout.contains("privacy/gps-metadata-leak"),
        "gps; {stdout}"
    );
    assert!(
        stdout.contains("orient/orientation-not-baked"),
        "orient; {stdout}"
    );
    assert!(
        stdout.contains("color/wrong-colorspace"),
        "colorspace; {stdout}"
    );
    assert!(stdout.contains("4 scanned"), "all four scanned; {stdout}");
}

#[test]
fn a_per_glob_byte_budget_from_config_drives_a_size_finding() {
    // Inherited from SPEC-051: the budget plumbing, now consumed by
    // `size/oversized-bytes`.
    let dir = TempDir::new().unwrap();
    write(&dir, "big.png", &solid_png(8, 8, [10, 20, 30]));
    // A 10-byte budget over every file → any real image is oversized.
    write_config(&dir, "[[budget]]\nglob = \"**\"\nmax_bytes = 10\n");

    let (code, stdout) = lint(dir.path());
    assert_eq!(
        code, 7,
        "over-budget is an error → exit 7; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("size/oversized-bytes"),
        "size rule; {stdout}"
    );
    assert!(stdout.contains("optimize"), "runnable fix; {stdout}");

    // With no budget configured, the rule does not fire.
    let dir2 = TempDir::new().unwrap();
    write(&dir2, "big.png", &solid_png(8, 8, [10, 20, 30]));
    let (code2, _) = lint(dir2.path());
    assert_eq!(code2, 0, "no budget ⇒ no size finding");
}

#[test]
fn per_rule_severity_flips_animated_gif_warn_to_error_and_changes_exit() {
    let dir = TempDir::new().unwrap();
    write(&dir, "loop.gif", &animated_gif(8, 8));

    // Default: animated-gif is a warning → does not fail.
    let (code, stdout) = lint(dir.path());
    assert_eq!(
        code, 0,
        "animated-gif warns, doesn't fail; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("format/animated-gif"),
        "gif rule fires; {stdout}"
    );

    // Config promotes it to error → exit 7.
    write_config(&dir, "[severity]\n\"format/animated-gif\" = \"error\"\n");
    let (code, _) = lint(dir.path());
    assert_eq!(code, 7, "promoted to error → exit 7");
}

#[test]
fn opt_in_rules_are_off_by_default_and_enabled_by_config() {
    let dir = TempDir::new().unwrap();
    // A plain image has no ICC and no camera metadata.
    write(&dir, "plain.png", &solid_png(4, 4, [5, 6, 7]));

    // By default the opt-in `color/missing-icc` does not fire.
    let (code, stdout) = lint(dir.path());
    assert_eq!(code, 0);
    assert!(
        !stdout.contains("color/missing-icc"),
        "opt-in off by default"
    );

    // Selecting it turns it on (info never changes the exit code).
    let (code, stdout) = lint_args(&[
        dir.path().as_os_str(),
        OsStr::new("--select"),
        OsStr::new("color/missing-icc"),
    ]);
    assert_eq!(code, 0, "info-severity never fails");
    assert!(
        stdout.contains("color/missing-icc"),
        "select enables it; {stdout}"
    );
}

// ── SPEC-071 fix 1: a valid image is never called "truncated or corrupt" ─────

/// `lint` on a **valid RAW** file must not call it corrupt.
///
/// `LintTarget` used to decode by BYTES (`Image::from_bytes`) even though it had
/// the path, so RAW — which is byte-ambiguous with TIFF and therefore routed by
/// EXTENSION (SPEC-061/DEC-055) — never reached its decoder. Every valid `.nef`
/// linted as "truncated or corrupt (failed to decode); re-export a valid image",
/// failing CI with exit 7 on a directory of RAW files, while `info` on the very
/// same file read it fine. Routing `lint` through `Image::decode_path` fixes it.
#[test]
fn lint_does_not_call_a_valid_raw_file_corrupt() {
    let raw = Path::new("tests/fixtures/raw/synthetic_preview.nef");
    assert!(raw.exists(), "the SPEC-061 RAW fixture must exist");

    let (code, stdout) = lint(raw);
    assert!(
        !stdout.contains("truncated-or-corrupt"),
        "a valid RAW file must not be diagnosed as corrupt, got:\n{stdout}"
    );
    assert_eq!(code, 0, "a clean RAW file must lint clean, got:\n{stdout}");
}

/// `lint` on a valid-but-**over-cap** image (>64 Mpix, SPEC-070/DEC-063) must not
/// call it corrupt either: it is a valid image we decline to decode, and
/// "re-export a valid image" is a remedy for a problem it does not have. The
/// fixture is the SPEC-070 pixel bomb — a `.nef` declaring 160 Mpix.
#[test]
fn lint_does_not_call_an_over_cap_image_corrupt() {
    let bomb = Path::new("tests/fixtures/fuzz/raw_preview/pixel_bomb.nef");
    assert!(bomb.exists(), "the SPEC-070 pixel-bomb fixture must exist");

    let (_code, stdout) = lint(bomb);
    assert!(
        !stdout.contains("truncated-or-corrupt"),
        "an image outside the decode budget must not be diagnosed as corrupt, got:\n{stdout}"
    );
}

/// The catch-all is intact: a genuinely corrupt file still fires the rule (exit 7).
#[test]
fn lint_still_reports_a_genuinely_corrupt_file() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "broken.png",
        &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x01],
    );

    let (code, stdout) = lint(&dir.path().join("broken.png"));
    assert!(
        stdout.contains("truncated-or-corrupt"),
        "a truly corrupt file must still be reported, got:\n{stdout}"
    );
    assert_eq!(code, 7, "an error finding exits 7");
}

// ── SPEC-119: animated-input lint coverage (AC-7 / AC-7b) ───────────────────

/// AC-7: the `fix:` string no longer names a command that discards frames —
/// pinned by asserting the ABSENCE of the destructive advice, not just new
/// wording (a test that only checks for new text would still pass if the
/// old advice were printed alongside it).
#[test]
fn animated_gif_rule_does_not_recommend_a_flattening_command() {
    let dir = TempDir::new().unwrap();
    write(&dir, "anim.gif", &animated_gif(4, 4));

    let (code, stdout) = lint(dir.path());
    assert_eq!(
        code, 0,
        "animated-gif warns, doesn't fail by default; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("format/animated-gif"),
        "the rule must still fire; stdout:\n{stdout}"
    );
    assert!(
        !stdout.contains("convert --format webp"),
        "the fix must no longer recommend the destructive command; stdout:\n{stdout}"
    );
}

/// APNG and animated WebP get the sibling `format/animated-input` finding,
/// and it carries the same no-destructive-fix property.
///
/// **Linted as two SINGLE-FILE arguments, not one shared directory.** A
/// single named file bypasses the directory walk's extension allow-list, so
/// this proves the RULE fires on a named WebP file. Directory mode is covered
/// separately by `max_warnings_zero_fails_on_animated_webp_in_directory_mode`
/// (SPEC-130 added `webp` to that allow-list; before it, a directory walk
/// skipped `.webp` files entirely).
#[test]
fn animated_input_rule_fires_on_apng_and_webp_with_no_destructive_fix() {
    let dir = TempDir::new().unwrap();
    let png = write_path(&dir, "anim.png", &animated_apng(4, 4));
    let webp = write_path(&dir, "anim.webp", &animated_webp(4, 4));

    let (png_code, png_stdout) = lint(&png);
    assert_eq!(png_code, 0, "warns, doesn't fail; stdout:\n{png_stdout}");
    assert!(
        png_stdout.contains("format/animated-input"),
        "the APNG must fire the sibling rule; stdout:\n{png_stdout}"
    );

    let (webp_code, webp_stdout) = lint(&webp);
    assert_eq!(webp_code, 0, "warns, doesn't fail; stdout:\n{webp_stdout}");
    assert!(
        webp_stdout.contains("format/animated-input"),
        "the animated WebP must fire the sibling rule; stdout:\n{webp_stdout}"
    );

    assert!(
        !png_stdout.contains("convert --format webp") && !webp_stdout.contains("convert --format webp"),
        "neither finding may recommend the destructive command; stdout:\n{png_stdout}\n{webp_stdout}"
    );
}

/// AC-7b: `lint --max-warnings 0` — the strict path Call 1's confirmation
/// leans on — fails independently on each of the three animated families,
/// with a static counterpart of each staying clean under the same flag.
/// This is the answer the spec gives to "warn and proceed still loses
/// data"; an answer nobody drove is not an answer.
///
/// GIF/PNG are linted by DIRECTORY (matching the rest of this file's style);
/// WebP is linted by its own single-file PATH here, and by directory in
/// `max_warnings_zero_fails_on_animated_webp_in_directory_mode`.
#[test]
fn max_warnings_zero_fails_on_each_animated_family_clean_on_static() {
    fn strict(dir: &TempDir) -> i32 {
        lint_args(&[
            dir.path().as_os_str(),
            OsStr::new("--max-warnings"),
            OsStr::new("0"),
        ])
        .0
    }
    fn strict_path(path: &std::path::Path) -> i32 {
        lint_args(&[
            path.as_os_str(),
            OsStr::new("--max-warnings"),
            OsStr::new("0"),
        ])
        .0
    }

    let gif_dir = TempDir::new().unwrap();
    write(&gif_dir, "anim.gif", &animated_gif(4, 4));
    assert_eq!(
        strict(&gif_dir),
        7,
        "an animated GIF must fail --max-warnings 0"
    );

    let gif_static = TempDir::new().unwrap();
    write(&gif_static, "still.gif", &static_gif(4, 4));
    assert_eq!(
        strict(&gif_static),
        0,
        "a static GIF must stay clean under --max-warnings 0"
    );

    let png_dir = TempDir::new().unwrap();
    write(&png_dir, "anim.png", &animated_apng(4, 4));
    assert_eq!(strict(&png_dir), 7, "an APNG must fail --max-warnings 0");

    let png_static = TempDir::new().unwrap();
    write(&png_static, "still.png", &solid_png(4, 4, [1, 2, 3]));
    assert_eq!(
        strict(&png_static),
        0,
        "a static PNG must stay clean under --max-warnings 0"
    );

    let webp_dir = TempDir::new().unwrap();
    let webp_anim = write_path(&webp_dir, "anim.webp", &animated_webp(4, 4));
    assert_eq!(
        strict_path(&webp_anim),
        7,
        "an animated WebP must fail --max-warnings 0"
    );

    let webp_static_dir = TempDir::new().unwrap();
    let webp_static = write_path(&webp_static_dir, "still.webp", &webp_lossless(4, 4));
    assert_eq!(
        strict_path(&webp_static),
        0,
        "a static WebP must stay clean under --max-warnings 0"
    );
}

/// SPEC-130: the strict gate DEC-093 rests on holds in DIRECTORY mode for
/// animated WebP. Before `webp` joined the source allow-list, a directory
/// holding an animated `.webp` resolved to zero inputs and the file was never
/// linted; with a sibling image present the run exited 0 — a false green.
///
/// The animated directory carries a static PNG beside the WebP so the
/// pre-fix behaviour is that false green (exit 0), not an "input not found"
/// exit 3 that would also fail this assertion for a different reason.
#[test]
fn max_warnings_zero_fails_on_animated_webp_in_directory_mode() {
    let anim_dir = TempDir::new().unwrap();
    write(&anim_dir, "anim.webp", &animated_webp(4, 4));
    write(&anim_dir, "still.png", &solid_png(4, 4, [1, 2, 3]));
    let (code, stdout) = lint_args(&[
        anim_dir.path().as_os_str(),
        OsStr::new("--max-warnings"),
        OsStr::new("0"),
    ]);
    assert!(
        stdout.contains("format/animated-input") && stdout.contains("anim.webp"),
        "the directory walk must lint anim.webp and fire the rule; stdout:\n{stdout}"
    );
    assert_eq!(
        code, 7,
        "an animated WebP in a directory must fail --max-warnings 0; stdout:\n{stdout}"
    );

    let static_dir = TempDir::new().unwrap();
    write(&static_dir, "still.webp", &webp_lossless(4, 4));
    let (code, stdout) = lint_args(&[
        static_dir.path().as_os_str(),
        OsStr::new("--max-warnings"),
        OsStr::new("0"),
    ]);
    assert_eq!(
        code, 0,
        "a directory of one static WebP must stay clean; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("1 scanned"),
        "the static WebP must actually be scanned, not skipped; stdout:\n{stdout}"
    );
}

// ── SPEC-148: printed fixes run verbatim ─────────────────────────────────────
//
// Fixtures here are hand-built bytes (an EXIF TIFF written below) or made by
// ImageMagick, never by crustyimg's own metadata writer, so a fixture cannot
// pass because the code under test wrote it. The shared `common::jpeg_with_gps`
// holds only a `GPSLatitudeRef`, which is no longer a location, so this file
// builds its own coordinate-bearing one.

/// One hand-built EXIF entry: (tag, TIFF type, count, value bytes).
type Entry = (u16, u16, u32, Vec<u8>);

const T_BYTE: u16 = 1;
const T_ASCII: u16 = 2;
const T_SHORT: u16 = 3;
const T_RATIONAL: u16 = 5;
const T_UNDEFINED: u16 = 7;

/// Encode one IFD. Values over 4 bytes go to `data`, whose first byte sits at
/// TIFF offset `data_base`; offsets are computed, not hard-coded.
fn encode_ifd(entries: &[Entry], data: &mut Vec<u8>, data_base: usize) -> Vec<u8> {
    let mut sorted: Vec<&Entry> = entries.iter().collect();
    sorted.sort_by_key(|e| e.0);
    let mut ifd = Vec::new();
    ifd.extend_from_slice(&(sorted.len() as u16).to_le_bytes());
    for (tag, ty, count, value) in sorted {
        ifd.extend_from_slice(&tag.to_le_bytes());
        ifd.extend_from_slice(&ty.to_le_bytes());
        ifd.extend_from_slice(&count.to_le_bytes());
        if value.len() <= 4 {
            let mut inline = value.clone();
            inline.resize(4, 0);
            ifd.extend_from_slice(&inline);
        } else {
            ifd.extend_from_slice(&((data_base + data.len()) as u32).to_le_bytes());
            data.extend_from_slice(value);
            if value.len() % 2 == 1 {
                data.push(0);
            }
        }
    }
    ifd.extend_from_slice(&[0, 0, 0, 0]); // next IFD = 0
    ifd
}

/// Splice an EXIF APP1 (little-endian TIFF: IFD0 `ifd0`, plus a GPS IFD holding
/// exactly `gps` when given) into `base`, right after SOI.
fn exif_jpeg(base: &[u8], ifd0: &[Entry], gps: Option<&[Entry]>) -> Vec<u8> {
    assert_eq!(&base[0..2], &[0xFF, 0xD8], "base must be a JPEG");
    let n0 = ifd0.len() + usize::from(gps.is_some());
    let ifd0_len = 2 + 12 * n0 + 4;
    let gps_off = 8 + ifd0_len;
    let gps_len = gps.map_or(0, |g| 2 + 12 * g.len() + 4);
    let data_base = gps_off + gps_len;

    let mut entries0: Vec<Entry> = ifd0.to_vec();
    if gps.is_some() {
        // 0x8825 GPSInfo pointer: LONG, count 1, value = GPS IFD offset.
        entries0.push((0x8825, 4, 1, (gps_off as u32).to_le_bytes().to_vec()));
    }
    let mut data = Vec::new();
    let ifd0_bytes = encode_ifd(&entries0, &mut data, data_base);
    let gps_bytes = gps.map(|g| encode_ifd(g, &mut data, data_base));

    let mut payload = b"Exif\0\0".to_vec();
    payload.extend_from_slice(&[0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00]);
    payload.extend_from_slice(&ifd0_bytes);
    if let Some(g) = gps_bytes {
        payload.extend_from_slice(&g);
    }
    payload.extend_from_slice(&data);

    let mut out = Vec::with_capacity(base.len() + payload.len() + 4);
    out.extend_from_slice(&base[0..2]);
    out.extend_from_slice(&[0xFF, 0xE1]);
    out.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
    out.extend_from_slice(&payload);
    out.extend_from_slice(&base[2..]);
    out
}

/// A JPEG whose GPS IFD holds exactly `entries`.
fn jpeg_with_gps_tags(w: u32, h: u32, entries: &[Entry]) -> Vec<u8> {
    exif_jpeg(&gradient_jpeg(w, h), &[], Some(entries))
}

fn rationals(values: &[(u32, u32)]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|(n, d)| n.to_le_bytes().into_iter().chain(d.to_le_bytes()))
        .collect()
}

fn g_lat_ref() -> Entry {
    (0x0001, T_ASCII, 2, b"N\0".to_vec())
}
fn g_lat() -> Entry {
    (
        0x0002,
        T_RATIONAL,
        3,
        rationals(&[(50, 1), (29, 1), (1113, 25)]),
    )
}
fn g_lon_ref() -> Entry {
    (0x0003, T_ASCII, 2, b"E\0".to_vec())
}
fn g_lon() -> Entry {
    (
        0x0004,
        T_RATIONAL,
        3,
        rationals(&[(30, 1), (31, 1), (0, 1)]),
    )
}
fn g_area() -> Entry {
    (0x001C, T_UNDEFINED, 12, b"ASCII\0\0\0Kyiv".to_vec())
}

/// A JPEG with a full position (lat + lon and their refs): the "leak" fixture.
fn jpeg_with_gps(w: u32, h: u32) -> Vec<u8> {
    jpeg_with_gps_tags(w, h, &[g_lat_ref(), g_lat(), g_lon_ref(), g_lon()])
}

/// A JPEG with an IFD0 `Make` ("ACME"), the camera-metadata trigger.
fn jpeg_with_make_acme() -> Vec<u8> {
    let base = gradient_jpeg(64, 64);
    exif_jpeg(&base, &[(0x010F, T_ASCII, 5, b"ACME\0".to_vec())], None)
}

/// A JPEG carrying an APP2 `ICC_PROFILE` of `len` filler bytes.
fn jpeg_with_icc_len(base: &[u8], len: usize) -> Vec<u8> {
    let mut payload = b"ICC_PROFILE\0".to_vec();
    payload.extend_from_slice(&[1, 1]);
    payload.extend(std::iter::repeat_n(0xABu8, len));
    let mut out = Vec::new();
    out.extend_from_slice(&base[0..2]);
    out.extend_from_slice(&[0xFF, 0xE2]);
    out.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
    out.extend_from_slice(&payload);
    out.extend_from_slice(&base[2..]);
    out
}

/// A CMYK JPEG made by ImageMagick (`tests/fixtures/lint/README.md`).
fn cmyk_jpeg() -> Vec<u8> {
    include_bytes!("fixtures/lint/cmyk_16x16.jpg").to_vec()
}

/// Run `crustyimg lint <args…>` with `dir` as the working directory (so the
/// printed path is the name as typed), returning (exit code, stdout, stderr).
fn lint_in(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let output = Command::new(BIN)
        .arg("lint")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run crustyimg lint");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// `lint --format json --select <rule> <args…>` in `dir`, parsed.
fn lint_json_in(dir: &Path, rule: &str, args: &[&str]) -> serde_json::Value {
    let mut full = vec!["--format", "json", "--select", rule];
    full.extend_from_slice(args);
    let (_, stdout, stderr) = lint_in(dir, &full);
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!("lint JSON did not parse ({e}); stdout:\n{stdout}\nstderr:\n{stderr}")
    })
}

/// The first finding for `rule` in a parsed lint JSON report.
fn finding_for<'a>(report: &'a serde_json::Value, rule: &str) -> Option<&'a serde_json::Value> {
    report["findings"]
        .as_array()
        .expect("findings is an array")
        .iter()
        .find(|f| f["rule"] == rule)
}

// ── The shell harness ────────────────────────────────────────────────────────

/// The shell a user on this platform is shown a fix for.
fn fix_shell_program() -> String {
    // Test-only override (mutation M-13): lets a run prove that a missing shell
    // FAILS the test instead of skipping it. Never set in CI.
    if let Ok(name) = std::env::var("CRUSTYIMG_TEST_FIX_SHELL") {
        return name;
    }
    if cfg!(windows) { "powershell" } else { "sh" }.to_string()
}

/// `PATH` with the test binary's directory first, so the shell resolves
/// `crustyimg` to the binary under test.
fn path_with_test_bin() -> std::ffi::OsString {
    let bin_dir = Path::new(BIN)
        .parent()
        .expect("bin has a parent")
        .to_path_buf();
    let mut dirs = vec![bin_dir];
    dirs.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    std::env::join_paths(dirs).expect("join PATH")
}

/// Run `script` through the host shell in `dir`, exactly as a user pastes it.
fn run_in_shell(dir: &Path, script: &str) -> std::process::Output {
    let program = fix_shell_program();
    let mut cmd = Command::new(&program);
    if cfg!(windows) {
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("{script}; exit $LASTEXITCODE"),
        ]);
    } else {
        cmd.args(["-c", script]);
    }
    cmd.current_dir(dir)
        .env("PATH", path_with_test_bin())
        .output()
        .unwrap_or_else(|e| panic!("cannot run the printed fix: no {program} on PATH: {e}"))
}

/// Ask the same shell where `crustyimg` resolves and assert it is the binary
/// under test (a Homebrew `crustyimg` earlier on PATH would report the same version).
fn assert_shell_resolves_the_test_binary(dir: &Path) {
    let script = if cfg!(windows) {
        "(Get-Command crustyimg).Source"
    } else {
        "command -v crustyimg"
    };
    let out = run_in_shell(dir, script);
    let found = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(
        !found.is_empty(),
        "the shell cannot find `crustyimg`; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let found =
        std::fs::canonicalize(&found).unwrap_or_else(|e| panic!("canonicalize {found}: {e}"));
    let want = std::fs::canonicalize(BIN).expect("canonicalize BIN");
    assert_eq!(
        found, want,
        "the shell resolved crustyimg to a different binary"
    );
}

/// One FT-1 row.
struct Row {
    name: &'static str,
    rule: &'static str,
    /// The file as typed, relative to the temp dir (may hold a sub dir).
    file: &'static str,
    bytes: Vec<u8>,
    /// `.crustyimg-lint.toml` body, if the row needs one.
    config: Option<String>,
    /// What to lint: `None` = the file; `Some("sub")` = a relative directory.
    lint_target: Option<&'static str>,
    /// Lint invoked as `lint … -- <file>` (a leading-dash name).
    dash_dash: bool,
    /// Also write a plain-`optimize`d copy and require it is STILL reported.
    budget_control: Option<Vec<u8>>,
    /// The metadata lane cannot rewrite this container (it handles JPEG and
    /// PNG only): the finding must still fire, but print NO command.
    no_fix: bool,
}

fn row(name: &'static str, rule: &'static str, file: &'static str, bytes: Vec<u8>) -> Row {
    Row {
        name,
        rule,
        file,
        bytes,
        config: None,
        lint_target: None,
        dash_dash: false,
        budget_control: None,
        no_fix: false,
    }
}

const GPS_RULE: &str = "privacy/gps-metadata-leak";

fn fix_rows() -> Vec<Row> {
    // oversized-bytes (JPEG): the budget must matter. Measure what plain
    // `optimize` leaves (argv, not the shell), then set the budget below it.
    let jpeg_fixture = detailed_jpeg_at_quality(900, 700, 97);
    let scratch = TempDir::new().unwrap();
    std::fs::write(scratch.path().join("p.jpg"), &jpeg_fixture).unwrap();
    let status = Command::new(BIN)
        .args(["optimize", "p.jpg", "-o", "p.jpg", "-y"])
        .current_dir(scratch.path())
        .output()
        .expect("run optimize");
    assert!(status.status.success(), "plain optimize failed");
    let plain = std::fs::read(scratch.path().join("p.jpg")).unwrap();
    let s1 = plain.len();
    let budget = s1 * 3 / 4;
    assert!(
        budget < jpeg_fixture.len(),
        "the budget {budget} must sit under the fixture ({} bytes)",
        jpeg_fixture.len()
    );

    let png_fixture = detailed_png(500, 400);
    let png_budget = png_fixture.len() / 2;

    let budget_cfg = |b: usize| format!("[[budget]]\nglob = \"**\"\nmax_bytes = {b}\n");

    let mut rows = vec![
        row("gps-coords", GPS_RULE, "leak.jpg", jpeg_with_gps(64, 64)),
        row(
            "gps-area-info",
            GPS_RULE,
            "area.jpg",
            jpeg_with_gps_tags(64, 64, &[g_area()]),
        ),
        row(
            "camera-metadata",
            "privacy/camera-metadata",
            "cam.jpg",
            jpeg_with_make_acme(),
        ),
        row(
            "unexpected-icc",
            "color/unexpected-icc",
            "icc.jpg",
            jpeg_with_icc_len(&gradient_jpeg(64, 64), 5000),
        ),
        row(
            "oversized-bytes-jpeg",
            "size/oversized-bytes",
            "big.jpg",
            jpeg_fixture,
        ),
        row(
            "oversized-bytes-png",
            "size/oversized-bytes",
            "big.png",
            png_fixture,
        ),
        row(
            "oversized-dimensions",
            "dims/oversized-dimensions",
            "wide.png",
            detailed_png(64, 48),
        ),
        row(
            "orientation",
            "orient/orientation-not-baked",
            "rot.jpg",
            jpeg_with_orientation(64, 48, 6),
        ),
        row("cmyk", "color/wrong-colorspace", "cmyk.jpg", cmyk_jpeg()),
        // Containers the metadata lane cannot write (JPEG/PNG only). Built by
        // ImageMagick + exiftool, not by the code under test.
        row(
            "gps-webp",
            GPS_RULE,
            "leak.webp",
            include_bytes!("fixtures/lint/gps_16x16.webp").to_vec(),
        ),
        row(
            "camera-webp",
            "privacy/camera-metadata",
            "cam.webp",
            include_bytes!("fixtures/lint/cam_16x16.webp").to_vec(),
        ),
        row(
            "gps-tiff",
            GPS_RULE,
            "leak.tif",
            include_bytes!("fixtures/lint/gps_16x16.tif").to_vec(),
        ),
        row("space", GPS_RULE, "my photo.jpg", jpeg_with_gps(64, 64)),
        row("quote", GPS_RULE, "it's.jpg", jpeg_with_gps(64, 64)),
        row("dash", GPS_RULE, "-x.jpg", jpeg_with_gps(64, 64)),
        row("subdir", GPS_RULE, "sub/leak.jpg", jpeg_with_gps(64, 64)),
    ];
    for r in &mut rows {
        match r.name {
            "oversized-bytes-jpeg" => {
                r.config = Some(budget_cfg(budget));
                r.budget_control = Some(plain.clone());
            }
            "oversized-bytes-png" => r.config = Some(budget_cfg(png_budget)),
            "oversized-dimensions" => {
                r.config = Some("[[budget]]\nglob = \"**\"\nmax_intended_width = 16\n".into());
            }
            "gps-webp" | "camera-webp" | "gps-tiff" => r.no_fix = true,
            "dash" => r.dash_dash = true,
            "subdir" => r.lint_target = Some("sub"),
            _ => {}
        }
    }
    rows
}

/// FT-1: for every rule that prints a fix, take the `fix` string AS PRINTED, run
/// it through the host shell, and check it rewrote the file and cleared the
/// finding. Failures are recorded per kind (trap 15), then reported together.
#[test]
fn every_printed_fix_run_verbatim_fixes_its_finding() {
    let probe = TempDir::new().unwrap();
    assert_shell_resolves_the_test_binary(probe.path());

    let mut counts: std::collections::BTreeMap<&'static str, usize> = Default::default();
    let mut failing_rows: Vec<String> = Vec::new();

    for r in fix_rows() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(r.file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &r.bytes).unwrap();
        if let Some(cfg) = &r.config {
            write_config(&dir, cfg);
        }
        if let Some(plain) = &r.budget_control {
            std::fs::write(dir.path().join("plain.jpg"), plain).unwrap();
        }

        let mut kinds: Vec<&'static str> = Vec::new();
        let mut stderr_note = String::new();
        let mut fix_note = String::new();

        let mut args: Vec<&str> = vec!["--format", "json", "--select", r.rule];
        let target = r.lint_target.unwrap_or(r.file);
        if r.dash_dash {
            args.push("--");
        }
        args.push(target);
        let (_, stdout, stderr) = lint_in(dir.path(), &args);
        let before: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "row {}: lint JSON ({e}); stdout:\n{stdout}\nstderr:\n{stderr}",
                r.name
            )
        });
        let finding = finding_for(&before, r.rule);
        if finding.is_none() {
            kinds.push("not_reported_before");
        }
        let fix = finding.and_then(|f| f["fix"].as_str()).map(str::to_string);
        if r.no_fix {
            // The finding fires; the fix is absent (JSON `null`), never a
            // command the metadata lane would refuse.
            match finding {
                None => kinds.push("not_reported_before"),
                Some(f) if !f["fix"].is_null() => {
                    kinds.push("printed_unrunnable_fix");
                    fix_note = f["fix"].to_string();
                    let out = run_in_shell(dir.path(), fix.as_deref().unwrap_or(""));
                    stderr_note = String::from_utf8_lossy(&out.stderr)
                        .chars()
                        .take(300)
                        .collect();
                    if !out.status.success() {
                        kinds.push("shell_failed");
                    }
                }
                Some(_) => {}
            }
            if !kinds.is_empty() {
                for k in &kinds {
                    *counts.entry(k).or_default() += 1;
                }
                failing_rows.push(format!(
                    "  row {}: {:?}\n    fix: {:?}\n    stderr: {}",
                    r.name,
                    kinds,
                    fix_note,
                    stderr_note.trim()
                ));
            }
            continue;
        }
        match &fix {
            Some(f) if f.starts_with("crustyimg ") && f.ends_with(" -y") => {}
            _ => kinds.push("fix_shape"),
        }

        if let Some(fix) = &fix {
            fix_note = fix.clone();
            let out = run_in_shell(dir.path(), fix);
            stderr_note = String::from_utf8_lossy(&out.stderr)
                .chars()
                .take(300)
                .collect();
            if !out.status.success() {
                kinds.push("shell_failed");
            }
            if !out.stdout.is_empty() {
                kinds.push("stdout_nonempty");
            }
            let after = std::fs::read(&path).unwrap();
            if after == r.bytes {
                kinds.push("unchanged");
            }
            let relint = lint_json_in(dir.path(), r.rule, &args[4..]);
            if finding_for(&relint, r.rule).is_some() {
                kinds.push("still_reported");
            }
            let same_format =
                ::image::guess_format(&after).ok() == ::image::guess_format(&r.bytes).ok();
            if !same_format || ::image::load_from_memory(&after).is_err() {
                kinds.push("format_changed");
            }
            if r.budget_control.is_some() {
                let ctl = lint_json_in(dir.path(), r.rule, &["plain.jpg"]);
                if finding_for(&ctl, r.rule).is_none() {
                    kinds.push("budget_control");
                }
            }
        } else {
            kinds.push("shell_failed");
        }

        if !kinds.is_empty() {
            for k in &kinds {
                *counts.entry(k).or_default() += 1;
            }
            failing_rows.push(format!(
                "  row {}: {:?}\n    fix: {:?}\n    stderr: {}",
                r.name,
                kinds,
                fix_note,
                stderr_note.trim()
            ));
        }
    }

    if !counts.is_empty() {
        let summary: Vec<String> = counts.iter().map(|(k, v)| format!("{k}={v}")).collect();
        panic!(
            "printed fixes do not fix their findings: {}\n{}",
            summary.join(" "),
            failing_rows.join("\n")
        );
    }
}

/// FT-2: the human `fix:` line, the JSON `fix` and the SARIF `— fix:` suffix are
/// one byte-identical string.
#[test]
fn human_json_and_sarif_carry_the_same_fix_string() {
    let dir = TempDir::new().unwrap();
    write(&dir, "leak.jpg", &jpeg_with_gps(64, 64));
    let want = "crustyimg meta clean --gps leak.jpg -o leak.jpg -y";

    let (_, human, _) = lint_in(dir.path(), &["--select", GPS_RULE, "leak.jpg"]);
    let human_fix = human
        .lines()
        .find_map(|l| l.trim_start().strip_prefix("fix: "))
        .unwrap_or_else(|| panic!("no `fix:` line in human output:\n{human}"));

    let json = lint_json_in(dir.path(), GPS_RULE, &["leak.jpg"]);
    let json_fix = finding_for(&json, GPS_RULE).unwrap()["fix"]
        .as_str()
        .expect("fix is a string")
        .to_string();

    let (_, sarif_out, _) = lint_in(
        dir.path(),
        &["--format", "sarif", "--select", GPS_RULE, "leak.jpg"],
    );
    let sarif: serde_json::Value = serde_json::from_str(&sarif_out).expect("sarif parses");
    let text = sarif["runs"][0]["results"][0]["message"]["text"]
        .as_str()
        .expect("message text");
    let sarif_fix = text
        .split_once(" — fix: ")
        .unwrap_or_else(|| panic!("no `— fix: ` in SARIF text: {text}"))
        .1;

    assert_eq!(human_fix, want, "human fix line");
    assert_eq!(json_fix, want, "JSON fix");
    assert_eq!(sarif_fix, want, "SARIF fix");
}

/// FT-8: a 16-bit PNG keeps its finding, with words and no command, in every
/// output format.
#[test]
fn a_16_bit_png_prints_words_and_no_command_in_every_format() {
    let dir = TempDir::new().unwrap();
    write(&dir, "deep.png", &png_16bit(8, 8));
    let rule = "color/wrong-colorspace";

    let (_, human, _) = lint_in(dir.path(), &["--select", rule, "deep.png"]);
    assert!(
        human.contains(rule),
        "finding present in human output:\n{human}"
    );
    assert!(
        !human.lines().any(|l| l.contains("fix:")),
        "human output must have no fix line:\n{human}"
    );

    let json = lint_json_in(dir.path(), rule, &["deep.png"]);
    let f = finding_for(&json, rule).expect("finding present in JSON");
    assert!(
        f["fix"].is_null(),
        "JSON fix must be null, got {}",
        f["fix"]
    );

    let (_, sarif_out, _) = lint_in(
        dir.path(),
        &["--format", "sarif", "--select", rule, "deep.png"],
    );
    let sarif: serde_json::Value = serde_json::from_str(&sarif_out).expect("sarif parses");
    let text = sarif["runs"][0]["results"][0]["message"]["text"]
        .as_str()
        .expect("SARIF result present");
    assert!(
        text.contains("cannot reduce bit depth"),
        "SARIF text: {text}"
    );
    assert!(!text.contains("fix:"), "SARIF text: {text}");
}

/// Pipe `bytes` into `crustyimg lint - <args…>`.
fn lint_stdin(bytes: &[u8], args: &[&str]) -> (i32, String) {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(BIN)
        .arg("lint")
        .arg("-")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn lint -");
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// FT-9: stdin findings print no command (the name `stdin` is not a file).
#[test]
fn stdin_input_prints_no_command() {
    let (_, stdout) = lint_stdin(
        &jpeg_with_gps(64, 64),
        &["--format", "json", "--select", GPS_RULE],
    );
    let json: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("JSON ({e}); stdout:\n{stdout}"));
    let f = finding_for(&json, GPS_RULE).expect("finding present for stdin");
    assert!(
        f["fix"].is_null(),
        "stdin fix must be null, got {}",
        f["fix"]
    );

    let (_, human) = lint_stdin(&jpeg_with_gps(64, 64), &["--select", GPS_RULE]);
    assert!(human.contains(GPS_RULE), "finding present:\n{human}");
    assert!(
        !human.lines().any(|l| l.contains("fix:")),
        "no fix line for stdin:\n{human}"
    );
}

/// FT-11: the GPS rule fires on a location and not on an empty block.
#[test]
fn gps_rule_fires_on_a_location_and_not_on_an_empty_block() {
    let dest_lat = (0x0014, T_RATIONAL, 3, rationals(&[(50, 1), (0, 1), (0, 1)]));
    let dest_lon = (0x0016, T_RATIONAL, 3, rationals(&[(30, 1), (0, 1), (0, 1)]));
    let alt = (0x0006, T_RATIONAL, 1, rationals(&[(120, 1)]));
    let alt_ref = (0x0005, T_BYTE, 1, vec![0]);
    let time = (0x0007, T_RATIONAL, 3, rationals(&[(12, 1), (0, 1), (0, 1)]));
    let date = (0x001D, T_ASCII, 11, b"2026:11:05\0".to_vec());
    let datum = (0x0012, T_ASCII, 6, b"WGS-84".to_vec());
    let version = (0x0000, T_BYTE, 4, vec![2, 3, 0, 0]);
    let _ = T_SHORT;

    let rows: Vec<(&str, Vec<u8>, bool)> = vec![
        ("coordinates", jpeg_with_gps(32, 32), true),
        (
            "latitude only",
            jpeg_with_gps_tags(32, 32, &[g_lat()]),
            true,
        ),
        (
            "longitude only",
            jpeg_with_gps_tags(32, 32, &[g_lon()]),
            true,
        ),
        (
            "dest latitude",
            jpeg_with_gps_tags(32, 32, &[dest_lat]),
            true,
        ),
        (
            "dest longitude",
            jpeg_with_gps_tags(32, 32, &[dest_lon]),
            true,
        ),
        (
            "area information",
            jpeg_with_gps_tags(32, 32, &[g_area()]),
            true,
        ),
        (
            "signed_gps.jpg (real coordinates)",
            include_bytes!("fixtures/c2pa/signed_gps.jpg").to_vec(),
            true,
        ),
        (
            "version only (P1100 shape)",
            jpeg_with_gps_tags(32, 32, &[version]),
            false,
        ),
        (
            "latitude ref only",
            jpeg_with_gps_tags(32, 32, &[g_lat_ref()]),
            false,
        ),
        (
            "altitude + time + date + datum",
            jpeg_with_gps_tags(32, 32, &[alt_ref, alt, time, date, datum]),
            false,
        ),
    ];
    for (name, bytes, fires) in rows {
        let dir = TempDir::new().unwrap();
        let p = write_path(&dir, "p.jpg", &bytes);
        let (code, stdout) =
            lint_args(&[p.as_os_str(), OsStr::new("--select"), OsStr::new(GPS_RULE)]);
        let want = if fires { 7 } else { 0 };
        assert_eq!(
            code,
            want,
            "{name}: expected exit {want} ({}); stdout:\n{stdout}",
            if fires { "a location" } else { "no location" }
        );
    }
}

/// FT-12: the Actions doc's fix table says what `lint` prints.
#[test]
fn the_github_actions_doc_fix_table_matches_what_lint_prints() {
    let full = include_str!("../docs/github-actions.md").replace("\r\n", "\n");
    // Only §5 is the fix table; later sections show unrelated commands.
    let start = full
        .find("## 5. Fix what it finds")
        .expect("doc has a section 5");
    let rest = &full[start + 5..];
    let doc = &rest[..rest.find("\n## ").unwrap_or(rest.len())];
    for want in [
        "crustyimg optimize --max-size <budget> <file> -o <file> -y",
        "crustyimg convert --format jpeg <file> -o <file> -y",
    ] {
        assert!(
            doc.contains(want),
            "docs/github-actions.md must show `{want}`"
        );
    }
    assert!(
        !doc.contains("--profile preserve --max 1600"),
        "the old optimize row is not what lint prints"
    );
}
