---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-006
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet 4.6, fresh session
  created_at: 2026-06-14

references:
  decisions:
    - DEC-005   # Recipe = TOML via serde + operation registry — THE governing decision
    - DEC-002   # the Operation trait this serializes; single-image-library
    - DEC-007   # typed thiserror errors in lib (RecipeError, registry errors)
  constraints:
    - untrusted-input-hardening          # recipes validate version + reject unknown ops; no panic
    - no-unwrap-on-recoverable-paths
    - no-new-top-level-deps-without-decision   # serde + toml are pre-justified by DEC-005
    - clippy-fmt-clean
    - every-public-fn-tested
    - test-before-implementation
  related_specs:
    - SPEC-003   # Operation trait + OperationParams (bare enum None) + Identity/Invert; Pipeline folds Vec<Box<dyn Operation>>
    - SPEC-005   # Sink — recipe drives decode->ops->sink (round-trip equivalence target)
    - SPEC-007   # clap skeleton (later) builds ops through the SAME registry; --save-recipe/apply (STAGE-005) consume Recipe

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: >
  This is the heart of the project thesis "tune once, replay across many":
  it makes an operation chain serializable to TOML and reconstructable through
  a registry, so the exact same recipe can later (STAGE-005) replay unchanged
  across one image or a whole batch.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Null numeric fields are fine (e.g. claude.ai web sessions); reports
# skip them in sums but count them in session_count. Examples of
# interface: claude-code | claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 45
      recorded_at: 2026-06-14
      notes: "subagent; cost not separately reported"
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 30
      recorded_at: 2026-06-14
      notes: "subagent; cost not separately reported"
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_input: null
      tokens_output: null
      estimated_usd: null
      duration_minutes: 25
      recorded_at: 2026-06-14
      notes: "subagent; read-only review; caught false build-line timeline text; cost not separately reported"
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_input: null
      tokens_output: null
      estimated_usd: null
      duration_minutes: 5
      recorded_at: 2026-06-14
      notes: "orchestrator; corrected timeline text on main; cost not separately reported"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 4
---

# SPEC-006: Recipe TOML and operation registry

## Context

This spec builds the **recipe layer** and the **operation registry** — the
keystone of the project thesis: *tune an edit once, replay it unchanged
across many images* (PROJ-001 `brief.md` `value.thesis`). A recipe is a
versioned, ordered list of operations serialized as TOML; the registry maps
an operation `name` (+ its params) to a constructed `Box<dyn Operation>`.
Both the recipe loader and (later, STAGE-005/SPEC-007) the CLI build
operations through the **same** registry, which is exactly what makes a
recipe round-trip: a recipe serialized from an op list deserializes back
into the identical op list.

It is the sixth spec in **STAGE-001 (foundation and pipeline core)**, after
SPEC-003 (`Operation` trait + `Pipeline`), SPEC-004 (`Source`), and SPEC-005
(`Sink`) shipped. The stage's Design Notes call this out directly: *"the
registry maps `name -> constructor(params)` so recipes parse back into
operations. New operations register here; nothing else changes."* It is
governed by **DEC-005** (Recipe = TOML via serde + operation registry),
which pre-justifies the `serde` + `toml` dependencies.

SPEC-003 deliberately shipped `OperationParams` as a dependency-free
placeholder (`enum { None }`) and documented that *"SPEC-006 will widen this
to a serde/TOML value when the recipe layer and its `toml`/`serde`
dependencies arrive."* That widening happens here — without breaking the
existing `Identity`/`Invert`/`Pipeline` tests.

## Goal

Add a `src/recipe/` module exposing a `Recipe` type (a `version` field + an
ordered list of `(op, params)` steps) that (de)serializes to/from TOML via
serde and round-trips losslessly, plus an `OperationRegistry` that constructs
the existing `identity` and `invert` operations by name; reject an
unsupported recipe `version` and an unknown operation `name` with typed
errors, never a panic.

## Inputs

