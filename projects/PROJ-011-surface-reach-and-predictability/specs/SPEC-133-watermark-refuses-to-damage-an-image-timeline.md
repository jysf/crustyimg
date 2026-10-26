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
- [x] **design** — 2026-10-25 (`prompts/SPEC-133-design.md`). 27 ACs, 33 new tests (1 is the bless generator) + 12 existing
      tests re-fixtured (measured by a throwaway prototype run: 12 flips in 5 files). Fit (margin on
      anchored axes, after scale) + 5 % of the mark's own non-transparent pixels (not changed
      pixels — row CNT shows those depend on the image); `tile`/`scale` coverage-exempt. Seam: a
      defaulted `Operation::refusal` + `Pipeline::run_with(Fail|Skip)`. Cached `build` does not
      replay the skip line (documented; filed). Text-mode `scale`/`tile = true` refused inside
      SPEC-131's mode check — edits two SPEC-131 test rows. Builds after SPEC-131; not parallel
      with SPEC-134. DEC-104 at build.
- [ ] **build** — prompt: `prompts/SPEC-133-build.md`. **Opus**, own worktree, branch
      `feat/spec-133-watermark-damage-rule`. ⛔ Starts after SPEC-134 merges (shared CLI code);
      SPEC-131 is merged. Opens a draft PR.
- [ ] **verify**
- [ ] **ship**
