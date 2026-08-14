//! Regenerate the committed "noise preview" RAW fixture (SPEC-115).
//!
//! Produces `tests/fixtures/raw/noise_preview.nef`: a RAW file whose embedded
//! preview reaches the DECIDE-path passthrough branch (AC-3) — every candidate
//! the auto-decide engine would encode is LARGER than the container, so
//! `pick_winner` returns `None` and `optimize_decide_one` must ship *something*
//! rather than the raw (TIFF-container) bytes under an adopted `jpeg` label.
//!
//! `tight_preview.nef` (SPEC-113) does not reproduce this defect: its preview is
//! genuine high-frequency noise, which classifies `Photograph` (OptBucket::Lossy)
//! — AVIF at the fast fixed quality beats it easily (2995 B < 4073 B container).
//! `oversize_preview.dng` reproduces it but at a pathological 59 MB / 156 MB
//! candidate size (a 62 MP declared preview) — unfit to commit and re-encode on
//! every test run.
//!
//! This searches for a preview that classifies `GraphicLogo`/`Icon`
//! (`few_colors`, `OptBucket::LosslessFlat` — no AVIF/lossy admitted at all,
//! regardless of which features are built) via Floyd–Steinberg dithering to a
//! small grey-level palette (the `dither_32color.png` recipe,
//! `tests/fixtures/classify/RECIPES.md`) — dithering is what keeps the LOSSLESS
//! re-encode large (adversarial to PNG/WebP's predictive filters) while the
//! stored JPEG preview stays small.
//!
//! Ground truth (not prediction): every candidate is round-tripped through the
//! actual `crustyimg` library — `raw_preview`, `Analysis::compute`, and
//! `sink::encode_to_bytes` for both lossless candidates — so the printed sizes
//! are exactly what `optimize_decide_one` would compute.

use image::codecs::jpeg::JpegEncoder;
use image::{ImageFormat as ExtFormat, Rgb, RgbImage};

use crustyimg::analysis::Analysis;
use crustyimg::image::Image;
use crustyimg::sink::encode_to_bytes;

/// `(width, height, grey levels, stored JPEG quality)` candidates, smallest
/// first.
const CANDIDATES: &[(u32, u32, u32, u8)] = &[
    (96, 72, 12, 2),
    (128, 96, 12, 2),
    (160, 120, 12, 2),
    (192, 144, 16, 2),
    (224, 168, 16, 2),
    (256, 192, 16, 2),
    (320, 240, 16, 2),
    (384, 288, 24, 2),
];

/// How much bigger the smaller lossless candidate must be than the container
/// for the fixture to be worth committing (encoder-version drift margin).
const REQUIRED_MARGIN: f64 = 1.10;

/// A smooth, non-uniform synthetic base (0.0..=1.0), concentrated toward
/// mid-tones like a real photograph's histogram — a linear ramp would spread
/// dithered levels ~uniformly and push entropy toward `log2(levels)`, too high
/// to stay in the graphic-classifier's low-entropy lane.
fn base_value(x: u32, y: u32, w: u32, h: u32) -> f64 {
    let fx = x as f64 / w.max(1) as f64;
    let fy = y as f64 / h.max(1) as f64;
    let wave = (fx * std::f64::consts::TAU * 3.0).sin() * (fy * std::f64::consts::TAU * 2.0).cos();
    // Centered near 0.5 with modest excursions — a "soft" tonal range.
    0.5 + 0.35 * wave
}

/// Floyd–Steinberg error diffusion to `levels` evenly spaced grey steps
/// (the `dither_32color.png` recipe: a FIXED ramp, not an adaptive palette, so
/// the source's own tonal distribution sets bin occupancy instead of being
/// driven toward uniform).
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

            // 7/16 - 3/16 - 5/16 - 1/16 kernel, left to right, no serpentine.
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

fn jpeg_at(img: &RgbImage, quality: u8) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(img.clone())
        .write_with_encoder(JpegEncoder::new_with_quality(&mut out, quality))
        .expect("encode jpeg at quality");
    out.into_inner()
}

fn build_blob(preview: &[u8]) -> Vec<u8> {
    let mut blob: Vec<u8> = vec![0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00];
    blob.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    blob.extend_from_slice(preview);
    blob.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);
    blob
}

fn main() {
    eprintln!(
        "  dims       levels q  preview  container  class        entropy colors  png       webp      ratio"
    );
    let mut chosen: Option<(u32, u32, u32, u8, Vec<u8>)> = None;

    for &(w, h, levels, q) in CANDIDATES {
        let dithered = dither(w, h, levels);
        let preview = jpeg_at(&dithered, q);
        let blob = build_blob(&preview);

        // Ground truth: decode the STORED preview exactly like `raw_preview`
        // would (the container routes here by extension in production; here we
        // decode the preview bytes directly, which is what `extract_preview`'s
        // scan converges on for this shape — a single embedded JPEG).
        let decoded = image::load_from_memory(&preview).expect("stored preview must decode");
        let img = Image::from_parts(decoded, ExtFormat::Jpeg, None);
        let analysis = Analysis::compute(&img).expect("non-degenerate");

        let png_bytes = encode_to_bytes(&img, ExtFormat::Png, None)
            .expect("png encode")
            .len();
        let webp_bytes = encode_to_bytes(&img, ExtFormat::WebP, None)
            .expect("webp lossless encode")
            .len();
        let smallest_lossless = png_bytes.min(webp_bytes);
        let ratio = smallest_lossless as f64 / blob.len() as f64;

        eprintln!(
            "  {w:>4}x{h:<4} {levels:>3}    q{q}  {:>6} B  {:>7} B  {:<12} {:>6.2}  {:>4}  {:>8} B {:>8} B  {ratio:.2}x{}",
            preview.len(),
            blob.len(),
            format!("{:?}", analysis.class()),
            analysis.entropy(),
            analysis.unique_colors().count(),
            png_bytes,
            webp_bytes,
            if ratio >= REQUIRED_MARGIN {
                "  <- chosen"
            } else {
                ""
            },
        );

        if ratio >= REQUIRED_MARGIN {
            chosen = Some((w, h, levels, q, blob));
            break;
        }
    }

    let (w, h, levels, q, blob) = chosen.unwrap_or_else(|| {
        panic!(
            "no candidate cleared {REQUIRED_MARGIN}x: the smallest LOSSLESS re-encode must \
             exceed the whole container, else the decide-path passthrough branch is never \
             reached. Widen CANDIDATES."
        )
    });

    let path = "tests/fixtures/raw/noise_preview.nef";
    std::fs::create_dir_all("tests/fixtures/raw").expect("create fixture dir");
    std::fs::write(path, &blob).expect("write fixture");
    eprintln!(
        "\nwrote {path} ({} B) — preview {w}x{h}, {levels} grey levels, q{q}",
        blob.len(),
    );
}
