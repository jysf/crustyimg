---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-016
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-15
supersedes: null
superseded_by: null

affected_scope:
  - src/sink/**
  - src/cli/**

tags:
  - encode
  - quality
  - jpeg
  - sink
  - web-prep
---

# DEC-016: `-q/--quality` maps to JPEG encode quality; ignored for lossless formats; `shrink` defaults to quality 80

## Decision

The encode path gains an optional `quality: Option<u8>` (0–100). When the
output format is **JPEG** and `quality` is `Some(q)`, encoding goes through
`image::codecs::jpeg::JpegEncoder::new_with_quality(writer, q)` instead of the
default `DynamicImage::write_to`. For **lossless / non-quality formats**
(PNG, GIF, BMP, TIFF, ICO) `quality` is **ignored** (those encoders take no
0–100 quality knob) — encoding stays on the default `write_to` path. The
`shrink` command applies a **default quality of 80** when `-q` is omitted; the
other pixel commands (`resize`, `thumbnail`) thread `global.quality` straight
through, so an explicit `-q` is honored where the format supports it but no
default is forced on them. `quality` is threaded `run_pixel_op → Sink::write →
encode_to_bytes`; the display sink ignores it.

## Context

`shrink` (SPEC-013) is the headline "optimize for web" command — resize +
**real quality-aware encode** + strip metadata. Until now `encode_to_bytes`
(`src/sink/mod.rs`) only called `img.pixels().write_to(&mut cursor, format)`,
which uses the `image` crate's *default* JPEG quality and exposes no way to set
a quality. `-q/--quality` already exists as a global CLI arg (`GlobalArgs`) but
was unwired (`resize`/`thumbnail` deferred it to "the shrink/convert story",
SPEC-011). SPEC-013 must wire it. `convert` (SPEC-014) re-encodes between
formats and will reuse the exact same quality knob, so the policy is durable
and belongs in a DEC rather than buried in one command.

Two questions had to be answered repo-wide: (1) which formats `-q` affects, and
(2) what `shrink` does when `-q` is omitted. The `image` crate only offers a
quality parameter for JPEG (`JpegEncoder::new_with_quality`); PNG is lossless
(its tunable is a *compression* level, a different axis we deliberately do not
expose yet), and GIF/BMP/TIFF/ICO have no 0–100 quality. So `-q` is
meaningfully a JPEG control. DEC-004 (pure-Rust core codecs) bounds the format
set; no new dependency is involved — `JpegEncoder` is in the already-pinned
`image` crate.

## Alternatives Considered

- **Option A: keep `write_to` everywhere; ignore `-q`.**
  - What it is: never honor `-q`; `shrink` just resizes + re-encodes at the
    encoder default.
  - Why rejected: defeats the point of `shrink` ("real quality encode" for web);
    `-q` would be a documented-but-dead flag. The whole value of a web-prep
    command is controlling the size/quality tradeoff.

- **Option B: map `-q` onto every format (e.g. PNG compression level).**
  - What it is: reinterpret `-q` per format — JPEG quality, PNG compression,
    etc.
  - Why rejected: conflates two different axes (lossy quality vs lossless
    compression effort) under one 0–100 number with format-dependent meaning —
    surprising and hard to document. Keep `-q` = lossy quality (JPEG today;
    WebP/AVIF later when those land), and leave PNG compression to a future
    explicit flag if ever wanted.

- **Option C: force a default quality on ALL commands' JPEG output.**
  - What it is: `resize photo.jpg` (no `-q`) would also re-encode at quality 80.
  - Why rejected: `resize`/`thumbnail` are geometry commands; silently lowering
    JPEG quality on a plain resize is a surprising lossy change. Only `shrink`
    (whose stated job is web optimization) opts into a default quality; the
    others pass `global.quality` through unchanged (encoder default when absent).

- **Option D (chosen): `-q` → JPEG quality, ignored for lossless formats;
  `shrink` defaults to 80, others thread `global.quality` as-is.**
  - What it is: the decision above.
  - Why selected: honors `-q` exactly where it has meaning (JPEG), keeps a plain
    resize lossless-by-default, gives `shrink` the ergonomic web default, and
    establishes one quality knob `convert` reuses verbatim. No new dependency,
    no second image library, format set unchanged (DEC-004).

## Consequences

- **Positive:** `shrink photo.jpg` "just works" (resize to a web bound + a
  sensible quality) and `-q` tunes size precisely; `convert` (SPEC-014) reuses
  the same `quality` plumbing for free; the change is localized to the Sink
  encode + the `run_pixel_op` thread.
- **Negative:** `-q` is silently a no-op for PNG/GIF/BMP/TIFF/ICO (documented in
  the `shrink`/`convert` api-contract entries, but a user could still expect
  PNG `-q` to shrink a PNG — it won't). Default quality 80 is a judgement call
  that may not suit every workflow (overridable with `-q`).
- **Neutral:** Metadata is dropped on the pixel-lane re-encode regardless of
  quality (the `image` crate discards it); the selective default-preserve /
  `--keep-gps` policy (DEC-003) is the STAGE-004 container lane, NOT part of this
  decision. `shrink`'s "strip metadata" is the inherent drop, not a new code
  path.

## Validation

Right if: `shrink in.jpg -q 30 -o lo.jpg` produces a meaningfully smaller file
than `-q 90`, both decode correctly at the resized dimensions; `shrink in.png`
resizes a PNG with `-q` ignored and no error; `resize`/`thumbnail` output is
byte-unchanged when no `-q` is passed (their existing tests stay green). Revisit
if: WebP/AVIF output lands (those take their own quality — extend the per-format
mapping), or users want PNG compression-level control (add an explicit flag
rather than overloading `-q`), or the default quality 80 proves wrong for the
common case.

## References
- Related specs: SPEC-013 (`shrink` — first consumer), SPEC-014 (`convert` —
  reuses the quality knob), SPEC-005 (the `Sink` this extends), SPEC-011
  (deferred `-q` to here).
- Related decisions: DEC-004 (pure-Rust codec policy / format set; `JpegEncoder`
  is in the existing `image` dep), DEC-015 (output-format preservation — `-q`
  applies to whatever format is preserved/chosen), DEC-003 (metadata dual-lane —
  why "strip metadata" is the inherent drop, not selective preserve), DEC-007
  (typed `SinkError` on encode failure).
- External docs: https://docs.rs/image/0.25.10/image/codecs/jpeg/struct.JpegEncoder.html
