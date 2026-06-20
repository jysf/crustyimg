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
use std::fmt;
use std::str::FromStr;

use ::image::imageops::{self, FilterType};
use ::image::{DynamicImage, RgbaImage};
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
                let n = self.width.ok_or_else(|| OperationError::Apply {
                    op: "resize",
                    reason: "internal: resize mode 'max' requires width".into(),
                })? as f64;
                let longest = w.max(h) as f64;
                let s = (n / longest).min(1.0);
                let tw = ((w as f64 * s).round() as u32).max(1);
                let th = ((h as f64 * s).round() as u32).max(1);
                (tw, th)
            }
            ResizeMode::Exact => {
                // Force exactly W×H; aspect ignored.
                let tw = self.width.ok_or_else(|| OperationError::Apply {
                    op: "resize",
                    reason: "internal: resize mode 'exact' requires width".into(),
                })?;
                let th = self.height.ok_or_else(|| OperationError::Apply {
                    op: "resize",
                    reason: "internal: resize mode 'exact' requires height".into(),
                })?;
                (tw, th)
            }
            ResizeMode::Percent => {
                // tw = round(w · P/100), th = round(h · P/100).
                let p = self.percent.ok_or_else(|| OperationError::Apply {
                    op: "resize",
                    reason: "internal: resize mode 'percent' requires percent".into(),
                })? as f64
                    / 100.0;
                let tw = ((w as f64 * p).round() as u32).max(1);
                let th = ((h as f64 * p).round() as u32).max(1);
                (tw, th)
            }
            ResizeMode::Fit => {
                // s = min(W/w, H/h, 1.0) — fit inside, never upscale.
                let cap_w = self.width.ok_or_else(|| OperationError::Apply {
                    op: "resize",
                    reason: "internal: resize mode 'fit' requires width".into(),
                })? as f64;
                let cap_h = self.height.ok_or_else(|| OperationError::Apply {
                    op: "resize",
                    reason: "internal: resize mode 'fit' requires height".into(),
                })? as f64;
                let s = (cap_w / w as f64).min(cap_h / h as f64).min(1.0);
                let tw = ((w as f64 * s).round() as u32).max(1);
                let th = ((h as f64 * s).round() as u32).max(1);
                (tw, th)
            }
            ResizeMode::Cover => {
                // s = max(W/w, H/h) — cover the box, may upscale, no crop.
                let cap_w = self.width.ok_or_else(|| OperationError::Apply {
                    op: "resize",
                    reason: "internal: resize mode 'cover' requires width".into(),
                })? as f64;
                let cap_h = self.height.ok_or_else(|| OperationError::Apply {
                    op: "resize",
                    reason: "internal: resize mode 'cover' requires height".into(),
                })? as f64;
                let s = (cap_w / w as f64).max(cap_h / h as f64);
                let tw = ((w as f64 * s).round() as u32).max(1);
                let th = ((h as f64 * s).round() as u32).max(1);
                (tw, th)
            }
            ResizeMode::Fill => {
                // Compute the cover dims (resize to, then we crop below).
                let cap_w = self.width.ok_or_else(|| OperationError::Apply {
                    op: "resize",
                    reason: "internal: resize mode 'fill' requires width".into(),
                })? as f64;
                let cap_h = self.height.ok_or_else(|| OperationError::Apply {
                    op: "resize",
                    reason: "internal: resize mode 'fill' requires height".into(),
                })? as f64;
                let s = (cap_w / w as f64).max(cap_h / h as f64);
                let tw = ((w as f64 * s).round() as u32).max(1);
                let th = ((h as f64 * s).round() as u32).max(1);
                (tw, th)
            }
        };

        // ── Oversize cap (untrusted-input-hardening, SPEC-010; tightened SPEC-037) ──
        // Reject before any resize backend allocates — one check covers all six modes
        // (including percent/cover/fill, whose output dims depend on the input).
        // MAX_AREA is the upscale-bomb defense; MAX_EDGE is a per-dimension sanity cap.
        const MAX_EDGE: u32 = 50_000;
        // 512 MiB RGBA output (== the decode allocation cap, DEC-034/DEC-038): a
        // resize cannot produce a buffer larger than what decode would accept.
        // Tightened from 256 Mpx to 128 Mpx for that symmetry (SPEC-037).
        const MAX_AREA: u64 = 134_217_728; // 128 * 1024 * 1024 px = 512 MiB at RGBA8

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
            let target_w = self.width.ok_or_else(|| OperationError::Apply {
                op: "resize",
                reason: "internal: resize mode 'fill' requires width for crop".into(),
            })?;
            let target_h = self.height.ok_or_else(|| OperationError::Apply {
                op: "resize",
                reason: "internal: resize mode 'fill' requires height for crop".into(),
            })?;
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

