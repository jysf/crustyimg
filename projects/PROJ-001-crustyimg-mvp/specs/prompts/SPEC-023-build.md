# SPEC-023 build prompt — `diff` perceptual comparison + CI gate

Start a **fresh session**. You are the IMPLEMENTER for SPEC-023 in the `crustyimg`
repo. The architect (Opus) wrote the spec + failing tests + DEC-025. Make the
`## Failing Tests` pass with the smallest correct change.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-023-diff-perceptual-and-visual-comparison.md`
   — especially `## Command surface (PINNED)`, `## Failing Tests`, and `## Notes for
   the Implementer` (it gives the exact `run_diff`/`diff_passes`/`write_diff_json`
   shapes, the clap variant, and the `CliError::CheckFailed` exit-7 wiring).
2. `decisions/DEC-025-diff-command-and-check-exit-code.md`.
3. `src/quality/mod.rs` — `score(reference, candidate)` (the function you call; same
   dimensions required).
4. `src/cli/mod.rs` — the `Commands` enum, `dispatch`, `CliError` + `code()` + the
   `exit_code_mapping_is_total` test, and `run_info`/`write_json`/`escape_json` (the
   hand-rolled-JSON pattern to mirror for `--json`).

## What to build (pure reuse — do NOT touch `src/quality`)
- `Commands::Diff { a, b, fail_under, json }` + the `dispatch` arm → `run_diff`.
- `run_diff` per the spec: validate `--fail-under` in 0..=100 (else `Usage`, exit 2);
  `Image::load` both; **dimension mismatch → `Usage` (exit 2)** with both dims in the
  message; `quality::score(a.pixels(), b.pixels())`; print `ssimulacra2: {score:.4}`
  (or JSON); on gate fail print a stderr diagnostic (unless `--quiet`) and return
  `CliError::CheckFailed`.
- `diff_passes(score, fail_under) -> bool` (no gate ⇒ true).
- `write_diff_json(...)` — mirror `write_json`, escape `a`/`b` with `escape_json`,
  `fail_under` as `{:.4}` or literal `null`, `passed` as a bare bool.
- `CliError::CheckFailed` unit variant; `code()` ⇒ **7**; extend
  `exit_code_mapping_is_total` to assert it.
- `docs/api-contract.md` already lists exit 7 + a `diff` entry (added at design) —
  leave them; just keep the code mapping consistent with the table.

## Tests — every one in the spec's `## Failing Tests` must exist and pass
- Unit in `src/cli/mod.rs` (`diff_parses_args`, `diff_passes_gate`, and the extended
  `exit_code_mapping_is_total`).
- Integration in `tests/cli.rs` (8 `diff_*` tests) using `common::detailed_png` and a
  q5 JPEG made inline from its decoded pixels (mirror SPEC-022's baseline encode).
  Keep scored images 96×96 (≥ the SSIMULACRA2 floor); the dimension-mismatch test
  (64 vs 32) never scores.
- Add `"diff"` to BOTH subcommand lists in `tests/cli.rs`
  (`help_lists_all_subcommands` + `each_subcommand_help_parses`).
- **Confirm each named test exists** (diff the spec list against the files) before
  claiming green.

## Gates (all must pass)
```
cargo fmt            # then `git add -u` so CI's fmt --check sees the same bytes
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check licenses   # stays green; this spec adds NO dependency
```

## Git / PR
- Branch `feat/spec-023-diff` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before each commit; ignore untracked
  `reports/daily|weekly/*.md`.
- PR title: `feat(cli): diff perceptual comparison + --fail-under CI gate (SPEC-023)`.
- PR body per AGENTS.md §13 (Summary / Spec metadata / Decisions referenced — DEC-025,
  DEC-019, DEC-007 / Constraints checked / New decisions — DEC-025).
- Fill the spec's `## Build Completion` + the 3 build-reflection answers.

## Cost
Append your build session to `cost.sessions` (leave numerics null — the orchestrator
fills real numbers at ship; if interactive, use `/cost`):
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-17
  notes: "diff command: clap variant + run_diff + CheckFailed exit-7 + tests; pure reuse of quality::score, no new dep"
```

## When done
`just advance-cycle SPEC-023 verify`, open the PR, and **stop** — the orchestrator
pauses for the user before any merge.
