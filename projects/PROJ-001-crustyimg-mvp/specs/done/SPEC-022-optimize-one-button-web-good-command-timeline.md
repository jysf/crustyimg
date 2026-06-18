# SPEC-022 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-022-<cycle>.md`.

## Instructions

- [x] design (2026-06-17) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`), emitted DEC-024 (optimize command shape), and
  added the SPEC-022 line to the STAGE-009 backlog. Pure-composition design: no new
  dep, reuses `auto-orient` + `run_pixel_op` + `resolve_effective_quality`.
- [ ] build — make the `## Failing Tests` pass: add `Commands::Optimize`,
  `run_optimize`, `optimize_auto_config`, the dispatch arm, and the unit +
  integration tests. Prompt: `prompts/SPEC-022-build.md`.
- [ ] verify — independent review: acceptance criteria, no decision drift
  (`just decisions-audit --changed`), constraints, every named failing test exists,
  cost session recorded.
- [ ] ship — PR merge (pause for the user first), reflections, cost totals, archive
  to `specs/done/`, flip the STAGE-009 backlog line.
