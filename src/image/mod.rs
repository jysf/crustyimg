//! The canonical in-memory image model (DEC-002).
//!
//! This module is the **stable pixel core**: it wraps the single pixel library
//! (`image`, referred to as `::image` here to avoid the module-name collision)
//! in one [`Image`] type, plus a read-only [`ImageInfo`] inspection struct and
//! a raw [`MetadataBundle`] captured at load.
//!
//! Layering (see `docs/architecture.md`): this module depends only on
//! `::image`, `std`, and [`crate::error`]. It must NOT touch `clap`,
//! files-policy, terminals, or recipe/source/sink types.
//!
//! ## Metadata capture (DEC-003)
//!
//! The `image` crate discards container metadata on encode, so the canonical
//! model captures the raw EXIF/ICC segments alongside the decoded pixels *at
//! load* — without interpreting them. Capture is byte-scanning of the
//! container (JPEG APP1 `Exif\0\0`; PNG `eXIf`/`iCCP` chunks), NOT EXIF
//! parsing: the bytes are stored verbatim for the later metadata lane
//! (STAGE-004). Capture is best-effort; an absent or unreadable segment is
//! simply `None`.

use std::io::{Cursor, Read, Seek};
use std::path::Path;

use ::image::{ColorType, DynamicImage, ImageFormat, ImageReader};

use crate::error::{ImageError, Result};

// The AVIF DECODER is native-only (SPEC-072, DEC-064): `re_rav1d` does not compile
// to wasm32. Its *sniff* lives in `sniff` (target-independent) so the wasm build
// still recognizes AVIF and rejects it with a typed error — see `decode_with_limits`.
#[cfg(not(target_arch = "wasm32"))]
mod avif;
mod heic;
// `pub(crate)` (not `mod raw;`) so the wasm surface (`src/wasm.rs`, a sibling
// module of `image`) can reach `raw::is_raw_extension` and
// `raw::largest_declared_preview_pixels` directly (SPEC-103) rather than this
// module growing a wrapper for each — same-crate-only, so it changes nothing
// about `raw`'s external API (still not `pub`).
pub(crate) mod raw;
mod sniff;
mod svg;

/// Maximum image dimension (width or height) in pixels accepted at decode time
/// (DEC-034). Any image declaring a dimension above this is rejected with
/// [`ImageError::LimitsExceeded`] before any pixel data is read.
const MAX_IMAGE_DIMENSION: u32 = 65_535;

/// Maximum memory that the decoder may allocate for a single image in bytes
/// (512 MiB, DEC-034). Inputs whose decoded buffer would exceed this cap are
/// rejected before allocation.
const MAX_ALLOC_BYTES: u64 = 512 * 1024 * 1024;

/// Maximum total pixels (`width × height`) accepted at decode time (DEC-063):
/// **64 Mpix** (67 108 864 px ≈ 8192×8192).
///
/// This is the **peak**-memory bound that [`MAX_ALLOC_BYTES`] cannot be:
/// `image::Limits.max_alloc` bounds a **single** allocation (the crate decrements
/// it per `reserve()` and restores it on free), not the decoder's **cumulative**
/// working set. So several sub-512 MiB buffers sum to a multi-gigabyte peak
/// without ever tripping it — a 782-byte `.nef` whose embedded JPEG declares
/// 16384×9776 (160 Mpix) drove a **~1.9 GB** peak while passing every DEC-034 cap
/// (SPEC-069's F-RAW-1). `image` 0.25's `Limits` has no total-pixel or peak field,
/// so the bound has to be ours, checked on the **declared** dimensions before the
/// decode allocates.
///
/// The value is derived in DEC-063: a **1 GiB peak budget** ÷ (4 bytes/px RGBA ×
/// a **4× amplification factor**, measured on JPEG in SPEC-069) = 64 Mpix. It
/// rejects the bomb and keeps every consumer/prosumer photo (a 24 MP or 50 MP
/// frame is far under); a **>64 MP** medium-format image is rejected — the
/// tradeoff DEC-063 states rather than hides. It supersedes the implicit 128 Mpix
/// single-RGBA-buffer bound the AVIF/SVG caps had via `max_alloc / 4`.
///
/// Enforced by [`check_pixel_budget`] at **every** decode seam (generic, RAW,
/// AVIF, SVG, HEIC); [`MAX_IMAGE_DIMENSION`] stays the per-side backstop.
pub(crate) const MAX_IMAGE_PIXELS: u64 = 64 * 1024 * 1024;

/// Reject an image whose **declared** dimensions exceed the [`MAX_IMAGE_PIXELS`]
/// peak-memory budget (DEC-063), before any pixel buffer is allocated.
///
/// The one source of truth for the cap: every decode path calls this with the
/// dimensions it reads from the container/frame header, so the four decoders
/// cannot drift apart. Multiplication is **saturating** — hostile dimensions must
/// never overflow or panic, only be rejected (`no-unwrap-on-recoverable-paths`).
pub(crate) fn check_pixel_budget(w: u32, h: u32) -> Result<()> {
    let pixels = (w as u64).saturating_mul(h as u64);
    if pixels > MAX_IMAGE_PIXELS {
        return Err(ImageError::LimitsExceeded(format!(
            "image {w}x{h} declares {pixels} pixels, over the {MAX_IMAGE_PIXELS}-pixel \
             decode budget (~64 MP; peak decode memory would exceed 1 GiB)"
        )));
    }
    Ok(())
}

/// Whether `bytes` is missing a well-formed JPEG's trailing end-of-image
/// marker (`FF D9`) — a container-level check, not a decoder change (F1,
/// SPEC-107, DEC-085). The `image` crate's JPEG decoder tolerates a missing
/// EOI by design (it decodes what entropy data is present and returns), so a
/// truncated JPEG otherwise decodes "successfully" and silently — unlike PNG
/// and AVIF, which both error on the equivalent truncation.
///
/// `bytes.ends_with` cannot panic on a shorter-than-2-byte input
/// (`no-unwrap-on-recoverable-paths`): it simply returns `false`, which reads
/// as "missing" — correct, since a buffer that short is not a well-formed
/// JPEG in the first place (this helper is only ever consulted after the
/// format has already been detected as JPEG).
fn jpeg_missing_eoi(bytes: &[u8]) -> bool {
    const EOI: [u8; 2] = [0xFF, 0xD9];
    !bytes.ends_with(&EOI)
}

/// The stderr warning the CLI prints on [`Image::is_truncated_jpeg`] (F1,
/// SPEC-107) — centralized so `info`/`web`/`convert`/`resize` print identical
/// wording and the test suite asserts on one string. Native-only (no `cli`
/// on `wasm32` — `Image` still computes and carries the flag on every
/// target, but only the native CLI turns it into a printed warning).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const TRUNCATED_JPEG_WARNING: &str =
    "truncated JPEG: missing end-of-image marker (FF D9) — the decoded image may be incomplete";

/// The stderr warning the CLI prints on [`Image::is_animated_input`]
/// (SPEC-119) — centralized so `convert`/`optimize`/`web`/`build` print
/// identical wording. Native-only, same reasoning as
/// [`TRUNCATED_JPEG_WARNING`]: `Image` computes and carries the flag on
/// every target, but only the native CLI turns it into a printed warning.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const ANIMATED_INPUT_WARNING: &str =
    "animated input flattened to a single frame — crustyimg does not yet write animated \
     output, so every frame but the first was discarded";

/// Whether `bytes` — already detected as `format` — is a multi-frame
/// animated source that [`Image::from_bytes`] flattens to `pixels`' single
/// frame (Call 2/3, SPEC-119).
///
/// Checked for the three `::image::AnimationDecoder` impls this repo's
/// pinned `image` 0.25.10 build enables — GIF, APNG, and animated WebP (the
/// SPEC-119 sweep: `grep -rn "impl.*AnimationDecoder.*for"` over the crate
/// source). TIFF/ICO/BMP/JPEG have no such impl and fall through `_ => false`.
///
/// **AVIF is not in this match** — not because it is unchecked, but because
/// it cannot reach here un-warned in the first place: an AVIF sequence
/// (`ftyp` major brand `avis`) is rejected by `avif_parse::read_avif` with a
/// typed `Unsupported` error *before* `decode_with_limits` returns, so
/// `Image::from_bytes` never constructs an `Image` from one — see
/// `avif::decode_avif_inner` and its `map_parse_err`. A single-image AVIF
/// (`avif` brand) has nothing to flatten.
fn detect_animated_input(bytes: &[u8], format: ImageFormat) -> bool {
    match format {
        ImageFormat::Gif => gif_is_animated(bytes),
        ImageFormat::Png => png_is_apng(bytes),
        ImageFormat::WebP => webp_is_animated(bytes),
        _ => false,
    }
}

