# SPEC-012 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-012-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-15 (Opus). Spec authored: thumbnail maps onto the shipped `Resize` op (`--square` ≡ resize `fill` NxN; plain ≡ resize `max` N), default `--size` 256; shared `run_pixel_op` fan-out helper extracted from `run_resize` (both call it). No new Operation, no new DEC. Complexity S. Build prompt: `prompts/SPEC-012-build.md`.
- [x] **build** — PR #13 opened 2026-06-15 (Sonnet 4.6 subagent). All four gates pass (171 tests; `cargo clippy --all-targets`). run_pixel_op extracted; run_thumbnail + thumbnail_params wired; stub test repointed to shrink; 10 integration + 4 unit thumbnail tests written and green. (Clean first-pass build — SPEC-010/011 lessons applied: incremental commits + explicit test-existence checklist.)
- [x] **verify** — ✅ APPROVED (read-only, Opus) at commit `6480c92`. Re-ran 4 gates cold (171 tests) + CI 6/6 green. `run_pixel_op` refactor confirmed byte-for-byte faithful (all 11 resize_* tests + resize unit tests green); thumbnail semantics proven hands-on (256×128 default, 64×64 square, JPEG preserved via --out-dir, --size 0→exit 2, no upscale); DEC-015 inherited; all 14 named tests present; src/cli+tests only; no new dep/DEC/CliError variant. No punch list. 2026-06-15.
- [x] **ship** — Merged PR #13 (squash) → main on 2026-06-15 (merge `95e664f`; clean merge). Cost: 4 sessions, $null (design/verify/ship Opus, build Sonnet 4.6 — subagent numerics null). Archived to done/. **STAGE-003: 3 of 6 shipped — resize op + resize CLI + thumbnail.**
