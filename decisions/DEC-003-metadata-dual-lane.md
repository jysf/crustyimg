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

## Amendment (2026-08-04, SPEC-110 / DEC-086)

**The orientation half of this record's Validation and Consequences no longer describes
the code, and is corrected here rather than silently drifting further.**

The Validation line above — *"a resize preserves orientation"* — and the Consequences
line — *"Orientation/ICC survive transforms"* — were never true of `convert`, `thumbnail`,
`edit`, or `responsive` (they drop the tag AND leave the pixels un-rotated), and are no
longer true of `resize` either. SPEC-110's design measured this directly: driving a
1200×800, `Orientation=6` fixture through every pixel-lane verb showed seven of eleven
invocations returning a sideways image with no EXIF to correct it by hand — the tag was
destroyed by the same re-encode that made the output wrong.

**Corrected claim:** on pixel-lane encodes, EXIF **orientation is baked into pixels, not
preserved as a tag** (DEC-086). ICC and copyright/artist remain the **preserve** claim this
record originally made — unaffected by SPEC-110, which touches only orientation. GPS is
still dropped by default unless `--keep-gps`, as originally decided. **This amendment does
not open a new investigation into whether ICC/copyright preservation actually holds on the
pixel lane today** — SPEC-110's scope was orientation only (`one-spec-per-pr`); if that
sweep surfaces evidence ICC is also being silently dropped against this record's claim, it
is a separate finding, filed as its own spec, not fixed here.

`AGENTS.md`'s "Default-preserve policy" glossary entry is corrected to match (§14).

## References

- Related specs: STAGE-004 backlog (metadata commands); SPEC-002 (capture metadata at load)
- Related decisions: DEC-002 (pixel lane), DEC-004 (codec/feature policy)
- External docs: https://docs.rs/kamadak-exif, https://docs.rs/img-parts, https://docs.rs/little_exif
- Open question: `metadata-icc-coverage` in `/guidance/questions.yaml`
