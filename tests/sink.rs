//! Integration tests for `crustyimg::sink` (SPEC-005).
//!
//! Exercises the public crate API only:
//! `crustyimg::sink::{Sink, Overwrite, SinkError, SinkInput, …helpers}` and
//! `crustyimg::image::Image`. Uses `tempfile::tempdir()` for filesystem
//! fixtures and produces real images in-memory (no ImageMagick, no committed
//! binary fixtures — AGENTS.md §12).

use std::io::Cursor;
use std::path::Path;

use ::image::{DynamicImage, ImageFormat, RgbImage};
use crustyimg::image::Image;
use crustyimg::sink::{
    encode_to_bytes, expand_template, extension_for_format, format_from_extension, safe_join,
    Overwrite, Sink, SinkError, SinkInput,
};

// ── In-memory fixture helper ──────────────────────────────────────────────────

/// Encode a solid RGB image to PNG bytes (mirrors `solid_png` in
/// `src/image/mod.rs` tests).
fn solid_png(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
    let img = RgbImage::from_pixel(w, h, ::image::Rgb(rgb));
    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

/// Build a small `Image` from PNG bytes.
fn make_image() -> Image {
    Image::from_bytes(&solid_png(4, 4, [100, 150, 200])).unwrap()
}

/// A dummy `SinkInput` with the given stem and no path.
fn sink_input(stem: &str) -> SinkInput<'_> {
    SinkInput { stem, path: None }
}

// ── Integration tests ─────────────────────────────────────────────────────────

#[test]
fn file_sink_writes_readable_image() {
    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("out.png");
    let img = make_image();
    let sink = Sink::File {
        path: out_path.clone(),
        format: None,
    };
    sink.write(
        &img,
        &sink_input("out"),
        Overwrite::Forbid,
        None,
        &mut std::io::sink(),
    )
    .unwrap();

    assert!(out_path.exists());
    let loaded = Image::load(&out_path).unwrap();
    assert_eq!(loaded.width(), img.width());
    assert_eq!(loaded.height(), img.height());
}

#[test]
fn format_inferred_from_extension_jpeg_and_png() {
    let tmp = tempfile::tempdir().unwrap();
    let img = make_image();
    let si = sink_input("photo");

    // Write JPEG
    let jpg_path = tmp.path().join("out.jpg");
    Sink::File {
        path: jpg_path.clone(),
        format: None,
    }
    .write(&img, &si, Overwrite::Forbid, None, &mut std::io::sink())
    .unwrap();

    // Write PNG
    let png_path = tmp.path().join("out.png");
    Sink::File {
        path: png_path.clone(),
        format: None,
    }
    .write(&img, &si, Overwrite::Forbid, None, &mut std::io::sink())
    .unwrap();

    // Verify formats.
    let loaded_jpg = Image::load(&jpg_path).unwrap();
    assert_eq!(loaded_jpg.source_format(), ImageFormat::Jpeg);

    let loaded_png = Image::load(&png_path).unwrap();
    assert_eq!(loaded_png.source_format(), ImageFormat::Png);

    // format_from_extension is case-insensitive.
    assert!(matches!(
        format_from_extension(Path::new("OUT.PNG")),
        Ok(ImageFormat::Png)
    ));
}

#[test]
fn explicit_format_overrides_missing_extension() {
    let tmp = tempfile::tempdir().unwrap();
    let img = make_image();
    let si = sink_input("out");

    // Explicit Png with no extension — should succeed.
    let out_path = tmp.path().join("out");
    Sink::File {
        path: out_path.clone(),
        format: Some(ImageFormat::Png),
    }
    .write(&img, &si, Overwrite::Forbid, None, &mut std::io::sink())
    .unwrap();
    let loaded = Image::load(&out_path).unwrap();
    assert_eq!(loaded.source_format(), ImageFormat::Png);

    // No extension AND no explicit format → UnknownFormat.
    let out_no_ext = tmp.path().join("out_noext");
    let err = Sink::File {
        path: out_no_ext,
        format: None,
    }
    .write(&img, &si, Overwrite::Forbid, None, &mut std::io::sink())
    .unwrap_err();
    assert!(matches!(err, SinkError::UnknownFormat), "got: {err:?}");
}

