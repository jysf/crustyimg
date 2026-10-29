---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-015                      # stable, zero-padded, never reused
  status: proposed                  # proposed | active | shipped | cancelled
  priority: medium                  # critical | high | medium | low
  target_ship: null                 # optional: YYYY-MM-DD

repo:
  id: crustyimg                     # must match .repo-context.yaml

created_at: 2026-10-28
shipped_at: null

# Business value. Testable claim, not marketing copy.
value:
  thesis: >
    crustyimg's pixel pipeline is correct about the space and the depth it
    works in, and on that correctness it exposes a grading surface — a LUT,
    then curves and exposure — that is only honest if the pipeline underneath
    it is. The grading surface is the deliverable; the pipeline work is what
    makes it defensible rather than a slider that bands.
  beneficiaries:
    - "Anyone applying a look across a batch — the grade is expressed in a recipe, versioned, and hashed into the build"
    - "Teams with brand colour as a requirement — `build --check` turns an accidental grade change into a failing review"
    - "Every existing verb, because the depth and working-type work under the grading surface improves `resize` and `watermark` too"
    - "The maintainer's own photo workflow, which is where a grade is wanted and where 8-bit banding is visible"
  success_signals:
    - "A recipe carrying a `.cube` LUT produces a graded image, and editing the `.cube` rebuilds exactly the outputs that used it"
    - "`build --check` fails on an unintended grade change, naming the target"
    - "A 16-bit source is graded at 16 bits in linear light, with the result checked against an independently computed expectation"
    - "The working-type question is answered with a measurement rather than carried as a design note for another two months"
    - "A `Rgb32F` input stops silently taking the 8-bit path, or the reason it still does is written down"
  risks_to_thesis:
    - "⚠ The grading surface alone is a thin project. Its value depends on the pipeline stages landing too, and those are the unglamorous half — if only STAGE-055 ships, this is a one-spec project wearing a project's clothes"
    - "⚠ Every stage here changes output bytes, so each invalidates PROJ-007 build lockfiles and needs its own migration story. DEC-095 already paid this cost once; this project pays it up to four more times"
    - "The working-type change (`F32x4` → `U16x4`) would move `Resize` as well as the grading path, so it is a shared-code change with a blast radius beyond this project — and a stale incremental build is a false green on exactly that shape of change"
    - "Curves and exposure are judgment-bound in a way the LUT is not: a LUT applies a file someone else authored, while a curve surface invites tuning-until-it-looks-right, which is an unbounded loop"
---

# PROJ-015: Colour and Grading

## What This Project Is

The wave that makes crustyimg's colour pipeline honest about what space and
depth it is working in, and then puts a grading surface on top of it.

It starts with the `.cube` LUT — already specced, and the missing consumer of
four shipped infrastructure specs — and continues into the three pipeline
questions the record has been deferring with the phrase "its own project":
the working type, the float path, and the grading ops beyond a LUT.

## Why Now

