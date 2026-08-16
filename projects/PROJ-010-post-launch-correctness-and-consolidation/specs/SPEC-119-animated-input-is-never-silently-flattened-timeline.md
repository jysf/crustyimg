# SPEC-119 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-119-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. 10 ACs, 4 settled design calls, 9 pre-written tests.
      The multi-frame sweep ran at design and changed the scope: the defect is
      **three formats (GIF, APNG, animated WebP), not one**, so complexity moved
      S → M against the stage's estimate.
- [~] **build** — prompt: `prompts/SPEC-119-build.md`. Sonnet. Worktree `../crustyimg-spec119`
      on `fix/spec-119-animated-input-never-silently-flattened`, created 2026-08-16 off `caf2739`.
      ✅ **Call 1 RULED 2026-08-16: warn and proceed**, maintainer-confirmed with a recorded
      reservation. The reservation is answered by `lint --max-warnings 0`, which already exits
      non-zero — `lint` is the gate, `convert` is the tool.
      ⚠ **That ruling REVISED Call 4**: because `lint` is now the designated strict path, the
      rule can no longer stay GIF-only, or APNG and animated-WebP users get no strict option at
      all. New **AC-7b** drives `lint --max-warnings 0` on all three families.
- [ ] **verify** — Opus, new session.
- [ ] **ship**
