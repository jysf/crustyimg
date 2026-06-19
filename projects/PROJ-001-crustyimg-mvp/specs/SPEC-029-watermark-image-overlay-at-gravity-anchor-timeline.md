# SPEC-029 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-029-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`); the compositing half of STAGE-004 and the first
  multi-image `Operation`. Emitted **DEC-031** (overlay loaded at the IO boundary /
  CLI, op holds in-memory pixels, `apply()` file-free, not in `with_builtins()` —
  recipe round-trip deferred to STAGE-005). A design-time probe confirmed
  `image::imageops` overlay/alpha-opacity/resize/clip primitives — **no new dep**.
  `Gravity` enum + `Watermark` op + `run_watermark` (IO boundary) over `run_pixel_op`.
  Fleshed out the api-contract entry. Design + DEC-031 pushed to `main` before build.
- [ ] build — see `prompts/SPEC-029-build.md`.
