---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-010
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # usually same Claude, different session
  created_at: 2026-06-14

references:
  decisions: [DEC-014, DEC-008, DEC-002, DEC-005, DEC-007]
  constraints:
    - single-image-library
    - decode-once-no-per-op-disk
    - no-new-top-level-deps-without-decision
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - test-before-implementation
    - untrusted-input-hardening
  related_specs: [SPEC-003, SPEC-006]

# One sentence on what this spec contributes to its stage's
# value_contribution.
value_link: "Delivers STAGE-003's first pixel-geometry Operation (`resize`, all six modes on the fast_image_resize SIMD backend) AND the first parameterized-operation params mechanism (DEC-014) + registry wiring + recipe round-trip — recipe-usable with zero CLI, the foundation SPEC-011's resize command and every later parameterized op build on."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "design cycle, Opus subagent; SPEC-010 = library half of split resize"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 1
---

# SPEC-010: resize operation and operation params mechanism

## Context

This spec lands the **library half** of `resize`: the `Resize` `Operation`
(all six modes on the `fast_image_resize` SIMD backend, DEC-008) **and** the
first real parameterized-operation params mechanism (DEC-014). It is
recipe-usable with **zero CLI** — a recipe step `op = "resize"` with the
pinned param keys constructs and runs a `Resize` through the existing
registry → pipeline path.

- **Parent stage:** `STAGE-003` (transform and output) — the first stage
  that mutates pixels. `resize` was assessed complexity **L** and **split**
  (approved): SPEC-010 is the library (operation + params mechanism +
  registry + parity tests); **SPEC-011** is a separate later spec for the
  `resize` CLI command + multi-input `--out-dir` fan-out. The split falls on
  the library↔CLI layering boundary. See the STAGE-003 backlog note
  (2026-06-15).
- **Project:** `PROJ-001` (crustyimg MVP).
- **Why the params mechanism comes now:** SPEC-006 left `OperationParams` a
  `None`-only enum that *errors* on any non-empty param map — a
  forward-compat stub. `resize` is the first op with real params, so this
  spec rewrites `OperationParams` into a generic ordered-map newtype and
  moves per-op validation into each op's constructor (**DEC-014**). The
  recipe round-trip (DEC-005) is preserved: `invert` stays zero-extra-keys;
  the data-model's resize step round-trips via `Recipe` `PartialEq`.

The core machinery already exists: the `Operation` trait + `Pipeline`
(SPEC-003), the TOML recipe + `OperationRegistry` (SPEC-006). This spec
extends `OperationParams`, adds the `Resize` op, registers `"resize"`, and
adds typed param-error plumbing through `registry` → `recipe`.

## Goal

Add a `Resize` `Operation` supporting six mutually-exclusive modes
(max/exact/percent/fit/fill/cover) on the `fast_image_resize` SIMD backend
(DEC-008), constructed **from `OperationParams`** via the registry; and
rewrite `OperationParams` into a generic ordered-map newtype with typed
accessors so each op parses and validates its own params (DEC-014), with the
recipe round-trip and `invert`'s zero-extra-keys form preserved.

## Inputs

- **Files to read:**
  - `src/operation/mod.rs` — the `Operation` trait, the `OperationParams`
    `None`-only enum + its hand-written serde (to be rewritten), `Identity` /
    `Invert` (their `params()` returns `OperationParams::None` → migrate to
    `empty()`), `OperationError::Apply { op, reason }`, and the SPEC-003 unit
    tests that assert `== OperationParams::None` (migrate to `empty()`).
  - `src/operation/registry.rs` — `Constructor =
    fn(&OperationParams) -> Result<Box<dyn Operation>, RegistryError>`;
    `RegistryError` (only `Unknown` today — add `InvalidParams`);
    `OperationRegistry::{with_builtins, register, build}`; the `build`-path
    tests.
  - `src/recipe/mod.rs` — `RecipeStep { op, #[serde(flatten)] params }`,
    `Recipe`, `build_pipeline` (the `.map_err(|_| UnknownOperation)` that
    must distinguish param errors), `RecipeError` (add `InvalidOperation`),
    the round-trip tests.
  - `src/pipeline/mod.rs` — `Pipeline` (unchanged; `Resize` flows through
    `run`).
  - `src/image/mod.rs` — `Image::pixels()`, `Image::with_pixels(self,
    DynamicImage)`, `Image::from_parts` (the `Invert` op is the structural
    template: convert to RGBA8, transform, `with_pixels(ImageRgba8(...))`).
  - `docs/data-model.md` — the recipe schema + the worked `resize` step
    (`mode="max"`, `width=1200`, height omitted) the round-trip must honor.
  - `tests/recipe_round_trip.rs`, `tests/pipeline.rs`, `tests/common/mod.rs`
    — integration conventions + native in-memory fixtures.
