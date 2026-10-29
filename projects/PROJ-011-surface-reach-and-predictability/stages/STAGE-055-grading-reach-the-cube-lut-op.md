---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-055                     # stable, zero-padded within the project
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-011                      # parent project
repo:
  id: crustyimg

created_at: 2026-10-28
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    The surface-reach half of the thesis: a recipe can express a colour grade,
    which is the one operation STAGE-050's asset seam was explicitly built to
    carry and the only one it never got.
  delivers:
    - "A `lut` operation, on the CLI and in a recipe, taking a `.cube` file and a strength"
    - "A grade applied in linear light at the source's own bit depth, not on 8-bit sRGB"
    - "The `.cube` file's CONTENT in the build cache key, so changing a grade invalidates exactly the affected outputs and `build --check` catches an accidental grade change in review"
  explicitly_does_not:
    - "Interpret ICC profiles — the transfer function stays assumed-sRGB (DEC-095)"
    - "Change crustyimg's working buffer type, or add a float working path"
    - "Add a dependency on `irradiance`, or any other crate, to do it"
    - "Author or edit LUTs — this applies a file someone else made"
---

# STAGE-055: Grading Reach — the `.cube` LUT op

## What This Stage Is

When this stage ships, a recipe can carry a colour grade. `crustyimg apply
--recipe look.toml photo.jpg` applies a `.cube` LUT at a declared strength, in
linear light, preserving the input's bit depth; `crustyimg build` treats the
`.cube` file as a hashed input, so changing the grade rebuilds exactly the
affected outputs and `build --check` fails on an unintended one.

The operation itself is small. What makes it worth a stage is that it is the
missing consumer of four pieces of infrastructure that already shipped, and
that the reproducible-build framing — not the grade — is the differentiator.

## Why Now

**Because the infrastructure was built for it and it never arrived.** Four
shipped specs each named the `.cube` LUT as their second consumer:

| Shipped | What it already gives this stage |
|---|---|
| SPEC-128 / DEC-100 | `register_with_assets` + `asset_keys` + the `OperationParams` resolved-bytes side channel + `resolve_recipe_assets` at the recipe IO boundary. DEC-100 states the seam was built "generalized enough for a SECOND parameter-rich, asset-bearing op (the `.cube` LUT, a decided but unscheduled STAGE-050 backlog item) to register through it with no further change to `registry.rs`" — and proves it with `tests/registry_seam.rs`'s `a_lut_shaped_op_is_strict_and_resolvable_through_one_table()`, a fixture op shaped as "a file to read plus a 0..=1 strength". |
| SPEC-129 / DEC-101 | Asset **content** in the cache key. `cache::absorb` is `pub(crate)` precisely so a resolved-asset fingerprint composes into `target_recipe_hash` with the same tag+length discipline. |
| SPEC-122 / DEC-095 | `srgb_to_linear` / `linear_to_srgb` / `encode_srgb8` / `encode_srgb16`, in the module this op will live in, with the round trip **measured exact for all 256 8-bit and all 65,536 16-bit values**. |
| SPEC-121 / DEC-095 | Every op preserves the input's colour type and bit depth, so a 16-bit source reaches the grade at 16 bits. |
| SPEC-132 | `build --watch` already watches recipe asset paths, so a `.cube` edit triggers a rebuild for free. |

**Because its stated blocker has cleared.** `docs/backlog.md`'s entry carried
"⚠ Blocked on the colour-space question below — a `.cube` applied in a
non-linear pipeline is baked against the wrong transfer function." Amended
2026-08-18 to make the 8-bit half conditional on an 8-bit source (SPEC-121),
that left the transfer-function half live. SPEC-122 then shipped the linear
machinery. Both halves are now answered: a grade applied through the same
linearize → transform → re-encode round trip `Resize` uses is evaluated in the
right space.

**Because it was homed nowhere and nothing could see it.** The entry sits under
`docs/backlog.md`'s own heading "Open — workhorse items with no home", and
`just backlog` reads stage files rather than that document, so no command in
this repo could surface it. Three separate records call it decided; none
scheduled it. This stage is that fix.

