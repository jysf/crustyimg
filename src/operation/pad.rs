//! `pad`: grow the canvas around the image and fill the new pixels.
//!
//! A pad is one of two forms, told apart by which keys the step carries:
//!
//! - an **aspect**: `aspect = "W:H"` and an optional `gravity`. The canvas is
//!   the smallest `W:H` rectangle that holds the image, and `gravity` says
//!   where the image sits in it;
//! - a **border**: `border = f`, a fraction of the image's long edge, added on
//!   every side so the border is the same width all round.
//!
//! `color` is required: a hex colour with any alpha (`#RRGGBBAA` is straight
//! alpha, stored as written) or the keyword `transparent`, which is
//! `#00000000`, zero alpha over black. Every input pixel is copied unchanged to
//! its place in the canvas and every new pixel is the fill, alpha included:
//! nothing is blended, a pad computes no pixel, so native and wasm agree
//! exactly. The bit depth is the input's (an 8-bit fill channel `c` is
//! `c × 257` at 16 bits and `c / 255` for a float image).
//!
//! The colour type is the input's, widened only where the fill needs it, which
//! is the colour-type rule applied to a fill: a channel is kept when it holds
//! a value that is not the neutral one, and gained when the fill needs it.
//!
//! - a grey image padded with a colour that is not grey widens to RGB (RGBA if
//!   it had alpha), because the fill genuinely adds chroma;
//! - an image with no alpha padded with a fill whose alpha is below `ff` gains
//!   alpha: grey plus alpha for a grey fill (which includes `transparent`),
//!   RGBA otherwise;
//! - an image that already has alpha keeps it.
//!
//! An image already at the aspect is returned unchanged with its own type,
//! whatever the fill, because no fill pixel is written.
//!
//! A recipe whose fill alpha the output format can't store (JPEG has none and
//! GIF keeps only `00` and `ff`) is refused before it runs; see
//! [`Operation::introduced_alpha`].
//!
//! The step acts on the image as it receives it and keeps the image's
//! metadata in memory for a later step, as `crop` does; it writes none.
//!
//! Both conditions that depend on the image (a border that rounds to 0 px, a
//! canvas past the size caps) are errors from `apply`, not refusals, because
//! skipping the step would write an unpadded image.

use std::borrow::Cow;
use std::str::FromStr;

use ::image::imageops;
use ::image::{DynamicImage, ImageBuffer, Luma, LumaA, Pixel, Rgb, Rgba};

use super::crop::parse_aspect;
use super::registry::{self, written};
use super::{
    Gravity, IntroducedAlpha, Operation, OperationError, OperationParams, ParamKind, ParamSpec,
    RegistryError, GRAVITY_NAMES, MAX_AREA, MAX_EDGE,
};
use crate::image::Image;

/// The step keys `pad` accepts, in the order an unknown-key error lists them.
pub const PAD_PARAMS: &[ParamSpec] = &[
    ParamSpec {
        key: "aspect",
        kind: ParamKind::Text,
    },
    ParamSpec {
        key: "gravity",
        kind: ParamKind::OneOf(GRAVITY_NAMES),
    },
    ParamSpec {
        key: "border",
        kind: ParamKind::Number {
            min: 0.0,
            min_inclusive: false,
            max: Some(1.0),
        },
    },
    ParamSpec {
        key: "color",
        kind: ParamKind::Text,
    },
];

#[derive(Debug, Clone, Copy)]
enum Form {
    Aspect {
        a: u32,
        b: u32,
        gravity: Gravity,
    },
    /// A fraction of the long edge, read as an `f64`.
    Border(f64),
}

/// A pad step: an aspect or a border, and a fill with any alpha.
///
/// Holds the step's keys exactly as written, so [`Operation::params`] gives
/// back an integer `border = 1` as an integer, a hex colour in the case it was
/// written in, and an absent `gravity` as absent.
#[derive(Debug, Clone)]
pub struct Pad {
    form: Form,
    /// The fill's red, green, blue and alpha, as written (never blended).
    fill: [u8; 4],
    params: OperationParams,
}

