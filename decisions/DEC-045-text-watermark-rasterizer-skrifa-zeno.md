---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-045
  type: decision
  confidence: 0.9
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

created_at: 2026-07-04
supersedes: null
superseded_by: null

affected_scope:
  - src/text/mod.rs
  - Cargo.toml
  - deny.toml

tags:
  - architecture
  - watermark
  - text
  - fonts
  - dependencies
  - license
  - security
  - advisory
---

# DEC-045: Text-watermark rasterizer moves to `skrifa` + `zeno` (drops `ttf-parser`)

## Decision

`watermark --text` rasterizes glyphs with **`skrifa` `=0.44.0`** (glyph outlines +
metrics, from Google's `fontations` project) piped into **`zeno` `=0.3.3`** (the
anti-aliased mask rasterizer), replacing **`ab_glyph` `=0.2.32`**. Both new crates are
pure-Rust and **MIT OR Apache-2.0**; neither pulls `ttf-parser`. This removes the
unmaintained `ttf-parser` from the dependency tree and lets us **delete the
`RUSTSEC-2026-0192` ignore** from `deny.toml`.

Everything else about text watermark is **unchanged**: the same public API
(`render_text(font_bytes, text, size_px, color) -> RgbaImage`, `parse_color`,
`DEFAULT_FONT`, `TextError`), the same bundled BSD-3 Go font (DEC-032), the same
single-line layout and the same transparent-RGBA-overlay composited through the SPEC-029
`Watermark` path. This decision changes **only the rasterizer**, not the feature.

This **supersedes the rasterizer choice in DEC-032** (`ab_glyph`); DEC-032's font choice
(bundled Go-Regular) and IO-boundary rules (DEC-031) stand.

## Context