## Success Criteria

- A recipe carrying a `lut` step applies a real `.cube` file and produces a
  visibly graded image; the same grade is reachable from the CLI verb
- A 16-bit input is graded at 16 bits, in linear light, and narrows back per
  DEC-095's rules — not flattened to 8-bit
- Editing the `.cube` file changes the cache key and rebuilds exactly the
  outputs that used it; `build --check` fails on an unintended grade change
- A malformed, truncated or hostile `.cube` is a typed error at exit 1, never a
  panic and never a silently wrong grade
- `strength = 0.0` is a byte-exact no-op, and `strength = 1.0` is the full grade
- No new top-level dependency, and `--no-default-features` still builds
- The op registers through the existing seam with **no change to
  `registry.rs`** — the claim DEC-100 made and tested with a fixture, now
  discharged against the real op

## Scope

### In scope
- A `.cube` parser, in-house: `LUT_3D_SIZE` and `LUT_1D_SIZE`, `DOMAIN_MIN` /
  `DOMAIN_MAX`, `TITLE`, comments, both orderings, with typed errors
- 3D interpolation (trilinear or tetrahedral — see Design Notes) and the 1D path
- A `Lut` operation implementing `Operation`, registered via
  `register_with_assets` with the `.cube` path as its asset key
- A `strength` parameter in `0.0..=1.0`, blended in linear
- Application in linear light at the input's own bit depth, reusing the
  SPEC-122 helpers
- The resolved `.cube` bytes folded into the build cache key through SPEC-129's
  fingerprint path
- Fuzzing the parser, on the existing `fuzz/` harness pattern

### Explicitly out of scope
- **ICC-aware conversion.** The transfer function stays assumed-sRGB, as
  DEC-095 made explicit. A `.cube` authored for a log or Rec.2020 source is
  applied under an assumption that is wrong for it; say so in the docs rather
  than pretending otherwise.
- **Changing the working buffer type.** See Design Notes — the `F32x4`-versus-
  `U16x4` question is open and inherited, not opened here.
- **A float working path end to end.** `Rgb32F` / `Rgba32F` sources still take
  the 8-bit route (`src/operation/mod.rs`'s `is_sixteen_bit` comment). That is
  its own project.
- **`irradiance`.** The LUT engine stays crustyimg's. Depending on a RAW
  develop library to grade a JPEG would collide with the blocking
  `single-image-library` constraint, and for an 8-bit source would buy nothing
  — the quantization is in the source, not the call site. See Design Notes.
- **The develop tone curve.** A curve applied to 14-bit linear sensor data
  before an output depth is chosen belongs to `irradiance`'s own develop stage,
  not here.
- **LUT authoring, curve editing, or a preview UI.** Per DEC-091's Fence B,
  authoring is lab.
- **The wasm surface.** The op compiles engine-side, but `wasm::transform`
  cannot resolve a path (DEC-100, AC-6), so a browser grade needs bytes handed
  in through a different entry point. A follow-on, not this stage.

## Spec Backlog

Ordered.

- [ ] **SPEC-136** (design) — [M] A recipe can express a `.cube` LUT: parse,
  interpolate, apply in linear at the source's depth, and hash the file's
  content into the build cache key.

**Count:** 0 shipped / 1 active / 0 pending

> The bold spec id above is load-bearing, not styling: `scripts/_lib.sh`'s
> `extract_unpromoted_bullets` keys on structure, and an open bullet that does
> **not** lead with `**SPEC-NNN**` is counted as un-promoted backlog. Written
> unbolded, this spec is reported twice — once in flight and once as not-yet-
> specced.

Candidates deliberately left unwritten until SPEC-136 measures:

- **A working-type change** (`F32x4` → `U16x4` on the grading path), only if
  SPEC-136's benchmark shows the linear round trip costs more than the grade.
  Its own spec, because it would move `Resize` too.
- **A wasm LUT entry point**, if the recipe playground (roadmap Wave 6) wants a
  grade in the browser.

## Design Notes

**Interpolation: trilinear or tetrahedral.** Trilinear is 8 taps and the
obvious first implementation; tetrahedral is 4 taps, closer to what colour
tools actually ship, and avoids trilinear's characteristic error on the cube
diagonals — which is where neutral greys live, so the artifact lands exactly
where it is most visible. Pick one in design with a measured comparison on a
real `.cube`, and record which and why. Do not ship both.

**The working type is an inherited open question, not this stage's to settle.**
SPEC-122's own benchmark found `resize` went 169.27 µs → 648.52 µs (+283.5%,
Apple M4 Pro, criterion, 100 measurements/arm) and decomposed it: the same
`F32x4` pipeline with **no transfer function at all** still cost 515.88 µs, so
**72% of the added time is the `F32x4` working type, not the maths**. The
backlog records that "a faster path means changing the working type (`U16x4`
has SIMD kernels `F32x4` does not), which is a design call, not a build one."
A LUT is more arithmetic per pixel than a resample, so SPEC-136 should
**measure and report** rather than assume, and leave the working-type change to
its own spec if the number is bad.

