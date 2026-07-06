# PROJ-002 design brief — Image classification (Analysis-layer)

> Design-only input for the PROJ-002 planning session (feeds `docs/research/proj-002-findings.md`).
> Deterministic, no-ML, internal enabler for the format auto-decision engine; surfaced at most
> as a one-word `explain` label. Produced by a research+design agent 2026-07-05; grounded in
> the actual codebase (`src/image/mod.rs`, `src/quality/mod.rs`) + cited prior art.

## Design stance
Classification exists for one reason: bias the format/quality engine toward the right codec
family (photographic → lossy JPEG/AVIF/lossy-WebP; graphic/flat → lossless palette/PNG/
lossless-WebP). Not a product surface (zero user-facing demand). Bar: *cheap enough to always
run, right often enough to beat a format-blind default, never blocking.* The load-bearing
prior-art insight: **the codec decision and the classification decision are the same signal**
— Cloudinary `q_auto`/`f_auto` detects "photographic vs non-photographic" precisely to pick
JPEG-vs-PNG. Reconstruct the "camera vs computer" bit deterministically from pixels + container.

## Feature set (all from the single decoded RGBA buffer + already-captured container facts)
**Single O(pixels) pass** accumulates: unique-color count (`HashSet<u32>` packed RGB,
short-circuit at 4096), quantized 4-4-4 color histogram `[u32;4096]`, alpha coverage
(translucent + fully-transparent ratios), saturation buckets (via chroma range `max-min`, no
HSV convert), near-gray ratio, luma histogram `[u32;256]`. **Derived from luma histogram
(O(256)):** entropy `H=-Σp·log2p` (low⇒graphic/flat, high⇒photo — the screenshot-detection
patent's exact heuristic), bimodality (mass in top-2 bins — documents/line-art are bimodal).
**Second linear pass — edge density** via Sobel-lite (`|L(x+1,y)-L(x-1,y)|+|L(x,y+1)-L(x,y-1)|`,
integer, no imageproc): edge-pixel ratio + flat-region ratio (photos = many soft textured
edges; graphics = few long axis-aligned edges + large flat fills). Subsample/stride on large
images (cap ~1–2M sampled px). **Zero-cost container features already in crustyimg:**
`source_format` (GIF/BMP/PNG→graphic lean, JPEG→photo, ICO→icon), **EXIF presence (the
decisive camera prior)**, ICC presence (weak photo lean), dimensions/aspect (≤128²→icon;
16:9/16:10 large→UI-screenshot), alpha present (logos/icons/UI, rarely photos).

## Rule-based cascade (short-circuit, cheapest/strongest first)
1. **Icon** — `max(w,h)≤128 & aspect∈[0.5,2]` (or `src==Ico`) → keep lossless/palette.
2. **Graphic/logo** — `n_colors≤256 OR (flat_ratio≥0.60 & low edge)` → lossless; the
   pngquant/WebP ≤256-color palette gate (industry-standard "few colors ⇒ lossless").
3. **Document/scan** — `bimodality≥0.55 & gray_ratio≥0.85 & entropy<4.5` → lossless.
4. **UI-screenshot** — `flat_ratio≥0.35 & n_colors∈(256,~50k] & !has_exif & screen aspect`
   → mixed; the hardest class (merge into graphic if the engine treats them identically).
5. **Photograph** — `has_exif` (decisive) OR `n_colors>4096 & entropy≥5.0 & flat_ratio<0.25`
   → lossy.
6. **Fallback → photograph, conf 0.4** (see below).

Thresholds are **starting anchors, tune against a labeled fixture corpus** during build and
record the constants in a DEC (like the existing `MAX_SEARCH_ITERS`/`AVIF_SPEED` constants).

## Confidence + fallback (never block)
Every class carries `confidence: f32`; classification never errors/blocks. On ambiguity
(conf < ~0.5) fall back to **photograph** — because the safe downside is "file a bit larger"
(photo forced lossless) vs the bad downside "artifacts" (graphic forced lossy smears
text/edges). *Tunable:* crustyimg's lossy path is already SSIMULACRA2-target-bounded
(`src/quality/mod.rs`), so artifact risk is bounded — record the bias choice in the DEC.
Contradictions resolved by **cascade precedence, not averaging**; lower reported confidence so
`explain` can hedge. **Honest gray zones:** photo-of-a-document / photo-of-a-screen (EXIF
wins → labeled "photograph", but that's usually the right *format* call anyway);
gradient-heavy modern UI/illustration (soft gradients inflate n_colors/entropy → looks
photographic); dithered GIFs; grayscale-photo vs document.

## The minimal subset (don't over-build)
Four features carry ~all of the format decision: **`has_exif`** (camera prior), **capped
`n_colors`** (palette gate), **`edge_ratio`+`flat_ratio`** (pair), **`entropy`** (tie-break),
plus **`has_alpha`** (can't be baseline JPEG). Everything else refines the cosmetic label, not
the decision. **Collapse six labels → three optimization buckets** the engine switches on:
`Lossy` (photograph) · `LosslessFlat` (logo/graphic/icon/document) · `MixedSafe`
(ui-screenshot/illustration). Keep fine labels only for `explain` cosmetics; don't build six
independent detectors.

## Slotting into the `Analysis` layer
New `src/analysis/mod.rs`, layered exactly like `src/quality/mod.rs` (depends only on
`::image`, `std`, `crate::image::{Image,ImageInfo,MetadataBundle}`; never touches clap/cli/
sink/files). `Analysis::compute(&Image)` runs the passes once after decode (decode-once, DEC-002
respected — no re-decode), caches the feature scalars + `class` + `confidence` + `opt_bucket`,
and is borrowed by the **format engine** (reads `opt_bucket`+`has_alpha` → codec family, then
its existing perceptual/byte search), **`lint`**, and **`explain`**. Cheap/reusable: the color
histogram is dual-use (palette decisions, compression-heatmap). Feature-flaggable: a lean build
skips it → `opt_bucket=Lossy`/format-preserving, `confidence=0`, engine falls back to current
behavior. Deterministic (integer/fixed-order f32, no RNG/ML).

## Prior art (cited)
USPTO photo/graphic classification patents [US 7657089](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/7657089),
color-discreteness [US 6996277](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6996277),
edge features [US 6985628](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6985628),
screenshot entropy [US 12008062](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/12008062);
[arXiv 2510.08332](https://arxiv.org/html/2510.08332v3) (edge density dominant),
[arXiv 2504.03020](https://arxiv.org/pdf/2504.03020) (page classification);
[Cloudinary q_auto/f_auto](https://cloudinary.com/documentation/image_optimization);
lossy-vs-lossless "camera→lossy, computer→lossless" [Image CDN](https://theimagecdn.com/docs/lossy-vs-lossless-compression),
[Pixboost](https://pixboost.com/blog/png-to-next-gen-conversion/); [pngquant](https://pngquant.org/),
[WebP palette study](https://developers.google.com/speed/webp/docs/webp_lossless_alpha_study).
