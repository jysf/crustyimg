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
- [ ] **design** (`prompts/SPEC-153-design.md`), Opus. Rulings 2026-11-10, all five as recommended: typed order,
  `--invert --resize-max` may change, `--crop` pixels + aspect only, past-the-edge exit 1, split as filed. Writes DEC-114.
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