// ─── AutoOrient ─────────────────────────────────────────────────────────────

/// Bake the EXIF orientation tag into the pixels, then drop the metadata
/// bundle so the now-stale tag does not propagate (DEC-017).
///
/// Reads the raw EXIF segment from the `Image`'s captured [`MetadataBundle`],
/// extracts the orientation via `image`'s native
/// `Orientation::from_exif_chunk`, applies the corresponding
/// rotation/flip with `DynamicImage::apply_orientation`, and returns a new
/// `Image` with **no** metadata bundle (`None`). Images with no EXIF, no
/// orientation tag, or orientation 1 (`NoTransforms`) are returned unchanged
/// (no-op, exit 0 — not an error).
///
/// The operation module depends only on `::image`; NO `kamadak-exif`
/// (constraint `single-image-library`, DEC-013, DEC-017).
#[derive(Debug)]
pub struct AutoOrient;

impl Operation for AutoOrient {
    fn name(&self) -> &'static str {
        "auto-orient"
    }

    fn params(&self) -> OperationParams {
        OperationParams::empty()
    }

    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        let orientation = img
            .metadata()
            .and_then(|m| m.exif.as_deref())
            .and_then(orientation_from_exif_segment);

        match orientation {
            // No EXIF, no tag, or orientation 1 — return unchanged (no-op).
            None | Some(::image::metadata::Orientation::NoTransforms) => Ok(img),
            Some(o) => {
                // Clone the pixel buffer, apply the rotation/flip in-place,
                // then return a new Image with NO metadata bundle (DEC-017):
                // the orientation tag is now stale; dropping the bundle is
                // the correct, future-proof choice.
                let mut pixels = img.pixels().clone();
                pixels.apply_orientation(o);
                let fmt = img.source_format();
                Ok(Image::from_parts(pixels, fmt, None))
            }
        }
    }
}

/// Extract an `image::metadata::Orientation` from a raw EXIF segment.
///
/// Accepts both JPEG-style (with a leading `"Exif\0\0"` signature) and
/// PNG-style (bare TIFF, no prefix) segments. Strips the six-byte
/// `b"Exif\0\0"` prefix when present, then delegates to
/// `Orientation::from_exif_chunk`, which parses the raw TIFF chunk.
///
/// Returns `None` on any parse failure (missing tag, malformed bytes, empty
/// slice) — never panics. Directly unit-testable as a free fn.
fn orientation_from_exif_segment(exif: &[u8]) -> Option<::image::metadata::Orientation> {
    // Strip the JPEG APP1 "Exif\0\0" signature if present; PNG eXIf chunks
    // are already bare TIFF, so no stripping is needed for those.
    let tiff = exif.strip_prefix(b"Exif\0\0").unwrap_or(exif);
    ::image::metadata::Orientation::from_exif_chunk(tiff)
}

// ─── Gravity ──────────────────────────────────────────────────────────────────

/// A compass anchor for placing an overlay within a base image.
///
/// A shared `operation`-level concept (DEC-031): the placement math lives here
/// and is reusable by a future `crop` (AGENTS.md §14). `FromStr` parses the nine
/// lowercase names; `Display` renders them back, so `params()` round-trips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gravity {
    Center,
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl FromStr for Gravity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "center" => Ok(Gravity::Center),
            "north" => Ok(Gravity::North),
            "south" => Ok(Gravity::South),
            "east" => Ok(Gravity::East),
            "west" => Ok(Gravity::West),
            "northeast" => Ok(Gravity::NorthEast),
            "northwest" => Ok(Gravity::NorthWest),
            "southeast" => Ok(Gravity::SouthEast),
            "southwest" => Ok(Gravity::SouthWest),
            other => Err(format!(
                "unknown gravity '{other}'; expected one of: center, north, south, \
                 east, west, northeast, northwest, southeast, southwest"
            )),
        }
    }
}