/// Whether a GIF has ≥2 frames. Reuses the shipped `image` GIF decoder
/// (`gif` is enabled in both the default and lean builds) and decodes at
/// most two frames — cheap, no full-animation decode. A decode error ⇒
/// `false`: a corrupt file is `lint`'s `size/truncated-or-corrupt` finding,
/// not this one's concern, and the caller already holds a successfully
/// decoded `pixels` buffer from a separate (non-animation-aware) decode, so
/// failing "closed" here would contradict a frame we already have.
fn gif_is_animated(bytes: &[u8]) -> bool {
    use ::image::codecs::gif::GifDecoder;
    use ::image::AnimationDecoder;
    match GifDecoder::new(Cursor::new(bytes)) {
        Ok(dec) => dec.into_frames().take(2).count() >= 2,
        Err(_) => false,
    }
}

/// Whether a PNG carries an `acTL` chunk (is an APNG). `PngDecoder::is_apng`
/// returns `ImageResult<bool>` rather than a plain `bool`
/// (`no-unwrap-on-recoverable-paths`: `.unwrap_or(false)`, never `.unwrap()`)
/// — a decode error or a plain (non-animated) PNG both read as `false`, same
/// reasoning as [`gif_is_animated`].
fn png_is_apng(bytes: &[u8]) -> bool {
    use ::image::codecs::png::PngDecoder;
    match PngDecoder::new(Cursor::new(bytes)) {
        Ok(dec) => dec.is_apng().unwrap_or(false),
        Err(_) => false,
    }
}

/// Whether a WebP carries an animation (`ANIM`/`ANMF` chunks). Cheaper than
/// the GIF/APNG checks — `WebPDecoder::has_animation` reads a header flag,
/// no frame decode at all. A decode error ⇒ `false`, same reasoning as
/// [`gif_is_animated`].
fn webp_is_animated(bytes: &[u8]) -> bool {
    use ::image::codecs::webp::WebPDecoder;
    match WebPDecoder::new(Cursor::new(bytes)) {
        Ok(dec) => dec.has_animation(),
        Err(_) => false,
    }
}

/// Where `source_format`'s value actually came from (SPEC-115).
///
/// Most decoders report a real `::image::ImageFormat` for the container on
/// disk. Three do not: `::image::ImageFormat` has no SVG/HEIC variant, and a
/// RAW container's *embedded preview* is a JPEG even though the file on disk
/// is not — so those three decoders adopt a raster stand-in label
/// (`source_format()` stays that label; `info` depends on it, DEC-055/AC-10).
/// This is the fact `source_format()` alone cannot carry: whether the raw
/// bytes on disk are actually a valid file of that label, i.e. whether they
/// can ever be shipped verbatim under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceContainer {
    /// `source_format` names the real container on disk — the raw bytes ARE a
    /// valid file of that format.
    Native,
    /// Rasterized from SVG/XML text; `source_format` reports `Png` as a
    /// stand-in (no `ImageFormat::Svg` exists).
    Svg,
    /// Decoded from an ISOBMFF/HEIF container; `source_format` reports `Png`
    /// as a stand-in (no `ImageFormat::Heic` exists).
    Heic,
    /// The extracted embedded preview of a RAW container; `source_format`
    /// reports `Jpeg` (the preview's own format), but the bytes on disk are
    /// the whole RAW container, not a standalone JPEG file.
    RawPreview,
}

impl SourceContainer {
    /// Whether the raw bytes on disk are actually a valid file of
    /// `source_format` — i.e. whether they can ever be shipped verbatim.
    pub fn is_native(self) -> bool {
        matches!(self, SourceContainer::Native)
    }
}

/// The one canonical in-memory image model (DEC-002).
///
/// Wraps the decoded pixels, the format detected at load, and an optional raw
/// [`MetadataBundle`]. The pipeline owns exactly one `Image` per input and
/// transforms it in memory (decode-once); SPEC-002 only provides the load
/// entries and inspection.
#[derive(Debug, Clone)]
pub struct Image {
    pixels: DynamicImage,
    source_format: ImageFormat,
    metadata: Option<MetadataBundle>,
    /// Set at decode time (F1, SPEC-107) when `source_format` is JPEG and the
    /// input bytes are missing the trailing end-of-image marker (`FF D9`).
    /// The `image` crate's JPEG decoder tolerates a missing EOI by design and
    /// returns a (possibly incomplete) image rather than erroring, unlike
    /// PNG/AVIF — so a truncated JPEG decodes "successfully" here and the CLI
    /// layer is what turns this flag into a stderr warning.
    truncated_jpeg: bool,
    /// Set at decode time (SPEC-119) when the source is a multi-frame
    /// animated GIF/APNG/WebP: the pixel path decodes exactly one
    /// `DynamicImage`, so every frame after the first is silently discarded
    /// unless the CLI layer turns this flag into a stderr warning — the
    /// sibling of `truncated_jpeg` (same carrier, same reasoning, a
    /// different loss).
    animated_input: bool,
    /// Where `source_format` actually came from (SPEC-115) — see
    /// [`SourceContainer`].
    source_container: SourceContainer,
}

impl Image {
    /// Open a file, detect its format, decode the pixels, and capture the raw
    /// metadata bundle.
    ///
    /// A missing/unreadable file is [`ImageError::Io`]; an undetectable format
    /// is [`ImageError::UnsupportedFormat`]; a decode failure is
    /// [`ImageError::Decode`].
    pub fn load(path: impl AsRef<Path>) -> Result<Image> {
        let path = path.as_ref();
        // `ImageReader::open` surfaces a missing/unreadable file as io::Error,
        // which maps to ImageError::Io via #[from].
        let bytes = std::fs::read(path)?;
        Image::decode_path(path, &bytes)
    }

    /// Decode already-read file `bytes` using the path's EXTENSION to route
    /// format detection — the single place the RAW-vs-generic decision lives.
    ///
    /// RAW takes a dedicated Tier-1 preview-extraction path (SPEC-061,
    /// DEC-055): a RAW container embeds a full-res JPEG preview but is
    /// byte-ambiguous with a plain TIFF, so it is routed by file EXTENSION
    /// (where the `Path` is available) BEFORE the generic byte decoder, which
    /// has no path. RAW-via-stdin (`from_bytes`) is a v1 non-goal.
    ///
    /// Every command that decodes a `Path` (including `info`, which reads the
    /// bytes once for the file size + EXIF) MUST route through here so RAW
    /// extension-routing is not bypassed. [`Image::load`] and `run_info` share
    /// this helper for exactly that reason.
    pub fn decode_path(path: impl AsRef<Path>, bytes: &[u8]) -> Result<Image> {
        if raw::is_raw_extension(path.as_ref()) {
            return raw_preview(bytes);
        }
        Image::from_bytes(bytes)
    }

    /// Detect the format of an in-memory byte slice, decode it, and capture the
    /// raw metadata bundle.
    pub fn from_bytes(bytes: &[u8]) -> Result<Image> {
        let (pixels, source_format, source_container) = decode_with_format(bytes)?;
        let metadata = MetadataBundle::capture(bytes, source_format);
        let truncated_jpeg = source_format == ImageFormat::Jpeg && jpeg_missing_eoi(bytes);
        let animated_input = detect_animated_input(bytes, source_format);
        Ok(Image {
            pixels,
            source_format,
            metadata,
            truncated_jpeg,
            animated_input,
            source_container,
        })
    }

