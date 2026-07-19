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
    use crustyimg::sink::FAST_LOSSY_QUALITY;
    use crustyimg::wasm::{info, optimize, optimize_detailed, score, transform};
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

    // ── SPEC-079: the detailed optimize surface ───────────────────────────────
    //
    // These drive `optimizeDetailed` and `score` — the surface SPEC-080's demo and
    // SPEC-081's quality readout are built on — inside the real wasm VM, and assert
    // on what came back, not on the call returning Ok.

    /// A **photographic** fixture: 192×160, no alpha, a gradient carrying texture and
    /// deterministic pseudo-noise. Two properties matter and both are load-bearing:
    ///
    /// - it is **> 128 px** on its long edge, so the analysis layer's icon rule
    ///   (`ICON_MAX_EDGE`) does not claim it before the photograph rule can — the
    ///   64×48 fixture above is an *Icon* to the engine, which is why it cannot be
    ///   reused here;
    /// - it has thousands of colours and high entropy, so it buckets `Lossy`
    ///   (`ImageClass::Photograph`) — the bucket the Auto-AVIF rule keys on.
    fn photo_png_192x160() -> Vec<u8> {
        use image::{ImageEncoder, Rgb, RgbImage};
        let mut img = RgbImage::new(192, 160);
        for (x, y, px) in img.enumerate_pixels_mut() {
            // A cheap deterministic hash — no rng dependency, identical every run.
            let n = ((x.wrapping_mul(2654435761)) ^ (y.wrapping_mul(40503))) % 37;
            let gx = (x * 255 / 192) as i32;
            let gy = (y * 255 / 160) as i32;
            let tex = if ((x / 5) + (y / 3)) % 2 == 0 { 24 } else { 0 };
            let r = (gx + tex + n as i32).clamp(0, 255) as u8;
            let g = (gy + n as i32 * 2).clamp(0, 255) as u8;
            let b = ((gx + gy) / 2 + tex - n as i32).clamp(0, 255) as u8;
            *px = Rgb([r, g, b]);
        }
        let mut out = Vec::new();
        image::codecs::png::PngEncoder::new(&mut out)
            .write_image(img.as_raw(), 192, 160, image::ExtendedColorType::Rgb8)
            .expect("encode photographic fixture");
        out
    }

    /// A **flat graphic** fixture: 192×160 (again over the icon edge), four flat
    /// colours in blocks. Few colours → `ImageClass::GraphicLogo` → the
    /// `LosslessFlat` bucket, the control for the Auto-AVIF rule.
    fn graphic_png_192x160() -> Vec<u8> {
        use image::{ImageEncoder, Rgb, RgbImage};
        const PALETTE: [[u8; 3]; 4] = [
            [255, 255, 255],
            [20, 40, 160],
            [230, 60, 40],
            [250, 210, 40],
        ];
        let mut img = RgbImage::new(192, 160);
        for (x, y, px) in img.enumerate_pixels_mut() {
            *px = Rgb(PALETTE[((x / 48 + y / 40) % 4) as usize]);
        }
        let mut out = Vec::new();
        image::codecs::png::PngEncoder::new(&mut out)
            .write_image(img.as_raw(), 192, 160, image::ExtendedColorType::Rgb8)
            .expect("encode graphic fixture");
        out
    }

    /// Re-encode a decodable image to JPEG at `q` (a degraded reference for `score`).
    fn jpeg_at(png: &[u8], q: u8) -> Vec<u8> {
        let img = image::load_from_memory(png).expect("fixture decodes");
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(
            &mut out, q,
        ))
        .expect("jpeg encode");
        out.into_inner()
    }

    /// **The headline of SPEC-079.** A photo through Auto now comes back as AVIF —
    /// at the speed the caller asked for.
    ///
    /// Before this spec, Auto ran `Mode::Perceptual`, whose shortlist refuses AVIF
    /// (it cannot be decoded, so it cannot be scored — DEC-020/DEC-048), so a photo
    /// fell through to a slow SSIMULACRA2 JPEG search that saved ~13 %. The
    /// Auto-AVIF rule keys on the content bucket instead, and the reported `speed`
    /// proves the rav1e knob really reached the encoder rather than being accepted
    /// and dropped.
    #[wasm_bindgen_test]
    fn optimize_detailed_auto_photo_picks_avif() {
        let src = photo_png_192x160();

        let r = optimize_detailed(&src, "auto", Some(10), None, None)
            .expect("auto optimize of a photo should succeed");

        assert_eq!(r.format(), "avif", "a photo must route to AVIF under Auto");
        assert!(
            r.bytes().len() < src.len(),
            "AVIF ({} B) must beat the source PNG ({} B) — an 'optimization' that grows \
             the file is the bug STAGE-029 exists to fix",
            r.bytes().len(),
            src.len()
        );
        assert_eq!(&r.bytes()[4..8], b"ftyp", "output should really be AVIF");
        assert_eq!(
            r.quality(),
            Some(FAST_LOSSY_QUALITY),
            "the default AVIF encode uses native web's FAST_LOSSY_QUALITY (SPEC-095), \
             not convert's AVIF_DEFAULT_QUALITY"
        );
        assert_eq!(r.speed(), Some(10), "the requested rav1e speed is honoured");
        assert_eq!(
            r.score(),
            None,
            "the wasm engine cannot decode AVIF, so it must NOT claim a score for it \
             (DEC-065) — SPEC-081 scores it browser-side via score()"
        );
        assert_eq!(r.scored_by(), "none");
    }

    /// **SPEC-095.** The demo's default photo→AVIF conversion must be the SAME
    /// quality setting native `crustyimg web` (`Mode::Fast`) actually ships — not
    /// merely a plausible-looking one. Two separate proofs, because a report that
    /// says "85" while the bytes underneath were still encoded at 80 would be
    /// exactly the kind of unchecked claim this project keeps catching:
    ///
    /// 1. `OptimizeResult.quality` reports [`FAST_LOSSY_QUALITY`] — the symbol
    ///    native `web` uses, not a wasm-local literal that happens to match it.
    /// 2. The RETURNED BYTES equal an independent encode at that same quality —
    ///    proving the value was actually threaded into the encoder, not just
    ///    reported next to a still-q80 encode.
    ///
    /// Before this spec: reported `Some(80)` and the bytes were the
    /// `AVIF_DEFAULT_QUALITY` encode (the `(_, true) => (None, None)` no-search arm
    /// falling through to `encode_to_bytes_with(..., None, ...)`).
    #[wasm_bindgen_test]
    fn wasm_default_avif_quality_is_web_fast_quality() {
        let src = photo_png_192x160();

        let r = optimize_detailed(&src, "auto", None, None, None)
            .expect("auto optimize of a photo should succeed");

        assert_eq!(r.format(), "avif", "a photo must route to AVIF under Auto");
        assert_eq!(
            r.quality(),
            Some(FAST_LOSSY_QUALITY),
            "wasm's default AVIF quality must equal native web's FAST_LOSSY_QUALITY"
        );

        // Independently re-derive what an honest q85 encode looks like, at the
        // SAME rav1e speed `optimize_detailed` resolves to when the caller doesn't
        // pass one (AVIF_SPEED) — and prove the returned bytes are exactly that.
        let img = crustyimg::image::Image::from_bytes(&src).expect("fixture decodes");
        let expected = crustyimg::sink::encode_to_bytes(
            &img,
            ::image::ImageFormat::Avif,
            Some(FAST_LOSSY_QUALITY),
        )
        .expect("independent q85 encode");
        assert_eq!(
            r.bytes(),
            expected,
            "the demo must actually ENCODE at FAST_LOSSY_QUALITY, not just report it"
        );
    }

    /// The control: the Auto-AVIF rule fires on **content**, not on every input. A
    /// flat graphic stays lossless — turning a logo into AVIF would be the mirror
    /// image of the bug.
    #[wasm_bindgen_test]
    fn optimize_detailed_auto_graphic_stays_lossless() {
        let r = optimize_detailed(&graphic_png_192x160(), "auto", None, None, None)
            .expect("auto optimize of a graphic should succeed");

        assert_ne!(r.format(), "avif", "a flat graphic must not route to AVIF");
        assert!(
            r.format() == "png" || r.format() == "webp",
            "a graphic should stay in the lossless family, got {}",
            r.format()
        );
        assert_eq!(r.quality(), None, "a lossless format has no quality knob");
        assert_eq!(r.speed(), None, "only AVIF has a speed knob");
    }

    /// JPEG still runs the real perceptual search, and now the achieved SSIMULACRA2
    /// comes back with the bytes — the number SPEC-081 puts on screen. `scoredBy`
    /// says the engine measured it (as opposed to nobody having measured it).
    #[wasm_bindgen_test]
    fn optimize_detailed_jpeg_returns_engine_score() {
        let r = optimize_detailed(&photo_png_192x160(), "jpeg", None, None, Some(90.0))
            .expect("jpeg optimize should succeed");

        assert_eq!(r.format(), "jpeg");
        assert!(
            r.quality().is_some(),
            "a searched JPEG has a chosen quality"
        );
        let s = r
            .score()
            .expect("a searched JPEG must carry its achieved score");
        assert!(
            s > 0.0 && s <= 100.0,
            "score {s} is not on the SSIMULACRA2 scale"
        );
        assert_eq!(r.scored_by(), "engine");

        let back = info(&r.bytes()).expect("output decodes");
        assert_eq!((back.width(), back.height()), (192, 160));
    }

    /// A byte budget is honoured for AVIF — the format the engine can size-search
    /// but never perceptually search — **at the requested speed**.
    ///
    /// This is the speed-parity contract seen from the outside (DEC-068): the size
    /// search probes candidates at speed 10, and the sink then writes at speed 10, so
    /// the budget the search met is the budget the returned bytes actually meet. Had
    /// the search probed at the default speed 6 and the sink emitted at 10, this
    /// assertion is what would catch it — the emitted file would be a different size
    /// than the one the search approved.
    #[wasm_bindgen_test]
    fn optimize_detailed_budget_is_honoured_and_speed_parity() {
        let r = optimize_detailed(&photo_png_192x160(), "avif", Some(10), Some(20_000), None)
            .expect("avif budget optimize should succeed");

        let bytes = r.bytes();
        assert!(bytes.len() > 8, "no usable bytes");
        assert_eq!(&bytes[4..8], b"ftyp", "output should be a real AVIF");
        assert!(
            bytes.len() <= 20_000,
            "the 20 000 B budget was not met: got {} B",
            bytes.len()
        );
        assert_eq!(r.format(), "avif");
        assert_eq!(r.speed(), Some(10));
        assert!(r.quality().is_some(), "the size search chose a quality");
    }

    /// A hostile input ERRORS and the module SURVIVES.
    ///
    /// The decode caps (DEC-034/DEC-063) live in the core, so they carry into wasm
    /// unchanged — but "carries" has to be driven, because the failure mode is not a
    /// wrong answer, it is a **panic that aborts the module** and takes the page's
    /// engine instance with it. So this asserts both halves: the over-cap call is an
    /// `Err`, AND a later ordinary call still works.
    #[wasm_bindgen_test]
    fn optimize_detailed_rejects_oversize_without_panic() {
        let bomb = png_header_declaring(100_000, 100_000);

        // (`OptimizeResult` is not `Debug` — it holds the output bytes — so this
        // matches rather than `expect_err`s.)
        let msg = match optimize_detailed(&bomb, "auto", Some(10), None, None) {
            Ok(_) => panic!("a 100000x100000 declaration is over the 64 MP cap — must be an Err"),
            Err(e) => format!("{:?}", wasm_bindgen::JsValue::from(e)),
        };
        assert!(!msg.is_empty(), "the error must carry a message");

        // The module is still alive: an ordinary call after the rejection succeeds.
        let ok = optimize_detailed(&graphic_png_192x160(), "auto", None, None, None)
            .expect("the wasm module must survive a rejected input");
        assert!(!ok.bytes().is_empty());
    }

    /// A PNG whose IHDR *declares* `w × h` and carries nothing else — the classic
    /// decompression-bomb shape: 40-odd bytes claiming ten billion pixels. The CRC is
    /// computed for real, so the decoder reads the header rather than bailing on a
    /// malformed chunk, and the dimensions are what it rejects.
    fn png_header_declaring(w: u32, h: u32) -> Vec<u8> {
        fn crc32(bytes: &[u8]) -> u32 {
            let mut crc = 0xFFFF_FFFFu32;
            for &b in bytes {
                crc ^= b as u32;
                for _ in 0..8 {
                    let mask = (crc & 1).wrapping_neg();
                    crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
                }
            }
            !crc
        }

        let mut ihdr = Vec::from(*b"IHDR");
        ihdr.extend_from_slice(&w.to_be_bytes());
        ihdr.extend_from_slice(&h.to_be_bytes());
        ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit, truecolour, no interlace

        let mut png = Vec::from(*b"\x89PNG\r\n\x1a\n");
        png.extend_from_slice(&13u32.to_be_bytes()); // IHDR payload length
        png.extend_from_slice(&ihdr);
        png.extend_from_slice(&crc32(&ihdr).to_be_bytes());
        png
    }

    /// `score(a, a)` — an image against itself — is the metric's maximum. The
    /// calibration point for SPEC-081's readout: whatever number the UI shows, THIS
    /// is what "identical" looks like.
    #[wasm_bindgen_test]
    fn score_identical_is_max() {
        let png = photo_png_192x160();

        let s = score(&png, &png).expect("scoring an image against itself must succeed");

        assert!(s > 99.0, "identical images should score ~100, got {s}");
    }

    /// …and a degraded copy scores lower — so the number is a real measurement of
    /// this image, not a constant. A bad input errs rather than panicking.
    #[wasm_bindgen_test]
    fn score_degraded_is_lower_and_bad_input_errs() {
        let png = photo_png_192x160();
        let degraded = jpeg_at(&png, 10);

        let s = score(&png, &degraded).expect("scoring a degraded copy must succeed");
        assert!(
            s < 99.0,
            "a quality-10 JPEG of a detailed photo must score below identity, got {s}"
        );

        let err = score(&png, b"not an image").expect_err("undecodable input must be an Err");
        let msg = format!("{:?}", wasm_bindgen::JsValue::from(err));
        assert!(!msg.is_empty(), "the error must carry a message");
    }

    /// The legacy two-argument `optimize` is UNCHANGED by all of the above: same
    /// call, same engine path, still decodable output. `optimize_detailed` is an
    /// addition, not a replacement — the npm package's existing callers (and the
    /// live demo) keep working byte-for-byte.
    #[wasm_bindgen_test]
    fn legacy_optimize_unchanged() {
        let src = png_64x48();

        let out = optimize(&src, "auto").expect("legacy optimize still works");
        let back = info(&out).expect("legacy optimize output still decodes");
        assert_eq!((back.width(), back.height()), (64, 48));

        // The legacy entry is a lossless/perceptual path over a 64×48 icon-bucket
        // image: it must NOT have acquired the new Auto-AVIF behaviour, which is
        // confined to `optimize_detailed`.
        let jpeg = optimize(&src, "jpeg").expect("legacy optimize → jpeg");
        assert_eq!(info(&jpeg).expect("decodes").format(), "jpeg");
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

/// SPEC-095 mechanical check
/// ([[mechanical-sweeps-need-a-mechanical-check]]): the wasm no-search AVIF quality
/// is anchored to the `FAST_LOSSY_QUALITY` SYMBOL, not a hardcoded `85` literal that
/// merely happens to match it today — a literal is exactly how DEC-069's
/// native(85)/wasm(80) divergence opened in the first place, and exactly how a
/// second one would open silently if the constant ever moved. Reading the two match
/// arms this spec changed and asserting they reference the symbol is a plausible
/// check; grepping the WHOLE file for a bare `85` is the mechanical one.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn wasm_avif_quality_anchored_not_hardcoded() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/wasm.rs"),
    )
    .expect("read src/wasm.rs");

    let fast_quality_refs = src.matches("sink::FAST_LOSSY_QUALITY").count();
    assert!(
        fast_quality_refs >= 2,
        "expected src/wasm.rs to reference sink::FAST_LOSSY_QUALITY at both the \
         optimize() and optimize_detailed() no-search AVIF sites, found {fast_quality_refs}"
    );

    // No line in the file may carry a bare `85` literal — the only place "85" is
    // allowed to appear is as part of the `FAST_LOSSY_QUALITY` symbol name itself,
    // which this grep does not match (it looks for the digits, not the identifier).
    let bare_85_lines: Vec<&str> = src
        .lines()
        .filter(|line| {
            let stripped = line.trim_start();
            if stripped.starts_with("//") {
                return false; // prose may say "85" while explaining the constant
            }
            contains_bare_number(line, "85")
        })
        .collect();
    assert!(
        bare_85_lines.is_empty(),
        "found a hardcoded 85 literal outside a comment in src/wasm.rs — anchor to \
         sink::FAST_LOSSY_QUALITY instead: {bare_85_lines:?}"
    );
}