- **Files to read:**
  - `src/operation/mod.rs` — the `Operation` trait, the `OperationParams`
    placeholder enum (`None`), `OperationError`, and the `Identity`/`Invert`
    impls you must keep working and register.
  - `src/pipeline/mod.rs` — how a `Pipeline` is built from
    `Box<dyn Operation>` and `run` over an `Image` (the round-trip-equivalence
    target).
  - `src/image/mod.rs` — `Image::from_parts` / `with_pixels` / `pixels()`
    (only needed for the recipe→Pipeline equivalence test fixtures).
  - `src/error.rs` and `src/lib.rs` — the `thiserror` `ImageError`/`Result`
    style to mirror, and the current `pub mod` declarations.
  - `Cargo.toml` — the `=`-pinned dependency style to match.
  - `docs/data-model.md` § "Recipe Schema (TOML)" — **the schema you must
    follow** (the `version` field, `[[step]]` array-of-tables, `step.op`,
    per-op params; the worked `web.toml` example; the round-trip guarantee).
  - `docs/architecture.md` § "Module / Layer Structure" and § Components —
    the layering rule (`recipe → operation (via Registry)`).
- **External APIs:** none. (`serde` https://docs.rs/serde, `toml`
  https://docs.rs/toml — both pre-justified by DEC-005.)
- **Related code paths:** `src/operation/`, `src/pipeline/`, `src/recipe/`
  (new).

## Outputs

- **Files created:**
  - `src/recipe/mod.rs` — the `Recipe` + `RecipeStep` types, TOML
    (de)serialization, validation (version + unknown-op), and the
    `RecipeError` typed-error enum. Plus `#[cfg(test)]` unit tests.
  - `tests/recipe_round_trip.rs` — integration tests asserting the TOML
    string round-trip, registry name resolution, unknown-op error,
    version-mismatch error, malformed-TOML error, and recipe→Pipeline
    equivalence.
- **Files modified:**
  - `src/operation/mod.rs` — (a) evolve `OperationParams` to be
    **serde-friendly** while preserving the existing `None` variant and the
    three SPEC-003 tests; (b) add the `OperationRegistry` (the registry lives
    in `src/operation/registry.rs`, re-exported from `operation` — see
    Notes); (c) add a `From<&dyn Operation> for RecipeStep`-style helper or a
    `Recipe::from_ops` builder (see Notes for the exact mechanism).
  - `src/operation/registry.rs` — **new** sub-module holding the
    `OperationRegistry` (constructor map) + `RegistryError`; declared
    `pub mod registry;` (or `mod registry;` + `pub use`) from
    `src/operation/mod.rs`.
  - `src/lib.rs` — add `pub mod recipe;`.
  - `Cargo.toml` — add `serde = { version = "=1.x.y", features = ["derive"] }`
    and `toml = "=0.8.z"` (exact pinned patch versions; DEC-005 — **no new
    DEC**).
- **New exports (signatures — see Implementation Context for the exact
  mechanism):**
  - `crustyimg::recipe::Recipe` — `{ version: String, name: Option<String>,
    description: Option<String>, steps: Vec<RecipeStep> }`, `#[derive(Debug,
    Clone, PartialEq, Serialize, Deserialize)]`.
  - `crustyimg::recipe::RecipeStep` — `{ op: String, params: OperationParams }`
    (params flattened into the step table — see mechanism).
  - `crustyimg::recipe::RecipeError` — typed `thiserror` enum:
    `UnsupportedVersion { found: String, supported: &'static str }`,
    `UnknownOperation { name: String }`, `Parse(String)` (malformed TOML),
    `Serialize(String)`.
  - `Recipe::to_toml(&self) -> Result<String, RecipeError>`.
  - `Recipe::from_toml(s: &str) -> Result<Recipe, RecipeError>` (validates
    `version` only; does NOT resolve ops).
  - `Recipe::from_ops(ops: &[Box<dyn Operation>]) -> Recipe` (build a recipe
    from a live op list — the "save" direction).
  - `Recipe::build_pipeline(&self, registry: &OperationRegistry) ->
    Result<Pipeline, RecipeError>` **or** `Recipe::to_ops(&self, registry) ->
    Result<Vec<Box<dyn Operation>>, RecipeError>` (the "load" direction;
    resolves each step's `op` through the registry, surfacing
    `UnknownOperation`).
  - `crustyimg::operation::OperationRegistry` — `new()` / `with_builtins()`
    (registers `identity` + `invert`), `register(name, constructor)`,
    `build(&self, name, &OperationParams) -> Result<Box<dyn Operation>,
    RegistryError>`, `contains(name) -> bool`.
  - `crustyimg::operation::OperationParams` — evolved to derive
    `Serialize`/`Deserialize`, keeping the `None` variant.
- **Database changes:** none.

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] A `Recipe` containing `[invert, invert]` (version `"1"`) serializes to
      a TOML string and `from_toml` of that string yields a `Recipe` **equal**
      to the original (`PartialEq`) — round-trip is lossless.
