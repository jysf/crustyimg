---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-133
  type: bug                        # epic | story | task | bug | chore
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
    - DEC-003
    - DEC-015
    - DEC-031
    - DEC-032
    - DEC-045
    - DEC-057
    - DEC-058
    - DEC-059
    - DEC-064
    - DEC-085
    - DEC-100
    - DEC-101
  constraints:
    - test-before-implementation
    - one-spec-per-pr
    - no-unwrap-on-recoverable-paths
  related_specs:
    - SPEC-029
    - SPEC-128
    - SPEC-129
    - SPEC-131
    - SPEC-134

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-050's <capability>". Optional; null is acceptable.
value_link: >
  Delivers STAGE-050's "a watermark that refuses to wreck an image, and skips one it cannot
  sensibly mark" — without it, one `build` over a mixed site silently writes clipped, ruined or
  unmarked-but-reported-as-marked thumbnails at exit 0.

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
      tokens_total: 5752030
      tokens_breakdown:
        input: 80
        output: 36599
        cache_creation: 156746
        cache_read: 5558605
      estimated_usd: 4.67
      rate_anchors: "Opus $5/$25 per MTok; cache_creation x1.25 input; cache_read x0.10 input"
      duration_minutes: null
      recorded_at: 2026-10-24
      note: >
        Interactive frame session in its own worktree (docs/spec-133-frame). Measured from the
        session transcript 64717781-9d83-4696-909c-b09ce170e311.jsonl, found by content
        ("SPEC-133 — FRAME prompt"), 40 assistant messages deduped by .message.id (input /
        cache_creation / cache_read taken once per id, MAX output_tokens). Measured just before
        the frame commit, so the final commit/push/read-back turns are not included.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-133: watermark refuses to damage an image

## Context

Two STAGE-050 backlog items, framed as one spec because the fit test is the first item's fix:
"[S] ⚡ **`watermark --text` overruns the right edge**" and "[S] ⚡ **`watermark` stops silently
doing nothing.**" The maintainer ruled **option D** on 2026-10-24 (settled — see the stage entry):
refuse when the rendered mark plus margin does not fit the image, **and** refuse when the mark
changes more than **5 %** of the pixels. Single explicit input → error, exit 2, naming the numbers.
Batch / recipe / `build` → skip that image, report it, keep going, never abort. Measure
geometrically (count changed pixels), never SSIMULACRA2.

**What the user sees today** — re-driven at frame on a release build of `main` @ `8471fb1`
(`crustyimg 0.7.1`). Base images are `tests/fixtures/classify/color_photo_fuji.png` resized with
`resize --exact NxN`; every output is converted to BMP with `convert --format bmp` and diffed
pixel by pixel (RGB) against the converted base. Negative control: each base diffed against itself
→ **0 px** at 24 px and at 256 px.

| row | command (abridged) | exit | changed px | changed-pixel bbox |
|---|---|---:|---:|---|
| control | base vs itself, 24 px and 256 px | — | **0** | none |
| **A** | `watermark b24.png --text "© crustyimg" --size 200` | **0** | **6 (1.04 %)** | 3×3 at (21,21) |
| A, recipe | same via `apply --recipe` (`text`, `size = 200.0`) | **0** | **6 (1.04 %)** | 3×3 at (21,21) |
| A+ | `watermark b24.png --text "© crustyimg"` (default size) | 0 | 261 (45.31 %) | 24×24 at (0,0) |
| **B** | `watermark b256.png --text "© 2026 crustyimg all rights reserved"` | **0** | **2446 (3.73 %)** | 248×31 at **x=0** |
| B′ | `watermark b256.png --text "© crustyimg"` (fits) | 0 | 1647 (2.51 %) | 168×31 at (87,225) |
| C | `watermark b24.png --image b256.png` (overlay > canvas) | **0** | 574 (99.65 %) | 24×24 at (0,0) |
| D | `apply --recipe` with `text = "x"`, `tile = true` on 256 px | 0 | 34536 (52.70 %) | whole canvas |
| E | `apply --recipe` with `text = "x"`, `scale = 0.9` on 256 px | 0 | 36851 (56.23 %) | 230×244 |
| F | `watermark in/b24.png in/b256.png --text … --size 200 --out-dir out` | **0** | — | both files written, no warning |

