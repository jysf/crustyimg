//! Perceptual quality metric + auto-quality search (SPEC-016, DEC-019).
//!
//! This module is a **self-contained pixel+metric unit** — like `src/operation`,
//! it depends only on `::image`, the `ssimulacra2` metric crate, `thiserror`, and
//! `std`. It must NOT depend on `clap`, `crate::cli`, `crate::sink`, files, or
//! terminals.
//!
//! It provides two things:
//! 1. [`score`] — the **SSIMULACRA2** perceptual similarity score between a
//!    reference image and a distorted candidate of the same dimensions (higher =
//!    more similar; ~100 = visually identical).
//! 2. [`search_jpeg_quality`] — a **generic binary search** for the lowest JPEG
//!    encoder quality whose candidate scores at/above a target, plus the
//!    production wiring [`auto_jpeg_quality`] that scores real JPEG round-trips.
//!
//! The search is generic over the per-quality scorer (`FnMut(u8) -> Result<f64,
//! _>`) so it is deterministically unit-testable AND reusable by later specs
//! (the `--max-size` byte budget, AVIF/WebP quality). The original image is
//! decoded once by the pipeline; this module re-encodes/decodes **candidates in
//! memory only** (capped iteration count, no per-candidate disk — DEC-002).

use std::collections::BTreeMap;
use std::io::Cursor;

use ::image::DynamicImage;
use ssimulacra2::{compute_frame_ssimulacra2, ColorPrimaries, Rgb, TransferCharacteristic};

// ── Policy constants (DEC-019) ────────────────────────────────────────────────

/// The lowest JPEG quality the search will consider.
pub const MIN_SEARCH_QUALITY: u8 = 1;
/// The highest JPEG quality the search will consider.
pub const MAX_SEARCH_QUALITY: u8 = 100;
/// The maximum number of distinct candidate evaluations per search (DEC-019).
/// Binary search over 1..=100 needs ~7; 8 is a safe cap that keeps the search
/// sub-second.
pub const MAX_SEARCH_ITERS: u8 = 8;

// ── Errors ────────────────────────────────────────────────────────────────────

/// Errors from perceptual scoring or the quality search (DEC-007 style: typed,
/// matchable). The binary maps these to exit code 1 (generic runtime error).
#[derive(Debug, thiserror::Error)]
pub enum QualityError {
    /// The SSIMULACRA2 computation itself failed (e.g. an image too small for the
    /// metric's internal downscaling, or a dimension mismatch).
    #[error("perceptual scoring failed: {0}")]
    Score(String),

    /// Converting an image into the metric's `Rgb` input failed.
    #[error("image conversion for scoring failed: {0}")]
    Convert(String),

    /// Encoding or decoding a candidate during the quality search failed.
    #[error("could not encode/decode candidate during quality search: {0}")]
    Encode(String),
}

// ── Scoring ───────────────────────────────────────────────────────────────────

/// Convert a decoded image to the `ssimulacra2::Rgb` input: 8-bit sRGB pixels
/// normalized to `0.0..=1.0`, tagged sRGB transfer + BT.709 primaries (DEC-019).
fn to_ss_rgb(img: &DynamicImage) -> Result<Rgb, QualityError> {
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let data: Vec<[f32; 3]> = rgb
        .pixels()
        .map(|p| {
            [
                p[0] as f32 / 255.0,
                p[1] as f32 / 255.0,
                p[2] as f32 / 255.0,
            ]
        })
        .collect();
    Rgb::new(
        data,
        w as usize,
        h as usize,
        TransferCharacteristic::SRGB,
        ColorPrimaries::BT709,
    )
    .map_err(|e| QualityError::Convert(e.to_string()))
}

/// Compute the SSIMULACRA2 score between `reference` and `candidate`.
///
/// Higher is better; ~100 means visually identical. The two images MUST have the
/// same dimensions (in practice `candidate` is `reference` round-tripped through
/// an encoder, so they do). Both are converted to `ssimulacra2::Rgb` and scored
/// via `compute_frame_ssimulacra2`.
pub fn score(reference: &DynamicImage, candidate: &DynamicImage) -> Result<f64, QualityError> {
    let reference_rgb = to_ss_rgb(reference)?;
    let candidate_rgb = to_ss_rgb(candidate)?;
    compute_frame_ssimulacra2(reference_rgb, candidate_rgb)
        .map_err(|e| QualityError::Score(e.to_string()))
}

// ── Search configuration + result ─────────────────────────────────────────────

/// Configuration for a quality search: the target score and the search bounds.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// The SSIMULACRA2 score the chosen quality must reach (≥).
    pub target: f64,
    /// The lowest quality to consider.
    pub min_quality: u8,
    /// The highest quality to consider.
    pub max_quality: u8,
    /// The cap on distinct candidate evaluations.
    pub max_iters: u8,
}

