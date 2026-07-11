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
use re_rav1d::dav1d::{Decoder, Picture, PixelLayout, PlanarImageComponent, Settings};
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

/// Whether every **top-level** ISOBMFF box in `bytes` declares a size that fits
/// within the buffer — a cheap, bounded structural sanity check run before
/// `avif-parse`.
///
/// `avif-parse` 2.1.0 reads a box's declared size into a fallible buffer before
/// validating it against the bytes actually present, so a `ftyp`/`mdat` header
/// that advertises gigabytes in a tiny file drives a multi-gigabyte allocation —
/// the SPEC-069 fuzz gate found exactly this (a 286-byte input whose `ftyp` size
/// field read `0xB8000018` ≈ 3.09 GB → a 3 GB `malloc`/OOM inside `read_avif`,
/// before any of our caps could run). A conforming file's top-level boxes always
/// fit within the file, so rejecting any box that claims more than the remaining
/// bytes drops the amplifying inputs without touching valid ones. Reported
/// upstream (a parser should bound reads by the available input). This walks only
/// the top level (bounded by the buffer length) and indexes via checked slices —
/// it never panics and never trusts a size to seek beyond the buffer.
fn box_sizes_fit(bytes: &[u8]) -> bool {
    let len = bytes.len() as u64;
    let mut off: u64 = 0;
    // Each iteration consumes a full box, so `off` strictly increases (every
    // accepted `box_size >= 8`) — the loop is bounded by `len`.
    while off + 8 <= len {
        let i = off as usize;
        let size32 =
            u32::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]) as u64;
        let box_size = match size32 {
            // Size 0 means "extends to end of file": inherently bounded, and only
            // legal as the last box — accept and stop.
            0 => return true,
            // Size 1 means a 64-bit `largesize` follows the 8-byte header.
            1 => {
                if off + 16 > len {
                    return false;
                }
                let j = i + 8;
                let large = u64::from_be_bytes([
                    bytes[j],
                    bytes[j + 1],
                    bytes[j + 2],
                    bytes[j + 3],
                    bytes[j + 4],
                    bytes[j + 5],
                    bytes[j + 6],
                    bytes[j + 7],
                ]);
                // A 64-bit box must be at least its 16-byte header.
                if large < 16 {
                    return false;
                }
                large
            }
            // 2..=7 is smaller than a legal 8-byte box header.
            2..=7 => return false,
            n => n,
        };
        // The declared box must fit within the bytes remaining from here.
        if box_size > len - off {
            return false;
        }
        off += box_size;
    }
    true
}

/// Decode an AVIF byte stream to an 8-bit RGB(A) [`DynamicImage`], enforcing the
/// decode caps in `limits` (DEC-034) before allocating pixels.
///
/// The `avif-parse` container parser and the `re_rav1d` AV1 decoder are
/// third-party code driven over fully untrusted bytes. The SPEC-069 fuzz gate
/// surfaced an input that trips `avif-parse`'s internal
/// `debug_assert_eq!(0, limit, "bad parser state bytes left")`
/// (avif-parse 2.1.0 `src/lib.rs:1398`, reached from `read_avif`): a
/// **debug-assertion** that panics under `cargo test`/`cargo fuzz`
/// (debug-assertions on) though a `--release` build compiles it out and returns
/// a clean `Err`. Our contract (`untrusted-input-hardening`) is a *typed error,
/// never a panic*, in **every** profile — so we isolate the whole decode behind
/// [`std::panic::catch_unwind`] and convert any unwind (from either upstream
/// crate) into [`ImageError::Decode`]. The minimized reproducer lives at
/// `tests/fixtures/fuzz/avif_decode/`; the durability policy is DEC-062. Reported
/// upstream (avif-parse: a debug-assert on malformed input should be a returned
/// error, not a panic).
pub(crate) fn decode_avif(bytes: &[u8], limits: &Limits) -> Result<DynamicImage> {
    // `AssertUnwindSafe`: the closure only borrows `&[u8]`/`&Limits` and returns
    // a `Result` by value, so a caught unwind cannot leave observable broken
    // state behind (no locks, no `&mut` across the boundary).
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        decode_avif_inner(bytes, limits)
    }))
    .unwrap_or_else(|_| {
        Err(ImageError::Decode(
            "avif: decoder panicked on malformed input".into(),
        ))
    })
}

