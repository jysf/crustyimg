# Design-time probe: a grayscale photograph is misclassified as a graphic

Repo @ `b69e344` (main, clean start AND end — probe ran read-only, all building in a
scratch `git worktree` removed afterward). Investigation only; no fix, no repo change.

Triggered by a live demo observation (2026-07-25): a real Leica B&W portrait DNG,
converted by the demo's **Auto** path, came out as **lossless WebP, 2048×1367, 823.5 KB**
instead of a lossy AVIF (~62 KB). Auto should pick AVIF for a photograph.

## Confirmed bucket + root feature

Native `optimize --explain=json` on `_incoming0/L1024678.DNG`:

```
class: graphic-logo   bucket: LosslessFlat   winner: webp lossless
entropy 7.45   edge 0.00   flat 0.96   unique_colors 256 (not saturated)   has_exif: NO
```

The classification cascade is `src/analysis/mod.rs::classify` (SPEC-047/DEC-047),
first-match-wins. Two things stacked to misroute the photo:

1. **Rule 4 clause 1 (`mod.rs:584`): `few_colors = !saturated && n <= PALETTE_COLORS` (≤256).**
   A grayscale image stores r=g=b, so it has **at most 256 distinct RGB colours** →
   `few_colors=true` → `GraphicLogo` → lossless. A grayscale photo trips this regardless
   of how detailed it is.
2. **`has_exif=false` bypassed the camera prior (rule 2, `mod.rs:572`).** The RAW
   embedded-preview decode drops EXIF (`crustyimg info` → `exif: no`), so the decisive
   "shot on a camera → Photograph" rule never fires. Real camera JPEGs carry EXIF and
   match rule 2 *before* the palette gate — which is exactly why this bug is invisible on
   normal JPEGs and shows up on RAW.

## Native affected? YES — this is the shared Analysis layer, not a demo bug

`crustyimg optimize --max 2048` on the same file natively produces **843,260 B = 823.5 KiB
lossless WebP — byte-identical to the demo symptom.** Any EXIF-stripped photograph is
exposed, not only the demo.

## Isolation matrix (all EXIF-stripped)

| input | edge | flat | colours | class |
|---|---|---|---|---|
| `gray_noise` (max edges) | 0.89 | 0.00 | 255 | **graphic-logo** (only `few_colors` can fire) |
| `color_noise` (max edges) | 0.86 | 0.00 | 4096 (sat) | photograph → AVIF |

→ **Grayscale alone flips the bucket via `few_colors`, independent of smoothness.**
Independently, **every** real high-res photo tested reads `edge≈0.00, flat 0.66–0.96`:
the `EDGE_THRESHOLD`/`FLAT_THRESHOLD` anchors were tuned on 64×64 synthetics, but at
megapixel resolution adjacent pixels are correlated, so forward-difference deltas are
~always ≤ threshold → everything reads "flat." So the **flat-graphic gate (rule 4 clause 2)
also mis-fires on EXIF-stripped *colour* photos.** Both triggers are active on the Leica;
grayscale/`few_colors` is the primary, robust one. **The EXIF prior has been masking a
scale-broken flat detector.**

## Cost quantified

At 2048×1367: lossless WebP **843,260 B** vs AVIF q85 **63,797 B = 92.4 % smaller**,
SSIMULACRA2 **83.2** (visually excellent; demo target ~82). ~780 KB / ~13× left on the table.

## Recommended direction (not implemented)

The discriminator is **entropy**: a B&W *photo* is 7.45; a logo is ~0.95, a document ~1.0 —
a wide, clean gap.

1. **A strong-entropy → Photograph signal ahead of the graphic gates** (equivalently,
   entropy-gate the graphic rules so `GraphicLogo` requires *low* entropy). A high-entropy
   image — grayscale or colour, EXIF or not — is never a graphic. Fixes the symptom *and*
   the broader EXIF-stripped-photo exposure at once, because it neutralises both broken
   gates for photos.
2. **Scale-normalize the edge/flat thresholds** (sample at a normalized stride, or
   downsample before the edge pass) so "flat" means the same at 64 px and 20 MP — a deeper,
   separate correctness fix. Non-urgent once (1) protects photos.
3. *(complementary)* carry EXIF through the RAW-preview decode so rule 2 covers RAW.

**Risk: safe in the dangerous direction.** Genuine graphics/logos/documents are
low-entropy → they stay lossless (the real differentiator is preserved); only high-entropy
≤256-colour images (grayscale photos) move to lossy. A grayscale photo forced lossless is
merely a bigger file (bounded); a graphic forced lossy smears edges — the entropy signal
protects that side.

## Could-not-verify

- Not driven in the live demo (probe constraint); reproduced via the shared native engine
  with byte-identical output and an identical verdict. The wasm Auto path
  (`auto_avif_quality` returns AVIF only for `Lossy`/`MixedSafe` buckets) confirmed by
  code reading.
- The probe's 2048 downscale kernel isn't guaranteed bit-identical to the demo's — but the
  bytes coincide exactly at 823.5 KiB, so the conclusion holds.
- **No real grayscale/graphic corpus exists** (the benchmark is 8 colour photographs — the
  known content-diversity gap). The entropy cutoff needs tuning against real grayscale
  photos AND real graphics before implementing.
