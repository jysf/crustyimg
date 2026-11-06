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
- [x] **verify** (`prompts/SPEC-147-verify.md`, 2026-11-05, Opus, ~$6.51; cache_read 9,263,318,
  output 32,855; the pasted report cut off the input and cache_creation rows). APPROVED at
  `ceda510`, no punch items. All 23 refusal messages diffed by script against the spec and the
  binary, 0 differ (negative control caught a 1-byte edit). Direction and axis match ImageMagick
  `-rotate N`, `-flop` and `-flip` at AE 0 on 8-bit RGB and 16-bit RGBA; depth and colour type kept.
  DEC-113 driven with exiftool tag 6: no Orientation written, and both step orders agree (AC-39).
  Mutations M1, M6b, M7b, M10 and M8 re-driven; the build's corrected M8 cell and extra M10 row
  confirmed. Default 1180 and lean 1117 passed in fresh target dirs; wasm 51/49; CI 18 pass at
  `ceda510`. Trap 17: angle exactness driven with 54 near-multiple inputs (all refused); "apply never
  fails" driven only over the AC-11 shapes; DEC-113's tag 3/8 agreement reasoned, not driven.
  Optional cleanups for ship (none needs a re-verify): (1, assert) `orientation_of` in
  `tests/rotate_flip.rs:592` uses `.ok()?`, so treat only `Err(NotFound)` as no tag; (2, wording)
  Build Completion says 31 files, the diff has 32; (3, wording) mutation table M8 row 3 and the M10
  `to_toml` row; (4, wording) DEC-113 §3 says only tag 6 was driven. One out-of-spec wording finding
  on STAGE-058 (the corpus guard README's "Adding a recipe" table) was cut off in the paste.
- [ ] **ship** (`prompts/SPEC-147-ship.md`)
