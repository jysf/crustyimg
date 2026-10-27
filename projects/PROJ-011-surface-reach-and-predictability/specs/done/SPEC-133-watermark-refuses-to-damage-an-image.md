---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-133
  type: bug                        # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
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
    - DEC-002
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
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 13392428
      tokens_breakdown:
        input: 144
        output: 82288
        cache_creation: 240587
        cache_read: 13069409
      estimated_usd: 10.10
      rate_anchors: "Opus $5/$25 per MTok; cache_creation x1.25 input; cache_read x0.10 input"
      duration_minutes: null
      recorded_at: 2026-10-25
      note: >
        Interactive design session in its own worktree (docs/spec-133-design). Measured from the
        session transcript b3fea33a-f11d-4abc-9f8a-fae74fb5cf19.jsonl, found by content
        ("SPEC-133 — DESIGN prompt"), 72 assistant messages deduped by .message.id (input /
        cache_creation / cache_read taken once per id, MAX output_tokens). Includes a release
        build of main and a full prototype test run (both in the scratchpad, never committed).
        Measured just before the design commit, so the final commit/push/report turns are not
        included.
    - cycle: build
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 40067139
      tokens_breakdown:
        input: 338
        output: 122479
        cache_creation: 307763
        cache_read: 39636559
      estimated_usd: 24.81
      rate_anchors: "Opus $5/$25 per MTok; cache_creation x1.25 input; cache_read x0.10 input"
      duration_minutes: null
      recorded_at: 2026-10-26
      note: >
        Interactive build session in its own worktree (feat/spec-133-watermark-damage-rule).
        Measured from the session transcript 7ade2af7-0cc3-48b8-a9f9-6ddf8605ce1f.jsonl, found
        by content ("SPEC-133-build.md exactly"), 164 assistant messages deduped by .message.id
        (input / cache_creation / cache_read taken once per id, MAX output_tokens), every
        message claude-opus-5-5. Re-measured at ship from the full transcript (was 35328702
        tokens / $22.18, measured before the PR). Includes the clean three-leg matrix,
        wasm-test x3 and demo-build.
    - cycle: build
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 8944578
      tokens_breakdown:
        input: 160
        output: 32318
        cache_creation: 120307
        cache_read: 8791793
      estimated_usd: 3.57
      rate_anchors: "Sonnet $3/$15 per MTok; cache_creation x1.25 input; cache_read x0.10 input"
      duration_minutes: null
      recorded_at: 2026-10-26
      note: >
        Fix session for the verify punch list (same branch, draft PR #206). Measured from the
        session transcript e832cf11-e841-4f2a-b89b-102d7a362133.jsonl, found by content
        ("SPEC-133 — FIX prompt"), 78 assistant messages deduped by .message.id (input /
        cache_creation / cache_read taken once per id, MAX output_tokens), every message
        claude-sonnet-5-5. Re-measured at ship from the full transcript (was 6455755 tokens /
        $2.72, measured before the final turns); now also covers the follow-up turn that fixed
        docs/data-model.md. Includes the clean three-leg matrix, wasm-test and demo-build.
    - cycle: verify
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 17032820
      tokens_breakdown:
        input: 188
        output: 70043
        cache_creation: 214034
        cache_read: 16748555
      estimated_usd: 11.46
      rate_anchors: "Opus $5/$25 per MTok; cache_creation x1.25 input; cache_read x0.10 input"
      duration_minutes: null
      recorded_at: 2026-10-27
      note: >
        Verify session (worktree verify-spec-133, verdict PUNCH LIST at 3dd23ff). Measured from
        transcript a446ba7b-e3b1-406c-9f79-07a2e1fceef2.jsonl, found by content
        ("SPEC-133 — VERIFY prompt"), 94 assistant messages deduped by .message.id, every message
        claude-opus-5-5. Measured at ship from the full transcript (the session reported $11.16).
    - cycle: build
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 569730
      tokens_breakdown:
        input: 18
        output: 2435
        cache_creation: 41508
        cache_read: 525769
      estimated_usd: 0.58
      rate_anchors: "Opus $5/$25 per MTok; cache_creation x1.25 input; cache_read x0.10 input"
      duration_minutes: null
      recorded_at: 2026-10-27
      note: >
        Aborted fix: the fix prompt was pasted into the verify worktree by mistake, then /exit.
        Nothing was built. Measured from transcript e07e474f-02c1-47f0-bc0e-1c4796a3e0bb.jsonl,
        found by content ("SPEC-133 — FIX prompt"), 8 assistant messages deduped by
        .message.id, every message claude-opus-5-5.
    - cycle: verify
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 4438803
      tokens_breakdown:
        input: 86
        output: 25460
        cache_creation: 101423
        cache_read: 4311834
      estimated_usd: 3.43
      rate_anchors: "Opus $5/$25 per MTok; cache_creation x1.25 input; cache_read x0.10 input"
      duration_minutes: null
      recorded_at: 2026-10-27
      note: >
        Re-verify session (worktree verify-spec-133-fix, verdict APPROVED at f1a46d3). Measured
        from transcript 07fa8377-e921-49c4-aeac-317440893ed4.jsonl, found by content
        ("SPEC-133 — RE-VERIFY prompt"), 43 assistant messages deduped by .message.id, every
        message claude-opus-5-5. Measured at ship from the full transcript (the session
        reported $3.22).
    - cycle: ship
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 1041586
      tokens_breakdown:
        input: 26
        output: 8568
        cache_creation: 59333
        cache_read: 973659
      estimated_usd: 0.64
      rate_anchors: "Sonnet $3/$15 per MTok; cache_creation x1.25 input; cache_read x0.10 input"
      duration_minutes: null
      recorded_at: 2026-10-27
      note: >
        Ship session (worktree docs/spec-133-ship). Measured from its own transcript, found by
        content ("SPEC-133 — SHIP prompt"), deduped by .message.id, every message
        claude-sonnet-5-5. Measured just before the ship commit, so the final commit/push/report
        turns are not included.
  totals:
    tokens_total: 91239114
    estimated_usd: 59.26
    session_count: 8
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

### Maintainer rulings on the design (2026-10-25)
3. **Keep** the 5 % cap on unscaled image overlays (a 64×64 opaque logo on 256×256 is refused).
4. Orchestrator (no product change): the cached-skip-line gap is accepted as documented (AC-23),
   fixed by its own item; SPEC-133's build edits SPEC-131's two affected test rows (Call 5).

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

## Design (2026-10-25)

### What the design measured on top of the frame

Driven on a release build of `main` at `e917380` (`crustyimg 0.7.1`), same method as the frame:
bases are `tests/fixtures/classify/color_photo_fuji.png` through `resize --exact`; every output
and its base are converted with `convert --format bmp` and diffed pixel by pixel (RGB).
**Negative control: each base diffed against itself → 0 px at 24 px and at 256 px.** Script:
the design session's scratchpad `drive.sh`; reproducible from this table alone.

| row | command (abridged) | exit | changed px | bbox |
|---|---|---:|---:|---|
| control | base vs itself, 24 / 256 px | — | **0** | none |
| A | `watermark b24 --text "© crustyimg" --size 200` | 0 | 6 (1.04 %) | 3×3 at (21,21) |
| A, recipe | same via `apply --recipe` (`size = 200.0`) | 0 | 6 (1.04 %) | 3×3 at (21,21) |
| B | `watermark b256 --text "© 2026 crustyimg all rights reserved"` | 0 | 2446 (3.73 %) | 248×31 at **(0,225)** |
| B′ | `watermark b256 --text "© crustyimg"` | 0 | 1647 (2.51 %) | 168×31 at (87,225) |
| C | `watermark b24 --image b256.png` | 0 | 574 (99.65 %) | 24×24 at (0,0) |
| D | recipe `text = "x"`, `tile = true`, on 256 | 0 | 34536 (52.70 %) | whole canvas |
| E | recipe `text = "x"`, `scale = 0.9`, on 256 | 0 | 36851 (56.23 %) | 230×244 |
| F | `watermark b24 b256 --text … --size 200 --out-dir` | 0 | — | both written, empty stderr |
| **CNT** | `watermark black800 --text "© crustyimg"` (white on solid black) | 0 | **1661** | 168×31 |
| K172 | default text on 172 px | 0 | 1647 (5.57 %) | 168×31 at (3,141) |
| K192 | default text on 192 px | 0 | 1647 (4.47 %) | 168×31 at (23,161) |
| J800 | default text on 800×600 | 0 | 1647 (0.34 %) | 168×31 at (631,569) |
| M | `watermark b256 --text "© crustyimg" --margin 100` | 0 | 1647 | 168×31 at **(0,125)** — margin silently not honoured |
| G | `watermark b256 --image logo64 --tile` | 0 | 65536 (100 %) | whole canvas |
| G3 | `watermark b256 --image logo64` (opaque 64×64) | 0 | 4096 (6.25 %) | 64×64 |
| H | `watermark b24 --image b256.png --scale 0.5` | 0 | 141 (24.48 %) | 12×12 at (12,12) |
| I | `watermark b24 --image b256.png --tile` | 0 | 574 (99.65 %) | same bytes as C |

Every frame row (A–F) reproduced exactly. What the new rows add:

1. **"Changed pixels" depends on the image, not just the mark (row CNT).** The same white mark
   changes 1661 px on solid black but 1647 on the photo — 14 glyph pixels land on photo pixels
   that were already white. A rule built on changed pixels would refuse a mark on a dark image
   and pass the same mark on a light one. **So coverage counts the mark's own non-transparent
   pixels** (≥ every pixel it can change), which depends only on the mark and the image's size.
   This corrects the frame's note that "1647 px is exactly" the overlay count. On that count the
   default string is a 169×31 mark with 1661 non-transparent px (read from the prototype):
   172 px → 5.61 % (refused), 192 px → 4.51 % (passes). Row A's mark is 1050×189.
2. **Margin is a fit failure the frame's table did not show (row M).** `placement` saturates the
   right inset to 0, so a 100 px margin on a 256 px image puts the mark flush left, at exit 0.
3. **A tile larger than the image is row C again (row I, byte-identical output).** So "one tile
   must fit" is a real rule, not a formality.
4. **A scaled overlay can be proportionate and still large (row H, 24.5 %).** That is the user's
   chosen `scale` doing what it says on every image; capping it would refuse every image at
   `scale ≥ 0.23` regardless of size. Scale stays coverage-exempt.

**Prototype of the rule against the existing suite** (throwaway, in a scratch worktree, never
committed): the fit + coverage predicate below, added to `Watermark::apply` as a plain error, then
`cargo test --no-fail-fast` (default features). This is how the "Existing tests that change"
list below was built — measured, not guessed.

### Call 1 — the rule (one predicate, pure geometry)

Decided once per image, in `Watermark`, from three things only: the image entering the step
(`bw×bh`), the mark's size **after `scale`** (`mw×mh`, computed by the same arithmetic `apply`
uses — extract it into one helper both bit-depth branches call, so the check and the composite
cannot disagree), and the mark's non-transparent pixel count. Nothing is drawn first.

