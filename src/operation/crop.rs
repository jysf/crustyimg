//! `crop`: cut a rectangle out of the image, given in fractions of the image.
//!
//! A crop is one of two forms, told apart by which keys the step carries:
//!
//! - a **rectangle**: `x`, `y`, `width`, `height`, all four required;
//! - an **aspect**: `aspect = "W:H"` and an optional `gravity`.
//!
//! Every length is a fraction from 0 to 1 of the image the step receives, so a
//! recipe written against a downscaled preview crops the same region of the
//! full-size original. Each edge is rounded to the nearest pixel on its own
//! (halves up), so every edge lands within half a pixel of its exact position
//! at any image size. A crop never resamples: it moves pixels and computes
//! none, so the colour type and bit depth are the input's.

use std::str::FromStr;

use super::registry::{self, written};
use super::{
    Gravity, Operation, OperationError, OperationParams, ParamKind, ParamSpec, RegistryError,
    GRAVITY_NAMES,
};
use crate::image::Image;

/// The step keys `crop` accepts, in the order an unknown-key error lists them.
pub const CROP_PARAMS: &[ParamSpec] = &[
    ParamSpec {
        key: "x",
        kind: ParamKind::Number {
            min: 0.0,
            min_inclusive: true,
            max: Some(1.0),
        },
    },
    ParamSpec {
        key: "y",
        kind: ParamKind::Number {
            min: 0.0,
            min_inclusive: true,
            max: Some(1.0),
        },
    },
    ParamSpec {
        key: "width",
        kind: ParamKind::Number {
            min: 0.0,
            min_inclusive: false,
            max: Some(1.0),
        },
    },
    ParamSpec {
        key: "height",
        kind: ParamKind::Number {
            min: 0.0,
            min_inclusive: false,
            max: Some(1.0),
        },
    },
    ParamSpec {
        key: "aspect",
        kind: ParamKind::Text,
    },
    ParamSpec {
        key: "gravity",
        kind: ParamKind::OneOf(GRAVITY_NAMES),
    },
];

/// The keys that make up a rectangle crop, in table order.
const RECT_KEYS: [&str; 4] = ["x", "y", "width", "height"];

/// Each side of an aspect is a whole number up to this. It keeps the integer
/// maths in `aspect_size` far from overflowing a `u64`.
const MAX_ASPECT_SIDE: u64 = 65_535;

#[derive(Debug, Clone, Copy)]
enum Form {
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    Aspect {
        a: u32,
        b: u32,
        gravity: Gravity,
    },
}

/// A crop step: a rectangle or an aspect, in fractions of the image.
///
/// Holds the step's keys exactly as written, so [`Operation::params`] gives
/// back an integer `x = 0` as an integer and an absent `gravity` as absent.
#[derive(Debug, Clone)]
pub struct Crop {
    form: Form,
    params: OperationParams,
}

impl Crop {
    /// Parse and validate a crop step. Never panics.
    ///
    /// Runs the key table first (unknown key, wrong type, out-of-range
    /// fraction), then the rules that need more than one key: which form the
    /// keys make, a rectangle with a key missing, a rectangle that runs past
    /// the image's edge, and the aspect's syntax.
    pub fn from_params(params: &OperationParams) -> Result<Crop, RegistryError> {
        let invalid = |reason: String| RegistryError::InvalidParams { op: "crop", reason };
        registry::check_params("crop", CROP_PARAMS, params).map_err(invalid)?;

        let first_rect_key = RECT_KEYS.iter().find(|k| params.contains(k));

        if params.contains("aspect") {
            if let Some(key) = first_rect_key {
                return Err(invalid(format!(
                    "'aspect' cannot be combined with '{key}'; a crop is either a rectangle \
                     (x, y, width, height) or an aspect (aspect, gravity)"
                )));
            }
            let aspect = params
                .get_str("aspect")
                .ok_or_else(|| invalid("missing required key 'aspect'".to_owned()))?;
            let (a, b) = parse_aspect(aspect).map_err(invalid)?;
            let gravity = match params.get_str("gravity") {
                // The table already limited it to the nine names.
                Some(name) => Gravity::from_str(name).map_err(invalid)?,
                None => Gravity::Center,
            };
            return Ok(Crop {
                form: Form::Aspect { a, b, gravity },
                params: params.clone(),
            });
        }

        if params.contains("gravity") {
            return Err(invalid(
                "'gravity' is only used with 'aspect'; a rectangle crop is placed by x and y"
                    .to_owned(),
            ));
        }
        if first_rect_key.is_none() {
            return Err(invalid(
                "crop needs a rectangle (x, y, width, height) or an aspect (aspect, and \
                 optionally gravity)"
                    .to_owned(),
            ));
        }
        if let Some(missing) = RECT_KEYS.iter().find(|k| !params.contains(k)) {
            return Err(invalid(format!(
                "a rectangle crop needs x, y, width and height; '{missing}' is missing"
            )));
        }

        // The table has checked each value, so a present key reads.
        let read = |key: &str| {
            params
                .get_f64(key)
                .ok_or_else(|| invalid(format!("'{key}' is not a number")))
        };
        let (x, y, width, height) = (read("x")?, read("y")?, read("width")?, read("height")?);

        // Operands print as written, never as a computed sum.
        if x + width > 1.0 {
            return Err(invalid(format!(
                "'x' + 'width' is {} + {}, past the right edge; together they must be at most 1",
                written(params, "x"),
                written(params, "width")
            )));
        }
        if y + height > 1.0 {
            return Err(invalid(format!(
                "'y' + 'height' is {} + {}, past the bottom edge; together they must be at most 1",
                written(params, "y"),
                written(params, "height")
            )));
        }

        Ok(Crop {
            form: Form::Rect {
                x,
                y,
                width,
                height,
            },
            params: params.clone(),
        })
    }
}

