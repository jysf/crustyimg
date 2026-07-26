---
# DRAFT — not committed. Lives in the session scratchpad.
# Project id is deliberately unassigned: this is core-pipeline work that predates
# any wave that needs it. Candidates: a new PROJ (core precision), or folded in
# ahead of roadmap wave 5 (Geometry, PROJ-006), which is its first real consumer.

stage:
  id: STAGE-XXX
  status: proposed
  priority: medium
  target_complete: null

project:
  id: PROJ-XXX
repo:
  id: crustyimg

created_at: 2026-07-25
shipped_at: null

value_contribution:
  advances: >
    The optimization thesis — "smallest file at a quality you can measure." Today
    every pixel op runs on gamma-encoded 8-bit sRGB, so the engine degrades the
    image it is trying to optimize, and its own SSIMULACRA2 score pays for it.
  delivers:
    - Downscaling in linear light — removes the well-known gamma-space darkening
      artifact on high-contrast detail, at no cost to the user
    - 10/12-bit AVIF and 16-bit PNG survive the pipeline instead of being
      quantized to 8-bit at decode
    - One pixel-format conversion per pipeline instead of one per operation
  explicitly_does_not:
    - Add tonal or color EDITING operations (exposure/contrast/curves/HSL) —
      those are a separate decision this stage deliberately does not pre-approve
    - Add ICC profile CONVERSION or a color-managed output transform
    - Touch RAW development, demosaic, or camera profiles
---

# STAGE-XXX: A linear, higher-precision pixel working buffer

## What This Stage Is

Today the pipeline is a bare fold in which **every** operation opens with
`to_rgba8()` (`src/operation/mod.rs:197, 396, 816`) and hands back a
`DynamicImage::ImageRgba8`. Three consequences, all measurable:

1. **Resize runs in gamma-encoded sRGB.** `fast_image_resize` is fed
   `PixelType::U8x4` with no linearization and no alpha premultiplication
   (`src/operation/mod.rs:511-533`; `MulDiv` appears nowhere in `src/`).
   Convolution over gamma-encoded values darkens high-contrast detail — a
   textbook artifact. DEC-008 never discusses light-linearity; a grep of that
   decision for `gamma|linear|premultipl|srgb` returns **0**.
2. **Bit depth is destroyed at decode.** 10/12-bit AVIF is quantized to 8-bit in
   `src/image/avif.rs:535` (`(v.clamp(0.0,1.0) * 255.0).round() as u8`),
   unrecoverably. A 16-bit PNG survives decode but is flattened by the first op.
3. **N ops = N unpack/repack round-trips.** The pipeline (`src/pipeline/mod.rs:51`)
   has no working buffer, and the `Operation` trait has no pixel-format or
   linearity hint (`src/operation/mod.rs:130-152`).

This stage introduces a **declared working representation** — a linear-light,
higher-precision buffer (f32 or u16; the choice is a spec, not a given) — plus a
capability hint on `Operation` so the pipeline converts **once at the boundary**
rather than per-op. It is a precision and correctness change to machinery that
already exists. It adds **no new user-visible operation.**

## Why Now

- **It is the only precondition that gets cheaper the earlier it lands.** Every
  operation added later inherits the 8-bit convention and has to be migrated.
  Roadmap wave 5 (Geometry — crop/rotate/flip/trim/pad) is the next wave that
  adds pixel ops; rotate by arbitrary angle resamples, so it wants linear light
  for exactly the reason resize does. Landing this first means wave 5 is written
  once.
- **It improves a number the project already publishes.** `BENCHMARKS.md` reports
  SSIMULACRA2, and the engine's auto-quality search optimizes against it
  (DEC-019). Downscaling in gamma space costs perceptual score the engine is
  currently paying silently. This is measurable before/after — see Success Criteria.
- **The AVIF truncation is a live defect on the default path** now that AVIF
  encode is in the default feature set (DEC-081).
