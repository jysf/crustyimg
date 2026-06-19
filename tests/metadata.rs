//! Integration tests for the container-lane metadata commands (SPEC-026):
//! `strip` and `clean --gps`. These drive the REAL compiled binary via
//! `std::process::Command` and assert exit codes + output bytes end-to-end.
//!
//! Fixtures are generated NATIVELY (no ImageMagick): pixels via the `image`
//! crate, EXIF (Orientation + Copyright + GPS refs) seeded with `little_exif`.
//! `.unwrap()` here is idiomatic test setup (the `no-unwrap` constraint is
//! scoped to `src/**`).

use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;

use image::{DynamicImage, ImageFormat, RgbImage};
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::ifd::ExifTagGroup;
use little_exif::metadata::Metadata;
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Fixture helpers (native; no ImageMagick) ──────────────────────────────────

/// A small deterministic 16×16 RGB image encoded to `format`, with no metadata.
fn base_bytes(format: ImageFormat) -> Vec<u8> {
    let mut img = RgbImage::new(16, 16);
    for (x, y, px) in img.enumerate_pixels_mut() {
        *px = image::Rgb([(x * 16) as u8, (y * 16) as u8, ((x + y) * 8) as u8]);
    }
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, format)
        .unwrap();
    buf.into_inner()
}

/// JPEG bytes seeded with Orientation + Copyright + GPS{Latitude,Longitude}Ref.
fn jpeg_with_exif() -> Vec<u8> {
    let mut bytes = base_bytes(ImageFormat::Jpeg);
    let mut md = Metadata::new();
    md.set_tag(ExifTag::Orientation(vec![1]));
    md.set_tag(ExifTag::Copyright("crustyimg test".to_string()));
    md.set_tag(ExifTag::GPSLatitudeRef("N".to_string()));
    md.set_tag(ExifTag::GPSLongitudeRef("E".to_string()));
    md.write_to_vec(&mut bytes, FileExtension::JPEG).unwrap();
    bytes
}

/// Write `bytes` to `dir/name` and return the path.
fn write_fixture(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Whether a JPEG byte stream still parses any EXIF via `little_exif`.
fn jpeg_has_exif(bytes: &[u8]) -> bool {
    match Metadata::new_from_vec(&bytes.to_vec(), FileExtension::JPEG) {
        Err(_) => false,
        Ok(mut md) => {
            !md.get_ifd_mut(ExifTagGroup::GENERIC, 0)
                .get_tags()
                .is_empty()
                || !md.get_ifd_mut(ExifTagGroup::GPS, 0).get_tags().is_empty()
        }
    }
}

/// Whether a JPEG byte stream carries a generic-IFD tag with the given id.
fn jpeg_has_generic_tag(bytes: &[u8], tag_id: u16) -> bool {
    match Metadata::new_from_vec(&bytes.to_vec(), FileExtension::JPEG) {
        Err(_) => false,
        Ok(mut md) => md
            .get_ifd_mut(ExifTagGroup::GENERIC, 0)
            .get_tags()
            .iter()
            .any(|t| t.as_u16() == tag_id),
    }
}

// IFD0 tag ids: Artist 0x013B, Copyright 0x8298, Orientation 0x0112.
const TAG_ARTIST: u16 = 0x013B;
const TAG_COPYRIGHT: u16 = 0x8298;
const TAG_ORIENTATION: u16 = 0x0112;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn strip_jpeg_to_stdout_has_no_exif() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.jpg", &jpeg_with_exif());

    let output = Command::new(BIN)
        .args(["strip", input.to_str().unwrap(), "-o", "-"])
        .output()
        .unwrap();

    assert!(output.status.success(), "strip should exit 0");
    assert!(
        !jpeg_has_exif(&output.stdout),
        "stripped JPEG on stdout should have no EXIF"
    );
}

#[test]
fn clean_gps_jpeg_removes_location_keeps_orientation() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.jpg", &jpeg_with_exif());
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "clean",
            input.to_str().unwrap(),
            "--gps",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "clean --gps should exit 0");

    let cleaned = std::fs::read(&out).unwrap();
    let mut md = Metadata::new_from_vec(&cleaned, FileExtension::JPEG).unwrap();
    // GPS gone.
    assert!(
        md.get_ifd_mut(ExifTagGroup::GPS, 0).get_tags().is_empty(),
        "GPS tags should be removed"
    );
    // Orientation (0x0112) survives.
    let generic_ids: Vec<u16> = md
        .get_ifd_mut(ExifTagGroup::GENERIC, 0)
        .get_tags()
        .iter()
        .map(|t| t.as_u16())
        .collect();
    assert!(
        generic_ids.contains(&0x0112),
        "Orientation should be preserved"
    );
}

#[test]
fn clean_without_gps_flag_exits_2() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.jpg", &jpeg_with_exif());

    let output = Command::new(BIN)
        .args(["clean", input.to_str().unwrap(), "-o", "-"])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(2),
        "clean without --gps should exit 2"
    );
}

#[test]
fn strip_unsupported_format_exits_4() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.bmp", &base_bytes(ImageFormat::Bmp));

    let output = Command::new(BIN)
        .args(["strip", input.to_str().unwrap(), "-o", "-"])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(4),
        "strip on a BMP should exit 4 (unsupported format)"
    );
}

