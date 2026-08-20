//! Integration tests for SPEC-122 — `resize` resamples in linear light.
//!
//! `Resize::apply` handed non-linear sRGB samples to `fast_image_resize` as if
//! they were linear light (`docs/backlog.md`, "Open — resize resamples in sRGB,
//! not linear light"). SPEC-120 measured it before it was specced: against an
//! independent linear-light reference (ImageMagick 7 Q16-HDRI) the shipped
//! downscale scored **70.45** and **84.45** SSIMULACRA2 on two corpus images,
//! and **−63.85** on a synthetic worst case (DEC-092).
//!
//! **These tests are hermetic; they are not the spec's independent oracle.**
//! AGENTS §12 forbids shelling out to ImageMagick for test fixtures, so the
//! reference here is derived from the sRGB standard's own transfer function
//! (IEC 61966-2-1), re-stated in this file rather than imported from the code
//! under test. The independent-tool measurement lives in
//! `scripts/spec120_linear_light.py` and is re-run per spec Call 3; these are
//! the regression guards that run in CI.
//!
//! The sources are built so the correct answer is known *analytically*, not
//! measured: alternating rows of two colours, downscaled 2:1 vertically. For
//! that geometry every symmetric pair of source rows around an output sample
//! point contains one row of each parity (distances ∓0.5, ∓1.5, ∓2.5 …), so
//! any symmetric kernel — Lanczos3 included — gives the two colours exactly
//! equal total weight. The output is therefore the *mean of the two colours*,
//! in whichever space the averaging happened: linear light (correct) or sRGB
//! (the defect). Those two answers are far apart, which is what makes the
//! assertions sharp.
//!
//! One `#[test]` fn per claim, deliberately not bundled: reverting the
//! linearization must turn red only the tests that assert it, while the alpha
//! and no-op guards still run and pass.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use ::image::{DynamicImage, ImageBuffer, ImageFormat, Rgb, RgbImage, Rgba, RgbaImage};

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── The sRGB transfer function, re-derived from IEC 61966-2-1 ───────────────
// Deliberately a second statement of the standard rather than a `use` of the
// production one: a reference computed by the code under test cannot fail it
// [[fixtures-from-the-code-under-test-cannot-fail]].

fn to_linear(signal: f64) -> f64 {
    if signal <= 0.04045 {
        signal / 12.92
    } else {
        ((signal + 0.055) / 1.055).powf(2.4)
    }
}

