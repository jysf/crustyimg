# SPEC-032 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-032-<cycle>.md`.

## Timeline

- [x] **design** (2026-06-19, Opus) — authored the spec (`## Command surface (PINNED)`,
  `## Round-trip guarantee (PINNED)`, `## Failing Tests`, `## Implementation Context`).
  Flag surface: `--auto-orient` / `--resize-max N` / `--invert`, canonical order
  auto-orient→resize→invert; `--save-recipe` serializes via `Recipe::from_ops`/`to_toml`;
  reuses `run_pixel_op` + the registry. No new dep, no new DEC. Updated
  `docs/api-contract.md` + the stage backlog. Build prompt at `prompts/SPEC-032-build.md`.
- [x] **build** (2026-06-19, Sonnet 4.6) — PR #36; clap op flags + `run_edit` +
  `build_edit_ops` reusing `run_pixel_op` + the registry + `Recipe::from_ops`/`to_toml`.
  378 tests green (4 unit + 7 integration new); clippy/fmt/lean/deny clean. subagent
  tokens=110450 (~$0.60).
- [x] **verify** (2026-06-19, Opus Explore) — APPROVED, no concerns; validated the DEC-005
  round-trip crux, canonical/flag-order independence, exit codes (1/3/5), no production
  unwraps, scope. Gates re-run green (378 tests). ~55k est.
- [x] **ship** (2026-06-19) — squash-merged PR #36 (ceb3af6); cost totals + ship reflection
  + archived to `specs/done/`. **STAGE-005 complete.**
