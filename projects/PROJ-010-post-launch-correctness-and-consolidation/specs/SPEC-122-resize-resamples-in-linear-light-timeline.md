# SPEC-122 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-122-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-16. Framed **as a pair with its sibling** — SPEC-121 and SPEC-122
      change output bytes for every recipe, share one DEC and one migration, and should be
      sequenced together so that cost is paid once.
      **Design's largest finding: the migration already exists.** `cache_key_for` includes
      `crate::version()` (`src/cli/build.rs:294`) and the lockfile never promised output-hash
      stability across versions (`src/build/lock.rs:32-36`), so the "invalidates every PROJ-007
      lockfile" risk both backlog entries flagged is already within the shipped contract. The
      builds drive it; they do not design it.
- [ ] **build** — prompt: `prompts/SPEC-122-build.md` (2026-08-16). Sonnet, own worktree, branch
      `fix/spec-122-resize-resamples-in-linear-light`. Third of three serial specs.
      ⛔ **Gated on SPEC-121 being merged to `main`** — same function, and the prompt opens with
      the check. **Amends DEC-095; does not mint a new DEC.** Its colour-type tests become
      regression guards on this change.
- [ ] **verify** — Opus, new session.
- [ ] **ship**
