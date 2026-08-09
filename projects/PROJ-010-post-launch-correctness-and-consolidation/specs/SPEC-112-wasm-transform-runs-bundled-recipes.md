---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-112
  type: bug                        # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: critical
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-010
  stage: STAGE-040
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5     # build on Sonnet: one call site, the helper
                                   # exists, and the design question is already
                                   # answered by DEC-087's precedent.
                                   # Verify stays on Opus.
  created_at: 2026-08-09

references:
  decisions:
    - DEC-005
    - DEC-087
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-072
    - SPEC-085
    - SPEC-111

value_link: >
  STAGE-040's precondition for the 0.7.0 cut: make the README's claim about
  `transform()` true before that README renders on the crates.io crate page.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Drove `transform`'s exact
        call chain natively against all three bundled recipes via a throwaway
        test, and confirmed from `parse_format` that `out_format` can never be
        `auto`.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-112: `wasm::transform` runs the bundled recipes

## Context

SPEC-111 fixed `build`'s inability to run bundled recipes and, in DEC-087, named
`wasm::transform` as carrying the identical defect — deliberately out of scope, on the
grounds that the shipped demo never reaches it. **That reasoning was right about the demo
and wrong about the README.**

`README.md:34–36` — the launch front door, which renders on the crates.io crate page:

> Pipelines are recipes: tune one with `edit --save-recipe`, or **start from a bundled
> `web`/`gallery`/`product`**, then replay it in parallel across a batch with
> `apply --recipe`. **The same recipe TOML runs in the browser demo too, via the wasm
> `transform()` binding.**

It does not. `transform` (`src/wasm.rs:158`) calls `recipe.build_pipeline(...)` with no
strip, and every bundled recipe ends with the reserved terminal `optimize` marker.

### Driven, 2026-08-09

`transform`'s exact call chain — `bundled::resolve(name)` → `Recipe::from_toml` →
`build_pipeline(&OperationRegistry::with_builtins())` — run natively against all three
shipped recipes:

| bundled recipe | result |
|---|---|
| `web` | `Err("unknown operation 'optimize'")` |
| `gallery` | `Err("unknown operation 'optimize'")` |
| `product` | `Err("unknown operation 'optimize'")` |

The demo escapes this only because `demo/worker.js`'s `geometryRecipe()` hand-builds a
*different*, terminal-step-free recipe (`auto-orient` + optional `resize`). So the demo is
fine and the **published `crustyimg-wasm` npm package is not**: a JS consumer following the
README hits the error.

### The design question, and why it is already answered

`build` had a real fork — with the terminal step stripped, *something* must choose the output
format. **`transform` has no such fork.** Its signature takes `out_format`, and
`parse_format` (`src/wasm.rs:86`) resolves it through `format_from_extension`, which
**cannot** accept `"auto"` or an empty string — only a concrete format. So the caller has
always pinned the format.

That is exactly DEC-087's pinned branch, and `apply`'s before it: *honour the pin, skip the
decision.* **Strip the marker, run the pixel steps, encode to `out_format`.**

The decide-path counterpart already exists and is untouched by this spec:
`optimizeDetailed` is where a caller goes to have the format chosen. Keeping `transform`
pinned and `optimizeDetailed` deciding is the clean split, and it matches the CLI's
`apply --recipe web -o hero.png` (pinned) versus `apply --recipe web` (decided).

## Goal

Make `transform` run the recipes crustyimg ships with, so the README's claim about it is
true — without giving it a format-decision path it does not need.

## Inputs

- **Files to read:**
  - `src/wasm.rs:155-170` — `transform`, and `:86-92` `parse_format` (why `out_format` is
    always concrete).
  - `src/cli/optimize.rs:25-45` — `OPTIMIZE_STEP_OP` and `split_terminal_optimize`, the
    helper to reuse. SPEC-111 made it `pub(super)`; reaching it from `wasm` will need a
    wider but still crate-internal visibility, or a move to a neutral module.
  - `src/recipe/bundled.rs` — the three recipes and `resolve()`.
  - `demo/worker.js` `geometryRecipe()` — why the demo is unaffected; do not change it.
  - `decisions/DEC-087-*.md` — names this as an exception; the amendment is part of the work.
  - `README.md:34-36` — the claim this makes true.
