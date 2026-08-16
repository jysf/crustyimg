# SPEC-117 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-117-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. Spec written; 6 ACs, 2 pre-written tests, neither red on HEAD by
      design. AC-5's per-verb negative control is the load-bearing criterion.
      Shipped on `main` via PR #166 (design-only, with SPEC-116).
- [x] **build** — 2026-08-16, Sonnet. PR #174, 16/16 CI green, 2 tests added, no `src/` changes.
      $23.06 / 62,804,433 tokens / 62 min. AC-5's per-verb control run as two independent reverts
      (build's wrapper, apply's pinned short-circuit), each turning only its own verb's test red.
- [ ] **verify** — prompt: `prompts/SPEC-117-verify.md`. Opus, new session, own worktree.
      Three open items flagged at handoff: the AC-6 baseline was counted (`grep -c '#[test]'`)
      rather than run; `build`'s AC-4 is satisfied by an interpretation (written name + lockfile,
      since `build` has no summary); and the exact-`WebP` assertion may encode today's codec race.
- [ ] **ship**