impl SearchConfig {
    /// A config targeting `target`, with the DEC-019 default bounds (1..=100) and
    /// iteration cap (8).
    pub fn for_target(target: f64) -> Self {
        SearchConfig {
            target,
            min_quality: MIN_SEARCH_QUALITY,
            max_quality: MAX_SEARCH_QUALITY,
            max_iters: MAX_SEARCH_ITERS,
        }
    }
}

/// The outcome of a quality search.
#[derive(Debug, Clone, Copy)]
pub struct QualityChoice {
    /// The chosen JPEG encoder quality.
    pub quality: u8,
    /// The SSIMULACRA2 score at the chosen quality (NaN if the target was
    /// unreachable and `max_quality` was never scored).
    pub score: f64,
    /// How many distinct candidate evaluations the search performed.
    pub iterations: u8,
    /// Whether a quality meeting the target was found. `false` means the result
    /// is the best-effort highest quality.
    pub met_target: bool,
}

// ── The generic search ────────────────────────────────────────────────────────

/// Binary-search the integer quality range for the **lowest** quality whose
/// score is ≥ `cfg.target`.
///
/// Generic over the per-quality scorer so the loop is deterministically testable
/// and reusable (DEC-019). Scores are memoized; the search performs at most
/// `cfg.max_iters` distinct `score_at` calls. If no quality in range meets the
/// target, returns the best-effort `cfg.max_quality` with `met_target = false`
/// (never an error for "unreachable target"). A real `score_at` error is
/// propagated unchanged.
pub fn search_jpeg_quality<F>(
    mut score_at: F,
    cfg: &SearchConfig,
) -> Result<QualityChoice, QualityError>
where
    F: FnMut(u8) -> Result<f64, QualityError>,
{
    let mut lo = cfg.min_quality;
    let mut hi = cfg.max_quality;
    let mut best: Option<(u8, f64)> = None; // lowest quality meeting the target
    let mut last: (u8, f64) = (cfg.max_quality, f64::NAN); // most recent evaluation
    let mut cache: BTreeMap<u8, f64> = BTreeMap::new();
    let mut iters: u8 = 0;

    while lo <= hi && iters < cfg.max_iters {
        let mid = lo + (hi - lo) / 2;
        let s = match cache.get(&mid) {
            Some(&s) => s,
            None => {
                iters += 1;
                let s = score_at(mid)?;
                cache.insert(mid, s);
                s
            }
        };
        last = (mid, s);

        if s >= cfg.target {
            best = Some((mid, s));
            // Try lower qualities for a smaller file; stop if we are at the floor.
            if mid == cfg.min_quality {
                break;
            }
            hi = mid - 1;
        } else {
            // Need higher quality; stop if we are at the ceiling.
            if mid == cfg.max_quality {
                break;
            }
            lo = mid + 1;
        }
    }

    Ok(match best {
        Some((quality, score)) => QualityChoice {
            quality,
            score,
            iterations: iters,
            met_target: true,
        },
        None => QualityChoice {
            // No tested quality met the target → best effort is the highest
            // quality. Report the most recent score for context (it may be the
            // max-quality score, or NaN if max was never scored).
            quality: cfg.max_quality,
            score: if last.0 == cfg.max_quality {
                last.1
            } else {
                f64::NAN
            },
            iterations: iters,
            met_target: false,
        },
    })
}

// ── Production wiring (real JPEG round-trip scoring) ──────────────────────────

/// Encode `reference` to JPEG at `quality`, decode it back, and score the
/// round-trip against `reference` — the real per-quality scorer the search uses.
fn score_jpeg_at(reference: &DynamicImage, quality: u8) -> Result<f64, QualityError> {
    // Mirror the DEC-016 JPEG encode path (`encode_to_bytes`): clamp 1..=100.
    let q = quality.clamp(1, 100);
    let mut cursor = Cursor::new(Vec::new());
    let encoder = ::image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, q);
    reference
        .write_with_encoder(encoder)
        .map_err(|e| QualityError::Encode(e.to_string()))?;
    let decoded = ::image::load_from_memory(&cursor.into_inner())
        .map_err(|e| QualityError::Encode(e.to_string()))?;
    score(reference, &decoded)
}

