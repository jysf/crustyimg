//! The `Operation` trait and its concrete implementations (DEC-002).
//!
//! Layering: this module depends only on `crate::image`, `std`, `thiserror`,
//! `serde`, `::image`, and `fast_image_resize`. It must NOT touch `clap`,
//! `recipe`, `source`, `sink`, `std::fs`, `std::path`, or any terminal types.
//! Ops are pure in-memory transforms (constraint `decode-once-no-per-op-disk`).
//!
//! # Module / crate name collision
//!
//! This crate has a `crate::image` module. Use `::image` to refer to the
//! pixel-library crate, the same convention as `src/image/mod.rs`.

use std::collections::BTreeMap;

use ::image::DynamicImage;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::image::Image;

pub mod registry;
pub use registry::{OperationRegistry, RegistryError};

// ─── OperationParams ────────────────────────────────────────────────────────

/// Operation parameters: an ordered map of TOML values (DEC-014).
///
/// Flatten-serialized into the `[[step]]` table by `RecipeStep`. An
/// empty map emits zero keys (so `op = "invert"` stays clean); a
/// populated map round-trips verbatim. Each `Operation` parses and
/// validates its own keys in its constructor via the typed accessors
/// below — there is no per-op logic in the serde impls (the flatten
/// boundary has no `op` context).
#[derive(Debug, Clone, PartialEq)]
pub struct OperationParams(BTreeMap<String, toml::Value>);

impl OperationParams {
    /// The empty param set (parameterless ops: Identity, Invert).
    pub fn empty() -> Self {
        OperationParams(BTreeMap::new())
    }

    /// Build from an ordered map (used by ops recording their params).
    pub fn from_map(map: BTreeMap<String, toml::Value>) -> Self {
        OperationParams(map)
    }

    /// Whether any params are present.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Borrow a string param, if present and a string.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(toml::Value::as_str)
    }

    /// Extract a `u32` param, if present and a non-negative integer in range.
    pub fn get_u32(&self, key: &str) -> Option<u32> {
        self.0
            .get(key)
            .and_then(toml::Value::as_integer)
            .and_then(|i| u32::try_from(i).ok())
    }

    /// Extract an `f32` param, if present (accepts integer or float TOML).
    pub fn get_f32(&self, key: &str) -> Option<f32> {
        self.0.get(key).and_then(|v| match v {
            toml::Value::Float(f) => Some(*f as f32),
            toml::Value::Integer(i) => Some(*i as f32),
            _ => None,
        })
    }
}

impl Default for OperationParams {
    fn default() -> Self {
        OperationParams::empty()
    }
}

impl Serialize for OperationParams {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Emit the inner map directly: empty map → zero keys (keeps
        // `op = "invert"` clean), populated map → those keys verbatim.
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for OperationParams {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Deserialize into an ordered map and wrap it. A non-empty map is
        // valid (it holds some op's params); per-op validation happens later
        // in each op's constructor (DEC-014). The old "error on non-empty map"
        // branch is intentionally dropped here.
        let map: BTreeMap<String, toml::Value> = BTreeMap::deserialize(deserializer)?;
        Ok(OperationParams(map))
    }
}

// ─── OperationError ─────────────────────────────────────────────────────────

/// Errors that an `Operation` can raise while transforming an `Image` (DEC-007).
///
/// Typed and matchable; no `unwrap`/`expect`/`panic!` on recoverable paths
/// (constraint `no-unwrap-on-recoverable-paths`).
#[derive(Debug, Error)]
pub enum OperationError {
    /// The operation could not be applied to this image (e.g. an unsupported
    /// color type or an invalid parameter).
    #[error("operation '{op}' failed: {reason}")]
    Apply {
        /// Stable registry/recipe key for the failing operation.
        op: &'static str,
        /// Human-readable reason for the failure.
        reason: String,
    },
}

// ─── Operation trait ────────────────────────────────────────────────────────

/// The single pixel-transform extension point (DEC-002).
///
/// Keep this trait **small**: a stable `name` (the recipe/registry key),
/// serde-friendly `params` (so a recipe can record + replay), and a pure
/// in-memory `apply`. Implementors MUST NOT read or write disk (constraint
/// `decode-once-no-per-op-disk`) and MUST NOT depend on clap/recipes/
/// terminals.
pub trait Operation {
    /// Stable registry/recipe key, e.g. `"identity"`, `"invert"`.
    fn name(&self) -> &'static str;