- **External APIs:** `fast_image_resize` 5.5.0 (DEC-008; the crate adds NO
  new DEC — DEC-008 covers it). The exact 5.5.0 API is pinned in
  **Notes for the Implementer** (verified to compile against the repo's
  `image v0.25.10`). Oracle: `image::imageops::resize` (already available via
  the `image` dep — no new feature).
- **Related code paths:** `src/operation/` (the op + params + registry),
  `src/recipe/` (the param-error plumbing). **NOT** `src/cli/` — see
  Out of scope.

## Outputs

- **Files modified:**
  - `Cargo.toml` — add under `[dependencies]` (always-on, NOT optional, NOT a
    feature; DEC-008 covers it, no new DEC):
    ```toml
    fast_image_resize = { version = "=5.5.0", features = ["image"] }
    ```
    Verified pure-Rust (deps: bytemuck / cfg-if / document-features /
    num-traits; the `image` feature resolves to the exact `image v0.25.10`
    pin without forcing its default features). No `-sys`/cc/cmake/bindgen.
  - `src/operation/mod.rs` —
    - **Rewrite `OperationParams`** into a newtype over an ordered map
      (DEC-014):
      ```rust
      use std::collections::BTreeMap;

      /// Operation parameters: an ordered map of TOML values (DEC-014).
      ///
      /// Flatten-serialized into the `[[step]]` table by `RecipeStep`. An
      /// empty map emits zero keys (so `op = "invert"` stays clean); a
      /// populated map round-trips verbatim. Each `Operation` parses and
      /// validates its own keys in its constructor via the typed accessors
      /// below — there is no per-op logic in the serde impls (the flatten
      /// boundary has no `op` context).
      #[derive(Debug, Clone, PartialEq)]
      pub struct OperationParams(BTreeMap<String, toml::Value>);

      impl OperationParams {
          /// The empty param set (parameterless ops: Identity, Invert).
          pub fn empty() -> Self { OperationParams(BTreeMap::new()) }

          /// Build from an ordered map (used by ops recording their params).
          pub fn from_map(map: BTreeMap<String, toml::Value>) -> Self {
              OperationParams(map)
          }

          /// Whether any params are present.
          pub fn is_empty(&self) -> bool { self.0.is_empty() }

          /// Borrow a string param, if present and a string.
          pub fn get_str(&self, key: &str) -> Option<&str> {
              self.0.get(key).and_then(toml::Value::as_str)
          }

          /// Extract a `u32` param, if present and a non-negative integer in range.
          pub fn get_u32(&self, key: &str) -> Option<u32> {
              self.0.get(key)
                  .and_then(toml::Value::as_integer)
                  .and_then(|i| u32::try_from(i).ok())
          }

          /// Extract an `f32` param, if present (accepts integer or float TOML).
          pub fn get_f32(&self, key: &str) -> Option<f32> {
              self.0.get(key).and_then(|v| match v {
                  toml::Value::Float(f) => Some(*f as f32),
                  toml::Value::Integer(i) => Some(*i as f32),
                  _ => None,
              })
          }
      }

      impl Default for OperationParams {
          fn default() -> Self { OperationParams::empty() }
      }
      ```
    - **Hand-write `Serialize`/`Deserialize`** delegating to the inner map:
      `Serialize` collects the map's entries as a serde map (empty → zero
      keys, preserving `op = "invert"`); `Deserialize` reads a
      `BTreeMap<String, toml::Value>` and wraps it — **no per-op validation,
      no error on a non-empty map** (the old enum's hard error is dropped;
      DEC-014).
    - **Migrate** `Identity::params()` / `Invert::params()` to return
      `OperationParams::empty()`. Update the SPEC-003 unit tests in this
      module that assert `== OperationParams::None` to assert
      `== OperationParams::empty()` (or `.is_empty()`).
    - **Add the `Resize` op** (see schema + math below):
      ```rust
      /// Mode of a Resize operation (the six geometry strategies).
      #[derive(Debug, Clone, Copy, PartialEq, Eq)]
      enum ResizeMode { Max, Exact, Percent, Fit, Fill, Cover }

      /// Geometric resize on the fast_image_resize SIMD backend (DEC-008).
      ///
      /// Constructed FROM params via `Resize::from_params` (the registry
      /// path). Converts to RGBA8 (like Invert), resizes (Lanczos3
      /// convolution), and—for `fill`—center-crops to the exact box.
      pub struct Resize {
          mode: ResizeMode,
          /// Per-mode target inputs (see the param schema). Carried so
          /// `params()` round-trips back to the same recipe step.
          width: Option<u32>,
          height: Option<u32>,
          percent: Option<f32>,
      }

      impl Resize {
          /// Parse + validate params (DEC-014). Returns a typed
          /// `RegistryError::InvalidParams` on a missing/wrong/out-of-range
          /// param. Never panics.
          pub fn from_params(params: &OperationParams)
              -> Result<Self, RegistryError> { /* ... */ }
      }

      impl Operation for Resize {
          fn name(&self) -> &'static str { "resize" }
          fn params(&self) -> OperationParams { /* reconstruct the map */ }
          fn apply(&self, img: Image) -> Result<Image, OperationError> { /* ... */ }
      }
      ```
  - `src/operation/registry.rs` —
    - Add a `RegistryError` variant:
      ```rust
      /// A constructor rejected its params (wrong/missing/out-of-range).
      #[error("invalid params for operation '{op}': {reason}")]
      InvalidParams { op: &'static str, reason: String },
      ```
    - Register `"resize"`:
      `reg.register("resize", |p| Ok(Box::new(Resize::from_params(p)?)));`
      (the `?` surfaces `RegistryError::InvalidParams`).
  - `src/recipe/mod.rs` —
    - Add a `RecipeError` variant for a param error (so it is **not**
      mis-reported as `UnknownOperation`):
      ```rust
      /// An op name resolved but its params were invalid (DEC-014).
      #[error("invalid operation '{name}': {reason}")]
      InvalidOperation { name: String, reason: String },
      ```
    - Change `build_pipeline`'s `.map_err` to **match on the registry error
      kind** instead of collapsing every error to `UnknownOperation`:
      ```rust
      let op = registry.build(&step.op, &step.params).map_err(|e| match e {
          RegistryError::Unknown { name } => RecipeError::UnknownOperation { name },
          RegistryError::InvalidParams { op, reason } =>
              RecipeError::InvalidOperation { name: op.to_owned(), reason },
      })?;
      ```