/// The rectangle's edges in a `iw`×`ih` image: `(left, top, right, bottom)`.
///
/// Each edge is rounded on its own, in `f64`, so it sits within half a pixel
/// of its exact position. (Rounding the size instead would let the far edge
/// drift by a whole pixel.) `f64::round` takes halves away from zero, which
/// is up for these non-negative values, and it is plain IEEE arithmetic, so
/// native and wasm agree on every integer.
fn rect_edges(x: f64, y: f64, w: f64, h: f64, iw: u32, ih: u32) -> (u32, u32, u32, u32) {
    let (fw, fh) = (f64::from(iw), f64::from(ih));
    (
        (x * fw).round() as u32,
        (y * fh).round() as u32,
        ((x + w) * fw).round() as u32,
        ((y + h) * fh).round() as u32,
    )
}

/// The size of the largest `a`:`b` rectangle that fits an `iw`×`ih` image.
///
/// Exact integer maths: the short side is `round(long · b / a)` with halves up.
/// A side is 0 only when the ratio is so extreme the short side rounds away;
/// `apply` refuses that rather than widening it.
fn aspect_size(a: u32, b: u32, iw: u32, ih: u32) -> (u32, u32) {
    let (a, b, iw, ih) = (u64::from(a), u64::from(b), u64::from(iw), u64::from(ih));
    let (cw, ch) = if iw * b <= ih * a {
        // At least as tall as a:b, so the crop takes the full width.
        (iw, (2 * iw * b + a) / (2 * a))
    } else {
        (((2 * ih * a + b) / (2 * b)), ih)
    };
    // Either side is at most its image side, so these never truncate.
    (cw as u32, ch as u32)
}

/// Parse `"W:H"`: two whole numbers from 1 to 65535. On failure the error is
/// the whole reason; a decimal ratio gets its whole-number spelling.
fn parse_aspect(s: &str) -> Result<(u32, u32), String> {
    let generic = || {
        format!(
            "'aspect' must be two whole numbers from 1 to 65535 written W:H, such as \"16:9\"; \
             got {s:?}"
        )
    };
    let sides: Vec<&str> = s.split(':').collect();
    let [left, right] = sides.as_slice() else {
        return Err(generic());
    };

    // Digit by digit, not `u32::from_str`, which accepts a leading '+'.
    let whole = |side: &str| -> Option<u32> {
        if side.is_empty() || !side.bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let mut value: u64 = 0;
        for c in side.bytes() {
            value = value * 10 + u64::from(c - b'0');
            if value > MAX_ASPECT_SIDE {
                return None;
            }
        }
        (value >= 1).then_some(value as u32)
    };
    if let (Some(a), Some(b)) = (whole(left), whole(right)) {
        return Ok((a, b));
    }

    match decimal_spelling(left, right) {
        Some((a, b)) => Err(format!(
            "'aspect' must be two whole numbers written W:H; write {s:?} as \"{a}:{b}\""
        )),
        None => Err(generic()),
    }
}

