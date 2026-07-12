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
//! The native half of the same guarantee (AVIF still decodes on native after the
//! target-gating) is asserted at the bottom, and DOES run under `cargo test`.

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
