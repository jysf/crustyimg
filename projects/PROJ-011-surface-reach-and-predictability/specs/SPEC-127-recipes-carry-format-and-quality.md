---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-127
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
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
  created_at: 2026-09-04

references:
  decisions:
    - DEC-005
    - DEC-015
    - DEC-087
    - DEC-098
    - DEC-058
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
    - every-public-fn-tested
    - ergonomic-defaults
  related_specs:
    - SPEC-126
    - SPEC-111

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-050's <capability>". Optional; null is acceptable.
value_link: >
  STAGE-050's thesis item. A recipe that cannot name its own output format is
  not a portable description of a job — the caller has to supply half of it.
  SPEC-126 made `apply` and `build` agree on what a format means, which is the
  precondition for the schema naming one.

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
        Un-metered main-loop design cycle (AGENTS §4). Added 2026-09-05 — verify
        found it missing; a null-with-note entry is required, an absent one is not
        the same thing.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 120553651
      duration_minutes: 53.4
      recorded_at: 2026-09-04
      tokens_breakdown:
        input: 508
        output: 7116
        cache_creation: 643816
        cache_read: 119902211
      estimated_usd: 38.49
      note: >
        MEASURED post-hoc by the orchestrator from the subagent's saved task
        transcript (254 distinct API calls, deduped by `.message.id` — the raw
        JSONL has 498 lines carrying `usage`, ~2 content-block lines per real
        call, and summing without dedup would double-count), priced at Sonnet
        anchors ($3/$15 per MTok, cache_creation x1.25, cache_read x0.10) per
        `.message.model` (DEC-083). The build prompt did not include the
        self-measurement instructions from `cost-snippet.md`, so the
        orchestrator reconstructed this from the Agent tool's saved output
        file rather than a `## Cost readout` block. ⚠ The Agent tool result's
        own `subagent_tokens` field read 649005 — ~185x smaller than the
        measured total, and within 0.6% of the LAST call's own cumulative
        usage alone (645167). That strongly suggests `subagent_tokens`
        reflects only the final turn's context snapshot, not a sum billed
        across the session, and is not a reliable cost source for a
        multi-turn subagent — flagged upstream, not used here.
  totals:
    tokens_total: 120553651
    estimated_usd: 38.49
    session_count: 1
---

# SPEC-127: recipes carry format and quality

## Context

A recipe describes a job, and today it can describe every part of that job **except what the
output should be**. `Recipe` carries `version`, `name`, `description` and `steps` — the pixel
operations — and nothing about the encode. So the same recipe run two ways produces two different
files, and the caller has to supply the missing half every time:

```
crustyimg apply --recipe web.toml hero.png -o out.jpg --format jpeg -q 82
                └─ the portable part ─┘   └──── the part that is not ────┘
```

⛔ **This was gated on STAGE-049 and is now unblocked.** SPEC-126 made `apply` and `build` resolve
output format by one rule (`--format` > `-o` ext > preserve source, DEC-015/DEC-098). Before that,
adding `format` to the schema would have baked a disagreement into it: the two paths would have
read the same field and produced different bytes.

**Driven on `main` at `7181eed` (0.7.1 + SPEC-126), with a valid recipe emitted by
`edit --save-recipe` rather than hand-written:**

| recipe | exit | message |
|---|---|---|
| valid `version = "1"` (control) | **0** | — |
| v1 **plus an unknown `format` key** | **1** | `could not parse recipe TOML: TOML parse error at line 2, column 1` |
| `version = "2"`, no unknown key | **1** | `unsupported recipe version '2' (supported: 1)` |

`Recipe` is `#[serde(deny_unknown_fields)]`, so **today's binary rejects any recipe carrying a
`format` key, and the error it gives is a TOML parse failure pointing at line 2** — which tells a
user nothing about needing a newer crustyimg. The version check, given a clean recipe, produces
exactly the right message. That asymmetry is measured, and it decides Call 1 below.

⚠ **Read the exit codes from `$?` after a redirect.** A first pass at this table piped through
`head` and reported `exit=0` for all three rows.

## Goal

`Recipe` can name its own `format` and `quality`; every path that runs a recipe honours them by
one precedence rule; and a recipe that uses them is rejected by older binaries with a message
that says so.

## Inputs

- `src/recipe/mod.rs` — `Recipe`, `RecipeStep`, `SUPPORTED_VERSION`, `from_toml`/`to_toml`,
  `split_terminal_optimize`.
- `src/cli/common.rs` — `encode_one(recipe, registry, input, format_override, quality)`, the seam
  every recipe-running path already funnels through.
