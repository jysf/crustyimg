---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-023
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent
    - operator

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-17
supersedes: null
superseded_by: null

affected_scope:
  - src/quality/**
  - src/cli/**

tags:
  - max-size
  - byte-budget
  - resize
  - quality
  - search
---

# DEC-023: `--max-size` dimension-reduction fallback (quality first, then downscale)

## Decision

When `--max-size <BUDGET>` cannot be met by lowering encoder quality alone,
crustyimg **progressively downscales the output dimensions** until the encoded
file fits (or a floor is reached). This completes SPEC-017's deliberately
quality-only v1 (which warned "dimension reduction not yet supported"):

1. **Quality first, dimensions second (for lossy formats).** For a lossy-quality
   format (JPEG; AVIF/WebP with their features) the existing quality search runs
   **at full dimensions** first. Only if even the **minimum** quality exceeds the
   budget does the dimension fallback engage: hold quality at the search's
   best-effort minimum and find the **largest scale** whose encode fits.
2. **Dimensions only (for lossless formats).** PNG / lossless WebP / GIF / BMP /
   TIFF / ICO have no quality knob, so `--max-size` was previously a no-op + a
   warning. Now it drives a **pure scale search**: the largest scale whose lossless
   encode fits the budget. This is the headline new capability — `--max-size` finally
   works for PNG and for very small budgets.
3. **Reuse the shipped search core for scale.** The scale search reuses
   `search_under_size` (the generic `search_threshold`, SPEC-017) over an integer
   **scale-percent** axis `1..=100` exactly as it searches quality `1..=100`:
   `probe(pct)` = encode the image resized to `pct%` and return its byte length;
   accept `≤ budget`; prefer the **highest** fitting percent. Capped iterations, no
   per-candidate disk (DEC-002). No new search algorithm.
4. **Resize via `image`'s Lanczos3, inside `src/quality`.** The fallback resizes
   candidates with `DynamicImage::resize_exact(.., FilterType::Lanczos3)` so the
   budget logic stays in the self-contained `quality` module (which may depend only
   on `::image`). It does NOT reuse the `Resize` *operation* (fast_image_resize,
   `src/operation`) — that would invert the layering. The two Lanczos3
   implementations differ negligibly, and correctness is preserved because the
   sink writes **exactly** the resized pixels the search chose (the cross-sync
   contract extends: same pixels + same quality in probe and write).
5. **Output pixels become budget-dependent — threaded back to the sink.** This is
   the model change: the chosen output may have **smaller dimensions** than the
   pipeline produced. The byte-budget resolver now returns an optional **replacement
   image** alongside the quality; the CLI writes that (rebuilt via
   `Image::from_parts`) instead of the pipeline's output. Perceptual (`--target`/
   `--ssim`) and fixed-`-q` paths are unchanged (no replacement image).
6. **Floor + best-effort.** The scale search will not go below a small floor (each
   dimension stays ≥ 1px; a `MIN_SCALE_PERCENT` keeps it sane). If even the floor
   does not fit, write the best-effort smallest and warn (mirrors the quality
   search's unmet-budget behavior). A budget already met at full size never resizes.

## Context

SPEC-017 shipped `--max-size` as a quality-only binary search and explicitly
deferred dimension reduction (the warning text and the constraint both say so).
That left two gaps: (a) lossless outputs can't honor a byte budget at all, and (b)
a small budget on a large image returns a too-big best-effort. Both are fixed by
the one lever SPEC-017 didn't touch — dimensions. The search machinery already
generalizes (it's a monotone threshold search), so the work is wiring, a resize,
and threading the resized pixels to the sink, not a new algorithm. STAGE-008's
formats are done; this is the last quality-core item before the stage ships.

## Alternatives Considered

- **Nested 2D search (re-optimize quality at each scale).**
  - Why rejected for v1: encode cost is quality-iters × scale-iters; and the
    product's "best fidelity under N bytes" gain over "min quality at the largest
    fitting scale" is marginal for the fit-it-in-a-budget use case. v1 holds quality
    at the floor and maximizes scale; a 2D refinement (bump quality back up at the
    chosen scale) is a documented fast-follow.
- **Resize via the `Resize` operation (fast_image_resize) in the CLI loop.**
  - Why rejected: it would push the budget search into the CLI (less testable) or
    invert the `quality`→`operation` layering. Keeping the search in `quality` with
    `image`'s resize is cohesive and unit-testable; the resize-impl difference is
    immaterial because the sink writes the exact chosen pixels.
- **Crop instead of scale.**
  - Why rejected: cropping discards content; scaling preserves the whole image, which
    is what a "make this fit in N bytes" request means. (A future `--max-size --crop`
    could be added if wanted.)
- **Leave it deferred.**
  - Why rejected: lossless `--max-size` being a no-op is a real gap, and this is the
    last item gating STAGE-008's ship.

## Consequences

- **Positive:** `--max-size` works for **every** output format (lossless included)
  and for arbitrarily small budgets; the result is the largest/highest-fidelity
  image that fits. Reuses the shipped search core; no new dependency.
- **Negative / model change:** the output dimensions can differ from what the
  pipeline produced — surprising if unnoticed, so a downscale emits a `--quiet`-able
  warning ("scaled to WxH to fit the N budget"). The byte-budget resolver's return
  type grows an optional replacement image (a contained refactor of two call sites).
  More encodes in the worst case (quality search + scale search), still capped.
- **Neutral:** v1 holds quality at the floor during the scale search (the 2D
  refinement is deferred). AVIF (no decoder) is unaffected — `--max-size` is
  encode-only and already worked; it just gains the scale fallback too.

## Validation

Right if: `convert big.png --format png --max-size 8KB` produces a PNG ≤ ~8KB by
downscaling (was a no-op + warning before); `shrink huge.jpg --max-size 3KB` fits by
lowering quality AND, if needed, scaling; a budget already met at full size never
resizes; a downscale warns (unless `--quiet`); the written file's bytes equal the
search's chosen-candidate bytes (cross-sync holds); the search stays capped/sub-second
on test images. Revisit if: users want the 2D quality-at-scale refinement, a crop
mode, or a min-dimension floor knob.

## References
- Related specs: SPEC-021 (this), SPEC-017 (the quality-only `--max-size` it
  completes), SPEC-016 (the search core), SPEC-010 (the `Resize` op — the pipeline
  resize, intentionally NOT reused here).
- Related decisions: DEC-019 (the search policy/core reused for the scale axis),
  DEC-016 (`-q` semantics), DEC-002 (decode-once, in-memory candidates), DEC-020/DEC-022
  (the cross-sync contract this extends to resized candidates).
- Related constraints: `every-public-fn-tested`, `no-unwrap-on-recoverable-paths`,
  `single-image-library` (resize via `image`, not a new lib).
