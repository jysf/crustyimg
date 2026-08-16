# SPEC-116 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-116-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. Spec written; 9 ACs, 3 settled design calls, 5 pre-written tests.
      Shipped on `main` via PR #166 (design-only, with SPEC-117).
- [x] **build** — 2026-08-15. PR #171, 16/16 CI green, 6 tests added, cycle advanced.
      $11.91 / 28,772,199 tokens / 104 min (Sonnet).
- [ ] **verify** — prompt: `prompts/SPEC-116-verify.md`. Opus, new session, own worktree.
      Two open items flagged at handoff: AC-6's test asserts build==apply on-branch rather
      than byte-identity vs `main`; and the Preserve/Pinned follow-up is filed only in
      Build Completion, which archives.
- [ ] **ship**
