---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-003                     # stable, zero-padded within the project
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: crustyimg

created_at: 2026-06-14
shipped_at: 2026-06-15

# What part of the project's value thesis this stage advances.
value_contribution:
  advances: >
    Delivers the first pixel-mutating Operations and real encodes — the
    everyday web-prep workhorses (resize, thumbnail, shrink, convert,
    auto-orient). This is where the "routine image prep is faster than a
    GUI" half of the thesis becomes true for single-image use.
  delivers:
    - "`resize` — max / exact / percent / fit / fill / cover via the SIMD backend"
    - "`thumbnail` — bounded small resize, `--square` center-crop"
    - "`shrink` — the headline web-prep command: resize + real quality encode + strip metadata"
    - "`convert` — re-encode between core formats (JPEG/PNG/GIF/BMP/TIFF/ICO)"
    - "`auto-orient` — apply EXIF orientation to pixels, then clear the tag"
  explicitly_does_not:
    - Implement watermarking or the metadata edit lane (STAGE-004)
    - Provide WebP output (fast-follow) or AVIF (feature-gated, later)
    - Run batches in parallel or load recipes (STAGE-005)
---

# STAGE-003: transform and output

## What This Stage Is

The first stage that changes pixels and writes images. It delivers the
core transform set as `Operation`s flowing through the STAGE-001 pipeline
to a real encoding Sink: `resize` (max / exact / percent / fit / fill /
cover, on the `fast_image_resize` SIMD backend, DEC-008), `thumbnail` (a
convenience bounded resize with `--square` center-crop), `shrink` (the
headline web-prep command — resize + real quality encode + metadata strip,
honoring `--keep-gps`), `convert` (re-encode between the core formats), and
`auto-orient` (bake EXIF orientation into pixels, then clear the tag —
fixing the most common silent rotation bug). When this ships, the everyday
single-image web-prep tasks each work as one short command.

## Why Now

These are the features the prototype was built for and the most-used real
commands. They depend only on the STAGE-001 pipeline + Sink and the
STAGE-002 load/inspect path being proven, and they must exist before
recipes (STAGE-005) have anything meaningful to chain. Resize is also the
performance-critical path, so landing the SIMD backend here de-risks the
batch story later.

## Success Criteria

- `resize` supports all six modes (max/exact/percent/fit/fill/cover),
  mutually exclusive, on the SIMD backend, with resize-parity tested within
  tolerance against `image::imageops` (DEC-008).
- `thumbnail` produces a bounded small image; `--square` center-crops.
- `shrink photo.jpg` produces a meaningfully smaller file (resize + real
  quality encode + metadata strip), respecting `--keep-gps`.
- `convert --format png` re-encodes correctly; an unbuilt codec (e.g. AVIF)
  exits 4 (DEC-004).
- `auto-orient` rotates pixels per the EXIF orientation tag and clears it,
  verified on a rotated fixture.

## Scope

### In scope
- `resize`, `thumbnail`, `shrink`, `convert`, `auto-orient` as `Operation`s + CLI surface.
- The SIMD resize backend wiring (DEC-008) and quality-aware encode for `shrink`/`convert`.
- Default-preserve / drop-GPS behavior on pixel-lane encodes (DEC-003) via the existing Sink.

