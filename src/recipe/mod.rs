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
//! - An unknown **top-level** key (`version` / `name` / `description` / `step`) →
//!   [`RecipeError::Parse`] via `deny_unknown_fields` on [`Recipe`], matching the
//!   manifest (DEC-057) and lockfile (DEC-059) discipline. This catches the
//!   silent footgun where a typo'd top-level key — `steps = [...]` or `stpe` — used
//!   to be ignored, leaving a zero-step recipe that copies its input unchanged
//!   (SPEC-068).
//! - Wrong `version` → [`RecipeError::UnsupportedVersion`] (checked before
//!   op resolution).
//! - Unknown op name → [`RecipeError::UnknownOperation`] (checked at
//!   `build_pipeline` time, never silently skipped).
//!
//! **Accepted (SPEC-068 / DEC-061):** an unknown **step** key is still tolerated.
//! [`RecipeStep`] cannot carry `deny_unknown_fields` — its `#[serde(flatten)]
//! params` (a `BTreeMap`) absorbs every extra key — and a strict per-step check
//! needs each operation to publish its accepted param names through the registry.
//! An extra step key is inert (never a path, a panic, or a wrong output), so it is
//! recorded as an accepted risk and filed as a follow-up, not fixed here.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::operation::registry::RegistryError;
use crate::operation::{Operation, OperationParams, OperationRegistry};
use crate::pipeline::Pipeline;

// ─── SUPPORTED_VERSION ──────────────────────────────────────────────────────

/// The only recipe schema version this build understands.
///
/// `from_toml` rejects any `version` value other than this string.
pub const SUPPORTED_VERSION: &str = "1";

// ─── Resource limits (DEC-036) ───────────────────────────────────────────────

/// Maximum allowed byte length of a recipe TOML string (64 KiB).
///
/// `from_toml` checks `s.len()` against this **before** calling `toml::from_str`
/// so an oversized string is never parsed (parse-time DoS prevention). The CLI
/// `run_apply` also checks the on-disk file size via `std::fs::metadata` before
/// reading the file into memory. Reject only on `>`; equality is accepted.
pub const RECIPE_MAX_BYTES: usize = 64 * 1024;

