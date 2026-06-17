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
//! 2. [`search_quality`] — a **generic binary search** for the lowest encoder
//!    quality whose candidate scores at/above a target, plus the production
//!    wiring [`auto_quality`] that scores real round-trips for a target format.
//!
//! The search is generic over the per-quality scorer (`FnMut(u8) -> Result<f64,
//! _>`) so it is deterministically unit-testable AND reusable by later specs
//! (the `--max-size` byte budget, AVIF/WebP quality). The original image is
//! decoded once by the pipeline; this module re-encodes/decodes **candidates in
//! memory only** (capped iteration count, no per-candidate disk — DEC-002).

use std::io::Cursor;

use ::image::{DynamicImage, ImageFormat};
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

/// The fixed `rav1e` encode speed for AVIF candidates (SPEC-018, DEC-020).
/// MUST equal `crate::sink::AVIF_SPEED` so a candidate probed here has the same
/// byte length as the bytes the sink ultimately writes — the cross-sync contract
/// (layering forbids `quality` depending on `sink`, so the two consts are kept
/// equal by this comment, not by a shared call). See `encode_candidate_bytes`.
#[cfg(feature = "avif")]
const AVIF_SPEED: u8 = 6;

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

    /// A config for a byte-budget search (SPEC-017): the default bounds (1..=100)
    /// and iteration cap (8). `target` is unused for a size search (the budget is
    /// supplied separately), so it is `NaN`.
    pub fn for_size_budget() -> Self {
        SearchConfig {
            target: f64::NAN,
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
    /// The search metric at the chosen quality — an SSIMULACRA2 score for a
    /// perceptual search ([`search_quality`]), or the encoded byte size for a
    /// size-budget search ([`search_under_size`]). `NaN` when the constraint
    /// was unreachable and the best-effort fallback quality was never probed.
    pub score: f64,
    /// How many distinct candidate evaluations the search performed.
    pub iterations: u8,
    /// Whether a quality meeting the target was found. `false` means the result
    /// is the best-effort highest quality.
    pub met_target: bool,
}

// ── The generic search ────────────────────────────────────────────────────────

/// Binary-search the integer quality range for the boundary quality of a
/// **monotone** predicate — the shared core of both quality searches (SPEC-017).
///
/// `probe(q)` returns the search metric at quality `q` (an SSIMULACRA2 score, or
/// an encoded byte size as `f64`); `accept(metric)` decides whether `q` satisfies
/// the constraint. `prefer_lower` picks which satisfying quality wins when several
/// do:
/// - `true`  → the **lowest** satisfying quality (perceptual: the smallest file
///   whose score still clears the target). The fallback (none satisfy) is
///   `max_quality`.
/// - `false` → the **highest** satisfying quality (size budget: the best quality
///   that still fits). The fallback is `min_quality`.
///
/// At most `cfg.max_iters` `probe` calls. No quality is ever revisited (each step
/// excludes `mid` from `[lo, hi]`), so no memoization is needed. A real `probe`
/// error is propagated unchanged. Never errors for "constraint unreachable" — it
/// returns the best-effort fallback quality with `met_target = false`.
fn search_threshold<F>(
    mut probe: F,
    cfg: &SearchConfig,
    accept: impl Fn(f64) -> bool,
    prefer_lower: bool,
) -> Result<QualityChoice, QualityError>
where
    F: FnMut(u8) -> Result<f64, QualityError>,
{
    let mut lo = cfg.min_quality;
    let mut hi = cfg.max_quality;
    let mut best: Option<(u8, f64)> = None; // the preferred satisfying quality
    let mut last: (u8, f64) = (cfg.max_quality, f64::NAN); // most recent evaluation
    let mut iters: u8 = 0;

    while lo <= hi && iters < cfg.max_iters {
        let mid = lo + (hi - lo) / 2;
        iters += 1;
        let m = probe(mid)?;
        last = (mid, m);

        // On accept, move toward the preferred side to look for a better quality;
        // on reject, move the other way. The u8 boundary guards avoid underflow
        // at `min_quality` / overflow past `max_quality`.
        let go_lower = if accept(m) {
            best = Some((mid, m));
            prefer_lower
        } else {
            !prefer_lower
        };
        if go_lower {
            if mid == cfg.min_quality {
                break;
            }
            hi = mid - 1;
        } else {
            if mid == cfg.max_quality {
                break;
            }
            lo = mid + 1;
        }
    }

    let fallback_q = if prefer_lower {
        cfg.max_quality
    } else {
        cfg.min_quality
    };
    Ok(match best {
        Some((quality, score)) => QualityChoice {
            quality,
            score,
            iterations: iters,
            met_target: true,
        },
        None => QualityChoice {
            // Nothing satisfied the constraint → best effort is the fallback
            // quality. Report its metric if it happened to be the last probed.
            quality: fallback_q,
            score: if last.0 == fallback_q {
                last.1
            } else {
                f64::NAN
            },
            iterations: iters,
            met_target: false,
        },
    })
}

/// Binary-search the integer quality range for the **lowest** quality whose
/// score is ≥ `cfg.target` (the perceptual search, DEC-019). Generic over the
/// scorer so it is deterministically testable. See [`search_threshold`].
pub fn search_quality<F>(score_at: F, cfg: &SearchConfig) -> Result<QualityChoice, QualityError>
where
    F: FnMut(u8) -> Result<f64, QualityError>,
{
    search_threshold(score_at, cfg, |m| m >= cfg.target, true)
}

/// Binary-search the integer quality range for the **highest** quality whose
/// encoded size is ≤ `budget_bytes` (the byte-budget search, SPEC-017). Generic
/// over the size probe so it is deterministically testable. If even the minimum
/// quality exceeds the budget, returns the best-effort `min_quality` with
/// `met_target = false`. See [`search_threshold`].
pub fn search_under_size<F>(
    mut size_at: F,
    budget_bytes: u64,
    cfg: &SearchConfig,
) -> Result<QualityChoice, QualityError>
where
    F: FnMut(u8) -> Result<u64, QualityError>,
{
    search_threshold(
        |q| Ok(size_at(q)? as f64),
        cfg,
        |m| m <= budget_bytes as f64,
        false,
    )
}

// ── Lossy-format seam ─────────────────────────────────────────────────────────

/// Extension predicates: which auto-quality search an output format supports.
///
/// There are **two** distinct capabilities, because the two searches need
/// different things from a format:
/// - The **byte-budget** search (`--max-size`, SPEC-017) only ENCODES candidates
///   and measures their length — it needs an encoder quality knob.
/// - The **perceptual** search (`--target`/`--ssim`, SPEC-016) encodes a
///   candidate AND **decodes it back** to score the round-trip with SSIMULACRA2 —
///   it needs both an encoder knob and a DECODER.
///
/// AVIF (SPEC-018) is the case that forces the split: with `--features avif` it
/// has an encoder quality knob (so the byte-budget search works) but **no decoder
/// is built** (AVIF decode needs `dav1d`/`avif-native`, deferred — DEC-020), so
/// the perceptual search cannot score AVIF round-trips. JPEG has both.
///
/// These are the **single seams** the CLI guard (`resolve_effective_quality`) and
/// the per-format candidate encode ([`encode_candidate_bytes`]) read. When a new
/// lossy format lands, set the right predicate(s) here AND add its encode arm in
/// `encode_candidate_bytes` — that is the whole change.
pub trait LossyFormat {
    /// `true` iff the **byte-budget** search can drive a quality knob for this
    /// format (encode-only; no decoder required).
    fn supports_lossy_quality(self) -> bool;

    /// `true` iff the **perceptual** search can score this format — it has both a
    /// quality knob AND a built-in decoder to round-trip candidates through.
    fn supports_perceptual_quality(self) -> bool;
}

impl LossyFormat for ImageFormat {
    fn supports_lossy_quality(self) -> bool {
        // Byte-budget-drivable formats (encode-only; no decoder required). JPEG
        // always; AVIF with the `avif` feature (SPEC-018); WebP with the
        // `webp-lossy` feature (SPEC-020). Without a format's feature, its output
        // is lossless / exit-4 and no search is attempted, so it reports `false`.
        match self {
            ImageFormat::Jpeg => true,
            #[cfg(feature = "avif")]
            ImageFormat::Avif => true,
            #[cfg(feature = "webp-lossy")]
            ImageFormat::WebP => true,
            _ => false,
        }
    }

    fn supports_perceptual_quality(self) -> bool {
        // The perceptual search must DECODE each candidate to score it, so this
        // needs both an encoder knob AND a built-in decoder. JPEG always. WebP
        // with `webp-lossy` qualifies because the pure-Rust WebP DECODER ships by
        // default (SPEC-019). AVIF does NOT (output-only, no decoder — DEC-020),
        // so it is excluded even with the `avif` feature (perceptual AVIF defers
        // with AVIF decode).
        match self {
            ImageFormat::Jpeg => true,
            #[cfg(feature = "webp-lossy")]
            ImageFormat::WebP => true,
            _ => false,
        }
    }
}

// ── Production wiring (real candidate encoding) ───────────────────────────────

/// Encode `reference` to `fmt` at `quality.clamp(1, 100)` and return the bytes —
/// the shared candidate encode behind both production probes (`score_at` decodes
/// then scores it; `size_at` measures its length). Only formats for which
/// [`LossyFormat::supports_lossy_quality`] is `true` are valid; any other format
/// is a caller bug (the CLI guards on the predicate) and returns a
/// [`QualityError::Encode`].
///
/// IMPORTANT: for JPEG this MUST stay byte-for-byte equivalent to the production
/// write path `crate::sink::encode_to_bytes` (DEC-016) — same `JpegEncoder::
/// new_with_quality` + `1..=100` clamp. The searches optimize the bytes THIS
/// produces, but `shrink`/`convert` write the file through `encode_to_bytes`; if
/// the two ever diverge (e.g. a switch to a progressive/optimized JPEG encoder),
/// the searched quality would no longer describe the bytes actually emitted,
/// silently breaking the perceptual / byte-budget guarantee. Layering forbids
/// `quality` depending on `sink`, so they are kept in sync by this contract, not
/// by a shared call. Each lossy format added here carries the same obligation.
fn encode_candidate_bytes(
    reference: &DynamicImage,
    fmt: ImageFormat,
    quality: u8,
) -> Result<Vec<u8>, QualityError> {
    match fmt {
        ImageFormat::Jpeg => {
            let q = quality.clamp(1, 100);
            let mut cursor = Cursor::new(Vec::new());
            let encoder = ::image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, q);
            reference
                .write_with_encoder(encoder)
                .map_err(|e| QualityError::Encode(e.to_string()))?;
            Ok(cursor.into_inner())
        }
        // AVIF candidate encode (SPEC-018, DEC-020) — IDENTICAL to the sink's
        // AVIF arm (`crate::sink::encode_to_bytes`): same `AvifEncoder`, same
        // fixed `AVIF_SPEED`, same `1..=100` clamp. The byte-budget / perceptual
        // guarantee depends on this probe matching the bytes the sink writes.
        #[cfg(feature = "avif")]
        ImageFormat::Avif => {
            let q = quality.clamp(1, 100);
            let mut cursor = Cursor::new(Vec::new());
            let encoder = ::image::codecs::avif::AvifEncoder::new_with_speed_quality(
                &mut cursor,
                AVIF_SPEED,
                q,
            );
            reference
                .write_with_encoder(encoder)
                .map_err(|e| QualityError::Encode(e.to_string()))?;
            Ok(cursor.into_inner())
        }
        // Lossy WebP candidate encode (SPEC-020, DEC-022) — IDENTICAL to the
        // sink's WebP lossy arm (`crate::sink::encode_to_bytes`): same `from_rgba`
        // on `to_rgba8()` bytes, same `1..=100` clamp, same `q as f32`. The
        // byte-budget / perceptual guarantee depends on this probe matching the
        // bytes the sink writes. (Lossless WebP has no quality knob, so only the
        // lossy path — behind `webp-lossy` — reaches here.)
        #[cfg(feature = "webp-lossy")]
        ImageFormat::WebP => {
            let q = quality.clamp(1, 100);
            let rgba = reference.to_rgba8();
            let (w, h) = rgba.dimensions();
            let encoder = ::webp::Encoder::from_rgba(rgba.as_raw(), w, h);
            Ok(encoder.encode(q as f32).to_vec())
        }
        // Only lossy formats reach here (the CLI guards on `supports_lossy_quality`).
        other => Err(QualityError::Encode(format!(
            "no auto-quality encoder for {other:?} (not a lossy quality format)"
        ))),
    }
}

