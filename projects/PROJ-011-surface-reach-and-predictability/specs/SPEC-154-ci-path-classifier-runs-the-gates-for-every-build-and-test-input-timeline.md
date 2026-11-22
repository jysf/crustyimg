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
- [ ] **design** — answer the three maintainer questions in `## Frame`; write ACs, the self-test and mutation rows.
- [ ] **build**
- [ ] **verify**
- [ ] **ship** — then the draft `recipes/web.toml`-only proof PR.
