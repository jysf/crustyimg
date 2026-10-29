# SPEC-138 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-138-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-10-28 (`prompts/SPEC-137-138-frame.md`, shared with SPEC-137). **GO, S**
      (on refuse). Premise confirmed on `main` (3 target kinds) and 0.7.1 (plain target):
      `build --format png` exits 0 with byte-identical non-PNG output; `--format bogus` also exits 0.
      Sweep: `-o`, `--out-dir`, `--name-template` are ignored the same way; `-q`/`-j`/`-Q` are honoured.
      Recommend refusing all four (exit 2, one guard). Honouring is [M].
- [x] **design** — 2026-10-28 (`prompts/SPEC-138-design.md`). 17 ACs, 15 failing tests (12 RED
      on `main`, 2 green controls, 1 new unit test). One guard in `dispatch`; `Option::is_some()` is
      exact (no clap default, no env feature). Every refused flag gets its own line, fixed order.
      No golden lock: a two-binary `compare-builds.sh` run, proven red by a one-byte key mutant.
      DEC-057 and DEC-105 amended; DEC-108 released. `Breaking` CHANGELOG entry.
- [~] **build** — prompt: `prompts/SPEC-138-build.md` (Sonnet). Handed out 2026-10-29, after SPEC-137 merged (`71b4f38`).
- [ ] **verify** — prompt: `prompts/SPEC-138-verify.md`.
- [ ] **ship** — prompt: `prompts/SPEC-138-ship.md`.
