---
insight:
  id: DEC-003
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

created_at: 2026-06-13
supersedes: null
superseded_by: null

affected_scope:
  - src/metadata/**
  - src/sink/**

tags:
  - architecture
  - metadata
  - exif
  - privacy
---

# DEC-003: Metadata dual-lane (pixel lane vs container lane) + default-preserve policy

## Decision

Metadata is handled in a **separate lane** from pixels. The **pixel lane**
(decode → ops → encode via `image`) inherently drops metadata. The
**container lane** edits/preserves metadata at the container level without
re-decoding pixels: read via `kamadak-exif` (read-only), edit/preserve via
`img-parts` (EXIF/ICC segments) and `little_exif` (tag write). Metadata-only
commands (`strip`, `clean --gps`, `set`, `copy-metadata`) never go through
the pixel encode path. **Default-preserve policy** on pixel-lane encodes:
keep orientation + ICC + copyright/artist; **drop GPS** unless `--keep-gps`.

## Context

The `image` crate discards all metadata on encode, so any pixel
transformation silently strips EXIF/ICC/XMP. Users expect orientation and
color profile to survive a resize, expect copyright to be retained, and —
for privacy — expect location data to be dropped by default when publishing
for the web. Forcing metadata edits through a pixel re-encode would
needlessly decode/re-encode (slow, lossy for JPEG) and is architecturally
wrong (feature-exploration.md § "Metadata dual-lane").

## Alternatives Considered

- **Option A: Treat metadata as `Operation`s in the pixel pipeline**
  - Why rejected: every metadata edit would re-decode/re-encode pixels
    (slow, JPEG-lossy), and conflates two unrelated concerns. DEC-002 keeps
    `Operation` for pixels only.

- **Option B: `rexiv2` for everything (native gexiv2)**
  - Why rejected: native dependency breaks pure-Rust-by-default CI (DEC-004).
    Kept as an optional feature, not the default.

- **Option C (chosen): two lanes; pure-Rust container crates by default; preserve policy**
  - Why selected: pixel and metadata work stay independent; metadata edits
    skip decode entirely; the preserve policy gives sane, privacy-aware
    defaults without surprising the user.

## Consequences

- **Positive:** Fast, lossless metadata edits. Privacy-by-default (GPS
  dropped). Orientation/ICC survive transforms. Clean separation.
- **Negative:** Two code paths to maintain. Pure-Rust metadata crates
  (`img-parts`, `little_exif`) are less battle-tested than `rexiv2`; format
  coverage for the preserve/transfer of ICC across all formats is the
  riskiest part of the MVP (reflected in the 0.8 confidence and PROJ-001
  risks_to_thesis).
- **Neutral:** `kamadak-exif` is read-only, so writes must use a different
  crate — accepted division of labor.

## Validation

Right if: a resize preserves orientation + ICC + copyright and drops GPS by
default; `clean --gps` removes only location; `strip` removes everything;
`copy-metadata` transfers across two files — all with byte/tag-level tests
and no pixel re-encode for metadata-only commands. Revisit if: pure-Rust
container crates can't preserve ICC reliably across core formats (then
promote `rexiv2` from optional to a recommended feature, or narrow the
preserve set).

## References

- Related specs: STAGE-004 backlog (metadata commands); SPEC-002 (capture metadata at load)
- Related decisions: DEC-002 (pixel lane), DEC-004 (codec/feature policy)
- External docs: https://docs.rs/kamadak-exif, https://docs.rs/img-parts, https://docs.rs/little_exif
- Open question: `metadata-icc-coverage` in `/guidance/questions.yaml`
