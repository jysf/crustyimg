# SPEC-052 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.
In the claude-only variant the spec's `## Implementation Context` section IS the build handoff.

## Instructions

- [x] **design** — `lint --format json` (hand-rolled, no serde_json) + human polish specified with
  a synthetic-outcome golden test. (PROJ-004 framing, 2026-07-06.)
- [ ] **build** — depends on SPEC-050. Copy SPEC-049's `ExplainTrace::write_json` discipline.
- [ ] **verify**
- [ ] **ship**
