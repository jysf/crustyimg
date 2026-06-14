---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-003
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet 4.6 (separate session)
  created_at: 2026-06-13

references:
  decisions:
    - DEC-002                       # single canonical Image + Operation trait + decode-once Pipeline
    - DEC-007                       # thiserror in lib; typed errors; no panic on recoverable paths
  constraints:
    - decode-once-no-per-op-disk
    - no-unwrap-on-recoverable-paths
    - single-image-library
    - clippy-fmt-clean
    - every-public-fn-tested
    - test-before-implementation
  related_specs:
    - SPEC-002                      # the Image type + pixels() accessor + ImageError this builds on
    - SPEC-004                      # next: Source feeds inputs into the pipeline (out of scope here)
    - SPEC-006                      # recipe + registry construct ops from name+params (out of scope here)

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-XXX's <capability>". Optional; null is acceptable.
value_link: "infrastructure: the pipeline engine + Operation extension point every transform/recipe plugs into"

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
      tokens_input: null
      tokens_output: null
      estimated_usd: null
      duration_minutes: 40
      recorded_at: 2026-06-13
      notes: "subagent; cost not separately reported"
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_input: null
      tokens_output: null
      estimated_usd: null
      duration_minutes: 25
      recorded_at: 2026-06-14
      notes: "subagent; cost not separately reported"
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_input: null
      tokens_output: null
      estimated_usd: null
      duration_minutes: 15
      recorded_at: 2026-06-14
      notes: "subagent; read-only review; cost not separately reported"
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_input: null
      tokens_output: null
      estimated_usd: null
      duration_minutes: 5
      recorded_at: 2026-06-14
      notes: "orchestrator; cost not separately reported"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 4
---

# SPEC-003: Operation trait and Pipeline

## Context

This is backlog item #3 of `STAGE-001` (foundation and pipeline core) in
`PROJ-001` (crustyimg MVP). SPEC-002 shipped the canonical `Image` type
(wrapping `image::DynamicImage`, with `pixels()`, `width()`, `height()`,
`info()`, and a `MetadataBundle` captured at load) and the first typed error,
`ImageError`. This spec builds the **pixel-lane engine** directly on top of
it.

The earlier prototype re-read each file from disk for every operation and
mixed two image libraries. The whole rebuild rests on **not** repeating that:
DEC-002 fixes one canonical `Image` and a small `Operation` trait that a
`Pipeline` folds over a *single* decoded image in memory — decode-once,
no per-operation disk round-trips (constraint `decode-once-no-per-op-disk`).

This spec is the structural keystone of the project's extension model. Every
later stage (resize, thumbnail, watermark, …) adds a transform purely as a
new `Operation` impl; nothing in `pipeline/` changes. The trait must be small
(STAGE-001 Design Notes) and must NOT know about files, recipes, terminals,
or the metadata lane (metadata is a separate container lane — DEC-003 — and
is deliberately *not* routed through `Operation`).

## Goal

Implement the `Operation` trait (small, serde-friendly, pure, in-memory) and a
`Pipeline` that owns one decoded `Image` and folds an ordered list of
`Operation`s over it (decode-once), returning the final `Image` or halting on
the first typed error. Prove the architecture with two trivial, dependency-free
concrete operations — `Identity` (returns the image unchanged) and `Invert`
(hand-rolled per-channel value inversion) — so the tests are real.

## Inputs

- **Files to read:**
  - `src/image/mod.rs` — the `Image` type. Use `Image::pixels() -> &DynamicImage`
    to read pixels; construct a new `Image` from transformed pixels for the
    return value (see Notes for the Implementer for the constructor question).
  - `src/error.rs` — the existing `ImageError` + `pub type Result<T>`.
  - `src/lib.rs` — module declarations (`pub mod error; pub mod image;`); you
    add `pub mod operation;` and `pub mod pipeline;`.
  - `docs/data-model.md` § "Operation" / "Pipeline" — the conceptual signatures.
  - `docs/architecture.md` § "Module / Layer Structure" — the layering rule
    (`pipeline → operation → image`; `operation → image`; the pixel core must
    not depend on clap/files/terminals).