**Because the LUT is ready and had nowhere to live.** `STAGE-055` and
`SPEC-136` were framed on 2026-10-28 (PR #212) under `PROJ-011`, which was the
wrong home: PROJ-011 is *surface reach and predictability*, it is `active` and
working toward a close, and a post-1.0 grading feature bolted onto it means
PROJ-011 cannot ship until the LUT does. Moving it here is the correction.

**Because three pipeline questions have been answered with "its own project"
and then left unowned.** `DEC-095` says of ICC-aware conversion that it "is its
own project and is explicitly not this one"; `src/operation/mod.rs`'s
`is_sixteen_bit` says of float sources that "widening it further is out of
scope"; and `docs/backlog.md` says of the working type that "a faster path means
changing the working type (`U16x4` has SIMD kernels `F32x4` does not), which is
a design call, not a build one." Each is a correct deferral. None of them named
where to. This project is that address for two of the three — ICC stays with
PROJ-012, see Scope.

**Because the LUT's value is contingent on them.** A grade applied through a
`F32x4` round trip that costs +283.5% is a grade people will turn off, and a
grade on an 8-bit source bands regardless of how carefully it is computed. The
grading surface is what users ask for; the pipeline work is what makes it worth
shipping.

## Success Criteria

- A `.cube` LUT is expressible in a recipe and reachable from the CLI, applied
  in linear light at the source's own bit depth
- The `.cube` file's content participates in the build cache key, so a grade
  change invalidates exactly the affected outputs and `build --check` catches
  an unintended one
- The working-type question is **closed with a benchmark**, in whichever
  direction — `F32x4` kept with the cost accepted and written down, or `U16x4`
  adopted with the SIMD win measured
- Float sources either stop taking the 8-bit path, or the record says plainly
  why they still do and what it would take
- No new top-level dependency across the whole project
- Each stage that changes output bytes ships with its lockfile-migration story,
  not after it

## Scope

### In scope
- The `.cube` LUT op: parser, interpolation, linear application, cache key
  (`STAGE-055` / `SPEC-136`)
- The working type on the pixel path: `F32x4` versus `U16x4`, decided by
  measurement
- The float working path — whether `Rgb32F` / `Rgba32F` get a real working
  buffer instead of the 8-bit route
- Grading ops beyond a LUT: curves and exposure, which `docs/backlog.md` names
  alongside the LUT as sharing the same correctness argument
- The lockfile-migration story for each byte-changing stage

### Explicitly out of scope
- **ICC colour transforms.** `PROJ-012` owns these — its stage plan carries
  "ICC colour transforms (`qcms`). Closes `color/wrong-colorspace`", and its
  risks record that `qcms` is the only permissible implementation because
  `lcms2` bindings would breach DEC-088's zero-system-dependency property.
  DEC-095's "ICC-aware conversion is its own project" and PROJ-012's lint-fix
  framing are **not quite the same work** — PROJ-012 closes a named rule, while
  the pipeline-side question is whether ops interpret the transfer function at
  all — but they share the one dependency decision and the one implementation,
  so they share an owner. If the pipeline side needs its own stage, it rides
  PROJ-012's `qcms` work rather than duplicating it. **Do not take ICC into
  this project without first re-homing it out of PROJ-012.**
- **Authoring** — LUT editing, curve drawing, a preview UI. DEC-091's Fence B
  puts authoring in lab.
- **`irradiance`.** The LUT engine is crustyimg's; the develop tone curve on
  linear sensor data is irradiance's. Settled 2026-10-28 — see `STAGE-055`'s
  Design Notes for the four reasons, the load-bearing one being that
  `single-image-library` is a blocking constraint.
- **Smart/auto colour** — auto-white-balance, auto-tone. Judgment-bound, and
  the roadmap already places it post-1.0 behind a beta flag.
- **Competing with a grading application.** This applies looks reproducibly in
  a build; it is not a colourist's tool.

## Stage Plan

Ordered by dependency, **not by priority** — and stage two carries an explicit
exception to that ordering, for the reason given under it.

- [ ] STAGE-055 (proposed) — Grading reach: the `.cube` LUT op (`SPEC-136`, [M], `cycle: design`)
- [ ] (not yet defined) — **The working type, closed by measurement.** `F32x4`
  versus `U16x4` on the pixel path. The number already exists and is damning:
  `resize` went 169.27 µs → 648.52 µs (+283.5%), and the same `F32x4` pipeline
  with **no transfer function at all** still cost 515.88 µs, so **72% of the
  cost is the working type, not the maths**. ⚠ Shared-code change — it moves
  `Resize` and `Watermark`, not just the grading path, so it needs a clean
  full-matrix verify rather than an incremental one.

  ⚠⚠ **This stage's position is NOT the project's position — decide it
  separately.** PROJ-015 sits post-1.0 (roadmap Wave 7) because a *grade* is
  not table-stakes for "a complete local toolchain." That argument does not
  cover this stage, which is not a grading feature at all: it is a **shipped
  performance regression in the most-used verb**. `resize` is **1.51–2.45×
  slower end-to-end in 0.8.0** than before SPEC-122, and users have it today.
  `docs/roadmap.md` put the resize *quality* defect above the 1.0 line on the
  reasoning that resize is "the most-used operation of a tool whose headline
  claim is quality-per-byte" — that reasoning applies unchanged to the fix for
  it having tripled the cost. **Pulling this stage forward does not require
  pulling the LUT forward with it**, and the two should not be sequenced as a
  unit just because they share a project.
- [ ] (not yet defined) — **The float working path.** Whether `Rgb32F` /
  `Rgba32F` get a real working buffer. Today `is_sixteen_bit` answers `false`
  for them deliberately, so a float source silently takes the 8-bit route —
  which is a quiet data loss on an input the crate accepts.
- [ ] (not yet defined) — **Curves and exposure.** The grading ops
  `docs/backlog.md` names beside the LUT. ⚠ Scope this one narrowly or it
  becomes the tuning loop its own risk entry warns about.

**Count:** 0 shipped / 0 active / 4 pending

## Dependencies

### Depends on
- **PROJ-011 / STAGE-050** (shipped) — the asset-op registry seam
  (SPEC-128/DEC-100) and asset-content cache hashing (SPEC-129/DEC-101).
  `STAGE-055` is that stage's continuation across a project boundary.
- **STAGE-046 / SPEC-121 + SPEC-122** (shipped, DEC-095) — depth preservation
  and the linear-light helpers in `src/operation/mod.rs`
- **PROJ-007** (shipped) — the build cache and lockfile this project repeatedly
  invalidates
- **PROJ-012**, for ICC, if the pipeline-side transfer-function question is
  ever pulled
- No external dependency. That is a success criterion, not an accident.

### Enables
- **Brand consistency as a build gate** — the one commercial angle in
  `docs/feature-set-triage-2026-08.md`'s set, and the reason the reproducible
  build rather than the grade is the framing
- A **grade in the browser**, once the recipe playground (roadmap Wave 6) has
  an entry point that can hand LUT bytes to wasm
- Closing a gap against a paid competitor with a permissive implementation —
  imazen's `zenresize` ships gamma-correct scaling as
  `AGPL-3.0-only OR LicenseRef-Imazen-Commercial`

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <yes/no + notes>
- **How many stages did it actually take?** <number, compare to plan>
- **What changed between starting and shipping?** <one or two sentences>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **What did we defer to the next project?**
  - <one-line items>
