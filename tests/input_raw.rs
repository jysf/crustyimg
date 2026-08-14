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
//!
//! Second fixture: `tests/fixtures/raw/tight_preview.nef` — same shape, sized so
//! a default-quality re-encode comes back LARGER than the whole container. Only
//! SPEC-113's guard test needs that property; see its doc comment below.
//! Regen: `cargo run --example gen_raw_tight_fixture`.

use std::process::Command;

use crustyimg::source::{resolve, Input};

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");
const RAW_FIXTURE: &[u8] = include_bytes!("fixtures/raw/synthetic_preview.nef");
const TIGHT_RAW_FIXTURE: &[u8] = include_bytes!("fixtures/raw/tight_preview.nef");
const NOISE_RAW_FIXTURE: &[u8] = include_bytes!("fixtures/raw/noise_preview.nef");

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

/// SPEC-113 regression: `optimize <fixture>.nef -o out.jpg` — pinning to the
/// SAME format the RAW preview's `source_format` reports (`Jpeg`) — must still
/// write a REAL, decodable JPEG, never the raw `.nef` CONTAINER bytes.
///
/// `source_format` is an adopted label here (SPEC-061): the preview extraction
/// reports `Jpeg` because that is what got decoded, but the bytes on disk are
/// the whole RAW container (a TIFF-style header plus an embedded preview JPEG),
/// not a standalone JPEG file. SPEC-113's never-bigger guard must not mistake
/// "`source_format` says Jpeg" for "the source file's raw bytes decode as JPEG";
/// it sniffs the container with `guess_format` (which reports Tiff here) before
/// ever treating those bytes as shippable.
///
/// **Why this uses `tight_preview.nef` and not `synthetic_preview.nef`.** The
/// sniff is only consulted once the guard has already decided the re-encode did
/// not beat the source — the condition is `re-encode >= container`. On
/// `synthetic_preview.nef` that is false and always will be: its preview is a
/// solid colour stored at the SAME default quality the re-encode uses, so the
/// re-encode comes back at roughly the preview's own size (~712 B) while the
/// container also carries a thumbnail and header (~1365 B). The comparison
/// short-circuits on size, the sniff is never reached, and the test passes
/// whether or not the sniff exists — green while proving nothing.
///
/// `tight_preview.nef` is built to invert exactly that: a high-frequency preview
/// stored at low quality, so the container is 4073 B while the default-quality
/// re-encode of those pixels is 5351 B (1.31x). The guard therefore reaches the
/// sniff for real, and deleting the sniff makes this test FAIL — it writes the
/// `.nef` container bytes under a `.jpg` name, which the assertion below catches.
/// The generator asserts that size relationship at regen time so it cannot
/// silently decay back into a no-op.
#[test]
fn optimize_raw_input_pinned_to_jpeg_writes_real_jpeg() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.nef");
    std::fs::write(&in_path, TIGHT_RAW_FIXTURE).unwrap();
    let out_path = dir.path().join("out.jpg");

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

    let bytes = std::fs::read(&out_path).expect("read jpg output");
    assert_ne!(
        bytes, TIGHT_RAW_FIXTURE,
        "the guard must not ship the raw .nef container verbatim under a .jpg \
         name just because source_format reports an adopted Jpeg label"
    );
    assert_eq!(
        image::guess_format(&bytes).ok(),
        Some(image::ImageFormat::Jpeg),
        "output must be a real, sniffable JPEG — never the raw .nef container \
         bytes written under a .jpg name"
    );
    let decoded = image::load_from_memory(&bytes).expect("output should decode as JPEG");
    assert_eq!(decoded.width(), 224);
    assert_eq!(decoded.height(), 168);
}

/// SPEC-115 (AC-3): `optimize <fixture>.nef --out-dir out/` (the auto-decide
/// path, no `-o`) writes a REAL raster, never the raw `.nef` TIFF-family
/// container under an adopted `jpeg` label.
///
/// Fixture: `tests/fixtures/raw/noise_preview.nef` — a Floyd–Steinberg-dithered
/// 96×72 preview (`tests/fixtures/classify/RECIPES.md`'s recipe), few enough
/// colours to classify `Icon`/`GraphicLogo` (`OptBucket::LosslessFlat`, no AVIF
/// or lossy WebP admitted regardless of built features), whose dithering
/// pattern is adversarial to PNG/WebP-lossless prediction filters — so both
/// lossless candidates (1130 B / 2975 B) exceed the 903 B container and
/// `pick_winner` returns `None`. `tight_preview.nef` (SPEC-113) does NOT
/// reproduce this: its noise preview classifies `Photograph` (`OptBucket::Lossy`),
/// so AVIF at the fast fixed quality beats the container easily (2995 B <
/// 4073 B) — it only reproduces the PINNED-path defect. **Must fail today**:
/// this test proves RED on `main` before the fix; if it does not, the fixture
/// is wrong, not the defect ([[a-harness-that-exercises-nothing-reports-green]]).
/// Regen: `cargo run --example gen_raw_noise_fixture`.
#[test]
fn optimize_raw_auto_decide_writes_a_real_raster() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("noise_preview.nef");
    std::fs::write(&in_path, NOISE_RAW_FIXTURE).unwrap();

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
        !stderr.contains("jpeg \u{2192} jpeg"),
        "must not claim a JPEG re-encode of JPEG source bytes that were never \
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
        "the written file must sniff as a real raster format, not the raw .nef \
         TIFF-family container; path={:?} sniff={sniffed:?}",
        written[0]
    );
    assert_ne!(
        bytes, NOISE_RAW_FIXTURE,
        "the .nef container bytes must never be written verbatim under a raster name"
    );
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

/// SPEC-070 / DEC-063: `info` on the F-RAW-1 pixel bomb (a 782-byte `.nef` whose
/// embedded JPEG declares 16384×9776 = 160 Mpix) is REJECTED, not decoded.
///
/// Before SPEC-070 this exited **0** and printed `dimensions: 16384x9776` after a
/// **~1.93 GB** peak working set — it passed every DEC-034 cap because
/// `image::Limits.max_alloc` bounds a single allocation, not the cumulative peak.
/// Now the SOF dimension peek rejects it at the header: exit **1** (the
/// `LimitsExceeded` code) and a message naming the pixel budget.
///
/// The RSS bound itself is not assertable portably from a test (no cross-platform
/// peak-RSS API without a dependency), so it is measured on the real binary and
/// recorded in DEC-063 / the run record; what this test pins is that the input is
/// *rejected* — which is what makes the peak cheap — and that the rejection is the
/// typed limits path, not a decode failure.
#[test]
fn info_on_pixel_bomb_is_rejected_with_limits_exit_code() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("bomb.nef");
    let bomb = include_bytes!("fixtures/fuzz/raw_preview/pixel_bomb.nef");
    assert!(bomb.len() < 1024, "the bomb is a sub-kilobyte file");
    std::fs::write(&in_path, bomb).unwrap();

    let output = Command::new(BIN)
        .args(["info", in_path.to_str().unwrap()])
        .output()
        .expect("failed to run info");

    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit 1 (LimitsExceeded); stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("pixel"),
        "expected the pixel-budget message; got:\n{stderr}"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("16384x9776"),
        "the bomb's dimensions must never be reported — that means it decoded"
    );
}
