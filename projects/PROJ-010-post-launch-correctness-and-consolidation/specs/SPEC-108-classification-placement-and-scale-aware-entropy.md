---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-108
  type: bug                        # epic | story | task | bug | chore
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
    - clippy-fmt-clean
    - test-before-implementation
    - no-unwrap-on-recoverable-paths
    - one-spec-per-pr
  related_specs:
    - SPEC-105
    - SPEC-109

value_link: >
  STAGE-034's whole reason to exist: stop `web` promoting dithered and halftoned
  graphics to lossy AVIF because the resize changed their entropy.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      note: >
        Un-metered main-loop design cycle. Included one release build and four
        instrumented `web --json` runs against the committed fixture.
      estimated_usd: null
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 1
---

# SPEC-108: classification placement and scale-aware entropy

## Context

`web` promotes dithered and halftoned graphics to lossy AVIF, returning a file
**18.5× larger than its input and visually degraded** (SSIMULACRA2 69.2) through the
default path with no flags. Root cause, confirmed by reading source: classification
runs on the **output of the resize pipeline**, not on the input —
`pipeline.run` (`src/cli/optimize.rs:989`) → `Analysis::compute` (`:1013`) →
`decide::format_shortlist` (`:1026`). So `--max` decides the content class.

Parent stage: **STAGE-034**. Sibling: **SPEC-109** (evidence integrity) — it commits the
boundary specimens and the negative controls this spec's fix will be measured against.
Full evidence: `docs/research/pr113-classifier-review-findings.md`, section
"Re-derivation (2026-07-25)".

### The design question this spec had to answer, and the measurement that answered it

STAGE-034 carried two candidate fixes and told the design cycle to choose on measured
complexity, flagging the second as attractive:

- **(a) Placement** — classify the *original*, before the resize pipeline.
- **(b) Narrow gating** — keep classification where it is, delete rule 3.5's unconditional
  early return, and instead gate rule 4's two clauses on `entropy < PHOTO_ENTROPY_STRONG`.
  Advertised benefits: same depth, keeps rules 5 and 6 reachable, keeps `PHOTO_ENTROPY`
  live, and the mask is deletable verbatim once the detector is fixed.

**Measured, against the committed fixture `tests/fixtures/classify/dithered_graphic.png`,
release build, `web --json`:**

| `--max` | class | entropy | edge_ratio | flat_ratio | unique_colors |
|---|---|---|---|---|---|
| 4096 (native) | `graphic-logo` | 3.03 | 0.28 | 0.49 | **9** |
| 512 | `graphic-logo` | 3.03 | 0.28 | 0.49 | **9** |
| **256** | **`photograph`** | **7.08** | 0.05 | **0.27** | **217** |
| 128 | `icon` | 7.15 | 0.07 | 0.36 | 207 |

Now trace option (b) at `--max 256` with the real constants
(`PALETTE_COLORS = 256`, `ICON_MAX_EDGE = 128`, `FLAT_GRAPHIC_RATIO = 0.60`,
`DOC_ENTROPY_MAX = 4.5`, `PHOTO_ENTROPY = 5.0`, `PHOTO_ENTROPY_STRONG = 4.0`):

- `unique_colors = 217 ≤ 256` and not saturated → **`few_colors` is TRUE**, so
  **`many_colors` is FALSE**.
- Rule 1 Icon — `256 > ICON_MAX_EDGE` → no.
- Rule 2 EXIF — none → no.
- Rule 3 Document — needs `entropy < 4.5`; 7.08 → no.
- Rule 4a — `few_colors` TRUE, but gated on `entropy < 4.0`; 7.08 → **skipped**.
- Rule 4b — `flat_ratio 0.27 >= 0.60` → no.
- Rule 5 UI — requires `many_colors` → **unreachable**.
- Rule 6 — requires `many_colors` → **unreachable**.
- Rule 7 fallback → **`Photograph`**, confidence 0.4.

**Option (b) does not fix the defect.** It changes which line returns `Photograph`, not the
answer. And the reason generalises: **rule 7's fallback bias is `Photograph`** (DEC-047,
deliberately — a photo forced lossless is merely a bigger file). So *any* fix that only
stops the graphic gates from firing lands on `Photograph` anyway. Option (b) is structurally
incapable of fixing an input whose correct answer is "graphic", which is precisely this
defect's blast radius.

Note this also disposes of the hope that (b) would rescue rules 5 and 6 for free here:
`few_colors` is TRUE at `--max 256`, so both stay unreachable on this input regardless.