    /// Decode from a seekable reader (the stdin path SPEC-004 will use).
    ///
    /// The reader is drained into memory so format detection can sniff and the
    /// raw metadata can be scanned; a seekable bound is kept for API stability
    /// with the convenience reader path.
    pub fn from_reader<R: Read + Seek>(mut reader: R) -> Result<Image> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Image::from_bytes(&bytes)
    }

    /// The decoded image width in pixels.
    pub fn width(&self) -> u32 {
        self.pixels.width()
    }

    /// The decoded image height in pixels.
    pub fn height(&self) -> u32 {
        self.pixels.height()
    }

    /// The format detected at load.
    pub fn source_format(&self) -> ImageFormat {
        self.source_format
    }

    /// Where `source_format` actually came from (SPEC-115) — whether the raw
    /// bytes on disk are a valid file of that format, or an adopted stand-in
    /// label for a container `::image::ImageFormat` cannot name.
    pub fn source_container(&self) -> SourceContainer {
        self.source_container
    }

    /// The raw metadata bundle captured at load, if any segment was present.
    pub fn metadata(&self) -> Option<&MetadataBundle> {
        self.metadata.as_ref()
    }

    /// Borrow the decoded pixels (for downstream operations, SPEC-003+).
    pub fn pixels(&self) -> &DynamicImage {
        &self.pixels
    }

    /// Whether this image was decoded from a JPEG missing its trailing
    /// end-of-image marker (F1, SPEC-107). The CLI layer checks this right
    /// after load (before any pipeline runs) to print the truncation warning
    /// on `info`/`web`/`convert`/`resize` — see `TRUNCATED_JPEG_WARNING`.
    /// Native-only: no `cli` on `wasm32` to consult it.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn is_truncated_jpeg(&self) -> bool {
        self.truncated_jpeg
    }

    /// Whether this image was decoded from a multi-frame animated GIF/APNG/
    /// WebP (SPEC-119) — the pixel path keeps only the first frame. The CLI
    /// layer checks this right after load (mirroring
    /// [`Image::is_truncated_jpeg`]) to print the flattening warning on
    /// `convert`/`optimize`/`web`/`build` — see [`ANIMATED_INPUT_WARNING`].
    /// Native-only: no `cli` on `wasm32` to consult it.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn is_animated_input(&self) -> bool {
        self.animated_input
    }

    /// Build an `Image` from already-decoded pixels, carrying through the
    /// source format and metadata bundle.
    ///
    /// Used by `Operation` impls (SPEC-003+) to return a transformed image
    /// without re-decoding (decode-once, DEC-002). Operations that have no
    /// access to the originating `Image` value (e.g. because they consumed
    /// it via `with_pixels`) can call this directly. Never itself the product
    /// of a truncated JPEG decode (that flag is only ever set in
    /// [`Image::from_bytes`], on the ORIGINAL loaded bytes — see
    /// [`Image::is_truncated_jpeg`]).
    pub fn from_parts(
        pixels: DynamicImage,
        source_format: ImageFormat,
        metadata: Option<MetadataBundle>,
    ) -> Image {
        Image {
            pixels,
            source_format,
            metadata,
            truncated_jpeg: false,
            // Never itself the product of an animated-input decode, same
            // reasoning as `truncated_jpeg` above: that flag is only ever set
            // in `Image::from_bytes`, on the ORIGINAL loaded bytes.
            animated_input: false,
            source_container: SourceContainer::Native,
        }
    }

    /// Set this `Image`'s [`SourceContainer`] (SPEC-115) — an additive builder
    /// on top of [`Image::from_parts`], whose signature stays unchanged (13
    /// call sites, mostly tests, on a published crate). Used by the three
    /// adopting decoders to record that `source_format` is a stand-in.
    pub fn with_source_container(mut self, container: SourceContainer) -> Image {
        self.source_container = container;
        self
    }

    /// Replace this image's pixels, preserving `source_format` and `metadata`.
    ///
    /// The ergonomic path for `Operation` impls: consume `self` and return a
    /// new `Image` with transformed pixels and the original metadata lane
    /// intact (DEC-002/DEC-003). Avoids cloning the metadata bundle.
    pub fn with_pixels(self, pixels: DynamicImage) -> Image {
        Image {
            pixels,
            source_format: self.source_format,
            metadata: self.metadata,
            truncated_jpeg: self.truncated_jpeg,
            animated_input: self.animated_input,
            source_container: self.source_container,
        }
    }

    /// A read-only inspection snapshot of this image.
    pub fn info(&self) -> ImageInfo {
        let color_type = self.pixels.color();
        let (has_exif, has_icc) = match &self.metadata {
            Some(m) => (m.has_exif(), m.has_icc()),
            None => (false, false),
        };
        ImageInfo {
            width: self.pixels.width(),
            height: self.pixels.height(),
            format: self.source_format,
            color_type,
            bit_depth: color_type_bit_depth(color_type),
            has_alpha: color_type.has_alpha(),
            byte_len: self.pixels.as_bytes().len() as u64,
            has_exif,
            has_icc,
        }
    }
}

/// Read-only inspection of a decoded [`Image`] — the data the future `info`
/// command (STAGE-002) will report. No mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageInfo {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Format detected at load.
    pub format: ImageFormat,
    /// Decoded color type.
    pub color_type: ColorType,
    /// Bits per channel (e.g. 8 for `Rgb8`/`Rgba8`, 16 for `Rgb16`).
    pub bit_depth: u8,
    /// Whether the color type carries an alpha channel.
    pub has_alpha: bool,
    /// Length in bytes of the decoded in-memory pixel buffer (not file size).
    pub byte_len: u64,
    /// Whether a raw ICC profile was captured at load.
    pub has_icc: bool,
    /// Whether a raw EXIF segment was captured at load.
    pub has_exif: bool,
}

/// Raw, **uninterpreted** container metadata segments captured at load
/// (DEC-003).
///
/// The bytes are stored verbatim for the later metadata lane (STAGE-004); this
/// type never parses, validates, or interprets them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetadataBundle {
    /// Raw EXIF segment bytes (e.g. a JPEG APP1 payload from `Exif\0\0`
    /// onward, or a PNG `eXIf` chunk payload). Not parsed.
    pub exif: Option<Vec<u8>>,
    /// Raw ICC profile bytes. Not parsed.
    pub icc: Option<Vec<u8>>,
}

impl MetadataBundle {
    /// Whether a raw EXIF segment was captured.
    pub fn has_exif(&self) -> bool {
        self.exif.is_some()
    }

    /// Whether a raw ICC profile was captured.
    pub fn has_icc(&self) -> bool {
        self.icc.is_some()
    }

    /// Whether this bundle carries no segments at all.
    fn is_empty(&self) -> bool {
        self.exif.is_none() && self.icc.is_none()
    }

    /// Scan the raw container bytes for EXIF/ICC segments (byte-scanning, not
    /// parsing — DEC-003). Returns `None` when no segment is present, so the
    /// "no metadata" case is represented as `Image::metadata() == None`.
    fn capture(bytes: &[u8], format: ImageFormat) -> Option<MetadataBundle> {
        let bundle = match format {
            ImageFormat::Jpeg => MetadataBundle {
                exif: scan_jpeg_exif(bytes),
                icc: scan_jpeg_icc(bytes),
            },
            ImageFormat::Png => MetadataBundle {
                exif: scan_png_chunk(bytes, b"eXIf"),
                icc: scan_png_chunk(bytes, b"iCCP"),
            },
            // Other formats: capture is added with the metadata lane (STAGE-004).
            _ => MetadataBundle::default(),
        };
        if bundle.is_empty() {
            None
        } else {
            Some(bundle)
        }
    }
}

/// Build the production [`::image::Limits`] from the DEC-034 caps:
/// `MAX_IMAGE_DIMENSION` per dimension and `MAX_ALLOC_BYTES` for allocation.
///
/// The struct is `#[non_exhaustive]`, so it must be constructed via
/// `Limits::default()` with field assignment — a struct literal will not compile.
fn decode_limits() -> ::image::Limits {
    let mut limits = ::image::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(MAX_ALLOC_BYTES);
    limits
}

