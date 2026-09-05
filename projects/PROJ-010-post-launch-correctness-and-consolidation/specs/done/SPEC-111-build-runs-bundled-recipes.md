---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-111
  type: bug                        # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
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
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 18584813
      duration_minutes: 1390.9
      recorded_at: 2026-08-08
      tokens_breakdown:
        input: 272
        output: 70607
        cache_creation: 172168
        cache_read: 18341766
      estimated_usd: 7.21
      note: >
        MEASURED — transcript sum over 443 assistant messages
        (~/.claude/projects/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg/e5a13298-502b-4c30-af59-4f49967d5398.jsonl),
        priced at Sonnet anchors ($3/$15 per MTok, cache_creation x1.25 input,
        cache_read x0.10 input). duration_minutes is the raw first->last
        transcript timestamp delta (18:42 2026-08-07 -> 17:53 2026-08-08) and
        includes wall-clock gaps waiting on 3 sequential fresh-target-dir
        matrix rebuilds plus a session boundary — not continuous active work.
        ⚠ CORRECTED 2026-09-05 (SPEC-127 verify + orchestrator, independently).
        The original figure summed EVERY transcript line carrying `usage`. Claude
        Code writes one line per CONTENT BLOCK, and lines sharing a `.message.id`
        repeat identical input/cache_creation/cache_read, so the three static
        fields were double-counted once per extra block. Recomputed by deduping on
        `.message.id`, taking those three from the group and MAX output.
        Was $56.84 / 144,391,578 tokens (1.94x over) over the same
        443 transcript lines = 231 real API calls. See STAGE-053.
    - cycle: verify
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 7920453
      duration_minutes: 192.4
      recorded_at: 2026-08-08
      tokens_breakdown:
        input: 103
        output: 62724
        cache_creation: 189605
        cache_read: 7668021
      estimated_usd: 6.59
      note: >
        MEASURED — transcript sum over 121 assistant messages, every
        `.message.model` = claude-opus-5, priced at Opus anchors ($5/$25 per
        MTok, cache_creation x1.25 input, cache_read x0.10 input) per the
        SPEC-107 verify precedent. 96.38% cache reads. duration_minutes is the
        raw first->last transcript timestamp delta (21:32 2026-08-08 -> 00:45
        2026-08-09 UTC) and includes wall-clock gaps waiting on the sequential
        fresh-target-dir matrix, four extra release builds (branch, main,
        webp-lossy, mutant) and two wasm runs — not continuous active work.
        Read at write-up time, so the tail of this session's own tokens is not
        included (same convention as prior cycles).
        Ordered BEFORE the punch-list build below: verify ran between the two
        build sessions and is what sent the spec back.
        ⚠ CORRECTED 2026-09-05 (SPEC-127 verify + orchestrator, independently).
        The original figure summed EVERY transcript line carrying `usage`. Claude
        Code writes one line per CONTENT BLOCK, and lines sharing a `.message.id`
        repeat identical input/cache_creation/cache_read, so the three static
        fields were double-counted once per extra block. Recomputed by deduping on
        `.message.id`, taking those three from the group and MAX output.
        Was $14.57 / 16,786,988 tokens (2.21x over) over the same
        121 transcript lines = 55 real API calls. See STAGE-053.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 35393900
      duration_minutes: 274.2
      recorded_at: 2026-08-09
      tokens_breakdown:
        input: 514
        output: 142989
        cache_creation: 380974
        cache_read: 34869423
      estimated_usd: 14.04
      note: >
        Second build session — the PUNCH LIST pass (record accuracy only, no
        behaviour change), on the same branch/PR. MEASURED — transcript sum
        over 257 assistant messages
        (~/.claude/projects/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg/cfece98d-ac6b-4bc6-bb2a-399c4c0ee7e5.jsonl),
        priced at the same Sonnet anchors as the first build session.
        duration_minutes is the raw first->last transcript timestamp delta
        (02:43 -> 07:17 UTC, 2026-08-09) and includes wall-clock gaps waiting
        on two rounds of full-matrix GitHub Actions CI (12 legs each) to
        settle after two pushes — not continuous active work.
        ⚠ CORRECTED 2026-09-05 (SPEC-127 verify + orchestrator, independently).
        The original figure summed EVERY transcript line carrying `usage`. Claude
        Code writes one line per CONTENT BLOCK, and lines sharing a `.message.id`
        repeat identical input/cache_creation/cache_read, so the three static
        fields were double-counted once per extra block. Recomputed by deduping on
        `.message.id`, taking those three from the group and MAX output.
        Was $14.04 / 35,393,900 tokens (1.95x over) over the same
        257 transcript lines = 136 real API calls. See STAGE-053.
    - cycle: ship
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered orchestrator main-loop cycle (AGENTS §4). Orchestrator work
        on this spec outside the metered total: driving the failure at design
        (four invocations with two controls), reconciling each cycle's cost,
        resolving the add/add merge conflict on `SPEC-111-verify.md` in
        `b92feef`, and the sequencing error that caused it — the punch-list
        prompt asked a fix session to correct a file that lives on `main`
        rather than on its branch, which should have been a separate PR.
  totals:
    tokens_total: 61899166
    estimated_usd: 27.84
    session_count: 3
    note: >
      Sum of the three METERED cycles: build $56.84 + verify $14.57 (Opus
      anchors) + punch-list build $14.04. Each independently reconciled by the
      orchestrator against its own component breakdown at the anchors of the
      model recorded in `agent` (DEC-083); all three reproduce exactly.
      ⚠ 98.0% of this token figure is cache re-reads. Non-cache-read volume is
      **3,843,644** — and note that is LOWER than SPEC-110's 5,352,083 despite a
      higher headline, because these sessions re-read more context rather than
      doing more work. `tokens_total` is a faithful sum of the usage records and
      NOT a measure of distinct work; see SPEC-110's totals note and DEC-083
      before comparing across specs, and never against pre-SPEC-107 specs, which
      recorded hand-estimates that did not count cache reads at all.
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

