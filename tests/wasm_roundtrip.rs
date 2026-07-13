//! The WASM round-trip (SPEC-072) — the load-bearing proof of the whole wave.
//!
//! These run under `just wasm-test`
//! (`cargo test --target wasm32-unknown-unknown --test wasm_roundtrip`, via the
//! `wasm-bindgen-test-runner` registered in `.cargo/config.toml` — NOT
//! `wasm-pack test`, which hardcodes `--tests` and would drag every CLI-driving
//! native integration test into the wasm build). They do not run under the native
//! `cargo test`: the wasm half of this file is `cfg(target_arch = "wasm32")`, so a
//! native `cargo test` compiles it to nothing.
//!
//! The bar this file exists to clear: **a green wasm compile is not a working
//! wasm build.** "It compiles to wasm32" is exactly the kind of unearned verdict
//! this project keeps catching, so every test below drives the real
//! `wasm-bindgen` surface over real bytes inside a real wasm VM and asserts on
//! the OUTPUT — the encoded bytes decode, and they decode to the dimensions and
//! format we asked for.
//!
//! The native half of the same guarantee (native AVIF decode and encode both
//! survive the target-gating) is asserted at the bottom, and DOES run under
//! `cargo test`.
//!
//! SPEC-073/DEC-065 added the AVIF **encode** tests: the shipped wasm artifact is
//! built `--features avif` (which `just wasm-test` passes), so `rav1e` really runs
//! in the VM. AVIF **decode** stays out — which is why the AVIF assertions here
//! sniff the output container instead of decoding it back.

// ─────────────────────────────────────────────────────────────────────────────
// WASM: the real decode → transform → encode round-trip
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(target_arch = "wasm32")]
mod wasm {
    use crustyimg::wasm::{info, optimize, transform};
    use wasm_bindgen_test::wasm_bindgen_test;

    /// The SVG fixture the native suite uses (SPEC-060) — a 40×30 plain-text SVG.
    /// `include_bytes!` bakes it into the `.wasm`: there is no filesystem to read
    /// it from at test time, which is the point.
    const SVG: &[u8] = include_bytes!("fixtures/svg/rect_text_40x30.svg");

    /// The AVIF fixture the native suite uses (SPEC-058) — a 16×16 solid AVIF.
    const AVIF: &[u8] = include_bytes!("fixtures/avif/solid_16x16.avif");

    /// Build a small PNG in-process (the native suite's convention: generate
    /// fixtures, never shell out). A 64×48 two-tone image — enough for a resize to
    /// be observable and for the analysis layer to have something to look at.
    fn png_64x48() -> Vec<u8> {
        use image::{ImageEncoder, Rgba, RgbaImage};
        let mut img = RgbaImage::new(64, 48);
        for (x, y, px) in img.enumerate_pixels_mut() {
            *px = if (x / 8 + y / 8) % 2 == 0 {
                Rgba([220, 40, 40, 255])
            } else {
                Rgba([30, 60, 200, 255])
            };
        }
        let mut out = Vec::new();
        image::codecs::png::PngEncoder::new(&mut out)
            .write_image(img.as_raw(), 64, 48, image::ExtendedColorType::Rgba8)
            .expect("encode fixture png");
        out
    }

    /// A recipe that halves the fixture: 64×48 → 32×24. Byte-for-byte the same TOML
    /// schema the CLI reads off disk (DEC-005) — `version = "1"` (a string), an
    /// `[[step]]` array-of-tables, and params resolved through the operation
    /// registry. That equivalence is what the wasm build is FOR: a recipe tuned in
    /// the terminal has to replay in the browser.
    const RESIZE_RECIPE: &str = r#"
version = "1"

[[step]]
op = "resize"
mode = "exact"
width = 32
height = 24
"#;

    /// A recipe that transforms nothing — the control for the resize assertion.
    const IDENTITY_RECIPE: &str = r#"
version = "1"

[[step]]
op = "identity"
"#;

    /// PNG + a resize recipe → `transform` → the RETURNED BYTES decode to the
    /// resized dimensions. Not "the call returned Ok" — the output is fed back
    /// through `info` (a real decode inside the wasm VM) and its dimensions
    /// asserted. This is the spec's load-bearing test.
    #[wasm_bindgen_test]
    fn transform_png_resize_roundtrip() {
        let src = png_64x48();

        let out = transform(&src, RESIZE_RECIPE, "png").expect("transform should succeed");

        assert!(!out.is_empty(), "transform returned no bytes");
        // The encoded bytes must be a real PNG that really decodes...
        let back = info(&out).expect("output bytes should decode");
        // ...to the dimensions the recipe asked for.
        assert_eq!(back.width(), 32, "resized width");
        assert_eq!(back.height(), 24, "resized height");
        assert_eq!(back.format(), "png", "output format");
    }