/// Map an [`::image::ImageError`] from the decoder to a typed [`ImageError`].
///
/// A `Limits(_)` variant becomes [`ImageError::LimitsExceeded`]; every other
/// decode failure becomes [`ImageError::Decode`]. This preserves the invariant
/// that limits rejections are matchable independently of ordinary decode errors.
fn map_image_decode_error(e: ::image::ImageError) -> ImageError {
    match e {
        ::image::ImageError::Limits(_) => ImageError::LimitsExceeded(e.to_string()),
        _ => ImageError::Decode(e.to_string()),
    }
}

/// Detect the format of `bytes`, apply `limits` to the reader, and decode.
///
/// This is the test seam: production code calls it with `decode_limits()`; unit
/// tests call it with a deliberately small `Limits` to prove enforcement. The
/// `limits` value is cloned into the reader because [`::image::ImageReader::limits`]
/// takes ownership and `Limits: Clone`.
fn decode_with_limits(
    bytes: &[u8],
    limits: &::image::Limits,
) -> Result<(DynamicImage, ImageFormat, SourceContainer)> {
    // AVIF takes a dedicated pure-Rust decode path (SPEC-058, DEC-053): the
    // `image` crate's own AVIF decoder is dav1d/C and is NOT used, so we detect
    // the container by brand and route it through `re_rav1d` + `avif-parse`,
    // enforcing the same DEC-034 caps via `limits`. Dispatch happens before the
    // generic `ImageReader` path (which cannot decode AVIF in the default build).
    //
    // On wasm32 the decoder is absent (`re_rav1d` does not compile there,
    // SPEC-072/DEC-064) but the SNIFF still runs, so an AVIF input gets a typed,
    // actionable error instead of the generic guesser's "unsupported format" —
    // and never a panic. Restoring AVIF decode on wasm is SPEC-073.
    if sniff::is_avif(bytes) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let pixels = avif::decode_avif(bytes, limits)?;
            return Ok((pixels, ImageFormat::Avif, SourceContainer::Native));
        }
        #[cfg(target_arch = "wasm32")]
        {
            return Err(ImageError::CodecUnavailableOnTarget { codec: "AVIF" });
        }
    }

    // SVG takes a dedicated pure-Rust rasterize path (SPEC-060, DEC-054): SVG is
    // XML/text, so the `image` guesser cannot detect it and the generic
    // `ImageReader` cannot decode it. We content-sniff `<svg`/`<?xml` and
    // rasterize via `resvg`/`usvg`/`tiny-skia`, enforcing the same DEC-034 caps.
    // There is no `ImageFormat::Svg`, so a rasterized SVG reports `Png` (its
    // pixels are now a lossless RGBA raster). Dispatch happens before the
    // generic `ImageReader` path.
    if svg::is_svg(bytes) {
        let pixels = svg::decode_svg(bytes, limits)?;
        return Ok((pixels, ImageFormat::Png, SourceContainer::Svg));
    }

    // HEIC is the ISOBMFF sibling of AVIF, dispatched AFTER it so an AVIF-in-HEIF
    // container (which also carries `mif1`) reaches the pure-Rust AVIF path. Decode
    // lives behind the off-by-default `heic` feature (system libheif, DEC-052/DEC-056),
    // but DETECTION is compiled into both builds: the default binary must answer a
    // `.heic` with a precise "rebuild with --features heic" (exit 4), not a vague
    // "unsupported format". There is no `ImageFormat::Heic`, so a decoded HEIC reports
    // `Png` (the materialized-raster convention, as for SVG).
    if heic::is_heic(bytes) {
        #[cfg(feature = "heic")]
        {
            let pixels = heic::decode_heic(bytes, limits)?;
            return Ok((pixels, ImageFormat::Png, SourceContainer::Heic));
        }
        #[cfg(not(feature = "heic"))]
        {
            return Err(ImageError::CodecNotBuilt {
                codec: "HEIC",
                feature: "heic",
            });
        }
    }

    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(ImageError::Io)?;
    let format = reader.format().ok_or(ImageError::UnsupportedFormat)?;
    reader.limits(limits.clone());

    // Peek the HEADER dimensions and enforce the peak-memory budget (DEC-063)
    // BEFORE `.decode()` allocates: `limits.max_alloc` bounds a single allocation,
    // not the decoder's cumulative working set, so a near-max-dimension JPEG/PNG
    // passes every DEC-034 cap and still peaks at gigabytes (SPEC-069 F-RAW-1).
    //
    // `ImageReader::into_dimensions()` CONSUMES the reader, so the peek runs on a
    // throwaway reader over the same in-memory bytes (a header re-parse — no pixel
    // work, and the bytes are already resident) and the reader built above does the
    // real decode. The alternative (`into_decoder()` → `.dimensions()` → decode via
    // that decoder) trades this cheap re-parse for a decoder we would have to drive
    // by hand; the re-parse is the smaller, clearer change.
    let mut peek = ImageReader::new(Cursor::new(bytes));
    peek.set_format(format);
    peek.limits(limits.clone());
    let (w, h) = peek.into_dimensions().map_err(map_image_decode_error)?;
    check_pixel_budget(w, h)?;

    let pixels = reader.decode().map_err(map_image_decode_error)?;
    Ok((pixels, format, SourceContainer::Native))
}

/// Detect the format of `bytes` and decode it with production resource limits
/// (DEC-034). Reused by every load entry so detection/decoding and limit
/// enforcement are consistent.
fn decode_with_format(bytes: &[u8]) -> Result<(DynamicImage, ImageFormat, SourceContainer)> {
    decode_with_limits(bytes, &decode_limits())
}

/// Extract the embedded full-res JPEG preview from RAW `bytes` as a canonical
/// [`Image`] under the production DEC-034 caps (SPEC-061, DEC-055).
///
/// This is the byte-level entry shared by the [`Image::load`] RAW branch and the
/// `raw_preview` cargo-fuzz target. Routing to it is by file **extension** in
/// [`Image::load`] (RAW containers are byte-ambiguous with plain TIFF), so this
/// is the only public surface for the untrusted-input path — the scan/decode
/// internals stay private to [`raw`].
///
/// The extracted preview *is* a JPEG, so `source_format` is reported as
/// [`ImageFormat::Jpeg`] (the "materialized raster format" convention, like
/// SVG→`Png`). Metadata is **not** captured in v1: the RAW container's EXIF is
/// out of scope and threading the winning preview's own APP1 through the scan is
/// a documented follow-up, so the bundle is `None` (best-effort).
pub fn raw_preview(bytes: &[u8]) -> Result<Image> {
    let pixels = raw::extract_preview(bytes, &decode_limits())?;
    Ok(Image {
        pixels,
        source_format: ImageFormat::Jpeg,
        metadata: None,
        // F1 (SPEC-107) is about a directly-loaded JPEG *file* missing its EOI;
        // a RAW container's embedded preview is a different artifact (already
        // extracted/decoded by `raw::extract_preview` above) and is out of this
        // spec's scope (F4/F5).
        truncated_jpeg: false,
        // The extracted preview is by construction a single JPEG frame — RAW
        // containers do not carry an animated preview (SPEC-119, same
        // reasoning as `truncated_jpeg` immediately above).
        animated_input: false,
        // The bytes on disk are the whole RAW container, not a standalone JPEG
        // file — `source_format` (Jpeg) is the preview's format, an adopted
        // label (SPEC-115).
        source_container: SourceContainer::RawPreview,
    })
}

/// Bits per channel for a [`ColorType`] (e.g. `Rgb8`/`Rgba8` → 8, `Rgb16` →
/// 16). A free fn so it is directly unit-testable.
///
/// `pub(crate)`: `operation` and `sink` both need this (SPEC-121, Call 1 and
/// Call 3) — they widen/narrow by bit depth and warn on a >8-bit source
/// hitting an 8-bit-only encoder, and both are in `crate::image`'s allowed
/// dependency set already, so this is the shared home rather than a second
/// copy.
pub(crate) fn color_type_bit_depth(ct: ColorType) -> u8 {
    // bits_per_pixel / channels = bits per channel.
    let channels = ct.channel_count() as u16;
    if channels == 0 {
        return 0;
    }
    (ct.bits_per_pixel() / channels) as u8
}