**Decision: option (a), placement.** It is directly supported by the same table — at native
and at `--max 512` the fixture measures 9 unique colours and entropy 3.03 and classifies
`graphic-logo` correctly. Classifying the input is classifying the thing the user actually
gave us; the resized buffer is a derived artifact whose entropy is a resampling artifact,
not content.

**Consequence to carry into the stage.** The brief recorded a risk: *"three of the seven
brought-in findings are only cheap if the narrow rule-4 gating fix wins."* **That risk has
materialised.** Under (a), rule 3.5 keeps its unconditional early return, so rules 5 and 6
stay unreachable and rule 6's dead code must be handled explicitly rather than falling out.
This spec does that (AC-6); it does not silently leave it.

## Goal

Classify the **source image**, not the resize output, so `--max` cannot change an image's
content class — and leave the cascade with no unreachable rule behind.

## Inputs

- **Files to read:**
  - `src/cli/optimize.rs:960-1070` — the pipeline/analyse/decide sequence being reordered;
    `:989` `pipeline.run`, `:1013` `Analysis::compute`, `:1026` `format_shortlist`,
    `:1043-1059` the lossless-fallback hazard comment and `fast_fallback_lossy_entry`.
  - `src/cli/optimize.rs:150-170` — the **second** `pipeline.run` at `:160`. Establish
    whether it feeds a classify path too; if it does, it needs the same treatment.
  - `src/analysis/mod.rs:556-631` — `classify`, the seven-rule cascade.
  - `src/analysis/mod.rs:75-101` — the constants.
  - `decisions/DEC-047-classification-thresholds-and-fallback-bias.md` — the rule this
    changes the *input* of; SPEC-109 corrects its two false claims separately.
- **Related code paths:** `src/analysis/`, `src/cli/optimize.rs`, `src/pipeline/`.

## Outputs

- **Files modified:**
  - `src/cli/optimize.rs` — compute `Analysis` from the source image before
    `pipeline.run`, and thread it to the decision site.
  - `src/analysis/mod.rs` — resolve rule 6 (AC-6) and the `[4.0, 4.5)` band (AC-5).
  - `decisions/DEC-NNN-*.md` — **a new decision record** for "classify the source, not the
    pipeline output", superseding the placement implied by DEC-047's context.
- **New exports:** none expected. If threading the analysis requires a signature change,
  keep it internal (`pub(super)`/`pub(crate)`).

## Acceptance Criteria

- [x] **AC-1.** `tests/fixtures/classify/dithered_graphic.png` classifies `graphic-logo` and
      takes a lossless disposition at **every** `--max` in {4096, 512, 256, 128} — the exact
      set measured above, so the fix is checked at the two sizes that currently break (256)
      and that currently pass for the wrong reason (128, rescued by the Icon rule).
- [x] **AC-2.** The reported `features` block for a given input is **identical across
      `--max` values**, because it now describes the source. This is the structural
      assertion; AC-1 is the behavioural one.
- [x] **AC-3.** The 1-bit halftone boundary specimen (committed by SPEC-109) produces a
      lossless or smaller-lossy output through default `web`, not an 18.5× lossy AVIF.
      `larger_than_source` is false.
- [x] **AC-4.** A real photograph still classifies `photograph` and routes lossy at every
      `--max` — the fix must not trade this defect for its mirror image. Use the committed
      `color_photo_fuji.png` and `grayscale_photo_leica.png`.
- [x] **AC-5.** The `[4.0, 4.5)` contradiction band is resolved: `DOC_ENTROPY_MAX` (4.5)
      must no longer exceed `PHOTO_ENTROPY_STRONG` (4.0), or the ordering dependency must be
      documented at the site and covered by a test. A halftone scan at entropy 4.2 with
      `bimodality ≈ 0.30` must not reach `Photograph`.
- [x] **AC-6.** Rule 6 (`src/analysis/mod.rs:625`) is **either reachable or deleted**, and
      `PHOTO_ENTROPY` / `PHOTO_FLAT_MAX` are correspondingly live or gone. No inert
      constants remain. Deleting is an acceptable outcome and probably the right one —
      but say which was chosen and why in the DEC.
- [x] **AC-7.** On the lean leg (`--no-default-features`), a promoted photograph **with
      alpha** does not ship a PNG blow-up (`src/analysis/decide.rs:150`). This is a
      precondition check on our own change: the finding is conditional on the pipeline being
      altered, and this spec alters it.