    /// The transform is a real transform: the same source encoded WITHOUT the
    /// resize keeps its original dimensions, so the assertion above is testing the
    /// pipeline, not a coincidence of the encoder.
    #[wasm_bindgen_test]
    fn transform_without_resize_keeps_dimensions() {
        let src = png_64x48();

        let out = transform(&src, IDENTITY_RECIPE, "png").expect("transform should succeed");

        let back = info(&out).expect("output bytes should decode");
        assert_eq!((back.width(), back.height()), (64, 48));
    }

    /// `info` reports the true width/height/format of a PNG decoded in wasm.
    #[wasm_bindgen_test]
    fn info_reports_png_dimensions() {
        let i = info(&png_64x48()).expect("info should succeed");

        assert_eq!(i.width(), 64);
        assert_eq!(i.height(), 48);
        assert_eq!(i.format(), "png");
        assert!(i.has_alpha(), "the RGBA fixture carries alpha");
    }

    /// SVG rasterizes IN WASM: the resvg/usvg/tiny-skia stack survives the target
    /// cut, so the browser build converts vector → raster with no backend. This is
    /// a headline capability of the wave, and the reason the AVIF loss is
    /// survivable for now.
    ///
    /// A rasterized SVG reports `source_format = png` (SPEC-060's
    /// materialized-raster convention), and the fixture's intrinsic size is 40×30.
    #[wasm_bindgen_test]
    fn svg_rasterizes_in_wasm() {
        let i = info(SVG).expect("svg should rasterize");
        assert_eq!((i.width(), i.height()), (40, 30), "intrinsic SVG size");

        // …and it flows all the way through the pipeline to encoded output bytes.
        let out = transform(SVG, RESIZE_RECIPE, "png").expect("svg → resize → png");
        let back = info(&out).expect("output bytes should decode");
        assert_eq!((back.width(), back.height()), (32, 24));
    }

    /// An AVIF input ERRORS — cleanly, with a typed message — instead of panicking.
    ///
    /// This is the one capability the wasm build gives up (`re_rav1d` does not
    /// compile to bare wasm32, DEC-064), and how it gives it up is the whole point:
    /// a panic in wasm ABORTS the module and takes the page's instance down with
    /// it, so "returns an Err" versus "traps" is a user-visible difference, not a
    /// stylistic one. The message must also be honest — no "--features" advice that
    /// would be a lie in a browser.
    #[wasm_bindgen_test]
    fn avif_input_errors_not_panics() {
        // Sanity: the fixture really is AVIF (so a passing test isn't testing a
        // misnamed PNG).
        assert_eq!(
            &AVIF[4..8],
            b"ftyp",
            "fixture should be an ISOBMFF container"
        );

        let err = transform(AVIF, RESIZE_RECIPE, "png")
            .expect_err("AVIF decode is unavailable in wasm — must be an Err, not a panic");

        let msg = format!("{:?}", wasm_bindgen::JsValue::from(err));
        assert!(msg.contains("AVIF"), "message should name the codec: {msg}");
        assert!(
            !msg.contains("--features"),
            "must not advise a cargo feature a browser user cannot use: {msg}"
        );
    }

