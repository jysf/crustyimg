# SPEC-156 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-156-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-22, Opus) — GO at [S]. Premise driven on a release build: stripped `cmyk_16x16.jpg` scores
  SSIMULACRA2 41.9374, AE 56.47, djpeg `srgb(191,64,32)` → `srgb(93,134,30)`. A 3-component Adobe transform-0 file with
  component IDs 1,2,3 breaks too (−21.30); transform 1, RGB-letter IDs and 4-component transform 0 don't. Keep APP14
  (with the `Adobe` signature) only; set/clean/copy/pixel lane already keep the render. Orientation finding filed.
- [ ] **design** — `prompts/SPEC-156-design.md` (after SPEC-154 has merged for the build)
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