- `src/cli/optimize.rs` — `run_apply`; `src/cli/build.rs` — the manifest path.
- `src/wasm.rs:171` — `transform(input, recipe_toml, out_format)`.
- DEC-015 (format precedence), DEC-098 (`apply` preserves at every arity), DEC-087 (a name
  template's literal extension names the file, it does not pin the format), DEC-005 (recipes
  round-trip through the registry), DEC-058 (the build cache key).

## Outputs

- `Recipe.format: Option<String>` and `Recipe.quality: Option<u8>`, round-tripping losslessly.
- `SUPPORTED_VERSION` accepts both `"1"` and `"2"`; a recipe using either new field must declare
  `version = "2"`.
- One precedence rule, applied by `apply`, `build` and `wasm::transform` alike.
- DEC-099 recording the calls below.

## The design calls — settled here

### Call 1 — a recipe using `format` or `quality` MUST declare `version = "2"`

Settled by the table in Context, not by preference. The naive answer is an optional field on v1:
`skip_serializing_if = "Option::is_none"` means existing recipes serialize unchanged and nothing
breaks. **That answer is wrong, and the measurement is why.** `deny_unknown_fields` means an older
binary handed a v1-plus-`format` recipe fails with a **TOML parse error at line 2**. The same
binary handed `version = "2"` says *"unsupported recipe version '2' (supported: 1)"* — the actual
problem, in the user's language.

crustyimg is a published crate with two shipped releases in the wild. A recipe is a file people
commit and share, so a forward recipe **will** meet an old binary. Gate the new fields behind the
version and that meeting produces a sentence someone can act on.

⚠ **`"1"` stays valid and unchanged.** This is not a migration: v1 recipes parse, run and
round-trip exactly as they do today, and `to_toml` must keep emitting `version = "1"` for a recipe
that uses neither new field. **Emitting `"2"` unconditionally would strand every existing recipe
on the next `--save-recipe`.**

### Call 2 — precedence: the CLI wins, the recipe beats the source, and `-o` sits between

Extend DEC-015's chain rather than inventing a second one:

```
--format (explicit CLI)  >  -o extension  >  recipe.format  >  preserve the source
```

The reasoning is DEC-098's, applied one level down: **the more specific and more immediate wins.**
A flag typed at the call site is a deliberate override of a file's default; a recipe's `format` is
a default the file carries. `-o hero.png` stays above the recipe because DEC-087 already rules a
recognised `-o` extension a pin, and reopening that here would change `web`/`optimize` behaviour
this spec has no business touching.

`quality` takes the same shape: `-q` > `recipe.quality` > the format's own default.

⚠ **The terminal `optimize` step is the carve-out.** A recipe ending in the reserved `optimize`
marker runs the auto-decide engine, which *chooses* a format and *searches* for a quality. A
recipe that both ends in `optimize` and declares `format`/`quality` is asking for two contradictory
things. **Rule: the explicit field wins and the decision is skipped** — that is already what a
`--format` pin does on that path (DEC-087), so this keeps one behaviour rather than adding a
second. It must be **tested**, not assumed.

### Call 3 — `wasm::transform` grows a source of truth, so make the parameter the override

`transform(input, recipe_toml, out_format)` takes the format as its own argument and parses it
itself (`parse_format` rejects `"auto"` and empty, so it is always a concrete pinned format). Give
the recipe a `format` and there are two answers to one question.

**Rule: `out_format` is the CLI-flag equivalent and wins; `recipe.format` is the fallback.** That
makes the wasm surface obey the same sentence as the native one, which is the whole point of
having a precedence rule at all.

📌 STAGE-053 raised this and correctly said it needs deciding **whether or not** the wasm CI leg
ever gets built. It is decided here. ⚠ But note what that means for evidence: `just wasm-test`
runs in no CI job today, so **a wasm assertion added by this spec is not covered by the required
matrix.** Say so in Build Completion rather than implying the leg is guarded.

### Call 4 — typed per-operation param structs are NOT in this spec

STAGE-050's backlog entry bundles "plus typed per-operation parameter structs" with this work.
**Split.** `OperationParams` is a `BTreeMap<String, toml::Value>` with hand-rolled `get_str` /
`get_u32` / `get_f32`, and typing it touches every operation in the registry — a refactor with its
own blast radius, its own round-trip risk under DEC-005, and no dependency on this schema change
in either direction. Bundling turns an **M** into an **L**, and AGENTS §2 says an L should be
split.

Filed back to STAGE-050 as its own item by this spec's design cycle.

## Acceptance Criteria