### Explicitly out of scope
- Watermark + metadata edit lane (STAGE-004).
- WebP output (fast-follow), AVIF (feature-gated) — see docs/backlog.md and DEC-004.
- Geometry extras (crop/rotate/flip/trim/pad), effects catalog, color/tone — post-MVP.
- Parallel batch + recipes (STAGE-005); single-input + `--out-dir` fan-out only here.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-010 (shipped 2026-06-15, PR #11) — `resize` **Operation** + the operation-params mechanism (DEC-014): max/exact/percent/fit/fill/cover via fast_image_resize SIMD backend (DEC-008), registry-registered, parity-tested — library only (recipe-usable)
- [x] SPEC-011 (shipped 2026-06-15, PR #12) — `resize` **CLI command** + multi-input `--out-dir` fan-out (sequential, no rayon); preserve-source-format default + partial-batch exit 6 (DEC-015); depends on SPEC-010
- [x] SPEC-012 (shipped 2026-06-15, PR #13) — `thumbnail` command: bounded small resize (default 256) + `--square` center-crop; thin wrapper over the `resize` op via a shared `run_pixel_op` fan-out helper (no new op/DEC)
- [x] SPEC-013 (shipped 2026-06-15, PR #14) — `shrink` command: resize to default max 1600 + quality-aware encode (default 80, DEC-016) + metadata drop (web-prep workhorse); added the quality-encode path to the Sink; `--keep-gps`/selective preserve deferred to STAGE-004
- [x] SPEC-014 (shipped 2026-06-15, PR #15) — `convert` command: re-encode across core formats (JPEG/PNG/GIF/BMP/TIFF/ICO), `--format` required + forced; exit 4 for unbuilt codecs resolved up front (DEC-004), reuses the DEC-016 quality knob + DEC-015 fan-out; CLI-only, no new op/DEC
- [x] SPEC-015 (shipped 2026-06-15, PR #16) — `auto-orient` command/Operation: a new registry-registered `AutoOrient` op that reads the captured EXIF orientation (DEC-017, image's native `Orientation`, no kamadak-exif) and bakes the rotation/flip into pixels; the re-encode clears the tag; reuses the `run_pixel_op` fan-out; no-op when no/identity orientation

**Count:** 6 shipped / 0 active / 0 pending

> **Note (2026-06-15):** `resize` was split into SPEC-010 (library: operation +
> the first parameterized-op params mechanism, DEC-014) and SPEC-011 (CLI +
> fan-out) — the original single `resize` backlog item assessed as complexity L
> (AGENTS §8). The split falls on the library↔CLI layering boundary.

## Design Notes

- Resize runs on `fast_image_resize` 5 (SIMD), not `image::imageops`
  (DEC-008); tests assert parity within tolerance rather than pixel
  exactness. `thumbnail` and `shrink`'s resize step reuse the same backend.
- `convert` codec support follows DEC-004: pure-Rust core formats by
  default; native/feature-gated codecs exit 4 when not built. WebP is
  fast-follow (post-MVP), AVIF feature-gated later.
- `shrink` and pixel-lane encodes apply the default-preserve / drop-GPS
  policy (DEC-003) through the Sink — they do **not** open the container
  edit lane (that's STAGE-004). `--keep-gps` opts out.
- `auto-orient` reads the orientation tag (kamadak-exif) to drive a pixel
  rotation, then ensures the emitted image's orientation is normalized.

## Dependencies

### Depends on
- STAGE-001 — `Operation` trait, pipeline, encoding Sink, name-template fan-out.
- STAGE-002 — proven load/inspect path on real images.
- External: `fast_image_resize` (DEC-008); `image` encoders (DEC-002/004).

### Enables
- STAGE-005 — these Operations are the building blocks recipes chain and batch replays.
- The lead post-MVP wave (geometry extras / effects) drops in as more `Operation`s.

## Stage-Level Reflection

*Filled in 2026-06-15 at stage ship (all 6 specs shipped).*

- **Did we deliver the outcome in "What This Stage Is"?** **Yes.** All five
  everyday single-image web-prep commands work as one short command and are
  tested + 3-OS-CI-green: `resize` (six modes on the `fast_image_resize` SIMD
  backend, parity-tested), `thumbnail` (`--size` bound + `--square` center-crop),
  `shrink` (resize + quality-aware JPEG encode + metadata drop), `convert`
  (forced-format re-encode across the core formats, exit 4 for unbuilt codecs),
  and `auto-orient` (bakes EXIF orientation into pixels and clears the tag). The
  pixel core now mutates pixels and writes real encodes end to end — the "routine
  prep is faster than a GUI" half of the thesis is true for single-image use.
- **How many specs did it actually take?** **6** (SPEC-010 … SPEC-015) vs. the 5
  planned backlog items — `resize` was split into SPEC-010 (the `Resize`
  Operation + the DEC-014 operation-params mechanism) and SPEC-011 (the CLI +
  multi-input fan-out) because the combined item assessed as complexity L on the
  library↔CLI layering boundary. Every other command was one spec.
- **What changed between starting and shipping?** The stage grew a reusable
  spine — DEC-014 (op-params), DEC-015 (source-format preserve + partial-batch
  exit 6), DEC-016 (quality encode), DEC-017 (ops may read metadata) and the
  shared `run_pixel_op` fan-out (now carrying `quality` + `forced_format`) — so
  each later command (`thumbnail`/`shrink`/`convert`/`auto-orient`) landed as a
  thin, prescriptive build on top of it rather than bespoke plumbing.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - None require an edit. The mid-stage CI change (clippy → `cargo clippy
    --all-targets -- -D warnings`) was already folded into AGENTS §5/§6/§11 and
    the `clippy-fmt-clean` constraint before this stage's builds.
  - Process lesson worth carrying forward (already proven across SPEC-013/014/015,
    not a doc change): the architect verifying the exact upstream crate API during
    **design** (e.g. `JpegEncoder::new_with_quality`, `image`'s
    `Orientation::from_exif_chunk`/`apply_orientation`, the convert `--format`
    clap-shadow, the exit-4-up-front semantics) turned every build into a
    zero-deviation Sonnet pass. De-risking in design > discovering in build.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - From SPEC-014: when a command's output format is *forced and global to the
    invocation*, resolve it **once up front** so a capability error (unbuilt
    codec → exit 4) doesn't masquerade as a per-input partial-batch failure
    (exit 6). General to any future forced-format command.
  - From SPEC-015: when an Operation needs metadata, check whether the pixel
    library (`image`) already exposes the exact reader before reaching for the
    dedicated metadata crate — it kept `auto-orient` inside the op module's
    `::image` surface (DEC-017). General to future metadata-driven ops.