/// Encode `reference` to `fmt` at `quality`, decode it back, and score the
/// round-trip against `reference` — the real per-quality scorer the perceptual
/// search uses.
fn score_at(reference: &DynamicImage, fmt: ImageFormat, quality: u8) -> Result<f64, QualityError> {
    let bytes = encode_candidate_bytes(reference, fmt, quality)?;
    let decoded =
        ::image::load_from_memory(&bytes).map_err(|e| QualityError::Encode(e.to_string()))?;
    score(reference, &decoded)
}

/// Find the lowest `fmt` quality whose decoded round-trip scores ≥ `cfg.target`
/// for `reference` (the production entry point for `shrink`'s auto-quality).
/// `fmt` must satisfy [`LossyFormat::supports_lossy_quality`].
pub fn auto_quality(
    reference: &DynamicImage,
    fmt: ImageFormat,
    cfg: &SearchConfig,
) -> Result<QualityChoice, QualityError> {
    search_quality(|q| score_at(reference, fmt, q), cfg)
}

/// Encode `reference` to `fmt` at `quality` and return the encoded byte length —
/// the per-quality probe for the byte-budget search (SPEC-017). Unlike
/// `score_at`, this does NOT decode or score: the size search only needs the
/// encoded length (it shares the exact encode via `encode_candidate_bytes`).
fn size_at(reference: &DynamicImage, fmt: ImageFormat, quality: u8) -> Result<u64, QualityError> {
    Ok(encode_candidate_bytes(reference, fmt, quality)?.len() as u64)
}

