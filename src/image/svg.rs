//! Pure-Rust SVG rasterize (SPEC-060, DEC-054).
//!
//! The `image` crate has no SVG support — SVG is XML/text, not a magic-byte
//! raster format — so `.svg` cannot flow through the generic `ImageReader`
//! path. This module rasterizes `.svg` on the **default** build with zero
//! system/build-tool deps by pairing three permissive crates that feed the
//! canonical [`crate::image::Image`] (the AVIF/webp-lossy precedent — NOT a
//! second pixel library, `single-image-library`):
//!
//! - `usvg` (Apache-2.0 OR MIT) parses the SVG XML into a resolved render tree.
//! - `tiny-skia` (BSD-3-Clause) rasterizes that tree to a premultiplied RGBA8
//!   pixmap.
//! - `resvg` (Apache-2.0 OR MIT) drives the render.
//!
//! The glue turns the premultiplied pixmap into a straight-alpha RGBA8
//! [`DynamicImage`] (the AVIF `unpremultiply` analog). There is no
//! `ImageFormat::Svg`, so the caller records a rasterized SVG's `source_format`
//! as [`::image::ImageFormat::Png`] (a lossless-RGBA target).
//!
//! ## Security (untrusted-input-hardening)
//!
//! SVG is a hostile, untrusted input surface (external file/URL refs →
//! local-file read / SSRF, decompression-bomb viewBoxes). Two hardenings apply:
//!
//! - **External references are refused.** `usvg::Options::resources_dir` is
//!   `None` (no filesystem resolution of relative paths) and the
//!   `image_href_resolver`'s string resolver returns `None` for every
//!   `href`/`xlink:href`, so `<image href="file:///etc/passwd">` /
//!   `href="http://…"` resolve to nothing (transparent) — no local-file read,
//!   no network request. `data:` URIs are still honored.
//! - **Output dimensions are capped before allocation** (DEC-034). A huge
//!   `viewBox` parses fine (usvg allocates no pixels), so the intrinsic size
//!   from [`usvg::Tree::size`] is checked against `limits` **before**
//!   `tiny_skia::Pixmap::new`, rejecting a decompression bomb without ever
//!   attempting the multi-GiB allocation.
//!
//! Every recoverable failure (malformed/truncated XML, zero size, buffer
//! mismatch) is a typed [`ImageError`] — no `unwrap`/`expect`/`panic!`. A
//! `cargo-fuzz` target (`fuzz/fuzz_targets/svg_decode.rs`) exercises the parse
//! and rasterize path together.
//!
//! ## Text (fonts)
//!
//! SVG `<text>` renders with the bundled BSD-3 Go font only (DEC-045), loaded
//! into an explicit `fontdb`. System fonts are **not** enumerated
//! (`load_system_fonts` is never called), keeping the render deterministic and
//! free of filesystem/font-enumeration surface. Rendering text to *nothing*
//! would be a silent-wrong-output footgun, so text is on with a guaranteed
//! font. This is what re-adds `ttf-parser` (the RUSTSEC-2026-0192 advisory
//! ignore in `deny.toml`) via usvg's text stack.

use std::sync::Arc;

use ::image::{DynamicImage, Limits, RgbaImage};
use resvg::{tiny_skia, usvg};

use crate::error::{ImageError, Result};

/// The bundled default font for SVG `<text>` (BSD-3 Go, DEC-045). Embedded so
/// the render needs no filesystem/system-font access.
const FONT_DATA: &[u8] = include_bytes!("../../assets/fonts/Go-Regular.ttf");

/// The font family name registered for [`FONT_DATA`] and used as the default.
const FONT_FAMILY: &str = "Go";

/// Bound the content-sniff scan so we never walk an unbounded buffer looking
/// for an `<svg` root.
const SNIFF_WINDOW: usize = 1024;

/// Whether `bytes` looks like an SVG document.
///
/// Detection is a bounded content sniff (not the `image` guesser, which has no
/// SVG support): skip a leading BOM + ASCII whitespace, then accept if the
/// stream's root is `<svg` (case-insensitive) OR it opens with an XML prolog
/// (`<?xml …`) that reaches an `<svg` root within the first [`SNIFF_WINDOW`]
/// bytes. Anything else (PNG/JPEG/AVIF magic bytes, `<html>`, arbitrary XML)
/// is rejected. Allocation-free.
pub(crate) fn is_svg(bytes: &[u8]) -> bool {
    let start = skip_bom_and_ws(bytes);
    let window = &bytes[start..bytes.len().min(start + SNIFF_WINDOW)];
    if is_svg_root_at(window, 0) {
        return true;
    }
    // XML prolog: only then scan for an `<svg` root inside the window. This
    // keeps `<html>` (and non-SVG XML with no `<svg` element) out.
    if starts_with_ci(window, b"<?xml") {
        return (0..window.len()).any(|i| is_svg_root_at(window, i));
    }
    false
}

