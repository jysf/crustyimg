# SPEC-051 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.
In the claude-only variant the spec's `## Implementation Context` section IS the build handoff.

## Instructions

- [x] **design** — `.crustyimg-lint.toml` config + severity/select CLI specified with Failing
  Tests. (PROJ-004 framing, 2026-07-06.)
- [x] **build** — `src/lint/config.rs` (LintConfig + discovery + effective_config/merge +
  validation) + runner integration (select/ignore/off/per-file-ignores/severity override) +
  LintTarget budget/width/threshold plumbing + LintFlags CLI. 8 unit + 5 integration tests; no
  new dep. PR #60. (2026-07-06.)
- [x] **verify** — all CI green on #60 (3-OS matrix, avif/webp-lossy, lean, msrv 1.89, cargo-deny).
  (2026-07-06.)
- [x] **ship** — squash-merged #60 → `main` (236581e); reflection + cost recorded; archived to
  `done/`. STAGE-013: 2 shipped / 2 pending. (2026-07-06.)
