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
- [x] **design** (2026-11-05, `prompts/SPEC-147-design.md`, Opus): DEC-113 (a geometry step acts on
  the buffer it receives and writes no metadata; the bundle stays in memory for a later `auto-orient`;
  a pixel lane that ever writes EXIF must set Orientation 1, held by a sink doc paragraph and an
  orientation test read with kamadak-exif). 58 ACs: 16 rotate + 7 flip load refusals with exact
  messages (the range reads `a number in -360.0..=360.0`), a 12-row normalisation table, ten colour
  types, 2 corpus recipes (`GOLDEN_LINES` 57 → 63), 2 wasm tests (`FULL_COUNT` 51, `SLIM_COUNT` 49),
  9 doc surfaces; 12-row mutation table, pixel rows reasoned. [S], no split. Rulings (2026-11-05, all
  four as recommended): positive `angle` is clockwise; `flip` keeps `horizontal`/`vertical`; the
  non-multiple refusal names no internal id; `0`/`±360` load as a no-op.
- [ ] **build** (`prompts/SPEC-147-build.md`)
- [ ] **verify** (`prompts/SPEC-147-verify.md`)
- [ ] **ship**
