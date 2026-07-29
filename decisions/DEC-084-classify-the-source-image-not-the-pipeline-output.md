---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-084                        # stable, never reused
  type: decision                     # decision | analysis | recommendation | observation
  confidence: 0.85                   # 0.0 - 1.0, honest assessment
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-07-26
supersedes: null
superseded_by: null

# Path globs this decision governs.
affected_scope:
  - src/cli/optimize.rs
  - src/analysis/mod.rs
  - src/analysis/decide.rs

tags:
  - analysis
  - classification
  - format-decision
  - thresholds
  - scale-invariance
---

# DEC-084: classify the source image, not the pipeline output — and resolve the four cascade contradictions that placement change stands in

## Decision

`Analysis::compute` now runs on the **decoded source image**, before `pipeline.run` resizes it,
and the resulting verdict (not a re-computed one) is threaded to `format_shortlist`. `--max` can
no longer change an image's content class. Alongside the placement move, four contradictions the
classification cascade already carried are resolved in the same change (SPEC-108 AC-5 through
AC-8): the `[4.0, 4.5)` entropy contradiction band between `PHOTO_ENTROPY_STRONG` and
`DOC_ENTROPY_MAX`, rule 6's dead code, a lean-leg PNG blow-up for a promoted image with alpha, and
`--profile docs`'s undefined behaviour on a promoted image.

## Context

`web` was promoting dithered and halftoned graphics to lossy AVIF — up to 18.5× larger than the
input, SSIMULACRA2 69.2 — because classification ran on the *output* of the resize pipeline
(`pipeline.run` at `src/cli/optimize.rs:989` → `Analysis::compute` → `format_shortlist`), so
`--max` decided an image's content class. Measured on the committed
`tests/fixtures/classify/dithered_graphic.png`: `graphic-logo` (entropy 3.03) at native size and
at `--max 512`, but `photograph` (entropy 7.08) at `--max 256` — the identical pixels, reclassified
purely by resize.

STAGE-034's design cycle (SPEC-108) carried two candidate fixes and picked on measured complexity:

- **(a) Placement** — classify the original, before the resize pipeline. Chosen.
- **(b) Narrow gating** — keep classification where it is; delete rule 3.5's unconditional early
  return; gate rule 4's two clauses on `entropy < PHOTO_ENTROPY_STRONG` instead.

**(b) was refuted by measurement, not preference.** Traced against the committed fixture at
`--max 256` with the real constants (`PALETTE_COLORS=256`, `FLAT_GRAPHIC_RATIO=0.60`,
`PHOTO_ENTROPY_STRONG=4.0`): `unique_colors=217 ≤ 256`, so `few_colors` is TRUE and `many_colors`
is FALSE. Rule 4a would be gated off by the `entropy < 4.0` clause (7.08 fails it); rule 4b's
`flat_ratio 0.27 < 0.60` already fails; rules 5 and 6 both require `many_colors`, so they are
unreachable on this input regardless; the cascade falls through to rule 7 — whose safe-fallback
bias is `Photograph` (DEC-047, deliberate: a photo forced lossless is merely a bigger file). Option
(b) changes *which line* returns `Photograph`, not the answer. This generalises: rule 7's fallback
bias is `Photograph`, so any fix that only stops the graphic gates from firing lands on
`Photograph` anyway — structurally incapable of fixing an input whose correct answer is "graphic",
which is precisely this defect's blast radius.

Placement is directly supported by the same table: at native size and at `--max 512` the fixture
measures 9 unique colours and entropy 3.03 and classifies `graphic-logo` correctly. Classifying the
input is classifying the thing the user actually gave us; the resized buffer is a derived artifact
whose entropy is a resampling artifact, not content.

**Consequence carried into the build.** Under placement, rule 3.5 keeps its unconditional early
return (it must — see Alternatives), so rules 5 and 6 stay unreachable, and rule 6's dead code has
to be handled explicitly rather than falling out of the placement fix for free (AC-6).

