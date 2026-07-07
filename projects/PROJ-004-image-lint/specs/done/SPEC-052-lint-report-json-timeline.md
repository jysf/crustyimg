# SPEC-052 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.
In the claude-only variant the spec's `## Implementation Context` section IS the build handoff.

## Instructions

- [x] **design** — `lint --format json` (hand-rolled, no serde_json) + human polish specified with
  a synthetic-outcome golden test. (PROJ-004 framing, 2026-07-06.)
- [x] **build** — `src/lint/report.rs` (write_json + render_human + local escape_json) +
  Finding.bytes_saved + LintOutcome.potential_bytes_saved + global-`--format` wiring. 5 report unit
  (exact golden) + 3 integration tests; no new dep. PR #61. (2026-07-06.)
- [x] **verify** — all CI green on #61 (3-OS matrix, avif/webp-lossy, lean, msrv 1.89, cargo-deny);
  cross-OS golden stable. (2026-07-06.)
- [x] **ship** — squash-merged #61 → `main` (d903b2e); reflection + cost recorded; archived to
  `done/`. STAGE-013: 3 shipped / 1 pending. (2026-07-06.)