- **External APIs:** none. No network, no new crate. Pure `std` + the existing
  `image` crate (already a dependency from SPEC-002).
- **Related code paths:** `src/image/`, `src/error.rs`, `src/lib.rs`.

## Outputs

- **Files created:**
  - `src/operation/mod.rs` — the `Operation` trait, an `OperationError`
    (thiserror), the `OperationParams` type alias, and the two concrete
    operations `Identity` and `Invert` (each its own struct implementing the
    trait), plus a `#[cfg(test)] mod tests`.
  - `src/pipeline/mod.rs` — the `Pipeline` struct + executor, plus a
    `#[cfg(test)] mod tests`.
  - `tests/pipeline.rs` — integration tests exercising the public pipeline API.
- **Files modified:**
  - `src/lib.rs` — add `pub mod operation;` and `pub mod pipeline;`.
- **New exports (exact signatures the build must produce):**

  ```rust
  // src/operation/mod.rs

  /// Serde-friendly carrier for an operation's parameters, so a recipe
  /// (SPEC-006) can record + replay them. Recipe-agnostic here: a generic
  /// TOML value keeps `operation/` from depending on `recipe/`.
  pub type OperationParams = toml::Value;   // SEE NOTE — may instead be a thin
                                            // local enum if `toml` is not yet a
                                            // dep; read Notes before choosing.

  /// Errors an Operation can raise while transforming an Image (DEC-007).
  #[derive(Debug, thiserror::Error)]
  pub enum OperationError {
      /// The operation could not be applied to this image (e.g. an
      /// unsupported color type or invalid parameter).
      #[error("operation '{op}' failed: {reason}")]
      Apply { op: &'static str, reason: String },
  }

  /// The single pixel-transform extension point (DEC-002).
  ///
  /// Small on purpose: a stable name (the recipe/registry key), serde-friendly
  /// params (so a recipe can record + replay), and a pure in-memory transform.
  /// Implementors MUST NOT read or write disk (constraint
  /// `decode-once-no-per-op-disk`) and MUST NOT touch clap/recipes/terminals.
  pub trait Operation {
      /// Stable registry/recipe key, e.g. "identity", "invert".
      fn name(&self) -> &'static str;

      /// This operation's parameters, serde-serializable for recipes.
      /// Parameterless ops return an empty params value.
      fn params(&self) -> OperationParams;

      /// Transform the image in memory. Pure: no disk I/O.
      fn apply(&self, img: Image) -> Result<Image, OperationError>;
  }

  /// No-op transform: returns the image unchanged. Proves the trait + fold.
  pub struct Identity;

  /// Per-channel value inversion (255 - v on 8-bit RGB(A); alpha preserved).
  /// Hand-rolled pixel loop — NO imageproc (that crate arrives STAGE-003).
  pub struct Invert;
  ```

  ```rust
  // src/pipeline/mod.rs

  /// Decode-once executor: owns an ordered list of operations and folds them
  /// over a single in-memory Image (DEC-002, constraint
  /// `decode-once-no-per-op-disk`).
  pub struct Pipeline {
      ops: Vec<Box<dyn Operation>>,
  }

  impl Pipeline {
      /// An empty pipeline (no operations).
      pub fn new() -> Self;

      /// Append an operation; returns self for chaining (builder style).
      pub fn push(self, op: Box<dyn Operation>) -> Self;

      /// Number of operations queued.
      pub fn len(&self) -> usize;

      /// Whether the pipeline has no operations.
      pub fn is_empty(&self) -> bool;

      /// Fold every operation over `img` in order, in memory. Returns the
      /// final Image, or the first OperationError (halting; later ops do NOT
      /// run). An empty pipeline returns `img` unchanged.
      pub fn run(&self, img: Image) -> Result<Image, OperationError>;
  }

  impl Default for Pipeline { /* = Pipeline::new() */ }
  ```

- **Database changes:** none.

## Acceptance Criteria

Testable outcomes. Each maps to at least one failing test below.

- [ ] **Empty pipeline is identity.** `Pipeline::new().run(img)` returns an
  image whose pixels equal the input's pixels (no ops, no change).
- [ ] **`[Identity]` returns equal pixels.** A pipeline with one `Identity`
  returns pixels equal to the input.
