//! `trim`: remove the outer rows and columns that match one colour.
//!
//! The colour is the step's `color` (hex, or the keyword `transparent` for
//! `#00000000`), or with none the image's top-left pixel
//! (read in the image's own samples, so a 16-bit or float corner is exact).
//! Each of the four sides is searched on its own, so the result is the
//! bounding box of the pixels that do not match, and a border can be wider on
//! one side than another. The step never looks at what the content is.
//!
//! **Matching.** Every sample becomes an integer *level*: an 8-bit sample is
//! its value, a 16-bit sample is its value, and a float sample is
//! `round(s × 65535)` clamped to `0..=65535` (a NaN has no level and never
//! matches). One 8-bit step is 1 level at 8 bits and 257 levels at 16 bits and
//! for float, so a stated `#ff` is 65535 at 16 bits and `tolerance = 8` is 2056
//! levels there. A pixel is four levels, red, green, blue and alpha: grey `L`
//! is `(L, L, L)`, and a type with no alpha has alpha at its maximum. A pixel
//! matches when every channel is within the tolerance of the colour's, except
//! that **a pixel with alpha 0 matches a colour with alpha 0 whatever colour is
//! hidden under it**.
//!
//! The output is a `crop_imm` of the input, so the colour type and bit depth
//! never change and no pixel is computed. An image with nothing to trim is
//! returned unchanged. The step acts on the image as it receives it and keeps
//! the image's metadata in memory for a later step; it writes none.
//!
//! When every pixel matches, `apply` fails rather than refusing, because
//! skipping the step would write an untrimmed image.

use ::image::DynamicImage;

use super::registry::{self};
use super::{Operation, OperationError, OperationParams, ParamKind, ParamSpec, RegistryError};
use crate::image::Image;

/// The step keys `trim` accepts, in the order an unknown-key error lists them.
pub const TRIM_PARAMS: &[ParamSpec] = &[
    ParamSpec {
        key: "color",
        kind: ParamKind::Text,
    },
    ParamSpec {
        key: "tolerance",
        kind: ParamKind::UInt {
            min: 0,
            max: Some(255),
        },
    },
];

/// A trim step: an optional colour and a tolerance in 8-bit steps.
///
/// Holds the step's keys exactly as written, so [`Operation::params`] gives
/// back an absent `tolerance` as absent and a hex colour in the case it was
/// written in.
#[derive(Debug, Clone)]
pub struct Trim {
    /// The colour to trim; `None` means the top-left pixel.
    color: Option<[u8; 4]>,
    tolerance: u8,
    params: OperationParams,
}

impl Trim {
    /// Parse and validate a trim step. Never panics.
    ///
    /// Runs the key table first (unknown key, wrong type, a tolerance outside
    /// `0..=255`), then the colour's syntax.
    pub fn from_params(params: &OperationParams) -> Result<Trim, RegistryError> {
        let invalid = |reason: String| RegistryError::InvalidParams { op: "trim", reason };
        registry::check_params("trim", TRIM_PARAMS, params).map_err(invalid)?;

        let color = match params.get_str("color") {
            Some(text) => Some(super::parse_fill_color(text).map_err(|_| {
                invalid(format!(
                    "'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got {text:?}"
                ))
            })?),
            None => None,
        };
        let tolerance = match params.get_u32("tolerance") {
            // The table already limited it to 0..=255.
            Some(t) => u8::try_from(t).map_err(|_| {
                invalid(format!("internal: 'tolerance' = {t} passed the key table"))
            })?,
            None => 0,
        };
        Ok(Trim {
            color,
            tolerance,
            params: params.clone(),
        })
    }

    /// Why a whole match fails: it names where the colour came from and never
    /// prints a colour the image does not hold.
    fn whole_match_reason(&self, w: u32, h: u32) -> String {
        let source = match self.params.get_str("color") {
            Some(text) => format!("'color' = {text:?}"),
            None => "its top-left pixel".to_owned(),
        };
        format!(
            "every pixel of this {w}×{h} px image matches {source} within 'tolerance' = {}, so \
             a trim would leave nothing",
            self.tolerance
        )
    }
}

