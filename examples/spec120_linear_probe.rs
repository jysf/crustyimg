//! SPEC-120 measurement probe — **throwaway prototype, not production code.**
//!
//! This example exists to answer one question: does resampling in linear light
//! measure better than resampling the non-linear sRGB values, and can
//! SSIMULACRA2 see the difference? It ships **no behaviour**; nothing in
//! `src/` calls it and nothing in `src/` changed for it.
//!
//! It is deliberately a *replica* of the production resize path rather than a
//! call into it, so the sRGB and linear arms differ in exactly one variable —
//! the transfer function — while sharing the same `fast_image_resize` version,
//! filter and options that `Resize::apply` uses. `mode srgb` is the control:
//! its output must be byte-identical to `crustyimg resize --exact`, which the
//! driver script asserts.
//!
//! Subcommands:
//!
//! ```text
//! synth        <out.png>                       thin bright features on black, 2048x2048
//! synth-alpha  <out.png>                       hard transparent edges over dirty RGB, 1024x1024
//! topng        <in> <out.png>                  decode anything, write RGBA8 PNG
//! resize       <in.png> <out.png> <W> <H> <mode>
//!                                              mode = srgb | linear | srgb-nopremul
//! luma         <ref.png> <cand.png>            linear-luminance error, JSON
//! alpha-edge   <ref.png> <cand.png>            premultiplied edge error, JSON
//! pixdiff      <a.png> <b.png>                 exact RGBA agreement, JSON
//! ```

use std::process::ExitCode;

use fast_image_resize as fir;
use image::RgbaImage;

// ── sRGB transfer function (IEC 61966-2-1) ───────────────────────────────────

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.040_448_237 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Relative luminance of a *linear* RGB triple (BT.709 primaries, the same
/// primaries `src/quality` tags its SSIMULACRA2 input with).
fn luminance(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

// ── Synthetic sources ────────────────────────────────────────────────────────

/// The Call-3 positive control: 2048x2048, pure white 1px features on pure
/// black, in four regimes so the 8x downscale still has structure and is not a
/// single flat field. Fully deterministic — no RNG, no input files.
fn synth_worst_case() -> RgbaImage {
    const N: u32 = 2048;
    let half = N / 2;
    let (cx, cy) = ((N as f32) * 0.75, (N as f32) * 0.75);
    RgbaImage::from_fn(N, N, |x, y| {
        let lit = match (x < half, y < half) {
            // top-left: horizontal hairlines, period 8 (aligns with the 8x downscale)
            (true, true) => y.is_multiple_of(8),
            // top-right: vertical hairlines, period 6 (deliberately not aligned)
            (false, true) => x.is_multiple_of(6),
            // bottom-left: diagonal hairlines
            (true, false) => (x + y).is_multiple_of(10),
            // bottom-right: concentric hairline rings
            (false, false) => {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                ((dx * dx + dy * dy).sqrt() as u32).is_multiple_of(12)
            }
        };
        if lit {
            image::Rgba([255, 255, 255, 255])
        } else {
            image::Rgba([0, 0, 0, 255])
        }
    })
}

/// The Call-5 source: a hard-edged opaque shape whose transparent surround
/// carries a maximally contrasting RGB ("dirty alpha", the classic halo
/// trigger). 1024x1024, downscaled 8x by the driver.
fn synth_alpha_case() -> RgbaImage {
    const N: u32 = 1024;
    let c = (N as f32) / 2.0;
    let r = (N as f32) * 0.35;
    RgbaImage::from_fn(N, N, |x, y| {
        let dx = x as f32 - c;
        let dy = y as f32 - c;
        let inside_disc = dx * dx + dy * dy <= r * r;
        // Thin opaque spokes so the edge band is long relative to the area.
        let spoke = (x % 64 < 3 || y % 64 < 3) && dx * dx + dy * dy <= (r * 1.6) * (r * 1.6);
        if inside_disc || spoke {
            image::Rgba([255, 0, 0, 255]) // opaque red
        } else {
            image::Rgba([0, 255, 0, 0]) // fully transparent, bright green underneath
        }
    })
}

// ── Resize arms ──────────────────────────────────────────────────────────────

/// Byte-for-byte replica of `Resize::apply`'s backend call (src/operation/mod.rs):
/// RGBA8 -> `PixelType::U8x4` -> `Resizer` with `ResizeOptions::new()` (which
/// leaves `mul_div_alpha` at its `true` default) and Lanczos3.
fn resize_srgb(rgba: &RgbaImage, dw: u32, dh: u32, use_alpha: bool) -> RgbaImage {
    let (w, h) = rgba.dimensions();
    let src =
        fir::images::Image::from_vec_u8(w, h, rgba.as_raw().clone(), fir::PixelType::U8x4).unwrap();
    let mut dst = fir::images::Image::new(dw, dh, fir::PixelType::U8x4);
    let opts = fir::ResizeOptions::new()
        .resize_alg(fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3))
        .use_alpha(use_alpha);
    fir::Resizer::new().resize(&src, &mut dst, &opts).unwrap();
    RgbaImage::from_raw(dw, dh, dst.into_vec()).unwrap()
}

/// The prototype: the same backend, filter and options, with the pixels
/// linearized to f32 first and re-encoded to sRGB on the way out.
fn resize_linear(rgba: &RgbaImage, dw: u32, dh: u32) -> RgbaImage {
    let (w, h) = rgba.dimensions();
    let mut lin: Vec<f32> = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for p in rgba.pixels() {
        lin.push(srgb_to_linear(p[0] as f32 / 255.0));
        lin.push(srgb_to_linear(p[1] as f32 / 255.0));
        lin.push(srgb_to_linear(p[2] as f32 / 255.0));
        lin.push(p[3] as f32 / 255.0); // alpha is already linear coverage
    }
    let bytes: Vec<u8> = lin.iter().flat_map(|v| v.to_ne_bytes()).collect();
    let src = fir::images::Image::from_vec_u8(w, h, bytes, fir::PixelType::F32x4).unwrap();
    let mut dst = fir::images::Image::new(dw, dh, fir::PixelType::F32x4);
    let opts = fir::ResizeOptions::new()
        .resize_alg(fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3));
    fir::Resizer::new().resize(&src, &mut dst, &opts).unwrap();

    let raw = dst.into_vec();
    let mut out = RgbaImage::new(dw, dh);
    for (i, px) in out.pixels_mut().enumerate() {
        let mut ch = [0u8; 4];
        for (c, slot) in ch.iter_mut().enumerate() {
            let mut b = [0u8; 4];
            b.copy_from_slice(&raw[(i * 4 + c) * 4..(i * 4 + c) * 4 + 4]);
            let v = f32::from_ne_bytes(b).clamp(0.0, 1.0);
            let v = if c == 3 { v } else { linear_to_srgb(v) };
            *slot = (v * 255.0).round() as u8;
        }
        *px = image::Rgba(ch);
    }
    out
}

