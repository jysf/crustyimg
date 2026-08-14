//! Integration tests for HEIC/HEIF as an OPT-IN input (SPEC-062, DEC-052).
//!
//! HEIC decode is gated behind the off-by-default `heic` cargo feature (system
//! libheif, decode-only) and never ships in a distributed artifact: HEVC is
//! patent-encumbered on every decode path, and the mature pure-Rust decoders are
//! AGPL. So these tests come in two halves:
//!
//! - **Default build** (`cargo test`): a `.heic` is DETECTED by container brand
//!   and rejected with exit 4 and a message naming `--features heic`.
//! - **`cargo test --features heic`** (needs a system libheif ≥ 1.17): the same
//!   `.heic` flows through the pipeline like any other input.
//!
//! Extension discovery works in both builds, so a directory of photos does not
//! silently hide the `.heic` files it contains.
//!
//! Fixture: `tests/fixtures/heic/solid_64x48.heic` — a 64×48 solid image.
//! Regen (macOS, OS encoder): `sips -s format heic solid.png --out
//! tests/fixtures/heic/solid_64x48.heic`. HEIC *encode* needs x265/GPL, so this
//! is a committed static asset rather than a natively-generated fixture.

use std::process::Command;

use crustyimg::source::{resolve, Input};

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");
const HEIC_FIXTURE: &[u8] = include_bytes!("fixtures/heic/solid_64x48.heic");
#[cfg(feature = "heic")]
const NOISE_HEIC_FIXTURE: &[u8] = include_bytes!("fixtures/heic/noise_preview_96x72.heic");

/// The DEFAULT binary refuses a `.heic` with **exit 4** and tells the user exactly
/// how to get HEIC support — no panic, no partial output file, and not the vague
/// "unsupported or undetectable image format" it would emit without brand detection.
#[cfg(not(feature = "heic"))]
#[test]
fn optimize_heic_exits_4_codec_not_built() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.heic");
    std::fs::write(&in_path, HEIC_FIXTURE).unwrap();
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
        Some(4),
        "expected exit 4 (codec not built); stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--features heic"),
        "stderr should name the feature to rebuild with; got:\n{stderr}"
    );
    assert!(
        !stderr.contains("unsupported or undetectable"),
        "should not fall through to the generic unsupported-format error; got:\n{stderr}"
    );
    assert!(!out_path.exists(), "no output file should be written");
}

/// `info` takes the path-based decode seam, so it must surface the same exit 4 —
/// the byte-path/extension-path split that broke `info <raw>` in SPEC-061.
#[cfg(not(feature = "heic"))]
#[test]
fn info_heic_exits_4_codec_not_built() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.heic");
    std::fs::write(&in_path, HEIC_FIXTURE).unwrap();

    let output = Command::new(BIN)
        .args(["info", in_path.to_str().unwrap()])
        .output()
        .expect("failed to run info");

    assert_eq!(
        output.status.code(),
        Some(4),
        "expected exit 4; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--features heic"), "got:\n{stderr}");
}

/// Under `--features heic`, `optimize <fixture>.heic -o out.webp` exits 0 and
/// writes a valid 64×48 WebP — HEIC input flows through the whole pipeline.
#[cfg(feature = "heic")]
#[test]
fn optimize_heic_input_writes_webp() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.heic");
    std::fs::write(&in_path, HEIC_FIXTURE).unwrap();
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

