# SPEC-046 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — `src/analysis/` layer specified: `Analysis` immutable computed-once context +
  single-pass feature extractors + bounded no-panic `AnalysisError`; Failing Tests written.
  (PROJ-002 framing, 2026-07-05.)
- [ ] **build** — NEXT. **First build target of PROJ-002.** Make the Failing Tests pass; land the
  module standalone (registered in `lib.rs`, wired into no command), all existing tests green,
  `just deny` green. Read the spec's `## Implementation Context` + the two design briefs first.
- [ ] **verify**
- [ ] **ship**
