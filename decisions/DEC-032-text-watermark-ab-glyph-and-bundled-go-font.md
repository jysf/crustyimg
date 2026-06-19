---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-032
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
  - src/operation/**
  - src/cli/mod.rs
  - Cargo.toml
  - assets/fonts/**

tags:
  - architecture
  - watermark
  - text
  - fonts
  - dependencies
  - license
---

# DEC-032: Text watermark via `ab_glyph` + a bundled BSD-3 Go font (no `imageproc`)

## Decision

`watermark --text` renders text with **`ab_glyph` `=0.2.32`** (Apache-2.0, pure-Rust)
— the rasterizer only; layout (advance/kern/ascent) + glyph coverage are hand-rolled.
The rendered text becomes a transparent **RGBA overlay** that is composited onto the
base through the **existing SPEC-029 `watermark` path** (`Gravity::placement` + the
`Watermark` op), so gravity/opacity/margin behave identically to the image overlay.
The **default font is bundled**: `assets/fonts/Go-Regular.ttf` (Go Regular, Bigelow &
Holmes, **BSD-3-Clause**, ~145 KB) embedded with `include_bytes!`; `--font PATH`
overrides it. We deliberately do **NOT** use `imageproc`.

## Context

Text watermark is the last STAGE-004 command. It needs (a) a glyph rasterizer and (b)
a default font (the user chose a bundled default + `--font` override over BYO-font).
Two design-time probes settled both:

- **Dependency:** `imageproc 0.27`'s `draw_text_mut` is convenient but its dependency
  tree pulls `nalgebra`, `rand`, `rustdct`, and **`sdl2`** (a C system dep) — fatal for
  crustyimg's pure-Rust, zero-system-deps default (DEC-004). `ab_glyph` pulls only
  `ab_glyph_rasterizer` (Apache-2.0) + `owned_ttf_parser`/`ttf-parser` (MIT OR
  Apache-2.0) — all pure-Rust, all permissive; `cargo deny` stays green. A probe
  confirmed `ab_glyph` lays out + rasterizes a string (advance/kern/outline coverage)
  in a few lines.
- **Font:** a probe downloaded candidate static TTFs; **Go-Regular** (BSD-3-Clause,
  ~145 KB) beat Roboto (Apache-2.0, ~515 KB) on size for an equally clean license. The
  font + its `LICENSE-Go` are committed under `assets/fonts/`.

**`ab_glyph` `std`-feature trap:** `ab_glyph`/`ttf-parser` need `std` for float math
(`f32::tan`). Declare `ab_glyph = "=0.2.32"` (default features include `std`) and do
**NOT** set `default-features = false`. crustyimg's own `--no-default-features` (the CI
lean build) only disables crustyimg's `display` feature — it does **not** touch
`ab_glyph`'s defaults, so the lean build keeps `std` and text watermark still compiles.

**Boundary (DEC-031):** a `--font PATH` is read at the CLI IO boundary
(`run_watermark`), never inside `src/operation/**`. The bundled font is compile-time
`include_bytes!` data (a `&'static [u8]`, not file IO), so the text rasterization fn
stays pure and lives in `operation/`.

## Alternatives Considered

- **`imageproc::drawing::draw_text_mut`** — rejected: drags in `sdl2` (C dep) +
  `nalgebra`/`rand`/`rustdct`; violates the pure-Rust zero-system-deps default.
- **`fontdue`** (MIT OR Apache) — a fine pure-Rust rasterizer, but `ab_glyph` covers
  layout + rasterization cleanly and is the crate the stage plan named; no advantage
  worth a second evaluation.
- **Require `--font PATH`, no bundled default** — rejected by the user (worse
  out-of-box UX); we bundle a default and still allow `--font`.
- **Bundle Roboto (Apache-2.0)** — rejected: 515 KB vs Go's 145 KB for no functional
  gain; both licenses are clean, so size decides.

## Consequences

- **Positive:** `watermark --text "© me"` works out of the box; one `watermark`
  command does image OR text; text reuses the entire SPEC-029 compositing/gravity path
  (minimal new code). Pure-Rust, permissive, `just deny` green.
- **Negative:** +1 runtime dependency (`ab_glyph`) and **+~145 KB** in every binary
  (the embedded font, even in the lean build). Acceptable for a headline command;
  could be feature-gated later if binary size becomes a concern.
- **Neutral:** BSD-3 requires retaining the font's copyright/license — satisfied by
  `assets/fonts/LICENSE-Go` + the attribution note. A second bundled font or
  feature-gating is a clean future extension.

## Validation

Right if: `watermark --text "..."` rasterizes legible text at the gravity anchor with
`--size`/`--color`/`--opacity`/`--font` behaving as documented, `cargo deny` stays
green, and the lean build still compiles (SPEC-030 tests + CI). Revisit if: binary
size matters (feature-gate the font/`ab_glyph`), or we want richer typography
(multi-line, alignment, stroke) — a later spec.

## References

- Related specs: SPEC-030 (text watermark); SPEC-029 (`watermark` image overlay — the
  reused compositing path)
- Related decisions: DEC-031 (multi-image overlay / IO boundary), DEC-004 (pure-Rust
  codec/dep policy), DEC-018 (permissive license / `cargo deny`)
- Constraints: `no-new-top-level-deps-without-decision`, `pure-rust-codecs-default`,
  `no-agpl-default-deps`
- Assets: `assets/fonts/Go-Regular.ttf` (BSD-3), `assets/fonts/LICENSE-Go`
- External docs: https://docs.rs/ab_glyph/0.2.32, https://github.com/golang/image
  (font/gofont)
