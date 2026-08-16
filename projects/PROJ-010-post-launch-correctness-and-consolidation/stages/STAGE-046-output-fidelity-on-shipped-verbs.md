---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-046
  status: proposed
  priority: critical                # live defects on shipped verbs, incl. the flagship
  target_complete: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-15
shipped_at: null

value_contribution:
  advances: >
    PROJ-010's correctness thesis at its sharpest point — the tool currently
    degrades what the user handed it, on shipped verbs, and reports the
    degradation as a success.
  delivers:
    - Animated input is never silently flattened, and `lint` stops recommending
      the command that destroys it.
    - Ops preserve the colour type and bit depth they were given when the target
      format can hold them.
    - Resampling is measured against its own premise before it is changed.
  explicitly_does_not:
    - Add animated output (animated WebP/AVIF encode) — that is the capability,
      not the repair; it is filed separately.
    - Introduce a 16-bit-throughout pipeline as a design goal. `DynamicImage`
      already carries the variants; only three op bodies collapse them.
    - Add any user-facing flag. "Preserve what you were given" is not something
      a user should have to ask for.
---

# STAGE-046: Output Fidelity on Shipped Verbs

## What This Stage Is

crustyimg's thesis is quality-per-byte. On four measured counts it currently
delivers less than it was given, silently, on verbs that shipped in 0.7.0 —
including `web`, the flagship. This stage makes the shipped verbs preserve what
they receive, and makes the tool say so when it genuinely cannot.

The defects were found and driven in separate exploration sessions on
2026-08-15. Their full measurements, root causes and fix constraints live in
`docs/backlog.md`; this stage is where they become schedulable work. **The
evidence is in the backlog entries — do not re-derive it, and do not trust this
summary over them.**

| # | defect | evidence | in `docs/backlog.md` |
|---|---|---|---|
| D-A | Animated input silently flattened; `lint` recommends the command that does it | driven end to end; 3 of 4 frames discarded, reported `72% smaller · ssim 100.0` | `## ⚠ Live defect — animated input is silently flattened` |
| D-B | Ops widen to RGBA and never narrow back | output IHDR read; `resize`/`thumbnail`/`edit`/`web` all emit `colour_type=6` from `colour_type=2` input; +12.4% bytes on a 512×512 PNG | `## ⚠ Live defect — ops widen to RGBA` |
| D-C | The same `to_rgba8()` call truncates 16-bit input to 8-bit | measured on a 32×32 `bit_depth=16` PNG; `convert`/`auto-orient` preserve, `resize`/`edit` halve | same entry as D-B |
| D-D | `resize` resamples in sRGB, not linear light; no premultiplied alpha | source read; `PixelType::U8x4` handed to Lanczos3 with no gamma handling | `## Open — resize resamples in sRGB, not linear light` |

**D-D's alpha half is confirmed.** The backlog entry filed it as *"not confirmed
— only two files were grepped."* A repo-wide grep (2026-08-15) returns
premultiply handling **only** in `src/image/avif.rs` (the MIAF `prem` flag on
AVIF decode). There is no `MulDiv` and no premultiply anywhere on the resize
path.

## Why Now

**Because STAGE-041 publishes the claim these defects contradict.** The launch
content promotes quality-per-byte. Shipping it while `web` invents an all-opaque
alpha channel, `resize` halves 16-bit input, and `lint` recommends a command
that discards three-quarters of an animation would put a claim in public that
the tool measurably does not meet. The maintainer sequenced this stage ahead of
STAGE-041 on 2026-08-15 for that reason.

Three further reasons the timing is now and not later:

1. **D-A has a wrong recommendation attached.** The linter tells users to run
   the destructive command. Every day it ships is a day users can follow it.
2. **The perceptual oracle structurally cannot catch D-A.** SSIMULACRA2 compares
   decoded-source to output, and both are frame 1 — the quantity it measures is
   *preserved* by the bug. Any future test asserting "score stayed high" stays
   green through this defect forever
   [[a-self-referential-control-cannot-detect-a-broken-pipeline]].
3. **D-B/C/D share one blast radius.** All three change output bytes for every
   existing recipe, which invalidates every PROJ-007 build lockfile. That
   migration should be paid **once**. The backlog entries reached this
   conclusion independently and both say to sequence them together.

## Success Criteria

- No shipped verb silently reduces colour type, bit depth, or frame count.
  Where a reduction is genuine and unavoidable (a lossy target is 8-bit), it is
  **reported**, in the spirit of SPEC-090's honest size reporting.
- `lint`'s `format/animated-gif` `fix:` string does not recommend a command that
  destroys data.
