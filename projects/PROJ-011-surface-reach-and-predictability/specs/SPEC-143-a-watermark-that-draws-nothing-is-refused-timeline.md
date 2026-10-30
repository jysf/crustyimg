# SPEC-143 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-143-<cycle>.md`.

## Instructions

- [x] **frame** (2026-10-30, `prompts/SPEC-143-frame.md`): GO, [S]. Premise confirmed on the CLI,
  `apply` and `build`: blank text, size 0.01, opacity 0, colour alpha 00 and a transparent image
  all exit 0 with 0 px changed. `visible_pixels` is counted before opacity, so design adds a
  separate post-opacity max-alpha predicate, decided from the mark (as DEC-104 is), at two call
  sites: `run_watermark` and `from_params`.
- [x] **design** (2026-10-30, `prompts/SPEC-143-design.md`): 14 ACs, 16 failing tests, DEC-109.
  `Watermark::invisible_mark()` (additive) refuses at 16-bit precision via the composite's own
  `faded_alpha16`, at the end of `from_params` and in `run_watermark`. Only opacity 0 and colour
  alpha 00 are flag checks: tab and U+3000 draw the font's fallback box, so blank text is not.
- [ ] **build**: `prompts/SPEC-143-build.md`
- [ ] **verify**
- [ ] **ship**
