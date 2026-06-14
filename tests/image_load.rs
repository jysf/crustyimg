//! Integration tests for the canonical `Image` load/decode + metadata capture
//! (SPEC-002). Fixtures are generated natively at test time (see
//! `tests/common`). Error assertions match on `ImageError` variants, not on
//! message strings.

mod common;

use common::{gradient_jpeg, jpeg_with_exif, rgba_png, solid_png};
use crustyimg::error::ImageError;
use crustyimg::image::Image;
use image::{ColorType, ImageFormat};

#[test]
fn load_png_from_bytes_reports_dimensions_and_format() {
    let png = solid_png(7, 5, [10, 20, 30]);
    let img = Image::from_bytes(&png).expect("valid PNG should decode");
    assert_eq!(img.width(), 7);
    assert_eq!(img.height(), 5);
    assert_eq!(img.source_format(), ImageFormat::Png);
}

#[test]
fn load_png_from_path_reports_dimensions_and_format() {
    let png = solid_png(7, 5, [10, 20, 30]);
    // Unique temp path under the system temp dir; no `tempfile` crate.
    let mut path = std::env::temp_dir();
    path.push(format!(
        "crustyimg-spec002-{}-{}.png",
        std::process::id(),
        line!()
    ));
    std::fs::write(&path, &png).expect("write temp fixture");

    let result = Image::load(&path);
    let _ = std::fs::remove_file(&path); // best-effort cleanup

    let img = result.expect("valid PNG path should load");
    assert_eq!(img.width(), 7);
    assert_eq!(img.height(), 5);
    assert_eq!(img.source_format(), ImageFormat::Png);
}

#[test]
fn load_jpeg_from_bytes_reports_dimensions_and_format() {
    let jpeg = gradient_jpeg(16, 9);
    let img = Image::from_bytes(&jpeg).expect("valid JPEG should decode");
    assert_eq!(img.width(), 16);
    assert_eq!(img.height(), 9);
    assert_eq!(img.source_format(), ImageFormat::Jpeg);
}

#[test]
fn bogus_bytes_return_typed_error_not_panic() {
    let err = Image::from_bytes(b"not an image at all").expect_err("bogus bytes must error");
    assert!(
        matches!(err, ImageError::UnsupportedFormat | ImageError::Decode(_)),
        "expected UnsupportedFormat or Decode, got {err:?}"
    );
}

#[test]
fn truncated_png_returns_decode_error() {
    let png = solid_png(8, 8, [1, 2, 3]);
    // Keep the PNG signature + a little header, but cut the body.
    let truncated = &png[..20.min(png.len())];
    let err = Image::from_bytes(truncated).expect_err("truncated PNG must error");
    assert!(
        matches!(err, ImageError::Decode(_)),
        "expected Decode, got {err:?}"
    );
}

#[test]
fn missing_file_returns_io_error() {
    let err = Image::load("/no/such/crustyimg-test-file.png").expect_err("missing file must error");
    assert!(matches!(err, ImageError::Io(_)), "expected Io, got {err:?}");
}

#[test]
fn jpeg_with_exif_captures_metadata_bundle() {
    let jpeg = jpeg_with_exif(16, 9);
    let img = Image::from_bytes(&jpeg).expect("EXIF-bearing JPEG should decode");
    let meta = img.metadata().expect("metadata bundle should be present");
    assert!(meta.has_exif(), "EXIF segment should be captured");
    assert!(img.info().has_exif, "ImageInfo.has_exif should be true");
}

#[test]
fn plain_png_has_no_metadata() {
    let png = solid_png(4, 4, [5, 6, 7]);
    let img = Image::from_bytes(&png).expect("plain PNG should decode");
    match img.metadata() {
        None => {}
        Some(m) => {
            assert!(!m.has_exif());
            assert!(!m.has_icc());
        }
    }
    assert!(!img.info().has_exif);
    assert!(!img.info().has_icc);
}

#[test]
fn info_fields_correct_for_rgb8() {
    let png = solid_png(4, 3, [1, 2, 3]);
    let info = Image::from_bytes(&png)
        .expect("RGB PNG should decode")
        .info();
    assert_eq!(info.width, 4);
    assert_eq!(info.height, 3);
    assert_eq!(info.format, ImageFormat::Png);
    assert_eq!(info.color_type, ColorType::Rgb8);
    assert_eq!(info.bit_depth, 8);
    assert!(!info.has_alpha);
    assert_eq!(info.byte_len, 4 * 3 * 3);
}

#[test]
fn info_fields_correct_for_rgba8() {
    let png = rgba_png(2, 2);
    let info = Image::from_bytes(&png)
        .expect("RGBA PNG should decode")
        .info();
    assert_eq!(info.color_type, ColorType::Rgba8);
    assert!(info.has_alpha);
    assert_eq!(info.bit_depth, 8);
    assert_eq!(info.byte_len, 2 * 2 * 4);
}