- **Branch:** `feat/spec-111-build-recipes` (own worktree)
- **PR:** #138 — https://github.com/jysf/crustyimg/pull/138 (NOT merged — verify runs next)
- **All acceptance criteria met?** yes — AC-1 through AC-11, all driven (see the matrix
  readout below for AC-11's full legs and `just wasm-test`).
- **New decisions emitted:**
  - `DEC-087` — `build`'s name template is the format pin; `edit --save-recipe` records
    the bake. Covers both design calls, the cache-key finding, and the wasm/warning
    exceptions named below.
- **Deviations from spec:**
  - **Split the pre-written AC-2 test.** The spec's `build_writes_the_decided_format_not_the_source_format`
    hardcodes ".avif" for a photographic source. That holds on lean (skipped, no avif)
    and default (avif is the only lossy candidate) but NOT on `webp-lossy` — the fast
    decision then has two competing lossy candidates, and correctly, legitimately,
    picked lossy WebP over AVIF for this fixture there; the negative-test-run also
    turned up that lean's own no-avif/no-webp-lossy shortlist correctly falls back to a
    baseline JPEG (`fast_fallback_lossy_entry`) rather than a lossless blow-up. Neither is
    a bug — both are the documented never-bigger/format-choice logic doing its job with a
    different codec roster. Kept the strict AVIF/bytes assertion, but re-gated it to
    `avif && !webp-lossy` (the one leg where AVIF is guaranteed to win the race, mirroring
    `tests/cli.rs`'s existing `web_equals_apply_recipe_web` precedent), and added an
    unconditional `build_decided_format_matches_apply_on_every_feature_leg` that proves
    AC-2's real "one rule" requirement — build matches `apply --recipe web` byte-for-byte,
    whichever format wins — on every leg, including `webp-lossy`. This was caught by
    actually running the required matrix, not by reading the spec; see reflection 2/3.
  - **`target_recipe_hash` (cache-key fix), not named in the spec's Outputs list.**
    Implementing decision 1 surfaced a real correctness gap: two `build` targets sharing
    one recipe file but different name templates (one Pinned, one Decide) would compute
    the SAME cache key for the same input yet need DIFFERENT bytes — `crate::build::cache`'s
    own module doc asserts the output format is "a pure function of the input bytes and
    extension," which SPEC-111 breaks for terminal-`optimize` targets specifically. Fixed
    by folding the format PLAN into the target's effective recipe hash, but ONLY for
    terminal-`optimize` targets (a plain pixel recipe hashes via the untouched
    `crate::build::cache::recipe_hash`, so no prior cache/lockfile goes stale). Named and
    reasoned through in DEC-087's Consequences; covered by a dedicated unit test
    (`target_recipe_hash_distinguishes_pinned_from_decided`).
  - **Sweep finding, named not fixed: `src/wasm.rs`'s `transform`.** The required
    `.build_pipeline(` sweep found `transform` builds a pipeline from a caller-supplied
    recipe the same unstripped way `build` did before this spec — a bundled/terminal-
    `optimize` recipe handed to it would hit the identical `unknown operation 'optimize'`
    failure. Same defect class, explicitly out of scope per the spec's own "wasm... work"
    exclusion. Not fixed; named in DEC-087 rather than silently passed over or silently
    fixed beyond scope.
  - **Sweep finding, named not fixed: the truncated-JPEG warning gap.** `build`'s new
    Decide path reaches `optimize_decide_one` — the same seam SPEC-107's truncated-JPEG
    stderr warning fires from on `apply --recipe web` — but `encode_one_optimize_decided`
    discards the `truncated_jpeg` signal rather than threading it through, so `build`
    does not yet warn where `apply` would for the identical input. Not an AC of this
    spec; named in `docs/api-contract.md` and DEC-087 as a follow-up rather than expanded
    into scope.
- **Follow-up work identified:**
  - Thread SPEC-107's truncated-JPEG stderr warning through `build`'s terminal-
    `optimize`/Decide path, for parity with `apply --recipe web` on the same input.
  - Strip the terminal `optimize` step in `src/wasm.rs`'s `transform` (or explicitly
    document bundled/terminal-`optimize` recipes as unsupported there) — the same defect
    class this spec fixed for `build`, found by the sweep, out of scope here.

### Punch-list pass (second build session) — record accuracy only, no behaviour change

Verify returned ⚠ PUNCH LIST on PR #138: all 11 ACs hold under driving; two documentation
claims were over-broad; one architect error had reached `main`. This session fixed all
three and decided-and-recorded three further, non-blocking items. No production behaviour
changed.

- **Item 1 — DEC-087's "complete" claim, narrowed.** Verify drove `edit --invert -q 40`
  → replay without `-q` produced different bytes than replay with `-q 40`: quality is not
  a recipe field, so "a complete, replayable description of what `edit` did" was false as
  written. Narrowed to "a complete, replayable description of the **pixel steps** `edit`
  ran," with the quality caveat stated explicitly. AC-8/AC-9 were never in question — only
  the claim's scope was wrong.
- **Item 2 — `cache.rs`'s module doc, re-justified.** It asserted the output-format-implies-
  hit invariant holds because format is "a pure function of the input bytes and extension."
  SPEC-111 itself falsifies that premise (a terminal-`optimize` target's format also
  depends on the target's Pin/Decide plan). The invariant still holds, but only because
  `target_recipe_hash` (`src/cli/build.rs`) folds the plan into the hash passed in as
  `recipe_hash` before this module ever sees it. Re-worded to state that basis.
- **Item 3 — an architect error in `SPEC-111-verify.md:63`, corrected.** The archived
  verify prompt (merged to `main` in #139) called the cache-collision risk "a real
  pre-existing defect." Verify itself refuted that while driving it: a terminal-`optimize`
  target cannot reach `build` on `main` at all (dies at prepare), and the closest
  `main`-reachable shape already serves identical bytes correctly. It is a regression
  *this spec's own new capability* would have introduced, caught and closed inside the
  same change — not a pre-existing bug. **This was the orchestrating architect's
  transcription error, not this build's** — DEC-087's own Consequences text never called
  it pre-existing; the verify prompt's relay of it did. Corrected in place on this branch
  (the file did not previously exist here; materialized from `main` with the fix applied,
  so it will need a trivial merge reconciliation against `main`'s copy).
- **Decided — the weak AC-7 test: strengthened.** `build_lock_entry_names_the_decided_extension`
  asserted a cache hit using `--check` without deleting the prior output first — `--check`
  never writes, so this proved nothing about a real hit and would have passed on a silent
  rebuild too. Fixed (test-only, no behaviour change): the hit leg now deletes the written
  output, re-runs a real (non-`--check`) build, and asserts both the "(1 cached, 0
  rebuilt)" summary line and byte-identical output — reproducing verify's own manual drive
  of this exact scenario. Test count is unchanged (strengthened in place, not added).
- **Decided — orphaned artifacts on an extension flip: named, not fixed.** A `{ext}`/Decide
  target whose source content change flips the winning format (e.g. `photo.avif` →
  `photo.webp` on a later run) leaves the old file behind; `build` has never cleaned `out`.
  Pre-existing class, newly reachable because Decide's whole point is a content-dependent
  extension. Named in DEC-087's Consequences per verify's "at minimum, name it" floor;
  cleaning `out` is a scope decision of its own, left for a future spec.
- **Decided — `name = "{stem}"`: docs wording fixed.** A name template with no extension at
  all (not `{ext}`, no literal extension either) already exits 4 via
  `SinkError::UnknownFormat` (confirmed by reading `format_from_extension` and its exit-code
  mapping, `src/cli/mod.rs`) — same exit code as an unrecognized literal extension, but
  `docs/api-contract.md`'s wording only named the latter case. Broadened the wording to
  cover both; no behaviour changed (exit 4 already covered this case, undocumented).
- **Verification:** re-ran the default leg through `rtk proxy` — 838/838, 0 failed, exact
  match to the build session's reference count (the AC-7 strengthening changed a test
  body, not the test count). `cargo clippy --all-targets -- -D warnings` and `cargo fmt
  --check` both clean. `just wasm-test` 30/30, unaffected (this pass touched no
  `#[cfg(not(target_arch = "wasm32"))]`-gated code). Lean and webp-lossy legs relied on
  CI rather than a local re-run, given the change surface (doc comments, markdown, one
  test body).
- **CI legs read (not just the local matrix):** pushed, then polled PR #138's checks
  through to completion — 12/12 green: `build / test / clippy / fmt` on macOS/Ubuntu/
  Windows, `avif feature`, `webp-lossy feature`, `heic feature` on macOS/Ubuntu, `lean
  build`, `msrv (rust 1.90.0)`, `supply-chain policy (cargo-deny)`, `cost-capture audit`,
  `front-matter validation`. No pending, no failed. Cross-checked the summary against
  `rtk proxy gh pr checks 138`'s raw per-job table (the summarized form is itself an
  `rtk`-rewritten command) — both agree, 12/12 pass, confirming the docs-only + one-test-
  body change did not move the matrix on any leg, not just the one re-run locally.
- **PR:** #138, still not merged (`mergeable: CONFLICTING` — this branch materializes
  `SPEC-111-verify.md`, which `main` already has a different version of via #139; a
  normal add/add merge conflict to resolve at merge time, not a defect in this pass).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing structural — both design decisions were genuinely made and inlined, exactly
   as promised. The one gap the spec text didn't anticipate: the SAME recipe file can be
   bound to two different `build` targets with different name templates (one Pinned, one
   Decide), which breaks the PRE-EXISTING cache-key invariant that the output format is a
   pure function of (recipe, input) — `crate::build::cache`'s own module doc states this
   explicitly. Tracing that out and designing a safe, minimal (only-when-needed) fix cost
   real time the spec's "the format thread is the work" framing didn't quite cover, since
   it's a step further than threading the format to the WRITE — it's threading the format
   PLAN into the cache's correctness contract too.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The pre-written AC-2 failing test's hardcoded ".avif" outcome is leg-dependent, and
   nothing in the spec flagged it. It happens to hold on lean (skipped) and default (AVIF
   is the only lossy candidate there), but not on `webp-lossy` — one of the three REQUIRED
   matrix legs — where a second competing lossy candidate makes the byte-race winner a
   measured outcome, not an assumed one (the exact caution `tests/cli.rs`'s
   `web_equals_apply_recipe_web` already documents for the analogous `apply`/`web` case,
   but that precedent wasn't cross-referenced from this spec's Failing Tests section).