// ── Metrics ──────────────────────────────────────────────────────────────────

fn load_rgba(path: &str) -> RgbaImage {
    let img = image::open(path).unwrap_or_else(|e| panic!("open {path}: {e}"));
    img.to_rgba8()
}

fn assert_same_dims(a: &RgbaImage, b: &RgbaImage, pa: &str, pb: &str) {
    assert_eq!(
        a.dimensions(),
        b.dimensions(),
        "dimension mismatch: {pa} is {:?}, {pb} is {:?}",
        a.dimensions(),
        b.dimensions()
    );
}

/// Mean/max linear-luminance error of `cand` against `ref`. The **signed** mean
/// is the number the premise predicts: gamma-incorrect downscaling of bright
/// features on dark should come out *darker*, i.e. negative.
fn luma_report(reference: &RgbaImage, cand: &RgbaImage) -> String {
    let mut sum_ref = 0f64;
    let mut sum_cand = 0f64;
    let mut sum_signed = 0f64;
    let mut sum_abs = 0f64;
    let mut max_abs = 0f64;
    let n = (reference.width() as f64) * (reference.height() as f64);
    for (pr, pc) in reference.pixels().zip(cand.pixels()) {
        let yr = luminance(
            srgb_to_linear(pr[0] as f32 / 255.0),
            srgb_to_linear(pr[1] as f32 / 255.0),
            srgb_to_linear(pr[2] as f32 / 255.0),
        ) as f64;
        let yc = luminance(
            srgb_to_linear(pc[0] as f32 / 255.0),
            srgb_to_linear(pc[1] as f32 / 255.0),
            srgb_to_linear(pc[2] as f32 / 255.0),
        ) as f64;
        sum_ref += yr;
        sum_cand += yc;
        sum_signed += yc - yr;
        sum_abs += (yc - yr).abs();
        max_abs = max_abs.max((yc - yr).abs());
    }
    let mean_ref = sum_ref / n;
    let rel = if mean_ref > 0.0 {
        (sum_signed / n) / mean_ref * 100.0
    } else {
        0.0
    };
    format!(
        "{{\"pixels\":{n},\"mean_luma_ref\":{:.6},\"mean_luma_cand\":{:.6},\
\"mean_signed_luma_err\":{:.6},\"mean_abs_luma_err\":{:.6},\"max_abs_luma_err\":{:.6},\
\"mean_signed_luma_err_pct_of_ref\":{:.3}}}",
        mean_ref,
        sum_cand / n,
        sum_signed / n,
        sum_abs / n,
        max_abs,
        rel
    )
}

