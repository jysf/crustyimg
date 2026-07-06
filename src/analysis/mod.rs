//! Computed-once image analysis (`Analysis`) — the shared feature layer for
//! PROJ-002's optimization engine (SPEC-046).
//!
//! Layering: this module depends only on `::image`, [`crate::image`], `std`,
//! and `thiserror`. It MUST NOT touch `clap`, `cli`, `sink`, `recipe`,
//! `source`, `std::fs`, or terminals — it is a pure, read-only pass over an
//! already-decoded [`crate::image::Image`] (mirrors the self-containment of
//! `src/quality/` and `src/operation/`).
//!
//! [`Analysis::compute`] runs a single accumulation pass over the decoded RGBA
//! buffer (luma + quantized-colour histograms, alpha coverage, saturation
//! buckets, capped unique-colour count, dominant colour), derives the luma
//! scalars (entropy, bimodality) in `O(256)`, then runs one linear edge pass.
//! It is bounded (the 512 MiB decode cap, DEC-034, already bounds the input;
//! `unique_colors` is capped at [`UNIQUE_COLOR_CAP`]) and **never panics** on
//! any input — a zero-area image is the one typed error (DEC-002, DEC-034,
//! constraint `untrusted-input-hardening`).
//!
//! Classification (`ImageClass`/`OptBucket`) is built on these features in
//! SPEC-047; nothing here is wired into a command yet.

use std::collections::HashSet;

use ::image::{ColorType, ImageFormat};
use thiserror::Error;

use crate::image::Image;

/// The maximum number of distinct RGB colours counted before the unique-colour
/// accumulator short-circuits to [`UniqueColors::Saturated`].
///
/// Exposed because it is a shared anchor with the classifier / format-decision
/// work (SPEC-047+): the "few-colour graphic" palette gate keys off this exact
/// cap, so it must not be duplicated.
pub const UNIQUE_COLOR_CAP: u32 = 4096;

// ── Feature thresholds ──────────────────────────────────────────────────────
// Starting anchors. A future tuning DEC (DEC-047, on the classifier) may adjust
// them against a labelled corpus. All named here so the tuning surface is one
// place (mirrors quality's `MAX_SEARCH_ITERS`).

/// Forward-difference gradient magnitude at/above which a pixel is an "edge".
const EDGE_THRESHOLD: u16 = 48;
/// Forward-difference gradient magnitude at/below which a pixel is "flat".
const FLAT_THRESHOLD: u16 = 4;
/// Chroma (max−min channel) at/below which a pixel counts as near-gray.
const GRAY_CHROMA_MAX: u8 = 12;
/// Chroma at/below which a pixel counts as low-saturation.
const SAT_LOW_CHROMA_MAX: u8 = 32;
/// Above this many evaluated pixels the edge pass strides whole rows
/// (deterministic, fixed stride) to stay linear-bounded on very large images.
const EDGE_SAMPLE_CAP: u64 = 4_000_000;

// ── Classification thresholds (SPEC-047, recorded in DEC-047) ────────────────
// Starting anchors, tuned against the synthetic corpus in the tests. The rule
// cascade below switches on these; the safe-fallback bias is `Photograph`.

/// `max(w, h)` at/below which an image may be an icon (with a squarish aspect).
const ICON_MAX_EDGE: u32 = 128;
/// Distinct-colour count at/below which the "few-colour graphic" palette gate
/// fires (the industry pngquant/WebP ≤256-colour lossless heuristic).
const PALETTE_COLORS: u32 = 256;
/// Long/short edge ratio at/below which an aspect counts as squarish (icons).
const ICON_ASPECT_MAX: f32 = 2.0;
/// Flat-region fraction at/above which (with low edges) an image is a graphic.
const FLAT_GRAPHIC_RATIO: f32 = 0.60;
/// Edge fraction below which the flat-graphic gate may fire (few, if any, edges).
const GRAPHIC_EDGE_MAX: f32 = 0.08;
/// Bimodality at/above which an image may be a document/scan.
const DOC_BIMODALITY: f32 = 0.55;
/// Near-gray fraction at/above which an image may be a document/scan.
const DOC_GRAY_RATIO: f32 = 0.85;
/// Entropy below which an image may be a document/scan.
const DOC_ENTROPY_MAX: f32 = 4.5;
/// Flat-region fraction at/above which a wide many-colour image may be a
/// UI screenshot.
const UI_FLAT_RATIO: f32 = 0.35;
/// Long/short edge ratio at/above which an aspect counts as "screen-wide".
const UI_ASPECT_MIN: f32 = 1.3;
/// Entropy at/above which (with few flat regions) an image is a photograph.
const PHOTO_ENTROPY: f32 = 5.0;
/// Flat-region fraction below which the entropy photo rule may fire.
const PHOTO_FLAT_MAX: f32 = 0.25;
/// Confidence reported when the cascade falls through to the safe default.
const FALLBACK_CONFIDENCE: f32 = 0.4;

