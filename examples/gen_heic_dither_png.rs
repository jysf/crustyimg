//! Dev-only helper (SPEC-115): write a dithered grey-level PNG so an
//! independent tool (`heif-enc`, system libheif) can encode it to HEIC for the
//! `tests/fixtures/heic` noise-preview candidate search. Not itself a fixture
//! generator — see `tests/fixtures/heic/RECIPES.md` (if a candidate is found)
//! for the committed encode command.

use image::{Rgb, RgbImage};

fn base_value(x: u32, y: u32, w: u32, h: u32) -> f64 {
    let fx = x as f64 / w.max(1) as f64;
    let fy = y as f64 / h.max(1) as f64;
    let wave = (fx * std::f64::consts::TAU * 3.0).sin() * (fy * std::f64::consts::TAU * 2.0).cos();
    0.5 + 0.35 * wave
}

fn dither(w: u32, h: u32, levels: u32) -> RgbImage {
    let mut buf: Vec<f64> = (0..(w * h))
        .map(|i| {
            let (x, y) = (i % w, i / w);
            base_value(x, y, w, h) * 255.0
        })
        .collect();
    let step = 255.0 / (levels.max(2) - 1) as f64;
    let mut img = RgbImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let old = buf[idx].clamp(0.0, 255.0);
            let level = (old / step).round();
            let new = level * step;
            let err = old - new;
            let v = new.round() as u8;
            img.put_pixel(x, y, Rgb([v, v, v]));
            if x + 1 < w {
                buf[idx + 1] += err * 7.0 / 16.0;
            }
            if y + 1 < h {
                if x > 0 {
                    buf[idx + w as usize - 1] += err * 3.0 / 16.0;
                }
                buf[idx + w as usize] += err * 5.0 / 16.0;
                if x + 1 < w {
                    buf[idx + w as usize + 1] += err * 1.0 / 16.0;
                }
            }
        }
    }
    img
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let w: u32 = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(128);
    let h: u32 = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(96);
    let levels: u32 = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(16);
    let out = args
        .get(4)
        .cloned()
        .unwrap_or_else(|| "/tmp/heic_dither.png".to_string());
    let img = dither(w, h, levels);
    img.save(&out).expect("save png");
    eprintln!("wrote {out} ({w}x{h}, {levels} levels)");
}
