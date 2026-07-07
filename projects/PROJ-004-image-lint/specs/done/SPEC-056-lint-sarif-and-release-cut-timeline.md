# SPEC-056 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.
In the claude-only variant the spec's `## Implementation Context` section IS the build handoff.

## Instructions

- [x] **design** — `lint --format sarif` (hand-rolled SARIF 2.1.0, no dep) + the 0.4.0 release-cut
  (version + CHANGELOG, untagged) specified with a synthetic-outcome golden test. Follows SPEC-052's
  `write_json` discipline; SARIF was anticipated by DEC-050. (PROJ-004 STAGE-015, 2026-07-06.)
- [x] **build** — `write_sarif` (hand-rolled SARIF 2.1.0, version-param golden, cwd-relativized uris)
  + `--format sarif` wiring + README code-scanning snippet; staged 0.4.0 (Cargo.toml + CHANGELOG +
  compare-links + Cargo.lock). 4 unit + 2 integration tests; no new dep. PR #64. (2026-07-06.)
- [x] **verify** — all CI green on #64 (3-OS matrix, avif/webp-lossy, lean, msrv 1.89, cargo-deny);
  cross-OS SARIF golden stable. (2026-07-06.)
- [x] **ship** — squash-merged #64 → `main` (5ca1d8c); reflection + cost recorded; archived to
  `done/`. STAGE-015 complete (2/2); 0.4.0 staged (untagged — maintainer tags). (2026-07-06.)
