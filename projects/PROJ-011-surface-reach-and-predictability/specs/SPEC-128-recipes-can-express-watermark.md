---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-128
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-050
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5     # usually same Claude, different session
  created_at: 2026-09-05

references:
  decisions:
    - DEC-031
    - DEC-064
    - DEC-005
    - DEC-002
    - DEC-099
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
    - every-public-fn-tested
    - decode-once-no-per-op-disk
  related_specs:
    - SPEC-127
    - SPEC-031

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-050's <capability>". Optional; null is acceptable.
value_link: >
  STAGE-050's structural item. A recipe cannot express `watermark` today — not because
  the op is missing, but because the registry seam cannot construct an operation whose
  parameters name a FILE. Widening that seam is what lets the registry take its first
  parameter-rich op, and the `.cube` LUT op is the known second customer.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4) — backfilled by the build session, which found
        it missing on starting SPEC-128's build.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 74887145
      estimated_usd: 26.72
      duration_minutes: null
      recorded_at: 2026-09-05
      notes: >
        Interactive session (not an orchestrated Agent call), measured from the session's own
        transcript JSONL, deduped by `.message.id` (217 unique ids), taking input/cache_creation/
        cache_read from the group and MAX output_tokens per id (summing every line over-counts;
        the first line's output under-counts). Priced by component at Sonnet $3/$15 per MTok,
        cache_creation x1.25, cache_read x0.10 — never a flat rate on tokens_total.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-128: recipes can express watermark

## Context

⚠ **The backlog called this "`watermark` becomes a registry operation." Reading the code shows that
is the wrong description, and the real problem is one level down.**

`Watermark` **is already an `Operation`** — `src/operation/mod.rs:1118`, with `impl Operation for
Watermark` at `:1161` and a `params()` that already serialises the overlay path for round-trip.
What it is not is **registered**. And it is unregistered *deliberately*, for a reason
[DEC-031](../../../decisions/DEC-031-multi-image-operation-overlay-loaded-at-io-boundary.md) states
outright:

> Watermark is **NOT registered in `OperationRegistry::with_builtins()`** in this iteration,
> because the registry constructor is a pure `fn(&OperationParams) -> Result<Box<dyn Operation>>`
> that cannot (and must not) load the overlay file.

So the gap is **structural, not clerical**. Three facts, read from the code:

| fact | where |
|---|---|
| `with_builtins()` registers exactly **4** ops — `identity`, `invert`, `resize`, `auto-orient` | `operation/registry.rs:78-85` |
| The constructor type is `fn(&OperationParams) -> Result<Box<dyn Operation>>` — **no IO, no context** | `operation/registry.rs` |
| `Recipe::build_pipeline(&self, registry)` takes **only** the registry | `recipe/mod.rs:361` |

And `src/operation/**` compiles for **wasm32** (DEC-064), so it cannot reach the filesystem even if
the signature allowed it.

**Three operations need the same thing, which is why this is a seam and not a special case:**

- **image watermark** — needs the overlay decoded from a path (`Watermark { overlay: DynamicImage }`)
- **text watermark** — `text::render_text(font_bytes, …)` is pure, but the **font bytes** come from
  `--font` (a file) or the bundled default
- **the `.cube` LUT op** — a decided "take" (STAGE-050), needs its cube file

## Goal

A recipe can express `watermark` in both modes, round-trip losslessly, and run identically through
`apply`, `build` and — where the assets can be supplied — `wasm::transform`, **without
`src/operation/**` gaining a filesystem dependency.**

## The design calls — settled here

### Call 1 — widen the seam by RESOLVING assets before `build_pipeline`, not by giving the registry IO

Three shapes were considered; the spec picks the second.

- **Pass a loader into `build_pipeline`.** Rejected: it changes the signature of the one function
  `apply`, `build` **and** `wasm::transform` all call, and pushes an IO-shaped trait into an engine
  module whose freedom from IO is the property DEC-064 exists to protect.
- **✅ Chosen — resolve at the recipe IO boundary.** The CLI walks the recipe's steps *before*
  `build_pipeline`, loads any file a step names, and hands the bytes to the registry alongside the
  params. `build_pipeline` and the constructor stay pure; the loading happens exactly where DEC-031
  already said it belongs ("the recipe loader (the IO boundary for recipes)").
- **Two-phase "unbound op" that gets bound later.** Rejected: it makes every op's lifecycle more
  complex to serve three of them, and an unbound op is a state in which `apply()` is invalid — a
  worse trade than a resolve step.

⚠ **The bytes must NOT round-trip into the params.** `to_toml` must keep emitting the *path*, never
the loaded bytes, or every recipe becomes enormous and `from_toml(to_toml(r)) == r` breaks. **This
is the single highest-consequence line in the spec** — it is the same shape as SPEC-127's
`to_toml`-must-emit-`"1"` guard, and it needs its own test.

### Call 2 — a recipe naming a missing asset fails at BUILD-PIPELINE time, not per-input

`apply`/`build` already probe the pipeline once, before touching any input, so a bad recipe exits 1
rather than failing per-input with exit 6. An unreadable overlay is a bad recipe, not a bad input.
⚠ **Drive this**: the failure must arrive before the first output is written, on a batch of ≥2.

### Call 3 — `wasm::transform` REFUSES a recipe whose steps need assets, with a typed error

The wasm surface has no filesystem. Silently dropping the watermark step would be the worst
outcome — the caller gets an unwatermarked image and no signal. ⚠ Refuse with a typed error naming
the step and the asset. Supplying assets over the wasm boundary is **explicitly out of scope**.

### Call 3b — ⚡ text mode's `params()` is WRONG today and must change

Found while checking what a watermark step would actually look like in TOML. `watermark_overlay`
(`src/cli/ops.rs:1220-1262`) returns the rendered pixels **plus a label**, and for `--text` **the
label is the text itself** (`Ok((… , text.to_owned()))`). `Watermark::params()` then writes that
label under the key **`image`**.

So a text watermark serialises today as:

```toml
[[step]]
op = "watermark"
image = "© crustyimg"      # ← the TEXT, in the field that means FILE PATH
```

Round-tripping that would try to **load a file named after the text**. It has never mattered
because watermark is unregistered and no recipe could carry it — this spec is what makes it matter.

**Rule: the two modes get distinct, non-overlapping keys.** Image mode keeps `image = "<path>"`.
Text mode emits `text`, plus `font` (path, optional — the bundled default when absent), `size` and
`color`, and **must not emit `image` at all**. A step carrying both, or neither, is a typed
`InvalidOperation` at build-pipeline time — the same XOR the CLI already enforces between
`--image` and `--text`.

⚠ **`Watermark` must therefore carry enough to round-trip text mode**, which its current fields
cannot: `overlay_path: String` is a single slot doing double duty. Widening that struct is part of
this spec, not a follow-up.

### Call 4 — this spec registers WATERMARK ONLY; the LUT op is a design constraint, not a deliverable

STAGE-050 already says the seam must be designed knowing a second customer exists. It must not be
built here. **The AC that enforces this is AC-7:** a second, asset-needing op must be registrable
in a test **without changing the seam** — a fixture op, not the real LUT.

## Acceptance Criteria

- [ ] **AC-1.** A recipe with a `watermark` step (image mode) round-trips losslessly —
      `from_toml(to_toml(r)) == r` — and the emitted TOML contains the **path**, never overlay bytes.
- [ ] **AC-2.** The same for text mode — emitting `text`/`font`/`size`/`color` + placement, and
      **never `image`** (Call 3b). ⚠ Assert the round-tripped step still renders the same pixels,
      not merely that the TOML parses: today `params()` puts the TEXT under `image`, so a test that
      only checks parseability would pass on the broken behaviour.
- [ ] **AC-2b.** A `watermark` step carrying **both** `image` and `text`, or **neither**, is a
      typed error at build-pipeline time — matching the CLI's existing XOR.
- [ ] **AC-3.** `apply --recipe` with a watermark step produces **byte-identical** output to the
      equivalent `watermark` CLI invocation, at **1 input and at N inputs**, as two tests.
- [ ] **AC-4.** `apply` and `build` agree byte-for-byte on the same watermark recipe (the SPEC-126
      property, extended to an asset-bearing op).
- [ ] **AC-5.** A recipe naming an unreadable overlay/font fails **before any output is written**,
      driven on a batch of ≥2 inputs, with a typed error and the documented exit code (Call 2).
- [ ] **AC-6.** `wasm::transform` on an asset-bearing recipe returns a **typed error naming the
      step**, never a silently unwatermarked image (Call 3).
- [ ] **AC-7.** ⚡ **The seam takes a SECOND asset-bearing op with no seam change.** A fixture op is
      registered in a test using the same resolve path; the test asserts `src/operation/registry.rs`
      and the resolve signature are untouched by it.
- [ ] **AC-8.** `src/operation/**` gains **no** filesystem dependency — `just wasm-check` passes and
      a grep for `std::fs`/`Image::load` under `src/operation/` stays empty.
- [ ] **AC-9.** Negative control, one revert per independent condition (Calls 1, 2, 3); each flips
      only its own tests. Evidence is the behavioural flip, never a hash.
- [ ] **AC-10.** **Nothing else changes bytes.** Every pixel-lane verb and a watermark-free recipe
      through `apply`/`build` produce output byte-identical to `main`, with a positive control.
      ⚠ **Sweep the verbs that reach `run_pixel_op`, not just the ones in `ops.rs`** — SPEC-127's
      narrowing was wrong because `run_convert`/`run_optimize`/`run_web` all reach it.
- [ ] **AC-11.** Clean matrix — default, `--no-default-features`, `--features webp-lossy`, fresh
      `CARGO_TARGET_DIR` each, sequential; clippy + `fmt --check` each; plus `just wasm-check`.

## Failing Tests

Written at design, made to pass at build. **All confirmed RED against `main` first**, baseline recorded.

- `tests/recipe_watermark.rs::watermark_recipe_round_trips_with_path_not_bytes` — AC-1, the guard.
- `tests/recipe_watermark.rs::text_watermark_round_trips_without_an_image_key` — AC-2, the
  guard on Call 3b's defect.
- `tests/recipe_watermark.rs::watermark_step_requires_exactly_one_source` — AC-2b.
- `tests/recipe_watermark.rs::missing_overlay_fails_before_any_output` — AC-5.
- `tests/apply_batch.rs::apply_watermark_recipe_matches_cli_at_one_input` — AC-3.
- `tests/apply_batch.rs::apply_watermark_recipe_matches_cli_at_n_inputs` — AC-3, second arity.
- `tests/apply_batch.rs::apply_and_build_agree_on_watermark_recipe` — AC-4.
- `tests/wasm_roundtrip.rs::transform_refuses_asset_bearing_recipe` — AC-6.
- `tests/registry_seam.rs::a_second_asset_op_registers_without_seam_change` — AC-7.

## Implementation Context

**Read first:** DEC-031 (why watermark is unregistered — it is the premise of this spec), DEC-064
(the engine/shell split and the wasm constraint), DEC-005 (recipe round-trip), DEC-002
(decode-once), DEC-099 (SPEC-127's precedence chain, which this must not disturb).

**The seam today.** `OperationRegistry::with_builtins()` (`operation/registry.rs:78`) registers four
ops through `Constructor = fn(&OperationParams) -> Result<Box<dyn Operation>>`. `Resize::from_params`
(`operation/mod.rs:626`) is the only parameter-rich precedent — read it before designing the
watermark constructor.

**What already exists and must not be rebuilt:** `Watermark::new` takes decoded overlay pixels and
placement; `params()` already serialises path + placement; `text::render_text` is already pure and
takes font bytes. `run_watermark` in `src/cli/ops.rs:~1247-1302` is the existing IO boundary and
shows exactly which bytes have to be resolved.

**Where the resolve step goes:** in `src/cli/` (and `src/build/` if it needs its own), *before*
`Recipe::build_pipeline`. Both `apply` and `build` must use the **same** resolver — the SPEC-126
lesson is that two paths resolving the same thing separately is how they drift.

⛔ **Byte-changing only for recipes that opt in.** A watermark-free recipe must be byte-identical
(AC-10). It still batches into PROJ-011's single lockfile migration — **do not bump the version, do
not cut a release.**

## Notes for the Implementer

- 📌 **DEC-100 is reserved.** Highest on `main` is DEC-099. Use the **block-list** `affected_scope`
  form — `scripts/decisions-audit.sh` silently drops inline arrays (filed, PROJ-013 STAGE-047).
- ⚡ **Write `docs/api-contract.md` and `docs/data-model.md` in the same change.** Both describe the
  recipe schema, and SPEC-127's build did this correctly — match it.
- ⚠ **Size expectation, stated honestly:** the code here is **M**; the verification is heavier than
  the code, as it was for SPEC-127. Budget **~250 exchanges**, and push a WIP commit as soon as it
  compiles.
- ⚠ **Do not run a backgrounded feature build while editing source for a revert** — SPEC-127's build
  contaminated a leg that way and had to re-run it from a clean tree.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout. A
  piped command reports the **pipe's** exit code — redirect and read `$?`. Never poll CI.

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-128-recipe-watermark`
- **PR:** opened against `main` (see PR description / URL in the build session's final report).
- **All acceptance criteria met?** yes (AC-1 through AC-11; see `DEC-100`'s `## Validation` for the
  test-by-test mapping and the AC-9/AC-10/AC-11 measured results).
- **New decisions emitted:**
  - `DEC-100` — `watermark` registers via a resolve-at-recipe-IO-boundary seam; the registry stays
    file-free.
- **Files this diff touches** — from `git diff --name-only main`, not recall:
  - `src/operation/mod.rs` — `OperationParams` gains a resolved-bytes side channel
    (`set_resolved_bytes`/`resolved_bytes`) plus `get_bool`; the module's allowed-dependency doc
    widened to `crate::text` (Call 3b); `Watermark` replaces its single `overlay_path: String` with
    a `WatermarkSource` enum (`Image{path}` / `Text{text,font_path,size,color}`); `new` split into
    `new_image`/`new_text`; a new `from_params` constructor (image mode decodes resolved bytes, text
    mode renders via `crate::text`, falling back to the bundled font when `font` is absent); `params()`
    rewritten to emit each mode's distinct key set; a `color_to_hex` helper; the existing
    `watermark()` test helper repointed to `new_image`.
  - `src/operation/registry.rs` — `OperationRegistry` gains an `asset_keys` map,
    `register_with_assets`, and the `asset_keys(name)` query; `with_builtins()` registers
    `"watermark"` via `register_with_assets("watermark", ..., &["image", "font"])`; four new tests.
  - `src/cli/ops.rs` — `watermark_overlay` returns a new `ResolvedOverlay` enum (image path / text +
    rendering flags) instead of the old flattened `(DynamicImage, String)` label; `run_watermark`
    dispatches to `Watermark::new_image`/`new_text` accordingly.
  - `src/cli/common.rs` — new `resolve_recipe_assets(recipe, registry)`: clones the recipe, reads
    the file named by each asset-bearing step's declared keys, attaches the bytes via
    `set_resolved_bytes`. The one resolver both `apply` and `build` call.
  - `src/cli/mod.rs` — new `CliError::RecipeAssetUnreadable` variant (exit 1) + its `code()` mapping.
  - `src/cli/optimize.rs` — `run_apply` builds the registry once and calls `resolve_recipe_assets`
    immediately after `load_recipe`, before the terminal-`optimize` branch and the upfront probe
    (removing the old duplicate `OperationRegistry::with_builtins()` call further down).
  - `src/cli/build.rs` — `prepare_target` calls `resolve_recipe_assets` right before its existing
    `build_pipeline` probe, so `PreparedTarget.recipe` (and therefore every `encode_one` call
    downstream) is already resolved.
  - `src/wasm.rs` — `transform` queries `OperationRegistry::asset_keys` for every step via a new
    `first_unresolvable_asset` helper and refuses with a typed, step-naming error before
    `build_pipeline` when any step needs a file it cannot resolve.
  - `docs/api-contract.md` — the `watermark` entry documents recipe support: the resolve-before-write
    behavior, the new exit-1 error, and the wasm refusal; drops the stale `ab_glyph` mention (SPEC-044
    already swapped the rasterizer) and the stale "not recipe-round-trippable until STAGE-005" line.
  - `docs/data-model.md` — the worked-example intro now names five registry ops (not four); the "not
    a recipe step" callout for watermark is replaced with a new `watermark` step param-key section and
    a two-step (image + text) TOML example.
  - `tests/recipe_watermark.rs` (new) — AC-1, AC-2, AC-2b, AC-5 (four tests).
  - `tests/registry_seam.rs` (new) — AC-7 (one test, a fixture asset-bearing op).
  - `tests/apply_batch.rs` — three new tests (AC-3 ×2 arities, AC-4).
  - `tests/wasm_roundtrip.rs` — two new tests (AC-6: the refusal itself, and a text-only step that
    needs no resolution and must still run).
  - `decisions/DEC-100-watermark-registers-via-resolve-at-io-boundary-seam.md` (new) — the decision
    record.
  - `projects/.../specs/SPEC-128-recipes-can-express-watermark.md` — this spec's own
    `## Build Completion`, cycle advance, and cost entry.
  - `projects/.../specs/SPEC-128-recipes-can-express-watermark-timeline.md` — the build mark.
- **Deviations from spec:** none from the four settled design calls. One correction to the build
  prompt itself: it says "write the eight failing tests," but the spec's own `## Failing Tests`
  section lists **nine** (`watermark_recipe_round_trips_with_path_not_bytes`,
  `text_watermark_round_trips_without_an_image_key`, `watermark_step_requires_exactly_one_source`,
  `missing_overlay_fails_before_any_output`, `apply_watermark_recipe_matches_cli_at_one_input`,
  `apply_watermark_recipe_matches_cli_at_n_inputs`, `apply_and_build_agree_on_watermark_recipe`,
  `transform_refuses_asset_bearing_recipe`, `a_second_asset_op_registers_without_seam_change`). Built
  all nine — the spec is the contract, and the prompt's count was simply off by one.
  One judgment call the spec left open, recorded in `DEC-100` rather than quietly decided: the
  wasm refusal (Call 3) is checked at the constructor's OWN fallback error path too (a
  `RegistryError::InvalidParams` fires there regardless of the dedicated refusal, since resolved
  bytes are simply absent on wasm) — so `transform_refuses_asset_bearing_recipe` had to assert the
  DEDICATED wording ("no filesystem" / "out of scope"), not just that an `Err` naming "watermark" and
  "image" came back, or a revert of Call 3 alone would not have been a real negative control.
