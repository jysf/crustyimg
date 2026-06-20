# SPEC-035 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-035-<cycle>.md`.

## Timeline

- [x] **design** (2026-06-19, Opus) — authored the spec (`## Limits policy (PINNED)`,
  `## Failing Tests`, `## Implementation Context`) + **DEC-036**. Read the recipe loader
  + the resize op: the functional validation (version/unknown-op/param) already exists;
  the gap is resource bounding. Caps: recipe text ≤ 64 KiB + ≤ 1024 steps, typed
  `RecipeError::TooLarge`/`TooManySteps` (exit 1), at the `from_toml` choke point + a CLI
  pre-read file-size guard. Noted the resize-upscale-bomb (op-param bound) as an explicit
  out-of-scope follow-up. std-only, no new dep. Updated SECURITY.md + api-contract. Build
  prompt at `prompts/SPEC-035-build.md`.
- [x] **build** (2026-06-19, Sonnet 4.6) — PR #39; `RECIPE_MAX_BYTES`/`RECIPE_MAX_STEPS`
  + `RecipeError::TooLarge`/`TooManySteps` in `from_toml` (size→parse, version→steps) +
  CLI `run_apply` pre-read metadata guard. 404 tests green (6 unit + 2 integration new);
  clippy/fmt/lean/deny clean. No deviations. subagent tokens=86548 (~$0.47).
- [x] **verify** (2026-06-19, Opus Explore) — APPROVED, no concerns; confirmed caps,
  load-bearing ordering, inclusive boundaries, pre-read guard, and (gap-hunt) `from_toml`
  as the single non-bypassable choke point. Gates re-run green (404 tests). ~55k est.
- [x] **ship** (2026-06-19) — squash-merged PR #39 (9bbb05e); cost totals + ship reflection
  + archived to `specs/done/`. STAGE-006 backlog #3 complete.
