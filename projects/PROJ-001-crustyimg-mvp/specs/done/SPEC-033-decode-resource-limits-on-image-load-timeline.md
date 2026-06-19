# SPEC-033 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-033-<cycle>.md`.

## Timeline

- [x] **design** (2026-06-19, Opus) — authored the spec (`## Limits policy (PINNED)`,
  `## Failing Tests`, `## Implementation Context`) + **DEC-034** (decode-limits policy).
  Probed `image::Limits` / `ImageReader::limits` / PNG codec enforcement against the
  vendored crate source: confirmed `check_dimensions` at decoder construction +
  `limits.reserve(total_bytes)` at decode enforce dimension + alloc caps and surface
  `ImageError::Limits`. Caps: per-dimension ≤ 65 535, alloc ≤ 512 MiB; reject→typed
  `LimitsExceeded`→exit 1; one choke point (`decode_with_format`). No new dep. Updated
  `SECURITY.md` + `docs/api-contract.md`. Build prompt at `prompts/SPEC-033-build.md`.
- [x] **build** (2026-06-19, Sonnet 4.6) — PR #37; `ImageError::LimitsExceeded` +
  `decode_limits()`/`map_image_decode_error()`/`decode_with_limits()` seam on the one
  choke point; `CliError` exit-1 arm. 388 tests green (7 image unit + 1 cli + 1 error +
  1 integration new); clippy/fmt/lean/deny clean. No deviations. subagent tokens=88701
  (~$0.48).
- [x] **verify** (2026-06-19, Opus Explore) — APPROVED, no concerns; adversarial
  choke-point-bypass grep clean, caps/mapping/exit-code confirmed, gates re-run green
  (388 tests). ~55k est.
- [x] **ship** (2026-06-19) — squash-merged PR #37 (4a55e0a); cost totals + ship
  reflection + archived to `specs/done/`. STAGE-006 backlog #1 complete.