- [ ] The serialized TOML matches the documented schema: a top-level
      `version = "1"` and two `[[step]]` tables each with `op = "invert"`
      (parameterless ops emit no extra param keys).
- [ ] `OperationRegistry::with_builtins().build("identity", &params)` returns
      a `Box<dyn Operation>` whose `name()` is `"identity"`; same for
      `"invert"`.
- [ ] `build("bogus", &params)` (or resolving a recipe step `op = "bogus"`)
      returns `RecipeError::UnknownOperation` / `RegistryError::Unknown` — a
      **typed error**, not a panic, and not silently skipped.
- [ ] `Recipe::from_toml` of a recipe with `version = "999"` returns
      `RecipeError::UnsupportedVersion` (only `"1"` is supported).
- [ ] A recipe of `[invert, invert]` built into a `Pipeline` via the registry
      and `run` over a test `Image` produces the **same** pixels as building
      `Pipeline::new().push(Box::new(Invert)).push(Box::new(Invert))` directly.
- [ ] Malformed TOML (e.g. `"this is not = = toml"`) passed to `from_toml`
      returns `RecipeError::Parse`, not a panic.
- [ ] A recipe with **no** `[[step]]` entries (empty op list) round-trips and
      builds an empty `Pipeline` (which is a valid no-op — matches
      `Pipeline`'s `empty_pipeline_returns_image_unchanged`).
- [ ] `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check`
      pass; the three SPEC-003 `OperationParams::None` tests still pass.

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass. Test names are normative — keep them.

- **`tests/recipe_round_trip.rs`** (integration; public crate API)
  - `recipe_round_trips_through_toml` — build `Recipe { version: "1",
    steps: [invert, invert] }` (via `Recipe::from_ops(&[Box::new(Invert),
    Box::new(Invert)])`), call `to_toml`, then `from_toml`, assert the
    result `== ` the original `Recipe`.
  - `serialized_toml_matches_schema` — assert the `to_toml` string contains
    `version = "1"` and exactly two `[[step]]` tables with `op = "invert"`.
  - `registry_resolves_builtins_by_name` — `OperationRegistry::with_builtins()`,
    assert `build("identity", &OperationParams::None)?.name() == "identity"`
    and same for `"invert"`.
  - `unknown_operation_is_typed_error` — a recipe TOML with
    `[[step]] op = "bogus"` resolved through the registry returns
    `RecipeError::UnknownOperation { name }` with `name == "bogus"`; assert
    via `matches!`, assert it is `Err` (not a panic, not a skipped step).
  - `unsupported_version_is_typed_error` — `Recipe::from_toml` of a string
    with `version = "999"` returns `RecipeError::UnsupportedVersion`; assert
    via `matches!`.
  - `malformed_toml_is_typed_error` — `Recipe::from_toml("not = = valid")`
    returns `RecipeError::Parse`; assert it is `Err`, not a panic.
  - `recipe_drives_pipeline_same_as_direct` — build a small RGBA `Image`;
    run it through (a) a `Pipeline` built from the recipe `[invert, invert]`
    via the registry, and (b) a hand-built
    `Pipeline::new().push(Box::new(Invert)).push(Box::new(Invert))`; assert
    the two output `to_rgba8().into_raw()` buffers are equal.
- **`src/recipe/mod.rs`** (`#[cfg(test)] mod tests`)
  - `empty_recipe_round_trips_and_builds_empty_pipeline` — a `Recipe` with
    `version "1"` and zero steps round-trips through TOML and
    `build_pipeline` yields a `Pipeline` with `is_empty() == true`.
  - `from_ops_records_names_in_order` — `Recipe::from_ops(&[Box::new(Identity),
    Box::new(Invert)])` has `steps[0].op == "identity"`, `steps[1].op ==
    "invert"`.
  - `default_version_is_one` — assert the supported version constant is
    `"1"` and a freshly built recipe carries it.
- **`src/operation/registry.rs`** (`#[cfg(test)] mod tests`)
  - `with_builtins_contains_identity_and_invert` — `contains("identity")` and
    `contains("invert")` are `true`; `contains("bogus")` is `false`.
  - `build_unknown_returns_typed_error` — `build("bogus", &OperationParams::None)`
    returns `RegistryError::Unknown { name }` via `matches!`, no panic.
  - `register_then_build_custom_op` — register a tiny test-only op constructor
    and assert `build` constructs it (proves the registry is open for the
    later stages' ops without editing the recipe parser — DEC-005).

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### The params-serialization mechanism (READ THIS FIRST — the trickiest bit)

SPEC-003 shipped `OperationParams` as a bare `enum { None }` with **no
serde**, and three tests assert `Identity.params() == OperationParams::None`
and `Invert`'s name. You must **not** break those. Use this exact mechanism:

1. **Keep the `None` variant.** Add `#[derive(Serialize, Deserialize)]` to
   `OperationParams` (it already derives `Debug, Clone, PartialEq, Eq` — keep
   them). The `None` variant stays so SPEC-003's three tests pass verbatim.

2. **The registry owns per-op construction, not the trait.** The
   `Operation` trait is unchanged (`name`, `params`, `apply`). A registry
   entry is a constructor `fn(&OperationParams) -> Result<Box<dyn Operation>,
   RegistryError>`. For `identity`/`invert` the constructor ignores the
   params (they are parameterless) and returns `Box::new(Identity)` /
   `Box::new(Invert)`. Forward-compat: an op WITH params (resize, later) will
   deserialize its own typed params struct from the params and construct
   itself — no change to the recipe parser, which is the whole point of
   DEC-005.

3. **`RecipeStep` is `{ op: String, #[serde(flatten)] params: OperationParams
   }`** — the `op` key plus the operation's params flattened into the same
   `[[step]]` table, matching the schema (`op = "invert"`, then any
   `step.<param>` keys in the same table). For parameterless ops,
   `OperationParams::None` must serialize to **zero extra keys** in the step
   table (so `[[step]]\nop = "invert"` is the exact output). **Critical
   serde detail:** a `#[serde(flatten)]`-ed unit-like enum variant can emit a
   stray tag key or fail to flatten cleanly. To guarantee the empty-table
   output and clean round-trip, model `OperationParams` so that `None`
   flattens to nothing — the recommended representation is a newtype over an
   inner serde value that is empty for `None`. Two acceptable shapes (pick
   one, document it in a code comment):
     - **(a)** `OperationParams::None` + a custom `Serialize`/`Deserialize`
       impl (or `#[serde(untagged)]`) such that `None` ⇄ an empty table; or
     - **(b)** represent params as `OperationParams(toml::Table)` internally
       with a `None` *constructor* returning the empty table — but this
       changes the public enum shape and would break the SPEC-003 `==
       OperationParams::None` tests, so **(a) is preferred**: keep the enum,
       give it a serde impl that round-trips `None` ⇄ empty.
   **Verify the empty-table round-trip with a unit test before wiring the
   rest** — this is the single most likely place to fail.

4. **Round-trip direction (save):** `Recipe::from_ops(ops)` walks the live
   `&[Box<dyn Operation>]`, reading `op.name()` and `op.params()` into
   `RecipeStep`s, with the recipe `version` set to the supported constant
   (`"1"`). **Load direction:** `Recipe::build_pipeline(&self, registry)`
   walks `self.steps`, calling `registry.build(&step.op, &step.params)?` for
   each, pushing into a `Pipeline`. `from_toml` does TOML→`Recipe` + version
   validation; op resolution happens at `build_pipeline` time so a malformed
   `op` name surfaces as `UnknownOperation` there.

### Registry location + API

Per `docs/architecture.md`, the registry lives in **`src/operation/`** (the
`operation/` module is "Operation trait, OperationParams, the operation
Registry"). Put it in **`src/operation/registry.rs`** and re-export
`OperationRegistry` + `RegistryError` from `src/operation/mod.rs` (so
`crustyimg::operation::OperationRegistry` is the public path). `recipe`
depends on `operation` (the documented layering `recipe → operation`); the
registry must **not** depend on `recipe`, `clap`, `source`, `sink`, files, or
terminals.

```text
OperationRegistry::new() -> Self                         // empty
OperationRegistry::with_builtins() -> Self               // registers identity + invert
  .register(name: &'static str, ctor: Constructor)       // open for later-stage ops
  .contains(name: &str) -> bool
  .build(name: &str, params: &OperationParams)
        -> Result<Box<dyn Operation>, RegistryError>      // Unknown(name) on miss
```

where `type Constructor = fn(&OperationParams) -> Result<Box<dyn Operation>,
RegistryError>;` (a plain `fn` pointer is enough for the builtins; a
`Box<dyn Fn>` is acceptable if a closure is needed). `RegistryError::Unknown
{ name: String }` is the typed miss. `RecipeError::UnknownOperation` wraps /
maps from it at the recipe layer (or `build_pipeline` constructs
`RecipeError::UnknownOperation` directly).

### Validation rules (untrusted-input-hardening, basic tier)

- **Version:** only `"1"` is supported. Define a
  `const SUPPORTED_VERSION: &str = "1";`. `from_toml` compares the parsed
  `version` against it and returns `RecipeError::UnsupportedVersion { found,
  supported }` on mismatch — **before** attempting op resolution. Reject, do
  not guess.
- **Unknown operation:** an `op` name not in the registry returns
  `RecipeError::UnknownOperation { name }` — never silently skip the step,
  never substitute identity, never panic.
- **Malformed TOML:** `toml::from_str` errors map to `RecipeError::Parse`
  (carry the message string). No `unwrap`/`expect`/`panic!` anywhere on these
  recoverable paths.
- Deeper security-grade validation/fuzzing (size bounds, adversarial recipe
  corpora) is **STAGE-006** — out of scope here.

### Decisions that apply

- `DEC-005` — **governs this spec.** Recipe = versioned ordered TOML op list
  via `serde`; an operation registry maps `name -> constructor(params)`;
  both CLI and loader go through it so recipes round-trip. **`serde` + `toml`
  are pre-justified by DEC-005 — adding them needs NO new DEC.** Its
  `affected_scope` already lists `src/recipe/**` and `src/operation/**`.
- `DEC-002` — the `Operation` trait + single-image-library this serializes;
  the registry constructs `Box<dyn Operation>`; ops stay pure in-memory.
- `DEC-007` — typed `thiserror` enums in the library (`RecipeError`,
  `RegistryError`); friendly `anyhow` + exit codes happen only at the binary
  boundary (a later spec), not here.

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `untrusted-input-hardening` — recipes validate `version` and reject unknown
  operations; surface failures as **typed errors, never panics** (basic tier;
  full assessment is STAGE-006).
- `no-unwrap-on-recoverable-paths` — no `unwrap`/`expect`/`panic!` on TOML
  parse, version check, or registry miss.
- `no-new-top-level-deps-without-decision` — `serde` + `toml` ARE pre-decided
  (DEC-005); **no other crate** may be added. If the build wants any other
  crate, STOP and emit a DEC / add a question.
- `clippy-fmt-clean` — `cargo clippy -- -D warnings` + `cargo fmt --check`
  clean; no dead code.
- `every-public-fn-tested` — every new public fn gets a test.
- `test-before-implementation` — the `## Failing Tests` above are written
  first; make them pass.
- `single-image-library` (advisory here) — do not add a second image lib;
  the recipe layer touches no pixels except in the equivalence test fixture
  (via the existing `Image`/`Invert`).

### Prior related work

- `SPEC-003` (shipped, PR #3) — `Operation` trait, `OperationParams::None`
  placeholder (explicitly flagged for widening here), `OperationError`,
  `Identity`/`Invert`, and the `Pipeline` fold over
  `Vec<Box<dyn Operation>>`. **Keep its three `OperationParams::None`
  tests + the Pipeline tests green.**
- `SPEC-002` (shipped, PR #2) — the canonical `Image` (`from_parts`,
  `with_pixels`, `pixels()`), used only by the equivalence-test fixture.
- `SPEC-005` (shipped, PR #5) — `Sink`; the module-local `SourceError`/
  `SinkError` typed-error pattern to **mirror** for `RecipeError`/
  `RegistryError`.

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- The `edit` / `--save-recipe` / `apply` CLI commands — **STAGE-005**.
- The clap subcommand skeleton + global args — **SPEC-007** (it will build
  ops through this same registry; do not add clap here).
- Parallel batch execution (`rayon`) + progress (`indicatif`) — **STAGE-005**.
- Any **real** transform operation (resize, sharpen, watermark, …) — later
  stages. Only `identity` + `invert` exist to register and round-trip.
- Metadata-lane recipe steps (e.g. `clean-gps` in the worked example) — the
  metadata lane is **STAGE-004**; do not implement those ops. (The schema
  permits them; the registry simply doesn't know them yet, which is the
  unknown-op path.)
- Security-grade recipe fuzzing / size-bound validation depth — **STAGE-006**.

## Notes for the Implementer

- **Match the existing typed-error pattern.** `RecipeError` and
  `RegistryError` should mirror `ImageError` (`src/error.rs`) / the
  module-local `SinkError` style: `#[derive(Debug, Error)]`, `#[error("…")]`
  messages that name the *failure* (path-agnostic), `#[from]` where it reads
  cleanly. Keep `RegistryError` in `registry.rs` and `RecipeError` in
  `recipe/mod.rs`.
- **The empty-params flatten is the trap.** Write
  `empty_recipe_round_trips_and_builds_empty_pipeline` and the
  schema-shape assertion FIRST and iterate on the `OperationParams` serde
  representation until `[[step]]\nop = "invert"` is the literal output with
  no stray keys. `toml` is picky about where tables/values may appear; a
  `#[serde(flatten)]` of an empty map is the clean path.
- **Round-trip equality is on the typed `Recipe`, not the TOML string.**
  TOML serializers may reorder keys or whitespace; assert
  `from_toml(to_toml(r)?)? == r` over `PartialEq`, and assert the *schema
  shape* (contains `version = "1"`, two `[[step]]`, `op = "invert"`)
  separately with substring checks — don't byte-compare the whole string.
- **Registry constructors for the builtins ignore params** but must still
  accept `&OperationParams` so the signature is uniform for later ops.
- **`Pipeline` has no public accessor for its ops** (the `ops` field is
  private). For `recipe_drives_pipeline_same_as_direct`, compare *outputs*
  (run both pipelines over the same `Image`, compare `to_rgba8().into_raw()`)
  — do not try to introspect the op list.
- **Pin exact patch versions** for `serde` and `toml` with `=` to match
  `Cargo.toml`'s existing style. Pick current `serde` 1.x and `toml` 0.8.x
  patch releases; `serde` needs `features = ["derive"]`.
- **Layering:** `src/recipe/mod.rs` may use `crate::operation::*` and
  `crate::pipeline::Pipeline` and `crate::image::Image`; it must NOT touch
  `clap`, `source`, `sink`, or terminals. The registry in
  `src/operation/registry.rs` must NOT depend on `recipe`.
- **Run the four gates locally before opening the PR** (see §6):
  `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`,
  `cargo build`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-006-recipe-toml-and-operation-registry`
- **PR (if applicable):** opened after commit (see timeline)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - No new DEC — `serde` + `toml` pre-justified by DEC-005.
- **Deviations from spec:**
  - None. All test names, signatures, and module layout implemented exactly as specified.
- **Follow-up work identified:**
  - None beyond what is already in the backlog (SPEC-007 clap skeleton, STAGE-004 metadata lane, STAGE-005 CLI recipe apply).

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing was genuinely unclear. The spec's "trickiest things" section correctly identified the empty-params flatten as the main hazard and provided two concrete implementation options. The `Debug` trait not being on `Box<dyn Operation>` / `Pipeline` caused a minor compile error in the `{:?}` format arg in one test assertion, but that was trivial to fix.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No missing constraints. The `no-unwrap-on-recoverable-paths` and `untrusted-input-hardening` constraints were both directly applied as typed errors on all three error paths (malformed TOML, bad version, unknown op). The spec was well-calibrated.

3. **If you did this task again, what would you do differently?**
   — Write a quick smoke test for the `OperationParams` serde impl in isolation (just serialize/deserialize to JSON or a TOML value directly) before wiring the whole `RecipeStep` flatten, to fail faster on the empty-map contract. The spec recommends this and it's good advice.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — The Sonnet build agent wrote a false timeline marker ("PR #6 merged… 85 tests")
   at build time. Tighten build prompts: the build mark must say "PR #N opened" (never
   "merged") and carry no claims that only become true after merge. Verify caught it —
   the read-only-verify + adversarial review loop did its job.

2. **Does any template, constraint, or decision need updating?**
   — No DEC/constraint change. Process note already applied: SPEC-007's build prompt will
   specify accurate build-mark wording. The serde-friendly `OperationParams` (hand-written
   empty-map impl) is the seam future parameterized ops (resize, etc.) plug into — it held
   without breaking SPEC-003.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec. SPEC-007 (clap CLI) is next and will wire Source→Pipeline(recipe)→Sink
   into real subcommands, completing STAGE-001. Deeper recipe fuzzing/validation is already
   owned by STAGE-006.