- **New exports / signatures:**
  - `src/operation/mod.rs`: `OperationParams` newtype with
    `empty()` / `from_map()` / `is_empty()` / `get_str()` / `get_u32()` /
    `get_f32()` + `impl Default`; `pub struct Resize` + `pub fn
    Resize::from_params(&OperationParams) -> Result<Resize, RegistryError>`.
    (`ResizeMode` stays private.)
  - `src/operation/registry.rs`: `RegistryError::InvalidParams { op, reason }`.
  - `src/recipe/mod.rs`: `RecipeError::InvalidOperation { name, reason }`.
- **Database changes:** none.

### Param-key schema (`op = "resize"`) — PINNED

A `resize` step carries a required `mode` plus per-mode dimension keys. Keys
are flat integer/float values in the `[[step]]` table (parsed via the
`OperationParams` accessors) — **not** a `"WxH"` string (string parsing is a
CLI concern → SPEC-011). This mirrors `docs/data-model.md`'s worked step.

| `mode` | Required keys | Meaning |
|---|---|---|
| `"max"` | `width` (= N, the long-edge cap) | Scale so the **longest edge ≤ N**; **never upscale**. |
| `"exact"` | `width`, `height` | Force exactly `width`×`height`; aspect ignored. |
| `"percent"` | `percent` (P, integer or float) | Scale both dims by `P/100`. |
| `"fit"` | `width`, `height` | Scale to **fit inside** `width`×`height` (aspect kept); **never upscale**. |
| `"cover"` | `width`, `height` | Scale to **cover** `width`×`height` (aspect kept; **may upscale**); **no crop**. |
| `"fill"` | `width`, `height` | `cover` THEN **center-crop** to exactly `width`×`height`. |