#[test]
fn strip_multi_input_requires_out_dir() {
    let dir = TempDir::new().unwrap();
    let a = write_fixture(&dir, "a.jpg", &jpeg_with_exif());
    let b = write_fixture(&dir, "b.jpg", &jpeg_with_exif());

    let output = Command::new(BIN)
        .args(["strip", a.to_str().unwrap(), b.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(2),
        "multiple inputs without --out-dir should exit 2"
    );
}

#[test]
fn strip_multi_input_fanout_writes_all() {
    let dir = TempDir::new().unwrap();
    let a = write_fixture(&dir, "a.jpg", &jpeg_with_exif());
    let b = write_fixture(&dir, "b.jpg", &jpeg_with_exif());
    let out_dir = TempDir::new().unwrap();

    let output = Command::new(BIN)
        .args([
            "strip",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "fan-out strip should exit 0");

    for name in ["a.jpg", "b.jpg"] {
        let path = out_dir.path().join(name);
        assert!(path.exists(), "{name} should be written");
        let bytes = std::fs::read(&path).unwrap();
        assert!(!jpeg_has_exif(&bytes), "{name} should be stripped");
    }
}

#[test]
fn strip_batch_partial_failure_exits_6() {
    let dir = TempDir::new().unwrap();
    let good = write_fixture(&dir, "good.jpg", &jpeg_with_exif());
    let bad = write_fixture(&dir, "bad.bmp", &base_bytes(ImageFormat::Bmp));
    let out_dir = TempDir::new().unwrap();

    let output = Command::new(BIN)
        .args([
            "strip",
            good.to_str().unwrap(),
            bad.to_str().unwrap(),
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(6),
        "a batch with one unsupported input should exit 6"
    );
    // The good input is still written.
    assert!(
        out_dir.path().join("good.jpg").exists(),
        "the good input should still be written"
    );
}

#[test]
fn strip_refuses_overwrite_without_yes() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.jpg", &jpeg_with_exif());
    let out = dir.path().join("out.jpg");
    // Pre-create the output so the overwrite guard trips.
    std::fs::write(&out, b"existing").unwrap();

    let refused = Command::new(BIN)
        .args([
            "strip",
            input.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        refused.status.code(),
        Some(5),
        "overwrite without --yes should exit 5"
    );

    let allowed = Command::new(BIN)
        .args([
            "strip",
            input.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--yes",
        ])
        .output()
        .unwrap();
    assert!(
        allowed.status.success(),
        "overwrite with --yes should exit 0"
    );
}

// ── set (SPEC-027) ────────────────────────────────────────────────────────────

#[test]
fn set_writes_tags_to_output() {
    let dir = TempDir::new().unwrap();
    // Plain JPEG with no EXIF: set creates the tags.
    let input = write_fixture(&dir, "in.jpg", &base_bytes(ImageFormat::Jpeg));
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "set",
            input.to_str().unwrap(),
            "--artist",
            "Jane",
            "--copyright",
            "2026",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "set should exit 0");
    let bytes = std::fs::read(&out).unwrap();
    assert!(
        jpeg_has_generic_tag(&bytes, TAG_ARTIST),
        "Artist should be written"
    );
    assert!(
        jpeg_has_generic_tag(&bytes, TAG_COPYRIGHT),
        "Copyright should be written"
    );
}

#[test]
fn set_without_any_flag_exits_2() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.jpg", &jpeg_with_exif());

    let output = Command::new(BIN)
        .args(["set", input.to_str().unwrap(), "-o", "-"])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(2),
        "set with no tag flags should exit 2"
    );
}

#[test]
fn set_preserves_other_metadata() {
    let dir = TempDir::new().unwrap();
    // jpeg_with_exif carries Orientation (0x0112).
    let input = write_fixture(&dir, "in.jpg", &jpeg_with_exif());
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "set",
            input.to_str().unwrap(),
            "--copyright",
            "X",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "set should exit 0");
    let bytes = std::fs::read(&out).unwrap();
    assert!(
        jpeg_has_generic_tag(&bytes, TAG_ORIENTATION),
        "Orientation should be preserved"
    );
}

#[test]
fn set_unsupported_format_exits_4() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.bmp", &base_bytes(ImageFormat::Bmp));

    let output = Command::new(BIN)
        .args(["set", input.to_str().unwrap(), "--artist", "A", "-o", "-"])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(4),
        "set on a BMP should exit 4 (unsupported format)"
    );
}

#[test]
fn set_multi_input_fanout_writes_all() {
    let dir = TempDir::new().unwrap();
    let a = write_fixture(&dir, "a.jpg", &base_bytes(ImageFormat::Jpeg));
    let b = write_fixture(&dir, "b.jpg", &base_bytes(ImageFormat::Jpeg));
    let out_dir = TempDir::new().unwrap();

    let output = Command::new(BIN)
        .args([
            "set",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--artist",
            "A",
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "fan-out set should exit 0");

    for name in ["a.jpg", "b.jpg"] {
        let path = out_dir.path().join(name);
        assert!(path.exists(), "{name} should be written");
        let bytes = std::fs::read(&path).unwrap();
        assert!(
            jpeg_has_generic_tag(&bytes, TAG_ARTIST),
            "{name} should be tagged with Artist"
        );
    }
}
