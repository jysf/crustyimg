//! Pure-Rust AVIF decode (SPEC-058, DEC-053). **Native targets only**
//! (SPEC-072, DEC-064): `re_rav1d` cannot compile to `wasm32-unknown-unknown`.
//! The AVIF *sniff* that dispatches here lives in `super::sniff` so it survives
//! into the wasm build, where an AVIF input is answered with a typed
//! `ImageError::CodecUnavailableOnTarget` rather than reaching this module.
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

/// Reject dimensions that exceed the `limits` (dimension or total allocation) or
/// the shared peak-memory pixel budget (DEC-063).
///
/// The allocation estimate uses the 8-bit RGBA working buffer (`w * h * 4`),
/// the largest intermediate this module allocates. The pixel budget is the
/// tighter, uniform bound (it supersedes this module's implicit 128 Mpix
/// `max_alloc / 4` ceiling) and lives in one place — `super::check_pixel_budget`.
fn check_caps(w: u32, h: u32, limits: &Limits) -> Result<()> {
    super::check_pixel_budget(w, h)?;
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
/// buffer.
///
/// The shared peak-memory pixel budget (DEC-063) is always in force, so unlike the
/// `limits`-derived bounds it is never absent — the returned limit is never dav1d's
/// `0` ("unlimited"), whatever `limits` carries.
fn frame_size_limit(limits: &Limits) -> u32 {
    // The peak-memory budget bounds every decode path (DEC-063) and is the tightest
    // bound in production; the `limits`-derived bounds can still be tighter in a
    // test seam, so take the minimum of whichever apply.
    let mut limit = super::MAX_IMAGE_PIXELS.min(u32::MAX as u64) as u32;
    // The RGBA working buffer (`w*h*4`) is the largest allocation `check_caps`
    // bounds, so cap the decoder's frame area to that same pixel budget. Also
    // honor the per-dimension caps via their product.
    if let Some(a) = limits.max_alloc {
        limit = limit.min((a / 4).min(u32::MAX as u64) as u32);
    }
    if let (Some(w), Some(h)) = (limits.max_image_width, limits.max_image_height) {
        limit = limit.min((w as u64 * h as u64).min(u32::MAX as u64) as u32);
    }
    limit
}

/// The AVIF-decode thread policy (DEC-077): decode every still frame on a single
/// thread rather than inheriting dav1d's `n_threads = 0` (= all logical cores).
///
/// We decode exactly **one** AV1 still frame per call, so frame-threading (the
/// `n_fc` frame contexts dav1d derives from the core count) buys nothing — there
/// is no second frame to overlap. Worse, batch runs already parallelize *across
/// files* with rayon (DEC-006), so an all-cores decoder pool spawned *per file*
/// is textbook oversubscription (on a 14-core box, 14 rayon workers each starting
/// a ~14-thread dav1d pool). That oversubscription is what trips `re_rav1d`'s
/// debug-mode `DisjointMut` overlap check (a *real* cross-thread data race between
/// its CDEF/loop-restoration workers; silent in release, where the targets are
/// provenanceless so the consequence is wrong pixels, not a memory-safety hole).
///
/// `re_rav1d` spawns its `rav1d-worker-*` threads only when the tile-context count
/// `n_tc > 1` (`re_rav1d-0.1.3/src/lib.rs:257`); `n_tc == 1` runs the whole decode
/// inline on the calling thread. Pinning to 1 thread therefore means **no worker
/// threads exist to overlap**, and file-level rayon still keeps every core busy.
/// This extends the DEC-034 precedent of setting decode `Settings` deliberately.
const AVIF_DECODE_THREADS: u32 = 1;

/// Stack size (bytes) for the thread the inline decode runs on (DEC-077).
///
/// With [`AVIF_DECODE_THREADS`] `== 1`, `re_rav1d` spawns **no** `rav1d-worker-*`
/// threads and runs the whole decode *inline on the calling thread* — so it loses
/// the 2 MiB stack those workers would otherwise get (Rust's default thread stack;
/// `re_rav1d` deliberately does not shrink it, `re_rav1d-0.1.3/src/lib.rs:258`,
/// "Don't set stack size like dav1d does", upstream rav1d#889). dav1d's decode has
/// a large *fixed* stack frame (on-stack tile/context structures) independent of
/// image size, so an inline decode overflows a small caller stack **regardless of
/// resolution**: on Windows — whose main thread gets only ~1 MiB — even a 16×16
/// AVIF stack-overflowed both windows-latest legs of PR #95 (macOS/Linux main
/// threads are ~8 MiB, so the inline decode never overflowed there). `frame_size_
/// limit` (DEC-034) bounds *heap* plane allocation, a separate concern from stack.
///
/// So we never run the inline decode on the caller's (OS-defined, possibly ~1 MiB)
/// stack: we re-spawn it onto a thread whose stack we control. 8 MiB matches the
/// macOS/Linux main-thread stack that has always decoded these frames without
/// overflow and is 4× dav1d's own 2 MiB worker default — ample headroom. A large
/// `stack_size` only *reserves* address space (committed lazily), so the cost of
/// the generous figure is negligible.
const AVIF_DECODE_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Build the decode [`Settings`] used for every AVIF frame: the DEC-034 frame-size
/// cap plus the explicit DEC-077 single-thread policy ([`AVIF_DECODE_THREADS`]).
/// Factored out so the thread policy is asserted against the exact `Settings`
/// production uses, not a re-derived copy.
fn decode_settings(limits: &Limits) -> Settings {
    let mut settings = Settings::new();
    settings.set_frame_size_limit(frame_size_limit(limits));
    settings.set_n_threads(AVIF_DECODE_THREADS);
    settings
}

/// Decode a single AV1 still image (one OBU stream) to a `re_rav1d` [`Picture`].
///
/// Because the DEC-077 single-thread policy makes `re_rav1d` decode inline (no
/// worker threads, see [`AVIF_DECODE_STACK_SIZE`]), we run that inline decode on a
/// thread we spawn with an ample stack rather than on the caller's — whose size is
/// set by the OS and is only ~1 MiB on the Windows main thread, too small for
/// dav1d's fixed decode frame. `Picture` is `Send` (`Arc`-backed), so the decoded
/// frame is returned across the `join`. A scoped thread lets the closure borrow
/// `obus`/`limits` without cloning; the parent blocks on `join`, so no CPU-bound
/// work overlaps it — under a rayon batch (DEC-006) each worker adds exactly one
/// (blocked→working) decode thread, not an all-cores pool, so this does not
/// reintroduce oversubscription. The spawn costs ~tens of µs against a ms-scale
/// decode — negligible.
///
/// **SPEC-094:** an empty `obus` slice must never reach `re_rav1d` — its
/// `send_data` validates `sz > 0` via a `debug_abort()` that calls
/// `std::process::abort()` under `cfg!(debug_assertions)`. `abort()` is not an
/// unwind, so it bypasses both `decode_avif`'s `catch_unwind` and this
/// function's own scoped-thread `join`; the only fix is to stop the empty
/// stream *before* it reaches the decoder. This is the single chokepoint both
/// the primary and alpha decode paths flow through (the only `send_data` /
/// `Decoder::with_settings` call sites in the crate), so guarding here covers
/// every caller.
fn decode_obus(obus: &[u8], limits: &Limits) -> Result<Picture> {
    if obus.is_empty() {
        return Err(ImageError::Decode(
            "avif: empty OBU stream (0 bytes)".into(),
        ));
    }
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("avif-decode".into())
            .stack_size(AVIF_DECODE_STACK_SIZE)
            .spawn_scoped(scope, || decode_obus_inline(obus, limits))
            .map_err(|e| ImageError::Decode(format!("avif: spawn decode thread: {e}")))?
            .join()
            // A stack overflow aborts the process, so a returned `Err` here is an
            // ordinary decoder panic (e.g. avif-parse debug-assert on malformed
            // input) — mapped to a typed error, matching the `decode_avif`
            // panic boundary it used to sit behind.
            .map_err(|_| ImageError::Decode("avif: decoder panicked on malformed input".into()))?
    })
}

