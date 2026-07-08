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
