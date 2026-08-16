# SPEC-116 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-116-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. Spec written; 9 ACs, 3 settled design calls, 5 pre-written tests.
      Shipped on `main` via PR #166 (design-only, with SPEC-117).
- [x] **build** — 2026-08-15. PR #171, 16/16 CI green, 6 tests added, cycle advanced.
      $11.91 / 28,772,199 tokens / 104 min (Sonnet).
- [x] **verify** — 2026-08-15, Opus. **⚠ PUNCH LIST** (3 items).
      $10.03 / 10,975,994 tokens / 129 min — **entry NOT yet in `cost.sessions`**: per AGENTS §13
      verify bookkeeping lands on `main` after the PR merges, and `main`'s copy of the spec is
      still `cycle: design`. The measured block is held in `prompts/SPEC-116-verify.md`'s readout
      and in the ship checklist below. Caveat from the verifier: measured before their final
      message, so it excludes ~4k output tokens (~$0.10).
      AC-6 ruled factually met (cross-version bytes driven identical, differing binary hashes as
      the control); AC-8 re-run in three stages; AC-9 re-baselined at +6/leg.
      Two new findings: a cache HIT swallows the warning (F1), and DEC-085's `affected_scope`
      was blind to its own second enforcement site (F2).
- [~] **punch list** — prompt: `prompts/SPEC-116-punchlist.md`. One branch-side item (rename +
      comment + Build Completion). Items 2 and 3 were `main` bookkeeping and are **already
      applied** by the orchestrator: STAGE-042 carries both bullets, DEC-085's scope now includes
      `src/cli/build.rs` and `tests/build.rs`.
- [ ] **ship** — after the punch-list item lands and verify re-approves. **At ship:** append the
      verify cost entry above to `cost.sessions`, compute `cost.totals`, run `just cost-audit`.
