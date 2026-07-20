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
- [x] build — Sonnet, primary checkout. **Caught a stale framing premise:** the spec claimed the README
  "doesn't mention" the CI Actions, but a full "Continuous integration" section already existed (DEC-051).
  Verified against the real README + both `action.yml` (via `gh api`); added only the genuinely-new parts:
  2 "Why crustyimg" bullets (RAW embedded-preview + declarative recipes) + one action-inputs line. RAW
  extensions + recipe names verified against `src/`; 6-command sweep clean; no `src/` change.
- [x] verify — ✅ CLEAN (orchestrator inline review, proportionate to ~4 lines). Independently confirmed
  the RAW list (nef/cr2/dng/arw/…) + "and more" against `RAW_EXTENSIONS`, the bundled recipe names against
  `src/recipe/bundled.rs`, that the CI section wasn't duplicated, and the voice (no AI-tells).
- [x] ship — squash-merged PR #106 (**f3fc965**) 2026-07-20, CI CLEAN. README RAW/recipes/CI content
  complete; rides to crates.io at **0.6.0** (no 0.5.1). ~$2.0 (build $1.2 / verify $0.5 / ship $0.3).
  Lesson: verify a "gap" against the whole artifact before framing it ([[read-whole-function-before-asserting-a-gap]]).
