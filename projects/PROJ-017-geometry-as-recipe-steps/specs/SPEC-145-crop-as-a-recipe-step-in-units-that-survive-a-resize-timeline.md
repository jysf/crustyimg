# SPEC-145 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-145-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-04, `prompts/SPEC-145-frame.md`, Opus): GO, [M]. No crop op exists today
  (`unknown operation 'crop'`, exit 1, on 0.8.2). Recommends fractions-only units with per-edge
  rounding (DEC-091 Fence A already rules pixel coordinates lab-side), one op with exclusive
  rectangle/aspect key groups, and a pre-named split (DEC + rectangle, then aspect). Q1 (pixels
  in a recipe?) needs the maintainer before design.
- [x] **design** (2026-11-05, `prompts/SPEC-145-design.md`, Opus): DEC-112 (fractions only, f64,
  each edge rounded on its own with halves up; 0 px refused at apply as an op failure, not via the
  skippable refusal hook). 65 ACs: 32 load and 4 apply refusals with exact messages, 2 corpus recipes
  (`GOLDEN_LINES` 51 → 57), 3 wasm rows, 9 doc surfaces; 13-row mutation table. One spec, no split.
  Rulings (2026-11-05, all three as recommended): fractions only in a recipe, pixels via `edit --crop`
  later; `aspect` takes whole numbers only (`"239:100"`); a crop that rounds to 0 px is refused, not
  clamped.
- [x] **build** (`prompts/SPEC-145-build.md`, 2026-11-05, Sonnet, ~$9.37). Draft PR #244 at `6611e79`, CI green
  on every leg; 65/65 ACs; M1–M12 driven (M2, M4, M6, M12 flip extra rows; M4b exits 2 on a single
  input but 0 with uncropped output on a batch). Corpus: both crop recipes byte-identical across
  the three targets, no existing output changed. wasm-test 49/0; bundle +3,271 B brotli. Finding:
  the batch refusal warning says "watermark skipped" for any refusing op. The maintainer accepted design's reading of ruling 3 on
  2026-11-05: the 0 px message names `crop` and both sizes, with no step index.
- [x] **verify** (`prompts/SPEC-145-verify.md`, 2026-11-05, Opus, read-only, ~$5.98). APPROVED at `6611e79`,
  no punch items. All 36 messages match by script and through the binary; the thesis holds on
  6000/1200 px images with pixels exact against `vips`; M1, M3b, M8, M12 and M4b driven; a 0 px
  batch exits 6 and writes nothing. Four optional cleanups; the maintainer chose to fix them
  before merge.
- [x] **punch list** (`prompts/SPEC-145-punchlist.md`, 2026-11-05, Sonnet, ~$1.37). PR #244 at `0c6f722`, CI
  green on `1316ad7` (the last code commit). Bound is 0.4 + 1e-9 (the worst case computes as
  0.4000000000000057); a +0.1 right-edge mutation drifts 0.6, red at 0.4 and green at 0.6. The
  A-table assert fails at 3 vs 4. Batch exits driven: one bad input 1, two bad 6 with nothing
  written, one bad + one good 6 with the good image written; `recipes/README.md` reworded. The
  bound test lives in `src/operation/crop.rs`'s test module, so `src/` changed in tests only.
  DEC-112's bound line now says 0.4.
- [ ] **re-verify** (`prompts/SPEC-145-reverify.md`, Opus, read-only)
- [ ] **ship** (`prompts/SPEC-145-ship.md`)