    /// PNG → **AVIF** in the browser: the wave's headline (SPEC-073, DEC-065).
    ///
    /// `rav1e`/`ravif` compile to wasm32, so the `avif` feature — which the shipped
    /// artifact is built with — puts a real AV1 encoder in the page. This test is
    /// the earned verdict on that: it runs the encoder inside the wasm VM and
    /// asserts the RETURNED BYTES are a real AVIF file, not merely that the call
    /// returned `Ok`.
    ///
    /// The output cannot be fed back through `info()` — the wasm build has no AVIF
    /// *decoder* (DEC-065's asymmetry, and precisely why this assertion sniffs the
    /// container instead of decoding it). So we check the ISOBMFF header the way
    /// `image::sniff` does: a `ftyp` box at offset 4 whose major brand (offset 8) is
    /// `avif` (still image) or `avis` (sequence).
    ///
    /// NOTE: this test requires `--features avif`, which `just wasm-test` passes.
    /// The shipped wasm artifact is an AVIF-encoding artifact (DEC-065); a build
    /// without the feature is only ever made to measure the size delta.
    #[wasm_bindgen_test]
    fn transform_png_to_avif_is_valid_avif() {
        let src = png_64x48();

        let out = transform(&src, RESIZE_RECIPE, "avif")
            .expect("AVIF encode must work in wasm (build with --features avif — DEC-065)");

        assert!(out.len() > 8, "AVIF encode returned no usable bytes");
        assert_eq!(&out[4..8], b"ftyp", "output should be an ISOBMFF container");
        let brand = &out[8..12];
        assert!(
            brand == b"avif" || brand == b"avis",
            "major brand should be avif/avis, got {:?}",
            std::str::from_utf8(brand)
        );
    }

    /// Turning AVIF **encode** on did NOT quietly turn AVIF **decode** on: an AVIF
    /// input still returns the typed `CodecUnavailableOnTarget` error.
    ///
    /// This is the load-bearing guard on DEC-065's asymmetry. `image`'s `avif`
    /// feature is encode-only (`ravif`), and the decoder we use (`re_rav1d`) is
    /// still native-only — but that is a property of two upstream crates, and a
    /// future `image` feature-flag change could silently flip it. The demo page
    /// promises the browser reads `.avif` via `createImageBitmap`, not via us, so
    /// the day this test goes red is a day the story changed.
    #[wasm_bindgen_test]
    fn avif_input_still_errors_on_wasm() {
        let err = transform(AVIF, RESIZE_RECIPE, "png")
            .expect_err("AVIF DECODE stays unavailable in wasm even with the avif feature on");

        let msg = format!("{:?}", wasm_bindgen::JsValue::from(err));
        assert!(msg.contains("AVIF"), "message should name the codec: {msg}");
        assert!(
            !msg.contains("--features"),
            "must not advise a cargo feature a browser user cannot use: {msg}"
        );
    }

    /// The same SVG as the fixture with the `<text>` element REMOVED — the control
    /// for `svg_text_renders_glyphs_in_wasm` below.
    const SVG_NO_TEXT: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30" viewBox="0 0 40 30">
  <rect x="0" y="0" width="40" height="30" fill="#3366cc"/>
  <circle cx="30" cy="10" r="8" fill="#ffffff" fill-opacity="0.5"/>
</svg>
"##;

    /// SVG `<text>` really rasterizes to GLYPHS in the wasm build — the guardrail on
    /// the one capability SPEC-074 measured, priced, and deliberately KEPT.
    ///
    /// resvg's `text` feature is the single biggest cluster in the binary (287,098 B
    /// brotli, a fifth of the bundle). Dropping it is therefore permanently
    /// tempting — and the reason it must not be dropped quietly is that dropping it
    /// is INVISIBLE: `usvg` without `text` silently drops `<text>` nodes from the
    /// tree, so the SVG still rasterizes, still reports 40x30, and `transform()`
    /// still returns `Ok` — with a hole where the label was. We built that artifact
    /// while measuring, and confirmed `svg_rasterizes_in_wasm` (which asserts only
    /// dimensions) stayed GREEN straight through the corruption. Dimensions cannot
    /// see this; only pixels can.
    ///
    /// So this test compares the fixture against the same SVG with its `<text>`
    /// removed. With the text stack linked, the two rasters differ (glyphs are drawn
    /// over the rect). Without it, both render identically — and this goes RED, which
    /// is exactly the alarm DEC-066 is owed. It fails LOUD on a silent capability loss.
    #[wasm_bindgen_test]
    fn svg_text_renders_glyphs_in_wasm() {
        let with_text = transform(SVG, IDENTITY_RECIPE, "png").expect("text SVG rasterizes");
        let without_text = transform(SVG_NO_TEXT.as_bytes(), IDENTITY_RECIPE, "png")
            .expect("text-free SVG rasterizes");

        // Both are the same 40x30 canvas with the same rect and circle...
        let a = info(&with_text).expect("decodes");
        let b = info(&without_text).expect("decodes");
        assert_eq!((a.width(), a.height()), (40, 30));
        assert_eq!((b.width(), b.height()), (40, 30));

        // ...so if the ONLY difference — the `<text>` element — produced no pixels,
        // these would be byte-identical. They must not be.
        assert_ne!(
            with_text, without_text,
            "SVG <text> rendered NOTHING: the resvg `text` feature is gone from the \
             wasm build, so every SVG with a label now rasterizes with a hole in it — \
             silently, with no error the user can see. That is a capability loss, not \
             a size win (DEC-066 priced it at 287,098 B brotli and kept it)."
        );
    }