impl Pad {
    /// Parse and validate a pad step. Never panics.
    ///
    /// Runs the key table first (unknown key, wrong type, out-of-range
    /// border), then the rules that need more than one key: which form the
    /// keys make, `gravity` with a border, the aspect's syntax, and the
    /// colour's presence and syntax.
    pub fn from_params(params: &OperationParams) -> Result<Pad, RegistryError> {
        let invalid = |reason: String| RegistryError::InvalidParams { op: "pad", reason };
        registry::check_params("pad", PAD_PARAMS, params).map_err(invalid)?;

        let has_aspect = params.contains("aspect");
        let has_border = params.contains("border");
        if has_aspect && has_border {
            return Err(invalid(
                "'aspect' cannot be combined with 'border'; a pad is either an aspect \
                 (aspect, gravity) or a border"
                    .to_owned(),
            ));
        }
        if !has_aspect && !has_border {
            return Err(invalid(
                "pad needs an aspect (aspect, and optionally gravity) or a border".to_owned(),
            ));
        }
        if has_border && params.contains("gravity") {
            return Err(invalid(
                "'gravity' is only used with 'aspect'; a border pad adds the same width on \
                 every side"
                    .to_owned(),
            ));
        }

        let form = if has_aspect {
            let aspect = params
                .get_str("aspect")
                .ok_or_else(|| invalid("missing required key 'aspect'".to_owned()))?;
            let (a, b) = parse_aspect(aspect).map_err(invalid)?;
            let gravity = match params.get_str("gravity") {
                // The table already limited it to the nine names.
                Some(name) => Gravity::from_str(name).map_err(invalid)?,
                None => Gravity::Center,
            };
            Form::Aspect { a, b, gravity }
        } else {
            // `f64`, never `f32`: a fraction keeps its full width so that a
            // length near a half pixel rounds the same way on every target.
            let fraction = params
                .get_f64("border")
                .ok_or_else(|| invalid("'border' is not a number".to_owned()))?;
            Form::Border(fraction)
        };

        let color = params.get_str("color").ok_or_else(|| {
            invalid(
                "missing required key 'color'; a pad needs a fill colour such as \"#ffffff\""
                    .to_owned(),
            )
        })?;
        let fill = super::parse_fill_color(color).map_err(|_| {
            invalid(format!(
                "'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got {color:?}"
            ))
        })?;

        Ok(Pad {
            form,
            fill,
            params: params.clone(),
        })
    }

    /// Where the pad puts things on an `iw`×`ih` image:
    /// `(canvas_w, canvas_h, left, top)`, or the reason `apply` fails.
    ///
    /// Pure and allocation-free. The canvas is computed in `u64` and checked
    /// against the size caps before any cast to `u32`, so no input size can
    /// reach an allocation that is too large.
    fn layout(&self, iw: u32, ih: u32) -> Result<(u32, u32, u32, u32), String> {
        let (w, h) = (u64::from(iw), u64::from(ih));

        let (cw, ch, what) = match self.form {
            Form::Aspect { a, b, .. } => {
                let (cw, ch) = aspect_canvas(a, b, w, h);
                let aspect = self.params.get_str("aspect").unwrap_or_default();
                (cw, ch, format!("'aspect' = {aspect:?}"))
            }
            Form::Border(fraction) => {
                let long = iw.max(ih);
                let px = (fraction * f64::from(long)).round() as u64;
                let text = written(&self.params, "border");
                if px == 0 {
                    return Err(format!(
                        "the border rounds to 0 px on this {iw}×{ih} px image ('border' = \
                         {text} of its {long} px long edge); it must add at least 1 px"
                    ));
                }
                (w + 2 * px, h + 2 * px, format!("'border' = {text}"))
            }
        };

        let max_edge = u64::from(MAX_EDGE);
        if cw > max_edge || ch > max_edge {
            return Err(format!(
                "{what} on this {iw}×{ih} px image needs a {cw}×{ch} px canvas, which exceeds \
                 the maximum edge limit of {MAX_EDGE} px"
            ));
        }
        if cw * ch > MAX_AREA {
            return Err(format!(
                "{what} on this {iw}×{ih} px image needs a {cw}×{ch} px canvas = {} px, which \
                 exceeds the maximum area of {MAX_AREA} px",
                cw * ch
            ));
        }
        // Both sides are within MAX_EDGE now, so these never truncate.
        let (cw, ch) = (cw as u32, ch as u32);

        let (left, top) = match self.form {
            // Floors the centre: with an odd leftover the extra fill pixel
            // goes right or bottom, as it does for a crop.
            Form::Aspect { gravity, .. } => {
                let (x, y) = gravity.placement(cw, ch, iw, ih, 0);
                (x as u32, y as u32)
            }
            Form::Border(_) => {
                let px = (cw - iw) / 2;
                (px, px)
            }
        };
        Ok((cw, ch, left, top))
    }
}

