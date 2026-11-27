---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-118
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5-5
  session_id: null

project:
  id: PROJ-017
repo:
  id: crustyimg

created_at: 2026-11-26
# Stands beside DEC-110 (native against wasm on the same input) and DEC-112 (crop's fractions
# and the orientation key); neither is changed. Supersedes none.
supersedes: null
superseded_by: null

affected_scope:
  - tests/ddphotos_parity.mjs
  - tests/ddphotos_upscale.py
  - integrations/ddphotos/edit/recipe.js

tags:
  - ddphotos
  - crop
  - replay
  - parity
---

# DEC-118: A crop recipe from the edit page replays on the original within a stated bound

## Decision

1. **Sizes.** The CLI's output from the original and the page's from the copy differ, per side, by at
   most:
   - **box:** 0 px when the original is a whole multiple of the copy, except where `resize`'s
     `side · (n / long)` falls on an exact half in one image and not the other in `f64`, then 1 px
     (4 of 1,521,000 cases); 1 px for an original at least 2× the copy; 2 px below 2×.
   - **aspect:** 1 px at the ratios driven, because the short side is rounded on each image separately.
     Exhaustive over 273,365,360 cases (long sides 1601–4800 and 6400, every short side from a third of
     the long side up).

   These are driven maxima, with the copy made from the original by `resize` mode `max` to 1600 px, at
   the ratios driven (1:1, 4:5, 3:2 and 16:9, in both orders). Portrait originals are assumed to mirror
   landscape; no one drove them. The box bound below 2× is measured over a 3.1M-case
   sweep, not proven.
2. **Orientation.** A preset that follows the photo writes `orientation = "match"`. A preset on a
   square copy, and 1:1 on any copy, write the aspect as shown, with no key: an original that is square
   on the copy but not in fact (|W − H| small enough to round away) would otherwise replay in the other
   orientation.
3. **Framing.** A box's edges are each within DEC-112's bound of their exact position on both images.
   An aspect is centred on each image separately, so its framing may differ by up to half a copy pixel.
4. **Pixels.** On the harness's Pillow-made original, the replay's PSNR (RGB, shared region) is at least
   **53.5 dB for an aspect** and **59 dB for a box**: the lowest measured on each form's rows (53.95 and
   59.46), rounded down to 0.5 dB, with no further headroom. A one-copy-pixel framing shift to the left
   reads at most 51.71 dB on an aspect row and 52.77 dB on a box row, below both floors. A shift to the
   right does not: it cancels an aspect's half-pixel centring offset, so driven right, R1 reads 54.66 dB
   and R9 54.05 dB, both above the 53.5 floor. The `placement` check pins aspect placement instead. The floors hold for Pillow
   12.3.0's `LANCZOS` upscale; a real photo's original has detail the copy lacks and reads lower, so
   they are not a promise about real originals.
5. **A smaller file.** `resize` mode `max` never upscales, so on a file smaller than the copy the CLI's
   output is smaller than the page's, or its watermark is refused. The bound applies to an original at
   least as large as the copy.

## Context

The edit page edits the published copy; its CLI recipe (`cliRecipe`) runs on the original, which is
larger. DEC-110 bounds native against wasm on the same input; nothing bounded the copy against the
original. Crop is the first page step whose meaning depends on the image's size, and it comes in two
forms: a preset is an `aspect` (readable and reusable), a placed box is four fractions.

## Alternatives Considered

- **A box for every preset.** Exact at an integer multiple, one crop form on the page.
  - Why rejected: a recipe copied from the page should read `aspect = "3:2"` and work on any photo, not
    carry four fractions that fit one photo.
- **The key on every preset, square photos included.**
  - Why rejected: driven, an original that is square on the copy but portrait by one pixel replays
    transposed.
- **One PSNR floor for both forms.**
  - Why rejected: an aspect's framing differs by up to half a copy pixel by design, so its rows read
    lower; a single floor would have to be the aspect's and would not see a one-pixel shift on a box.
- **Option chosen:** the bounds and per-form floors above, with the sizes asserted as literals.

## Consequences

- **Positive:** the property "the recipe means the same thing at 4× the size" has a stated bound and a
  test that fails when the bound, a floor or the orientation rule moves.
- **Negative:** a recipe made from a preset on a square photo does not follow orientation when reused on
  a folder. The floors are not a promise about real originals.
- **Neutral:** the harness asserts exact sizes on its integer-4× rows (none of them is at a tie), rule
  1's bound and rule 4's floors on every replay row, and a framing-shift control that must turn every
  replay row red. Changing the floors, the bound, or the rule in 2 needs a new measurement and an edit
  here. ddphotos' photogen makes the real copy, not crustyimg's `resize`; if it rounds differently, rule
  1's non-integer bounds must be re-driven against it.

## Validation

`tests/ddphotos_parity.mjs`: the crop rows, the replay rows R1–R17, the comparator cases B11–B19, and the
controls N4 (one pixel of crop) and N5 (the original's content moved one copy pixel). Revisit if
photogen's resizer is found to round differently from `resize` mode `max`.

## References

- DEC-110, DEC-112 (rule 8), DEC-113, DEC-114 (rule 5)
- `tests/ddphotos_upscale.py` makes the replay originals.
