---
# Maps to ContextCore project.* semantic conventions.
project:
  id: PROJ-009
  status: active                    # proposed | active | shipped | cancelled
  priority: high
  target_ship: null

repo:
  id: crustyimg

created_at: 2026-07-07
shipped_at: null

value:
  thesis: >
    Broaden what crustyimg can READ so the already-shipped engine (optimize /
    convert / lint / info / responsive) applies to the modern-format assets
    developers actually have — AVIF, SVG, camera RAW — from the pure-Rust,
    zero-system-dep default where the format and its patents allow, and honestly
    feature-gated where they don't (HEIC). AVIF decode is the headline: it is
    patent royalty-free, pure-Rust-attainable, and immediately makes the shipped
    tools more useful while also giving the optimize/format-decision engine a new
    input to reason about.
  beneficiaries:
    - "Web/content developers who already produce or receive AVIF and SVG assets"
    - "Photographers/prosumers with camera RAW who want a quick web derivative"
    - "The shipped optimize engine (a new input format to ingest/convert)"
    - "Adoption (a concrete, honest 'it reads modern formats' story for the demo/HN)"
  success_signals:
    - "`crustyimg optimize photo.avif -o out.webp` works in the DEFAULT build (no system deps)"
    - "`.svg` rasterizes to a raster pipeline input; `.raw/.nef/.cr3/...` yields the embedded preview"
    - "`.heic` decodes only under `--features heic` (never the default binary) — DEC-052"
    - "Every default input path stays pure-Rust + zero-system-dep; `just deny` green; lean build green"
    - "AVIF/SVG/RAW inputs are discovered by directory/glob sources like any other image"
  risks_to_thesis:
    - "AVIF-decode pure-Rust dependency maturity (rav1d/re_rav1d) — the load-bearing probe (DEC-053)"
    - "RAW breadth is a corpus problem (which cameras/models); Tier-1 preview is deliberately narrow"
    - "Scope creep into full RAW development (demosaic) or AVIF animation/grid — explicitly out"
    - "Untrusted-input parsers (AVIF/SVG/RAW/HEIF containers) widen the attack surface — fuzz + caps"
---

# PROJ-009: Input reach (modern-format decode)

## What This Project Is

The **input-reach wave** — roadmap Wave 1 (`docs/roadmap.md`). crustyimg already
*writes* modern formats and runs a strong optimization/lint engine; this wave broadens
what it can *read* so that engine applies to AVIF, SVG, and camera RAW from the default
pure-Rust binary, plus HEIC as an honest opt-in. The default experience gains
**AVIF decode** (patent-free, pure-Rust), **SVG rasterize**, and **RAW embedded-preview
extraction** with zero system dependencies; **HEIC** is feature-gated real decode only
(DEC-052 — AGPL wall + HEVC patents). The headline is AVIF decode, both because it is the
most-requested modern format and because it feeds the shipped format-decision engine as a
new candidate input.

## Why Now

Reconciled adoption-first roadmap (2026-07-07) pulled input reach to first. It is the
highest-leverage way to make everything already shipped more useful, and — unlike the
"iPhone photos just work" HEIC headline that the spike retired (DEC-052) — the AVIF/SVG/RAW
default path is permissive, pure-Rust, and (for AVIF/AV1) **patent royalty-free**, so it can
ship on by default without asterisks. The HEIF/ISOBMFF container work started here (for AVIF
and RAW-CR3 preview) also lays reusable groundwork for a future permissive HEIC path.

## Success Criteria

- Default build reads `.avif` and `.svg`, and extracts the embedded preview from common RAW,
  end to end through `optimize`/`convert`/`info` — with no system libraries and `just deny` green.
- HEIC decode is available only under `--features heic` and returns exit 4 ("codec not built")
  in the default binary — matching DEC-004/DEC-052.
- New inputs are discovered by directory/glob sources and flow through the pipeline unchanged.
- No regression to the lean (`--no-default-features`) build; untrusted-input hardening upheld
  (decode caps, fuzz targets for any new container parser).

## Scope

### In scope
- AVIF **decode** as a default input (pure-Rust decoder; DEC-053 to pick it). **(STAGE-016)**
- SVG **rasterize** as an input (`resvg`, MIT). **(STAGE-017)**
- RAW **Tier-1 embedded-preview** extraction (permissive, no RAW codec). **(STAGE-018)**
- HEIC decode **feature-gated** (`--features heic`, libheif decode-only). **(STAGE-019, DEC-052)**

### Explicitly out of scope
- Full RAW **development** (demosaic/white-balance — LGPL `rawler`, watchlist Tier-2).
- HEIC in the **default** binary (DEC-052 — patents + AGPL); AVIF **animation/grid**; JPEG XL (post-1.0).
- Format-**preservation bias** in the decide engine (a possible follow-up spec, not required here).
- New OUTPUT formats (this wave is about reading, not new encoders).

## Stage Plan

Format: `- [status] STAGE-ID — one-line summary`

- [x] STAGE-016 (shipped on 2026-07-07) — AVIF decode as a default, pure-Rust input (SPEC-058, PR #65, DEC-053).
- [x] STAGE-017 (shipped on 2026-07-08) — SVG rasterize input via the `resvg`/`usvg`/`tiny-skia` stack
  (all permissive: Apache-2.0 OR MIT / BSD-3-Clause — **no deny license exception**; one RUSTSEC-2026-0192
  advisory ignore), default build (SPEC-060, PR #66, DEC-054).
- [~] STAGE-018 (active — framed 2026-07-08) — RAW Tier-1 embedded-preview extraction (permissive, **no new
  dep**): a format-agnostic byte scan for the largest embedded JPEG covers TIFF-based RAW + CR3 + RAF with
  no ISOBMFF/IFD parsing (probe finding corrects the "ISOBMFF glue" assumption); extension-routed in
  `Image::load` (SPEC-061, DEC-055). **← active**
- [ ] (not yet framed) STAGE-019 — HEIC decode behind `--features heic` (libheif decode-only; DEC-052). **← next**

**Count:** 2 shipped / 1 active / 1 pending (STAGE-016 + 017 shipped; 018 active/framed, then 019)

## Dependencies

### Depends on
- Shipped decode seam: `src/image/mod.rs` (`decode_with_limits`), `src/source/mod.rs`
  (`IMAGE_EXTENSIONS`), `src/sink/mod.rs` (`ensure_codec_built`/exit-4), `src/error.rs`.
- DEC-004 (pure-Rust default + feature-gated native), DEC-034 (decode caps), DEC-052 (HEIC gating).
- External: a permissive pure-Rust AVIF decoder (DEC-053, decided at STAGE-016 build after a probe).

### Enables
- The demo/WASM wave's in-browser AVIF/SVG conversion (roadmap Wave 3).
- A future permissive HEIC path (the ISOBMFF container parser is shared groundwork).
- Richer optimize/lint coverage over modern-format asset trees.

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <yes/no + notes>
- **How many stages did it actually take?** <number, compare to plan>
- **What changed between starting and shipping?** <one or two sentences>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **What did we defer to the next project?**
  - <one-line items>
