# SPEC-161 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-161-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-22, Opus) — SPLIT; this half GO at [M]; DEC-116 (replay parity) recommended. Premise confirmed: crop runs byte-identical through native `apply` and slim wasm `transform`;
  `just ddphotos-parity` 16 rows green. Driven: `info()` reads 1600x1071 for an Orientation 6 JPEG (output
  1071x1600); copy-vs-original size bound 0 px at exact 4x, 1 px from 2x, 2 px below; `aspect` misses at 4x, so presets emit a box.
- [ ] **rulings** (maintainer) — the split, the interaction, box presets, the orientation entry's home, DEC-116.
- [ ] **design** — `prompts/SPEC-161-design.md`
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
