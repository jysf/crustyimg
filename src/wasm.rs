//! The `wasm-bindgen` surface over the pure engine (SPEC-072, DEC-064).
//!
//! This module is the WASM build's entry point, standing where `cli` stands on
//! native: it turns JS-shaped arguments (`&[u8]`, `&str`) into the crate's own
//! types, runs the SAME decode → transform → encode path the CLI runs, and hands
//! bytes back. It is **glue, not logic** — every function below is a few lines
//! over [`crate::image::Image`], [`crate::recipe::Recipe`], [`crate::pipeline`],
//! [`crate::analysis`], [`crate::quality`] and [`crate::sink::encode_to_bytes`].
//! Nothing here re-implements a transform, and nothing here decides anything the
//! native `optimize` doesn't already decide.
//!
//! Compiled only for `wasm32` (`#[cfg]` in `lib.rs`), so it adds nothing to the
//! native binary.
//!
//! ## AVIF: encode in, decode out (DEC-065)
//!
//! The wasm artifact is built `--features avif` (see the `wasm-*` justfile
//! recipes), so `transform(png, recipe, "avif")` really runs `rav1e` in the
//! browser — the wave's headline. AVIF **decode** is a different codec
//! (`re_rav1d`), and it does not compile to bare wasm32 (DEC-064), so an AVIF
//! *input* still returns a typed error. The asymmetry is deliberate and recorded
//! in DEC-065.
//!
//! ## What is NOT here
//!
//! - **AVIF decode.** An AVIF input returns a typed
//!   [`crate::image::ImageError::CodecUnavailableOnTarget`].
//! - **TIFF / BMP / ICO decode.** Trimmed from the wasm build's `image` feature set
//!   to save 84 KB brotli (SPEC-074, DEC-066) — a browser file picker is fed
//!   PNG/JPEG/GIF/WebP/SVG, not scanner TIFFs and favicons. Such an input errors
//!   cleanly, exactly like AVIF; the native CLI still reads all three.
//!
//!   So the wasm build's input reach is **PNG, JPEG, GIF, WebP, and SVG** (resvg
//!   rasterizes in wasm, `<text>` and all — DEC-066 priced dropping the text stack
//!   at 287 KB and kept it rather than silently eat your labels).
//! - **The filesystem.** There is no `source`/`sink` path handling in wasm: bytes
//!   in, bytes out. The caller (JS) owns the `File`/`Blob`.
//!
//! ## Errors
//!
//! Every fallible entry returns `Result<_, JsError>`, which surfaces in JS as a
//! thrown `Error` carrying the crate's own typed error message. Nothing here
//! panics on bad input — a panic in wasm aborts the module and takes the page's
//! instance with it, so the `untrusted-input-hardening` contract ("typed error,
//! never a panic") matters MORE in the browser, not less. The DEC-034/DEC-063
//! decode caps live in the core and therefore apply here unchanged.

use ::image::ImageFormat;
use wasm_bindgen::prelude::*;

use crate::analysis::decide::{self, BuiltCodecs, Disposition, Mode, Profile};
use crate::analysis::{Analysis, OptBucket};
use crate::image::Image;
use crate::operation::OperationRegistry;
use crate::quality::{self, LossyFormat, SearchConfig};
use crate::recipe::Recipe;
use crate::sink;

/// The perceptual target `optimize` aims for when the caller doesn't say —
/// "visually lossless" on the SSIMULACRA2 scale, the same default the native
/// `optimize --target` command uses (DEC-019).
const DEFAULT_TARGET: f64 = 90.0;

/// Which optional codecs this build has. The wasm build is the pure-Rust set: no
/// `webp-lossy` (a vendored C library, and no `cc` in a wasm build), but `avif`
/// **encode** when the artifact is built with the feature — which the shipped one
/// is (DEC-065). Read from `cfg!` rather than hardcoded, exactly as the CLI does
/// (`cli::built_codecs`), so this can never drift from what is actually linked.
///
/// Stated once here so [`decide::format_shortlist`] shortlists only formats we can
/// actually encode.
const WASM_CODECS: BuiltCodecs = BuiltCodecs {
    webp_lossy: false,
    avif: cfg!(feature = "avif"),
};

