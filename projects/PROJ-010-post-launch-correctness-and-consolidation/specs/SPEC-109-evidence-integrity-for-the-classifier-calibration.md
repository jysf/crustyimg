---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-109
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: critical
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-010
  stage: STAGE-034
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-opus-5       # usually same Claude, different session
  created_at: 2026-07-26

references:
  decisions:
    - DEC-047
  constraints:
    - test-before-implementation
    - every-public-fn-tested
    - one-spec-per-pr
  related_specs:
    - SPEC-105
    - SPEC-108
    - SPEC-084

value_link: >
  Builds the instrument SPEC-108 is measured with. Until a threshold mutation can
  make a test go red, nothing in STAGE-034 is proven.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      note: >
        Un-metered main-loop design cycle. Re-measured all five committed classify
        fixtures with a release build; verified all six guard sites against source.
      estimated_usd: null
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 1
---

# SPEC-109: evidence integrity for the classifier calibration

## Context

STAGE-034's second spec. SPEC-108 changes *what the classifier is given*; this spec makes
the test suite capable of noticing when that goes wrong. The two are separable, and this one
should land **first**: today the classifier's headline guard cannot fail.

**Mutation-verified**: setting `PHOTO_ENTROPY_STRONG = 5.5` — the value that reinstates the
original SPEC-105 bug — leaves `cargo test --release --lib analysis` green at **52 passed,
0 failed**, including the guard named for the job.

### Re-measured during this design cycle

All five committed classify fixtures, release build, `web --json --max 8192` (no resize):

| fixture | class | entropy | flat_ratio | edge_ratio | unique_colors |
|---|---|---|---|---|---|
| `grayscale_photo_leica.png` | photograph | **6.07** | 0.83 | 0.00 | 182 |
| `grayscale_photo_canon.png` | photograph | **6.83** | 0.83 | 0.00 | 233 |
| `color_photo_fuji.png` | photograph | **6.37** | 0.76 | 0.00 | 4096 (sat) |
| `dithered_graphic.png` | graphic-logo | **3.03** | 0.49 | 0.28 | 9 |
| `checker_graphic.jpg` | graphic-logo | 2.78 | 0.00 | 1.00 | 8 |

**The tautology, confirmed with our own numbers.**
`calibration_gap_holds_for_committed_fixtures` (`src/analysis/mod.rs:945`) asserts
`graphic_max < PHOTO_ENTROPY_STRONG <= photo_min` where `graphic_max = 3.03` and
`photo_min = min(6.07, 6.83, 6.37) = 6.07`. So it holds for **any threshold in (3.03, 6.07]**
— a **3.04-wide** window. The gap DEC-047 documents is **(3.43, 4.58]**, width 1.15. The
guard is loose by a factor of ~2.6 and cannot see 5.5.

**The specimens that would close it are not in the repo.** DEC-047 cites a 4.58-floor photo
and a 3.43 16-colour dither; `tests/fixtures/classify/` contains five files and neither is
among them.

**A finding this re-measurement adds, not in the review.** All three photo fixtures measure
`flat_ratio` **0.76–0.83, above `FLAT_GRAPHIC_RATIO` (0.60)**, with `edge_ratio` 0.00, below
`GRAPHIC_EDGE_MAX` (0.08). **Rule 4b would classify every one of them as `GraphicLogo`** if
rule 3.5 did not fire first. That is the "scale-broken flat detector reads photos as ~flat"
hazard DEC-047 describes, and it means **rule 3.5 is genuinely load-bearing** — weakening it
is not a safe simplification. This is design context SPEC-108 depends on, and it is why
SPEC-108 chose to change the classifier's *input* rather than its cascade.

### A correction to the review's site list

The review named five diluted guard sites. Two of them — `tests/cli.rs:4381` and
`tests/cli.rs:4392` — are **the same test**: `:4381` is its doc comment, `:4392` its
signature (`optimize_detailed_icc_source_ships_compact_lossy_not_lossless_blowup`). So the
work is **four distinct test functions plus the two `iso_luma` fixtures**, not five plus two.
Verified by reading each site.

## Goal

Make the classifier's guards capable of failing: commit the two boundary specimens, and give
every diluted guard a negative control that proves it can go red.

## Inputs

