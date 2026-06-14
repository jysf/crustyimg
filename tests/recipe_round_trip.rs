//! Integration tests for the recipe TOML round-trip and operation registry
//! (SPEC-006). Exercises the public crate API; no disk I/O beyond the in-memory
//! fixture images constructed here.

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
        .build("identity", &OperationParams::None)
        .expect("build('identity') should succeed");
    assert_eq!(identity_op.name(), "identity");

    let invert_op = registry
        .build("invert", &OperationParams::None)
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
        steps: vec![
            RecipeStep {
                op: "invert".to_owned(),
                params: OperationParams::None,
            },
            RecipeStep {
                op: "invert".to_owned(),
                params: OperationParams::None,
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
