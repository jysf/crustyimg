//! Recipe TOML (de)serialization and operation pipeline construction (SPEC-006).
//!
//! A **recipe** is a versioned, ordered list of operation steps serialized as
//! TOML (DEC-005). The recipe layer sits above `operation/` in the layer order
//! (`recipe → operation → image`) and must NOT touch `clap`, `source`, `sink`,
//! or terminals.
//!
//! ## Round-trip guarantee
//!
//! `Recipe::from_toml(recipe.to_toml()?)? == recipe` over [`PartialEq`].
//! The equality is on the typed struct, not on byte-equal TOML strings
//! (serializers may reorder keys/whitespace).
//!
//! ## Validation (untrusted-input-hardening)
//!
//! - Malformed TOML → [`RecipeError::Parse`] (never a panic).
//! - Wrong `version` → [`RecipeError::UnsupportedVersion`] (checked before
//!   op resolution).
//! - Unknown op name → [`RecipeError::UnknownOperation`] (checked at
//!   `build_pipeline` time, never silently skipped).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::operation::{Operation, OperationParams, OperationRegistry};
use crate::pipeline::Pipeline;

// ─── SUPPORTED_VERSION ──────────────────────────────────────────────────────

/// The only recipe schema version this build understands.
///
/// `from_toml` rejects any `version` value other than this string.
pub const SUPPORTED_VERSION: &str = "1";

// ─── RecipeError ────────────────────────────────────────────────────────────

/// Errors that can occur while loading, saving, or building a [`Recipe`]
/// (DEC-007).
///
/// Typed and matchable; the binary maps these to a human-friendly message
/// and exit code at the CLI boundary (a future spec).
#[derive(Debug, Error)]
pub enum RecipeError {
    /// The recipe's `version` field is not supported by this build.
    #[error("unsupported recipe version '{found}' (supported: {supported})")]
    UnsupportedVersion {
        /// The `version` value found in the file.
        found: String,
        /// The only version this binary understands (`"1"`).
        supported: &'static str,
    },

    /// An op name in the recipe has no registered constructor.
    ///
    /// Never silently skipped — an unknown op name is a hard error so
    /// the caller knows the recipe was not fully applied (DEC-005,
    /// `untrusted-input-hardening`).
    #[error("unknown operation '{name}'")]
    UnknownOperation {
        /// The op name that had no constructor in the registry.
        name: String,
    },

    /// The TOML text could not be parsed.
    #[error("could not parse recipe TOML: {0}")]
    Parse(String),

    /// The recipe could not be serialized to TOML.
    #[error("could not serialize recipe to TOML: {0}")]
    Serialize(String),
}

// ─── RecipeStep ─────────────────────────────────────────────────────────────

/// One step in a recipe: an operation name plus its parameters.
///
/// The `params` field is **flattened** into the same `[[step]]` TOML table as
/// `op`, matching the documented schema:
///
/// ```toml
/// [[step]]
/// op = "invert"
/// # (no extra keys for parameterless ops)
/// ```
///
/// For parameterless ops, `OperationParams::None` serializes to an empty map
/// (zero extra keys), so the table contains only `op`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeStep {
    /// The registry key / recipe name of this operation.
    pub op: String,
    /// The operation's parameters, flattened into the step table.
    #[serde(flatten)]
    pub params: OperationParams,
}

// ─── Recipe ─────────────────────────────────────────────────────────────────

/// A versioned, ordered list of operation steps serialized as TOML (DEC-005).
///
/// Both `to_toml` / `from_toml` and `from_ops` / `build_pipeline` guarantee
/// that the typed struct round-trips losslessly: running `from_toml` on the
/// output of `to_toml` yields a `Recipe` equal to the original via
/// [`PartialEq`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    /// Schema version. Only `"1"` is supported; `from_toml` rejects others.
    pub version: String,

    /// Optional human label for the recipe (the `name` key in the TOML).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,

    /// Optional free-text description of what the recipe does.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,

    /// Ordered list of operation steps. Serialized as `[[step]]` (TOML
    /// array-of-tables). Empty is valid — an empty recipe builds an empty
    /// `Pipeline`, which is a no-op.
    #[serde(rename = "step", default)]
    pub steps: Vec<RecipeStep>,
}

