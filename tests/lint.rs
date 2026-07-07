//! Integration tests for `crustyimg lint` (SPEC-050, DEC-050).
//!
//! Drives the real compiled binary via `env!("CARGO_BIN_EXE_crustyimg")`.
//! Fixtures are generated in-memory (see `tests/common`) — no committed binary
//! files, no ImageMagick. Exit codes: `0` clean · `7` ≥1 error · `3` no inputs.

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

mod common;
use common::{jpeg_with_gps, solid_png};

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// Write `bytes` to `dir/name`.
fn write(dir: &TempDir, name: &str, bytes: &[u8]) {
    std::fs::write(dir.path().join(name), bytes).unwrap();
}

/// Run `crustyimg lint <path>` and return (exit code, stdout).
fn lint(path: &Path) -> (i32, String) {
    let output = Command::new(BIN)
        .arg("lint")
        .arg(path)
        .output()
        .expect("failed to run crustyimg lint");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (code, stdout)
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
        stdout.contains("clean --gps"),
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