/// Find the highest `fmt` quality whose encoded size is ≤ `budget_bytes` for
/// `reference` (the production entry point for `--max-size`, SPEC-017). The
/// returned [`QualityChoice::score`] carries the achieved encoded size in bytes.
/// `fmt` must satisfy [`LossyFormat::supports_lossy_quality`].
pub fn auto_under_size(
    reference: &DynamicImage,
    fmt: ImageFormat,
    budget_bytes: u64,
) -> Result<QualityChoice, QualityError> {
    search_under_size(
        |q| size_at(reference, fmt, q),
        budget_bytes,
        &SearchConfig::for_size_budget(),
    )
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
        let choice = search_quality(
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
        let choice = search_quality(|_q| Ok(10.0), &cfg).expect("search should succeed");
        assert_eq!(
            choice.quality, MAX_SEARCH_QUALITY,
            "unreachable target → best-effort highest quality"
        );
        assert!(!choice.met_target, "target was not met");
    }

    #[test]
    fn search_propagates_scorer_error() {
        let cfg = SearchConfig::for_target(50.0);
        let result = search_quality(|_q| Err(QualityError::Encode("boom".into())), &cfg);
        assert!(
            matches!(result, Err(QualityError::Encode(ref m)) if m == "boom"),
            "a scorer error must propagate, got {result:?}"
        );
    }

    #[test]
    fn auto_quality_is_monotone_in_target() {
        let img = detailed_rgb(96, 96);
        let lo = auto_quality(&img, ImageFormat::Jpeg, &SearchConfig::for_target(50.0))
            .expect("lo search");
        let hi = auto_quality(&img, ImageFormat::Jpeg, &SearchConfig::for_target(90.0))
            .expect("hi search");
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

    // ── SPEC-017: byte-budget search ──────────────────────────────────────────

    #[test]
    fn search_under_size_finds_highest_fitting() {
        // Synthetic monotone size fn: size == quality * 10. Highest q with
        // q*10 <= 500 is q == 50.
        let calls = Cell::new(0u8);
        let cfg = SearchConfig::for_size_budget();
        let choice = search_under_size(
            |q| {
                calls.set(calls.get() + 1);
                Ok(q as u64 * 10)
            },
            500,
            &cfg,
        )
        .expect("size search should succeed");
        assert_eq!(choice.quality, 50, "highest quality fitting 500 is 50");
        assert!(choice.met_target, "budget 500 is reachable");
        assert!(
            calls.get() <= cfg.max_iters,
            "probe called {} times, exceeds cap {}",
            calls.get(),
            cfg.max_iters
        );
    }

    #[test]
    fn search_under_size_unfittable_is_best_effort() {
        // Every quality exceeds the budget → best effort is the smallest (min q).
        let cfg = SearchConfig::for_size_budget();
        let choice = search_under_size(|_q| Ok(10_000), 100, &cfg).expect("search should succeed");
        assert_eq!(
            choice.quality, MIN_SEARCH_QUALITY,
            "unfittable budget → best-effort lowest quality (smallest file)"
        );
        assert!(!choice.met_target, "budget was not met");
    }

    #[test]
    fn search_under_size_propagates_error() {
        let cfg = SearchConfig::for_size_budget();
        let result = search_under_size(|_q| Err(QualityError::Encode("boom".into())), 1000, &cfg);
        assert!(
            matches!(result, Err(QualityError::Encode(ref m)) if m == "boom"),
            "a probe error must propagate, got {result:?}"
        );
    }

    #[test]
    fn auto_under_size_is_monotone_in_budget() {
        let img = detailed_rgb(96, 96);
        // A small budget forces a lower quality than a large budget.
        let small = auto_under_size(&img, ImageFormat::Jpeg, 1_500).expect("small-budget search");
        let large = auto_under_size(&img, ImageFormat::Jpeg, 12_000).expect("large-budget search");
        assert!(
            small.quality <= large.quality,
            "smaller budget should pick a lower-or-equal quality: 1500B q={} vs 12000B q={}",
            small.quality,
            large.quality
        );
    }

    #[test]
    fn search_config_for_size_budget_bounds() {
        let cfg = SearchConfig::for_size_budget();
        assert_eq!(cfg.min_quality, 1);
        assert_eq!(cfg.max_quality, 100);
        assert_eq!(cfg.max_iters, 8);
    }

    // ── SPEC-018: AVIF (feature-gated) ────────────────────────────────────────

    /// With the feature on, AVIF is a lossy-quality format the search can drive.
    #[cfg(feature = "avif")]
    #[test]
    fn avif_supports_lossy_quality() {
        assert!(
            ImageFormat::Avif.supports_lossy_quality(),
            "AVIF must support the lossy-quality search under --features avif"
        );
    }

    /// The byte-budget search drives AVIF: a smaller budget picks a lower-or-equal
    /// quality than a larger budget (the same monotonicity the JPEG search has).
    #[cfg(feature = "avif")]
    #[test]
    fn auto_under_size_avif_is_monotone() {
        let img = detailed_rgb(96, 96);
        let small =
            auto_under_size(&img, ImageFormat::Avif, 1_000).expect("small-budget AVIF search");
        let large =
            auto_under_size(&img, ImageFormat::Avif, 8_000).expect("large-budget AVIF search");
        assert!(
            small.quality <= large.quality,
            "smaller budget should pick a lower-or-equal AVIF quality: 1000B q={} vs 8000B q={}",
            small.quality,
            large.quality
        );
    }
}
