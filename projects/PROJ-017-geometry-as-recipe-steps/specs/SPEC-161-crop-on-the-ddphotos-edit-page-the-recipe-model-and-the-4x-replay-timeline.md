# SPEC-161 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-161-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-22, Opus) — SPLIT; this half GO at [M]; DEC-116 (replay parity) recommended. Premise confirmed: crop runs byte-identical through native `apply` and slim wasm `transform`;
  `just ddphotos-parity` 16 rows green. Driven: `info()` reads 1600x1071 for an Orientation 6 JPEG (output
  1071x1600); copy-vs-original size bound 0 px at exact 4x, 1 px from 2x, 2 px below; `aspect` misses at 4x, so presets emit a box.
- [x] **rulings** (2026-11-22, maintainer) — split yes; interaction yes; presets emit `aspect` (reversed: readable and
  reusable), a dragged box emits a box; the orientation key stays in STAGE-060 ahead of this spec (SPEC-163, shipped
  2026-11-26); the replay DEC yes, number allocated at design.
- [x] **design** (2026-11-26, Opus) — `prompts/SPEC-161-design.md`. [M] kept: 35 crop rows, 17 replay rows,
  model/placement/golden checks, N4 (one pixel of crop), N5 (replay framing shift); DEC-118 allocated. A
  preset emits `aspect` (+ key unless square or 1:1), a box emits fractions; `loadPhoto` in recipe.js gives
  the oriented size; no Rust, 0 B wasm. Corrections: box not exact at a resize tie; aspect worst 1 px (exhaustive).
- [ ] **build** — `prompts/SPEC-161-build.md`
- [ ] **verify**
- [ ] **ship**