- The lockfile-invalidating changes land together, behind one migration story
  and one DEC, not three.
- D-D is **measured before it is fixed** — see Design Notes.

## Scope

### In scope
- The three op bodies that collapse the input: `Invert::apply`
  (`src/operation/mod.rs:197`), `Resize::apply` (`:396`), `Watermark::apply`
  (`:816-817`).
- Multi-frame detection on the pixel path, and the lint rule's advice.
- The resampling colour space and alpha handling inside `Resize::apply`.
- The build-cache key: does it need a colour-pipeline-version component so old
  and new renders cannot collide? (Open sub-question from the D-D entry.)

### Explicitly out of scope
- **Animated output encode** (animated GIF → animated WebP/AVIF). That is the
  capability; this stage is the repair. `webp-animation` v0.10.0
  (MIT OR Apache-2.0) is verified and filed, and DEC-091 places it on the
  workhorse side of the fence — but it is not this stage.
- A 16-bit-throughout pipeline as a goal in itself. Decode, `Identity`,
  `AutoOrient` and the default encode path already preserve the variant; only
  the three op bodies collapse it. **No type change is needed anywhere.**
- Any new user-facing flag.
- `crustyimg-lab`. Per DEC-091 and the D-A entry: a defect in crustyimg cannot
  be fixed in a different binary.

## Spec Backlog

Ordered. D-A first because it is small, urgent, independent of the others, and
carries no lockfile cost.

- [ ] **SPEC-119** (design 2026-08-15) — [M] animated input is never silently flattened: warn on
      the pixel path instead of discarding frames, and stop `lint` recommending
      the destructive command. **The sweep ran at design and the rule name WAS
      too narrow** — GIF, APNG and animated WebP all flatten silently, so this is
      three formats, not one. Re-estimated S → M for that reason.
- [ ] (not yet written) — [M] ops preserve colour type and bit depth: widen to
      work, narrow to write. Fixes D-B and D-C in one change across the three op
      bodies. Lockfile-invalidating.
- [ ] (not yet written) — [S] measure D-D's premise: does SSIMULACRA2 score
      linear-light output better than current output on a representative
      downscale? **Gates the next spec** — if it does not, close D-D rather than
      spec it.
- [ ] (not yet written) — [M] resize in linear light with premultiplied alpha,
      conditional on the measurement above. Lockfile-invalidating; sequence with
      the colour-type spec so the migration is paid once.

**Count:** 0 shipped / 1 in flight (SPEC-119, design) / 3 pending

## Design Notes

- **"Widen to work, narrow to write"** is the shape of the D-B/C fix. An op
  preserves the input's colour type and bit depth unless it genuinely needs more
  (compositing a translucent overlay). `Watermark` is the arguable case —
  compositing genuinely needs alpha — and should be decided explicitly rather
  than swept in.
- **Measure before fixing D-D.** The backlog entry sets its own falsification
  gate and it should be honoured: if SSIMULACRA2 does not prefer the
  linear-light output, the premise is wrong and the entry closes. That is a
  separate, cheap spec on purpose — it is the difference between a measured
  change and a plausible one
  [[a-plausible-test-result-is-not-a-checked-one]].
- **The lockfile migration needs a DEC**, covering the cache-key question. One
  DEC for the combined byte-changing wave, not one per spec.
- **The backend is already in place.** `fast_image_resize` 5.x ships
  `U16x4`/`F32x4` and `MulDiv`; `DynamicImage` already has `ImageRgb16` /
  `ImageRgba16`. Neither fix needs a new dependency.
- ⚠ **"crustyimg is 8-bit internally" is not accurate as stated** and appears in
  places that outlive this stage. Correcting that claim wherever it is written is
  part of the colour-type spec, not a follow-up.

## Dependencies

### Depends on
- Nothing. All four defects are in shipped code and reproduce on `main`.
- STAGE-042's conformance matrix (SPEC-118) would likely have caught D-B and D-C
  mechanically. It is **not** a blocker — but if SPEC-118 is built first, its
  matrix should be checked against this stage's findings as a live test of
  whether the instrument works.

### Enables
- **STAGE-041** (launch content). The maintainer's sequencing call: the public
  quality-per-byte claim should be true before it is published.
- Animated WebP/AVIF encode — the linter's advice cannot become correct until
  the destructive path is closed.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?**
- **How many specs did it actually take?**
- **What changed between starting and shipping?**
- **Lessons that should update AGENTS.md, templates, or constraints?**
- **Should any spec-level reflections be promoted to stage-level lessons?**