## Alternatives Considered

- **Option A: narrow gating (rejected).** Delete rule 3.5's unconditional return; gate rule 4 on
  `entropy < PHOTO_ENTROPY_STRONG` instead. Refuted by measurement (Context, above) — it cannot fix
  an input whose correct answer is "graphic", because the classifier's safe fallback is
  `Photograph`, not "whichever graphic gate would have fired at native size."

- **Option B (AC-5): lower `DOC_ENTROPY_MAX` below `PHOTO_ENTROPY_STRONG` to close the `[4.0, 4.5)`
  overlap numerically.** Rejected: both STAGE-034 specs forbid retuning threshold *values* (the
  numbers stay; only the input changes), and it would not even fix the concrete failure case AC-5
  names — a halftone scan at entropy 4.2 with bimodality ≈ 0.30 fails the Document rule's
  `bimodality >= 0.55` gate regardless of where `DOC_ENTROPY_MAX` sits, since that gate never
  reads entropy in isolation.

- **Option C (AC-5, chosen): split rule 3.5 into an unconditional zone (`entropy >=
  DOC_ENTROPY_MAX`) and a contested zone (`[PHOTO_ENTROPY_STRONG, DOC_ENTROPY_MAX)`) where the
  existing graphic-gate conditions (`few_colors`, or `flat_ratio`/`edge_ratio`) — reused, not
  retuned — must also fail before conceding `Photograph`.** Every committed real-photo fixture
  measures at/above `DOC_ENTROPY_MAX` (floor 4.5176), so the unconditional zone stays exactly as
  load-bearing as before for all of them; no committed fixture (photo or graphic) falls inside the
  contested band, so nothing measured regresses. This is deliberately narrower than option (a)
  above: it does not touch rule 3.5's behaviour for `entropy >= DOC_ENTROPY_MAX` at all, only adds
  a check inside the specific ambiguous sub-range AC-5 identifies.