- **Files to read:**
  - `src/analysis/mod.rs:945-957` — the tautological calibration guard.
  - `src/analysis/mod.rs:1009-1020` — `iso_luma` and its comment.
  - `src/analysis/mod.rs:1060` — `ambiguous_square_falls_back_to_photograph_low_confidence`.
  - `tests/cli.rs:4380-4438` — the ICC / SPEC-084 test, its `matches!` and its self-encoded
    `blowup` bound.
  - `tests/cli.rs:5016-5032` — `web_normal_case_no_larger_flag` and its EXIF source.
  - `tests/audit_bench.rs:165-173` — the `#[cfg(feature = "avif")]` gate and its `:43` sibling.
  - `decisions/DEC-047-classification-thresholds-and-fallback-bias.md`.

## Outputs

- **Files created:**
  - `tests/fixtures/classify/photo_entropy_floor.png` — the ≈4.58 photo. **Seed it
    independently**; do not generate it by searching for an input that produces 4.58 with the
    code under test ([[fixtures-from-the-code-under-test-cannot-fail]]).
  - `tests/fixtures/classify/dither_16color.png` — the ≈3.43 16-colour dither.
- **Files modified:** `src/analysis/mod.rs`, `tests/cli.rs`, `tests/audit_bench.rs`,
  `decisions/DEC-047-*.md`.

## Acceptance Criteria

- [ ] **AC-1.** The two boundary specimens are committed, with their measured entropy
      **asserted as values**, and registered as `FX_` constants beside the existing four.
- [ ] **AC-2.** `calibration_gap_holds_for_committed_fixtures` asserts the **documented**
      gap, not the loose one: with the specimens in, the window narrows from (3.03, 6.07] to
      approximately (3.43, 4.58]. State the achieved bounds in the test message.
- [ ] **AC-3 — the gate that matters.** With `PHOTO_ENTROPY_STRONG = 5.5`, `cargo test
      --release --lib analysis` **fails**. Record which test fails and its message. Also
      check the other direction: `3.2` should fail too. A guard that only catches one side
      of its window is half a guard.
- [ ] **AC-4.** The ICC test (`tests/cli.rs:4392`) asserts disposition from
      `--explain=json` (`"disposition":"lossy"`) instead of `matches!` on `guess_format`,
      which cannot fail for the formats it lists and whose own comment concedes it cannot
      distinguish lossy from lossless WebP. The stronger form already exists 200 lines away
      in `optimize_grayscale_photo_is_photograph_lossy_avif` (`tests/cli.rs:4637`) — reuse it.
- [ ] **AC-5.** The self-referential `blowup` bound is replaced or supplemented. It encodes
      a lossless WebP with the `image` crate at default effort and asserts we beat it — a
      shipped lossless WebP at higher effort satisfies it
      ([[a-self-referential-control-cannot-detect-a-broken-pipeline]]).
- [ ] **AC-6.** The SPEC-084 metadata-forced lossy-fallback branch
      (`src/cli/optimize.rs:1059`) regains end-to-end coverage. The comment claiming the
      scenario is "only reachable via the misclassification this spec removes" is **false**:
      `checker_graphic.jpg` (measured entropy **2.78**, above) plus an ICC profile still
      reaches it. Use that, and correct the comment.
- [ ] **AC-7.** `web` regains coverage of the **no-EXIF** classification path — the path the
      demo and RAW-preview extraction actually take, since both strip EXIF. The current test
      uses `jpeg_with_exif(3000, 2000)`, whose EXIF makes rule 2 return before rule 3.5 runs.
      Add a no-EXIF case; keep the EXIF one.
- [ ] **AC-8.** `json_shape_consistent_across_verbs` runs on the lean build, or the schema
      fork is **fixed** rather than gated. The `#[cfg(feature = "avif")]` at
      `tests/audit_bench.rs:171` and its sibling at `:43` were silencers. The lean leg is
      CI's only no-AVIF leg, so further forks there ship undetected.
- [ ] **AC-9.** Both `iso_luma` fixtures are corrected. `wide_flat_manycolour_with_edges_is_ui_screenshot`
      reproduces at **25 occupied luma bins, not the ~5 its four flat panels intend, and
      entropy 3.3964** — 0.60 under the threshold it asserts — because `(l + 2*j).clamp(0,255)`
      saturates red, which the comment at `:1009` denies. Either fix the generator so the
      fixture means what it says, or correct the comment and assert the real bin count. Same
      for `ambiguous_square_falls_back_to_photograph_low_confidence` (`:1060`), whose comment
      "frequent steps → not flat-graphic" is false: measured `flat_ratio` is **0.611, above**
      `FLAT_GRAPHIC_RATIO` 0.60; only `edge_ratio` keeps that gate shut.
