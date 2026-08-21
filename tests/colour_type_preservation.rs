//! Integration tests for SPEC-121 — ops preserve colour type and bit depth.
//!
//! `Invert`, `Resize` (reached by `resize`/`thumbnail`/`web`) and `Watermark`
//! used to widen every image to RGBA8 and never narrow back (docs/backlog.md,
//! "Live defect — ops widen to RGBA and never narrow back"). These drive the
//! REAL compiled binary end-to-end (matching `tests/watermark.rs`'s
//! convention) and read the output's `ColorType` back via
//! `crustyimg::image::Image::load` — a structural assertion on the decoded
//! IHDR, not a byte-size guess.
//!
//! **One `#[test]` fn per op body per claim, deliberately not bundled**
//! (AC-9): reverting one op body's fix must turn red only the tests below
//! that call it, while the other tests — independent functions — actually
//! run and pass, not merely go unreached by an early panic in a shared fn.

mod common;

use std::path::Path;
use std::process::Command;

use ::image::{ColorType, DynamicImage, ImageFormat, Rgba, RgbaImage};
use crustyimg::image::Image;

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Fixture helpers (native, no ImageMagick — AGENTS §12) ──────────────────

fn write_fixture(dir: &Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// A solid-color opaque RGBA8 PNG — used as a watermark overlay that should
/// narrow the composite back to RGB (Call 2's "fully opaque" direction).
fn write_opaque_overlay(dir: &Path, name: &str) -> std::path::PathBuf {
    let img = RgbaImage::from_pixel(6, 6, Rgba([10, 20, 30, 255]));
    let path = dir.join(name);
    DynamicImage::ImageRgba8(img)
        .save_with_format(&path, ImageFormat::Png)
        .unwrap();
    path
}

/// A solid-color translucent RGBA8 PNG — used as a watermark overlay that
/// genuinely contributes alpha (Call 2's "stays RGBA" direction).
fn write_translucent_overlay(dir: &Path, name: &str) -> std::path::PathBuf {
    let img = RgbaImage::from_pixel(6, 6, Rgba([10, 20, 30, 128]));
    let path = dir.join(name);
    DynamicImage::ImageRgba8(img)
        .save_with_format(&path, ImageFormat::Png)
        .unwrap();
    path
}

/// A grayscale (`L8`) gradient PNG — no alpha, one channel.
fn gray8_png(w: u32, h: u32) -> Vec<u8> {
    use ::image::{GrayImage, Luma};
    let img = GrayImage::from_fn(w, h, |x, y| Luma([((x * 7 + y * 5) % 256) as u8]));
    let mut out = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageLuma8(img)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

/// A 16-bit grayscale (`L16`) gradient PNG.
fn gray16_png(w: u32, h: u32) -> Vec<u8> {
    use ::image::{ImageBuffer, Luma};
    let img: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_fn(w, h, |x, y| Luma([((x * 1300 + y * 900) % 65536) as u16]));
    let mut out = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageLuma16(img)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

/// A grayscale-with-alpha (`La8`) PNG whose alpha is fully opaque everywhere.
/// The user supplied the channel, so it must survive (see
/// `graya_opaque_input_keeps_its_alpha_channel`).
fn graya8_opaque_png(w: u32, h: u32) -> Vec<u8> {
    use ::image::{ImageBuffer, LumaA};
    let img: ImageBuffer<LumaA<u8>, Vec<u8>> =
        ImageBuffer::from_fn(w, h, |x, y| LumaA([((x * 7 + y * 5) % 256) as u8, 255]));
    let mut out = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageLumaA8(img)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

/// An `Rgb8` gradient PNG.
fn rgb8_gradient_png(w: u32, h: u32) -> Vec<u8> {
    use ::image::{Rgb, RgbImage};
    let img = RgbImage::from_fn(w, h, |x, y| {
        Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8])
    });
    let mut out = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

/// The same pixels as [`rgb8_gradient_png`] plus an all-opaque alpha channel.
fn rgba8_opaque_gradient_png(w: u32, h: u32) -> Vec<u8> {
    let img = RgbaImage::from_fn(w, h, |x, y| {
        Rgba([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8, 255])
    });
    let mut out = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(img)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

/// An `Rgb8` PNG whose pixels all happen to be gray (`r == g == b`). The
/// control for "the rule preserves, it does not minimise".
fn rgb8_but_actually_gray_png(w: u32, h: u32) -> Vec<u8> {
    use ::image::{Rgb, RgbImage};
    let img = RgbImage::from_fn(w, h, |x, y| {
        let v = ((x * 7 + y * 5) % 256) as u8;
        Rgb([v, v, v])
    });
    let mut out = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

/// An `Rgba8` PNG with a genuinely transparent left half — the only base from
/// which a source-over composite can come out non-opaque.
fn rgba8_half_transparent_png(w: u32, h: u32) -> Vec<u8> {
    let img = RgbaImage::from_fn(w, h, |x, y| {
        Rgba([
            (x % 256) as u8,
            (y % 256) as u8,
            ((x + y) % 256) as u8,
            if x < w / 2 { 0 } else { 255 },
        ])
    });
    let mut out = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(img)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(BIN).args(args).output().expect("run binary")
}

fn load_color(path: &Path) -> ColorType {
    Image::load(path)
        .unwrap_or_else(|e| panic!("output {path:?} should decode: {e}"))
        .pixels()
        .color()
}

/// Run `args`, assert exit 0, and return the output file's decoded colour type.
fn run_ok_and_load_color(args: &[&str], out: &Path) -> ColorType {
    let output = run(args);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    load_color(out)
}

// ── AC-1: an RGB (colour_type=2) source stays RGB through every op verb ────
// rgb_png_stays_rgb_through_resize_thumbnail_edit_and_web

#[test]
fn resize_max_preserves_rgb_colour_type() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(
        dir.path(),
        "src.png",
        &common::solid_png(32, 32, [200, 100, 50]),
    );
    let out = dir.path().join("out.png");
    let output = run(&[
        "resize",
        src.to_str().unwrap(),
        "--max",
        "16",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgb8);
}

#[test]
fn thumbnail_preserves_rgb_colour_type() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(
        dir.path(),
        "src.png",
        &common::solid_png(32, 32, [200, 100, 50]),
    );
    let out = dir.path().join("out.png");
    let output = run(&[
        "thumbnail",
        src.to_str().unwrap(),
        "--size",
        "16",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgb8);
}

#[test]
fn edit_invert_preserves_rgb_colour_type() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(
        dir.path(),
        "src.png",
        &common::solid_png(32, 32, [200, 100, 50]),
    );
    let out = dir.path().join("out.png");
    let output = run(&[
        "edit",
        src.to_str().unwrap(),
        "--invert",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgb8);
}

/// The flagship verb: `web` is `auto-orient → resize → optimize` (docs/backlog.md),
/// so fixing `Resize` fixes `web`.
#[test]
fn web_preserves_rgb_colour_type() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(
        dir.path(),
        "src.png",
        &common::solid_png(32, 32, [200, 100, 50]),
    );
    let out = dir.path().join("out.bin");
    let output = run(&["web", src.to_str().unwrap(), "-o", out.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgb8);
}

// ── AC-2: a 16-bit RGB source stays 16-bit ──────────────────────────────────
// sixteen_bit_png_stays_sixteen_bit

#[test]
fn resize_max_preserves_sixteen_bit() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "src16.png", &common::png_16bit(32, 32));
    let out = dir.path().join("out.png");
    let output = run(&[
        "resize",
        src.to_str().unwrap(),
        "--max",
        "16",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgb16);
}

#[test]
fn edit_invert_preserves_sixteen_bit() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "src16.png", &common::png_16bit(32, 32));
    let out = dir.path().join("out.png");
    let output = run(&[
        "edit",
        src.to_str().unwrap(),
        "--invert",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgb16);
}

// `web` is deliberately NOT covered here for the 16-bit case (unlike AC-1's
// RGB case above): `web`'s final step is `optimize`'s smallest-candidate
// search, which — independently of this spec — may pick WebP/JPEG/AVIF over
// PNG, and `image`'s own *lossless* WebP encoder has no 16-bit mode either
// (no permissive Rust encoder exists — out of scope, a dependency question,
// not a fix here per SPEC-125's "Out of scope"), so the depth is still lost
// the same way Call 3 documented for JPEG/lossy-WebP. What changed
// (SPEC-125, STAGE-042): this is no longer SILENT — the sink now warns on
// stderr for every measured 8-bit-only target (BMP/lossless-WebP/AVIF, not
// just JPEG/lossy-WebP), and a scored winner's ssim line is qualified rather
// than reading a false-perfect 100.0 (`tests/sink.rs`'s
// `web_and_optimize_reach_the_widened_downgrade_warning` and
// `ssim_line_is_qualified_across_a_depth_change` drive exactly this case).
// This file still doesn't assert it — that coverage belongs to `sink.rs`,
// which owns the diagnostic — but the depth loss itself is real and
// unfixable without a 16-bit WebP encoder, so this test suite still doesn't
// claim `web` preserves 16-bit depth.

// ── AC-3: an RGBA input with genuine translucency keeps its alpha ──────────
// translucent_rgba_input_keeps_its_alpha — the control that stops "always
// narrow" from destroying real transparency. Passes today; must keep passing.

#[test]
fn resize_keeps_translucent_alpha() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "src.png", &common::rgba_png(32, 32));
    let out = dir.path().join("out.png");
    let output = run(&[
        "resize",
        src.to_str().unwrap(),
        "--max",
        "16",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgba8);
}

#[test]
fn edit_invert_keeps_translucent_alpha() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "src.png", &common::rgba_png(32, 32));
    let out = dir.path().join("out.png");
    let output = run(&[
        "edit",
        src.to_str().unwrap(),
        "--invert",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(load_color(&out), ColorType::Rgba8);
}

// ── AC-4: `Watermark` decides in code, not by exemption (Call 2) ───────────
//
// The rule, stated once: source-over compositing can only *produce* a
// non-opaque pixel where the base already had one. `a_out = a_base +
// a_ov·(1 − a_base)` is 1 for every overlay alpha when `a_base` is 1 — so a
// base with no alpha channel composites to a fully opaque result no matter
// how translucent the overlay is, and the result narrows. A base that
// carries genuine transparency keeps its channel.

/// watermark_narrows_when_the_composite_is_opaque
#[test]
fn watermark_narrows_when_composite_is_opaque() {
    let dir = tempfile::tempdir().unwrap();
    let base = write_fixture(
        dir.path(),
        "base.png",
        &common::solid_png(32, 32, [200, 100, 50]),
    );
    let overlay = write_opaque_overlay(dir.path(), "overlay.png");
    let out = dir.path().join("out.png");
    let output = run(&[
        "watermark",
        base.to_str().unwrap(),
        "--image",
        overlay.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        load_color(&out),
        ColorType::Rgb8,
        "a fully opaque overlay over an RGB base must narrow like the other ops"
    );
}

/// **The verb's commonest invocation, and the one that pays this spec's bill.**
///
/// `--text` rasterizes anti-aliased glyphs, so the overlay carries partial
/// alpha at every glyph edge and every one of those pixels goes down
/// `Rgba::blend`'s `f32` path — unlike the two uniform-overlay tests above,
/// which hit its `alpha == MAX` / `alpha == 0` short-circuits. That float path
/// truncates `255 × 0.99999994` to **254**, and a few dozen such pixels used
/// to defeat the narrow on an otherwise fully opaque composite (measured: 36
/// of 65,536 on a 256×256 base, output 66,313 B as RGBA vs 53,970 B as RGB —
/// **18.6 % of the file** spent on a channel carrying nothing).
#[test]
fn watermark_text_narrows_on_an_opaque_base() {
    let dir = tempfile::tempdir().unwrap();
    let base = write_fixture(dir.path(), "base.png", &rgb8_gradient_png(96, 96));
    let out = dir.path().join("out.png");
    assert_eq!(
        run_ok_and_load_color(
            &[
                "watermark",
                base.to_str().unwrap(),
                "--text",
                "hello",
                "-o",
                out.to_str().unwrap(),
            ],
            &out,
        ),
        ColorType::Rgb8,
        "anti-aliased text over an opaque RGB base composites to an opaque \
         image; the alpha channel carries nothing and must not be kept"
    );
}

/// The same ruling from the other side, on a uniform half-transparent overlay:
/// 128 is one of the 32 overlay alphas whose blended result truncates to 254.
/// The composite is still mathematically opaque, so it narrows.
#[test]
fn watermark_translucent_overlay_on_an_opaque_base_narrows() {
    let dir = tempfile::tempdir().unwrap();
    let base = write_fixture(
        dir.path(),
        "base.png",
        &common::solid_png(32, 32, [200, 100, 50]),
    );
    let overlay = write_translucent_overlay(dir.path(), "overlay.png");
    let out = dir.path().join("out.png");
    assert_eq!(
        run_ok_and_load_color(
            &[
                "watermark",
                base.to_str().unwrap(),
                "--image",
                overlay.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
            ],
            &out,
        ),
        ColorType::Rgb8,
        "source-over onto an opaque base cannot produce transparency"
    );
}

/// watermark_keeps_alpha_when_the_overlay_is_translucent — **the control.**
///
/// The base carries genuine transparency, so the composite genuinely has
/// non-opaque samples and the channel must survive. This asserts the pixels,
/// not only the colour type: a fix that narrowed by tolerance rather than by
/// rule would destroy real transparency, which is a worse bug than the wasted
/// channel it removes.
#[test]
fn watermark_keeps_alpha_when_overlay_is_translucent() {
    let dir = tempfile::tempdir().unwrap();
    let base = write_fixture(dir.path(), "base.png", &rgba8_half_transparent_png(32, 32));
    let overlay = write_translucent_overlay(dir.path(), "overlay.png");
    let out = dir.path().join("out.png");
    let output = run(&[
        "watermark",
        base.to_str().unwrap(),
        "--image",
        overlay.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        load_color(&out),
        ColorType::Rgba8,
        "a composite with genuinely non-opaque samples must stay RGBA"
    );

    let decoded = Image::load(&out).unwrap().pixels().to_rgba8();
    let transparent = decoded.pixels().filter(|p| p.0[3] == 0).count();
    assert!(
        transparent > 0,
        "the transparency itself must survive, not only the channel: \
         {transparent} fully transparent pixels of {}",
        decoded.pixels().count()
    );
}

// ── AC-6: the byte win is measured, not assumed ─────────────────────────────
// rgb_output_is_smaller_than_rgba_for_the_same_pixels
//
// This has to run an OP. Comparing an RGB encode to an RGBA encode of the
// same pixels measures the PNG encoder, not the fix — it passes identically
// on `main`, where every op returns RGBA. [[fixtures-from-the-code-under-test
// -cannot-fail]] So: run the same op over the same pixels, once from an RGB
// source and once from an RGBA one, and compare what the tool actually
// writes. On `main` both come out RGBA8 and the sizes match, so the `<` is
// red; here the RGB source stays RGB and is materially smaller.

#[test]
fn rgb_output_is_smaller_than_rgba_for_the_same_pixels() {
    let dir = tempfile::tempdir().unwrap();
    let (w, h) = (128, 128);
    let rgb_src = write_fixture(dir.path(), "rgb.png", &rgb8_gradient_png(w, h));
    let rgba_src = write_fixture(dir.path(), "rgba.png", &rgba8_opaque_gradient_png(w, h));
    let rgb_out = dir.path().join("rgb_out.png");
    let rgba_out = dir.path().join("rgba_out.png");

    assert_eq!(
        run_ok_and_load_color(
            &[
                "edit",
                rgb_src.to_str().unwrap(),
                "--invert",
                "-o",
                rgb_out.to_str().unwrap(),
            ],
            &rgb_out,
        ),
        ColorType::Rgb8,
        "the op must return the RGB source's own colour type"
    );
    assert_eq!(
        run_ok_and_load_color(
            &[
                "edit",
                rgba_src.to_str().unwrap(),
                "--invert",
                "-o",
                rgba_out.to_str().unwrap(),
            ],
            &rgba_out,
        ),
        ColorType::Rgba8,
        "an alpha channel the user supplied is not stripped, even all-opaque"
    );

    let rgb_bytes = std::fs::metadata(&rgb_out).unwrap().len();
    let rgba_bytes = std::fs::metadata(&rgba_out).unwrap().len();
    assert!(
        rgb_bytes < rgba_bytes,
        "the same op over the same pixels: RGB out {rgb_bytes} B should be \
         smaller than RGBA out {rgba_bytes} B"
    );
}

// ── Grayscale: the same rule, one channel wider (punch-list item 6) ─────────
//
// AC-1/AC-2 name RGB only, so they were literally met while `Gray8 → resize`
// still returned RGB8 — Call 1 ("an op preserves the input's colour type")
// was not. An AC may not transfer between surfaces (AGENTS §15), so the
// criterion is written here against the surface grayscale actually has.
//
// Measured on a 32×32 gradient, `resize --max 16`: `L8` was 852 B as RGB8 and
// is 340 B as `L8` (−60.1 %); `L16` was 1,559 B and is 596 B (−61.8 %);
// `La8` was 962 B as RGBA8 and is 447 B as `La8` (−53.5 %).

#[test]
fn resize_preserves_grayscale_colour_type() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "gray.png", &gray8_png(32, 32));
    let out = dir.path().join("out.png");
    assert_eq!(
        run_ok_and_load_color(
            &[
                "resize",
                src.to_str().unwrap(),
                "--max",
                "16",
                "-o",
                out.to_str().unwrap(),
            ],
            &out,
        ),
        ColorType::L8
    );
}

#[test]
fn edit_invert_preserves_grayscale_colour_type() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "gray.png", &gray8_png(32, 32));
    let out = dir.path().join("out.png");
    assert_eq!(
        run_ok_and_load_color(
            &[
                "edit",
                src.to_str().unwrap(),
                "--invert",
                "-o",
                out.to_str().unwrap(),
            ],
            &out,
        ),
        ColorType::L8
    );
}

