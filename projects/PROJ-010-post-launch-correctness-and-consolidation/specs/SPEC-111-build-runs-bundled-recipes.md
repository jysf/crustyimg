---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-111
  type: bug                        # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: critical
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-010
  stage: STAGE-039
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5     # build on Sonnet: both decisions are made and
                                   # inlined, the failure is driven, and the fix
                                   # reuses an existing helper and an existing
                                   # precedent. Verify stays on Opus.
  created_at: 2026-08-07

references:
  decisions:
    - DEC-057
    - DEC-070
    - DEC-086
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
    - every-public-fn-tested
  related_specs:
    - SPEC-065
    - SPEC-085
    - SPEC-110

value_link: >
  STAGE-039's D-2: `build` can actually run the recipes it ships with, and a
  recipe saved by `edit` reproduces what `edit` did.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Drove the failure on a
        release build with a real manifest — bundled recipe by path AND by name,
        plus two controls (`apply --recipe web` and a plain pixel recipe through
        `build`) — and traced the format-choice question through `encode_one`,
        `lock_output_path` and `EXT_SENTINEL`.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-111: `build` runs bundled recipes

## Context

STAGE-039's D-2, and the **last launch-gating repo item**. Every bundled recipe ends with
the reserved terminal `optimize` step (`recipes/web.toml`, `gallery.toml`, `product.toml`);
the registry holds four ops and none is named `optimize`. `build` never strips that step, so
it reaches `build_pipeline` and dies.

### Driven, release build at `3dd8fa7`

A real manifest (`crustyimg.build.toml`, `version = 1`), one PNG source, `out = dist`:

| invocation | exit | result |
|---|---|---|
| `build`, `recipe = "web.toml"` (bundled file) | **1** | `error: target #0 (recipe web.toml, out dist)` / `error: unknown operation 'optimize'` |
| `build`, `recipe = "web"` (bundled **by name**) | **1** | identical failure |
| `apply --recipe web` on the same source | **0** | writes a real AVIF (`ftypavif`) |
| `build`, plain pixel recipe (no terminal `optimize`) | **0** | `built 1 target, 1 output` |

**A total failure, not a degradation** — and the two controls prove the fault is precisely the
terminal marker: `apply` handles the same recipe, and `build` handles a recipe without it.

### The mechanism, and the part the framing missed

`prepare_target` (`src/cli/build.rs:85`) calls `recipe.build_pipeline(registry)` unstripped —
that is the error site the stage named. But the fix does **not** end there.

`encode_one` (`src/cli/common.rs:33`), which `build` uses at `build.rs:317`, does two things:

```rust
let pipeline = recipe.build_pipeline(registry)?;   // :46 — dies on the terminal step
...
let fmt = img.source_format();                      // :52 — "no --format override in batch path v1"
Ok((crate::sink::extension_for_format(fmt), bytes))
```

It **preserves the source format**. But the whole point of the terminal `optimize` step is
that the fast decision *chooses* a format — AVIF for photos, lossless WebP for graphics
(SPEC-085). So stripping the step alone would run the pixel pipeline and then write the
result in the *source* format, silently discarding the modernization the recipe exists to do.
That would be a worse bug than the current one, because it fails silently.

**Good news: `build` already anticipates a post-decode extension.** `EXT_SENTINEL`
(`build.rs:115`) exists because *"the real output extension is only knowable after a decode"*,
and `lock_output_path` (`:221`) takes the real `ext` as a parameter. The naming and lockfile
layers are already shaped for this; only `encode_one`'s format choice is not.

### Design question 1: what picks the output format?

`apply` already answers this, and the answer should be copied rather than re-invented.
`run_apply` (`src/cli/optimize.rs:80-102`) splits the terminal step, then checks whether the
format is **pinned** — an explicit `--format`, or an `-o` whose extension names a real format.
If pinned, it honours the pin and skips the decision, with a comment explaining why:
`apply --recipe web hero.jpg -o hero.png` must match `web hero.jpg -o hero.png`, "a real PNG
of the downscaled image, not AVIF-in-a-`.png`".

