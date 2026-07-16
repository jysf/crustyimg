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
use common::{animated_gif, jpeg_with_gps, jpeg_with_orientation, png_16bit, solid_png};

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// Write `bytes` to `dir/name`.
fn write(dir: &TempDir, name: &str, bytes: &[u8]) {
    std::fs::write(dir.path().join(name), bytes).unwrap();
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
