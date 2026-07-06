# SPEC-049 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.
In the claude-only variant the spec's `## Implementation Context` section IS the build handoff.

## Instructions

- [x] **design** — `--explain`/`--explain=json` specified with a golden + determinism Failing-Tests
  set; DEC-049 (ExplainTrace schema / hand-rolled JSON / subset contract). (framing, 2026-07-05).
- [x] **build** — ExplainTrace + renderers in src/analysis/decide.rs (exact-JSON golden + 3 more
  unit tests) + --explain[=json] wiring (human→stderr, json→stdout) + 2 integration tests. Green on
  default/webp-lossy/lean/avif; no new dep. PR #56, 2026-07-06.
- [x] **verify** — CI green on #56 (all jobs); drift clean; live-verified (photo PNG → JPEG −43%). 2026-07-06.
- [x] **ship** — squash-merged #56 → main (c87e81e); reflection + cost recorded; archived to done/.
  Completed STAGE-012 + PROJ-002. 2026-07-06.