/// SPEC-115 (AC-4), `--features heic` only: `optimize <fixture>.heic --out-dir
/// out/` (the auto-decide path, no `-o`) writes a REAL raster, never the raw
/// `.heic` ISOBMFF/HEIF container under an adopted `png` label.
///
/// Fixture: `tests/fixtures/heic/noise_preview_96x72.heic` — the same
/// Floyd–Steinberg-dithered 96×72 pattern as the RAW noise fixture
/// (`tests/fixtures/classify/RECIPES.md`'s recipe, dithered PNG written by
/// `cargo run --example gen_heic_dither_png`), encoded to HEIC by the
/// INDEPENDENT `heif-enc` tool (system libheif, `-q 10`), never by crustyimg.
/// The 96×72 size classifies `Icon` (`OptBucket::LosslessFlat`), so both
/// lossless candidates (webp 4252 B / png 12000 B) exceed the 463 B container
/// and `pick_winner` returns `None`. **Must fail today**: this test proves RED
/// on `main` before the fix.
#[cfg(feature = "heic")]
#[test]
fn optimize_heic_auto_decide_writes_a_real_raster() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("noise_preview.heic");
    std::fs::write(&in_path, NOISE_HEIC_FIXTURE).unwrap();

    let output = Command::new(BIN)
        .args([
            "optimize",
            in_path.to_str().unwrap(),
            "--out-dir",
            dir.path().to_str().unwrap(),
            "--explain",
        ])
        .output()
        .expect("failed to run optimize");
    assert_eq!(
        output.status.code(),
        Some(0),
        "optimize should exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("png \u{2192} png"),
        "must not claim a PNG re-encode of PNG source bytes that were never \
         produced; got:\n{stderr}"
    );

    let written: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p != &in_path)
        .collect();
    assert_eq!(
        written.len(),
        1,
        "expected exactly one output file, got {written:?}"
    );
    let bytes = std::fs::read(&written[0]).expect("read output");
    let sniffed = image::guess_format(&bytes);
    assert!(
        sniffed.is_ok(),
        "the written file must sniff as a real raster format, not the raw .heic \
         ISOBMFF container; path={:?} sniff={sniffed:?}",
        written[0]
    );
    assert_ne!(
        bytes, NOISE_HEIC_FIXTURE,
        "the .heic container bytes must never be written verbatim under a raster name"
    );
}

/// Under `--features heic`, `convert <fixture>.heic --format png` exits 0 with the
/// fixture's dimensions.
#[cfg(feature = "heic")]
#[test]
fn convert_heic_to_png() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.heic");
    std::fs::write(&in_path, HEIC_FIXTURE).unwrap();
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
    let decoded = image::load_from_memory(&bytes).expect("output should decode as PNG");
    assert_eq!(decoded.width(), 64);
    assert_eq!(decoded.height(), 48);
}

/// Under `--features heic`, `info <fixture>.heic` reports the decoded raster: the
/// path-based seam routes HEIC the same way the byte-based one does.
#[cfg(feature = "heic")]
#[test]
fn info_heic_reports_dimensions() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.heic");
    std::fs::write(&in_path, HEIC_FIXTURE).unwrap();

    let output = Command::new(BIN)
        .args(["info", in_path.to_str().unwrap(), "--json"])
        .output()
        .expect("failed to run info");
    assert_eq!(
        output.status.code(),
        Some(0),
        "info should exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = String::from_utf8_lossy(&output.stdout);
    assert!(json.contains("\"width\":64"), "got:\n{json}");
    assert!(json.contains("\"height\":48"), "got:\n{json}");
}

/// A directory source containing a `.heic` (plus a non-image `.txt`) yields exactly
/// the `.heic` — extension discovery is feature-independent, so the default build
/// surfaces a codec-not-built error rather than silently skipping the file.
#[test]
fn directory_source_discovers_heic() {
    let dir = tempfile::tempdir().expect("tempdir");
    let heic = dir.path().join("a.heic");
    std::fs::write(&heic, HEIC_FIXTURE).unwrap();
    std::fs::write(dir.path().join("notes.txt"), b"not an image").unwrap();

    let inputs = resolve(dir.path().to_str().unwrap(), &mut std::io::empty()).unwrap();
    assert_eq!(
        inputs.len(),
        1,
        "expected exactly the .heic, got {inputs:?}"
    );
    match &inputs[0] {
        Input::Path(p) => assert_eq!(p.extension().and_then(|e| e.to_str()), Some("heic")),
        other => panic!("expected Path, got {other:?}"),
    }
}