- Missing `mode`, an unknown `mode` value, a missing required key for the
  chosen mode, a wrong-typed key, or a non-positive dimension/percent →
  `RegistryError::InvalidParams { op: "resize", reason: <which> }`.
- `params()` reconstructs **only** the keys the mode uses (so `max` records
  `{mode, width}` — height omitted — and round-trips to the same step).

### Six-mode math — EXACT

Given source `(w, h)`, compute target `(tw, th)` (then resize; for `fill`,
crop after). All scale results round to nearest, then clamp each target dim
to `≥ 1`.

- **`max` (cap N):** `s = min(N / max(w, h), 1.0)` (NO upscale);
  `tw = round(w·s)`, `th = round(h·s)`.
- **`exact` (W, H):** `tw = W`, `th = H` (aspect ignored).
- **`percent` (P):** `tw = round(w · P/100)`, `th = round(h · P/100)`.
- **`fit` (W, H):** `s = min(W/w, H/h, 1.0)` (fit inside, NO upscale);
  `tw = round(w·s)`, `th = round(h·s)`.
- **`cover` (W, H):** `s = max(W/w, H/h)` (cover the box, MAY upscale, NO
  crop); `tw = round(w·s)`, `th = round(h·s)`.
- **`fill` (W, H):** `s = max(W/w, H/h)` (cover); resize to
  `(round(w·s), round(h·s))`; THEN **center-crop** to exactly `(W, H)`
  (offsets `x = (rw − W)/2`, `y = (rh − H)/2`, clamped ≥ 0). This resolves
  the api-contract's fill-vs-cover ambiguity: **fill = cover + center-crop.**

### Oversize hardening — PINNED (`untrusted-input-hardening`)

Before allocating the destination buffer, reject targets that are too large:
**any target edge `> 50_000` px OR target area `> 268_435_456` (256·1024·1024
≈ 268M) px** → `OperationError::Apply { op: "resize", reason: <bounds> }`.
Never OOM or panic. (This bounds the recipe-driven path; the image *decode*
bound is separate, in `Image::load`.)

## Acceptance Criteria

Each criterion maps to a test in **Failing Tests**.

- [ ] AC1 — `Resize` in each mode produces the **exact** documented output
  dimensions (max/exact/percent/fit/cover; and `fill` = exactly W×H). →
  `resize_<mode>_exact_dims` unit tests
- [ ] AC2 — `max` and `fit` **never upscale** (a target larger than the
  source clamps to source size / `s = 1.0`). → `resize_max_no_upscale`,
  `resize_fit_no_upscale`
- [ ] AC3 — `cover` **may upscale** and never crops (output aspect may
  differ from the box; both dims ≥ the box). → `resize_cover_may_upscale`
- [ ] AC4 — `fill` center-crops to exactly W×H, and the crop is **centered**
  (a constructed off-center-feature image lands the feature where centering
  predicts). → `resize_fill_center_crops_exact`,
  `resize_fill_crop_is_centered`
- [ ] AC5 — **Parity vs the `image::imageops::resize` oracle:** resizing a
  native fixture with `Resize` and with the oracle at the same target dims
  yields **mean per-channel absolute difference below tolerance** (NOT
  pixel-exact — SIMD/filter rounding differs). → `resize_parity_within_tolerance`
- [ ] AC6 — Bad/missing params → typed `RegistryError::InvalidParams`
  (missing `mode`; unknown `mode`; missing required dim; non-positive dim).
  → `resize_from_params_*` unit tests
- [ ] AC7 — Oversize target → `OperationError::Apply { op: "resize", .. }`
  (no panic/OOM). → `resize_oversize_is_typed_error`
- [ ] AC8 — A `resize` recipe step round-trips via `Recipe` `PartialEq`
  (`mode="max"`, `width=1200`, height omitted ⇒ same step back). →
  `resize_recipe_round_trips`
- [ ] AC9 — `invert` still serializes to **zero extra keys** after the
  `OperationParams` rewrite (the round-trip + schema tests stay green). →
  `invert_params_still_zero_keys` (+ the existing SPEC-006 round-trip tests)
- [ ] AC10 — A `resize` step with an invalid param surfaces as
  `RecipeError::InvalidOperation` (NOT `UnknownOperation`) through
  `build_pipeline`. → `resize_invalid_params_is_invalid_operation`
