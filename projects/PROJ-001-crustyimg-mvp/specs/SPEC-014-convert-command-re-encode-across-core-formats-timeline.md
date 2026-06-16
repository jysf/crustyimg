# SPEC-014 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-014-<cycle>.md`.

## Instructions

- [x] **design** — spec authored (Context, Goal, Failing Tests, Implementation
  Context), `convert` api-contract entry pinned, build prompt written. No new
  DEC (reuses DEC-004/015/016). Authored by the orchestrator (Opus), 2026-06-15.
- [ ] **build** — implement `run_convert` + thread `forced_format` through
  `run_pixel_op`; make the `## Failing Tests` pass; 4 gates; open PR. Prompt:
  `prompts/SPEC-014-build.md` (Sonnet 4.6).
- [ ] **verify** — Opus read-only review of the PR against the spec; ✅/⚠/❌.
- [ ] **ship** — squash-merge PR, bookkeeping on `main`, archive, brag.
</content>
