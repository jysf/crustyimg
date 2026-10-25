---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-131
  type: story                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-050
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: null                # chosen at build dispatch
  created_at: 2026-10-24

references:
  decisions:
    - DEC-005
    - DEC-014
    - DEC-031
    - DEC-058
    - DEC-061
    - DEC-064
    - DEC-099
    - DEC-100
    - DEC-101
  constraints:
    - untrusted-input-hardening
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-127
    - SPEC-128
    - SPEC-129
    - SPEC-133
    - SPEC-134

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-050's <capability>". Optional; null is acceptable.
value_link: >
  STAGE-050's recipe-reach outcome is "a recipe can express the whole job"; that only holds if a
  recipe that says something wrong is refused rather than silently run with defaults. Closes the
  silent-default path the watermark recipe surface (SPEC-128, unreleased) would otherwise ship in
  0.8.0.

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
    - cycle: frame
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 5329474
      tokens_breakdown:
        input: 94
        output: 24567
        cache_creation: 116763
        cache_read: 5188050
      estimated_usd: 3.94
      duration_minutes: null
      recorded_at: 2026-10-24
      note: >
        Interactive frame session in its own worktree (docs/spec-131-frame). Measured from the
        session's transcript (cf81d232-….jsonl, identified by content), deduped by `.message.id`
        (47 unique ids), input/cache_creation/cache_read once per id, MAX output_tokens per id.
        Priced by component at Opus $5/$25 per MTok, cache_creation x1.25 ($6.25), cache_read
        x0.10 ($0.50). Measured before the commit/push turns, so it slightly under-counts.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-131: typed per-operation parameter structs

## Context

STAGE-050 backlog item "[M] **Typed per-operation parameter structs**" (split out of SPEC-127 by
its design, Call 4). PROJ-011 ships as one release, 0.8.0.

`OperationParams` is a `BTreeMap<String, toml::Value>` plus a resolved-asset side channel
(`src/operation/mod.rs:53`), read through `get_str` / `get_u32` / `get_f32` / `get_bool`
(`:73-99`), each of which returns `None` for a missing key **and** for a wrong-typed one. Each op
validates its own keys in its constructor. Five ops are registered
(`src/operation/registry.rs:90-98`): `identity`, `invert`, `auto-orient` (no params), `resize`
(`mode`, `width`, `height`, `percent`) and `watermark` (`image`, `text`, `font`, `size`, `color`,
`gravity`, `opacity`, `scale`, `margin`, `tile`). The terminal `optimize` marker is not an op.