Rows A, A+, B and B′ match the stage entry's 2026-10-24 re-drive to the hundredth. Row B is the
clipping case: the rendered run is wider than 256 px, so `Gravity::placement` saturates x to 0
(`src/operation/mod.rs:1117-1139`, `saturating_sub` at :1121) and `imageops::overlay` silently
cuts the right side off — the "all rights reserved" clause is gone at 3.73 % coverage, which a 5 %
cap alone would pass. Row A is the silent no-op: a ~1,050 px run on a 24 px canvas leaves 6
pixels changed, exit 0.

## Goal

When a watermark cannot sensibly be applied — the mark plus margin does not fit, or it would change
more than 5 % of the image — `crustyimg` says so instead of writing damage at exit 0. On one
explicit input: `error: …` naming the mark's size, the image's size (and coverage %), **exit 2**.
On a batch, a multi-input `apply --recipe` or a `build`: one stderr line per skipped image, a skip
count in the run's summary, the run completes, and the image's other recipe steps still apply.

## Frame

**Verdict: GO, as one M spec — at the top of M, with `--min-size` split out.** Premise confirmed.

### Premise check
- **Confirmed, exactly.** Rows A and B reproduce on `main` with a 0 px self-diff control (table
  above). The row carrying each argument uses the feature: A uses `--size 200`, B the long string.
- **Wider than the backlog says — image overlays have the same failure.** Row C: a 256 px overlay
  on a 24 px base changes 99.65 % at exit 0. The fit test must cover image mode, not only `--text`.
- **New: the recipe accepts mode combinations the CLI refuses.** clap rejects `--text` with
  `--tile` or `--scale` (`src/cli/mod.rs:422-427`, `conflicts_with = "text"`;
  "the argument '--text <TEXT>' cannot be used with '--tile'"), but `Watermark::from_params`
  reads `scale`/`tile` regardless of mode (`src/operation/mod.rs:1372-1380`), so rows D and E run
  at exit 0 with 52.7 % / 56.2 % coverage. This matters to the damage rule: if `--tile` is exempt
  from coverage (the likely design), a text recipe could bypass the cap through `tile`. SPEC-131
  (typed params) adds a per-op key schema but does not claim cross-mode validation (its scope list,
  `SPEC-131-…md` "Scope"). **In scope here** — one check in `from_params`.
