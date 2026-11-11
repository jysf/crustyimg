---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-115
  type: decision
  confidence: 0.85
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

created_at: 2026-11-11
# Stands beside DEC-095 (unamended: a translucent fill's widening is DEC-095 applied) and
# DEC-110 (wasm refuses the same pairs with the same line). Amends neither.
supersedes: null
superseded_by: null

affected_scope:
  - src/operation/alpha.rs
  - src/operation/mod.rs
  - src/operation/pad.rs
  - src/pipeline/mod.rs
  - src/cli/optimize.rs
  - src/cli/common.rs
  - src/cli/ops.rs
  - src/cli/build.rs
  - src/wasm.rs

tags:
  - recipe
  - alpha
  - output-format
  - pad
---

# DEC-115: A step whose alpha the output format can't store is refused before the pipeline runs

## Decision

When a recipe step writes new pixels with an alpha below opaque and the resolved output format
can't store that alpha exactly, the input is refused before its pipeline runs. The check reads
the recipe and the format, never the output pixels, and covers only alpha a step introduces.

The rule has eight parts:

1. **Which steps.** A step answers `Operation::introduced_alpha`: the 8-bit alpha of every pixel
   it adds, the key that set it and that key's value as written, or nothing (the default). Today
   only `pad` answers, when its `color` has an alpha below `ff` (`transparent` is alpha `00`). A
   future op that can introduce alpha implements that method and is checked on every surface
   with no change to this record.
2. **Each format's capacity** (`operation::alpha::alpha_capacity`):
   - *none* (every pixel written opaque): JPEG;
   - *binary* (alpha `00` and `ff` only): GIF;
   - *full*: PNG, WebP, AVIF, TIFF, BMP, ICO. AVIF keeps alpha lossily; TIFF keeps it for
     crustyimg, macOS and ffmpeg, though libtiff and ImageMagick read crustyimg's RGBA TIFF as
     opaque (no `ExtraSamples` tag; filed as `tiff-alpha-extrasamples`).
   A format stores an alpha when its capacity is full, or binary and the alpha is `00` or `ff`,
   or none and the alpha is `ff` (`operation::alpha::stores`).
3. **Static, not pixel-based.** `Pipeline::unstorable_alpha(format)` returns the first step, in
   recipe order, whose introduced alpha the format can't store. A recipe is refused for every
   input of that format, including an image already at a `pad`'s aspect, which gets no fill
   pixel. A later step that cuts the new pixels off again doesn't lift the refusal.
4. **When it runs: once the format is known, before the pipeline.**
   - One input (`apply`, the pixel verbs through `run_pixel_op`): exit **4**, nothing written.
   - A batch or `build` target with one format for every input (`--format`, a name template's
     literal extension, `recipe.format`, a forced format): checked once before any input is
     read; exit **4**, nothing written, the out-dir not created. This is the shape a bad
     `--format` already has.
   - A batch or `build` target that keeps each input's own format: that input fails with one
     `error: <input>: …` line, the others are written, exit **6** (DEC-015).
   - wasm `transform`: throws the same message (DEC-110).
5. **Where the format is decided after the pipeline** (a terminal `optimize` step with no pinned
   format, on `apply` and `build`): no refusal. The decision treats the output as having alpha
   when the source has alpha *or* a step introduces it (`Pipeline::introduces_alpha`), so the
   shortlist never offers JPEG and the JPEG fallback is never added; GIF is never on the
   shortlist. When nothing beats the source and the pipeline changed nothing, the source is
   kept unchanged; it carries no introduced alpha, because no fill pixel was written.
6. **Scope: alpha a step introduces.** Alpha the source already carried is out: an RGBA PNG
   written to JPEG with no such step still drops its alpha silently with exit 0, as before this
   record. The idea `alpha-dropped-silently-on-encode` owns that case.
7. **Flattening is rejected.** A transparent fill into JPEG is never composited against a
   colour (ruled 2026-11-06).
