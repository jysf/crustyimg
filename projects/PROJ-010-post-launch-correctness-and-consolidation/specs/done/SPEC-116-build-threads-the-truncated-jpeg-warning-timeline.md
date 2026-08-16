# SPEC-116 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-116-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. Spec written; 9 ACs, 3 settled design calls, 5 pre-written tests.
      Shipped on `main` via PR #166 (design-only, with SPEC-117).
- [x] **build** — 2026-08-15, Sonnet. PR #171, 16/16 CI green, 6 tests added.
      $11.91 / 28,772,199 tokens / 104 min.
- [x] **verify** — 2026-08-15, Opus. **⚠ PUNCH LIST** (3 items).
      $10.03 / 10,975,994 tokens / 129 min. Caveat from the verifier: measured before their final
      report message, so it excludes ~4k output tokens (~$0.10).
      AC-6 ruled factually met (cross-version bytes driven identical, with differing binary hashes
      as the control); AC-8 re-run in three stages; AC-9 re-baselined at +6 on every leg; the extra
      AC-7 test mutation-proven non-vacuous.
      Two new findings: a cache HIT swallows the warning (F1), and DEC-085's `affected_scope` was
      blind to its own second enforcement site (F2).
- [x] **punch list** — 2026-08-15, `8e62c98`. One branch-side item: renamed the AC-6 test to
      `build_and_apply_agree_on_bytes_for_a_clean_input`, rewrote its doc comment, recorded the
      cross-version evidence in Build Completion. No source changes, no new tests. Same build
      cycle, so no separate cost entry.
      Items 2 and 3 were `main`-side bookkeeping, applied by the orchestrator before this ran:
      STAGE-042 gained the cache-hit bullet, DEC-085's scope gained `src/cli/build.rs` and
      `tests/build.rs`.
- [x] **ship** — 2026-08-15. PR #171 squash-merged as `016e89f`, 16/16 CI green after
      `update-branch`. Verify cost appended and independently re-derived at ship — sum and dollar
      figure both match to the cent. `cost.totals` = 39,748,193 tokens / **$21.94** / 4 sessions;
      `just cost-audit` passes.
      Also corrected on `main`: the `## Failing Tests` roster still named the renamed test — a
      dangling reference the punch list introduced.
      **A second verify cycle was deliberately skipped** — maintainer's call, 2026-08-15. Verify
      had already driven the matrix, the negative control, the mutation check and the
      cross-version bytes, and the punch-list change was a rename plus a comment. Recorded as a
      knowing deviation from the cycle model, not a drift.
      **This closes STAGE-043.**