- [x] **AC-8.** `--profile docs` has a decided, tested behaviour for a promoted image.
      There is currently no `(Profile::Docs, OptBucket::Lossy)` arm and **no `--profile
      docs` test in `tests/cli.rs` at all**. Decide the behaviour, then test it.
- [x] **AC-9.** Clean **full-matrix** green: default, `--no-default-features`,
      `--features webp-lossy`; `clippy -D warnings` on each; `fmt --check`. Confirm the log
      says `Compiling crustyimg` — an incremental build false-greens here and cost this repo
      about a day on SPEC-105. [[a-stale-incremental-build-is-a-false-green]]

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in **build** is to make these
pass. All are expected to FAIL against current `main` except where noted.

- **`tests/cli.rs`**
  - `"dithered_graphic_stays_graphic_at_every_max"` — drives the committed fixture at
    `--max` 4096/512/256/128 with `--json`, asserts `class == "graphic-logo"` and a lossless
    disposition at each. **Fails today at 256** (`photograph`) **and at 128 for the wrong
    reason** (`icon`). Assert on `--explain`/`--json` output, not on output bytes — the
    stronger form already used by `optimize_grayscale_photo_is_photograph_lossy_avif`
    (`tests/cli.rs:4637`).
  - `"classification_is_independent_of_max"` — same input at two `--max` values; asserts the
    emitted `features.entropy` and `features.unique_colors` are **equal**. Fails today
    (3.03/9 vs 7.08/217). This is the structural guard: it stays meaningful even if the
    thresholds are later retuned.
  - `"real_photo_stays_photograph_at_every_max"` — the AC-4 mirror. **Passes today**; it is
    the regression guard against over-correcting, and must be written anyway
    ([[a-plausible-test-result-is-not-a-checked-one]] — a test that only ever passed is not
    evidence until something can make it fail; pair it with the AC-4 note below).
  - `"docs_profile_downgrades_a_promoted_image"` — AC-8. Fails today (silent no-op).
- **`src/analysis/mod.rs`** (unit)
  - `"halftone_scan_in_the_contradiction_band_is_not_a_photograph"` — synthesises the
    entropy-4.2 / `bimodality ≈ 0.30` case for AC-5. Fails today.
  - `"rule_six_is_reachable_or_absent"` — if rule 6 is kept, a test that **hits it**; if
    deleted, this test is deleted with it and the DEC records that. Do not leave a test that
    asserts a rule exists without exercising it
    ([[a-harness-that-exercises-nothing-reports-green]]).
- **Negative controls** (the point of SPEC-109, restated here because this spec must not
  ship without them):
  - Setting `PHOTO_ENTROPY_STRONG = 5.5` must make at least one test **RED**. It currently
    leaves the suite green — `cargo test --release --lib analysis` passes **52/52** with that
    mutation applied. Until that mutation goes red, none of the above is proven to be
    load-bearing.

## Implementation Context

### Decisions that apply

- `DEC-047` — classification thresholds and fallback bias. This spec changes what the
  classifier is *given*, not its thresholds. Its two false claims are SPEC-109's, not
  this spec's — do not fix them here (`one-spec-per-pr`).
- **A new DEC is required** for "classify the source, not the pipeline output", recording
  the measured refutation of the narrow alternative above so it is not re-proposed.

### Constraints that apply

- `clippy-fmt-clean` (**blocking**) — "No dead code; delete rather than comment out." Rule 6
  breaks this today and its automated gate **cannot** catch it (the constants stay
  syntactically referenced, so `-D warnings` is green). AC-6 is the constraint, not a nicety.
- `test-before-implementation` (**blocking**) — the Failing Tests above go in first.
- `no-unwrap-on-recoverable-paths` (**blocking**) — `Analysis::compute` already returns a
  `Result` handled with a `match` at `:1013`; preserve that shape when moving it.
- `one-spec-per-pr` (**blocking**) — SPEC-109's specimens and DEC-047 corrections are a
  separate PR.

### What SPEC-109 established that changes this spec's picture

SPEC-109 shipped and was independently verified after this spec was designed. Four things it
found bear directly on the build:

1. **Rule 3.5 is load-bearing, confirmed twice from independent directions.** All three photo
   fixtures measure `flat_ratio` 0.76–0.83 (above `FLAT_GRAPHIC_RATIO` 0.60) with `edge_ratio`
   0.00 — and the new `photo_entropy_floor.png` specimen measures `flat_ratio` **1.00**. Rule 4b
   would claim every one of them if rule 3.5 did not fire first. **Do not weaken or reorder rule
   3.5 as part of the placement change.** Its unconditional early return is the only thing
   holding real photographs off the lossless path while the flat detector stays scale-broken.
