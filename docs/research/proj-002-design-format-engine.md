# PROJ-002 design brief — format auto-decision engine (`optimize` = "local f_auto")

> Silent auto-decide of output format + lossy/lossless inside the existing `optimize`,
> composing the shipped SSIMULACRA2 search + `LossyFormat` seam. Design-only, 2026-07-05.
> **See also `proj-002-design-planner.md` — the format engine is the shortlist/per-format
> layer; the planner is the general goal-solver that wraps it. Reconcile the two into one
> "decision engine" spec set (see findings §6 note).**

## Where it lands
Today `optimize` (src/cli/mod.rs:238, :2763) is **format-preserving** — auto-orient, optional
`--max`, auto-tune *quality only* vs a visually-lossless target, write the input's format.
This engine inserts **one new step** between decode and the quality search: **candidate-format
selection**, firing only when the user has NOT pinned a format (`--format`/`-o` absent). Pinned
format bypasses the engine (deterministic escape hatch). Add `--profile <web|docs|preserve>`
(default web; `preserve` = today's behavior, strictly additive) and `--explain`.

## Decision tree (deterministic feature → ordered candidate shortlist)
Compute a cheap single-pass `FeatureVector` once (reuse the `Analysis` layer): `has_alpha`,
`alpha_binary` (α∈{0,255} fraction), `alpha_coverage`, `unique_colors` (capped 4096, early-exit),
`is_animated`, `photo_score`, `src_format`. Never encodes to decide the shortlist.

**`photo_score`** on a ≤512px downsample (size-invariant): `0.5·color_richness + 0.3·gradient_continuity
+ 0.2·(1-flat_region_fraction)`, all normalized. Cutoffs: `≥0.55` photographic (lossy-first);
`≤0.35` graphic (lossless-first); between → ambiguous (include both, let measured bytes decide —
the Cloudinary "trial-encode a shortlist, keep smallest meeting quality" strategy). Mirrors the
field rule "camera→lossy, computer-generated→lossless."

Tree (profile=web), first match fixes disposition + ordered shortlist:
- **A animated** → keep animation: `[WebP(lossless if graphic else lossy), GIF]` (the `-mixed` analog).
- **B `unique_colors≤256 AND flat>0.30`** → **lossless** `[WebP(lossless), PNG(indexed)]` (pngquant/WebP palette gate).
- **C `photo_score≤0.35`** (graphic, >256 colors: gradient UI, AA text) → **lossless** `[WebP(lossless), PNG]`.
- **D `has_alpha AND photo_score≥0.55`** → **lossy RGB** (alpha preserved) `[WebP(lossy), PNG]` (JPEG excluded, no alpha).
- **E `photo_score≥0.55 AND !has_alpha`** (classic photo) → **lossy** `[WebP(lossy), JPEG]`.
- **F ambiguous** → **both** `[WebP(lossy), WebP(lossless), PNG (+JPEG if !alpha)]`.
- **Alpha sub-rule:** binary alpha (≥0.95 on/off) + `unique_colors≤256` → promote row B (lossy alpha fringes).

**Profiles:** `docs` widens graphic bias (prefer lossless/PNG for crisp text, `photo_score<0.60`
→ graphic); `web` appends AVIF *last* to lossy shortlists **only when the feature is built AND in
byte-budget mode** (AVIF has no decoder → can't be perceptually scored, DEC-020); `preserve` =
engine off.

## Composition with the existing search
Thin orchestrator around `src/quality/mod.rs` — **no new search math.** Per candidate produce
`Candidate { fmt, disposition, bytes, score, met_target, quality }`:
- lossy + perceptual mode → `auto_quality(ref, fmt, cfg)` then encode-measure bytes.
- lossy + size mode → `fit_under_size(ref, fmt, budget)` (already does quality-search→dimension fallback).
- lossless → single encode, measure; perceptual: `score=100, met_target=true` (lossless always meets any perceptual target); size: `met=bytes≤budget`.
Reuses the sink's exact encode paths (DEC-016 byte-for-byte contract), so a winner's measured
bytes == the shipped file.

**Winner rule (perceptual/default):** among `met_target` candidates, smallest bytes that also
**beats source bytes** (never regress — else keep source untouched). Tie-breaks (deterministic):
smaller bytes → earlier in shortlist order → source format. **Size mode:** smallest fitting bytes;
if none fit even after downscale, globally smallest best-effort (`met=false` surfaced).

## Search-cost control
Shortlist ≤3 (the big lever) · quality axis = existing binary search (≤`MAX_SEARCH_ITERS=8`) ·
dimension axis lazy (only if quality alone can't fit, quality pinned to floor) · cheap features
gate expensive encodes · fast paths: tiny images (≤64²) skip shortlist; row B tries lossless-WebP
first and stops if it beats source; AVIF only as last budget-mode resort; row F early-accept if the
preferred candidate is already `<40%` of source. Worst case ≈25 in-memory encodes; common case
1 search + 1 encode.

## Explain trace
Off by default (silent decide); `--explain` → stderr (keep stdout pipe-clean). Per run: input
facts, features, profile/mode/target, each evaluated candidate (fmt/quality/score/bytes/met), the
winner + one-line reason, source→output bytes + savings%. `--explain=json` emits the full vector +
candidate array for regression fixtures. Doubles as a manifest `optimization` field.

## Edge cases → exit
Already-optimal (no candidate beats source) → write nothing/passthrough, exit 0 · tiny image →
byte-compare only (SSIMULACRA2 unreliable), exit 0 · perceptual target unreachable → best-effort +
`met=false`, exit 0 · budget unfittable → smallest best-effort, exit 0 · shortlist proposes only
buildable candidates, else fall back to lossless PNG (always built) · explicit pin of unbuilt codec
→ exit 4 (DEC-004) · batch partial failure → exit 6 (DEC-015). Determinism: identical
`(pixels, profile, feature-flags, mode)` ⇒ identical output; all thresholds in one `DecisionPolicy`
consts block.

## Open decisions
`photo_score` weights/cutoffs need a labeled fixture set (tune in-build, record a DEC) · profile
set (`web`/`docs`/`preserve`, maybe `thumbnail`/`email`) · AVIF-in-budget default vs opt-in ·
explain channel · **row-B indexed-PNG needs a permissive quantizer** (`image` PNG encoder doesn't
auto-quantize; imagequant is GPL-excluded → **defer PNG-indexed, use lossless WebP's palette
transform**; log `quantette` on the license-watchlist).

Load-bearing files: src/quality/mod.rs (search + LossyFormat + fit_under_size), src/cli/mod.rs:238
(Optimize args), :2763 (run_optimize), :2722 (optimize_auto_config), src/sink/mod.rs:250 (CodecNotBuilt→exit 4).

Prior art: Cloudinary q_auto/f_auto (examine pixels→trial-encode shortlist→perceptual score→pick
smallest meeting quality), cwebp/gif2webp `-mixed`, libvips/sharp (lossy RGB + lossless alpha),
pngquant/oxipng palette heuristics.
