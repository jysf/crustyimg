//! Container-lane metadata edits (SPEC-026, DEC-003).
//!
//! This is the **container lane**: it edits container-level metadata
//! (EXIF/ICC/XMP/IPTC/comments) by operating on the **raw container bytes** and
//! never re-decodes or re-encodes pixels. The compressed scan (JPEG) / `IDAT`
//! (PNG) is carried through verbatim, so decoding the output yields pixels that
//! are byte-identical to decoding the input — the constraint
//! `metadata-not-via-pixel-encode` made concrete.
//!
//! Division of labor (DEC-003 / DEC-046):
//! - **`strip_all`** removes *all* user metadata at the segment/chunk level via
//!   [`img_parts`] (JPEG APP1..APP15 + COM; PNG `eXIf`/`iCCP`/`tEXt`/…).
//! - **`clean_gps`** removes *only* the GPS IFD at the tag level via the
//!   in-house [`tiff`] writer, preserving every other tag (orientation,
//!   copyright, …).
//!
//! The format is sniffed with [`image::guess_format`] (a magic-byte check, no
//! decode). Only JPEG and PNG are supported in v1; any other format is a
//! [`MetadataError::UnsupportedFormat`]. The read side stays `kamadak-exif`
//! elsewhere; this module is the write half.

mod tiff;

use ::image::ImageFormat;
use img_parts::jpeg::Jpeg;
use img_parts::png::Png;
use img_parts::{Bytes, ImageEXIF, ImageICC};

// ── Errors ────────────────────────────────────────────────────────────────────

/// A container-lane metadata error (DEC-007; typed, no `unwrap`/`panic!` on
/// recoverable paths). The binary (`src/cli`) maps these to exit codes:
/// [`MetadataError::UnsupportedFormat`] → 4; [`MetadataError::Container`] /
/// [`MetadataError::Exif`] → 1.
#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    /// The detected format is not one the metadata lane supports in v1
    /// (JPEG + PNG only). Carries a human label for the message.
    #[error("metadata lane does not support {0} yet")]
    UnsupportedFormat(String),

    /// A segment/chunk-level parse or rewrite failure (`img-parts`).
    #[error("container metadata edit failed: {0}")]
    Container(String),

    /// A tag-level EXIF parse or rewrite failure (the in-house [`tiff`]
    /// reader/writer), excluding the benign "no EXIF" case which
    /// [`clean_gps`] treats as a no-op.
    #[error("EXIF edit failed: {0}")]
    Exif(String),
}

// ── Format sniff ──────────────────────────────────────────────────────────────

/// The two container formats the v1 metadata lane handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lane {
    Jpeg,
    Png,
}

/// Sniff the container format from magic bytes (NO pixel decode) and map it to
/// the supported [`Lane`], or [`MetadataError::UnsupportedFormat`] for anything
/// outside JPEG/PNG (or undeterminable bytes).
fn sniff(bytes: &[u8]) -> Result<Lane, MetadataError> {
    match ::image::guess_format(bytes) {
        Ok(ImageFormat::Jpeg) => Ok(Lane::Jpeg),
        Ok(ImageFormat::Png) => Ok(Lane::Png),
        Ok(other) => Err(MetadataError::UnsupportedFormat(format!("{other:?}"))),
        Err(_) => Err(MetadataError::UnsupportedFormat("unknown".to_owned())),
    }
}

// ── strip_all ─────────────────────────────────────────────────────────────────

/// PNG chunk types that carry user/ancillary metadata and are removed by
/// [`strip_all`]. Critical/render chunks (`IHDR`, `PLTE`, `IDAT`, `IEND`,
/// `tRNS`, `gAMA`, `cHRM`, `sRGB`, `bKGD`, `pHYs`) are intentionally kept.
const PNG_METADATA_CHUNKS: [[u8; 4]; 6] =
    [*b"eXIf", *b"iCCP", *b"tEXt", *b"zTXt", *b"iTXt", *b"tIME"];

/// Remove **all** container metadata, preserving pixels exactly.
///
/// - **JPEG:** drop APP1..APP15 (`0xE1..=0xEF` — EXIF/XMP/ICC/…) and COM
///   (`0xFE`). APP0/JFIF is structural and kept.
/// - **PNG:** drop the [`PNG_METADATA_CHUNKS`]; keep critical/render chunks.
///
/// Returns the rewritten container bytes. The compressed image data is carried
/// verbatim — no pixel re-encode (`metadata-not-via-pixel-encode`).
pub fn strip_all(bytes: &[u8]) -> Result<Vec<u8>, MetadataError> {
    match sniff(bytes)? {
        Lane::Jpeg => {
            let mut jpeg = Jpeg::from_bytes(Bytes::from(bytes.to_vec()))
                .map_err(|e| MetadataError::Container(e.to_string()))?;
            // APP1 (EXIF/XMP) .. APP15, plus COM (0xFE). APP0/JFIF kept.
            for marker in 0xE1u8..=0xEF {
                jpeg.remove_segments_by_marker(marker);
            }
            jpeg.remove_segments_by_marker(0xFE);
            let mut out = Vec::new();
            jpeg.encoder()
                .write_to(&mut out)
                .map_err(|e| MetadataError::Container(e.to_string()))?;
            Ok(out)
        }
        Lane::Png => {
            let mut png = Png::from_bytes(Bytes::from(bytes.to_vec()))
                .map_err(|e| MetadataError::Container(e.to_string()))?;
            for kind in PNG_METADATA_CHUNKS {
                png.remove_chunks_by_type(kind);
            }
            let mut out = Vec::new();
            png.encoder()
                .write_to(&mut out)
                .map_err(|e| MetadataError::Container(e.to_string()))?;
            Ok(out)
        }
    }
}

// ── clean_gps ─────────────────────────────────────────────────────────────────