/// Both halves of the rule at once: one channel *and* 16 bits.
#[test]
fn resize_preserves_sixteen_bit_grayscale() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "gray16.png", &gray16_png(32, 32));
    let out = dir.path().join("out.png");
    assert_eq!(
        run_ok_and_load_color(
            &[
                "resize",
                src.to_str().unwrap(),
                "--max",
                "16",
                "-o",
                out.to_str().unwrap(),
            ],
            &out,
        ),
        ColorType::L16
    );
}

/// The `has_alpha` half of the rule, on the luma surface: an all-opaque alpha
/// channel the user supplied survives, as `La8` rather than `L8`.
#[test]
fn graya_opaque_input_keeps_its_alpha_channel() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "graya.png", &graya8_opaque_png(32, 32));
    let out = dir.path().join("out.png");
    assert_eq!(
        run_ok_and_load_color(
            &[
                "resize",
                src.to_str().unwrap(),
                "--max",
                "16",
                "-o",
                out.to_str().unwrap(),
            ],
            &out,
        ),
        ColorType::La8,
        "do not strip a channel the user supplied"
    );
}

/// The same rule on the colour surface, and the reason it is a rule about the
/// *input* rather than about the pixels: an all-opaque `Rgba8` input stays
/// `Rgba8`.
#[test]
fn rgba_opaque_input_keeps_its_alpha_channel() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(dir.path(), "rgba.png", &rgba8_opaque_gradient_png(32, 32));
    let out = dir.path().join("out.png");
    assert_eq!(
        run_ok_and_load_color(
            &[
                "resize",
                src.to_str().unwrap(),
                "--max",
                "16",
                "-o",
                out.to_str().unwrap(),
            ],
            &out,
        ),
        ColorType::Rgba8
    );
}

