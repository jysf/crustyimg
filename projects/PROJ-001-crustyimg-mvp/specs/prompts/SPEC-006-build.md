# SPEC-006 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. Do not rely on any prior conversation. This prompt is
> deliberately prescriptive — follow it literally.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-006 ("Recipe TOML and operation registry").
You are NOT the architect; the spec file is your source of truth. Use ABSOLUTE
paths for every file you read or write.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════

1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — conventions: §5 stack (`image` 0.25, `thiserror` 2 in lib, serde 1 + toml
   0.8 for recipes per DEC-005, NO async, NO new top-level dep without a DEC),
   §6 the EXACT commands (the four gates below), §11 coding conventions
   (library-first; typed errors; NO unwrap/expect/panic! on recoverable paths;
   DIAGNOSTICS TO STDERR NEVER STDOUT; group imports std/external/local;
   comments explain WHY not WHAT; no dead code), §12 testing (unit in
   #[cfg(test)] at the bottom of the module, integration under tests/, NATIVE
   in-memory fixtures — NO ImageMagick, NO committed binary fixtures), §13
   git/PR (branch naming, conventional commits + Co-Authored-By trailer, PR
   body template), §15 build-cycle rules (spec edits LIMITED to the
   `## Build Completion` section; append a build cost session entry; create
   DEC-* only for NON-trivial NEW decisions — there are none expected here).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-006-recipe-toml-and-operation-registry.md
   — THE SPEC. Implement its "## Failing Tests" and "## Outputs" exactly. Read
   "## Implementation Context" and "## Notes for the Implementer" in FULL —
   they are written for you, including the EXACT params-serialization
   mechanism, the registry location/API, and the validation rules.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-005-recipe-toml-and-operation-registry.md
   — THE governing decision. Recipe = versioned ordered TOML op list via serde;
   an operation registry maps name -> constructor(params); both CLI and loader
   go through it so recipes round-trip. `serde` + `toml` are PRE-JUSTIFIED by
   this DEC — adding them needs NO new DEC. Do NOT add any other crate.
   Also read:
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-002-single-image-model-and-operation-trait.md
   (the Operation trait + single-image-library this serializes)
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-007-error-handling-thiserror-anyhow.md
   (typed thiserror in lib; anyhow + exit codes ONLY at the binary boundary —
   not in this spec).
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/constraints.yaml
   — full text of the constraints that apply: untrusted-input-hardening
   (validate version + reject unknown ops, typed errors not panics),
   no-unwrap-on-recoverable-paths, no-new-top-level-deps-without-decision
   (serde+toml are pre-decided — no other crate), clippy-fmt-clean,
   every-public-fn-tested, test-before-implementation, single-image-library.
5. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/data-model.md
   — § "Recipe Schema (TOML)": the version field, [[step]] array-of-tables,
   step.op, per-op params; the worked `web.toml` example; the round-trip
   guarantee. FOLLOW THIS SCHEMA. (The web.toml ops like resize/watermark/
   clean-gps are ILLUSTRATIVE and belong to later stages — do NOT implement
   them; only identity + invert exist.)
   AND /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/architecture.md
   — § "Module / Layer Structure" (layering: `recipe -> operation (via
   Registry)`; the registry lives in `operation/`) and § Components (Recipe +
   Operation Registry responsibilities).
6. THE EXISTING CODE YOU MUST EXTEND WITHOUT BREAKING:
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/operation/mod.rs
   (the Operation trait, the `OperationParams { None }` placeholder you must
   make serde-friendly WITHOUT breaking its 3 tests, OperationError, the
   Identity + Invert impls you register)
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/pipeline/mod.rs
   (Pipeline::new()/push(Box<dyn Operation>)/run(Image)/is_empty()/len(); the
   `ops` field is PRIVATE — compare pipeline OUTPUTS, never introspect ops)
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/image/mod.rs
   (Image::from_parts / with_pixels / pixels() — for the equivalence fixture)
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/error.rs
   (the ImageError + Result thiserror style to MIRROR for RecipeError)
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/sink/mod.rs
   (the module-local typed-error pattern — mirror it for RegistryError)
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/lib.rs
   (current `pub mod` declarations — you add `pub mod recipe;`)
7. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/Cargo.toml
   — the `=`-pinned dependency style to match.

═══════════════════════════════════════════════════════════════════════════
BEFORE CODING
═══════════════════════════════════════════════════════════════════════════

Mark the build cycle `[~]` in:
  /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-006-recipe-toml-and-operation-registry-timeline.md

Sync main and branch off it:
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg switch main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg switch -c feat/spec-006-recipe-toml-and-operation-registry

If anything is genuinely ambiguous or needs architect judgment, add a line to
/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml
and STOP. Do not guess on an architectural fork.

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD — EXACT MODULE LAYOUT
═══════════════════════════════════════════════════════════════════════════

A) Cargo.toml — add (DEC-005, NO new DEC; pin exact patch versions with `=`):
     serde = { version = "=1.x.y", features = ["derive"] }
     toml  = "=0.8.z"
   Use current published patch releases. Add NO other crate. Keep the existing
   `image`/`thiserror`/`glob`/`viuer` lines untouched.

B) src/operation/mod.rs — evolve OperationParams + wire the registry sub-module:
   - Add `#[derive(Serialize, Deserialize)]` to `OperationParams` (keep its
     existing `Debug, Clone, PartialEq, Eq` derives and its `None` variant —
     the 3 SPEC-003 tests assert `== OperationParams::None`; THEY MUST STILL
     PASS).
   - `OperationParams::None` MUST serialize to ZERO extra keys when
     `#[serde(flatten)]`-ed into a step table (so a parameterless step is
     exactly `[[step]]` + `op = "invert"`). If a derive-based flatten emits a
     stray tag, hand-write a `Serialize`/`Deserialize` impl (or use a custom
     representation) so `None` <-> empty table round-trips cleanly. PROVE this
     with a unit test BEFORE building the rest (see Failing Tests).
   - Add `mod registry;` + `pub use registry::{OperationRegistry, RegistryError};`
     (so the public path is `crustyimg::operation::OperationRegistry`).
   - Do NOT change the Operation trait signature. Do NOT make ops do I/O.

