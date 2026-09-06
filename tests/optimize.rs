//! Integration tests for SPEC-127's terminal-`optimize` carve-out (Call 2):
//! a recipe that both ends in the reserved `optimize` step and declares an
//! explicit `format` skips the auto-decision and honours the pin; the same
//! recipe without one still auto-decides.
//!
//! Drives the real compiled binary via `env!("CARGO_BIN_EXE_crustyimg")`, the
//! same convention `tests/apply_batch.rs` uses.

use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;

use image::{DynamicImage, ImageFormat, RgbImage};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// A photographic-shaped fixture: high-entropy gradient + pseudo-noise, well
/// over any icon-size threshold, so the analysis layer buckets it as a real
/// photograph (`ImageClass::Photograph`) rather than a flat graphic — the
/// bucket whose shortlist favors a lossy modern format over a lossless
/// re-encode, on every feature leg (with or without AVIF/webp-lossy built).
/// Mirrors `tests/wasm_roundtrip.rs`'s `photo_png_192x160` generator.
fn photo_jpeg(dir: &TempDir, name: &str) -> PathBuf {
    let (w, h) = (192u32, 160u32);
    let mut img = RgbImage::new(w, h);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let n = ((x.wrapping_mul(2654435761)) ^ (y.wrapping_mul(40503))) % 37;
        let gx = (x * 255 / w) as i32;
        let gy = (y * 255 / h) as i32;
        let tex = if ((x / 5) + (y / 3)) % 2 == 0 { 24 } else { 0 };
        let r = (gx + tex + n as i32).clamp(0, 255) as u8;
        let g = (gy + n as i32 * 2).clamp(0, 255) as u8;
        let b = ((gx + gy) / 2 + tex - n as i32).clamp(0, 255) as u8;
        *px = image::Rgb([r, g, b]);
    }
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Jpeg)
        .unwrap();
    let path = dir.path().join(name);
    std::fs::write(&path, buf.into_inner()).unwrap();
    path
}

/// Write a recipe TOML string to `dir/name`. Returns the path.
fn write_recipe(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

/// A terminal-`optimize` recipe (auto-orient + a small resize, matching the
/// shape of every bundled flow) that ALSO declares an explicit `format` —
/// SPEC-127 Call 2's carve-out. Pinned to `png`, a format the fast auto-decide
/// shortlist essentially never picks for real photographic content on ANY
/// feature leg (the Lossy-bucket shortlist favors a lossy modern format over
/// a lossless re-encode) — so a PNG output can only mean the pin fired, not a
/// coincidence of whichever codecs happen to be compiled in.
const TERMINAL_OPTIMIZE_WITH_FORMAT: &str = r#"
version = "2"
format = "png"

[[step]]
op = "auto-orient"

[[step]]
op = "resize"
mode = "max"
width = 64

[[step]]
op = "optimize"
"#;

/// The identical shape, with no `format` declared — the auto-decision must
/// still run exactly as it did before this spec.
const TERMINAL_OPTIMIZE_NO_FORMAT: &str = r#"
version = "1"

[[step]]
op = "auto-orient"

[[step]]
op = "resize"
mode = "max"
width = 64

[[step]]
op = "optimize"
"#;

/// AC-6: a bundled-shaped recipe declaring an explicit `format` skips the
/// auto-decision and honours the pin; the same recipe without one still
/// auto-decides.
#[test]
fn terminal_optimize_honours_an_explicit_recipe_format() {
    let dir = TempDir::new().unwrap();
    let src = photo_jpeg(&dir, "photo.jpg");

    let pinned_recipe = write_recipe(&dir, "pinned.toml", TERMINAL_OPTIMIZE_WITH_FORMAT);
    let auto_recipe = write_recipe(&dir, "auto.toml", TERMINAL_OPTIMIZE_NO_FORMAT);

    let pinned_out = dir.path().join("pinned_out");
    std::fs::create_dir_all(&pinned_out).unwrap();
    let pinned_output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            pinned_recipe.to_str().unwrap(),
            src.to_str().unwrap(),
            "--out-dir",
            pinned_out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply with a pinned terminal-optimize recipe");
    assert!(
        pinned_output.status.success(),
        "pinned run: exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&pinned_output.stderr)
    );

    let auto_out = dir.path().join("auto_out");
    std::fs::create_dir_all(&auto_out).unwrap();
    let auto_output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            auto_recipe.to_str().unwrap(),
            src.to_str().unwrap(),
            "--out-dir",
            auto_out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply with the auto-deciding terminal-optimize recipe");
    assert!(
        auto_output.status.success(),
        "auto-decide run: exit 0 expected; stderr: {}",
        String::from_utf8_lossy(&auto_output.stderr)
    );

    // Find whichever output file each run actually wrote (the auto-decide run
    // names it whatever extension it decided; `--out-dir` writes exactly one
    // file per input here).
    let read_only_output = |out_dir: &std::path::Path| -> Vec<u8> {
        let mut entries: Vec<_> = std::fs::read_dir(out_dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "exactly one output file expected in {out_dir:?}, found {entries:?}"
        );
        std::fs::read(entries.remove(0)).unwrap()
    };

    let pinned_bytes = read_only_output(&pinned_out);
    let auto_bytes = read_only_output(&auto_out);

    assert_eq!(
        image::guess_format(&pinned_bytes).expect("pinned output must sniff as a known format"),
        ImageFormat::Png,
        "an explicit recipe.format must skip the auto-decision and honour the pin"
    );
    assert_ne!(
        image::guess_format(&auto_bytes).expect("auto-decide output must sniff as a known format"),
        ImageFormat::Png,
        "the same recipe with no explicit format must still let the fast \
         auto-decide engine choose — for real photographic content, PNG is \
         never the auto-decided winner on any feature leg, so seeing PNG here \
         would mean the carve-out fired unconditionally instead of only when \
         `format` is set"
    );

    // Both runs actually resized (the pixel steps ran, not just the terminal
    // marker's own decision) — decode and check dimensions shrank from the
    // 192-wide source to the recipe's `max 64` bound. Decoded via crustyimg's
    // own `Image::from_bytes` (which wires AVIF decode via `re_rav1d`), not
    // the bare `image` crate — the auto-decide winner may be AVIF, which the
    // `image` crate alone cannot decode in this build.
    let pinned_img =
        crustyimg::image::Image::from_bytes(&pinned_bytes).expect("pinned output decodes");
    assert!(
        pinned_img.width() <= 64 && pinned_img.height() <= 64,
        "pinned output must have been resized to max 64, got {}x{}",
        pinned_img.width(),
        pinned_img.height()
    );
    let auto_img =
        crustyimg::image::Image::from_bytes(&auto_bytes).expect("auto-decide output decodes");
    assert!(
        auto_img.width() <= 64 && auto_img.height() <= 64,
        "auto-decide output must have been resized to max 64, got {}x{}",
        auto_img.width(),
        auto_img.height()
    );
}
