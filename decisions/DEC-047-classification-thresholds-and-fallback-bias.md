---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-047                        # stable, never reused
  type: decision                     # decision | analysis | recommendation | observation
  confidence: 0.7                    # 0.0 - 1.0, honest assessment
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002                       # the project during which this was decided
repo:
  id: crustyimg

created_at: 2026-07-05
supersedes: null
superseded_by: null

# Path globs this decision governs.
affected_scope:
  - src/analysis/mod.rs
  - tests/fixtures/classify/**
  - scripts/seed-classify-specimens.py

tags:
  - analysis
  - classification
  - thresholds
  - format-decision
  - safe-fallback
---

# DEC-047: image-classification thresholds, three-bucket collapse, and safe-fallback-to-photograph bias

## Decision

`Analysis` classifies an image with a **deterministic, no-ML rule cascade** (Icon → Graphic/logo →
Document → UI-screenshot → Photograph → fallback), evaluated cheapest/strongest-first on the
SPEC-046 features plus the container priors (`source_format`, `has_exif`). The five fine
`ImageClass` labels collapse — via a **fixed, exhaustive** map — to three `OptBucket`s the format
engine switches on: `Photograph→Lossy`, `GraphicLogo|Icon|Document→LosslessFlat`,
`UiScreenshot→MixedSafe`. The threshold constants live in **one named block** in `src/analysis/mod.rs`
(starting anchors from the design brief). Under ambiguity (`confidence < ~0.5`) the class defaults
to **`Photograph`/`Lossy`** — the deliberate safe-fallback bias. `has_exif` is the decisive
Photograph prior; contradictions resolve by **cascade precedence, not averaging**. Classification
**never errors and never blocks** (it is advisory, always yields a class + confidence).

## Amendment — 2026-07-25 (SPEC-105): strong-entropy → Photograph, ahead of the graphic gates

**New rule.** A sixth threshold, `PHOTO_ENTROPY_STRONG = 4.0` (bits of luma entropy), adds a
decisive rule to the cascade: an image with luma entropy ≥ `PHOTO_ENTROPY_STRONG` **that reaches
rule 3.5** is a `Photograph`. It fires *after* the EXIF (rule 2) and Document (rule 3) rules and
*before* the graphic gates (rule 4), so a high-entropy image — grayscale or colour, EXIF or not —
can no longer be misfiled as a `GraphicLogo`.

> **Correction — 2026-07-26 (SPEC-109).** This paragraph originally read "**any** image with luma
> entropy ≥ `PHOTO_ENTROPY_STRONG` is a `Photograph`". That is false as written, and the rules
> ahead of 3.5 are why. **Rule 1 (`Icon`) claims every no-EXIF image with
> `max(w, h) <= ICON_MAX_EDGE` (128) and aspect ≤ 2.0 before entropy is ever consulted** — so a
> 128×128 EXIF-stripped photo thumbnail classifies `Icon` → `LosslessFlat` at any entropy.
> Measured on a 128×128 centre crop of `grayscale_photo_leica.png`: **entropy 5.15 → `icon` →
> lossless** (the PR-113 re-derivation measured 6.02 on a 128×128 *downscale* of the same frame —
> same verdict). This is not a corner case: 128 px squares are what gallery and thumbnail
> pipelines emit. Rule 2 (EXIF) reaches the same class by another route and rule 3 (`Document`)
> can claim a bimodal near-gray image below `DOC_ENTROPY_MAX` 4.5, which overlaps this rule's
> range in the band `[4.0, 4.5)`. The reach of rule 3.5 is "whatever rules 1–3 did not already
> claim", and the `Icon` ordering itself is SPEC-108's to decide.

**Why.** A real Leica B&W portrait (a RAW embedded preview, so EXIF-stripped) shipped as an 823 KB
lossless WebP instead of a ~62 KB AVIF — 13× oversized (probe:
`docs/research/proj-008-grayscale-photo-misclassification-probe.md`). Two shared-classifier gates
mis-fired: a grayscale photo has ≤256 RGB colours (r=g=b), tripping the palette gate (rule 4
clause 1); and at megapixel resolution the flat detector — anchored on 64×64 synthetics — reads
*every* photo as ~flat, tripping the flat-graphic gate (rule 4 clause 2) for EXIF-stripped **colour**
photos too. The EXIF prior (rule 2) had been *masking* both; RAW preview extraction drops EXIF and
exposed them. Entropy is the clean discriminator: real photographs are high-entropy, genuine
graphics are low-entropy.

**Calibration (real photos vs real graphics; every value measured via `optimize --explain=json`).**

| Side | Set (all EXIF-stripped) | Entropy |
|---|---|---|
| Photos (real) | 48 grayscale + colour crops from the maintainer's library; committed fixtures: Leica 6.07, Canon 6.83, Fuji colour 6.37 | **floor ≈ 4.58**, median ≈ 6.8, up to 7.6 |
| Graphics — hard-edged (must stay lossless) | solid 0.00 · logo 0.96 · text 0.39 · document 0.49 · realistic UI dashboard 1.56 · code-editor screenshot 0.32 | **≤ ~1.6** |
| Graphics — dithers | ordered 2-colour 1.00–1.50 · Floyd–Steinberg 8-colour (committed `dithered_graphic.png`) **3.03** · 16-colour 3.43 | ≤ 3.43 |

**Committed boundary specimens (added 2026-07-26, SPEC-109).** The two values that define the gap
above were cited here but were not in the repo, so nothing pinned them and the calibration guard
held for any threshold in (3.03, 6.07]. Both edges are now committed, seeded from documented
recipes and measured outside the crate (`tests/fixtures/classify/RECIPES.md`,
`scripts/seed-classify-specimens.py`):

| specimen | entropy | what it is |
|---|---|---|
| `photo_entropy_floor.png` | **4.5176** | the photo floor — `grayscale_photo_leica.png` under a flat-light curve (tonal range compressed to a third). `flat_ratio` **1.00**, so rule 3.5 is the only thing keeping it off the lossless path. |
| `dither_32color.png` | **3.6414** | the graphic ceiling — `color_photo_fuji.png` at 32 grey levels with Floyd–Steinberg. Entropy bounded by `log2(32)` = 5 bits by construction. |

The realised window is therefore **(3.6414, 4.5176]**, 0.88 bits — near the (3.43, 4.58] this
record documents, and narrow enough that the guard now fails at 5.5 *and* at 3.2. The 16-colour
3.43 figure in the table above could not be reproduced from the photographs this repo holds
(16 levels of a 6.07–6.83 bit source lands at 2.46–2.88); the reasoning is in the RECIPES note.

`PHOTO_ENTROPY_STRONG = 4.0` sits in the gap between the highest realistic hard-edged graphic
(≈1.6, or ≤3.64 counting dithers-of-photos) and the lowest real photo (≈4.52): ~2.4 margin above
the genuine graphics it must protect, ~0.5 below the photo floor.

**Known crossings (accepted, both lossy-safe).** Two high-entropy inputs clear the floor and route
lossy: a **smooth full-frame gradient** (~7.5 — no hard edges, so lossy at high quality doesn't
smear anything) and a **heavy 32-colour error-diffusion dither of a photo** (~5.1 — a dithered
*photograph*, not a logo). Both are the safe error direction DEC-047 already commits to: a photo
forced lossless is merely a bigger file.

> **Correction — 2026-07-26 (SPEC-109).** This paragraph originally closed "only a *hard-edged
> graphic* forced lossy is harmful, and **none of those reach 4.0**". The second half is false,
> and the committed fixture refutes it: `dithered_graphic.png` measures 3.03 at native size but
> **7.08 at `--max 256`**, where it classifies `photograph` and is offered a lossy candidate
> (SSIMULACRA2 81.8). Entropy is **not scale-invariant** — the classifier runs on the *output* of
> the resize pipeline, so `--max` alone can carry a hard-edged graphic across the floor. The
> claim was never about a property of graphics; it was about a property of graphics *at the size
> they happened to be measured*. This is the STAGE-034 regression, and fixing the placement is
> SPEC-108's job. Until then the safety argument for this threshold holds only for images
> classified at their native size.

**Scope guardrail.** This rule *masks* the scale-broken flat/edge detector for photos; it does **not
fix** it. Scale-normalizing that detector (so "flat" means the same at 64 px and 20 MP) and carrying
EXIF through the RAW-preview decode remain separate follow-ups. The two synthetic tests that used
full-range gradients/noise as stand-ins for a "screenshot" / "ambiguous" image were replaced with
realistic **low-entropy** constructions (iso-luma tint for >256 colours without spreading the luma
histogram), since a high-entropy gradient is exactly what this rule now — correctly — reads as
photographic.

**Confidence.** The rule returns `Photograph` at confidence 0.8 (a strong, calibrated signal — above
the no-EXIF photo heuristic's 0.7, below the decisive EXIF prior's 0.9).

## Context

STAGE-011 (SPEC-047) needs a photo-vs-graphic verdict to bias STAGE-012's format engine toward the
right codec family (photographic → lossy JPEG/AVIF/lossy-WebP; graphic/flat → lossless
PNG/lossless-WebP). The load-bearing prior-art insight (`docs/research/proj-002-design-classification.md`):
**the codec decision and the classification decision are the same signal** — Cloudinary
`q_auto`/`f_auto` detects "photographic vs non-photographic" precisely to pick JPEG-vs-PNG. The
design questions this DEC settles:

1. What are the starting threshold anchors, and where do they live?
2. How many labels does the engine actually switch on?
3. Which way do we err when the signal is ambiguous?

Constraints in play: `untrusted-input-hardening` (advisory, no panic, no block);
`no-agpl-default-deps` / `no-new-top-level-deps-without-decision` (hand-computed, no `imageproc`,
no ML crate); determinism (integer / fixed-order f32). crustyimg's lossy path is already
SSIMULACRA2-target-bounded (`src/quality/`), which bounds the downside of the fallback bias.

## Alternatives Considered

- **Option A: an ML/statistical classifier (e.g. a small trained model or a crate).**
  - What it is: learn photo-vs-graphic from a labeled corpus at runtime or via bundled weights.
  - Why rejected: violates pure-Rust / zero-deps (imgproxy's `ml` path is the anti-pattern), is
    non-deterministic to reproduce across platforms, and is far more than the format decision
    needs — four features carry ~all of the decision.

- **Option B: expose all five (or six) fine labels to the engine and let it branch on each.**
  - What it is: the format engine switches on `Photograph`/`GraphicLogo`/`Icon`/`Document`/
    `UiScreenshot` independently.
  - Why rejected: six independent detectors is more surface to tune and drift; the engine only
    needs three dispositions (lossy / lossless-flat / mixed-safe). Keep the fine label for
    `explain` cosmetics only.

- **Option C: resolve ambiguity by averaging/blending contradictory signals, or by defaulting to
  the *smaller-file* class (graphic/lossless).**
  - What it is: when rules conflict, blend confidences, or bias toward lossless to save bytes.
  - Why rejected: averaging hides which rule fired (hurts `explain`); biasing toward lossless has
    the *bad* downside — a graphic forced lossy smears text/edges (visible artifacts), whereas a
    photo forced lossless is merely a slightly larger file. The safe error direction is toward
    Photograph/Lossy, bounded by the perceptual target.

- **Option D (chosen): fixed rule cascade + three-bucket collapse + fallback-to-Photograph, all
  thresholds in one named consts block.**
  - What it is: the design-brief cascade with precedence resolution, an exhaustive six→three map,
    and the safe bias; anchors named in one place like `MAX_SEARCH_ITERS`/`AVIF_SPEED`.
  - Why selected: deterministic, zero-dependency, cheap (O(1) after the SPEC-046 feature pass),
    honest under ambiguity, and it gives the engine exactly the three switches it needs while
    keeping the tuning surface in one auditable block.

## Consequences

- **Positive:** the format engine reads one `OptBucket` (+ `has_alpha`) and never re-scans pixels;
  the tuning surface is one consts block; the fallback bias is safe (bounded by the SSIMULACRA2
  target); `explain` can surface a one-word label + a hedged confidence.
- **Negative:** the anchors are *starting* values, not corpus-tuned — some gray-zone images will
  mis-label (photo-of-a-document, gradient-heavy UI, dithered GIF). The exhaustive collapse means
  adding a future label is a compile error until its bucket is chosen (intended friction).
- **Neutral:** `UiScreenshot`/`MixedSafe` is kept distinct from `LosslessFlat` **only because** the
  engine's row-F "try both, let bytes decide" differs; if a future engine treats them identically,
  the bucket can be merged.

## Validation

- **Right if:** classification routes a labeled fixture corpus (photo→Lossy,
  logo/graphic/icon/document→LosslessFlat, ui-screenshot→MixedSafe) correctly, and the resulting
  format choices beat a format-blind default without introducing visible artifacts on graphics.
- **Revisit when:** a *real* labeled corpus is assembled (tune the anchors, keep the structure);
  or `explain`/user feedback shows a systematic mis-label (e.g. gradient UI read as photo) worth a
  new rule; or a permissive quantizer lands (PROJ-007) and changes the lossless-flat economics.

## References

- Related specs: SPEC-047 (classification), SPEC-046 (the feature layer), SPEC-048 (the engine
  that switches on `opt_bucket`), SPEC-105 (the strong-entropy amendment), SPEC-109 (the committed
  boundary specimens and the two corrections above)
- Fixture provenance: `tests/fixtures/classify/RECIPES.md`, `scripts/seed-classify-specimens.py`
- Related decisions: DEC-002 (decode-once), DEC-034 (decode limits), DEC-048 (format engine that
  consumes the verdict), DEC-019 (the SSIMULACRA2 target that bounds the fallback downside)
- External docs: `docs/research/proj-002-design-classification.md` (cascade + cited USPTO /
  Cloudinary prior art)
- Discussions: PROJ-002 framing session 2026-07-05