/// Read the current TIFF/EXIF block for `lane` out of `bytes` via
/// `img-parts`. Returns `None` for the benign "no EXIF at all" case, which
/// callers treat as a no-op / fresh-create fallback rather than an error.
fn read_exif_block(lane: Lane, bytes: &[u8]) -> Result<Option<Bytes>, MetadataError> {
    match lane {
        Lane::Jpeg => {
            let jpeg = Jpeg::from_bytes(Bytes::from(bytes.to_vec()))
                .map_err(|e| MetadataError::Container(e.to_string()))?;
            Ok(jpeg.exif())
        }
        Lane::Png => {
            let png = Png::from_bytes(Bytes::from(bytes.to_vec()))
                .map_err(|e| MetadataError::Container(e.to_string()))?;
            Ok(png.exif())
        }
    }
}

/// Re-embed `tiff_bytes` as the EXIF block for `lane`, rewriting `bytes`'s
/// container (JPEG APP1 / PNG `eXIf`) while leaving the pixels untouched.
fn write_exif_block(
    lane: Lane,
    bytes: &[u8],
    tiff_bytes: Vec<u8>,
) -> Result<Vec<u8>, MetadataError> {
    let mut out = Vec::new();
    match lane {
        Lane::Jpeg => {
            let mut jpeg = Jpeg::from_bytes(Bytes::from(bytes.to_vec()))
                .map_err(|e| MetadataError::Container(e.to_string()))?;
            jpeg.set_exif(Some(Bytes::from(tiff_bytes)));
            jpeg.encoder()
                .write_to(&mut out)
                .map_err(|e| MetadataError::Container(e.to_string()))?;
        }
        Lane::Png => {
            let mut png = Png::from_bytes(Bytes::from(bytes.to_vec()))
                .map_err(|e| MetadataError::Container(e.to_string()))?;
            png.set_exif(Some(Bytes::from(tiff_bytes)));
            png.encoder()
                .write_to(&mut out)
                .map_err(|e| MetadataError::Container(e.to_string()))?;
        }
    }
    Ok(out)
}

/// Remove **only** GPS/location metadata, preserving every other tag and the
/// pixels exactly.
///
/// Parses the TIFF/EXIF block with the in-house [`tiff`] reader, drops the
/// IFD0 GPS pointer entry (orphaning its sub-IFD), and re-embeds the result.
/// A file with **no EXIF** is a byte-faithful no-op (DEC-029 edge case,
/// preserved by DEC-046).
pub fn clean_gps(bytes: &[u8]) -> Result<Vec<u8>, MetadataError> {
    let lane = sniff(bytes)?;

    let Some(exif) = read_exif_block(lane, bytes)? else {
        return Ok(bytes.to_vec());
    };

    let mut parsed = tiff::parse(&exif).map_err(|e| MetadataError::Exif(e.to_string()))?;
    tiff::remove_gps(&mut parsed.ifd0);
    let out_tiff = tiff::serialize(&parsed);

    write_exif_block(lane, bytes, out_tiff)
}

// ── set_tags ──────────────────────────────────────────────────────────────────

/// The attribution tags `set` can write (SPEC-027). Each `Some` is written into
/// the container EXIF; each `None` is left untouched. All three are STRING tags
/// in the generic/IFD0 group (`Artist`, `Copyright`, `ImageDescription`).
#[derive(Debug, Clone, Default)]
pub struct TagSet {
    pub artist: Option<String>,
    pub copyright: Option<String>,
    pub description: Option<String>,
}

/// Write the given attribution tags into the container EXIF, **overwriting** any
/// existing value of the same tag and **preserving** every other tag, segment,
/// and the pixels exactly (no re-encode — `metadata-not-via-pixel-encode`).
///
/// Loads the existing TIFF/EXIF block first so other tags survive; a file
/// with **no EXIF** falls back to a fresh minimal TIFF carrying just the
/// given tags (DEC-029 edge case, preserved by DEC-046). Only JPEG + PNG are
/// supported in v1; any other format is a [`MetadataError::UnsupportedFormat`].
pub fn set_tags(bytes: &[u8], tags: &TagSet) -> Result<Vec<u8>, MetadataError> {
    let lane = sniff(bytes)?;

    // Load-then-set preserves existing tags; no existing EXIF falls back to
    // a fresh minimal TIFF (probe-verified, DEC-029/DEC-046).
    let mut parsed = match read_exif_block(lane, bytes)? {
        Some(exif) => tiff::parse(&exif).map_err(|e| MetadataError::Exif(e.to_string()))?,
        None => tiff::minimal(),
    };

    if let Some(ref description) = tags.description {
        tiff::set_ascii_tag(&mut parsed.ifd0, tiff::TAG_IMAGE_DESCRIPTION, description);
    }
    if let Some(ref artist) = tags.artist {
        tiff::set_ascii_tag(&mut parsed.ifd0, tiff::TAG_ARTIST, artist);
    }
    if let Some(ref copyright) = tags.copyright {
        tiff::set_ascii_tag(&mut parsed.ifd0, tiff::TAG_COPYRIGHT, copyright);
    }

    let out_tiff = tiff::serialize(&parsed);
    write_exif_block(lane, bytes, out_tiff)
}

// ── copy_metadata ─────────────────────────────────────────────────────────────

