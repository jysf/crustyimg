# SPEC-014 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-014-<cycle>.md`.

## Instructions

- [x] **design** — spec authored (Context, Goal, Failing Tests, Implementation
  Context), `convert` api-contract entry pinned, build prompt written. No new
  DEC (reuses DEC-004/015/016). Authored by the orchestrator (Opus), 2026-06-15.
- [x] **build** — implement `run_convert` + thread `forced_format` through
  `run_pixel_op`; make the `## Failing Tests` pass; 4 gates; open PR. Prompt:
  `prompts/SPEC-014-build.md` (Sonnet 4.6). PR #15 opened 2026-06-15.
- [x] **verify** — Opus read-only review of PR #15. ✅ APPROVED, no punch list;
  all 12 named tests independently confirmed; CI 3-OS green. 2026-06-15.
- [x] **ship** — PR #15 squash-merged (`bdd89f5`); bookkeeping on `main`; archived
  to `specs/done/`; brag added. 2026-06-15.
</content>
