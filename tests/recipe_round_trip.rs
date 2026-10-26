//! Integration tests for the recipe TOML round-trip and operation registry
//! (SPEC-006, SPEC-010). Exercises the public crate API; no disk I/O beyond
//! the in-memory fixture images constructed here.

use std::collections::BTreeMap;

use ::image::{DynamicImage, ImageFormat, RgbaImage};

use crustyimg::image::Image;
use crustyimg::operation::{Invert, OperationParams, OperationRegistry};
use crustyimg::pipeline::Pipeline;
use crustyimg::recipe::{Recipe, RecipeError, RecipeStep, SUPPORTED_VERSION};

// ─── Fixture helper ─────────────────────────────────────────────────────────

/// Build a small in-memory RGBA `Image` for tests. Mirrors the `make_image`
/// helper in `src/operation/mod.rs` and `src/pipeline/mod.rs` — native
/// in-memory fixture, no disk I/O (AGENTS.md §12).
fn make_image(w: u32, h: u32) -> Image {
    let buf = RgbaImage::from_fn(w, h, |x, y| {
        ::image::Rgba([(x * 10 + 5) as u8, (y * 10 + 5) as u8, 50, 200])
    });
    Image::from_parts(DynamicImage::ImageRgba8(buf), ImageFormat::Png, None)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn recipe_round_trips_through_toml() {
    // Build a recipe from a live op list: [invert, invert].
    let ops: Vec<Box<dyn crustyimg::operation::Operation>> =
        vec![Box::new(Invert), Box::new(Invert)];
    let original = Recipe::from_ops(&ops);

    // Serialize to TOML then parse back.
    let toml_str = original.to_toml().expect("to_toml should succeed");
    let reloaded = Recipe::from_toml(&toml_str).expect("from_toml should succeed");

    assert_eq!(
        original, reloaded,
        "recipe must round-trip through TOML via typed PartialEq"
    );
}

#[test]
fn serialized_toml_matches_schema() {
    // Build [invert, invert] recipe and verify the TOML shape.
    let ops: Vec<Box<dyn crustyimg::operation::Operation>> =
        vec![Box::new(Invert), Box::new(Invert)];
    let recipe = Recipe::from_ops(&ops);
    let toml_str = recipe.to_toml().expect("to_toml should succeed");

    // Must contain the version field.
    assert!(
        toml_str.contains("version = \"1\""),
        "TOML must contain `version = \"1\"`, got:\n{toml_str}"
    );

    // Must contain exactly two [[step]] tables.
    let step_count = toml_str.matches("[[step]]").count();
    assert_eq!(
        step_count, 2,
        "TOML must contain exactly 2 [[step]] tables, found {step_count} in:\n{toml_str}"
    );

    // Each step must carry op = "invert".
    let op_count = toml_str.matches("op = \"invert\"").count();
    assert_eq!(
        op_count, 2,
        "TOML must contain exactly 2 `op = \"invert\"` entries, found {op_count} in:\n{toml_str}"
    );
}

#[test]
fn registry_resolves_builtins_by_name() {
    let registry = OperationRegistry::with_builtins();

    let identity_op = registry
        .build("identity", &OperationParams::empty())
        .expect("build('identity') should succeed");
    assert_eq!(identity_op.name(), "identity");

    let invert_op = registry
        .build("invert", &OperationParams::empty())
        .expect("build('invert') should succeed");
    assert_eq!(invert_op.name(), "invert");
}

#[test]
fn unknown_operation_is_typed_error() {
    // A recipe with an unknown op name must produce RecipeError::UnknownOperation.
    let toml_str = "version = \"1\"\n\n[[step]]\nop = \"bogus\"\n";
    let recipe = Recipe::from_toml(toml_str).expect("parse should succeed");

    let registry = OperationRegistry::with_builtins();
    let result = recipe.build_pipeline(&registry);

    assert!(
        matches!(
            &result,
            Err(RecipeError::UnknownOperation { name }) if name == "bogus"
        ),
        "expected RecipeError::UnknownOperation {{ name: \"bogus\" }}"
    );
}

#[test]
fn unsupported_version_is_typed_error() {
    let toml_str = "version = \"999\"\n";
    let result = Recipe::from_toml(toml_str);

    assert!(
        matches!(result, Err(RecipeError::UnsupportedVersion { .. })),
        "expected RecipeError::UnsupportedVersion, got {:?}",
        result
    );
}

#[test]
fn malformed_toml_is_typed_error() {
    // "not = = valid" is syntactically malformed TOML.
    let result = Recipe::from_toml("not = = valid");

    assert!(
        matches!(result, Err(RecipeError::Parse(_))),
        "expected RecipeError::Parse, got {:?}",
        result
    );
}

#[test]
fn recipe_drives_pipeline_same_as_direct() {
    // Build a small test image.
    let img = make_image(4, 4);
    let img2 = img.clone();

    // (a) Pipeline built from a recipe [invert, invert] via the registry.
    let recipe = Recipe {
        version: SUPPORTED_VERSION.to_owned(),
        name: None,
        description: None,
        format: None,
        quality: None,
        steps: vec![
            RecipeStep {
                op: "invert".to_owned(),
                params: OperationParams::empty(),
            },
            RecipeStep {
                op: "invert".to_owned(),
                params: OperationParams::empty(),
            },
        ],
    };
    let registry = OperationRegistry::with_builtins();
    let recipe_pipeline = recipe
        .build_pipeline(&registry)
        .expect("pipeline should build from recipe");
    let recipe_result = recipe_pipeline
        .run(img)
        .expect("recipe pipeline should run without error");

    // (b) Hand-built pipeline [invert, invert].
    let direct_pipeline = Pipeline::new()
        .push(Box::new(Invert))
        .push(Box::new(Invert));
    let direct_result = direct_pipeline
        .run(img2)
        .expect("direct pipeline should run without error");

    assert_eq!(
        recipe_result.pixels().to_rgba8().into_raw(),
        direct_result.pixels().to_rgba8().into_raw(),
        "recipe-driven and hand-built [invert, invert] pipelines must produce identical pixels"
    );
}

// ─── SPEC-010 integration tests ──────────────────────────────────────────────

#[test]
fn resize_recipe_round_trips() {
    // Build a Recipe with one resize step: mode="max", width=1200, NO height.
    // This mirrors docs/data-model.md's worked step.
    let mut params_map = BTreeMap::new();
    params_map.insert("mode".to_owned(), toml::Value::String("max".into()));
    params_map.insert("width".to_owned(), toml::Value::Integer(1200));
    let params = OperationParams::from_map(params_map);

    let original = Recipe {
        version: SUPPORTED_VERSION.to_owned(),
        name: None,
        description: None,
        format: None,
        quality: None,
        steps: vec![RecipeStep {
            op: "resize".to_owned(),
            params,
        }],
    };

    // Serialize to TOML and verify content.
    let toml_str = original.to_toml().expect("to_toml should succeed");
    assert!(
        toml_str.contains("op = \"resize\""),
        "TOML must contain op = \"resize\", got:\n{toml_str}"
    );
    assert!(
        toml_str.contains("mode = \"max\""),
        "TOML must contain mode = \"max\", got:\n{toml_str}"
    );
    assert!(
        toml_str.contains("width = 1200"),
        "TOML must contain width = 1200, got:\n{toml_str}"
    );
    assert!(
        !toml_str.contains("height"),
        "TOML must NOT contain 'height' for max mode, got:\n{toml_str}"
    );

    // Round-trip via from_toml.
    let reloaded = Recipe::from_toml(&toml_str).expect("from_toml should succeed");
    assert_eq!(
        original, reloaded,
        "resize recipe must round-trip through TOML via PartialEq"
    );
}

#[test]
fn resize_invalid_params_is_invalid_operation() {
    // A resize step missing 'mode': from_toml succeeds (parse-valid),
    // but build_pipeline must return RecipeError::InvalidOperation (NOT UnknownOperation).
    let toml_str = "version = \"1\"\n\n[[step]]\nop = \"resize\"\nwidth = 100\n";
    let recipe = Recipe::from_toml(toml_str).expect("parse should succeed");

    let registry = OperationRegistry::with_builtins();
    let result = recipe.build_pipeline(&registry);

    assert!(
        matches!(
            &result,
            Err(RecipeError::InvalidOperation { name, .. }) if name == "resize"
        ),
        "expected RecipeError::InvalidOperation {{ name: \"resize\" }}"
    );
}

// ─── Golden corpus: `to_toml()` stays byte-identical ─────────────────────────
//
// Every recipe in `tests/fixtures/recipe_corpus/*.toml` plus the bundled
// `recipes/*.toml` is parsed and re-emitted; the result must equal its
// `.golden` sibling byte for byte. The goldens were captured from code that
// predates strict step-param validation, so a validation change that rewrote
// a step's map (and so the cache key and lockfile hash built from it) fails
// here. Regenerate only with `CRUSTYIMG_BLESS=1`, and only on the code the
// goldens are meant to describe.

/// The corpus recipes, in a stable order. Asserted against the directory so a
/// new file cannot be silently skipped.
const CORPUS: &[&str] = &[
    "v1_resize_cover",
    "v1_resize_exact",
    "v1_resize_fill",
    "v1_resize_fit",
    "v1_resize_max",
    "v1_resize_percent",
    "v1_watermark_image_placement",
    "v1_watermark_text_font",
    "v2_format_quality_optimize",
    "v2_watermark_integer_numbers",
];

/// The bundled recipes shipped in `recipes/`.
const BUNDLED: &[&str] = &["gallery", "product", "web"];

fn corpus_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/recipe_corpus")
}

