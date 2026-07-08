//! Pure-Rust AVIF decode (SPEC-058, DEC-053).
//!
//! `image` 0.25's own AVIF decoder is dav1d (a C system library), which would
//! break the pure-Rust default (DEC-004). This module decodes `.avif` on the
//! **default** path with zero system/build-tool deps by pairing two permissive
//! CODEC crates that feed the canonical [`crate::image::Image`] (the webp-lossy
//! precedent — NOT a second pixel library):
//!
//! - [`avif_parse`] (MPL-2.0) parses the ISOBMFF/MIAF container into the
//!   primary-item + optional alpha AV1 OBU streams, and rejects grid/tiled
//!   collages cleanly.
//! - `re_rav1d` (BSD-2-Clause), built no-asm, decodes those OBUs to YUV planes
//!   via its re-exported safe `dav1d` API.
//!
//! The glue here turns the YUV planes into 8-bit RGB(A), honoring bit depth
//! (8/10/12 → stored as u8/u16), chroma subsampling (4:2:0 / 4:2:2 / 4:4:4),
//! YUV range (full/limited), matrix coefficients (BT.601/709/2020 + GBR
//! identity), and premultiplied alpha. The `re_rav1d` surface is kept THIN (it
//! is a fork we pin) so we can migrate to `image`'s built-in pure-Rust decode
//! once image-rs #2621 lands (DEC-053).
//!
//! ## Security (untrusted-input-hardening)
//!
//! AVIF is untrusted binary input. Dimensions are capped from the container
//! metadata **before** any pixel allocation (DEC-034), so a decompression-bomb
//! header is rejected without decoding. Every recoverable failure (malformed
//! container, unsupported feature, decode error, plane-geometry mismatch) is a
//! typed [`ImageError`] — no `unwrap`/`expect`/`panic!` on these paths. A
//! `cargo-fuzz` target (`fuzz/fuzz_targets/avif_decode.rs`) exercises the
//! container parse and the decode/convert path together.

use ::image::{DynamicImage, Limits, RgbImage, RgbaImage};
use re_rav1d::dav1d::pixel::{MatrixCoefficients as Mc, YUVRange};
use re_rav1d::dav1d::{Decoder, Picture, PixelLayout, PlanarImageComponent};
use std::io::Cursor;

use crate::error::{ImageError, Result};

/// Whether `bytes` is an ISOBMFF file whose `ftyp` box advertises an AVIF brand.
///
/// Detection is by container brand (not the `image` guesser) so dispatch does
/// not depend on `image`'s optional avif feature. Scans the major brand and the
/// compatible-brands list of the leading `ftyp` box for `avif`/`avis`.
pub(crate) fn is_avif(bytes: &[u8]) -> bool {
    // ftyp box: [size:u32][b"ftyp"][major:4][minor:4][compatible brands: 4*n].
    if bytes.len() < 12 || &bytes[4..8] != b"ftyp" {
        return false;
    }
    let box_size = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    // Clamp the brand scan to the declared box size and the actual buffer.
    let end = box_size.clamp(8, bytes.len());
    // Major brand at 8..12, then compatible brands every 4 bytes from 16.
    if is_avif_brand(&bytes[8..12]) {
        return true;
    }
    let mut i = 16;
    while i + 4 <= end {
        if is_avif_brand(&bytes[i..i + 4]) {
            return true;
        }
        i += 4;
    }
    false
}

fn is_avif_brand(brand: &[u8]) -> bool {
    matches!(brand, b"avif" | b"avis")
}