1. **Fit (always).** Effective margins: `mx = margin` when gravity anchors a left/right edge
   (`east`, `west`, any corner), else 0; `my = margin` when it anchors a top/bottom edge
   (`north`, `south`, any corner), else 0 — exactly the axes `Gravity::placement` insets. `center`
   uses neither; `tile` uses neither (it ignores margin). **Refuse when `mw + mx > bw` or
   `mh + my > bh`** (`u64` arithmetic). Equality fits. For `tile` this is "one tile fits".
2. **Coverage (only when the mark's size is fixed: not `tile`, no `scale`).** `covered` = the
   number of the mark's pixels with alpha > 0, at the overlay's own bit depth (count on
   `to_rgba16()`, which is exact for 8- and 16-bit overlays), **before** `opacity` is applied —
   opacity changes a mark's strength, not its footprint. **Refuse when
   `covered × 100 > bw × bh × 5`** (integer; exactly 5 % passes). The cap is a `const` (5), not a
   setting (frame scope).
3. **Order.** Fit first; if both fail, the fit refusal is reported.

The count does not depend on the image, so `Watermark` stores it at construction (`new_image`,
`new_text`, `from_params`) rather than rescanning the overlay per image.

**What the rule does to the measured rows:** A, A+, B, C, I, M → refused (fit). K172, G3 →
refused (coverage). B′, K192, the 800 px default, G (tile), H (scaled) → applied, byte-identical
to `main`.

**Rejected:** changed-pixel counting (measured above: content-dependent); SSIMULACRA2 (ruled out);
a glyph-bounding-box fit (the text overlay is already tight to the glyphs — `render_text` sizes
its canvas to the union of glyph bounds, `src/text/mod.rs:126-130,197-199` — and an image overlay's
transparent border is the user's own choice of file, which `placement` already treats as part of
the mark); coverage on a scaled image (finding 4); applying margin to `tile`/`center` (they ignore
it when placing, so the check must too).

### Call 2 — how the op says "refused", and how each surface turns that into a result

`apply()` can only return an image or an error, and an error consumes the image — so a skipping
caller would lose the pixels it needs to keep going. **The decision is exposed before `apply`, as
one defaulted trait method, and the pipeline chooses what to do with it:**

```rust
// src/operation/mod.rs (illustrative — names may differ, behaviour may not)
pub const WATERMARK_COVERAGE_CAP_PERCENT: u64 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The mark (plus the margin on the anchored axes) is larger than the image.
    DoesNotFit { mark: (u32, u32), margin: u32, image: (u32, u32) },
    /// The mark would cover more than the cap of the image's pixels.
    CoverageAboveCap { covered: u64, image: (u32, u32) },
}
// Display:
//   the 1050×189 px mark does not fit inside the 24×24 px image
//   the 169×31 px mark plus its 100 px margin does not fit inside the 256×256 px image
//   the mark would cover 5.61 % of the 172×172 px image (1661 of 29584 px); the limit is 5 %

pub enum OperationError {
    Apply { .. },                                   // unchanged
    #[error("{op} refused: {refusal}")]
    Refused { op: &'static str, refusal: Refusal }, // new
}

pub trait Operation {
    // name / params / apply unchanged
    /// Whether this operation declines to run on `img`, and why. Pure; checked
    /// before `apply`. The default never declines.
    fn refusal(&self, _img: &Image) -> Option<Refusal> { None }
}
```

- `Watermark::refusal` is **the one decision point** (Call 1). `Watermark::apply` calls it first
  and returns `Refused` — so any direct caller of `apply` is refused too, and nothing is drawn.
- `margin` in `DoesNotFit` is the effective margin (0 for `tile`/`center`); Display omits the
  "plus its N px margin" clause when it is 0.

```rust
// src/pipeline/mod.rs
pub enum OnRefusal { Fail, Skip }
pub struct SkippedStep { pub step: usize, pub op: &'static str, pub refusal: Refusal }
pub struct PipelineRun { pub image: Image, pub skipped: Vec<SkippedStep> }

impl Pipeline {
    /// Unchanged signature and meaning: a refusal is an error (Fail).
    pub fn run(&self, img: Image) -> Result<Image, OperationError>;
    /// Fail: as `run`. Skip: a step whose `refusal` is Some is not applied, is
    /// recorded, and the remaining steps still run on the unchanged image.
    pub fn run_with(&self, img: Image, on_refusal: OnRefusal) -> Result<PipelineRun, OperationError>;
}
```

`run` stays the default, so every caller that is not touched below keeps today's behaviour plus
the refusal-as-error (wasm, single inputs). **Which policy each driver uses — the whole surface
table:**

| surface | code path | policy | user sees on a refused image |
|---|---|---|---|
| `watermark`, 1 input | `run_pixel_op` single (`src/cli/ops.rs:369`) | `run` (Fail) | `error: watermark refused: …`, **exit 2**, nothing written |
| `watermark`, N inputs | `run_pixel_op` batch (`:469`) | Skip | written unmarked + warning line; exit 0 |
| `apply --recipe`, plain, 1 | `run_apply` single (`src/cli/optimize.rs:175`) | Fail | exit 2 |
| `apply --recipe`, plain, N | `apply_one` → `encode_one` (`src/cli/common.rs:61`) | Skip | written unmarked + warning; exit 0 |
| `apply --recipe` + `optimize`, pinned, 1 / N | `run_pixel_op` (`src/cli/optimize.rs:98`) | Fail / Skip | as `watermark` |
| `apply --recipe` + `optimize`, auto-decide, 1 / N | `run_optimize_autodecide` → `optimize_decide_one` (`:1185`) | Fail / Skip | as above |
| `build`, plans Preserve / Pinned | `build_one` → `encode_one` | Skip | written unmarked + warning; exit 0; lockfile written |
| `build`, plan Decide | `build_one` → `encode_one_optimize_decided` → `optimize_decide_one` | Skip | same |
| wasm `transform` | `src/wasm.rs:242` `pipeline.run` | Fail | `JsError` text `watermark refused: …` — **no `src/wasm.rs` change** |

- `encode_one` is only ever called on batch paths (`apply_one`, `build_one`; `run_apply`'s single
  input runs its own pipeline at `src/cli/optimize.rs:175`), so it always uses Skip and returns the
  skipped steps alongside `(ext, bytes)`. `optimize_decide_one` gains an `OnRefusal` parameter
  (single branch Fail, batch branch Skip, `encode_one_optimize_decided` Skip) and returns the
  skipped steps — add to its tuple or turn the tuple into a named struct, builder's choice.
  `optimize`/`web` reach `optimize_decide_one` too; their pipelines hold no refusing op, so the
  parameter changes nothing for them.
- **Exit 2** for a refused single input: `CliError::code()` gains
  `CliError::Operation(OperationError::Refused { .. }) => 2` above the generic `Operation(_) => 1`
  arm. No new `CliError` variant.

**Rejected:** a `Refused` error that carries the image back (an error type holding a pixel buffer,
printed by `Debug`); cloning the image before every step so a skipper can recover it (a full copy
per step per image, for every op, to serve one); per-surface checks (seven copies of the rule);
re-running the pipeline without the watermark step (a second decode and a second set of rules
for "which step").

DEC-002 keeps the trait "small": this adds one **defaulted** method, so no implementor changes. The
new DEC records that.

### Call 3 — what a batch prints

Matching DEC-085 (an incomplete output with no downstream signal warns unconditionally):

- **One line per skipped image, on stderr, never `--quiet`-gated:**
  `warning: <label>: watermark skipped, image written without it: <refusal Display>`
  (`<label>` = the input path, as every other per-input line uses). The verb's name in the line is
  the skipped op's `name()` — `watermark` is the only op that can refuse.
- **The run's summary carries the count**, defined per verb because the verbs report differently
  (AGENTS §15, an AC may not transfer between verbs):
  - `build` already has a summary line; it gains `, watermark skipped on K` after the
    `(… cached, … rebuilt)` group and before `, F failed`:
    `built 1 target, 3 outputs (0 cached, 3 rebuilt), watermark skipped on 1`. Omitted when K = 0,
    so a run with no skip prints exactly today's line. `--quiet`-gated like the line it extends.
  - `watermark`, `apply --recipe` (all three batch loops) have **no** summary on success today. They
    gain one closing stderr line **only when K > 0**: `watermark skipped on K of N images`,
    `--quiet`-gated. A run with no skip prints exactly what it prints today.
  - K counts **images** with at least one skipped step, not steps.
- `--json` / `--explain json` reports are unchanged (no schema change); the warning lines go to
  stderr as every other warning does.
- A batch with skips **and** failures: the failures alone set `failed` and exit 6; the skip lines
  and count are still printed; `PartialBatch { failed, total }` counts only the failures. A skip
  never reaches `failed`, so `build`'s lockfile phase (`src/cli/build.rs:823-829`) runs whenever
  the only problem is skips.

### Call 4 — a cached `build` does not repeat the skip line (accepted, documented, filed)

A cache hit writes the stored bytes and returns before the pipeline (`src/cli/build.rs:531-540`).
With an unmarked write the output is deterministic, so it caches like any other: **the first
`build` prints the skip line; a cached re-run prints `(N cached, 0 rebuilt)` and no skip line.**

**Settled: accept it and document it** (`docs/api-contract.md`, the `build` section; the
CHANGELOG line). Why:
- It is the repo's existing contract for per-input warnings: a cache hit does not replay DEC-085's
  truncated-JPEG or SPEC-119's animated-input warnings either (both are emitted only on the
  rebuild path's Decide branch, `src/cli/build.rs:558-578`). Replaying only the skip would make one warning out of
  three cache-aware.
- Carrying it needs a cache-entry frame change (`write_entry`/`parse_entry` carry only `ext` +
  payload, `src/build/cache.rs:455-503`) → `CACHE_SCHEMA_VERSION` 1 → 2, a new magic, and its own
  corruption tests. That is the right fix **for all three warnings together**, not for one.
- The output is still correct and locked: the lockfile pins the unmarked bytes, so `build --check`
  is clean, and `--no-cache` lists every skip again.

**Filed** on STAGE-050 by this design: "a cached `build` replays no per-input warnings (skipped
watermark, truncated JPEG, animated input)". A cache-schema bump is cheap whenever it lands (a
miss is a rebuild), so it is not coupled to 0.8.0.