    /// This operation's parameters. Parameterless ops return
    /// `OperationParams::empty()`.
    fn params(&self) -> OperationParams;

    /// Transform the image in memory. Pure: no disk I/O.
    ///
    /// Takes `img` by value (consuming it) and returns a new `Image`.
    /// `Pipeline::run` threads the return value into the next operation,
    /// so no intermediate clones are needed (decode-once, DEC-002).
    fn apply(&self, img: Image) -> Result<Image, OperationError>;
}

// ─── Identity ───────────────────────────────────────────────────────────────

/// No-op transform: returns the image unchanged.
///
/// Proves the trait + fold machinery and serves as the sentinel in pipeline
/// error-propagation tests.
pub struct Identity;

impl Operation for Identity {
    fn name(&self) -> &'static str {
        "identity"
    }

    fn params(&self) -> OperationParams {
        OperationParams::empty()
    }

    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        Ok(img)
    }
}

// ─── Invert ─────────────────────────────────────────────────────────────────

/// Per-channel value inversion on 8-bit RGBA (alpha preserved).
///
/// Converts to RGBA8, maps `[r, g, b, a] → [255-r, 255-g, 255-b, a]`
/// with a hand-written pixel loop, and wraps the result back into a
/// `DynamicImage`. No `imageproc` — hand-rolled as required by
/// constraint `single-image-library`.
pub struct Invert;

impl Operation for Invert {
    fn name(&self) -> &'static str {
        "invert"
    }

    fn params(&self) -> OperationParams {
        OperationParams::empty()
    }

    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        // Convert to RGBA8 (lossless for the inversion; going through RGBA8 is
        // intentionally simple — later ops can be color-type-aware).
        let mut buf = img.pixels().to_rgba8();
        for pixel in buf.pixels_mut() {
            let [r, g, b, a] = pixel.0;
            pixel.0 = [255 - r, 255 - g, 255 - b, a];
        }
        Ok(img.with_pixels(DynamicImage::ImageRgba8(buf)))
    }
}

// ─── Resize ─────────────────────────────────────────────────────────────────

/// Mode of a Resize operation (the six geometry strategies).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResizeMode {
    Max,
    Exact,
    Percent,
    Fit,
    Fill,
    Cover,
}

/// Geometric resize on the fast_image_resize SIMD backend (DEC-008).
///
/// Constructed FROM params via `Resize::from_params` (the registry
/// path). Converts to RGBA8 (like Invert), resizes (Lanczos3
/// convolution), and—for `fill`—center-crops to the exact box.
#[derive(Debug)]
pub struct Resize {
    mode: ResizeMode,
    /// Per-mode target inputs (see the param schema). Carried so
    /// `params()` round-trips back to the same recipe step.
    width: Option<u32>,
    height: Option<u32>,
    percent: Option<f32>,
}