/// Scan a JPEG byte stream for the first APP1 (`0xFF 0xE1`) segment whose
/// payload begins with the `Exif\0\0` signature, returning the raw payload
/// bytes (signature included). Byte-scanning, not EXIF parsing (DEC-003).
fn scan_jpeg_exif(bytes: &[u8]) -> Option<Vec<u8>> {
    const EXIF_SIG: &[u8] = b"Exif\0\0";
    scan_jpeg_app_segment(bytes, 0xE1, EXIF_SIG)
}

/// Scan a JPEG byte stream for an APP2 (`0xFF 0xE2`) `ICC_PROFILE\0` segment,
/// returning the raw payload bytes. Best-effort; multi-chunk ICC profiles are
/// not reassembled here (full ICC handling is STAGE-004).
fn scan_jpeg_icc(bytes: &[u8]) -> Option<Vec<u8>> {
    const ICC_SIG: &[u8] = b"ICC_PROFILE\0";
    scan_jpeg_app_segment(bytes, 0xE2, ICC_SIG)
}

/// Walk JPEG marker segments and return the payload of the first APPn segment
/// (`0xFF marker`) whose payload starts with `sig`.
fn scan_jpeg_app_segment(bytes: &[u8], marker: u8, sig: &[u8]) -> Option<Vec<u8>> {
    // JPEG must start with SOI (FF D8).
    if bytes.len() < 2 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return None;
    }
    let mut i = 2;
    while i + 4 <= bytes.len() {
        // Each marker is 0xFF followed by a marker byte.
        if bytes[i] != 0xFF {
            // Not aligned on a marker; bail rather than guess.
            return None;
        }
        let m = bytes[i + 1];
        // Start-of-scan (DA): compressed data follows; stop scanning headers.
        if m == 0xDA {
            return None;
        }
        // Standalone markers (RSTn, SOI, EOI, TEM) have no length field.
        if m == 0xD8 || m == 0xD9 || m == 0x01 || (0xD0..=0xD7).contains(&m) {
            i += 2;
            continue;
        }
        // Segment length is a 2-byte big-endian value that includes itself.
        let seg_len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
        if seg_len < 2 {
            return None;
        }
        let payload_start = i + 4;
        let payload_end = i + 2 + seg_len;
        if payload_end > bytes.len() {
            return None;
        }
        if m == marker {
            let payload = &bytes[payload_start..payload_end];
            if payload.starts_with(sig) {
                return Some(payload.to_vec());
            }
        }
        i = payload_end;
    }
    None
}