- [ ] AC11 — `OperationParams::empty()` is the parameterless form
  (`Identity`/`Invert` return it; `is_empty()` is true); migrated SPEC-003
  unit tests pass. → `params_empty_is_parameterless`
- [ ] AC12 — the existing recipe/registry/pipeline suites
  (`recipe_round_trips_through_toml`, `serialized_toml_matches_schema`,
  `unknown_operation_is_typed_error`, etc.) stay **green** (no behavioral
  regression from the rewrite). → asserted by re-running the full suite

## Failing Tests

Written during **design**, BEFORE build. The implementer makes these pass.
Native in-memory fixtures only (the `image` crate); no committed binaries.

- **`src/operation/mod.rs`** unit tests (in the existing `#[cfg(test)] mod
  tests`; build images with the module's `make_image` helper):
  - `params_empty_is_parameterless` — `OperationParams::empty().is_empty()`
    is true; `Identity.params() == OperationParams::empty()`;
    `Invert.params() == OperationParams::empty()`. (AC11)
  - `resize_max_exact_dims` — source 100×50, `max` N=20 ⇒ `s = 20/100 = 0.2`
    ⇒ exactly 20×10. (AC1)
  - `resize_max_no_upscale` — source 40×30, `max` N=100 ⇒ `s` clamps to 1.0
    ⇒ exactly 40×30 (no upscale). (AC2)
  - `resize_exact_exact_dims` — source 100×50, `exact` 33×77 ⇒ exactly
    33×77 (aspect ignored). (AC1)
  - `resize_percent_exact_dims` — source 100×50, `percent` 50 ⇒ exactly
    50×25. (AC1)
  - `resize_fit_exact_dims` — source 100×50, `fit` 40×40 ⇒
    `s = min(40/100, 40/50, 1) = 0.4` ⇒ exactly 40×20. (AC1)
  - `resize_fit_no_upscale` — source 30×20, `fit` 300×300 ⇒ `s` clamps to
    1.0 ⇒ exactly 30×20. (AC2)
  - `resize_cover_exact_dims` — source 100×50, `cover` 40×40 ⇒
    `s = max(40/100, 40/50) = 0.8` ⇒ exactly 80×40 (covers the box, no
    crop). (AC1)
  - `resize_cover_may_upscale` — source 20×10, `cover` 100×100 ⇒
    `s = max(5, 10) = 10` ⇒ exactly 200×100 (upscaled, ≥ box on both dims).
    (AC3)
  - `resize_fill_center_crops_exact` — source 100×50, `fill` 40×40 ⇒ cover
    `s = 0.8` → 80×40 then center-crop ⇒ exactly 40×40. (AC4)
  - `resize_fill_crop_is_centered` — build a source whose center column/row
    carries a distinct color; `fill` to a smaller square; assert the
    distinctive feature lands at the cropped image's center (within ±1 px),
    proving the crop is centered, not top-left. (AC4)
  - `resize_parity_within_tolerance` — build a non-trivial gradient RGBA
    fixture (e.g. 64×48); resize to 32×24 with `Resize` (Lanczos3) and with
    `image::imageops::resize(&rgba, 32, 24, FilterType::Lanczos3)`; assert
    **mean per-channel abs diff < a small tolerance** (start at `<= 6.0` on
    0–255; tighten if comfortably lower — document the chosen number in a
    comment). Same target dims on both sides. (AC5)
  - `resize_oversize_is_typed_error` — `exact` 60_000×10 (edge > 50_000)
    ⇒ `Resize::from_params` succeeds but `.apply(small_img)` returns
    `Err(OperationError::Apply { op: "resize", .. })`; also an area-bound
    case (e.g. 20_000×20_000 = 4·10^8 > 268M) ⇒ same. No panic. (AC7)
  - `resize_from_params_missing_mode` — params `{}` (or no `mode`) ⇒
    `Err(RegistryError::InvalidParams { op: "resize", .. })`. (AC6)
  - `resize_from_params_unknown_mode` — `mode="bogus"` ⇒ `InvalidParams`. (AC6)
  - `resize_from_params_missing_dim` — `mode="exact"` with only `width` ⇒
    `InvalidParams`. (AC6)
  - `resize_from_params_nonpositive_dim` — `mode="exact"`, `width=0`,
    `height=10` ⇒ `InvalidParams`. (AC6)
  - `resize_params_round_trips_max` — construct `Resize` via `from_params`
    `{mode="max", width=1200}`; `op.params()` ⇒ a map with exactly
    `{mode:"max", width:1200}` (height absent), so the step round-trips. (AC8)
  - `invert_params_still_zero_keys` — `Invert.params().is_empty()` is true
    (the rewrite kept `invert` parameterless). (AC9)

- **`src/operation/registry.rs`** unit tests (existing `#[cfg(test)] mod
  tests`):
  - `with_builtins_contains_resize` — `OperationRegistry::with_builtins()`
    contains `"resize"` (and still `"identity"`/`"invert"`).
  - `build_resize_with_valid_params` — `build("resize", &params{mode="max",
    width=64})` returns an op whose `name() == "resize"`.
  - `build_resize_invalid_params_is_typed` — `build("resize", &empty)` ⇒
    `Err(RegistryError::InvalidParams { op: "resize", .. })`. (AC6)
  - (Update the existing `build("...", &OperationParams::None)` call sites to
    `&OperationParams::empty()`.)

- **`tests/recipe_round_trip.rs`** (integration; reuse its conventions):
  - `resize_recipe_round_trips` — build a `Recipe` with one step
    `op="resize"`, params `{mode="max", width=1200}` (height omitted);
    `to_toml` → `from_toml` ⇒ equal via `PartialEq`; assert the TOML contains
    `op = "resize"`, `mode = "max"`, `width = 1200`, and does **not** contain
    a `height` key. (AC8 — mirrors `docs/data-model.md`.)
  - `resize_invalid_params_is_invalid_operation` — a recipe
    `version="1"` with a `resize` step missing `mode`; `from_toml` succeeds
    (parse-valid); `build_pipeline(&with_builtins())` ⇒
    `Err(RecipeError::InvalidOperation { name, .. })` where `name == "resize"`
    (NOT `UnknownOperation`). (AC10)
  - `invert_recipe_still_round_trips` — an `invert`-only recipe still
    round-trips with zero extra keys (the existing
    `recipe_round_trips_through_toml` / `serialized_toml_matches_schema` also
    cover this; add this only if it reads clearer). (AC9/AC12)
  - (Update the existing `OperationParams::None` references in this file to
    `OperationParams::empty()`.)

- **`tests/pipeline.rs`** (integration) — OPTIONAL but recommended:
  - `resize_runs_through_pipeline` — push a `Resize` (built via
    `from_params {mode="exact", width=8, height=8}`) into a `Pipeline`, run
    it over a native 16×16 fixture, assert the output is 8×8. (Proves the op
    flows through the executor end-to-end.)

The existing `pipeline.rs` / `recipe_round_trip.rs` tests that reference
`OperationParams::None` must be migrated to `OperationParams::empty()` and
must stay green. Run the FULL `cargo test`.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-014` — `OperationParams` becomes a generic ordered-map newtype; each
  op parses/validates its own params in its constructor; recipe round-trip
  preserved. **This spec is DEC-014's first implementation.** The old
  `None`-only enum's hard error on a non-empty map is **dropped**.
- `DEC-008` — resize backend is `fast_image_resize` 5 (SIMD). DEC-008 covers
  the crate — **no new DEC for the dependency**. `image::imageops::resize` is
  the correctness oracle (parity test). Stay on 5.x (the pin is `=5.5.0`; do
  NOT jump to 6.0).
- `DEC-002` — single canonical `Image` over `DynamicImage`; the op converts
  in/out of RGBA8 at its boundary (like `Invert`) and returns via
  `with_pixels` — decode-once preserved. The op owns its transform and (now)
  its param contract.
- `DEC-005` — recipe round-trip + the registry seam: a new op registers in
  `with_builtins`; the recipe parser is **not** edited for the op itself
  (only the shared param-error mapping is touched, once).
- `DEC-007` — typed errors → exit codes at the binary boundary. New typed
  variants `RegistryError::InvalidParams` and `RecipeError::InvalidOperation`
  flow through the existing `CliError::Recipe(_) => 1` mapping (confirm in
  `src/cli/mod.rs` that `CliError::Recipe(#[from] RecipeError)` exists and
  maps to exit 1 — it covers the new variant generically; **do not edit the
  CLI**).

### Constraints that apply

These apply to the paths this task touches (see `/guidance/constraints.yaml`):

- `single-image-library` — `fast_image_resize` is NOT a second *pixel
  model*; it is a resize kernel that converts in/out of `image`'s RGBA8 at
  its boundary (DEC-008). The canonical `Image`/`DynamicImage` is unchanged.
- `decode-once-no-per-op-disk` — `Resize::apply` is pure in-memory; no disk.
- `no-new-top-level-deps-without-decision` — `fast_image_resize` is covered
  by **DEC-008** (no new DEC). Call it out in the PR.
- `no-unwrap-on-recoverable-paths` — NO `unwrap`/`expect`/`panic!` in `src/`.
  Map **every** `fast_image_resize` error (`from_vec_u8`, `resize`) and the
  `from_raw`/`from_vec` reconstruction to
  `OperationError::Apply { op: "resize", reason: ... }`. Param errors are
  typed `RegistryError::InvalidParams`.
- `every-public-fn-tested` — `OperationParams` accessors, `Resize::from_params`,
  and `Resize::apply` each get unit coverage (above).
- `clippy-fmt-clean` — `cargo clippy -- -D warnings` + `cargo fmt --check`
  clean. Watch float casts (`as f32`/`as u32`) — keep rounding explicit
  (`(x).round() as u32` after a `.max(1.0)` clamp) to avoid clippy churn.
- `test-before-implementation` — the failing tests above are the contract.
- `untrusted-input-hardening` — the oversize cap (≤ 50_000 px/edge, area ≤
  268M) bounds the recipe-driven target before allocation; failure is a typed
  error, never an OOM/panic.

### Prior related work

- `SPEC-003` (shipped) — the `Operation` trait, `Pipeline`, `Identity` /
  `Invert`. `Invert::apply` is the **structural template** for `Resize::apply`
  (RGBA8 in, transform, `with_pixels(ImageRgba8(...))` out).
- `SPEC-006` (shipped) — the TOML recipe, `OperationRegistry`, the round-trip
  guarantee, and the `OperationParams` `None`-only stub this spec rewrites.

### Out of scope (for this spec specifically)

If any of these feel necessary during build, write a new spec — do not
expand this one. **If anything forces a `src/cli/` change, STOP and flag it**
(it belongs in SPEC-011):

- **The `resize` CLI command** (clap `Commands::Resize`, `run_resize`, the
  `--max`/`--exact`/`--fit`/… flags, `"WxH"`-string parsing) — **SPEC-011**.
  This spec is library-only; touch **nothing** under `src/cli/`.
- **Multi-input `--out-dir` fan-out** — SPEC-011 (sequential, no rayon).
- **`thumbnail` / `shrink` / `convert` / `auto-orient`** — later STAGE-003
  specs (they reuse this op + the params mechanism).
- **`rayon` / any parallelism** — STAGE-005 (DEC-006 forbids it here).
- **WebP / AVIF** — fast-follow / feature-gated later (DEC-004).
- A second pixel library, or exposing the resize *backend choice* on the
  recipe/CLI surface (DEC-008 keeps it an internal detail).

## Notes for the Implementer

### VERIFIED `fast_image_resize` 5.5.0 API (use verbatim — do not guess)

This block was **compiled and run** against the repo's exact `image v0.25.10`
pin. Operate via RGBA8 (like `Invert`):

```rust
// img: crate::image::Image  →  rgba: image::RgbaImage
let rgba = img.pixels().to_rgba8();
let (w, h) = (rgba.width(), rgba.height());

let src = fast_image_resize::images::Image::from_vec_u8(
    w, h, rgba.into_raw(), fast_image_resize::PixelType::U8x4,
).map_err(|e| OperationError::Apply { op: "resize", reason: e.to_string() })?;

let mut dst = fast_image_resize::images::Image::new(
    dw, dh, fast_image_resize::PixelType::U8x4,
);

let mut resizer = fast_image_resize::Resizer::new();
let opts = fast_image_resize::ResizeOptions::new().resize_alg(
    fast_image_resize::ResizeAlg::Convolution(fast_image_resize::FilterType::Lanczos3),
);
resizer.resize(&src, &mut dst, &opts)
    .map_err(|e| OperationError::Apply { op: "resize", reason: e.to_string() })?;

let out = image::RgbaImage::from_raw(dw, dh, dst.into_vec())
    .ok_or(OperationError::Apply { op: "resize", reason: "buffer/dim mismatch".into() })?;
let result = img.with_pixels(image::DynamicImage::ImageRgba8(out));
```

- Symbol paths confirmed for 5.5.0: `fast_image_resize::images::Image`,
  `::PixelType::U8x4`, `::Resizer::new()`, `::ResizeOptions::new()`,
  `::ResizeAlg::Convolution(..)`, `::FilterType::Lanczos3`. Error types
  (`from_vec_u8` → `ImageBufferError`; `resize` → `ResizeError`) are
  `Display`, so `e.to_string()` works. (If 5.5.0 ever differs in your
  toolchain, correct against `cargo doc` and note it — but it was verified to
  compile as written.)
- **`fill`:** resize to the *cover* dims first (as above), then center-crop.
  Crop in `image`-land with `image::imageops::crop_imm(&out, x, y, W, H)
  .to_image()` (returns an owned `RgbaImage`), then wrap via `with_pixels`.
  `crop_imm` is total (it clamps), but compute `x`/`y` so the crop is
  centered and within bounds.

### Param parsing + validation (`Resize::from_params`)

- Read `mode = get_str("mode")` → match the six literals; anything else (or
  absent) → `RegistryError::InvalidParams { op: "resize", reason: ... }`.
- Per mode, pull the required keys with `get_u32` / `get_f32`; a missing or
  wrong-typed key, or a non-positive value, → `InvalidParams`. `max` reads
  `width` (the cap N); `percent` reads `percent`; the box modes read
  `width`+`height`.
- Store only what the mode uses (so `params()` reconstructs the minimal map).
- `params()` rebuilds a `BTreeMap<String, toml::Value>` with `mode` plus the
  mode's keys (`toml::Value::Integer(n as i64)` for dims;
  `toml::Value::Float`/`Integer` for percent) → `OperationParams::from_map`.
  Keep the key set minimal so the data-model round-trip holds (no `height`
  for `max`).

