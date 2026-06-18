# SPEC-023 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-023-<cycle>.md`.

## Instructions

- [x] design (2026-06-17) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`), emitted DEC-025 (diff command + new exit code 7 +
  heatmap deferral), and added exit code 7 + a `diff` entry to `docs/api-contract.md`.
  Pure reuse of `crate::quality::score`; v1 = score + `--fail-under` gate + `--json`,
  visual heatmap deferred.
- [ ] build — make the `## Failing Tests` pass: add `Commands::Diff`, `run_diff`,
  `diff_passes`, `write_diff_json`, the `CliError::CheckFailed` (exit 7) variant
  (+ extend `exit_code_mapping_is_total`), the dispatch arm, and unit + integration
  tests. Prompt: `prompts/SPEC-023-build.md`.
- [ ] verify — independent review: acceptance criteria, exit-7 mapping total,
  no decision drift, every named failing test exists, cost session recorded.
- [ ] ship — PR merge (pause for the user first), reflections, cost totals, archive
  to `specs/done/`, flip the STAGE-009 backlog line.