DEC-032 chose `ab_glyph` on 2026-06-18 and noted at the time that its `ttf-parser`
transitive dep kept `cargo deny` green. Ten days later, **2026-06-28**, the `ttf-parser`
author declared the crate end-of-life and **RUSTSEC-2026-0192** ("`ttf-parser` is
unmaintained") was published. crustyimg shipped v0.1.0/v0.1.1 with a documented ignore for
it (DEC-042). The STAGE-010 fast-follow is to eliminate that ignore at the source.

**Why not `fontdue` (the original backlog plan):** the backlog named `fontdue` as a
pure-Rust rasterizer with "its OWN parser — no ttf-parser". A design-time probe disproved
that for the current release: **fontdue `0.9.3` depends on `ttf-parser` `0.21.1`**
(`cargo tree` confirmed). RUSTSEC-2026-0192 is **crate-wide** — the advisory file has
`package = "ttf-parser"`, `patched = []`, `informational = "unmaintained"` — so it flags
*every* version. Swapping to fontdue would keep the ignore and change the rasterizer for no
supply-chain gain: a pure loss. The advisory's own recommended alternative is **`skrifa`**
(fontations), which is `ttf-parser`-free.

**Design-time probe (on the real bundled `Go-Regular.ttf`):** `skrifa` +
`zeno` reproduce the exact metrics `ab_glyph`/`fontdue` report — ascent `30.234375`,
`advance_width('H') = 23.109375`, glyph 'H' bounds `19×24` at `xmin=2` — and `zeno`'s
`Mask::render()` returns a `(Vec<u8>` coverage `, Placement)` pair that is a direct analog
of `ab_glyph`'s `outline_glyph().px_bounds()` + `draw()` closure. Glyph outlines flow
through a small `OutlinePen` impl that pushes `zeno::Command`s (negating y: `skrifa` emits
y-up font space, the raster is y-down). The confirmed calls are recorded in SPEC-044.

**Kerning:** `ab_glyph`'s `sf.kern()` reads only the legacy `kern` table; the bundled
Go-Regular has **no** legacy `kern` table (the fontdue probe's `horizontal_kern` returned
`None` for every pair), so `ab_glyph` was already applying zero kerning to the default
font. `skrifa` has no equivalent one-call pair lookup (kerning lives in GPOS and needs
shaping). We therefore **drop pairwise kerning** — a verified nil visual change for the
bundled font and a sub-pixel spacing change only for a rare user `--font` that carries a
legacy `kern` table. Shaping (harfrust/rustybuzz) is out of scope for a short watermark.

## Alternatives Considered

- **`fontdue` `0.9.3`** — rejected: still pulls `ttf-parser 0.21.1`; does not remove the
  `-0192` ignore (the whole reason for the swap). Probe-confirmed.
- **`swash` `0.2.9`** — a higher-level wrapper over `skrifa`+`zeno`; also `ttf-parser`-free
  and permissive. Rejected for SPEC-044: it adds `yazi` (woff2 inflate) we don't need and
  hides the outline→mask path we already hand-roll. `skrifa`+`zeno` is the leaner, more
  direct pairing for our existing coverage-compositing code.
- **Keep `ab_glyph`, keep the `-0192` ignore** — rejected: `-0192` is a real (if
  informational) advisory and the agreed 0.2.0 goal is an empty ignore list. Input being
  trusted (bundled/explicit font) lowers risk but doesn't clear the exception.
- **Read the legacy `kern` table directly via `read-fonts`** to preserve exact `ab_glyph`
  behavior — rejected as unwarranted: zero effect on the default font, and it re-adds
  complexity to save a sub-pixel edge case.
- **Downgrade `cargo deny` advisories to `warn`** — rejected in DEC-042 and still rejected
  (would swallow future real advisories).

## Consequences

- **Positive:** `ttf-parser` leaves the tree; the `RUSTSEC-2026-0192` ignore is deleted;
  the rasterizer is now backed by an actively-maintained, Google-supported font stack
  (`fontations`). Pure-Rust, permissive, `just deny` green with one fewer exception.
- **Negative:** the dependency set changes from `ab_glyph` (+`ab_glyph_rasterizer`,
  `owned_ttf_parser`, `ttf-parser`) to `skrifa` + `zeno` (+`read-fonts`, `font-types`,
  `bytemuck`, `once_cell`) — a comparable, all-permissive footprint. Rasterizer
  anti-aliasing differs at the sub-pixel level (not user-observable for a watermark).
  Pairwise kerning is dropped (nil impact on the bundled font; see Context).
- **Neutral:** the bundled Go font, the `--font PATH` override, `--size`/`--color`/
  `--opacity`, and the public `src/text` API are all unchanged.

## Validation

Right if: after the swap, `cargo tree` shows no `ttf-parser`, `deny.toml` has no `-0192`
entry, `just deny` passes, the existing `src/text` tests plus SPEC-044's new tests pass,
`watermark --text "..."` still rasterizes legible text at the gravity anchor, and the lean
(`--no-default-features`) build still compiles. Revisit if: we later want real shaping /
kerning / multi-line typography (add `harfrust`/`rustybuzz` in a dedicated spec), or if
binary size warrants feature-gating the font + rasterizer.

## References

- Related specs: SPEC-044 (this swap); SPEC-030 (text watermark — the code being
  re-based); SPEC-029 (`watermark` image overlay — the reused compositing path)
- Related decisions: **DEC-032** (superseded rasterizer choice; font choice retained),
  DEC-042 (accepted the `-0192` ignore this eliminates), DEC-031 (IO boundary), DEC-004
  (pure-Rust dep policy), DEC-018 (permissive license / `cargo deny`)
- Constraints: `no-new-top-level-deps-without-decision`, `pure-rust-codecs-default`,
  `no-agpl-default-deps`
- Advisory: RUSTSEC-2026-0192 (`ttf-parser` unmaintained; `patched = []`; recommends
  `skrifa`)
- External docs: https://docs.rs/skrifa/0.44.0, https://docs.rs/zeno/0.3.3,
  https://github.com/googlefonts/fontations
