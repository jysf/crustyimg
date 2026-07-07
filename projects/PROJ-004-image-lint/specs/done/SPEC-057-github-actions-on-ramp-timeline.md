# SPEC-057 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.
In the claude-only variant the spec's `## Implementation Context` section IS the build handoff.

## Instructions

- [x] **design** — the GitHub Actions on-ramp specified (setup-crustyimg + crustyimg-action in their
  own repos + in-repo pre-commit/recipe/docs glue) with Failing Tests; DEC-051 pins the wrap-the-
  installer / two-repo / no-crate-dep contract. (PROJ-004 STAGE-015, 2026-07-06.)
- [x] **build** — created + pushed `jysf/setup-crustyimg` + `jysf/crustyimg-action` (composite
  actions + README + 3-OS self-test, both GREEN); landed the in-repo glue (`.pre-commit-hooks.yaml`,
  `just lint-images`, README CI section, `tests/adoption_glue.rs`) via PR #63. No new crate dep.
  (2026-07-06.)
- [x] **verify** — both Action 3-OS self-tests green (install/annotate/exit) + PR #63 CI green
  (matrix/feature/lean/msrv/deny). (2026-07-06.)
- [x] **ship** — squash-merged #63 → `main` (b6ee724); reflection + cost recorded; archived to
  `done/`. STAGE-015: 1 shipped / 1 pending (SPEC-056 = SARIF + 0.4.0 cut). (2026-07-06.)
