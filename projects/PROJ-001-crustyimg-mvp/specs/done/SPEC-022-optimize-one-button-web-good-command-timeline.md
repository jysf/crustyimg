# SPEC-022 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-022-<cycle>.md`.

## Instructions

- [x] design (2026-06-17) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`), emitted DEC-024 (optimize command shape), and
  added the SPEC-022 line to the STAGE-009 backlog. Pure-composition design: no new
  dep, reuses `auto-orient` + `run_pixel_op` + `resolve_effective_quality`.
- [x] build (2026-06-17, PR #25) — Opus, main loop. Added `Commands::Optimize`,
  `run_optimize`, `optimize_auto_config`, dispatch arm + 6 unit + 10 integration
  tests. Pure composition, no new dep. All gates + 3-OS CI green.
- [x] verify (2026-06-17) — independent read-only Explore subagent: ✅ APPROVED. All
  16 named tests present, no DEC-024 drift, no `unwrap` outside tests, pipeline order
  + reject-quality wiring correct, `cargo test`/clippy/fmt green.
- [x] ship (2026-06-17, PR #25 squash-merged) — reflections + cost totals filled,
  STAGE-009 backlog line flipped, archived to `specs/done/`, `just cost-audit` green.