fn to_signal(linear: f64) -> f64 {
    if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

/// The correct 8-bit result of averaging two 8-bit sRGB codes **in linear
/// light** — the value this spec exists to produce.
fn linear_mean(a: u8, b: u8) -> f64 {
    to_signal((to_linear(a as f64 / 255.0) + to_linear(b as f64 / 255.0)) / 2.0) * 255.0
}

/// The same average taken **in sRGB space** — the value `main` produced.
fn signal_mean(a: u8, b: u8) -> f64 {
    (a as f64 + b as f64) / 2.0
}

// ── Fixtures (native, no ImageMagick — AGENTS §12) ──────────────────────────

fn write_png(dir: &Path, name: &str, img: DynamicImage) -> PathBuf {
    let path = dir.join(name);
    img.save_with_format(&path, ImageFormat::Png).unwrap();
    path
}

/// Rows alternate between a horizontal ramp and black. Downscaled 2:1
/// vertically, every output pixel is the mean of `(x, 0)` — analytically
/// known, and different in the two colour spaces at every column but x=0.
fn ramp_stripes(w: u32, h: u32) -> DynamicImage {
    let img: RgbImage = ImageBuffer::from_fn(w, h, |x, y| {
        if y % 2 == 0 {
            let v = (x * 255 / (w - 1)) as u8;
            Rgb([v, v, v])
        } else {
            Rgb([0, 0, 0])
        }
    });
    DynamicImage::ImageRgb8(img)
}

/// A hard-edged opaque red shape whose transparent surround carries a
/// maximally contrasting green ("dirty alpha", the classic halo trigger — the
/// same construction SPEC-120's probe used).
fn dirty_alpha_disc(n: u32) -> DynamicImage {
    let c = n as f32 / 2.0;
    let r = n as f32 * 0.35;
    let img: RgbaImage = ImageBuffer::from_fn(n, n, |x, y| {
        let dx = x as f32 - c;
        let dy = y as f32 - c;
        if dx * dx + dy * dy <= r * r {
            Rgba([255, 0, 0, 255])
        } else {
            Rgba([0, 255, 0, 0])
        }
    });
    DynamicImage::ImageRgba8(img)
}

/// Constant RGB, rows alternating fully transparent and fully opaque. A 2:1
/// vertical downscale gives every output pixel exactly half coverage.
fn alpha_stripes(w: u32, h: u32) -> DynamicImage {
    let img: RgbaImage = ImageBuffer::from_fn(w, h, |_, y| {
        Rgba([200, 100, 50, if y % 2 == 0 { 0 } else { 255 }])
    });
    DynamicImage::ImageRgba8(img)
}

fn resize_exact(src: &Path, out: &Path, w: u32, h: u32) {
    let output = Command::new(BIN)
        .args([
            "resize",
            src.to_str().unwrap(),
            "--exact",
            &format!("{w}x{h}"),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("run binary");
    assert!(
        output.status.success(),
        "resize failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Rows within `margin` of the top or bottom edge, where the kernel is clipped
/// against the image border and the exact parity symmetry no longer holds.
const EDGE_MARGIN: u32 = 4;

// ── AC-1: the perceptual oracle, against an analytic reference ──────────────

#[test]
fn downscale_scores_better_against_an_independent_reference() {
    let dir = tempfile::tempdir().unwrap();
    let (w, h) = (256u32, 512u32);
    let src = write_png(dir.path(), "src.png", ramp_stripes(w, h));
    let out = dir.path().join("out.png");
    resize_exact(&src, &out, w, h / 2);

    let got = ::image::open(&out).unwrap().to_rgb8();
    let interior = |img: &RgbImage| {
        DynamicImage::ImageRgb8(
            ::image::imageops::crop_imm(img, 0, EDGE_MARGIN, w, h / 2 - 2 * EDGE_MARGIN).to_image(),
        )
    };

    // The reference: what a correct linear-light 2:1 average of (ramp, black)
    // produces, computed from the standard's transfer function alone.
    let reference: RgbImage = ImageBuffer::from_fn(w, h / 2, |x, _| {
        let v = linear_mean((x * 255 / (w - 1)) as u8, 0).round() as u8;
        Rgb([v, v, v])
    });
    // What `main` produced: the same average taken in sRGB space.
    let defect: RgbImage = ImageBuffer::from_fn(w, h / 2, |x, _| {
        let v = signal_mean((x * 255 / (w - 1)) as u8, 0).round() as u8;
        Rgb([v, v, v])
    });

    let score = crustyimg::quality::score(&interior(&reference), &interior(&got)).unwrap();
    let defect_score =
        crustyimg::quality::score(&interior(&reference), &interior(&defect)).unwrap();

    // The defect scores are the point of the control: without it, "the output
    // scores well" could just mean the metric cannot see this at all.
    assert!(
        defect_score < 90.0,
        "control: an sRGB-space average must score badly against the \
         linear-light reference, got {defect_score:.2}"
    );
    assert!(
        score > 95.0,
        "linear-light resize should score near the reference, got {score:.2} \
         (an sRGB-space average scores {defect_score:.2})"
    );
}

// ── AC-2: the physical quantity ─────────────────────────────────────────────

#[test]
fn mean_luminance_error_moves_toward_zero() {
    let dir = tempfile::tempdir().unwrap();
    let (w, h) = (256u32, 512u32);
    let src = write_png(dir.path(), "src.png", ramp_stripes(w, h));
    let out = dir.path().join("out.png");
    resize_exact(&src, &out, w, h / 2);
    let got = ::image::open(&out).unwrap().to_rgb8();

    // Mean *linear* luminance error against the analytic reference, over the
    // interior. The channels are equal here, so luminance is the channel.
    let mut sum_err = 0.0f64;
    let mut sum_ref = 0.0f64;
    let mut max_abs = 0.0f64;
    let mut n = 0u32;
    let mut worst_defect = 0.0f64;
    for y in EDGE_MARGIN..(h / 2 - EDGE_MARGIN) {
        for x in 0..w {
            let source = (x * 255 / (w - 1)) as u8;
            let want = to_linear(linear_mean(source, 0).round() / 255.0);
            let have = to_linear(got.get_pixel(x, y).0[0] as f64 / 255.0);
            let defect = to_linear(signal_mean(source, 0).round() / 255.0);
            sum_err += have - want;
            sum_ref += want;
            max_abs = max_abs.max((have - want).abs());
            worst_defect = worst_defect.max((defect - want).abs());
            n += 1;
        }
    }
    let mean_signed = sum_err / n as f64;
    let mean_ref = sum_ref / n as f64;
    let pct = mean_signed / mean_ref * 100.0;

    // The control: the sRGB-space average this replaced is far from the
    // reference, so a small error here is a real result, not a loose bound.
    assert!(
        worst_defect > 0.05,
        "control: the sRGB-space average should be far from the reference, \
         max |linear err| was only {worst_defect:.6}"
    );
    assert!(
        pct.abs() < 0.5,
        "mean signed linear-luminance error should be near zero, got \
         {mean_signed:+.6} ({pct:+.3}% of the reference mean; max |err| {max_abs:.6})"
    );
}

// ── AC-5: alpha behaviour did not move ──────────────────────────────────────

#[test]
fn translucent_edge_error_is_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_png(dir.path(), "disc.png", dirty_alpha_disc(256));
    let out = dir.path().join("out.png");
    resize_exact(&src, &out, 64, 64);
    let got = ::image::open(&out).unwrap().to_rgba8();

    // With premultiplication on, a fully transparent source pixel contributes
    // nothing, so no green can reach the composite: premultiplied green must
    // be ~0 across the anti-aliased edge band. Without it, the transparent
    // surround's green bleeds in — the classic halo.
    let mut band = 0u32;
    let mut max_premul_green = 0u32;
    for p in got.pixels() {
        let [_, g, _, a] = p.0;
        if a == 0 || a == 255 {
            continue;
        }
        band += 1;
        max_premul_green = max_premul_green.max((g as u32 * a as u32 + 127) / 255);
    }
    assert!(
        band > 100,
        "the edge band must exist for this to test anything, got {band} pixels"
    );

    // The control: the same downscale with alpha ignored (a plain box average
    // of straight RGB), computed here, shows the halo the assertion forbids.
    let source = ::image::open(&src).unwrap().to_rgba8();
    let mut control_max = 0u32;
    for by in 0..64u32 {
        for bx in 0..64u32 {
            let (mut g, mut a) = (0u32, 0u32);
            for dy in 0..4 {
                for dx in 0..4 {
                    let p = source.get_pixel(bx * 4 + dx, by * 4 + dy).0;
                    g += p[1] as u32;
                    a += p[3] as u32;
                }
            }
            let (g, a) = (g / 16, a / 16);
            if a == 0 || a == 255 {
                continue;
            }
            control_max = control_max.max((g * a + 127) / 255);
        }
    }
    assert!(
        control_max > 20,
        "control: ignoring alpha must produce a visible green halo, got \
         max premultiplied green {control_max}"
    );
    assert!(
        max_premul_green <= 2,
        "transparent green must not bleed into the composite: max \
         premultiplied green {max_premul_green} over {band} edge pixels \
         (alpha-ignoring control reaches {control_max})"
    );
}

#[test]
fn resize_does_not_apply_the_transfer_function_to_alpha() {
    let dir = tempfile::tempdir().unwrap();
    let (w, h) = (64u32, 512u32);
    let src = write_png(dir.path(), "src.png", alpha_stripes(w, h));
    let out = dir.path().join("out.png");
    resize_exact(&src, &out, w, h / 2);
    let got = ::image::open(&out).unwrap().to_rgba8();

    // Alpha is coverage, not a light value, so half coverage is 127.5 → 128.
    // Putting it through the sRGB curve as well would give
    // `to_signal(0.5) * 255` ≈ 188, which is what this rules out.
    let mangled = (to_signal(0.5) * 255.0).round() as u8;
    for y in EDGE_MARGIN..(h / 2 - EDGE_MARGIN) {
        for x in 0..w {
            let a = got.get_pixel(x, y).0[3];
            assert!(
                (127..=128).contains(&a),
                "half coverage should stay half: alpha {a} at ({x},{y}) \
                 (a gamma-mangled alpha would be ~{mangled})"
            );
        }
    }
}

// ── AC-6: the paths that must not move ──────────────────────────────────────

/// AC-6 pairs "upscale" with "no-op resize" as both being unaffected. Only the
/// no-op half holds, and it is asserted here at four colour types. The upscale
/// half is a measured deviation — see `upscale_is_resampled_in_linear_light_too`.
#[test]
fn noop_resize_is_byte_identical_to_its_source() {
    let dir = tempfile::tempdir().unwrap();
    let cases: [(&str, DynamicImage); 4] = [
        ("rgb8", ramp_stripes(64, 64)),
        ("rgba8", alpha_stripes(64, 64)),
        (
            "luma8",
            DynamicImage::ImageLuma8(ImageBuffer::from_fn(64, 64, |x, y| {
                ::image::Luma([((x * 7 + y * 5) % 256) as u8])
            })),
        ),
        (
            "rgb16",
            DynamicImage::ImageRgb16(ImageBuffer::from_fn(64, 64, |x, y| {
                ::image::Rgb([
                    (x * 1000 % 65536) as u16,
                    (y * 1000 % 65536) as u16,
                    ((x + y) * 700 % 65536) as u16,
                ])
            })),
        ),
    ];
    for (name, img) in cases {
        let src = write_png(dir.path(), &format!("{name}.png"), img);
        let out = dir.path().join(format!("{name}_out.png"));
        resize_exact(&src, &out, 64, 64);
        assert_eq!(
            ::image::open(&src).unwrap(),
            ::image::open(&out).unwrap(),
            "a same-size resize must return the source samples untouched ({name})"
        );
    }
}

/// The other half of AC-6, pinned deliberately rather than left to chance.
///
/// An upscale **is** a resample — Lanczos3 interpolates, and interpolating
/// non-linear samples is wrong in the same way averaging them is — so the
/// linearization applies there too and the bytes necessarily change. Measured
/// against the same independent reference the spec uses, upscaling improved
/// from 65.93 to 100.00 SSIMULACRA2 on `graphic_large.png` (512²→1024²) and
/// from 89.16 to 98.44 on `photo_forest_cc0.jpg` (800×532→1600×1064). This
/// test holds the behaviour in place; the deviation is recorded in the spec's
/// Build Completion and in DEC-095.
#[test]
fn upscale_is_resampled_in_linear_light_too() {
    let dir = tempfile::tempdir().unwrap();
    let (w, h) = (64u32, 64u32);
    let src = write_png(dir.path(), "src.png", ramp_stripes(w, h));
    let out = dir.path().join("out.png");
    resize_exact(&src, &out, w, h * 2);
    let got = ::image::open(&out).unwrap().to_rgb8();
    let source = ::image::open(&src).unwrap().to_rgb8();

    // A resample conserves the mean of whichever quantity it averages. Lanczos3
    // reconstruction of alternating rows does not blur them to a midpoint — it
    // reproduces the alternation with overshoot — but the *pair* mean is
    // conserved, so the whole-image mean is the thing to read. Measured at
    // column 63 (values 255/0): this build returns 238/106, whose mean linear
    // luminance is 0.4915 against the source's 0.5; `main` returned 218/37,
    // whose mean sRGB value is exactly the source's 127.5 and whose mean
    // linear luminance is 0.3526 — 29% low.
    let mean = |img: &RgbImage, y0: u32, y1: u32, f: &dyn Fn(f64) -> f64| {
        let mut sum = 0.0f64;
        let mut n = 0u32;
        for y in y0..y1 {
            for x in 0..w {
                sum += f(img.get_pixel(x, y).0[0] as f64 / 255.0);
                n += 1;
            }
        }
        sum / n as f64
    };
    let identity = |v: f64| v;
    let src_linear = mean(&source, EDGE_MARGIN, h - EDGE_MARGIN, &to_linear);
    let out_linear = mean(&got, 2 * EDGE_MARGIN, h * 2 - 2 * EDGE_MARGIN, &to_linear);
    let src_signal = mean(&source, EDGE_MARGIN, h - EDGE_MARGIN, &identity);
    let out_signal = mean(&got, 2 * EDGE_MARGIN, h * 2 - 2 * EDGE_MARGIN, &identity);

    let linear_err = (out_linear - src_linear).abs() / src_linear;
    let signal_err = (out_signal - src_signal).abs() / src_signal;
    assert!(
        linear_err < 0.05,
        "an upscale in linear light conserves mean linear luminance: \
         source {src_linear:.4} vs output {out_linear:.4} ({:+.1}%)",
        (out_linear - src_linear) / src_linear * 100.0
    );
    // The discriminator: `main` conserved the mean *sRGB* value instead, which
    // is what reverting the linearization would restore.
    assert!(
        signal_err > 0.10,
        "an sRGB-space upscale would conserve the mean sRGB value instead: \
         source {src_signal:.4} vs output {out_signal:.4} ({:+.1}%)",
        (out_signal - src_signal) / src_signal * 100.0
    );
}
