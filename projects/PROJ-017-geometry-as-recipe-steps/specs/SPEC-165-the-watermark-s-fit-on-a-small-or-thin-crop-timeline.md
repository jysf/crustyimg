# SPEC-165 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-165-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-27, `prompts/SPEC-165-frame.md`, Opus): GO at [M]. Of 37.7M planned runs, 465,951
  are refused (fit-w 288,078, fit-w-margin 166,680, coverage 11,193; 0 without a crop), and 97.6 % have no
  size that fits. Recommend asking the engine through a wasm export (+697 B brotli on `pkg-slim`), with
  2 px of headroom for DEC-118's original, and leaving the mark off with a note when nothing fits.
  **Ruled 2026-11-28:** (c), one spec, no floor; and when nothing fits, **Make is blocked** with the
  fixes named (larger box, shorter text, watermark off), rather than the mark left off.
- [x] **design** (2026-11-28, `prompts/SPEC-165-design.md`, Opus): a verdict export `watermarkRefusal`
  (+966 B brotli), called by `planEdit` through a callback that the worker satisfies with a `plan`
  message. A blocked plan disables Make, with the reason beside it, and renders nothing unmarked.
  15 fit, 3 replay, 5 export and 3 page rows, all literal and driven; portrait originals driven
  (at most 1 px per side).
- [~] **build** (`prompts/SPEC-165-build.md`, Sonnet)
- [ ] **verify**
- [ ] **ship**
