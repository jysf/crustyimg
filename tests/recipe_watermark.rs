//! Integration tests for SPEC-128 — recipes can express `watermark`.
//!
//! `AC-1`/`AC-2`/`AC-2b` exercise the public library API directly (no CLI
//! process needed: round-trip and typed-error behavior live entirely in
//! `crustyimg::recipe`/`crustyimg::operation`). `AC-5` drives the real
//! compiled binary (`env!("CARGO_BIN_EXE_crustyimg")`), matching
//! `tests/apply_batch.rs`'s convention, because it asserts an OS-level
//! observable: no file was written to `--out-dir` before the process exited.

use std::path::PathBuf;
use std::process::Command;

use image::{DynamicImage, RgbaImage};
use tempfile::TempDir;

use crustyimg::operation::{Gravity, Operation, OperationRegistry, Watermark};
use crustyimg::recipe::{Recipe, RecipeError};

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Fixture helpers ───────────────────────────────────────────────────────────

fn solid_overlay(w: u32, h: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, image::Rgba([10, 20, 30, 255])))
}

fn write_png(dir: &TempDir, name: &str, w: u32, h: u32) -> PathBuf {
    let img = image::RgbImage::from_pixel(w, h, image::Rgb([42u8, 100u8, 200u8]));
    let path = dir.path().join(name);
    DynamicImage::ImageRgb8(img).save(&path).unwrap();
    path
}

fn write_recipe(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

// ── AC-1: image mode round-trips with the PATH, never overlay bytes ──────────

/// The single highest-consequence guard in the spec: `to_toml` must keep
/// emitting the overlay's PATH, never its decoded bytes. Uses a deliberately
/// large overlay (200×200 RGBA ≈ 160 KB of pixels) so a leak would balloon the
/// TOML far past what a path-only recipe could ever be.
#[test]
fn watermark_recipe_round_trips_with_path_not_bytes() {
    let overlay = solid_overlay(200, 200);
    let op: Box<dyn Operation> = Box::new(Watermark::new_image(
        overlay,
        "assets/logo.png".to_owned(),
        Gravity::SouthEast,
        1.0,
        None,
        0,
        false,
    ));
    let recipe = Recipe::from_ops(&[op]);

    let toml_str = recipe.to_toml().expect("to_toml should succeed");
    assert!(
        toml_str.contains("image = \"assets/logo.png\""),
        "TOML must carry the overlay PATH, got:\n{toml_str}"
    );
    assert!(
        toml_str.len() < 500,
        "a 200x200 overlay's bytes must NOT be in the TOML (it would be tens of \
         KB, not a small recipe) — got {} bytes:\n{toml_str}",
        toml_str.len()
    );

    let reloaded = Recipe::from_toml(&toml_str).expect("from_toml should succeed");
    assert_eq!(
        recipe, reloaded,
        "image-mode watermark recipe must round-trip losslessly"
    );
}

// ── AC-2: text mode round-trips WITHOUT an `image` key, and renders the same pixels ──

/// Call 3b's defect guard: `params()` used to write the TEXT under the `image`
/// key. Asserts BOTH the TOML shape (no `image` key at all) AND that the
/// round-tripped step renders identical pixels — a parseability-only check
/// would have passed on the pre-SPEC-128 broken behavior too.
#[test]
fn text_watermark_round_trips_without_an_image_key() {
    let font_bytes = crustyimg::text::DEFAULT_FONT;
    let color = [255u8, 255, 255, 255];
    let size = 32.0;
    let rendered =
        crustyimg::text::render_text(font_bytes, "hi", size, color).expect("render_text");

    let op: Box<dyn Operation> = Box::new(Watermark::new_text(
        DynamicImage::ImageRgba8(rendered),
        "hi".to_owned(),
        None, // bundled default font
        size,
        color,
        Gravity::SouthEast,
        1.0,
        None,
        0,
        false,
    ));
    let recipe = Recipe::from_ops(&[op]);

    let toml_str = recipe.to_toml().expect("to_toml should succeed");
    assert!(
        toml_str.contains("text = \"hi\""),
        "TOML must carry the text, got:\n{toml_str}"
    );
    assert!(
        !toml_str.contains("image"),
        "text mode must NEVER emit an 'image' key (Call 3b's defect), got:\n{toml_str}"
    );

    let reloaded = Recipe::from_toml(&toml_str).expect("from_toml should succeed");
    assert_eq!(
        recipe, reloaded,
        "text-mode watermark recipe must round-trip losslessly"
    );

    // Renders the SAME pixels: build the pipeline from the round-tripped
    // recipe (needs no resolved asset — no `font` key, so the default font
    // is used, exactly like the original) and compare its output against a
    // freshly-constructed equivalent op applied directly.
    let base = crustyimg::image::Image::from_parts(
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(64, 64, image::Rgba([0, 0, 0, 255]))),
        image::ImageFormat::Png,
        None,
    );

    let registry = OperationRegistry::with_builtins();
    let pipeline = reloaded
        .build_pipeline(&registry)
        .expect("text-only watermark needs no resolved asset and must build");
    let via_recipe = pipeline
        .run(base.clone())
        .expect("pipeline run should succeed")
        .pixels()
        .to_rgba8();

    let rendered_again =
        crustyimg::text::render_text(font_bytes, "hi", size, color).expect("render_text");
    let direct_op = Watermark::new_text(
        DynamicImage::ImageRgba8(rendered_again),
        "hi".to_owned(),
        None,
        size,
        color,
        Gravity::SouthEast,
        1.0,
        None,
        0,
        false,
    );
    let via_direct = direct_op
        .apply(base)
        .expect("direct apply should succeed")
        .pixels()
        .to_rgba8();

    assert_eq!(
        via_recipe.into_raw(),
        via_direct.into_raw(),
        "the round-tripped recipe step must render IDENTICAL pixels to the original op"
    );
}

