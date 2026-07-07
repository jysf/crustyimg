# SPEC-050 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.
In the claude-only variant the spec's `## Implementation Context` section IS the build handoff.

## Instructions

- [x] **design** — `lint` command core specified (Rule/Finding/Severity framework, source
  resolution, human output, exit-7 reuse, 2 foundational rules) with Failing Tests; DEC-050 lands
  with it. (PROJ-004 framing, 2026-07-06.)
- [x] **build** — `src/lint/mod.rs` (Severity/Finding/Rule/LintTarget + runner + human report +
  `privacy/gps-metadata-leak` + `size/truncated-or-corrupt`) + `lint` subcommand + `jpeg_with_gps`
  fixture + `tests/lint.rs`. 5 unit + 5 integration tests green; no new dep. PR #59. (2026-07-06.)
- [x] **verify** — all CI green on #59 (3-OS matrix, avif/webp-lossy, lean, msrv 1.89, cargo-deny);
  read-only + determinism + exit-code mapping confirmed. (2026-07-06.)
- [x] **ship** — squash-merged #59 → `main` (14e425b); reflection + cost recorded; archived to
  `done/`. STAGE-013: 1 shipped / 3 pending. (2026-07-06.)