- [ ] **AC-10.** DEC-047's two false claims are corrected in place:
      (a) the reach claim — "**any** image with luma entropy ≥ `PHOTO_ENTROPY_STRONG` is a
      `Photograph`" is false for `width.max(height) <= 128` with aspect ≤ 2.0 and no EXIF
      (a 128×128 EXIF-stripped photo thumbnail measures entropy **6.02** and classifies
      `Icon` → lossless); (b) the safety claim that no hard-edged graphic reaches 4.0 — the
      committed dither reaches 7.08 at `--max 256`. Its evidence roster gains the two
      specimens.
- [ ] **AC-11.** Clean full-matrix green (default / lean / `webp-lossy`, clippy `-D warnings`
      each, `fmt --check`), with `Compiling crustyimg` in the log.

## Failing Tests

Written during **design**, BEFORE build.

- **`src/analysis/mod.rs`** (unit)
  - `"calibration_gap_matches_the_documented_gap"` — asserts the window is ≈(3.43, 4.58],
    not (3.03, 6.07]. Fails today (no specimens).
  - `"boundary_specimens_measure_their_recorded_values"` — asserts the two new fixtures'
    entropy **as values**, independently seeded. Fails today (files absent).
  - `"iso_luma_fixture_occupies_the_bin_count_it_claims"` — asserts the real occupied-bin
    count. Fails today (25 vs ~5).
- **`tests/cli.rs`**
  - `"optimize_detailed_icc_source_ships_lossy_disposition"` — asserts
    `"disposition":"lossy"` from `--explain=json`. **May pass today** — that is expected and
    is the point: it must be paired with AC-3's mutation to show it is load-bearing
    ([[a-plausible-test-result-is-not-a-checked-one]]).
  - `"spec_084_metadata_forced_fallback_is_reached"` — `checker_graphic.jpg` + ICC. Fails
    today (no coverage).
  - `"web_classifies_a_no_exif_source"` — fails today (no coverage).
- **`tests/audit_bench.rs`**
  - `json_shape_consistent_across_verbs` un-gated. Fails today on lean.
- **The mutation control is not optional.** AC-3 is the spec. If the specimens land and 5.5
  still leaves the suite green, this spec has not delivered, however many tests were added.

## Implementation Context

### Decisions that apply

- `DEC-047` — the record this spec corrects. Do **not** change any threshold *value* here;
  that is out of scope for both specs in this stage.

### Constraints that apply

- `test-before-implementation` (**blocking**).
- `every-public-fn-tested` (warning) — relevant to AC-8's un-gating.
- `one-spec-per-pr` (**blocking**) — the placement change is SPEC-108's PR, not this one.

### Prior related work

- `SPEC-105` — introduced the guards this spec repairs. Its diffs are where the dilutions
  happened; read them for intent before rewriting an assertion.
- `SPEC-084` — the never-bigger guarantee whose branch lost coverage (AC-6).
- `SPEC-108` — the fix. **This spec should land first**; it is the instrument.

### Out of scope (for this spec specifically)

- Moving classification, changing the cascade, or retuning any threshold — SPEC-108.
- Rule 6's dead code and the `Icon` ordering *code* fix — SPEC-108 (AC-6 there). This spec
  corrects only DEC-047's *claim* about Icon reach.
- The dirty-alpha finding — unverified, no specimen, `docs/backlog.md`.

## Notes for the Implementer

- **Seed the specimens independently.** The temptation is to hunt for an image that makes the
  current code print 4.58. That produces a fixture that cannot fail, which is the exact defect
  this spec exists to remove. Construct them from a documented recipe (stated in the fixture's
  companion note), measure what you get, and assert *that* value.
- **Run the mutation before you start and after you finish.** Before: confirm 52/52 green at
  5.5, so you know the baseline. After: confirm red. Without the before-run you cannot claim
  you changed anything ([[a-control-you-never-verified-applied-is-not-a-control]]).
- **`checker_graphic.jpg` is already in the repo at entropy 2.78** — AC-6 needs no new fixture,
  only an ICC profile attached to it.
- **Do not "fix" the `iso_luma` fixture by nudging it under the threshold.** It sits at 3.3964
  against a 4.0 assertion; the failure mode is that any future change to luma weights or panel
  levels pushes it over and the test then blames the classifier. Either make the generator
  produce what the comment claims, or make the comment true and assert the measured value.
- Count the guard sites yourself before claiming completeness — the review's five are four
  tests plus two fixtures, and its own site list conflates a doc comment with a signature.

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

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
