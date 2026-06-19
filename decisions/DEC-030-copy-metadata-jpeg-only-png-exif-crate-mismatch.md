---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-030
  type: decision
  confidence: 0.85
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

affected_scope:
  - src/metadata/**

tags:
  - architecture
  - metadata
  - exif
  - png
  - scope
---

# DEC-030: `copy-metadata` is JPEG-only in v1 (pure-Rust PNG EXIF crate mismatch)

## Decision

The `copy-metadata --from SRC --to DST` command (SPEC-028) copies a container's
**EXIF + ICC** from SRC onto DST via `img-parts`' `ImageEXIF`/`ImageICC` traits
(`exif()`/`set_exif`, `icc_profile()`/`set_icc_profile`), with DST's pixels
preserved exactly. **v1 supports JPEG only**; a non-JPEG `--from`/`--to` exits **4**.
PNG `copy-metadata` is deferred because our two pure-Rust metadata crates write PNG
EXIF into **incompatible chunks**.

## Context

`strip`/`clean`/`set` (SPEC-026/027) cover JPEG **and** PNG because each stays
inside a single crate's convention: `strip` removes chunks structurally
(`img-parts`), `clean`/`set` read+write EXIF entirely within `little_exif`. A design
-time probe for `copy-metadata` exposed a cross-crate conflict that only bites when
the two crates must interoperate on PNG EXIF:

- **`little_exif`** writes PNG EXIF as a **`zTXt` "Raw profile type exif"** text
  chunk (the ImageMagick convention) — even with `FileExtension::PNG { as_zTXt_chunk:
  false }` in 0.6.23.
- **`img-parts`** reads/writes PNG EXIF as the **native `eXIf` chunk** (PNG 1.5+).
- Consequence: `img-parts`' `Png::exif()` returns `None` for a `little_exif`-written
  PNG (and vice-versa). A PNG `copy-metadata` built on `img-parts` traits therefore
  silently transfers nothing for the common (our-own-tooling) case.

The same probe confirmed **JPEG works cleanly**: both crates agree on the APP1
`Exif\0\0` segment, so `dst.set_exif(src.exif())` + `dst.set_icc_profile(
src.icc_profile())` transfers EXIF + ICC with **byte-identical decoded pixels** (the
compressed scan is carried verbatim — `metadata-not-via-pixel-encode` holds).

## Alternatives Considered

- **Option A: ship PNG copy-metadata anyway via `img-parts` traits**
  - Why rejected: silently no-ops for PNGs whose EXIF is in a `zTXt` chunk (incl.
    everything our own `set` writes) — a correctness trap worse than an honest exit 4.

- **Option B: bridge the conventions now (read `zTXt` "Raw profile type exif",
  re-emit as `eXIf`, or copy at the chunk level)**
  - Why rejected: real work (hex-decode the ImageMagick text profile, or do
    chunk-level surgery handling both conventions) for the least-common case
    (PNG-with-EXIF copy). Out of scope for finishing the lane; tracked as follow-up.

- **Option C (chosen): JPEG-only v1, exit 4 on non-JPEG, defer PNG**
  - Why selected: JPEG is the dominant EXIF container and is probe-verified clean;
    an explicit exit 4 is honest; the PNG bridge is a well-scoped follow-up.

## Consequences

- **Positive:** Finishes the metadata lane's last command correctly for the common
  case; no silent data loss; pure-Rust; no new dep / no `just deny` change.
- **Negative:** Coverage asymmetry — `strip`/`clean`/`set` do JPEG+PNG but
  `copy-metadata` does JPEG only. Documented in `docs/api-contract.md` + the spec.
- **Neutral:** v1 copies EXIF + ICC (what `img-parts` exposes via traits); XMP/IPTC
  segment transfer is not included (separate deferral, noted in SPEC-028).

## Validation

Right if: `copy-metadata --from a.jpg --to b.jpg` gives `b` `a`'s EXIF + ICC with
`b`'s pixels byte-identical, and any non-JPEG input exits 4 with a clear message
(SPEC-028 failing tests). Revisit when: we add the PNG bridge (decode the `zTXt`
"Raw profile type exif" or do `eXIf`/`zTXt`-aware chunk copy) — then widen to PNG and
update this DEC + `metadata-icc-coverage`.

## References

- Related specs: SPEC-028 (`copy-metadata`); SPEC-026/027 (the lane it completes)
- Related decisions: DEC-003 (metadata dual-lane), DEC-029 (pinned `img-parts` +
  `little_exif`)
- Constraints: `metadata-not-via-pixel-encode`, `pure-rust-codecs-default`
- Open question: `metadata-icc-coverage` (this narrows it further — JPEG EXIF/ICC
  transfer confirmed; PNG EXIF cross-crate bridge still open)
- External docs: https://docs.rs/img-parts/0.4.0 (ImageEXIF/ImageICC),
  https://docs.rs/little_exif/0.6.23
