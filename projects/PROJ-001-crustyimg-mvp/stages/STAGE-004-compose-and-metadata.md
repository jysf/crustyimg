---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-004                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: crustyimg

created_at: 2026-06-14
shipped_at: null

# What part of the project's value thesis this stage advances.
value_contribution:
  advances: >
    Adds the compositing capability (image + text watermark) and opens the
    container-level metadata lane (strip / clean-GPS / set / copy) — the
    privacy and attribution features that round out single-image prep and
    that the metadata-edit half of the thesis depends on.
  delivers:
    - "`watermark` — overlay an image watermark at a gravity anchor (opacity/scale/margin/tile); text watermark as a trailing addition"
    - "`strip` — remove all metadata (EXIF/IPTC/XMP/ICC) at the container level"
    - "`clean --gps` — remove only GPS/location metadata, keep the rest (privacy)"
    - "`set` — write specific EXIF tags (artist/copyright/description), pixels untouched"
    - "`copy-metadata` — copy a container's metadata from one image to another"
  explicitly_does_not:
    - Route metadata edits through the pixel decode/encode path (container lane only, DEC-003)
    - Implement recipes, the `edit` one-shot, or parallel batch (STAGE-005)
    - Do the full security hardening / assessment (STAGE-006)
---

# STAGE-004: compose and metadata

## What This Stage Is

This stage adds the two remaining single-image capability areas:
compositing and metadata. `watermark` overlays an image watermark at a
gravity anchor with opacity, scale, margin, and tile options (a text
watermark is a trailing addition within the stage, via `ab_glyph` +
`imageproc::drawing`). The **metadata lane** opens here — a container-level
path (DEC-003) that edits EXIF/IPTC/XMP/ICC segments **without re-decoding
pixels**: `strip` (remove all metadata), `clean --gps` (drop only location
— a privacy win), `set` (write artist/copyright/description tags), and
`copy-metadata` (transfer a container's metadata from one image to
another). When this ships, every single-image command in the MVP exists.

## Why Now

Watermark is a user-requested headline feature and the only `Operation`
that composes two images, so it needs the pixel pipeline (STAGE-003) in
place. The metadata lane is architecturally distinct from the pixel lane
and must NOT be forced through encode (DEC-003, constraint
`metadata-not-via-pixel-encode`); building it as its own coherent chunk
here keeps that separation clean. Both are prerequisites for recipes — a
recipe should be able to chain a resize, a watermark, and a metadata strip.

## Success Criteria

- `watermark --image logo.png --gravity southeast` composites the overlay
  at the right anchor; opacity/scale/margin/tile behave as documented.
- Text watermark renders legible text at a gravity anchor.
- `strip` removes all metadata at the container level (verified: no EXIF/
  ICC/XMP afterward) without re-encoding pixels.
- `clean --gps` removes GPS tags while preserving other metadata.
- `set --artist/--copyright/--description` writes the tags; `copy-metadata
  --from a --to b` transfers the container's metadata. Pixels byte-identical
  in both metadata-lane cases (no pixel re-encode).

## Scope

### In scope
- `watermark` (image overlay: gravity/opacity/scale/margin/tile) + text watermark (trailing).
- Container-lane metadata: `strip`, `clean --gps`, `set` (artist/copyright/description), `copy-metadata`.
- Wiring the metadata lane to bypass the pixel encode path entirely (DEC-003).

### Explicitly out of scope
- Pixel-lane re-encode for metadata edits (forbidden — DEC-003).
- Recipes / `edit` / `--save-recipe` / parallel batch (STAGE-005).
- Caption/shapes/borders, blend modes, montage/append — post-MVP (docs/backlog.md).
- Full security assessment, decode limits, traversal tests (STAGE-006).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-026 (shipped 2026-06-18, PR #30) — metadata lane v1: `strip` (remove all metadata) + `clean --gps` (remove only location) via container lane, JPEG+PNG, no pixel re-encode (DEC-003, DEC-029)
- [x] SPEC-029 (shipped 2026-06-18, PR #33) — `watermark` command/Operation: image overlay at gravity anchor (`--opacity`/`--scale`/`--margin`/`--tile`); first multi-image Operation (DEC-031)
- [ ] (not yet written) — text watermark: render text at a gravity anchor (ab_glyph + imageproc::drawing) — reuses `Gravity` + placement; needs a font-dep DEC
- [x] SPEC-027 (shipped 2026-06-18, PR #31) — `set` command: write EXIF tags (`--artist`/`--copyright`/`--description`) via little_exif, pixels untouched (reuses `run_metadata_lane`)
- [x] SPEC-028 (shipped 2026-06-18, PR #32) — `copy-metadata` command: copy container EXIF+ICC `--from` one image `--to` another, DST pixels untouched; JPEG-only v1 (DEC-030)

**Count:** 4 shipped / 0 active / 1 pending  (metadata lane COMPLETE: SPEC-026/027/028; image `watermark` SPEC-029 shipped; remaining: text watermark — needs a font-dep DEC)

## Design Notes

- The metadata lane is **container-level only** (DEC-003, constraint
  `metadata-not-via-pixel-encode`): `img-parts` for EXIF/ICC segment
  manipulation, `little_exif` for tag writes; pixels are never re-decoded
  or re-encoded by `strip`/`clean`/`set`/`copy-metadata`. Keep these out of
  the `Operation` trait (it's the pixel extension point).
- `watermark` is a pixel-lane `Operation` (it composes pixels) and follows
  the default-preserve / drop-GPS policy on its encode like any pixel op.
- Gravity is the shared compass-anchor concept (AGENTS.md §14) reused later
  by `crop` in the post-MVP geometry wave.
- Text watermark depends on a font-rendering crate (`ab_glyph` +
  `imageproc::drawing`); if that introduces a new top-level dep it needs a
  DEC (constraint `no-new-top-level-deps-without-decision`).

## Dependencies

### Depends on
- STAGE-003 — pixel pipeline + encode Sink for the watermark composite.
- STAGE-001 — `Operation` trait (watermark) and the Sink.
- External: `img-parts`, `little_exif` (metadata lane); `imageproc`/`ab_glyph` (text).

### Enables
- STAGE-005 — recipes can chain resize → watermark → strip; the full edit chain.
- STAGE-006 — metadata/recipe write surfaces are part of the hardening target.

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
