# SPEC-004 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-004-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-14
- [x] **build** — prompt: `prompts/SPEC-004-build.md`
         PR #4, $null, completed 2026-06-14
- [x] **verify** — ✅ APPROVED (read-only) at commit `6352be0`; 55 tests pass, 4 local
       gates + 3-OS CI green, deps = image+thiserror+glob (tempfile dev-only, no walkdir),
       symlink-traversal guard empirically probed correct in dir + glob branches. 2026-06-14.
- [x] **ship** — Merged PR #4 (squash) → main on 2026-06-14. Cost: 4 sessions, $null
       (build on Sonnet 4.6). Archived to done/. Glob escape-check defensive note → STAGE-006.