impl Recipe {
    /// Build a `Recipe` from a live ordered slice of operations (the "save"
    /// direction).
    ///
    /// Each op's [`Operation::name`] and [`Operation::params`] are recorded
    /// into a [`RecipeStep`]. The resulting recipe carries
    /// [`SUPPORTED_VERSION`] and no `name` / `description` (those are
    /// user-supplied metadata, not derivable from the op list).
    pub fn from_ops(ops: &[Box<dyn Operation>]) -> Recipe {
        let steps = ops
            .iter()
            .map(|op| RecipeStep {
                op: op.name().to_owned(),
                params: op.params(),
            })
            .collect();
        Recipe {
            version: SUPPORTED_VERSION.to_owned(),
            name: None,
            description: None,
            steps,
        }
    }

    /// Serialize this recipe to a TOML string.
    ///
    /// Maps serialization failures to [`RecipeError::Serialize`] (no panics).
    pub fn to_toml(&self) -> Result<String, RecipeError> {
        toml::to_string(self).map_err(|e| RecipeError::Serialize(e.to_string()))
    }

    /// Parse a TOML string into a `Recipe` and validate the `version` field.
    ///
    /// - Malformed TOML → [`RecipeError::Parse`].
    /// - Any `version` other than [`SUPPORTED_VERSION`] → [`RecipeError::UnsupportedVersion`].
    ///
    /// Op name resolution does **not** happen here; call [`Recipe::build_pipeline`]
    /// to resolve ops through a registry.
    pub fn from_toml(s: &str) -> Result<Recipe, RecipeError> {
        let recipe: Recipe = toml::from_str(s).map_err(|e| RecipeError::Parse(e.to_string()))?;

        // Version check must occur before op resolution so callers get a clear
        // "unsupported version" error rather than a cascade of unknown-op errors.
        if recipe.version != SUPPORTED_VERSION {
            return Err(RecipeError::UnsupportedVersion {
                found: recipe.version,
                supported: SUPPORTED_VERSION,
            });
        }

        Ok(recipe)
    }

    /// Resolve each step's op name through the `registry` and build a
    /// [`Pipeline`] (the "load" direction).
    ///
    /// An op name not found in `registry` surfaces immediately as
    /// [`RecipeError::UnknownOperation`] — never silently skipped
    /// (`untrusted-input-hardening`).
    pub fn build_pipeline(&self, registry: &OperationRegistry) -> Result<Pipeline, RecipeError> {
        let mut pipeline = Pipeline::new();
        for step in &self.steps {
            let op = registry.build(&step.op, &step.params).map_err(|_| {
                RecipeError::UnknownOperation {
                    name: step.op.clone(),
                }
            })?;
            pipeline = pipeline.push(op);
        }
        Ok(pipeline)
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::{Identity, Invert};

    #[test]
    fn default_version_is_one() {
        // The supported version constant must be "1".
        assert_eq!(SUPPORTED_VERSION, "1");
        // A freshly built recipe carries it.
        let r = Recipe::from_ops(&[]);
        assert_eq!(r.version, "1");
    }

    #[test]
    fn from_ops_records_names_in_order() {
        let ops: Vec<Box<dyn Operation>> = vec![Box::new(Identity), Box::new(Invert)];
        let recipe = Recipe::from_ops(&ops);
        assert_eq!(recipe.steps.len(), 2);
        assert_eq!(recipe.steps[0].op, "identity");
        assert_eq!(recipe.steps[1].op, "invert");
    }

    #[test]
    fn empty_recipe_round_trips_and_builds_empty_pipeline() {
        let r = Recipe {
            version: SUPPORTED_VERSION.to_owned(),
            name: None,
            description: None,
            steps: vec![],
        };
        // Round-trip through TOML.
        let toml_str = r.to_toml().expect("serialization should succeed");
        let r2 = Recipe::from_toml(&toml_str).expect("parse should succeed");
        assert_eq!(r, r2, "empty recipe must round-trip through TOML");

        // Build pipeline from the re-parsed recipe.
        let registry = OperationRegistry::with_builtins();
        let pipeline = r2
            .build_pipeline(&registry)
            .expect("empty pipeline should build without error");
        assert!(
            pipeline.is_empty(),
            "pipeline built from empty recipe must be empty"
        );
    }
}