/// The distinct-colour count, capped for bounded memory (DEC-034 discipline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniqueColors {
    /// Fewer than [`UNIQUE_COLOR_CAP`] distinct RGB colours: the exact count.
    Exact(u32),
    /// At least [`UNIQUE_COLOR_CAP`] distinct colours; counting short-circuited
    /// at the cap (the value carried is the cap, not the true total).
    Saturated(u32),
}

impl UniqueColors {
    /// The carried count (the exact total, or the cap when saturated).
    pub fn count(self) -> u32 {
        match self {
            UniqueColors::Exact(n) | UniqueColors::Saturated(n) => n,
        }
    }

    /// Whether counting hit the cap (i.e. the image has *at least* the cap many
    /// distinct colours).
    pub fn is_saturated(self) -> bool {
        matches!(self, UniqueColors::Saturated(_))
    }
}

/// The fine-grained image class (SPEC-047). Deterministic, no-ML; kept mainly
/// for the `explain` cosmetic label. The optimization engine switches on the
/// coarser [`OptBucket`] via [`ImageClass::opt_bucket`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageClass {
    /// Camera/continuous-tone content → compress lossy.
    Photograph,
    /// Logo / flat-colour graphic → compress lossless.
    GraphicLogo,
    /// Small squarish icon → keep lossless/palette.
    Icon,
    /// Scanned/rendered document or line-art → lossless.
    Document,
    /// Application/UI screenshot or illustration → mixed (try both families).
    UiScreenshot,
}

impl ImageClass {
    /// Collapse the five classes into the three optimization buckets the format
    /// engine switches on. Exhaustive by design (no wildcard arm): adding a
    /// class is a compile error until its bucket is chosen.
    pub fn opt_bucket(self) -> OptBucket {
        match self {
            ImageClass::Photograph => OptBucket::Lossy,
            ImageClass::GraphicLogo | ImageClass::Icon | ImageClass::Document => {
                OptBucket::LosslessFlat
            }
            ImageClass::UiScreenshot => OptBucket::MixedSafe,
        }
    }
}

/// The coarse optimization disposition the format-decision engine (SPEC-048)
/// switches on, alongside `has_alpha`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptBucket {
    /// Photographic → lossy codec family (JPEG / lossy-WebP / AVIF).
    Lossy,
    /// Flat/graphic/icon/document → lossless family (PNG / lossless-WebP).
    LosslessFlat,
    /// Ambiguous (UI/illustration) → try both and let measured bytes decide.
    MixedSafe,
}

/// Errors from [`Analysis::compute`]. Typed and no-panic
/// (constraint `untrusted-input-hardening`); the only degenerate case is a
/// zero-area image, for which every ratio/scalar is undefined.
#[derive(Debug, Error)]
pub enum AnalysisError {
    /// The image has zero area (a zero width or height) — nothing to analyse.
    #[error("cannot analyse a degenerate image with zero area ({width}×{height})")]
    DegenerateDimensions {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
    },
}

/// An immutable, computed-once snapshot of an image's decisive features
/// (SPEC-046). Construct it via [`Analysis::compute`]; read it through the
/// accessors. There are no public fields and no `&mut self` method.
#[derive(Debug, Clone, PartialEq)]
pub struct Analysis {
    width: u32,
    height: u32,
    color_type: ColorType,
    alpha_translucent: f32,
    alpha_transparent: f32,
    unique_colors: UniqueColors,
    luma_histogram: [u32; 256],
    entropy: f32,
    bimodality: f32,
    edge_ratio: f32,
    flat_ratio: f32,
    sat_low_ratio: f32,
    gray_ratio: f32,
    dominant_color: [u8; 4],
    class: ImageClass,
    opt_bucket: OptBucket,
    confidence: f32,
}

