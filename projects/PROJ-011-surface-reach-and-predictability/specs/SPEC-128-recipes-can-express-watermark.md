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
  sessions: []
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

### Call 4 — this spec registers WATERMARK ONLY; the LUT op is a design constraint, not a deliverable

STAGE-050 already says the seam must be designed knowing a second customer exists. It must not be
built here. **The AC that enforces this is AC-7:** a second, asset-needing op must be registrable
in a test **without changing the seam** — a fixture op, not the real LUT.

## Acceptance Criteria

- [ ] **AC-1.** A recipe with a `watermark` step (image mode) round-trips losslessly —
      `from_toml(to_toml(r)) == r` — and the emitted TOML contains the **path**, never overlay bytes.
- [ ] **AC-2.** The same for text mode (`text`, `font`, `size`, `color` + placement).
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
- `tests/recipe_watermark.rs::text_watermark_recipe_round_trips` — AC-2.
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
