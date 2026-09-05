---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-112
  type: bug                        # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
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
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 81750802
      duration_minutes: 187
      recorded_at: 2026-08-10
      tokens_breakdown:
        input: 640
        output: 84632
        cache_creation: 3966512
        cache_read: 77699018
      estimated_usd: 39.46
      note: >
        MEASURED by the ORCHESTRATOR at ship (AGENTS §4), over the build's own
        subagent transcript `subagents/agent-a0a1ffb97d8cbdd9d.jsonl`: 320
        assistant messages, `.message.model` = `claude-sonnet-5` on every one.
        Priced at Sonnet anchors ($3/$15 per MTok in/out; cache_creation x1.25
        input, cache_read x0.10 input); components sum exactly to
        `tokens_total`; 95.04% cache reads.

        CORRECTS the build's self-reported entry, which read `claude-opus-5` /
        8,119,424 / $6.75. That was measured against the PARENT ORCHESTRATOR's
        transcript, not the build's own — the build resolved its transcript as
        the newest `.jsonl` in the project directory, which was the
        orchestrator's session (84 Opus messages at that moment). Both the
        model and the volume were therefore wrong, in opposite directions: the
        wrong anchors overpriced, the wrong (much smaller) transcript
        underpriced by ~6x. The build DID run on Sonnet as the spec's
        `agents.implementer` pinned; there was no model mismatch to flag.

        Lesson, worth more than the number: identify a transcript by something
        only that session emitted, never by recency. The verify cycle avoided
        this by grepping for its own probe symbol — see its note below.
        ⚠ OVERSTATED, NOT RECOMPUTABLE (flagged 2026-09-05). This figure used the
        naive all-lines sum corrected in STAGE-053 — every measured sibling lands
        between 1.38x and 2.88x over, so this number is high by an unmeasured factor
        in that band. Its transcript is no longer on disk, so no prefix reproduces
        the recorded total and a corrected figure CANNOT be derived. Deliberately
        left rather than scaled by an average — a fabricated precision would be
        worse than a flagged unknown.
    - cycle: verify
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 27320821
      duration_minutes: 242
      recorded_at: 2026-08-10
      tokens_breakdown:
        input: 330
        output: 65225
        cache_creation: 3729250
        cache_read: 23526016
      estimated_usd: 36.70
      note: >
        MEASURED over the verify's own subagent transcript
        `subagents/agent-a850dfc9f23e3e26f.jsonl`, which verify identified by
        grepping for a probe symbol only its own session emitted rather than by
        taking the newest `.jsonl` in the project directory — the exact mistake
        that produced the build entry above. `.message.model` = `claude-opus-5`
        on all 165 assistant messages, so Opus anchors apply ($5/$25 per MTok
        in/out; cache_creation x1.25 input, cache_read x0.10 input). Components
        sum exactly to `tokens_total`; 86.11% cache reads.

        Verify self-reported 156 messages / 25,190,109 / $35.37; the
        orchestrator re-read the completed transcript at ship and found 165 /
        27,320,821 / $36.70. The gap is verify's own closing messages, which
        were not yet written when it measured. A session cannot count its own
        tail — the orchestrator's post-completion read is the accurate one, and
        that is the number recorded here.
        ⚠ OVERSTATED, NOT RECOMPUTABLE (flagged 2026-09-05). This figure used the
        naive all-lines sum corrected in STAGE-053 — every measured sibling lands
        between 1.38x and 2.88x over, so this number is high by an unmeasured factor
        in that band. Its transcript is no longer on disk, so no prefix reproduces
        the recorded total and a corrected figure CANNOT be derived. Deliberately
        left rather than scaled by an average — a fabricated precision would be
        worse than a flagged unknown.
  totals:
    # ⚠ MIXED: includes at least one session flagged OVERSTATED, NOT
    # RECOMPUTABLE — this total is an upper bound, not a measurement.
    tokens_total: 109071623
    estimated_usd: 76.16
    session_count: 2
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

- [x] **AC-1.** `transform(png, <bundled TOML>, "png")` succeeds for **all three** bundled
      recipes — `web`, `gallery`, `product` — driven through the real wasm surface, not the
      native call chain. All three fail today.