3. **If you did this task again, what would you do differently?**
   — Run the full three-leg matrix against the NEW tests immediately after writing them —
   before touching any `src/` file — rather than only smoke-testing against default. The
   lean/webp-lossy-specific format-choice fragility (the JPEG fallback, the WebP-vs-AVIF
   race) would have surfaced during test authoring instead of during the final matrix
   verify pass, avoiding a rebuild-and-recheck cycle on two of the three legs.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — **Don't ask a build branch to correct a file that lives on `main`.** The punch list's
   item 3 was a correction to `SPEC-111-verify.md`, which had merged to `main` in #139 and did
   not exist on the build branch. The fix session did exactly what was asked, created the file
   there, and it collided add/add — `mergeable: CONFLICTING` on an otherwise finished PR. It
   should have been a separate one-line PR against `main`. Cheap to resolve, entirely avoidable,
   and the orchestrator's error rather than the build's.

   **The design cycle earned its keep, against the stage's own framing.** STAGE-039 said this
   needed no design cycle — "the fix is wiring, not design." Driving it found that the framed
   fix was *necessary but not sufficient*, and that shipping only it would have written the
   source format and silently discarded the modernization: a **quieter** bug than the loud one
   being fixed. The lesson generalises past this spec — "no design needed" is itself a claim
   about code nobody has driven yet, and it costs one short cycle to falsify.