/// Turn any crate error into a JS `Error` carrying its typed message.
fn js_err(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

/// Resolve an output-format name (`"png"`, `"jpeg"`, `"webp"`, …) the way the CLI
/// resolves an output file's extension — through [`sink::format_from_extension`],
/// so the wasm surface and the CLI accept exactly the same format spellings and
/// cannot drift apart.
fn parse_format(name: &str) -> Result<ImageFormat, JsError> {
    // `format_from_extension` reads the extension off a Path; give it a filename
    // whose extension is the requested format. (A Path is pure string handling —
    // nothing touches a filesystem here.)
    let probe = std::path::PathBuf::from(format!("x.{name}"));
    sink::format_from_extension(&probe).map_err(js_err)
}

/// A decoded image's vital statistics, as a JS object (`{ width, height, format }`).
#[wasm_bindgen]
pub struct ImageInfo {
    width: u32,
    height: u32,
    format: String,
    has_alpha: bool,
}

#[wasm_bindgen]
impl ImageInfo {
    /// Decoded width in pixels.
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Decoded height in pixels.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The format detected at decode, lowercased (`"png"`, `"jpeg"`, …). A
    /// rasterized SVG reports `"png"` — it has become a lossless RGBA raster (the
    /// materialized-raster convention, SPEC-060).
    #[wasm_bindgen(getter)]
    pub fn format(&self) -> String {
        self.format.clone()
    }

    /// Whether the decoded pixels carry an alpha channel.
    #[wasm_bindgen(getter, js_name = hasAlpha)]
    pub fn has_alpha(&self) -> bool {
        self.has_alpha
    }
}

/// Decode `input` and report its width, height, format, and alpha — without
/// encoding anything.
///
/// The browser twin of `crustyimg info`.
#[wasm_bindgen]
pub fn info(input: &[u8]) -> Result<ImageInfo, JsError> {
    let img = Image::from_bytes(input).map_err(js_err)?;
    let i = img.info();
    Ok(ImageInfo {
        width: i.width,
        height: i.height,
        format: format_name(i.format),
        has_alpha: i.has_alpha,
    })
}

/// Decode `input`, run `recipe_toml` over it, and encode the result to
/// `out_format`. The browser twin of `crustyimg apply --recipe`.
///
/// `recipe_toml` is the SAME recipe format the CLI reads from disk (DEC-005) —
/// deserialized by [`Recipe::from_toml`] and built through the SAME
/// [`OperationRegistry`], so a recipe tuned in the terminal replays byte-for-byte
/// in the browser. That equivalence is the whole point of the wasm build; it is
/// why this function resolves operations through the registry rather than
/// switching on a name.
#[wasm_bindgen]
pub fn transform(input: &[u8], recipe_toml: &str, out_format: &str) -> Result<Vec<u8>, JsError> {
    let fmt = parse_format(out_format)?;
    let img = Image::from_bytes(input).map_err(js_err)?;

    let recipe = Recipe::from_toml(recipe_toml).map_err(js_err)?;
    let pipeline = recipe
        .build_pipeline(&OperationRegistry::with_builtins())
        .map_err(js_err)?;
    let out = pipeline.run(img).map_err(js_err)?;

    sink::encode_to_bytes(&out, fmt, None).map_err(js_err)
}

/// Decode `input` and re-encode it well: pick the format (when `out_format` is
/// empty or `"auto"`) and, for a lossy format, search for the lowest quality that
/// still hits the perceptual target.
///
/// This is a MINIMAL entry over the optimization engine, not the whole of it. It
/// reuses the engine's real parts — [`Analysis::compute`] for the content bucket,
/// [`decide::format_shortlist`] for the candidate ordering, and
/// [`quality::auto_quality`] for the SSIMULACRA2 quality search — but it takes the
/// shortlist's FIRST candidate rather than solving every candidate and running
/// [`decide::pick_winner`] over the measured outcomes, which is what native
/// `optimize` does (SPEC-048). So it will pick the same FORMAT as native for the
/// same image, and a genuinely searched quality for it, but it does not comparison-
/// shop encodings. Wiring the full multi-candidate solve into wasm is deliberately
/// left to a follow-up (it belongs in a shared engine seam both `cli` and `wasm`
/// call, not copy-pasted here) — see the SPEC-072 follow-ups.
#[wasm_bindgen]
pub fn optimize(input: &[u8], out_format: &str) -> Result<Vec<u8>, JsError> {
    let img = Image::from_bytes(input).map_err(js_err)?;

    // Resolve the target format: either the caller's, or the engine's own first
    // choice for this image's content bucket.
    let (fmt, disposition) = if out_format.is_empty() || out_format == "auto" {
        let analysis = Analysis::compute(&img).map_err(js_err)?;
        let shortlist = decide::format_shortlist(
            analysis.opt_bucket(),
            img.info().has_alpha,
            Profile::Web,
            Mode::Perceptual,
            WASM_CODECS,
        );
        // `format_shortlist` is documented never to return empty (it always carries
        // at least the always-available lossless PNG entry), but this surface must
        // not `unwrap` on ANY path — a panic aborts the wasm module.
        let first = shortlist
            .first()
            .ok_or_else(|| JsError::new("no encodable format for this image"))?;
        (first.fmt, first.disposition)
    } else {
        let fmt = parse_format(out_format)?;
        let disposition = if fmt.supports_lossy_quality() {
            Disposition::Lossy
        } else {
            Disposition::Lossless
        };
        (fmt, disposition)
    };

    // Nothing to search — encode at the format's default and hand back the bytes.
    // Two cases land here:
    //   * a LOSSLESS target (PNG, lossless WebP): no quality knob at all;
    //   * a lossy target this build cannot PERCEPTUALLY score. The perceptual
    //     search encodes a candidate and DECODES IT BACK to score the round-trip
    //     (DEC-019), so it needs a decoder — and AVIF has an encoder here but no
    //     decoder (DEC-065). Asking `auto_quality` to search AVIF would fail on the
    //     first candidate's decode, so the honest answer is a single encode at the
    //     encoder's default quality. `supports_perceptual_quality` is the same seam
    //     the CLI guards on, so wasm and native agree about which formats are
    //     searchable.
    if disposition == Disposition::Lossless || !fmt.supports_perceptual_quality() {
        return sink::encode_to_bytes(&img, fmt, None).map_err(js_err);
    }

    // Lossy: find the lowest quality that still reaches the perceptual target. This
    // is the real SSIMULACRA2 binary search (DEC-019) — the same code `optimize
    // --target` runs — so it decodes each candidate to score it. That is the
    // expensive part of the wasm build's runtime, and honestly so: it is what makes
    // the output good rather than merely small.
    let choice =
        quality::auto_quality(img.pixels(), fmt, &SearchConfig::for_target(DEFAULT_TARGET))
            .map_err(js_err)?;

    sink::encode_to_bytes(&img, fmt, Some(choice.quality)).map_err(js_err)
}

// ── The detailed optimize surface (SPEC-079, DEC-068) ─────────────────────────

/// What an [`optimize_detailed`] call actually did: the bytes, and the decisions
/// behind them.
///
/// `optimize` returns bytes and nothing else, which is why the demo could not tell
/// a user what quality it chose, how fast it encoded, or how good the result is.
/// This is the same engine with its work shown. Fields follow [`ImageInfo`]'s
/// pattern — a `#[wasm_bindgen]` struct with getters, no `serde-wasm-bindgen`
/// (DEC-064).
#[wasm_bindgen]
pub struct OptimizeResult {
    bytes: Vec<u8>,
    format: String,
    quality: Option<u8>,
    speed: Option<u8>,
    score: Option<f64>,
    scored_by: String,
}

#[wasm_bindgen]
impl OptimizeResult {
    /// The encoded output bytes (a `Uint8Array` in JS).
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    /// The format actually encoded, lowercased (`"avif"`, `"jpeg"`, `"png"`, …) —
    /// which, for `"auto"`, is the engine's choice, not the caller's.
    #[wasm_bindgen(getter)]
    pub fn format(&self) -> String {
        self.format.clone()
    }

    /// The encoder quality used, or `undefined` for a lossless format (no knob).
    #[wasm_bindgen(getter)]
    pub fn quality(&self) -> Option<u8> {
        self.quality
    }

    /// The rav1e encode speed used — `undefined` for anything but AVIF, the only
    /// encoder here with a speed knob.
    #[wasm_bindgen(getter)]
    pub fn speed(&self) -> Option<u8> {
        self.speed
    }

    /// The achieved SSIMULACRA2 score, when the engine could measure it — i.e. when
    /// a **perceptual search** ran, which requires decoding candidates back.
    ///
    /// `undefined` for AVIF and for lossless output. That is not a gap in the
    /// plumbing, it is DEC-065: the wasm build encodes AVIF but cannot decode it, so
    /// it cannot score its own AVIF, and inventing a number here would be a lie. To
    /// put a score on an AVIF output, the page decodes it with the browser
    /// (`createImageBitmap`) and calls [`score`] — which is exactly what SPEC-081
    /// does.
    #[wasm_bindgen(getter)]
    pub fn score(&self) -> Option<f64> {
        self.score
    }

    /// Where [`OptimizeResult::score`] came from: `"engine"` (a real SSIMULACRA2
    /// search inside wasm) or `"none"` (unscored — see `score`). A string rather
    /// than a bare `score == undefined` check so SPEC-081's UI can say *why* there
    /// is no number, and so a future `"browser"` provenance can be added without
    /// changing the shape.
    #[wasm_bindgen(getter, js_name = scoredBy)]
    pub fn scored_by(&self) -> String {
        self.scored_by.clone()
    }
}

/// The Auto-AVIF rule (SPEC-079, DEC-068): does this content bucket get AVIF, and
/// at what default quality?
///
/// `Some(AVIF_DEFAULT_QUALITY)` for a **lossy-family** bucket (`Lossy`, or the
/// ambiguous `MixedSafe` — `Profile::Web`, so no docs bias toward lossless) when
/// this build has an AVIF encoder; `None` otherwise, which falls back to today's
/// shortlist-first behaviour.
///
/// This is a narrow reuse of [`decide::format_shortlist`]'s **`Mode::SizeBudget`
/// AVIF admission** — "AVIF only where it is not perceptually searched, and only
/// for lossy-family content" (DEC-048) — restated as a predicate rather than read
/// off the shortlist, because `format_shortlist` truncates to `MAX_SHORTLIST` and
/// AVIF is *appended last*, so a `MixedSafe` image without alpha would lose it to
/// the truncation. The rule the demo needs is about the bucket, not about how many
/// other candidates happened to precede it.
///
/// Why this rule exists at all: `optimize` runs `Mode::Perceptual`, in which the
/// shortlist never admits AVIF (it cannot be decoded, so it cannot be scored), so a
/// photo fell through to a slow SSIMULACRA2 JPEG search that saved ~13 % — while the
/// AVIF the engine refused to consider is both much smaller and, at speed 10, much
/// faster to produce. That is the bug this rule fixes, and it is confined to this
/// wasm Auto path: the native `optimize` is untouched.
#[cfg(feature = "avif")]
fn auto_avif_quality(bucket: OptBucket) -> Option<u8> {
    matches!(bucket, OptBucket::Lossy | OptBucket::MixedSafe).then_some(sink::AVIF_DEFAULT_QUALITY)
}

/// Without the `avif` feature there is no AVIF encoder to pick (`WASM_CODECS.avif`
/// is `false`), so Auto behaves exactly as it did before this spec.
#[cfg(not(feature = "avif"))]
fn auto_avif_quality(_bucket: OptBucket) -> Option<u8> {
    None
}

/// The rav1e speed a call will actually encode AVIF at: the caller's, clamped to
/// `1..=10`, or the engine default. `None` for any other format — nothing else here
/// has a speed knob, and reporting one would imply it did.
fn resolved_speed(fmt: ImageFormat, requested: Option<u8>) -> Option<u8> {
    #[cfg(feature = "avif")]
    if fmt == ImageFormat::Avif {
        return Some(requested.unwrap_or(sink::AVIF_SPEED).clamp(1, 10));
    }
    let _ = (fmt, requested);
    None
}

/// Decode `input` and re-encode it well, reporting **what it did** — the surface
/// SPEC-080's demo and SPEC-081's quality readout are built on.
///
/// It is the same engine as [`optimize`] (analysis → shortlist → search → encode),
/// with three things `optimize` cannot express:
///
/// - **`speed`** — the rav1e encode speed for AVIF (1 = slowest/best … 10 = fastest;
///   default 6). A 12 MP photo takes ~33 s at speed 6 in a browser tab and ~9 s at
///   speed 10, for ~4 % more bytes. Ignored for every other format. The native CLI
///   still has no speed flag (DEC-020's deferral stands); this is a wasm-only knob.
/// - **`max_bytes`** — a byte budget. Runs the engine's **size** search (encode-only,
///   no decode) for the highest quality that fits, at the requested speed. Ignored
///   for a lossless target, which has no quality knob to search (fitting a lossless
///   file into a budget means resizing it, and that is a *choice a user makes* —
///   SPEC-080 offers it, this call will not do it behind their back).
/// - **`target`** — the SSIMULACRA2 target for the perceptual search on a searchable
///   lossy format (JPEG here). Defaults to [`DEFAULT_TARGET`].
///
/// **Auto picks AVIF for photographic input** (see `auto_avif_quality`) rather than
/// running the slow JPEG search the perceptual shortlist would have forced. A flat /
/// graphic image is untouched by that rule and still resolves to a lossless format.
///
/// Every failure — an undecodable input, an over-cap image (DEC-034/DEC-063), an
/// unencodable format — comes back as a typed `JsError`. Nothing here panics: a
/// panic aborts the wasm module and takes the page's engine instance with it.
#[wasm_bindgen(js_name = optimizeDetailed)]
pub fn optimize_detailed(
    input: &[u8],
    out_format: &str,
    speed: Option<u8>,
    max_bytes: Option<u32>,
    target: Option<f64>,
) -> Result<OptimizeResult, JsError> {
    let img = Image::from_bytes(input).map_err(js_err)?;

    // 1. Resolve the format — the caller's, or the engine's own choice for this
    //    image's content bucket (with the Auto-AVIF rule in front of the shortlist).
    let (fmt, disposition) = if out_format.is_empty() || out_format == "auto" {
        let analysis = Analysis::compute(&img).map_err(js_err)?;
        let bucket = analysis.opt_bucket();

        match auto_avif_quality(bucket) {
            Some(_) => (ImageFormat::Avif, Disposition::Lossy),
            None => {
                let shortlist = decide::format_shortlist(
                    bucket,
                    img.info().has_alpha,
                    Profile::Web,
                    Mode::Perceptual,
                    WASM_CODECS,
                );
                // Documented never-empty, but this surface must not `unwrap` on ANY
                // path — a panic aborts the module.
                let first = shortlist
                    .first()
                    .ok_or_else(|| JsError::new("no encodable format for this image"))?;
                (first.fmt, first.disposition)
            }
        }
    } else {
        let fmt = parse_format(out_format)?;
        let disposition = if fmt.supports_lossy_quality() {
            Disposition::Lossy
        } else {
            Disposition::Lossless
        };
        (fmt, disposition)
    };

    let speed = resolved_speed(fmt, speed);
    let lossy = disposition == Disposition::Lossy && fmt.supports_lossy_quality();

    // 2. Choose a quality, by whichever search the caller's arguments ask for.
    //
    //    * a BUDGET (and a lossy format) → the size search: encode-only, no decode,
    //      so it drives AVIF too. It MUST run at the same speed the sink will emit
    //      at, or the budget it reports describes bytes nobody writes (DEC-068's
    //      speed parity).
    //    * a PERCEPTUAL target (and a format we can decode back) → the SSIMULACRA2
    //      binary search — the expensive, honest one.
    //    * neither → the encoder's default quality. AVIF lands here: it has a
    //      quality knob but no decoder (DEC-065), so it can be size-searched but
    //      never perceptually searched.
    let (quality, engine_score) = match (max_bytes, lossy) {
        (Some(budget), true) => {
            let choice =
                quality::auto_under_size_at_speed(img.pixels(), fmt, u64::from(budget), speed)
                    .map_err(js_err)?;
            (Some(choice.quality), None)
        }
        (_, true) if fmt.supports_perceptual_quality() => {
            let cfg = SearchConfig::for_target(target.unwrap_or(DEFAULT_TARGET));
            let choice = quality::auto_quality(img.pixels(), fmt, &cfg).map_err(js_err)?;
            (Some(choice.quality), Some(choice.score))
        }
        (_, true) => (None, None),
        (_, false) => (None, None),
    };

    // 3. Encode the winner — through the SAME sink entry the byte-budget search
    //    probed, at the SAME speed, so the search's promise holds.
    let bytes = sink::encode_to_bytes_with(&img, fmt, quality, speed).map_err(js_err)?;

    // A size-searched candidate's `score` field is BYTES, not a perceptual score, so
    // only the perceptual search yields a number here — anything else reports "none"
    // rather than a fabricated one.
    let (score, scored_by) = match engine_score {
        Some(s) if s.is_finite() => (Some(s), "engine"),
        _ => (None, "none"),
    };

    Ok(OptimizeResult {
        bytes,
        format: format_name(fmt),
        // A lossy format encoded at its default (AVIF with no budget) still has a
        // real, reportable quality: the default the sink will use.
        quality: quality.or_else(|| default_quality_for(fmt)),
        speed,
        score,
        scored_by: scored_by.to_string(),
    })
}

/// The quality the sink encodes `fmt` at when no quality is given — reported so an
/// AVIF result can say "q80" instead of "undefined". `None` for a lossless format,
/// which genuinely has no quality.
fn default_quality_for(fmt: ImageFormat) -> Option<u8> {
    #[cfg(feature = "avif")]
    if fmt == ImageFormat::Avif {
        return Some(sink::AVIF_DEFAULT_QUALITY);
    }
    let _ = fmt;
    None
}

/// The SSIMULACRA2 score between two encoded images — the engine's perceptual
/// metric, exposed directly (SPEC-079; consumed by SPEC-081).
///
/// Both `reference` and `candidate` are decoded (so both must be formats this build
/// reads) and scored via the crate's public `quality::score`. Higher is better;
/// ~100 is visually identical.
///
/// This binding exists because the wasm build cannot score its own AVIF output — it
/// has an AVIF encoder and no AVIF decoder (DEC-065). The browser, however, decodes
/// AVIF natively, so a page can hand the pixels back: decode the output with
/// `createImageBitmap`, re-encode them to PNG on a canvas, and call
/// `score(originalBytes, pngOfDecodedOutput)`. The number is then the engine's own
/// metric, computed by the engine, on the real output — not an approximation.
///
/// Mismatched dimensions, an undecodable input, or an over-cap image return a typed
/// `JsError`.
#[wasm_bindgen]
pub fn score(reference: &[u8], candidate: &[u8]) -> Result<f64, JsError> {
    let reference = Image::from_bytes(reference).map_err(js_err)?;
    let candidate = Image::from_bytes(candidate).map_err(js_err)?;

    quality::score(reference.pixels(), candidate.pixels()).map_err(js_err)
}

/// The crate version, so a demo page can show which build it loaded.
#[wasm_bindgen]
pub fn version() -> String {
    crate::version().to_string()
}

/// The format's name for the JS side: the lowercased `ImageFormat` variant —
/// `"png"`, `"jpeg"`, `"webp"`, `"gif"`, `"bmp"`, `"tiff"`, `"ico"`.
///
/// Deliberately NOT `extensions_str().first()`, which yields `"jpg"` for JPEG: the
/// name a JS caller gets back from `info()` should be a name they can hand straight
/// to `transform()`/`optimize()`, and the variant name round-trips through
/// [`parse_format`] for every format the wasm build can encode.
fn format_name(fmt: ImageFormat) -> String {
    format!("{fmt:?}").to_lowercase()
}
