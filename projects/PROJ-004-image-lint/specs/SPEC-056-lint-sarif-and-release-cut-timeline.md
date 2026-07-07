# SPEC-056 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.
In the claude-only variant the spec's `## Implementation Context` section IS the build handoff.

## Instructions

- [x] **design** — `lint --format sarif` (hand-rolled SARIF 2.1.0, no dep) + the 0.4.0 release-cut
  (version + CHANGELOG, untagged) specified with a synthetic-outcome golden test. Follows SPEC-052's
  `write_json` discipline; SARIF was anticipated by DEC-050. (PROJ-004 STAGE-015, 2026-07-06.)
- [ ] **build** — add `write_sarif` (mirror `write_json`; relativize the location uri; version as a
  param) + `--format sarif` wiring + the README code-scanning note; then stage 0.4.0 (Cargo.toml +
  CHANGELOG). Reads SPEC-052's report.rs + DEC-049 first.
- [ ] **verify**
- [ ] **ship**