**The backlog premise ("found on input 400 of 500") is wrong on today's `main`.** `apply`
builds the pipeline once before resolving any input (`src/cli/optimize.rs:133-136`, "fails BEFORE
we touch any inputs"), and `build` does the same per target (`src/cli/build.rs:297-298`). Every
error a constructor *does* raise already fails at load, exit 1, zero outputs. **The real defect
is what the constructors do not raise.** Driven on a release build of `main` (`crustyimg 0.7.1`,
commit `804236a`), 5 generated 200×150 PNGs, `apply --recipe r.toml in/*.png --out-dir out`:

| recipe step (after `[[step]]`) | exit | outputs | stderr | output vs reference |
|---|---|---|---|---|
| `op="resize"` `mode="max"` `width="100"` (wrong type) | 1 | 0 | `error: invalid operation 'resize': mode 'max' requires a positive integer 'width' param (the long-edge cap)` | — |
| `op="resize"` `mode="max"` `widht=100` (misspelt) | 1 | 0 | *same line* — never names `widht` | — |
| `op="resize"` `mode="max"` `width=100` `bogus="x"` (unknown key) | **0** | 5 | *(empty)* | **0/5 differ** from the same step without `bogus` |
| `op="watermark"` `text="hi"` `opacity="0.5"` (string) | **0** | 5 | *(empty)* | **0/5 differ** from `opacity = 1.0` — the value is silently replaced by the default |
| `op="watermark"` `text="hi"` `opactiy=0.5` (misspelt) | **0** | 5 | *(empty)* | **0/5 differ** from `opacity = 1.0` |
| `op="watermark"` `text="hi"` `margin=-5` `tile="yes"` `size="big"` | **0** | 5 | *(empty)* | **0/5 differ** from `text="hi"` alone |
| `op="optimize"` `quality="max"` (terminal marker) | **0** | 1 | *(normal report)* | key ignored |

Controls, each proving the variable moves when well-typed: `opacity = 0.5` (float) vs
`opacity = 1.0` → **5/5 differ**; `margin = 5` `size = 64.0` vs defaults → **5/5 differ**;
self-diff of `opacity = 1.0` → 0/5. `build` (a one-target manifest over the same 5 inputs)
reproduces both halves: wrong-typed resize → exit 1, `target #0 (recipe r.toml, out dist)` +
the same line, 0 outputs; watermark `opacity = "0.5"` → exit 0, `built 1 target, 5 outputs`,
**0/5 differ** from `opacity = 1.0`.

So the user sees three things today:
1. **A watermark recipe that says something wrong runs anyway, with defaults, exit 0** — on every
   surface. The cause is `.unwrap_or(default)` over an accessor that conflates absent with
   wrong-typed (`src/operation/mod.rs:1337, 1341, 1360, 1365, 1379, 1380`). This path is
   SPEC-128's, merged to `main` but **not yet released** — fixing it before 0.8.0 means no user
   ever meets it.
2. **An unknown or misspelt key on any step is silently ignored.** This is a recorded accepted risk
   (DEC-061 verdict #1, pinned by `from_toml_tolerates_unknown_step_param_by_design`,
   `src/recipe/mod.rs:597`), which itself says a strict-validation spec flips it deliberately.
3. **Errors that do fire name neither the step nor the offending key** — `invalid operation
   'resize'` for a two-resize recipe does not say which one, and a misspelling is reported as a
   missing *correct* key. Precedents for good wording already exist in the repo: the top-level
   `deny_unknown_fields` error (`unknown field 'bogus', expected one of …`, with line/column) and
   `wasm::transform`'s `recipe step {index} ('{op}') needs its '{key}' asset …`
   (`src/wasm.rs:233-238`).

## Goal

A recipe step with an unknown key, a wrong-typed value or an out-of-range value is refused when
the recipe is loaded — before any input is decoded on `apply --recipe` and `build`, and before
any pipeline runs on wasm `transform` — with one stderr line naming the step index, the op and the
key (e.g. `error: recipe step 2 ('watermark'): 'opacity' must be a number in 0.0..=1.0, got the
string "0.5"`), exit code unchanged from today's recipe errors. Every recipe that is valid today
keeps parsing, producing the same pixels and (design to confirm) the same `to_toml()` bytes.

## Frame

**Verdict: GO, as one M spec** — with the premise corrected (above) and the scope pinned to the
user outcome, not to an internal storage refactor.

### Premise check
- **Late failure (input 400 of 500): WRONG.** Constructor errors already fire before any input on
  `apply` and `build` (cited above). Nothing in a malformed recipe fails per-input today.
- **Unknown key: silently ignored — confirmed**, on every op, the `optimize` marker included.
  This is the scope-changing answer the frame prompt asked for: the spec must reverse DEC-061
  verdict #1, which the design session records in DEC-102.
- **Wrong-typed value: split by op.** Required keys (`resize`'s `mode`/`width`/`height`/`percent`,
  watermark's `image`/`text`) fail at load with a misleading message. **Optional keys (watermark's
  `size`/`color`/`gravity`/`opacity`/`margin`/`tile`, and `scale` for the record) silently take
  their default** — the worst case in the table, wrong output at exit 0.
- **Out-of-range: mostly already caught at load** (resize `width = 0`, watermark `opacity`,
  `scale`, `size`), but `margin = -5` is wrong-typed-to-`u32` and therefore silently 0.

### Sizing — M, and why it is not L
Counting the verification, not the code:
- **Conditions × keys:** unknown key on 5 ops + the `optimize` marker (6 rows); wrong type on 14
  keys; out of range on the ~8 keys with a range. ≈ 30 rows — but they live in **one** function
  (op construction / step validation), so they are one table-driven unit test per op, not 30
  integration tests.
- **Surfaces:** `apply --recipe` (file path; bundled names are fixed text and need only a "still
  parses" row), `build`, wasm `transform`. All three already funnel through
  `Recipe::build_pipeline` (`src/cli/optimize.rs:73,136`, `src/cli/build.rs:298`,
  `src/wasm.rs:241`). So the surface axis needs **one integration test per surface** proving the
  error reaches the user with step + key, the documented exit code and zero outputs — not the full
  op × condition cross product per surface. The matrix is ~30 unit rows + ~4 surface rows +
  a round-trip corpus pin. That is M.
- **It becomes L only if** the design chooses to make typed structs the *serialized* form (struct
  → TOML), because then every recipe's `to_toml()` bytes, the cache key and the lockfile must be
  re-pinned across the corpus. **Recommendation: do not.** Validate into typed structs (or a
  per-op key schema) at load, and keep the ordered map as the canonical serialized form. That
  delivers the whole user outcome with zero DEC-005 round-trip risk. If the design finds typed
  storage necessary, that is the split point: "parse-time rejection" (this spec) vs "typed
  structs as storage" (a follow-up). No second stub is written here.

### DEC-005 round-trip risk
`Recipe::to_toml()` (`src/recipe/mod.rs:281`) serializes the *parsed* step map verbatim
(BTreeMap order, user-authored types), and `build` hashes that string into the cache key
(`src/cli/build.rs:107-137`, DEC-058) and the lockfile (DEC-101; SPEC-129 pinned that a
watermark-free recipe hashes exactly as before). If validation leaves the map untouched,
re-emission is byte-identical for every recipe that still parses — trivially achievable. What
does change: a recipe carrying an unknown key, which today parses, hashes (the key enters the
cache key: an inert `bogus` already forces a rebuild with identical output) and builds, will now
be refused — an intended break inside 0.8.0's single lockfile migration. If the design instead
re-emits from typed structs, three byte changes are expected and must be measured on the corpus:
integer-for-float (`opacity = 1` → `1.0`), default emission (absent `gravity` →
`gravity = "southeast"`?), and key order. `--save-recipe` (`src/cli/ops.rs:1089`) already emits
from `Operation::params()` and is a second, separate round-trip direction to keep green.

### Scope
**In:**
- Unknown key on any step → refused at load (all five ops and the terminal `optimize` marker).
- Wrong-typed value on any key → refused at load, never silently defaulted.
- Out-of-range values → refused at load (existing range checks keep firing; `margin` negative now
  caught).
- Error names step index (design picks 0- or 1-based and says why), op and key; wording for
  unknown / wrong-type / out-of-range.
- All three surfaces: `apply --recipe`, `build`, wasm `transform`. Bundled recipes keep parsing.
- `version = "1"` and `"2"` recipes both validated identically (params are not versioned today).
- Flip `from_toml_tolerates_unknown_step_param_by_design` deliberately; DEC-102 records the
  reversal of DEC-061 verdict #1.

**Out (and where it lives):**
- A wasm `validate(recipe_toml)` export returning structured errors — **does not exist today**;
  it is roadmap row #6 (recipe playground, `docs/roadmap.md:38`). **Needs filing** as a
  backlog item: "wasm `validate()` export returning structured, per-step recipe errors (playground
  prerequisite; consumes SPEC-131's errors)".
- "Did you mean `opacity`?" suggestions — nice-to-have, design's call whether it is cheap; if not,
  fold it into the `validate()` item above.
- Making typed structs the serialized form (see sizing) — follow-up only if design proves it
  necessary.
- Data-dependent runtime failures (e.g. an overlay larger than a given input) — not a malformed
  recipe; stays per-input.
- Watermark damage refusal / `--min-size` — SPEC-133.
- Asset files that do not exist — already refused at load by `resolve_recipe_assets`
  (`src/cli/common.rs:207`); unchanged.
- The direct CLI verbs (`resize`, `watermark` flags) — clap already types them.

### Rough build file list
- `src/operation/mod.rs` — `OperationParams` accessors (absent vs wrong-typed), `Resize::from_params`,
  `Watermark::from_params` (**collides with SPEC-133**).
- `src/operation/registry.rs` — per-op known-key schema / paramless ops reject keys.
- `src/recipe/mod.rs` — step index in errors, `RecipeError` variant(s), `optimize` marker keys,
  flipped pin test.
- `src/wasm.rs` — error text only, probably no logic.
- `src/cli/mod.rs` (where `CliError` wraps `RecipeError` and maps exit codes) — only if a new
  variant needs a mapping.
- Tests: `tests/recipe_round_trip.rs`, `tests/recipe_watermark.rs`, `tests/registry_seam.rs`,
  `tests/build.rs`, `tests/apply_batch.rs`, `tests/wasm_roundtrip.rs`, and a new strict-params test
  file.
- Docs: `recipes/README.md`, `docs/api-contract.md` (recipe error contract), `CHANGELOG.md`;
  `decisions/DEC-102-*.md` (new), a superseded-by note on `decisions/DEC-061-*.md`.

**Build order:** shares `src/operation/mod.rs` (`Watermark`) with SPEC-133 → **sequential with
SPEC-133**; recommend SPEC-131 first, so SPEC-133's `--min-size` recipe key (if it lands in
SPEC-133) is added under the strict schema rather than retrofitted. No file overlap expected with
SPEC-132 (`src/build/watch.rs`) or SPEC-134 (`src/sink/mod.rs`, CLI naming) — can run in
parallel with both, unless design routes the error through `src/cli/common.rs`.

### Corrections to `prompts/SPEC-131-design.md`
- It says a malformed recipe "is discovered when the op runs — on input 400 of 500". It is not:
  constructor errors already fire before any input (see Premise). The defect is silent
  acceptance, not late failure.
- It names "the wasm `validate()` surface in `src/wasm.rs`". **No such export exists** — the wasm
  surface that loads a recipe is `transform` (`src/wasm.rs:204`). `validate()` is out of scope
  (needs filing, above).
- "Before any image is decoded" holds for `apply`/`build` but not for wasm `transform`, which
  decodes `input` before it builds the pipeline (`src/wasm.rs:241`). Design decides whether to
  validate first there (cheap reorder) or to word that surface's AC as "before any pipeline runs".

### Open maintainer questions
- **Reverse DEC-061 verdict #1 (unknown step keys become errors)?** Recommend **yes** — the stage
  item and DEC-061 itself anticipate it, and a warning would be ignored in CI. Only the maintainer
  can retire a recorded accepted risk.
- **Should `apply --recipe` with an unknown key in a recipe that worked on 0.7.x print a hint
  that this is new in 0.8.0?** Recommend **no** — a CHANGELOG line under "Breaking" is enough;
  the error already names the key.

## Inputs

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth>
- **Related code paths:** `src/some/module/`

## Outputs

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New exports:** <names and signatures>
- **Database changes:** <migrations>

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- A wasm `validate(recipe_toml)` export — needs filing (roadmap #6, recipe playground).
- "Did you mean …?" key suggestions, unless design finds them cheap.
- Typed structs as the *serialized* form of a step (struct → TOML) — follow-up if ever needed.
- Data-dependent per-input runtime failures — not a malformed recipe.
- Watermark damage refusal and `--min-size` — SPEC-133.
- Missing asset files — already refused at load by `resolve_recipe_assets`; unchanged.
- The direct CLI verbs' flags — clap already types them.

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Not "src/foo and tests": the enumeration is what a verify cycle diffs
  the real change against, and what tells you whether every affected `DEC-*` has
  the path in its `affected_scope`. SPEC-122's Deviations claimed "`src/operation`
  and `tests/` only" and was wrong by two `scripts/` files, which left
  `affected_scope` blind; SPEC-124's build added this section by hand after that
  lesson and still listed 8 of its 9 files. It is a checklist because it is easy
  to get wrong from memory — build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
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