/// Decode an AVIF byte stream to an 8-bit RGB(A) [`DynamicImage`], enforcing the
/// decode caps in `limits` (DEC-034) before allocating pixels.
pub(crate) fn decode_avif(bytes: &[u8], limits: &Limits) -> Result<DynamicImage> {
    let parsed = avif_parse::read_avif(&mut Cursor::new(bytes)).map_err(map_parse_err)?;

    // Cap dimensions/allocation from the container metadata BEFORE decoding, so
    // an oversized header is rejected without allocating pixel planes.
    let meta = parsed.primary_item_metadata().map_err(map_parse_err)?;
    check_caps(
        meta.max_frame_width.get(),
        meta.max_frame_height.get(),
        limits,
    )?;

    let pic = decode_obus(&parsed.primary_item)?;
    // Defense in depth: the decoded dimensions must also satisfy the caps.
    check_caps(pic.width(), pic.height(), limits)?;

    let w = pic.width();
    let h = pic.height();
    let mut rgba = yuv_to_rgba(&pic)?;

    // Merge the alpha plane (a separate monochrome OBU stream), if present.
    if let Some(alpha) = &parsed.alpha_item {
        let apic = decode_obus(alpha)?;
        apply_alpha(&mut rgba, w, h, &apic)?;
        if parsed.premultiplied_alpha {
            unpremultiply(&mut rgba);
        }
        let buf = RgbaImage::from_raw(w, h, rgba)
            .ok_or_else(|| ImageError::Decode("avif: alpha buffer size mismatch".into()))?;
        Ok(DynamicImage::ImageRgba8(buf))
    } else {
        // No alpha: drop the (opaque) alpha channel to a compact RGB image.
        let mut rgb = Vec::with_capacity((w as usize) * (h as usize) * 3);
        for px in rgba.chunks_exact(4) {
            rgb.extend_from_slice(&px[..3]);
        }
        let buf = RgbImage::from_raw(w, h, rgb)
            .ok_or_else(|| ImageError::Decode("avif: rgb buffer size mismatch".into()))?;
        Ok(DynamicImage::ImageRgb8(buf))
    }
}

/// Reject dimensions that exceed the `limits` (dimension or total allocation).
///
/// The allocation estimate uses the 8-bit RGBA working buffer (`w * h * 4`),
/// the largest intermediate this module allocates.
fn check_caps(w: u32, h: u32, limits: &Limits) -> Result<()> {
    if let Some(max_w) = limits.max_image_width {
        if w > max_w {
            return Err(ImageError::LimitsExceeded(format!(
                "avif width {w} exceeds cap {max_w}"
            )));
        }
    }
    if let Some(max_h) = limits.max_image_height {
        if h > max_h {
            return Err(ImageError::LimitsExceeded(format!(
                "avif height {h} exceeds cap {max_h}"
            )));
        }
    }
    if let Some(max_alloc) = limits.max_alloc {
        let bytes = (w as u64) * (h as u64) * 4;
        if bytes > max_alloc {
            return Err(ImageError::LimitsExceeded(format!(
                "avif buffer {bytes} bytes exceeds alloc cap {max_alloc}"
            )));
        }
    }
    Ok(())
}

/// Decode a single AV1 still image (one OBU stream) to a `re_rav1d` [`Picture`].
fn decode_obus(obus: &[u8]) -> Result<Picture> {
    let mut dec = Decoder::new().map_err(|e| ImageError::Decode(format!("avif: {e}")))?;
    dec.send_data(obus.to_vec(), None, None, None)
        .map_err(|e| ImageError::Decode(format!("avif send_data: {e}")))?;
    // A single still frame is produced after the data is sent; a bounded retry
    // loop drains any decoder delay without looping unboundedly on bad input.
    for _ in 0..8 {
        match dec.get_picture() {
            Ok(p) => return Ok(p),
            Err(e) if e.is_again() => {
                dec.send_pending_data()
                    .map_err(|e| ImageError::Decode(format!("avif drain: {e}")))?;
            }
            Err(e) => return Err(ImageError::Decode(format!("avif get_picture: {e}"))),
        }
    }
    Err(ImageError::Decode("avif: no frame produced".into()))
}

/// Read one YUV sample from a plane, honoring bit depth (u8 vs little-endian
/// u16). Out-of-range reads return 0 rather than panicking (defense in depth).
#[inline]
fn sample(plane: &[u8], stride: u32, x: u32, y: u32, depth: usize) -> u32 {
    if depth <= 8 {
        plane
            .get((y as usize) * (stride as usize) + x as usize)
            .map(|&b| b as u32)
            .unwrap_or(0)
    } else {
        let off = (y as usize) * (stride as usize) + (x as usize) * 2;
        match (plane.get(off), plane.get(off + 1)) {
            (Some(&lo), Some(&hi)) => u16::from_le_bytes([lo, hi]) as u32,
            _ => 0,
        }
    }
}

