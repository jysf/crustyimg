# SPEC-164 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-164-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-27, Opus, `prompts/SPEC-164-frame.md`): SPLIT; this half GO at [M]. The crop
  mode's state model and box geometry as pure `recipe.js` functions with a Node table, the CLI panel's
  smaller-file sentence (DEC-118 rule 5), and an in-repo headless-Chrome check of the edit page
  (prototype driven: render count, debounce, touch drag, a red negative control). The crop mode's DOM
  is a new STAGE-060 entry; the watermark fit on small crops is a new finding entry.
- [x] **rulings** (2026-11-27, maintainer) — all six accepted: split; `MIN_CROP = 64` with the watermark fit
  filed apart; swap twice stays explicit; crop mode updates only the size line, Done renders, Make disabled;
  headless check in-repo in the advisory `ddphotos-parity` job; README crop rows go with the DOM entry.
- [x] **design** (2026-11-27, Opus, `prompts/SPEC-164-design.md`): the six crop-mode functions, with
  the drag's start box in the reducer's `{ crop, preset, drag }` state; 153 literal Node rows (114
  reducer, 31 geometry, 8 size-line) and 18 Node mutants plus 5 headless controls, all driven red;
  a headless check H1–H8 driven green in a prototype. `MIN_CROP = 64` refused on thin resized strips
  (smallest clean 122 over the sweep); ruling kept, question put to the maintainer.
- [x] **design rulings** (2026-11-27, maintainer): keep `MIN_CROP = 64`; the watermark-fit entry takes
  the width-plus-margin case and lands before the crop mode's DOM entry.
- [x] **build** (2026-11-27, Sonnet, `prompts/SPEC-164-build.md`): draft PR #274 at `b7fee27`, CI green;
  17/17 ACs; parity 153 crop-mode rows, page check 8 rows 0 failed; every mutant matched except M4
  (also red on X21); $7.50 measured before the last commits.
- [x] **verify** (2026-11-27, Opus, `prompts/SPEC-164-verify.md`): PUNCH LIST at `b7fee27`, wording only,
  no re-verify. Own model agreed with 153/153 literals and 187,096 fuzzed steps; every mutant and failure
  exit as described; M4 also red on X21 (shared `toBox`). $9.18.
- [x] **punch list** (2026-11-27, Sonnet, `prompts/SPEC-164-punchlist.md`): `3852b45` + `10ae6e5`; three
  comments, X31 on four shapes, 157 crop-mode rows; negative control red on X31 (and X7) ×4. CI red at
  `10ae6e5` on the page check: `open()` returns before the new document exists. $1.17.
- [x] **punch list 2** (2026-11-27, Sonnet, `prompts/SPEC-164-punchlist-2.md`): `81ea04f`; `open()` waits for
  the new document; CI failure reproduced (body delay) red then green; stale-page form not reproduced. $1.37.
- [x] **merge** (2026-11-27, maintainer) — PR #274 as 81905d9, head f102e2a.
- [x] **ship** (2026-11-27, Sonnet, `prompts/SPEC-164-ship.md`): PR #274 merged as 81905d9; spec corrected (design sentence, AC-4, M4, 153 to 157), two ideas filed, archived. Total $39.41.