impl Analysis {
    /// Compute the feature snapshot in a single accumulation pass over the
    /// decoded buffer plus one edge pass. Never re-decodes, never touches disk,
    /// never panics (a zero-area image is [`AnalysisError::DegenerateDimensions`]).
    pub fn compute(img: &Image) -> Result<Analysis, AnalysisError> {
        // One conversion to a working RGBA view, exactly as the ops do
        // (decode-once, DEC-002 — the pixels are already decoded).
        let rgba = img.pixels().to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        if w == 0 || h == 0 {
            return Err(AnalysisError::DegenerateDimensions {
                width: w,
                height: h,
            });
        }
        let total = (w as u64) * (h as u64);
        let color_type = img.pixels().color();

        let mut luma_histogram = [0u32; 256];
        // Quantized 4-4-4 RGB histogram (4096 bins) → deterministic dominant
        // colour; O(1) in image size.
        let mut quant_hist = vec![0u32; 4096];
        // Luma buffer for the edge pass; O(pixels) but bounded by the decode cap.
        let mut luma = Vec::with_capacity(w as usize * h as usize);

        let mut transparent = 0u64;
        let mut translucent = 0u64;
        let mut gray = 0u64;
        let mut sat_low = 0u64;

        let mut uniq: HashSet<u32> = HashSet::new();
        let mut saturated = false;

        for px in rgba.pixels() {
            let [r, g, b, a] = px.0;

            // Integer BT.601-ish luma (77+150+29 = 256 ⇒ >>8 stays in 0..=255).
            let l = ((77 * r as u32 + 150 * g as u32 + 29 * b as u32) >> 8) as u8;
            luma.push(l);
            luma_histogram[l as usize] += 1;

            let chroma = r.max(g).max(b) - r.min(g).min(b);
            if chroma <= GRAY_CHROMA_MAX {
                gray += 1;
            }
            if chroma <= SAT_LOW_CHROMA_MAX {
                sat_low += 1;
            }

            if a == 0 {
                transparent += 1;
            } else if a < 255 {
                translucent += 1;
            }

            let qbin =
                (((r >> 4) as usize) << 8) | (((g >> 4) as usize) << 4) | ((b >> 4) as usize);
            quant_hist[qbin] += 1;

            if !saturated {
                let packed = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                uniq.insert(packed);
                if uniq.len() as u32 >= UNIQUE_COLOR_CAP {
                    saturated = true; // short-circuit: stop inserting further
                }
            }
        }

        let unique_colors = if saturated {
            UniqueColors::Saturated(UNIQUE_COLOR_CAP)
        } else {
            UniqueColors::Exact(uniq.len() as u32)
        };

        let alpha_transparent = ratio(transparent, total);
        let alpha_translucent = ratio(translucent, total);
        let gray_ratio = ratio(gray, total);
        let sat_low_ratio = ratio(sat_low, total);

        let entropy = shannon_entropy(&luma_histogram, total);
        let bimodality = top_two_mass(&luma_histogram, total);
        let dominant_color = dominant_from_quant(&quant_hist);
        let (edge_ratio, flat_ratio) = edge_flat_ratios(&luma, w, h);

        // Container priors read off `Image` — never re-parsed (DEC-002/DEC-003).
        let has_exif = img.info().has_exif;
        let source_format = img.source_format();
        let (class, confidence) = classify(ClassifyInput {
            unique_colors,
            flat_ratio,
            edge_ratio,
            entropy,
            bimodality,
            gray_ratio,
            has_exif,
            source_format,
            width: w,
            height: h,
        });
        let opt_bucket = class.opt_bucket();

        Ok(Analysis {
            width: w,
            height: h,
            color_type,
            alpha_translucent,
            alpha_transparent,
            unique_colors,
            luma_histogram,
            entropy,
            bimodality,
            edge_ratio,
            flat_ratio,
            sat_low_ratio,
            gray_ratio,
            dominant_color,
            class,
            opt_bucket,
            confidence,
        })
    }

