//! The shipped-capability lint rules (SPEC-053, DEC-050).
//!
//! Each rule reads only capabilities crustyimg already ships (`info`/EXIF read
//! plus the captured metadata bundle) and names a runnable `crustyimg` fix. None
//! re-encodes or runs the engine (the "could be smaller" rules are STAGE-014).
//! Rule ids are the DEC-050 stability surface. Config (severities, per-glob
//! budgets, intended width, opt-in enable) is applied by the runner and
//! resolved onto the [`LintTarget`](super::LintTarget) (SPEC-051).

use super::{Finding, LintTarget, Rule, Severity};

/// An embedded ICC profile larger than this (bytes) is flagged as "bulky"
/// (`color/unexpected-icc`). Standard sRGB/Adobe profiles are ~0.5–3 KiB, so a
/// 4 KiB floor avoids flagging the common cases.
const ICC_BULKY_BYTES: usize = 4096;

// ── privacy/camera-metadata (info, opt-in) ──────────────────────────────────

/// Identifying non-GPS camera EXIF (Make/Model/serial/lens/original-timestamp).
/// Opt-in (`info`). Fix: `meta strip`.
pub struct CameraMetadata;

impl Rule for CameraMetadata {
    fn id(&self) -> &'static str {
        "privacy/camera-metadata"
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn default_enabled(&self) -> bool {
        false
    }
    fn check(&self, target: &LintTarget) -> Option<Finding> {
        target.has_camera_metadata().then(|| {
            Finding::new(
                target.path().to_path_buf(),
                self.id(),
                self.default_severity(),
                "image carries identifying camera metadata (Make/Model/serial/timestamp)",
                Some("meta strip".to_string()),
            )
        })
    }
}

// ── orient/orientation-not-baked (warn) ─────────────────────────────────────

/// EXIF Orientation ≠ 1 — the pixels are not stored upright; some pipelines
/// ignore the tag. Fix: `auto-orient`.
pub struct OrientationNotBaked;

impl Rule for OrientationNotBaked {
    fn id(&self) -> &'static str {
        "orient/orientation-not-baked"
    }
    fn default_severity(&self) -> Severity {
        Severity::Warn
    }
    fn check(&self, target: &LintTarget) -> Option<Finding> {
        match target.exif_orientation() {
            Some(o) if o != 1 => Some(Finding::new(
                target.path().to_path_buf(),
                self.id(),
                self.default_severity(),
                format!("EXIF Orientation is {o} (pixels are not baked upright)"),
                Some("auto-orient".to_string()),
            )),
            _ => None,
        }
    }
}

// ── size/oversized-bytes (error) ────────────────────────────────────────────

/// The file exceeds the per-glob byte budget from config (the format-aware
/// `--maxkb`). Only fires when a `[[budget]]` `max_bytes` applies to this file.
/// Fix: `optimize`.
pub struct OversizedBytes;

impl Rule for OversizedBytes {
    fn id(&self) -> &'static str {
        "size/oversized-bytes"
    }
    fn default_severity(&self) -> Severity {
        Severity::Error
    }
    fn check(&self, target: &LintTarget) -> Option<Finding> {
        let budget = target.byte_budget()?;
        let actual = target.bytes().len() as u64;
        (actual > budget).then(|| {
            Finding::new(
                target.path().to_path_buf(),
                self.id(),
                self.default_severity(),
                format!("file is {actual} bytes, over the {budget}-byte budget"),
                Some("optimize".to_string()),
            )
        })
    }
}

// ── dims/oversized-dimensions (warn, opt-in via declared width) ──────────────

/// The natural width exceeds a **declared** intended width (a source-file
/// analogue of "properly size", honest without a page). Fires only when an
/// intended width is declared (config `max_intended_width` / `[[budget]]` /
/// `--max-intended-width`). Fix: `resize --max <W>`.
pub struct OversizedDimensions;