- **Related code paths:** `src/wasm.rs`, `src/cli/optimize.rs`, `tests/wasm_roundtrip.rs`.

## Outputs

- **Files modified:**
  - `src/wasm.rs` — `transform` strips the terminal marker before `build_pipeline`.
  - `src/cli/optimize.rs` (or a neutral module) — widen `split_terminal_optimize` /
    `OPTIMIZE_STEP_OP` visibility so `wasm` can reuse them. **Do not copy them.**
  - `tests/wasm_roundtrip.rs` — the failing tests below.
  - `decisions/DEC-087-*.md` — dated amendment: the exception is closed, with the reason it
    was reopened (the README, not the demo).
- **New exports:** none public beyond the existing `transform`. Keep reuse `pub(crate)`.

## Acceptance Criteria

- [ ] **AC-1.** `transform(png, <bundled TOML>, "png")` succeeds for **all three** bundled
      recipes — `web`, `gallery`, `product` — driven through the real wasm surface, not the
      native call chain. All three fail today.
- [ ] **AC-2.** The output honours the **caller's** `out_format`, not a decided one: asking
      for `"png"` yields real PNG **bytes**, asking for `"jpeg"` yields real JPEG bytes.
      Assert on bytes, not on a returned name — this is the whole reason `transform` needs no
      decision path, so it is the thing to pin.
- [ ] **AC-3.** The pixel steps actually ran. A bundled recipe resizes; assert the output
      **dimensions** differ from the input where the recipe says they should. A strip that
      dropped the whole recipe rather than just the marker would pass AC-1 and AC-2 and fail
      here. [[a-harness-that-exercises-nothing-reports-green]]
- [ ] **AC-4.** **A recipe with no terminal marker is unchanged.** `geometryRecipe()`'s exact
      shape (`auto-orient` + `resize`, no `optimize`) produces byte-identical output to
      `main`. This is the guard that the live demo cannot regress.
- [ ] **AC-5.** The strip keys on the **reserved name**: a recipe whose terminal step is a
      genuinely unknown op still returns a `JsError`, and an `optimize` step **not last**
      still errors — matching `split_terminal_optimize`'s documented contract and `build`'s
      behaviour.
- [ ] **AC-6.** The module survives: after a rejected recipe, a subsequent ordinary
      `transform` call still succeeds — the established pattern from
      `optimize_detailed_rejects_oversize_without_panic`.
- [ ] **AC-7.** **DEC-087 amended**, dated, stating that the exception is closed and why it
      was reopened: the demo reasoning held, the README's claim did not. Do not silently
      delete the exception — the record should show the call and its correction.
      [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]
- [ ] **AC-8.** **The README claim is now true**, and nothing else in it overstates the wasm
      surface. Re-read `README.md:34-36` as text against what `transform` now does.
      [[documentation-has-no-green]]
- [ ] **AC-9.** A **negative control**: reverting the strip must turn at least one AC-1 test
      RED. Record it, and prove the revert reached the built artifact rather than only the
      source. [[reverting-source-does-not-rebuild-the-binary]]
- [ ] **AC-10.** Clean **full matrix** from fresh per-leg `CARGO_TARGET_DIR`s, sequentially,
      **through `rtk proxy` from the first leg**: default, `--no-default-features`,
      `--features webp-lossy`; `clippy -D warnings` each; `fmt --check`; plus
      `just wasm-test`. Confirm each log says `Compiling crustyimg`. **Then read the CI legs.**

## Failing Tests

Written during **design**, BEFORE build. Expected to FAIL against current `main` except where
noted.