- It is **not** blocked on any dependency question: a color-math crate is
  optional, and if wanted, `palette` (MIT/Apache) fits the carve-out already used
  for `ssimulacra2` and `resvg` ("a metric crate only… does not violate
  `single-image-library`", `Cargo.toml:72-74`) — needing a DEC per
  `no-new-top-level-deps-without-decision` (severity: warning).

## Success Criteria

- **A measured perceptual win on downscale.** On a fixture set with high-contrast
  detail, `resize` in linear light scores higher on SSIMULACRA2 than the current
  gamma-space path, reported as a before/after table. **If the win is not
  measurable, this stage should be cancelled** — that is the honest gate, and it
  should be the first spec, not the last.
- **A negative control proving the harness can show the other result:** deliberately
  re-run the linear path with linearization disabled and confirm the score drops
  back to the baseline. (Per [[a-plausible-test-result-is-not-a-checked-one]].)
- 10-bit AVIF and 16-bit PNG inputs round-trip through a resize without being
  quantized to 8 bits — asserted structurally on the buffer's type, not inferred
  from output file size.
- A pipeline of N ops performs **one** conversion in and one out, asserted
  mechanically (e.g. a counting test double), not by reading the code.
- Alpha is premultiplied before convolution and un-premultiplied after — with a
  fixture that has hard alpha edges over a contrasting colour, which is the case
  that fails visibly today.
- Every existing test stays green; the recipe round-trip is byte-stable; `just deny`
  green; the lean (`--no-default-features`) and wasm builds still build.

## Scope

### In scope
- A declared working representation and where the conversion boundary sits
- Linearization / de-linearization (sRGB transfer function) at that boundary
- Alpha premultiply / un-premultiply around resampling
- An `Operation` capability hint so the pipeline can batch the conversion
- Preserving >8-bit from decode through the pipeline to encode
- Migrating the four existing ops (`identity`, `invert`, `resize`, `auto-orient`)
- A DEC recording the representation choice and the boundary contract
  (this supplements DEC-002, which pins "exactly one in-memory image type")

### Explicitly out of scope
- **Tonal/color editing ops.** This stage makes them *cheaper to build later*; it
  does not authorize them. That call is governed by
  `docs/research/proj-002-findings.md:212` ("Draw the line at *automatic*") and
  should be argued on its own merits, not smuggled in as a consequence.
- **ICC profile interpretation or conversion.** ICC stays opaque container-lane
  bytes (`src/image/mod.rs:264`, constraint `metadata-not-via-pixel-encode`).
  "Linear light" here means the sRGB transfer function, *not* colour management.
  Wider-gamut input is assumed sRGB, as it is today.
- RAW development, demosaic, camera profiles (DEC-055; LGPL-blocked).
- Any change to the encode/quality decision engine.

## Design Notes

- **The representation choice is a real trade, not a formality.** f32 RGBA is
  4× the memory of RGBA8, which collides with the DEC-034/DEC-063 decode pixel
  budgets and matters on wasm (bundle size and the demo's megapixel gate,
  DEC-082). u16 is 2× and probably sufficient for a pipeline with no tonal ops.
  **Spec 1 should decide this with a measurement,** including peak-RSS on the
  largest gated input, not by preference.
- `fast_image_resize` supports `U16x4`/`F32x4` pixel types and `MulDiv` for
  premultiplication — the backend does not need replacing.
- The op trait extension has a precedent to follow: DEC-017 extended `Operation`
  once already (metadata read access) and required its own decision record. Same
  shape here.
- Keep the trait hint minimal — one associated capability, not a general
  negotiation protocol. DEC-002: "Keep this trait **small**."
- Watch the wasm leg specifically: this is engine/shared code, so it needs a
  **clean, full-matrix verify** (default / lean / webp-lossy, clippy each, fmt) —
  see [[a-stale-incremental-build-is-a-false-green]].

## Dependencies

### Depends on
- Nothing. All four preconditions are internal.

### Enables
- Roadmap wave 5 (Geometry, PROJ-006) — rotate/crop resample correctly the first time
- Any future auto-only tone work (normalize / auto-contrast / gray-world WB),
  which is the *only* tonal direction the stated thesis sanctions
- Honest >8-bit handling if AVIF/HDR output is ever revisited

## Stage-Level Reflection

*Filled in when status moves to shipped.*