/// Find the lowest JPEG quality whose decoded round-trip scores ≥ `cfg.target`
/// for `reference` (the production entry point for `shrink`'s auto-quality).
pub fn auto_jpeg_quality(
    reference: &DynamicImage,
    cfg: &SearchConfig,
) -> Result<QualityChoice, QualityError> {
    search_jpeg_quality(|q| score_jpeg_at(reference, q), cfg)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    use ::image::{ImageFormat, RgbImage};

    /// A deterministic STRUCTURED image: a smooth gradient + a mild 8px checker
    /// texture (DEC-019 / SPEC-016 fixture). Compresses to a high score at high
    /// quality yet degrades cleanly at low quality — neither flat (which would
    /// score ~100 everywhere) nor pure noise (which JPEG can't reach a high
    /// score on at all).
    fn detailed_rgb(w: u32, h: u32) -> DynamicImage {
        let mut img = RgbImage::new(w, h);
        for (x, y, px) in img.enumerate_pixels_mut() {
            let gx = (x * 255 / w.max(1)) as i32;
            let gy = (y * 255 / h.max(1)) as i32;
            let tex = if ((x / 8) + (y / 8)) % 2 == 0 { 30 } else { 0 };
            let r = (gx + tex).clamp(0, 255) as u8;
            let g = (gy + tex).clamp(0, 255) as u8;
            let b = ((gx + gy) / 2).clamp(0, 255) as u8;
            *px = ::image::Rgb([r, g, b]);
        }
        DynamicImage::ImageRgb8(img)
    }

    /// Encode a `DynamicImage` to JPEG bytes at a given quality (test helper).
    fn jpeg_at(img: &DynamicImage, q: u8) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        let encoder = ::image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, q);
        img.write_with_encoder(encoder).unwrap();
        cursor.into_inner()
    }

    #[test]
    fn score_identical_is_high() {
        let img = detailed_rgb(96, 96);
        let s = score(&img, &img).expect("scoring identical images should succeed");
        assert!(s > 90.0, "identical images should score ~100, got {s}");
    }

    #[test]
    fn score_degraded_is_lower() {
        let img = detailed_rgb(96, 96);
        let identity = score(&img, &img).expect("identity score");
        // Heavily degrade via a quality-8 JPEG round-trip.
        let bytes = jpeg_at(&img, 8);
        let decoded =
            ::image::load_from_memory_with_format(&bytes, ImageFormat::Jpeg).expect("decode q8");
        let degraded = score(&img, &decoded).expect("degraded score");
        assert!(
            degraded < identity,
            "degraded ({degraded}) should score below identity ({identity})"
        );
        assert!(
            degraded < 90.0,
            "a quality-8 round-trip on a detailed image should score below 90, got {degraded}"
        );
    }

    #[test]
    fn search_finds_lowest_meeting_target() {
        // Synthetic monotonic scorer: score == quality. Lowest q with q >= 50 is 50.
        let calls = Cell::new(0u8);
        let cfg = SearchConfig::for_target(50.0);
        let choice = search_jpeg_quality(
            |q| {
                calls.set(calls.get() + 1);
                Ok(q as f64)
            },
            &cfg,
        )
        .expect("search should succeed");
        assert_eq!(choice.quality, 50, "lowest quality meeting target 50 is 50");
        assert!(choice.met_target, "target 50 is reachable");
        assert!(
            calls.get() <= cfg.max_iters,
            "scorer called {} times, exceeds cap {}",
            calls.get(),
            cfg.max_iters
        );
        assert!(
            choice.iterations <= cfg.max_iters,
            "iterations {} exceeds cap {}",
            choice.iterations,
            cfg.max_iters
        );
    }

    #[test]
    fn search_unreachable_target_is_best_effort() {
        // Scorer always below the target → no quality meets it.
        let cfg = SearchConfig::for_target(90.0);
        let choice = search_jpeg_quality(|_q| Ok(10.0), &cfg).expect("search should succeed");
        assert_eq!(
            choice.quality, MAX_SEARCH_QUALITY,
            "unreachable target → best-effort highest quality"
        );
        assert!(!choice.met_target, "target was not met");
    }

    #[test]
    fn search_propagates_scorer_error() {
        let cfg = SearchConfig::for_target(50.0);
        let result = search_jpeg_quality(|_q| Err(QualityError::Encode("boom".into())), &cfg);
        assert!(
            matches!(result, Err(QualityError::Encode(ref m)) if m == "boom"),
            "a scorer error must propagate, got {result:?}"
        );
    }

    #[test]
    fn auto_jpeg_quality_is_monotone_in_target() {
        let img = detailed_rgb(96, 96);
        let lo = auto_jpeg_quality(&img, &SearchConfig::for_target(50.0)).expect("lo search");
        let hi = auto_jpeg_quality(&img, &SearchConfig::for_target(90.0)).expect("hi search");
        assert!(
            lo.quality <= hi.quality,
            "lower target should pick a lower-or-equal quality: target50 q={} vs target90 q={}",
            lo.quality,
            hi.quality
        );
    }

    #[test]
    fn search_config_defaults_match_dec019() {
        let cfg = SearchConfig::for_target(90.0);
        assert_eq!(cfg.target, 90.0);
        assert_eq!(cfg.min_quality, 1);
        assert_eq!(cfg.max_quality, 100);
        assert_eq!(cfg.max_iters, 8);
    }
}