**No `CACHE_SCHEMA_VERSION` change, no lockfile change** in this spec. ⚠ One dev-only consequence,
stated so nobody is surprised: until 0.8.0 is tagged the cache key still embeds `0.7.1`, so a dev
build carrying this spec can serve a `.crustyimg/cache` entry written by an older 0.7.1 build — the
pre-fix, damaged-mark bytes — for an image that would now be skipped. Released users never see it
(0.8.0 changes every key); developers clear `.crustyimg/` once. Same as every other byte-changing
PROJ-011 spec.

### Call 5 — recipe text mode refuses `scale` and `tile = true`, inside SPEC-131's mode check

Rows D/E: a `text` step with `tile`/`scale` runs at 52.7 % / 56.2 % today, and with `tile` exempt
from the cap it would bypass coverage entirely. The CLI already refuses both (`conflicts_with =
"text"`, `src/cli/mod.rs:422-427`).

**Where:** in `Watermark::from_params`'s **unused-by-mode check** (SPEC-131 Call 4 — the
constructor, which knows the mode), not in `WATERMARK_PARAMS`. SPEC-131's schema is
mode-independent by design (every range is mode-free), so `scale`/`tile` stay legal keys in the
table and the refusal cannot be re-allowed by editing it. Messages in SPEC-131's existing shape:
- `recipe step 0 ('watermark'): 'scale' only applies to an image watermark (this step sets 'text')`
- `recipe step 0 ('watermark'): 'tile = true' only applies to an image watermark (this step sets 'text')`

⚠ **Only `tile = true` is refused, not the key.** `Watermark::params()` writes `tile = false` for
every mark, text included (`src/operation/mod.rs:1436`), so refusing the key would refuse
crustyimg's own text recipes (SPEC-131 AC-11(b)). `scale` is written only when set, and nothing
crustyimg writes sets it on a text mark (clap refuses `--text --scale`), so refusing any `scale`
on a text step is safe. Surfaces: a recipe error at load — exit 1 on `apply`/`build`, the same
`JsError` on wasm — via SPEC-131's `InvalidOperation`, before any input is read.

