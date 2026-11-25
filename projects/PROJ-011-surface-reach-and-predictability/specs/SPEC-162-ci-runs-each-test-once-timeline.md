# SPEC-162 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-162-<cycle>.md`.

## Instructions

- [x] **frame + design** (2026-11-24, orchestrator, Opus) — folded into the spec: the measurement of run
  45501714010 is the frame; `## Outputs`, the ACs and the mutation table are the design. No separate sessions.
- [ ] **build** — prompts/SPEC-162-build.md (Sonnet), on branch `chore/spec-162-ci-runs-each-test-once`.
- [ ] **verify** — prompt written after the build report (Opus, read-only).
- [ ] **ship** — after the maintainer merges the PR.
