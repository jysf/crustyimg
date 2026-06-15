# SPEC-010 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-010-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-15 (Opus architect; SPEC-010 = library half of split `resize`; emitted DEC-014; build prompt + failing tests authored; fast_image_resize 5.5.0 API verified against the repo's `image v0.25.10`)
- [x] **build** — PR #11 opened (2026-06-15); all 136 tests pass; all four gates green; verify follow-up: Resize::apply made total (invariant unwraps → typed errors, 136 tests still pass)
