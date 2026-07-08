---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-018
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-009
repo:
  id: crustyimg

created_at: 2026-07-08
shipped_at: null

value_contribution:
  advances: >
    Extends the project's "read the modern-format assets developers actually
    have" thesis to camera RAW: extract the embedded full-res JPEG preview from
    common RAW files on the default, pure-Rust, zero-system-dep build, so the
    shipped commands (optimize / convert / info / resize / batch) produce a quick
    web derivative from a `.nef`/`.cr2`/`.cr3`/`.arw`/`.dng`/… with no RAW codec,
    no copyleft, and no patents — the third default input after AVIF and SVG.
  delivers:
    - "Common RAW (`.nef .cr2 .cr3 .arw .dng .raf .rw2 .orf .pef .srw .nrw .rwl`) yields its embedded full-res JPEG preview as the canonical raster `Image` in the DEFAULT build"
    - "RAW files are discovered by directory/glob sources and flow through optimize/convert/info/resize"
    - "A crafted/hostile RAW is handled safely — the preview decode is bounded by the DEC-034 caps, false candidates are skipped, and failures are typed errors not panics"
    - "A recorded decision (DEC-055) that RAW = Tier-1 largest-embedded-JPEG preview (format-agnostic byte scan + capped `image` JPEG decode), NOT RAW development, with NO new dependency"
  explicitly_does_not:
    - "RAW *development* (demosaic / white-balance / color-matrix) — that is Tier-2 (LGPL `rawler`, watchlist), explicitly out"
    - "Decode the raw sensor mosaic itself, or produce anything better than the camera's own embedded preview"
    - "Guarantee every RAW model/vendor — RAW breadth is a corpus problem; a RAW with no baseline-JPEG preview (e.g. Sigma `.x3f`) yields a typed 'no preview' error, not a crash"
    - "Support RAW via stdin (`-`) in v1 — routing is by file extension in `Image::load`; stdin/`from_bytes` stays a documented follow-up"
    - "Pull any new dependency, C/system code, or copyleft onto the default path"
---

# STAGE-018: RAW Tier-1 embedded-preview extraction as a default input

## What This Stage Is