/// The reduced whole-number spelling of a decimal ratio such as `2.39:1`
/// (`239:100`), when both sides are plain decimals, at least one has a point,
/// and the result fits an aspect. `None` otherwise.
fn decimal_spelling(left: &str, right: &str) -> Option<(u64, u64)> {
    // (digits with the point removed, count of digits after the point)
    let split = |side: &str| -> Option<(String, usize, bool)> {
        let mut parts = side.split('.');
        let int = parts.next()?;
        let frac = parts.next();
        if parts.next().is_some() {
            return None;
        }
        let digits = |p: &str| !p.is_empty() && p.bytes().all(|c| c.is_ascii_digit());
        match frac {
            None if digits(int) => Some((int.to_owned(), 0, false)),
            Some(f) if digits(int) && digits(f) => Some((format!("{int}{f}"), f.len(), true)),
            _ => None,
        }
    };
    let (ld, lf, l_point) = split(left)?;
    let (rd, rf, r_point) = split(right)?;
    if !(l_point || r_point) {
        return None;
    }
    // Scale both sides to the longer fractional part, with checked maths so a
    // hostile 40-digit side is a plain refusal.
    let places = lf.max(rf);
    let scaled = |digits: &str, frac: usize| -> Option<u128> {
        let mut v: u128 = digits.parse().ok()?;
        for _ in 0..(places - frac) {
            v = v.checked_mul(10)?;
        }
        Some(v)
    };
    let (mut a, mut b) = (scaled(&ld, lf)?, scaled(&rd, rf)?);
    if a == 0 || b == 0 {
        return None;
    }
    let (mut p, mut q) = (a, b);
    while q != 0 {
        (p, q) = (q, p % q);
    }
    a /= p;
    b /= p;
    let max = u128::from(MAX_ASPECT_SIDE);
    (a <= max && b <= max).then_some((a as u64, b as u64))
}

