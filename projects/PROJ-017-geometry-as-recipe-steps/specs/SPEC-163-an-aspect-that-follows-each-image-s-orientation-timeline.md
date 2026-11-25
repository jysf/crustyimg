# SPEC-163 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-163-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-25, Opus) — GO at [M], one spec; DEC-112 amendment recommended (rule 8). Driven: `aspect = "3:2"`
  crops a portrait 800×1200 to 800×533 and pads it to 1800×1200; the step sees the stored buffer unless
  `auto-orient` runs first. Rule: larger side along the buffer's larger side, strictly wider = landscape, square
  as written, `1:1` a no-op; 1000×1001 flips a 1000×667 crop to 667×1001. wasm needs no change.
- [ ] **rulings** (maintainer) — key name/values, the square rule, `edit` flags in or out, DEC-112 amendment vs DEC-118.
- [ ] **design** — `prompts/SPEC-163-design.md`
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
