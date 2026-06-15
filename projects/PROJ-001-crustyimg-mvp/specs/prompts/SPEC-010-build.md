# SPEC-010 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. Do not rely on any prior conversation. This prompt is
> deliberately prescriptive — follow it literally. Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-010 ("resize operation and operation params
mechanism"). You are NOT the architect; the spec file is your source of truth.
This spec is the LIBRARY HALF of a split `resize` feature: it adds the `Resize`
Operation (six modes on the fast_image_resize SIMD backend), rewrites
`OperationParams` into a generic TOML-map newtype so each Operation parses its
own params (DEC-014), registers `"resize"`, and plumbs a typed param-error
through registry → recipe — recipe-usable with ZERO CLI. The `resize` CLI
command + multi-input fan-out is a SEPARATE later spec (SPEC-011): do NOT design
or implement any CLI here. Use ABSOLUTE paths for every file you read or write.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════

1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — conventions: §5 stack (fast_image_resize is the resize backend, version
   "5" — DEC-008 justifies it; the pin is `=5.5.0`, do NOT jump to 6.0), §6 the
   EXACT commands (the gates below), §11 coding conventions (library-first; the
   pixel core `src/operation/` may depend ONLY on `crate::image`, `serde`,
   `::image`, `thiserror`, and now `fast_image_resize` — NOT clap/files/
   terminals/recipe/sink; typed errors; NO unwrap/expect/panic! on recoverable
   paths; group imports std/external/local; comments explain WHY not WHAT; no
   dead code), §12 testing (unit in `#[cfg(test)]`, integration under tests/,
   NATIVE in-memory fixtures via the `image` crate — NO ImageMagick, NO committed
   binary fixtures), §13 git/PR (branch naming, conventional commits +
   Co-Authored-By trailer, PR body template), §15 build-cycle rules (spec edits
   LIMITED to `## Build Completion`; append a build cost session entry; create
   DEC-* only for NON-trivial NEW decisions — NONE expected here, DEC-014 is
   already written).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-010-resize-operation-and-operation-params-mechanism.md
   — THE SPEC. Implement its "## Failing Tests", "## Outputs" (the rewritten
   `OperationParams` newtype + accessors, the `Resize` struct + `from_params`/
   `params`/`apply`, the PINNED param-key schema, the EXACT six-mode math, the
   oversize cap, the registry line, the `RegistryError::InvalidParams` +
   `RecipeError::InvalidOperation` additions, the Cargo.toml dep line), and
   "## Acceptance Criteria" exactly. Read "## Implementation Context" and "##
   Notes for the Implementer" in FULL — they carry the VERIFIED fast_image_resize
   5.5.0 API block (compiled against the repo's image pin — use it verbatim), the
   per-mode math, the fill=cover+center-crop resolution, the serde-migration
   detail (drop the old "error on non-empty map" branch), and the error-plumbing.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-014-operation-params-mechanism.md
   — `OperationParams` is a generic ordered-map newtype `(BTreeMap<String,
   toml::Value>)`; each Operation owns its param parsing/validation in its
   constructor; recipe round-trip preserved. The flatten boundary has NO `op`
   context — that's WHY validation lives in the constructor, not in Deserialize.
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-008-resize-backend-fast-image-resize.md ,
   .../DEC-002-single-image-model-and-operation-trait.md ,
   .../DEC-005-recipe-toml-and-operation-registry.md ,
   .../DEC-007-error-handling-thiserror-anyhow.md
   — DEC-008: fast_image_resize is the backend (no new DEC for the crate; stay on
   5.x); image::imageops::resize is the parity oracle. DEC-002: convert in/out of
   RGBA8 at the op boundary (like Invert), return via `with_pixels` — decode-once.
   DEC-005: register the op in `with_builtins`; do NOT edit the recipe parser for
   the op itself (only the shared param-error mapping, once). DEC-007: new typed
   variants flow through the existing `CliError::Recipe(_) => 1` mapping — do NOT
   touch the CLI.
5. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/data-model.md
   — the recipe schema + the worked `resize` step (`op="resize"`, `mode="max"`,
   `width=1200`, height omitted). Your `resize_recipe_round_trips` test mirrors
   this; `params()` must emit exactly these keys (no `height` for `max`). The
   architect pinned the per-mode param schema in this doc — do NOT change it.
6. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/constraints.yaml
   — single-image-library (fast_image_resize is a resize KERNEL, not a second
   pixel model — it converts in/out of image's RGBA8), decode-once-no-per-op-disk
   (apply is pure in-memory), no-new-top-level-deps-without-decision (DEC-008
   covers fast_image_resize — call it out in the PR), no-unwrap-on-recoverable-
   paths (map EVERY fast_image_resize error to OperationError::Apply), every-
   public-fn-tested, clippy-fmt-clean, test-before-implementation, untrusted-
   input-hardening (the oversize cap).
7. The SHIPPED code you extend (read the real signatures):
   src/operation/mod.rs      — the `Operation` trait, the `OperationParams`
                               `None`-only enum + its hand-written Serialize/
                               Deserialize (you REWRITE this), `Identity`/`Invert`
                               (migrate `params()` → `empty()`), `OperationError::
                               Apply { op, reason }`, the `#[cfg(test)] mod tests`
                               that assert `== OperationParams::None` (migrate to
                               `empty()`), and `Invert::apply` (the STRUCTURAL
                               TEMPLATE for `Resize::apply`).
   src/operation/registry.rs — `Constructor = fn(&OperationParams) -> Result<Box<
                               dyn Operation>, RegistryError>`; `RegistryError`
                               (add `InvalidParams`); `with_builtins` (register
                               `"resize"`); `build`; the build-path tests (update
                               `&OperationParams::None` → `&OperationParams::empty()`).
   src/recipe/mod.rs         — `RecipeStep { op, #[serde(flatten)] params }` and
                               `Recipe` (serde derives UNCHANGED); `RecipeError`
                               (add `InvalidOperation`); `build_pipeline` (change
                               the `.map_err(|_| UnknownOperation)` to MATCH the
                               registry error kind — see WHAT TO BUILD §E).
   src/pipeline/mod.rs       — `Pipeline` (UNCHANGED; Resize flows through `run`);
                               its tests reference `OperationParams::None` (migrate
                               to `empty()`).
   src/image/mod.rs          — `Image::pixels()` (→ `.to_rgba8()`),
                               `Image::with_pixels(self, DynamicImage)`,
                               `Image::from_parts`.
   Cargo.toml                — `[dependencies]` (add the fast_image_resize line).
   tests/recipe_round_trip.rs, tests/pipeline.rs, tests/common/mod.rs
                               — integration conventions + native fixtures; the
                               first two reference `OperationParams::None` (migrate).

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST (before editing ANY file)
═══════════════════════════════════════════════════════════════════════════

Do this BEFORE touching code so nothing ever lands on `main`:

  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-010-resize-operation-and-operation-params-mechanism

ALL code, test, and spec edits below happen ON THIS BRANCH. Never commit to
`main`. Confirm `git branch --show-current` prints
`feat/spec-010-resize-operation-and-operation-params-mechanism`, NOT `main`,
before committing.

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact)
═══════════════════════════════════════════════════════════════════════════

A. Cargo.toml — add the resize backend (DEC-008; no new DEC).

   A1. Under `[dependencies]`, add (always-on, NOT optional, NOT a feature):
         fast_image_resize = { version = "=5.5.0", features = ["image"] }
       Do NOT add it to `[features]`. Do NOT bump to 6.0. Do NOT add any other
       crate. (Verified: it resolves to the exact `image v0.25.10` pin and is
       pure-Rust — no -sys/cc/cmake/bindgen.)

B. src/operation/mod.rs — REWRITE OperationParams (DEC-014).

   B1. Replace the `OperationParams` `None`-only enum with a newtype over an
       ordered map, EXACTLY as the spec's "## Outputs" block specifies:
         pub struct OperationParams(std::collections::BTreeMap<String, toml::Value>);
       Derive `Debug, Clone, PartialEq` ONLY (NOT `Eq` — toml::Value holds floats
       and is not Eq). Add the methods from the spec: `empty()`, `from_map()`,
       `is_empty()`, `get_str()`, `get_u32()`, `get_f32()`, and `impl Default`
       (delegates to `empty()`).
   B2. REWRITE the hand-written serde impls to delegate to the inner map:
       - `Serialize`: iterate the BTreeMap into `serialize_map` — an EMPTY map
         emits ZERO keys (this keeps `op = "invert"` clean). Do NOT emit null or
         a wrapper key.
       - `Deserialize`: deserialize a `BTreeMap<String, toml::Value>` and wrap
         it. DROP the old "error on a non-empty map" branch — a non-empty map is
         now VALID (it is some op's params); per-op validation happens later in
         the constructor. This single change is what lets resize's keys survive
         the flatten boundary while invert stays empty.
   B3. Migrate `Identity::params()` and `Invert::params()` to return
       `OperationParams::empty()`. Update the SPEC-003 unit tests in this module
       that assert `== OperationParams::None` to `== OperationParams::empty()`.

C. src/operation/mod.rs — ADD the Resize Operation.

   C1. Private `enum ResizeMode { Max, Exact, Percent, Fit, Fill, Cover }`
       (derive `Debug, Clone, Copy, PartialEq, Eq`).
   C2. `pub struct Resize { mode: ResizeMode, width: Option<u32>,
       height: Option<u32>, percent: Option<f32> }`.
   C3. `pub fn Resize::from_params(params: &OperationParams)
       -> Result<Resize, RegistryError>` — parse + validate per the PINNED
       param-key schema:
         - `mode = params.get_str("mode")` → match the six literals; absent or
           unknown → `RegistryError::InvalidParams { op: "resize", reason }`.
         - `max`: require `width` (the long-edge cap N) via `get_u32`, > 0.
         - `exact`/`fit`/`cover`/`fill`: require `width` AND `height` (get_u32),
           both > 0.
         - `percent`: require `percent` (get_f32), > 0.0.
         - Any missing/wrong-typed/non-positive value → InvalidParams. NO panic.
         - Store ONLY the keys the mode uses (so `params()` reconstructs minimal).
   C4. `impl Operation for Resize`:
         - `name(&self) -> &'static str { "resize" }`
         - `params(&self) -> OperationParams` — rebuild a BTreeMap with `mode`
           (`toml::Value::String`) plus the mode's keys (`toml::Value::Integer(n
           as i64)` for dims; for `percent` use `toml::Value::Float` if it had a
           fractional part, else `Integer` — simplest correct: store what you
           parsed). Keep the set MINIMAL (no `height` for `max`). → `from_map`.
         - `apply(&self, img: Image) -> Result<Image, OperationError>` — see C5/C6.
   C5. `apply` — compute target `(tw, th)` per the EXACT six-mode math in the
       spec (round to nearest, then `.max(1)`), then ENFORCE the oversize cap on
       `(tw, th)` (and on the cover dims for `fill`): any edge > 50_000 OR area
       > 268_435_456 → `OperationError::Apply { op: "resize", reason }` BEFORE
       allocating. Then resize via the VERIFIED API (C6). For `fill`: resize to
       the COVER dims first, then `image::imageops::crop_imm(&out, x, y, W, H)
       .to_image()` with centered offsets (`x=(rw-W)/2`, `y=(rh-H)/2`, clamped
       ≥ 0), then `with_pixels`.
   C6. Use the VERIFIED fast_image_resize 5.5.0 block from the spec's Notes
       VERBATIM (it compiled against the repo's image v0.25.10):
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
             fast_image_resize::ResizeAlg::Convolution(
                 fast_image_resize::FilterType::Lanczos3,
             ),
         );
         resizer.resize(&src, &mut dst, &opts)
             .map_err(|e| OperationError::Apply { op: "resize", reason: e.to_string() })?;
         let out = image::RgbaImage::from_raw(dw, dh, dst.into_vec())
             .ok_or(OperationError::Apply { op: "resize", reason: "buffer/dim mismatch".into() })?;
       Map EVERY fast_image_resize error → `OperationError::Apply { op: "resize",
       reason: e.to_string() }`. NEVER unwrap/expect/panic.

D. src/operation/registry.rs — typed param error + register resize.

   D1. Add the `RegistryError` variant (thiserror):
         /// A constructor rejected its params (wrong/missing/out-of-range).
         #[error("invalid params for operation '{op}': {reason}")]
         InvalidParams { op: &'static str, reason: String },
   D2. In `with_builtins`, register:
         reg.register("resize", |p| Ok(Box::new(Resize::from_params(p)?)));
       (the `?` surfaces `RegistryError::InvalidParams`).
   D3. Update the existing `build("...", &OperationParams::None)` call sites in
       this module's tests to `&OperationParams::empty()`.

E. src/recipe/mod.rs — distinguish a param error from an unknown op.

   E1. Add the `RecipeError` variant (thiserror):
         /// An op name resolved but its params were invalid (DEC-014).
         #[error("invalid operation '{name}': {reason}")]
         InvalidOperation { name: String, reason: String },
   E2. Change `build_pipeline`'s `.map_err` to MATCH the registry error kind
       (do NOT collapse every error to UnknownOperation):
         let op = registry.build(&step.op, &step.params).map_err(|e| match e {
             RegistryError::Unknown { name } => RecipeError::UnknownOperation { name },
             RegistryError::InvalidParams { op, reason } =>
                 RecipeError::InvalidOperation { name: op.to_owned(), reason },
         })?;
       (Import `RegistryError` if not already in scope.) `RecipeStep`/`Recipe`
       serde derives stay UNCHANGED.

F. Migrate the remaining `OperationParams::None` references (compile fix).
   `src/pipeline/mod.rs` tests and `tests/recipe_round_trip.rs` /
   `tests/pipeline.rs` reference `OperationParams::None` — change each to
   `OperationParams::empty()`. (Search the whole repo for `OperationParams::None`
   and migrate ALL of them.)

G. DO NOT touch `src/cli/**` AT ALL. The new `RecipeError::InvalidOperation`
   flows through the existing `CliError::Recipe(_) => 1` mapping generically
   (confirm that arm exists in `src/cli/mod.rs` by READING — do not edit). If a
   `src/cli/` edit ever seems FORCED to make things compile, STOP and flag it in
   `## Build Completion` → Deviations and in your final report (it belongs in
   SPEC-011), then add a question to
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml
   rather than editing the CLI.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (make them pass)
═══════════════════════════════════════════════════════════════════════════

Implement EVERY test named in the spec's "## Failing Tests". Native in-memory
fixtures only (the `image` crate); no committed binaries. Build images with the
module's existing `make_image` helper in `src/operation/mod.rs`.

In src/operation/mod.rs `#[cfg(test)] mod tests` (use `super::*`):
  - params_empty_is_parameterless
  - resize_max_exact_dims            (100x50, max N=20 → 20x10)
  - resize_max_no_upscale            (40x30, max N=100 → 40x30)
  - resize_exact_exact_dims          (100x50, exact 33x77 → 33x77)
  - resize_percent_exact_dims        (100x50, percent 50 → 50x25)
  - resize_fit_exact_dims            (100x50, fit 40x40 → 40x20)
  - resize_fit_no_upscale            (30x20, fit 300x300 → 30x20)
  - resize_cover_exact_dims          (100x50, cover 40x40 → 80x40)
  - resize_cover_may_upscale         (20x10, cover 100x100 → 200x100)
  - resize_fill_center_crops_exact   (100x50, fill 40x40 → 40x40)
  - resize_fill_crop_is_centered     (center feature lands at cropped center ±1px)
  - resize_parity_within_tolerance   (64x48 → 32x24 vs image::imageops::resize
                                      Lanczos3; mean per-channel abs diff <= 6.0,
                                      tighten if comfortably lower; comment the #)
  - resize_oversize_is_typed_error   (exact 60_000x10 AND 20_000x20_000 →
                                      OperationError::Apply{op:"resize",..}; no panic)
  - resize_from_params_missing_mode  (→ RegistryError::InvalidParams{op:"resize",..})
  - resize_from_params_unknown_mode
  - resize_from_params_missing_dim   (exact, only width)
  - resize_from_params_nonpositive_dim (exact, width=0)
  - resize_params_round_trips_max    (from_params{mode="max",width=1200} →
                                      params() map == {mode:"max", width:1200},
                                      NO height)
  - invert_params_still_zero_keys    (Invert.params().is_empty())

  Build an `OperationParams` in tests via `OperationParams::from_map(BTreeMap)`
  with `toml::Value::String`/`Integer`/`Float` entries (e.g.
  `[("mode".to_string(), toml::Value::String("max".into())),
    ("width".to_string(), toml::Value::Integer(20))].into_iter().collect()`).

In src/operation/registry.rs `#[cfg(test)] mod tests`:
  - with_builtins_contains_resize
  - build_resize_with_valid_params       (name()=="resize")
  - build_resize_invalid_params_is_typed (build("resize",&empty) → InvalidParams)

In tests/recipe_round_trip.rs (reuse its conventions):
  - resize_recipe_round_trips                  (mode="max", width=1200, height
                                                omitted → PartialEq equal; TOML
                                                has op/mode/width, NO height key)
  - resize_invalid_params_is_invalid_operation (resize step missing mode →
                                                from_toml OK, build_pipeline →
                                                RecipeError::InvalidOperation{
                                                name:"resize",..}, NOT Unknown)
  - migrate this file's `OperationParams::None` refs to `empty()`

In tests/pipeline.rs (integration) — RECOMMENDED:
  - resize_runs_through_pipeline   (push Resize from_params{mode="exact",
                                    width=8,height=8} over a 16x16 fixture → 8x8)
  - migrate this file's `OperationParams::None` refs to `empty()`

The existing recipe/registry/pipeline tests (recipe_round_trips_through_toml,
serialized_toml_matches_schema, unknown_operation_is_typed_error,
registry_resolves_builtins_by_name, the pipeline order/halt tests, etc.) MUST
stay green after the rewrite. Run the FULL `cargo test`.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════

- The `resize` CLI command — clap `Commands::Resize`, a `run_resize` handler,
  the `--max`/`--exact`/`--fit`/`--fill`/`--cover`/`--percent` flags, or any
  `"WxH"`-STRING parsing. That is SPEC-011. Touch NOTHING under `src/cli/`.
  (If a CLI edit seems forced to compile, STOP and flag it — see §G.)
- Multi-input / `--out-dir` fan-out — SPEC-011.
- `thumbnail` / `shrink` / `convert` / `auto-orient` — later STAGE-003 specs.
- `rayon` or ANY parallelism — DEC-006 forbids it here.
- WebP / AVIF — fast-follow / feature-gated later (DEC-004).
- A second pixel library, or exposing the resize backend on the recipe surface.
- A new DEC (DEC-014 + DEC-008 already cover this spec) or a new top-level crate
  beyond fast_image_resize.
If you think a new RUNTIME crate or a new DEC is needed, STOP and add a question
to /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml
instead of inventing it.

═══════════════════════════════════════════════════════════════════════════
THE GATES (run from the repo root; ALL must pass before the PR)
═══════════════════════════════════════════════════════════════════════════

  cargo build
  cargo test
  cargo clippy -- -D warnings
  cargo fmt --check                              # `cargo fmt` to fix, then re-check

NOTE: there is NO `--features display` gate for SPEC-010 — resize does not touch
viuer. fast_image_resize is always-on, so the four standard gates above fully
cover it. Run exactly these four.

═══════════════════════════════════════════════════════════════════════════
WHEN DONE
═══════════════════════════════════════════════════════════════════════════

1. Fill in ONLY the spec's `## Build Completion` section (branch, PR, criteria
   met, deviations — INCLUDING any parity-tolerance number you settled on and
   the §G CLI-untouched confirmation, follow-ups, and the 3-question build
   reflection). Do NOT edit any other part of the spec body.
2. Append a build cost session entry to the spec front-matter `cost.sessions`
   (cycle: build, agent: claude-sonnet-4-6, interface: claude-code,
   tokens_total: null, estimated_usd: null, duration_minutes: <est>,
   recorded_at: 2026-06-15, notes: "subagent; cost not separately reported").
   Do NOT recompute cost.totals (ship does that).
3. Advance the cycle to verify by HAND-EDITING the spec front-matter `task.cycle`
   from `build` to `verify`. DO NOT run `just advance-cycle` or `just
   archive-spec` — they MIS-GLOB in this repo; the orchestrator does all other
   bookkeeping by hand. Only edit the spec's Build Completion section + the cost
   session + task.cycle.
4. Commit ON THE BRANCH (created in Step 0) with Conventional Commits, e.g.
   `feat(operation): resize op + generic operation-params mechanism (SPEC-010)`
   — a single commit covering Cargo.toml + operation + registry + recipe + tests
   + spec is fine; end EACH commit message with:
       Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
   (Confirm `git branch --show-current` prints
   `feat/spec-010-resize-operation-and-operation-params-mechanism`, NOT `main`,
   before committing.)
5. Mark build `[x]` in the timeline
   (/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-010-resize-operation-and-operation-params-mechanism-timeline.md).
   ACCURATE BOOKKEEPING: when you mark build `[x]`, write ONLY what is true at
   build time — say "PR #N opened" (with the real number). Do NOT write "merged",
   do NOT claim the PR is approved, and do NOT assert any post-merge fact. Verify
   and ship record those later.
6. Push the branch and open a PR on the `jysf/crustyimg` remote per AGENTS.md §13
   (one spec per branch / per PR):
   - PR title carries the spec id, e.g.
     `feat(operation): resize operation and operation params mechanism (SPEC-010)`.
   - PR body uses the §13 template — Summary; Spec metadata PROJ-001/STAGE-003/
     SPEC-010; Decisions referenced [DEC-014 (generic operation-params newtype;
     per-op validation), DEC-008 (fast_image_resize SIMD backend), DEC-002
     (op converts in/out of RGBA8, decode-once), DEC-005 (registry round-trip),
     DEC-007 (typed errors → exit codes)]; Constraints checked with one-line
     evidence each (single-image-library, decode-once-no-per-op-disk,
     no-new-top-level-deps-without-decision [DEC-008 covers fast_image_resize],
     no-unwrap-on-recoverable-paths, every-public-fn-tested, clippy-fmt-clean,
     test-before-implementation, untrusted-input-hardening [oversize cap]);
     New decisions: list "DEC-014 — operation-params mechanism (emitted during
     design; first implemented here)".
   - End the PR body with the Claude Code generated-with footer.

Remember: build edits to the spec are LIMITED to `## Build Completion` (plus the
front-matter cost session + task.cycle). Verify/ship bookkeeping lands on main
later, not on this branch.
```