/// A pixel's four levels (red, green, blue, alpha); `None` is a NaN sample.
type Levels = [Option<i64>; 4];

/// A float sample as the nearest 16-bit level, or `None` for NaN.
fn float_level(s: f32) -> Option<i64> {
    if s.is_nan() {
        return None;
    }
    // The product of an `f32` and 65535 is exact in `f64`, so the only rounding
    // is the final `round`.
    Some((f64::from(s) * 65535.0).round().clamp(0.0, 65535.0) as i64)
}

/// Whether one sample's level is within `tolerance` of the colour's.
fn channel_matches(sample: Option<i64>, reference: Option<i64>, tolerance: i64) -> bool {
    match (sample, reference) {
        (Some(a), Some(b)) => (a - b).abs() <= tolerance,
        _ => false,
    }
}

/// A reader over one image: its pixels as levels, the colour to match, and the
/// tolerance in levels.
struct Scan<'a> {
    width: u32,
    height: u32,
    read: Box<dyn Fn(u32, u32) -> Levels + 'a>,
    reference: Levels,
    tolerance: i64,
}

impl<'a> Scan<'a> {
    /// A scan of `img` against `color` (or its top-left pixel) within
    /// `tolerance` 8-bit steps, or `None` for a colour type this build cannot
    /// read.
    fn new(img: &'a DynamicImage, color: Option<[u8; 4]>, tolerance: u8) -> Option<Scan<'a>> {
        let some = |s: u8| Some(i64::from(s));
        let some16 = |s: u16| Some(i64::from(s));
        let (read, step): (Box<dyn Fn(u32, u32) -> Levels + 'a>, i64) = match img {
            DynamicImage::ImageLuma8(b) => (
                Box::new(move |x, y| {
                    let l = some(b.get_pixel(x, y).0[0]);
                    [l, l, l, Some(255)]
                }),
                1,
            ),
            DynamicImage::ImageLumaA8(b) => (
                Box::new(move |x, y| {
                    let p = b.get_pixel(x, y).0;
                    [some(p[0]), some(p[0]), some(p[0]), some(p[1])]
                }),
                1,
            ),
            DynamicImage::ImageRgb8(b) => (
                Box::new(move |x, y| {
                    let p = b.get_pixel(x, y).0;
                    [some(p[0]), some(p[1]), some(p[2]), Some(255)]
                }),
                1,
            ),
            DynamicImage::ImageRgba8(b) => (
                Box::new(move |x, y| {
                    let p = b.get_pixel(x, y).0;
                    [some(p[0]), some(p[1]), some(p[2]), some(p[3])]
                }),
                1,
            ),
            DynamicImage::ImageLuma16(b) => (
                Box::new(move |x, y| {
                    let l = some16(b.get_pixel(x, y).0[0]);
                    [l, l, l, Some(65535)]
                }),
                257,
            ),
            DynamicImage::ImageLumaA16(b) => (
                Box::new(move |x, y| {
                    let p = b.get_pixel(x, y).0;
                    [some16(p[0]), some16(p[0]), some16(p[0]), some16(p[1])]
                }),
                257,
            ),
            DynamicImage::ImageRgb16(b) => (
                Box::new(move |x, y| {
                    let p = b.get_pixel(x, y).0;
                    [some16(p[0]), some16(p[1]), some16(p[2]), Some(65535)]
                }),
                257,
            ),
            DynamicImage::ImageRgba16(b) => (
                Box::new(move |x, y| {
                    let p = b.get_pixel(x, y).0;
                    [some16(p[0]), some16(p[1]), some16(p[2]), some16(p[3])]
                }),
                257,
            ),
            DynamicImage::ImageRgb32F(b) => (
                Box::new(move |x, y| {
                    let p = b.get_pixel(x, y).0;
                    [
                        float_level(p[0]),
                        float_level(p[1]),
                        float_level(p[2]),
                        Some(65535),
                    ]
                }),
                257,
            ),
            DynamicImage::ImageRgba32F(b) => (
                Box::new(move |x, y| {
                    let p = b.get_pixel(x, y).0;
                    [
                        float_level(p[0]),
                        float_level(p[1]),
                        float_level(p[2]),
                        float_level(p[3]),
                    ]
                }),
                257,
            ),
            _ => return None,
        };
        let (width, height) = (img.width(), img.height());
        if width == 0 || height == 0 {
            return None;
        }
        let reference = match color {
            Some(c) => c.map(|v| Some(i64::from(v) * step)),
            None => read(0, 0),
        };
        Some(Scan {
            width,
            height,
            read,
            reference,
            tolerance: i64::from(tolerance) * step,
        })
    }

    /// Whether the pixel at `(x, y)` matches the colour.
    fn matches(&self, x: u32, y: u32) -> bool {
        let px = (self.read)(x, y);
        // A fully transparent pixel matches a fully transparent colour
        // whatever colour is hidden under it.
        if px[3] == Some(0) && self.reference[3] == Some(0) {
            return true;
        }
        (0..4).all(|i| channel_matches(px[i], self.reference[i], self.tolerance))
    }

    /// The smallest box `(left, top, right, bottom)`, all inclusive, outside
    /// which every pixel matches; `None` when every pixel matches. Each side is
    /// its own search.
    fn bounds(&self) -> Option<(u32, u32, u32, u32)> {
        let row_differs = |y: u32| (0..self.width).any(|x| !self.matches(x, y));
        let top = (0..self.height).find(|&y| row_differs(y))?;
        let bottom = (top..self.height)
            .rev()
            .find(|&y| row_differs(y))
            .unwrap_or(top);
        let column_differs = |x: u32| (top..=bottom).any(|y| !self.matches(x, y));
        let left = (0..self.width).find(|&x| column_differs(x)).unwrap_or(0);
        let right = (left..self.width)
            .rev()
            .find(|&x| column_differs(x))
            .unwrap_or(left);
        Some((left, top, right, bottom))
    }
}

impl Operation for Trim {
    fn name(&self) -> &'static str {
        "trim"
    }