- [ ] **AC-1.** A recipe declaring `version = "2"` with `format` and `quality` **round-trips
      losslessly** — `from_toml(to_toml(r)) == r` — and a v1 recipe using neither field still
      round-trips and still serialises as `version = "1"`.
- [ ] **AC-2.** `apply` honours `recipe.format` at **1 input and at N inputs**, identically —
      driven for at least two target formats, asserted on the written **bytes** via
      `image::guess_format`, never the filename extension (SPEC-126, Call 4).
- [ ] **AC-3.** **Precedence holds in both directions:** `--format` overrides `recipe.format`, and
      with no `--format` the recipe's value beats preserve-source. Same for `-q` vs
      `recipe.quality`.
- [ ] **AC-4.** **`apply` and `build` produce byte-identical output** for the same v2 recipe and
      input — the SPEC-126 property, re-asserted against the new field. This is the one that
      catches a `build` path that reads the recipe differently.
- [~] **AC-5.** ⚠ **First half MET, second half NOT MET — and it was unachievable as written.**
      *Met:* a recipe using `format` or `quality` without `version = "2"` is rejected with a typed
      error (`RecipeError::NewFieldNeedsVersion2`), covered by
      `tests/recipe_v2.rs::new_field_without_v2_is_rejected`.
      *Not met:* a pre-spec binary does **not** say `unsupported recipe version` for a real v2
      recipe. `deny_unknown_fields` fires during deserialization **before** `from_toml`'s version
      check, so a v2 recipe carrying `format` gets `TOML parse error … unknown field 'format'`.
      The promised message appears only for a v2 recipe using **neither** new field — the case
      where v2 buys nothing. **No change on this branch can alter what an already-released
      0.7.0/0.7.1 binary prints**, so this criterion could not have been satisfied by any
      implementation. The design call that rested on it is corrected in DEC-099; the version gate
      stands on schema hygiene, not on forward-compatibility ergonomics.
      Found by SPEC-127's verify, re-driven by the orchestrator 2026-09-05.
- [ ] **AC-6.** The terminal `optimize` carve-out (Call 2): a bundled recipe with an explicit
      `format` **skips the auto-decision and honours the pin**; the same recipe without one still
      auto-decides and its output is **byte-identical to `main`**.
- [ ] **AC-7.** **A negative control, one revert per independent condition** (AGENTS §15). Call 1
      (version gate), Call 2 (precedence) and Call 3 (wasm) are independent; reverting each must
      flip only the tests that exercise it. The evidence is the **behavioural flip**, never a hash.
- [ ] **AC-8.** **Nothing else changes bytes.** `resize`, `thumbnail`, `watermark`, `optimize`,
      `web`, `convert`, `responsive` and a **v1** recipe through `apply`/`build` all produce
      byte-identical output to `main` on the corpus. ⚠ State the corpus boundary in the DEC —
      SPEC-126's verify had to add that.
- [ ] **AC-9.** Clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential: default,
      `--no-default-features`, `--features webp-lossy`. Clippy and `fmt --check` each. Plus
      `just wasm-check` — Call 3 touches an engine module.

## Failing Tests

Written during design, made to pass during build (constraint `test-before-implementation`). All
must be confirmed **RED against `main`** before any implementation, and the baseline recorded.

- `tests/recipe_v2.rs::v2_round_trips_format_and_quality` — AC-1.
- `tests/recipe_v2.rs::v1_still_round_trips_and_stays_v1` — AC-1's other half; the strand guard.
- `tests/recipe_v2.rs::new_field_without_v2_is_rejected` — AC-5.
- `tests/apply_batch.rs::apply_honours_recipe_format_at_every_arity` — AC-2. **Two tests, one per
  arity**, per SPEC-126's re-approve finding: a combined test dies at the first failing arity and
  the suite cannot say which regressed.
- `tests/apply_batch.rs::cli_format_overrides_recipe_format` — AC-3.
- `tests/apply_batch.rs::apply_and_build_agree_on_v2_recipe` — AC-4.
- `tests/optimize.rs::terminal_optimize_honours_an_explicit_recipe_format` — AC-6.

## Implementation Context

**Read before writing code:** this section, then DEC-015, DEC-087, DEC-098, DEC-005, DEC-058, the
parent `STAGE-050-recipe-reach.md`, the project `brief.md`, and `/guidance/constraints.yaml`.

