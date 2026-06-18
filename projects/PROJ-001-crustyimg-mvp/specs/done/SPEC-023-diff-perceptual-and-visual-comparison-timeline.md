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
- [x] build (2026-06-18, PR #26) — Opus, main loop. Added `CliError::CheckFailed`
  (exit 7) + extended `exit_code_mapping_is_total`, `Commands::Diff`, `run_diff`,
  `diff_passes`, `write_diff_json` + 2 unit + 8 integration tests. Pure reuse of
  `quality::score`, no new dep. All gates + 3-OS + feature CI green.
- [x] verify (2026-06-18) — independent read-only Explore subagent: ✅ APPROVED.
  Exit-7 contract consistent across the enum / `code()` / its doc table / api-contract
  / the test; all 10 named tests present; no DEC-025 drift; no `unwrap` outside tests.
- [x] ship (2026-06-18, PR #26 squash-merged) — reflections + cost totals filled
  (index-verified to avoid the SPEC-022 cost-audit churn), STAGE-009 backlog flipped,
  archived to `specs/done/`, `just cost-audit` green.
