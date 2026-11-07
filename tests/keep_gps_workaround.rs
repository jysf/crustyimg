//! SPEC-149 fix round: the `--keep-gps` refusal names a workaround. This proves the
//! workaround does what the message says, instead of only wording it.
//!
//! The command is parsed out of the refusal text the binary prints (never retyped), so
//! a reworded message that no longer names a runnable command fails here.

mod common;

use std::path::Path;
use std::process::{Command, Output};

use img_parts::jpeg::Jpeg;
use img_parts::{Bytes, ImageEXIF};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .output()
        .expect("spawn crustyimg")
}

/// The `crustyimg meta copy ... -y` command quoted in the refusal, as argv after the
/// program name, with `<in>`/`<out>` replaced by the given paths.
fn workaround_args(refusal: &str, from: &str, to: &str) -> Vec<String> {
    let start = refusal
        .find("`crustyimg ")
        .expect("refusal quotes a command")
        + 1;
    let end = start + refusal[start..].find('`').expect("closing backtick");
    refusal[start..end]
        .split_whitespace()
        .skip(1) // the program name
        .map(|t| match t {
            "<in>" => from.to_owned(),
            "<out>" => to.to_owned(),
            other => other.to_owned(),
        })
        .collect()
}

/// Whether the JPEG's EXIF has a GPS `GPSLatitudeRef` field equal to "N".
fn has_gps_lat_ref_n(bytes: &[u8]) -> bool {
    let Some(tiff) = Jpeg::from_bytes(Bytes::from(bytes.to_vec()))
        .ok()
        .and_then(|j| j.exif())
    else {
        return false;
    };
    let Ok(parsed) = exif::Reader::new().read_raw(tiff.to_vec()) else {
        return false;
    };
    let found = parsed
        .fields()
        .any(|f| f.tag == exif::Tag::GPSLatitudeRef && f.display_value().to_string().contains('N'));
    found
}

fn refusal_text(dir: &Path) -> String {
    let out = run(dir, &["resize", "a.jpg", "--max", "8", "--keep-gps"]);
    assert_eq!(out.status.code(), Some(2), "--keep-gps must be refused");
    String::from_utf8(out.stderr).unwrap()
}

#[test]
fn keep_gps_workaround_restores_gps_on_jpeg() {
    let dir = TempDir::new().unwrap();
    let d = dir.path();
    std::fs::write(d.join("a.jpg"), common::jpeg_with_gps(32, 32)).unwrap();
    let refusal = refusal_text(d);

    // Pixel lane drops the GPS (the premise of the message).
    let out = run(
        d,
        &["resize", "a.jpg", "--max", "16", "-o", "out.jpg", "-y"],
    );
    assert!(out.status.success(), "resize: {out:?}");
    assert!(!has_gps_lat_ref_n(
        &std::fs::read(d.join("out.jpg")).unwrap()
    ));

    // The quoted command, parsed from the message, puts it back.
    let args = workaround_args(&refusal, "a.jpg", "out.jpg");
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = run(d, &argv);
    assert_eq!(out.status.code(), Some(0), "workaround {argv:?}: {out:?}");
    assert!(has_gps_lat_ref_n(
        &std::fs::read(d.join("out.jpg")).unwrap()
    ));
}

/// If this starts failing, `meta copy` gained PNG: the "(JPEG only)" in the refusal
/// message, README and docs is now stale and must be reworded.
#[test]
fn keep_gps_workaround_on_png_is_exit_4() {
    let dir = TempDir::new().unwrap();
    let d = dir.path();
    std::fs::write(d.join("a.jpg"), common::jpeg_with_gps(32, 32)).unwrap();
    std::fs::write(d.join("in.png"), common::solid_png(16, 16, [9, 9, 9])).unwrap();
    std::fs::write(d.join("out.png"), common::solid_png(16, 16, [9, 9, 9])).unwrap();
    let refusal = refusal_text(d);
    let args = workaround_args(&refusal, "in.png", "out.png");
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = run(d, &argv);
    assert_eq!(out.status.code(), Some(4), "PNG meta copy: {out:?}");
}
