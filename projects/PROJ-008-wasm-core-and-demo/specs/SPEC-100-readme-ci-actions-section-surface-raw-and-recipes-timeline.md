# SPEC-100 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-100-<cycle>.md`.

## Instructions
- [x] design — framed build-ready 2026-07-20. Small README enhancement (ships with the README in 0.5.0):
  add a "Use it in CI" section with the two real GitHub Actions (`jysf/setup-crustyimg@v1` +
  `jysf/crustyimg-action@v1`, verified live, DEC-051) + working snippets, and surface RAW (embedded-preview
  extraction, honest) + recipes (declarative/reusable, CLI + browser via wasm `transform`) as
  differentiators in the positioning. Keep SPEC-082's human/non-AI voice + command sweep. Sonnet build /
  Opus verify. Complexity S.
