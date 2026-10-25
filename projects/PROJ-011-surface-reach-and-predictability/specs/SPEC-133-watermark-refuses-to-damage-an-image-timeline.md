# SPEC-133 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-133-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-10-24 (`prompts/SPEC-133-frame.md`). **GO, M (top of M).** Premise
      confirmed on `main` @ `8471fb1` (24 px `--size 200` → 6 px changed, exit 0; long string on
      256 px clipped at 3.73 %; self-diff 0). Image overlays clip too; recipe text accepts
      `tile`/`scale` the CLI refuses (in scope). `--min-size` split to its own backlog item. Size
      is driven by the batch *skipped* outcome across 5 driver loops + `build`'s lockfile/cache.
      Sequential after SPEC-131 (`src/operation/mod.rs`).
- [ ] **design** — `prompts/SPEC-133-design.md`
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