/// Maximum allowed number of steps in a recipe (1024).
///
/// `from_toml` checks `recipe.steps.len()` after the version check so an
/// over-version recipe is still `UnsupportedVersion`, not `TooManySteps`.
/// Reject only on `>`; equality is accepted.
pub const RECIPE_MAX_STEPS: usize = 1024;

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

    /// An op name resolved but its params were invalid (DEC-014).
    ///
    /// Distinct from `UnknownOperation` so callers can distinguish a typo in the
    /// op name from a valid op name with bad params.
    #[error("invalid operation '{name}': {reason}")]
    InvalidOperation {
        /// The op name that resolved but whose params were rejected.
        name: String,
        /// Human-readable reason the params were rejected.
        reason: String,
    },

    /// The TOML text could not be parsed.
    #[error("could not parse recipe TOML: {0}")]
    Parse(String),

    /// The recipe could not be serialized to TOML.
    #[error("could not serialize recipe to TOML: {0}")]
    Serialize(String),

    /// The recipe text exceeds [`RECIPE_MAX_BYTES`] (checked before parsing).
    ///
    /// Prevents parse-time memory/CPU exhaustion from a hostile oversized recipe.
    /// The CLI also guards file size before `read_to_string` via the same constant.
    #[error("recipe is too large ({size} bytes; max {max})")]
    TooLarge {
        /// The actual byte length of the oversized string.
        size: usize,
        /// The cap that was exceeded (`RECIPE_MAX_BYTES`).
        max: usize,
    },

    /// The recipe has more than [`RECIPE_MAX_STEPS`] steps (checked after parsing
    /// and the version check, before pipeline build).
    ///
    /// Prevents pipeline-build exhaustion from a recipe with an excessive step count.
    #[error("recipe has too many steps ({count}; max {max})")]
    TooManySteps {
        /// The actual number of steps in the recipe.
        count: usize,
        /// The cap that was exceeded (`RECIPE_MAX_STEPS`).
        max: usize,
    },
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
#[serde(deny_unknown_fields)]
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
    /// - String length exceeds [`RECIPE_MAX_BYTES`] → [`RecipeError::TooLarge`]
    ///   (checked **before** `toml::from_str` to avoid parse-time DoS).
    /// - Malformed TOML → [`RecipeError::Parse`].
    /// - Any `version` other than [`SUPPORTED_VERSION`] → [`RecipeError::UnsupportedVersion`].
    /// - Step count exceeds [`RECIPE_MAX_STEPS`] → [`RecipeError::TooManySteps`]
    ///   (checked after the version check so a bad-version recipe is still
    ///   `UnsupportedVersion`, not `TooManySteps`).
    ///
    /// Op name resolution does **not** happen here; call [`Recipe::build_pipeline`]
    /// to resolve ops through a registry.
    pub fn from_toml(s: &str) -> Result<Recipe, RecipeError> {
        // Size check BEFORE parsing: reject an oversized string without touching toml::from_str.
        if s.len() > RECIPE_MAX_BYTES {
            return Err(RecipeError::TooLarge {
                size: s.len(),
                max: RECIPE_MAX_BYTES,
            });
        }

        let recipe: Recipe = toml::from_str(s).map_err(|e| RecipeError::Parse(e.to_string()))?;

        // Version check must occur before op resolution so callers get a clear
        // "unsupported version" error rather than a cascade of unknown-op errors.
        if recipe.version != SUPPORTED_VERSION {
            return Err(RecipeError::UnsupportedVersion {
                found: recipe.version,
                supported: SUPPORTED_VERSION,
            });
        }

        // Step count check AFTER version check: an over-version recipe is UnsupportedVersion,
        // not TooManySteps.
        if recipe.steps.len() > RECIPE_MAX_STEPS {
            return Err(RecipeError::TooManySteps {
                count: recipe.steps.len(),
                max: RECIPE_MAX_STEPS,
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
            let op = registry
                .build(&step.op, &step.params)
                .map_err(|e| match e {
                    RegistryError::Unknown { name } => RecipeError::UnknownOperation { name },
                    RegistryError::InvalidParams { op, reason } => RecipeError::InvalidOperation {
                        name: op.to_owned(),
                        reason,
                    },
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

    // ─── SPEC-035: resource-limit unit tests ─────────────────────────────────

    /// A string of length `RECIPE_MAX_BYTES + 1` must be rejected as TooLarge
    /// BEFORE being parsed (so even a TOML-comment-only oversized string fails).
    #[test]
    fn from_toml_rejects_oversized_recipe() {
        // Build a string that exceeds the cap by exactly 1 byte.
        // Use '#' so it would be valid TOML (a comment), confirming the size
        // check fires before toml::from_str is called.
        let oversized = "#".repeat(RECIPE_MAX_BYTES + 1);
        let result = Recipe::from_toml(&oversized);
        assert!(
            matches!(result, Err(RecipeError::TooLarge { size, max })
                if size == RECIPE_MAX_BYTES + 1 && max == RECIPE_MAX_BYTES),
            "expected TooLarge, got {result:?}"
        );
    }

    /// A valid recipe whose text length is exactly RECIPE_MAX_BYTES must be accepted
    /// (boundary is inclusive; only `>` is rejected).
    #[test]
    fn from_toml_accepts_recipe_at_size_cap() {
        // Start with a minimal valid recipe and pad it with TOML comments up to
        // exactly RECIPE_MAX_BYTES. Comments don't affect parsing.
        let base = "version = \"1\"\n";
        assert!(
            base.len() <= RECIPE_MAX_BYTES,
            "base recipe must not itself exceed the cap"
        );
        let padding = "#".repeat(RECIPE_MAX_BYTES - base.len());
        let at_cap = format!("{base}{padding}");
        assert_eq!(
            at_cap.len(),
            RECIPE_MAX_BYTES,
            "padded recipe must be exactly RECIPE_MAX_BYTES"
        );
        let result = Recipe::from_toml(&at_cap);
        assert!(
            result.is_ok(),
            "recipe at exactly the byte cap must be accepted, got {result:?}"
        );
    }

    /// A recipe with RECIPE_MAX_STEPS + 1 identity steps must be rejected as
    /// TooManySteps. The 1025-step fixture is ~18 KB, well under the 64 KiB byte
    /// cap, so the step gate — not the size gate — is what fires.
    #[test]
    fn from_toml_rejects_too_many_steps() {
        let n = RECIPE_MAX_STEPS + 1;
        let step_block = "[[step]]\nop = \"identity\"\n";
        let toml_str = format!("version = \"1\"\n{}", step_block.repeat(n));
        // Verify the fixture is under the byte cap (exercises step gate, not size gate).
        assert!(
            toml_str.len() <= RECIPE_MAX_BYTES,
            "step-cap fixture must be under the byte cap; len = {}",
            toml_str.len()
        );
        let result = Recipe::from_toml(&toml_str);
        assert!(
            matches!(result, Err(RecipeError::TooManySteps { count, max })
                if count == n && max == RECIPE_MAX_STEPS),
            "expected TooManySteps {{ count: {n}, max: {RECIPE_MAX_STEPS} }}, got {result:?}"
        );
    }

    /// A recipe with exactly RECIPE_MAX_STEPS identity steps must be accepted
    /// (boundary is inclusive; only `>` is rejected).
    #[test]
    fn from_toml_accepts_recipe_at_step_cap() {
        let n = RECIPE_MAX_STEPS;
        let step_block = "[[step]]\nop = \"identity\"\n";
        let toml_str = format!("version = \"1\"\n{}", step_block.repeat(n));
        let result = Recipe::from_toml(&toml_str);
        assert!(
            result.is_ok(),
            "recipe at exactly the step cap must be accepted, got {result:?}"
        );
    }

    /// A normal small recipe must still load, round-trip, and build its pipeline
    /// unchanged (no regression to SPEC-006 behavior).
    #[test]
    fn from_toml_normal_recipe_still_round_trips() {
        let toml_str =
            "version = \"1\"\n\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 800\n";
        let recipe = Recipe::from_toml(toml_str).expect("should parse successfully");
        // Round-trip.
        let serialized = recipe.to_toml().expect("to_toml should succeed");
        let reloaded = Recipe::from_toml(&serialized).expect("re-parse should succeed");
        assert_eq!(recipe, reloaded, "recipe must round-trip through TOML");
        // Pipeline builds without error.
        let registry = OperationRegistry::with_builtins();
        recipe
            .build_pipeline(&registry)
            .expect("pipeline build should succeed");
    }

    // ─── SPEC-068: unknown-key posture ────────────────────────────────────────

    /// A hostile/typo'd **top-level** key is a hard parse error (`deny_unknown_fields`),
    /// matching the manifest + lockfile discipline. Before SPEC-068 this was silently
    /// tolerated: `stpe`/`steps`/`verison` would parse to a zero-step recipe that
    /// copies its input unchanged — a silent wrong output on a committed file the
    /// maintainer did not write. Driven from a hand-authored TOML string, not a struct.
    #[test]
    fn from_toml_rejects_unknown_top_level_key() {
        // A plain typo, and the specific footgun: `steps` (plural) vs the `step` key.
        for bad in [
            "version = \"1\"\nbogus = 42\n",
            "version = \"1\"\nsteps = []\n",
            "version = \"1\"\n[[step]]\nop = \"invert\"\n\n[extra]\nx = 1\n",
        ] {
            let result = Recipe::from_toml(bad);
            assert!(
                matches!(&result, Err(RecipeError::Parse(_))),
                "an unknown top-level key must be a typed Parse error, got {result:?}"
            );
        }
    }

    /// PINS the accepted risk (SPEC-068 / DEC-061): an unknown **step** param is
    /// tolerated by design — `RecipeStep`'s `#[serde(flatten)] params` absorbs it,
    /// and it is inert. If a future spec adds strict per-op param validation, this
    /// test flips deliberately, not by accident.
    #[test]
    fn from_toml_tolerates_unknown_step_param_by_design() {
        // An extra key on a param-taking op (resize) and on a paramless op (invert):
        // both parse and build a working pipeline; the extra key is dropped.
        for toml_str in [
            "version = \"1\"\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 8\nbogus = \"x\"\n",
            "version = \"1\"\n[[step]]\nop = \"invert\"\nbogus = \"x\"\n",
        ] {
            let recipe = Recipe::from_toml(toml_str)
                .expect("an unknown STEP param is tolerated (flatten), not rejected");
            let registry = OperationRegistry::with_builtins();
            recipe
                .build_pipeline(&registry)
                .expect("the pipeline still builds; the extra param is inert");
        }
    }

    /// An unsupported version must still be rejected as UnsupportedVersion even
    /// after the size/step caps are added (existing behavior unchanged).
    #[test]
    fn from_toml_unsupported_version_still_rejected() {
        let toml_str = "version = \"2\"\n";
        let result = Recipe::from_toml(toml_str);
        assert!(
            matches!(result, Err(RecipeError::UnsupportedVersion { ref found, .. }) if found == "2"),
            "expected UnsupportedVersion with found=\"2\", got {result:?}"
        );
    }
}
