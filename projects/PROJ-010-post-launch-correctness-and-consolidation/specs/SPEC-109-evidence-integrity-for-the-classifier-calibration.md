---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-109
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
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
    - cycle: build
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 65339132
      duration_minutes: 141
      recorded_at: 2026-07-26
      tokens_breakdown:
        input: 560
        output: 296789
        cache_creation: 568120
        cache_read: 64473663
      estimated_usd: 43.21
      note: >
        MEASURED, not estimated. Ran interactively (main-loop), so there was no
        `subagent_tokens` to read; the numbers are summed from this session's own
        transcript at
        ~/.claude/projects/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg/35c1e8dd-a7cd-4dbb-a812-5f018b157ea9.jsonl
        over 297 assistant messages, 17:30:39Z-19:52:08Z.
        `estimated_usd` DEPARTS from the AGENTS.md formula deliberately. That formula
        (tokens_total x list rate, ~80/20 in/out, no cache discount) assumes cache reads
        are absent or negligible; here they are 98.7% of the volume, and it yields $588
        against a component-accurate $43.21 — a 14x overstatement that would corrupt
        every cost report this spec feeds. The figure above prices each component at the
        Opus $5/$25 per MTok anchors AGENTS.md names, with the standard cache multipliers
        (write 1.25x input, read 0.10x input). See the follow-up on `cost-snippet.md`.
    - cycle: verify
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 21152459
      duration_minutes: 227
      recorded_at: 2026-07-26
      tokens_breakdown:
        input: 289
        output: 143450
        cache_creation: 637593
        cache_read: 20371127
      estimated_usd: 17.76
      note: >
        MEASURED, not estimated. Ran interactively (main-loop), so there was no
        `subagent_tokens` to read; summed from this session's own transcript at
        ~/.claude/projects/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg/5fb69733-2d8c-4220-8428-dfc7ee9cdebf.jsonl
        over 154 assistant messages, 21:28:25Z-01:14:55Z. Measured at the point of
        writing this entry; the tail of the session is not included.
        `estimated_usd` DEPARTS from the AGENTS.md formula for the same reason the
        build entry does: cache reads are 96.3% of the volume here, and the flat
        formula yields $190.37 against a component-accurate $17.76. Priced at the
        Opus $5/$25 per MTok anchors AGENTS.md names, with the standard cache
        multipliers (write 1.25x input, read 0.10x input). Much of the wall-clock is
        cargo builds: three full-matrix legs from an empty target dir plus eleven
        mutation rebuilds.
  totals:
    tokens_total: 86491591
    estimated_usd: 60.97
    session_count: 3
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

- [x] **AC-1.** The two boundary specimens are committed, with their measured entropy
      **asserted as values**, and registered as `FX_` constants beside the existing four.
- [x] **AC-2.** `calibration_gap_holds_for_committed_fixtures` asserts the **documented**
      gap, not the loose one: with the specimens in, the window narrows from (3.03, 6.07] to
      approximately (3.43, 4.58]. State the achieved bounds in the test message.
- [x] **AC-3 — the gate that matters.** With `PHOTO_ENTROPY_STRONG = 5.5`, `cargo test
      --release --lib analysis` **fails**. Record which test fails and its message. Also
      check the other direction: `3.2` should fail too. A guard that only catches one side
      of its window is half a guard.
- [x] **AC-4.** The ICC test (`tests/cli.rs:4392`) asserts disposition from
      `--explain=json` (`"disposition":"lossy"`) instead of `matches!` on `guess_format`,
      which cannot fail for the formats it lists and whose own comment concedes it cannot
      distinguish lossy from lossless WebP. The stronger form already exists 200 lines away
      in `optimize_grayscale_photo_is_photograph_lossy_avif` (`tests/cli.rs:4637`) — reuse it.
- [x] **AC-5.** The self-referential `blowup` bound is replaced or supplemented. It encodes
      a lossless WebP with the `image` crate at default effort and asserts we beat it — a
      shipped lossless WebP at higher effort satisfies it
      ([[a-self-referential-control-cannot-detect-a-broken-pipeline]]).
- [x] **AC-6.** The SPEC-084 metadata-forced lossy-fallback branch
      (`src/cli/optimize.rs:1059`) regains end-to-end coverage. The comment claiming the
      scenario is "only reachable via the misclassification this spec removes" is **false**:
      `checker_graphic.jpg` (measured entropy **2.78**, above) plus an ICC profile still
      reaches it. Use that, and correct the comment.
