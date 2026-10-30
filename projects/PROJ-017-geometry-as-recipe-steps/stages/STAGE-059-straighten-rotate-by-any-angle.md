---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-059
  status: proposed
  priority: high
  target_complete: null

project:
  id: PROJ-017
repo:
  id: crustyimg

created_at: 2026-10-30
shipped_at: null

value_contribution:
  advances: >
    The "straighten" half of PROJ-017's thesis: `rotate` takes any angle, so the
    second edit a photo owner reaches for after crop is a recipe step too.
  delivers:
    - "`rotate` with any angle, in `expand` mode (a canvas that holds the whole rotated image, filled with a colour or transparent)"
    - "`rotate` in `crop-to-fit` mode (the largest axis-aligned rectangle inside the rotated image, so a straighten has no corners)"
    - "Resampling in linear light at the source's bit depth, checked against an independent implementation"
  explicitly_does_not:
    - "Detect the horizon or suggest an angle. That's judgment, and smart-crop territory"
    - "Correct perspective or lens distortion"
    - "Fix the working-type performance regression. PROJ-015 owns it, and this stage depends on it"
---

# STAGE-059: Straighten: rotate by any angle, resampled in linear light, crop-to-fit or expand

## What This Stage Is

When this stage ships, `rotate` with `angle = 2.5` works. In `crop-to-fit` mode it returns
the largest upright rectangle with no empty corners. In `expand` mode it returns the whole
rotated image on a larger canvas. Either way the resampling happens in linear light at the
source's bit depth, which is the same rule SPEC-122 set for `resize`, for the same reason:
averaging gamma-encoded values darkens high-contrast edges.

## Why Now

It extends STAGE-058's `rotate` op and key table rather than adding an op, so it follows
directly. ⚠ **It should not be built before PROJ-015's working-type stage.** SPEC-122's
`F32x4` round trip made `resize` 1.5–2.5× slower in 0.8.0, and 72% of that cost is the working
type rather than the maths. A straighten built on the same path inherits it. If the
working-type stage hasn't landed when this stage is designed, the design measures the cost
on a 24 MP image and states it as accepted, in the spec, rather than discovering it in
verify.

## Success Criteria

- `rotate` accepts any finite angle. The STAGE-058 refusal of non-multiples of 90 is
  removed, and multiples of 90 still take the exact path (no resampling, same bytes as
  STAGE-058).
- `crop-to-fit` output dimensions match the closed-form largest inscribed rectangle for the
  angle and aspect, asserted exactly. `expand` dimensions match the rotated bounding box.
- Output is checked against an independent implementation (`vips rotate` or ImageMagick
  `-distort SRT`, run as a test oracle and not a dependency), with SSIMULACRA2 or a stated
  per-pixel bound. It's never checked against itself.
- A gamma-space control shows the linear path is the one running: the edge-darkening
  difference is measured, not assumed.
- 16-bit and alpha inputs keep their depth and alpha (DEC-095).

## Scope

### In scope
- The resampler: filter choice, edge handling, and whether it is in-house or a dependency
  (a DEC if a dependency)
- Both modes, the fill colour for `expand`, and transparency where the output allows it
- `edit --rotate <degrees>` accepting any angle

### Explicitly out of scope
- Auto-straighten / horizon detection
- Arbitrary affine transforms beyond rotation (shear, perspective)
- The working-type change itself

## Spec Backlog

In build order.

- [ ] (not yet written) — [M] **Any-angle `rotate`, `expand` mode, and the resampler.**
  Lifts STAGE-058's refusal on the same op. ⚠ **The resampler is the design call.** In-house
  (an inverse-mapped bilinear or bicubic sampler over the linear working buffer SPEC-122
  already builds; price it in lines, per the repo's in-house-small-formats precedent) vs a
  dependency (`imageproc`'s rotate: probe its licence, its transitive graph with
  `cargo tree -i --all-features`, and whether it works on 16-bit before naming it). A
  dependency needs a DEC under `no-new-top-level-deps-without-decision`. Checked against
  `vips rotate` as an oracle. Includes the 24 MP timing against `vips`, stated in the spec.
- [ ] (not yet written) — [S] **`crop-to-fit` mode.** The largest axis-aligned rectangle
  inside a w×h image rotated by θ, centred, as a closed form with exact dimension tests at
  several angles and aspects (including portrait, and angles near 0 and 45°). Then crop to it
  with STAGE-058's crop. ⚠ Decide the default mode. The ddphotos page will want
  `crop-to-fit`; a CLI user rotating a logo may want `expand`.
- [ ] (not yet written) — [S] **The linear-light control and the depth/alpha tests.** A
  high-contrast fixture (a one-pixel black/white grid) rotated 2.5° in linear vs gamma space,
  with the difference measured and the linear path asserted. Plus 16-bit and alpha round
  trips. ⚡ This can fold into the first spec if design finds the first spec stays [M] with
  it. It's split out here so it can't be dropped quietly.

**Count:** 0 shipped / 0 active / 3 pending

## Design Notes

- **Multiples of 90 never resample.** `angle = 90` must produce STAGE-058's exact bytes, or
  the two stages disagree about one op. Test it.
- **Angle convention.** State the sign (counter-clockwise positive is the usual convention;
  `vips` and ImageMagick differ), and pin it in a test against the oracle. A sign error
  passes every self-test.
- **The fill colour** for `expand` reuses `pad`'s colour parsing and its alpha-less-format
  rule from STAGE-058.

## Dependencies

### Depends on
- STAGE-058: the `rotate` op, `crop`, and `pad`'s fill rule
- SPEC-122 (shipped): the linear working buffer
- ⚠ PROJ-015's working-type stage (not yet defined): see Why Now

### Enables
- STAGE-060's straighten control

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
