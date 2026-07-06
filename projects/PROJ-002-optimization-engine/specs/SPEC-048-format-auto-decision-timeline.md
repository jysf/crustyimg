# SPEC-048 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — format auto-decision engine specified: `Analysis`-driven decision tree + ≤3
  shortlist + per-candidate solve over the existing `src/quality/` search + winner rule +
  `--profile web|docs|preserve`; pure `sink`-free Phase A/B/D so PROJ-003 wraps it. Failing Tests
  (pure winner-rule + integration) written; DEC-048 captures the engine/profiles/AVIF-budget rule.
  (PROJ-002 framing, 2026-07-05.)
- [ ] **build** — depends on SPEC-046 + SPEC-047 (STAGE-011 shipped) — the `Analysis`/`opt_bucket`
  the engine reads. Compose `src/quality/` unchanged; add `--profile`; keep `preserve` byte-identical.
- [ ] **verify**
- [ ] **ship** — ships as part of 0.3.0 (with SPEC-049).