    /// Image dimensions `(width, height)` in pixels.
    pub fn dims(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Decoded colour type.
    pub fn color_type(&self) -> ColorType {
        self.color_type
    }

    /// Fraction of pixels with partial alpha (`0 < a < 255`).
    pub fn alpha_translucent(&self) -> f32 {
        self.alpha_translucent
    }

    /// Fraction of pixels that are fully transparent (`a == 0`).
    pub fn alpha_transparent(&self) -> f32 {
        self.alpha_transparent
    }

    /// Distinct RGB colour count, capped at [`UNIQUE_COLOR_CAP`].
    pub fn unique_colors(&self) -> UniqueColors {
        self.unique_colors
    }

    /// The 256-bin luma histogram (dual-use: the entropy/bimodality source and
    /// a future compression heatmap).
    pub fn luma_histogram(&self) -> &[u32; 256] {
        &self.luma_histogram
    }

    /// Shannon entropy of the luma histogram (bits, `0.0..=8.0`). Low ⇒
    /// graphic/flat; high ⇒ photographic.
    pub fn entropy(&self) -> f32 {
        self.entropy
    }

    /// Mass concentrated in the two largest luma bins (`0.0..=1.0`). High ⇒
    /// bimodal (documents / line-art).
    pub fn bimodality(&self) -> f32 {
        self.bimodality
    }

    /// Fraction of evaluated pixels whose forward-difference gradient is an edge.
    pub fn edge_ratio(&self) -> f32 {
        self.edge_ratio
    }

    /// Fraction of evaluated pixels whose forward-difference gradient is flat.
    pub fn flat_ratio(&self) -> f32 {
        self.flat_ratio
    }

    /// Fraction of low-saturation pixels (chroma ≤ [`SAT_LOW_CHROMA_MAX`]).
    pub fn sat_low_ratio(&self) -> f32 {
        self.sat_low_ratio
    }

    /// Fraction of near-gray pixels (chroma ≤ [`GRAY_CHROMA_MAX`]).
    pub fn gray_ratio(&self) -> f32 {
        self.gray_ratio
    }

    /// A representative dominant colour `[r, g, b, a]` (the centre of the most
    /// populated quantized-colour bin; `a` is reported opaque).
    pub fn dominant_color(&self) -> [u8; 4] {
        self.dominant_color
    }

    /// The fine-grained image class (SPEC-047). Mainly an `explain` cosmetic;
    /// the engine reads [`Analysis::opt_bucket`].
    pub fn class(&self) -> ImageClass {
        self.class
    }

    /// The coarse optimization bucket the format engine switches on.
    pub fn opt_bucket(&self) -> OptBucket {
        self.opt_bucket
    }

    /// Classification confidence `0.0..=1.0`. Low values (≤ the fallback
    /// anchor) mean the safe-default `Photograph` bias fired; `explain` can
    /// hedge on them.
    pub fn confidence(&self) -> f32 {
        self.confidence
    }
}

/// `numerator / denominator` as an `f32` fraction (denominator is a nonzero
/// pixel count here). Kept as a free fn so the accumulation reads cleanly.
fn ratio(numerator: u64, denominator: u64) -> f32 {
    (numerator as f64 / denominator as f64) as f32
}

/// Shannon entropy `H = -Σ p·log2 p` over the luma histogram, in fixed bin
/// order (deterministic run-to-run).
fn shannon_entropy(hist: &[u32; 256], total: u64) -> f32 {
    let t = total as f64;
    let mut h = 0.0f64;
    for &c in hist.iter() {
        if c > 0 {
            let p = c as f64 / t;
            h -= p * p.log2();
        }
    }
    h as f32
}

/// Fraction of total mass in the two most-populated luma bins.
fn top_two_mass(hist: &[u32; 256], total: u64) -> f32 {
    let mut m1 = 0u32;
    let mut m2 = 0u32;
    for &c in hist.iter() {
        if c >= m1 {
            m2 = m1;
            m1 = c;
        } else if c > m2 {
            m2 = c;
        }
    }
    ratio(m1 as u64 + m2 as u64, total)
}

/// The representative colour of the most-populated quantized 4-4-4 bin
/// (tie → lowest bin index, for determinism).
fn dominant_from_quant(quant_hist: &[u32]) -> [u8; 4] {
    let mut best = 0usize;
    let mut best_count = 0u32;
    for (i, &c) in quant_hist.iter().enumerate() {
        if c > best_count {
            best_count = c;
            best = i;
        }
    }
    let r = ((best >> 8) & 0xF) as u8;
    let g = ((best >> 4) & 0xF) as u8;
    let b = (best & 0xF) as u8;
    // Map the 4-bit bin to the centre of its 16-value range.
    [(r << 4) | 8, (g << 4) | 8, (b << 4) | 8, 255]
}

/// Edge / flat ratios via a forward-difference gradient
/// `|L(x+1,y)-L(x,y)| + |L(x,y+1)-L(x,y)|` (integer, no kernel library).
///
/// Forward (not central) difference is deliberate: a central difference
/// `L(x+1)-L(x-1)` is blind to a 1-pixel checkerboard (its opposite neighbours
/// cancel), which would report a hard checkerboard as flat. Border pixels
/// (last row/column) are not evaluated. On very large images rows are strided
/// with a fixed step (deterministic).
fn edge_flat_ratios(luma: &[u8], w: u32, h: u32) -> (f32, f32) {
    if w < 2 || h < 2 {
        // No interior gradient to measure — treat as flat, never an edge.
        return (0.0, 1.0);
    }
    let w = w as usize;
    let eval_w = w - 1;
    let eval_h = h as usize - 1;

    let total_eval = (eval_w as u64) * (eval_h as u64);
    let row_stride = if total_eval > EDGE_SAMPLE_CAP {
        ((total_eval / EDGE_SAMPLE_CAP) as usize).max(1)
    } else {
        1
    };

    let mut edges = 0u64;
    let mut flats = 0u64;
    let mut evaluated = 0u64;

    let mut y = 0usize;
    while y < eval_h {
        let row = y * w;
        let row_below = (y + 1) * w;
        for x in 0..eval_w {
            let c = luma[row + x] as i16;
            let right = luma[row + x + 1] as i16;
            let below = luma[row_below + x] as i16;
            let grad: u16 = (right - c).unsigned_abs() + (below - c).unsigned_abs();
            if grad >= EDGE_THRESHOLD {
                edges += 1;
            }
            if grad <= FLAT_THRESHOLD {
                flats += 1;
            }
            evaluated += 1;
        }
        y += row_stride;
    }

    if evaluated == 0 {
        return (0.0, 1.0);
    }
    (ratio(edges, evaluated), ratio(flats, evaluated))
}

/// Inputs to [`classify`]: the computed features plus the container priors.
/// Grouped in a struct to keep the cascade signature readable (and clippy
/// quiet about argument count).
struct ClassifyInput {
    unique_colors: UniqueColors,
    flat_ratio: f32,
    edge_ratio: f32,
    entropy: f32,
    bimodality: f32,
    gray_ratio: f32,
    has_exif: bool,
    source_format: ImageFormat,
    width: u32,
    height: u32,
}

/// The deterministic, no-ML classification cascade (SPEC-047 / DEC-047).
///
/// Cheapest/strongest first, **first match wins** (precedence, not averaging).
/// The camera prior (`has_exif`) is decisive and checked early — a
/// photo-of-a-document still routes lossy, which is usually the right *format*
/// call. The safe fallback is `Photograph`: a photo forced lossless is merely a
/// larger file, whereas a graphic forced lossy smears text/edges (and the lossy
/// downside is bounded anyway by the SSIMULACRA2 target downstream, DEC-019).
///
/// `source_format` is used only as the decisive `Ico → Icon` signal in v1; the
/// softer JPEG/PNG family leans are deferred (they would over-bias without a
/// real corpus, and `has_exif` already carries the camera prior).
fn classify(input: ClassifyInput) -> (ImageClass, f32) {
    let ClassifyInput {
        unique_colors,
        flat_ratio,
        edge_ratio,
        entropy,
        bimodality,
        gray_ratio,
        has_exif,
        source_format,
        width,
        height,
    } = input;

    let n = unique_colors.count();
    let few_colors = !unique_colors.is_saturated() && n <= PALETTE_COLORS;
    let many_colors = !few_colors;
    let long_short = width.max(height) as f32 / width.min(height).max(1) as f32;

    // 1. Icon — an `.ico`, or small + squarish and not a camera capture.
    if source_format == ImageFormat::Ico
        || (!has_exif && width.max(height) <= ICON_MAX_EDGE && long_short <= ICON_ASPECT_MAX)
    {
        return (ImageClass::Icon, 0.8);
    }

    // 2. Photograph via the decisive camera prior (EXIF) — before the
    //    graphic/document rules, so a flat-ish camera photo still routes lossy.
    if has_exif {
        return (ImageClass::Photograph, 0.9);
    }

    // 3. Document/scan — bimodal, near-gray, low-entropy (more specific than the
    //    graphic rule, so checked first even though both bucket LosslessFlat).
    if bimodality >= DOC_BIMODALITY && gray_ratio >= DOC_GRAY_RATIO && entropy < DOC_ENTROPY_MAX {
        return (ImageClass::Document, 0.7);
    }

    // 4. Graphic/logo — the ≤256-colour palette gate, or large flat fills with
    //    few edges.
    if few_colors {
        return (ImageClass::GraphicLogo, 0.85);
    }
    if flat_ratio >= FLAT_GRAPHIC_RATIO && edge_ratio < GRAPHIC_EDGE_MAX {
        return (ImageClass::GraphicLogo, 0.7);
    }

    // 5. UI-screenshot — wide, many-colour, moderately flat, but with real edges
    //    (so it did not match the flat-graphic gate above).
    if many_colors && flat_ratio >= UI_FLAT_RATIO && long_short >= UI_ASPECT_MIN {
        return (ImageClass::UiScreenshot, 0.6);
    }

    // 6. Photograph — the no-EXIF heuristic: rich colour, high entropy, few flat
    //    regions.
    if many_colors && entropy >= PHOTO_ENTROPY && flat_ratio < PHOTO_FLAT_MAX {
        return (ImageClass::Photograph, 0.7);
    }

    // 7. Fallback → Photograph (safe bias), low confidence.
    (ImageClass::Photograph, FALLBACK_CONFIDENCE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::image::{DynamicImage, ImageFormat, Rgb, RgbImage, Rgba, RgbaImage};

    /// Wrap a `DynamicImage` as an `Image` for analysis (no encode round-trip;
    /// analysis is a pure pass over already-decoded pixels).
    fn image_of(dyn_img: DynamicImage) -> Image {
        Image::from_parts(dyn_img, ImageFormat::Png, None)
    }

    fn solid_rgb(w: u32, h: u32, rgb: [u8; 3]) -> Image {
        image_of(DynamicImage::ImageRgb8(RgbImage::from_pixel(
            w,
            h,
            Rgb(rgb),
        )))
    }

    #[test]
    fn solid_image_zero_entropy_one_colour_flat() {
        let a = Analysis::compute(&solid_rgb(64, 64, [120, 120, 120])).unwrap();
        assert!(a.entropy() < 0.01, "entropy {}", a.entropy());
        assert_eq!(a.unique_colors(), UniqueColors::Exact(1));
        assert!(a.flat_ratio() > 0.99, "flat {}", a.flat_ratio());
        assert!(a.edge_ratio() < 0.01, "edge {}", a.edge_ratio());
    }

    #[test]
    fn vertical_gradient_nonzero_entropy_low_edges() {
        let (w, h) = (64u32, 64u32);
        let mut img = RgbImage::new(w, h);
        for y in 0..h {
            let v = (y * 255 / (h - 1)) as u8;
            for x in 0..w {
                img.put_pixel(x, y, Rgb([v, v, v]));
            }
        }
        let a = Analysis::compute(&image_of(DynamicImage::ImageRgb8(img))).unwrap();
        assert!(a.entropy() > 1.0, "entropy {}", a.entropy());
        assert!(a.edge_ratio() < 0.1, "edge {}", a.edge_ratio());
    }

    #[test]
    fn checkerboard_high_edge_ratio() {
        let (w, h) = (8u32, 8u32);
        let mut img = RgbImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let v = if (x + y) % 2 == 0 { 255 } else { 0 };
                img.put_pixel(x, y, Rgb([v, v, v]));
            }
        }
        let a = Analysis::compute(&image_of(DynamicImage::ImageRgb8(img))).unwrap();
        assert!(a.edge_ratio() > 0.9, "edge {}", a.edge_ratio());
        assert!(a.flat_ratio() < 0.1, "flat {}", a.flat_ratio());
    }