impl fmt::Display for Gravity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Gravity::Center => "center",
            Gravity::North => "north",
            Gravity::South => "south",
            Gravity::East => "east",
            Gravity::West => "west",
            Gravity::NorthEast => "northeast",
            Gravity::NorthWest => "northwest",
            Gravity::SouthEast => "southeast",
            Gravity::SouthWest => "southwest",
        };
        f.write_str(s)
    }
}

impl Gravity {
    /// Compute the top-left `(x, y)` placement of an `ow×oh` overlay over a
    /// `bw×bh` base at this anchor, inset by `margin` from the anchored edges.
    ///
    /// `margin` is ignored for [`Gravity::Center`] (centering has no edge to
    /// inset from). Offsets saturate at zero so an oversized overlay still
    /// anchors at the origin rather than wrapping negative; `imageops::overlay`
    /// then clips any out-of-bounds region (probe-verified, no panic).
    fn placement(self, bw: u32, bh: u32, ow: u32, oh: u32, margin: u32) -> (i64, i64) {
        // Horizontal: left / center / right.
        let x_left = margin;
        let x_center = (bw.saturating_sub(ow)) / 2;
        let x_right = bw.saturating_sub(ow).saturating_sub(margin);
        // Vertical: top / center / bottom.
        let y_top = margin;
        let y_center = (bh.saturating_sub(oh)) / 2;
        let y_bottom = bh.saturating_sub(oh).saturating_sub(margin);

        let (x, y) = match self {
            Gravity::Center => (x_center, y_center),
            Gravity::North => (x_center, y_top),
            Gravity::South => (x_center, y_bottom),
            Gravity::East => (x_right, y_center),
            Gravity::West => (x_left, y_center),
            Gravity::NorthEast => (x_right, y_top),
            Gravity::NorthWest => (x_left, y_top),
            Gravity::SouthEast => (x_right, y_bottom),
            Gravity::SouthWest => (x_left, y_bottom),
        };
        (x as i64, y as i64)
    }
}

// ─── Watermark ────────────────────────────────────────────────────────────────

/// Composite an overlay image onto each base at a gravity anchor (SPEC-029).
///
/// The first `Operation` that composes a **second** image. Per DEC-031 the
/// overlay is loaded at the IO boundary (`run_watermark` in `src/cli/`) and
/// handed in as in-memory `DynamicImage` pixels, so `apply()` never touches a
/// file. The source `overlay_path` is stored purely so `params()` can serialize
/// it for the future recipe round-trip (STAGE-005).
///
/// Compositing uses `image::imageops` only (no new dependency): `overlay`
/// (source-over alpha), `resize` for `--scale`, and an alpha multiply for
/// `--opacity`.
pub struct Watermark {
    /// The decoded overlay pixels (loaded at the CLI boundary, DEC-031).
    overlay: DynamicImage,
    /// The overlay's source path, kept only for `params()` round-trip.
    overlay_path: String,
    gravity: Gravity,
    /// Alpha multiplier in `[0.0, 1.0]` (validated at the CLI).
    opacity: f32,
    /// Optional scale: overlay width = `scale × base width` (aspect preserved).
    scale: Option<f32>,
    /// Inset in px from the anchored edges (ignored for center / tile).
    margin: u32,
    /// Tile the overlay across the whole base (ignores gravity + margin).
    tile: bool,
}

impl Watermark {
    /// Build a `Watermark` from an already-decoded overlay and its placement.
    ///
    /// Validation of `opacity`/`scale`/`gravity` happens at the CLI boundary
    /// (`run_watermark`) before this is called, so the constructor is total.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        overlay: DynamicImage,
        overlay_path: String,
        gravity: Gravity,
        opacity: f32,
        scale: Option<f32>,
        margin: u32,
        tile: bool,
    ) -> Self {
        Watermark {
            overlay,
            overlay_path,
            gravity,
            opacity,
            scale,
            margin,
            tile,
        }
    }
}