    fn params(&self) -> OperationParams {
        self.params.clone()
    }

    /// Remove the matching border. Fails when every pixel matches, because a
    /// trim would leave nothing and skipping the step would write an untrimmed
    /// image; returns the input unchanged when there is nothing to trim.
    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        let fail = |reason: String| OperationError::Apply { op: "trim", reason };
        let (w, h) = (img.pixels().width(), img.pixels().height());
        if w == 0 || h == 0 {
            return Ok(img);
        }
        // The scan borrows the pixels; end the borrow before `img` moves.
        let bounds = {
            let scan = Scan::new(img.pixels(), self.color, self.tolerance)
                .ok_or_else(|| fail("this image's colour type cannot be trimmed".to_owned()))?;
            scan.bounds()
        };
        let Some((left, top, right, bottom)) = bounds else {
            return Err(fail(self.whole_match_reason(w, h)));
        };
        if (left, top, right, bottom) == (0, 0, w - 1, h - 1) {
            return Ok(img);
        }
        let out = img
            .pixels()
            .crop_imm(left, top, right - left + 1, bottom - top + 1);
        Ok(img.with_pixels(out))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ::image::{ImageBuffer, ImageFormat};

    use super::*;

    fn params_of(pairs: &[(&str, toml::Value)]) -> OperationParams {
        let map: BTreeMap<String, toml::Value> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect();
        OperationParams::from_map(map)
    }

    fn string(s: &str) -> toml::Value {
        toml::Value::String(s.to_owned())
    }

    /// One pixel of any colour type, for a 2×1 buffer.
    #[derive(Clone, Copy)]
    enum Pix {
        L8(u8),
        La8([u8; 2]),
        Rgb8([u8; 3]),
        Rgba8([u8; 4]),
        L16(u16),
        La16([u16; 2]),
        Rgb16([u16; 3]),
        Rgba16([u16; 4]),
        Rgb32F([f32; 3]),
        Rgba32F([f32; 4]),
    }