/// Number of leading bytes to skip: a UTF-8/UTF-16 BOM, then ASCII whitespace.
fn skip_bom_and_ws(bytes: &[u8]) -> usize {
    let mut i = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        3
    } else if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        2
    } else {
        0
    };
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// Whether an `<svg` root tag begins at `w[i]` (case-insensitive, with a valid
/// tag-boundary byte — whitespace, `>`, `/`, or end-of-window — after it, so
/// `<svgfoo` does not match).
fn is_svg_root_at(w: &[u8], i: usize) -> bool {
    if i + 4 > w.len()
        || w[i] != b'<'
        || !w[i + 1].eq_ignore_ascii_case(&b's')
        || !w[i + 2].eq_ignore_ascii_case(&b'v')
        || !w[i + 3].eq_ignore_ascii_case(&b'g')
    {
        return false;
    }
    match w.get(i + 4) {
        None => true,
        Some(c) => c.is_ascii_whitespace() || *c == b'>' || *c == b'/',
    }
}

/// Case-insensitive ASCII prefix test (allocation-free).
fn starts_with_ci(hay: &[u8], needle: &[u8]) -> bool {
    hay.len() >= needle.len()
        && hay[..needle.len()]
            .iter()
            .zip(needle)
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

/// Rasterize an SVG byte stream to a straight-alpha 8-bit RGBA [`DynamicImage`]
/// at intrinsic (1x) size, enforcing the decode caps in `limits` (DEC-034)
/// before allocating pixels and refusing all external resource references.
pub(crate) fn decode_svg(bytes: &[u8], limits: &Limits) -> Result<DynamicImage> {
    let opt = hardened_options();

    // Parse to a resolved render tree. Malformed/truncated/empty input is a
    // typed error, never a panic.
    let tree =
        usvg::Tree::from_data(bytes, &opt).map_err(|e| ImageError::Decode(format!("svg: {e}")))?;

    // Intrinsic size (width/height, else viewBox). Cap it BEFORE allocating —
    // a 100000×100000 viewBox parses fine but would otherwise attempt a
    // multi-GiB pixmap.
    let size = tree.size();
    let w = size.width().ceil() as u32;
    let h = size.height().ceil() as u32;
    check_caps(w, h, limits)?;

    let mut pixmap = tiny_skia::Pixmap::new(w, h)
        .ok_or_else(|| ImageError::Decode("svg: zero/invalid raster size".into()))?;
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );

    // tiny-skia stores PREMULTIPLIED RGBA8; demultiply to straight alpha (the
    // AVIF `unpremultiply` analog) so the canonical `Image` holds straight RGBA.
    let mut straight = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for p in pixmap.pixels() {
        let c = p.demultiply();
        straight.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }

    let buf = RgbaImage::from_raw(w, h, straight)
        .ok_or_else(|| ImageError::Decode("svg: raster buffer size mismatch".into()))?;
    Ok(DynamicImage::ImageRgba8(buf))
}

/// Build the hardened [`usvg::Options`]: no filesystem resource resolution, all
/// external `href`s refused (data: URIs kept), and only the bundled Go font
/// loaded (no system fonts).
fn hardened_options() -> usvg::Options<'static> {
    let mut opt = usvg::Options {
        // No filesystem resolution of relative paths.
        resources_dir: None,
        ..usvg::Options::default()
    };
    // Refuse every file/URL href; keep inline `data:` URIs.
    opt.image_href_resolver = usvg::ImageHrefResolver {
        resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
        resolve_string: Box::new(|_href, _opts| None),
    };

    // Load ONLY the bundled font; never enumerate system fonts.
    let mut db = usvg::fontdb::Database::new();
    db.load_font_data(FONT_DATA.to_vec());
    opt.font_family = FONT_FAMILY.to_string();
    opt.fontdb = Arc::new(db);
    opt
}

