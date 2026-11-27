# SPEC-165 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-165-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-27, `prompts/SPEC-165-frame.md`, Opus): GO at [M]. Of 37.7M planned runs, 465,951
  are refused (fit-w 288,078, fit-w-margin 166,680, coverage 11,193; 0 without a crop), and 97.6 % have no
  size that fits. Recommend asking the engine through a wasm export (+697 B brotli on `pkg-slim`), with
  2 px of headroom for DEC-118's original, and leaving the mark off with a note when nothing fits.
- [ ] **design** (`prompts/SPEC-165-design.md`)
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
