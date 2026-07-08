---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-016
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-009
repo:
  id: crustyimg

created_at: 2026-07-07
shipped_at: null

value_contribution:
  advances: >
    Delivers the project's headline — reading AVIF from the default, pure-Rust,
    zero-system-dep binary — which immediately extends every shipped command to
    AVIF inputs and gives the optimize/format-decision engine a new candidate.
  delivers:
    - "`.avif` decodes to the canonical `Image` in the DEFAULT build (no dav1d/C, no system libs)"
    - "`.avif` is discovered by directory/glob sources and flows through optimize/convert/info"
    - "A recorded decoder-dependency decision (DEC-053) proving the pure-Rust, permissive, patent-clean path"
  explicitly_does_not:
    - "Add AVIF animation or grid/tiled multi-image decode (single primary image only)"
    - "Change AVIF OUTPUT (still the off-by-default `avif`/ravif encode feature, DEC-020)"
    - "Add format-preservation bias to the decide engine (possible follow-up, not here)"
    - "Pull dav1d or any C/system dependency onto the default path"
---

# STAGE-016: AVIF decode as a default, pure-Rust input

## What This Stage Is

The stage that lets the default crustyimg binary **read AVIF**. Today AVIF is output-only
and off-by-default, because `image` 0.25's AVIF *decode* path pulls **dav1d (a C system
dep)** — unacceptable for the pure-Rust default (DEC-004). This stage adds a **permissive,
pure-Rust AVIF decoder** so `crustyimg optimize photo.avif -o out.webp` (and `convert`,
`info`, batch) just work in the default build with no system libraries. AVIF/AV1 is
**patent royalty-free** (AOMedia grant), so — unlike HEIC (DEC-052) — it belongs on the
default path, not behind a feature. The format-detection seam already dispatches by magic
bytes; the work is choosing + wiring the decoder and admitting `.avif` to the source
allow-list.

## Why Now

- **Highest-leverage, patent-clean input.** AVIF is the most-requested modern format and the
  one we can ship by default without a patent asterisk. It makes every shipped command more
  useful overnight and feeds the optimize engine a new candidate.
- **Foundational.** The ISOBMFF/container handling here (if the chosen decoder needs box
  parsing) is reusable for RAW-CR3 preview (STAGE-018) and a future permissive HEIC path.

## Success Criteria

- `Image::load("x.avif")` / `Image::from_bytes(..)` decode a real AVIF to pixels in the
  **default** build (no `avif`/dav1d/system-lib features), honoring the DEC-034 decode caps.
- `optimize`/`convert`/`info` operate on `.avif` inputs end to end; directory/glob sources
  discover `.avif`; a corrupt/oversize AVIF surfaces a typed `ImageError` (not a panic).
- **No C/system dependency on the default path**; `just deny` green (permissive license, no
  new copyleft/exception); the lean `--no-default-features` build still succeeds.
- A **DEC-053** records the decoder-dependency choice from a design-time probe (candidates,
  license, pure-Rust/patent verification, and why it beats the dav1d path).

## Scope

### In scope
- Pick + wire a permissive pure-Rust AVIF decoder; admit `.avif` to `IMAGE_EXTENSIONS`;
  decode-cap + typed-error coverage. **(SPEC-058)**

### Explicitly out of scope
- AVIF animation / grid-tiled multi-image; AVIF output changes; decide-engine format
  preservation; SVG/RAW/HEIC (later stages).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-058 (design) — AVIF decode as a default input: DEC-053 decoder-dependency probe +
  wiring into `decode_with_limits` + `IMAGE_EXTENSIONS`, decode-cap + typed-error + `just deny`
  coverage. **(build-ready)**

**Count:** 0 shipped / 0 active / 1 pending (SPEC-058 build-ready; split only if the probe shows the decoder needs substantial container glue)

## Design Notes

- **PROBE RESULT (2026-07-07) — this stage is real work, not a feature toggle.** A design-time
  probe found **no mature, permissive, pure-Rust AVIF decoder with a usable Rust API + container
  handling**: `image`'s avif decode = dav1d (C) and the pure-Rust path (image-rs #2621) is
  **unmerged**; `rav1d`/`re_rav1d` (BSD-2) are pure-Rust but expose a **C-style API** + need AVIF
  container parsing; `rav1d-safe`/`zenavif` (clean Rust API) are **AGPL — excluded**; `avif-decode`
  uses AOM (C). See `guidance/license-watchlist.yaml` → `avif-decode` and SPEC-058's probe section.
- **DEC-053 picks one of three paths:** (a) `rav1d`/`re_rav1d` (BSD-2) + our own AVIF/ISOBMFF
  container parser (reuse the HEIC-spike box-parse tech) + a no-asm pure-Rust build — keeps the
  default pure-Rust, **split SPEC-058 into a container-parse + a decode spec**; (b) feature-gate a
  C decoder (dav1d/aom) off the default — acceptable since AVIF is patent-clean, but not the
  pure-Rust default headline; (c) wait for image-rs #2621 / a re_rav1d Rust API (then it's nearly a
  free `image` bump). Any new crate is a top-level dep → DEC-053 also covers `no-new-top-level-deps`.
- **Open sequencing question:** because AVIF-decode is no longer a quick win, whether it still
  *leads* Wave 1 (vs SVG via `resvg`, which IS a clean pure-Rust drop-in) is a maintainer call.
- **Patent contrast to record:** AV1/AVIF is royalty-free (no HEVC-style pool) — this is *why*
  it can be default where HEIC cannot (DEC-052).
- **Wiring is small once the decoder exists:** format dispatch is automatic in
  `decode_with_limits` (`ImageReader::with_guessed_format`); the deltas are the `Cargo.toml`
  feature/dep, one line in `IMAGE_EXTENSIONS`, and decode-cap/error tests.

## Dependencies

### Depends on
- Shipped decode seam (`src/image/mod.rs:271`, `src/source/mod.rs:94`, `src/error.rs`).
- DEC-004 (pure-Rust default), DEC-034 (decode caps), DEC-018 (`no-agpl-default-deps`).

### Enables
- STAGE-017/018 (shared container/decoder patterns), roadmap Wave 3 (in-browser AVIF), and
  richer optimize/lint coverage.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
