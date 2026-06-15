# SPEC-007 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-007-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-14
- [x] **build** — PR #7 opened 2026-06-14; all four gates pass (97 tests green, clippy clean, fmt clean)
- [x] **verify** — ✅ APPROVED (read-only) at commit `1872b74`; 97 tests (all prior SPEC-001..006
       intact, smoke.rs not weakened), 3-OS CI green incl. Windows, `apply` confirmed genuinely
       end-to-end (real inverted PNG), exit-code mapping matches contract, clap-free pixel core,
       accurate timeline marker. 2026-06-14.
- [x] **ship** — Merged PR #7 (squash) → main on 2026-06-14. Cost: 4 sessions, $null
       (build on Sonnet 4.6). Archived to done/. **Completes STAGE-001 (7/7).**
