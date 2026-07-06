# SPEC-047 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — deterministic no-ML classification specified: `ImageClass` (5 labels) → three
  `OptBucket`s, cascade + safe-fallback bias, synthetic labeled corpus in Failing Tests; thresholds
  captured in DEC-047. (PROJ-002 framing, 2026-07-05.)
- [ ] **build** — depends on SPEC-046 (extends `src/analysis/mod.rs`). Make the corpus tests pass;
  land the classifier + the three `Analysis` fields; classification stays wired into no command.
- [ ] **verify**
- [ ] **ship**