- [ ] **`Invert` is correct and involutive.** A pipeline of `[Invert, Invert]`
  round-trips to pixels equal to the original; a single `Invert` produces the
  per-channel complement (`255 - v`) on an 8-bit image with at least one
  non-trivial pixel value, alpha unchanged.
- [ ] **Application order is observable.** With a test-only order-recording
  operation, a pipeline of `[A, B]` records `A` then `B` (not `B, A`); order
  is preserved by the fold.
- [ ] **A failing operation propagates a typed error and halts.** With a
  test-only operation that always returns `OperationError::Apply`, a pipeline
  of `[Identity, Failing, <marker>]` returns `Err(OperationError::Apply { .. })`
  and the `<marker>` operation after the failing one never runs.
- [ ] **`Operation` metadata is stable.** `Identity.name() == "identity"`,
  `Invert.name() == "invert"`; `params()` returns an empty/parameterless value
  for both (round-trippable shape for SPEC-006).
- [ ] **No disk I/O during apply** (structural / by-review): neither
  `src/operation/**` nor `src/pipeline/**` references `std::fs`,
  `std::io` file/path I/O, `std::path`, or any read/write of disk; ops operate
  only on the in-memory `Image`. Asserted by a source-level grep in the
  integration test (see Failing Tests) and confirmed at review.
- [ ] **Pipeline `len`/`is_empty`/`push` behave.** `new().is_empty()` is true;
  after two `push`es, `len() == 2` and `is_empty()` is false.
- [ ] **Gates green:** `cargo build`, `cargo test`, `cargo clippy -- -D warnings`,
  `cargo fmt --check` all pass.

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in **build** is
to make these pass. Use native in-memory fixtures (no committed binary files,
no ImageMagick). Build a `DynamicImage` directly (e.g. `RgbaImage::from_fn`)
and wrap it via the `Image` constructor (see Notes for the Implementer for the
exact constructor to add/use). `.unwrap()` is fine inside `#[cfg(test)]` and
`tests/`.

- **`src/operation/mod.rs` → `#[cfg(test)] mod tests`** (unit, the ops):
  - `"identity_name_and_params_are_stable"` — asserts `Identity.name() ==
    "identity"` and `Identity.params()` is the empty/parameterless value.
  - `"invert_name_is_stable"` — asserts `Invert.name() == "invert"`.
  - `"identity_returns_pixels_unchanged"` — build a small RGBA `Image`, apply
    `Identity`, assert the returned `pixels()` equal the input's (compare the
    decoded buffers, e.g. `to_rgba8().into_raw()`).
  - `"invert_complements_each_channel_preserving_alpha"` — build a 2×2 RGBA
    `Image` with a known non-uniform pixel (e.g. `[10, 20, 30, 128]`), apply
    `Invert`, assert each RGB channel became `255 - v` (`[245, 235, 225, 128]`)
    and alpha is unchanged.
  - `"invert_is_involutive"` — apply `Invert` twice, assert pixels equal the
    original buffer.

- **`src/pipeline/mod.rs` → `#[cfg(test)] mod tests`** (unit, the executor):
  - `"new_pipeline_is_empty"` — `Pipeline::new().is_empty()` is true and
    `len() == 0`.
  - `"push_increments_len"` — after pushing two ops, `len() == 2` and
    `is_empty()` is false.
  - `"empty_pipeline_returns_image_unchanged"` — `Pipeline::new().run(img)`
    yields pixels equal to the input.
  - `"single_identity_returns_equal_pixels"` — pipeline `[Identity]` yields
    pixels equal to the input.
  - `"double_invert_round_trips"` — pipeline `[Invert, Invert]` yields pixels
    equal to the original.
  - `"order_is_preserved"` — using a test-only `RecordOrder` op (pushes its
    label into a shared `Rc<RefCell<Vec<&'static str>>>` / `Arc<Mutex<…>>` on
    `apply`), a pipeline `[A, B]` records `["A", "B"]`.
  - `"failing_op_halts_and_propagates"` — using a test-only `AlwaysFails` op
    (returns `OperationError::Apply`), a pipeline
    `[Identity, AlwaysFails, RecordOrder("after")]` returns
    `Err(OperationError::Apply { .. })` (match the variant, not the string) and
    the recorder shows `"after"` was never invoked.