### Math + hardening

- Compute `(tw, th)` per the EXACT math; round to nearest then `.max(1)`.
- Enforce the oversize cap **on `(tw, th)`** (and, for `fill`, on the cover
  dims) **before** `Image::new(dw, dh, ..)` allocates → typed
  `OperationError::Apply`. This belongs in `apply` (it depends on the source
  dims, known only at apply time), not `from_params`.
- A 1×1 or degenerate target is valid (clamped ≥ 1); only the *upper* bound
  is enforced.

### Serde for `OperationParams` (the load-bearing migration)

- `Serialize`: iterate the inner `BTreeMap` into `serialize_map` — empty map
  emits **zero keys** (this is what keeps `op = "invert"` clean). Do NOT emit
  a `null` or a wrapper key.
- `Deserialize`: deserialize a `BTreeMap<String, toml::Value>` and wrap it.
  **Drop the old "error on a non-empty map" branch** — a non-empty map is now
  valid (it is some op's params); per-op validation happens later in the
  constructor. This is the single behavioral change that lets `resize`'s keys
  survive the flatten boundary while `invert` stays empty.
- `RecipeStep` / `Recipe` serde derives are **UNCHANGED**.

### Error plumbing (do this once, cleanly)

- `RegistryError::InvalidParams { op: &'static str, reason: String }` — the
  constructor's typed rejection. `RecipeError::InvalidOperation { name:
  String, reason: String }` — its recipe-layer counterpart. `build_pipeline`
  maps `Unknown → UnknownOperation` and `InvalidParams → InvalidOperation`
  (match, don't collapse). Confirm `CliError::Recipe(_) => 1` in
  `src/cli/mod.rs` already covers `RecipeError` generically — it does; **do
  not touch the CLI**. If a CLI edit ever seems forced, STOP (it's SPEC-011).

### Gotchas

- `OperationParams` can no longer derive `Eq` (it holds `toml::Value`, which
  is not `Eq` because of floats). Derive `PartialEq` only. Adjust any test or
  bound that assumed `Eq` (the SPEC-003 tests compared with `assert_eq!`,
  which needs only `PartialEq` + `Debug` — fine).
- Use `BTreeMap` (not `HashMap`) for a deterministic key order so the
  round-trip TOML is stable.
- Keep `ResizeMode` private; only `Resize` (+ `from_params`) is public.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