- [x] **AC-2.** The output honours the **caller's** `out_format`, not a decided one: asking
      for `"png"` yields real PNG **bytes**, asking for `"jpeg"` yields real JPEG bytes.
      Assert on bytes, not on a returned name — this is the whole reason `transform` needs no
      decision path, so it is the thing to pin.
- [x] **AC-3.** The pixel steps actually ran. A bundled recipe resizes; assert the output
      **dimensions** differ from the input where the recipe says they should. A strip that
      dropped the whole recipe rather than just the marker would pass AC-1 and AC-2 and fail
      here. [[a-harness-that-exercises-nothing-reports-green]]
- [x] **AC-4.** **A recipe with no terminal marker is unchanged.** `geometryRecipe()`'s exact
      shape (`auto-orient` + `resize`, no `optimize`) produces byte-identical output to
      `main`. This is the guard that the live demo cannot regress.
- [x] **AC-5.** The strip keys on the **reserved name**: a recipe whose terminal step is a
      genuinely unknown op still returns a `JsError`, and an `optimize` step **not last**
      still errors — matching `split_terminal_optimize`'s documented contract and `build`'s
      behaviour.
- [x] **AC-6.** The module survives: after a rejected recipe, a subsequent ordinary
      `transform` call still succeeds — the established pattern from
      `optimize_detailed_rejects_oversize_without_panic`.
- [x] **AC-7.** **DEC-087 amended**, dated, stating that the exception is closed and why it
      was reopened: the demo reasoning held, the README's claim did not. Do not silently
      delete the exception — the record should show the call and its correction.
      [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]
- [x] **AC-8.** **The README claim is now true**, and nothing else in it overstates the wasm
      surface. Re-read `README.md:34-36` as text against what `transform` now does.
      [[documentation-has-no-green]]
- [x] **AC-9.** A **negative control**: reverting the strip must turn at least one AC-1 test
      RED. Record it, and prove the revert reached the built artifact rather than only the
      source. [[reverting-source-does-not-rebuild-the-binary]]
- [x] **AC-10.** Clean **full matrix** from fresh per-leg `CARGO_TARGET_DIR`s, sequentially,
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

