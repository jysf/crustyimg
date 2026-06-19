//! Text rendering for `watermark --text` (SPEC-030, DEC-032).
//!
//! Pure, file-free glyph rasterization: a string + font bytes + size + color
//! become a tightly-cropped, transparent [`RgbaImage`] overlay. That overlay is
//! then composited onto the base through the **existing SPEC-029 `Watermark`
//! op** (`Gravity::placement` + source-over `apply`) — so gravity/opacity/margin
//! behave identically to the image overlay (DEC-032).
//!
//! Layout (advance/kern/ascent) is hand-rolled; `ab_glyph` is used only as the
//! glyph rasterizer (DEC-032). There is **NO file IO** here: the font is an
//! input (`&[u8]`); the bundled default is compile-time [`include_bytes!`] data
//! (DEC-031). The `--font PATH` read happens at the CLI IO boundary
//! (`run_watermark`), never in this module.

use ab_glyph::{point, Font, FontRef, PxScale, ScaleFont};
use image::RgbaImage;

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

/// Rasterize `text` with `font_bytes` at `size_px` in `color`, returning a tight,
/// transparent [`RgbaImage`] overlay (SPEC-030, DEC-032).
///
/// Layout is single-line: glyphs advance left-to-right with horizontal advance +
/// pairwise kerning; the baseline sits at the font's ascent. Each glyph's coverage
/// is rasterized and written as `color` with `alpha = round(coverage * color[3])`,
/// source-over within the glyph's bounding box. The canvas is sized to the union
/// of all glyph pixel bounds (no surrounding padding).
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

    let font = FontRef::try_from_slice(font_bytes).map_err(|e| TextError::Font(e.to_string()))?;
    let scale = PxScale::from(size_px);
    let sf = font.as_scaled(scale);

    // ── First pass: lay out glyphs and collect their pixel bounds ─────────────
    // Advance along the baseline (y = ascent); accumulate kerning between glyphs.
    let mut outlined = Vec::new();
    let mut x = 0.0_f32;
    let mut prev: Option<ab_glyph::GlyphId> = None;
    let ascent = sf.ascent();

    for ch in text.chars() {
        let gid = font.glyph_id(ch);
        if let Some(p) = prev {
            x += sf.kern(p, gid);
        }
        let glyph = gid.with_scale_and_position(scale, point(x, ascent));
        if let Some(o) = font.outline_glyph(glyph) {
            outlined.push(o);
        }
        x += sf.h_advance(gid);
        prev = Some(gid);
    }

    // Union of every glyph's pixel bounds → the tight canvas extent. Whitespace
    // glyphs (no outline) still contribute advance but no bounds.
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for o in &outlined {
        let b = o.px_bounds();
        min_x = min_x.min(b.min.x);
        min_y = min_y.min(b.min.y);
        max_x = max_x.max(b.max.x);
        max_y = max_y.max(b.max.y);
    }

    // No drawable glyphs (e.g. all-whitespace): emit a 1x1 transparent canvas so
    // callers still get a valid, compositable overlay.
    if !min_x.is_finite() {
        return Ok(RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 0])));
    }

    let w = (max_x - min_x).ceil().max(1.0) as u32;
    let h = (max_y - min_y).ceil().max(1.0) as u32;
    let mut canvas = RgbaImage::from_pixel(w, h, image::Rgba([0, 0, 0, 0]));

    // ── Second pass: rasterize coverage into the canvas ───────────────────────
    let base_alpha = color[3] as f32;
    for o in &outlined {
        let b = o.px_bounds();
        let off_x = b.min.x - min_x;
        let off_y = b.min.y - min_y;
        o.draw(|gx, gy, coverage| {
            let a = (coverage * base_alpha).round();
            if a <= 0.0 {
                return;
            }
            let px = off_x + gx as f32;
            let py = off_y + gy as f32;
            if px < 0.0 || py < 0.0 {
                return;
            }
            let (px, py) = (px as u32, py as u32);
            if px >= w || py >= h {
                return;
            }
            let a = a.min(255.0) as u8;
            // Source-over within the glyph box: keep the darker/more-opaque alpha
            // where glyphs overlap (single-line text rarely overlaps, but be safe).
            let cur = canvas.get_pixel(px, py);
            if a >= cur.0[3] {
                canvas.put_pixel(px, py, image::Rgba([color[0], color[1], color[2], a]));
            }
        });
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
}