    /// A 2×1 buffer: `a` at (0, 0), `b` at (1, 0). Both must be one type.
    fn two(a: Pix, b: Pix) -> DynamicImage {
        use Pix::*;
        match (a, b) {
            (L8(a), L8(b)) => {
                DynamicImage::ImageLuma8(ImageBuffer::from_raw(2, 1, vec![a, b]).unwrap())
            }
            (La8(a), La8(b)) => {
                DynamicImage::ImageLumaA8(ImageBuffer::from_raw(2, 1, [a, b].concat()).unwrap())
            }
            (Rgb8(a), Rgb8(b)) => {
                DynamicImage::ImageRgb8(ImageBuffer::from_raw(2, 1, [a, b].concat()).unwrap())
            }
            (Rgba8(a), Rgba8(b)) => {
                DynamicImage::ImageRgba8(ImageBuffer::from_raw(2, 1, [a, b].concat()).unwrap())
            }
            (L16(a), L16(b)) => {
                DynamicImage::ImageLuma16(ImageBuffer::from_raw(2, 1, vec![a, b]).unwrap())
            }
            (La16(a), La16(b)) => {
                DynamicImage::ImageLumaA16(ImageBuffer::from_raw(2, 1, [a, b].concat()).unwrap())
            }
            (Rgb16(a), Rgb16(b)) => {
                DynamicImage::ImageRgb16(ImageBuffer::from_raw(2, 1, [a, b].concat()).unwrap())
            }
            (Rgba16(a), Rgba16(b)) => {
                DynamicImage::ImageRgba16(ImageBuffer::from_raw(2, 1, [a, b].concat()).unwrap())
            }
            (Rgb32F(a), Rgb32F(b)) => {
                DynamicImage::ImageRgb32F(ImageBuffer::from_raw(2, 1, [a, b].concat()).unwrap())
            }
            (Rgba32F(a), Rgba32F(b)) => {
                DynamicImage::ImageRgba32F(ImageBuffer::from_raw(2, 1, [a, b].concat()).unwrap())
            }
            _ => panic!("two pixels of different types"),
        }
    }

    /// Whether pixel (1, 0) of the 2×1 buffer matches, for a colour source and
    /// a tolerance.
    fn sample_matches(img: &DynamicImage, color: Option<[u8; 4]>, t: u8) -> bool {
        Scan::new(img, color, t)
            .expect("readable colour type")
            .matches(1, 0)
    }

    const fn hex(r: u8, g: u8, b: u8, a: u8) -> Option<[u8; 4]> {
        Some([r, g, b, a])
    }

    /// A float sample `levels` 16-bit levels from zero.
    fn at_level(k: u32) -> f32 {
        (f64::from(k) / 65535.0) as f32
    }

