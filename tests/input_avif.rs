//! Integration tests for AVIF as a DEFAULT, pure-Rust input (SPEC-058).
//!
//! The default build reads `.avif` end to end (decode via `re_rav1d` +
//! `avif-parse`, no dav1d/C), so `optimize`/`convert`/batch see it like any
//! other image. A separate `#[cfg(feature = "avif")]` round-trip proves the
//! decode path against crustyimg's own AVIF encoder.
//!
//! Fixture: `tests/fixtures/avif/solid_16x16.avif`, a 16×16 solid image.
//! Regen: `cargo run --example gen_avif_fixture --features avif`.

use std::process::Command;

use crustyimg::source::{resolve, Input};

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");
const AVIF_FIXTURE: &[u8] = include_bytes!("fixtures/avif/solid_16x16.avif");

/// `optimize <fixture>.avif -o out.webp` exits 0 and writes a valid WebP with
/// the fixture's dimensions — proving AVIF input flows through the pipeline on
/// the default build.
#[test]
fn optimize_avif_input_writes_webp() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("in.avif");
    std::fs::write(&in_path, AVIF_FIXTURE).unwrap();
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
    assert_eq!(decoded.width(), 16);
    assert_eq!(decoded.height(), 16);
}

/// A directory source containing an `.avif` (plus a non-image `.txt`) yields
/// exactly the `.avif` — `.avif` is in the source allow-list.
#[test]
fn directory_source_discovers_avif() {
    let dir = tempfile::tempdir().expect("tempdir");
    let avif = dir.path().join("a.avif");
    std::fs::write(&avif, AVIF_FIXTURE).unwrap();
    std::fs::write(dir.path().join("notes.txt"), b"not an image").unwrap();

    let inputs = resolve(dir.path().to_str().unwrap(), &mut std::io::empty()).unwrap();
    assert_eq!(
        inputs.len(),
        1,
        "expected exactly the .avif, got {inputs:?}"
    );
    match &inputs[0] {
        Input::Path(p) => assert_eq!(p.extension().and_then(|e| e.to_str()), Some("avif")),
        other => panic!("expected Path, got {other:?}"),
    }
}

/// `#[cfg(feature = "avif")]` round-trip: a natively-generated 32×32 gradient
/// encoded to AVIF via crustyimg's own encoder and decoded back matches
/// dimensions and is perceptually close (SSIMULACRA2 high) — proving the decode
/// path against the encoder.
#[cfg(feature = "avif")]
#[test]
fn avif_roundtrip_gradient() {
    use crustyimg::image::Image;
    use crustyimg::sink::encode_to_bytes;
    use image::{DynamicImage, ImageFormat, Rgb, RgbImage};

    let (w, h) = (32u32, 32u32);
    let mut buf = RgbImage::new(w, h);
    for (x, y, px) in buf.enumerate_pixels_mut() {
        let r = (x * 255 / (w - 1)) as u8;
        let g = (y * 255 / (h - 1)) as u8;
        let b = ((x + y) * 255 / (w + h - 2)) as u8;
        *px = Rgb([r, g, b]);
    }
    let source = DynamicImage::ImageRgb8(buf);
    let src_image = Image::from_parts(source.clone(), ImageFormat::Png, None);

    // Encode to AVIF via the sink (crustyimg's own ravif path).
    let avif_bytes = encode_to_bytes(&src_image, ImageFormat::Avif, Some(90)).expect("encode avif");
    assert_eq!(
        image::guess_format(&avif_bytes).unwrap(),
        ImageFormat::Avif,
        "encoded bytes should be AVIF"
    );

    // Decode back through the default pure-Rust path.
    let decoded = Image::from_bytes(&avif_bytes).expect("decode avif");
    assert_eq!(decoded.width(), w);
    assert_eq!(decoded.height(), h);
    assert_eq!(decoded.source_format(), ImageFormat::Avif);

    // Perceptually close. SSIMULACRA2 is a 0–100 scale (100 = identical); a q90
    // gradient decodes to a high score, far above any "clearly wrong colors"
    // failure. Threshold is conservative to stay non-flaky across platforms.
    let s = crustyimg::quality::score(&source, decoded.pixels()).expect("ssimulacra2");
    assert!(s >= 70.0, "round-trip perceptual score too low: {s}");
}

/// SPEC-091/DEC-077: the single-thread (inline) decode must not depend on the
/// caller's OS-defined stack. With `n_threads == 1`, `re_rav1d` decodes inline and
/// dav1d's fixed decode frame overflows a small stack *regardless of image size* —
/// on Windows, whose main thread is only ~1 MiB, even a 16×16 AVIF stack-overflowed
/// (both windows-latest legs of PR #95). This mirrors that exact scenario on any
/// platform: run the decode from a deliberately ~1 MiB caller thread. It passes
/// only because `decode_obus` re-spawns the decode onto its own ample stack; if it
/// ever ran inline again, this thread would overflow and abort the test process.
/// Ungated (decode is a default dep) so it guards the default `cargo test` too,
/// like the `optimize_avif_input_writes_webp` test that first caught the overflow.
#[test]
fn avif_decode_survives_a_small_caller_stack() {
    use crustyimg::image::Image;

    // ~1 MiB — the Windows main-thread stack that overflowed on the inline decode.
    // The container parse + YUV→RGBA conversion run on this caller stack and fit
    // (Windows got past parse); only the inline decode overflowed, and that now
    // runs elsewhere. A returned dimension proves the decode completed, not aborted.
    let dims = std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            let img = Image::from_bytes(AVIF_FIXTURE).expect("decode 16x16 avif on small stack");
            (img.width(), img.height())
        })
        .expect("spawn small-stack caller thread")
        .join()
        .expect("small-stack decode thread panicked or overflowed");
    assert_eq!(dims, (16, 16));
}

/// SPEC-091/DEC-077: the single-thread decode policy must not change decoded
/// pixels. This pins the decoded RGBA of a committed 128×128 photo AVIF to a
/// digest **captured from the pre-change (all-cores) binary** at HEAD cd39f17 —
/// an independent value the code under test cannot fabricate. dav1d is a
/// conformant decoder whose output is bit-exact regardless of thread count, so a
/// digest change here means the thread policy altered pixels (a bug), not a
/// benign difference. The fixture is `photo_128.avif` (real photo content, so the
/// multi-threaded CDEF/loop-restoration path that the cap removes actually ran
/// when the golden was captured).
#[cfg(feature = "avif")]
#[test]
fn avif_decode_pixels_unchanged_by_thread_policy() {
    use crustyimg::image::Image;

    const PHOTO_128: &[u8] = include_bytes!("fixtures/avif/photo_128.avif");
    // FNV-1a of the decoded RGBA, captured on the pre-change binary (n_threads=0).
    const PRE_CHANGE_RGBA_FNV1A: u64 = 0x0d2b_956b_63f0_cd85;

    fn fnv1a(bytes: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }

    let img = Image::from_bytes(PHOTO_128).expect("decode photo_128.avif");
    assert_eq!((img.width(), img.height()), (128, 128));
    let rgba = img.pixels().to_rgba8();
    assert_eq!(
        fnv1a(rgba.as_raw()),
        PRE_CHANGE_RGBA_FNV1A,
        "single-thread decode changed pixels vs the pre-change all-cores decode"
    );
}
