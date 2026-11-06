//! `rotate`: turn the image by a multiple of 90 degrees.
//!
//! `angle` is a number from -360 to 360 and a multiple of 90. A positive
//! angle turns the image clockwise as it is displayed after any earlier step;
//! `-90` turns it anticlockwise. `0`, `360` and `-360` change nothing.
//! Any other angle is refused when the recipe loads.
//!
//! A rotation moves pixels and computes none, so the colour type and bit
//! depth are the input's. It acts on the image as the step receives it, and
//! keeps the image's metadata in memory for a later step, as `crop` does; it
//! writes none.

use super::registry::{self, written};
use super::{Operation, OperationError, OperationParams, ParamKind, ParamSpec, RegistryError};
use crate::image::Image;

/// The step keys `rotate` accepts.
pub const ROTATE_PARAMS: &[ParamSpec] = &[ParamSpec {
    key: "angle",
    kind: ParamKind::Number {
        min: -360.0,
        min_inclusive: true,
        max: Some(360.0),
    },
}];

/// A rotate step: a whole number of clockwise quarter turns.
///
/// Holds the step's keys exactly as written, so [`Operation::params`] gives
/// back `-90` as `-90` and `90.0` as a float.
#[derive(Debug, Clone)]
pub struct Rotate {
    /// Clockwise quarter turns, 0 to 3.
    turns: u8,
    params: OperationParams,
}

/// The clockwise quarter turns an angle makes, 0 to 3, or `None` when the
/// angle is not a multiple of 90.
///
/// Exact in `f64` for every angle the key table lets through: each is a whole
/// multiple of 90 within [-360, 360] or is refused here, and `%` and `/` by 90
/// are exact on such values. `-0.0 % 90.0` is `0.0`, so negative zero is a
/// no-op like zero.
fn quarter_turns(angle: f64) -> Option<u8> {
    if angle % 90.0 != 0.0 {
        return None;
    }
    Some(((angle / 90.0) as i64).rem_euclid(4) as u8)
}

impl Rotate {
    /// Parse and validate a rotate step. Never panics.
    ///
    /// Runs the key table first (unknown key, wrong type, an angle outside
    /// -360 to 360), then requires `angle` and that it be a multiple of 90.
    pub fn from_params(params: &OperationParams) -> Result<Rotate, RegistryError> {
        let invalid = |reason: String| RegistryError::InvalidParams {
            op: "rotate",
            reason,
        };
        registry::check_params("rotate", ROTATE_PARAMS, params).map_err(invalid)?;

        // Read as `f64`: an `f32` read turns `1e-300` into `0.0`, which would
        // load as a silent no-op instead of being refused.
        let angle = params
            .get_f64("angle")
            .ok_or_else(|| invalid("missing required key 'angle'".to_owned()))?;
        let turns = quarter_turns(angle).ok_or_else(|| {
            invalid(format!(
                "'angle' must be a multiple of 90, got {}; rotating by any other angle is not \
                 supported yet",
                written(params, "angle")
            ))
        })?;
        Ok(Rotate {
            turns,
            params: params.clone(),
        })
    }
}