C) src/operation/registry.rs — NEW. The registry (DEC-005). Public API:
     pub type Constructor =
         fn(&OperationParams) -> Result<Box<dyn Operation>, RegistryError>;
     pub struct OperationRegistry { /* name -> Constructor map */ }
     impl OperationRegistry {
         pub fn new() -> Self                       // empty
         pub fn with_builtins() -> Self             // registers "identity" + "invert"
         pub fn register(&mut self, name: &'static str, ctor: Constructor)
         pub fn contains(&self, name: &str) -> bool
         pub fn build(&self, name: &str, params: &OperationParams)
             -> Result<Box<dyn Operation>, RegistryError>   // RegistryError::Unknown on miss
     }
     impl Default for OperationRegistry { fn default() -> Self { Self::new() } }  // clippy
     #[derive(Debug, thiserror::Error)]
     pub enum RegistryError {
         #[error("unknown operation '{name}'")]
         Unknown { name: String },
     }
   The identity/invert constructors IGNORE params (parameterless) but must
   accept `&OperationParams` so the signature is uniform for later ops.
   This module must NOT depend on `recipe`, `clap`, `source`, `sink`, files,
   or terminals.

D) src/recipe/mod.rs — NEW. The recipe layer. Public API:
     pub const SUPPORTED_VERSION: &str = "1";
     #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
     pub struct Recipe {
         pub version: String,
         #[serde(skip_serializing_if = "Option::is_none", default)]
         pub name: Option<String>,
         #[serde(skip_serializing_if = "Option::is_none", default)]
         pub description: Option<String>,
         #[serde(rename = "step", default)]   // [[step]] array-of-tables
         pub steps: Vec<RecipeStep>,
     }
     #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
     pub struct RecipeStep {
         pub op: String,
         #[serde(flatten)]
         pub params: OperationParams,
     }
     #[derive(Debug, thiserror::Error)]
     pub enum RecipeError {
         #[error("unsupported recipe version '{found}' (supported: {supported})")]
         UnsupportedVersion { found: String, supported: &'static str },
         #[error("unknown operation '{name}'")]
         UnknownOperation { name: String },
         #[error("could not parse recipe TOML: {0}")]
         Parse(String),
         #[error("could not serialize recipe to TOML: {0}")]
         Serialize(String),
     }
     impl Recipe {
         pub fn from_ops(ops: &[Box<dyn Operation>]) -> Recipe  // version = SUPPORTED_VERSION
         pub fn to_toml(&self) -> Result<String, RecipeError>
         pub fn from_toml(s: &str) -> Result<Recipe, RecipeError>     // parse + version check only
         pub fn build_pipeline(&self, registry: &OperationRegistry)
             -> Result<Pipeline, RecipeError>                         // resolves ops; UnknownOperation on miss
     }
   - `from_toml`: `toml::from_str` errors -> `RecipeError::Parse(e.to_string())`;
     then if `version != SUPPORTED_VERSION` return `UnsupportedVersion`. Do NOT
     resolve ops here.
   - `to_toml`: `toml::to_string` errors -> `RecipeError::Serialize(...)`.
   - `build_pipeline`: fold steps, calling `registry.build(&step.op,
     &step.params)`; map `RegistryError::Unknown { name }` ->
     `RecipeError::UnknownOperation { name }`; push each into a `Pipeline`.
   - This module may use `crate::operation::*`, `crate::pipeline::Pipeline`,
     `crate::image::Image`. It must NOT touch clap/source/sink/terminals.

E) src/lib.rs — add `pub mod recipe;` (keep the others). Add a one-line doc
   comment for the new module matching the existing SPEC-by-SPEC style.

═══════════════════════════════════════════════════════════════════════════
TESTS — WRITE THESE EXACT NAMES (they ARE the acceptance criteria)
═══════════════════════════════════════════════════════════════════════════

tests/recipe_round_trip.rs (integration, public API):
  - recipe_round_trips_through_toml
  - serialized_toml_matches_schema           (contains `version = "1"`, two
                                               `[[step]]`, `op = "invert"`;
                                               substring checks, NOT byte-equal)
  - registry_resolves_builtins_by_name
  - unknown_operation_is_typed_error         (matches! RecipeError::UnknownOperation)
  - unsupported_version_is_typed_error       (matches! RecipeError::UnsupportedVersion)
  - malformed_toml_is_typed_error            (from_toml("not = = valid") -> Err Parse)
  - recipe_drives_pipeline_same_as_direct    (compare to_rgba8().into_raw() of
                                               recipe-built vs hand-built pipeline)

src/recipe/mod.rs  #[cfg(test)] mod tests:
  - empty_recipe_round_trips_and_builds_empty_pipeline
  - from_ops_records_names_in_order
  - default_version_is_one

src/operation/registry.rs  #[cfg(test)] mod tests:
  - with_builtins_contains_identity_and_invert
  - build_unknown_returns_typed_error        (matches! RegistryError::Unknown)
  - register_then_build_custom_op            (register a test-only op ctor, build it)

For Image fixtures in tests, mirror the existing `make_image` helper pattern
in src/operation/mod.rs / src/pipeline/mod.rs (RgbaImage::from_fn ->
Image::from_parts(DynamicImage::ImageRgba8(buf), ImageFormat::Png, None)).
NO ImageMagick, NO committed binary fixtures.

═══════════════════════════════════════════════════════════════════════════
THE THREE TRICKIEST THINGS — DO NOT TRIP ON THESE
═══════════════════════════════════════════════════════════════════════════