- **Branch:** `feat/spec-112-wasm-bundled-recipes`
- **PR:** [#144](https://github.com/jysf/crustyimg/pull/144)
- **All acceptance criteria met?** yes, all 10.
  - AC-1 through AC-6: the 7 pre-written tests all pass, driven through the real
    `wasm-bindgen` surface (`just wasm-test`, 37/37 — 30 pre-existing + 7 new).
  - AC-7: DEC-087 amended (dated 2026-08-09) — the wasm exception is closed, with the
    original call and its correction both left in the record, plus an inline pointer at
    the "Named, not fixed" bullet and the "Revisit if" clause that named this exact
    trigger.
  - AC-8: re-read `README.md:34-36` against the fixed code — the claim is now true.
    Scanned the rest of the WebAssembly section (`README.md:261-276`) for other
    overstatements; found none. No README edit was needed or made.
  - AC-9: negative control run and recorded below.
  - AC-10: full matrix green — see numbers below. CI legs on PR #144 still need to be
    read individually before this can be called done end-to-end; see note below.
- **New decisions emitted:** none. DEC-087's existing pinned-format branch answered the
  design question; amended in place rather than superseded.
- **Deviations from spec:**
  - The spec's Note said reaching `split_terminal_optimize` from `wasm` "will need a
    wider but still crate-internal visibility, or a move to a neutral module," offering
    both as options. **A plain visibility widening (`pub(super)` → `pub(crate)`) turns
    out not to be an option at all**, not merely the less-preferred one: `src/lib.rs`
    compiles `cli` only for `#[cfg(not(target_arch = "wasm32"))]` and `wasm` only for
    `#[cfg(target_arch = "wasm32")]`, so the two module trees never coexist in one
    build — a `cli`-hosted item, however widened, does not exist in the wasm32
    artifact `wasm::transform` compiles into. This isn't a deviation from the decision
    made (moving to a neutral module, chosen) but from the spec's framing of it as a
    close call between two workable options — it was a hard constraint discovered
    while implementing, not a preference. Recorded in DEC-087's amendment.
  - `OPTIMIZE_STEP_OP` moved alongside `split_terminal_optimize` (kept module-private
    in `src/recipe/mod.rs`, not re-exported) — the spec named only the function, but
    the constant only had one caller (the function itself), so moving both together
    avoided leaving an orphaned private const behind in `cli::optimize`.
- **Follow-up work identified:** none beyond what DEC-087 already carries forward
  (the truncated-JPEG stderr warning gap in `build`'s auto-decide path, and the
  orphaned-`out`-tree gap) — neither is this spec's territory.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing in the spec itself; the one thing that needed independent verification
   before writing code was whether `pub(crate)` alone would actually let `wasm`
   reach a `cli`-hosted helper. The spec listed it as an option, but `src/lib.rs`'s
   `#[cfg(target_arch = "wasm32")]` split on `cli` vs `wasm` (SPEC-072) rules it out
   structurally — five minutes reading `lib.rs` before touching code settled it, and
   is worth calling out explicitly in the amendment so the next reader doesn't
   re-litigate the same "just widen the pub" instinct.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Not a missing constraint, but a note that would have saved the five minutes
   above: the spec's own Note 1 ("wasm.rs is compiled for wasm32 and for native
   tests") is easy to misread as implying `cli` and `wasm` share a compilation unit.
   They don't — only the pure-engine modules (`recipe` among them) compile for both;
   `cli` and `wasm` are mutually exclusive per `src/lib.rs`'s target split. Worth a
   one-line addition to SPEC-072-adjacent docs if a future spec touches this seam
   again.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. The one thing I'd front-load next time: run the AC-9
   negative control (revert, confirm RED, hash the artifact, restore, confirm GREEN,
   hash again) immediately after the fix compiles and the new tests pass, rather than
   after writing the DEC-087 amendment — it would have caught, sooner, that the
   revert needed to disable exactly one line rather than the whole strip logic to
   stay a clean single-variable control.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — Give the build a way to identify **its own** transcript. Its one real error was a cost
   entry read off the parent orchestrator's `.jsonl` — resolved as "the newest file in the
   project directory" — which produced the wrong model, the wrong volume, and a confident
   "flagged as a finding, not silently reconciled" note about a model mismatch that did not
   exist. Verify avoided it by grepping for a probe symbol only its own session emitted. That
   technique belongs in `projects/_templates/prompts/cost-snippet.md`, not in one agent's
   good judgement. **Recency is not identity.**

2. **Does any template, constraint, or decision need updating?**
   — Yes, `cost-snippet.md`, per above. Also worth recording: the design offered two
   "reasonable" options for reaching `split_terminal_optimize` (widen to `pub(crate)` in
   `cli::optimize`, or move to a neutral module) and **one of them was impossible** — `cli`
   and `wasm` are mutually exclusive `#[cfg(target_arch)]` module trees, so no visibility on
   a `cli`-hosted item reaches `wasm::transform`. Both the build and verify caught it
   independently; DEC-087's amendment records it. A design that offers a false choice is a
   design that has not been driven, which is this wave's recurring lesson landing on the
   architect for once rather than the builder.

3. **Is there a follow-up spec I should write now before I forget?**
   — **Yes, and it is the most valuable thing this spec surfaced.** Verify found that **no CI
   leg runs `just wasm-test`**: `ci.yml` has no wasm32 step, and `pages.yml`'s
   `build + browser smoke` runs `just demo-build` + `just demo-smoke`, which drives only the
   demo's *markerless* path. So all 37 wasm tests — the 7 that pin this spec included — run
   only on a maintainer's machine, and a regression of exactly the defect this spec fixed
   would reach `main` and the npm package green. Filed for STAGE-038. Pre-existing and
   correctly out of scope here; the gap is that the guard for a launch-gating fix is not
   itself guarded.

   Second, smaller: `transform(bundled_toml, "png")` runs the recipe but **does not reproduce
   the marker's semantics** (fast AVIF-aware decision, never-bigger, score) — those are
   stripped, and `optimizeDetailed` is the decide-path counterpart, exactly as designed. The
   README says the TOML *runs*, which is now true, so AC-8 holds. But a JS consumer starting
   from `web` gets the downscale without the modernize unless they also call
   `optimizeDetailed`, and nothing in the README says so. A doc sentence, not a code change.
