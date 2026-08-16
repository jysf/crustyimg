# SPEC-119 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-119-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. 10 ACs, 4 settled design calls, 9 pre-written tests.
      The multi-frame sweep ran at design and changed the scope: the defect is
      **three formats (GIF, APNG, animated WebP), not one**, so complexity moved
      S → M against the stage's estimate.
- [ ] **build** — prompt not yet written. Sonnet, own worktree.
      ⚠ **Call 1 (warn vs refuse) is the maintainer-overturnable one.** If it
      flips to refuse, AC-2/AC-3 invert and the exit-code table changes.
- [ ] **verify** — Opus, new session.
- [ ] **ship**