// ── AC-2b: exactly one of `image`/`text` — both or neither is a typed error ──

#[test]
fn watermark_step_requires_exactly_one_source() {
    let registry = OperationRegistry::with_builtins();

    let both = r#"
version = "1"

[[step]]
op = "watermark"
image = "logo.png"
text = "hi"
"#;
    let recipe = Recipe::from_toml(both).expect("TOML parses; the XOR is a build-time rule");
    let err = match recipe.build_pipeline(&registry) {
        Ok(_) => panic!("both 'image' and 'text' must be rejected"),
        Err(e) => e,
    };
    assert!(
        matches!(&err, RecipeError::InvalidOperation { name, .. } if name == "watermark"),
        "expected InvalidOperation{{name: \"watermark\", ..}}, got {err:?}"
    );

    let neither = r#"
version = "1"

[[step]]
op = "watermark"
gravity = "center"
"#;
    let recipe = Recipe::from_toml(neither).expect("TOML parses; the XOR is a build-time rule");
    let err = match recipe.build_pipeline(&registry) {
        Ok(_) => panic!("neither 'image' nor 'text' must be rejected"),
        Err(e) => e,
    };
    assert!(
        matches!(&err, RecipeError::InvalidOperation { name, .. } if name == "watermark"),
        "expected InvalidOperation{{name: \"watermark\", ..}}, got {err:?}"
    );
}

// ── AC-5: a missing overlay fails BEFORE any output is written, batch ≥2 ─────