**Decision: `build` uses the same rule, with the name template as the pin.** A target whose
template names a **literal extension** (`name = "{stem}.png"`) is an explicit pin — honour it,
skip the decision. A template using **`{ext}`** (including the default `{stem}.{ext}`) lets the
decision choose, and `{ext}` expands to the chosen format. `build.rs:575` already contemplates
literal-extension templates, so the distinction exists in the codebase; this gives it meaning
on the optimize path.

This keeps one rule across `apply` and `build` instead of two, which is the lesson SPEC-110
paid for.

### Design question 2: the recipe divergence SPEC-110 introduced

SPEC-110 made `edit` bake orientation on the CLI path but did **not** record `auto-orient` as
a step in `--save-recipe` output. Verify drove both sides: on pre-SPEC-110 `main` the `edit`
output and its replayed recipe agreed at 1200×800; after SPEC-110, `edit` gives 800×1200 and
the replay still gives 1200×800. **SPEC-110 introduced this** (DEC-086 records it as such), and
this spec is where it lands.

Two candidate fixes:

- **(a) `edit --save-recipe` records `auto-orient` explicitly.**
- **(b) `apply`/`build` gain the same implicit prefix the CLI verbs have.**

**Decision: (a).** Three reasons. A recipe is a *record of what happened*, and if `edit` baked
orientation the recipe should say so. Option (b) would make a recipe no longer a complete
description of its own behaviour — a hand-written recipe would silently gain a step it never
names, which is the "implicit behaviour nobody can state" pattern SPEC-110 just spent three
cycles removing. And the precedent already exists in this repo: **`recipes/web.toml` lists
`op = "auto-orient"` as an explicit first step.** Bundled recipes already name it; saved
recipes should too.

## Goal

Make `build` run the recipes the binary ships with — choosing the output format the way
`apply` already does — and make a recipe saved by `edit` reproduce what `edit` did.

## Inputs