- **`tests/pipeline.rs`** (integration, public API + structural guard):
  - `"public_pipeline_inverts_via_crate_api"` — through the crate's public
    exports only (`crustyimg::pipeline::Pipeline`,
    `crustyimg::operation::{Identity, Invert}`, `crustyimg::image::Image`),
    build an `Image`, run a pipeline `[Invert]`, and assert the channels are
    complemented; run `[Invert, Invert]` and assert round-trip equality.
  - `"empty_pipeline_is_identity_via_crate_api"` — `Pipeline::new().run(img)`
    equals the input pixels through the public API.
  - `"operation_and_pipeline_sources_do_no_disk_io"` — read
    `src/operation/mod.rs` and `src/pipeline/mod.rs` as text (via
    `std::fs::read_to_string` of `CARGO_MANIFEST_DIR`-relative paths — this is
    a *test* reading source, which is allowed; the guard is about the library
    code, not the test) and assert neither file's **non-test** code references
    `std::fs`, `std::io::`, `File`, `OpenOptions`, `read_to_string`,
    `std::path`, or `Path` (a coarse but real decode-once guard;
    `#[cfg(test)]` regions may be excluded by splitting on the `mod tests`
    marker). If this heuristic proves brittle, the build agent may instead
    assert the absence of the substrings in the whole file *except* lines
    inside a `#[cfg(test)]` block — document the choice in Build Completion.

## Implementation Context

*Read this section (and the files it points to) before starting the build
cycle. It is the equivalent of a handoff document, folded into the spec since
there is no separate receiving agent. This build runs on **Sonnet 4.6** — the
section is deliberately prescriptive.*

### Decisions that apply

- `DEC-002` — Single canonical `Image` + small `Operation` trait
  (`name`, `params`, `apply(Image) -> Result<Image>`) + a `Pipeline` that
  decodes once and folds ops in memory. This spec *is* the trait + pipeline
  half of DEC-002 (SPEC-002 shipped the `Image` half). Keep the trait small;
  it ossifies if widened. `image`/`thiserror` are already justified by
  DEC-002/DEC-007 — **no new DEC for them**.
- `DEC-007` — Library returns typed `thiserror` enums; no
  `unwrap`/`expect`/`panic!` on recoverable paths. The new `OperationError`
  follows the existing `ImageError` pattern in `src/error.rs`. The library
  does not depend on `anyhow`.

### Constraints that apply

(See `/guidance/constraints.yaml` for full text.)

- `decode-once-no-per-op-disk` (**blocking**) — The pipeline decodes once and
  applies all ops in memory. Operations must **not** read or write disk; no
  per-op disk round-trips. Concretely: `src/operation/**` and
  `src/pipeline/**` library code touches no `std::fs`, file `std::io`, or
  `std::path`. Enforced by the structural test above and by review.
- `no-unwrap-on-recoverable-paths` (**blocking**) — Typed errors in library
  code; `.unwrap()`/`.expect()` only inside `#[cfg(test)]` and `tests/`.
  `Pipeline::run` returns `Result<Image, OperationError>`; it must not panic on
  an op error — it returns the `Err`.
- `single-image-library` (**blocking**) — Only the `image` crate; no second
  pixel library. `Invert` is a hand-rolled loop over `image` pixel buffers —
  do NOT add `imageproc` or `photon-rs`.