/// The canvas of the smallest `a`:`b` rectangle that holds a `w`×`h` image.
///
/// The axis that is at least as wide as `a:b` keeps its length and the other
/// grows to the aspect, to the nearest pixel with halves up. Exact integer
/// maths in `u64`: no value reaches 2^50 for sides up to `u32::MAX` and an
/// aspect of 65535, so nothing overflows and no `f64` near-tie can arise.
fn aspect_canvas(a: u32, b: u32, w: u64, h: u64) -> (u64, u64) {
    let (a, b) = (u64::from(a), u64::from(b));
    if w * b >= h * a {
        (w, (2 * w * b + a) / (2 * a))
    } else {
        ((2 * h * a + b) / (2 * b), h)
    }
}

/// A `cw`×`ch` canvas of `fill` with `src` copied in at `(left, top)`. An
/// exact copy: nothing is blended.
fn grow<P>(
    src: &ImageBuffer<P, Vec<P::Subpixel>>,
    fill: P,
    (cw, ch, left, top): (u32, u32, u32, u32),
) -> ImageBuffer<P, Vec<P::Subpixel>>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
{
    let mut canvas = ImageBuffer::from_pixel(cw, ch, fill);
    imageops::replace(&mut canvas, src, i64::from(left), i64::from(top));
    canvas
}

/// One 8-bit fill channel at 16 bits: `c × 257`, so `0xff` is 65535.
fn wide(c: u8) -> u16 {
    (u16::from(c) << 8) | u16::from(c)
}

/// One 8-bit fill channel as a float sample.
fn unit(c: u8) -> f32 {
    f32::from(c) / 255.0
}