impl Rule for OversizedDimensions {
    fn id(&self) -> &'static str {
        "dims/oversized-dimensions"
    }
    fn default_severity(&self) -> Severity {
        Severity::Warn
    }
    fn check(&self, target: &LintTarget) -> Option<Finding> {
        let intended = target.intended_width()?;
        let width = target.info()?.width;
        (width > intended).then(|| {
            Finding::new(
                target.path().to_path_buf(),
                self.id(),
                self.default_severity(),
                format!("width {width}px exceeds the declared intended width {intended}px"),
                Some(format!("resize --max {intended}")),
            )
        })
    }
}

// ── color/wrong-colorspace (warn) ───────────────────────────────────────────

/// A needless 16-bit PNG for the web, or a CMYK JPEG (renders wrong / bloated).
/// Fix: `convert --format`.
pub struct WrongColorspace;

impl Rule for WrongColorspace {
    fn id(&self) -> &'static str {
        "color/wrong-colorspace"
    }
    fn default_severity(&self) -> Severity {
        Severity::Warn
    }
    fn check(&self, target: &LintTarget) -> Option<Finding> {
        let info = target.info()?;
        // Needless high bit depth (≥16 bpc) for the web.
        if info.bit_depth >= 16 {
            return Some(Finding::new(
                target.path().to_path_buf(),
                self.id(),
                self.default_severity(),
                format!("{}-bit channels are needless for the web", info.bit_depth),
                Some("convert --format".to_string()),
            ));
        }
        // CMYK JPEG (4-component SOF) — off-web colorspace.
        if info.format == ::image::ImageFormat::Jpeg
            && jpeg_component_count(target.bytes()) == Some(4)
        {
            return Some(Finding::new(
                target.path().to_path_buf(),
                self.id(),
                self.default_severity(),
                "CMYK JPEG (not an sRGB web colorspace)",
                Some("convert --format".to_string()),
            ));
        }
        None
    }
}

// ── color/missing-icc + color/unexpected-icc (info, opt-in) ──────────────────

/// No embedded ICC profile — colors may render inconsistently across viewers.
/// Opt-in (`info`). Fix: tag sRGB (no direct command; guidance in the message).
pub struct MissingIcc;

impl Rule for MissingIcc {
    fn id(&self) -> &'static str {
        "color/missing-icc"
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn default_enabled(&self) -> bool {
        false
    }
    fn check(&self, target: &LintTarget) -> Option<Finding> {
        // Only meaningful for a decoded image (a corrupt file is handled elsewhere).
        target.info()?;
        (!target.has_icc()).then(|| {
            Finding::new(
                target.path().to_path_buf(),
                self.id(),
                self.default_severity(),
                "no embedded ICC color profile (may render inconsistently); tag sRGB",
                None,
            )
        })
    }
}

/// A bulky embedded ICC profile worth stripping for the web. Opt-in (`info`).
/// Fix: `meta strip`.
pub struct UnexpectedIcc;

impl Rule for UnexpectedIcc {
    fn id(&self) -> &'static str {
        "color/unexpected-icc"
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn default_enabled(&self) -> bool {
        false
    }
    fn check(&self, target: &LintTarget) -> Option<Finding> {
        let len = target.icc_len()?;
        (len > ICC_BULKY_BYTES).then(|| {
            Finding::new(
                target.path().to_path_buf(),
                self.id(),
                self.default_severity(),
                format!("bulky {len}-byte ICC profile; strip it for the web"),
                Some("meta strip".to_string()),
            )
        })
    }
}

// ── format/animated-gif (warn) ──────────────────────────────────────────────

/// An animated GIF that should be a modern format (WebP/video). Fix:
/// `convert --format webp`.
pub struct AnimatedGif;

impl Rule for AnimatedGif {
    fn id(&self) -> &'static str {
        "format/animated-gif"
    }
    fn default_severity(&self) -> Severity {
        Severity::Warn
    }
    fn check(&self, target: &LintTarget) -> Option<Finding> {
        let info = target.info()?;
        if info.format == ::image::ImageFormat::Gif && gif_is_animated(target.bytes()) {
            Some(Finding::new(
                target.path().to_path_buf(),
                self.id(),
                self.default_severity(),
                "animated GIF (a modern format encodes far smaller)",
                Some("convert --format webp".to_string()),
            ))
        } else {
            None
        }
    }
}

