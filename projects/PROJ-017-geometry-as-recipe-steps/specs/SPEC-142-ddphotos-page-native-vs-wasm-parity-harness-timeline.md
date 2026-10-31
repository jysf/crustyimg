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
- [x] **build** (`prompts/SPEC-142-build.md`, 2026-10-30, Sonnet, ~$4.65). Draft PR #234, code head `f204d43`.
  Outcome (i), K = 1 on aarch64 and x86_64 (AMD EPYC 9V45); wasm hashes identical on both; JPEG bytes asserted on
  rows 9 and 11. The test bed's automated stage now fails: it reads GO_ADVANCE from `edit.js`, which moved to `recipe.js`.
- [x] **verify** (`prompts/SPEC-142-verify.md`, 2026-10-30, Opus, ~$5.20). ⚠ PUNCH LIST at `1b1c010`: K = 1 reproduced,
  every mutation matched, a second x86 CPU (EPYC 7763) agreed. Gaps: a crashing classifier makes the `changes` step
  report docs-only (CI skips green), and no golden text pins the preview recipe. Native `libm::powf` measured K = 0.
- [ ] **punch list** (`prompts/SPEC-142-punchlist.md`, Sonnet). Then a short re-verify.
- [ ] **ship**
