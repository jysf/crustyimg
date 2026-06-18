//! Criterion micro-benchmarks for crustyimg's hot paths (SPEC-025, DEC-028).
//!
//! Groups: `decode` / `resize` / `encode_jpeg` / `score` / `pipeline`, over an
//! in-memory generated fixture (no committed binary files, mirroring the test
//! fixtures, DEC-009). Dev-only — not part of the shipped binary. Run with
//! `just bench` (`cargo bench`).
//!
//! These measure the real library paths: decode (DEC-002), the resize backend
//! (DEC-008), JPEG quality encode (DEC-016), and the SSIMULACRA2 perceptual metric
//! (DEC-019). Any size/speed claim built on these MUST be gated on equal quality
//! (DEC-028).

use std::collections::BTreeMap;
use std::hint::black_box;
use std::io::Cursor;

use criterion::{criterion_group, criterion_main, Criterion};
use image::{DynamicImage, ImageFormat, RgbImage};

use crustyimg::image::Image;
use crustyimg::operation::{OperationParams, OperationRegistry};
use crustyimg::pipeline::Pipeline;

/// A deterministic detailed RGB image — a smooth gradient plus a mild 8px checker —
/// so the encode/resize/metric paths do realistic work (neither flat nor pure
/// noise). Mirrors the `detailed_rgb` test/bench fixture used elsewhere.
fn detailed_rgb(w: u32, h: u32) -> DynamicImage {
    let mut img = RgbImage::new(w, h);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let gx = (x * 255 / w.max(1)) as i32;
        let gy = (y * 255 / h.max(1)) as i32;
        let tex = if ((x / 8) + (y / 8)) % 2 == 0 { 30 } else { 0 };
        let r = (gx + tex).clamp(0, 255) as u8;
        let g = (gy + tex).clamp(0, 255) as u8;
        let b = ((gx + gy) / 2).clamp(0, 255) as u8;
        *px = image::Rgb([r, g, b]);
    }
    DynamicImage::ImageRgb8(img)
}

/// Encode a `DynamicImage` to JPEG bytes at a quality (fixture helper).
fn jpeg_bytes(img: &DynamicImage, q: u8) -> Vec<u8> {
    let mut c = Cursor::new(Vec::new());
    let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut c, q);
    img.write_with_encoder(enc).expect("encode jpeg fixture");
    c.into_inner()
}

/// `resize max <w>` op params (long-edge bound), built like the CLI does.
fn resize_max_params(width: u32) -> OperationParams {
    let mut m: BTreeMap<String, toml::Value> = BTreeMap::new();
    m.insert("mode".into(), toml::Value::String("max".into()));
    m.insert("width".into(), toml::Value::Integer(width as i64));
    OperationParams::from_map(m)
}

fn hot_paths(c: &mut Criterion) {
    let dim = 256;
    let src = detailed_rgb(dim, dim);
    let jpeg = jpeg_bytes(&src, 90);
    let img = Image::from_bytes(&jpeg).expect("decode fixture");
    // A degraded copy gives the perceptual metric a non-trivial comparison.
    let degraded = image::load_from_memory(&jpeg_bytes(&src, 20)).expect("decode degraded");

    c.bench_function("decode", |b| {
        b.iter(|| Image::from_bytes(black_box(&jpeg)).expect("decode"));
    });

    c.bench_function("resize", |b| {
        b.iter(|| {
            let op = OperationRegistry::with_builtins()
                .build("resize", &resize_max_params(128))
                .expect("build resize");
            let out = Pipeline::new()
                .push(op)
                .run(img.clone())
                .expect("resize run");
            black_box(out)
        });
    });

    c.bench_function("encode_jpeg", |b| {
        b.iter(|| {
            crustyimg::sink::encode_to_bytes(black_box(&img), ImageFormat::Jpeg, Some(80))
                .expect("encode")
        });
    });

    c.bench_function("score", |b| {
        b.iter(|| {
            crustyimg::quality::score(black_box(img.pixels()), black_box(&degraded)).expect("score")
        });
    });

    c.bench_function("pipeline", |b| {
        b.iter(|| {
            let decoded = Image::from_bytes(black_box(&jpeg)).expect("decode");
            let op = OperationRegistry::with_builtins()
                .build("resize", &resize_max_params(128))
                .expect("build resize");
            let out = Pipeline::new().push(op).run(decoded).expect("pipeline run");
            crustyimg::sink::encode_to_bytes(&out, ImageFormat::Jpeg, Some(80)).expect("encode")
        });
    });
}

criterion_group!(benches, hot_paths);
criterion_main!(benches);