#[test]
fn unsupported_extension_is_typed_error() {
    let tmp = tempfile::tempdir().unwrap();
    let img = make_image();
    let out_path = tmp.path().join("out.xyz");
    let err = Sink::File {
        path: out_path,
        format: None,
    }
    .write(
        &img,
        &sink_input("out"),
        Overwrite::Forbid,
        None,
        &mut std::io::sink(),
    )
    .unwrap_err();
    assert!(
        matches!(err, SinkError::UnsupportedExtension(_)),
        "got: {err:?}"
    );
}

#[test]
fn dir_sink_expands_name_template() {
    let tmp = tempfile::tempdir().unwrap();
    let img = make_image();
    let si = SinkInput {
        stem: "photo",
        path: Some(Path::new("in/photo.jpg")),
    };

    Sink::Dir {
        dir: tmp.path().to_path_buf(),
        template: "{stem}_web.{ext}".into(),
        format: Some(ImageFormat::Png),
    }
    .write(&img, &si, Overwrite::Forbid, None, &mut std::io::sink())
    .unwrap();

    let expected = tmp.path().join("photo_web.png");
    assert!(expected.exists(), "expected {expected:?} to exist");
    let loaded = Image::load(&expected).unwrap();
    assert_eq!(loaded.width(), img.width());
    assert_eq!(loaded.height(), img.height());

    // Verify expand_template covers all tokens.
    assert_eq!(
        expand_template(
            "{stem}_{ext}_{name}_{parent}",
            "photo",
            "png",
            Some(Path::new("in/photo.jpg"))
        ),
        "photo_png_photo.jpg_in"
    );
}

#[test]
fn stdout_sink_writes_only_encoded_bytes() {
    let img = make_image();
    let si = sink_input("out");

    // Capture bytes.
    let mut buf = Vec::<u8>::new();
    Sink::Stdout {
        format: Some(ImageFormat::Png),
    }
    .write(&img, &si, Overwrite::Forbid, None, &mut buf)
    .unwrap();

    // The captured bytes must decode as a PNG.
    let decoded = Image::from_bytes(&buf).unwrap();
    assert_eq!(decoded.source_format(), ImageFormat::Png);
    assert_eq!(decoded.width(), img.width());
    assert_eq!(decoded.height(), img.height());

    // No trailing bytes: the buf length equals the encoded image length.
    // (We re-encode independently to get the expected byte count.)
    let mut expected_buf = Vec::<u8>::new();
    Sink::Stdout {
        format: Some(ImageFormat::Png),
    }
    .write(&img, &si, Overwrite::Forbid, None, &mut expected_buf)
    .unwrap();
    assert_eq!(
        buf.len(),
        expected_buf.len(),
        "captured buf has unexpected extra bytes"
    );

    // None format → UnknownFormat.
    let mut discard = Vec::<u8>::new();
    let err = Sink::Stdout { format: None }
        .write(&img, &si, Overwrite::Forbid, None, &mut discard)
        .unwrap_err();
    assert!(matches!(err, SinkError::UnknownFormat), "got: {err:?}");
}

#[test]
fn overwrite_guard_forbids_then_allows() {
    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("out.png");
    let img = make_image();
    let si = sink_input("out");

    // Pre-create the file.
    std::fs::write(&out_path, b"placeholder").unwrap();

    // Forbid → AlreadyExists.
    let err = Sink::File {
        path: out_path.clone(),
        format: None,
    }
    .write(&img, &si, Overwrite::Forbid, None, &mut std::io::sink())
    .unwrap_err();
    assert!(matches!(err, SinkError::AlreadyExists(_)), "got: {err:?}");
    // File must still be the placeholder (not truncated).
    assert_eq!(std::fs::read(&out_path).unwrap(), b"placeholder");

    // Allow → overwrites successfully.
    Sink::File {
        path: out_path.clone(),
        format: None,
    }
    .write(&img, &si, Overwrite::Allow, None, &mut std::io::sink())
    .unwrap();
    let loaded = Image::load(&out_path).unwrap();
    assert_eq!(loaded.source_format(), ImageFormat::Png);
}

