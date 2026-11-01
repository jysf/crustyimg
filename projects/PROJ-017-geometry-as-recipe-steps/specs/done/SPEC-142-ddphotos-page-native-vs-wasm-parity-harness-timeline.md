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
- [x] **punch list** (`prompts/SPEC-142-punchlist.md`, 2026-10-30, Sonnet, $1.47). PR #234 code head `e71acbd`, CI green.
  The `changes` step fails on a classifier crash; P5 pins row 12; native hashes and a K summary line print. A third
  x86 CPU (Xeon 8370C) agreed at K = 1.
- [x] **re-verify** (`prompts/SPEC-142-reverify.md`, 2026-10-31, Opus, $2.34). ⚠ PUNCH LIST at `474c9fc`: both
  items closed (classifier crash now fails the step; P5's derived row-12 literal pins placement, byte-equal to the
  pasted text; native hashes match Mac vs Xeon on all 16 rows). One spec sentence named the wrong run for the 7763.
- [x] **punch list 2** (`prompts/SPEC-142-punchlist-2.md`, 2026-10-31, Sonnet, ~$0.75). PR head `1001831`, CI green.
  The sentence corrected against all three run logs; the summary line counts and names table failures
  (`rows-threw` and `resize-set` labels added).
- [x] **re-verify 2** (`prompts/SPEC-142-reverify-2.md`, 2026-10-31, Opus, $1.39). ✅ APPROVED at `1001831`: every
  table check counted and named, N1/N2/B1–B10 unchanged, clean run `0 row failures, 0 table failures, K=1`.
  The orchestrator updated the branch from `main` (head `e26339b`, CI green) and kept it a draft for the test bed's
  `check.py` fix. That session had not run by 2026-11-01; on the maintainer's call the orchestrator marked PR #234
  ready at `5d20736` (CI green). Until the test-bed fix runs, its automated stage crashes on `GO_ADVANCE`.
- [x] **ship** (`prompts/SPEC-142-ship.md`, 2026-11-01, Sonnet, ~$1.9). PR #234 merged 2026-11-01 as `c8aa9c03c9f49d34b3633d36f3e2f0af9f46e418`.
  Reflected and archived. Total spec cost ~$36.8 across 10 sessions (cost entries re-measured from transcripts at ship).
  One follow-up filed in STAGE-060 (label the harness's P5 checks); HANDOFF trap proposed, not applied.
