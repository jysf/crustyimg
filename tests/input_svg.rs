//! Integration tests for SVG as a DEFAULT, pure-Rust input (SPEC-060).
//!
//! The default build rasterizes `.svg` end to end (via `resvg`/`usvg`/
//! `tiny-skia`, no system deps), so `optimize`/`convert`/batch see it like any
//! other image. There is no `ImageFormat::Svg`, so a rasterized SVG's
//! `source_format` is reported as `Png` (a lossless-RGBA target).
//!
//! Fixture: `tests/fixtures/svg/rect_text_40x30.svg` — a 40×30 plain-text SVG
//! with a rect, a semi-transparent circle, and a `<text>` element. Committed
//! verbatim as text (no encoder feature or ImageMagick needed).

use std::process::Command;

use crustyimg::source::{resolve, Input};

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");
const SVG_FIXTURE: &[u8] = include_bytes!("fixtures/svg/rect_text_40x30.svg");

/// `optimize <fixture>.svg -o out.png` exits 0 and writes a valid PNG with the
/// fixture's intrinsic dimensions — proving SVG input flows through the pipeline
/// on the default build.
#[test]
fn optimize_svg_input_writes_png() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.svg");
    std::fs::write(&in_path, SVG_FIXTURE).unwrap();
    let out_path = dir.path().join("out.png");

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

    let bytes = std::fs::read(&out_path).expect("read png output");
    assert_eq!(
        image::guess_format(&bytes).unwrap(),
        image::ImageFormat::Png,
        "output should be PNG"
    );
    let decoded = image::load_from_memory(&bytes).expect("output should decode as PNG");
    assert_eq!(decoded.width(), 40);
    assert_eq!(decoded.height(), 30);
}

/// `convert <fixture>.svg -o out.webp` exits 0 and writes a valid WebP with the
/// fixture's dimensions — SVG rasterizes then re-encodes to another format.
#[test]
fn convert_svg_to_webp() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.svg");
    std::fs::write(&in_path, SVG_FIXTURE).unwrap();
    let out_path = dir.path().join("out.webp");

    let output = Command::new(BIN)
        .args([
            "convert",
            in_path.to_str().unwrap(),
            "--format",
            "webp",
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

    let bytes = std::fs::read(&out_path).expect("read webp output");
    assert_eq!(
        image::guess_format(&bytes).unwrap(),
        image::ImageFormat::WebP,
        "output should be WebP"
    );
    let decoded = image::load_from_memory(&bytes).expect("output should decode as WebP");
    assert_eq!(decoded.width(), 40);
    assert_eq!(decoded.height(), 30);
}

/// A directory source containing an `.svg` (plus a non-image `.txt`) yields
/// exactly the `.svg` — `.svg` is in the source allow-list.
#[test]
fn directory_source_discovers_svg() {
    let dir = tempfile::tempdir().expect("tempdir");
    let svg = dir.path().join("a.svg");
    std::fs::write(&svg, SVG_FIXTURE).unwrap();
    std::fs::write(dir.path().join("notes.txt"), b"not an image").unwrap();

    let inputs = resolve(dir.path().to_str().unwrap(), &mut std::io::empty()).unwrap();
    assert_eq!(inputs.len(), 1, "expected exactly the .svg, got {inputs:?}");
    match &inputs[0] {
        Input::Path(p) => assert_eq!(p.extension().and_then(|e| e.to_str()), Some("svg")),
        other => panic!("expected Path, got {other:?}"),
    }
}

/// The text fixture rasterizes to a non-empty raster of the expected dims using
/// the bundled Go font (no system fonts) — `<text>` is not silently dropped.
#[test]
fn svg_text_uses_bundled_font() {
    use crustyimg::image::Image;

    let img = Image::from_bytes(SVG_FIXTURE).expect("rasterize text svg");
    assert_eq!(img.width(), 40);
    assert_eq!(img.height(), 30);

    // The raster must carry actual pixels (a non-empty buffer), and the text
    // glyphs must have painted white pixels the background rect does not
    // contain. The rect is #3366cc and the text is #ffffff, so at least one
    // near-white pixel proves the bundled font rendered the `<text>`.
    let rgba = img.pixels().to_rgba8();
    assert!(!rgba.as_raw().is_empty(), "raster buffer is empty");
    let has_white_text = rgba
        .pixels()
        .any(|p| p.0[0] > 230 && p.0[1] > 230 && p.0[2] > 230 && p.0[3] > 200);
    assert!(
        has_white_text,
        "expected white text glyphs from the bundled Go font"
    );
}