- [x] **AC-7.** `web` regains coverage of the **no-EXIF** classification path — the path the
      demo and RAW-preview extraction actually take, since both strip EXIF. The current test
      uses `jpeg_with_exif(3000, 2000)`, whose EXIF makes rule 2 return before rule 3.5 runs.
      Add a no-EXIF case; keep the EXIF one.
- [x] **AC-8.** `json_shape_consistent_across_verbs` runs on the lean build, or the schema
      fork is **fixed** rather than gated. The `#[cfg(feature = "avif")]` at
      `tests/audit_bench.rs:171` and its sibling at `:43` were silencers. The lean leg is
      CI's only no-AVIF leg, so further forks there ship undetected.
- [x] **AC-9.** Both `iso_luma` fixtures are corrected. `wide_flat_manycolour_with_edges_is_ui_screenshot`
      reproduces at **25 occupied luma bins, not the ~5 its four flat panels intend, and
      entropy 3.3964** — 0.60 under the threshold it asserts — because `(l + 2*j).clamp(0,255)`
      saturates red, which the comment at `:1009` denies. Either fix the generator so the
      fixture means what it says, or correct the comment and assert the real bin count. Same
      for `ambiguous_square_falls_back_to_photograph_low_confidence` (`:1060`), whose comment
      "frequent steps → not flat-graphic" is false: measured `flat_ratio` is **0.611, above**
      `FLAT_GRAPHIC_RATIO` 0.60; only `edge_ratio` keeps that gate shut.
- [x] **AC-10.** DEC-047's two false claims are corrected in place:
      (a) the reach claim — "**any** image with luma entropy ≥ `PHOTO_ENTROPY_STRONG` is a
      `Photograph`" is false for `width.max(height) <= 128` with aspect ≤ 2.0 and no EXIF
      (a 128×128 EXIF-stripped photo thumbnail measures entropy **6.02** and classifies
      `Icon` → lossless); (b) the safety claim that no hard-edged graphic reaches 4.0 — the
      committed dither reaches 7.08 at `--max 256`. Its evidence roster gains the two
      specimens.