impl Resize {
    /// Parse + validate params (DEC-014). Returns a typed
    /// `RegistryError::InvalidParams` on a missing/wrong/out-of-range
    /// param. Never panics.
    pub fn from_params(params: &OperationParams) -> Result<Self, RegistryError> {
        let mode_str = params
            .get_str("mode")
            .ok_or_else(|| RegistryError::InvalidParams {
                op: "resize",
                reason: "missing required param 'mode'".to_owned(),
            })?;

        let mode = match mode_str {
            "max" => ResizeMode::Max,
            "exact" => ResizeMode::Exact,
            "percent" => ResizeMode::Percent,
            "fit" => ResizeMode::Fit,
            "fill" => ResizeMode::Fill,
            "cover" => ResizeMode::Cover,
            other => {
                return Err(RegistryError::InvalidParams {
                    op: "resize",
                    reason: format!("unknown mode '{other}'; expected one of: max, exact, percent, fit, fill, cover"),
                });
            }
        };

        match mode {
            ResizeMode::Max => {
                let width = params.get_u32("width").ok_or_else(|| RegistryError::InvalidParams {
                    op: "resize",
                    reason: "mode 'max' requires a positive integer 'width' param (the long-edge cap)".to_owned(),
                })?;
                if width == 0 {
                    return Err(RegistryError::InvalidParams {
                        op: "resize",
                        reason: "mode 'max' requires 'width' > 0".to_owned(),
                    });
                }
                Ok(Resize {
                    mode,
                    width: Some(width),
                    height: None,
                    percent: None,
                })
            }

            ResizeMode::Exact | ResizeMode::Fit | ResizeMode::Fill | ResizeMode::Cover => {
                let width =
                    params
                        .get_u32("width")
                        .ok_or_else(|| RegistryError::InvalidParams {
                            op: "resize",
                            reason: format!(
                                "mode '{}' requires a positive integer 'width' param",
                                mode_str
                            ),
                        })?;
                if width == 0 {
                    return Err(RegistryError::InvalidParams {
                        op: "resize",
                        reason: format!("mode '{}' requires 'width' > 0", mode_str),
                    });
                }
                let height =
                    params
                        .get_u32("height")
                        .ok_or_else(|| RegistryError::InvalidParams {
                            op: "resize",
                            reason: format!(
                                "mode '{}' requires a positive integer 'height' param",
                                mode_str
                            ),
                        })?;
                if height == 0 {
                    return Err(RegistryError::InvalidParams {
                        op: "resize",
                        reason: format!("mode '{}' requires 'height' > 0", mode_str),
                    });
                }
                Ok(Resize {
                    mode,
                    width: Some(width),
                    height: Some(height),
                    percent: None,
                })
            }

            ResizeMode::Percent => {
                let percent =
                    params
                        .get_f32("percent")
                        .ok_or_else(|| RegistryError::InvalidParams {
                            op: "resize",
                            reason: "mode 'percent' requires a positive numeric 'percent' param"
                                .to_owned(),
                        })?;
                if percent <= 0.0 {
                    return Err(RegistryError::InvalidParams {
                        op: "resize",
                        reason: "mode 'percent' requires 'percent' > 0.0".to_owned(),
                    });
                }
                Ok(Resize {
                    mode,
                    width: None,
                    height: None,
                    percent: Some(percent),
                })
            }
        }
    }
}