// ── Byte-level sniffs (reuse the shipped decode; no new dependency) ──────────

/// The component count (`Nf`) from a JPEG's SOF marker — `3` for RGB/YCbCr, `4`
/// for CMYK/YCCK. `None` when the bytes are not a JPEG or no SOF is found.
fn jpeg_component_count(bytes: &[u8]) -> Option<u8> {
    if bytes.len() < 2 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return None;
    }
    let mut i = 2;
    while i + 4 <= bytes.len() {
        if bytes[i] != 0xFF {
            return None;
        }
        let m = bytes[i + 1];
        // Standalone markers (no length): RST0–7, TEM.
        if (0xD0..=0xD7).contains(&m) || m == 0x01 {
            i += 2;
            continue;
        }
        // EOI / SOS: no SOF beyond here.
        if m == 0xD9 || m == 0xDA {
            return None;
        }
        let len = ((bytes[i + 2] as usize) << 8) | (bytes[i + 3] as usize);
        if len < 2 {
            return None;
        }
        // SOF markers: 0xC0–0xCF except DHT(C4), JPG(C8), DAC(CC).
        if (0xC0..=0xCF).contains(&m) && m != 0xC4 && m != 0xC8 && m != 0xCC {
            // Segment: FF Cn, len(2), precision(1), height(2), width(2), Nf(1).
            return bytes.get(i + 9).copied();
        }
        i += 2 + len;
    }
    None
}

