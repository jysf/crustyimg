---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-100
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-09-05
supersedes: null
superseded_by: null

affected_scope:
  - src/operation/mod.rs
  - src/operation/registry.rs
  - src/cli/common.rs
  - src/cli/ops.rs
  - src/cli/mod.rs
  - src/cli/optimize.rs
  - src/cli/build.rs
  - src/wasm.rs
  - docs/api-contract.md
  - docs/data-model.md

tags:
  - recipe
  - registry
  - watermark
  - wasm
  - seam
  - dec-031
  - dec-064
---

# DEC-100: `watermark` registers via a resolve-at-recipe-IO-boundary seam; the registry stays file-free

## Decision

`watermark` is now registered in `OperationRegistry::with_builtins()`. The registry itself gained
**no** filesystem access — it still constructs operations from a pure `fn(&OperationParams) ->
Result<Box<dyn Operation>, RegistryError>` (DEC-031's constraint holds unchanged). Four things,
together, are what make this possible:

1. **`OperationRegistry::register_with_assets(name, ctor, asset_keys)`** — a second registration
   entry point alongside `register`, declaring which of an op's param KEYS name a file (`watermark`:
   `["image", "font"]`). `OperationRegistry::asset_keys(name)` is the read side, returning `&[]` for
   any op (or unknown name) with none. Both are pure — no IO, string slices only.
2. **`OperationParams` gained a resolved-bytes side channel** (`set_resolved_bytes`/`resolved_bytes`),
   a second field the struct's hand-written `Serialize` impl never touches. `to_toml` keeps emitting
   only the path a step named; the bytes never round-trip.
3. **A new native-only resolver, `cli::common::resolve_recipe_assets(recipe, registry)`**, called
   ONCE by both `run_apply` (before its terminal-`optimize` branch and its upfront probe) and
   `prepare_target` (before its own probe) — the recipe IO boundary DEC-031 always said this belonged
   at, now actually built. It clones the recipe, and for each step whose op declares asset keys,
   reads the named file and attaches the bytes via `OperationParams::set_resolved_bytes`. A missing
   or unreadable file is `CliError::RecipeAssetUnreadable` (exit 1) — surfaced before either caller
   touches a single input.
4. **`Watermark::from_params`** (the registry constructor) reads the resolved bytes back via
   `OperationParams::resolved_bytes`, decodes the overlay (image mode) or renders the text (text
   mode, needing no resolution at all when `font` is absent — the bundled default is compiled in),
   and enforces the same `image` XOR `text` rule the CLI already had, as a typed
   `RegistryError::InvalidParams` → `RecipeError::InvalidOperation` (exit 1).

`wasm::transform` cannot resolve anything (no filesystem), so it queries the SAME
`OperationRegistry::asset_keys` before calling `build_pipeline` and refuses with a typed,
step-naming error when a step needs an asset key it cannot supply — never a silently unwatermarked
image.

Call 3b (found during design): `Watermark`'s single `overlay_path: String` field could not express
text mode's identity — `params()` used to write the raw TEXT under the `image` key. `Watermark` now
carries a `WatermarkSource` enum (`Image { path }` / `Text { text, font_path, size, color }`); the two
modes emit distinct, non-overlapping TOML keys, and text mode never emits `image` at all.

## Context

DEC-031 registered the fact that `watermark` could not go into `with_builtins()` because "the
registry constructor is a pure `fn(&OperationParams) -> Result<..>` that cannot (and must not) load
the overlay file," and named the recipe loader as the future IO boundary. STAGE-050 asked for that
seam, generalized enough for a SECOND parameter-rich, asset-bearing op (the `.cube` LUT, a decided
but unscheduled STAGE-050 backlog item) to register through it with no further change to
`registry.rs` — verified directly by `tests/registry_seam.rs`, using a fixture op, not the real LUT
(Call 4, explicitly out of scope here).

## Alternatives Considered

### Call 1 — where does resolution happen

