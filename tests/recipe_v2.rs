//! Integration tests for SPEC-127: `Recipe` gains `format`/`quality`, gated
//! behind `version = "2"`.
//!
//! These exercise the public crate API directly (no CLI process spawn needed
//! for the schema-level assertions; `tests/apply_batch.rs` and a new
//! `tests/optimize.rs` cover the CLI-facing precedence/carve-out behaviour).

use crustyimg::recipe::{Recipe, RecipeError};

/// AC-1: a `version = "2"` recipe declaring both `format` and `quality`
/// round-trips losslessly through TOML — `from_toml(to_toml(r)) == r`.
#[test]
fn v2_round_trips_format_and_quality() {
    let toml_str = r#"
version = "2"
format = "webp"
quality = 82

[[step]]
op = "identity"
"#;
    let recipe = Recipe::from_toml(toml_str).expect("a v2 recipe with format+quality must parse");
    assert_eq!(recipe.version, "2");
    assert_eq!(recipe.format.as_deref(), Some("webp"));
    assert_eq!(recipe.quality, Some(82));

    let serialized = recipe.to_toml().expect("to_toml should succeed");
    let reloaded = Recipe::from_toml(&serialized).expect("re-parse should succeed");
    assert_eq!(
        recipe, reloaded,
        "a v2 recipe with format+quality must round-trip losslessly"
    );
    // The fields must actually be present in the serialized form, not just
    // survive because PartialEq happens to ignore them.
    assert!(
        serialized.contains("format = \"webp\""),
        "serialized TOML must carry format, got:\n{serialized}"
    );
    assert!(
        serialized.contains("quality = 82"),
        "serialized TOML must carry quality, got:\n{serialized}"
    );
}

/// AC-1's other half — the strand guard. A v1 recipe using NEITHER new field
/// must still round-trip, and — the highest-consequence line in the spec —
/// `to_toml` must keep emitting `version = "1"`, never bumping it
/// unconditionally. Emitting `"2"` unconditionally would strand every
/// existing recipe on its next `--save-recipe`.
#[test]
fn v1_still_round_trips_and_stays_v1() {
    let toml_str = r#"
version = "1"

[[step]]
op = "resize"
mode = "max"
width = 800
"#;
    let recipe = Recipe::from_toml(toml_str).expect("a plain v1 recipe must parse");
    assert_eq!(recipe.version, "1");
    assert_eq!(recipe.format, None);
    assert_eq!(recipe.quality, None);

    let serialized = recipe.to_toml().expect("to_toml should succeed");
    assert!(
        serialized.contains("version = \"1\""),
        "a v1 recipe using neither new field must still serialize as \
         version \"1\", got:\n{serialized}"
    );
    assert!(
        !serialized.contains("format"),
        "format must not appear in the serialized TOML when unset, got:\n{serialized}"
    );
    assert!(
        !serialized.contains("quality"),
        "quality must not appear in the serialized TOML when unset, got:\n{serialized}"
    );

    let reloaded = Recipe::from_toml(&serialized).expect("re-parse should succeed");
    assert_eq!(recipe, reloaded, "a v1 recipe must round-trip through TOML");
    assert_eq!(
        reloaded.version, "1",
        "the round-tripped recipe must still be v1"
    );
}

/// AC-5: a recipe using `format` OR `quality` without declaring
/// `version = "2"` is rejected with a typed error naming the field and the
/// version actually found — not a generic parse failure.
#[test]
fn new_field_without_v2_is_rejected() {
    let with_format = r#"
version = "1"
format = "png"

[[step]]
op = "identity"
"#;
    let result = Recipe::from_toml(with_format);
    assert!(
        matches!(
            &result,
            Err(RecipeError::NewFieldNeedsVersion2 { field, found })
                if *field == "format" && found == "1"
        ),
        "a v1 recipe with `format` must be NewFieldNeedsVersion2, got {result:?}"
    );

    let with_quality = r#"
version = "1"
quality = 75

[[step]]
op = "identity"
"#;
    let result = Recipe::from_toml(with_quality);
    assert!(
        matches!(
            &result,
            Err(RecipeError::NewFieldNeedsVersion2 { field, found })
                if *field == "quality" && found == "1"
        ),
        "a v1 recipe with `quality` must be NewFieldNeedsVersion2, got {result:?}"
    );

    // Sanity: the same recipe, declaring version 2, is accepted.
    let as_v2 = with_format.replacen("version = \"1\"", "version = \"2\"", 1);
    let ok = Recipe::from_toml(&as_v2);
    assert!(
        ok.is_ok(),
        "the identical fields under version 2 must be accepted, got {ok:?}"
    );
}