/// Call-5 oracle. Over the **alpha edge band** (any pixel where either image is
/// partially transparent), report the maximum per-channel difference in
/// *premultiplied* 8-bit RGB. Premultiplied difference is exactly the visible
/// composite error and is background-independent whenever the two alphas agree
/// — which `max_alpha_err` reports, so that assumption is checked, not assumed.
fn alpha_edge_report(reference: &RgbaImage, cand: &RgbaImage) -> String {
    let mut band = 0u64;
    let mut max_premul_err = 0i32;
    let mut sum_premul_err = 0f64;
    let mut max_alpha_err = 0i32;
    let mut sum_alpha_err = 0f64;
    let mut max_straight_err = 0i32;
    for (pr, pc) in reference.pixels().zip(cand.pixels()) {
        let (ar, ac) = (pr[3] as i32, pc[3] as i32);
        max_alpha_err = max_alpha_err.max((ar - ac).abs());
        sum_alpha_err += (ar - ac).abs() as f64;
        let edge = (1..255).contains(&ar) || (1..255).contains(&ac);
        if !edge {
            continue;
        }
        band += 1;
        for c in 0..3 {
            let pr_p = (pr[c] as i32 * ar + 127) / 255;
            let pc_p = (pc[c] as i32 * ac + 127) / 255;
            let d = (pr_p - pc_p).abs();
            max_premul_err = max_premul_err.max(d);
            sum_premul_err += d as f64;
            max_straight_err = max_straight_err.max((pr[c] as i32 - pc[c] as i32).abs());
        }
    }
    let mean = if band > 0 {
        sum_premul_err / (band as f64 * 3.0)
    } else {
        0.0
    };
    let n = (reference.width() as f64) * (reference.height() as f64);
    let mean_alpha_err = sum_alpha_err / n;
    format!(
        "{{\"edge_pixels\":{band},\"max_premul_rgb_err\":{max_premul_err},\
\"mean_premul_rgb_err\":{mean:.4},\"max_straight_rgb_err\":{max_straight_err},\
\"max_alpha_err\":{max_alpha_err},\"mean_alpha_err\":{mean_alpha_err:.4}}}"
    )
}

/// Exact per-channel RGBA agreement between two images. Used for the harness's
/// own controls: the prototype's sRGB arm must reproduce the shipped binary's
/// output exactly, or the linear arm's delta is not attributable to gamma.
fn pixdiff_report(a: &RgbaImage, b: &RgbaImage) -> String {
    let mut max_err = 0i32;
    let mut differing = 0u64;
    for (pa, pb) in a.pixels().zip(b.pixels()) {
        let mut d = 0i32;
        for c in 0..4 {
            d = d.max((pa[c] as i32 - pb[c] as i32).abs());
        }
        if d > 0 {
            differing += 1;
        }
        max_err = max_err.max(d);
    }
    format!(
        "{{\"identical\":{},\"max_abs_rgba_err\":{max_err},\"differing_pixels\":{differing}}}",
        max_err == 0
    )
}

// ── Entry point ──────────────────────────────────────────────────────────────

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  \
         spec120_linear_probe synth <out.png>\n  \
         spec120_linear_probe synth-alpha <out.png>\n  \
         spec120_linear_probe topng <in> <out.png>\n  \
         spec120_linear_probe resize <in.png> <out.png> <W> <H> <srgb|linear|srgb-nopremul>\n  \
         spec120_linear_probe luma <ref.png> <cand.png>\n  \
         spec120_linear_probe alpha-edge <ref.png> <cand.png>\n  \
         spec120_linear_probe pixdiff <a.png> <b.png>"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first().map(String::as_str) else {
        return usage();
    };
    match (cmd, args.len()) {
        ("synth", 2) => synth_worst_case().save(&args[1]).unwrap(),
        ("synth-alpha", 2) => synth_alpha_case().save(&args[1]).unwrap(),
        ("topng", 3) => {
            let img = image::open(&args[1]).unwrap();
            println!("{}x{}", img.width(), img.height());
            img.to_rgba8().save(&args[2]).unwrap();
        }
        ("resize", 6) => {
            let src = load_rgba(&args[1]);
            let dw: u32 = args[3].parse().unwrap();
            let dh: u32 = args[4].parse().unwrap();
            let out = match args[5].as_str() {
                "srgb" => resize_srgb(&src, dw, dh, true),
                "srgb-nopremul" => resize_srgb(&src, dw, dh, false),
                "linear" => resize_linear(&src, dw, dh),
                other => {
                    eprintln!("unknown mode {other}");
                    return usage();
                }
            };
            out.save(&args[2]).unwrap();
        }
        ("luma", 3) => {
            let r = load_rgba(&args[1]);
            let c = load_rgba(&args[2]);
            assert_same_dims(&r, &c, &args[1], &args[2]);
            println!("{}", luma_report(&r, &c));
        }
        ("alpha-edge", 3) => {
            let r = load_rgba(&args[1]);
            let c = load_rgba(&args[2]);
            assert_same_dims(&r, &c, &args[1], &args[2]);
            println!("{}", alpha_edge_report(&r, &c));
        }
        ("pixdiff", 3) => {
            let a = load_rgba(&args[1]);
            let b = load_rgba(&args[2]);
            assert_same_dims(&a, &b, &args[1], &args[2]);
            println!("{}", pixdiff_report(&a, &b));
        }
        _ => return usage(),
    }
    ExitCode::SUCCESS
}
