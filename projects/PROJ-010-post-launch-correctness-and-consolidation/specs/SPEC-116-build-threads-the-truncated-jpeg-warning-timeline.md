# SPEC-116 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-116-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. Spec written; 9 ACs, 3 settled design calls, 5 pre-written tests.
      Shipped on `main` via PR #166 (design-only, with SPEC-117).
- [ ] **build** — prompt: `prompts/SPEC-116-build.md`. Sonnet, own worktree, branch
      `feat/spec-116-build-threads-truncated-jpeg-warning` off `main`. Runs FIRST of the two
      remaining STAGE-043/045 specs — it edits `tests/build.rs`, which SPEC-117 also touches.
- [ ] **verify** — Opus, new session.
- [ ] **ship**