**Why the engine is crustyimg's and not `irradiance`'s.** Reviewed 2026-10-28
against irradiance's proposed DEC-028, which asked for the engine to live there.
Four reasons it stays here: (1) `single-image-library` is **blocking** and
forbids a second pixel-processing library — `irradiance` as a *decoder* is
precedented (`re_rav1d`, `libheif-rs`), as a *transformer of pixels crustyimg
already holds* it is not; (2) the recorded integration seam is bytes-in /
pixels-out one way only, and `irradiance` has no `image` dependency by design,
so every grade would cost two buffer conversions; (3) the wasm build is
`--no-default-features --features avif`, so a feature-gated engine could never
reach the browser; (4) the shared kernel is ~40 lines of grid interpolation —
a DCP `HueSatMap` is indexed in HSV and returns hue/sat/value scales, a `.cube`
is indexed in RGB and returns RGB, and `irradiance` needs no `.cube` parser at
all. A third crate for 40 lines would add a top-level dependency to both repos.
Precedent for in-housing at this size is `little_exif` → `src/metadata/tiff.rs`
at 718 lines.

**Budget.** `docs/backlog.md` measured the closest real comparable —
`lut-cube` 0.2.0 — at 329 lines (`lib.rs` 53 + `cube.rs` 59 + `lut.rs` 217) and
budgets **250–350 lines** for parse + interpolate + typed errors. The crate is
plain MIT (`license-file`, which is why crates.io reports "non-standard"), and
the better independent reason to decline it is that its only tests hardcode
absolute paths into the author's local DaVinci Resolve install — a
supply-chain quality signal, not a licence problem.

**`strength` is blended in linear, not in sRGB.** Blending a graded and
ungraded pixel is a weighted average, and the same argument that put `Resize`
in linear light applies: averaging non-linear samples is arithmetic on values
not proportional to the physical quantity. `strength = 0.0` must be byte-exact,
which is a test, not an aspiration.

## Dependencies

### Depends on
- **STAGE-050** (shipped) — the asset-op registry seam (SPEC-128/DEC-100) and
  asset-content cache hashing (SPEC-129/DEC-101). This stage is its
  continuation; STAGE-050 is closed in place and is not reopened.
- **STAGE-046 / SPEC-121 + SPEC-122** (shipped, DEC-095) — depth preservation
  and the linear-light helpers
- **SPEC-131** (shipped) — typed per-operation parameter structs, which the
  `lut` params will use
- No external dependency. That is a success criterion, not an accident.

### Enables
- **Brand consistency as a build gate** — the one commercial angle in
  `docs/feature-set-triage-2026-08.md`'s set, and the reason the reproducible
  build rather than the grade is the framing
- **Roadmap Wave 7** — the grading surface that wave names
- A **wasm grade** in the recipe playground (Wave 6), once an entry point
  exists that can hand LUT bytes in

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