impl Operation for Pad {
    fn name(&self) -> &'static str {
        "pad"
    }

    fn params(&self) -> OperationParams {
        self.params.clone()
    }

    /// Grow the canvas. Fails for a border that rounds to 0 px and for a
    /// canvas past the size caps, before anything is allocated; neither is a
    /// refusal, because skipping the step would write an unpadded image.
    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        let fail = |reason: String| OperationError::Apply { op: "pad", reason };
        let (iw, ih) = (img.pixels().width(), img.pixels().height());
        let layout = self.layout(iw, ih).map_err(fail)?;
        if (layout.0, layout.1) == (iw, ih) {
            // Already at the aspect: no fill pixel is written, so the image
            // keeps its type too.
            return Ok(img);
        }

        let [r, g, b, a] = self.fill;
        let chromatic = r != g || g != b;
        let translucent = a != u8::MAX;

        // Widen first, so the fill fits: a grey image takes a colour that is
        // not grey as RGB, and an image with no alpha takes a fill below `ff`
        // as alpha. The conversions copy every sample exactly (luma to all
        // three colour channels, alpha at its maximum).
        let pixels = img.pixels();
        let src: Cow<'_, DynamicImage> = match (pixels, chromatic, translucent) {
            (DynamicImage::ImageLuma8(_), true, false) => Cow::Owned(pixels.to_rgb8().into()),
            (DynamicImage::ImageLuma8(_), true, true) => Cow::Owned(pixels.to_rgba8().into()),
            (DynamicImage::ImageLuma8(_), false, true) => {
                Cow::Owned(pixels.to_luma_alpha8().into())
            }
            (DynamicImage::ImageLumaA8(_), true, _) => Cow::Owned(pixels.to_rgba8().into()),
            (DynamicImage::ImageLuma16(_), true, false) => Cow::Owned(pixels.to_rgb16().into()),
            (DynamicImage::ImageLuma16(_), true, true) => Cow::Owned(pixels.to_rgba16().into()),
            (DynamicImage::ImageLuma16(_), false, true) => {
                Cow::Owned(pixels.to_luma_alpha16().into())
            }
            (DynamicImage::ImageLumaA16(_), true, _) => Cow::Owned(pixels.to_rgba16().into()),
            (DynamicImage::ImageRgb8(_), _, true) => Cow::Owned(pixels.to_rgba8().into()),
            (DynamicImage::ImageRgb16(_), _, true) => Cow::Owned(pixels.to_rgba16().into()),
            (DynamicImage::ImageRgb32F(_), _, true) => Cow::Owned(pixels.to_rgba32f().into()),
            (other, _, _) => Cow::Borrowed(other),
        };

        // A new pixel is the fill, alpha included, copied and never blended.
        // An opaque fill writes `ff`, 65535 and 1.0, the type's maximum.
        let out: DynamicImage = match src.as_ref() {
            DynamicImage::ImageLuma8(s) => grow(s, Luma([r]), layout).into(),
            DynamicImage::ImageLumaA8(s) => grow(s, LumaA([r, a]), layout).into(),
            DynamicImage::ImageRgb8(s) => grow(s, Rgb([r, g, b]), layout).into(),
            DynamicImage::ImageRgba8(s) => grow(s, Rgba([r, g, b, a]), layout).into(),
            DynamicImage::ImageLuma16(s) => grow(s, Luma([wide(r)]), layout).into(),
            DynamicImage::ImageLumaA16(s) => grow(s, LumaA([wide(r), wide(a)]), layout).into(),
            DynamicImage::ImageRgb16(s) => grow(s, Rgb([wide(r), wide(g), wide(b)]), layout).into(),
            DynamicImage::ImageRgba16(s) => {
                grow(s, Rgba([wide(r), wide(g), wide(b), wide(a)]), layout).into()
            }
            DynamicImage::ImageRgb32F(s) => {
                grow(s, Rgb([unit(r), unit(g), unit(b)]), layout).into()
            }
            DynamicImage::ImageRgba32F(s) => {
                grow(s, Rgba([unit(r), unit(g), unit(b), unit(a)]), layout).into()
            }
            _ => return Err(fail("this image's colour type cannot be padded".to_owned())),
        };
        Ok(img.with_pixels(out))
    }

    /// The fill's alpha, when it is below `ff`: every new pixel carries it.
    fn introduced_alpha(&self) -> Option<IntroducedAlpha> {
        let alpha = self.fill[3];
        (alpha != u8::MAX).then(|| IntroducedAlpha {
            alpha,
            key: "color",
            written: self.params.get_str("color").unwrap_or_default().to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ::image::{GrayImage, ImageFormat};

    use super::*;

    // ── Helpers ─────────────────────────────────────────────────────────────

    /// A pad from the body of a step, written as TOML key lines.
    fn pad(body: &str) -> Pad {
        let table: toml::Table = body.parse().expect("TOML");
        let map: BTreeMap<String, toml::Value> = table.into_iter().collect();
        Pad::from_params(&OperationParams::from_map(map)).expect("params are valid")
    }

    fn aspect(a: &str) -> Pad {
        pad(&format!("aspect = \"{a}\"\ncolor = \"#ffffff\""))
    }

    fn border(f: &str) -> Pad {
        pad(&format!("border = {f}\ncolor = \"#ffffff\""))
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

    fn rgb8(w: u32, h: u32) -> DynamicImage {
        let mut r = XorShift(0x5EED_0151 ^ (u64::from(w) << 20) ^ u64::from(h));
        DynamicImage::ImageRgb8(ImageBuffer::from_fn(w, h, |_, _| {
            let n = r.next();
            Rgb([n as u8, (n >> 8) as u8, (n >> 16) as u8])
        }))
    }

    fn rgba16(w: u32, h: u32) -> DynamicImage {
        let mut r = XorShift(0x5EED_0152 ^ (u64::from(w) << 20) ^ u64::from(h));
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

    fn l8(w: u32, h: u32) -> DynamicImage {
        let mut r = XorShift(0x5EED_0153 ^ (u64::from(w) << 20) ^ u64::from(h));
        DynamicImage::ImageLuma8(GrayImage::from_fn(w, h, |_, _| Luma([r.next() as u8])))
    }

    fn wrap(px: DynamicImage) -> Image {
        Image::from_parts(px, ImageFormat::Png, None)
    }

    fn canvas_of(p: &Pad, w: u32, h: u32) -> (u32, u32) {
        let (cw, ch, _, _) = p.layout(w, h).expect("a layout");
        (cw, ch)
    }

    // ── The aspect canvas ───────────────────────────────────────────────────

    #[test]
    fn canvas_follows_the_binding_axis() {
        let rows: [(u32, u32, &str, (u32, u32)); 7] = [
            (4000, 4000, "16:9", (7111, 4000)),
            (4000, 4000, "9:16", (4000, 7111)),
            (6000, 4000, "4:5", (6000, 7500)),
            (1730, 2600, "4:5", (2080, 2600)),
            (1200, 801, "3:2", (1202, 801)),
            (1919, 1080, "16:9", (1920, 1080)),
            (4, 3, "1:1", (4, 4)),
        ];
        for (w, h, a, want) in rows {
            assert_eq!(canvas_of(&aspect(a), w, h), want, "{w}x{h} to {a}");
        }
    }

    #[test]
    fn canvas_sweep_never_smaller_and_within_half_a_pixel() {
        let aspects: [(u64, u64); 13] = [
            (1, 1),
            (16, 9),
            (9, 16),
            (4, 5),
            (3, 2),
            (2, 3),
            (40, 21),
            (239, 100),
            (7, 3),
            (65535, 1),
            (1, 65535),
            (65535, 65534),
            (1, 2),
        ];
        let (mut cases, mut ties) = (0u32, 0u32);
        for (a, b) in aspects {
            for w64 in 1..=64u64 {
                for h64 in 1..=64u64 {
                    cases += 1;
                    let (cw, ch) = aspect_canvas(a as u32, b as u32, w64, h64);
                    assert!(cw >= w64 && ch >= h64, "{a}:{b} on {w64}x{h64} shrank");
                    if w64 * b >= h64 * a {
                        assert_eq!(cw, w64, "{a}:{b} on {w64}x{h64}: width binds");
                        let err = (2 * ch * a).abs_diff(2 * w64 * b);
                        assert!(err <= a, "{a}:{b} on {w64}x{h64}: {ch} is off by over ½ px");
                        ties += u32::from(err == a);
                    } else {
                        assert_eq!(ch, h64, "{a}:{b} on {w64}x{h64}: height binds");
                        let err = (2 * cw * b).abs_diff(2 * h64 * a);
                        assert!(err <= b, "{a}:{b} on {w64}x{h64}: {cw} is off by over ½ px");
                        ties += u32::from(err == b);
                    }
                }
            }
        }
        assert_eq!(cases, 13 * 64 * 64, "every case ran");
        assert!(ties > 0, "no case reached exactly ½ px");
    }

    #[test]
    fn no_op_returns_the_input() {
        for (px, a) in [
            (rgb8(1200, 800), "3:2"),
            (rgb8(1918, 1079), "16:9"),
            (rgba16(1200, 800), "3:2"),
            (rgba16(1918, 1079), "16:9"),
        ] {
            let out = aspect(a).apply(wrap(px.clone())).expect("a no-op");
            assert_eq!(out.pixels().color(), px.color());
            assert_eq!(out.pixels().as_bytes(), px.as_bytes());
        }
        // A grey image with a colour fill that needs no padding stays grey.
        let px = l8(6, 4);
        let p = pad("aspect = \"3:2\"\ncolor = \"#123456\"");
        let out = p.apply(wrap(px.clone())).expect("a no-op");
        assert!(matches!(out.pixels(), DynamicImage::ImageLuma8(_)));
        assert_eq!(out.pixels().as_bytes(), px.as_bytes());
    }

    #[test]
    fn border_is_a_fraction_of_the_long_edge() {
        type Layout = (u32, u32, u32, u32);
        let rows: [(u32, u32, &str, Layout); 7] = [
            (40, 30, "0.1", (48, 38, 4, 4)),
            (30, 40, "0.1", (38, 48, 4, 4)),
            (1, 1, "1", (3, 3, 1, 1)),
            (4001, 2999, "0.013", (4105, 3103, 52, 52)),
            (40, 30, "0.0125", (42, 32, 1, 1)),
            // Half-pixel products that split f64 from f32: 0.01 * 50 is exactly 0.5 in f64
            // (rounds up to 1) but 0.49999999 in f32 (rounds to 0); 0.7 * 5 likewise (4 vs 3).
            (50, 10, "0.01", (52, 12, 1, 1)),
            (5, 3, "0.7", (13, 11, 4, 4)),
        ];
        for (w, h, f, want) in rows {
            assert_eq!(border(f).layout(w, h), Ok(want), "{w}x{h} at {f}");
        }
    }

    #[test]
    fn odd_remainder_puts_the_extra_fill_right_or_bottom() {
        let p = aspect("1:1");
        assert_eq!(p.layout(4, 3), Ok((4, 4, 0, 0)));
        assert_eq!(p.layout(3, 4), Ok((4, 4, 0, 0)));
        assert_eq!(p.layout(5, 2), Ok((5, 5, 0, 1)));

        // Through apply: row 3 of a 4×3 image padded to 4×4 is all fill.
        let src = rgb8(4, 3);
        let out = pad("aspect = \"1:1\"\ncolor = \"#123456\"")
            .apply(wrap(src.clone()))
            .expect("pads");
        let bytes = out.pixels().as_bytes();
        assert_eq!(&bytes[..4 * 3 * 3], src.as_bytes());
        for px in bytes[4 * 3 * 3..].as_chunks::<3>().0 {
            assert_eq!(px, &[0x12, 0x34, 0x56]);
        }
    }

    #[test]
    fn gravity_places_by_absolute_position() {
        // (gravity, offset on a 5×2 image, offset on a 2×5 image) at `1:1`.
        type Offset = (u32, u32);
        let table: [(&str, Offset, Offset); 9] = [
            ("center", (0, 1), (1, 0)),
            ("north", (0, 0), (1, 0)),
            ("south", (0, 3), (1, 0)),
            ("east", (0, 1), (3, 0)),
            ("west", (0, 1), (0, 0)),
            ("northeast", (0, 0), (3, 0)),
            ("northwest", (0, 0), (0, 0)),
            ("southeast", (0, 3), (3, 0)),
            ("southwest", (0, 3), (0, 0)),
        ];
        for (gravity, wide_at, tall_at) in table {
            let p = pad(&format!(
                "aspect = \"1:1\"\ngravity = \"{gravity}\"\ncolor = \"#ffffff\""
            ));
            assert_eq!(
                p.layout(5, 2),
                Ok((5, 5, wide_at.0, wide_at.1)),
                "{gravity} 5x2"
            );
            assert_eq!(
                p.layout(2, 5),
                Ok((5, 5, tall_at.0, tall_at.1)),
                "{gravity} 2x5"
            );
        }
    }

    #[test]
    fn caps_and_zero_px_fail_before_allocating() {
        let a1 = "the border rounds to 0 px on this 40×30 px image ('border' = 0.001 of its 40 px long edge); it must add at least 1 px";
        assert_eq!(border("0.001").layout(40, 30), Err(a1.to_owned()));
        let a1b = "the border rounds to 0 px on this 40×30 px image ('border' = 0.0124 of its 40 px long edge); it must add at least 1 px";
        assert_eq!(border("0.0124").layout(40, 30), Err(a1b.to_owned()));
        assert_eq!(border("0.0125").layout(40, 30), Ok((42, 32, 1, 1)));

        assert_eq!(
            aspect("65535:1").layout(1, 1),
            Err("'aspect' = \"65535:1\" on this 1×1 px image needs a 65535×1 px canvas, which exceeds the maximum edge limit of 50000 px".to_owned())
        );
        assert_eq!(
            border("1").layout(20000, 1),
            Err("'border' = 1 on this 20000×1 px image needs a 60000×40001 px canvas, which exceeds the maximum edge limit of 50000 px".to_owned())
        );
        assert_eq!(
            aspect("65535:1").layout(6000, 4000),
            Err("'aspect' = \"65535:1\" on this 6000×4000 px image needs a 262140000×4000 px canvas, which exceeds the maximum edge limit of 50000 px".to_owned())
        );
        assert_eq!(
            aspect("1:1").layout(1, 11586),
            Err("'aspect' = \"1:1\" on this 1×11586 px image needs a 11586×11586 px canvas = 134235396 px, which exceeds the maximum area of 134217728 px".to_owned())
        );
        assert!(aspect("1:1").layout(1, 11585).is_ok());
        assert!(aspect("50000:1").layout(1, 1).is_ok());
        let over = aspect("50001:1")
            .layout(1, 1)
            .expect_err("past the edge cap");
        assert!(
            over.contains("exceeds the maximum edge limit of 50000 px"),
            "{over}"
        );
    }

    #[test]
    fn params_round_trip_as_written() {
        let p = pad("border = 1\ncolor = \"#FFFFFF\"");
        assert_eq!(p.params().get_f64("border"), Some(1.0));
        assert!(matches!(
            p.params()
                .iter()
                .find(|(k, _)| *k == "border")
                .map(|(_, v)| v),
            Some(toml::Value::Integer(1))
        ));
        assert_eq!(p.params().get_str("color"), Some("#FFFFFF"));

        let p = pad("border = 0.05\ncolor = \"#000000\"");
        assert!(matches!(
            p.params().iter().find(|(k, _)| *k == "border").map(|(_, v)| v),
            Some(toml::Value::Float(f)) if (*f - 0.05).abs() < f64::EPSILON
        ));

        let p = pad("aspect = \"4:5\"\ncolor = \"#000000\"");
        assert!(
            !p.params().contains("gravity"),
            "an absent gravity stays absent"
        );
        assert_eq!(p.name(), "pad");
    }

    #[test]
    fn refusal_hook_is_not_used() {
        let steps = [aspect("1:1"), aspect("16:9"), aspect("9:16"), border("1")];
        for (w, h) in [(1, 1), (1, 7), (7, 1)] {
            let img = wrap(rgb8(w, h));
            for step in &steps {
                assert!(step.refusal(&img).is_none(), "{w}x{h} {:?}", step.params());
                step.apply(img.clone())
                    .unwrap_or_else(|e| panic!("{w}x{h}: {e}"));
            }
        }
    }
}
