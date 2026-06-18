# SPEC-024 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-024-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`), emitted DEC-026 (responsive command + HTML
  emission in-scope + scope/deferrals), and added a `responsive` entry to
  `docs/api-contract.md`. Composition over the resize `fit`-by-width primitive + the
  per-format sink; HTML is dependency-free string building. v1 = width×format
  variants + `<picture>`/srcset to stdout; blurhash/perceptual-per-variant/batch
  deferred.
- [ ] build — make the `## Failing Tests` pass: add `Commands::Responsive`,
  `run_responsive`, `parse_widths`/`parse_formats`/`mime_for_format`/
  `build_picture_html`, the dispatch arm, and unit + integration tests. Prompt:
  `prompts/SPEC-024-build.md`.
- [ ] verify — independent review: acceptance criteria, no-upscale/dedup correct,
  feature-gate exit 4 up front, decode-once, no decision drift, every named failing
  test exists, cost session recorded.
- [ ] ship — PR merge (pause for the user first), reflections, cost totals, archive
  to `specs/done/`, flip the STAGE-009 backlog line. **Index-verify before the ship
  commit** ([[verify-git-index-before-ship-commit]]).
