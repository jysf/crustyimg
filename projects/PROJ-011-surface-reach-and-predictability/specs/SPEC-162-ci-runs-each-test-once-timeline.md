# SPEC-162 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-162-<cycle>.md`.

## Instructions

- [x] **frame + design** (2026-11-24, orchestrator, Opus) — folded into the spec: the measurement of run
  45501714010 is the frame; `## Outputs`, the ACs and the mutation table are the design. No separate sessions.
- [x] **build** (2026-11-25, Sonnet, $2.17 measured before its last edits) — draft PR #271 at `e54e5b5`; code last
  changed at `ecf0dc7` (run 45539781614 green). 102.6 → 62.3 runner-minutes; cache kept (Build + Test −43/−23/−47%).
  M3 did not match AC-5: a shared `CARGO_TARGET_DIR` serves stale outer artifacts across worktrees (the AC's mistake).
- [~] **verify** — prompts/SPEC-162-verify.md (Opus, read-only). Don't merge #271 before it reports.
- [ ] **ship** — after the maintainer merges the PR.