1. EMPTY PARAMS FLATTEN. The single most likely failure. `OperationParams::None`
   with `#[serde(flatten)]` inside `RecipeStep` MUST produce exactly
   `[[step]]\nop = "invert"\n` — no stray `type`/`kind`/`None` key. Write
   `serialized_toml_matches_schema` and
   `empty_recipe_round_trips_and_builds_empty_pipeline` FIRST and iterate on
   the OperationParams serde representation until they pass. A clean approach:
   represent `None` so it flattens to an EMPTY map (e.g. a custom Serialize that
   emits an empty map, and a Deserialize that accepts an empty/absent table).
   Keep the `None` enum variant so SPEC-003's `== OperationParams::None` tests
   still compile and pass.

2. ROUND-TRIP EQUALITY IS ON THE TYPED `Recipe`, NOT THE STRING. Assert
   `Recipe::from_toml(&r.to_toml()?)? == r` over PartialEq. Do NOT byte-compare
   the TOML string (serializers reorder/space differently). Assert the schema
   SHAPE separately with substring checks.

3. DON'T PANIC ON BAD INPUT. malformed TOML, bad version, unknown op — all must
   return typed Err. NO unwrap()/expect()/panic!() on these paths (constraint
   no-unwrap-on-recoverable-paths). In tests, `.unwrap()` on the EXPECTED-Ok
   results is fine; library code must not.

ALSO: keep ALL of SPEC-003's existing tests green —
src/operation/mod.rs (identity_name_and_params_are_stable, invert_name_is_stable,
identity_returns_pixels_unchanged, invert_complements_each_channel_preserving_alpha,
invert_is_involutive) and src/pipeline/mod.rs (all 7). Run `cargo test` and
confirm the full suite (SPEC-001..005 + 006) is green, not just your new tests.

═══════════════════════════════════════════════════════════════════════════
THE FOUR GATES (run from the repo root; all must pass before the PR)
═══════════════════════════════════════════════════════════════════════════

  cargo build
  cargo test
  cargo clippy -- -D warnings
  cargo fmt --check        # `cargo fmt` to fix, then re-check

═══════════════════════════════════════════════════════════════════════════
WHEN DONE
═══════════════════════════════════════════════════════════════════════════

1. Fill in ONLY the spec's `## Build Completion` section (branch, PR, criteria
   met, deviations, follow-ups, and the 3-question build reflection). Do NOT
   edit any other part of the spec file.
2. Append a build cost session entry to the spec front-matter `cost.sessions`
   (agent: claude-sonnet-4-6, interface: claude-code, tokens_total: null,
   estimated_usd: null, duration_minutes: <est>, recorded_at: 2026-06-14,
   notes: "subagent; cost not separately reported"). Do NOT recompute
   cost.totals (ship does that).
3. Advance the cycle to verify. NOTE: `just advance-cycle` mis-globs in this
   repo — instead HAND-EDIT the spec front-matter `task.cycle` from `build`
   to `verify`, and verify the change is correct before committing.
4. Commit with conventional commits (e.g.
   `feat(recipe): TOML recipe + operation registry round-trip (SPEC-006)`),
   ending EACH commit message with:
       Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
5. Mark build `[x]` in the timeline with the PR number + completion date.
6. Push the branch and open a PR on the `jysf/crustyimg` remote per AGENTS.md
   §13 (PR title carries the spec id; PR body uses the §13 template —
   Summary, Spec metadata PROJ-001/STAGE-001/SPEC-006, Decisions referenced
   [DEC-005 governs; DEC-002; DEC-007], Constraints checked with one-line
   evidence each, New decisions: "No new DEC — serde+toml pre-justified by
   DEC-005"). End the PR body with the Claude Code generated-with footer.

Expected new decisions: NONE. serde + toml are pre-justified by DEC-005. If you
believe a new DEC is needed, STOP and add a question instead.

Scope reminder — DO NOT build: clap/CLI (SPEC-007), edit/--save-recipe/apply
(STAGE-005), rayon/indicatif batch (STAGE-005), any real transform op (resize/
watermark/etc.), metadata-lane ops (STAGE-004), security-grade recipe fuzzing
(STAGE-006). Only identity + invert exist to register and round-trip.
```