impl Operation for Resize {
    fn name(&self) -> &'static str {
        "resize"
    }

    fn params(&self) -> OperationParams {
        // Reconstruct the minimal map the mode needs so `params()` round-trips.
        let mut map = BTreeMap::new();
        let mode_str = match self.mode {
            ResizeMode::Max => "max",
            ResizeMode::Exact => "exact",
            ResizeMode::Percent => "percent",
            ResizeMode::Fit => "fit",
            ResizeMode::Fill => "fill",
            ResizeMode::Cover => "cover",
        };
        map.insert("mode".to_owned(), toml::Value::String(mode_str.to_owned()));

        match self.mode {
            ResizeMode::Max => {
                if let Some(w) = self.width {
                    map.insert("width".to_owned(), toml::Value::Integer(w as i64));
                }
            }
            ResizeMode::Exact | ResizeMode::Fit | ResizeMode::Fill | ResizeMode::Cover => {
                if let Some(w) = self.width {
                    map.insert("width".to_owned(), toml::Value::Integer(w as i64));
                }
                if let Some(h) = self.height {
                    map.insert("height".to_owned(), toml::Value::Integer(h as i64));
                }
            }
            ResizeMode::Percent => {
                if let Some(p) = self.percent {
                    // Use Integer when the value has no fractional part, Float otherwise.
                    if p.fract() == 0.0 {
                        map.insert("percent".to_owned(), toml::Value::Integer(p as i64));
                    } else {
                        map.insert("percent".to_owned(), toml::Value::Float(p as f64));
                    }
                }
            }
        }

        OperationParams::from_map(map)
    }

    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        let rgba = img.pixels().to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());

        // ── Compute target dimensions per the EXACT six-mode math ────────────
        let (tw, th) = match self.mode {
            ResizeMode::Max => {
                // s = min(N / max(w, h), 1.0) — never upscale.
                let n = self.width.unwrap() as f64;
                let longest = w.max(h) as f64;
                let s = (n / longest).min(1.0);
                let tw = ((w as f64 * s).round() as u32).max(1);
                let th = ((h as f64 * s).round() as u32).max(1);
                (tw, th)
            }
            ResizeMode::Exact => {
                // Force exactly W×H; aspect ignored.
                (self.width.unwrap(), self.height.unwrap())
            }
            ResizeMode::Percent => {
                // tw = round(w · P/100), th = round(h · P/100).
                let p = self.percent.unwrap() as f64 / 100.0;
                let tw = ((w as f64 * p).round() as u32).max(1);
                let th = ((h as f64 * p).round() as u32).max(1);
                (tw, th)
            }
            ResizeMode::Fit => {
                // s = min(W/w, H/h, 1.0) — fit inside, never upscale.
                let cap_w = self.width.unwrap() as f64;
                let cap_h = self.height.unwrap() as f64;
                let s = (cap_w / w as f64).min(cap_h / h as f64).min(1.0);
                let tw = ((w as f64 * s).round() as u32).max(1);
                let th = ((h as f64 * s).round() as u32).max(1);
                (tw, th)
            }
            ResizeMode::Cover => {
                // s = max(W/w, H/h) — cover the box, may upscale, no crop.
                let cap_w = self.width.unwrap() as f64;
                let cap_h = self.height.unwrap() as f64;
                let s = (cap_w / w as f64).max(cap_h / h as f64);
                let tw = ((w as f64 * s).round() as u32).max(1);
                let th = ((h as f64 * s).round() as u32).max(1);
                (tw, th)
            }
            ResizeMode::Fill => {
                // Compute the cover dims (resize to, then we crop below).
                let cap_w = self.width.unwrap() as f64;
                let cap_h = self.height.unwrap() as f64;
                let s = (cap_w / w as f64).max(cap_h / h as f64);
                let tw = ((w as f64 * s).round() as u32).max(1);
                let th = ((h as f64 * s).round() as u32).max(1);
                (tw, th)
            }
        };

        // ── Oversize cap (untrusted-input-hardening) ─────────────────────────
        // For fill, we cap the cover dims (tw, th) before allocating.
        const MAX_EDGE: u32 = 50_000;
        const MAX_AREA: u64 = 268_435_456; // 256 * 1024 * 1024

        if tw > MAX_EDGE || th > MAX_EDGE {
            return Err(OperationError::Apply {
                op: "resize",
                reason: format!(
                    "target dimensions {tw}×{th} exceed the maximum edge limit of {MAX_EDGE} px"
                ),
            });
        }
        if (tw as u64) * (th as u64) > MAX_AREA {
            return Err(OperationError::Apply {
                op: "resize",
                reason: format!(
                    "target area {tw}×{th} = {} px exceeds the maximum area of {MAX_AREA} px",
                    (tw as u64) * (th as u64)
                ),
            });
        }

        // ── Resize via fast_image_resize (VERIFIED 5.5.0 API block) ─────────
        let dw = tw;
        let dh = th;

        let src = fast_image_resize::images::Image::from_vec_u8(
            w,
            h,
            rgba.into_raw(),
            fast_image_resize::PixelType::U8x4,
        )
        .map_err(|e| OperationError::Apply {
            op: "resize",
            reason: e.to_string(),
        })?;

        let mut dst =
            fast_image_resize::images::Image::new(dw, dh, fast_image_resize::PixelType::U8x4);

        let mut resizer = fast_image_resize::Resizer::new();
        let opts = fast_image_resize::ResizeOptions::new().resize_alg(
            fast_image_resize::ResizeAlg::Convolution(fast_image_resize::FilterType::Lanczos3),
        );
        resizer
            .resize(&src, &mut dst, &opts)
            .map_err(|e| OperationError::Apply {
                op: "resize",
                reason: e.to_string(),
            })?;

        let out =
            ::image::RgbaImage::from_raw(dw, dh, dst.into_vec()).ok_or(OperationError::Apply {
                op: "resize",
                reason: "buffer/dim mismatch".into(),
            })?;

        // ── For fill: center-crop to the exact target W×H ───────────────────
        if self.mode == ResizeMode::Fill {
            let target_w = self.width.unwrap();
            let target_h = self.height.unwrap();
            // rw/rh are the cover dims (dw/dh from the resize above).
            let rw = dw;
            let rh = dh;
            // Centered offsets, clamped ≥ 0.
            let x = (rw.saturating_sub(target_w)) / 2;
            let y = (rh.saturating_sub(target_h)) / 2;
            let cropped = ::image::imageops::crop_imm(&out, x, y, target_w, target_h).to_image();
            return Ok(img.with_pixels(DynamicImage::ImageRgba8(cropped)));
        }

        Ok(img.with_pixels(DynamicImage::ImageRgba8(out)))
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ::image::{DynamicImage, ImageFormat, RgbaImage};

    use crate::image::Image;

    /// Build a small in-memory RGBA `Image` from `RgbaImage::from_fn`.
    fn make_image(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Image {
        let buf = RgbaImage::from_fn(w, h, |x, y| ::image::Rgba(f(x, y)));
        Image::from_parts(DynamicImage::ImageRgba8(buf), ImageFormat::Png, None)
    }

    // ── migrated SPEC-003 tests ───────────────────────────────────────────────

    #[test]
    fn identity_name_and_params_are_stable() {
        assert_eq!(Identity.name(), "identity");
        assert_eq!(Identity.params(), OperationParams::empty());
    }

    #[test]
    fn invert_name_is_stable() {
        assert_eq!(Invert.name(), "invert");
    }

    #[test]
    fn identity_returns_pixels_unchanged() {
        let img = make_image(2, 2, |x, y| [(x * 10) as u8, (y * 20) as u8, 50, 200]);
        let original_raw = img.pixels().to_rgba8().into_raw();
        let result = Identity.apply(img).unwrap();
        assert_eq!(result.pixels().to_rgba8().into_raw(), original_raw);
    }

    #[test]
    fn invert_complements_each_channel_preserving_alpha() {
        let img = make_image(2, 2, |x, y| {
            if x == 0 && y == 0 {
                [10, 20, 30, 128]
            } else {
                [0, 0, 0, 255]
            }
        });
        let result = Invert.apply(img).unwrap();
        let raw = result.pixels().to_rgba8().into_raw();
        // Pixel (0,0): [10,20,30,128] → [245,235,225,128]
        assert_eq!(&raw[0..4], &[245, 235, 225, 128]);
    }

    #[test]
    fn invert_is_involutive() {
        let img = make_image(3, 3, |x, y| {
            [(x * 30 + 5) as u8, (y * 20 + 10) as u8, 100, 200]
        });
        let original_raw = img.pixels().to_rgba8().into_raw();
        let once = Invert.apply(img).unwrap();
        let twice = Invert.apply(once).unwrap();
        assert_eq!(twice.pixels().to_rgba8().into_raw(), original_raw);
    }

    // ── SPEC-010 unit tests ───────────────────────────────────────────────────

    /// Helper: build OperationParams from a slice of (key, value) pairs.
    fn params_from_pairs(pairs: &[(&str, toml::Value)]) -> OperationParams {
        let map: BTreeMap<String, toml::Value> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        OperationParams::from_map(map)
    }

    #[test]
    fn params_empty_is_parameterless() {
        assert!(OperationParams::empty().is_empty());
        assert_eq!(Identity.params(), OperationParams::empty());
        assert_eq!(Invert.params(), OperationParams::empty());
    }

    #[test]
    fn resize_max_exact_dims() {
        // 100×50, max N=20 → s=0.2 → 20×10
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("max".into())),
            ("width", toml::Value::Integer(20)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let img = make_image(100, 50, |_, _| [128, 64, 32, 255]);
        let result = op.apply(img).unwrap();
        assert_eq!(result.pixels().width(), 20);
        assert_eq!(result.pixels().height(), 10);
    }

    #[test]
    fn resize_max_no_upscale() {
        // 40×30, max N=100 → s clamps to 1.0 → 40×30
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("max".into())),
            ("width", toml::Value::Integer(100)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let img = make_image(40, 30, |_, _| [128, 64, 32, 255]);
        let result = op.apply(img).unwrap();
        assert_eq!(result.pixels().width(), 40);
        assert_eq!(result.pixels().height(), 30);
    }

    #[test]
    fn resize_exact_exact_dims() {
        // 100×50, exact 33×77 → 33×77 (aspect ignored)
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("exact".into())),
            ("width", toml::Value::Integer(33)),
            ("height", toml::Value::Integer(77)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let img = make_image(100, 50, |_, _| [128, 64, 32, 255]);
        let result = op.apply(img).unwrap();
        assert_eq!(result.pixels().width(), 33);
        assert_eq!(result.pixels().height(), 77);
    }

    #[test]
    fn resize_percent_exact_dims() {
        // 100×50, percent 50 → 50×25
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("percent".into())),
            ("percent", toml::Value::Integer(50)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let img = make_image(100, 50, |_, _| [128, 64, 32, 255]);
        let result = op.apply(img).unwrap();
        assert_eq!(result.pixels().width(), 50);
        assert_eq!(result.pixels().height(), 25);
    }

    #[test]
    fn resize_fit_exact_dims() {
        // 100×50, fit 40×40 → s=min(40/100,40/50,1)=0.4 → 40×20
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("fit".into())),
            ("width", toml::Value::Integer(40)),
            ("height", toml::Value::Integer(40)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let img = make_image(100, 50, |_, _| [128, 64, 32, 255]);
        let result = op.apply(img).unwrap();
        assert_eq!(result.pixels().width(), 40);
        assert_eq!(result.pixels().height(), 20);
    }

    #[test]
    fn resize_fit_no_upscale() {
        // 30×20, fit 300×300 → s clamps to 1.0 → 30×20
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("fit".into())),
            ("width", toml::Value::Integer(300)),
            ("height", toml::Value::Integer(300)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let img = make_image(30, 20, |_, _| [128, 64, 32, 255]);
        let result = op.apply(img).unwrap();
        assert_eq!(result.pixels().width(), 30);
        assert_eq!(result.pixels().height(), 20);
    }

    #[test]
    fn resize_cover_exact_dims() {
        // 100×50, cover 40×40 → s=max(40/100,40/50)=0.8 → 80×40 (no crop)
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("cover".into())),
            ("width", toml::Value::Integer(40)),
            ("height", toml::Value::Integer(40)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let img = make_image(100, 50, |_, _| [128, 64, 32, 255]);
        let result = op.apply(img).unwrap();
        assert_eq!(result.pixels().width(), 80);
        assert_eq!(result.pixels().height(), 40);
    }

    #[test]
    fn resize_cover_may_upscale() {
        // 20×10, cover 100×100 → s=max(100/20,100/10)=10 → 200×100
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("cover".into())),
            ("width", toml::Value::Integer(100)),
            ("height", toml::Value::Integer(100)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let img = make_image(20, 10, |_, _| [128, 64, 32, 255]);
        let result = op.apply(img).unwrap();
        assert_eq!(result.pixels().width(), 200);
        assert_eq!(result.pixels().height(), 100);
    }

    #[test]
    fn resize_fill_center_crops_exact() {
        // 100×50, fill 40×40 → cover s=0.8 → 80×40, then center-crop → 40×40
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("fill".into())),
            ("width", toml::Value::Integer(40)),
            ("height", toml::Value::Integer(40)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let img = make_image(100, 50, |_, _| [128, 64, 32, 255]);
        let result = op.apply(img).unwrap();
        assert_eq!(result.pixels().width(), 40);
        assert_eq!(result.pixels().height(), 40);
    }

    #[test]
    fn resize_fill_crop_is_centered() {
        // Build a source where the center column has a distinct color (red).
        // 100×50 source: left 50 columns are blue, right 50 columns are red.
        // After fill to 40×40:
        //   - cover: s = max(40/100, 40/50) = 0.8 → 80×40
        //   - center-crop x = (80-40)/2 = 20 from left of the cover image.
        //   - The cover image: blue on left 40 px, red on right 40 px.
        //   - After cropping x=20..60 of the cover image (columns 20-59),
        //     we get the center 40 px. The original left/right boundary at 50px
        //     maps to cover column 50*0.8=40, so in the crop [20..60] it's at
        //     column 40-20=20 inside the result. Pixels left of 20 are blue,
        //     right of 20 are red.
        //
        // Key assertion: the FIRST column of the result is NOT from the leftmost
        // edge of the source (proving we didn't do a top-left crop).
        let img = make_image(100, 50, |x, _y| {
            if x < 50 {
                [0, 0, 255, 255]
            } else {
                [255, 0, 0, 255]
            }
        });
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("fill".into())),
            ("width", toml::Value::Integer(40)),
            ("height", toml::Value::Integer(40)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let result = op.apply(img).unwrap();
        let out = result.pixels().to_rgba8();

        // The result must be exactly 40×40.
        assert_eq!(out.width(), 40);
        assert_eq!(out.height(), 40);

        // The first column of the result should NOT be pure-blue (which would
        // indicate a top-left crop). With a centered crop, the first column
        // maps to a region that includes blue-ish pixels — but the leftmost
        // pixel of the result (at x=0) is from the cover image at x=20,
        // which maps back to source x ≈ 25 — still blue. However at x=20 in
        // the result (cover x=40 → source x=50) we start transitioning to red.
        // Confirm at least one right-side column is reddish (not all blue).
        let rightmost_row0 = out.get_pixel(39, 0);
        // rightmost pixel maps to cover column 59, source x ≈ 73.75 → red
        assert!(
            rightmost_row0.0[0] > rightmost_row0.0[2],
            "rightmost column of fill result should be reddish (r={} g={} b={})",
            rightmost_row0.0[0],
            rightmost_row0.0[1],
            rightmost_row0.0[2]
        );
        // And leftmost column is bluish.
        let leftmost_row0 = out.get_pixel(0, 0);
        assert!(
            leftmost_row0.0[2] > leftmost_row0.0[0],
            "leftmost column of fill result should be bluish (r={} g={} b={})",
            leftmost_row0.0[0],
            leftmost_row0.0[1],
            leftmost_row0.0[2]
        );
    }

    #[test]
    fn resize_parity_within_tolerance() {
        // 64×48 gradient fixture → resize to 32×24.
        // Compare fast_image_resize (Lanczos3) vs image::imageops::resize (Lanczos3).
        // Mean per-channel abs diff should be <= 6.0 (in practice much lower; ~1-2).
        use ::image::imageops::FilterType;

        let img = make_image(64, 48, |x, y| {
            [(x * 4) as u8, (y * 5) as u8, ((x + y) * 2) as u8, 255]
        });

        // fast_image_resize path.
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("exact".into())),
            ("width", toml::Value::Integer(32)),
            ("height", toml::Value::Integer(24)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let fir_result = op.apply(img.clone()).unwrap();
        let fir_raw = fir_result.pixels().to_rgba8();

        // image::imageops oracle.
        let oracle = ::image::imageops::resize(img.pixels(), 32, 24, FilterType::Lanczos3);

        assert_eq!(fir_raw.width(), oracle.width());
        assert_eq!(fir_raw.height(), oracle.height());

        let fir_bytes = fir_raw.into_raw();
        let oracle_bytes = oracle.into_raw();
        assert_eq!(fir_bytes.len(), oracle_bytes.len());

        let total_diff: f64 = fir_bytes
            .iter()
            .zip(oracle_bytes.iter())
            .map(|(&a, &b)| (a as f64 - b as f64).abs())
            .sum();
        let mean_diff = total_diff / fir_bytes.len() as f64;

        // Tolerance: <= 6.0 per channel on 0-255 scale.
        // In practice, Lanczos3 implementations differ only in SIMD rounding, so
        // observed mean is typically < 2.0.
        assert!(
            mean_diff <= 6.0,
            "mean per-channel abs diff {mean_diff:.3} exceeds tolerance 6.0"
        );
    }

    #[test]
    fn resize_oversize_is_typed_error() {
        // Edge > 50_000: exact 60_000×10
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("exact".into())),
            ("width", toml::Value::Integer(60_000)),
            ("height", toml::Value::Integer(10)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let img = make_image(10, 10, |_, _| [128, 64, 32, 255]);
        let result = op.apply(img);
        assert!(
            matches!(result, Err(OperationError::Apply { op: "resize", .. })),
            "expected Apply error for oversize edge, got {result:?}"
        );

        // Area > 268_435_456: 20_000×20_000 = 400_000_000
        let p2 = params_from_pairs(&[
            ("mode", toml::Value::String("exact".into())),
            ("width", toml::Value::Integer(20_000)),
            ("height", toml::Value::Integer(20_000)),
        ]);
        let op2 = Resize::from_params(&p2).unwrap();
        let img2 = make_image(10, 10, |_, _| [128, 64, 32, 255]);
        let result2 = op2.apply(img2);
        assert!(
            matches!(result2, Err(OperationError::Apply { op: "resize", .. })),
            "expected Apply error for oversize area, got {result2:?}"
        );
    }

    #[test]
    fn resize_from_params_missing_mode() {
        let p = OperationParams::empty();
        let result = Resize::from_params(&p);
        assert!(
            matches!(
                result,
                Err(RegistryError::InvalidParams { op: "resize", .. })
            ),
            "expected InvalidParams for missing mode, got {result:?}"
        );
    }

    #[test]
    fn resize_from_params_unknown_mode() {
        let p = params_from_pairs(&[("mode", toml::Value::String("bogus".into()))]);
        let result = Resize::from_params(&p);
        assert!(
            matches!(
                result,
                Err(RegistryError::InvalidParams { op: "resize", .. })
            ),
            "expected InvalidParams for unknown mode, got {result:?}"
        );
    }

    #[test]
    fn resize_from_params_missing_dim() {
        // exact mode, only width provided — missing height.
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("exact".into())),
            ("width", toml::Value::Integer(100)),
        ]);
        let result = Resize::from_params(&p);
        assert!(
            matches!(
                result,
                Err(RegistryError::InvalidParams { op: "resize", .. })
            ),
            "expected InvalidParams for missing height, got {result:?}"
        );
    }

    #[test]
    fn resize_from_params_nonpositive_dim() {
        // exact mode, width=0
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("exact".into())),
            ("width", toml::Value::Integer(0)),
            ("height", toml::Value::Integer(10)),
        ]);
        let result = Resize::from_params(&p);
        assert!(
            matches!(
                result,
                Err(RegistryError::InvalidParams { op: "resize", .. })
            ),
            "expected InvalidParams for width=0, got {result:?}"
        );
    }

    #[test]
    fn resize_params_round_trips_max() {
        // from_params{mode="max", width=1200} → params() == {mode:"max", width:1200}, NO height
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("max".into())),
            ("width", toml::Value::Integer(1200)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let round_tripped = op.params();
        assert_eq!(round_tripped.get_str("mode"), Some("max"));
        assert_eq!(round_tripped.get_u32("width"), Some(1200));
        // height must NOT be present for max mode
        assert!(round_tripped.0.get("height").is_none());
        // The reconstructed params must equal the original
        assert_eq!(round_tripped, p);
    }

    #[test]
    fn invert_params_still_zero_keys() {
        assert!(Invert.params().is_empty());
    }
}
