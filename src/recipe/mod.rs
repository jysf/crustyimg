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

pub mod bundled;

use crate::operation::registry::RegistryError;
use crate::operation::{Operation, OperationParams, OperationRegistry};
use crate::pipeline::Pipeline;

// ─── SUPPORTED_VERSION ──────────────────────────────────────────────────────

/// The original recipe schema version: `version`, `name`, `description`,
/// `steps` — no `format`/`quality`. `from_ops` still emits this (SPEC-127
/// changes nothing about what `edit --save-recipe` writes).
pub const SUPPORTED_VERSION: &str = "1";

/// The recipe schema version required to set `format` and/or `quality`
/// (SPEC-127, Call 1). `from_toml` rejects a recipe that sets either field
/// without declaring this version — see [`RecipeError::NewFieldNeedsVersion2`].
pub const SUPPORTED_VERSION_2: &str = "2";

/// Every version this build understands, for the "supported: …" half of
/// [`RecipeError::UnsupportedVersion`]'s message.
const SUPPORTED_VERSIONS_DISPLAY: &str = "1, 2";

/// Is `v` one of the versions this build understands (`from_toml`'s FIRST
/// gate — a value outside this set is [`RecipeError::UnsupportedVersion`],
/// checked before the narrower "new field needs v2" rule below).
fn is_supported_version(v: &str) -> bool {
    v == SUPPORTED_VERSION || v == SUPPORTED_VERSION_2
}

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

    /// `format` and/or `quality` were set without declaring `version = "2"`
    /// (SPEC-127, Call 1).
    ///
    /// Those fields are new in schema version 2, gated deliberately: a v1
    /// recipe that sets them anyway (rather than a v1 recipe that simply
    /// omits them, which parses unchanged) gets this actionable message
    /// instead of a `deny_unknown_fields` TOML parse error pointing at an
    /// arbitrary line — the asymmetry SPEC-127's design measured on `main`.
    #[error("recipe field '{field}' requires `version = \"2\"` (found version \"{found}\")")]
    NewFieldNeedsVersion2 {
        /// Which new field triggered the gate: `"format"` or `"quality"`.
        field: &'static str,
        /// The `version` value the recipe actually declared.
        found: String,
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
    /// Schema version. `"1"` and `"2"` are both accepted; `from_toml` rejects
    /// any other value. `"2"` is required by a recipe that sets `format`
    /// and/or `quality` (SPEC-127, Call 1) — an older binary handed a
    /// `version = "2"` recipe fails with a message naming the version, not a
    /// generic TOML parse error.
    pub version: String,

    /// Optional human label for the recipe (the `name` key in the TOML).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,

    /// Optional free-text description of what the recipe does.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,

    /// The output format to encode to (e.g. `"png"`, `"jpeg"`, `"webp"`,
    /// `"avif"`) — a string resolved the same way `--format` is
    /// (`crate::cli::common::resolve_format`), so the recipe layer stays free
    /// of any format-specific validation of its own. Requires
    /// `version = "2"` (SPEC-127, Call 1): `from_toml` rejects a `"1"`
    /// recipe that sets this.
    ///
    /// One rung in the precedence chain `--format` > `-o` ext >
    /// `recipe.format` > preserve source (DEC-015, extended by SPEC-127
    /// Call 2) — resolved at the CALL SITE (`apply`/`build`/`wasm::transform`),
    /// never inside `encode_one`, so the decision stays where the CLI flags
    /// that can override it also live.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub format: Option<String>,

    /// The output encoder quality (0-100, where the resolved format supports
    /// one). Same version-2 gate as `format`. One rung in the precedence
    /// chain `-q` > `recipe.quality` > the format's own default.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub quality: Option<u8>,

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
            format: None,
            quality: None,
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
    /// - Any `version` other than [`SUPPORTED_VERSION`]/[`SUPPORTED_VERSION_2`]
    ///   → [`RecipeError::UnsupportedVersion`].
    /// - `format`/`quality` set without `version = "2"` →
    ///   [`RecipeError::NewFieldNeedsVersion2`] (SPEC-127, Call 1; checked
    ///   after the version-support check so a version this build does not
    ///   recognize at all is still `UnsupportedVersion`).
    /// - Step count exceeds [`RECIPE_MAX_STEPS`] → [`RecipeError::TooManySteps`]
    ///   (checked after both version checks so a bad-version recipe is still
    ///   `UnsupportedVersion`/`NewFieldNeedsVersion2`, not `TooManySteps`).
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

        // Version-support check must occur before op resolution so callers get
        // a clear "unsupported version" error rather than a cascade of
        // unknown-op errors.
        if !is_supported_version(&recipe.version) {
            return Err(RecipeError::UnsupportedVersion {
                found: recipe.version,
                supported: SUPPORTED_VERSIONS_DISPLAY,
            });
        }

        // SPEC-127, Call 1: `format`/`quality` require version "2". Gated
        // here (a domain rule), not via serde — a v1 recipe that simply omits
        // both fields keeps parsing exactly as it did before this spec; only
        // a v1 recipe that SETS one is rejected, with a message naming the
        // field and the version, not a `deny_unknown_fields` parse error.
        if recipe.version != SUPPORTED_VERSION_2 {
            if recipe.format.is_some() {
                return Err(RecipeError::NewFieldNeedsVersion2 {
                    field: "format",
                    found: recipe.version,
                });
            }
            if recipe.quality.is_some() {
                return Err(RecipeError::NewFieldNeedsVersion2 {
                    field: "quality",
                    found: recipe.version,
                });
            }
        }

        // Step count check AFTER both version checks: an over-version or
        // needs-v2 recipe reports THAT problem, not TooManySteps.
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

// ─── The terminal `optimize` marker (SPEC-085, SPEC-111, SPEC-112) ────────────

/// The reserved terminal recipe step that encodes via the fast AVIF-aware decision
/// (`Mode::Fast`: modernize format + never-bigger + score) instead of a plain
/// format-preserving sink write (SPEC-085). This is what makes `apply --recipe web`
/// == the `web` verb — the bundled flows end with it. It is NOT a registry
/// operation (it produces bytes + a format choice, not a transformed `Image`), so it
/// must be stripped before [`Recipe::build_pipeline`] ever sees it.
const OPTIMIZE_STEP_OP: &str = "optimize";

/// If `recipe` ends with the terminal [`OPTIMIZE_STEP_OP`] step, return a copy with
/// that step removed — the pixel pipeline to run before the caller's own terminal
/// decision (a fast auto-decide encode on the CLI side, a pinned-format encode on
/// the wasm side). `None` when the recipe has no terminal `optimize` step (a plain
/// pixel recipe). An `optimize` step anywhere but last is left in place, so
/// [`Recipe::build_pipeline`] surfaces it as a typed `UnknownOperation` error rather
/// than silently reordering intent.
///
/// Lives here, not in `cli`, because it is a **recipe** concern shared by every
/// caller that hands a recipe to `build_pipeline` — and because `cli` is compiled
/// only for native targets (`#[cfg(not(target_arch = "wasm32"))]`, `src/lib.rs`)
/// while `wasm` is compiled only for `wasm32` (`#[cfg(target_arch = "wasm32")]`):
/// the two module trees never coexist in one build, so a `cli`-hosted `pub(crate)`
/// helper would not even compile into the wasm32 artifact for `wasm::transform` to
/// call. `recipe` is one of the modules that compiles for BOTH targets (`src/lib.rs`
/// §"the pure engine"), so it is the only home that reaches callers on both targets.
/// `pub(crate)`: `cli::optimize::run_apply` and `cli::build::prepare_target` (native)
/// and `wasm::transform` (wasm32) each import it straight from `crate::recipe` and
/// reuse this exact function rather than each carrying its own copy — a second copy
/// is exactly the kind of drift the "anywhere but last stays an error" rule (AC-5,
/// SPEC-112) would silently diverge on.
pub(crate) fn split_terminal_optimize(recipe: &Recipe) -> Option<Recipe> {
    match recipe.steps.last() {
        Some(step) if step.op == OPTIMIZE_STEP_OP => {
            let mut pixel = recipe.clone();
            pixel.steps.pop();
            Some(pixel)
        }
        _ => None,
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
            format: None,
            quality: None,
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
    ///
    /// SPEC-127 made `"2"` a real, supported version — this test now probes a
    /// version genuinely outside the supported set (`"3"`), not `"2"` (which
    /// this file's own `from_toml_unsupported_version_still_rejected`
    /// predecessor used before this spec).
    #[test]
    fn from_toml_unsupported_version_still_rejected() {
        let toml_str = "version = \"3\"\n";
        let result = Recipe::from_toml(toml_str);
        assert!(
            matches!(result, Err(RecipeError::UnsupportedVersion { ref found, .. }) if found == "3"),
            "expected UnsupportedVersion with found=\"3\", got {result:?}"
        );
    }

    // ─── SPEC-127: version-2 gate and the new fields ─────────────────────────

    /// `version = "2"` alone (no `format`/`quality`) is now accepted — Call 1
    /// widens the supported set, it does not narrow it.
    #[test]
    fn version_2_with_no_new_fields_is_accepted() {
        let toml_str = "version = \"2\"\n\n[[step]]\nop = \"identity\"\n";
        let result = Recipe::from_toml(toml_str);
        assert!(result.is_ok(), "version 2 alone must parse, got {result:?}");
    }

    /// A `version = "1"` recipe that sets `format` is rejected with the new
    /// typed error, naming the field and the declared version — not a
    /// `deny_unknown_fields` TOML parse error (the pre-SPEC-127 asymmetry the
    /// design measured on `main`).
    #[test]
    fn v1_with_format_is_rejected() {
        let toml_str = "version = \"1\"\nformat = \"png\"\n";
        let result = Recipe::from_toml(toml_str);
        assert!(
            matches!(
                &result,
                Err(RecipeError::NewFieldNeedsVersion2 { field, found })
                    if *field == "format" && found == "1"
            ),
            "expected NewFieldNeedsVersion2 {{ field: \"format\", found: \"1\" }}, got {result:?}"
        );
    }

    /// Same gate, `quality` half.
    #[test]
    fn v1_with_quality_is_rejected() {
        let toml_str = "version = \"1\"\nquality = 80\n";
        let result = Recipe::from_toml(toml_str);
        assert!(
            matches!(
                &result,
                Err(RecipeError::NewFieldNeedsVersion2 { field, found })
                    if *field == "quality" && found == "1"
            ),
            "expected NewFieldNeedsVersion2 {{ field: \"quality\", found: \"1\" }}, got {result:?}"
        );
    }

    /// `version = "2"` with both fields round-trips and carries them.
    #[test]
    fn v2_round_trips_format_and_quality_unit() {
        let toml_str = "version = \"2\"\nformat = \"webp\"\nquality = 90\n";
        let recipe = Recipe::from_toml(toml_str).expect("v2 with format+quality should parse");
        assert_eq!(recipe.format.as_deref(), Some("webp"));
        assert_eq!(recipe.quality, Some(90));

        let serialized = recipe.to_toml().expect("to_toml should succeed");
        let reloaded = Recipe::from_toml(&serialized).expect("re-parse should succeed");
        assert_eq!(recipe, reloaded, "v2 recipe must round-trip through TOML");
    }

    /// A recipe using neither `format` nor `quality` still serializes as
    /// `version = "1"` — `to_toml` never bumps the version on its own (the
    /// highest-consequence line in the spec: doing so unconditionally would
    /// strand every existing recipe on its next `--save-recipe`).
    #[test]
    fn to_toml_does_not_bump_version_without_new_fields() {
        let r = Recipe::from_ops(&[]);
        assert_eq!(r.version, "1");
        let toml_str = r.to_toml().expect("to_toml should succeed");
        assert!(
            toml_str.contains("version = \"1\""),
            "a recipe using neither new field must still serialize as version \"1\", got:\n{toml_str}"
        );
        assert!(
            !toml_str.contains("format"),
            "format must not appear when unset, got:\n{toml_str}"
        );
        assert!(
            !toml_str.contains("quality"),
            "quality must not appear when unset, got:\n{toml_str}"
        );
    }
}
