---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-112
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-017
repo:
  id: crustyimg

created_at: 2026-11-05
# Applies DEC-091 (Fence A) to geometry; amends nothing.
supersedes: null
superseded_by: null

affected_scope:
  - src/operation/mod.rs
  - src/operation/crop.rs
  - src/operation/registry.rs
  - recipes/README.md
  - docs/data-model.md
  - docs/recipes.md
  - skills/crustyimg/SKILL.md
  - integrations/ddphotos/edit/**

tags:
  - recipe
  - geometry
  - crop
  - units
  - wasm
  - ddphotos

references:
  - dec-091
  - dec-102
  - dec-095
  - dec-109
  - dec-110
---

# DEC-112: A recipe carries geometry lengths as fractions of the image, and rounds each edge on its own

## Decision

**1. Units.** A recipe step's geometry lengths and positions are **fractions from 0 to 1 of the
image the step receives**, written as TOML numbers. There are no pixel units in a recipe. For
`crop`, `x` and `width` are fractions of the width, `y` and `height` fractions of the height. TOML
integers `0` and `1` are read as 0.0 and 1.0. A pixel-minded value such as `width = 800` is refused
at load by the key table (`'width' must be a number > 0 and <= 1, got 800`), never read as pixels.
The values are read as `f64`.

**2. Rounding.** For a W×H buffer, each edge rounds on its own, in `f64`, with `f64::round` (halves
round away from zero, which is up for these non-negative values):

```
left   = round(x · W)            right  = round((x + width) · W)
top    = round(y · H)            bottom = round((y + height) · H)
crop   = (left, top, right − left, bottom − top)
```

The testable claim: **every edge is within 0.5 px of its exact position `f · W`, on every image
size**, and so the same fractions on a 1200 px and a 6000 px copy of one image give edges that agree
within `0.5 + 0.5/5 = 0.6` px once scaled. Drift comes in multiples of 0.2 and halves round up, so
in exact maths the largest value is 0.4 px. In `f64` it is not: `x * 1200` and `x * 6000` are
rounded separately, so near a half-pixel tie the two sizes can round in opposite directions and
the drift reaches 0.6. **0.4 px is measured over SPEC-145's seeded 200,000-case sweep; 0.6 px is the
bound in general, reached at `f64` near-ties.** Driven on the binary: `width = 0.009583333333333333`
(`x = 0`, `height = 1`) gives a 12 px crop at 1200 and 57 px at 6000, and 57/5 = 11.4 is a drift
of 0.6. SPEC-145's test asserts 0.4 for its fixed seed only.

**3. Bounds, without a clamp.** Load refuses `x + width > 1` and `y + height > 1` (computed in
`f64`). For any `f64` value `s ≤ 1`, `s · W ≤ W`, so `right ≤ W` and `bottom ≤ H` by construction.
Nothing is clamped, at load or at apply.

**4. Zero pixels is refused.** A crop whose rounded width or height is 0 has no meaning. It is
refused when the step runs (it needs the image size), as an op failure (`operation 'crop' failed:
…`, exit 1 on a single-input `apply`), never clamped to 1 px. It is not a skippable refusal: a
batch that skipped it would write the uncropped image. The precedent is DEC-109 (a watermark that
would draw nothing is refused), not `resize percent`'s 1 px floor.

**5. Aspects are whole numbers.** An aspect is `"W:H"` with each side an integer from 1 to 65535
(`"239:100"`, not `"2.39:1"`). A decimal is refused at load with a message that shows its
whole-number spelling, reduced by the greatest common divisor (`"1.5:1"` is `"3:2"`). The fit is
exact integer maths in `u64`, with the short side rounded half up (SPEC-145 states it in full).

**6. What later ops inherit.** `pad` is the next op to carry lengths. **A `pad` length or position
in a recipe follows this rule**: a fraction of the image the step receives, `f64`, each edge rounded
on its own as above, and zero-pixel results refused, not clamped. Two things stay `pad`'s own call:
what a pad fraction is a fraction *of* when it grows the canvas (the input side is the default this
DEC expects), and whether `pad` takes an exact output size at all. An exact pixel size is the same
kind of parameter this DEC excludes, so `pad` either refuses it in a recipe or writes a DEC that
amends this one. `trim`'s `tolerance` is a value-space threshold, not a length, and is outside this
rule.

**7. Pixels for people.** A CLI user who thinks in pixels gets them from `edit --crop`
(STAGE-058's `edit`-flags spec), which knows the image and converts to fractions. That round trip is
exact (below), so `--save-recipe` replays to identical pixels.

## Context

PROJ-017's first-named risk: the ddphotos edit page edits ddphotos' published copy (1600 px long
edge) and the CLI replays the recipe on the original. Measured for SPEC-145's frame on a 6000×4000
original whose copy is 1600×1067, with a crop dragged at x=400, y=300, 800×500 px on the copy:

| recipe carries | lands on the original | result |
|---|---|---|
| copy pixels | columns 400–1200, rows 300–800 | 13.3 % × 12.5 % of the frame, top-left, instead of 50 % × 46.9 % centred; exit 0 |
| fractions (`x=0.25 y=0.28116… width=0.5 height=0.46860…`) | columns 1500–4500, rows 1125–2999 | each edge within 0.5 px of exact (1500, 4500, 1124.65, 2999.06) |

Both halves exit 0 on the pixel recipe, so nothing would report it. A crop has to come before the
page's resize (or the output loses the original's resolution), so the page's watermark workaround
(resize to the copy's size first) can't help.

DEC-091's Fence A already settles the category: a workhorse parameter must mean the same thing on a
different image. Fractions, aspects and gravities do. Absolute pixel coordinates don't, and
"absolute rect → lab" is the fence's own worked example. STAGE-058's design note ("carry both") is
overruled by the maintainer's 2026-11-05 ruling, which accepted the frame's three recommendations:
fractions only, whole-number aspects, refuse zero pixels.

**The pixel round trip is exact** (measured at the frame, f64): for every rectangle on every width
W ≤ 300 (4,545,100, exhaustive) and 2,000,000 random rectangles with W ≤ 50,000,
`round((a/W)·W) = a`, `round((a/W + b/W)·W) = a + b` and `a/W + b/W ≤ 1.0`, with 0 mismatches. For
every `i/n + (n−i)/n` with n ≤ 3000, the f64 sum is never above 1.0. At design (2026-11-05), every
pair of 2-, 3- and 4-decimal fractions that add up to exactly 1 in decimal also sums to at most 1.0
in `f64` (0 of 99, 999 and 9,999 pairs exceed it), so a hand-written `x = 0.25, width = 0.75` is
never refused by float error.

**Why `f64` and not the `f32` other ops read:** an `f32` read moves `x = 0.0009375` on a 1600 px
image from 1.5 px (rounds to 2) to 1.49999997 px (rounds to 1). The key table still checks the
`f32` reading too (DEC-102), so a value that is 0 as an `f32` (`1e-50`) is refused at load.

## Alternatives Considered

- **Both units, told apart by TOML type** (integer = pixels, float = fraction). STAGE-058's note.
  - Rejected: `width = 1` and `width = 1.0` would mean different things, while `resize` already
    refuses `width = 800.0` for a pixel key, so the convention would flip between ops. It needs a
    `ParamKind` whose meaning depends on the value's type, which is DEC-102's "Revisit if". And it
    puts lab-side parameters in the workhorse against Fence A.
- **Percent strings** (`x = "10%"`).
  - Rejected: a new string-parsing `ParamKind` for no gain over a float, and a 0–100 scale beside
    the 0–1 aspect maths. The page would format strings it then has to parse back in tests.
- **A pixel key family** (`x_px`, `width_px` … beside the fraction keys).
  - Rejected: doubles the table and the exclusivity rules, and the pixel family is still lab-side
    under Fence A.
- **Round the origin and the size** (`left = round(x·W)`, `cw = round(width·W)`).
  - Rejected: the far edge then drifts up to 1 px, so two crops that abut in fractions overlap or
    leave a gap, and the 0.5 px claim fails (a 200,000-case sweep measures 0.80 px for this rule,
    and the same for `floor` on the right edge).
- **Clamp a zero-pixel crop to 1 px**, as `resize percent` does.
  - Rejected by the maintainer: a 1 px crop is not what was asked, and it would be written at exit 0.

## Consequences

- **Positive:** the ddphotos page can write a crop with no knowledge of the original's size; native
  and wasm get the same integers by construction (IEEE multiply and round, no transcendental
  function), and crop never resamples, so DEC-110 limb 2 applies: dimensions and pixels exact.
- **Negative:** a recipe valid on a 6000 px original can be refused on a 40 px thumbnail (its crop
  rounds to 0 px). Accepted. A user who wants "1 px" cannot say it; `width = 1` means the full width,
  and the docs say so.
- **Neutral:** no `CACHE_SCHEMA_VERSION` change and no lockfile change: `crop` is a new op name, an
  existing recipe's canonical TOML is unchanged, and 0.8.2 and earlier refuse a crop recipe with
  `unknown operation 'crop'`, so no old cache entry can hold one.
- **Orientation is not decided here.** The fractions are of the buffer the step receives; that is
  the stored orientation unless an earlier `auto-orient` step baked it. STAGE-058's rotate/flip spec
  owns what an orientation tag means after geometry.

## Validation

Right if: SPEC-145's `crop_edges_within_half_a_pixel` and `crop_fraction_agrees_at_1200_and_6000`
stay green, and STAGE-060's replay-parity row (the page's recipe on the copy versus the CLI's on
the original) holds within these bounds.

Revisit if: a recipe consumer needs an exact pixel size that no fraction can carry (for example an
og-card preset of exactly 1200×630 in PROJ-014; that is a `resize`, which already takes pixels, and
belongs after the crop), or `pad` needs a length that is not a fraction of its input.

## References

- Related specs: SPEC-145 (crop, this DEC's first user), SPEC-142 (native/wasm parity), the
  STAGE-058 `edit`-flags spec (`edit --crop`, pixel to fraction)
- Related decisions: DEC-091 (Fence A), DEC-102 (load-time key tables), DEC-095 (colour type and
  depth), DEC-109 (refuse a step that would do nothing), DEC-110 (native/wasm agreement)
