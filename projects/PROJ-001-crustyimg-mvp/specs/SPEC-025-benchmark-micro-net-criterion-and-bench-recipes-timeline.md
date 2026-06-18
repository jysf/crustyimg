# SPEC-025 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-025-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Authored the spec + DEC-028 (criterion
  micro-benches + the equal-quality principle). Infrastructure spec: no `## Failing
  Tests` (benches aren't behavioral unit tests); verification is `cargo bench
  --no-run` + `just bench`. Dev-dependency only; no shipped-binary impact.
- [ ] build — add `criterion` (dev-dep) + `benches/pipeline.rs` (decode/resize/
  encode_jpeg/score/pipeline groups) + `[[bench]]` + `just bench`/`bench-cli`. Prompt:
  `prompts/SPEC-025-build.md`.
- [ ] verify — `cargo bench --no-run` compiles all 5 groups, `just bench` runs,
  `cargo deny` green, default/lean builds + clippy/fmt unaffected, no decision drift,
  cost session recorded.
- [ ] ship — PR merge (pause for the user first), reflections, cost totals, archive
  to `specs/done/`, flip the STAGE-009 backlog line. **Index-verify before the ship
  commit** ([[verify-git-index-before-ship-commit]]).
