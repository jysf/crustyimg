---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-017
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
  - src/operation/**

tags:
  - operations
  - metadata
  - exif
  - orientation
  - architecture
---

# DEC-017: Operations may READ the captured `MetadataBundle` to parameterize a pixel transform; `auto-orient` uses `image`'s native `Orientation`

## Decision

An `Operation` is allowed to **read** (never edit) the `Image`'s captured
`MetadataBundle` (the raw EXIF/ICC bytes captured at load, DEC-003) to decide
how to transform pixels. `auto-orient` is the first such op: it reads the raw
EXIF segment, extracts the orientation via the **`image` crate's own**
`image::metadata::Orientation::from_exif_chunk` and applies it with
`DynamicImage::apply_orientation` — staying entirely within the operation
module's existing `::image` dependency surface (NO `kamadak-exif`). After baking
the rotation/flip into pixels, the op **drops the carried metadata bundle**
(returns `Image::from_parts(pixels, source_format, None)`), so the now-stale
orientation tag does not propagate; the pixel-lane re-encode would drop it
anyway, so the emitted image carries no orientation tag.

## Context

`auto-orient` (SPEC-015) is the last STAGE-003 command and the first
`Operation` whose pixel transform is **driven by container metadata**: cameras
record the sensor orientation as an EXIF tag (1–8) rather than rotating pixels,
and the `image` crate's decoder does **not** apply it on `decode()` (it exposes
`ImageDecoder::orientation()` separately and a manual
`DynamicImage::apply_orientation`). So a freshly decoded `Image` holds
unoriented pixels; something must read the tag and rotate.

This brushes against DEC-003, which states the metadata lane is **not**
expressed as `Operation`s and that the pixel and container lanes are separate.
The boundary needs a clear ruling: is reading orientation inside an `Operation`
a DEC-003 violation? Two facts make it clean: (1) the `Image` already carries
the raw EXIF bytes (`MetadataBundle`, captured at load, DEC-003) — the op needs
no new I/O, no decode, no container crate; (2) the op only **reads** metadata to
choose a pixel transform — it never **edits** container metadata, which is what
DEC-003 reserves for the container lane (`strip`/`clean`/`set`/`copy-metadata`,
STAGE-004). Two implementation choices also needed pinning: which EXIF reader to
use, and what happens to the (now-stale) carried tag.

`image` 0.25.10 supplies `Orientation::from_exif(u8)`,
`Orientation::from_exif_chunk(&[u8])` (parses a raw TIFF chunk beginning with
the `II`/`MM` magic — no `Exif\0\0` prefix), and `DynamicImage::apply_orientation`.
`kamadak-exif` is already a dependency (DEC-013, read-only, used by `info --exif`)
but lives outside the operation module's allowed deps.

## Alternatives Considered

- **Option A: Parse orientation with `kamadak-exif` inside the op.**
  - What it is: add the `exif` crate to the operation module and call
    `Reader::read_raw` on the captured segment.
  - Why rejected: widens the operation module's dependency surface (it currently
    depends only on `::image`, `std`, `thiserror`, `serde`, `fast_image_resize`),
    and forces handling the `read_raw` vs `read_from_container` distinction and
    the JPEG `Exif\0\0` prefix manually. `image`'s `from_exif_chunk` already does
    exactly this and keeps the op single-image-library (constraint
    `single-image-library`). `kamadak-exif` stays where it belongs — the
    read-only `info` lane (DEC-013).

- **Option B: Make orientation a non-`Operation` step (a CLI-only pre-rotate).**
  - What it is: read+rotate in `src/cli` outside the pipeline, so no op reads
    metadata.
  - Why rejected: `auto-orient` must be **recipe-usable** (a `[[step]]` op =
    "orient" in `docs/data-model.md`'s worked recipe). Keeping it an `Operation`
    is the whole point — it has to compose in a pipeline before resize/watermark.

- **Option C: Apply orientation but keep the original metadata bundle on the
  image (`with_pixels`).**
  - What it is: bake pixels but carry the old EXIF (with its orientation tag)
    forward.
  - Why rejected: the carried tag now describes the OLD orientation; if a future
    metadata-preserving sink (STAGE-004) emitted it, a correct EXIF viewer would
    rotate the already-rotated pixels again (double rotation). Dropping the
    bundle after baking is the correct, future-proof choice (and matches the
    inherent pixel-lane drop today).

- **Option D (chosen): op READS the captured bundle via `image`'s native
  `Orientation`, applies it, and drops the bundle.**
  - Why selected: no new dependency, stays in `::image` (single-image-library),
    no I/O in the op (`decode-once-no-per-op-disk` honored — it reads
    already-captured bytes), respects DEC-003 (reads, never edits container
    metadata), recipe-usable, and correct under future metadata preservation.

## Consequences

- **Positive:** `auto-orient` is a clean, composable `Operation` reusing the
  existing capture path and `image`'s tested orientation math; no second EXIF
  parser in the pixel core; establishes the pattern for any future
  metadata-driven op (e.g. a color op reading ICC). The tag is cleared
  correctly (bundle dropped + inherent encode drop).
- **Negative:** The op depends on the `MetadataBundle` capture being present and
  correct for the source format. Capture is currently implemented for JPEG
  (APP1 `Exif\0\0`) and PNG (`eXIf` chunk) only (DEC-003 / `src/image`); for
  formats without capture (e.g. TIFF/BMP/GIF/ICO) `auto-orient` is a safe no-op
  (no orientation found) rather than an error — acceptable, since those rarely
  carry an orientation tag, and capture for more formats is a STAGE-004 item.
- **Neutral:** Reading the captured segment (rather than re-reading the file)
  means `auto-orient` reflects exactly what was captured at load — consistent
  with `info`'s `has_exif`. The op handles both the JPEG `Exif\0\0`-prefixed and
  the PNG bare-TIFF segment shapes (strip the signature if present).

## Validation

Right if: `auto-orient` on a JPEG whose EXIF Orientation is 6 (rotate 90)
produces an image whose width/height are swapped and whose pixels are rotated,
and whose re-encoded output carries no orientation tag (`info --exif` reports no
EXIF); an image with orientation 1 or no EXIF is returned unchanged (a no-op,
exit 0, not an error); the op pulls in no new crate and the operation module
still depends only on `::image` + its existing deps. Revisit if: the metadata
lane (STAGE-004) wants to PRESERVE non-orientation metadata across
`auto-orient` (then the op must selectively clear only the orientation tag in a
carried bundle rather than dropping the whole bundle), or capture is extended to
more formats (then `auto-orient` gains coverage for free).

## References
- Related specs: SPEC-015 (`auto-orient` — this decision's first and only
  consumer), SPEC-002 (the `MetadataBundle` capture at load), SPEC-010 (the
  `Resize` op template `AutoOrient` mirrors).
- Related decisions: DEC-003 (metadata dual-lane — this clarifies that an op may
  READ, never edit, container metadata), DEC-002 (the `Operation` boundary),
  DEC-013 (`kamadak-exif` is the read-only `info` lane, kept out of the op),
  DEC-015 (the `run_pixel_op` fan-out `auto-orient` reuses), DEC-016 (quality
  threading inherited).
- External docs: https://docs.rs/image/0.25.10/image/metadata/enum.Orientation.html ,
  https://docs.rs/image/0.25.10/image/enum.DynamicImage.html#method.apply_orientation
</content>
