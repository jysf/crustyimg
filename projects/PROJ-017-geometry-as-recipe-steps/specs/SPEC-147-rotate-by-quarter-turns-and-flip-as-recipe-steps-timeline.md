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
- [x] **build** (`prompts/SPEC-147-build.md`, 2026-11-05, Sonnet, ~$7.84). Draft PR #252 at `ceda510`;
  CI green on every leg at `f2f5b59` (the last code commit). 58/58 ACs; all 12 mutation rows driven
  and every red set matched, with one cell corrected (M8 row 3 goes red as a wrong verdict) and one
  extra red row (M10 also reds the corpus `to_toml` test). Corpus: both recipes byte-identical across
  the three targets, no existing file changed. wasm 51/49 locally and in CI; `--self-test` 18/18 and
  `workflow` 16/16 locally. Lean leg runs the new tests (1117 passed). Bundle +2,423 B brotli. One
  file outside `## Outputs`: `tests/common/mod.rs`, which the Failing Tests section asks for.
- [~] **verify** (`prompts/SPEC-147-verify.md`)
- [ ] **ship**