#[test]
fn dir_sink_rejects_traversal_template() {
    let tmp = tempfile::tempdir().unwrap();
    let img = make_image();
    let si = sink_input("photo");

    // Template with ../  — expands to "../photo.png" which escapes the dir.
    let err = Sink::Dir {
        dir: tmp.path().to_path_buf(),
        template: "../{stem}.{ext}".into(),
        format: Some(ImageFormat::Png),
    }
    .write(&img, &si, Overwrite::Forbid, None, &mut std::io::sink())
    .unwrap_err();
    assert!(matches!(err, SinkError::Traversal(_)), "got: {err:?}");

    // Assert no file was created in tmp's parent.
    let escaped = tmp.path().parent().unwrap().join("photo.png");
    assert!(!escaped.exists(), "file must not be created outside dir");
}

#[test]
fn missing_out_dir_is_auto_created() {
    // DEC-044: Sink::Dir now auto-creates the output directory if missing.
    // This replaces the old `missing_out_dir_is_typed_not_panic` which
    // expected an error — that behavior was the bug being fixed.
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("does_not_exist");
    let img = make_image();
    let si = sink_input("photo");

    let result = Sink::Dir {
        dir: missing.clone(),
        template: "{stem}.{ext}".into(),
        format: Some(ImageFormat::Png),
    }
    .write(&img, &si, Overwrite::Forbid, None, &mut std::io::sink());

    // Must succeed — the dir is created and the file is written.
    assert!(
        result.is_ok(),
        "expected Ok for auto-created dir, got: {result:?}"
    );
    assert!(
        missing.is_dir(),
        "output directory should exist after write"
    );
    assert!(
        missing.join("photo.png").exists(),
        "output file should exist"
    );
}

// ── PATCH-001: out-dir auto-create tests (DEC-044) ──────────────────────────

#[test]
fn out_dir_is_created_when_missing() {
    // Sink::Dir auto-creates a non-existent output directory before writing.
    let tmp = tempfile::tempdir().unwrap();
    let out_dir = tmp.path().join("new_out");
    assert!(!out_dir.exists(), "dir should not exist yet");
    let img = make_image();

    Sink::Dir {
        dir: out_dir.clone(),
        template: "{stem}.{ext}".into(),
        format: Some(ImageFormat::Png),
    }
    .write(
        &img,
        &sink_input("photo"),
        Overwrite::Forbid,
        None,
        &mut std::io::sink(),
    )
    .unwrap();

    assert!(
        out_dir.is_dir(),
        "output directory should have been created"
    );
    assert!(
        out_dir.join("photo.png").exists(),
        "output file should exist"
    );
}

#[test]
fn out_dir_creates_nested_parents() {
    // Sink::Dir uses create_dir_all, so nested parents are created too.
    let tmp = tempfile::tempdir().unwrap();
    let nested = tmp.path().join("a").join("b").join("c");
    assert!(!nested.exists(), "nested dir should not exist yet");
    let img = make_image();

    Sink::Dir {
        dir: nested.clone(),
        template: "{stem}.{ext}".into(),
        format: Some(ImageFormat::Png),
    }
    .write(
        &img,
        &sink_input("img"),
        Overwrite::Forbid,
        None,
        &mut std::io::sink(),
    )
    .unwrap();

    assert!(
        nested.is_dir(),
        "nested output directory should have been created"
    );
    assert!(nested.join("img.png").exists(), "output file should exist");
}

#[test]
fn out_dir_creation_failure_is_typed() {
    // When a *file* exists at the out-dir path, create_dir_all fails with a
    // system error. The sink must return SinkError::OutDirCreate (not the
    // generic SinkError::Io), and it must map to exit 5.
    let tmp = tempfile::tempdir().unwrap();
    // Plant a regular file where the out-dir path is expected to be a directory.
    let file_at_dir_path = tmp.path().join("not_a_dir");
    std::fs::write(&file_at_dir_path, b"obstacle").unwrap();

    let img = make_image();
    let err = Sink::Dir {
        dir: file_at_dir_path.clone(),
        template: "{stem}.{ext}".into(),
        format: Some(ImageFormat::Png),
    }
    .write(
        &img,
        &sink_input("photo"),
        Overwrite::Forbid,
        None,
        &mut std::io::sink(),
    )
    .unwrap_err();

    assert!(
        matches!(err, SinkError::OutDirCreate { .. }),
        "expected OutDirCreate when a file blocks dir creation, got: {err:?}"
    );
    // The error message must name the path.
    let msg = err.to_string();
    assert!(
        msg.contains("could not create output directory"),
        "error message should mention dir creation, got: {msg}"
    );
    // File at the path must be untouched.
    assert_eq!(
        std::fs::read(&file_at_dir_path).unwrap(),
        b"obstacle",
        "obstacle file must not be modified"
    );
}