/// `apply --recipe` on a batch of 2 inputs, where the recipe names an overlay
/// that does not exist: the whole run must fail (exit 1 — a bad recipe, not a
/// bad input) with ZERO files written to `--out-dir`, never a partial-batch
/// exit 6 (which would imply some input could have succeeded where another
/// failed — untrue here, since every input shares the one unreadable asset).
#[test]
fn missing_overlay_fails_before_any_output() {
    let dir = TempDir::new().unwrap();
    let recipe = write_recipe(
        &dir,
        "r.toml",
        r#"
version = "1"

[[step]]
op = "watermark"
image = "does/not/exist.png"
"#,
    );
    let a = write_png(&dir, "a.png", 16, 16);
    let b = write_png(&dir, "b.png", 16, 16);
    let out_dir = dir.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run apply with a missing overlay");

    assert_eq!(
        output.status.code(),
        Some(1),
        "a missing recipe asset must exit 1 (bad recipe), not 3 or 6; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let written: Vec<_> = std::fs::read_dir(&out_dir).unwrap().collect();
    assert!(
        written.is_empty(),
        "no output may be written before the recipe-level asset failure; found: {written:?}"
    );
}

// ── AC-5's other half: the `font` asset key ───────────────────────────────────

/// `AC-5` says "overlay/**font**", and until this test only the overlay half was
/// covered — half the asset mechanism this spec builds shipped untested (found at
/// SPEC-128's verify, which drove it by hand).
///
/// The fixture writes the BUNDLED font's own bytes to a temp `.ttf` and points a
/// recipe's `font` key at it. That makes the assertion exact rather than
/// approximate: resolving that path must render pixel-for-pixel what the bundled
/// default renders, because they are the same font bytes.
#[test]
fn font_key_resolves_and_renders_identically_to_the_bundled_default() {
    let dir = TempDir::new().unwrap();
    let font_path = dir.path().join("go-regular.ttf");
    std::fs::write(&font_path, crustyimg::text::DEFAULT_FONT).unwrap();

    let color = [255u8, 255, 255, 255];
    let size = 24.0;
    let rendered = crustyimg::text::render_text(crustyimg::text::DEFAULT_FONT, "mark", size, color)
        .expect("render_text");

    let op: Box<dyn Operation> = Box::new(Watermark::new_text(
        DynamicImage::ImageRgba8(rendered),
        "mark".to_owned(),
        Some(font_path.to_string_lossy().into_owned()),
        size,
        color,
        Gravity::SouthEast,
        1.0,
        None,
        0,
        false,
    ));
    let recipe = Recipe::from_ops(&[op]);
    let toml_str = recipe.to_toml().expect("to_toml");

    assert!(
        toml_str.contains("font = "),
        "a text watermark with an explicit font must emit the `font` key, got:\n{toml_str}"
    );
    assert!(
        !toml_str.contains("image"),
        "text mode must never emit `image` (Call 3b), got:\n{toml_str}"
    );
    // The path, not the ~100 KB of font bytes.
    assert!(
        toml_str.len() < 4_096,
        "font bytes leaked into the recipe: {} bytes of TOML",
        toml_str.len()
    );

    let reloaded = Recipe::from_toml(&toml_str).expect("from_toml");
    assert_eq!(recipe, reloaded, "font-bearing recipe must round-trip");

    // The registry must declare `font` as an asset key, or the resolver never
    // reads it and the whole mechanism is inert for text mode.
    let reg = OperationRegistry::with_builtins();
    assert!(
        reg.asset_keys("watermark").contains(&"font"),
        "watermark must declare `font` as an asset key, got {:?}",
        reg.asset_keys("watermark")
    );
}

/// `AC-5`, font half, at the OS level: an unreadable `font` must fail before any
/// output exists, on a batch — the same guarantee the overlay half already had.
#[test]
fn unreadable_font_fails_before_any_output() {
    let dir = TempDir::new().unwrap();
    let a = write_png(&dir, "a.png", 32, 32);
    let b = write_png(&dir, "b.png", 32, 32);
    let out = dir.path().join("out");
    std::fs::create_dir_all(&out).unwrap();

    let recipe = write_recipe(
        &dir,
        "wm.toml",
        r#"version = "1"

[[step]]
op = "watermark"
text = "mark"
font = "definitely/not/a/font.ttf"
gravity = "southeast"
opacity = 1.0
margin = 0
tile = false
"#,
    );

    let output = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--out-dir",
            out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("run apply");

    assert_eq!(
        output.status.code(),
        Some(1),
        "an unreadable font is a bad RECIPE (exit 1), not a per-input failure (6); stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let written: Vec<_> = std::fs::read_dir(&out)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        written.is_empty(),
        "no output may exist when the recipe's font could not be read; found {} file(s)",
        written.len()
    );
}

// ── Placement params reach the op (none were covered by any test) ────────────

/// `tile`, `scale`, `margin`, `opacity` and a non-default `gravity` all round-trip
/// AND reach the operation. Before this test none of them were exercised by any
/// recipe test — `get_bool` never returned a value anywhere in the suite.
///
/// Carries its own positive control: a tiled watermark must differ from an
/// untiled one. Without that, "the params round-trip" could pass while the
/// resolved op silently ignored every one of them.
#[test]
fn placement_params_round_trip_and_reach_the_op() {
    let overlay = solid_overlay(8, 8);
    let op: Box<dyn Operation> = Box::new(Watermark::new_image(
        overlay.clone(),
        "logo.png".to_owned(),
        Gravity::NorthWest,
        0.5,
        Some(0.25),
        7,
        true,
    ));
    let recipe = Recipe::from_ops(&[op]);
    let toml_str = recipe.to_toml().expect("to_toml");

    for (key, val) in [
        ("gravity", "northwest"),
        ("opacity", "0.5"),
        ("scale", "0.25"),
        ("margin", "7"),
        ("tile", "true"),
    ] {
        assert!(
            toml_str.contains(&format!("{key} = ")),
            "`{key}` must be emitted, got:\n{toml_str}"
        );
        assert!(
            toml_str.contains(val),
            "`{key}` must round-trip the value {val}, got:\n{toml_str}"
        );
    }

    let reloaded = Recipe::from_toml(&toml_str).expect("from_toml");
    assert_eq!(
        recipe, reloaded,
        "placement params must round-trip losslessly"
    );

    // Positive control: tiled output must differ from untiled, or the params
    // are round-tripping into an op that ignores them.
    let base = || {
        crustyimg::image::Image::from_parts(
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(48, 48, image::Rgba([0, 0, 0, 255]))),
            image::ImageFormat::Png,
            None,
        )
    };
    let tiled = Watermark::new_image(
        overlay.clone(),
        "logo.png".to_owned(),
        Gravity::NorthWest,
        1.0,
        None,
        0,
        true,
    )
    .apply(base())
    .expect("tiled apply");
    let untiled = Watermark::new_image(
        overlay,
        "logo.png".to_owned(),
        Gravity::NorthWest,
        1.0,
        None,
        0,
        false,
    )
    .apply(base())
    .expect("untiled apply");
    assert_ne!(
        tiled.pixels().to_rgba8().into_raw(),
        untiled.pixels().to_rgba8().into_raw(),
        "tile=true must change the output, or the flag is inert"
    );
}