- **`tests/wasm_roundtrip.rs`**
  - `"transform_runs_every_bundled_recipe"` — AC-1, all three via `bundled::resolve`.
    **Fails today** (`unknown operation 'optimize'` on each).
  - `"transform_honours_the_callers_out_format"` — AC-2, PNG and JPEG, asserting magic
    bytes. **Fails today** (never reaches the encode).
  - `"transform_actually_runs_the_pixel_steps"` — AC-3, asserting the resize happened.
    **Fails today.**
  - `"transform_leaves_a_markerless_recipe_unchanged"` — AC-4, the demo-shape guard.
    **Passes today**; it is the did-not-break-the-demo control and must be written anyway.
  - `"transform_still_rejects_an_unknown_terminal_op"` — AC-5. **Passes today**; guards that
    the strip keys on the reserved name rather than dropping whatever is last.
  - `"transform_still_rejects_optimize_not_last"` — AC-5. **Passes today**; regression guard.
  - `"the_module_survives_a_rejected_recipe"` — AC-6. **Fails today** only in that the
    preceding case errors for the wrong reason; write it as the survival pattern.
- **Negative control** (AC-9, run and recorded, not committed)
  - Revert the strip → `transform_runs_every_bundled_recipe` must go RED.

## Implementation Context

### Decisions that apply

- `DEC-005` — recipes are the portable unit; the same TOML is meant to run in both surfaces.
  That is the promise this spec makes true.
- `DEC-087` — SPEC-111's decision, which names this exception. AC-7's amendment is required
  work, not a nicety: the record currently says this is out of scope, and after this spec it
  is not.
- **No new DEC is expected.** The design question is answered by DEC-087's existing pinned
  branch. If the build finds a reason to deviate, that is a finding — report it rather than
  inventing a second rule.

### Constraints that apply

- `test-before-implementation` (**blocking**) — the Failing Tests go in first.
- `clippy-fmt-clean` (**blocking**) — every leg of AC-10, wasm included.
- `one-spec-per-pr` (**blocking**) — the 0.7.0 cut is STAGE-040's separate chore. Do not bump
  the version here.

### Prior related work

- `SPEC-111` (shipped) — the same defect in `build`, and the source of the helper. Read its
  Context for why stripping without threading a format was a trap **there** — and note that
  the trap does not exist here, because `out_format` is always pinned.
- `SPEC-085` (shipped) — the terminal `optimize` marker and what it means.
- `SPEC-072` (shipped) — the wasm seam and `transform`'s original contract.

### Out of scope (for this spec specifically)

- **The 0.7.0 cut** — STAGE-040's chore, and it depends on this landing.
- **Giving `transform` a decision path.** `optimizeDetailed` is that surface. Adding `"auto"`
  support to `transform` would be a new capability and needs its own spec.
- Changing `demo/worker.js`. The demo builds its own recipe and is unaffected; AC-4 proves it.
- The other two DEC-087 follow-ups (`build`'s truncated-JPEG warning, orphaned artifacts).

## Notes for the Implementer

- **Reuse, do not copy.** `split_terminal_optimize` lives in `src/cli/optimize.rs:39` and
  already documents the "an `optimize` step anywhere but last stays an error" rule AC-5 pins.
  It is `pub(super)` after SPEC-111; widening it to `pub(crate)` or moving it somewhere
  neutral (it is really a *recipe* concern, not a *cli* one) are both reasonable — say which
  you chose and why. A second copy in `wasm.rs` is the one outcome to avoid.
- **`wasm.rs` is compiled for `wasm32` and for native tests.** Check the visibility change
  works on both — `just wasm-check` is the fast gate before the full `just wasm-test`.
- **AC-3 is the trap.** A strip that removes the whole recipe, or that runs no steps, still
  returns bytes in the right format and passes AC-1 and AC-2. Only a dimension assertion
  catches it.
- **AC-4 is the other trap.** The live demo depends on markerless recipes behaving exactly as
  they do today. Byte-identity against `main`, not "it still works".
- **Do not fix the README by weakening it.** The claim is a good one and the code should meet
  it. If you find you cannot make it true, stop and report rather than editing the sentence.
- **Enumerate before claiming completeness.** SPEC-111's verify established that
  `registry.build(&step.op, …)` is the only route from an op name to an `Operation`, with one
  production caller — so `transform` should be the last unstripped site. Confirm that with
  your own grep and state its scope as a claim; do not inherit it.
  [[mechanical-sweeps-need-a-mechanical-check]]

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

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
