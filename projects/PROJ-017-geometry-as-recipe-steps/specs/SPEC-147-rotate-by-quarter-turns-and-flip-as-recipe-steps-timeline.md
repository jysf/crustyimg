# SPEC-147 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-147-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-05, `prompts/SPEC-147-frame.md`, Opus): GO, [S], two ops in one spec. No
  `rotate`/`flip` today (`unknown operation 'rotate'`, exit 1, on 0.8.2). Driven with exiftool: the
  pixel lane writes no EXIF, so no output carries a stale Orientation or pixel-dimension tag; the live
  trap is the lost tag (a recipe without `auto-orient` displays sideways). Recommends DEC-113 (geometry
  acts on the received buffer, writes no metadata), `angle` a number in [-360, 360], clockwise, and
  `flip axis = horizontal` = mirror left/right. Four maintainer questions, all with recommendations.
  Rulings (2026-11-05, all four as recommended): positive `angle` is clockwise; `flip` keeps
  `horizontal`/`vertical` with a doc sentence; the non-multiple refusal names no stage id; `0`/`±360`
  load as a no-op.
- [ ] **design** (`prompts/SPEC-147-design.md`)
- [ ] **build** (`prompts/SPEC-147-build.md`)
- [ ] **verify** (`prompts/SPEC-147-verify.md`)
- [ ] **ship**