/// Convert a decoded YUV [`Picture`] to a straight (non-premultiplied) 8-bit
/// RGBA buffer with an opaque alpha channel (alpha is merged by the caller).
fn yuv_to_rgba(pic: &Picture) -> Result<Vec<u8>> {
    let w = pic.width();
    let h = pic.height();
    let depth = pic.bit_depth().max(8);
    let layout = pic.pixel_layout();
    let full = matches!(pic.color_range(), YUVRange::Full);
    let maxval = ((1u32 << depth) - 1) as f32;
    let scale = (1u32 << (depth - 8)) as f32; // limited-range headroom per bit depth
    let mono = layout == PixelLayout::I400;
    let identity = matches!(pic.matrix_coefficients(), Mc::Identity);

    // Luma coefficients (Kr, Kb) per matrix; unspecified defaults to BT.601,
    // matching libavif's behavior for AVIF stills.
    let (kr, kb) = match pic.matrix_coefficients() {
        Mc::BT709 => (0.2126f32, 0.0722f32),
        Mc::BT2020NonConstantLuminance | Mc::BT2020ConstantLuminance => (0.2627, 0.0593),
        Mc::ST240M => (0.212, 0.087),
        _ => (0.299, 0.114), // BT.601 / BT470BG / unspecified
    };
    let kg = 1.0 - kr - kb;

    let y_plane = pic.plane(PlanarImageComponent::Y);
    let y_stride = pic.stride(PlanarImageComponent::Y);
    let (u_plane, v_plane, c_stride, sx, sy) = if mono {
        (None, None, 0u32, 1u32, 1u32)
    } else {
        let (sx, sy) = match layout {
            PixelLayout::I420 => (2u32, 2u32),
            PixelLayout::I422 => (2, 1),
            _ => (1, 1),
        };
        (
            Some(pic.plane(PlanarImageComponent::U)),
            Some(pic.plane(PlanarImageComponent::V)),
            pic.stride(PlanarImageComponent::U),
            sx,
            sy,
        )
    };

    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
    for y in 0..h {
        for x in 0..w {
            let yv = sample(&y_plane, y_stride, x, y, depth) as f32;
            let (r, g, b) = if mono {
                let l = if full {
                    yv / maxval
                } else {
                    (yv - 16.0 * scale) / (219.0 * scale)
                };
                let v = to_u8(l);
                (v, v, v)
            } else {
                // Safe: u/v planes are Some in the non-mono branch.
                let up = u_plane.as_deref().unwrap_or(&[]);
                let vp = v_plane.as_deref().unwrap_or(&[]);
                let uu = sample(up, c_stride, x / sx, y / sy, depth) as f32;
                let vv = sample(vp, c_stride, x / sx, y / sy, depth) as f32;
                if identity {
                    // GBR identity: plane order is G(Y), B(U), R(V) (lossless AVIF).
                    (to_u8(vv / maxval), to_u8(yv / maxval), to_u8(uu / maxval))
                } else {
                    let (yl, cb, cr) = if full {
                        (yv / maxval, uu / maxval - 0.5, vv / maxval - 0.5)
                    } else {
                        (
                            (yv - 16.0 * scale) / (219.0 * scale),
                            (uu - 128.0 * scale) / (224.0 * scale),
                            (vv - 128.0 * scale) / (224.0 * scale),
                        )
                    };
                    let rf = yl + 2.0 * (1.0 - kr) * cr;
                    let bf = yl + 2.0 * (1.0 - kb) * cb;
                    let gf =
                        yl - (kr / kg) * 2.0 * (1.0 - kr) * cr - (kb / kg) * 2.0 * (1.0 - kb) * cb;
                    (to_u8(rf), to_u8(gf), to_u8(bf))
                }
            };
            let idx = ((y as usize) * (w as usize) + x as usize) * 4;
            out[idx] = r;
            out[idx + 1] = g;
            out[idx + 2] = b;
            out[idx + 3] = 255;
        }
    }
    Ok(out)
}

