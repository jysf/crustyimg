# SPEC-142 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-142-<cycle>.md`.

## Instructions

- [x] **frame** (2026-10-30, `prompts/SPEC-142-frame.md`) — GO, [S]. Native and wasm (full and
  slim) agree exactly on today's page recipes: 48/48 rows pixel-identical, 47/48 byte-identical
  (one lossless WebP differs in bytes, not pixels). Asserts engine parity (a) exactly; replay
  parity (b) goes to the crop spec. Evidence is aarch64 only: design must run it on x86_64 CI.
- [x] **design** (2026-10-30, `prompts/SPEC-142-design.md`; amend `prompts/SPEC-142-design-amend.md`)
  — Ruled (B), **DEC-110**: dimensions exact, no-resample rows (1, 3, 9, 11) exact, resampling rows
  within 1/255 on at most K pixels (K from the build's runs, cap 78). FT-3 gains comparator cases B1–B7.
- [ ] **build** (`prompts/SPEC-142-build.md`, Sonnet). Orchestrator ruling: JPEG bytes asserted on rows 9 and 11
  only (10 and 12 resample). Carries SPEC-141's cross-target evidence and records the runner's CPU next to K.
- [ ] **verify**
- [ ] **ship**