/// **The rule preserves; it does not minimise.** An `Rgb8` source whose pixels
/// all happen to be gray stays `Rgb8` — collapsing it would be a promotion of
/// the tool's judgement over the user's declared colour type, and would make
/// the output's type depend on its content.
#[test]
fn rgb_input_that_happens_to_be_gray_stays_rgb() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_fixture(
        dir.path(),
        "grayish.png",
        &rgb8_but_actually_gray_png(32, 32),
    );
    let out = dir.path().join("out.png");
    assert_eq!(
        run_ok_and_load_color(
            &[
                "resize",
                src.to_str().unwrap(),
                "--max",
                "16",
                "-o",
                out.to_str().unwrap(),
            ],
            &out,
        ),
        ColorType::Rgb8
    );
}

/// The other direction, and the reason the luma narrow is conditional on the
/// pixels as well as on the input type: a colour overlay over a gray base
/// genuinely adds chroma, so the output must widen and stay RGB.
#[test]
fn colour_watermark_on_a_gray_base_becomes_rgb() {
    let dir = tempfile::tempdir().unwrap();
    let base = write_fixture(dir.path(), "gray.png", &gray8_png(32, 32));
    let overlay = write_opaque_overlay(dir.path(), "overlay.png");
    let out = dir.path().join("out.png");
    assert_eq!(
        run_ok_and_load_color(
            &[
                "watermark",
                base.to_str().unwrap(),
                "--image",
                overlay.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
            ],
            &out,
        ),
        ColorType::Rgb8,
        "the composite really did gain colour — narrowing to luma would lose it"
    );
}

