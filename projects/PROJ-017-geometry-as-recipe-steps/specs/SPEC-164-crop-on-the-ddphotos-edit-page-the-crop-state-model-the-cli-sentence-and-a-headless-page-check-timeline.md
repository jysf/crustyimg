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
- [ ] **design** — `prompts/SPEC-164-design.md`
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
