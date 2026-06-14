# SPEC-006 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-006-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-14
- [x] **build** — prompt: `prompts/SPEC-006-build.md`; PR #6 opened, 85 tests pass,
       four gates green, completed 2026-06-14 (corrected: build-time text wrongly said "merged")
- [x] **verify** — ✅ APPROVED (read-only) at code commit `2506082`; 85 tests (all prior
       SPEC-001..005 green after the OperationParams serde change), 3-OS CI green, round-trip
       typed-equality + recipe-drives-Pipeline confirmed, version/unknown-op/malformed-TOML all
       typed errors (no panic on hostile input). Flagged the false build-line text (fixed here). 2026-06-14.
- [x] **ship** — Merged PR #6 (squash) → main on 2026-06-14. Cost: 4 sessions, $null
       (build on Sonnet 4.6). Archived to done/.