**Where the seams are.** `encode_one` in `src/cli/common.rs` already takes
`format_override: Option<ImageFormat>` and `quality: Option<u8>` and is the funnel **both** `apply`
and `build` reach the encoder through — SPEC-126 confirmed `build.rs` calls it directly with an
unchanged signature. So the recipe's values should be resolved into those two parameters by each
caller, **not** read inside `encode_one`: that keeps the precedence decision at the call site where
the CLI flags also live, and it is why `build` cannot silently diverge.

`ops::output_format_for` (`src/cli/ops.rs`, `pub(super)` since SPEC-126) is the existing
`--format` > `-o` ext > source resolution. **Call 2 inserts `recipe.format` as a new lowest-but-one
rung.** Widen that function rather than reimplementing the chain beside it — reimplementation is
exactly what SPEC-126 avoided and the reason `apply` and `build` now agree.

**The version gate.** `SUPPORTED_VERSION` is a single `&str`. It becomes a set, and `from_toml`
grows a rule: the new fields require `"2"`. `to_toml` must emit `"1"` when neither field is set —
that is `v1_still_round_trips_and_stays_v1`, and it is the difference between an additive change
and stranding every recipe in the wild.

**The wasm target.** `src/recipe/` is an **engine** module (CLAUDE.md, DEC-064): it compiles for
`wasm32-unknown-unknown`. No `std::fs`, no `clap`. Run `just wasm-check`.

**Cost.** Follow `projects/_templates/prompts/cost-snippet.md`. Build and verify are metered
cycles and must not be left null (AGENTS §4, DEC-083 — price per component, never a flat rate).

**⛔ Byte-changing on a shipped verb? Only for recipes that opt in.** A v1 recipe must produce
identical bytes, which is AC-8. But the surface changes, so this still batches into PROJ-011's
single lockfile migration with the rest of STAGE-050 — **do not bump the version, do not cut a
release.**

## Notes for the Implementer

- 📌 **DEC-099 is reserved for this spec.** `next_id` scans only the working tree, so a record on
  an unmerged branch is invisible and the id can collide. Highest on `main` is DEC-098.
- ⚡ **Write `docs/api-contract.md` in the same change, not after.** SPEC-126 shipped without it
  and verify caught it; DEC-015's own `affected_scope` names that file, and the decisions audit
  **cannot** tell you so — it silently drops inline-array scopes (filed, PROJ-013 STAGE-047).
  The recipe section and the `apply`/`build` entries all describe format resolution.
