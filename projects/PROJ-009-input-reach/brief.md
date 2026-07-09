---
# Maps to ContextCore project.* semantic conventions.
project:
  id: PROJ-009
  status: shipped                   # proposed | active | shipped | cancelled
  priority: high
  target_ship: null

repo:
  id: crustyimg

created_at: 2026-07-07
shipped_at: 2026-07-08

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
- [x] STAGE-018 (shipped on 2026-07-08) — RAW Tier-1 embedded-preview extraction (permissive, **no new
  dep**): a format-agnostic byte scan for the largest embedded JPEG covers TIFF-based RAW + CR3 + RAF with
  no ISOBMFF/IFD parsing (probe finding corrected the "ISOBMFF glue" assumption); extension-routed via a
  shared `Image::decode_path` helper (SPEC-061, PR #67, DEC-055).
- [x] STAGE-019 (shipped on 2026-07-08) — HEIC decode behind an off-by-default `heic` feature
  (system libheif, decode-only; DEC-052): default binary detects `.heic` and exits 4 ("rebuild with
  --features heic"); `--features heic` decodes via libheif-rs (MIT crates, LGPL system lib → **no deny
  exception**), never in a distributed artifact (SPEC-062, PR #68, DEC-056).

**Count:** 4 shipped / 0 active / 0 pending — **PROJ-009 complete** (all four input-reach stages shipped).

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

*Shipped 2026-07-08.*

- **Did we deliver the outcome in "What This Project Is"?** Yes. The default, pure-Rust, zero-system-dep
  binary now reads **AVIF** (STAGE-016), **SVG** (STAGE-017), and **RAW embedded previews** (STAGE-018)
  end to end through the shipped engine (optimize/convert/info/resize/batch), and **HEIC** decodes under
  an honest opt-in `--features heic` (STAGE-019) with a clear exit-4 in the default binary — exactly the
  "AVIF/SVG/RAW just work with no system deps; HEIC in the `heic` build" story, not the retired "iPhone
  photos just work" headline. Every default input path stayed pure-Rust and `just deny`-green; the lean
  build never regressed. Total recorded AI cost across the four stages ≈ 2.2M tokens / ~$20 (labelled
  estimates, `just cost-audit` green).
- **How many stages did it actually take?** 4, exactly as planned (STAGE-016–019), each a clean
  single-spec stage (SPEC-058, 060, 061, 062; SPEC-059 was folded into 058 at build).
- **What changed between starting and shipping?** The load-bearing probes repeatedly *simplified* the
  work versus the framing: AVIF's "no clean permissive drop-in" pessimism was overturned (re_rav1d +
  avif-parse); SVG's assumed MPL license was actually permissive (resvg relicensed) at the cost of one
  advisory ignore; RAW needed no ISOBMFF/IFD parsing and no new dep (a format-agnostic JPEG byte scan
  covers TIFF-RAW + CR3 + RAF); and HEIC's Rust crates were MIT (no deny exception — the LGPL is the
  system lib). The recurring surprise was environmental, not algorithmic: the license/advisory/system-lib
  *tail* of each decoder (MPL/CC0/`paste`, `ttf-parser`, ubuntu's libde265 plugin) is where the CI
  round-trips lived.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - **A new input format touches every decode caller.** A new `IMAGE_EXTENSIONS` entry or `ImageError`
    variant needs an audit of every decode caller + every `Err(_)` catch-all, not just the exit-code map
    — it bit SPEC-061 (`info <raw>`) and SPEC-062 (`lint` calling a valid `.heic` "corrupt"). The
    tripwire is a shared path-decode seam (`Image::decode_path`) + listing caller files in the DEC's
    `affected_scope`. Captured in `image-extensions-expose-every-decode-caller`.
  - **Probe the license/advisory/system tail immediately after `cargo add`** — `just deny` right away,
    and for a versioned system lib pin the API-version feature to the oldest distro package + install the
    decoder *backend* in CI (not just headers).
  - **`source_format` has no variant for SVG/RAW/HEIC** — all three report a materialized raster format
    (Png/Jpeg/Png); a faithful `SourceFormat` enum is the standing cross-cutting follow-up.
- **What did we defer to the next project?**
  - Pre-1.0 hardening: **run the four fuzz targets** (`avif/svg/raw/heic_decode`) under nightly — they
    ship but were never run (tracked in `docs/roadmap.md`).
  - RAW **Tier-2 development** (rawler, LGPL); RAW/HEIC **stdin**; HEIC **Windows** (vcpkg) + the `v1_19`
    `set_security_limits` upgrade; a **stride-padding test** with an odd-width HEIC fixture; `lint <raw>`;
    the shared `SourceFormat` enum; preview EXIF/orientation passthrough. AVIF **animation/grid**, HEIC
    in a distributed artifact, and a pure-Rust HEIC decoder remain out (DEC-052).