impl Operation for Rotate {
    fn name(&self) -> &'static str {
        "rotate"
    }

    fn params(&self) -> OperationParams {
        self.params.clone()
    }

    /// Turn the image. A quarter turn is defined on every buffer, down to
    /// 1×1, so this never fails and the step has no refusal. A step that makes
    /// no turn hands the same `Image` straight back, copying nothing.
    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        let turned = match self.turns {
            1 => img.pixels().rotate90(),
            2 => img.pixels().rotate180(),
            3 => img.pixels().rotate270(),
            _ => return Ok(img),
        };
        Ok(img.with_pixels(turned))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ::image::{DynamicImage, ImageBuffer, ImageFormat, Rgb, Rgba};

    use super::*;

    fn value_of(v: &str) -> toml::Value {
        // `v` is a TOML value as a recipe author writes it: `90`, `90.0`, `-0.0`.
        let table: toml::Table = format!("v = {v}").parse().expect("a TOML value");
        table["v"].clone()
    }

    fn params(angle: &str) -> OperationParams {
        let mut m = BTreeMap::new();
        m.insert("angle".to_owned(), value_of(angle));
        OperationParams::from_map(m)
    }

    fn rotate(angle: &str) -> Rotate {
        Rotate::from_params(&params(angle)).expect("params are valid")
    }

    struct XorShift(u64);
    impl XorShift {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
    }

    /// A seeded rgb8 buffer, every pixel different, from an xorshift fill.
    fn seeded_rgb8(w: u32, h: u32) -> DynamicImage {
        let mut r = XorShift(0x5EED_0147 ^ (u64::from(w) << 20) ^ u64::from(h));
        DynamicImage::ImageRgb8(ImageBuffer::from_fn(w, h, |_, _| {
            let n = r.next();
            Rgb([n as u8, (n >> 8) as u8, (n >> 16) as u8])
        }))
    }

    fn seeded_rgba16(w: u32, h: u32) -> DynamicImage {
        let mut r = XorShift(0x5EED_0148 ^ (u64::from(w) << 20) ^ u64::from(h));
        DynamicImage::ImageRgba16(ImageBuffer::from_fn(w, h, |_, _| {
            let n = r.next();
            Rgba([
                n as u16,
                (n >> 16) as u16,
                (n >> 32) as u16,
                (n >> 48) as u16,
            ])
        }))
    }

    fn wrap(px: DynamicImage) -> Image {
        Image::from_parts(px, ImageFormat::Png, None)
    }

    fn turned(angle: &str, px: &DynamicImage) -> DynamicImage {
        rotate(angle)
            .apply(wrap(px.clone()))
            .expect("rotate applies")
            .pixels()
            .clone()
    }

    /// The input pixel at output (ox, oy) after `turns` clockwise quarter
    /// turns of a `w`×`h` buffer, written out by hand.
    fn source_of(turns: u8, w: u32, h: u32, ox: u32, oy: u32) -> (u32, u32) {
        match turns {
            0 => (ox, oy),
            1 => (oy, h - 1 - ox),
            2 => (w - 1 - ox, h - 1 - oy),
            _ => (w - 1 - oy, ox),
        }
    }

    fn size_after(turns: u8, w: u32, h: u32) -> (u32, u32) {
        if turns % 2 == 1 {
            (h, w)
        } else {
            (w, h)
        }
    }

    /// `Ok` when `out` is `src` turned `turns` quarter turns, pixel for pixel.
    fn matches_turns(src: &DynamicImage, out: &DynamicImage, turns: u8) -> Result<(), String> {
        let (w, h) = (src.width(), src.height());
        let (ow, oh) = size_after(turns, w, h);
        if (out.width(), out.height()) != (ow, oh) {
            return Err(format!("size {}x{}", out.width(), out.height()));
        }
        let (s, o) = (src.as_rgb8().expect("rgb8"), out.as_rgb8().expect("rgb8"));
        for oy in 0..oh {
            for ox in 0..ow {
                let (ix, iy) = source_of(turns, w, h, ox, oy);
                if o.get_pixel(ox, oy) != s.get_pixel(ix, iy) {
                    return Err(format!("pixel ({ox}, {oy}) is not input ({ix}, {iy})"));
                }
            }
        }
        Ok(())
    }

    #[test]
    fn normalises_every_accepted_angle() {
        // (angle as written, clockwise quarter turns)
        let rows: [(&str, u8); 12] = [
            ("-360", 0),
            ("-270", 1),
            ("-180", 2),
            ("-90", 3),
            ("0", 0),
            ("90", 1),
            ("180", 2),
            ("270", 3),
            ("360", 0),
            ("90.0", 1),
            ("-90.0", 3),
            ("-0.0", 0),
        ];
        let src = seeded_rgb8(7, 5);
        let mut failures = Vec::new();
        for (written, turns) in rows {
            let angle: f64 = match value_of(written) {
                toml::Value::Integer(i) => i as f64,
                toml::Value::Float(f) => f,
                other => panic!("not a number: {other:?}"),
            };
            if quarter_turns(angle) != Some(turns) {
                failures.push(format!(
                    "{written}: quarter_turns gave {:?}",
                    quarter_turns(angle)
                ));
            }
            match Rotate::from_params(&params(written)) {
                Err(e) => failures.push(format!("{written}: refused: {e}")),
                Ok(op) => {
                    if op.turns != turns {
                        failures.push(format!("{written}: stored {} turns", op.turns));
                    }
                    let out = turned(written, &src);
                    if let Err(e) = matches_turns(&src, &out, turns) {
                        failures.push(format!("{written}: {e}"));
                    }
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
        assert_eq!(rows.len(), 12, "all 12 rows ran");
    }

    #[test]
    fn rotate_90_is_clockwise_by_position() {
        let src = seeded_rgb8(7, 5);
        let s = src.as_rgb8().expect("rgb8");

        let o90 = turned("90", &src);
        assert_eq!((o90.width(), o90.height()), (5, 7));
        // Input (0, 0) lands at (4, 0).
        assert_eq!(
            o90.as_rgb8().expect("rgb8").get_pixel(4, 0),
            s.get_pixel(0, 0)
        );
        matches_turns(&src, &o90, 1).expect("the 1-turn map");

        let o270 = turned("270", &src);
        assert_eq!(
            o270.as_rgb8().expect("rgb8").get_pixel(0, 6),
            s.get_pixel(0, 0)
        );
        matches_turns(&src, &o270, 3).expect("the 3-turn map");

        let o180 = turned("180", &src);
        assert_eq!(
            o180.as_rgb8().expect("rgb8").get_pixel(6, 4),
            s.get_pixel(0, 0)
        );
        matches_turns(&src, &o180, 2).expect("the 2-turn map");

        // Negative control: a quarter turn is not the identity here.
        assert_ne!(o90.as_bytes(), src.as_bytes());
    }

    #[test]
    fn rotate_refuses_non_multiples_of_90() {
        for angle in [45.0, 90.5, 1.1, 1e-300] {
            assert_eq!(quarter_turns(angle), None, "{angle}");
        }
    }

    #[test]
    fn no_op_angles_return_the_input() {
        let mut cases = 0;
        for src in [seeded_rgb8(7, 5), seeded_rgba16(7, 5)] {
            for angle in ["0", "360", "-360", "-0.0"] {
                cases += 1;
                let out = turned(angle, &src);
                assert_eq!(out.color(), src.color(), "{angle}: colour type");
                assert_eq!((out.width(), out.height()), (7, 5), "{angle}: size");
                assert_eq!(out.as_bytes(), src.as_bytes(), "{angle}: bytes");
            }
        }
        assert_eq!(cases, 8, "4 angles x 2 colour types");
    }

    #[test]
    fn params_round_trip_as_written() {
        for (written, want) in [
            ("-90", toml::Value::Integer(-90)),
            ("90.0", toml::Value::Float(90.0)),
            ("270", toml::Value::Integer(270)),
        ] {
            let got = rotate(written).params();
            assert_eq!(got.iter().count(), 1, "{written}: one key");
            let (key, value) = got.iter().next().expect("one key");
            assert_eq!(key, "angle");
            assert_eq!(value, &want, "{written}");
        }
    }

    #[test]
    fn refusal_hook_is_not_used_and_apply_never_fails() {
        let mut cases = 0;
        for angle in ["0", "90", "180", "270"] {
            for (w, h) in [(1u32, 1u32), (1, 7), (7, 1)] {
                cases += 1;
                let op = rotate(angle);
                let src = wrap(seeded_rgb8(w, h));
                assert!(op.refusal(&src).is_none(), "{angle} {w}x{h}: refusal");
                let out = op.apply(src).expect("apply never fails");
                let turns = op.turns;
                assert_eq!(
                    (out.pixels().width(), out.pixels().height()),
                    size_after(turns, w, h),
                    "{angle} {w}x{h}: size"
                );
            }
        }
        assert_eq!(cases, 12, "4 turns x 3 shapes");
    }
}