- `clippy-fmt-clean` (**blocking**) — `cargo clippy -- -D warnings` and
  `cargo fmt --check` pass; no dead code (delete, don't comment out). Note:
  clippy wants `is_empty()` whenever you define `len()` — both are in the
  contract, so this is satisfied. Provide `impl Default for Pipeline` (clippy
  `new_without_default`).
- `every-public-fn-tested` (**warning**) — Each new public fn/op has a test;
  the Failing Tests cover them.
- `test-before-implementation` (**blocking**) — The Failing Tests above are
  the contract; make them pass. Do not delete or weaken them to go green.

### What SPEC-002 already provides (your foundation)

- `crustyimg::image::Image` — wraps `image::DynamicImage`. Public accessors:
  `pixels() -> &DynamicImage`, `width()`, `height()`, `source_format()`,
  `metadata()`, `info()`. It derives `Clone` and `Debug`.
- **There is currently NO public constructor that takes a `DynamicImage`
  directly** — SPEC-002 only exposes `load`/`from_bytes`/`from_reader`.
  `apply` needs to *return a new `Image`* built from transformed pixels, and
  the tests need to *build an `Image` from an in-memory `DynamicImage`*.
  See "Notes for the Implementer → constructor" for the sanctioned way to
  handle this — it is the one real design choice in this spec.
- `crustyimg::error::{ImageError, Result}` — the existing typed-error pattern
  to mirror for `OperationError`.
- The `image` crate is already a dependency (pure-Rust feature set, DEC-004).
  `toml` is **not** yet a dependency — see the `OperationParams` note.

### Out of scope (for this spec specifically)

If any of these feels necessary during build, STOP and raise it (questions.yaml
or `[?]` in the timeline) rather than expanding this spec:

- **Source / Sink** (SPEC-004 / SPEC-005) — no file resolution, globbing,
  stdin, stdout, file writing, or viuer display. The pipeline takes an
  in-memory `Image` and returns one.
- **Recipe / registry / TOML round-trip** (SPEC-006) — no `Recipe` struct, no
  registry, no `name + params -> Operation` construction, no TOML files. Keep
  the `Operation` trait *shaped* so a registry can build ops later (stable
  `name()`, serde-friendly `params()`), but do not build the registry.
- **clap / CLI** (SPEC-007) — `operation/` and `pipeline/` must not touch clap.
- **Any real image transform** — resize, crop, rotate, flip, thumbnail,
  filters, watermark, sharpen, grayscale, etc. are STAGE-003/004. `Identity`
  and `Invert` are the *only* concrete ops here, and they exist purely to make
  the trait + fold testable. `Invert` is hand-rolled — **no `imageproc`**.
- **Metadata operations** — the container/metadata lane (DEC-003) is a
  separate path and must NOT be expressed as an `Operation`. Do not route
  metadata through the trait. (Note: it is acceptable for `Invert`/`Identity`
  to simply carry the input's `MetadataBundle` through unchanged — see Notes.)

### Exact commands (AGENTS.md §6)

```bash
cargo build                  # debug build
cargo test                   # all tests (unit + integration)
cargo clippy -- -D warnings  # lint; warnings are errors
cargo fmt --check            # formatting gate (use `cargo fmt` to fix)
```

All four must pass before opening the PR (the four gates).

### Dependency note (read before reaching for a crate)

**No new dependency is needed.** This spec is pure `std` + the existing
`image` crate. If you believe you need a new crate, **STOP** — add a note to
`/guidance/questions.yaml` and mark the timeline `[?]` rather than adding it
(constraint `no-new-top-level-deps-without-decision` requires a DEC first).

The one nuance is `OperationParams` (see Notes): the data-model envisions it as
a serde value, but `toml`/`serde` are **not yet dependencies** (they arrive
with SPEC-006). Do NOT add them here just to type `params()`. Use the
zero-dependency option in Notes instead.

## Notes for the Implementer

- **`OperationParams` — keep it dependency-free.** The data-model (§Operation)
  describes params as "a serde-friendly value (e.g. a `toml::Value`)". But
  `toml`/`serde` are NOT dependencies yet (they land in SPEC-006), and this
  spec must add **no new dep**. So for SPEC-003, define a minimal local type
  that is forward-compatible:

  ```rust
  /// Operation parameters. SPEC-003 ships no parameterized ops, so this is a
  /// placeholder that SPEC-006 will widen to a serde/TOML value when the
  /// recipe layer (and its `toml`/`serde` deps) arrive. Identity/Invert return
  /// `OperationParams::None`.
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum OperationParams {
      /// The operation takes no parameters.
      None,
  }
  ```

  This keeps `operation/` recipe-agnostic and dependency-free while giving
  SPEC-006 a single type to extend. `params()` for both `Identity` and `Invert`
  returns `OperationParams::None`. **Do not** import `toml`/`serde`. (The
  signature in Outputs shows `toml::Value` only to mark intent — implement the
  enum above.)

- **The constructor question (the one real design decision).** `Operation::apply`
  must return a *new* `Image` built from transformed pixels, and tests must
  build an `Image` from an in-memory `DynamicImage`. SPEC-002's `Image` has no
  public `DynamicImage` constructor. Add one **minimal** public constructor to
  `src/image/mod.rs` — this is a sanctioned small extension of the SPEC-002
  type, expected by DEC-002 ("the pipeline … transforms it"):

  ```rust
  impl Image {
      /// Build an Image from already-decoded pixels, carrying through the
      /// source format and metadata bundle. Used by Operations to return a
      /// transformed image without re-decoding (decode-once, DEC-002).
      pub fn from_parts(
          pixels: DynamicImage,
          source_format: ImageFormat,
          metadata: Option<MetadataBundle>,
      ) -> Image { /* construct the struct */ }

      /// Replace this image's pixels, preserving source_format + metadata.
      /// Ergonomic helper for Operations (consumes self, returns a new Image).
      pub fn with_pixels(self, pixels: DynamicImage) -> Image { /* ... */ }
  }
  ```

  Add a unit test for each (every-public-fn-tested). `with_pixels` is the
  ergonomic path an `Operation` uses: `Ok(img.with_pixels(inverted))`. Carrying
  `source_format` + `metadata` through unchanged keeps the metadata lane
  (DEC-003) intact — operations never touch metadata, they just pass it
  along. If you would rather expose only `from_parts` and have ops call
  `Image::from_parts(new_pixels, img.source_format(), img.metadata().cloned())`,
  that is acceptable too — but `metadata()` returns `Option<&MetadataBundle>`,
  so you would clone; `with_pixels(self, …)` avoids the clone. Prefer
  `with_pixels`. Document whichever you choose in Build Completion. **This
  edit to `src/image/mod.rs` does NOT need a DEC** (it is a minor additive
  accessor under DEC-002), but it does need tests.

- **`Invert` implementation.** Convert `img.pixels().to_rgba8()` (gives an
  owned `RgbaImage`), map each pixel `[r,g,b,a] -> [255-r, 255-g, 255-b, a]`
  with a hand-written loop or `pixels_mut()`, wrap back into
  `DynamicImage::ImageRgba8(buf)`, and return `img.with_pixels(...)`. This is
  deliberately simple and lossless-for-the-test; do NOT optimize per color
  type or pull in `imageproc`. (Going through RGBA8 is fine for SPEC-003 —
  correctness over fidelity; later real ops can be color-type-aware.)

- **`Identity` implementation.** `fn apply(&self, img: Image) -> Result<Image,
  OperationError> { Ok(img) }`. That is the whole op.

- **`Pipeline::run` is a fold.** Iterate `self.ops`, threading the `Image`
  through `op.apply(img)?`. The `?` gives halt-on-first-error for free. An
  empty `ops` vec returns the input untouched. No cloning of the image between
  ops is required (each `apply` takes `Image` by value and returns one) — this
  is the decode-once property in code.

- **Pixel-equality in tests.** Compare `img.pixels().to_rgba8().into_raw()`
  (a `Vec<u8>`) for equality — robust across the RGBA conversion `Invert`
  uses. Build fixtures with `image::RgbaImage::from_fn(w, h, |x, y| …)` then
  `DynamicImage::ImageRgba8(buf)` then your chosen `Image` constructor.

- **Layering.** `src/operation/mod.rs` imports only `crate::image::Image` (and
  its sub-types) + `std` + `thiserror` + `image` types. `src/pipeline/mod.rs`
  imports `crate::operation::{Operation, OperationError}` + `crate::image::Image`.
  Neither imports clap, recipe, source, sink, or `std::fs`/`std::path`.

- **Refer to the crate as `::image`** inside modules to avoid the
  module-name collision with `crate::image` (SPEC-002 did the same).

- **Test-only helper ops** (`RecordOrder`, `AlwaysFails`) live inside the
  `#[cfg(test)] mod tests` blocks; they are not public API and do not count
  against the no-disk-IO guard (they do no disk IO anyway).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-003-operation-trait-and-pipeline`
- **PR (if applicable):** #3 — https://github.com/jysf/crustyimg/pull/3
- **All acceptance criteria met?** yes
- **Constructor choice:** Both `from_parts` and `with_pixels` were implemented.
  `with_pixels(self, DynamicImage) -> Image` is the ergonomic path used by
  `Invert::apply` — it consumes the input `Image` and replaces only its pixels,
  carrying `source_format` + `metadata` through without any clone. `from_parts`
  is the lower-level constructor used in tests (and available to future ops that
  need to manufacture an `Image` without an existing one to consume). Providing
  both avoids any ambiguity; the spec explicitly listed both and preferred
  `with_pixels` for ops, which matches usage here.
- **No-disk-IO guard:** Two-stage heuristic:
  1. Split each source file on `#[cfg(test)]` and take only the text before that
     marker (the library code). This isolates the non-test code in both
     `src/operation/mod.rs` and `src/pipeline/mod.rs`, which each have a single
     `#[cfg(test)] mod tests` block at the end.
  2. Filter out all comment lines (lines whose trimmed prefix is `//`) before
     checking for the forbidden tokens. This is necessary because both module
     doc-comment headers (`//!`) mention `std::fs`/`std::path` to *document* the
     constraint — the heuristic must not fire on those. Documented in the
     integration test (`tests/pipeline.rs` §`operation_and_pipeline_sources_do_no_disk_io`).
- **New decisions emitted:**
  - No new DEC. `image`/`thiserror` are pre-justified by DEC-002/DEC-007;
    `Image::from_parts`/`with_pixels` and `OperationParams::None` are
    spec-mandated additive changes.
- **Deviations from spec:**
  - None. `OperationParams` is a local `enum` (not `toml::Value`), exactly as
    the "Notes for the Implementer" specifies. The structural guard strips
    comment lines in addition to splitting on `#[cfg(test)]`; this is the
    "brittle heuristic" fallback the spec explicitly sanctions and asks to be
    documented here.
- **Follow-up work identified:**
  - None beyond the existing SPEC-004 through SPEC-007 backlog items.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The structural guard heuristic for `operation_and_pipeline_sources_do_no_disk_io`
   was underspecified at first glance: the spec mentions two options (split on
   `#[cfg(test)]` vs. exclude test lines) but does not foreground the subtlety
   that module doc comments (`//!`) will also contain the forbidden tokens.
   The first implementation caught fire on `//! … std::fs …` in the doc header.
   The fix (filter comment lines) is natural but took one iteration. The spec
   does note "this heuristic may prove brittle" and authorizes the comment-strip
   approach — so it was handled correctly, but a note in the spec that doc
   comments are the likely first failure mode would have saved that iteration.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — All relevant constraints were listed. One minor gap: the spec says
   "`with_pixels` is preferred" for ops but doesn't explicitly say whether
   `from_parts` is also required. Implementing both (as the Notes section
   implies) is the right call and needed no judgment call, but the sentence
   could be clearer.

3. **If you did this task again, what would you do differently?**
   — Read the structural-guard test description more carefully before writing it,
   specifically noting the doc-comment pitfall. Everything else was
   straightforward: the trait/pipeline design was fully prescribed, the
   `OperationParams` enum was clear, and the `Invert` pixel loop was simple.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — First spec built on Sonnet 4.6 — the highly prescriptive build prompt
   (exact signatures, exact test names, hard rules) was the key; it passed
   Opus verify with no punch list. Keep build prompts that prescriptive when
   routing builds to Sonnet.

2. **Does any template, constraint, or decision need updating?**
   — No DEC/constraint change from the pipeline itself. But this spec made the
   security gap concrete: `Image::load` still has no decode limits, and the
   pipeline now runs untrusted-input ops — both belong in the planned STAGE-006
   hardening pass (added to the plan this session).

3. **Is there a follow-up spec I should write now before I forget?**
   — The added `Image::with_pixels`/`from_parts` constructors are the seam every
   real transform (SPEC resize/crop/filters) will use; no new spec needed — they're
   covered. Decode limits are tracked in STAGE-006, not a separate near-term spec.
