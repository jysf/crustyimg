# SPEC-030 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-030-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`); the LAST STAGE-004 command (text mode for
  `watermark`). Emitted **DEC-032** (`ab_glyph` rasterizer + bundled BSD-3 Go font via
  `include_bytes!`, NOT `imageproc` — it pulls sdl2/nalgebra). Two design-time probes:
  ab_glyph laid out + rasterized the Go font (with the `std`-feature trap found and
  documented), and imageproc's dep-tree (sdl2) ruled it out. Added `ab_glyph =0.2.32`
  + `assets/fonts/Go-Regular.ttf` (+ LICENSE) — `just deny` + lean build green. Design
  = render text → RGBA overlay → reuse the SPEC-029 `Watermark` compositing. Design +
  DEC-032 + font asset pushed to `main` before build.
- [ ] build — see `prompts/SPEC-030-build.md`.