/// Whether a GIF has ≥2 frames (animated). Reuses the shipped `image` GIF
/// decoder (`gif` feature is on in both the default and lean builds); decodes at
/// most two frames. A decode error ⇒ `false` (a corrupt file is a separate
/// finding).
fn gif_is_animated(bytes: &[u8]) -> bool {
    use ::image::codecs::gif::GifDecoder;
    use ::image::AnimationDecoder;
    match GifDecoder::new(std::io::Cursor::new(bytes)) {
        Ok(dec) => dec.into_frames().take(2).count() >= 2,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::{Budget, LintConfig};
    use super::super::LintTarget;
    use super::*;
    use crate::source::Input;

    // ── Fixture builders (native; no ImageMagick, no committed binaries) ─────

    fn solid_jpeg() -> Vec<u8> {
        use ::image::{DynamicImage, ImageFormat, RgbImage};
        let img = RgbImage::from_pixel(8, 8, ::image::Rgb([120, 130, 140]));
        let mut buf = std::io::Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, ImageFormat::Jpeg)
            .unwrap();
        buf.into_inner()
    }

    fn png16() -> Vec<u8> {
        use ::image::{DynamicImage, ImageBuffer, ImageFormat};
        let img: ImageBuffer<::image::Rgb<u16>, Vec<u16>> =
            ImageBuffer::from_pixel(4, 4, ::image::Rgb([40000u16, 20000, 10000]));
        let mut buf = std::io::Cursor::new(Vec::new());
        DynamicImage::ImageRgb16(img)
            .write_to(&mut buf, ImageFormat::Png)
            .unwrap();
        buf.into_inner()
    }

    fn png8(w: u32) -> Vec<u8> {
        use ::image::{DynamicImage, ImageFormat, RgbImage};
        let img = RgbImage::from_pixel(w, 8, ::image::Rgb([1, 2, 3]));
        let mut buf = std::io::Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, ImageFormat::Png)
            .unwrap();
        buf.into_inner()
    }

    fn static_gif() -> Vec<u8> {
        use ::image::{DynamicImage, ImageFormat, RgbImage};
        let img = RgbImage::from_pixel(4, 4, ::image::Rgb([9, 9, 9]));
        let mut buf = std::io::Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, ImageFormat::Gif)
            .unwrap();
        buf.into_inner()
    }

    fn animated_gif() -> Vec<u8> {
        use ::image::codecs::gif::GifEncoder;
        use ::image::{Frame, RgbaImage};
        let mut buf = Vec::new();
        {
            let mut enc = GifEncoder::new(&mut buf);
            let f1 = Frame::new(RgbaImage::from_pixel(4, 4, ::image::Rgba([255, 0, 0, 255])));
            let f2 = Frame::new(RgbaImage::from_pixel(4, 4, ::image::Rgba([0, 255, 0, 255])));
            enc.encode_frames(vec![f1, f2]).unwrap();
        }
        buf
    }

    /// A JPEG carrying an EXIF APP1 with one IFD0 entry (little-endian TIFF).
    fn exif_jpeg(tag: u16, ty: u16, count: u32, value: [u8; 4]) -> Vec<u8> {
        let base = solid_jpeg();
        let mut tiff: Vec<u8> = Vec::new();
        tiff.extend_from_slice(&[0x49, 0x49, 0x2A, 0x00]); // II, 42
        tiff.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // IFD0 @ 8
        tiff.extend_from_slice(&[0x01, 0x00]); // 1 entry
        tiff.extend_from_slice(&tag.to_le_bytes());
        tiff.extend_from_slice(&ty.to_le_bytes());
        tiff.extend_from_slice(&count.to_le_bytes());
        tiff.extend_from_slice(&value);
        tiff.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // next IFD = 0

        let mut payload = Vec::new();
        payload.extend_from_slice(b"Exif\0\0");
        payload.extend_from_slice(&tiff);
        let seg_len = (payload.len() + 2) as u16;

        let mut out = Vec::new();
        out.extend_from_slice(&base[0..2]);
        out.push(0xFF);
        out.push(0xE1);
        out.extend_from_slice(&seg_len.to_be_bytes());
        out.extend_from_slice(&payload);
        out.extend_from_slice(&base[2..]);
        out
    }

    fn jpeg_with_orientation(o: u8) -> Vec<u8> {
        // tag 0x0112 (Orientation), type SHORT(3), count 1, value o.
        exif_jpeg(0x0112, 3, 1, [o, 0, 0, 0])
    }

    fn jpeg_with_make() -> Vec<u8> {
        // tag 0x010F (Make), type ASCII(2), count 3, value "AB\0".
        exif_jpeg(0x010F, 2, 3, [b'A', b'B', 0, 0])
    }

    /// A JPEG carrying an APP2 `ICC_PROFILE` segment of `payload_len` bytes.
    fn jpeg_with_icc(payload_len: usize) -> Vec<u8> {
        let base = solid_jpeg();
        let mut payload = Vec::new();
        payload.extend_from_slice(b"ICC_PROFILE\0");
        payload.push(1); // seq
        payload.push(1); // count
        payload.extend(std::iter::repeat_n(0xABu8, payload_len)); // profile bytes
        let seg_len = (payload.len() + 2) as u16;

        let mut out = Vec::new();
        out.extend_from_slice(&base[0..2]);
        out.push(0xFF);
        out.push(0xE2); // APP2
        out.extend_from_slice(&seg_len.to_be_bytes());
        out.extend_from_slice(&payload);
        out.extend_from_slice(&base[2..]);
        out
    }

    fn target(bytes: Vec<u8>) -> LintTarget {
        LintTarget::from_bytes("x", bytes)
    }

    /// Build a target with a config-resolved byte budget / intended width.
    fn configured_target(bytes: Vec<u8>, config: &LintConfig) -> LintTarget {
        LintTarget::from_input(
            &Input::Stdin {
                bytes,
                stem: "photo.png".into(),
            },
            config,
        )
    }

    // ── Per-rule positive + negative ─────────────────────────────────────────

    #[test]
    fn camera_metadata_fires_on_make_clean_on_none_and_distinct_from_gps() {
        assert!(CameraMetadata.check(&target(jpeg_with_make())).is_some());
        assert!(CameraMetadata.check(&target(solid_jpeg())).is_none());
        // An orientation-only EXIF is not camera-identifying.
        assert!(CameraMetadata
            .check(&target(jpeg_with_orientation(1)))
            .is_none());
    }

    #[test]
    fn orientation_not_baked_fires_on_6_clean_on_1() {
        let f = OrientationNotBaked
            .check(&target(jpeg_with_orientation(6)))
            .expect("orientation 6 fires");
        assert_eq!(f.rule(), "orient/orientation-not-baked");
        assert_eq!(f.fix(), Some("auto-orient"));
        assert!(OrientationNotBaked
            .check(&target(jpeg_with_orientation(1)))
            .is_none());
    }

    #[test]
    fn oversized_bytes_fires_over_budget_clean_under_and_none_without_budget() {
        // A budget of 10 bytes → any real image is over.
        let cfg = LintConfig {
            budgets: vec![Budget {
                glob: "*".into(),
                max_bytes: Some(10),
                max_intended_width: None,
            }],
            ..Default::default()
        };
        let t = configured_target(png8(4), &cfg);
        assert!(OversizedBytes.check(&t).is_some(), "over a 10-byte budget");

        // A generous budget → clean.
        let cfg_big = LintConfig {
            budgets: vec![Budget {
                glob: "*".into(),
                max_bytes: Some(10_000_000),
                max_intended_width: None,
            }],
            ..Default::default()
        };
        assert!(OversizedBytes
            .check(&configured_target(png8(4), &cfg_big))
            .is_none());

        // No budget configured → never fires.
        assert!(OversizedBytes.check(&target(png8(4))).is_none());
    }

    #[test]
    fn oversized_dimensions_fires_over_declared_width_silent_when_undeclared() {
        let cfg = LintConfig {
            max_intended_width: Some(2),
            ..Default::default()
        };
        // png8(10) is 10px wide > 2 → fires.
        let f = OversizedDimensions
            .check(&configured_target(png8(10), &cfg))
            .expect("width over intended fires");
        assert_eq!(f.fix(), Some("resize --max 2"));
        // No declared width → silent.
        assert!(OversizedDimensions.check(&target(png8(10))).is_none());
    }

    #[test]
    fn wrong_colorspace_fires_on_16bit_clean_on_8bit() {
        assert!(WrongColorspace.check(&target(png16())).is_some());
        assert!(WrongColorspace.check(&target(png8(4))).is_none());
    }

    #[test]
    fn jpeg_component_count_reads_nf() {
        // A crafted SOF0 declaring 4 components (CMYK).
        let cmyk_sof = [
            0xFF, 0xD8, // SOI
            0xFF, 0xC0, // SOF0
            0x00, 0x14, // len
            0x08, // precision
            0x00, 0x01, // height
            0x00, 0x01, // width
            0x04, // Nf = 4
        ];
        assert_eq!(jpeg_component_count(&cmyk_sof), Some(4));
        // A real RGB JPEG reports 3.
        assert_eq!(jpeg_component_count(&solid_jpeg()), Some(3));
        // Not a JPEG.
        assert_eq!(jpeg_component_count(&png8(4)), None);
    }

    #[test]
    fn missing_icc_fires_without_profile_clean_with_one() {
        assert!(MissingIcc.check(&target(solid_jpeg())).is_some());
        assert!(MissingIcc.check(&target(jpeg_with_icc(2000))).is_none());
    }

    #[test]
    fn unexpected_icc_fires_on_bulky_profile_clean_on_small_or_none() {
        assert!(UnexpectedIcc.check(&target(jpeg_with_icc(5000))).is_some());
        assert!(UnexpectedIcc.check(&target(jpeg_with_icc(500))).is_none());
        assert!(UnexpectedIcc.check(&target(solid_jpeg())).is_none());
    }

    #[test]
    fn animated_gif_fires_on_two_frames_clean_on_static() {
        let f = AnimatedGif
            .check(&target(animated_gif()))
            .expect("2-frame gif fires");
        assert_eq!(f.fix(), Some("convert --format webp"));
        assert!(AnimatedGif.check(&target(static_gif())).is_none());
        // A non-gif is never animated-gif.
        assert!(AnimatedGif.check(&target(png8(4))).is_none());
    }

    #[test]
    fn every_rule_carries_a_runnable_or_noted_fix() {
        // Rules that name a command expose a fix fragment; missing-icc is a note.
        assert_eq!(
            CameraMetadata
                .check(&target(jpeg_with_make()))
                .unwrap()
                .fix(),
            Some("meta strip")
        );
        assert_eq!(MissingIcc.check(&target(solid_jpeg())).unwrap().fix(), None);
    }
}
