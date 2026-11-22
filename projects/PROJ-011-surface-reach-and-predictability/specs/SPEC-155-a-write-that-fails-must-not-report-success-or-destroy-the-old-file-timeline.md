# SPEC-155 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-155-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-22, Opus): GO at [M]. Premise driven on a `main` release build under
  `ulimit -f 1` with XFSZ ignored: outputs under 8 KiB exit **0** with a 512 B PNG (`convert -o`,
  `--out-dir`, `meta strip`). Larger ones exit 5 with the old file destroyed. `--save-recipe`
  overwrites without `-y`. A new DEC ("a file write is all or nothing") is recommended, and four
  rulings are open (read-only directory, fsync, `--save-recipe` guard, full disk).
- [ ] **rulings** (maintainer): Q1 to Q4 in the spec's Frame.
- [ ] **design**: `prompts/SPEC-155-design.md` (build starts after SPEC-154 merges)
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