    /// One row of the match-rule table.
    type Row = (&'static str, DynamicImage, Option<[u8; 4]>, u8, bool);

    /// The match rule, row by row: 36 rows, each a 2×1 buffer with the top-left
    /// at (0, 0) and the sample at (1, 0).
    #[test]
    fn match_rule_table() {
        use Pix::*;
        let third = 1.0f32 / 255.0;
        let tl = Rgba32F([0.25, 0.5, 0.75, 1.0]);
        // (name, buffer, colour, tolerance, expected)
        let rows: Vec<Row> = vec![
            (
                "rgb8 8",
                two(Rgb8([0; 3]), Rgb8([8, 0, 0])),
                hex(0, 0, 0, 255),
                8,
                true,
            ),
            (
                "rgb8 9",
                two(Rgb8([0; 3]), Rgb8([9, 0, 0])),
                hex(0, 0, 0, 255),
                8,
                false,
            ),
            (
                "rgb8 exact",
                two(Rgb8([0; 3]), Rgb8([16, 32, 48])),
                hex(16, 32, 48, 255),
                0,
                true,
            ),
            (
                "rgb8 exact+1",
                two(Rgb8([0; 3]), Rgb8([16, 32, 49])),
                hex(16, 32, 48, 255),
                0,
                false,
            ),
            (
                "rgb16 2056",
                two(Rgb16([0; 3]), Rgb16([2056, 0, 0])),
                hex(0, 0, 0, 255),
                8,
                true,
            ),
            (
                "rgb16 2057",
                two(Rgb16([0; 3]), Rgb16([2057, 0, 0])),
                hex(0, 0, 0, 255),
                8,
                false,
            ),
            (
                "rgb16 ff",
                two(Rgb16([0; 3]), Rgb16([65535; 3])),
                hex(255, 255, 255, 255),
                0,
                true,
            ),
            (
                "rgb16 ff-1",
                two(Rgb16([0; 3]), Rgb16([65534, 65535, 65535])),
                hex(255, 255, 255, 255),
                0,
                false,
            ),
            (
                "rgb16 tl",
                two(Rgb16([4660, 22136, 39612]), Rgb16([4660, 22136, 39612])),
                None,
                0,
                true,
            ),
            (
                "rgb16 tl+1",
                two(Rgb16([4660, 22136, 39612]), Rgb16([4661, 22136, 39612])),
                None,
                0,
                false,
            ),
            (
                "f32 1/255",
                two(Rgb32F([0.0; 3]), Rgb32F([third, 0.0, 0.0])),
                hex(0, 0, 0, 255),
                1,
                true,
            ),
            (
                "f32 2/255",
                two(Rgb32F([0.0; 3]), Rgb32F([2.0 / 255.0, 0.0, 0.0])),
                hex(0, 0, 0, 255),
                1,
                false,
            ),
            (
                "f32 under",
                two(Rgb32F([0.0; 3]), Rgb32F([at_level(63479); 3])),
                hex(255, 255, 255, 255),
                8,
                true,
            ),
            (
                "f32 over",
                two(Rgb32F([0.0; 3]), Rgb32F([at_level(63478), 1.0, 1.0])),
                hex(255, 255, 255, 255),
                8,
                false,
            ),
            ("rgba32f tl level", two(tl, tl), None, 0, true),
            (
                "rgba32f tl next_up",
                two(tl, Rgba32F([0.25f32.next_up(), 0.5, 0.75, 1.0])),
                None,
                0,
                true,
            ),
            (
                "rgba32f tl +1 level",
                two(
                    tl,
                    Rgba32F([(0.25f64 + 1.0 / 65535.0) as f32, 0.5, 0.75, 1.0]),
                ),
                None,
                0,
                false,
            ),
            (
                "l8 grey",
                two(L8(0), L8(128)),
                hex(128, 128, 128, 255),
                0,
                true,
            ),
            (
                "l8 chroma",
                two(L8(0), L8(128)),
                hex(128, 0, 0, 255),
                0,
                false,
            ),
            (
                "l16 grey",
                two(L16(0), L16(32896)),
                hex(128, 128, 128, 255),
                0,
                true,
            ),
            (
                "l16 chroma",
                two(L16(0), L16(32896)),
                hex(128, 0, 0, 255),
                0,
                false,
            ),
            (
                "la8 match",
                two(La8([0, 0]), La8([128, 128])),
                hex(128, 128, 128, 128),
                0,
                true,
            ),
            (
                "la8 alpha",
                two(La8([0, 0]), La8([128, 255])),
                hex(128, 128, 128, 128),
                0,
                false,
            ),
            (
                "la16 a0 tl",
                two(La16([1000, 0]), La16([54321, 0])),
                None,
                0,
                true,
            ),
            (
                "rgba8 a0",
                two(Rgba8([0; 4]), Rgba8([200, 10, 90, 0])),
                hex(0, 0, 0, 0),
                0,
                true,
            ),
            (
                "rgba8 a1",
                two(Rgba8([0; 4]), Rgba8([0, 0, 0, 1])),
                hex(0, 0, 0, 0),
                0,
                false,
            ),
            (
                "rgba8 a3 far",
                two(Rgba8([0; 4]), Rgba8([200, 10, 90, 3])),
                hex(0, 0, 0, 0),
                8,
                false,
            ),
            (
                "rgba8 a3 near",
                two(Rgba8([0; 4]), Rgba8([5, 5, 5, 3])),
                hex(0, 0, 0, 0),
                8,
                true,
            ),
            (
                "rgba8 opaque colour",
                two(Rgba8([0; 4]), Rgba8([0, 0, 0, 0])),
                hex(0, 0, 0, 255),
                0,
                false,
            ),
            (
                "rgba16 a0 tl",
                two(Rgba16([9, 9, 9, 0]), Rgba16([60000, 1, 2, 0])),
                None,
                0,
                true,
            ),
            (
                "f32 nan",
                two(Rgb32F([0.0; 3]), Rgb32F([f32::NAN, 1.0, 1.0])),
                hex(255, 255, 255, 255),
                255,
                false,
            ),
            (
                "f32 over range",
                two(Rgb32F([0.0; 3]), Rgb32F([1.5, 1.0, 7.0])),
                hex(255, 255, 255, 255),
                0,
                true,
            ),
            (
                "f32 under range",
                two(Rgb32F([0.0; 3]), Rgb32F([-0.25, 0.0, -9.0])),
                hex(0, 0, 0, 255),
                0,
                true,
            ),
            (
                "rgba32f a -0",
                two(Rgba32F([0.0; 4]), Rgba32F([0.3, 0.3, 0.3, -0.0])),
                hex(0, 0, 0, 0),
                0,
                true,
            ),
            (
                "rgba32f a 1e-6",
                two(Rgba32F([0.0; 4]), Rgba32F([0.3, 0.3, 0.3, 1e-6])),
                hex(0, 0, 0, 0),
                0,
                true,
            ),
            (
                "rgba32f a 8e-6",
                two(Rgba32F([0.0; 4]), Rgba32F([0.3, 0.3, 0.3, 8e-6])),
                hex(0, 0, 0, 0),
                0,
                false,
            ),
        ];
        let mut wrong = Vec::new();
        for (name, img, color, t, want) in &rows {
            let got = sample_matches(img, *color, *t);
            if got != *want {
                wrong.push(format!("{name}: matched {got}, expected {want}"));
            }
        }
        assert!(wrong.is_empty(), "rows wrong:\n{}", wrong.join("\n"));
        assert_eq!(rows.len(), 36, "every row ran");
    }

    /// Every `c`, `t` and sign: the float rule puts the edge exactly at `t`
    /// steps, whichever way the `f32` was built and within 3 ulp of it.
    #[test]
    fn float_rule_is_exact_at_every_edge() {
        // The `f32` values within 3 ulp of `v`, `v` included.
        fn walk(v: f32) -> [f32; 7] {
            let (mut down, mut up) = (v, v);
            let mut out = [v; 7];
            for i in 0..3 {
                down = down.next_down();
                up = up.next_up();
                out[1 + i] = down;
                out[4 + i] = up;
            }
            out
        }
        let mut cases = 0u64;
        let mut wrong = Vec::new();
        let mut check = |label: &str, v: f32, c: i64, t: i64, want: bool| {
            cases += 1;
            let got = channel_matches(float_level(v), Some(c * 257), t * 257);
            if got != want && wrong.len() < 20 {
                wrong.push(format!("{label}: c={c} t={t} v={v:?} matched {got}"));
            }
        };
        for c in 0i64..=255 {
            for t in 0i64..=255 {
                for sign in [1i64, -1] {
                    let edge = c + sign * t;
                    if (0..=255).contains(&edge) {
                        for v in walk(edge as f32 / 255.0) {
                            check("edge (f32 maths)", v, c, t, true);
                        }
                        for v in walk((edge as f64 / 255.0) as f32) {
                            check("edge (f64 maths)", v, c, t, true);
                        }
                    }
                    let past = c + sign * (t + 1);
                    if (0..=255).contains(&past) {
                        for v in walk(past as f32 / 255.0) {
                            check("one step past (f32)", v, c, t, false);
                        }
                        for v in walk((past as f64 / 255.0) as f32) {
                            check("one step past (f64)", v, c, t, false);
                        }
                    }
                    let level = c * 257 + sign * (t * 257 + 1);
                    if (0..=65535).contains(&level) {
                        for v in walk((level as f64 / 65535.0) as f32) {
                            check("one level past", v, c, t, false);
                        }
                    }
                }
            }
        }
        // Every level reads back as itself, from both ways of building it.
        for k in 0i64..=65535 {
            cases += 1;
            let a = float_level((k as f64 / 65535.0) as f32);
            let b = float_level(k as f32 / 65535.0);
            if (a, b) != (Some(k), Some(k)) && wrong.len() < 20 {
                wrong.push(format!("level {k} read back as {a:?} / {b:?}"));
            }
        }
        println!("{cases} cases");
        assert!(wrong.is_empty(), "wrong:\n{}", wrong.join("\n"));
        assert_eq!(cases, 2_357_504, "every case ran");
        assert_eq!(float_level(1.0f32 / 255.0), Some(257));
    }

    #[test]
    fn params_round_trip_as_written() {
        let none = Trim::from_params(&params_of(&[])).expect("no keys");
        assert!(!none.params().contains("tolerance"), "absent stays absent");
        assert!(!none.params().contains("color"));

        let some = Trim::from_params(&params_of(&[
            ("tolerance", toml::Value::Integer(8)),
            ("color", string("#FFFFFF")),
        ]))
        .expect("both keys");
        let back = some.params();
        assert_eq!(back.get_u32("tolerance"), Some(8));
        assert_eq!(back.get_str("color"), Some("#FFFFFF"), "case is kept");
        assert_eq!(some.name(), "trim");
    }

    #[test]
    fn refusal_hook_is_not_used() {
        let seeded = DynamicImage::ImageRgb8(ImageBuffer::from_fn(4, 3, |x, y| {
            ::image::Rgb([(x * 40 + y) as u8, (y * 70 + x) as u8, (x * y) as u8])
        }));
        let uniform =
            DynamicImage::ImageRgb8(ImageBuffer::from_pixel(3, 2, ::image::Rgb([9, 9, 9])));
        let one = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(1, 1, ::image::Rgb([9, 9, 9])));
        for body in [params_of(&[]), params_of(&[("color", string("#ffffff"))])] {
            let op = Trim::from_params(&body).expect("builds");
            for img in [&one, &uniform, &seeded] {
                let image = Image::from_parts(img.clone(), ImageFormat::Png, None);
                assert!(op.refusal(&image).is_none(), "trim never declines");
            }
        }
    }

