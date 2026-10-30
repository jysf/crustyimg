# SPEC-142 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-142-<cycle>.md`.

## Instructions

- [x] **frame** (2026-10-30, `prompts/SPEC-142-frame.md`) — GO, [S]. Native and wasm (full and
  slim) agree exactly on today's page recipes: 48/48 rows pixel-identical, 47/48 byte-identical
  (one lossless WebP differs in bytes, not pixels). Asserts engine parity (a) exactly; replay
  parity (b) goes to the crop spec. Evidence is aarch64 only: design must run it on x86_64 CI.
- [?] **design** (2026-10-30, `prompts/SPEC-142-design.md`) — BLOCKED on a ruling. The design is
  written in full (16 rows, Pillow 12.3.0 as decoder, `recipe.js` + `planEdit`, CI job), but the
  frame's premise fails at the page's own sizes: native and wasm `resize` differ by 1 px at 1/255
  at 1200 px (and 800 on portraits). Maintainer picks (A) engine fix first, exact, or (B) a bound DEC.
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
