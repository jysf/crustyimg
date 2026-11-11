# SPEC-159 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-159-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-10, Opus) — GO at [M], one DEC (DEC-115 recommended). Premise confirmed
  (`transparent`/`#ff000080` refused at load, exit 1). Driven: JPEG drops alpha silently (exit 0),
  GIF keeps only alpha 0, the other six formats keep it. Recommends `transparent` = `#00000000`, a
  static recipe × format check on four surfaces, and GIF allowed at alpha 00. Four maintainer questions.
- [x] **design** (2026-11-11, Opus, $12.84) — [M] holds; DEC-115 written. Frame corrected: the auto-decided
  format CAN be JPEG (the shortlist read the source's alpha; driven: black border, exit 0), fixed by
  OR-ing `Pipeline::introduces_alpha`. Check lives on `Operation::introduced_alpha` + `alpha.rs` +
  `Pipeline::unstorable_alpha`, six call sites; 18 mutation rows, 17 driven on a prototype; byte pin recorded.
- [ ] **build** — `prompts/SPEC-159-build.md`
- [ ] **verify**
- [ ] **ship**
