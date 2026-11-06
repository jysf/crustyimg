//! `flip`: mirror the image left to right or top to bottom.
//!
//! `axis = "horizontal"` mirrors left and right; `axis = "vertical"` mirrors
//! top and bottom. A flip moves pixels and computes none, so the colour type
//! and bit depth are the input's. It acts on the image as the step receives
//! it, and keeps the image's metadata in memory for a later step, as `crop`
//! does; it writes none.

use super::registry;
use super::{Operation, OperationError, OperationParams, ParamKind, ParamSpec, RegistryError};
use crate::image::Image;

/// The two values `axis` takes.
pub(crate) const AXIS_NAMES: &[&str] = &["horizontal", "vertical"];

/// The step keys `flip` accepts.
pub const FLIP_PARAMS: &[ParamSpec] = &[ParamSpec {
    key: "axis",
    kind: ParamKind::OneOf(AXIS_NAMES),
}];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    /// Mirror left and right.
    Horizontal,
    /// Mirror top and bottom.
    Vertical,
}

/// A flip step: one mirror, on one axis.
///
/// Holds the step's keys exactly as written, so [`Operation::params`] gives
/// back what the recipe said.
#[derive(Debug, Clone)]
pub struct Flip {
    axis: Axis,
    params: OperationParams,
}

impl Flip {
    /// Parse and validate a flip step. Never panics.
    ///
    /// Runs the key table first (unknown key, wrong type, a value that is not
    /// one of the two axes), then requires `axis`.
    pub fn from_params(params: &OperationParams) -> Result<Flip, RegistryError> {
        let invalid = |reason: String| RegistryError::InvalidParams { op: "flip", reason };
        registry::check_params("flip", FLIP_PARAMS, params).map_err(invalid)?;

        let axis = match params.get_str("axis") {
            Some("horizontal") => Axis::Horizontal,
            Some("vertical") => Axis::Vertical,
            // The table has already limited a present value to the two names.
            Some(_) | None => return Err(invalid("missing required key 'axis'".to_owned())),
        };
        Ok(Flip {
            axis,
            params: params.clone(),
        })
    }
}

impl Operation for Flip {
    fn name(&self) -> &'static str {
        "flip"
    }

    fn params(&self) -> OperationParams {
        self.params.clone()
    }

    /// Mirror the image. A mirror is defined on every buffer, down to 1×1, so
    /// this never fails and the step has no refusal.
    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        let flipped = match self.axis {
            Axis::Horizontal => img.pixels().fliph(),
            Axis::Vertical => img.pixels().flipv(),
        };
        Ok(img.with_pixels(flipped))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ::image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};

    use super::*;

    fn flip(axis: &str) -> Flip {
        let mut m = BTreeMap::new();
        m.insert("axis".to_owned(), toml::Value::String(axis.to_owned()));
        Flip::from_params(&OperationParams::from_map(m)).expect("params are valid")
    }

    /// A seeded rgb8 buffer, every pixel different, from an xorshift fill.
    fn seeded(w: u32, h: u32) -> DynamicImage {
        let mut s: u64 = 0x5EED_0147 ^ (u64::from(w) << 20) ^ u64::from(h);
        DynamicImage::ImageRgb8(ImageBuffer::from_fn(w, h, |_, _| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            Rgb([s as u8, (s >> 8) as u8, (s >> 16) as u8])
        }))
    }

    fn wrap(px: DynamicImage) -> Image {
        Image::from_parts(px, ImageFormat::Png, None)
    }

    fn rgb(img: &DynamicImage) -> &ImageBuffer<Rgb<u8>, Vec<u8>> {
        img.as_rgb8().expect("rgb8")
    }

    #[test]
    fn horizontal_mirrors_left_and_right_by_position() {
        let src = seeded(7, 5);
        let (sw, sh) = (7u32, 5u32);

        let h = flip("horizontal")
            .apply(wrap(src.clone()))
            .expect("flip applies");
        let (h, s) = (rgb(h.pixels()), rgb(&src));
        assert_eq!((h.width(), h.height()), (7, 5));
        // Input (0, 0) lands at (6, 0).
        assert_eq!(h.get_pixel(6, 0), s.get_pixel(0, 0));
        for oy in 0..sh {
            for ox in 0..sw {
                assert_eq!(
                    h.get_pixel(ox, oy),
                    s.get_pixel(sw - 1 - ox, oy),
                    "horizontal ({ox}, {oy})"
                );
            }
        }

        let v = flip("vertical")
            .apply(wrap(src.clone()))
            .expect("flip applies");
        let v = rgb(v.pixels());
        assert_eq!((v.width(), v.height()), (7, 5));
        // Input (0, 0) lands at (0, 4).
        assert_eq!(v.get_pixel(0, 4), s.get_pixel(0, 0));
        for oy in 0..sh {
            for ox in 0..sw {
                assert_eq!(
                    v.get_pixel(ox, oy),
                    s.get_pixel(ox, sh - 1 - oy),
                    "vertical ({ox}, {oy})"
                );
            }
        }
        // Negative control: the two axes differ on this buffer.
        assert_ne!(h.get_pixel(0, 0), v.get_pixel(0, 0));
    }

    #[test]
    fn params_round_trip_as_written() {
        for axis in ["horizontal", "vertical"] {
            let params = flip(axis).params();
            assert_eq!(params.get_str("axis"), Some(axis));
            assert_eq!(params.iter().count(), 1, "no other key appears");
        }
    }

    #[test]
    fn refusal_hook_is_not_used_and_apply_never_fails() {
        let mut cases = 0;
        for axis in ["horizontal", "vertical"] {
            for (w, h) in [(1u32, 1u32), (1, 7), (7, 1)] {
                cases += 1;
                let op = flip(axis);
                let src = wrap(seeded(w, h));
                assert!(op.refusal(&src).is_none(), "{axis} {w}x{h}: refusal");
                let out = op.apply(src).expect("apply never fails");
                assert_eq!(
                    (out.pixels().width(), out.pixels().height()),
                    (w, h),
                    "{axis} {w}x{h}: size"
                );
            }
        }
        assert_eq!(cases, 6, "2 axes x 3 shapes");
    }
}