- **Where the rule can live.** `Watermark::apply` (`src/operation/mod.rs:1441`) holds the canvas
  size and the post-`scale` overlay size before either `imageops::overlay` call (:1495, :1546), so
  fit and coverage can both be decided **before compositing**, in pure code (DEC-031 /
  DEC-064 stay intact). Coverage is the overlay's non-transparent pixel count over the canvas area
  — no second render (the stage entry's 1647 px constant is exactly this for the default string).
- **But `Operation::apply` can only return an image or an error** (`src/operation/mod.rs:187`).
  There is no "applied, but skipped" signal. The single-input lanes can use the error directly.
  The batch lanes cannot: today a per-input error is `failed += 1` → `CliError::PartialBatch` →
  **exit 6**, and in `build` it also means **the lockfile is never written**
  (`src/cli/build.rs:791`, :827-829: "A partial build has nothing coherent to lock"). Treating a
  refusal as an ordinary failure would break the ruling's "never abort" for the normal case — a
  mixed site directory could never be locked. So the batch lanes need a third outcome, *skipped*,
  distinct from *failed*. That is the bulk of this spec.

### Sizing — M (top of M), and why
The code is small: one predicate, one error variant, one exit mapping, one `from_params` check.
**The verification is what sizes it.** The surfaces a watermark runs through are five separate
driver loops plus wasm, each with its own error handling:

| surface | code path | per-input error today |
|---|---|---|
| `watermark`, 1 input | `run_watermark` → `run_pixel_op` single (`src/cli/ops.rs:1294`, :369) | `CliError::Operation` → exit **1** (`src/cli/mod.rs:757`) |
| `watermark`, N inputs | `run_pixel_op` loop (`src/cli/ops.rs:443-519`) | exit 6 |
| `apply --recipe`, plain, 1 / N | `run_apply` (`src/cli/optimize.rs:175`; rayon batch :291-293) | exit 1 / 6 |
| `apply --recipe` ending in `optimize`, pinned | `run_pixel_op` (`src/cli/optimize.rs:98`) | as `watermark` |
| `apply --recipe` ending in `optimize`, auto-decide | `run_optimize_autodecide` (`src/cli/optimize.rs:1535`, :1618-1650) | exit 6 |
| `build` | `build_one` → `encode_one` / `encode_one_optimize_decided` (`src/cli/build.rs:512`, `src/cli/common.rs:44`) | exit 6, **no lockfile** |
| wasm `transform` (text-only recipes) | `src/wasm.rs:204`, :242 | `JsError` (single input by construction) |

It stays M **only if the matrix is additive**: the predicates (fit-width, fit-height, coverage,
text vs image, tile, margin, center gravity, scale) are unit-tested once on the op, and each
surface gets **one** refusal row proving the lane behaviour (exit 2 on one input; skip + report +
continue on many; `build` writes its lockfile). Predicate × surface would be ~8 × 7 and is L.
Shared code (the pipeline, every batch driver, `build`) means verify must run a clean full matrix
(a stale incremental build is a false green here).

`build` carries the subtle part: a **cache hit never runs the pipeline**
(`src/cli/build.rs:533-541`), so a skip recorded only at run time would be reported on the first
build and silently absent on every later one. Design must decide how a cached, skipped target is
reported (and, depending on the "skip" ruling below, what the cache stores and what the lockfile
records).

**`--min-size` is out** (see Scope) — with it the spec is L: a new CLI flag on `watermark`, a new
recipe key (coupled to SPEC-131's typed schema), its own single-input semantics the ruling does not
state, and its own row on every surface.

**Fallback split if design finds it L anyway:** (a) predicate + single-input refusal on every
surface + wasm; (b) the batch/`build` *skipped* outcome. (a) alone would leave a mixed batch at
exit 6 and an unlocked `build` on `main` until (b) lands — tolerable only because PROJ-011 ships
as one release. No second stub is written here.

### Scope
**In:**
- Fit test (no threshold): the mark plus margin on the anchored side(s) must fit the canvas on
  both axes — text **and** image overlays, after `scale`. Centre gravity ignores margin, as
  placement already does (`src/operation/mod.rs:1113`).
- Coverage cap: refuse above 5 % of canvas pixels, measured geometrically before compositing.
- `--tile`: a defined rule (design call; see the design-prompt notes below).
- Recipe `text` mode rejects `tile` and `scale`, matching the CLI (rows D/E) — at load, so it is a
  recipe error (exit as SPEC-131 defines), not a per-input refusal.
- Single explicit input → error, **exit 2**, naming the numbers — on `watermark`, `apply --recipe`
  (every branch), and wasm `transform` (a `JsError` carrying the same message).
- Multi-input `watermark`, multi-input `apply --recipe` (plain and terminal-`optimize`, pinned and
  auto-decide), and `build` → skip that image, one stderr line naming it and the numbers, a skip
  count in the summary, never exit 6 for a skip alone, and `build` still writes / checks its
  lockfile.
- No behaviour change for a watermark that fits under the cap: output byte-identical to `main`
  (e.g. row B′ and the 800 px default).
- Docs: `docs/api-contract.md` (exit 2 for a refused single input; skip reporting in batches),
  the `watermark` help text / `recipes/README.md` where they describe the op, `CHANGELOG.md`.

**Out (and where it lives):**
- **`--min-size`, the opt-in value rule** (2026-08-23 ruling, settable in a recipe) — **filed as
  its own STAGE-050 backlog item** by this frame. It reuses this spec's *skipped* outcome, and
  landing it after SPEC-131 puts its recipe key straight into the strict schema. ⚠ SPEC-131's
  out-of-scope list says "Watermark damage refusal / `--min-size` — SPEC-133"; the `--min-size`
  half of that pointer now means the new backlog item.
- A relative `--size` (percent of the image) — the stage's Design Notes raise it; not ruled, not
  requested. Needs filing only if the maintainer wants it.
- Making the 5 % a user knob — the ruling's default is fixed; design may argue otherwise.
- Any change to placement, compositing, opacity or rendering of a mark that fits.
- `build --watch` asset watching — SPEC-132.
- Anything SSIMULACRA2-based — ruled out.

### Rough build file list
- `src/operation/mod.rs` — fit + coverage predicate in `Watermark::apply` (both bit-depth
  branches, :1453 / :1504), a typed refusal variant on `OperationError`, the text-mode
  `tile`/`scale` rejection in `Watermark::from_params`. **Collides with SPEC-131.**
- `src/pipeline/mod.rs` — only if design carries *skipped* as a pipeline outcome (needed if a
  skipped image is still written with its other steps applied).
- `src/cli/mod.rs` — exit mapping for the refusal (exit 2), possibly a skip-count error/summary
  type.
- `src/cli/ops.rs` — `run_pixel_op` single + batch.
- `src/cli/optimize.rs` — `run_apply` single + batch, `run_optimize_autodecide`.
- `src/cli/common.rs` — `encode_one` / `encode_one_optimize_decided` (the `build` path).
- `src/cli/build.rs` — a skipped outcome alongside cached/rebuilt, summary line, lockfile phase,
  cache-hit reporting.
- `src/build/cache.rs` / `src/build/lock.rs` — only if a skip must be stored or recorded.
- `src/wasm.rs` — probably no logic (the error path already becomes a `JsError`); a test.
- `src/text/mod.rs` — probably none; `render_text` already returns the overlay whose size is the
  extent.
- Tests: a new `tests/watermark_damage.rs` (or similar); touches to `tests/recipe_watermark.rs`,
  `tests/build.rs`, `tests/apply_batch.rs`, `tests/wasm_roundtrip.rs`; op unit tests in
  `src/operation/mod.rs`.
- Docs: `docs/api-contract.md`, `recipes/README.md`, `CHANGELOG.md`, `decisions/DEC-104-*.md`.

**Build order:** shares `src/operation/mod.rs` (`Watermark::from_params`) with **SPEC-131 → run
sequentially, SPEC-131 first.** Also overlaps **SPEC-134** (`--name-template` pins format) —
checked against SPEC-134's frame list: both touch `src/cli/ops.rs` (`run_pixel_op`),
`src/cli/optimize.rs` (`run_apply`), `src/cli/common.rs` (`encode_one`), `src/cli/build.rs`,
`tests/apply_batch.rs`, `tests/build.rs` and `docs/api-contract.md`. **Do not run SPEC-133 and
SPEC-134 builds in parallel.** No overlap expected with SPEC-132 (`src/build/watch.rs`).

**Recipe-visible parameters:** this spec adds **none** (5 % fixed, `--min-size` out). It does make
one existing combination an error (`text` with `tile`/`scale`), which SPEC-131's strict schema must
not re-permit.

### Maintainer rulings (2026-10-25) — both questions below are settled
1. **A skipped image is written without the mark**, with the recipe's other steps applied; it is
   never left unwritten, and a skip is not a failure (no exit 6, `build` still writes its lockfile).
2. **A batch or `build` whose only problem is skips exits 0**, with one stderr line per skipped
   image and a skip count in the summary (precedent: DEC-085).

### Open maintainer questions (as asked at frame)
1. **"Skip" — is a skipped image written without the mark, or not written at all?** The ruling's
   word does not settle it. **Recommend: written without the mark** (its other recipe steps still
   applied), reported on stderr. Why: the case the ruling protects is one `build` over a whole
   site; a thumbnail that is *not written* is a broken `<img>` on the page and a hole in the
   lockfile, while the maintainer's own rationale — "small images don't make sense to watermark" —
   says the image needs no mark, not that it should not exist. It also keeps `build`'s cache and
   lockfile simple (the unmarked bytes are deterministic for that source + recipe, so they cache
   and lock like any other output), and it avoids leaving a stale output from an earlier run in
   place. Cost: the pipeline needs a "skipped" outcome so the remaining steps still run.
2. **Exit code for a batch whose only problem is skips.** Recommend **0**, with the skip count in
   the summary — exit 6 would fail CI on the normal mixed-directory case and, in `build`, block the
   lockfile. Precedent: DEC-085 (truncated JPEG warns on stderr, exit stays 0). If the maintainer
   wants skips machine-visible, a distinct non-zero code is the alternative; 6 is not.

### Corrections / additions for `prompts/SPEC-133-design.md`
- Its surface list ("`watermark` single + batch, `apply --recipe` single + batch, `build`") misses
  **`apply --recipe` ending in `optimize`** (two more loops: pinned via `run_pixel_op`, auto-decide
  via `run_optimize_autodecide`) and **wasm `transform`**, which runs text-only watermark recipes.
- It frames the evidence as text-only; row C shows **image overlays clip too**, so fit applies to
  both modes. Coverage for image mode stays a design call (the ruling's wording is about text);
  frame's suggestion: coverage applies whenever the mark's size is absolute (text, unscaled image);
  a `scale`d image is proportional by construction, so a cap there validates the user's chosen
  scale on every input identically rather than detecting damage per image.
- `--tile` suggestion: exempt from coverage (it covers the image by design), but one tile must
  still fit the canvas — otherwise every tile is clipped, which is the fit failure again. Only
  image mode can tile once text+tile is rejected (above).
- Add: `build`'s cache-hit path skips the pipeline, so skip reporting must survive a cached run.
- Add: the recipe/CLI mode mismatch (rows D/E) is in scope.
- `--min-size` is out of this spec (filed separately); the prompt's "decide whether it belongs"
  is answered.

## Inputs

- **Files to read:** see `## Frame` → Rough build file list.
- **Related code paths:** `src/operation/mod.rs` (`Watermark`, `Gravity::placement`),
  `src/cli/{ops,optimize,build,common}.rs`, `src/wasm.rs`, `src/text/mod.rs`.

## Outputs

- **Files created:** *(design)*
- **Files modified:** *(design)*
- **New exports:** *(design)*

## Acceptance Criteria

*Written at design.*

- [ ] Criterion 1 (testable)

## Failing Tests

*Written at design, BEFORE build.*

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*Written at design.*

### Decisions that apply

- `DEC-NNN` — *(design)*

### Constraints that apply

- `constraint-id` — *(design)*

### Prior related work

- `SPEC-029` (shipped) — the `Watermark` op and `Gravity::placement`.
- `SPEC-128` (shipped) — watermark reachable from recipes / `build` (DEC-100).
- `SPEC-129` (shipped) — `build` cache key hashes asset content (DEC-101).

### Out of scope (for this spec specifically)

- `--min-size`, the opt-in value rule — its own STAGE-050 backlog item (filed by this frame).
- A relative / percent `--size` — not ruled; needs filing only if wanted.
- A user-settable coverage threshold — fixed 5 % unless design justifies a knob.
- Any change to how a mark that fits is placed, composited or rendered.
- `build --watch` watching asset paths — SPEC-132.
- Typed per-op params / unknown-key rejection — SPEC-131 (this spec only adds the text-mode
  `tile`/`scale` rejection).
- SSIMULACRA2 or any perceptual measure — ruled out.

## Notes for the Implementer

*(design)*

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
