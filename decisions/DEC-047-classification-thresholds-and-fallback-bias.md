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
  that switches on `opt_bucket`)
- Related decisions: DEC-002 (decode-once), DEC-034 (decode limits), DEC-048 (format engine that
  consumes the verdict), DEC-019 (the SSIMULACRA2 target that bounds the fallback downside)
- External docs: `docs/research/proj-002-design-classification.md` (cascade + cited USPTO /
  Cloudinary prior art)
- Discussions: PROJ-002 framing session 2026-07-05
