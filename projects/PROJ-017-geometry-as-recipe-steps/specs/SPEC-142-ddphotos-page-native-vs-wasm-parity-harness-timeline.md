# SPEC-142 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-142-<cycle>.md`.

## Instructions

- [x] **frame** (2026-10-30, `prompts/SPEC-142-frame.md`) — GO, [S]. Native and wasm (full and
  slim) agree exactly on today's page recipes: 48/48 rows pixel-identical, 47/48 byte-identical
  (one lossless WebP differs in bytes, not pixels). Asserts engine parity (a) exactly; replay
  parity (b) goes to the crop spec. Evidence is aarch64 only: design must run it on x86_64 CI.
- [ ] **design** — `prompts/SPEC-142-design.md`
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
