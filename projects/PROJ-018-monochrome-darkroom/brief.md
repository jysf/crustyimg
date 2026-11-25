---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-018                      # stable, zero-padded, never reused
  status: proposed                  # proposed | active | shipped | cancelled
  priority: medium                  # critical | high | medium | low
  target_ship: null                 # optional: YYYY-MM-DD

repo:
  id: crustyimg                     # must match .repo-context.yaml

created_at: 2026-11-02
shipped_at: null

# Business value. Testable claim, not marketing copy.
value:
  thesis: >
    A developed monochrome RAW can be taken to a finished 16-bit print master
    by a darkroom model that is expressed as recipe steps and compiled to a
    deterministic 65,536-entry u16 lookup table, so any renderer that applies
    that table gets the same bits as the CLI. The deliverable is the operators
    and the table compiler, with each paper and toner shipped as data; a
    "look" is a profile, not code, and is accepted on measurable properties
    rather than by eye.
  beneficiaries:
    - "Anyone developing monochrome DNGs through crustyimg who wants a grade that is a reviewable, re-runnable recipe rather than a one-off edit"
    - "Library consumers that need the grading operator as a table they can apply on another backend and verify against the CLI"
    - "Anyone sharing a developed image who wants its sensor facts (geometry, noise floor, zone distribution) in one rendered plate that `info` already agrees with"
    - "PROJ-015's grading surface, which gains a second real consumer for the exposure, curve and LUT machinery it builds"
  success_signals:
    - "A recipe with enlarger, paper and toner steps turns a Q2 Monochrom DNG into a 16-bit print master, and re-running it produces byte-identical output"
    - "The compiled 65,536-entry u16 table, applied by an independent applier written for the test (not the generator), reproduces the CLI's output pixel for pixel"
    - "The Zone Anchor Solver round-trips: applying the returned EV shift places the sampled value in the requested zone, under a zone definition written down before the solver is built"
    - "Every shipped paper profile names its source (a measured step wedge, or a published curve) or is labelled modelled; none is called calibrated without the measurement"
    - "The sensor card's figures are read from the same report `info` prints, and a test asserts the two agree"
    - "No existing recipe's output bytes change: every operator is new, and the recipe-output guard (SPEC-141) stays green"
  risks_to_thesis:
    - "⚠ A paper curve invites tune-until-it-looks-right, which is the unbounded loop `docs/roadmap.md` already keeps smart-crop and auto-colour behind a beta flag for. The defence is acceptance by sensitometric properties (monotone, endpoints at Dmin and Dmax, shoulder and toe anchors that stay put as grade moves, the slope relation between grades), written into each spec before any profile is tuned"
    - "⚠ 'Calibrated' is a falsifiable claim and the repo holds no step-wedge scans. Naming profiles after real papers (Ilford, Agfa, Kodak) is both that claim and a trademark use, and 'Wetzlar' in a profile or card name implies a Leica affiliation. Names and wording need a maintainer ruling before any spec ships them"
    - "⚠ Overlap with PROJ-015: EV exposure, curves and the `.cube` seam are its subject. Building a second exposure and curve implementation here would split the working-type question PROJ-015 exists to answer. This project consumes PROJ-015's operators or waits for them; it does not fork them. Ruled 2026-11-02: PROJ-015 owns EV exposure and the generic curve operator; this project adds the Zone Anchor Solver, edge burn, paper curve and toning on top"
    - "⚠ The largest proposed surface after animated AVIF, and the roadmap's own discipline is that when building is cheap the scarce act is saying no. It sits post-1.0 and not gated, and the first stage is the cheap test of whether the rest earns its place"
    - "16-bit output is not uniformly available: AV1 tops out at 12 bits, so a 16-bit `.avif` print master cannot exist, and STAGE-056 already records that `L16` through the AVIF and WebP sinks is unverified. The export stage has to measure each sink rather than assume"
    - "Bit-identity between the CLI and a GPU preview is not something this repo can prove. What it can prove is a deterministic table plus test vectors; the other side owns the rest"
---

# PROJ-018: Monochrome darkroom — enlarger, paper, toner, print master

## What This Project Is

A grading and export pipeline for developed monochrome RAW, built on the
scene-linear `u16` plane that `irradiance` emits behind `raw-develop`
(PROJ-016). Four operators and two renderers, all reachable from a recipe:
a linear enlarger (EV shift, a Zone Anchor Solver, a radial edge burn), a
split-grade paper curve with shipped profiles, a density-mapped toner
(`u16` grey to `3×u16` RGB), and a soft-proof operator; then archival
exports and a rendered sensor card. The whole grading chain compiles to one
deterministic 65,536-entry `u16` table, which is also what the `.cube`
export writes.