- **Pass a loader/IO trait into `build_pipeline`.** Rejected: changes the signature of the one
  function `apply`, `build`, AND `wasm::transform` all call, and pushes an IO-shaped trait into an
  engine module whose freedom from IO (DEC-064, the wasm boundary) is the property being protected.
- **Two-phase "unbound op" bound later.** Rejected: makes every op's lifecycle more complex to serve
  three ops, and an unbound op is a state in which `apply()` is invalid.
- **Chosen: resolve at the recipe IO boundary, before `build_pipeline`, via a side channel on
  `OperationParams`.** `build_pipeline`'s signature never changes; `Recipe`/`OperationRegistry` stay
  exactly as pure as before. The resolved recipe is a transient clone used only for this one
  construction — never serialized, never compared for round-trip equality.

### Call 2 — where the failure surfaces

- **Resolve lazily, per input (inside `encode_one`).** Rejected: every input in a batch sharing one
  bad recipe would then fail INDEPENDENTLY through the existing per-input error path, aggregating
  into `CliError::PartialBatch` (exit 6) — implying some inputs could have succeeded where others
  failed, which is false for a recipe-level asset failure. Verified as the negative control for this
  call (see Build Completion): reverting to a lazy per-input resolve flips
  `missing_overlay_fails_before_any_output`'s expected exit code from 1 to 6.