/// Reject dimensions that exceed the `limits` (dimension or total allocation) or
/// the shared peak-memory pixel budget (DEC-063).
///
/// The allocation estimate uses the 8-bit RGBA raster buffer (`w * h * 4`), the
/// largest buffer this module allocates. The pixel budget is the tighter, uniform
/// bound across all four decode paths. Mirrors `avif::check_caps`.
fn check_caps(w: u32, h: u32, limits: &Limits) -> Result<()> {
    super::check_pixel_budget(w, h)?;
    if let Some(max_w) = limits.max_image_width {
        if w > max_w {
            return Err(ImageError::LimitsExceeded(format!(
                "svg width {w} exceeds cap {max_w}"
            )));
        }
    }
    if let Some(max_h) = limits.max_image_height {
        if h > max_h {
            return Err(ImageError::LimitsExceeded(format!(
                "svg height {h} exceeds cap {max_h}"
            )));
        }
    }
    if let Some(max_alloc) = limits.max_alloc {
        let bytes = (w as u64) * (h as u64) * 4;
        if bytes > max_alloc {
            return Err(ImageError::LimitsExceeded(format!(
                "svg buffer {bytes} bytes exceeds alloc cap {max_alloc}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_svg_detects_svg_and_xml_prolog() {
        assert!(is_svg(b"<svg xmlns='http://www.w3.org/2000/svg'></svg>"));
        assert!(is_svg(b"<SVG></SVG>")); // case-insensitive
        assert!(is_svg(b"   \n  <svg>")); // leading whitespace
        assert!(is_svg(b"<?xml version='1.0'?><svg></svg>")); // XML prolog
        assert!(is_svg(b"<?xml version='1.0'?>\n<!-- c -->\n<svg/>")); // prolog + comment

        // Non-SVG inputs are rejected.
        assert!(!is_svg(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a])); // PNG
        let mut ftyp = Vec::new();
        ftyp.extend_from_slice(&0x18u32.to_be_bytes());
        ftyp.extend_from_slice(b"ftypavif");
        assert!(!is_svg(&ftyp)); // AVIF ftyp
        assert!(!is_svg(b"<html><body></body></html>")); // HTML
        assert!(!is_svg(b"<?xml version='1.0'?><rss></rss>")); // non-SVG XML
        assert!(!is_svg(b"<svgnot>")); // not a real svg root tag
    }

    #[test]
    fn svg_dimension_cap_rejects_before_raster() {
        // A 100000×100000 SVG parses fine but must be rejected by the cap, never
        // allocating a ~38 GiB pixmap.
        let svg = b"<svg xmlns='http://www.w3.org/2000/svg' width='100000' height='100000'></svg>";
        let mut limits = Limits::default();
        limits.max_image_width = Some(1000);
        limits.max_image_height = Some(1000);
        let result = decode_svg(svg, &limits);
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    #[test]
    fn corrupt_svg_bytes_are_decode_error_not_panic() {
        // Truncated / non-SVG-but-SVG-sniffed input is a typed decode error.
        let truncated = decode_svg(
            b"<svg xmlns='http://www.w3.org/2000/svg'><rect",
            &Limits::default(),
        );
        assert!(
            matches!(truncated, Err(ImageError::Decode(_))),
            "expected Decode, got {truncated:?}"
        );
        let garbage = decode_svg(b"not an svg", &Limits::default());
        assert!(
            matches!(garbage, Err(ImageError::Decode(_))),
            "expected Decode, got {garbage:?}"
        );
    }

    #[test]
    fn check_caps_rejects_oversize() {
        let mut limits = Limits::default();
        limits.max_image_width = Some(10);
        assert!(matches!(
            check_caps(16, 16, &limits),
            Err(ImageError::LimitsExceeded(_))
        ));

        let mut alloc = Limits::default();
        alloc.max_alloc = Some(16);
        assert!(matches!(
            check_caps(16, 16, &alloc),
            Err(ImageError::LimitsExceeded(_))
        ));

        assert!(check_caps(16, 16, &Limits::default()).is_ok());
    }

    /// SPEC-070: a render size that passes EVERY DEC-034 cap (each side < 65 535,
    /// the RGBA pixmap under the 512 MiB alloc cap) is still rejected when it
    /// exceeds the DEC-063 pixel budget — an SVG `viewBox` is free to declare it.
    #[test]
    fn check_caps_rejects_over_pixel_budget() {
        let mut prod = Limits::default();
        prod.max_image_width = Some(65_535);
        prod.max_image_height = Some(65_535);
        prod.max_alloc = Some(512 * 1024 * 1024);

        // 10000×10000 = 100 Mpix (400 MB RGBA — under the alloc cap).
        assert!(matches!(
            check_caps(10_000, 10_000, &prod),
            Err(ImageError::LimitsExceeded(_))
        ));
        assert!(check_caps(6_000, 4_000, &prod).is_ok());
    }
}