/// Merge a decoded monochrome alpha [`Picture`] into the RGBA buffer's A channel.
fn apply_alpha(rgba: &mut [u8], w: u32, h: u32, apic: &Picture) -> Result<()> {
    let ad = apic.bit_depth().max(8);
    let amax = ((1u32 << ad) - 1) as f32;
    let a_scale = (1u32 << (ad - 8)) as f32;
    let full = matches!(apic.color_range(), YUVRange::Full);
    let a_plane = apic.plane(PlanarImageComponent::Y);
    let a_stride = apic.stride(PlanarImageComponent::Y);
    let aw = apic.width();
    let ah = apic.height();
    if aw < w || ah < h {
        return Err(ImageError::Decode(
            "avif: alpha plane smaller than image".into(),
        ));
    }
    for y in 0..h {
        for x in 0..w {
            let av = sample(&a_plane, a_stride, x, y, ad) as f32;
            let a = if full {
                av / amax
            } else {
                (av - 16.0 * a_scale) / (219.0 * a_scale)
            };
            let idx = ((y as usize) * (w as usize) + x as usize) * 4 + 3;
            rgba[idx] = to_u8(a);
        }
    }
    Ok(())
}

/// Convert premultiplied-alpha RGBA to straight alpha in place (MIAF `prem`).
fn unpremultiply(rgba: &mut [u8]) {
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3];
        if a == 0 {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
        } else if a < 255 {
            let af = a as f32 / 255.0;
            for c in &mut px[..3] {
                *c = (((*c as f32) / af).round()).clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// Clamp a normalized [0,1] float channel to an 8-bit sample.
#[inline]
fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Map an [`avif_parse::Error`] to a typed [`ImageError`]. Grid/tiled and other
/// unsupported-but-valid containers surface as `Decode` with the parser's
/// message (never a panic or garbage pixels).
fn map_parse_err(e: avif_parse::Error) -> ImageError {
    ImageError::Decode(format!("avif container: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_avif_detects_ftyp_avif_major_brand() {
        // size(0) + "ftyp" + "avif" + minor + compat
        let mut b = Vec::new();
        b.extend_from_slice(&0x20u32.to_be_bytes());
        b.extend_from_slice(b"ftypavif");
        b.extend_from_slice(&0u32.to_be_bytes());
        b.extend_from_slice(b"avifmif1miafMA1B");
        assert!(is_avif(&b));
    }

    #[test]
    fn is_avif_detects_compatible_brand() {
        let mut b = Vec::new();
        b.extend_from_slice(&0x1cu32.to_be_bytes());
        b.extend_from_slice(b"ftypmif1"); // major = mif1
        b.extend_from_slice(&0u32.to_be_bytes());
        b.extend_from_slice(b"mif1avif"); // compatible brands include avif
        assert!(is_avif(&b));
    }

    #[test]
    fn is_avif_rejects_png_and_short() {
        assert!(!is_avif(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]));
        assert!(!is_avif(b"ftyp")); // too short
        let mut heic = Vec::new();
        heic.extend_from_slice(&0x18u32.to_be_bytes());
        heic.extend_from_slice(b"ftypheic");
        heic.extend_from_slice(&0u32.to_be_bytes());
        heic.extend_from_slice(b"heicmif1");
        assert!(!is_avif(&heic));
    }

    #[test]
    fn to_u8_clamps() {
        assert_eq!(to_u8(-0.5), 0);
        assert_eq!(to_u8(0.0), 0);
        assert_eq!(to_u8(1.0), 255);
        assert_eq!(to_u8(2.0), 255);
        assert_eq!(to_u8(0.5), 128);
    }

    #[test]
    fn unpremultiply_divides_by_alpha() {
        // premultiplied (100,100,100) at a=128 → straight ~ (199,199,199).
        let mut px = vec![100u8, 100, 100, 128];
        unpremultiply(&mut px);
        assert!((px[0] as i32 - 199).abs() <= 1, "got {}", px[0]);
        assert_eq!(px[3], 128);

        // a=0 → transparent, color zeroed.
        let mut z = vec![50u8, 60, 70, 0];
        unpremultiply(&mut z);
        assert_eq!(&z[..3], &[0, 0, 0]);
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

        // Generous limits: OK.
        assert!(check_caps(16, 16, &Limits::default()).is_ok());
    }

    #[test]
    fn corrupt_bytes_are_decode_error_not_panic() {
        let junk = [0u8; 40];
        let err = decode_avif(&junk, &Limits::default());
        assert!(matches!(err, Err(ImageError::Decode(_))), "got {err:?}");
    }
}