2. **The margin above the graphic class is 0.16 bits, not 0.36 — and the ceiling is unknown.**
   DEC-047's "≤3.64 counting dithers-of-photos" is one specimen's value; the same recipe on the
   Canon frame gives 3.8396. If any dither of a photo exceeds 4.0 at native size, placement does
   not save it — it classifies `photograph` on its own merits. Filed in `docs/backlog.md`. **Do
   not treat 4.0 as validated headroom.**
3. **SPEC-084 makes no never-bigger-than-source promise on the metadata-forced branch.** The
   build session learned this from the code at the cost of two red tests. Read
   `src/cli/optimize.rs:1043-1059` before assuming that branch guarantees anything about output
   size relative to input.
4. **AC-7 here is the root-cause fix for a defect SPEC-109 could only work around.** The
   cross-verb schema fork in `tests/audit_bench.rs` is a **`has_alpha` disagreement** between
   `optimize` and `web`/`apply` — *not* the "unscored JPEG winner" its comment claimed. SPEC-109
   un-gated the test rather than fixing the fork, and in doing so moved its source from a
   `Lossy`-bucket photo to a `LosslessFlat` graphic: the lean leg gained coverage, **the default
   leg lost cross-verb coverage for a `Lossy` source.** Fixing `decide.rs:150` should restore it;
   check that it does.

### Prior related work

- `SPEC-105` — introduced rule 3.5 and this regression. Read its reasoning before changing
  the cascade; the rule it added is correct in intent and was given the wrong input.
- `SPEC-109` — commits the boundary specimens and the negative controls. **Sequencing:** its
  specimens are needed for AC-3. Either land SPEC-109 first, or have this spec's build commit
  the two specimens and let SPEC-109 own the guard rework.

### Out of scope (for this spec specifically)

- Retuning any threshold value. The numbers stay; the *input* changes.
- Rewriting the flat/edge detector. This spec **subsumes** the queued "scale-normalize the
  flat/edge detector" follow-up in the sense that classifying the source removes the
  scale-dependence that motivated it — **verify that before closing the follow-up**, and if
  a residue remains, re-file it rather than letting it disappear.
- Luma entropy ignoring alpha (`src/analysis/mod.rs:248`) — unverified, no specimen; see
  `docs/backlog.md`. Note the fixture at `--max 256` reports `has_alpha: true`, so resist the
  temptation to chase this here.
- The `Icon` ordering *code* fix, and DEC-047's false claims — SPEC-109.

## Notes for the Implementer

- **The `--max 128` row is a trap.** It currently returns `icon` → lossless, which *looks*
  like a pass. It is the Icon rule firing on size, not the classifier being right. A test
  that only checks "lossless at 128" goes green on the broken build. Assert the **class**.
- **Check both `pipeline.run` sites.** `:989` is the one the findings name; there is another
  at `:160`. If it also feeds a classify path, fixing only one leaves a second door open —
  and cite the grep when you claim it doesn't ([[mechanical-sweeps-need-a-mechanical-check]]).
