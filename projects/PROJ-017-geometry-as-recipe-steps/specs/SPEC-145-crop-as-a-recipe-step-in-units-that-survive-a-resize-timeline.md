# SPEC-145 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-145-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-04, `prompts/SPEC-145-frame.md`, Opus): GO, [M]. No crop op exists today
  (`unknown operation 'crop'`, exit 1, on 0.8.2). Recommends fractions-only units with per-edge
  rounding (DEC-091 Fence A already rules pixel coordinates lab-side), one op with exclusive
  rectangle/aspect key groups, and a pre-named split (DEC + rectangle, then aspect). Q1 (pixels
  in a recipe?) needs the maintainer before design.
- [x] **design** (2026-11-05, `prompts/SPEC-145-design.md`, Opus): DEC-112 (fractions only, f64,
  each edge rounded on its own with halves up; 0 px refused at apply as an op failure, not via the
  skippable refusal hook). 65 ACs: 32 load and 4 apply refusals with exact messages, 2 corpus recipes
  (`GOLDEN_LINES` 51 → 57), 3 wasm rows, 9 doc surfaces; 13-row mutation table. One spec, no split.
  Rulings (2026-11-05, all three as recommended): fractions only in a recipe, pixels via `edit --crop`
  later; `aspect` takes whole numbers only (`"239:100"`); a crop that rounds to 0 px is refused, not
  clamped.
- [ ] **build** (`prompts/SPEC-145-build.md`, not yet written)
- [ ] **verify** (`prompts/SPEC-145-verify.md`)
- [ ] **ship** (`prompts/SPEC-145-ship.md`)