    #[test]
    fn few_colour_graphic_capped_exact_and_small() {
        // Four distinct colours in quadrants of a 32×32 image.
        let (w, h) = (32u32, 32u32);
        let colours = [
            Rgb([255, 0, 0]),
            Rgb([0, 255, 0]),
            Rgb([0, 0, 255]),
            Rgb([255, 255, 0]),
        ];
        let mut img = RgbImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let q = (x / (w / 2)) + 2 * (y / (h / 2));
                img.put_pixel(x, y, colours[q as usize]);
            }
        }
        let a = Analysis::compute(&image_of(DynamicImage::ImageRgb8(img))).unwrap();
        assert_eq!(a.unique_colors(), UniqueColors::Exact(4));
    }

    #[test]
    fn many_colour_photo_like_saturates_at_cap() {
        // 128×128 with a distinct (r,g) per pixel ⇒ >4096 unique colours.
        let (w, h) = (128u32, 128u32);
        let mut img = RgbImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let r = (i % 256) as u8;
                let g = (i / 256) as u8;
                let b = ((i * 7) % 256) as u8;
                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        let a = Analysis::compute(&image_of(DynamicImage::ImageRgb8(img))).unwrap();
        assert!(a.unique_colors().is_saturated());
        assert_eq!(a.unique_colors(), UniqueColors::Saturated(UNIQUE_COLOR_CAP));
    }

    #[test]
    fn rgba_with_transparent_region_alpha_transparent_positive() {
        let (w, h) = (32u32, 32u32);
        let mut img = RgbaImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                // Left third transparent, middle third translucent, right opaque.
                let a = match x * 3 / w {
                    0 => 0,
                    1 => 128,
                    _ => 255,
                };
                img.put_pixel(x, y, Rgba([10, 20, 30, a]));
            }
        }
        let a = Analysis::compute(&image_of(DynamicImage::ImageRgba8(img))).unwrap();
        assert!(
            a.alpha_transparent() > 0.0,
            "transp {}",
            a.alpha_transparent()
        );
        assert!(
            a.alpha_translucent() > 0.0,
            "transl {}",
            a.alpha_translucent()
        );
    }

    #[test]
    fn degenerate_dimensions_do_not_panic() {
        // Zero-area ⇒ typed error, no panic.
        let zero = image_of(DynamicImage::ImageRgba8(RgbaImage::new(0, 0)));
        assert!(matches!(
            Analysis::compute(&zero),
            Err(AnalysisError::DegenerateDimensions { .. })
        ));

        // A 1×1 image is well-defined (not an error) and must not panic.
        let one = solid_rgb(1, 1, [7, 7, 7]);
        let a = Analysis::compute(&one).unwrap();
        assert_eq!(a.dims(), (1, 1));
        assert_eq!(a.unique_colors(), UniqueColors::Exact(1));
        assert!(a.edge_ratio() < 0.01 && a.flat_ratio() > 0.99);
    }

    #[test]
    fn determinism_two_computes_identical() {
        let img = solid_rgb(48, 48, [33, 90, 200]);
        let a1 = Analysis::compute(&img).unwrap();
        let a2 = Analysis::compute(&img).unwrap();
        assert_eq!(a1, a2);
    }

    #[test]
    fn unique_colors_helpers() {
        assert_eq!(UniqueColors::Exact(3).count(), 3);
        assert!(!UniqueColors::Exact(3).is_saturated());
        assert_eq!(
            UniqueColors::Saturated(UNIQUE_COLOR_CAP).count(),
            UNIQUE_COLOR_CAP
        );
        assert!(UniqueColors::Saturated(UNIQUE_COLOR_CAP).is_saturated());
    }

    // ── SPEC-047 classification: a small labelled synthetic corpus ───────────

    /// Wrap pixels with an explicit source format + optional captured EXIF.
    fn image_with(dyn_img: DynamicImage, fmt: ImageFormat, exif: bool) -> Image {
        let meta = exif.then(|| crate::image::MetadataBundle {
            exif: Some(vec![0x45, 0x78, 0x69, 0x66]),
            icc: None,
        });
        Image::from_parts(dyn_img, fmt, meta)
    }

    /// A full-colour pseudo-noise RGB image: each channel XOR-mixes the scaled
    /// coordinates, giving a high-variety (saturated), high-entropy, edgy field
    /// (adjacent pixels differ sharply) — photographic-like.
    fn noise_rgb(w: u32, h: u32) -> DynamicImage {
        let mut img = RgbImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let r = (x.wrapping_mul(61) ^ y.wrapping_mul(97)) as u8;
                let g = (x.wrapping_mul(113) ^ y.wrapping_mul(29)) as u8;
                let b = (x.wrapping_mul(17) ^ y.wrapping_mul(191)) as u8;
                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn noise_is_photograph_lossy() {
        let a = Analysis::compute(&image_of(noise_rgb(256, 192))).unwrap();
        assert_eq!(a.class(), ImageClass::Photograph);
        assert_eq!(a.opt_bucket(), OptBucket::Lossy);
    }

    #[test]
    fn exif_prior_forces_photograph_even_when_flat() {
        // A flat gray field would read as a graphic — the camera prior overrides.
        let flat = DynamicImage::ImageRgb8(RgbImage::from_pixel(200, 150, Rgb([180, 180, 180])));
        let a = Analysis::compute(&image_with(flat, ImageFormat::Jpeg, true)).unwrap();
        assert_eq!(a.class(), ImageClass::Photograph);
        assert!(a.confidence() >= 0.85, "conf {}", a.confidence());
    }

    #[test]
    fn few_colour_graphic_is_lossless_flat() {
        let (w, h) = (200u32, 200u32);
        let colours = [
            Rgb([200, 30, 30]),
            Rgb([30, 200, 30]),
            Rgb([30, 30, 200]),
            Rgb([200, 200, 30]),
            Rgb([200, 30, 200]),
            Rgb([30, 200, 200]),
        ];
        let mut img = RgbImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                img.put_pixel(x, y, colours[(y * 6 / h) as usize]);
            }
        }
        let a = Analysis::compute(&image_of(DynamicImage::ImageRgb8(img))).unwrap();
        assert_eq!(a.class(), ImageClass::GraphicLogo);
        assert_eq!(a.opt_bucket(), OptBucket::LosslessFlat);
    }

    #[test]
    fn tiny_square_is_icon() {
        let a = Analysis::compute(&solid_rgb(48, 48, [10, 120, 200])).unwrap();
        assert_eq!(a.class(), ImageClass::Icon);
        assert_eq!(a.opt_bucket(), OptBucket::LosslessFlat);
    }

    #[test]
    fn bimodal_grayscale_is_document() {
        let (w, h) = (200u32, 160u32);
        let mut img = RgbImage::from_pixel(w, h, Rgb([255, 255, 255]));
        // Black "text" bars on ~25% of rows: two luma levels, near-gray, low entropy.
        for y in 0..h {
            if y % 8 < 2 {
                for x in 0..w {
                    img.put_pixel(x, y, Rgb([0, 0, 0]));
                }
            }
        }
        let a = Analysis::compute(&image_of(DynamicImage::ImageRgb8(img))).unwrap();
        assert_eq!(a.class(), ImageClass::Document);
        assert_eq!(a.opt_bucket(), OptBucket::LosslessFlat);
    }

    #[test]
    fn wide_flat_manycolour_with_edges_is_ui_screenshot() {
        let (w, h) = (320u32, 180u32);
        let mut img = RgbImage::new(w, h);
        // Smooth 2-axis gradient → many colours, low local gradient (flat-ish).
        for y in 0..h {
            for x in 0..w {
                let r = (x * 255 / (w - 1)) as u8;
                let g = (y * 255 / (h - 1)) as u8;
                img.put_pixel(x, y, Rgb([r, g, 128]));
            }
        }
        // Hard grid lines → real edges, so the flat-graphic gate does NOT fire.
        for y in 0..h {
            for x in 0..w {
                if x % 16 == 0 || y % 16 == 0 {
                    img.put_pixel(x, y, Rgb([0, 0, 0]));
                }
            }
        }
        let a = Analysis::compute(&image_of(DynamicImage::ImageRgb8(img))).unwrap();
        assert_eq!(a.class(), ImageClass::UiScreenshot);
        assert_eq!(a.opt_bucket(), OptBucket::MixedSafe);
    }

    #[test]
    fn ambiguous_square_falls_back_to_photograph_low_confidence() {
        // Half smooth gradient (flat, many colours), half noise (edgy) — trips no
        // strong rule, square so not UI ⇒ the safe fallback to Photograph.
        let (w, h) = (200u32, 200u32);
        let mut img = RgbImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                if x < w / 2 {
                    let v = (x * 255 / (w / 2 - 1).max(1)) as u8;
                    img.put_pixel(x, y, Rgb([v, (y * 255 / (h - 1)) as u8, 200]));
                } else {
                    let i = y * w + x;
                    img.put_pixel(
                        x,
                        y,
                        Rgb([
                            (i.wrapping_mul(61) % 256) as u8,
                            (i.wrapping_mul(113) % 256) as u8,
                            (i.wrapping_mul(191) % 256) as u8,
                        ]),
                    );
                }
            }
        }
        let a = Analysis::compute(&image_of(DynamicImage::ImageRgb8(img))).unwrap();
        assert_eq!(a.class(), ImageClass::Photograph);
        assert_eq!(a.opt_bucket(), OptBucket::Lossy);
        assert!(
            a.confidence() <= FALLBACK_CONFIDENCE + 1e-6,
            "conf {}",
            a.confidence()
        );
    }

    #[test]
    fn opt_bucket_collapse_is_total() {
        assert_eq!(ImageClass::Photograph.opt_bucket(), OptBucket::Lossy);
        assert_eq!(
            ImageClass::GraphicLogo.opt_bucket(),
            OptBucket::LosslessFlat
        );
        assert_eq!(ImageClass::Icon.opt_bucket(), OptBucket::LosslessFlat);
        assert_eq!(ImageClass::Document.opt_bucket(), OptBucket::LosslessFlat);
        assert_eq!(ImageClass::UiScreenshot.opt_bucket(), OptBucket::MixedSafe);
    }

    #[test]
    fn one_pixel_classifies_without_panic() {
        let a = Analysis::compute(&solid_rgb(1, 1, [9, 9, 9])).unwrap();
        assert_eq!(a.class(), ImageClass::Icon); // 1×1 satisfies the icon size rule
        assert!((0.0..=1.0).contains(&a.confidence()));
    }
}
