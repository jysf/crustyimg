//! Integration tests for RAW as a DEFAULT, Tier-1 input (SPEC-061).
//!
//! The default build extracts the embedded full-res JPEG preview from common
//! RAW files (`.nef`/`.cr2`/`.cr3`/`.arw`/`.dng`/…) via a format-agnostic byte
//! scan + the capped `image` JPEG decoder — no RAW codec, no new dependency — so
//! `optimize`/`convert`/`info`/batch see them like any other image. The preview
//! *is* a JPEG, so a RAW's `source_format` is reported as `Jpeg`.
//!
//! Fixture: `tests/fixtures/raw/synthetic_preview.nef` — a hand-built blob (a
//! TIFF header + a 16×12 embedded JPEG thumbnail + a 64×48 embedded JPEG
//! preview), generated natively (no camera/ImageMagick, AGENTS §12).
//! Regen: `cargo run --example gen_raw_fixture`.

use std::process::Command;

use crustyimg::source::{resolve, Input};

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");
const RAW_FIXTURE: &[u8] = include_bytes!("fixtures/raw/synthetic_preview.nef");

/// `optimize <fixture>.nef -o out.webp` exits 0 and writes a valid WebP with the
/// preview's (64×48) dimensions — proving RAW input flows through the pipeline on
/// the default build.
#[test]
fn optimize_raw_input_writes_webp() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.nef");
    std::fs::write(&in_path, RAW_FIXTURE).unwrap();
    let out_path = dir.path().join("out.webp");

    let output = Command::new(BIN)
        .args([
            "optimize",
            in_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run optimize");
    assert_eq!(
        output.status.code(),
        Some(0),
        "optimize should exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let bytes = std::fs::read(&out_path).expect("read webp output");
    assert_eq!(
        image::guess_format(&bytes).unwrap(),
        image::ImageFormat::WebP,
        "output should be WebP"
    );
    let decoded = image::load_from_memory(&bytes).expect("output should decode as WebP");
    assert_eq!(decoded.width(), 64);
    assert_eq!(decoded.height(), 48);
}

/// `convert <fixture>.nef --format png -o out.png` exits 0 and writes a valid PNG
/// with the preview's dimensions — the extracted preview re-encodes to PNG.
#[test]
fn convert_raw_to_png() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.nef");
    std::fs::write(&in_path, RAW_FIXTURE).unwrap();
    let out_path = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "png",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert");
    assert_eq!(
        output.status.code(),
        Some(0),
        "convert should exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let bytes = std::fs::read(&out_path).expect("read png output");
    assert_eq!(
        image::guess_format(&bytes).unwrap(),
        image::ImageFormat::Png,
        "output should be PNG"
    );
    let decoded = image::load_from_memory(&bytes).expect("output should decode as PNG");
    assert_eq!(decoded.width(), 64);
    assert_eq!(decoded.height(), 48);
}

/// `info <fixture>.nef` (and `--json`) exits 0 and reports the preview as a
/// JPEG at the preview's (64×48) dimensions — proving `info` routes a RAW path
/// through the same extension-aware decode as the pipeline (SPEC-061, DEC-055),
/// not the generic byte decoder that would mis-read the RAW container.
#[test]
fn info_raw_reports_jpeg_dims() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.nef");
    std::fs::write(&in_path, RAW_FIXTURE).unwrap();

    // Human output.
    let output = Command::new(BIN)
        .args(["info", in_path.to_str().unwrap()])
        .output()
        .expect("failed to run info");
    assert_eq!(
        output.status.code(),
        Some(0),
        "info should exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("dimensions: 64x48"),
        "expected preview dims 64x48; got:\n{stdout}"
    );
    assert!(
        stdout.contains("format:     jpeg"),
        "expected format jpeg; got:\n{stdout}"
    );

    // JSON output.
    let output = Command::new(BIN)
        .args(["info", in_path.to_str().unwrap(), "--json"])
        .output()
        .expect("failed to run info --json");
    assert_eq!(
        output.status.code(),
        Some(0),
        "info --json should exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = String::from_utf8_lossy(&output.stdout);
    assert!(json.contains("\"format\":\"jpeg\""), "got:\n{json}");
    assert!(json.contains("\"width\":64"), "got:\n{json}");
    assert!(json.contains("\"height\":48"), "got:\n{json}");
}

/// A `.nef` with no decodable embedded preview fails `info` with the typed,
/// `raw:`-prefixed error (from the preview path), NOT the generic "failed to
/// fill whole buffer" that the byte decoder would emit on RAW container bytes.
#[test]
fn info_raw_without_preview_reports_typed_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("empty.nef");
    // TIFF header + noise: a RAW extension with no embedded JPEG stream.
    std::fs::write(&in_path, b"II*\0\x08\0\0\0no jpeg preview here at all").unwrap();

    let output = Command::new(BIN)
        .args(["info", in_path.to_str().unwrap()])
        .output()
        .expect("failed to run info");
    assert_ne!(output.status.code(), Some(0), "info should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("raw:"),
        "expected typed raw: error; got:\n{stderr}"
    );
    assert!(
        !stderr.contains("failed to fill whole buffer"),
        "should not surface the generic byte-decoder error; got:\n{stderr}"
    );
}

/// A directory source containing a `.nef` (plus a non-image `.txt`) yields
/// exactly the `.nef` — RAW extensions are in the source allow-list.
#[test]
fn directory_source_discovers_raw() {
    let dir = tempfile::tempdir().expect("tempdir");
    let raw = dir.path().join("a.nef");
    std::fs::write(&raw, RAW_FIXTURE).unwrap();
    std::fs::write(dir.path().join("notes.txt"), b"not an image").unwrap();

    let inputs = resolve(dir.path().to_str().unwrap(), &mut std::io::empty()).unwrap();
    assert_eq!(inputs.len(), 1, "expected exactly the .nef, got {inputs:?}");
    match &inputs[0] {
        Input::Path(p) => assert_eq!(p.extension().and_then(|e| e.to_str()), Some("nef")),
        other => panic!("expected Path, got {other:?}"),
    }
}