impl Operation for Crop {
    fn name(&self) -> &'static str {
        "crop"
    }

    fn params(&self) -> OperationParams {
        self.params.clone()
    }

    /// Cut the region. A crop that rounds to 0 px is an error here, in `apply`,
    /// and deliberately not through [`Operation::refusal`]: a native pipeline
    /// skips a refusing step and still writes the image, which would write the
    /// uncropped image and exit 0.
    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        let fail = |reason: String| OperationError::Apply { op: "crop", reason };
        let (iw, ih) = (img.pixels().width(), img.pixels().height());

        let (left, top, cw, ch) = match self.form {
            Form::Rect {
                x,
                y,
                width,
                height,
            } => {
                let (left, top, right, bottom) = rect_edges(x, y, width, height, iw, ih);
                (
                    left,
                    top,
                    right.saturating_sub(left),
                    bottom.saturating_sub(top),
                )
            }
            Form::Aspect { a, b, gravity } => {
                let (cw, ch) = aspect_size(a, b, iw, ih);
                let (left, top) = gravity.placement(iw, ih, cw, ch, 0);
                // Placement is non-negative, and at most the image's side.
                (left as u32, top as u32, cw, ch)
            }
        };

        // Unreachable after `from_params`; here so that a removed load check
        // can never hand `crop_imm` a rectangle it would silently clamp.
        let right = u64::from(left) + u64::from(cw);
        let bottom = u64::from(top) + u64::from(ch);
        if right > u64::from(iw) {
            return Err(fail(format!(
                "internal: the crop's right edge {right} is past the image's {iw} px width"
            )));
        }
        if bottom > u64::from(ih) {
            return Err(fail(format!(
                "internal: the crop's bottom edge {bottom} is past the image's {ih} px height"
            )));
        }
        if cw == 0 || ch == 0 {
            return Err(fail(format!(
                "the crop rounds to {cw}×{ch} px on this {iw}×{ih} px image; it must keep at \
                 least 1 px each way"
            )));
        }

        let cropped = img.pixels().crop_imm(left, top, cw, ch);
        Ok(img.with_pixels(cropped))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ::image::{
        DynamicImage, GrayAlphaImage, GrayImage, ImageBuffer, ImageFormat, LumaA, Rgb, Rgba,
    };

    use super::*;
    use crate::image::Image;

    // ── Helpers ─────────────────────────────────────────────────────────────

    fn value_of(v: &str) -> toml::Value {
        // `v` is a TOML value as a recipe author writes it: `0.5`, `0`, `"16:9"`.
        let table: toml::Table = format!("v = {v}").parse().expect("a TOML value");
        table["v"].clone()
    }

    fn params(pairs: &[(&str, &str)]) -> OperationParams {
        OperationParams::from_map(
            pairs
                .iter()
                .map(|(k, v)| ((*k).to_owned(), value_of(v)))
                .collect::<BTreeMap<_, _>>(),
        )
    }

    fn crop(pairs: &[(&str, &str)]) -> Crop {
        Crop::from_params(&params(pairs)).expect("params are valid")
    }

    fn wrap(px: DynamicImage) -> Image {
        Image::from_parts(px, ImageFormat::Png, None)
    }

    /// An rgb8 image whose every pixel is unique (for dimensions under 4096).
    fn unique_rgb8(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgb8(ImageBuffer::from_fn(w, h, |x, y| {
            Rgb([x as u8, y as u8, (((x >> 8) << 4) | (y >> 8)) as u8])
        }))
    }

    fn unique_rgb8_pixel(x: u32, y: u32) -> Rgb<u8> {
        Rgb([x as u8, y as u8, (((x >> 8) << 4) | (y >> 8)) as u8])
    }

    fn apply_to(c: &Crop, px: DynamicImage) -> DynamicImage {
        c.apply(wrap(px)).expect("crop applies").pixels().clone()
    }

    struct XorShift(u64);
    impl XorShift {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        /// A uniform draw in [0, 1).
        fn unit(&mut self) -> f64 {
            (self.next() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
        }
    }

    // ── Rectangle maths ─────────────────────────────────────────────────────

    #[test]
    fn rect_edges_exact_example() {
        assert_eq!(
            rect_edges(0.123, 0.2345, 0.5, 0.4321, 1200, 800),
            (148, 188, 748, 533)
        );
        assert_eq!(
            rect_edges(0.123, 0.2345, 0.5, 0.4321, 6000, 4000),
            (738, 938, 3738, 2666)
        );
    }

    #[test]
    fn crop_edges_within_half_a_pixel() {
        let mut cases = 0u64;
        for size in 1u32..=64 {
            for n in [97u32, 8] {
                for i in 0..=n {
                    for j in 1..=n {
                        let x = f64::from(i) / f64::from(n);
                        let width = f64::from(j) / f64::from(n);
                        if x + width > 1.0 {
                            continue;
                        }
                        cases += 1;
                        // Both axes through one call: the same fractions on a
                        // square image read the horizontal and vertical edges.
                        let (left, top, right, bottom) = rect_edges(x, x, width, width, size, size);
                        let s = f64::from(size);
                        for (edge, exact, name) in [
                            (left, x * s, "left"),
                            (top, x * s, "top"),
                            (right, (x + width) * s, "right"),
                            (bottom, (x + width) * s, "bottom"),
                        ] {
                            assert!(
                                (f64::from(edge) - exact).abs() <= 0.5,
                                "{name}: x={x} width={width} size={size}: edge {edge}, exact {exact}"
                            );
                        }
                    }
                }
            }
        }
        // 64 sizes x (97·98/2 + 8·9/2) fraction pairs, counted once.
        assert_eq!(cases, 306_496, "every case ran");
    }

    #[test]
    fn crop_fraction_agrees_at_1200_and_6000() {
        let mut r = XorShift(0x5EED_0145);
        let pair = |r: &mut XorShift| loop {
            let (a, b) = (r.unit(), r.unit());
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            if hi - lo > 0.0 {
                return (lo, hi - lo);
            }
        };
        // 0.4 exactly, plus f64 noise: l / 5.0 - s lands at 0.4000000000000057. 0.4 is what this
        // seed measures, not the general bound: near an f64 half-pixel tie the two sizes can round
        // apart and drift reaches 0.6 (width = 0.009583333333333333 gives 12 px at 1200, 57 at
        // 6000). A different seed or case count can reach 0.6 on correct code.
        const BOUND: f64 = 0.4 + 1e-9;
        let mut cases = 0u32;
        let mut worst = 0.0f64;
        while cases < 200_000 {
            let (x, width) = pair(&mut r);
            let (y, height) = pair(&mut r);
            let small = rect_edges(x, y, width, height, 1200, 800);
            let large = rect_edges(x, y, width, height, 6000, 4000);
            for (s, l) in [
                (small.0, large.0),
                (small.1, large.1),
                (small.2, large.2),
                (small.3, large.3),
            ] {
                let drift = (f64::from(l) / 5.0 - f64::from(s)).abs();
                worst = worst.max(drift);
                assert!(
                    drift <= BOUND,
                    "x={x} y={y} width={width} height={height}: {s} at 1200, {l} at 6000 (drift {drift} > {BOUND})"
                );
            }
            cases += 1;
        }
        assert_eq!(cases, 200_000);
        assert!(worst > 0.0, "the sweep saw rounding at all");
    }

    #[test]
    fn halves_round_up_and_fractions_read_as_f64() {
        // 0.0009375 * 1600 = 1.5: an f32 read gives 1.49999997 and rounds down.
        let c = crop(&[
            ("x", "0.0009375"),
            ("y", "0"),
            ("width", "0.5"),
            ("height", "1"),
        ]);
        let src = unique_rgb8(1600, 1067);
        let out = apply_to(&c, src);
        assert_eq!((out.width(), out.height()), (800, 1067));
        assert_eq!(
            out.to_rgb8().get_pixel(0, 0),
            &unique_rgb8_pixel(2, 0),
            "left edge is 2, not 1"
        );

        // 0.25 * 1730 = 432.5 rounds up to 433.
        let c = crop(&[
            ("x", "0.1"),
            ("y", "0.25"),
            ("width", "0.5"),
            ("height", "0.4321"),
        ]);
        let out = apply_to(&c, unique_rgb8(2600, 1730));
        assert_eq!((out.width(), out.height()), (1300, 747));
        assert_eq!(out.to_rgb8().get_pixel(0, 0), &unique_rgb8_pixel(260, 433));
    }

    // ── Aspect maths ────────────────────────────────────────────────────────

    #[test]
    fn aspect_fit_worked_example() {
        assert_eq!(aspect_size(5, 4, 2600, 1730), (2163, 1730));
        assert_eq!(aspect_size(5, 4, 1730, 2600), (1730, 1384));
        assert_eq!(aspect_size(16, 9, 2600, 1730), (2600, 1463));
        assert_eq!(aspect_size(32, 18, 2600, 1730), (2600, 1463));
        assert_eq!(aspect_size(10000, 1, 1600, 1067), (1600, 0));
        assert_eq!(aspect_size(1, 10000, 1600, 1067), (0, 1067));
    }

    #[test]
    fn aspect_never_exceeds_the_image() {
        let mut checked = 0u32;
        for (a, b) in [
            (1u32, 1u32),
            (16, 9),
            (9, 16),
            (239, 100),
            (65535, 1),
            (1, 65535),
        ] {
            for iw in [1u32, 2, 3, 7, 64, 1067, 1600, 50_000] {
                for ih in [1u32, 2, 5, 48, 1067, 2600, 50_000] {
                    let (cw, ch) = aspect_size(a, b, iw, ih);
                    assert!(cw <= iw && ch <= ih, "{a}:{b} on {iw}x{ih} gave {cw}x{ch}");
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 6 * 8 * 7);
    }

    #[test]
    fn aspect_gravity_places_all_nine() {
        // (image w, image h, aspect, gravity, expected offset, expected size)
        let landscape = (2600u32, 1730u32, "5:4", (2163u32, 1730u32));
        let portrait = (1730u32, 2600u32, "5:4", (1730u32, 1384u32));
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
        let landscape_x = [218, 218, 218, 437, 0, 437, 0, 437, 0];
        let portrait_y = [608, 0, 1216, 608, 608, 0, 0, 1216, 1216];

        // (image w, image h, aspect, gravity, offset, size)
        type Row<'a> = (u32, u32, &'a str, &'a str, (u32, u32), (u32, u32));
        let mut rows: Vec<Row> = Vec::new();
        for (i, name) in names.iter().enumerate() {
            rows.push((
                landscape.0,
                landscape.1,
                landscape.2,
                name,
                (landscape_x[i], 0),
                landscape.3,
            ));
            rows.push((
                portrait.0,
                portrait.1,
                portrait.2,
                name,
                (0, portrait_y[i]),
                portrait.3,
            ));
        }
        rows.push((2600, 1730, "16:9", "center", (0, 133), (2600, 1463)));

        let mut mismatches = Vec::new();
        for (w, h, aspect, gravity, (left, top), (cw, ch)) in &rows {
            let aspect_toml = format!("\"{aspect}\"");
            let gravity_toml = format!("\"{gravity}\"");
            let c = crop(&[
                ("aspect", aspect_toml.as_str()),
                ("gravity", gravity_toml.as_str()),
            ]);
            // An apply error is a failing row too, reported with the rest.
            let out = match c.apply(wrap(unique_rgb8(*w, *h))) {
                Ok(img) => img.pixels().clone(),
                Err(e) => {
                    mismatches.push(format!("{gravity} on {w}x{h}: {e}"));
                    continue;
                }
            };
            let got_px = *out.to_rgb8().get_pixel(0, 0);
            if (out.width(), out.height()) != (*cw, *ch) || got_px != unique_rgb8_pixel(*left, *top)
            {
                mismatches.push(format!(
                    "{gravity} on {w}x{h}: wanted {cw}x{ch} at ({left}, {top}), got {}x{} starting {got_px:?}",
                    out.width(),
                    out.height()
                ));
            }
        }
        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
        assert_eq!(rows.len(), 19, "9 gravities x 2 orientations + 16:9");
    }

    // ── Pixels and colour types ─────────────────────────────────────────────

    #[test]
    fn luma_crops_are_exact() {
        let c = crop(&[
            ("x", "0.25"),
            ("y", "0.25"),
            ("width", "0.5"),
            ("height", "0.5"),
        ]);

        let l8 = DynamicImage::ImageLuma8(GrayImage::from_fn(64, 48, |x, y| {
            ::image::Luma([(x * 3 + y * 5) as u8])
        }));
        let out = apply_to(&c, l8.clone());
        assert_eq!(out.color(), l8.color());
        assert_eq!((out.width(), out.height()), (32, 24));
        let (s, o) = (l8.to_luma8(), out.to_luma8());
        for j in 0..24 {
            for i in 0..32 {
                assert_eq!(
                    o.get_pixel(i, j),
                    s.get_pixel(16 + i, 12 + j),
                    "L8 ({i}, {j})"
                );
            }
        }

        let la8 = DynamicImage::ImageLumaA8(GrayAlphaImage::from_fn(64, 48, |x, y| {
            LumaA([(x * 3 + y * 5) as u8, (x + y * 7) as u8])
        }));
        let out = apply_to(&c, la8.clone());
        assert_eq!(out.color(), la8.color());
        let (s, o) = (la8.to_luma_alpha8(), out.to_luma_alpha8());
        for j in 0..24 {
            for i in 0..32 {
                assert_eq!(
                    o.get_pixel(i, j),
                    s.get_pixel(16 + i, 12 + j),
                    "La8 ({i}, {j})"
                );
            }
        }
    }

    #[test]
    fn identity_crop_is_byte_equal() {
        let c = crop(&[("x", "0"), ("y", "0"), ("width", "1"), ("height", "1")]);
        let rgb = unique_rgb8(37, 29);
        let rgba16 = DynamicImage::ImageRgba16(ImageBuffer::from_fn(37, 29, |x, y| {
            Rgba([
                x as u16 * 1000,
                y as u16 * 2000,
                (x + y) as u16 * 300,
                65535 - x as u16,
            ])
        }));
        for (name, src) in [("rgb8", rgb), ("rgba16", rgba16)] {
            let out = apply_to(&c, src.clone());
            assert_eq!(out.color(), src.color(), "{name}: colour type");
            assert_eq!(out.as_bytes(), src.as_bytes(), "{name}: bytes");
        }
    }

    // ── The step as written ─────────────────────────────────────────────────

    #[test]
    fn params_round_trip_as_written() {
        let rect = params(&[("x", "0"), ("y", "0.25"), ("width", "1"), ("height", "0.5")]);
        let c = Crop::from_params(&rect).expect("rect builds");
        assert_eq!(c.params(), rect, "integers stay integers");

        let aspect = params(&[("aspect", "\"16:9\"")]);
        let c = Crop::from_params(&aspect).expect("aspect builds");
        assert_eq!(c.params(), aspect, "an absent gravity stays absent");
        assert!(!c.params().contains("gravity"));

        let with_gravity = params(&[("aspect", "\"16:9\""), ("gravity", "\"west\"")]);
        let c = Crop::from_params(&with_gravity).expect("aspect + gravity builds");
        assert_eq!(c.params(), with_gravity, "a given gravity is kept");
    }

    #[test]
    fn refusal_hook_is_not_used() {
        // A zero-pixel crop fails in `apply`; a skippable refusal would write
        // the uncropped image on every native pipeline.
        let c = crop(&[
            ("x", "0.5"),
            ("y", "0"),
            ("width", "0.0001"),
            ("height", "1"),
        ]);
        let img = wrap(unique_rgb8(1600, 1067));
        assert!(c.refusal(&img).is_none());
        assert!(c.apply(img).is_err(), "and apply is where it fails");
    }
}
