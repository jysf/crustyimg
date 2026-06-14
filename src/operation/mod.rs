//! The `Operation` trait and its first two concrete implementations (DEC-002).
//!
//! Layering: this module depends only on `crate::image`, `std`, `thiserror`,
//! and `::image` types. It must NOT touch `clap`, `recipe`, `source`, `sink`,
//! `std::fs`, `std::path`, or any terminal types. Ops are pure in-memory
//! transforms (constraint `decode-once-no-per-op-disk`).
//!
//! # Module / crate name collision
//!
//! This crate has a `crate::image` module. Use `::image` to refer to the
//! pixel-library crate, the same convention as `src/image/mod.rs`.

use ::image::DynamicImage;
use thiserror::Error;

use crate::image::Image;

// ─── OperationParams ────────────────────────────────────────────────────────

/// Operation parameters — a dependency-free placeholder for SPEC-003.
///
/// SPEC-006 will widen this to a serde/TOML value when the recipe layer
/// and its `toml`/`serde` dependencies arrive. Parameterless operations
/// (`Identity`, `Invert`) return `OperationParams::None`. Keeping this as
/// a local enum now prevents `operation/` from depending on `toml`/`serde`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationParams {
    /// The operation takes no parameters.
    None,
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

    /// This operation's parameters, forward-compatible with the serde/TOML
    /// value SPEC-006 will introduce. Parameterless ops return
    /// `OperationParams::None`.
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
        OperationParams::None
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
        OperationParams::None
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

    #[test]
    fn identity_name_and_params_are_stable() {
        assert_eq!(Identity.name(), "identity");
        assert_eq!(Identity.params(), OperationParams::None);
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
        // A 2×2 image with one known non-uniform pixel.
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
        // Apply Invert twice: should round-trip to original.
        let once = Invert.apply(img).unwrap();
        let twice = Invert.apply(once).unwrap();
        assert_eq!(twice.pixels().to_rgba8().into_raw(), original_raw);
    }
}
