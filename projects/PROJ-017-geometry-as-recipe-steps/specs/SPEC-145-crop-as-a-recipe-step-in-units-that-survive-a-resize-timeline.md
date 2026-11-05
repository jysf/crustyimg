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
- [ ] **design** (`prompts/SPEC-145-design.md`): writes the unit DEC, ACs, failing tests and
  Implementation Context, after the maintainer rules Q1-Q3.
- [ ] **build** (`prompts/SPEC-145-build.md`)
- [ ] **verify** (`prompts/SPEC-145-verify.md`)
- [ ] **ship** (`prompts/SPEC-145-ship.md`)
