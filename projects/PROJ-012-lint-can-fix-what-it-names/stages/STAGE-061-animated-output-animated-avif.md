---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-061
  status: proposed
  priority: high
  target_complete: null

project:
  id: PROJ-012
repo:
  id: crustyimg

created_at: 2026-10-30
shipped_at: null

value_contribution:
  advances: >
    The animated half of PROJ-012's thesis: `format/animated-gif` and
    `format/animated-input` name a defect crustyimg can now fix, by writing an
    animated AVIF that keeps every frame instead of flattening to frame 1.
  delivers:
    - "An animated AVIF writer: every frame, in order, with per-frame timing and loop count, plus a still poster frame for browsers that cannot animate"
    - "Quality searched per sequence by the existing perceptual search, at a speed pinned for animation"
    - "`lint`'s animated rules naming a command that preserves every frame"
  explicitly_does_not:
    - "Animated WebP. No pure-Rust encoder exists"
    - "Alpha in animated output. Refused loudly in v1, deferred to a follow-on"
    - "Video containers or any codec not already shipped (DEC-088)"
---

# STAGE-061: Animated output: animated AVIF

## What This Stage Is

When this stage ships, `crustyimg convert anim.gif -o anim.avif` writes an animated AVIF
that keeps every frame, in order, at its own duration, looping as the GIF did. `lint`'s two
animated rules point at that command instead of one that flattens the animation.

## Why Now

The blocker is gone: **DEC-108 (2026-10-30) adopts `mp4-atom`** on four conditions the first
design session verifies. The research is already measured
(`docs/research/draft-spec-animated-avif-output.md`, `docs/probes/animated-gif-to-av1/`):
11.2× smaller than GIF and 6.3× smaller than animated WebP, at higher quality. The draft is
complexity L with its own note that L means split it. This stage is that split.

## Success Criteria

- A GIF with N frames produces an AVIF that **`re_rav1d` decodes to N frames in the same
  order**, asserted structurally before any pixel or score assertion. A pipeline that dropped
  3 of 8 frames scored SSIMULACRA2 100.0, so a score alone proves nothing here.
- Unequal per-frame delays and the loop count round-trip.
- The output carries a still primary item (`meta`) and a sequence (`moov`), with brand `avis`
  and `avif`/`mif1` compatible, so a browser without sequence support shows a poster frame.
- A static input's output is byte-identical to `main`'s.
- `lint` on an animated GIF prints a fix that, run verbatim, preserves every frame.

## Scope

### In scope
- The muxing driver on `mp4-atom` boxes, the frame loop, timing and loop count
- Routing a multi-frame input to the animated writer when the output is `.avif`
- The quality search and speed pinning for sequences
- The `lint` fix text for both animated rules
- Alpha refusal

### Explicitly out of scope
- Animated WebP, alpha sequences, any audio or video container
- The ICC half and the roster test (PROJ-012's other stages)

## Spec Backlog

In build order. Each is framed in its own session first.

- [ ] (not yet written) — [M] **The muxer and the frame/timing round trip.** The draft's
  AC-1 to AC-6 and AC-9 to AC-12: every frame, in order (by an independent decoder), unequal
  `stts` durations from a deliberately unequal fixture, loop count, the still-plus-sequence
  box layout, alpha refused (warn or exit 4, never silently opaque), static input
  byte-identical, a lean build exiting 4. Uses a fixed quantizer, since quality search is the
  next spec. ⚠ **Its design session runs DEC-108's four Validation checks first** and amends
  DEC-108 with the results (or with the fall back to an in-house box writer) before writing a
  failing test. ⚠ Budget the driver at about 1,000 lines, measured. ⚠ Run a near-lossless
  colour-range control first: a score insensitive to the quantizer is a range/matrix bug, not
  a codec problem (57.2 → 96.5 once `Range: Limited` was fixed).
- [ ] (not yet written) — [S] **Quality search and speed pinning for sequences.** The
  draft's AC-7 and AC-8. The existing perceptual search drives the quantizer per sequence (two
  real sources scored 86.7 and 76.1 at one fixed quantizer, so a pinned value cannot serve
  both). Speed is pinned at 6 for sequences, with a test that fails if the still-image or wasm
  speed knob reaches this path. ⚠ Speed 10 made a 37% *larger* file at lower quality on
  animation, which is the opposite of DEC-068's still-image finding.
- [ ] (not yet written) — [S] **`lint` names the fix.** `format/animated-gif` and
  `format/animated-input` print a command that writes an animated AVIF. Covered by SPEC-139's
  run-every-printed-fix-verbatim test once that lands; if it hasn't, this spec adds the row.
  The fix must preserve every frame, asserted with the same independent-decoder check.

**Count:** 0 shipped / 0 active / 3 pending

## Design Notes

- **Browser support is a moving fact.** caniuse's still-AVIF figure doesn't break out
  sequences. Re-check before any published claim, and lean on the poster frame.
- **The wasm leg.** DEC-108's fourth condition decides whether this is native-only in v1. If
  so, the wasm `transform` refuses a multi-frame `.avif` output with a message, rather than
  flattening.

## Dependencies

### Depends on
- **DEC-108** and its Validation results
- `rav1e`, `re_rav1d` (in tree); the perceptual search in `src/quality/`
- SPEC-139 (the lint fix-text guard), preferably first

### Enables
- PROJ-012's thesis for the animated rules
- A later alpha-sequence follow-on

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