- [x] **AC-11.** Clean full-matrix green (default / lean / `webp-lossy`, clippy `-D warnings`
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

- **Branch:** `spec-109-classifier-evidence-integrity`
- **PR (if applicable):** not yet opened
- **All acceptance criteria met?** yes (AC-1 … AC-11), with two deviations recorded below
- **New decisions emitted:** none. DEC-047 was amended in place (AC-10); nothing here
  chose anything a `DEC-*` would record.

### The gate — AC-3, before and after

| run | `PHOTO_ENTROPY_STRONG` | result |
|---|---|---|
| **before** (baseline, tree clean) | 5.5 | **52 passed, 0 failed** — green, as the design measured |
| before, positive control | 7.0 | 49 passed, **3 failed** — proves the mutation reaches the build |
| after | 4.0 (shipped) | 54 passed, 0 failed |
| **after** | **5.5** | **RED — 52 passed, 2 failed** |
| **after** | **3.2** | **RED — 51 passed, 3 failed** |

The 7.0 run is the control the baseline needed. 5.5 green before and 5.5 red after only
means something if a threshold move can move the suite at all
([[a-control-you-never-verified-applied-is-not-a-control]]); at 7.0 the pre-change suite
went red in three places, so the harness was live and 52/52 at 5.5 was a real result.

Failing tests and messages at 5.5:

- `calibration_gap_matches_the_documented_gap` — *"threshold 5.5 must fall in the
  calibration window (3.6414278, 4.5176096] (width 0.87618184 bits, cap 1.2)"*
- `boundary_specimens_measure_their_recorded_values` — `photo_entropy_floor.png` falls to
  `GraphicLogo` (61 colours trips the palette gate once rule 3.5 stops firing)

At 3.2 a third test fails — `wide_flat_manycolour_with_edges_is_ui_screenshot`, *"a
realistic screenshot must stay below the strong-entropy floor: 3.3964376"*. Both edges of
the window are caught, so this is a whole guard rather than half of one.

### Acceptance criteria

| AC | Status | Evidence |
|---|---|---|
| AC-1 | ✅ | `photo_entropy_floor.png` (4.5176) and `dither_32color.png` (3.6414) committed, registered as `FX_PHOTO_FLOOR` / `FX_DITHER_32`, entropies asserted **as values** |
| AC-2 | ✅ | window narrowed (3.03, 6.07] → **(3.6414, 4.5176]**, 3.04 bits → 0.88; achieved bounds and width are in the failure message |
| AC-3 | ✅ | table above — red at 5.5 **and** at 3.2 |
| AC-4 | ✅ | `optimize_detailed_icc_source_ships_lossy_disposition` asserts `"disposition":"lossy"` from `--explain=json`; the `matches!` on `guess_format` is gone |
| AC-5 | ✅ | self-referential bound demoted to a labelled supplement; the load-bearing byte check is now report-honesty vs the file on disk |
| AC-6 | ✅ | `spec_084_metadata_forced_fallback_is_reached` — `checker_graphic.jpg` + ICC; the false comment is corrected |
| AC-7 | ✅ | `web_classifies_a_no_exif_source` (photo → `photograph`, dither → `graphic-logo`), EXIF-absence asserted; the EXIF case kept |
| AC-8 | ✅ | both `#[cfg(feature = "avif")]` gates removed; `audit_bench` runs 6 tests on the lean leg, was 5 |
| AC-9 | ✅ | `iso_luma_fixture_occupies_the_bin_count_it_claims` pins 25 / 14 bins, 3.3964 / 3.1905 entropy, and flat_ratio 0.611 > `FLAT_GRAPHIC_RATIO`; all three comments corrected |
| AC-10 | ✅ | DEC-047 carries two dated correction blocks + the specimens in its roster |
| AC-11 | ✅ | clean full matrix, fresh `CARGO_TARGET_DIR`, `Compiling crustyimg` observed — table below |

### AC-11 — clean full matrix

Built into an empty `CARGO_TARGET_DIR` (`rm -rf` first), so nothing is inherited. Each
leg's log contains `Compiling crustyimg` exactly once, which is the check that this is a
real build and not a stale-artifact green ([[a-stale-incremental-build-is-a-false-green]]).

| leg | exit | suites | passed | failed | `Compiling crustyimg` |
|---|---|---|---|---|---|
| `cargo test --no-default-features` | 0 | 32 | 776 | 0 | 1 |
| `cargo test` | 0 | 32 | 796 | 0 | 1 |
| `cargo test --features webp-lossy` | 0 | 32 | 803 | 0 | 1 |
| `cargo clippy --all-targets -- -D warnings` | 0 | | | | |
| `cargo clippy --all-targets --no-default-features -- -D warnings` | 0 | | | | |
| `cargo clippy --all-targets --features webp-lossy -- -D warnings` | 0 | | | | |
| `cargo fmt --check` | 0 | | | | |

Exit codes were read directly, not through a pipe — `cmd | tail` reports *tail's* status,
which silently turned a failing leg into a green one earlier in this cycle.

### The six guard sites, counted independently

Confirmed by reading each, not by trusting the handed list. The review's "five" is four
tests plus two fixtures: `tests/cli.rs:4381` is a doc comment and `:4392` the signature of
**the same function**, as the spec says.

1. `src/analysis/mod.rs` calibration guard — rewritten, window cap added
2. `tests/cli.rs` ICC never-bigger assertion — disposition from `--explain=json`
3. `tests/cli.rs` `web` no-EXIF path — new test
4. `tests/cli.rs` SPEC-084 fallback coverage — new test (same function as site 2's doc)
5. `tests/audit_bench.rs` schema test + its `top_level_keys` helper — both un-gated
6. `src/analysis/mod.rs` two `iso_luma` fixtures — pinned and corrected

Mechanical cross-check on the gates, run in Python and again through `rtk proxy grep`
(plain `grep` is rewritten to a broken `rg` regex in this environment and reported 0
matches — [[rtk-can-silently-corrupt-grep-counts]]): 27 `cfg(feature)` attributes remain
under `tests/`, of which `audit_bench.rs` holds **0** — its single remaining match is the
words inside my own doc comment.

### Deviations from spec

1. **`dither_16color.png` → `dither_32color.png` (32 grey levels, not 16).** A 16-colour
   dither cannot do this job with the photographs this repo holds. Quantising to L levels
   costs about `log2(256/L)` bits, so 16 levels of a 6.07–6.83-bit source lands at
   2.46–2.88 — measured 2.80 for the Fuji frame, 2.46 for Leica, 2.88 for Canon — all
   **below** the 3.03 dither already committed, so the lower bound would not move and 3.2
   would still pass. Reaching DEC-047's cited 3.43 at 16 levels needs a ~7.4-bit source,
   which none of these are; histogram-equalising first gets there (measured 3.94) but
   leaves 0.06 bits of margin under the threshold, which is a fixture that flips on any
   small change. 32 levels of the unmodified photograph gives the same boundary role with
   0.36 bits of margin. Recorded in `tests/fixtures/classify/RECIPES.md` and in DEC-047.
2. **AC-8 resolved by running the test everywhere, not by fixing the schema fork.** The
   fork is real and I reproduced it, but its cause is not what the gate's comment claimed
   (see below), and the fix belongs to SPEC-108. The test now runs on every leg against a
   `LosslessFlat` source whose shortlist is codec-independent — so it will catch a *new*
   fork on the lean leg, which is what the gate was suppressing. It does not catch the
   existing one. Follow-up filed.

### Things found that the spec did not predict

- **The gate's stated reason for the schema fork was false.** It blamed "an unscored JPEG
  `web` winner" on no-AVIF builds. Measured cause: `web`/`apply` run the source through the
  resize pipeline and report `has_alpha: true`, while `optimize` reports `false` for the
  same file. On a no-AVIF build a `Lossy`-bucket image **with alpha** shortlists exactly
  `[lossless(Png)]`, that PNG loses to the source, and the verb passes through with no
  winner and so nothing to score. That is `decide.rs`'s missing lossy-alpha fallback — the
  finding STAGE-034 already lists for SPEC-108 — seen from a third direction.
- **SPEC-084 does not promise never-bigger-than-source on this branch, and I asserted that
  it did.** Twice: the new `spec_084_*` test and the rewritten ICC test both first asserted
  `out < src`, and both failed — on the checker graphic (28,652 B out for 15,291 B in) and
  on the lean leg of the ICC test (7,231 B for 6,101 B). Reading the branch, that is
  deliberate: when stripping metadata forces a re-encode an already-tight source can beat,
  it ships the smallest correct output *and reports the truth* rather than clamping to a
  break-even "0% smaller". Both tests now assert the honesty invariant, which is the
  guarantee that actually exists.
- **`photo_entropy_floor.png` measures `flat_ratio` 1.00.** The flat detector reads the
  photo floor as *completely* flat, so rule 3.5 alone keeps it off the lossless path. This
  is independent confirmation of the stage's "rule 3.5 is load-bearing" finding, from a
  specimen built without that finding in view.
- **The `optimize_detailed_icc_source_*` shortlist is leg-dependent** — one candidate lean,
  three under `webp-lossy` where the winner is index 1. A `"winner":0` assertion (copied
  from the grayscale-photo test) pinned the codec set rather than the guarantee; replaced.

### Follow-up work identified

- **The `explain/v1` schema forks on `ssim` between `optimize` and `web`/`apply` on
  no-AVIF builds**, because the two disagree on `has_alpha` for the same source. Root cause
  is the missing lossy-alpha fallback at `src/analysis/decide.rs` — already SPEC-108's
  AC. Worth naming explicitly there: the schema symptom is a second consumer-visible face
  of that bug, and `json_shape_consistent_across_verbs` will not catch it.
- **`--profile docs` still has no `tests/cli.rs` coverage** (STAGE-034 lists it; SPEC-108
  owns the behaviour decision). Untouched here.
- **DEC-047's 16-colour 3.43 row is unreproducible** from the repo's fixtures. Left in place
  with the arithmetic recorded beside it rather than silently deleted, since the original
  measurement was presumably of a source we no longer have.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing was unclear; one thing was *unstated and wrong to assume*. The spec names the
   boundary specimen by its target value (`dither_16color.png`, ≈3.43) without saying which
   property is load-bearing — the number, the palette size, or the filename. It is the
   number, and specifically that it sit above 3.2, because AC-3's lower edge depends on it.
   I spent four seeding attempts (EGA-16 → median-cut-16 → 4-bit grey → equalised) before
   working out that the entropy of an L-level dither is pinned by arithmetic to about
   `source − log2(256/L)`, which makes 16 levels impossible here and 32 obvious. Stating the
   window the specimen must land in, rather than the value it should print, would have got
   me there first try.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — SPEC-084's actual guarantee. Both AC-5 and AC-6 concern a never-bigger branch, and the
   natural reading of "never-bigger" is "output ≤ source" — which is false on exactly the
   branch AC-6 targets. It cost me two red tests to find out from the code. A one-line
   pointer to the branch's own comment ("if even the smallest correct output still exceeds
   the source, we ship it anyway — but the report tells the truth") would have prevented
   both. Worth pulling into DEC-084's territory or the spec's Implementation Context.

3. **If you did this task again, what would you do differently?**
   — Build the independent measuring tool first, before touching any fixture. I wrote the
   Python PNG reader/entropy implementation to *seed* the specimens, then noticed it made a
   far better oracle — it reproduces all four committed fixtures to four decimals, which is
   what lets the new assertions be values rather than round-trips. Had I built it first I
   would also have predicted the 16-level dead end analytically instead of discovering it by
   generating three fixtures that did not tighten the window at all.

---

## Reflection (Ship)

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
