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
- [x] **design rulings** (2026-11-26, maintainer) — both design calls accepted: no `orientation` key on a square
  photo or `1:1`; a swapped preset writes the aspect as shown with no key.
- [x] **build** (2026-11-26, Sonnet) — `prompts/SPEC-161-build.md`. Draft PR #273 at `f9c81ae`, CI green
  (17 pass). 13 of 13 ACs; 35 crop, 17 replay rows; lowest PSNR R9 53.95 (aspect), R12 59.46 (box); N1–N5
  red as listed; M1–M11 and trap 14 match. Cells to correct: N4 reads `max` not `dims` on four rows; M4's
  C30/R12/R15 throw. 6 files, 0 B wasm. $6.13.
- [x] **verify** (2026-11-26, Opus) — `prompts/SPEC-161-verify.md`. PUNCH LIST at `f9c81ae`, no claim items, no
  re-verify. 52 literals and 22,908 model cases match; 1,854,856 readback reproduced. Fix: DEC-118 rules 1 and 4
  wording, one DEC id in a comment, AC ticks and reflection; asserts for `Math.max(1, …)`, an R row M4 hits by size
  (maintainer: include), counted summary line (maintainer: include). Ship: M4/N4/M5 cells, AC-12 wording, the N3
  leftover folder to `aspect-orientation-loose-ends`, rule 5's CLI-panel note to the crop-mode entry. $7.60.
- [ ] **punch list** — `prompts/SPEC-161-punchlist.md`
- [ ] **ship**
