# SPEC-117 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-117-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. Spec written; 6 ACs, 2 pre-written tests, neither red on HEAD by
      design. AC-5's per-verb negative control is the load-bearing criterion.
      Shipped on `main` via PR #166 (design-only, with SPEC-116).
- [~] **build** — prompt: `prompts/SPEC-117-build.md`. Sonnet. Worktree
      `../crustyimg-spec117` on `test/spec-117-pin-build-and-apply-adopted-format`, created
      2026-08-15 off `main` at `213d570`. **Unblocked** — SPEC-116 shipped as `016e89f`.
      ⚠ AC-5's per-verb negative control is the load-bearing criterion: neither test is red on
      HEAD by design.
- [ ] **verify** — Opus, new session.
- [ ] **ship**
