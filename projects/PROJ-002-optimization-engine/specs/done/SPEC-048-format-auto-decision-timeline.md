# SPEC-048 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.
In the claude-only variant the spec's `## Implementation Context` section IS the build handoff.

## Instructions

- [x] **design** — format auto-decision engine specified with pure winner-rule + integration
  Failing Tests; DEC-048 (engine/profiles/AVIF-budget/clear-win-guard). (framing, 2026-07-05).
- [x] **build** — src/analysis/decide.rs (pure shortlist + winner + clear-win guard, 13 tests) +
  optimize autodecide path (--profile web|docs|preserve, one-line summary) + 7 integration tests.
  Green on default/webp-lossy/lean/avif; no new dep. PR #55, 2026-07-06.
- [x] **verify** — CI green on #55 (all jobs incl. avif/webp/lean/msrv/cost-data); drift clean. 2026-07-06.
- [x] **ship** — squash-merged #55 → main (494eb05); reflection + cost recorded; archived to done/. 2026-07-06.