- **Follow-up work identified:** the `build` cache key does not hash a watermark asset's own file
  CONTENT — only the recipe's `to_toml()` (the path). Editing `logo.png` in place without touching
  the recipe/manifest is invisible to `build`'s cache. No AC names this; filed to STAGE-050's backlog
  below rather than fixed here (its own design surface: which hash, computed where, at what
  per-target cost).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing genuinely ambiguous, but one design step needed working out rather than being handed to
   me: HOW resolved bytes get from the CLI's resolver to a pure `fn(&OperationParams) -> Result<...>`
   registry constructor without widening `Constructor`'s signature or letting bytes leak into
   `to_toml`. The spec says "hands the bytes to the registry alongside the params" but doesn't say
   *how*; the answer (a second, non-serialized field on `OperationParams`, populated by
   `set_resolved_bytes`/read by `resolved_bytes`) is what makes AC-7's fixture-op test possible
   without touching `registry.rs` further, so I'm fairly confident it's the intended shape, but the
   spec text alone doesn't pin it down.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-031 and DEC-064 were exactly what was needed; STAGE-050's backlog entry naming the
   `.cube` LUT op as the seam's second customer directly motivated AC-7's shape (a fixture, not the
   real op).

3. **If you did this task again, what would you do differently?**
   — Nothing structural. I'd write `transform_refuses_asset_bearing_recipe`'s STRONGER assertion (the
   dedicated-wording check) on the first pass instead of tightening it after noticing, mid-AC-9, that
   the weaker version couldn't discriminate Call 3's revert from the constructor's own fallback error
   — a smaller version of the same "a claim that a test isn't vacuous needs driving too" lesson this
   repo has hit before.
