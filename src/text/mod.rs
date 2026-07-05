//! Text rendering for `watermark --text` (SPEC-030, DEC-032; rasterizer swapped
//! to `skrifa`+`zeno` by SPEC-044, DEC-045).
//!
//! Pure, file-free glyph rasterization: a string + font bytes + size + color
//! become a tightly-cropped, transparent [`RgbaImage`] overlay. That overlay is
//! then composited onto the base through the **existing SPEC-029 `Watermark`
//! op** (`Gravity::placement` + source-over `apply`) — so gravity/opacity/margin
//! behave identically to the image overlay (DEC-032).
//!
//! Layout (advance/ascent) is hand-rolled; `skrifa` provides glyph outlines +
//! metrics and `zeno` rasterizes each outline into an anti-aliased coverage
//! mask (DEC-045). No pairwise kerning is applied (DEC-045: nil effect on the
//! bundled font). There is **NO file IO** here: the font is an input (`&[u8]`);
//! the bundled default is compile-time [`include_bytes!`] data (DEC-031). The
//! `--font PATH` read happens at the CLI IO boundary (`run_watermark`), never
//! in this module.

use image::RgbaImage;
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{instance::LocationRef, instance::Size, FontRef, GlyphId, MetadataProvider};
use zeno::{Command, Mask, Point};

/// The bundled default font: Go Regular (Bigelow & Holmes, BSD-3-Clause), embedded
/// at compile time so `watermark --text` works out of the box (DEC-032). This is a
/// `&'static [u8]` (compile-time data), NOT file IO.
pub const DEFAULT_FONT: &[u8] = include_bytes!("../../assets/fonts/Go-Regular.ttf");

/// A text-rendering / color-parsing error (SPEC-030). The CLI maps every variant
/// to exit 2 (usage); only a `--font` *file* read failure (at the IO boundary) is
/// exit 3 — that lives in `run_watermark`, not here.
#[derive(Debug, thiserror::Error)]
pub enum TextError {
    /// The font bytes could not be parsed as a TTF/OTF.
    #[error("could not parse font: {0}")]
    Font(String),

    /// A `--color` value was not a recognized hex form.
    #[error("invalid color '{0}': expected RRGGBB, #RRGGBB, or RRGGBBAA")]
    Color(String),

    /// The text to render was empty.
    #[error("text to render must not be empty")]
    Empty,
}

/// Parse a hex color string into RGBA bytes.
///
/// Accepts `RRGGBB`, `#RRGGBB` (alpha defaults to `255`), and `RRGGBBAA`
/// (`#RRGGBBAA` too). Each pair is a base-16 byte. Any other length, a non-hex
/// digit, or an empty string → [`TextError::Color`].
pub fn parse_color(s: &str) -> Result<[u8; 4], TextError> {
    let hex = s.strip_prefix('#').unwrap_or(s);
    let bad = || TextError::Color(s.to_owned());

    let bytes = match hex.len() {
        6 => {
            let mut out = [0u8; 4];
            out[3] = 255;
            for i in 0..3 {
                out[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).map_err(|_| bad())?;
            }
            out
        }
        8 => {
            let mut out = [0u8; 4];
            for i in 0..4 {
                out[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).map_err(|_| bad())?;
            }
            out
        }
        _ => return Err(bad()),
    };
    Ok(bytes)
}

/// Collects a glyph outline's path commands as `zeno::Command`s, negating `y`:
/// `skrifa` emits glyph outlines in y-up font space, but the raster canvas (and
/// `zeno`'s mask origin) is y-down.
#[derive(Default)]
struct ZenoPen(Vec<Command>);

impl OutlinePen for ZenoPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push(Command::MoveTo(Point::new(x, -y)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.push(Command::LineTo(Point::new(x, -y)));
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.0
            .push(Command::QuadTo(Point::new(cx, -cy), Point::new(x, -y)));
    }

    fn curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
        self.0.push(Command::CurveTo(
            Point::new(a, -b),
            Point::new(c, -d),
            Point::new(x, -y),
        ));
    }

    fn close(&mut self) {
        self.0.push(Command::Close);
    }
}

/// A single laid-out glyph: its rasterized coverage mask, the mask's tight
/// placement (relative to the glyph's own origin), and the glyph's absolute
/// pixel origin on the (not-yet-sized) canvas.
struct LaidOutGlyph {
    coverage: Vec<u8>,
    width: u32,
    height: u32,
    /// Absolute pixel origin of the mask's top-left corner.
    origin_x: f32,
    origin_y: f32,
}