/// Scan a PNG byte stream for the first chunk of the given 4-byte type,
/// returning its raw data bytes. Byte-scanning, not parsing (DEC-003).
fn scan_png_chunk(bytes: &[u8], chunk_type: &[u8; 4]) -> Option<Vec<u8>> {
    const PNG_SIG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if !bytes.starts_with(PNG_SIG) {
        return None;
    }
    let mut i = PNG_SIG.len();
    while i + 8 <= bytes.len() {
        let len = u32::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]) as usize;
        let ty = &bytes[i + 4..i + 8];
        let data_start = i + 8;
        let data_end = data_start + len;
        // Chunk has a trailing 4-byte CRC after the data.
        if data_end + 4 > bytes.len() {
            return None;
        }
        if ty == chunk_type {
            return Some(bytes[data_start..data_end].to_vec());
        }
        if ty == b"IEND" {
            return None;
        }
        i = data_end + 4;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::image::{RgbImage, RgbaImage};

    /// Encode a real oversized PNG: `RgbImage::new(70_000, 1)` (~210 KB encoded).
    /// The decoder checks the IHDR dimension before allocating pixel data, so
    /// this fixture is cheap and never OOMs — it just hits the dimension cap.
    fn oversized_png() -> Vec<u8> {
        let img = RgbImage::new(70_000, 1);
        let mut out = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut out, ImageFormat::Png)
            .unwrap();
        out.into_inner()
    }

    /// Encode a solid RGB image to PNG bytes (in-memory fixture).
    fn solid_png(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let img = RgbImage::from_pixel(w, h, ::image::Rgb(rgb));
        let mut out = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut out, ImageFormat::Png)
            .unwrap();
        out.into_inner()
    }

    /// Encode an RGBA image (alpha) to PNG bytes.
    fn rgba_png(w: u32, h: u32) -> Vec<u8> {
        let img = RgbaImage::from_pixel(w, h, ::image::Rgba([10, 20, 30, 128]));
        let mut out = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(img)
            .write_to(&mut out, ImageFormat::Png)
            .unwrap();
        out.into_inner()
    }

    #[test]
    fn info_derives_bit_depth_and_alpha_from_color_type() {
        // Rgb8 → (8, false)
        let png = solid_png(2, 2, [1, 2, 3]);
        let img = Image::from_bytes(&png).unwrap();
        let info = img.info();
        assert_eq!(info.color_type, ColorType::Rgb8);
        assert_eq!(info.bit_depth, 8);
        assert!(!info.has_alpha);

        // Rgba8 → (8, true)
        let png = rgba_png(2, 2);
        let img = Image::from_bytes(&png).unwrap();
        let info = img.info();
        assert_eq!(info.color_type, ColorType::Rgba8);
        assert_eq!(info.bit_depth, 8);
        assert!(info.has_alpha);
    }

    #[test]
    fn color_type_bit_depth_free_fn() {
        assert_eq!(color_type_bit_depth(ColorType::Rgb8), 8);
        assert_eq!(color_type_bit_depth(ColorType::Rgba8), 8);
        assert_eq!(color_type_bit_depth(ColorType::Rgb16), 16);
        assert_eq!(color_type_bit_depth(ColorType::L8), 8);
    }

    #[test]
    fn metadata_bundle_predicates() {
        let bundle = MetadataBundle {
            exif: Some(vec![1]),
            icc: None,
        };
        assert!(bundle.has_exif());
        assert!(!bundle.has_icc());

        let empty = MetadataBundle::default();
        assert!(!empty.has_exif());
        assert!(!empty.has_icc());
        assert!(empty.is_empty());
    }

    #[test]
    fn capture_returns_none_for_plain_png() {
        let png = solid_png(3, 3, [9, 9, 9]);
        assert!(MetadataBundle::capture(&png, ImageFormat::Png).is_none());
    }

    #[test]
    fn accessors_report_dimensions_and_format() {
        let png = solid_png(7, 5, [1, 2, 3]);
        let img = Image::from_bytes(&png).unwrap();
        assert_eq!(img.width(), 7);
        assert_eq!(img.height(), 5);
        assert_eq!(img.source_format(), ImageFormat::Png);
        assert!(img.metadata().is_none());
        assert_eq!(img.pixels().width(), 7);
    }

    #[test]
    fn from_parts_carries_format_and_metadata() {
        // Build a 2×2 RGBA image, wrap it via from_parts, confirm accessors.
        let buf = RgbaImage::from_pixel(2, 2, ::image::Rgba([10, 20, 30, 255]));
        let dyn_img = DynamicImage::ImageRgba8(buf);
        let meta = MetadataBundle {
            exif: Some(vec![1, 2, 3]),
            icc: None,
        };
        let img = Image::from_parts(dyn_img, ImageFormat::Png, Some(meta.clone()));
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
        assert_eq!(img.source_format(), ImageFormat::Png);
        assert_eq!(img.metadata().unwrap().exif, meta.exif);
    }

    #[test]
    fn with_pixels_replaces_pixels_and_preserves_metadata() {
        // Build original image via from_bytes so metadata is captured.
        let png = solid_png(4, 4, [5, 6, 7]);
        let original = Image::from_bytes(&png).unwrap();
        let format = original.source_format();

        // Replace pixels with a smaller 2×2 RGBA buffer.
        let new_buf = RgbaImage::from_pixel(2, 2, ::image::Rgba([200, 100, 50, 128]));
        let new_dyn = DynamicImage::ImageRgba8(new_buf);
        let replaced = original.with_pixels(new_dyn);

        // Dimensions reflect the new pixels; format is preserved.
        assert_eq!(replaced.width(), 2);
        assert_eq!(replaced.height(), 2);
        assert_eq!(replaced.source_format(), format);
    }

    // ── SPEC-033 decode resource limits tests ────────────────────────────────

    /// A 70 000×1 PNG (width > MAX_IMAGE_DIMENSION=65535) must be rejected with
    /// `LimitsExceeded`, not a panic, OOM, or plain `Decode` error.
    #[test]
    fn oversized_dimension_png_is_limits_exceeded() {
        let png = oversized_png();
        let result = Image::from_bytes(&png);
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    /// A normal small image must decode successfully under the production limits —
    /// no regression for realistic images.
    #[test]
    fn normal_image_decodes_under_production_limits() {
        let png = solid_png(64, 64, [128, 64, 32]);
        let result = decode_with_limits(&png, &decode_limits());
        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    /// Passing a tiny dimension cap (`max_image_width = Some(1)`) through the
    /// seam must reject a normal image — proving the limit is enforced, not just
    /// that the constant happens to be large enough.
    #[test]
    fn tiny_dimension_limit_rejects_via_seam() {
        let png = solid_png(4, 4, [1, 2, 3]);
        let mut limits = ::image::Limits::default();
        limits.max_image_width = Some(1);
        let result = decode_with_limits(&png, &limits);
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    /// Passing a tiny allocation cap (`max_alloc = Some(16)`) through the seam
    /// must reject a 64×64 image whose decoded buffer (~12 288 bytes) far exceeds
    /// 16 bytes — proving the allocation/`reserve` path, not only dimensions.
    #[test]
    fn tiny_alloc_limit_rejects_via_seam() {
        let png = solid_png(64, 64, [10, 20, 30]);
        let mut limits = ::image::Limits::default();
        limits.max_alloc = Some(16);
        let result = decode_with_limits(&png, &limits);
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    /// `map_image_decode_error` must map `::image::ImageError::Limits(_)` to
    /// `ImageError::LimitsExceeded`, not `Decode`.
    #[test]
    fn map_limit_error_to_limits_exceeded() {
        use ::image::error::{LimitError, LimitErrorKind};
        let limit_err =
            ::image::ImageError::Limits(LimitError::from_kind(LimitErrorKind::DimensionError));
        let mapped = map_image_decode_error(limit_err);
        assert!(
            matches!(mapped, ImageError::LimitsExceeded(_)),
            "expected LimitsExceeded, got {mapped:?}"
        );
    }

    /// A truncated PNG (valid signature/IHDR, corrupt/missing IDAT) must return
    /// `Err(ImageError::Decode(_))`, NOT `LimitsExceeded`. Limits must not mask
    /// ordinary decode failures.
    #[test]
    fn truncated_png_is_decode_not_limits() {
        // Encode a valid 2×2 PNG then truncate it deeply into the IDAT data.
        let full = solid_png(2, 2, [1, 2, 3]);
        // Keep enough for the PNG signature + IHDR (8 + 25 = 33 bytes), then
        // drop the rest — the decoder sees a recognized PNG with missing IDAT.
        let truncated = &full[..33.min(full.len())];
        let result = Image::from_bytes(truncated);
        assert!(
            matches!(result, Err(ImageError::Decode(_))),
            "expected Decode, got {result:?}"
        );
    }

    /// `Image::from_reader` must also be bounded by the production limits,
    /// because it funnels through `from_bytes` → `decode_with_format`.
    #[test]
    fn from_reader_is_also_limited() {
        let png = oversized_png();
        let result = Image::from_reader(Cursor::new(&png));
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    // ── SPEC-070 peak decode memory: the pixel budget (DEC-063) ──────────────

    /// A JPEG whose SOF0 header **declares** `w`×`h` while the entropy data is
    /// only the original 8×8 image's — the F-RAW-1 pixel-bomb shape (a huge frame
    /// declared in well under a kilobyte). JPEG has no header checksum, so the
    /// dimensions can be patched in place.
    ///
    /// SOF0 layout: `FF C0 [len:2] [precision:1] [height:2] [width:2] …`
    fn jpeg_declaring(w: u16, h: u16) -> Vec<u8> {
        let mut jpg = solid_jpeg(8, 8);
        let sof = jpg
            .windows(2)
            .position(|m| m == [0xFF, 0xC0])
            .expect("encoded JPEG carries an SOF0 marker");
        jpg[sof + 5..sof + 7].copy_from_slice(&h.to_be_bytes());
        jpg[sof + 7..sof + 9].copy_from_slice(&w.to_be_bytes());
        jpg
    }

    /// The pure helper: at the cap is allowed, one pixel over is rejected, and the
    /// largest possible `u32`×`u32` product is rejected without overflowing or
    /// panicking (saturating arithmetic — `u32::MAX²` fits `u64` only by a hair,
    /// so the total-ness must not depend on that).
    #[test]
    fn pixel_budget_helper_math() {
        // Exactly at the cap (8192×8192 = 67_108_864 = MAX_IMAGE_PIXELS): allowed.
        assert!(check_pixel_budget(8192, 8192).is_ok());
        assert_eq!(8192u64 * 8192, MAX_IMAGE_PIXELS);

        // One pixel over the cap: rejected.
        let over = check_pixel_budget(8192, 8193);
        assert!(
            matches!(over, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {over:?}"
        );

        // The F-RAW-1 bomb's declared dims (160 Mpix) are rejected...
        assert!(matches!(
            check_pixel_budget(16384, 9776),
            Err(ImageError::LimitsExceeded(_))
        ));
        // ...while a 24 MP (6000×4000) and a 50 MP (8688×5792) photo are not.
        assert!(check_pixel_budget(6000, 4000).is_ok());
        assert!(check_pixel_budget(8688, 5792).is_ok());

        // Hostile extremes: a typed error, never an overflow panic.
        assert!(matches!(
            check_pixel_budget(u32::MAX, u32::MAX),
            Err(ImageError::LimitsExceeded(_))
        ));
        // A zero dimension is 0 pixels — not this check's business (the decoder
        // rejects it), so it must not error here.
        assert!(check_pixel_budget(0, u32::MAX).is_ok());
    }

    /// The generic path rejects a header that DECLARES more pixels than the budget
    /// **before** decoding: the input is a few hundred bytes, so it could not
    /// possibly hold 160 Mpix — the error can only have come from the pre-decode
    /// peek, not from a decode that allocated the buffer.
    #[test]
    fn declared_oversize_pixels_rejected_before_decode() {
        let bomb = jpeg_declaring(16384, 9776);
        assert!(
            bomb.len() < 1024,
            "the fixture must be tiny to prove the check is pre-decode ({} bytes)",
            bomb.len()
        );
        let result = Image::from_bytes(&bomb);
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    /// The allowed side of the boundary, through the real wiring: a header
    /// declaring EXACTLY the cap (8192×8192 = `MAX_IMAGE_PIXELS`) is let through
    /// and decodes — no off-by-one turning the cap into a rejection at its own
    /// boundary.
    ///
    /// It decoding *at all* from a sub-kilobyte file is the amplification this spec
    /// bounds: `image`'s JPEG decoder pads truncated entropy data out to the full
    /// declared frame. That is exactly why the budget has to be a **declared-dims**
    /// check — the file size tells you nothing about the memory it will cost.
    #[test]
    fn declared_at_cap_pixels_pass_the_budget_check() {
        let at_cap = jpeg_declaring(8192, 8192);
        let img = Image::from_bytes(&at_cap).expect("at-cap dims must not be rejected");
        assert_eq!(img.width() as u64 * img.height() as u64, MAX_IMAGE_PIXELS);
    }

    /// A legitimate large photo well inside the budget still decodes to the right
    /// dimensions and pixels — the cap is a bomb filter, not a low ceiling
    /// (`ergonomic-defaults`). 4000×3000 = 12 Mpix of real, allocated pixels.
    #[test]
    fn legitimate_large_image_within_budget_decodes() {
        let png = solid_png(4000, 3000, [7, 8, 9]);
        let img = Image::from_bytes(&png).expect("a 12 MP image must still decode");
        assert_eq!(img.width(), 4000);
        assert_eq!(img.height(), 3000);
        assert_eq!(img.pixels().to_rgb8().get_pixel(3999, 2999).0, [7, 8, 9]);
    }

    // ── SPEC-058 AVIF decode (default, pure-Rust) ─────────────────────────────

    /// The committed 16×16 AVIF fixture (regen: `cargo run --example
    /// gen_avif_fixture --features avif`).
    const AVIF_FIXTURE: &[u8] = include_bytes!("../../tests/fixtures/avif/solid_16x16.avif");

    /// The DEFAULT build decodes a real `.avif` to the canonical `Image` with
    /// correct dimensions and `source_format == Avif` — proving the pure-Rust
    /// path is active (no `avif`/dav1d feature).
    #[test]
    fn avif_decodes_to_expected_dimensions() {
        let img = Image::from_bytes(AVIF_FIXTURE).expect("decode avif fixture");
        assert_eq!(img.width(), 16);
        assert_eq!(img.height(), 16);
        assert_eq!(img.source_format(), ImageFormat::Avif);
    }

    /// A truncated AVIF is a typed decode error, never a panic/`unwrap`.
    #[test]
    fn corrupt_avif_is_decode_error_not_panic() {
        let truncated = &AVIF_FIXTURE[..32.min(AVIF_FIXTURE.len())];
        let result = Image::from_bytes(truncated);
        assert!(
            matches!(
                result,
                Err(ImageError::Decode(_) | ImageError::UnsupportedFormat)
            ),
            "expected Decode/UnsupportedFormat, got {result:?}"
        );
    }

    /// AVIF decode routes through the DEC-034 caps: a dimension cap below the
    /// fixture yields `LimitsExceeded`, not an OOM or panic.
    #[test]
    fn avif_respects_decode_dimension_cap() {
        let mut limits = ::image::Limits::default();
        limits.max_image_width = Some(8);
        limits.max_image_height = Some(8);
        let result = decode_with_limits(AVIF_FIXTURE, &limits);
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    // ── SPEC-060 SVG rasterize (default, pure-Rust) ───────────────────────────

    /// The DEFAULT build rasterizes a real `.svg` to the canonical `Image` at its
    /// intrinsic `width`/`height`, reporting `source_format == Png` (no
    /// `ImageFormat::Svg` exists).
    #[test]
    fn svg_decodes_to_intrinsic_dimensions() {
        let svg = b"<svg xmlns='http://www.w3.org/2000/svg' width='40' height='30'></svg>";
        let img = Image::from_bytes(svg).expect("rasterize svg");
        assert_eq!(img.width(), 40);
        assert_eq!(img.height(), 30);
        assert_eq!(img.source_format(), ImageFormat::Png);
    }

    /// With no `width`/`height`, the intrinsic size comes from the `viewBox`.
    #[test]
    fn svg_uses_viewbox_when_no_width_height() {
        let svg = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 50'></svg>";
        let img = Image::from_bytes(svg).expect("rasterize svg");
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 50);
    }

    /// An SVG declaring a dimension above `MAX_IMAGE_DIMENSION` is rejected with
    /// `LimitsExceeded` via the production caps — before any raster allocation.
    #[test]
    fn oversize_svg_is_limits_exceeded() {
        let svg = b"<svg xmlns='http://www.w3.org/2000/svg' width='70000' height='10'></svg>";
        let result = Image::from_bytes(svg);
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    /// A malformed (unclosed) SVG is a typed decode error, never a panic.
    #[test]
    fn malformed_svg_is_decode_error_not_panic() {
        let svg = b"<svg xmlns='http://www.w3.org/2000/svg'><rect";
        let result = Image::from_bytes(svg);
        assert!(
            matches!(result, Err(ImageError::Decode(_))),
            "expected Decode, got {result:?}"
        );
    }

    /// An SVG referencing an external file rasterizes WITHOUT reading it: the
    /// href resolver refuses the reference (transparent region), the local file
    /// is never opened, and decode still returns `Ok` with the intrinsic dims.
    #[test]
    fn svg_external_file_ref_is_ignored() {
        let svg = b"<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' width='10' height='10'>\
            <rect width='10' height='10' fill='#00ff00'/>\
            <image href='file:///etc/hostname' xlink:href='file:///etc/hostname' x='0' y='0' width='10' height='10'/>\
            </svg>";
        let img = Image::from_bytes(svg).expect("rasterize svg with refused external ref");
        assert_eq!(img.width(), 10);
        assert_eq!(img.height(), 10);
        // The external image resolved to nothing, so the green background shows
        // through — proving no local-file read replaced the pixels.
        let rgba = img.pixels().to_rgba8();
        let px = rgba.get_pixel(5, 5);
        assert_eq!(px.0[1], 255, "expected green background, got {:?}", px.0);
        assert_eq!(px.0[0], 0, "expected green background, got {:?}", px.0);
    }

    // ── SPEC-061 RAW embedded-preview extraction (default) ────────────────────

    /// Encode a solid-color JPEG in memory (the RAW-fixture primitive).
    fn solid_jpeg(w: u32, h: u32) -> Vec<u8> {
        let img = RgbImage::from_pixel(w, h, ::image::Rgb([200, 150, 100]));
        let mut out = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut out, ImageFormat::Jpeg)
            .unwrap();
        out.into_inner()
    }

    /// Assemble a synthetic RAW blob: `[II*\0 TIFF hdr][thumb jpeg][junk][preview jpeg]`.
    fn synthetic_raw(thumb: (u32, u32), preview: (u32, u32)) -> Vec<u8> {
        let mut b = vec![0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00];
        b.extend_from_slice(&solid_jpeg(thumb.0, thumb.1));
        b.extend_from_slice(&[0x00, 0x11, 0x22, 0x33]);
        b.extend_from_slice(&solid_jpeg(preview.0, preview.1));
        b
    }

    /// A `.nef` on disk routes to preview extraction: `Image::load` returns the
    /// larger embedded JPEG (the full preview) as the canonical `Image` with
    /// `source_format == Jpeg`.
    #[test]
    fn raw_extension_routes_to_preview_extraction() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("photo.nef");
        std::fs::write(&path, synthetic_raw((16, 12), (64, 48))).unwrap();

        let img = Image::load(&path).expect("load raw preview");
        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 48);
        assert_eq!(img.source_format(), ImageFormat::Jpeg);
    }

    /// A non-RAW extension still loads via the generic decoder — the RAW branch
    /// is extension-gated and does not affect ordinary inputs.
    #[test]
    fn non_raw_extension_still_uses_generic_decoder() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.png");
        std::fs::write(&path, solid_png(5, 7, [1, 2, 3])).unwrap();

        let img = Image::load(&path).expect("load png");
        assert_eq!(img.width(), 5);
        assert_eq!(img.height(), 7);
        assert_eq!(img.source_format(), ImageFormat::Png);
    }

    /// The public `raw_preview` byte entry (also the fuzz surface) extracts the
    /// largest embedded JPEG regardless of the surrounding container bytes.
    #[test]
    fn raw_preview_entry_extracts_largest() {
        let blob = synthetic_raw((16, 12), (48, 32));
        let img = raw_preview(&blob).expect("extract preview");
        assert_eq!(img.width(), 48);
        assert_eq!(img.height(), 32);
        assert_eq!(img.source_format(), ImageFormat::Jpeg);
    }

    // ── SPEC-062 HEIC decode (off-by-default `heic` feature) ──────────────────

    /// The committed 64×48 solid HEIC fixture (regen: `sips -s format heic
    /// solid.png --out tests/fixtures/heic/solid_64x48.heic`).
    const HEIC_FIXTURE: &[u8] = include_bytes!("../../tests/fixtures/heic/solid_64x48.heic");

    /// The DEFAULT build detects `.heic` and returns the precise `CodecNotBuilt`
    /// (→ exit 4), not `UnsupportedFormat`, `Decode`, or a panic (DEC-052).
    #[cfg(not(feature = "heic"))]
    #[test]
    fn heic_without_feature_is_codec_not_built() {
        let result = Image::from_bytes(HEIC_FIXTURE);
        assert!(
            matches!(
                result,
                Err(ImageError::CodecNotBuilt {
                    codec: "HEIC",
                    feature: "heic"
                })
            ),
            "expected CodecNotBuilt, got {result:?}"
        );
    }

    /// Under `--features heic`, the fixture decodes through the canonical `Image`
    /// with `source_format == Png` (no `ImageFormat::Heic` exists).
    #[cfg(feature = "heic")]
    #[test]
    fn heic_decodes_to_expected_dimensions() {
        let img = Image::from_bytes(HEIC_FIXTURE).expect("decode heic fixture");
        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 48);
        assert_eq!(img.source_format(), ImageFormat::Png);
    }

    /// HEIC decode routes through the DEC-034 caps in BOTH the seam and production.
    #[cfg(feature = "heic")]
    #[test]
    fn heic_respects_decode_dimension_cap() {
        let mut limits = ::image::Limits::default();
        limits.max_image_width = Some(8);
        limits.max_image_height = Some(8);
        let result = decode_with_limits(HEIC_FIXTURE, &limits);
        assert!(
            matches!(result, Err(ImageError::LimitsExceeded(_))),
            "expected LimitsExceeded, got {result:?}"
        );
    }

    /// AVIF is dispatched BEFORE HEIC, so the AVIF fixture still decodes as AVIF
    /// in both builds — HEIC brand detection must not steal `mif1`-carrying AVIF.
    #[test]
    fn avif_is_not_mis_detected_as_heic() {
        assert!(!heic::is_heic(AVIF_FIXTURE));
        let img = Image::from_bytes(AVIF_FIXTURE).expect("decode avif fixture");
        assert_eq!(img.source_format(), ImageFormat::Avif);
    }

    /// A RAW-extension file with no decodable embedded JPEG is a typed error,
    /// never a panic.
    #[test]
    fn raw_with_no_preview_is_typed_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.cr3");
        std::fs::write(&path, b"ftypcrx not really a jpeg in here").unwrap();

        let result = Image::load(&path);
        assert!(
            matches!(
                result,
                Err(ImageError::Decode(_) | ImageError::UnsupportedFormat)
            ),
            "expected typed error, got {result:?}"
        );
    }

    // ── F1 truncated-JPEG warning (SPEC-107, DEC-085) ──────────────────────────

    /// The container-level EOI check itself: a well-formed JPEG ends `FF D9`;
    /// cutting even the last byte off must flip the verdict. Also a
    /// `no-unwrap-on-recoverable-paths` proof: an empty/1-byte slice must not
    /// panic, and correctly reads as "missing".
    #[test]
    fn jpeg_missing_eoi_detects_the_trailing_marker() {
        let jpeg = solid_jpeg(8, 8);
        assert!(jpeg.ends_with(&[0xFF, 0xD9]), "test fixture sanity check");
        assert!(
            !jpeg_missing_eoi(&jpeg),
            "a well-formed JPEG must not read as truncated"
        );

        let cut = &jpeg[..jpeg.len() - 1];
        assert!(
            jpeg_missing_eoi(cut),
            "removing the last byte must read as truncated"
        );

        assert!(
            jpeg_missing_eoi(&[]),
            "an empty slice must not panic and reads as missing"
        );
        assert!(
            jpeg_missing_eoi(&[0xD9]),
            "a 1-byte slice must not panic and reads as missing"
        );
    }

    /// End-to-end through `Image::from_bytes`: a well-formed JPEG reports
    /// `is_truncated_jpeg() == false`; truncating it (short enough to lose the
    /// EOI, long enough to still decode a full frame) flips it to `true`.
    #[test]
    fn from_bytes_flags_a_truncated_jpeg_but_not_a_whole_one() {
        let jpeg = solid_jpeg(64, 64);
        let whole = Image::from_bytes(&jpeg).expect("decode whole jpeg");
        assert!(!whole.is_truncated_jpeg());

        // A solid-color JPEG compresses to very little entropy data, so cut
        // deep enough to lose the EOI but keep enough of the header + scan
        // for the decoder to still return a frame.
        let cut = jpeg.len() * 9 / 10;
        let truncated = Image::from_bytes(&jpeg[..cut]).expect("decode truncated jpeg");
        assert!(truncated.is_truncated_jpeg());
    }

    /// A non-JPEG whose bytes happen to end `FF D9` must NOT be flagged — the
    /// check only applies once the format is already known to be JPEG.
    #[test]
    fn non_jpeg_is_never_flagged_truncated() {
        let mut png = solid_png(8, 8, [1, 2, 3]);
        png.extend_from_slice(&[0xFF, 0xD9]); // trailing junk, coincidentally EOI-shaped
        let img = Image::from_bytes(&png).expect("PNG decoder ignores trailing junk");
        assert_eq!(img.source_format(), ImageFormat::Png);
        assert!(!img.is_truncated_jpeg());
    }

    // ── SPEC-119 animated-input flag ────────────────────────────────────────

    /// A 2-frame GIF, built with `image`'s own `GifEncoder` (the fixture
    /// style `src/lint/rules.rs` already uses).
    fn animated_gif(w: u32, h: u32) -> Vec<u8> {
        use ::image::codecs::gif::GifEncoder;
        use ::image::Frame;
        let mut buf = Vec::new();
        {
            let mut enc = GifEncoder::new(&mut buf);
            let f1 = Frame::new(RgbaImage::from_pixel(w, h, ::image::Rgba([255, 0, 0, 255])));
            let f2 = Frame::new(RgbaImage::from_pixel(w, h, ::image::Rgba([0, 255, 0, 255])));
            enc.encode_frames(vec![f1, f2]).unwrap();
        }
        buf
    }

    fn static_gif(w: u32, h: u32) -> Vec<u8> {
        let img = RgbImage::from_pixel(w, h, ::image::Rgb([9, 9, 9]));
        let mut out = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut out, ImageFormat::Gif)
            .unwrap();
        out.into_inner()
    }

    /// End-to-end through `Image::from_bytes` (AC-9's GIF claim, unit level):
    /// an animated GIF sets `is_animated_input()`; a static one does not —
    /// the did-not-break-it control, without which "always flag" would pass
    /// the positive half and ruin the field
    /// [[a-harness-that-exercises-nothing-reports-green]].
    #[test]
    fn from_bytes_flags_an_animated_gif_but_not_a_static_one() {
        let animated = Image::from_bytes(&animated_gif(4, 4)).expect("decode animated gif");
        assert!(animated.is_animated_input());

        let still = Image::from_bytes(&static_gif(4, 4)).expect("decode static gif");
        assert!(!still.is_animated_input());
    }

    /// A JPEG (no `AnimationDecoder` impl at all) must never be flagged,
    /// regardless of content — `detect_animated_input`'s `_ => false` arm.
    #[test]
    fn jpeg_is_never_flagged_animated() {
        let jpeg = solid_jpeg(8, 8);
        let img = Image::from_bytes(&jpeg).expect("decode jpeg");
        assert!(!img.is_animated_input());
    }

    /// A pipeline operation's output (`with_pixels`) carries the SOURCE
    /// image's `animated_input` flag through unchanged — the flag describes
    /// what was decoded, not what a later transform produced.
    #[test]
    fn with_pixels_preserves_the_animated_flag() {
        let animated = Image::from_bytes(&animated_gif(4, 4)).expect("decode animated gif");
        assert!(animated.is_animated_input());
        let replaced = animated
            .clone()
            .with_pixels(RgbImage::from_pixel(2, 2, ::image::Rgb([1, 1, 1])).into());
        assert!(replaced.is_animated_input());
    }

    /// `from_parts` is never itself the product of a decode (an `Operation`
    /// output), so it must never carry the flag, even when the pixels happen
    /// to have come from an animated source upstream.
    #[test]
    fn from_parts_is_never_flagged_animated() {
        let built = Image::from_parts(
            RgbImage::from_pixel(2, 2, ::image::Rgb([1, 1, 1])).into(),
            ImageFormat::Gif,
            None,
        );
        assert!(!built.is_animated_input());
    }
}