⚠ **This turns two SPEC-131 tests red, by design** — both are on `feat/spec-131-…`
(`tests/recipe_strict_params.rs`, built from SPEC-131's spec):
- `recipes_crustyimg_writes_are_never_refused` builds a text mark with `scale: Some(0.5)` via
  `Watermark::new_text`. No crustyimg surface can write that. SPEC-133 moves that row's
  `Some(0.5)` to `None` (the image row two above already covers `scale`).
- `documented_interplay_is_not_refused` has `text = "hi"` + `tile = true` + `gravity`/`margin`.
  SPEC-133 removes that row (the image-mode row above it keeps the interplay covered) and updates
  the row-count assertion.

### Call 6 — DEC-104 is written at **build**

It describes code as merged (DEC-101/DEC-102 precedent). Contents: the rule (Call 1, the 5 % cap
as a fixed constant, mark-pixel counting and why not changed pixels); the seam (Call 2 — a
defaulted `Operation::refusal`, extending DEC-002's trait by one method; `Pipeline::run_with`);
the behaviour split (single explicit input → exit 2; batch / `build` → skip, written unmarked,
warning, count; a skip is not a failure — amends DEC-015's partial-batch rule for this one
outcome); and Call 4's cached-run limitation with its backlog pointer. `supersedes: null`.
Block-list `affected_scope`:
```yaml
affected_scope:
  - src/operation/mod.rs
  - src/pipeline/mod.rs
  - src/cli/mod.rs
  - src/cli/ops.rs
  - src/cli/optimize.rs
  - src/cli/common.rs
  - src/cli/build.rs
  - docs/api-contract.md
```

### Call 7 — size: M (top of M), the matrix adds up

Each rule is tested once on the op (14 unit tests), the two policies once on the pipeline (3),
one refusal test per single-input surface (6, including one coverage row), one skip test per
batch surface (6), plus skip-vs-failure, the cached re-run, the recipe mode check, the byte-identity
guard and the exit-code row. No rule is tested per surface: the surfaces share
`Watermark::refusal`, so a surface test proves the *lane* (exit 2 vs skip), using fit only. The
existing-test churn (12 tests in 5 files, measured) is mechanical — larger bases, no behavioural
assertion changes. Not L.

## Inputs

- **Files to read:** `src/operation/mod.rs:146-190` (`OperationError`, `Operation`), `:1109-1145`
  (`Gravity::placement`), `:1150-1565` (`Watermark`: `from_params`, `params`, `apply`);
  `src/pipeline/mod.rs` (whole, 223 lines); `src/cli/ops.rs:286-520` (`run_pixel_op`),
  `:1290-1350` (`run_watermark`); `src/cli/optimize.rs:44-300` (`run_apply`), `:1083-1250`
  (`optimize_decide_one`), `:1370-1398` (`encode_one_optimize_decided`), `:1535-1655`
  (`run_optimize_autodecide`); `src/cli/common.rs:21-100` (`encode_one`, `apply_one`);
  `src/cli/build.rs:500-600` (`build_one`), `:735-830` (execute, summary, lockfile gate);
  `src/cli/mod.rs:735-800` (`code()`), `:1090-1150` (its mapping test);
  `src/text/mod.rs:121-200` (`render_text` — the overlay is tight to the glyphs).
- **After SPEC-131 merges:** its `WATERMARK_PARAMS`, the unused-by-mode check in
  `Watermark::from_params`, and `tests/recipe_strict_params.rs` (the two rows in Call 5).
- **Related, not edited:** `src/wasm.rs:204-245`; `src/build/cache.rs:455-503`.

## Outputs

- **Files created:** `tests/watermark_damage.rs`; `decisions/DEC-104-watermark-refuses-a-mark-that-does-not-fit-or-covers-over-5-percent.md`.
- **Files modified:** see Implementation Context → Files the build touches.
- **New exports:** `operation::Refusal`, `operation::WATERMARK_COVERAGE_CAP_PERCENT`,
  `OperationError::Refused`, `Operation::refusal` (defaulted), `pipeline::{OnRefusal,
  PipelineRun, SkippedStep}`, `Pipeline::run_with`.

## Acceptance Criteria

"Refused" on the op = `Watermark::refusal(&img)` returns `Some(..)` **and** `apply` returns
`OperationError::Refused` with the same `Refusal`. Op rows use synthetic overlays (exact sizes and
alpha patterns built with `RgbaImage::from_fn`) unless a row names text; text rows use the bundled
font. Numbers asserted are the `Refusal` fields, not Display, except where Display is the subject.

**The rule — library, `Watermark` (one test per rule × mode):**
- [ ] **AC-1. Fit refuses a text mark wider than the image, and one taller.** (a) Row A: `"© crustyimg"`
      at size 200 on 24×24 → `DoesNotFit { mark: <the rendered overlay's own size>, image: (24,24) }`.
      (b) Row B: the long string at default size on 256×256 (width-only overflow) → `DoesNotFit`.
      (c) a text mark whose height alone exceeds a short wide image (default string on 400×20).
- [ ] **AC-2. Fit refuses an image mark, on each axis.** 256×256 overlay on 24×24 (row C); 10×40
      overlay on 64×32 (height only); 70×5 on 64×32 (width only).
- [ ] **AC-3. Margin applies only on the axes the gravity anchors.** With a 100×10 overlay that has
      one non-transparent pixel (so coverage cannot interfere), on 120×120: `southeast`
      margin 20 → fits (equality); margin 21 → refused, `margin: 21`; `center` margin 50 → fits;
      `north` margin 50 → fits (x is centred and ignores margin; 10 + 50 ≤ 120); `east`
      margin 50 → refused (100 + 50 > 120).
- [ ] **AC-4. Coverage refuses above 5 % — text and image both.** Image: on 100×100, an overlay
      with exactly 500 non-transparent pixels → applied; 501 → `CoverageAboveCap { covered: 501 }`.
      Transparent pixels do not count: a 40×40 overlay with 400 non-transparent pixels (16 % area,
      4 % coverage) → applied. Text: the default string on 172×172 → refused, `covered` equal to
      the rendered overlay's alpha > 0 count (1661, measured at design); on 192×192 → applied.
- [ ] **AC-5. Opacity does not change coverage.** The 501-pixel overlay (alpha 255) at `opacity = 0.001`
      — where every alpha composites as round(255 × 0.001) = 0, so the mark changes no pixel —
      is still refused with `covered: 501`. (At 0.01 the alpha is round(2.55) = 3, which a count
      taken after opacity would still see; only a value that rounds to 0 can tell the two apart.)
- [ ] **AC-6. `scale` is fit-checked after scaling and exempt from coverage.** (a) Row H: 256×256
      overlay, `scale = 0.5`, on 24×24 → applied (12×12, 25 % covered). (b) 10×100 overlay,
      `scale = 1.0`, on 64×32 → `DoesNotFit { mark: (64, 640) }` — the post-scale size.
- [ ] **AC-7. `tile`: one tile must fit; coverage exempt; margin ignored.** (a) Row G: 64×64
      opaque overlay tiled on 256×256 → applied. (b) Row I: 256×256 tiled on 24×24 → `DoesNotFit`.
      (c) a 120×120 overlay tiled on 120×120 with `margin = 30` → applied (tile ignores margin).
- [ ] **AC-8. Fit is reported before coverage; nothing is drawn on refusal.** A mark failing both
      (row A+ style: default string on 24×24) → `DoesNotFit`. `apply` returns `Err` (no image).
- [ ] **AC-9. Display names the numbers.** `DoesNotFit` with margin 0 / margin 21 and
      `CoverageAboveCap` render exactly the three Call 2 shapes (mark size, image size, margin
      when > 0; percent to 2 dp, covered, area, "the limit is 5 %"); `OperationError::Refused`
      prefixes `watermark refused: `.
- [ ] **AC-10. A mark that passes is byte-identical to `main`.** Decoded-pixel hashes of
      `Watermark::apply` for six passing cases — text default on 256 (B′) and 800×600; opaque
      64×64 on 800×600; tile (G); scale (H); a 16-bit base with a fitting image mark — equal
      goldens captured from unmodified `main` in the build's first commit.

**The pipeline — library:**
- [ ] **AC-11. `run` (and `run_with(Fail)`) stops at a refusal; `run_with(Skip)` skips it and runs
      the rest.** Pipeline `[invert, watermark(refuses), invert]`: `run` → `Err(Refused)`;
      `run_with(Skip)` → `Ok`, image equals the input (two inverts), `skipped == [SkippedStep
      { step: 1, op: "watermark", .. }]`. A pipeline whose watermark fits returns the same image and
      empty `skipped` under both policies.

**The recipe — library + load:**
- [ ] **AC-12. A `text` step refuses `scale` and `tile = true` at load; `tile = false` and image
      mode are unaffected.** `build_pipeline` → `InvalidOperation` naming `'scale'` / `'tile = true'`
      for rows D and E; a text step with `tile = false` builds; an image step with `scale` and
      `tile = true` builds; `Recipe::from_ops` over a text mark (as the CLI builds it) →
      `to_toml` → `build_pipeline` builds.

**Single explicit input → error, exit 2, nothing written** (stderr is one `error:` line):
- [ ] **AC-13. `watermark`, one input.** Row A → exit 2, no `-o` file, stderr contains
      `watermark refused`, `does not fit` and `24×24 px`. **And coverage reaches the surface:**
      default string on 172×172 → exit 2, stderr contains `5.61 %` (1661 of 29584 px, measured
      at design) and `the limit is 5 %`.
- [ ] **AC-14. `apply --recipe`, plain, one input.** Row A's step as a recipe → exit 2, no output.
- [ ] **AC-15. `apply --recipe` ending in `optimize`, one input — both branches.** Row A's step
      followed by a terminal `optimize` step. (a) pinned (`-o out.png`) → exit 2, no output; (b) auto-decide (`--out-dir d`) → exit 2, `d` empty.
- [ ] **AC-16. wasm `transform`** (under `just wasm-test`): a 24×24 PNG + a text recipe with
      `size = 200.0` → `Err` whose text contains `watermark refused` and `24×24`; a following
      valid call on the same module succeeds.

**Batch / `build` → skip: written without the mark, other steps applied, reported, exit 0:**
- [ ] **AC-17. `watermark`, N inputs.** `b24.png` + `b256.png` (24×24 and 256×256
      generated bases), default text, `--out-dir`: exit 0;
      both outputs exist; `b24`'s output pixels equal its input's; `b256`'s output is byte-identical
      to the single-input run on `b256.png`; stderr has exactly one
      `warning: …b24.png: watermark skipped, image written without it: …` line and the closing
      `watermark skipped on 1 of 2 images`. **With `--quiet`:** the warning line is still there; the
      closing line is not.
- [ ] **AC-18. `apply --recipe`, plain, N inputs — other steps still apply.** Recipe
      `[resize percent 50, watermark text]` over a 48×48 and a 512×512 input: exit 0; the small
      input's output is byte-identical to the same recipe **without** the watermark step; one
      warning line; `watermark skipped on 1 of 2 images`.
- [ ] **AC-19. `apply --recipe` ending in `optimize`, N inputs — both branches.** AC-18's recipe
      plus a terminal `optimize` step, AC-18's inputs. (a) pinned (`--format png`) and (b) auto-decide: exit 0, 2 outputs, one warning line naming the small
      input, the closing count line.
- [ ] **AC-20. `build`, a plain recipe (Preserve plan).** One target, recipe as AC-18, inputs as
      AC-18: exit 0; the skipped output is byte-identical to the watermark-free recipe's; one
      warning line; summary `built 1 target, 2 outputs (0 cached, 2 rebuilt), watermark skipped on 1`;
      `crustyimg.build.lock` written with 2 records; a following `build --check` exits 0.
- [ ] **AC-21. `build`, terminal-`optimize` recipe (Decide plan).** AC-19's recipe and
      inputs, target name `{stem}.{ext}`:
      exit 0, one warning line, the summary suffix, lockfile written.
- [ ] **AC-22. A skip is not a failure.** `build` over [small (skips), corrupt.png (fails),
      large]: exit **6**; summary ends `watermark skipped on 1, 1 failed`; the final error reports
      1 failed of 3 (not 2); no lockfile written (unchanged failure rule).
- [ ] **AC-23. A cached re-run (Call 4, the documented behaviour).** Re-running AC-20's `build`:
      exit 0, `(2 cached, 0 rebuilt)`, outputs byte-identical, **no** skip line, no suffix. Then
      `build --no-cache`: the skip line and suffix are back.

**Exit mapping, docs, controls, matrix:**
- [ ] **AC-24. `CliError::Operation(OperationError::Refused { .. }).code() == 2`**;
      `Operation(Apply { .. })` stays 1 (`exit_code_mapping_is_total` gains the row).
- [ ] **AC-25. Docs.** `docs/api-contract.md`: the `watermark` section states the rule (fit with
      margin on anchored axes; 5 % of the image's pixels for a fixed-size mark; `tile` and `scale`
      exempt from the 5 %), exit 2 on one explicit input, the batch skip line and count, and the
      cached-run behaviour under `build`; the exit-code table's row for 2 mentions a refused
      watermark. `recipes/README.md` (watermark keys) and `docs/data-model.md` (watermark table):
      the rule in one paragraph, and `scale`/`tile = true` image-only. The `watermark` command's
      help text (`src/cli/mod.rs` doc comment) gains one sentence. `CHANGELOG.md`: one **Changed**
      line (refusal + skip), one **Fixed** line (text no longer clipped / silently tiny; recipe text
      with `tile`/`scale` refused). DEC-104 exists (Call 6).
- [ ] **AC-26. Negative controls, one revert per independent condition** (table under Failing
      Tests). Evidence is the behavioural flip, never a hash.
- [ ] **AC-27. Clean matrix.** Default, `--no-default-features`, `--features webp-lossy`, each in a
      fresh `CARGO_TARGET_DIR`, **sequential**; `clippy -D warnings`, `fmt --check`;
      `just wasm-check`, `just wasm-test`, **`just demo-build`** (bundle-size gate; this reaches wasm
      through `src/operation/**`). `src/operation/**` and `src/pipeline/**` gain no
      `std::fs`/`std::path` (grep stays empty).

## Failing Tests

Written at design, made to pass at build. "RED on `main`" says why each fails today; **guard** =
green on `main`, must stay green.

- **`src/operation/mod.rs`** (unit; synthetic overlays via `RgbaImage::from_fn`, text via
  `crate::text::render_text` + `Watermark::new_text`)
  - `fit_refuses_text_wider_or_taller_than_the_image` — AC-1 (3 rows). RED: no `refusal`/`Refused`.
  - `fit_refuses_an_image_mark_on_each_axis` — AC-2 (3 rows). RED: same.
  - `margin_applies_only_on_anchored_axes` — AC-3 (5 rows). RED: same.
  - `coverage_cap_is_5_percent_of_image_pixels` — AC-4 image rows (500 / 501 / 400-of-1600). RED.
  - `coverage_cap_applies_to_text` — AC-4 text rows (172 / 192). RED.
  - `opacity_does_not_change_coverage` — AC-5. RED.
  - `scale_is_fit_checked_after_scaling` — AC-6(b). RED.
  - `scale_is_exempt_from_coverage` — AC-6(a). **Guard** (applies on `main`; must still).
  - `one_tile_must_fit` — AC-7(b). RED.
  - `tile_is_exempt_from_coverage_and_margin` — AC-7(a)(c). **Guard.**
  - `fit_is_reported_before_coverage` — AC-8. RED.
  - `refusal_display_names_the_numbers` — AC-9 (3 shapes + the `Refused` prefix). RED.
  - `passing_marks_are_byte_identical_to_main` — AC-10. **Guard** (goldens from `main`).
  - `bless_watermark_goldens` — `#[ignore]`, writes AC-10's goldens only under
    `CRUSTYIMG_BLESS=1`; verify re-runs it on `main` and diffs to zero.
- **`src/pipeline/mod.rs`** (unit; a stub op whose `refusal` always returns `Some`)
  - `run_fails_on_a_refusal_and_stops` — AC-11. RED: `refusal` does not exist.
  - `run_with_skip_skips_the_step_and_runs_the_rest` — AC-11. RED: `run_with` does not exist.
  - `policies_agree_when_nothing_refuses` — AC-11. RED: does not compile.
- **`src/cli/mod.rs`** (unit)
  - `exit_code_mapping_is_total` — gains the AC-24 row. RED: `Refused` does not exist.
- **`tests/watermark_damage.rs`** (new; library half via `Recipe::from_toml` +
  `build_pipeline(&OperationRegistry::with_builtins())`; CLI half drives
  `env!("CARGO_BIN_EXE_crustyimg")`; bases generated natively — a gradient, not a solid colour, so
  "pixels equal the input" is not trivially true)
  - `text_step_refuses_scale_and_tile_true` — AC-12 (D, E). RED: both build `Ok`.
  - `text_step_accepts_tile_false_and_image_steps_keep_scale_and_tile` — AC-12 positives. **Guard.**
  - `watermark_one_input_refused_exits_2` — AC-13 fit row. RED: exit 0, file written.
  - `watermark_one_input_over_the_cap_exits_2` — AC-13 coverage row. RED: exit 0.
  - `apply_one_input_refused_exits_2` — AC-14. RED: exit 0.
  - `apply_optimize_pinned_one_input_refused_exits_2` — AC-15(a). RED: exit 0.
  - `apply_optimize_autodecide_one_input_refused_exits_2` — AC-15(b). RED: exit 0.
  - `watermark_batch_skips_and_reports` — AC-17 (incl. `--quiet`). RED: no warning line.
  - `apply_batch_skip_keeps_the_other_steps` — AC-18. RED: the small output is marked (differs).
  - `apply_optimize_pinned_batch_skips` — AC-19(a). RED: no warning line.
  - `apply_optimize_autodecide_batch_skips` — AC-19(b). RED: no warning line.
  - `build_skip_writes_unmarked_output_and_the_lockfile` — AC-20. RED: no suffix, marked output.
  - `build_decide_plan_skips` — AC-21. RED: no warning line.
  - `build_skip_is_not_counted_as_a_failure` — AC-22. RED: summary lacks the skip suffix (exit 6
    and the missing lockfile hold on `main` too, for the corrupt input — the suffix and the
    `1 failed` count are what flip).
  - `build_cached_rerun_does_not_repeat_the_skip` — AC-23. RED: the first run has no skip line.
- **`tests/wasm_roundtrip.rs`** (wasm half)
  - `transform_refuses_a_mark_that_does_not_fit` — AC-16. RED: returns `Ok`.

**Existing tests that change (measured by the prototype, not guessed) — fixture changes only, no
assertion is weakened:**
- `tests/apply_batch.rs`: `apply_watermark_recipe_matches_cli_at_one_input`,
  `apply_watermark_recipe_matches_cli_at_n_inputs`, `apply_and_build_agree_on_watermark_recipe` —
  an 8×8 overlay on 32×32 bases is 6.25 %. Raise the bases to 64×64 (1.56 %).
- `tests/build_watermark_cache.rs`: `build_hits_when_overlay_bytes_unchanged`,
  `build_rebuilds_when_overlay_bytes_change` (8×8 overlay on 32×32 = 6.25 % → 64×64 bases);
  `build_hits_when_bundled_font_is_used`, `build_rebuilds_when_font_bytes_change` (a 70×26 text
  mark on 32×32 / 48×48 does not fit → bases of 128×128 or more).
- `tests/colour_type_preservation.rs`: `watermark_text_narrows_on_an_opaque_base` (775 of 9216 px
  = 8.41 % → a base of 160×160 or more).
- `tests/recipe_watermark.rs`: `text_watermark_round_trips_without_an_image_key` (278 of 4096 =
  6.79 % → 96×96 or more).
- `tests/watermark.rs`: `text_watermark_writes_output`, `text_watermark_default_font`,
  `text_watermark_custom_font` (the default 169×31 mark on an 80×40 base does not fit → e.g.
  320×160, 3.2 %; check the custom font's mark too).
- Prototype totals: **12 tests in 5 files flip; all 486 unit tests and every other integration
  test stay green** (default features). The wasm half was not run by `cargo test`; its one text
  case (`transform_runs_a_text_only_watermark_with_no_font_key`: `"hi"`, size 12, on 64×48) was
  driven through the prototype binary and passes (a size-40 control on the same base refuses at
  12.50 %, so the check did run). Change each fixture's size and any assertion on that size only;
  no behavioural assertion is weakened.
- `tests/recipe_strict_params.rs` (SPEC-131's): the two rows in Call 5.

**Negative controls (AC-26) — revert one condition, expect exactly these RED:**

| # | condition (revert only this) | flips RED | must stay GREEN |
|---|---|---|---|
| N1 | fit check, both axes | AC-1, AC-2, AC-3, AC-6(b), AC-7(b), AC-8, every single-input fit test (AC-13 fit, 14, 15, 16), every batch test (AC-17–23 — they all skip on fit) | the coverage tests (AC-4, 5), AC-13 coverage row |
| N2 | coverage check | AC-4, AC-5, AC-13 coverage row | every fit and surface test |
| N3 | margin only on anchored axes (revert: margin on both axes always) | AC-3's `center`/`north` rows, AC-7(c) | the rest |
| N4 | `scale` exempt from coverage (revert: cap scaled marks too) | AC-6(a) | the rest |
| N5 | `tile` exempt from coverage (revert: cap tiles too) | AC-7(a) | the rest |
| N6 | `run_with(Skip)` skips (revert: behaves as Fail) | AC-11 skip test, AC-17–23 (exit 6, no lockfile) | every single-input test, AC-24 |
| N7 | exit 2 for `Refused` (revert: falls to the generic 1) | AC-13, 14, 15, AC-24 | AC-16 (wasm has no exit code), every batch test |
| N8 | text mode refuses `scale` / `tile = true` | `text_step_refuses_scale_and_tile_true` | the rest |
| N9 | batch warning + count lines (revert: don't print) | the stderr assertions of AC-17–22 | AC-23's `--no-cache` half also flips; nothing else |

Two rules make the table hold: every surface test uses a **fit** failure (row A or a 48 px input
under the default string) except the one coverage row, so N1/N2 split cleanly; and each library
table asserts `Refusal` fields, not Display, except AC-9.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Files the build touches (best estimate — the orchestrator schedules on this)

- `src/operation/mod.rs` — `Refusal`, the cap const, `OperationError::Refused`, defaulted
  `Operation::refusal`; `Watermark`: stored mark-pixel count, `refusal` impl, the guard at the top
  of `apply`, the shared post-scale-size helper used by both bit-depth branches; text-mode
  `scale` / `tile = true` refusal inside SPEC-131's unused-by-mode check in `from_params`; unit
  tests. ⚠ **SPEC-131 edits `Watermark::from_params` too — build after it merges.**
- `src/pipeline/mod.rs` — `OnRefusal`, `SkippedStep`, `PipelineRun`, `run_with`; `run` delegates;
  module doc; unit tests.
- `src/cli/mod.rs` — one `code()` arm; the mapping test row; one sentence in the `watermark`
  command's doc comment (help text).
- `src/cli/ops.rs` — **`run_pixel_op` batch loop only** (Skip, warning line, count, closing line).
  The single-input branch is unchanged (it already calls `run`).
- `src/cli/optimize.rs` — **`run_apply`** (batch closures print warnings, count, closing line),
  **`optimize_decide_one`** (an `OnRefusal` parameter; returns the skipped steps),
  **`encode_one_optimize_decided`** (Skip; returns them), **`run_optimize_autodecide`** (single
  Fail; batch Skip + lines + count).
- `src/cli/common.rs` — **`encode_one`** (Skip; returns the skipped steps), **`apply_one`**
  (returns them).
- `src/cli/build.rs` — **`build_one`** (collect skips from both encode paths; warning lines),
  `BuildOutcome` (a skipped field, empty on a hit), **`run_build`'s summary** (the suffix). The
  lockfile phase is untouched.
- **Not touched:** `src/wasm.rs`, `src/build/**`, `src/text/**`, `src/operation/registry.rs`
  (SPEC-131's `WATERMARK_PARAMS` is unchanged — Call 5).
- Tests: `tests/watermark_damage.rs` (new), `tests/wasm_roundtrip.rs` (one test appended),
  `tests/recipe_strict_params.rs` (two rows, Call 5), `tests/apply_batch.rs` (three fixtures),
  `tests/build_watermark_cache.rs`, `tests/colour_type_preservation.rs`,
  `tests/recipe_watermark.rs`, `tests/watermark.rs` (fixture sizes only — the measured list under
  Failing Tests).
- Docs: `docs/api-contract.md`, `recipes/README.md`, `docs/data-model.md`, `CHANGELOG.md`.
- Decisions: `decisions/DEC-104-watermark-refuses-a-mark-that-does-not-fit-or-covers-over-5-percent.md` (new).

**Overlap with SPEC-134 (`--name-template` pins format) — do not build in parallel.** Shared
functions: `run_pixel_op` (it edits format resolution at `:377`/`:475`; this spec edits the batch
loop's run/report lines), `run_apply` (single + batch format resolution vs this spec's batch
reporting), `encode_one` / `apply_one` (possible signature changes on both sides — the likeliest
conflict), `build_one` / `run_build`'s summary; files `tests/apply_batch.rs`,
`docs/api-contract.md`, `CHANGELOG.md`. **Overlap with SPEC-131:** `src/operation/mod.rs`
(`Watermark::from_params`), `tests/recipe_strict_params.rs` — sequential, SPEC-131 first.
No overlap with SPEC-132 (`src/build/watch.rs`).

### Decisions that apply

- `DEC-002` — the `Operation` trait stays small; one defaulted method is added (Call 2), recorded
  in DEC-104.
- `DEC-015` — exit codes and the partial-batch rule; a refused single input is exit 2, and a skip
  is not a per-input failure (DEC-104 amends the batch rule for this one outcome).
- `DEC-085` — the warn-and-exit-0 precedent; skip lines are unconditional like it.
- `DEC-031` / `DEC-064` — `src/operation/**` stays filesystem-free and wasm32-clean; the rule is
  pure geometry on in-memory pixels; `src/pipeline/**` likewise.
- `DEC-032` / `DEC-045` — the text overlay is rendered tight to its glyphs, which is why the
  overlay's size is the mark's extent.
- `DEC-100` — the recipe watermark seam; unchanged.
- `DEC-057` / `DEC-058` / `DEC-059` / `DEC-101` — `build`'s cache and lockfile; unchanged (Call 4).
- `DEC-003` — the pixel lane only; no metadata involvement.
- `DEC-102` (SPEC-131, written at its build) — the unused-by-mode check this spec extends.

### Constraints that apply

- `test-before-implementation` — goldens first (own commit, on an untouched tree), then RED tests.
- `no-unwrap-on-recoverable-paths` — refusals are typed values, never panics.
- `decode-once-no-per-op-disk` — the check reads the in-memory image; skip keeps one decode.
- `untrusted-input-hardening` — `u64` arithmetic for areas and sums (a 65535×65535 image times 100
  overflows `u32`).
- `one-spec-per-pr`.

### Prior related work

- `SPEC-029` (shipped) — the `Watermark` op and `Gravity::placement`.
- `SPEC-030` — text mode; `render_text`.
- `SPEC-128` (shipped) — watermark reachable from recipes / `build` (DEC-100).
- `SPEC-129` (shipped) — `build` cache key hashes asset content (DEC-101).
- `SPEC-131` (building) — strict step params; Call 5 extends its mode check.
- `SPEC-107` / DEC-085 — the warning precedent.

### Out of scope (for this spec specifically)

- `--min-size`, the opt-in value rule — its own STAGE-050 item; it will reuse `OnRefusal::Skip`.
- A relative / percent `--size`; a user-settable cap.
- Replaying per-input warnings from the `build` cache — filed by this design (Call 4).
- Any change to how a mark that passes is placed, composited or rendered (AC-10 pins it).
- A skip field in `--json` / `--explain json` reports.
- `build --watch` asset watching — SPEC-132.
- SSIMULACRA2 or any perceptual measure — ruled out.

## Notes for the Implementer

- **Wait for SPEC-131 to merge**, rebase, and read its `Watermark::from_params` before editing it.
- **Start with the goldens** (AC-10): an `#[ignore]`d bless test, run on an untouched tree, in its
  own commit. Then the RED tests. Then `src/`.
- **One helper for the post-scale size.** Both bit-depth branches of `apply` compute it today with
  identical arithmetic (`src/operation/mod.rs:1458-1465`, `:1509-1516`); move that into one
  function and call it from both branches **and** from `refusal`. AC-10 proves the move changed
  no byte.
- **Count on `to_rgba16()`**, not `to_rgba8()` — a 16-bit overlay's alpha 1..=256 becomes 0 in
  8 bits, and those pixels still change the image.
- **Integer comparisons only** for the cap (`covered * 100 > w * h * 5`, all `u64`); the percent
  in the message is formatting only.
- The warning line's label is the same `label` each loop already builds for its `error:` lines.
- Print a skip's warning line where the loop prints its `error:` line (inside the rayon closures
  for `apply`/`build`), so it lands next to its image's other output.
- **`build --check` after a skip must stay clean** — the lock records the unmarked bytes; do not
  add anything to the lock record.
- Run the clean matrix **sequentially, each leg in its own `CARGO_TARGET_DIR`**; redirect test
  output to a file and read `$?` (never `cargo test | tail`).
- **Push a WIP commit once it compiles**, before the matrix.
- Comments and user-facing text are behaviour-first — no spec/DEC ids in messages or doc comments.
- Every commit `git commit -s`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-133-watermark-damage-rule`
- **PR (if applicable):** #206 (draft)
- **All acceptance criteria met?** yes — AC-1…AC-27, with the deviations below. AC-27's legs
  are listed under "Matrix".
- **New decisions emitted:**
  - `DEC-104` — A watermark refuses a mark that does not fit or covers over 5 % of the image;
    a batch skips it and writes the image without it
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`, 24 files):
  - `src/operation/mod.rs` — `Refusal` + Display, `WATERMARK_COVERAGE_CAP_PERCENT`,
    `OperationError::Refused`, defaulted `Operation::refusal`; `Gravity::inset_axes`;
    `Watermark`: a `visible` count set by all three constructors, `mark_size` (the one
    post-`scale` size helper, called by both bit-depth branches and by `refusal`), `refusal`,
    the guard at the top of `apply`, the text-mode `scale` / `tile = true` refusal in
    `from_params`; 12 unit tests.
  - `src/pipeline/mod.rs` — `OnRefusal`, `SkippedStep`, `PipelineRun`, `run_with`; `run`
    delegates to `run_with(Fail)`; module doc; 3 unit tests.
  - `src/cli/mod.rs` — `Operation(Refused) => 2` arm; the exit-mapping test row; one help
    sentence on the `watermark` command.
  - `src/cli/common.rs` — `warn_skipped_steps`, `report_skipped_images`; `encode_one` runs
    `Skip` and returns the skipped steps; `apply_one` returns them.
  - `src/cli/ops.rs` — `run_pixel_op` batch loop only: `Skip`, warning lines, count, closing
    line. The single-input branch is unchanged.
  - `src/cli/optimize.rs` — `run_apply` batch closures (warnings, count, closing line);
    `optimize_decide_one` takes an `OnRefusal` and returns the skipped steps (6th tuple
    element); `encode_one_optimize_decided` uses `Skip` and returns a named `DecidedEncode`;
    `run_optimize_autodecide` single `Fail`, batch `Skip` + lines + count.
  - `src/cli/build.rs` — `BuildOutcome.skipped` (empty on a hit), `build_one` collects skips
    from both encode paths, the rayon closure prints the warning, the summary suffix. The
    lockfile phase is untouched.
  - `tests/watermark_damage.rs` — new: AC-10 goldens + bless, AC-12, AC-13…AC-15, AC-17…AC-23.
  - `tests/fixtures/watermark/passing_marks.golden` — new: the six AC-10 hashes, blessed on
    untouched `src/` (`origin/main` `65cb95d`, commit `ab94505`).
  - `tests/wasm_roundtrip.rs` — `transform_refuses_a_mark_that_does_not_fit` (AC-16).
  - `tests/recipe_strict_params.rs` — Call 5's two rows.
  - `tests/apply_batch.rs`, `tests/build_watermark_cache.rs`, `tests/colour_type_preservation.rs`,
    `tests/recipe_watermark.rs`, `tests/watermark.rs`, `tests/build_watch.rs` — fixture sizes only.
  - `tests/fixtures/recipe_corpus/v2_watermark_integer_numbers.{toml,golden}` — one line
    (`scale = 1`) removed from each (Deviation 2).
  - `docs/api-contract.md` — exit-code row 2; the `watermark` rule, single-input vs batch
    behaviour, the recipe text-mode refusal; the `build` skip paragraph incl. the cached run.
  - `docs/data-model.md` — watermark keys: `scale` / `tile = true` image-only, the rule.
  - `recipes/README.md` — `scale` / `tile = true` image-only; "A watermark that does not fit
    is left off".
  - `CHANGELOG.md` — a new Unreleased `### Changed` (refusal + skip) and one `### Fixed` line.
  - `decisions/DEC-104-…md` — new, block-list `affected_scope`.

- **RED baseline (on unmodified `src/`, commit `7bfecb4`'s parent tree for `src/`):**
  `cargo test --lib` does not compile — `refusal`, `Refusal`, `Refused`, `OnRefusal`,
  `SkippedStep`, `run_with` do not exist (every op/pipeline unit test and the AC-24 row).
  `cargo test --test '*'`: only `watermark_damage` fails — 14 of 16, each for the reason the
  spec gives (single inputs exit 0 instead of 2; batch/build print no skip line; AC-17/18's small
  output is marked; AC-12 D/E load `Ok`); its 2 guards (AC-10 goldens, AC-12 positives) pass.
  Every other integration test green. `just wasm-test`: `transform_refuses_a_mark_that_does_not_fit`
  fails (`transform` returns `Ok`), 44 others pass. Guards `scale_is_exempt_from_coverage` /
  `tile_is_exempt_from_coverage_and_margin` are in the lib target, so on `main` they were
  uncompilable rather than green; they pass after the build.

- **Negative controls (AC-26)** — each a single revert, binary rebuilt by the run, `src/`
  restored with `git checkout` after; run on `--lib` + `--test watermark_damage` (+ `just
  wasm-test` for N1):

  | # | reverted | flipped RED | stayed green as required |
  |---|---|---|---|
  | N1 | fit check | 6 op tests (AC-1, 2, 3, 6b, 7b, 8); AC-13 fit, 14, 15a, 15b; AC-17…23 (8 tests); wasm AC-16 | AC-4, AC-5, AC-13 coverage row |
  | N2 | coverage check | AC-4 image + text, AC-5, AC-13 coverage row | all fit + surface tests |
  | N3 | margin on both axes always | AC-3 (`center` row), AC-7(c) | the rest |
  | N4 | cap scaled marks too | AC-6(a) **and AC-10** (its golden includes row H) | the rest |
  | N5 | cap tiles too | AC-7(a) **and AC-10** (its golden includes row G) | the rest |
  | N6 | `Skip` behaves as `Fail` | AC-11 skip test; AC-17…23 | single-input tests, AC-24 |
  | N7 | exit 2 arm | AC-13 (both rows), 14, 15a, 15b, AC-24 | every batch test (AC-16 has no exit code) |
  | N8 | text-mode `scale` / `tile = true` | `text_step_refuses_scale_and_tile_true` | the rest |
  | N9 | warning + count lines (+ `build` suffix) | AC-17…22, AC-23 | the rest |

  N4/N5's extra AC-10 flip is the golden doing its job, not a leak: two of its six passing
  cases are the scaled and tiled marks.

- **Existing tests changed** (fixture sizes only unless said; each still asserts what it did):
  - `tests/apply_batch.rs`: `apply_watermark_recipe_matches_cli_at_one_input` base 32→64;
    `…_at_n_inputs` both bases 32→64; `apply_and_build_agree_on_watermark_recipe` 32→64.
  - `tests/build_watermark_cache.rs`: image project `src/a.png` 32→64 (`b` 48 already at
    2.8 %, unchanged) — covers `build_hits_when_overlay_bytes_unchanged`,
    `build_rebuilds_when_overlay_bytes_change`; both text projects `a` 32→128, `b` 48→144
    (`"mark"` is 70×26 with 707 visible px: 96 px is 7.67 %, 128 px 4.3 %) — covers
    `build_rebuilds_when_font_bytes_change`, `build_hits_when_bundled_font_is_used`.
  - `tests/colour_type_preservation.rs`: `watermark_text_narrows_on_an_opaque_base` 96→160.
  - `tests/recipe_watermark.rs`: `text_watermark_round_trips_without_an_image_key` 64→96.
  - `tests/watermark.rs`: `text_watermark_writes_output`, `text_watermark_default_font`,
    `text_watermark_custom_font` 80×40→320×160.
  - `tests/build_watch.rs` (**not on the spec's list** — SPEC-132 landed after the design's
    prototype): `brand_project`'s `src/a.png` 64→128 (a 16×16 opaque logo on 64×64 is 6.25 %).
    Fixes `watch_rebuilds_when_the_watermark_image_changes`,
    `watch_follows_a_recipe_repointed_to_a_new_image`,
    `watch_picks_up_an_asset_directory_created_mid_session`,
    `watch_with_asset_roots_does_not_self_trigger`.
  - `tests/recipe_strict_params.rs` (Call 5): `recipes_crustyimg_writes_are_never_refused`'s
    text row `Some(0.5)` → `None` (+ its comment); `documented_interplay_is_not_refused` drops
    the `text` + `tile = true` row, count 3 → 2.
  - `tests/fixtures/recipe_corpus/v2_watermark_integer_numbers.{toml,golden}` — see Deviation 2.
  - **Proof none of them passes by skipping:** with `warn_skipped_steps` temporarily made to
    `exit(99)` on any skip, `apply_batch`, `build_watermark_cache`, `build_watch`, `watermark`
    and `colour_type_preservation` all passed (no skip reached), while the 8 skip tests in
    `watermark_damage` hit it (the probe could fire). Reverted.

- **Matrix (AC-27):** run sequentially, each feature leg in its own fresh `CARGO_TARGET_DIR`,
  deleted after the leg, at `2e0a4cc`:
  - default — `cargo test --no-fail-fast` 48/48 test binaries ok, 0 failed; `clippy --all-targets -D warnings` clean
  - `--no-default-features` — 48/48 ok, 0 failed; clippy clean
  - `--features webp-lossy` — 48/48 ok, 0 failed; clippy clean
  - `cargo fmt --check` clean; `just wasm-check` ok; `just wasm-test` 45 passed, 0 failed
  - `just demo-build` ok: **1,270,008 B brotli** vs the gate's baseline 1,266,535 B
    (+3,473 B, +0.27 %, inside ±5 %; the baseline is the gate's anchor, not a fresh `main`
    build). The gate file is untouched.
  - `grep -rnE 'std::fs|std::path' src/operation src/pipeline` (excluding the two `//!`
    module-doc lines that already name them as forbidden): empty.
  - `scripts/decisions-audit.sh`: 0 structural errors; DEC-104 overlap warnings with
    DEC-102 (`src/operation/mod.rs`; DEC-104 extends its mode check) and DEC-105
    (`src/cli/mod.rs`; unrelated rules). No contradiction.

- **`--size nan` backlog item (HANDOFF rule 6):** driven on this build —
  `watermark b192.png --text "© crustyimg" --size nan` and `--size inf` exit **0**, and
  `--image logo16.png --scale nan` exits **0**: the fit check does **not** refuse them (the mark
  renders tiny and fits). `--scale inf` is now refused (`the 4294967295×4294967295 px mark does
  not fit…`). The STAGE-050 item stays open; it is not closed by this spec.

- **Deviations from spec:**
  1. **AC-10's golden and bless tests live in `tests/watermark_damage.rs`, not in
     `src/operation/mod.rs`.** They read and write a file, and AC-27 requires
     `src/operation/**` to gain no `std::fs`/`std::path`. Same cases, same rule (blessed on
     untouched `src/`, own commit `ab94505`).
  2. **A third Call-5 conflict the design did not list:** SPEC-131's corpus recipe
     `v2_watermark_integer_numbers.toml` puts `scale = 1` on a `text` step, so it no longer
     loads. I removed that one line from the `.toml` and the `.golden` (the golden is otherwise
     byte-identical). Its purpose — integers in number keys serialize as integers — still holds
     through `size = 24` and `opacity = 1`; integer `scale` is no longer in the corpus.
     **This is more than a fixture-size change**; flagging it for verify to rule on.
  3. **Four predicted flips passed vacuously instead.** The design's prototype made a refusal a
     plain error, so batch tests failed (exit 6). With the real `Skip` policy,
     `apply_watermark_recipe_matches_cli_at_n_inputs`, `build_hits_when_overlay_bytes_unchanged`,
     `build_hits_when_bundled_font_is_used` and `build_rebuilds_when_font_bytes_change` stayed
     green **with the mark silently skipped** — hollow. Their fixtures were changed anyway, and
     the probe above proves they mark again.
  4. The coverage comparison is `u128`, not `u64`: `width × height × 5` can exceed `u64` for
     `u32` dimensions. Same integer semantics.
  5. `tests/wasm_roundtrip.rs`'s new test also asserts `does not fit`: without it, N1 left it
     green (the coverage cap refuses the same size-200 mark with the same two words).
  6. AC-13's coverage row asserts `5.61 %`, `(1661 of 29584 px)` and `the limit is 5 %`, and
     the AC-4 text row asserts the count equals 1661 as well as the rendered overlay's count.
  7. `CHANGELOG.md` had no Unreleased `### Changed`; one was added. DEC-104's `affected_scope`
     also lists `docs/data-model.md` and `recipes/README.md`.
- **Follow-up work identified:**
  - None new. The cached-run gap is the design's filed item; the `--size nan` item stays open
    (above).

### Fix pass (verify punch list, same draft PR #206)

- Items 1–4 each have a test that goes RED under the named mutation (fresh `CARGO_TARGET_DIR`,
  non-empty `git diff` first) and GREEN once the source is restored:
  M2 → `opacity_does_not_change_coverage` (now `opacity = 0.001`); M1 →
  `coverage_counts_a_sixteen_bit_overlay_at_its_own_depth`; M7 →
  `rule_judges_the_image_entering_the_step_when_it_grew` (the shrink test stays green under M7
  because `apply`'s own guard backs it up, as the prompt predicted); M5 →
  `skipped_count_is_per_image_not_per_step`.
- Docs: the `watermark --help` 5 % clause is scoped to a fixed-size mark; "one explicit input"
  became "one input, counted after globs and directories resolve" in `docs/api-contract.md`,
  `recipes/README.md` and DEC-104; the CHANGELOG Fixed sentence states only the rule; DEC-104
  records that a faint 16-bit mark on an 8-bit base is refused by its pixel count.
- Cleanups: `optimize_decide_one` returns a named `DecideOne`; `run_apply`'s input label is one
  `input_label` helper, computed only when a line is printed; SPEC-131's `scale` rows (the
  range rows and the `scale = true` type row) moved to an image step, each now also asserting it
  is not the mode message; `scale = 1` on an image step is back in the integer corpus recipe.
- **Out-of-spec finding:** `docs/data-model.md:248` still says "On one explicit input"; it is
  outside the fix prompt's boundaries. Proposed: reword it like `docs/api-contract.md`.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The "existing tests that change" list was measured on a prototype whose refusal was an
   *error*, not the final *skip*. So four of its predicted flips never flipped: they went green
   while skipping the mark. Finding them took a probe (abort on any skip), not a test run. And
   the list predated SPEC-132 (`build_watch.rs`) and SPEC-131's corpus recipe.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — AC-27's "no `std::fs` in `src/operation/**`" conflicts with putting a file-writing bless
   test there (the Failing Tests list did). Also, SPEC-131's recipe corpus is a third place
   Call 5 bites.

3. **If you did this task again, what would you do differently?**
   — Run the "abort on any skip" probe straight after implementing, before touching fixtures:
   it splits "flips RED" from "passes hollow" in one run, and it is cheaper than reading each
   test.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Design would probe the change under the behaviour that ships, not the prototype's. The list
   of tests that would flip was measured with a refusal as an *error*; under the real *skip*,
   four of them passed hollow (the mark was skipped, the test stayed green), and only an
   "abort on any skip" probe found them. Verify's own mutations then found four more gaps (M1
   16-bit count, M2 count after opacity, M5 K per step, M7 judging the original input), and
   AC-5's premise was wrong: opacity 0.01 never rounds an alpha to 0. The design's surface table
   also missed the glob or directory that resolves to one file (exit 2, undocumented), the same
   kind of edge SPEC-134 hit. Re-verify left three optional test gaps, filed rather than fixed.

2. **Does any template, constraint, or decision need updating?**
   — Proposed, not made (the orchestrator decides): (a) the design template should ask for an
   "abort on any skip" style probe whenever a change turns an error into a silent fallback or
   skip; (b) a design's surface table should always include "one input reached through a glob
   or directory, and stdin". No DEC or constraint change.

3. **Is there a follow-up spec I should write now before I forget?**
   — None new. Re-verify already filed "SPEC-133's three test gaps…" and "`watermark` prints
   absolute paths…" on STAGE-050's Spec Backlog, and the `--size nan` item is updated there.
