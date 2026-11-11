# SPEC-159 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-159-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-10, Opus) — GO at [M], one DEC (DEC-115 recommended). Premise confirmed
  (`transparent`/`#ff000080` refused at load, exit 1). Driven: JPEG drops alpha silently (exit 0),
  GIF keeps only alpha 0, the other six formats keep it. Recommends `transparent` = `#00000000`, a
  static recipe × format check on four surfaces, and GIF allowed at alpha 00. Four maintainer questions.
- [ ] **design** — `prompts/SPEC-159-design.md`
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
