# SPEC-154 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-154-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-22, Opus) — GO, [S]. Premise confirmed: the classifier calls 20 tracked
  build/test inputs in 9 classes docs-only (the review's four plus `docs/github-actions.md`,
  `docs/data-model.md`, `README.md`, `.pre-commit-hooks.yaml`, `demo/`, `bench/corpus/`). Recommends
  inverting to a docs-only allow-list (fails safe; 12 of 600 replayed commits flip to a full run)
  and adding `recipes/**`, `assets/**` to `pages.yml`. Prompt: `prompts/SPEC-154-frame.md`.
- [x] **design** (2026-11-22, Opus) — allow-list `DOCS` (15 alternatives) + `EXCEPT` (README.md,
  docs/github-actions.md, docs/data-model.md); derived include check (35 targets, floor, canonical-path,
  concat!/env! refused) and pages drift check, all in `--self-test` (88 rows). 32 mutations driven red on a
  prototype; replay 12/600 flips. Prompt: `prompts/SPEC-154-design.md`.
- [ ] **build** — prompt: `prompts/SPEC-154-build.md`.
- [ ] **verify**
- [ ] **ship** — then the draft `recipes/web.toml`-only proof PR.