/// Copy the container **EXIF (APP1) + ICC (APP2)** from `from` (the metadata
/// donor) onto `to` (the pixel recipient), returning `to`'s rewritten container
/// bytes with its pixels **preserved exactly** (no re-encode —
/// `metadata-not-via-pixel-encode`). DST's prior EXIF/ICC are **replaced** by
/// SRC's; if SRC has none, DST's are cleared (`None` flows straight through the
/// `img-parts` `ImageEXIF`/`ImageICC` traits).
///
/// **JPEG only in v1 (DEC-030):** both `from` and `to` must sniff as JPEG, else
/// [`MetadataError::UnsupportedFormat`]. PNG `copy-metadata` is deferred because
/// `little_exif` writes PNG EXIF as a `zTXt` "Raw profile type exif" chunk while
/// `img-parts` uses the native `eXIf` chunk, so the two can't interoperate.
pub fn copy_metadata(from: &[u8], to: &[u8]) -> Result<Vec<u8>, MetadataError> {
    // Both inputs must be JPEG (DEC-030). A clear message names the limitation.
    if sniff(from)? != Lane::Jpeg || sniff(to)? != Lane::Jpeg {
        return Err(MetadataError::UnsupportedFormat(
            "copy-metadata supports JPEG only in v1".to_owned(),
        ));
    }

    let src = Jpeg::from_bytes(Bytes::from(from.to_vec()))
        .map_err(|e| MetadataError::Container(e.to_string()))?;
    let mut dst = Jpeg::from_bytes(Bytes::from(to.to_vec()))
        .map_err(|e| MetadataError::Container(e.to_string()))?;

    // Graft SRC's EXIF + ICC onto DST (Option<Bytes> flows through; None clears).
    dst.set_exif(src.exif());
    dst.set_icc_profile(src.icc_profile());

    let mut out = Vec::new();
    dst.encoder()
        .write_to(&mut out)
        .map_err(|e| MetadataError::Container(e.to_string()))?;
    Ok(out)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::tiff::{self, Entry, Ifd};
    use super::*;
    use ::image::{ImageFormat, RgbImage};
    use std::io::Cursor;

    /// A small deterministic 16×16 RGB image encoded to `format` bytes (no
    /// metadata). The gradient gives a non-trivial pixel buffer for the
    /// decode-equality assertions.
    fn base_image(format: ImageFormat) -> Vec<u8> {
        let mut img = RgbImage::new(16, 16);
        for (x, y, px) in img.enumerate_pixels_mut() {
            *px = ::image::Rgb([(x * 16) as u8, (y * 16) as u8, ((x + y) * 8) as u8]);
        }
        let mut buf = Cursor::new(Vec::new());
        ::image::DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, format)
            .expect("encode base image");
        buf.into_inner()
    }

    // ── Hand-assembled TIFF seeding (no little_exif — the crate is gone,
    //    DEC-046). Tests build a `tiff::Ifd` tree directly and embed it via
    //    `img-parts` `set_exif`, mirroring exactly what `set_tags`/`clean_gps`
    //    do internally, but as an independent seeding path. ──────────────────

    // IFD0 / generic tags used across fixtures.
    const TAG_ORIENTATION: u16 = 0x0112;
    const TAG_COPYRIGHT: u16 = tiff::TAG_COPYRIGHT;
    // ExifIFD sub-tag used to lock sub-IFD preservation.
    const TAG_EXPOSURE_TIME: u16 = 0x829A;
    // GPS sub-tags.
    const TAG_GPS_LAT_REF: u16 = 0x0001;
    const TAG_GPS_LON_REF: u16 = 0x0003;
    // IFD1 thumbnail location tags.
    const TAG_THUMB_OFFSET: u16 = 0x0201;
    const TAG_THUMB_LENGTH: u16 = 0x0202;

    /// A SHORT (type 3, count 1) entry inlined in the 4-byte value slot.
    fn short_entry(tag: u16, v: u16) -> Entry {
        let mut value = vec![0u8; 2];
        value[0..2].copy_from_slice(&v.to_le_bytes());
        Entry {
            tag,
            ty: 3,
            count: 1,
            value,
            sub: None,
        }
    }

    /// A RATIONAL (type 5, count 1) entry — always out-of-line (8 bytes).
    fn rational_entry(tag: u16, num: u32, den: u32) -> Entry {
        let mut value = Vec::with_capacity(8);
        value.extend_from_slice(&num.to_le_bytes());
        value.extend_from_slice(&den.to_le_bytes());
        Entry {
            tag,
            ty: 5,
            count: 1,
            value,
            sub: None,
        }
    }

    /// An ASCII (type 2) entry: UTF-8 bytes + trailing NUL, per TIFF 6.0.
    fn ascii_entry(tag: u16, text: &str) -> Entry {
        let mut value = text.as_bytes().to_vec();
        value.push(0);
        let count = value.len() as u32;
        Entry {
            tag,
            ty: 2,
            count,
            value,
            sub: None,
        }
    }

    /// A pointer entry (ExifIFD/GPS/Interop) carrying a parsed sub-`Ifd`. The
    /// raw `value` is a placeholder — the serializer recomputes the offset
    /// from `sub`.
    fn pointer_entry(tag: u16, sub: Ifd) -> Entry {
        Entry {
            tag,
            ty: 4,
            count: 1,
            value: vec![0u8; 4],
            sub: Some(Box::new(sub)),
        }
    }

    /// Build + serialize a TIFF block whose IFD0 is `ifd0`, and embed it as
    /// `format`'s EXIF via `img-parts` (JPEG APP1 / PNG `eXIf`) on top of a
    /// fresh [`base_image`]. This is the crate-internal equivalent of what
    /// `little_exif` used to do for the test fixtures.
    fn image_with_tiff(format: ImageFormat, ifd0: Ifd) -> Vec<u8> {
        let tiff = tiff::Tiff { ifd0 };
        let tiff_bytes = tiff::serialize(&tiff);
        let base = base_image(format);
        match format {
            ImageFormat::Jpeg => {
                let mut jpeg = Jpeg::from_bytes(Bytes::from(base)).expect("parse jpeg");
                jpeg.set_exif(Some(Bytes::from(tiff_bytes)));
                let mut out = Vec::new();
                jpeg.encoder().write_to(&mut out).expect("encode jpeg");
                out
            }
            ImageFormat::Png => {
                let mut png = Png::from_bytes(Bytes::from(base)).expect("parse png");
                png.set_exif(Some(Bytes::from(tiff_bytes)));
                let mut out = Vec::new();
                png.encoder().write_to(&mut out).expect("encode png");
                out
            }
            _ => panic!("image_with_tiff only supports Jpeg/Png"),
        }
    }

    /// A JPEG seeded with Orientation + Copyright (IFD0) and
    /// GPS{Latitude,Longitude}Ref (GPS sub-IFD). Used to verify selective GPS
    /// removal.
    fn jpeg_with_exif() -> Vec<u8> {
        let gps = Ifd {
            entries: vec![
                ascii_entry(TAG_GPS_LAT_REF, "N"),
                ascii_entry(TAG_GPS_LON_REF, "E"),
            ],
            next: None,
            thumbnail: None,
        };
        let ifd0 = Ifd {
            entries: vec![
                short_entry(TAG_ORIENTATION, 1),
                ascii_entry(TAG_COPYRIGHT, "crustyimg test"),
                pointer_entry(tiff::GPS_PTR, gps),
            ],
            next: None,
            thumbnail: None,
        };
        image_with_tiff(ImageFormat::Jpeg, ifd0)
    }

    /// Whether a JPEG byte stream contains any segment with the given marker.
    fn jpeg_has_marker(bytes: &[u8], marker: u8) -> bool {
        let jpeg = Jpeg::from_bytes(Bytes::from(bytes.to_vec())).expect("parse jpeg");
        let present = jpeg.segments_by_marker(marker).next().is_some();
        present
    }

    /// Whether a PNG byte stream contains a chunk of the given type.
    fn png_has_chunk(bytes: &[u8], kind: [u8; 4]) -> bool {
        let png = Png::from_bytes(Bytes::from(bytes.to_vec())).expect("parse png");
        let present = png.chunks_by_type(kind).next().is_some();
        present
    }

    /// Decode two image byte streams and assert their RGBA pixel buffers match.
    fn assert_pixels_equal(a: &[u8], b: &[u8]) {
        let da = ::image::load_from_memory(a).expect("decode a").to_rgba8();
        let db = ::image::load_from_memory(b).expect("decode b").to_rgba8();
        assert_eq!(da.dimensions(), db.dimensions(), "dimensions differ");
        assert_eq!(da.into_raw(), db.into_raw(), "pixel buffers differ");
    }

    // ── kamadak-exif read-back helpers (semantic assertions, not byte-compare;
    //    per SPEC-045: our TIFF bytes won't match little_exif's, but must be
    //    semantically equivalent). ─────────────────────────────────────────

    /// Extract the bare TIFF/EXIF block from a JPEG or PNG via `img-parts`,
    /// or `None` if the container carries no EXIF at all.
    fn container_exif(bytes: &[u8]) -> Option<Bytes> {
        match ::image::guess_format(bytes) {
            Ok(ImageFormat::Jpeg) => Jpeg::from_bytes(Bytes::from(bytes.to_vec())).ok()?.exif(),
            Ok(ImageFormat::Png) => Png::from_bytes(Bytes::from(bytes.to_vec())).ok()?.exif(),
            _ => None,
        }
    }

    /// Parse a container's EXIF with `kamadak-exif`, or `None` if there is
    /// none.
    fn read_exif(bytes: &[u8]) -> Option<exif::Exif> {
        let tiff = container_exif(bytes)?;
        exif::Reader::new().read_raw(tiff.to_vec()).ok()
    }

    /// Read an IFD0 (primary) ASCII field's string value (NUL-trimmed).
    fn primary_string(exif: &exif::Exif, tag: exif::Tag) -> Option<String> {
        let field = exif.get_field(tag, exif::In::PRIMARY)?;
        match &field.value {
            exif::Value::Ascii(v) => v
                .first()
                .map(|b| String::from_utf8_lossy(b).trim_end_matches('\0').to_owned()),
            _ => None,
        }
    }

    // exif crate tag constants used across tests.
    const TAG_EXIF_ARTIST: exif::Tag = exif::Tag::Artist;
    const TAG_EXIF_COPYRIGHT: exif::Tag = exif::Tag::Copyright;
    const TAG_EXIF_DESCRIPTION: exif::Tag = exif::Tag::ImageDescription;
    const TAG_EXIF_ORIENTATION: exif::Tag = exif::Tag::Orientation;
    const TAG_EXIF_EXPOSURE_TIME: exif::Tag = exif::Tag::ExposureTime;

    #[test]
    fn strip_all_jpeg_removes_all_metadata() {
        let input = jpeg_with_exif();
        // Precondition: the seeded JPEG actually carries an APP1 (EXIF) segment.
        assert!(
            jpeg_has_marker(&input, 0xE1),
            "fixture should have APP1 EXIF"
        );

        let out = strip_all(&input).expect("strip");

        // No APP1..APP15 and no COM segments survive.
        for marker in 0xE1u8..=0xEF {
            assert!(
                !jpeg_has_marker(&out, marker),
                "marker {marker:#x} should be gone"
            );
        }
        assert!(!jpeg_has_marker(&out, 0xFE), "COM should be gone");

        // No EXIF at all should be readable back.
        assert!(read_exif(&out).is_none(), "no EXIF should remain");
    }

    #[test]
    fn strip_all_jpeg_preserves_pixels() {
        let input = jpeg_with_exif();
        let out = strip_all(&input).expect("strip");
        assert_pixels_equal(&input, &out);
    }

    #[test]
    fn strip_all_png_removes_metadata_chunks() {
        // Seed a PNG with a tEXt chunk via img-parts (native, no ImageMagick).
        let base = base_image(ImageFormat::Png);
        let mut png = Png::from_bytes(Bytes::from(base)).expect("parse png");
        let text = img_parts::png::PngChunk::new(*b"tEXt", Bytes::from_static(b"Comment\0hi"));
        png.chunks_mut().insert(1, text);
        let mut seeded = Vec::new();
        png.encoder()
            .write_to(&mut seeded)
            .expect("encode seeded png");
        assert!(png_has_chunk(&seeded, *b"tEXt"), "fixture should have tEXt");

        let out = strip_all(&seeded).expect("strip");
        assert!(!png_has_chunk(&out, *b"tEXt"), "tEXt should be gone");
        assert_pixels_equal(&seeded, &out);
    }

    #[test]
    fn clean_gps_removes_only_gps() {
        let input = jpeg_with_exif();
        let out = clean_gps(&input).expect("clean");

        let exif = read_exif(&out).expect("reparse");
        // GPS IFD has no tags left (the GPS pointer entry itself is dropped).
        assert!(
            exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY)
                .is_none(),
            "GPS tags should be gone"
        );
        // Orientation + Copyright survive in IFD0.
        assert!(
            exif.get_field(TAG_EXIF_ORIENTATION, exif::In::PRIMARY)
                .is_some(),
            "Orientation should survive"
        );
        assert_eq!(
            primary_string(&exif, TAG_EXIF_COPYRIGHT).as_deref(),
            Some("crustyimg test"),
            "Copyright should survive"
        );
    }

    #[test]
    fn clean_gps_preserves_pixels() {
        let input = jpeg_with_exif();
        let out = clean_gps(&input).expect("clean");
        assert_pixels_equal(&input, &out);
    }

    #[test]
    fn clean_gps_no_exif_is_noop_ok() {
        // A plain JPEG with no EXIF: clean_gps returns Ok with identical pixels.
        let input = base_image(ImageFormat::Jpeg);
        let out = clean_gps(&input).expect("clean no-exif must be Ok");
        assert_pixels_equal(&input, &out);
    }

    /// `clean_gps_no_exif_is_noop` (SPEC-045 failing test): a no-EXIF input's
    /// bytes are returned byte-identical, not merely pixel-equal.
    #[test]
    fn clean_gps_no_exif_is_noop() {
        let input = base_image(ImageFormat::Jpeg);
        let out = clean_gps(&input).expect("clean no-exif must be Ok");
        assert_eq!(
            input, out,
            "no-EXIF clean_gps must be a byte-identical no-op"
        );
    }

    /// `clean_gps_removes_only_gps` (SPEC-045 failing test, PNG variant): GPS
    /// is removed and non-GPS tags (Orientation/Copyright) survive.
    #[test]
    fn clean_gps_removes_only_gps_png() {
        let gps = Ifd {
            entries: vec![
                ascii_entry(TAG_GPS_LAT_REF, "N"),
                ascii_entry(TAG_GPS_LON_REF, "E"),
            ],
            next: None,
            thumbnail: None,
        };
        let ifd0 = Ifd {
            entries: vec![
                short_entry(TAG_ORIENTATION, 1),
                ascii_entry(TAG_COPYRIGHT, "png owner"),
                pointer_entry(tiff::GPS_PTR, gps),
            ],
            next: None,
            thumbnail: None,
        };
        let input = image_with_tiff(ImageFormat::Png, ifd0);

        let out = clean_gps(&input).expect("clean");
        let exif = read_exif(&out).expect("reparse");
        assert!(
            exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY)
                .is_none(),
            "GPS tags should be gone"
        );
        assert!(
            exif.get_field(TAG_EXIF_ORIENTATION, exif::In::PRIMARY)
                .is_some(),
            "Orientation should survive"
        );
        assert_eq!(
            primary_string(&exif, TAG_EXIF_COPYRIGHT).as_deref(),
            Some("png owner")
        );
        assert_pixels_equal(&input, &out);
    }

    #[test]
    fn strip_all_unsupported_format_errors() {
        let bmp = base_image(ImageFormat::Bmp);
        assert!(matches!(
            strip_all(&bmp),
            Err(MetadataError::UnsupportedFormat(_))
        ));
    }

    #[test]
    fn clean_gps_unsupported_format_errors() {
        let bmp = base_image(ImageFormat::Bmp);
        assert!(matches!(
            clean_gps(&bmp),
            Err(MetadataError::UnsupportedFormat(_))
        ));
    }

    // ── set_tags ──────────────────────────────────────────────────────────────

    #[test]
    fn set_tags_writes_all_three() {
        let input = base_image(ImageFormat::Jpeg);
        let tags = TagSet {
            artist: Some("Jane".to_string()),
            copyright: Some("2026 Jane".to_string()),
            description: Some("a test image".to_string()),
        };
        let out = set_tags(&input, &tags).expect("set");
        let exif = read_exif(&out).expect("reparse");
        assert_eq!(
            primary_string(&exif, TAG_EXIF_ARTIST).as_deref(),
            Some("Jane")
        );
        assert_eq!(
            primary_string(&exif, TAG_EXIF_COPYRIGHT).as_deref(),
            Some("2026 Jane")
        );
        assert_eq!(
            primary_string(&exif, TAG_EXIF_DESCRIPTION).as_deref(),
            Some("a test image")
        );
    }

    #[test]
    fn set_tags_preserves_existing_metadata() {
        // jpeg_with_exif seeds Orientation + Copyright + GPS refs.
        let input = jpeg_with_exif();
        let tags = TagSet {
            artist: Some("Added".to_string()),
            ..TagSet::default()
        };
        let out = set_tags(&input, &tags).expect("set");

        let exif = read_exif(&out).expect("reparse");
        assert!(
            exif.get_field(TAG_EXIF_ORIENTATION, exif::In::PRIMARY)
                .is_some(),
            "Orientation should survive"
        );
        assert_eq!(
            primary_string(&exif, TAG_EXIF_ARTIST).as_deref(),
            Some("Added")
        );
        // GPS refs survive too.
        assert!(
            exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY)
                .is_some(),
            "GPS tags should survive"
        );
    }

    #[test]
    fn set_tags_overwrites_existing_tag() {
        // Seed Copyright="OLD".
        let ifd0 = Ifd {
            entries: vec![ascii_entry(TAG_COPYRIGHT, "OLD")],
            next: None,
            thumbnail: None,
        };
        let input = image_with_tiff(ImageFormat::Jpeg, ifd0);
        assert_eq!(
            primary_string(
                &read_exif(&input).expect("reparse seed"),
                TAG_EXIF_COPYRIGHT
            )
            .as_deref(),
            Some("OLD")
        );

        let tags = TagSet {
            copyright: Some("NEW".to_string()),
            ..TagSet::default()
        };
        let out = set_tags(&input, &tags).expect("set");
        let exif = read_exif(&out).expect("reparse");
        assert_eq!(
            primary_string(&exif, TAG_EXIF_COPYRIGHT).as_deref(),
            Some("NEW")
        );
    }

    /// `set_overwrites_existing_tag` (SPEC-045 failing test): setting the
    /// same tag twice with different values leaves exactly ONE IFD0 entry
    /// for that tag (no duplicate), asserted via the raw parsed `Ifd`.
    #[test]
    fn set_overwrites_existing_tag_no_duplicate() {
        let input = base_image(ImageFormat::Jpeg);
        let out1 = set_tags(
            &input,
            &TagSet {
                copyright: Some("OLD".to_string()),
                ..TagSet::default()
            },
        )
        .expect("set 1");
        let out2 = set_tags(
            &out1,
            &TagSet {
                copyright: Some("NEW".to_string()),
                ..TagSet::default()
            },
        )
        .expect("set 2");

        let tiff_bytes = container_exif(&out2).expect("exif present");
        let parsed = tiff::parse(&tiff_bytes).expect("parse");
        let copyright_entries: Vec<_> = parsed
            .ifd0
            .entries
            .iter()
            .filter(|e| e.tag == TAG_COPYRIGHT)
            .collect();
        assert_eq!(
            copyright_entries.len(),
            1,
            "exactly one Copyright entry, no duplicate"
        );

        let exif = read_exif(&out2).expect("reparse");
        assert_eq!(
            primary_string(&exif, TAG_EXIF_COPYRIGHT).as_deref(),
            Some("NEW")
        );
    }

    #[test]
    fn set_tags_on_no_exif_creates_them() {
        let input = base_image(ImageFormat::Jpeg);
        // Precondition: no EXIF at all.
        assert!(read_exif(&input).is_none(), "fixture should have no EXIF");
        let tags = TagSet {
            artist: Some("Fresh".to_string()),
            ..TagSet::default()
        };
        let out = set_tags(&input, &tags).expect("set");
        let exif = read_exif(&out).expect("reparse");
        assert_eq!(
            primary_string(&exif, TAG_EXIF_ARTIST).as_deref(),
            Some("Fresh")
        );
    }

    /// `set_on_no_exif_creates_minimal` (SPEC-045 failing test, JPEG + PNG):
    /// setting on a no-EXIF file produces output whose EXIF reads back
    /// exactly the set tag(s) — nothing more, nothing less than expected.
    #[test]
    fn set_on_no_exif_creates_minimal() {
        for format in [ImageFormat::Jpeg, ImageFormat::Png] {
            let input = base_image(format);
            assert!(
                read_exif(&input).is_none(),
                "fixture ({format:?}) should have no EXIF"
            );
            let tags = TagSet {
                artist: Some("Solo".to_string()),
                ..TagSet::default()
            };
            let out = set_tags(&input, &tags).expect("set");
            let exif = read_exif(&out).expect("reparse");
            assert_eq!(
                primary_string(&exif, TAG_EXIF_ARTIST).as_deref(),
                Some("Solo"),
                "{format:?}: artist should read back"
            );
            assert!(
                primary_string(&exif, TAG_EXIF_COPYRIGHT).is_none(),
                "{format:?}: copyright was never set"
            );
            assert_pixels_equal(&input, &out);
        }
    }

    #[test]
    fn set_tags_preserves_pixels() {
        let input = jpeg_with_exif();
        let tags = TagSet {
            artist: Some("Jane".to_string()),
            ..TagSet::default()
        };
        let out = set_tags(&input, &tags).expect("set");
        assert_pixels_equal(&input, &out);
    }

    #[test]
    fn set_tags_png() {
        let input = base_image(ImageFormat::Png);
        let tags = TagSet {
            copyright: Some("PNG owner".to_string()),
            ..TagSet::default()
        };
        let out = set_tags(&input, &tags).expect("set");
        let exif = read_exif(&out).expect("reparse");
        assert_eq!(
            primary_string(&exif, TAG_EXIF_COPYRIGHT).as_deref(),
            Some("PNG owner")
        );
        assert_pixels_equal(&input, &out);
    }

    #[test]
    fn set_tags_unsupported_format_errors() {
        let bmp = base_image(ImageFormat::Bmp);
        let tags = TagSet {
            artist: Some("x".to_string()),
            ..TagSet::default()
        };
        assert!(matches!(
            set_tags(&bmp, &tags),
            Err(MetadataError::UnsupportedFormat(_))
        ));
    }

    /// `set_preserves_exififd_subtag` (SPEC-045 failing test): a JPEG whose
    /// EXIF has an IFD0 tag AND an ExifIFD sub-tag (ExposureTime); after
    /// `set_tags(.. artist ..)`, Artist is present AND ExposureTime survives
    /// with the same value — the probe-proven sub-IFD-preservation core.
    #[test]
    fn set_preserves_exififd_subtag() {
        let exif_ifd = Ifd {
            entries: vec![rational_entry(TAG_EXPOSURE_TIME, 1, 250)],
            next: None,
            thumbnail: None,
        };
        let ifd0 = Ifd {
            entries: vec![
                ascii_entry(TAG_COPYRIGHT, "orig"),
                pointer_entry(tiff::EXIF_PTR, exif_ifd),
            ],
            next: None,
            thumbnail: None,
        };
        let input = image_with_tiff(ImageFormat::Jpeg, ifd0);

        // Precondition: ExposureTime is readable before the edit.
        let before = read_exif(&input).expect("reparse seed");
        let exposure_before = before
            .get_field(TAG_EXIF_EXPOSURE_TIME, exif::In::PRIMARY)
            .expect("seed should carry ExposureTime")
            .value
            .clone();

        let tags = TagSet {
            artist: Some("Jane".to_string()),
            ..TagSet::default()
        };
        let out = set_tags(&input, &tags).expect("set");

        let exif = read_exif(&out).expect("reparse out");
        assert_eq!(
            primary_string(&exif, TAG_EXIF_ARTIST).as_deref(),
            Some("Jane")
        );
        let exposure_after = exif
            .get_field(TAG_EXIF_EXPOSURE_TIME, exif::In::PRIMARY)
            .expect("ExposureTime should survive the edit")
            .value
            .clone();
        assert_eq!(
            format!("{exposure_before:?}"),
            format!("{exposure_after:?}"),
            "ExposureTime value should be unchanged"
        );
        // Non-GPS/other IFD0 tag also survives.
        assert_eq!(
            primary_string(&exif, TAG_EXIF_COPYRIGHT).as_deref(),
            Some("orig")
        );
    }

    /// `set_preserves_ifd1_thumbnail` (SPEC-045 failing test): a JPEG with an
    /// IFD1 thumbnail; after `set_tags`, the output still contains a
    /// readable thumbnail (IFD1 `JPEGInterchangeFormat` blob intact).
    #[test]
    fn set_preserves_ifd1_thumbnail() {
        // A tiny valid JPEG to act as the thumbnail blob (decodability is
        // not required by IFD1 semantics here — we just need bytes that
        // round-trip and can be located via 0x0201/0x0202).
        let thumb_bytes = base_image(ImageFormat::Jpeg);

        let ifd1 = Ifd {
            entries: vec![
                Entry {
                    tag: TAG_THUMB_OFFSET,
                    ty: 4, // LONG — the serializer patches this offset
                    count: 1,
                    value: 0u32.to_le_bytes().to_vec(),
                    sub: None,
                },
                Entry {
                    tag: TAG_THUMB_LENGTH,
                    ty: 4,
                    count: 1,
                    value: (thumb_bytes.len() as u32).to_le_bytes().to_vec(),
                    sub: None,
                },
            ],
            next: None,
            thumbnail: Some(thumb_bytes.clone()),
        };
        let ifd0 = Ifd {
            entries: vec![ascii_entry(TAG_COPYRIGHT, "orig")],
            next: Some(Box::new(ifd1)),
            thumbnail: None,
        };
        let input = image_with_tiff(ImageFormat::Jpeg, ifd0);

        // Precondition: the thumbnail is present + readable before the edit.
        let tiff_before = container_exif(&input).expect("exif present");
        let parsed_before = tiff::parse(&tiff_before).expect("parse seed");
        assert_eq!(
            parsed_before
                .ifd0
                .next
                .as_ref()
                .and_then(|n| n.thumbnail.as_ref()),
            Some(&thumb_bytes),
            "seed should carry the thumbnail in IFD1"
        );

        let tags = TagSet {
            artist: Some("Jane".to_string()),
            ..TagSet::default()
        };
        let out = set_tags(&input, &tags).expect("set");

        let tiff_after = container_exif(&out).expect("exif present after set");
        let parsed_after = tiff::parse(&tiff_after).expect("parse out");
        let ifd1_after = parsed_after.ifd0.next.expect("IFD1 should survive");
        assert_eq!(
            ifd1_after.thumbnail.as_deref(),
            Some(thumb_bytes.as_slice()),
            "thumbnail blob should be intact after set_tags"
        );

        let exif = read_exif(&out).expect("reparse out");
        assert_eq!(
            primary_string(&exif, TAG_EXIF_ARTIST).as_deref(),
            Some("Jane")
        );
    }

    /// `set_and_clean_preserve_pixels` (SPEC-045 failing test): both
    /// `set_tags` and `clean_gps` preserve pixels exactly, for JPEG + PNG.
    #[test]
    fn set_and_clean_preserve_pixels() {
        for format in [ImageFormat::Jpeg, ImageFormat::Png] {
            let gps = Ifd {
                entries: vec![ascii_entry(TAG_GPS_LAT_REF, "N")],
                next: None,
                thumbnail: None,
            };
            let ifd0 = Ifd {
                entries: vec![
                    ascii_entry(TAG_COPYRIGHT, "orig"),
                    pointer_entry(tiff::GPS_PTR, gps),
                ],
                next: None,
                thumbnail: None,
            };
            let input = image_with_tiff(format, ifd0);

            let set_out = set_tags(
                &input,
                &TagSet {
                    artist: Some("Jane".to_string()),
                    ..TagSet::default()
                },
            )
            .expect("set");
            assert_pixels_equal(&input, &set_out);

            let clean_out = clean_gps(&input).expect("clean");
            assert_pixels_equal(&input, &clean_out);
        }
    }

    /// `malformed_exif_errors_not_panics` (SPEC-045 failing test): a
    /// truncated/garbage TIFF block (bad IFD offset, out-of-bounds value
    /// offset, self-referential sub-IFD pointer) must yield a
    /// `MetadataError`, never a panic.
    #[test]
    fn malformed_exif_errors_not_panics() {
        // 1. Too short to even hold a header.
        assert!(tiff::parse(&[0u8; 4]).is_err());

        // 2. Valid header, IFD0 offset points past the end of the buffer.
        let bad_offset = {
            let mut buf = vec![b'I', b'I', 42, 0];
            buf.extend_from_slice(&1_000_000u32.to_le_bytes());
            buf
        };
        assert!(tiff::parse(&bad_offset).is_err());

        // 3. One entry whose out-of-line value offset is out of bounds.
        let bad_value_offset = {
            let mut buf = vec![b'I', b'I', 42, 0, 8, 0, 0, 0];
            buf.extend_from_slice(&1u16.to_le_bytes()); // 1 entry
            buf.extend_from_slice(&TAG_COPYRIGHT.to_le_bytes()); // tag
            buf.extend_from_slice(&2u16.to_le_bytes()); // type ASCII
            buf.extend_from_slice(&100u32.to_le_bytes()); // count (vlen > 4)
            buf.extend_from_slice(&9_999_999u32.to_le_bytes()); // OOB value offset
            buf.extend_from_slice(&0u32.to_le_bytes()); // next IFD = none
            buf
        };
        assert!(tiff::parse(&bad_value_offset).is_err());

        // 4. Self-referential sub-IFD: the ExifIFD pointer offset is the
        //    IFD0 offset itself (a 1-cycle).
        let cyclic = {
            let mut buf = vec![b'I', b'I', 42, 0, 8, 0, 0, 0];
            buf.extend_from_slice(&1u16.to_le_bytes()); // 1 entry
            buf.extend_from_slice(&tiff::EXIF_PTR.to_le_bytes());
            buf.extend_from_slice(&4u16.to_le_bytes()); // type LONG
            buf.extend_from_slice(&1u32.to_le_bytes()); // count 1
            buf.extend_from_slice(&8u32.to_le_bytes()); // points at IFD0 itself
            buf.extend_from_slice(&0u32.to_le_bytes()); // next IFD = none
            buf
        };
        assert!(tiff::parse(&cyclic).is_err());

        // And through the public API: set_tags/clean_gps on a container
        // whose EXIF is one of these malformed blocks must error, not panic.
        let mut jpeg =
            Jpeg::from_bytes(Bytes::from(base_image(ImageFormat::Jpeg))).expect("parse jpeg");
        jpeg.set_exif(Some(Bytes::from(bad_value_offset)));
        let mut malformed = Vec::new();
        jpeg.encoder()
            .write_to(&mut malformed)
            .expect("encode malformed jpeg");

        assert!(matches!(
            set_tags(
                &malformed,
                &TagSet {
                    artist: Some("x".to_string()),
                    ..TagSet::default()
                }
            ),
            Err(MetadataError::Exif(_))
        ));
        assert!(matches!(clean_gps(&malformed), Err(MetadataError::Exif(_))));
    }

    // ── copy_metadata ──────────────────────────────────────────────────────────

    /// A JPEG seeded with a single Copyright tag (no other metadata). Used
    /// as the SRC donor / DST baseline in the copy tests.
    fn jpeg_with_copyright(value: &str) -> Vec<u8> {
        let ifd0 = Ifd {
            entries: vec![ascii_entry(TAG_COPYRIGHT, value)],
            next: None,
            thumbnail: None,
        };
        image_with_tiff(ImageFormat::Jpeg, ifd0)
    }

    /// Inject an ICC profile blob into a JPEG via `img-parts` `set_icc_profile`
    /// (native, no ImageMagick) and return the rewritten bytes.
    fn jpeg_with_icc(bytes: &[u8], icc: &[u8]) -> Vec<u8> {
        let mut jpeg = Jpeg::from_bytes(Bytes::from(bytes.to_vec())).expect("parse jpeg");
        jpeg.set_icc_profile(Some(Bytes::from(icc.to_vec())));
        let mut out = Vec::new();
        jpeg.encoder().write_to(&mut out).expect("encode icc jpeg");
        out
    }

    /// Read the Copyright (0x8298) string from a JPEG's EXIF, if present.
    fn jpeg_copyright(bytes: &[u8]) -> Option<String> {
        let exif = read_exif(bytes)?;
        primary_string(&exif, TAG_EXIF_COPYRIGHT)
    }

    #[test]
    fn copy_metadata_transfers_exif() {
        let src = jpeg_with_copyright("SRC owner");
        let dst = base_image(ImageFormat::Jpeg);
        // Precondition: DST has no Copyright.
        assert_eq!(jpeg_copyright(&dst), None, "DST should start without EXIF");

        let out = copy_metadata(&src, &dst).expect("copy");
        assert_eq!(
            jpeg_copyright(&out).as_deref(),
            Some("SRC owner"),
            "out should carry SRC's Copyright"
        );
    }

    #[test]
    fn copy_metadata_transfers_icc() {
        // A small but valid-shaped ICC blob; img-parts treats it opaquely.
        let icc: Vec<u8> = (0u8..=255).cycle().take(512).collect();
        let src = jpeg_with_icc(&base_image(ImageFormat::Jpeg), &icc);
        let dst = base_image(ImageFormat::Jpeg);

        let out = copy_metadata(&src, &dst).expect("copy");
        let out_jpeg = Jpeg::from_bytes(Bytes::from(out)).expect("parse out");
        assert_eq!(
            out_jpeg.icc_profile().as_deref(),
            Some(icc.as_slice()),
            "out should carry SRC's ICC profile"
        );
    }

    #[test]
    fn copy_metadata_preserves_recipient_pixels() {
        let src = jpeg_with_copyright("SRC owner");
        let dst = jpeg_with_exif(); // a DIFFERENT pixel buffer is irrelevant; use seeded DST
        let out = copy_metadata(&src, &dst).expect("copy");
        // Decoded pixels of out must equal DST's (no re-encode).
        assert_pixels_equal(&dst, &out);
    }

    #[test]
    fn copy_metadata_replaces_recipient_metadata() {
        let src = jpeg_with_copyright("A");
        let dst = jpeg_with_copyright("B");
        assert_eq!(jpeg_copyright(&dst).as_deref(), Some("B"));

        let out = copy_metadata(&src, &dst).expect("copy");
        assert_eq!(
            jpeg_copyright(&out).as_deref(),
            Some("A"),
            "DST's metadata should be replaced by SRC's"
        );
    }

    #[test]
    fn copy_metadata_src_without_metadata_clears_dst() {
        // SRC: a plain JPEG with no EXIF/ICC.
        let src = base_image(ImageFormat::Jpeg);
        // DST: seeded with both EXIF (Copyright) and an ICC profile.
        let icc: Vec<u8> = (0u8..=255).cycle().take(256).collect();
        let dst = jpeg_with_icc(&jpeg_with_copyright("DST owner"), &icc);
        // Precondition: DST has both.
        let dst_jpeg = Jpeg::from_bytes(Bytes::from(dst.clone())).expect("parse dst");
        assert!(dst_jpeg.exif().is_some(), "DST should start with EXIF");
        assert!(
            dst_jpeg.icc_profile().is_some(),
            "DST should start with ICC"
        );

        let out = copy_metadata(&src, &dst).expect("copy");
        let out_jpeg = Jpeg::from_bytes(Bytes::from(out)).expect("parse out");
        assert!(out_jpeg.exif().is_none(), "out EXIF should be cleared");
        assert!(
            out_jpeg.icc_profile().is_none(),
            "out ICC should be cleared"
        );
    }

    #[test]
    fn copy_metadata_unsupported_format_errors() {
        let jpeg = jpeg_with_copyright("owner");
        let png = base_image(ImageFormat::Png);
        // PNG as `to` → unsupported.
        assert!(matches!(
            copy_metadata(&jpeg, &png),
            Err(MetadataError::UnsupportedFormat(_))
        ));
        // PNG as `from` → unsupported.
        assert!(matches!(
            copy_metadata(&png, &jpeg),
            Err(MetadataError::UnsupportedFormat(_))
        ));
    }
}
