# SPEC-049 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — `--explain` / `--explain=json` specified: a typed `ExplainTrace` (subset of the
  planner schema) rendered human→stderr + hand-rolled JSON (no `serde_json`); golden fixture in
  Failing Tests; schema captured in DEC-049. (PROJ-002 framing, 2026-07-05.)
- [ ] **build** — depends on SPEC-048 (renders its decision record; must not re-run the engine).
- [ ] **verify**
- [ ] **ship** — ships as part of 0.3.0 (with SPEC-048).
