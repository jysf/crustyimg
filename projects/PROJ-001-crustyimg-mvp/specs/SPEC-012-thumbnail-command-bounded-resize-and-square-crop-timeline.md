# SPEC-012 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-012-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-15 (Opus). Spec authored: thumbnail maps onto the shipped `Resize` op (`--square` ≡ resize `fill` NxN; plain ≡ resize `max` N), default `--size` 256; shared `run_pixel_op` fan-out helper extracted from `run_resize` (both call it). No new Operation, no new DEC. Complexity S. Build prompt: `prompts/SPEC-012-build.md`.
- [x] **build** — completed 2026-06-15 (Sonnet 4.6 subagent). All four gates pass (171 tests). run_pixel_op extracted; run_thumbnail + thumbnail_params wired; stub test repointed to shrink; 10 integration + 4 unit thumbnail tests written and green. PR #N opened (see PR).
