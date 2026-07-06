# Deciding photo-vs-graphic without machine learning

*2026-07-06 · feature deep-dive · the analysis layer*

To pick the best format for an image, you first have to answer one
question: **is this a photograph or a graphic?** Photographs want a lossy
codec (JPEG, lossy WebP, AVIF) — they're full of soft, continuous detail
that lossy compression squeezes beautifully and the eye forgives. Graphics
— logos, icons, screenshots, line art — want a lossless codec: sharp
edges, flat fills, and text that lossy compression turns to mush.

It's the same distinction Cloudinary's `f_auto` makes ("photographic vs
non-photographic") to choose JPEG-vs-PNG. Reach for that problem today and
the reflex is to train a classifier. crustyimg doesn't. It decides the same
thing with a handful of cheap measurements and a short list of rules — no
model, no training data, no new dependency — and it's *better* for the job
precisely because it isn't ML.

## The key idea: the codec decision *is* the classification

You don't actually need to know what an image *depicts*. You need one bit:
**camera or computer?** A camera captures a continuous scene — lots of
unique colors, texture everywhere, soft edges. A computer draws graphics —
few colors, big flat regions, a handful of hard edges. That "camera vs
computer" signal is reconstructable straight from the pixels, plus a couple
of facts the file already carries.

crustyimg computes these once per image, in a single pass over the decoded
pixels:

- **Is there EXIF?** Cameras write it; graphics editors usually don't. A
  present `Exif` block is the single strongest "this is a photo" signal —
  the *decisive* prior.
- **How many distinct colors?** (Counted up to a cap of 4096, then it
  short-circuits — you never need the exact number to know "lots.") A logo
  has a dozen; a photo saturates the cap instantly.
- **Edges and flat regions.** A quick gradient pass gives the fraction of
  "edge" pixels and "flat" pixels. Photos are edges-everywhere with almost
  no flat fills; graphics are the opposite — large flat areas, few edges.
- **Entropy** of the brightness histogram — low for flat/graphic, high for
  photographic — as a tie-breaker.
- Plus **transparency** (can't be a baseline JPEG) and the **source
  format** (a `.ico` is an icon; a JPEG leans photo).

Four features carry almost the entire decision. Everything else just
refines a cosmetic label.

## A cascade, not a blend

The rules run cheapest-and-strongest first, and the **first match wins** —
no weighting, no averaging:

1. **Icon** — tiny and squarish (or an actual `.ico`).
2. **Photograph** — has EXIF. The camera prior is decisive, so it's checked
   early: a flat-ish photo of a whiteboard is still a *photo* for format
   purposes.
3. **Document** — bimodal, near-gray, low-entropy (black text on white).
4. **Graphic / logo** — few colors, or large flat fills with few edges.
5. **UI screenshot** — wide, many-colored, moderately flat, with real
   edges.
6. **Photograph** (the no-EXIF path) — rich color, high entropy, few flat
   regions.
7. **Fallback → photograph.**

Those five labels then collapse to the three things the format engine
actually cares about: *lossy* (photograph), *lossless* (graphic / icon /
document), and *mixed* (screenshots — try both, let the bytes decide).

## When in doubt, guess "photo"

Real images live in gray zones — a photo of a document, a gradient-heavy
illustration, a dithered GIF. So every verdict carries a confidence, and
under ambiguity the classifier **falls back to photograph**.

That bias is deliberate, and it's about which mistake is cheaper to make.
Call a photo a "graphic" and you compress it losslessly: the file is a bit
bigger than it needed to be — annoying, invisible. Call a graphic a "photo"
and you compress it lossily: you smear the text and ring the edges — a
*visible* defect. One mistake is reversible-looking; the other isn't. So we
err toward the safe one. (And even that is bounded — crustyimg's lossy path
is held to a perceptual quality target, so "wrong guess" still can't
produce an ugly file.)

## Why not ML — the part that's actually a feature

Skipping the model isn't a compromise here; it's the point:

- **It's deterministic.** The same image produces the same verdict on every
  run and every machine — no model version, no nondeterministic inference.
  That's what lets crustyimg promise byte-for-byte reproducible output.
- **It's free to run.** One extra pass over pixels you already decoded —
  cheap enough to run on *every* image, always on, no opt-in.
- **It ships nothing.** No weights, no runtime, no new dependency; the
  default build stays pure-Rust and zero-system-deps.
- **It's explainable.** Because the decision is just features and a rule,
  crustyimg can *show* you the exact numbers and the branch that fired —
  which is precisely what `optimize --explain` prints:

```
class=photograph entropy=7.62 edges=0.42 flat=0.00 colors=4096+
```

No black box, no "the model said so."

## Where it lives

Classification is deliberately internal — there's no `classify` command. It
exists to bias one decision (which codec family to try) and it surfaces
only as that one-word label in `--explain`. But it's computed as part of a
shared analysis layer, which means the same verdict is what the format
auto-decision reads today — and what a future asset linter and optimization
planner will read tomorrow. One cheap, honest measurement; many features
built on top.