/// `true` iff `line` contains `needle` as digits not adjacent to another digit or
/// underscore (so it doesn't false-positive on `185`, `85_000`, or `x85y`).
#[cfg(not(target_arch = "wasm32"))]
fn contains_bare_number(line: &str, needle: &str) -> bool {
    let bytes = line.as_bytes();
    let mut start = 0;
    while let Some(pos) = line[start..].find(needle) {
        let idx = start + pos;
        let before_ok = idx == 0 || !(bytes[idx - 1].is_ascii_digit() || bytes[idx - 1] == b'_');
        let after = idx + needle.len();
        let after_ok =
            after >= bytes.len() || !(bytes[after].is_ascii_digit() || bytes[after] == b'_');
        if before_ok && after_ok {
            return true;
        }
        start = idx + needle.len();
    }
    false
}

/// **SPEC-095 native regression anchor.** Closing the wasm/native AVIF-quality
/// divergence must NOT touch `convert`'s byte-identity contract (DEC-071): native
/// `convert` output is entirely unchanged by this spec. This complements the
/// existing `sink::tests::convert_avif_bytes_unchanged_at_default` unit test
/// (`src/sink/mod.rs:961`) — that one proves the *default* (`None`) still resolves
/// to `AVIF_DEFAULT_QUALITY`; this one drives the real `convert` binary end-to-end
/// and pins the actual output bytes so a regression anywhere in the CLI's argument
/// resolution (not just the sink) would be caught too.
#[cfg(all(not(target_arch = "wasm32"), feature = "avif"))]
#[test]
fn convert_avif_default_unchanged() {
    use std::process::Command;

    use tempfile::TempDir;

    let src = crustyimg::image::Image::from_bytes(include_bytes!("fixtures/avif/solid_16x16.avif"))
        .expect("fixture decodes");
    let png = crustyimg::sink::encode_to_bytes(&src, image::ImageFormat::Png, None)
        .expect("encode fixture as png source");

    let dir = TempDir::new().unwrap();
    let src_path = dir.path().join("in.png");
    let out_path = dir.path().join("out.avif");
    std::fs::write(&src_path, &png).unwrap();

    let bin = env!("CARGO_BIN_EXE_crustyimg");
    let output = Command::new(bin)
        .args(["convert", "--format", "avif"])
        .arg(&src_path)
        .arg("-o")
        .arg(&out_path)
        .arg("--yes")
        .output()
        .expect("run convert");
    assert!(
        output.status.success(),
        "convert --format avif must succeed; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let produced = std::fs::read(&out_path).expect("read convert output");
    let expected = crustyimg::sink::encode_to_bytes(
        &crustyimg::image::Image::from_bytes(&png).unwrap(),
        image::ImageFormat::Avif,
        None,
    )
    .expect("independent default-quality encode");

    assert_eq!(
        produced, expected,
        "native `convert --format avif` output must stay byte-identical to the \
         AVIF_DEFAULT_QUALITY (q80) encode — SPEC-095 must not move convert's bytes"
    );
}