/// `(source path, golden path)` for every recipe under the golden pin.
fn golden_pairs() -> Vec<(std::path::PathBuf, std::path::PathBuf)> {
    let dir = corpus_dir();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut pairs: Vec<_> = CORPUS
        .iter()
        .map(|stem| {
            (
                dir.join(format!("{stem}.toml")),
                dir.join(format!("{stem}.golden")),
            )
        })
        .collect();
    pairs.extend(BUNDLED.iter().map(|name| {
        (
            root.join(format!("recipes/{name}.toml")),
            dir.join(format!("bundled_{name}.golden")),
        )
    }));
    pairs
}

/// Read a recipe as text with line endings normalized, so a Windows checkout
/// that converts to CRLF still parses to the same recipe.
fn read_text(path: &std::path::Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
        .replace("\r\n", "\n")
}

/// A tiny decodable PNG to stand in for a watermark `image` asset.
fn tiny_png() -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, ::image::Rgba([1, 2, 3, 255])))
        .write_to(&mut out, ImageFormat::Png)
        .expect("encode tiny png");
    out.into_inner()
}

#[test]
fn to_toml_is_byte_identical_to_main_over_the_corpus() {
    // The corpus list must match the directory exactly.
    let mut on_disk: Vec<String> = std::fs::read_dir(corpus_dir())
        .expect("corpus dir")
        .filter_map(|e| {
            let name = e.ok()?.file_name().to_string_lossy().into_owned();
            name.strip_suffix(".toml").map(str::to_owned)
        })
        .collect();
    on_disk.sort();
    assert_eq!(on_disk, CORPUS, "CORPUS must list every corpus recipe");

    let pairs = golden_pairs();
    assert_eq!(pairs.len(), 13, "10 corpus recipes + 3 bundled recipes");

    let registry = OperationRegistry::with_builtins();
    let png = tiny_png();
    for (src, golden) in &pairs {
        let recipe = Recipe::from_toml(&read_text(src))
            .unwrap_or_else(|e| panic!("{} must parse: {e}", src.display()));
        let emitted = recipe.to_toml().expect("to_toml");
        assert_eq!(
            emitted,
            read_text(golden),
            "{}: to_toml() changed from its golden",
            src.display()
        );

        // It must also still build a pipeline. Asset keys get stand-in bytes,
        // as the CLI's resolver would attach; a terminal `optimize` marker is
        // recipe syntax, not an op, so it is dropped first.
        let mut pixel = recipe.clone();
        if pixel.steps.last().is_some_and(|s| s.op == "optimize") {
            pixel.steps.pop();
        }
        for step in &mut pixel.steps {
            if step.params.get_str("image").is_some() {
                step.params.set_resolved_bytes("image", png.clone());
            }
            if step.params.get_str("font").is_some() {
                step.params
                    .set_resolved_bytes("font", crustyimg::text::DEFAULT_FONT.to_vec());
            }
        }
        pixel
            .build_pipeline(&registry)
            .unwrap_or_else(|e| panic!("{} must build a pipeline: {e}", src.display()));
    }
}

/// Writes the `.golden` files. Not an assertion: run it with
/// `CRUSTYIMG_BLESS=1 cargo test --test recipe_round_trip -- --ignored bless`
/// only when the goldens should describe the current code.
#[test]
#[ignore]
fn bless_recipe_corpus_goldens() {
    if std::env::var_os("CRUSTYIMG_BLESS").is_none() {
        eprintln!("CRUSTYIMG_BLESS is not set; not writing goldens");
        return;
    }
    for (src, golden) in golden_pairs() {
        let recipe = Recipe::from_toml(&read_text(&src))
            .unwrap_or_else(|e| panic!("{} must parse: {e}", src.display()));
        std::fs::write(&golden, recipe.to_toml().expect("to_toml")).expect("write golden");
    }
}
