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
//! ## What is NOT here
//!
//! - **AVIF decode.** `re_rav1d` does not compile to bare wasm32 (DEC-064), so an
//!   AVIF input returns a typed error. SPEC-073 decides the restore path. Every
//!   other default input format — PNG, JPEG, GIF, BMP, TIFF, ICO, WebP, and **SVG**
//!   (resvg rasterizes in wasm) — works.
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
use crate::analysis::Analysis;
use crate::image::Image;
use crate::operation::OperationRegistry;
use crate::quality::{self, LossyFormat, SearchConfig};
use crate::recipe::Recipe;
use crate::sink;

/// The perceptual target `optimize` aims for when the caller doesn't say —
/// "visually lossless" on the SSIMULACRA2 scale, the same default the native
/// `optimize`/`shrink` commands use (DEC-019).
const DEFAULT_TARGET: f64 = 90.0;

/// Which optional codecs this build has. The wasm build is the pure-Rust default
/// set: no `webp-lossy` (a vendored C library) and no `avif` encode feature. Stated
/// once here so [`decide::format_shortlist`] shortlists only formats we can
/// actually encode.
const WASM_CODECS: BuiltCodecs = BuiltCodecs {
    webp_lossy: false,
    avif: false,
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

    // Lossless: nothing to search — encode and hand back the bytes.
    if disposition == Disposition::Lossless {
        return sink::encode_to_bytes(&img, fmt, None).map_err(js_err);
    }

    // Lossy: find the lowest quality that still reaches the perceptual target. This
    // is the real SSIMULACRA2 binary search (DEC-019) — the same code `shrink
    // --target` runs — so it decodes each candidate to score it. That is the
    // expensive part of the wasm build's runtime, and honestly so: it is what makes
    // the output good rather than merely small.
    let choice =
        quality::auto_quality(img.pixels(), fmt, &SearchConfig::for_target(DEFAULT_TARGET))
            .map_err(js_err)?;

    sink::encode_to_bytes(&img, fmt, Some(choice.quality)).map_err(js_err)
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