## Why Now

Not yet. This project is **proposed**, not scheduled: PROJ-011 is active,
PROJ-016 has not shipped (and is gated on `irradiance` 0.1.0 reaching
crates.io), and PROJ-015 owns the exposure and curve machinery this reuses.
It is framed now so the dependency order is written down while the design is
fresh, and so the overlap with PROJ-015 is decided deliberately rather than
discovered in a build.

## Success Criteria

See `value.success_signals`.

## Scope

### In scope
- Linear enlarger: EV shift (−5.0..=+5.0), Zone Anchor Solver, radial edge burn applied in scene-linear space before the paper curve
- Split-grade H&D paper curve (yellow 00–2.5 shapes the shoulder; magenta 2.5–5.0 shapes the toe and Dmax) and five shipped profiles as data
- Density-mapped toning: paper-base tint, a primary shadow toner, a secondary split toner with a configurable crossover zone
- Compiling the chain to a 65,536-entry `u16` table; 1D and 3D `.cube` export of any configured state
- 16-bit grey and RGB `.tiff` and `.png` print masters with embedded ICC profiles; a linear `u16` TIFF of the uncropped, un-warped plane; a soft-proof operator for glossy and matte Dmax
- A sensor card renderer fed by the same report `info` prints
- Every operator expressed as a recipe step with a key table that refuses bad parameters at load (DEC-102)

### Explicitly out of scope
- Any app, GPU or Metal code. This repo delivers the table and the vectors
- A new sidecar format. The handoff asked for a `.irr` file; the recipe is already a versioned, diffable TOML pipeline, the roadmap says not to invent new syntax, and row 6b already designs a recipe sidecar. The darkroom state is recipe steps
- Colour RAW, and writing DNG
- A 16-bit `.avif`. AV1 does not carry 16 bits; the export stage decides what AVIF may offer (10 or 12 bit) after measuring it
- New CLI verbs. The surface was frozen at 14 by STAGE-030, so exports ride `web`, `convert`, `edit` and `info` or a new verb needs its own ruling
- Auto-tone, auto-zone placement or any judgment-bound behaviour

## Stage Plan

Provisional. Stage ids are assigned at framing, and the order follows dependency.

- [ ] (not yet defined) — **Linear enlarger and the table compiler.** EV shift, Zone Anchor Solver (zone definition first), edge burn, the 65,536-entry `u16` table and its determinism test. The cheap first stage: it needs no paper data
- [ ] (not yet defined) — **Split-grade paper curve and profiles.** The H&D transfer function, the grade-to-slope relation, five profiles with sources. Blocked on the naming and calibration rulings above
- [ ] (not yet defined) — **Toning and `.cube` export.** Paper-base tint, shadow and split toners, crossover zone, 1D and 3D `.cube` writers checked by an independent reader
- [ ] (not yet defined) — **Print masters and soft-proofing.** 16-bit TIFF and PNG with ICC, the linear TIFF, the soft-proof operator, a measured answer for each sink's bit depth
- [ ] (not yet defined) — **The sensor card.** A composite plate (the font stack already in tree for SVG rendering is the likely route) whose figures are asserted against `info`

**Count:** 0 shipped / 0 active / 5 pending

## Dependencies

### Depends on
- **PROJ-016 / STAGE-056 and STAGE-062** — the developed `u16` plane, truthful geometry, and the sensor facts the card prints
- **PROJ-015** — exposure and curve operators, the working-type answer, and STAGE-055's `.cube` asset seam. Ruled 2026-11-02: PROJ-015 owns EV exposure and the generic curve operator, so PROJ-015 needs a stage for them (it has only STAGE-055 today) and the first stage here starts once that EV operator exists
- **External: `irradiance` 0.2** — the warp field sampler, preview slice accessor and ADU histogram named in the handoff. None of those names has been checked against irradiance's source from this repo
- **Maintainer rulings:** profile names and the word "calibrated"; the zone definition

### Enables
- A second, independent consumer for PROJ-015's grading surface
- Geometry (PROJ-017) and the darkroom chain composing in one recipe

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <yes/no + notes>
- **How many stages did it actually take?** <number, compare to plan>
- **What changed between starting and shipping?** <one or two sentences>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **What did we defer to the next project?**
  - <one-line items>