    #[test]
    fn whole_match_names_its_source() {
        let reason = |params: OperationParams, img: DynamicImage| -> String {
            let op = Trim::from_params(&params).expect("builds");
            op.apply(Image::from_parts(img, ImageFormat::Png, None))
                .expect_err("a whole match fails")
                .to_string()
        };
        let one = DynamicImage::ImageRgb16(ImageBuffer::from_pixel(1, 1, ::image::Rgb([7, 8, 9])));
        assert_eq!(
            reason(params_of(&[]), one),
            "operation 'trim' failed: every pixel of this 1×1 px image matches its top-left \
             pixel within 'tolerance' = 0, so a trim would leave nothing"
        );
        let near =
            DynamicImage::ImageRgb8(ImageBuffer::from_pixel(3, 2, ::image::Rgb([250, 255, 255])));
        assert_eq!(
            reason(
                params_of(&[
                    ("color", string("#FFFFFF")),
                    ("tolerance", toml::Value::Integer(5)),
                ]),
                near,
            ),
            "operation 'trim' failed: every pixel of this 3×2 px image matches 'color' = \
             \"#FFFFFF\" within 'tolerance' = 5, so a trim would leave nothing"
        );
        let four = DynamicImage::ImageRgb8(ImageBuffer::from_fn(4, 3, |x, y| {
            ::image::Rgb([(x * 40 + y) as u8, (y * 70 + x) as u8, (x * y) as u8])
        }));
        assert_eq!(
            reason(params_of(&[("tolerance", toml::Value::Integer(255))]), four),
            "operation 'trim' failed: every pixel of this 4×3 px image matches its top-left \
             pixel within 'tolerance' = 255, so a trim would leave nothing"
        );
    }
}