8. **The message** names the step index, the op, the key and its value as written, the alpha in
   two hex digits, the format, and a way out. It names no spec, stage or decision.

## Context

SPEC-151 shipped `pad` with an opaque fill only. STAGE-058 promises a fill "with a colour or
transparent", and the maintainer ruled (SPEC-151 Q1/Q4, 2026-11-06) that `transparent` into a
format with no alpha is refused per input with exit 4, naming the step and the format, and
(SPEC-159 Q1-Q3, 2026-11-10) that GIF takes alpha `00` and opaque but refuses `01`-`fe`, and that
source alpha stays out. It is the first step whose validity depends on the output format, so the
rule binds every surface and every later op that can introduce alpha.

Driven at SPEC-159's frame and design: JPEG writes every pixel opaque and keeps the colour under
alpha 0 (a `transparent` border would be black); GIF writes alpha 0 exactly (crustyimg reads a
`#ff000000` border back as `(255,0,0,0)`) and every other alpha below `ff` as opaque; PNG, WebP,
BMP, ICO and TIFF keep `(255,0,0,128)` exactly for crustyimg's decoder; AVIF keeps alpha within a
few levels. And with no format pinned, the auto-decision chose JPEG for a JPEG source padded with
`transparent` and wrote a black border with exit 0, because its shortlist read the source's alpha.

## Alternatives Considered

- **Option A: flatten against a colour (white, or a stated one)**
  - What it is: composite the fill onto a background before a JPEG write.
  - Why rejected: ruled out 2026-11-06; it writes pixels nobody asked for, and the colour is a
    guess.

- **Option B: a pixel-based check after the pipeline**
  - What it is: scan the output for alpha below the format's capacity and refuse then.
  - Why rejected: it would also refuse an RGBA source with no `pad` written to JPEG, which exits
    0 today, so existing outputs would change; it runs after the expensive work; and the same
    recipe would pass or fail depending on the image.

- **Option C: refuse every non-opaque fill to GIF**
  - What it is: treat GIF as having no alpha.
  - Why rejected: GIF stores alpha 0 exactly, so `transparent` into GIF writes exactly the pixel
    asked for (maintainer ruling Q1).

- **Option D (chosen): a static recipe × format check before the pipeline, on every surface**
  - What it is: parts 1-8 above.
  - Why selected: predictable (one answer per recipe and format), cheap (before decode-heavy
    work in a batch), leaves every existing output unchanged, and a future op joins by
    implementing one method.

## Consequences

- **Positive:** a transparent `pad` never becomes a black or opaque border silently; a batch
  with one format fails fast with nothing written; native and wasm refuse the same pairs.
- **Negative:** the asymmetry is deliberate: source alpha into JPEG is still dropped silently.
  A recipe that pads and then crops the border off is refused into JPEG although its output
  would be opaque.
- **Neutral:** single-input `apply` and `run_pixel_op` now resolve the output format before the
  pipeline runs, so an invocation that is wrong in both ways (a bad `--format` and a border that
  rounds to 0 px) reports the format first (exit 4, was exit 1), as the batch path already did.

## Validation

Right if no output with an introduced alpha below `ff` reaches a JPEG or GIF file except alpha
`00` into GIF. Revisit if a format's capacity changes (an encoder that writes JPEG with an alpha
plane, or a GIF writer that dithers alpha), or if the source-alpha idea lands and a warning or
refusal for source alpha needs the same table.

## References

- Related specs: SPEC-151 (opaque pad), SPEC-159 (this rule), SPEC-160 (`edit --pad`, which
  reaches the check through `run_pixel_op`)
- Related decisions: DEC-015 (per-input batch failures, exit 6), DEC-095 (colour type
  preservation; a translucent fill widens to the alpha twin by its alpha clause, unamended),
  DEC-102 (load-time errors stay exit 1), DEC-110 (wasm agrees with native)
- Ideas: `alpha-dropped-silently-on-encode`, `tiff-alpha-extrasamples` (`guidance/ideas.yaml`)
