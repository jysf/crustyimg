//! Integration tests for `info --exif` on a JPEG that carries an EXIF APP1
//! segment (SPEC-009, AC4). Uses the shared `jpeg_with_exif` fixture from
//! `tests/common/mod.rs`.

mod common;
use common::jpeg_with_exif;

use std::process::Command;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// `info --exif <jpeg_with_exif>` exits 0 and reports EXIF present.
///
/// AC4: the fixture's IFD is zero-entry, so we do NOT assert any specific tag
/// exists. We pin "detect-and-report EXIF presence, succeed even with no
/// readable tags."
#[test]
fn info_exif_reports_present_on_jpeg_with_exif() {
    let dir = tempfile::tempdir().expect("tempdir");
    let jpeg_path = dir.path().join("exif.jpg");
    std::fs::write(&jpeg_path, jpeg_with_exif(8, 8)).expect("failed to write fixture JPEG");

    let output = Command::new(BIN)
        .args(["info", "--exif", jpeg_path.to_str().unwrap()])
        .output()
        .expect("failed to run info --exif on jpeg_with_exif");

    assert_eq!(
        output.status.code(),
        Some(0),
        "info --exif on jpeg_with_exif should exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );

    let stdout = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_ascii_lowercase();
    assert!(
        stdout.contains("exif"),
        "stdout should contain 'exif': {stdout}"
    );
    // has_exif is true → the exif: line should say "yes".
    assert!(
        stdout.contains("yes"),
        "stdout should contain 'yes' (EXIF present): {stdout}"
    );
}