- **AC-6: keep rule 6 alive by loosening its guard, vs. delete it (chosen).** Rule 6
  (`many_colors && entropy >= PHOTO_ENTROPY(5.0) && flat_ratio < PHOTO_FLAT_MAX(0.25)`) requires a
  strictly higher entropy bar (5.0) than rule 3.5 now claims unconditionally from (4.0), so any
  input that could reach rule 6 was already claimed by 3.5a/3.5b. This was true before the
  placement fix too (DEC-047 already noted rules 5/6 are unreachable under any fix that keeps rule
  3.5's early return, which every considered option does). Loosening rule 6's own thresholds to
  make it reachable would be retuning; deleting it removes dead code with no coverage loss (nothing
  exercised it). `PHOTO_ENTROPY` and `PHOTO_FLAT_MAX` are deleted with it — leaving them would be
  inert constants, which the blocking `clippy-fmt-clean` constraint forbids in spirit even though
  `-D warnings` cannot see it (both stay syntactically referenced only by the deleted rule, which
  is deleted in the same change).

- **AC-7: give the lean leg a real lossy-alpha codec, vs. offer a smaller lossless fallback
  (chosen).** No codec in this codebase encodes lossy content with alpha without a Cargo feature
  (JPEG structurally never carries alpha; lossy WebP and AVIF are both feature-gated). Adding one
  by default was out of scope (a much larger, unrelated decision). Lossless WebP has no feature
  gate at all (SPEC-019/DEC-021) and compresses photographic content well below PNG, so
  `format_shortlist`'s `OptBucket::Lossy` + `has_alpha` arm now offers it ahead of PNG. This does
  not eliminate the possibility of a larger-than-source output on the lean leg (SPEC-084 makes no
  never-bigger promise on the metadata-forced branch — that promise was never made and is not this
  spec's to add), but it removes the specific defect: previously PNG was the *only* candidate, so
  it always won by default; now the smaller of two lossless options ships instead.

- **AC-8: leave `--profile docs` a no-op on a promoted image, vs. have it downgrade to lossless
  (chosen).** `docs`'s own contract ("crisp-text bias: widen the lossless/graphic preference")
  already widens the ambiguous `MixedSafe` bucket to `LosslessFlat`; extending the same widening to
  a confidently-classified `Photograph` gives users an explicit escape hatch for corpora where
  they'd rather over-preserve than risk a lossy artifact on content adjacent to text/line-art — the
  same safe-error-direction logic DEC-047 already applies to the fallback bias, offered as an
  opt-in rather than forced on every profile.

## Consequences

- **Positive:** `--max` (and any other resize-driving flag) can no longer change an image's
  content class — the headline defect is closed, not routed around. The `features` block in
  `--json`/`--explain` output is now identical across `--max` values for the same input (a
  structural guarantee, not just a passing behavioural test). Rule 6 and its two constants are
  gone; the cascade has no unreachable rule left. The lean leg's alpha-photo fallback is less bad
  (smaller lossless candidate offered) without adding a feature dependency. `--profile docs` has a
  decided, tested behaviour on every `OptBucket`, not three of four.
- **Negative:** classifying pre-pipeline means EXIF is now present where it previously was not —
  rule 2 (the decisive camera prior) fires for every EXIF-bearing input, which it effectively could
  not before (analysis always ran on the metadata-stripped post-pipeline buffer). This is the
  *correct* input, but it changes behaviour for EXIF-bearing sources and is the most likely place
  this spec surprises a caller relying on the old (wrong) input.

  > **Measured 2026-07-28 (SPEC-108 verify, re-measured on committed fixtures).** This paragraph
  > was written as a predicted risk; it now has fixtures and numbers. Across 12 fixtures (4 EXIF
  > variants × 3 content types) exactly **one** genuine flip reproduces: a document-shaped scan
  > carrying a **real, non-identity orientation tag**. Confirmed to be a property of EXIF rather
  > than of one generator — it reproduces on an independently constructed document source (different
  > geometry, 218 colours vs 117), and structurally it must, since rule 2 returns before rule 3.
  >
  > | source | this branch | pre-SPEC-108 |
  > |---|---|---|
  > | `scan_jpeg(1200,1600)`+o6, 78,670 B *(the committed fixture)* | AVIF 6,261 B, **88.3** | lossless WebP 6,748 B, **100.0** |
  > | `scan_jpeg(450,600)`+o6, 20,829 B | AVIF 4,811 B, 87.3 | WebP 9,010 B, 100.0 |
  > | independent LCG scan+o6, 155,022 B | AVIF 87,033 B, 86.7 | WebP 230,422 B, 100.0, **`larger_than_source`** |
  >
  > **The trade is quality for size, not quality for nothing.** An earlier draft of this note said
  > "near-identical size … no benefit … a strict regression", from a single ad-hoc fixture that was
  > never committed and whose 92.2 figure **cannot be re-derived from this repo**. Re-measured on
  > committed fixtures the branch is **7.2% / 46.6% / 62.2% smaller**, and on the largest source the
  > *old* lossless path shipped 49% **larger than the input**. The quality loss is real (86.7–88.3
  > against a pixel-perfect 100.0) and is the reason this is recorded as a downside — but anyone
  > reading this to decide whether to reorder the cascade needs both halves.
  >
  > **The reason it stayed invisible is worth more than the number.** The suite's `jpeg_with_exif`
  > carries a **zero-entry IFD**, and `orientation_from_exif_segment` returns `None` for it, so
  > `AutoOrient` no-ops and metadata survives the pipeline either way. Orientation 1 is a no-op for
  > the same reason. **That fixture cannot express the case its name implies**, which is why no
  > existing test caught this. Pinned by `scan_with_real_orientation_tag_classifies_photograph`
  > (`tests/cli.rs`), which carries a control arm asserting the same pixels classify `document`
  > without EXIF — without it the assertion would pass for reasons unrelated to EXIF.

- **Negative (measured 2026-07-28, SPEC-108 verify): a ~40% slowdown on large graphic/lossless
  inputs.** `Analysis::compute` is an unconditional O(pixels) scan; it now runs on the source
  rather than the bounded post-resize buffer. On a 24 MP (6000×4000) coarse checker
  (`graphic-logo`, cheap encode): **484–507 ms vs 344–352 ms** on the pre-change build, a
  repeatable **~140–150 ms** gap isolated to the classify+resize segment (`decode_ms` 8–14 ms and
  `encode_ms` ~6 ms are near-identical on both). Independently reproduced at 501–546 ms vs 350–354
  ms (+43%). That path also became **size-sensitive**: on this branch 2 MP → 24 MP costs +70%
  (311 → 523–534 ms) against +21% before (317 → 381–406 ms).

  The photograph/AVIF path is **near-unaffected**, not unaffected: 3204/3259/3255 ms vs
  3174/3174/3173 ms is a consistent **+50–85 ms (1.5–2.5%)** — small, but outside the ±5 ms
  measurement spread, so it is a real delta rather than noise. "Finishes about as fast" holds
  (2 MP → 24 MP is +3.5% on the branch); "unaffected" would not.
  `web`'s help text has been corrected accordingly; the mitigation (bound analysis cost by sampling
  the source under a fixed rule, which preserves the `--max`-independence this decision buys) is
  filed in `docs/backlog.md` rather than attempted here.

- **Negative:** the lean-leg alpha-photo fallback
  is improved, not fixed: a lossless WebP/PNG re-encode of a lossy-family alpha source can still
  exceed the source's size when no lossy-alpha codec is built — this is a structural limitation of
  the lean feature set, not a bug this change closes.
- **Neutral:** the true ceiling of "dither of a photo" entropy remains unmeasured across a corpus
  (DEC-047's open item, filed in `docs/backlog.md`) — placement does not depend on that ceiling
  (native-size dithers are classified on their own real entropy either way), but a native-size
  dither whose entropy happens to exceed 4.0 on its own merits still classifies `photograph`, same
  as before this change.

## Validation

- **Right if:** `dithered_graphic.png` classifies `graphic-logo` and stays lossless at every
  measured `--max`; the `features` block is byte-identical across `--max` for the same input; the
  SPEC-109 boundary specimens (`photo_entropy_floor.png`, `dither_32color.png`) and the four real
  photo/graphic fixtures keep their documented classes; the calibration guard
  (`calibration_gap_matches_the_documented_gap`) still fails when `PHOTO_ENTROPY_STRONG` is mutated
  to 5.5.
- **Revisit when:** a real corpus establishes the dither-of-photo entropy ceiling (may narrow or
  widen the `[PHOTO_ENTROPY_STRONG, DOC_ENTROPY_MAX)` band's practical risk); a lossy-alpha codec
  becomes part of the lean feature set (would let AC-7's fallback stop being merely "less bad");
  EXIF-bearing input now reaching rule 2 pre-pipeline surfaces a real-world misclassification worth
  its own fixture.

## References

- Related specs: SPEC-108 (this build), SPEC-105 (introduced rule 3.5), SPEC-109 (boundary
  specimens, calibration guard, the corrections to DEC-047 this decision does not re-litigate)
- Related decisions: DEC-047 (classification thresholds, three-bucket collapse, fallback bias —
  this decision changes classification's *input*, not DEC-047's thresholds), DEC-048 (the format
  engine that consumes `opt_bucket`), DEC-017 (EXIF baking/stripping in the pipeline), DEC-019 (the
  SSIMULACRA2 target bounding the fallback-bias downside), DEC-021 (lossless WebP's unconditional
  availability)
- Fixture provenance: `tests/fixtures/classify/RECIPES.md`, `scripts/seed-classify-specimens.py`
- External docs: `docs/research/pr113-classifier-review-findings.md` §"Re-derivation (2026-07-25)",
  `docs/backlog.md` (dither-of-photo ceiling, filed open)