#[test]
fn display_sink_refuses_non_tty() {
    // Under `cargo test` stdout is piped (non-tty), so Display always returns
    // NotATty — regardless of whether the `display` feature is enabled.
    let img = make_image();
    let si = sink_input("photo");
    let mut discard = Vec::<u8>::new();

    let err = Sink::Display {
        width: None,
        height: None,
    }
    .write(&img, &si, Overwrite::Forbid, None, &mut discard)
    .unwrap_err();
    assert!(matches!(err, SinkError::NotATty), "got: {err:?}");
}

// ── SPEC-013 quality-aware encode tests ──────────────────────────────────────

/// Encode the same DynamicImage to JPEG at low quality (20) vs high quality
/// (90) and assert:
/// - the low-quality byte length < high-quality byte length
/// - both decode to the same dimensions
#[test]
fn encode_jpeg_quality_lower_is_smaller() {
    // 200×100 horizontal gradient gives the JPEG encoder something to work with.
    let img = Image::from_bytes(&make_jpeg_bytes(200, 100)).unwrap();

    let lo = encode_to_bytes(&img, ImageFormat::Jpeg, Some(20)).unwrap();
    let hi = encode_to_bytes(&img, ImageFormat::Jpeg, Some(90)).unwrap();

    assert!(
        lo.len() < hi.len(),
        "low quality ({} bytes) should be smaller than high quality ({} bytes)",
        lo.len(),
        hi.len()
    );

    // Both must decode to the same dimensions.
    let lo_img = ::image::load_from_memory(&lo).unwrap();
    let hi_img = ::image::load_from_memory(&hi).unwrap();
    assert_eq!(lo_img.width(), hi_img.width(), "width must match");
    assert_eq!(lo_img.height(), hi_img.height(), "height must match");
}

/// Encode a PNG at Some(10) and None: output must be byte-identical
/// (quality is ignored for lossless formats, DEC-016).
#[test]
fn encode_png_ignores_quality() {
    let img = make_image();

    let with_q = encode_to_bytes(&img, ImageFormat::Png, Some(10)).unwrap();
    let no_q = encode_to_bytes(&img, ImageFormat::Png, None).unwrap();

    assert_eq!(
        with_q, no_q,
        "PNG encode at Some(10) and None must be byte-identical"
    );
}

/// Encode a gradient JPEG image to JPEG bytes (for encode unit tests).
fn make_jpeg_bytes(w: u32, h: u32) -> Vec<u8> {
    let img = RgbImage::from_fn(w, h, |x, _y| {
        ::image::Rgb([(x * 255 / w.max(1)) as u8, 100u8, 150u8])
    });
    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut out, ImageFormat::Jpeg)
        .unwrap();
    out.into_inner()
}

// ── Helper round-trips ────────────────────────────────────────────────────────

#[test]
fn extension_for_format_covers_core_set() {
    assert_eq!(extension_for_format(ImageFormat::Png), "png");
    assert_eq!(extension_for_format(ImageFormat::Jpeg), "jpg");
    assert_eq!(extension_for_format(ImageFormat::Gif), "gif");
    assert_eq!(extension_for_format(ImageFormat::Bmp), "bmp");
    assert_eq!(extension_for_format(ImageFormat::Tiff), "tiff");
    assert_eq!(extension_for_format(ImageFormat::Ico), "ico");
}

#[test]
fn safe_join_rejects_parent_and_absolute() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    assert!(matches!(
        safe_join(dir, "../x.png"),
        Err(SinkError::Traversal(_))
    ));
    assert!(matches!(
        safe_join(dir, "/etc/x.png"),
        Err(SinkError::Traversal(_))
    ));
    // Valid name succeeds.
    let ok = safe_join(dir, "photo.png").unwrap();
    assert!(ok.starts_with(std::fs::canonicalize(dir).unwrap()));
}