- **Files to read:**
  - `src/cli/build.rs:85` (`prepare_target`'s unstripped `build_pipeline`), `:106-115`
    (`EXT_SENTINEL` and why it exists), `:215-234` (`lock_output_path`), `:317` (the
    `encode_one` call), `:575` (literal-extension templates).
  - `src/cli/common.rs:33-56` — `encode_one`, including the `img.source_format()` at `:52`
    that this spec must make conditional.
  - `src/cli/optimize.rs:25-45` — `OPTIMIZE_STEP_OP` and `split_terminal_optimize`, the
    helper to reuse; `:80-102` — `run_apply`'s pinned-format rule, the precedent to copy.
  - `src/recipe/bundled.rs:39-52` — the three bundled recipes; `recipes/web.toml` — note it
    already names `auto-orient` explicitly.
  - `src/cli/ops.rs` — `run_edit` and its `--save-recipe` path (design question 2).
  - `decisions/DEC-070` point 4 — this defect is self-documented there.
- **Related code paths:** `src/cli/build.rs`, `src/cli/common.rs`, `src/cli/optimize.rs`,
  `src/build/`.

## Outputs

- **Files modified:**
  - `src/cli/build.rs` and/or `src/cli/common.rs` — strip the terminal step, and thread the
    decided format through to the extension `lock_output_path` and the sink receive.
  - `src/cli/optimize.rs` — `split_terminal_optimize` / `OPTIMIZE_STEP_OP` likely need to
    become `pub(super)` so `build` can reuse them. **Do not copy them.**
  - `src/cli/ops.rs` — `--save-recipe` records `auto-orient`.
  - `docs/api-contract.md`, `docs/data-model.md` — `build`'s recipe support and format choice.
  - `decisions/DEC-NNN-*.md` — **a new decision** covering both calls above.
- **New exports:** none public. Keep reuse at `pub(super)`/`pub(crate)`.

## Acceptance Criteria

- [ ] **AC-1.** `crustyimg build` completes and writes output for a manifest target bound to
      each of `web`, `product` and `gallery` — **both by bundled name and by file path**, since
      both fail identically today.
- [ ] **AC-2.** The output is the format the **decision** chose, not the source format. On a
      photographic PNG source through `web`, the bytes are AVIF (`ftypavif`) and the file is
      named `.avif` — matching what `apply --recipe web` produces from the same input. Assert
      on the **bytes**, not the extension alone; an AVIF-in-a-`.png` would pass an
      extension-only check.
- [ ] **AC-3.** A target whose template names a **literal extension** (`name = "{stem}.png"`)
      pins the format: a real PNG, decision skipped — the `build` twin of
      `apply --recipe web -o hero.png`. Assert the bytes are PNG.
- [ ] **AC-4.** **Negative control:** a recipe whose terminal step is a genuinely unknown op
      (not `optimize`) still fails with `UnknownOperation` and a non-zero exit. The strip must
      key on the reserved name, not "drop whatever is last".
- [ ] **AC-5.** An `optimize` step **anywhere but last** still surfaces as `UnknownOperation` —
      the existing documented behaviour of `split_terminal_optimize`, which must not regress.
- [ ] **AC-6.** A plain pixel recipe through `build` is **unchanged** — byte-identical output
      to before. This is the "did not break the working path" guard; today it exits 0 and must
      keep doing exactly what it did.
- [ ] **AC-7.** The lockfile and cache name the file that was actually written, with the real
      chosen extension — `lock_output_path` already takes `ext`, so confirm the decided format
      reaches it. A cache **hit** must reproduce the same output path as the miss that filled it.
- [ ] **AC-8.** **The recipe divergence is closed.** `edit --invert --save-recipe` on an
      `Orientation=6` JPEG produces an output, and replaying that saved recipe via `apply`
      reproduces the **same dimensions** — 800×1200 from a 1200×800 source. Today: `edit` gives
      800×1200 and the replay gives 1200×800.
- [ ] **AC-9.** The saved recipe **names `auto-orient` explicitly**, matching how
      `recipes/web.toml` already writes it. Assert on the recipe TOML, not only on the replay —
      a replay that agrees for the wrong reason still passes AC-8.
- [ ] **AC-10.** `build --check` / `--frozen` / `--locked` and `--watch` still behave, since the
      output extension now varies with content. Name which of these you drove.
- [ ] **AC-11.** Clean **full-matrix** green from fresh per-leg `CARGO_TARGET_DIR`s,
      sequentially, **through `rtk proxy` from the first leg**: default,
      `--no-default-features`, `--features webp-lossy`; `clippy -D warnings` each;
      `fmt --check`; plus `just wasm-test`. Confirm each log says `Compiling crustyimg`.
      **Then read the CI legs** — a local macOS pass is not the required matrix.

## Failing Tests

Written during **design**, BEFORE build. Expected to FAIL against current `main` except where
noted.

- **`tests/build.rs`**
  - `"build_runs_each_bundled_recipe_by_name"` — AC-1, all three, by name. **Fails today**
    (`unknown operation 'optimize'`).
  - `"build_runs_a_bundled_recipe_by_path"` — AC-1. **Fails today.**
  - `"build_writes_the_decided_format_not_the_source_format"` — AC-2, asserting AVIF magic
    bytes. **Fails today.**
  - `"build_honours_a_literal_extension_template_as_a_format_pin"` — AC-3, asserting PNG
    bytes. **Fails today.**
  - `"build_still_rejects_an_unknown_terminal_op"` — AC-4. **Passes today**; it is the guard
    that the strip keys on the reserved name. Must be written anyway.
  - `"build_still_rejects_optimize_not_last"` — AC-5. **Passes today**; regression guard.
  - `"build_plain_pixel_recipe_output_is_unchanged"` — AC-6. **Passes today**; the
    did-not-break-it guard.
  - `"build_lock_entry_names_the_decided_extension"` — AC-7. **Fails today.**
- **`tests/cli.rs`** (or `tests/orientation.rs`, beside SPEC-110's)
  - `"saved_recipe_replays_to_the_same_dimensions"` — AC-8. **Fails today** (800×1200 vs
    1200×800).
  - `"saved_recipe_names_auto_orient_explicitly"` — AC-9. **Fails today.**
- **Negative control** (AC-4 + AC-10, run and recorded)
  - Make the strip drop the last step unconditionally → `build_still_rejects_an_unknown_terminal_op`
    must go RED.

## Implementation Context

### Decisions that apply

- `DEC-057` — manifest paths resolve relative to the process CWD. Do not change path
  resolution while threading the format through.
- `DEC-070` **point 4** — this defect is self-documented there; the record predicted it.
- `DEC-086` — SPEC-110's bake decision, which **introduced** the recipe divergence AC-8/AC-9
  close. Its Consequences already say so; do not re-describe it as pre-existing.
- **A new DEC is required** covering both design calls: the name-template-as-format-pin rule,
  and `--save-recipe` recording `auto-orient` explicitly.

### Constraints that apply

- `test-before-implementation` (**blocking**) — the Failing Tests go in first.
- `clippy-fmt-clean` (**blocking**) — every leg of AC-11.
- `one-spec-per-pr` (**blocking**) — the `docs/data-model.md` worked-example chore is separate
  (it is STAGE-039's third item). Correcting `data-model.md`'s *recipe/build* description where
  this spec changes behaviour is in scope; rewriting its three fictional ops is not.
- `every-public-fn-tested` — any helper this introduces.

### Prior related work

- `SPEC-085` (shipped) — the terminal `optimize` step and the fast decision that makes
  `apply --recipe web` == the `web` verb. This spec extends that equivalence to `build`.
- `SPEC-065` (shipped) — injective source→output, `EXT_SENTINEL`, and the collision key that
  deliberately leaves `{ext}` unexpanded. Read it before changing anything about output paths.
- `SPEC-110` (shipped) — introduced the divergence AC-8 closes; also the source of this spec's
  process warnings below.

### Out of scope (for this spec specifically)

- The `docs/data-model.md` worked example's three fictional ops — STAGE-039's separate chore.
- Recipe schema v2 / a top-level `[output]` table. It would subsume this, but it is a much
  larger change and is not launch work.
- Adding a `--format` flag to `build` or the manifest. The name template is the pin; a
  manifest-level format key is a surface change on a frozen CLI and needs its own spec.
- Classifier, codec, wasm, or demo work.

## Notes for the Implementer

- **Reuse `split_terminal_optimize`; do not copy it.** It lives in `src/cli/optimize.rs:39`
  and already documents the "anywhere but last stays an error" rule AC-5 pins. Making it
  `pub(super)` is the intended move.
- **The format thread is the real work, not the strip.** `encode_one` hardcodes
  `img.source_format()` at `common.rs:52` with the comment *"no --format override in batch path
  v1"* — that comment is the thing this spec retires. Whatever you return must reach
  `lock_output_path`'s `ext` parameter, or AC-7 fails silently and the lockfile names a file
  that does not exist.
- **`build` is used by `apply` too** — `encode_one` is in `common.rs` and shared. Changing its
  signature touches both callers; check `apply`'s behaviour is unchanged (AC-6's sibling).
- **Assert on bytes, not extensions.** AVIF-in-a-`.png` passes every extension check and is
  exactly the bug the pinned-format rule exists to prevent.
- **Three process warnings, all paid for by SPEC-110:**
  1. **Enumerate, do not trust the roster.** SPEC-110's design table omitted one verb and it
     cost a full extra build cycle. Before claiming the fix is complete, enumerate every code
     path that builds a pipeline from a `Recipe` and classify each.
     [[mechanical-sweeps-need-a-mechanical-check]]
  2. **If a sweep and this spec disagree, the sweep wins** — fix what it finds or name the
     exception in the DEC. Do not file it and ship a universal claim.
  3. **Read the CI legs.** A "full matrix clean" claim from a local macOS run shipped a red
     Windows leg on SPEC-107. [[a-green-gate-on-one-os-is-not-the-required-matrix]]
- **`rtk` corrupts output intermittently**, including collapsing `cargo test` and deleting the
  `Compiling crustyimg` line. `rtk proxy` from the first command; cross-check counts with
  `python3`. [[rtk-can-silently-corrupt-grep-counts]]

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
