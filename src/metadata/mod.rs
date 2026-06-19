//! Container-lane metadata edits (SPEC-026, DEC-003).
//!
//! This is the **container lane**: it edits container-level metadata
//! (EXIF/ICC/XMP/IPTC/comments) by operating on the **raw container bytes** and
//! never re-decodes or re-encodes pixels. The compressed scan (JPEG) / `IDAT`
//! (PNG) is carried through verbatim, so decoding the output yields pixels that
//! are byte-identical to decoding the input — the constraint
//! `metadata-not-via-pixel-encode` made concrete.
//!
//! Division of labor (DEC-003 / DEC-029):
//! - **`strip_all`** removes *all* user metadata at the segment/chunk level via
//!   [`img_parts`] (JPEG APP1..APP15 + COM; PNG `eXIf`/`iCCP`/`tEXt`/…).
//! - **`clean_gps`** removes *only* the GPS IFD at the tag level via
//!   [`little_exif`], preserving every other tag (orientation, copyright, …).
//!
//! The format is sniffed with [`image::guess_format`] (a magic-byte check, no
//! decode). Only JPEG and PNG are supported in v1; any other format is a
//! [`MetadataError::UnsupportedFormat`]. The read side stays `kamadak-exif`
//! elsewhere; this module is the write half.

use ::image::ImageFormat;
use img_parts::jpeg::Jpeg;
use img_parts::png::Png;
use img_parts::{Bytes, ImageEXIF, ImageICC};
use little_exif::filetype::FileExtension;
use little_exif::ifd::ExifTagGroup;
use little_exif::metadata::Metadata;

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

    /// A tag-level EXIF parse or rewrite failure (`little_exif`), excluding the
    /// benign "no EXIF" case which [`clean_gps`] treats as a no-op.
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

/// The `little_exif` [`FileExtension`] for a [`Lane`] (PNG never re-encodes EXIF
/// as a `zTXt` chunk — it uses the dedicated `eXIf` chunk).
fn file_extension(lane: Lane) -> FileExtension {
    match lane {
        Lane::Jpeg => FileExtension::JPEG,
        Lane::Png => FileExtension::PNG {
            as_zTXt_chunk: false,
        },
    }
}

