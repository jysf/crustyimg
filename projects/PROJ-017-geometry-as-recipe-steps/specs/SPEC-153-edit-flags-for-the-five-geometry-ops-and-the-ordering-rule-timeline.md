# SPEC-153 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-153-<cycle>.md`.

## Instructions

- [x] **frame** (`prompts/SPEC-153-frame.md`), 2026-11-10, Opus. **SPLIT.** Premise confirmed (`edit --crop`/`--rotate`
  exit 2, unexpected argument). The order problem was driven: on 60×40, crop→rotate gives 40×30 and rotate→crop gives 20×60. Recommends
  typed order for every op flag (DEC-114), `auto-orient` always first, no repeats. Today's `--invert --resize-max` changes
  (1789/1800 samples on noise). The crop pixel round trip is exact over 2.68e9 rectangles to 50,000 px; a 6-decimal writer is
  refused at load at (640, 9, 631). This half covers crop, rotate and flip [M]; `--pad`/`--trim` [S] is split off and closes STAGE-058.
- [x] **design** (`prompts/SPEC-153-design.md`), 2026-11-10, Opus. **[M]**, at its upper edge. DEC-114 written (typed order, seven
  parts). Corrections: `edit` takes a batch (a pixel `--crop` with `--save-recipe` needs one input, exit 2); no edge-touching rectangle
  is noisy (1.25e9 checked), so the replay test uses an interior one. Byte pin recorded from `main`; 15 mutation rows + the trap-14
  control all driven on a prototype. 27 ACs, 31 test functions, 86 rows.
- [x] **build** (`prompts/SPEC-153-build.md`), 2026-11-10, Sonnet, $5.29. Draft PR #261 at `d917d60`, CI green (18 passed,
  8 release jobs skipped; checked by the orchestrator). 27/27 ACs; FT-1 green on `main` before any edit and after. Every
  mutation red; M1, M2 and M15 red more than the table (M2 also reds `tests/cli.rs`' stub test, for verify to explain).
  Deviations: stages 2–4 in one push; FT-14 uses the little-endian EXIF helper.
- [x] **verify** (`prompts/SPEC-153-verify.md`), 2026-11-10, Opus, $5.64 (read-only, not recorded). **PUNCH LIST** at
  `d917d60`, wording only, no re-verify. Messages, typed order, pixel crops (vs ImageMagick at AE 0), the byte pin (own
  decoder), replay and batches all hold. Two doc sentences wrong: `docs/recipes.md` says a saved pixel crop "replays on any
  size" (it crops proportions; driven exit 1), and both recipe docs put `--auto-orient` in typed order. Cleanups for ship: the
  M1/FT-10 and M6 table cells; the stale "on a single image" `edit` wording (pre-existing). M2's stub-test red is an `edit` test.
- [ ] **punch list** (`prompts/SPEC-153-punchlist.md`): the two doc sentences and two Build Completion lines.
- [ ] **ship**