The stage that lets the default crustyimg binary **read camera RAW** — not by
decoding the raw sensor mosaic (that is Tier-2 development, LGPL/out of scope),
but by extracting the **embedded full-res JPEG preview** that nearly every RAW
file carries (the image the camera's own screen shows). So
`crustyimg optimize photo.nef -o photo.webp` (and `convert`, `info`, `resize`,
batch) produce a quick web derivative from `.nef`/`.cr2`/`.cr3`/`.arw`/`.dng`/…
with **no RAW codec, no new dependency, no copyleft, no patents, no demosaic**.
The mechanism is deliberately format-agnostic: scan the file for embedded JPEG
streams, decode each candidate with the existing (capped) `image` JPEG decoder,
and keep the largest that decodes — which covers TIFF-based RAW **and** Canon CR3
(ISOBMFF) **and** Fuji RAF in one path, with no per-vendor IFD/box parsing. It is
the third default input of PROJ-009, mirroring STAGE-016 (AVIF) and STAGE-017
(SVG): explicit routing before the generic decoder, RAW extensions in the source
allow-list, typed errors, DEC-034 caps, a cargo-fuzz target, and a decision DEC.

## Why Now

- **Third permissive, patent-clean default input.** After AVIF and SVG, RAW
  preview is the remaining high-value "modern asset" a photographer/prosumer has
  on disk. Tier-1 preview is the honest, achievable slice (the watchlist's
  "recommended basic-conversion build") — unlike Tier-2 development, which is
  LGPL (`rawler`) or a multi-month demosaic effort, deliberately deferred.
- **Cheaper and broader than framed.** The design-time probe (2026-07-08) showed
  a byte-scan-for-largest-JPEG approach needs **no new dependency** (reuses
  `image`'s JPEG decoder) and covers CR3/RAF **without** the ISOBMFF box parsing
  the brief assumed — collapsing what looked like a multi-spec stage into one.

## Success Criteria

- `Image::load("x.nef")` (and `.cr2/.cr3/.arw/.dng/.raf/.rw2/.orf/…`) returns the
  embedded full-res JPEG preview as the canonical raster `Image` in the **default**
  build (no new deps, no system libs), honoring the DEC-034 decode caps.
- `optimize`/`convert`/`info`/`resize` operate on RAW inputs end to end;
  directory/glob sources discover RAW files; a RAW with no usable embedded preview,
  or a corrupt one, surfaces a typed `ImageError` (never a panic).
- **Hostile-input safety:** candidate preview decodes are bounded by the DEC-034
  caps (a decompression-bomb preview is rejected, not OOM'd); false SOI matches in
  compressed data are skipped cheaply; the candidate-decode count is bounded; a
  cargo-fuzz target exercises the scan+decode path.
- **No new dependency and no C/system dep on the default path**; `just deny` green
  (no new crate, no license/advisory change); the lean `--no-default-features`
  build still succeeds.
- A **DEC-055** records the approach: RAW = Tier-1 largest-embedded-JPEG preview
  via a format-agnostic byte scan + capped `image` JPEG decode; extension-routed;
  no new dependency; Tier-2 development explicitly out.

## Scope

### In scope
- A `src/image/raw.rs` module: `is_raw_extension` + `extract_preview(bytes, limits)`
  (bounded JPEG-SOI scan → capped decode of each candidate → largest decodable
  wins → canonical `Image`, `source_format = Jpeg`). Extension-routed in
  `Image::load`; RAW extensions added to `IMAGE_EXTENSIONS`; typed errors; a
  cargo-fuzz target; DEC-055. **(SPEC-061)**

### Explicitly out of scope
- RAW **development** (demosaic/white-balance/color-matrix — Tier-2, LGPL `rawler`,
  watchlist); RAW **via stdin**; guaranteeing every vendor/model; RAW-specific
  metadata/EXIF passthrough beyond what the preview JPEG itself carries; AVIF/SVG/
  HEIC inputs (other stages).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-061 (design) — RAW **Tier-1 embedded-preview extraction**: extension-routed in
  `Image::load`; `src/image/raw.rs` scans for embedded JPEG streams, decodes each with the
  DEC-034-capped `image` decoder, keeps the largest → canonical `Image` (`source_format = Jpeg`);
  RAW extensions in `IMAGE_EXTENSIONS`; typed errors, candidate-count + per-decode caps,
  `fuzz/raw_preview` target; DEC-055 — **no new dependency**, format-agnostic (covers TIFF-based
  RAW + CR3 + RAF).

**Count:** 0 shipped / 1 active / 0 pending — single-spec stage (mirrors STAGE-016/017's shape).

## Design Notes

- **PROBE RESULT (2026-07-08) — a format-agnostic byte scan beats per-format IFD/box parsing;
  no new dep; covers CR3/RAF too.** A firsthand probe (the repo's pinned `image` =0.25.10) confirmed:
  `image::load_from_memory` **tolerates trailing bytes after a JPEG's EOI**, so scanning a RAW for
  JPEG start-of-image markers (`FF D8 FF`), decoding from each, and keeping the **largest that
  decodes** cleanly extracts the full-res preview and skips the small thumbnail — with **no IFD or
  ISOBMFF parsing**. This corrects two framing assumptions: (1) the brief's "reuses ISOBMFF glue for
  CR3" is unnecessary — CR3 (and Fuji RAF) previews are found by the same scan; (2) the watchlist's
  "parse the TIFF/EXIF IFDs (kamadak-exif)" is a *harder* path than needed. Net: **no new
  dependency** (reuses `image`'s JPEG decoder), one module, one `load` branch.
- **DEC-055 (at build):** RAW input = **Tier-1 largest-embedded-JPEG preview**, extracted by a
  bounded byte scan + capped `image` JPEG decode; extension-routed; NOT RAW development. Records the
  approach, the covered extensions, the no-new-dep result, the security bounds, and that Tier-2
  (`rawler`, LGPL) stays out (watchlist `raw-camera-decode`).
- **Detection is extension-driven, by necessity.** TIFF-based RAW (`.nef/.cr2/.arw/.dng/.rw2/.orf/…`)
  starts with the **TIFF magic** (`II*\0`/`MM\0*`), indistinguishable from a plain `.tif` by bytes —
  so a byte-content sniff would risk mis-routing legitimate TIFFs. Route by **file extension** in
  `Image::load` (which has the path) → `raw::extract_preview`; keep the generic byte-sniff path for
  everything else. RAW-via-stdin (`from_bytes`, no path) is a documented v1 non-goal (a `--format raw`
  hint or a content sniff is a later option). This is the one architectural wrinkle: the RAW branch
  lives in `load`, not in the byte-only `decode_with_limits` seam where AVIF/SVG dispatch.
- **`source_format = Jpeg`** — the extracted preview *is* a JPEG, so the canonical `Image` reports
  `ImageFormat::Jpeg` (the same "report the materialized raster format" call as SVG→`Png`, DEC of
  STAGE-017). Consequence: `info x.nef` reports `jpeg`; an accepted wart, consistent with the SVG
  precedent. A faithful `SourceFormat` enum is the shared follow-up.
- **Security (untrusted binary input):** every candidate decode goes through the DEC-034 caps
  (`decode_with_limits`-equivalent) so a bomb preview is rejected, not OOM'd; prune candidates cheaply
  (require a plausible JPEG marker byte after `FF D8 FF`) and **cap the number of full decode
  attempts** so a file stuffed with fake SOIs can't cause unbounded work; a candidate that exceeds the
  caps is skipped, and if the only preview present is oversize → typed `LimitsExceeded`; if none
  decode → typed `Decode`/`UnsupportedFormat`. Add a `fuzz/raw_preview` target. Rust slicing keeps the
  scan in-bounds; no maker-supplied offset is trusted (we scan, we don't seek by IFD offset).
- **Wiring is small (mirror AVIF/SVG):** a `src/image/raw.rs` module (`is_raw_extension` +
  `extract_preview` + a public byte entry for the fuzz target), one branch in `Image::load`, the RAW
  extensions added to `IMAGE_EXTENSIONS`, tests + a fuzz target, DEC-055. No `Cargo.toml`/`deny.toml`
  change (no new crate).

## Dependencies

### Depends on
- Shipped decode seam + `image`'s JPEG decoder (`src/image/mod.rs`: `Image::load`, `decode_with_limits`,
  `decode_limits()` DEC-034 caps), `src/source/mod.rs` (`IMAGE_EXTENSIONS`), `src/error.rs`.
- DEC-004 (pure-Rust default), DEC-034 (decode caps), DEC-018 (`no-agpl-default-deps` — Tier-2 `rawler`
  is LGPL and stays out).
- The AVIF/SVG input precedents (STAGE-016/017) for the mirror-pattern.

### Enables
- Optimize/convert/lint coverage over RAW asset trees; a future Tier-2 development path (opt-in
  `rawler` feature) if development-grade quality is ever pulled; the shared "materialized-raster
  source_format" cleanup.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