    /// A TIFF/BMP/ICO input — the three decoders SPEC-074 trimmed from the wasm build
    /// (−84,327 B brotli, DEC-066) — fails CLEANLY rather than panicking.
    ///
    /// How a dropped codec fails is the whole point of dropping it deliberately: a
    /// panic in wasm aborts the module and takes the page's instance down with it, so
    /// "typed Err" versus "trap" is user-visible. These three formats are still
    /// DETECTED (the sniff is format-agnostic) — they just have no decoder here, the
    /// same shape as AVIF (DEC-064). The native build still reads all three; its twin
    /// test below pins that, so this trim cannot leak into the CLI.
    #[wasm_bindgen_test]
    fn trimmed_codecs_error_cleanly_in_wasm() {
        for (name, bytes) in [
            ("bmp", &include_bytes!("fixtures/raster/tiny_16x12.bmp")[..]),
            (
                "tiff",
                &include_bytes!("fixtures/raster/tiny_16x12.tiff")[..],
            ),
            ("ico", &include_bytes!("fixtures/raster/tiny_16x12.ico")[..]),
        ] {
            let err = transform(bytes, RESIZE_RECIPE, "png").expect_err(&format!(
                "{name} decode was trimmed from the wasm build — it must return an \
                 Err, not decode, and above all not panic"
            ));
            let msg = format!("{:?}", wasm_bindgen::JsValue::from(err));
            assert!(
                !msg.contains("--features"),
                "must not advise a cargo feature a browser user cannot use: {msg}"
            );
        }
    }

    /// `optimize` runs the real engine in wasm: the analysis layer buckets the
    /// image, the shortlist picks a format, and — for a lossy target — the
    /// SSIMULACRA2 quality search actually runs. Asserted on the output bytes.
    #[wasm_bindgen_test]
    fn optimize_produces_decodable_output() {
        let src = png_64x48();

        let out = optimize(&src, "jpeg").expect("optimize should succeed");

        let back = info(&out).expect("optimized bytes should decode");
        assert_eq!(
            (back.width(), back.height()),
            (64, 48),
            "dimensions preserved"
        );
        assert_eq!(back.format(), "jpeg");
    }

    /// `optimize` with no format named lets the engine choose one — and whatever it
    /// chooses must be something we can actually encode and decode back.
    #[wasm_bindgen_test]
    fn optimize_auto_format_picks_an_encodable_format() {
        let out = optimize(&png_64x48(), "auto").expect("auto optimize should succeed");

        let back = info(&out).expect("auto-optimized bytes should decode");
        assert_eq!((back.width(), back.height()), (64, 48));
    }

    /// `optimize(_, "avif")` encodes rather than searching — and, crucially, does
    /// not blow up trying to.
    ///
    /// The perceptual quality search decodes every candidate to score it (DEC-019),
    /// so it needs a DECODER, which AVIF does not have here. The wasm surface
    /// therefore skips the search for AVIF and encodes once at the encoder's
    /// default quality. Asking `auto_quality` to search AVIF would fail on the
    /// first candidate's decode, so this asserts the guard is in place — it is the
    /// one behavioral seam SPEC-073 changed inside `optimize`.
    #[wasm_bindgen_test]
    fn optimize_to_avif_encodes_without_the_perceptual_search() {
        let out =
            optimize(&png_64x48(), "avif").expect("optimize → avif must not attempt a search");

        assert!(out.len() > 8, "AVIF encode returned no usable bytes");
        assert_eq!(&out[4..8], b"ftyp", "output should be an ISOBMFF container");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// NATIVE guard: the target-gating must not have broken native AVIF decode
// ─────────────────────────────────────────────────────────────────────────────
/// SPEC-072 moved `re_rav1d`/`avif-parse` into a `cfg(not(wasm32))` dependency
/// table and the AVIF sniff out of `image::avif` into `image::sniff`. None of that
/// may cost the NATIVE build its AVIF decode. The existing suite (`tests/input_avif.rs`)
/// covers AVIF end-to-end through the binary; this is the direct library-level
/// assertion that the gate itself didn't sever the path.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_avif_still_decodes() {
    let bytes = include_bytes!("fixtures/avif/solid_16x16.avif");

    let img = crustyimg::image::Image::from_bytes(bytes).expect("native AVIF decode must survive");

    assert_eq!((img.width(), img.height()), (16, 16));
    assert_eq!(img.source_format(), image::ImageFormat::Avif);
}

/// SPEC-074 trimmed `tiff`, `bmp` and `ico` out of the WASM build's `image` feature
/// set (DEC-066). The native CLI must still read all three — it is a filesystem tool
/// pointed at whatever a scanner or a favicon pipeline produced, and `image` is now
/// declared once per target table, which is exactly the shape where a "cleanup" of
/// the duplicated line would quietly take the native codecs down with it (DEC-064
/// requires the native build to stay byte-identical).
///
/// The fixtures are the trim's other half: the same three files the wasm test asserts
/// must FAIL to decode, asserted here to decode. One pair of tests, opposite verdicts
/// — that is the target-cfg boundary, pinned.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_still_decodes_the_codecs_wasm_trimmed() {
    use crustyimg::image::Image;