/// Remove **only** GPS/location metadata, preserving every other tag and the
/// pixels exactly.
///
/// Parses the EXIF with `little_exif`, drops every tag in the GPS IFD, and
/// writes the result back into a clone of the input bytes. A file with **no
/// EXIF** is a no-op success: `little_exif` returns an error whose message
/// contains "No EXIF", which is caught here and the input is returned unchanged
/// (DEC-029 edge case).
pub fn clean_gps(bytes: &[u8]) -> Result<Vec<u8>, MetadataError> {
    let lane = sniff(bytes)?;
    let ext = file_extension(lane);

    let owned = bytes.to_vec();
    let mut md = match Metadata::new_from_vec(&owned, ext) {
        Ok(md) => md,
        Err(e) => {
            // "No EXIF data found!" → nothing to clean, byte-faithful no-op.
            if e.to_string().contains("No EXIF") {
                return Ok(owned);
            }
            return Err(MetadataError::Exif(e.to_string()));
        }
    };

    // Collect the GPS IFD's tag ids, then remove each. `get_ifd_mut` creates an
    // (empty) GPS IFD if none exists — harmless: there is then nothing to drop.
    let gps_tag_ids: Vec<u16> = md
        .get_ifd_mut(ExifTagGroup::GPS, 0)
        .get_tags()
        .iter()
        .map(|t| t.as_u16())
        .collect();

    if !gps_tag_ids.is_empty() {
        let gps = md.get_ifd_mut(ExifTagGroup::GPS, 0);
        for id in gps_tag_ids {
            gps.remove_tag(id);
        }
    }

    let mut out = owned.clone();
    md.write_to_vec(&mut out, ext)
        .map_err(|e| MetadataError::Exif(e.to_string()))?;
    Ok(out)
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
/// Loads the existing metadata first so other tags survive; a file with **no
/// EXIF** falls back to a fresh EXIF block carrying just the given tags
/// (`little_exif` returns an error on parse, caught here). Only JPEG + PNG are
/// supported in v1; any other format is a [`MetadataError::UnsupportedFormat`].
pub fn set_tags(bytes: &[u8], tags: &TagSet) -> Result<Vec<u8>, MetadataError> {
    use little_exif::exif_tag::ExifTag;

    let lane = sniff(bytes)?;
    let ext = file_extension(lane);

    let owned = bytes.to_vec();
    // Load-then-set preserves existing tags; the Err branch is the "No EXIF"
    // fresh-create fallback (probe-verified, DEC-029).
    let mut md = Metadata::new_from_vec(&owned, ext).unwrap_or_else(|_| Metadata::new());

    if let Some(ref artist) = tags.artist {
        md.set_tag(ExifTag::Artist(artist.clone()));
    }
    if let Some(ref copyright) = tags.copyright {
        md.set_tag(ExifTag::Copyright(copyright.clone()));
    }
    if let Some(ref description) = tags.description {
        md.set_tag(ExifTag::ImageDescription(description.clone()));
    }

    let mut out = owned.clone();
    md.write_to_vec(&mut out, ext)
        .map_err(|e| MetadataError::Exif(e.to_string()))?;
    Ok(out)
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
    use super::*;
    use ::image::{ImageFormat, RgbImage};
    use little_exif::exif_tag::ExifTag;
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

    /// A JPEG seeded with Orientation + Copyright + GPS{Latitude,Longitude}Ref
    /// via `little_exif` (no ImageMagick). Used to verify selective GPS removal.
    fn jpeg_with_exif() -> Vec<u8> {
        let mut bytes = base_image(ImageFormat::Jpeg);
        let mut md = Metadata::new();
        md.set_tag(ExifTag::Orientation(vec![1]));
        md.set_tag(ExifTag::Copyright("crustyimg test".to_string()));
        md.set_tag(ExifTag::GPSLatitudeRef("N".to_string()));
        md.set_tag(ExifTag::GPSLongitudeRef("E".to_string()));
        md.write_to_vec(&mut bytes, FileExtension::JPEG)
            .expect("seed JPEG EXIF");
        bytes
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

        // And little_exif no longer finds any EXIF.
        let reparse = Metadata::new_from_vec(&out, FileExtension::JPEG);
        match reparse {
            Err(e) => assert!(e.to_string().contains("No EXIF"), "got: {e}"),
            Ok(mut md) => assert!(
                md.get_ifd_mut(ExifTagGroup::GENERIC, 0)
                    .get_tags()
                    .is_empty()
                    && md.get_ifd_mut(ExifTagGroup::GPS, 0).get_tags().is_empty(),
                "no tags should remain"
            ),
        }
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
        let text = img_parts::png::PngChunk::new(
            [b't', b'E', b'X', b't'],
            Bytes::from_static(b"Comment\0hi"),
        );
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

        let mut md = Metadata::new_from_vec(&out, FileExtension::JPEG).expect("reparse");
        // GPS IFD has no tags left.
        assert!(
            md.get_ifd_mut(ExifTagGroup::GPS, 0).get_tags().is_empty(),
            "GPS tags should be gone"
        );
        // Orientation (0x0112) + Copyright (0x8298) survive in the generic IFD.
        let generic_ids: Vec<u16> = md
            .get_ifd_mut(ExifTagGroup::GENERIC, 0)
            .get_tags()
            .iter()
            .map(|t| t.as_u16())
            .collect();
        assert!(generic_ids.contains(&0x0112), "Orientation should survive");
        assert!(generic_ids.contains(&0x8298), "Copyright should survive");
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

    /// Read one generic-IFD STRING tag value by its tag id from container bytes.
    fn read_generic_string(bytes: &[u8], ext: FileExtension, tag_id: u16) -> Option<String> {
        let mut md = Metadata::new_from_vec(&bytes.to_vec(), ext).ok()?;
        md.get_ifd_mut(ExifTagGroup::GENERIC, 0)
            .get_tags()
            .iter()
            .find(|t| t.as_u16() == tag_id)
            .map(|t| t.value_as_u8_vec(&little_exif::endian::Endian::Little))
            .map(|raw| {
                String::from_utf8_lossy(&raw)
                    .trim_end_matches('\0')
                    .to_owned()
            })
    }

    // Tag ids (IFD0): Artist 0x013B, Copyright 0x8298, ImageDescription 0x010E.
    const TAG_ARTIST: u16 = 0x013B;
    const TAG_COPYRIGHT: u16 = 0x8298;
    const TAG_DESCRIPTION: u16 = 0x010E;

    #[test]
    fn set_tags_writes_all_three() {
        let input = base_image(ImageFormat::Jpeg);
        let tags = TagSet {
            artist: Some("Jane".to_string()),
            copyright: Some("2026 Jane".to_string()),
            description: Some("a test image".to_string()),
        };
        let out = set_tags(&input, &tags).expect("set");
        assert_eq!(
            read_generic_string(&out, FileExtension::JPEG, TAG_ARTIST).as_deref(),
            Some("Jane")
        );
        assert_eq!(
            read_generic_string(&out, FileExtension::JPEG, TAG_COPYRIGHT).as_deref(),
            Some("2026 Jane")
        );
        assert_eq!(
            read_generic_string(&out, FileExtension::JPEG, TAG_DESCRIPTION).as_deref(),
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

        let mut md = Metadata::new_from_vec(&out, FileExtension::JPEG).expect("reparse");
        let generic_ids: Vec<u16> = md
            .get_ifd_mut(ExifTagGroup::GENERIC, 0)
            .get_tags()
            .iter()
            .map(|t| t.as_u16())
            .collect();
        assert!(generic_ids.contains(&0x0112), "Orientation should survive");
        assert!(generic_ids.contains(&TAG_ARTIST), "Artist should be added");
        // GPS refs survive too.
        assert!(
            !md.get_ifd_mut(ExifTagGroup::GPS, 0).get_tags().is_empty(),
            "GPS tags should survive"
        );
    }

    #[test]
    fn set_tags_overwrites_existing_tag() {
        // Seed Copyright="OLD".
        let mut input = base_image(ImageFormat::Jpeg);
        let mut seed = Metadata::new();
        seed.set_tag(ExifTag::Copyright("OLD".to_string()));
        seed.write_to_vec(&mut input, FileExtension::JPEG)
            .expect("seed");
        assert_eq!(
            read_generic_string(&input, FileExtension::JPEG, TAG_COPYRIGHT).as_deref(),
            Some("OLD")
        );

        let tags = TagSet {
            copyright: Some("NEW".to_string()),
            ..TagSet::default()
        };
        let out = set_tags(&input, &tags).expect("set");
        assert_eq!(
            read_generic_string(&out, FileExtension::JPEG, TAG_COPYRIGHT).as_deref(),
            Some("NEW")
        );
    }

    #[test]
    fn set_tags_on_no_exif_creates_them() {
        let input = base_image(ImageFormat::Jpeg);
        // Precondition: no EXIF at all.
        assert!(
            Metadata::new_from_vec(&input, FileExtension::JPEG).is_err(),
            "fixture should have no EXIF"
        );
        let tags = TagSet {
            artist: Some("Fresh".to_string()),
            ..TagSet::default()
        };
        let out = set_tags(&input, &tags).expect("set");
        assert_eq!(
            read_generic_string(&out, FileExtension::JPEG, TAG_ARTIST).as_deref(),
            Some("Fresh")
        );
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
        let ext = FileExtension::PNG {
            as_zTXt_chunk: false,
        };
        assert_eq!(
            read_generic_string(&out, ext, TAG_COPYRIGHT).as_deref(),
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

    // ── copy_metadata ──────────────────────────────────────────────────────────

    /// A JPEG seeded with a single Copyright tag via `little_exif` (no other
    /// metadata). Used as the SRC donor / DST baseline in the copy tests.
    fn jpeg_with_copyright(value: &str) -> Vec<u8> {
        let mut bytes = base_image(ImageFormat::Jpeg);
        let mut md = Metadata::new();
        md.set_tag(ExifTag::Copyright(value.to_string()));
        md.write_to_vec(&mut bytes, FileExtension::JPEG)
            .expect("seed JPEG Copyright");
        bytes
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
        read_generic_string(bytes, FileExtension::JPEG, TAG_COPYRIGHT)
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