- **Watch the alpha/EXIF interaction when moving the analysis earlier.** The pipeline bakes
  orientation and drops metadata (DEC-017); rule 2 keys on `has_exif`. Classifying *before*
  the pipeline means EXIF is still present where it previously was not — that is the correct
  input, but it will change behaviour for EXIF-bearing inputs, and `web`'s no-EXIF path
  currently has **zero** coverage (`tests/cli.rs:5023`, SPEC-109's site). Expect real
  fallout here and measure it rather than assuming it is inert. This is the single most
  likely place for this spec to surprise you.
- Reuse the existing `--json` `features` block for assertions; it already emits entropy,
  `edge_ratio`, `flat_ratio`, `unique_colors`, `unique_saturated`, `has_alpha`
  (`src/analysis/decide.rs:499`).
- Reproduce the measurement table before changing anything, to confirm your build agrees
  with this design's numbers. If it does not, stop and reconcile — do not build on top of a
  disagreement.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `spec-108-classify-the-source-image`
- **PR (if applicable):** not opened (maintainer's call, per repo guardrails)
- **All acceptance criteria met?** yes — AC-1 through AC-9, all verified (see per-AC checkboxes
  above and the timeline's build entry for detail).
- **New decisions emitted:**
  - `DEC-084` — classify the source image, not the pipeline output (also records the AC-5/6/7/8
    sub-decisions and the measured refutation of the narrow alternative). Numbered 084, not the
    next-looking 083: DEC-083 is reserved on the unmerged `chore/cost-measurement-methodology`
    branch per the build prompt's cost section, so 084 avoids a future collision.
- **Deviations from spec:**
  - AC-5's contradiction-band fix touches `classify()`'s cascade structure (splits rule 3.5 into
    an unconditional zone and a contested zone that reuses rule 4's own conditions) rather than
    only "documenting the ordering dependency" — the spec offered that as one of two options, but
    it would not have satisfied the AC's own literal test case (a halftone scan at entropy 4.2 /
    bimodality ≈0.30 fails Document's bimodality gate regardless of where `DOC_ENTROPY_MAX` sits,
    so a comment-only fix could not make the AC's test pass). Reasoning and the measured safety
    argument (no committed fixture falls in the contested band) are in DEC-084.
  - Found and fixed a second, related bug en route to AC-2: `has_alpha` was read from the
    post-pipeline buffer (always internally RGBA) rather than the source, so a plain JPEG — which
    never has a real alpha channel — reported `has_alpha: true`. This is the same root cause
    SPEC-109 identified as "SPEC-108's to fix" for the `optimize`/`web` cross-verb `has_alpha`
    disagreement (its own comment in `tests/audit_bench.rs`). Fixing it (required by AC-2's own
    text: the features block "now describes the source") also required updating three pre-existing
    tests (`non_json_output_unchanged`, `web_output_larger_than_original_is_surfaced`,
    `web_larger_than_original_noted_on_default_channel`) whose fixtures had unknowingly depended on
    the incorrect `has_alpha: true` to construct a "no lossy candidate available" reproduction on
    the lean leg. Not listed as a numbered AC because it surfaced during implementation, not design
    — but it is in scope of AC-2/AC-7 and recorded here rather than silently folded in.
  - AC-3 has no separate named test in the spec's Failing Tests list; added
    `boundary_specimen_stays_lossless_or_smaller_through_default_web` to cover it directly (it
    passes both before and after the fix — the specimen's native size is well under `web`'s default
    downscale bound, so it is a regression guard, not a red-to-green test, same status as AC-4's
    mirror guard).
- **Follow-up work identified:**
  - A concurrent `cargo test` invocation with a different `--features`/`--no-default-features` set
    against a *shared* `CARGO_TARGET_DIR` can corrupt the resulting binary (observed directly: a
    lean-leg run picked up AVIF-enabled artifacts from a concurrently-running default-feature
    build, and every AVIF-must-be-absent test failed as a result). Worth a memory/lesson entry —
    the existing "fresh CARGO_TARGET_DIR" guidance is about *incremental* staleness; this is a
    *concurrency* hazard on top of it, distinct enough to bite someone who dutifully uses a fresh
    target dir per leg but runs the legs in parallel against the same one.
  - `docs/backlog.md`'s dither-of-photo entropy ceiling item is still open and still gates how much
    headroom AC-5's contested band actually has in the wild (no committed fixture measures inside
    it, but nothing rules a real one out).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — AC-5's "or" framing (retune `DOC_ENTROPY_MAX`, or document + test) reads as two independent
   options, but the spec's own out-of-scope note ("the numbers stay") rules out the first, and the
   literal test case it names (bimodality ≈0.30) cannot be satisfied by documentation alone. Working
   out that the real requirement was a third thing — a structural cascade change reusing existing,
   unretouched constants — took the most reasoning of any single AC.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The `has_alpha` post-pipeline-vs-source bug (see Deviations) wasn't named anywhere, but it's a
   direct structural consequence of the same class of defect this spec exists to fix (a feature
   read from the wrong buffer) and it was hiding behind AC-7's fixture. A pointer in AC-2 or the
   traps section — "has_alpha has the same placement question as everything else Analysis reads" —
   would have saved the detour of tracing three test regressions back to one root cause.

3. **If you did this task again, what would you do differently?**
   — Run the three feature-leg full-matrix verifies in fully isolated `CARGO_TARGET_DIR`s from the
   start, sequentially or with distinct dirs, rather than launching them concurrently against one
   shared dir and discovering the corruption after the fact. The spec's "fresh CARGO_TARGET_DIR"
   instruction was about incremental staleness; I read it too narrowly and only isolated after a
   run failed for the wrong reason.

---

## Reflection (Ship)

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