    for (name, bytes) in [
        ("bmp", &include_bytes!("fixtures/raster/tiny_16x12.bmp")[..]),
        (
            "tiff",
            &include_bytes!("fixtures/raster/tiny_16x12.tiff")[..],
        ),
        ("ico", &include_bytes!("fixtures/raster/tiny_16x12.ico")[..]),
    ] {
        let img = Image::from_bytes(bytes)
            .unwrap_or_else(|e| panic!("native {name} decode must survive the wasm trim: {e}"));
        assert_eq!(
            (img.width(), img.height()),
            (16, 12),
            "{name} decoded to the wrong size"
        );
    }
}

/// The native twin of `svg_text_renders_glyphs_in_wasm`: `resvg`'s `text` feature is
/// now declared per-target, so this pins that the NATIVE build still renders SVG
/// `<text>` as real glyphs (DEC-054) — the wasm-side feature line cannot be edited
/// into taking native's text stack with it.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_svg_text_still_renders_glyphs() {
    use crustyimg::image::Image;

    const NO_TEXT: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30" viewBox="0 0 40 30">
  <rect x="0" y="0" width="40" height="30" fill="#3366cc"/>
  <circle cx="30" cy="10" r="8" fill="#ffffff" fill-opacity="0.5"/>
</svg>
"##;

    let with_text = Image::from_bytes(include_bytes!("fixtures/svg/rect_text_40x30.svg"))
        .expect("text SVG rasterizes natively");
    let without_text = Image::from_bytes(NO_TEXT.as_bytes()).expect("text-free SVG rasterizes");

    assert_eq!((with_text.width(), with_text.height()), (40, 30));
    assert_ne!(
        with_text.pixels().as_bytes(),
        without_text.pixels().as_bytes(),
        "native SVG <text> rendered no pixels — the resvg `text` feature (DEC-054) is \
         gone from the NATIVE build"
    );
}

/// SPEC-073 turned the `avif` feature ON for the wasm artifact and taught the wasm
/// surface to skip the perceptual search for AVIF. Neither may cost the NATIVE build
/// its AVIF ENCODE: `--features avif` must still produce a valid AVIF that this
/// build's own decoder reads back. (Native `cargo test --features avif` runs this;
/// the unit test `sink::encode_avif_respects_quality` covers the quality knob, so
/// this asserts only the end-to-end encode → decode identity the wasm change could
/// plausibly have broken.)
#[cfg(all(not(target_arch = "wasm32"), feature = "avif"))]
#[test]
fn native_avif_encode_still_works() {
    use crustyimg::image::Image;

    let src = Image::from_bytes(include_bytes!("fixtures/avif/solid_16x16.avif"))
        .expect("fixture decodes");

    let out = crustyimg::sink::encode_to_bytes(&src, image::ImageFormat::Avif, Some(80))
        .expect("native AVIF encode must survive");

    assert_eq!(
        &out[4..8],
        b"ftyp",
        "encoded output should be an ISOBMFF box"
    );
    let back = Image::from_bytes(&out).expect("native build decodes what it encoded");
    assert_eq!((back.width(), back.height()), (16, 16));
    assert_eq!(back.source_format(), image::ImageFormat::Avif);
}
