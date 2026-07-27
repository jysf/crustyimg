# Classifier fixtures — what each one is, and where its numbers come from

These images calibrate `PHOTO_ENTROPY_STRONG` (DEC-047). The entropy figures below
are **luma entropy in bits**: Shannon entropy of the 256-bin histogram of the integer
BT.601-ish luma `(77R + 150G + 29B) >> 8`.

| fixture | class | entropy | role |
|---|---|---|---|
| `grayscale_photo_leica.png` | photograph | 6.0743 | real photo (Leica B&W) |
| `grayscale_photo_canon.png` | photograph | 6.8300 | real photo (Canon B&W) |
| `color_photo_fuji.png` | photograph | 6.3737 | real photo (Fuji colour) |
| **`photo_entropy_floor.png`** | photograph | **4.5176** | **lower boundary — the lowest-entropy image that must still be lossy** |
| **`dither_32color.png`** | graphic-logo | **3.6414** | **upper boundary — the highest-entropy image that must still be lossless** |
| `dithered_graphic.png` | graphic-logo | 3.0259 | real 8-colour error-diffusion dither |
| `checker_graphic.jpg` | graphic-logo | 2.7800 | hard-edged graphic |

The two boundary specimens define the calibration window the threshold has to sit in:
**(3.6414, 4.5176]**, 0.88 bits wide. Without them the window is (3.03, 6.07] — 3.04
bits — which is wide enough that it stays green at `PHOTO_ENTROPY_STRONG = 5.5`, the
value that reinstates the SPEC-105 bug. That is why they exist.

## How the boundary specimens were seeded

Both are produced by `scripts/seed-classify-specimens.py`, which is dependency-free,
never calls crustyimg, and measures entropy with **its own** implementation of the
definition above. Run it to regenerate, or with `--check` to verify the committed bytes
still match the recipe:

```bash
python3 scripts/seed-classify-specimens.py --check
```

This separation is the point. A fixture whose asserted value was read off the code
under test cannot fail — it would agree with any implementation, including a broken
one. Here the recipe predicts the value, an independent implementation measures it, and
the Rust test asserts that measured number. Change the luma weights or the entropy
maths and `boundary_specimens_measure_their_recorded_values` goes red.

Positive control for the independent implementation: it reproduces the four
pre-existing fixtures to four decimals (6.0743 / 6.8300 / 6.3737 / 3.0259), matching
what `crustyimg web --json` reports for each.

### `photo_entropy_floor.png` — a real photograph under flat light

Source: `grayscale_photo_leica.png`. Curve: compress the tonal range to a **third** of
full range about mid-gray, `v' = 128 + (v - 128) // 3`, per channel.

That is what heavy haze or flat overcast light does to a capture — real photographic
structure living in a third of the tonal scale. `k = 1/3` comes from that story, not
from any threshold. The entropy was predictable before measuring: merging three input
levels into one costs `log2(3)` bits, so `6.07 - 1.58 ≈ 4.49`. Measured: **4.5176**.

Its `flat_ratio` is **1.00** — the scale-broken flat detector reads it as completely
flat — so cascade rule 3.5 is the only thing keeping it off the lossless path. That is
what makes it the boundary rather than just another photo.

### `dither_32color.png` — the adversarial high-entropy graphic

Source: `color_photo_fuji.png`. Palette: **32 evenly spaced grey levels** (5-bit grey),
a fixed ramp. Render: Floyd–Steinberg error diffusion, the textbook
7/16 – 3/16 – 5/16 – 1/16 kernel, left to right, no serpentine.

The entropy is bounded **by construction**: quantising 256 luma levels to 32 discards
at most `log2(256/32) = 3` bits, so `6.37 - 3 ≈ 3.37` was the prediction. Measured:
**3.6414** — under the 4-bit floor with real margin, and well above the flat 8-colour
dither already committed at 3.03.

Two notes on the choices, since both were arrived at by measurement:

- **A fixed ramp, not an adaptive palette.** Median cut splits boxes at medians, so its
  boxes hold equal pixel populations and the resulting histogram is driven toward
  uniform — pushing entropy at `log2(levels)`. A fixed ramp lets the image's own tonal
  distribution set the occupancy.
- **32 levels, not 16.** DEC-047 cites a 16-colour dither at 3.43, but that figure
  cannot be reproduced from the photographs this repo actually has. Quantising to
  L levels costs about `log2(256/L)` bits, so a 16-level dither of a 6.07–6.83 bit
  source lands at 2.46–2.88 — measured 2.80 for the Fuji frame — below the 3.03 dither
  already committed, so it would not tighten the window at all. Reaching 3.43 at 16
  levels needs a ~7.4-bit source, which none of these are. Histogram-equalising first
  gets there (measured 3.94) but leaves only 0.06 bits of margin under the threshold,
  which is a fixture that will flip on any small change. 32 levels of the unmodified
  photograph gives the same boundary role with 0.36 bits of margin.

A 32-**colour** dither is a different animal from a 32-**level grey** one: an adaptive
RGB palette's 32 colours carry far more than 32 distinct lumas, which is why DEC-047
records one reaching 5.1 and crossing the floor. This specimen is the grayscale-level
variant precisely because its ceiling is structural.

## The other fixtures

`grayscale_photo_leica.png`, `grayscale_photo_canon.png` and `color_photo_fuji.png` are
downscaled crops of real frames the maintainer owns, EXIF-stripped.
`dithered_graphic.png` is a real 8-colour error-diffusion dither and
`checker_graphic.jpg` a hard-edged synthetic checkerboard; both predate SPEC-109.