// ── AC-7: the verbs that were already correct are unchanged ─────────────────
// convert_optimize_auto_orient_bytes_unchanged
//
// AC-7's evidence at build/verify was a byte diff against `main`'s binary,
// which a test cannot carry. What it CAN pin — and what would actually break
// if the narrowing rule ever leaked out of `Operation` into the clean verbs —
// is the property that diff was evidence for: `convert`, `optimize` and
// `auto-orient` run no `Operation`, so they hand the decoded image to the
// sink untouched. Two assertions per fixture:
//
//  1. the output's colour type and bit depth are the INPUT's, across all six
//     colour types the ops narrow between (an op-side narrow leaking in would
//     move `La8`→`L8` or `Rgba8`→`Rgb8` here);
//  2. `convert --format png` is byte-identical to `auto-orient`'s PNG output —
//     two independent entry points that must agree because neither transforms
//     pixels.
//
// Re-driven against `main`'s binary during this cycle: 6 fixtures × 3 verbs,
// all byte-identical, with `resize` as the positive control (it differs).
#[test]
fn convert_optimize_auto_orient_bytes_unchanged() {
    let dir = tempfile::tempdir().unwrap();

    let cases: [(&str, Vec<u8>, ColorType); 6] = [
        ("gray8", gray8_png(32, 32), ColorType::L8),
        ("graya8", graya8_opaque_png(32, 32), ColorType::La8),
        ("rgb8", rgb8_gradient_png(32, 32), ColorType::Rgb8),
        (
            "rgba8",
            rgba8_half_transparent_png(32, 32),
            ColorType::Rgba8,
        ),
        ("gray16", gray16_png(32, 32), ColorType::L16),
        ("rgb16", common::png_16bit(32, 32), ColorType::Rgb16),
    ];

    for (name, bytes, expected) in cases {
        let src = write_fixture(dir.path(), &format!("{name}.png"), &bytes);
        assert_eq!(
            load_color(&src),
            expected,
            "{name}: the fixture itself must be the colour type this case claims"
        );

        let conv = dir.path().join(format!("{name}_convert.png"));
        assert_eq!(
            run_ok_and_load_color(
                &[
                    "convert",
                    src.to_str().unwrap(),
                    "--format",
                    "png",
                    "-o",
                    conv.to_str().unwrap(),
                ],
                &conv,
            ),
            expected,
            "{name}: convert runs no Operation and must not narrow"
        );

        let orient = dir.path().join(format!("{name}_orient.png"));
        assert_eq!(
            run_ok_and_load_color(
                &[
                    "auto-orient",
                    src.to_str().unwrap(),
                    "-o",
                    orient.to_str().unwrap()
                ],
                &orient,
            ),
            expected,
            "{name}: auto-orient runs no Operation and must not narrow"
        );

        let opt = dir.path().join(format!("{name}_optimize.png"));
        assert_eq!(
            run_ok_and_load_color(
                &[
                    "optimize",
                    src.to_str().unwrap(),
                    "-o",
                    opt.to_str().unwrap()
                ],
                &opt,
            ),
            expected,
            "{name}: optimize runs no Operation and must not narrow"
        );

        assert_eq!(
            std::fs::read(&conv).unwrap(),
            std::fs::read(&orient).unwrap(),
            "{name}: convert and auto-orient both pass the decoded image \
             straight to the sink, so their PNG bytes must agree"
        );
    }
}
