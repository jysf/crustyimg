# SPEC-057 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.
In the claude-only variant the spec's `## Implementation Context` section IS the build handoff.

## Instructions

- [x] **design** — the GitHub Actions on-ramp specified (setup-crustyimg + crustyimg-action in their
  own repos + in-repo pre-commit/recipe/docs glue) with Failing Tests; DEC-051 pins the wrap-the-
  installer / two-repo / no-crate-dep contract. (PROJ-004 STAGE-015, 2026-07-06.)
- [ ] **build** — create the two Action repos (composite `action.yml` + README + 3-OS self-test),
  push, and drive their self-tests green; land the in-repo glue (`.pre-commit-hooks.yaml`, `just
  lint-images`, README CI section) via a PR. Reads DEC-051 + the real release installer first.
- [ ] **verify**
- [ ] **ship**