/// The AVIF decode body, wrapped by [`decode_avif`]'s panic boundary.
fn decode_avif_inner(bytes: &[u8], limits: &Limits) -> Result<DynamicImage> {
    // Reject a container whose top-level box sizes overrun the buffer BEFORE
    // `avif-parse` sees it: it reads a box's declared size into a buffer before
    // validating it against the available bytes, so an inflated header is a
    // decompression-bomb-by-header (DEC-034).
    if !box_sizes_fit(bytes) {
        return Err(ImageError::Decode(
            "avif: container box size exceeds input (malformed)".into(),
        ));
    }
    let parsed = avif_parse::read_avif(&mut Cursor::new(bytes)).map_err(map_parse_err)?;

    // Cap dimensions/allocation from the container metadata BEFORE decoding, so
    // an oversized header is rejected without allocating pixel planes.
    let meta = parsed.primary_item_metadata().map_err(map_parse_err)?;
    check_caps(
        meta.max_frame_width.get(),
        meta.max_frame_height.get(),
        limits,
    )?;

    let pic = decode_obus(&parsed.primary_item, limits)?;
    // Defense in depth: the decoded dimensions must also satisfy the caps.
    check_caps(pic.width(), pic.height(), limits)?;

    let w = pic.width();
    let h = pic.height();
    let mut rgba = yuv_to_rgba(&pic)?;

    // Merge the alpha plane (a separate monochrome OBU stream), if present.
    if let Some(alpha) = &parsed.alpha_item {
        let apic = decode_obus(alpha, limits)?;
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

/// Derive dav1d's `frame_size_limit` (maximum decoded frame **area**, in pixels)
/// from the DEC-034 `limits`.
///
/// The container's pre-decode dimensions (`ispe`/`avif-parse` metadata) that
/// [`check_caps`] sees are **independent** of the AV1 frame-header dimensions the
/// decoder actually allocates for: a malformed AVIF can advertise a tiny image in
/// the container yet carry an OBU whose sequence/frame header declares an enormous
/// frame, so `re_rav1d` allocates gigabytes of planes *before* our post-decode
/// `check_caps` runs. The SPEC-069 fuzz gate found exactly this — a 286-byte input
/// drove a ~3 GB allocation / OOM. Passing this limit makes `re_rav1d` reject an
/// oversize frame at header-parse time (a returned error, not an allocation), so
/// the decoder's own allocation is bounded by the same pixel budget as our output
/// buffer. `0` means unlimited (dav1d's convention) and is only used if neither
/// cap is set — never in production, where `decode_limits` sets both.
fn frame_size_limit(limits: &Limits) -> u32 {
    // The RGBA working buffer (`w*h*4`) is the largest allocation `check_caps`
    // bounds, so cap the decoder's frame area to that same pixel budget. Also
    // honor the per-dimension caps via their product, and take the tighter bound.
    let by_alloc = limits
        .max_alloc
        .map(|a| (a / 4).min(u32::MAX as u64) as u32);
    let by_dims = match (limits.max_image_width, limits.max_image_height) {
        (Some(w), Some(h)) => Some((w as u64 * h as u64).min(u32::MAX as u64) as u32),
        _ => None,
    };
    [by_alloc, by_dims].into_iter().flatten().min().unwrap_or(0)
}

/// Decode a single AV1 still image (one OBU stream) to a `re_rav1d` [`Picture`],
/// bounding the decoder's frame allocation via [`frame_size_limit`] (DEC-034).
fn decode_obus(obus: &[u8], limits: &Limits) -> Result<Picture> {
    let mut settings = Settings::new();
    settings.set_frame_size_limit(frame_size_limit(limits));
    let mut dec =
        Decoder::with_settings(&settings).map_err(|e| ImageError::Decode(format!("avif: {e}")))?;
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

    #[test]
    fn box_sizes_fit_accepts_well_formed_and_rejects_overrun() {
        // A single well-formed `ftyp` box that exactly spans the buffer
        // (size 16 = 4 size + 4 type + 4 major brand + 4 minor version).
        let mut ok = Vec::new();
        ok.extend_from_slice(&16u32.to_be_bytes());
        ok.extend_from_slice(b"ftyp");
        ok.extend_from_slice(b"avif");
        ok.extend_from_slice(&0u32.to_be_bytes()); // minor version → 16 bytes total
        assert!(box_sizes_fit(&ok));

        // The SPEC-069 OOM shape: a `ftyp` whose 32-bit size claims ~3 GB in a
        // tiny buffer. This is the decompression-bomb-by-header we reject.
        let mut bomb = Vec::new();
        bomb.extend_from_slice(&0xB800_0018u32.to_be_bytes());
        bomb.extend_from_slice(b"ftyp");
        bomb.extend_from_slice(b"avif");
        assert!(!box_sizes_fit(&bomb), "oversize box must be rejected");

        // A `size == 0` last box (extends to EOF) is accepted, not treated as an
        // overrun.
        let mut eof = Vec::new();
        eof.extend_from_slice(&0u32.to_be_bytes());
        eof.extend_from_slice(b"mdat");
        eof.extend_from_slice(&[0u8; 8]);
        assert!(box_sizes_fit(&eof));

        // A 64-bit `largesize` (size32 == 1) that overruns is rejected; a valid
        // one is accepted.
        let mut large_ok = Vec::new();
        large_ok.extend_from_slice(&1u32.to_be_bytes());
        large_ok.extend_from_slice(b"mdat");
        large_ok.extend_from_slice(&24u64.to_be_bytes()); // header(16)+8 body
        large_ok.extend_from_slice(&[0u8; 8]);
        assert!(box_sizes_fit(&large_ok));

        let mut large_bad = Vec::new();
        large_bad.extend_from_slice(&1u32.to_be_bytes());
        large_bad.extend_from_slice(b"mdat");
        large_bad.extend_from_slice(&(1u64 << 40).to_be_bytes()); // 1 TiB claim
        large_bad.extend_from_slice(&[0u8; 8]);
        assert!(!box_sizes_fit(&large_bad));

        // A degenerate size (2..=7, smaller than a legal 8-byte header) is
        // rejected rather than advancing `off` by a sub-header amount.
        let mut tiny = Vec::new();
        tiny.extend_from_slice(&4u32.to_be_bytes());
        tiny.extend_from_slice(b"ftyp");
        assert!(!box_sizes_fit(&tiny));
    }

    #[test]
    fn frame_size_limit_is_tighter_of_alloc_and_dims() {
        // Production caps: max_alloc = 512 MiB → 512Mi/4 = 134_217_728 px, which
        // is tighter than 65_535² so it wins.
        let mut l = Limits::default();
        l.max_image_width = Some(65_535);
        l.max_image_height = Some(65_535);
        l.max_alloc = Some(512 * 1024 * 1024);
        assert_eq!(frame_size_limit(&l), 134_217_728);

        // No alloc cap → falls back to the dimension product (saturating to u32).
        let mut d = Limits::default();
        d.max_image_width = Some(1000);
        d.max_image_height = Some(2000);
        d.max_alloc = None;
        assert_eq!(frame_size_limit(&d), 2_000_000);

        // Neither cap set → 0 (dav1d "unlimited"); never happens in production
        // (`decode_limits` always sets both). Note `Limits::default()` already
        // carries a `max_alloc`, so both must be cleared explicitly.
        let mut none = Limits::default();
        none.max_alloc = None;
        none.max_image_width = None;
        none.max_image_height = None;
        assert_eq!(frame_size_limit(&none), 0);
    }
}