2. **Does any template, constraint, or decision need updating?**
   — **DEC-087** is new and, after the punch-list pass, accurate on all three counts verify
   pushed back on: its "complete recipe" claim is narrowed to the pixel steps (quality is not a
   recipe field — driven), the orphaned-artifact consequence is named, and the `wasm::transform`
   exception is stated without a universality claim that would falsify it. That last point is
   SPEC-110's hardest lesson applied one spec later, and this time it held on the first pass.

   `src/build/cache.rs`'s module doc no longer justifies the cache invariant with a premise this
   spec falsifies — it now cites `target_recipe_hash` folding the plan into the key, which is
   the real basis.

   No template or constraint change.

3. **Is there a follow-up spec I should write now before I forget?**
   — **`src/wasm.rs::transform` carries the identical defect class** — an unstripped terminal
   `optimize` step. Verify drove it (`unknown operation 'optimize'`) and confirmed it is
   genuinely out of scope rather than conveniently so: `demo/worker.js:135` builds its own
   terminal-step-free recipe, so the shipped demo never reaches it, and `transform` takes an
   explicit `out_format`, making the fix a design question rather than a strip. It is reachable
   only by a third-party npm consumer of `crustyimg-wasm`. **Worth a small spec; not launch
   work.**

   **`build`'s new auto-decide path does not thread SPEC-107's truncated-JPEG warning.** Named
   in DEC-087 and confirmed by verify — `apply` warns, `build` does not. This is the same
   warning-coverage sweep SPEC-107's ship already filed for `diff`, `responsive`,
   `watermark --image`, `lint` and `meta strip`; `build` now joins that list, and one spec should
   close all of them.

   **Orphaned artifacts** when a content change flips the decided extension (`photo.avif` *and*
   `photo.webp` both left in `out/`). Pre-existing class, newly triggerable by this change, named
   in DEC-087. `build` has never cleaned and `--check` catches it loudly, so it is a papercut
   rather than a defect — but it is now easier to hit.