- ⚠ **`cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal.** Redirect
  stdout; do not "fix" it. And a piped command reports the **pipe's** exit code — redirect and
  read `$?`. This spec's own Context table got that wrong on the first pass.
- **Never poll CI.** Background `gh pr checks --watch` and read a direct snapshot at the true head
  SHA when it exits.
- **Budget ~150 exchanges**, and push a WIP commit once it compiles, before the matrix.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-127-recipe-format-quality`
- **PR:** opened against `main` (see PR description / URL in the build session's final report).
- **All acceptance criteria met?** yes (AC-1 through AC-9; see `DEC-099`'s `## Validation` for the
  test-by-test mapping and the AC-8/AC-9 measured results).
- **New decisions emitted:**
  - `DEC-099` — `Recipe` gains `format`/`quality`, gated behind `version = "2"`; one precedence
    chain across `apply`, `build`, `wasm::transform`.
- **Files this diff touches** — from `git diff --name-only main`, not recall:
  - `src/recipe/mod.rs` — `Recipe` gains `format: Option<String>` / `quality: Option<u8>`;
    `SUPPORTED_VERSION_2` + `is_supported_version`; `RecipeError::NewFieldNeedsVersion2`;
    `from_toml` gates the two new fields behind `version = "2"`; `from_ops` and the module's own
    unit tests updated for the two new struct fields; one existing test
    (`from_toml_unsupported_version_still_rejected`) repointed from `version = "2"` (now valid) to
    `"3"`; several new unit tests for the version-2 gate.
  - `src/recipe/bundled.rs` — one test's `Recipe` struct literal updated for the two new fields (no
    behavior change).
  - `src/cli/ops.rs` — `output_format_for` widened with a `recipe_format: Option<&str>` parameter
    (new rung 3, between `-o` ext and preserve-source); its two non-recipe call sites in
    `run_pixel_op` pass `None`; three new unit tests for the new rung's ranking.
  - `src/cli/optimize.rs` — `run_apply`'s single-input branch threads `recipe.format`/
    `recipe.quality` through `output_format_for`/the sink write; the multi-input branch folds
    `recipe.format`/`recipe.quality` into `format_override`/`quality`, resolved once for the whole
    batch; the terminal-`optimize` "pinned" check now also treats an explicit `recipe.format` as a
    pin (Call 2's carve-out), materializing the resolved format explicitly so it applies at every
    arity.
  - `src/cli/build.rs` — `OutputFormatPlan`'s doc comments updated; `prepare_target` resolves
    `recipe.format` into `Pinned`/`Preserve`/`Decide` for both the terminal-`optimize` and
    plain-recipe cases; `build_one` resolves an effective per-target quality
    (`ctx.quality.or(prepared.recipe.quality)`) used for both the cache key and `encode_one`.
  - `src/wasm.rs` — `transform` grows the same precedence rung: an empty `out_format` defers to
    `recipe.format` (typed error if neither is set); `recipe.quality`, when set, reaches the
    encoder.
  - `docs/api-contract.md` — the `apply --recipe`/`build` entries document the widened precedence
    chain, the new `Recipe.format`/`Recipe.quality` fields, and the terminal-`optimize` carve-out.
  - `docs/data-model.md` — the Recipe Schema table gains `format`/`quality` rows, a precedence
    paragraph, the class diagram, and a Schema Evolution note.
  - `tests/recipe_v2.rs` (new) — the three schema-level failing tests (AC-1, AC-1's strand guard,
    AC-5).
  - `tests/apply_batch.rs` — four new tests (AC-2 ×2 arities, AC-3, AC-4).
  - `tests/optimize.rs` (new) — one test (AC-6, the terminal-`optimize` carve-out).
  - `tests/wasm_roundtrip.rs` — four new tests exercising Call 3 (format fallback, CLI-equivalent
    override wins, the neither-given error, quality threading).
  - `tests/recipe_round_trip.rs` — two existing `Recipe` struct literals updated for the two new
    fields (no behavior change).
  - `decisions/DEC-099-recipes-carry-format-and-quality.md` (new) — the decision record.
  - `projects/.../specs/SPEC-127-recipes-carry-format-and-quality.md` — this spec's own
    `## Build Completion`, cycle advance and cost entry.
  - `projects/.../specs/SPEC-127-recipes-carry-format-and-quality-timeline.md` — the build mark.
  (⚠ **Sixteen files.** The first thirteen bullets above were the whole list until 2026-09-05;
  verify found that `git diff --name-only main` — this entry's own stated derivation — returns
  **16**, and the three records the build itself wrote were missing from it.)
- **Deviations from spec:** none from the settled design calls. Two judgment calls the spec left
  open (recorded in `DEC-099`, not quietly re-decided):
  - Call 3's exact "no override" spelling for `out_format` wasn't specified in the spec text beyond
    "the parameter is the override, `recipe.format` is the fallback." Chose the empty string, the
    same sentinel `optimize`/`optimize_detailed` already use in `src/wasm.rs` for "let something
    else decide," rather than inventing a second convention in the same module.
  - `build`'s cache key needed no new explicit component for `format`/`quality` — verified
    structurally that `crate::build::cache::recipe_hash` already hashes the recipe's canonical
    TOML, which now includes both fields. Recorded as a finding in `DEC-099`, not built as new code.
- **Follow-up work identified:** none beyond what the spec's own Call 4 already filed back to
  STAGE-050's backlog (typed per-operation parameter structs — not built here, by design).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing genuinely ambiguous. The one place that took real thought rather than being handed to
   me was Call 2's carve-out for `build`: the spec states the carve-out in terms of `apply`
   (`--format`/`-o` pin skips the decision), and I had to work out its `build`-side analogue myself
   — that a plain (non-terminal-`optimize`) recipe's `format` should ALSO pin (not just the
   terminal-`optimize` case), since `build` has no `--format`/`-o` at all and `recipe.format` is
   its only rung. The spec's Outputs section ("one precedence rule ... applied by `apply`, `build`
   and `wasm::transform` alike") pointed at this but didn't spell it out verb-by-verb.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-098's "apply moves, build does not" ruling was exactly the reference needed to know
   `build` gets no new CLI-flag rung, only `recipe.format`/`recipe.quality`.

3. **If you did this task again, what would you do differently?**
   — Run the AC-8 corpus comparison (build `main` + this branch as separate binaries) BEFORE
   starting the AC-7 negative controls, not interleaved with them. A backgrounded `--features
   webp-lossy` build happened to be mid-compile when I started editing source files for a
   negative-control revert, and the two races contaminated that leg's first run (one test failed
   for a reason that was really "the source was mid-edit when cargo read it," not a real defect).
   Caught by re-deriving from a positive control rather than trusting the first red, and fixed by
   re-running the leg from a byte-for-byte clean tree — but sequencing background builds and
   source edits more deliberately would have avoided the detour.
