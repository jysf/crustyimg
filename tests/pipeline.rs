//! Integration tests for the `Pipeline` executor and `Operation` trait
//! (SPEC-003, SPEC-010). Exercises only public crate exports.
//!
//! Fixtures are built natively from `::image::RgbaImage::from_fn` —
//! no committed binary files, no ImageMagick shell-out (AGENTS.md §12).

use std::collections::BTreeMap;

use ::image::{DynamicImage, ImageFormat, RgbaImage};
use crustyimg::image::Image;
use crustyimg::operation::{Identity, Invert, OperationParams, Resize};
use crustyimg::pipeline::Pipeline;

// ─── fixture helper ─────────────────────────────────────────────────────────

/// Build a small RGBA `Image` in memory — no file I/O.
fn make_image(w: u32, h: u32) -> Image {
    let buf = RgbaImage::from_fn(w, h, |x, y| {
        ::image::Rgba([(x * 25 + 10) as u8, (y * 25 + 10) as u8, 80, 200])
    });
    Image::from_parts(DynamicImage::ImageRgba8(buf), ImageFormat::Png, None)
}

// ─── integration tests ───────────────────────────────────────────────────────

#[test]
fn public_pipeline_inverts_via_crate_api() {
    let img = make_image(4, 4);
    let original_raw = img.pixels().to_rgba8().into_raw();

    // Single invert: RGB channels should be complemented; alpha unchanged.
    let inverted = Pipeline::new()
        .push(Box::new(Invert))
        .run(make_image(4, 4))
        .unwrap();
    let inv_raw = inverted.pixels().to_rgba8().into_raw();

    // Spot-check pixel (0,0): [10,10,80,200] → [245,245,175,200]
    assert_eq!(inv_raw[0], 255 - original_raw[0]);
    assert_eq!(inv_raw[1], 255 - original_raw[1]);
    assert_eq!(inv_raw[2], 255 - original_raw[2]);
    assert_eq!(inv_raw[3], original_raw[3]); // alpha unchanged

    // Double invert: must round-trip to original.
    let round_tripped = Pipeline::new()
        .push(Box::new(Invert))
        .push(Box::new(Invert))
        .run(img)
        .unwrap();
    assert_eq!(
        round_tripped.pixels().to_rgba8().into_raw(),
        original_raw,
        "double Invert must be identity"
    );
}

#[test]
fn empty_pipeline_is_identity_via_crate_api() {
    let img = make_image(3, 3);
    let original_raw = img.pixels().to_rgba8().into_raw();
    let result = Pipeline::new().run(img).unwrap();
    assert_eq!(
        result.pixels().to_rgba8().into_raw(),
        original_raw,
        "empty pipeline must return image unchanged"
    );
}

#[test]
fn operation_and_pipeline_sources_do_no_disk_io() {
    // Structural guard (SPEC-003 § Failing Tests / `decode-once-no-per-op-disk`):
    // Read the library source files as text and assert the NON-TEST code
    // references none of the disk-I/O identifiers.
    //
    // Heuristic: split on `#[cfg(test)]` (the line that starts the test
    // module in both files) and inspect only the text BEFORE that marker.
    // This is sound because both src/operation/mod.rs and src/pipeline/mod.rs
    // place their test module at the end, after a `#[cfg(test)]` guard.
    // Any future addition of disk I/O in the library code (above the test
    // block) would be caught here.

    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set in tests");

    let forbidden = [
        "std::fs",
        "std::io::",
        "std::path",
        "File",
        "OpenOptions",
        "read_to_string",
        "Path",
    ];

    let paths = [
        format!("{manifest_dir}/src/operation/mod.rs"),
        format!("{manifest_dir}/src/pipeline/mod.rs"),
    ];

    for path in &paths {
        let source =
            std::fs::read_to_string(path).unwrap_or_else(|e| panic!("could not read {path}: {e}"));

        // Take only the portion before the `#[cfg(test)]` marker that opens
        // the test module — tests themselves may reference paths for the guard.
        let library_code = source.split("#[cfg(test)]").next().unwrap_or(&source);

        // Strip comment lines (lines whose first non-whitespace chars are
        // `//`). Doc comments (`//!`, `///`) mention the forbidden tokens
        // intentionally (to document constraints), and the heuristic must not
        // fire on them. This is the "brittle heuristic" note in the spec:
        // we filter comments rather than matching the whole file minus tests.
        let non_comment_code: String = library_code
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                !trimmed.starts_with("//")
            })
            .collect::<Vec<_>>()
            .join("\n");

        for &token in &forbidden {
            assert!(
                !non_comment_code.contains(token),
                "library code (non-comment) in {path} must not reference `{token}` \
                 (decode-once-no-per-op-disk)"
            );
        }
    }
}

#[test]
fn identity_op_leaves_pixels_and_format_intact() {
    // Extra coverage: verify Identity via the public API works correctly.
    let img = make_image(2, 2);
    let original_raw = img.pixels().to_rgba8().into_raw();
    let original_format = img.source_format();
    let result = Pipeline::new().push(Box::new(Identity)).run(img).unwrap();
    assert_eq!(result.pixels().to_rgba8().into_raw(), original_raw);
    assert_eq!(result.source_format(), original_format);
}

#[test]
fn resize_runs_through_pipeline() {
    // Push a Resize (exact 8×8) into a Pipeline over a 16×16 fixture → 8×8.
    let params = OperationParams::from_map({
        let mut m = BTreeMap::new();
        m.insert("mode".to_owned(), toml::Value::String("exact".into()));
        m.insert("width".to_owned(), toml::Value::Integer(8));
        m.insert("height".to_owned(), toml::Value::Integer(8));
        m
    });
    let op = Resize::from_params(&params).expect("from_params should succeed for exact 8×8");
    let img = make_image(16, 16);

    let result = Pipeline::new().push(Box::new(op)).run(img).unwrap();

    assert_eq!(result.pixels().width(), 8, "output width must be 8");
    assert_eq!(result.pixels().height(), 8, "output height must be 8");
}
