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
        Image::from_bytes(&bytes)
    }

    /// Detect the format of an in-memory byte slice, decode it, and capture the
    /// raw metadata bundle.
    pub fn from_bytes(bytes: &[u8]) -> Result<Image> {
        let (pixels, source_format) = decode_with_format(bytes)?;
        let metadata = MetadataBundle::capture(bytes, source_format);
        Ok(Image {
            pixels,
            source_format,
            metadata,
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

    /// The raw metadata bundle captured at load, if any segment was present.
    pub fn metadata(&self) -> Option<&MetadataBundle> {
        self.metadata.as_ref()
    }

    /// Borrow the decoded pixels (for downstream operations, SPEC-003+).
    pub fn pixels(&self) -> &DynamicImage {
        &self.pixels
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

/// Detect the format of `bytes` and decode it, mapping failures to typed
/// errors. Reused by every load entry so detection/decoding is consistent.
fn decode_with_format(bytes: &[u8]) -> Result<(DynamicImage, ImageFormat)> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(ImageError::Io)?;
    let format = reader.format().ok_or(ImageError::UnsupportedFormat)?;
    let pixels = reader
        .decode()
        .map_err(|e| ImageError::Decode(e.to_string()))?;
    Ok((pixels, format))
}

/// Bits per channel for a [`ColorType`] (e.g. `Rgb8`/`Rgba8` → 8, `Rgb16` →
/// 16). A free fn so it is directly unit-testable.
fn color_type_bit_depth(ct: ColorType) -> u8 {
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
}
