---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-029
  type: decision
  confidence: 0.85                   # raised from DEC-003's 0.8: a design-time probe
                                     # verified strip/clean on real JPEG + PNG.
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

created_at: 2026-06-18
supersedes: null
superseded_by: null

# The container-lane write path these crates implement.
affected_scope:
  - src/metadata/**
  - Cargo.toml

tags:
  - architecture
  - metadata
  - exif
  - privacy
  - dependencies
  - license
---

# DEC-029: Pin `img-parts` + `little_exif` as the container-lane write crates

## Decision

Add two pinned, pure-Rust, permissively-licensed dependencies to implement the
metadata **write** half of the container lane (DEC-003): **`img-parts` `=0.4.0`**
(MIT OR Apache-2.0) for **segment/chunk-level** strip + rewrite, and
**`little_exif` `=0.6.23`** (MIT OR Apache-2.0) for **tag-level** EXIF edits
(selective GPS removal). The read side stays `kamadak-exif` (already a dep,
read-only). Division of labor: **read = kamadak-exif · segment strip = img-parts ·
tag edit = little_exif**. Both crates are pure-Rust with no `-sys`/C build deps, so
the default build stays zero-system-deps and `just deny` stays green.

## Context

DEC-003 chose the two-lane model and *pre-named* `img-parts` + `little_exif` as the
container-lane write crates, but pre-naming is not a dependency-add decision
(constraint `no-new-top-level-deps-without-decision`). SPEC-026 (the first metadata
spec — `strip` + `clean --gps`) needs them in `Cargo.toml`, so this DEC pins the
exact versions, confirms the license + pure-Rust property, and records a
**design-time probe** that verified the crates actually do what DEC-003 assumed —
the riskiest assumption in the MVP (DEC-003 is 0.8 confidence; question
`metadata-icc-coverage`).

**Probe result (real 16×16 JPEG, EXIF seeded with `little_exif`):**

- `img-parts` `Jpeg::from_bytes` → `remove_segments_by_marker(0xE1..=0xEF, 0xFE)` →
  `encoder().write_to(..)`: removed APP1(EXIF/XMP)/APP2(ICC)/COM; **decoded pixels
  byte-identical** to the original (the compressed scan is carried verbatim — no
  re-encode).
- `little_exif` `Metadata::new_from_vec` → `get_ifd_mut(ExifTagGroup::GPS, 0)` +
  `remove_tag(..)` → `write_to_vec`: GPS tags gone, **Orientation + Copyright
  preserved**, **decoded pixels byte-identical**.
- `img-parts` `Png::from_bytes` → `remove_chunks_by_type(eXIf/iCCP/tEXt/zTXt/iTXt/
  tIME)` → re-encode: parses + round-trips losslessly (pixels identical).
- Edge: `little_exif::Metadata::new_from_vec` on a JPEG with **no EXIF** returns
  `Err("No EXIF data found!")` — `clean --gps` must treat this as a **no-op
  success** (exit 0), not a failure.

Transitive deps are all permissive + pure-Rust: img-parts → `bytes`, `crc32fast`,
`miniz_oxide`; little_exif → `brotli`, `crc`, `log`, `miniz_oxide`, `paste`,
`quick-xml`. `cargo deny check licenses` (DEC-018, all-features) passes with no new
exception.

## Alternatives Considered

- **Option A: `rexiv2` (native gexiv2) for all metadata**
  - What it is: mature C-backed EXIF/IPTC/XMP/ICC library.
  - Why rejected: native/system dependency breaks the pure-Rust, zero-system-deps
    default (DEC-004, constraint `pure-rust-codecs-default`). Stays an *optional*
    off-by-default feature, not the default write path (DEC-003 Option B).

- **Option B: `img-parts` alone (segment surgery for GPS too)**
  - What it is: hand-parse the EXIF TIFF/IFD structure inside the APP1 segment to
    excise only GPS, using `img-parts` for everything.
  - Why rejected: re-implements an EXIF tag parser/writer we'd have to maintain;
    `little_exif` already does tag-level edits correctly (probe-verified). Use the
    right tool per altitude — segments for `strip`, tags for `clean`.

- **Option C: `little_exif` alone (no `img-parts`)**
  - What it is: use `little_exif`'s `clear_metadata`/`reduce_to_a_minimum` for
    `strip` too.
  - Why rejected: `little_exif` is EXIF-centric; a true `strip` must remove **all**
    container metadata (XMP/ICC/IPTC/comments), which is a segment/chunk concern.
    `img-parts` removes them structurally without interpreting EXIF.

- **Option D (chosen): `img-parts` for `strip`, `little_exif` for `clean`**
  - What it is: the DEC-003 division — segment-level removal via `img-parts`,
    tag-level GPS removal via `little_exif`; both pure-Rust, both permissive.
  - Why selected: matches each operation's natural altitude, keeps the build
    pure-Rust, and the probe confirms both preserve pixels exactly.

## Consequences

- **Positive:** Unlocks the verifiable-privacy axis (`strip` + `clean --gps`) with
  fast, lossless, no-re-encode metadata edits. Pure-Rust → CI stays trivial,
  `just deny` green, no new C/system dep. Probe de-risks the MVP's riskiest area.
- **Negative:** Two more crates to track (security/licensing). `little_exif` is
  comparatively young; its quirks (e.g. the "No EXIF data found" error) become our
  edge cases. v1 format coverage is **JPEG + PNG only** (see SPEC-026); WebP/TIFF
  clean is deferred to a later spec even though `little_exif` nominally supports
  them.
- **Neutral:** `img-parts` and `little_exif` both pull `miniz_oxide` (already in the
  tree via `image`/png) — no new compression backend.

## Validation

Right if: `strip` produces a file with no EXIF/ICC/XMP and `clean --gps` removes
only GPS while preserving orientation/copyright, both with **decode-identical
pixels** and **no pixel re-encode**, across JPEG + PNG (SPEC-026 failing tests).
Revisit if: the crates can't reliably preserve **ICC across a pixel-lane encode**
(the still-open `metadata-icc-coverage` question — this DEC only covers the
metadata-only commands, not encode-time preserve), or if WebP/TIFF coverage proves
unworkable when we extend the lane (then narrow scope or promote optional
`rexiv2`).

## References

- Related specs: SPEC-026 (`strip` + `clean --gps`); STAGE-004 metadata backlog
- Related decisions: DEC-003 (metadata dual-lane — governing), DEC-004 (pure-Rust
  codec/feature policy), DEC-018 (permissive license policy / `cargo deny`)
- Constraints: `no-new-top-level-deps-without-decision`, `metadata-not-via-pixel-encode`,
  `pure-rust-codecs-default`, `no-agpl-default-deps`
- Open question: `metadata-icc-coverage` in `/guidance/questions.yaml` (partially
  retired for strip/clean by this DEC's probe; encode-time ICC-preserve still open)
- External docs: https://docs.rs/img-parts/0.4.0, https://docs.rs/little_exif/0.6.23