- **Chosen: resolve ONCE, upfront — reusing the exact spot `apply`/`build` already probe
  `build_pipeline` at, before touching any input** (DEC-015's existing multi-input contract). A bad
  asset is a bad recipe, not a bad input: `CliError::RecipeAssetUnreadable` → exit 1, matching every
  other recipe/operation error, never exit 3 (which names the PRIMARY input) or 6.

### Call 3 — the wasm surface

- **Let `build_pipeline` fail on its own (no dedicated check).** Rejected even though it still
  produces an `Err` (the constructor's "not resolved" `InvalidParams` fires regardless): the message
  is an internal-error phrasing aimed at a resolver bug, not a caller-facing explanation of WHY the
  wasm surface specifically cannot do this. `tests/wasm_roundtrip.rs::transform_refuses_asset_bearing_recipe`
  asserts the dedicated wording ("no filesystem" / "out of scope") precisely so a revert of this
  check is a real, assertable behavioral flip, not just a message the fallback error also happens to
  satisfy on the surface (it names "watermark" and "image" either way).
- **Silently skip an asset-bearing step.** Rejected outright per the spec: the caller gets an
  unwatermarked image and no signal — the worst outcome.
- **Chosen: query `OperationRegistry::asset_keys` for every step before `build_pipeline`, and refuse
  with a typed error naming the step index, op, and asset key.** Supplying assets over the wasm
  boundary is explicitly out of scope. A text-only step with no `font` key needs nothing resolved and
  runs unaffected — verified by `transform_runs_a_text_only_watermark_with_no_font_key`.

### Call 3b — `Watermark`'s struct shape

- **Keep the single `overlay_path: String` field, add a boolean "is this text" flag.** Rejected: still
  conflates the two modes' identity into one slot and doesn't fix the actual defect (the TEXT written
  under the `image` key) any more cleanly than a proper enum.
- **Chosen: `WatermarkSource::{Image{path}, Text{text, font_path, size, color}}`.** Distinct,
  non-overlapping TOML key sets per mode, matching the CLI's existing `--image`/`--text` XOR exactly.

## Consequences

- **Positive:** `apply --recipe`, `build`, and (for text-only steps) `wasm::transform` can all express
  `watermark`. The registry seam is proven, not just asserted, to generalize to a second asset-bearing
  op (AC-7). `Watermark::params()` is now correct for text mode, which it never was before (Call 3b).
- **Byte-changing, but only for recipes that opt in.** A watermark-free recipe, and every other verb,
  are untouched — verified by AC-10's sweep (see Build Completion for the corpus). Batches into
  PROJ-011's single lockfile migration; no version bump, no release cut here.
- **The build cache key does not (yet) hash asset file CONTENT.** `target_recipe_hash` hashes the
  recipe's own `to_toml()` output — which, by design, is only the PATH a watermark step names, never
  the file's bytes. So editing `logo.png` in place (same path, different pixels) without touching the
  recipe/manifest is invisible to `build`'s cache: a stale, pre-edit watermark could be served from
  cache. This is a real gap, out of scope for this spec (no AC names it), and is filed to STAGE-050's
  backlog rather than fixed here — the fix (folding a content hash of each resolved asset into the
  cache key) has its own design surface (which hash, computed where, at what cost per target) that
  deserves its own spec rather than a bolt-on here.
- **`crate::operation::mod`'s allowed-dependency list widened to include `crate::text`** (Call 3b) —
  `crate::text` is itself file-free and already wasm32-safe, so this does not reopen the filesystem
  question DEC-064/AGENTS §11 protect; `src/operation/**` still has zero `std::fs`/`Image::load` calls
  (AC-8, `just wasm-check` green).

## Validation

- **AC-1/AC-2** (round-trip, path not bytes / no `image` key in text mode; pixel-identity assertion
  for text mode): `tests/recipe_watermark.rs::watermark_recipe_round_trips_with_path_not_bytes`,
  `::text_watermark_round_trips_without_an_image_key`.
- **AC-2b** (XOR is a typed build-time error): `tests/recipe_watermark.rs::watermark_step_requires_exactly_one_source`.
- **AC-3** (byte-identical to the CLI verb, 1 and N inputs):
  `tests/apply_batch.rs::apply_watermark_recipe_matches_cli_at_one_input`, `::apply_watermark_recipe_matches_cli_at_n_inputs`.
- **AC-4** (`apply`/`build` agree): `tests/apply_batch.rs::apply_and_build_agree_on_watermark_recipe`.
- **AC-5** (fails before any output, batch ≥2, exit 1 not 3/6): `tests/recipe_watermark.rs::missing_overlay_fails_before_any_output`.
- **AC-6** (wasm refusal, typed + step-naming; a resolution-free text step still runs):
  `tests/wasm_roundtrip.rs::transform_refuses_asset_bearing_recipe`, `::transform_runs_a_text_only_watermark_with_no_font_key`.
- **AC-7** (seam generalizes to a fixture second op, no further `registry.rs` change):
  `tests/registry_seam.rs::a_second_asset_op_registers_without_seam_change`.
- **AC-8** (`src/operation/**` stays file-free): `just wasm-check` green; `/usr/bin/grep -rn
  'std::fs\|Image::load' src/operation/` empty (see Build Completion for the exact command run).
- **AC-9** (negative controls, one revert per independent condition — Calls 1/2/3): see Build
  Completion for the exact reverts and the observed behavioral flips.
- **AC-10** (nothing else changes bytes, positive control included) and **AC-11** (clean matrix): see
  Build Completion.

## References

- Related specs: **SPEC-128** (this decision's spec), **SPEC-029**/**SPEC-030** (the original
  `watermark` op, image + text modes), **SPEC-127**/DEC-099 (the precedence-chain precedent this must
  not disturb), **SPEC-126**/DEC-098 (`apply`/`build` sharing one resolver — the lesson this spec's
  Call 1 reapplies).
- Related decisions: **DEC-031** (why watermark was unregistered — the premise this closes),
  **DEC-064** (the wasm/native target split and the file-free constraint on `src/operation/**`),
  **DEC-005** (recipe round-trip through the registry), **DEC-002** (the `Operation` trait / registry
  extension point).
- Code: `src/operation/registry.rs` (`register_with_assets`, `asset_keys`), `src/operation/mod.rs`
  (`OperationParams::{set_resolved_bytes,resolved_bytes}`, `Watermark`, `WatermarkSource`),
  `src/cli/common.rs` (`resolve_recipe_assets`), `src/cli/mod.rs` (`CliError::RecipeAssetUnreadable`),
  `src/wasm.rs` (`first_unresolvable_asset`).
- Stage: `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-050-recipe-reach.md`.
