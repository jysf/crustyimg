# SPEC-083 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-083-<cycle>.md`.

## Instructions
- [x] design — framed build-ready 2026-07-20 (build AFTER the 0.5.0 cut, so the crustyimg side is the
  shipped 0.5.0 surface). BENCHMARKS.md = honest, EQUAL-QUALITY, reproducible cross-tool comparison vs
  sharp/`@squoosh/cli`/ImageMagick on size+speed, off a real `--corpus` (committed CC0 corpus is all
  <2048px — SPEC-088 carry). Extends the SPEC-088 `just bench` discipline to competitors. Matched-quality
  is THE credibility question (show the quality column, not just smallest); state machine/versions/exact
  commands; report losses honestly. Sonnet build / Opus verify. Complexity M.
