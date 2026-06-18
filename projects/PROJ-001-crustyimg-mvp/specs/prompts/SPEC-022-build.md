# SPEC-022 build prompt — `optimize` one-button web-good command

Start a **fresh session**. You are the IMPLEMENTER for SPEC-022 in the `crustyimg`
repo. The architect (Opus) has written the spec + failing tests + DEC-024. Your job:
make the `## Failing Tests` pass with the smallest correct change.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-022-optimize-one-button-web-good-command.md`
   — especially `## Command surface (PINNED)`, `## Failing Tests`, and
   `## Notes for the Implementer` (it gives you the exact `run_optimize` /
   `optimize_auto_config` shapes and the clap variant).
2. `decisions/DEC-024-optimize-command-shape.md` — the command-shape decision you
   build against.
3. `decisions/` DEC-019, DEC-017, DEC-016, DEC-015, DEC-003 (one-liners in the spec).
4. `src/cli/mod.rs` — copy the structure of `run_shrink` / `shrink_auto_config` /
   `reject_quality_with_auto`; reuse `run_pixel_op`, `resolve_effective_quality`,
   `shrink_params`, `parse_size`, `QualityTarget`, `AutoQuality`.
5. `guidance/constraints.yaml` for the listed constraints.

## What to build (this is composition — do NOT touch `src/quality` or `src/sink`)
- Add `Commands::Optimize { inputs, max, target, ssim, max_size }` with the clap
  attributes from the spec (mirror `Shrink`'s `conflicts_with`/`conflicts_with_all`).
- Add the `dispatch` arm → `run_optimize(...)`.
- Add `optimize_auto_config(target, ssim, max_size) -> Result<AutoQuality, CliError>`
  (always returns a mode; default = `Perceptual(visually-lossless)` = score 90).
- Add `run_optimize(...)`: build the `auto-orient` op, push it; iff `--max N` push a
  `resize` op via `shrink_params(N)`; `reject_quality_with_auto`; delegate to
  `run_pixel_op(pipeline, inputs, global, None, None, Some(auto))`.
- If `QualityTarget::target_score` needs wider visibility, bump it to `pub(crate)`.

## Tests — every one in the spec's `## Failing Tests` must exist and pass
- Unit tests in `src/cli/mod.rs` `#[cfg(test)] mod tests` (6 named tests).
- Integration tests in `tests/cli.rs` (10 named `optimize_*` tests) using the
  `tests/common/mod.rs` fixtures (`jpeg_with_orientation`, `detailed_jpeg`,
  `gradient_jpeg`, `solid_png`). Mirror `auto_orient_cli_rotates_and_clears_tag` for
  the orientation-6 → 20×40 dims + `exif: no` assertions, and the `shrink`/`convert`
  `--max-size`/`-q`-conflict tests for the rest.
- **Confirm each named test exists** (diff the spec list against the files) before
  claiming green — absent tests just don't run.

## Gates (all must pass)
```
cargo fmt            # then `git add -u` so CI's fmt --check sees the same bytes
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check licenses   # must stay green; this spec adds NO dependency
```
`just check` runs fmt-check + lint + build + test together.

## Git / PR
- Branch `feat/spec-022-optimize` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before each commit; ignore any untracked
  `reports/daily|weekly/*.md`.
- PR title: `feat(cli): optimize one-button web-good command (SPEC-022)`.
- PR body per AGENTS.md §13 (Summary / Spec metadata / Decisions referenced —
  DEC-024 + the reused ones / Constraints checked / New decisions — DEC-024).
- Fill the spec's `## Build Completion` + the 3 build-reflection answers.

## Cost
Append your build session to the spec `cost.sessions` (leave `tokens_total` /
`estimated_usd` / `duration_minutes` null — the orchestrator fills the real numbers
at ship from the Agent result; if you run interactively, use `/cost`):
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-17
  notes: "optimize command: clap variant + run_optimize + tests; pure composition, no new dep"
```

## When done
`just advance-cycle SPEC-022 verify`, open the PR, and **stop** — the orchestrator
pauses for the user before any merge.