/// The actual inline `re_rav1d` decode, run on the ample-stack thread spawned by
/// [`decode_obus`]. Bounds the decoder's frame allocation via [`frame_size_limit`]
/// (DEC-034) and pins threads via [`decode_settings`] (DEC-077).
fn decode_obus_inline(obus: &[u8], limits: &Limits) -> Result<Picture> {
    let settings = decode_settings(limits);
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

    /// Build a byte-exact box: `[u32 size][4-byte fourcc][payload]`.
    fn bx(fourcc: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut out = ((8 + payload.len()) as u32).to_be_bytes().to_vec();
        out.extend_from_slice(fourcc);
        out.extend_from_slice(payload);
        out
    }

    /// Builds a conforming (non-corrupt) AVIF whose **alpha** item's OBU
    /// stream is genuinely empty (`alpha_item == Some(<empty>)`), to answer
    /// SPEC-094's reachability question directly rather than assume it.
    ///
    /// The mechanism: `avif-parse`'s iloc `matches_extent` check for a
    /// `ToEnd` extent (wire `extent_length == 0`) compares only the mdat's
    /// **file offset**, never its current byte count — so two items whose
    /// `iloc` entries both point at the SAME mdat offset each get a
    /// `mem::take` of whatever bytes remain there at the time they're
    /// processed. Listing the primary item first drains the real OBU bytes
    /// into `primary_item`; the alpha item's identical-offset entry then
    /// drains the (already-emptied) leftover, yielding
    /// `alpha_item == Some(TryVec::new())`. This is a legal reading of the
    /// ISOBMFF `iloc` box, not a corrupt/malformed container.
    fn build_avif_with_empty_alpha(primary_obu: &[u8]) -> Vec<u8> {
        let mut ftyp_payload = b"avif".to_vec();
        ftyp_payload.extend_from_slice(&0u32.to_be_bytes()); // minor_version
        let ftyp = bx(b"ftyp", &ftyp_payload);

        let infe = |item_id: u16| -> Vec<u8> {
            let mut p = vec![2, 0, 0, 0]; // fullbox version=2 (mif1 min), flags=0
            p.extend_from_slice(&item_id.to_be_bytes());
            p.extend_from_slice(&0u16.to_be_bytes()); // item_protection_index
            p.extend_from_slice(b"av01");
            bx(b"infe", &p)
        };
        let mut iinf_payload = vec![0, 0, 0, 0]; // fullbox v0 f0
        iinf_payload.extend_from_slice(&2u16.to_be_bytes()); // entry_count
        iinf_payload.extend_from_slice(&infe(1)); // primary
        iinf_payload.extend_from_slice(&infe(2)); // alpha
        let iinf = bx(b"iinf", &iinf_payload);

        let mut pitm_payload = vec![0, 0, 0, 0];
        pitm_payload.extend_from_slice(&1u16.to_be_bytes()); // primary_item_id = 1
        let pitm = bx(b"pitm", &pitm_payload);

        let mut auxl_payload = 2u16.to_be_bytes().to_vec(); // from_item_id = 2 (alpha)
        auxl_payload.extend_from_slice(&1u16.to_be_bytes()); // reference_count
        auxl_payload.extend_from_slice(&1u16.to_be_bytes()); // to_item_id = 1 (primary)
        let auxl = bx(b"auxl", &auxl_payload);
        let mut iref_payload = vec![0, 0, 0, 0]; // fullbox v0 f0
        iref_payload.extend_from_slice(&auxl);
        let iref = bx(b"iref", &iref_payload);

        const ALPHA_URN: &[u8] = b"urn:mpeg:mpegB:cicp:systems:auxiliary:alpha";
        let mut auxc_payload = vec![0, 0, 0, 0]; // fullbox v0 f0
        auxc_payload.extend_from_slice(ALPHA_URN);
        let auxc = bx(b"auxC", &auxc_payload);
        let ipco = bx(b"ipco", &auxc);

        let mut ipma_payload = vec![0, 0, 0, 0]; // fullbox v0 f0
        ipma_payload.extend_from_slice(&1u32.to_be_bytes()); // entry_count
        ipma_payload.extend_from_slice(&2u16.to_be_bytes()); // item_id = 2 (alpha)
        ipma_payload.push(1); // association_count
        ipma_payload.push(0x01); // essential=0, property_index=1 (1-based -> auxC)
        let ipma = bx(b"ipma", &ipma_payload);

        let mut iprp_payload = Vec::new();
        iprp_payload.extend_from_slice(&ipco);
        iprp_payload.extend_from_slice(&ipma);
        let iprp = bx(b"iprp", &iprp_payload);

        // iloc (version 0, offset_size=length_size=base_offset_size=4) has a
        // fixed size regardless of the base_offset value it carries: fullbox
        // (4) + sizes byte + base/reserved byte (2) + item_count (2) + 2
        // items * (item_id(2) + data_reference_index(2) + base_offset(4) +
        // extent_count(2) + extent_offset(4) + extent_length(4) = 18) = 44.
        const ILOC_PAYLOAD_LEN: usize = 4 + 2 + 2 + 2 * 18;
        let meta_payload_len_without_iloc = 4 + iinf.len() + pitm.len() + iref.len() + iprp.len();
        let meta_total_len = 8 + meta_payload_len_without_iloc + 8 + ILOC_PAYLOAD_LEN;
        // The mdat's *content* offset (what avif-parse's OffsetReader reports
        // as `MediaDataBox::offset`) is everything written before it,
        // including mdat's own 8-byte header.
        let mdat_offset = (ftyp.len() + meta_total_len + 8) as u32;

        let iloc_item = |item_id: u16| -> Vec<u8> {
            let mut p = item_id.to_be_bytes().to_vec();
            p.extend_from_slice(&0u16.to_be_bytes()); // data_reference_index
            p.extend_from_slice(&mdat_offset.to_be_bytes()); // base_offset
            p.extend_from_slice(&1u16.to_be_bytes()); // extent_count
            p.extend_from_slice(&0u32.to_be_bytes()); // extent_offset
            p.extend_from_slice(&0u32.to_be_bytes()); // extent_length = 0 => ToEnd
            p
        };
        let mut iloc_payload = vec![0, 0, 0, 0]; // fullbox v0 f0
        iloc_payload.push(0x44); // offset_size=4, length_size=4
        iloc_payload.push(0x40); // base_offset_size=4, reserved=0
        iloc_payload.extend_from_slice(&2u16.to_be_bytes()); // item_count
        iloc_payload.extend_from_slice(&iloc_item(1)); // primary: drains the real mdat
        iloc_payload.extend_from_slice(&iloc_item(2)); // alpha: drains the (now-empty) leftover
        assert_eq!(iloc_payload.len(), ILOC_PAYLOAD_LEN);
        let iloc = bx(b"iloc", &iloc_payload);

        let mut meta_payload = vec![0, 0, 0, 0]; // fullbox v0 f0
        meta_payload.extend_from_slice(&iinf);
        meta_payload.extend_from_slice(&pitm);
        meta_payload.extend_from_slice(&iref);
        meta_payload.extend_from_slice(&iprp);
        meta_payload.extend_from_slice(&iloc);
        let meta = bx(b"meta", &meta_payload);
        assert_eq!(meta.len(), meta_total_len);

        let mdat = bx(b"mdat", primary_obu);

        let mut out = Vec::new();
        out.extend_from_slice(&ftyp);
        out.extend_from_slice(&meta);
        out.extend_from_slice(&mdat);
        out
    }

    /// Extract the real primary-item OBU bytes from the committed 16x16
    /// fixture, so the crafted container decodes a genuine still image on
    /// the primary path (only the alpha path is hostile).
    fn real_primary_obu() -> Vec<u8> {
        const FIXTURE: &[u8] = include_bytes!("../../tests/fixtures/avif/solid_16x16.avif");
        let parsed = avif_parse::read_avif(&mut std::io::Cursor::new(FIXTURE))
            .expect("parse committed avif fixture");
        parsed.primary_item.as_slice().to_vec()
    }

    /// Confirms SPEC-094's reachability question directly: `avif-parse` DOES
    /// accept a crafted-but-conforming container whose `alpha_item` is
    /// `Some(<empty>)`, and the empty slice really does reach
    /// `parsed.alpha_item`. This is a probe over the parse layer only (no
    /// decode) so it can run safely in any build profile; the
    /// debug-mode-abort proof is [`empty_alpha_obu_is_typed_error_not_abort`]
    /// below, which drives the same crafted bytes all the way to `decode_avif`.
    #[test]
    fn crafted_container_yields_empty_alpha_item() {
        let bytes = build_avif_with_empty_alpha(&real_primary_obu());
        let parsed = avif_parse::read_avif(&mut std::io::Cursor::new(bytes))
            .expect("crafted container must parse as well-formed AVIF");
        assert!(
            !parsed.primary_item.is_empty(),
            "primary item must still carry the real OBU bytes"
        );
        assert_eq!(
            parsed.alpha_item.as_deref(),
            Some(&[][..]),
            "alpha item must be Some(<empty>) -- the exact shape SPEC-094 asks about"
        );
    }

    /// SPEC-094 crux. A conforming AVIF whose alpha item is genuinely empty
    /// (see [`build_avif_with_empty_alpha`]) must decode to a typed
    /// `ImageError::Decode`, never crash the process.
    ///
    /// **Pre-fix, this test proves the bug by aborting the test binary.**
    /// `decode_obus` hands the empty alpha slice straight to
    /// `re_rav1d::dav1d::Decoder::send_data`, which validates `sz > 0` via the
    /// `validate_input!` macro; failing that check calls `debug_abort()` →
    /// `std::process::abort()` under `cfg!(debug_assertions)` (on in `cargo
    /// test`). `abort()` is not an unwind, so neither `decode_avif`'s
    /// `catch_unwind` nor `decode_obus`'s scoped-thread `join` catches it —
    /// the whole process dies (observed: `cargo test` reports the test binary
    /// exiting via `signal: 6, SIGABRT`, not a failed assertion). Post-fix,
    /// the `is_empty()` guard at the top of `decode_obus` rejects the alpha
    /// slice before it ever reaches `re_rav1d`, and this test passes normally.
    #[test]
    fn empty_alpha_obu_is_typed_error_not_abort() {
        let bytes = build_avif_with_empty_alpha(&real_primary_obu());
        let result = decode_avif(&bytes, &Limits::default());
        assert!(
            matches!(result, Err(ImageError::Decode(_))),
            "got {result:?}"
        );
    }

    /// SPEC-094 belt-and-suspenders: the primary path is already screened by
    /// `avif-parse`'s `parse_obu` metadata pre-check (an empty primary item
    /// never reaches `decode_obus` via the normal `decode_avif` route), so this
    /// pins the `decode_obus` chokepoint's OWN contract directly — no empty
    /// slice reaches `re_rav1d` from ANY caller, not just the ones reachable
    /// through a full container parse.
    #[test]
    fn empty_primary_obu_is_typed_error() {
        let result = decode_obus(&[], &Limits::default());
        assert!(
            matches!(result, Err(ImageError::Decode(_))),
            "got {result:?}"
        );
    }

    /// SPEC-094: the guard must reject only genuinely empty streams. A real
    /// AVIF with a non-empty alpha channel decodes pixel-identically to the
    /// pre-fix binary. The golden FNV-1a digest below was captured from this
    /// exact fixture on the pre-fix binary (mirrors the SPEC-091
    /// `avif_decode_pixels_unchanged_by_thread_policy` pattern: an
    /// independent value the code under test cannot fabricate).
    #[test]
    fn valid_avif_with_alpha_unchanged() {
        const FIXTURE: &[u8] = include_bytes!("../../tests/fixtures/avif/solid_16x16_alpha.avif");
        const PRE_FIX_RGBA_FNV1A: u64 = 0x4a53_054d_660f_c525;

        fn fnv1a(bytes: &[u8]) -> u64 {
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for &b in bytes {
                h ^= b as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
            h
        }

        let img = decode_avif(FIXTURE, &Limits::default()).expect("decode alpha avif");
        assert_eq!((img.width(), img.height()), (16, 16));
        let rgba = img.to_rgba8();
        assert_eq!(
            fnv1a(rgba.as_raw()),
            PRE_FIX_RGBA_FNV1A,
            "the empty-OBU guard must not change valid-alpha decode output"
        );
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
    fn frame_size_limit_is_tighter_of_budget_alloc_and_dims() {
        // Production caps: the DEC-063 pixel budget (67_108_864 px) is tighter than
        // both max_alloc/4 (134_217_728 px) and 65_535², so it wins — the mirror of
        // the production constants that must not drift from `decode_limits()`.
        let mut l = Limits::default();
        l.max_image_width = Some(65_535);
        l.max_image_height = Some(65_535);
        l.max_alloc = Some(512 * 1024 * 1024);
        assert_eq!(frame_size_limit(&l), 67_108_864);
        assert_eq!(
            frame_size_limit(&l) as u64,
            crate::image::MAX_IMAGE_PIXELS,
            "the mirror must not drift from the production pixel budget"
        );

        // A tighter `limits` still wins over the budget: dims product (saturating).
        let mut d = Limits::default();
        d.max_image_width = Some(1000);
        d.max_image_height = Some(2000);
        d.max_alloc = None;
        assert_eq!(frame_size_limit(&d), 2_000_000);

        // No `limits` caps at all → the pixel budget is STILL in force (it is not a
        // `Limits` field), so the decoder is never handed dav1d's 0 = "unlimited".
        let mut none = Limits::default();
        none.max_alloc = None;
        none.max_image_width = None;
        none.max_image_height = None;
        assert_eq!(
            frame_size_limit(&none) as u64,
            crate::image::MAX_IMAGE_PIXELS
        );
    }

    /// SPEC-070: dims that pass EVERY DEC-034 cap are still rejected when they
    /// exceed the DEC-063 pixel budget — the gap the implicit 128 Mpix
    /// (`max_alloc / 4`) ceiling left open.
    #[test]
    fn check_caps_rejects_over_pixel_budget() {
        let mut prod = Limits::default();
        prod.max_image_width = Some(65_535);
        prod.max_image_height = Some(65_535);
        prod.max_alloc = Some(512 * 1024 * 1024);

        // 10000×10000 = 100 Mpix: each side < 65_535 AND the RGBA buffer (400 MB)
        // is under the 512 MiB alloc cap — it passes every DEC-034 cap and is
        // caught ONLY by the pixel budget.
        assert!(matches!(
            check_caps(10_000, 10_000, &prod),
            Err(ImageError::LimitsExceeded(_))
        ));
        // A 24 MP frame passes.
        assert!(check_caps(6_000, 4_000, &prod).is_ok());
    }

    /// SPEC-091/DEC-077: the decode `Settings` set a thread count deliberately,
    /// rather than silently inheriting dav1d's `n_threads = 0` (= all cores). The
    /// guard on `Settings::new()` documents the default we override — if a future
    /// dav1d bump changed that default, this pins what we are deviating from.
    #[test]
    fn avif_decode_thread_policy_is_explicit() {
        assert_eq!(
            AVIF_DECODE_THREADS, 1,
            "still-image decode is single-threaded (DEC-077)"
        );
        // The exact Settings production uses (not a re-derivation) carry the cap.
        let s = decode_settings(&Limits::default());
        assert_eq!(
            s.get_n_threads(),
            AVIF_DECODE_THREADS,
            "decode_settings must set the explicit thread policy"
        );
        // The dav1d default is all-cores auto (0); prove we are not inheriting it.
        assert_eq!(
            Settings::new().get_n_threads(),
            0,
            "dav1d default n_threads is 0 (=auto all cores); the policy overrides it"
        );
        assert_ne!(s.get_n_threads(), Settings::new().get_n_threads());
    }

    /// SPEC-091/DEC-077: a rayon batch (DEC-006) must not spawn an ~N-core decoder
    /// pool *per file*. `re_rav1d` spawns its `rav1d-worker-*` threads only when the
    /// tile-context count `n_tc > 1` (`re_rav1d-0.1.3/src/lib.rs:257`), and `n_tc`
    /// is derived directly from `n_threads`; `n_threads == 1` ⇒ `n_tc == 1` ⇒ zero
    /// worker threads ⇒ no oversubscription and no cross-thread overlap. The
    /// observable, deterministic proxy is the thread count on the production
    /// `Settings`; the wall-clock effect is covered by the DEC-077 measurements.
    #[test]
    fn avif_batch_decode_does_not_oversubscribe() {
        let s = decode_settings(&Limits::default());
        assert_eq!(
            s.get_n_threads(),
            1,
            "a single decode thread spawns no rav1d-worker pool, so N rayon workers \
             cannot each start an N-core dav1d pool"
        );
    }
}