/// Rasterize `text` with `font_bytes` at `size_px` in `color`, returning a tight,
/// transparent [`RgbaImage`] overlay (SPEC-030, DEC-032, DEC-045).
///
/// Layout is single-line: glyphs advance left-to-right with horizontal advance
/// (no kerning, DEC-045); the baseline sits at the font's ascent. Each glyph's
/// coverage is rasterized and written as `color` with
/// `alpha = round(coverage * color[3])`, source-over within the glyph's
/// bounding box. The canvas is sized to the union of all glyph pixel bounds (no
/// surrounding padding).
///
/// Errors: empty `text` → [`TextError::Empty`]; unparseable `font_bytes` →
/// [`TextError::Font`]. Pure — performs no file IO (font bytes are an input).
pub fn render_text(
    font_bytes: &[u8],
    text: &str,
    size_px: f32,
    color: [u8; 4],
) -> Result<RgbaImage, TextError> {
    if text.is_empty() {
        return Err(TextError::Empty);
    }

    let font = FontRef::new(font_bytes).map_err(|e| TextError::Font(e.to_string()))?;
    let size = Size::new(size_px);
    let loc = LocationRef::default();

    let ascent = font.metrics(size, loc).ascent;
    let charmap = font.charmap();
    let gmetrics = font.glyph_metrics(size, loc);
    let outlines = font.outline_glyphs();

    // ── First pass: lay out glyphs and rasterize their coverage masks ────────
    // Advance along the baseline (y = ascent); no kerning (DEC-045).
    let mut laid_out: Vec<LaidOutGlyph> = Vec::new();
    let mut pen_x = 0.0_f32;
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for ch in text.chars() {
        let gid = charmap.map(ch).unwrap_or(GlyphId::new(0));

        if let Some(outline) = outlines.get(gid) {
            let mut pen = ZenoPen::default();
            if outline
                .draw(DrawSettings::unhinted(size, loc), &mut pen)
                .is_ok()
            {
                let (coverage, placement) = Mask::new(pen.0.as_slice()).render();
                if placement.width > 0 && placement.height > 0 {
                    let origin_x = pen_x + placement.left as f32;
                    let origin_y = ascent + placement.top as f32;
                    let w = placement.width as f32;
                    let h = placement.height as f32;
                    min_x = min_x.min(origin_x);
                    min_y = min_y.min(origin_y);
                    max_x = max_x.max(origin_x + w);
                    max_y = max_y.max(origin_y + h);
                    laid_out.push(LaidOutGlyph {
                        coverage,
                        width: placement.width,
                        height: placement.height,
                        origin_x,
                        origin_y,
                    });
                }
            }
        }

        pen_x += gmetrics.advance_width(gid).unwrap_or(0.0);
    }

    // No drawable glyphs (e.g. all-whitespace): emit a 1x1 transparent canvas so
    // callers still get a valid, compositable overlay.
    if !min_x.is_finite() {
        return Ok(RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 0])));
    }

    let w = (max_x - min_x).ceil().max(1.0) as u32;
    let h = (max_y - min_y).ceil().max(1.0) as u32;
    let mut canvas = RgbaImage::from_pixel(w, h, image::Rgba([0, 0, 0, 0]));

    // ── Second pass: composite coverage into the canvas ───────────────────────
    let base_alpha = color[3] as f32;
    for glyph in &laid_out {
        let off_x = glyph.origin_x - min_x;
        let off_y = glyph.origin_y - min_y;
        for gy in 0..glyph.height {
            for gx in 0..glyph.width {
                let cov = glyph.coverage[(gy * glyph.width + gx) as usize];
                if cov == 0 {
                    continue;
                }
                let a = ((cov as f32 / 255.0) * base_alpha).round();
                if a <= 0.0 {
                    continue;
                }
                let px = off_x + gx as f32;
                let py = off_y + gy as f32;
                if px < 0.0 || py < 0.0 {
                    continue;
                }
                let (px, py) = (px as u32, py as u32);
                if px >= w || py >= h {
                    continue;
                }
                let a = a.min(255.0) as u8;
                // Source-over within the glyph box: keep the darker/more-opaque
                // alpha where glyphs overlap (single-line text rarely overlaps,
                // but be safe).
                let cur = canvas.get_pixel(px, py);
                if a >= cur.0[3] {
                    canvas.put_pixel(px, py, image::Rgba([color[0], color[1], color[2], a]));
                }
            }
        }
    }

    Ok(canvas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_text_produces_nonempty_coverage() {
        let img = render_text(DEFAULT_FONT, "Hi", 32.0, [255, 255, 255, 255])
            .expect("rendering should succeed");
        assert!(img.width() > 0 && img.height() > 0, "image is non-empty");
        let opaque = img.pixels().filter(|p| p.0[3] > 0).count();
        assert!(opaque >= 1, "at least one (partly) opaque pixel");
    }

    #[test]
    fn render_text_applies_color() {
        let img = render_text(DEFAULT_FONT, "Hi", 48.0, [255, 0, 0, 255])
            .expect("rendering should succeed");
        let reddish = img
            .pixels()
            .any(|p| p.0[3] > 0 && p.0[0] > 200 && p.0[1] < 60 && p.0[2] < 60);
        assert!(reddish, "some rendered pixel should be ~red");
    }

    #[test]
    fn render_text_size_scales() {
        let small = render_text(DEFAULT_FONT, "Hg", 16.0, [255, 255, 255, 255]).unwrap();
        let large = render_text(DEFAULT_FONT, "Hg", 64.0, [255, 255, 255, 255]).unwrap();
        assert!(
            large.height() > small.height(),
            "size 64 should be taller than size 16 ({} vs {})",
            large.height(),
            small.height()
        );
    }

    #[test]
    fn render_text_empty_is_error() {
        let err = render_text(DEFAULT_FONT, "", 32.0, [255, 255, 255, 255]).unwrap_err();
        assert!(matches!(err, TextError::Empty));
    }

    #[test]
    fn render_text_bad_font_is_error() {
        let junk = [0u8, 1, 2, 3, 4, 5, 6, 7];
        let err = render_text(&junk, "Hi", 32.0, [255, 255, 255, 255]).unwrap_err();
        assert!(matches!(err, TextError::Font(_)));
    }

    #[test]
    fn parse_color_hex_variants() {
        assert_eq!(parse_color("ffffff").unwrap(), [255, 255, 255, 255]);
        assert_eq!(parse_color("#000000").unwrap(), [0, 0, 0, 255]);
        assert_eq!(parse_color("ff0000").unwrap(), [255, 0, 0, 255]);
        assert_eq!(parse_color("ff000080").unwrap(), [255, 0, 0, 128]);
        assert!(matches!(parse_color("zzz"), Err(TextError::Color(_))));
        assert!(matches!(parse_color("fff"), Err(TextError::Color(_))));
    }

    // ── SPEC-044 tests (skrifa+zeno invariants) ───────────────────────────────

    #[test]
    fn render_text_accumulates_advance() {
        let one = render_text(DEFAULT_FONT, "W", 32.0, [255, 255, 255, 255]).unwrap();
        let three = render_text(DEFAULT_FONT, "WWW", 32.0, [255, 255, 255, 255]).unwrap();
        assert!(
            three.width() > one.width(),
            "WWW should be wider than W ({} vs {})",
            three.width(),
            one.width()
        );
    }

    #[test]
    fn render_text_whitespace_contributes_advance() {
        let with_space = render_text(DEFAULT_FONT, "A B", 32.0, [255, 255, 255, 255]).unwrap();
        let without_space = render_text(DEFAULT_FONT, "AB", 32.0, [255, 255, 255, 255]).unwrap();
        assert!(
            with_space.width() > without_space.width(),
            "'A B' should be wider than 'AB' ({} vs {})",
            with_space.width(),
            without_space.width()
        );
    }

    #[test]
    fn render_text_all_whitespace_is_1x1() {
        let img = render_text(DEFAULT_FONT, "   ", 32.0, [255, 255, 255, 255]).unwrap();
        assert_eq!((img.width(), img.height()), (1, 1));
        let p = img.get_pixel(0, 0);
        assert_eq!(p.0, [0, 0, 0, 0]);
    }

    #[test]
    fn render_text_height_tracks_font_size() {
        let small = render_text(DEFAULT_FONT, "Hg", 16.0, [255, 255, 255, 255]).unwrap();
        let large = render_text(DEFAULT_FONT, "Hg", 64.0, [255, 255, 255, 255]).unwrap();
        assert!(
            large.height() > 32 && large.height() <= 96,
            "size 64 height should be in a sane window: {}",
            large.height()
        );
        assert!(
            large.height() > small.height(),
            "size 64 should be taller than size 16 ({} vs {})",
            large.height(),
            small.height()
        );
    }
}