impl Operation for Watermark {
    fn name(&self) -> &'static str {
        "watermark"
    }

    fn params(&self) -> OperationParams {
        let mut map = BTreeMap::new();
        map.insert(
            "image".to_owned(),
            toml::Value::String(self.overlay_path.clone()),
        );
        map.insert(
            "gravity".to_owned(),
            toml::Value::String(self.gravity.to_string()),
        );
        map.insert(
            "opacity".to_owned(),
            toml::Value::Float(self.opacity as f64),
        );
        if let Some(s) = self.scale {
            map.insert("scale".to_owned(), toml::Value::Float(s as f64));
        }
        map.insert(
            "margin".to_owned(),
            toml::Value::Integer(self.margin as i64),
        );
        map.insert("tile".to_owned(), toml::Value::Boolean(self.tile));
        OperationParams::from_map(map)
    }

    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        // Work in RGBA8 (source-over compositing needs an alpha channel).
        let mut canvas = img.pixels().to_rgba8();
        let mut ov: RgbaImage = self.overlay.to_rgba8();

        // ── Scale: overlay width = scale × base width (aspect preserved) ──────
        if let Some(s) = self.scale {
            let target_w = ((canvas.width() as f32 * s).round() as u32).max(1);
            // Preserve the overlay's aspect ratio.
            let (ow, oh) = (ov.width().max(1), ov.height().max(1));
            let target_h = (((target_w as f32) * (oh as f32) / (ow as f32)).round() as u32).max(1);
            ov = imageops::resize(&ov, target_w, target_h, FilterType::Lanczos3);
        }

        // ── Opacity: multiply the overlay's alpha channel ─────────────────────
        if self.opacity < 1.0 {
            for px in ov.pixels_mut() {
                px.0[3] = (px.0[3] as f32 * self.opacity).round() as u8;
            }
        }

        // ── Placement ─────────────────────────────────────────────────────────
        if self.tile {
            // Tile the overlay to cover the whole base. Edge tiles clip cleanly.
            let (ow, oh) = (ov.width().max(1), ov.height().max(1));
            let mut y = 0u32;
            while y < canvas.height() {
                let mut x = 0u32;
                while x < canvas.width() {
                    imageops::overlay(&mut canvas, &ov, x as i64, y as i64);
                    x += ow;
                }
                y += oh;
            }
        } else {
            let (x, y) = self.gravity.placement(
                canvas.width(),
                canvas.height(),
                ov.width(),
                ov.height(),
                self.margin,
            );
            imageops::overlay(&mut canvas, &ov, x, y);
        }

        Ok(img.with_pixels(DynamicImage::ImageRgba8(canvas)))
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
        assert!(!round_tripped.0.contains_key("height"));
        // The reconstructed params must equal the original
        assert_eq!(round_tripped, p);
    }

    #[test]
    fn invert_params_still_zero_keys() {
        assert!(Invert.params().is_empty());
    }

    // ── SPEC-015 AutoOrient unit tests ────────────────────────────────────────

    use crate::image::MetadataBundle;

    /// Build a minimal little-endian TIFF chunk with a single Orientation
    /// entry (tag 0x0112, type SHORT, count 1, value = `orientation`).
    /// This is the format `Orientation::from_exif_chunk` expects.
    fn tiff_orientation_chunk(orientation: u8) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&[0x49, 0x49]); // "II" — little-endian
        bytes.extend_from_slice(&[0x2A, 0x00]); // TIFF magic = 42
        bytes.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // IFD offset = 8
        bytes.extend_from_slice(&[0x01, 0x00]); // entry count = 1
        bytes.extend_from_slice(&[0x12, 0x01]); // tag 0x0112 = Orientation
        bytes.extend_from_slice(&[0x03, 0x00]); // type 3 = SHORT
        bytes.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // count = 1
        bytes.push(orientation);
        bytes.push(0x00); // value padding
        bytes.extend_from_slice(&[0x00, 0x00]); // value padding (SHORT is 2 bytes, stored in 4-byte value field)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // next-IFD offset = 0
        bytes
    }

    /// Build a JPEG-style EXIF bundle (prefixed with `b"Exif\0\0"`).
    fn jpeg_exif_bundle(orientation: u8) -> MetadataBundle {
        let mut exif_bytes: Vec<u8> = Vec::new();
        exif_bytes.extend_from_slice(b"Exif\0\0");
        exif_bytes.extend(tiff_orientation_chunk(orientation));
        MetadataBundle {
            exif: Some(exif_bytes),
            icc: None,
        }
    }

    /// Build a PNG-style EXIF bundle (bare TIFF, no prefix).
    fn png_exif_bundle(orientation: u8) -> MetadataBundle {
        MetadataBundle {
            exif: Some(tiff_orientation_chunk(orientation)),
            icc: None,
        }
    }

    #[test]
    fn auto_orient_name_and_params_stable() {
        assert_eq!(AutoOrient.name(), "auto-orient");
        assert_eq!(AutoOrient.params(), OperationParams::empty());
    }

    #[test]
    fn auto_orient_noop_without_metadata() {
        // A 4×2 image with no metadata → apply returns 4×2 unchanged.
        let img = make_image(4, 2, |x, y| [(x * 20) as u8, (y * 20) as u8, 50, 255]);
        let result = AutoOrient.apply(img).unwrap();
        assert_eq!(result.pixels().width(), 4);
        assert_eq!(result.pixels().height(), 2);
    }

    #[test]
    fn auto_orient_noop_on_orientation_1() {
        // Orientation 1 = NoTransforms → dims unchanged.
        let buf = RgbaImage::from_fn(4, 2, |x, y| {
            ::image::Rgba([(x * 20) as u8, (y * 40) as u8, 50, 255])
        });
        let img = Image::from_parts(
            DynamicImage::ImageRgba8(buf),
            ImageFormat::Jpeg,
            Some(jpeg_exif_bundle(1)),
        );
        let result = AutoOrient.apply(img).unwrap();
        assert_eq!(result.pixels().width(), 4);
        assert_eq!(result.pixels().height(), 2);
    }

    #[test]
    fn auto_orient_rotate90_swaps_dims() {
        // Orientation 6 = Rotate90 (EXIF 6 → image::Orientation::Rotate90).
        // A 4×2 image → after rotation: 2×4.
        // Also asserts the metadata bundle is dropped (DEC-017).
        let buf = RgbaImage::from_fn(4, 2, |x, y| {
            ::image::Rgba([(x * 20) as u8, (y * 40) as u8, 50, 255])
        });
        let img = Image::from_parts(
            DynamicImage::ImageRgba8(buf),
            ImageFormat::Jpeg,
            Some(jpeg_exif_bundle(6)),
        );
        let result = AutoOrient.apply(img).unwrap();
        assert_eq!(
            result.pixels().width(),
            2,
            "width should swap to 2 after rotate90"
        );
        assert_eq!(
            result.pixels().height(),
            4,
            "height should swap to 4 after rotate90"
        );
        assert!(
            result.metadata().is_none(),
            "metadata bundle must be dropped after baking orientation (DEC-017)"
        );
    }

    #[test]
    fn auto_orient_reads_png_style_exif_chunk() {
        // Bare TIFF (no Exif\0\0 prefix), orientation 6 → 4×2 becomes 2×4.
        let buf = RgbaImage::from_fn(4, 2, |x, y| {
            ::image::Rgba([(x * 20) as u8, (y * 40) as u8, 50, 255])
        });
        let img = Image::from_parts(
            DynamicImage::ImageRgba8(buf),
            ImageFormat::Png,
            Some(png_exif_bundle(6)),
        );
        let result = AutoOrient.apply(img).unwrap();
        assert_eq!(
            result.pixels().width(),
            2,
            "width should swap to 2 after rotate90 with bare TIFF"
        );
        assert_eq!(
            result.pixels().height(),
            4,
            "height should swap to 4 after rotate90 with bare TIFF"
        );
    }

    #[test]
    fn auto_orient_flip_horizontal_moves_pixels() {
        // Orientation 2 = FlipHorizontal: a 2×1 image with left=red, right=blue
        // → after flip, col 0 = blue, col 1 = red. Dims unchanged (2×1).
        let buf = RgbaImage::from_fn(2, 1, |x, _y| {
            if x == 0 {
                ::image::Rgba([255, 0, 0, 255]) // red on left
            } else {
                ::image::Rgba([0, 0, 255, 255]) // blue on right
            }
        });
        let img = Image::from_parts(
            DynamicImage::ImageRgba8(buf),
            ImageFormat::Jpeg,
            Some(jpeg_exif_bundle(2)),
        );
        let result = AutoOrient.apply(img).unwrap();
        assert_eq!(result.pixels().width(), 2, "width unchanged after flip");
        assert_eq!(result.pixels().height(), 1, "height unchanged after flip");

        let out = result.pixels().to_rgba8();
        let col0 = out.get_pixel(0, 0);
        let col1 = out.get_pixel(1, 0);
        // After flip: left should now be blue (was right), right should be red (was left).
        assert_eq!(col0.0, [0, 0, 255, 255], "col 0 should be blue after flip");
        assert_eq!(col1.0, [255, 0, 0, 255], "col 1 should be red after flip");
    }

    #[test]
    fn orientation_from_exif_segment_prefixed_and_bare() {
        use ::image::metadata::Orientation;

        let tiff = tiff_orientation_chunk(6);

        // Prefixed (JPEG APP1 style).
        let mut prefixed = Vec::new();
        prefixed.extend_from_slice(b"Exif\0\0");
        prefixed.extend_from_slice(&tiff);
        let result_prefixed = orientation_from_exif_segment(&prefixed);
        assert_eq!(
            result_prefixed,
            Some(Orientation::Rotate90),
            "prefixed orientation-6 should parse as Rotate90"
        );

        // Bare (PNG eXIf style).
        let result_bare = orientation_from_exif_segment(&tiff);
        assert_eq!(
            result_bare,
            Some(Orientation::Rotate90),
            "bare orientation-6 should parse as Rotate90"
        );

        // Garbage bytes → None.
        let garbage = b"this is not a tiff";
        assert!(
            orientation_from_exif_segment(garbage).is_none(),
            "garbage bytes should return None"
        );

        // Empty slice → None.
        assert!(
            orientation_from_exif_segment(&[]).is_none(),
            "empty slice should return None"
        );
    }

    // ── SPEC-029 Watermark unit tests ─────────────────────────────────────────

    /// Build a solid-color RGBA `DynamicImage` overlay fixture.
    fn solid_overlay(w: u32, h: u32, rgba: [u8; 4]) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, ::image::Rgba(rgba)))
    }

    /// Build a `Watermark` op directly from an in-memory overlay (DEC-031).
    #[allow(clippy::too_many_arguments)]
    fn watermark(
        overlay: DynamicImage,
        gravity: Gravity,
        opacity: f32,
        scale: Option<f32>,
        margin: u32,
        tile: bool,
    ) -> Watermark {
        Watermark::new(
            overlay,
            "logo.png".to_owned(),
            gravity,
            opacity,
            scale,
            margin,
            tile,
        )
    }

    #[test]
    fn gravity_parse_round_trips() {
        let names = [
            "center",
            "north",
            "south",
            "east",
            "west",
            "northeast",
            "northwest",
            "southeast",
            "southwest",
        ];
        for name in names {
            let g: Gravity = name.parse().expect("known gravity should parse");
            assert_eq!(g.to_string(), name, "Display should round-trip {name}");
        }
        // Junk → error.
        assert!(
            "sideways".parse::<Gravity>().is_err(),
            "unknown gravity should error"
        );
    }

    #[test]
    fn watermark_southeast_places_overlay() {
        // 20×20 red base, 4×4 blue overlay, SE, margin 0.
        let base = make_image(20, 20, |_, _| [255, 0, 0, 255]);
        let op = watermark(
            solid_overlay(4, 4, [0, 0, 255, 255]),
            Gravity::SouthEast,
            1.0,
            None,
            0,
            false,
        );
        let out = op.apply(base).unwrap();
        let buf = out.pixels().to_rgba8();
        // Bottom-right corner (18,18) is inside the 4×4 SE block (cols/rows 16..20).
        assert_eq!(
            buf.get_pixel(18, 18).0,
            [0, 0, 255, 255],
            "SE corner is blue"
        );
        // Top-left corner unchanged.
        assert_eq!(buf.get_pixel(0, 0).0, [255, 0, 0, 255], "NW corner is red");
    }

    #[test]
    fn watermark_center_places_overlay() {
        // 20×20 red base, 4×4 blue overlay, centered.
        let base = make_image(20, 20, |_, _| [255, 0, 0, 255]);
        let op = watermark(
            solid_overlay(4, 4, [0, 0, 255, 255]),
            Gravity::Center,
            1.0,
            None,
            0,
            false,
        );
        let out = op.apply(base).unwrap();
        let buf = out.pixels().to_rgba8();
        // Center pixel (10,10) is inside the centered block (cols 8..12).
        assert_eq!(buf.get_pixel(10, 10).0, [0, 0, 255, 255], "center is blue");
        // Corners untouched.
        assert_eq!(buf.get_pixel(0, 0).0, [255, 0, 0, 255], "NW corner is red");
        assert_eq!(
            buf.get_pixel(19, 19).0,
            [255, 0, 0, 255],
            "SE corner is red"
        );
    }

    #[test]
    fn watermark_opacity_blends() {
        // 20×20 red base, 4×4 blue overlay at half opacity → blend in the block.
        let base = make_image(20, 20, |_, _| [255, 0, 0, 255]);
        let op = watermark(
            solid_overlay(4, 4, [0, 0, 255, 255]),
            Gravity::SouthEast,
            0.5,
            None,
            0,
            false,
        );
        let out = op.apply(base).unwrap();
        let buf = out.pixels().to_rgba8();
        let p = buf.get_pixel(18, 18).0;
        // Neither pure red base nor pure blue overlay: a mix of both channels.
        assert!(p[0] > 0 && p[0] < 255, "red channel blended: {p:?}");
        assert!(p[2] > 0 && p[2] < 255, "blue channel blended: {p:?}");
    }

    #[test]
    fn watermark_scale_resizes_overlay() {
        // 20-wide base, 4×4 overlay, scale 0.5 → overlay ~10 px wide.
        let base = make_image(20, 20, |_, _| [255, 0, 0, 255]);
        let op = watermark(
            solid_overlay(4, 4, [0, 0, 255, 255]),
            Gravity::SouthEast,
            1.0,
            Some(0.5),
            0,
            false,
        );
        let out = op.apply(base).unwrap();
        let buf = out.pixels().to_rgba8();
        // The SE block is now ~10×10 (cols 10..20). A pixel at the left edge of
        // the block (col 11) on the bottom row should be blue — proving the
        // overlay spans much wider than the original 4 px.
        assert_eq!(
            buf.get_pixel(11, 19).0,
            [0, 0, 255, 255],
            "scaled overlay spans ~half width"
        );
        // And a pixel well left of the block (col 5) stays red.
        assert_eq!(
            buf.get_pixel(5, 19).0,
            [255, 0, 0, 255],
            "outside block is red"
        );
    }

    #[test]
    fn watermark_margin_offsets_anchor() {
        // 20×20 base, 4×4 overlay, SE, margin 2 → block shifted 2 px inward.
        let base = make_image(20, 20, |_, _| [255, 0, 0, 255]);
        let op = watermark(
            solid_overlay(4, 4, [0, 0, 255, 255]),
            Gravity::SouthEast,
            1.0,
            None,
            2,
            false,
        );
        let out = op.apply(base).unwrap();
        let buf = out.pixels().to_rgba8();
        // The SE corner pixel reverts to base (margin pushed the overlay inward).
        assert_eq!(
            buf.get_pixel(19, 19).0,
            [255, 0, 0, 255],
            "corner reverts to base with margin"
        );
        // The inset block (cols/rows 14..18) is overlay; (15,15) is blue.
        assert_eq!(
            buf.get_pixel(15, 15).0,
            [0, 0, 255, 255],
            "inset block is overlay"
        );
    }

    #[test]
    fn watermark_tile_covers_base() {
        // 20×20 base, 4×4 overlay, tiled → overlay color in far-apart regions.
        let base = make_image(20, 20, |_, _| [255, 0, 0, 255]);
        let op = watermark(
            solid_overlay(4, 4, [0, 0, 255, 255]),
            Gravity::SouthEast,
            1.0,
            None,
            0,
            true,
        );
        let out = op.apply(base).unwrap();
        let buf = out.pixels().to_rgba8();
        // Top-left and bottom-right both carry the overlay color.
        assert_eq!(buf.get_pixel(0, 0).0, [0, 0, 255, 255], "top-left tiled");
        assert_eq!(
            buf.get_pixel(18, 18).0,
            [0, 0, 255, 255],
            "bottom-right tiled"
        );
    }

    #[test]
    fn watermark_params_includes_path_and_placement() {
        let op = watermark(
            solid_overlay(4, 4, [0, 0, 255, 255]),
            Gravity::SouthEast,
            0.5,
            Some(0.25),
            3,
            true,
        );
        let p = op.params();
        assert_eq!(p.get_str("image"), Some("logo.png"));
        assert_eq!(p.get_str("gravity"), Some("southeast"));
        assert_eq!(p.get_f32("opacity"), Some(0.5));
        assert_eq!(p.get_f32("scale"), Some(0.25));
        assert_eq!(p.get_u32("margin"), Some(3));
        // `tile` is a bool key; confirm it is present and true.
        assert_eq!(p.0.get("tile"), Some(&toml::Value::Boolean(true)));
    }

    // ── SPEC-037 unit tests — resize output byte cap (DEC-038) ───────────────

    /// A 2×2 input, `exact 40000x40000` → rejected before allocation.
    ///
    /// 40000 × 40000 × 4 bytes ≈ 6.4 GB > 512 MiB cap (DEC-038).
    /// The guard fires before the resize backend allocates, so this test
    /// stays cheap — the tiny 2×2 buffer is never enlarged.
    #[test]
    fn resize_apply_exact_rejects_oversized_output() {
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("exact".into())),
            ("width", toml::Value::Integer(40_000)),
            ("height", toml::Value::Integer(40_000)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let img = make_image(2, 2, |_, _| [128, 64, 32, 255]);
        let result = op.apply(img);
        assert!(
            matches!(result, Err(OperationError::Apply { op: "resize", .. })),
            "expected Apply error for oversized exact output, got {result:?}"
        );
    }

    /// A 100×100 input, `percent 2000000` → rejected before allocation.
    ///
    /// Output would be 2 000 000 × 2 000 000 px, far exceeding 512 MiB (DEC-038).
    /// Confirms the byte cap covers input-dependent modes (percent, cover, fill).
    #[test]
    fn resize_apply_percent_rejects_oversized_output() {
        let p = params_from_pairs(&[
            ("mode", toml::Value::String("percent".into())),
            ("percent", toml::Value::Integer(2_000_000)),
        ]);
        let op = Resize::from_params(&p).unwrap();
        let img = make_image(100, 100, |_, _| [10, 20, 30, 255]);
        let result = op.apply(img);
        assert!(
            matches!(result, Err(OperationError::Apply { op: "resize", .. })),
            "expected Apply error for oversized percent output, got {result:?}"
        );
    }

    /// Normal resizes stay below the 512 MiB cap and succeed (regression guard).
    ///
    /// Covers `exact 64x64`, `max 32` (no upscale on small input), and
    /// `percent 50` — all three must return `Ok`.
    #[test]
    fn resize_apply_normal_outputs_succeed() {
        let img = make_image(64, 64, |_, _| [200, 100, 50, 255]);

        // exact 64x64
        let p_exact = params_from_pairs(&[
            ("mode", toml::Value::String("exact".into())),
            ("width", toml::Value::Integer(64)),
            ("height", toml::Value::Integer(64)),
        ]);
        let result_exact = Resize::from_params(&p_exact).unwrap().apply(img.clone());
        assert!(
            result_exact.is_ok(),
            "exact 64x64 must succeed; got {result_exact:?}"
        );

        // max 32 (small input; does not upscale, outputs 32x32)
        let p_max = params_from_pairs(&[
            ("mode", toml::Value::String("max".into())),
            ("width", toml::Value::Integer(32)),
        ]);
        let result_max = Resize::from_params(&p_max).unwrap().apply(img.clone());
        assert!(
            result_max.is_ok(),
            "max 32 must succeed; got {result_max:?}"
        );

        // percent 50 → 32x32
        let p_pct = params_from_pairs(&[
            ("mode", toml::Value::String("percent".into())),
            ("percent", toml::Value::Integer(50)),
        ]);
        let result_pct = Resize::from_params(&p_pct).unwrap().apply(img);
        assert!(
            result_pct.is_ok(),
            "percent 50 must succeed; got {result_pct:?}"
        );
    }
}
