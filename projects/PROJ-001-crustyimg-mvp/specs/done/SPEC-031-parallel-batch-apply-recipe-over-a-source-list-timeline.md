# SPEC-031 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-031-<cycle>.md`.

## Instructions

- [x] design (2026-06-19) — Opus, main loop. Activated STAGE-005; authored the spec
  (`## Failing Tests` + `## Implementation Context`); first STAGE-005 spec = the thesis
  payoff (parallel batch `apply --recipe`). Emitted **DEC-033** (indicatif); rayon is
  pre-justified by DEC-006. Added `rayon =1.12.0` + `indicatif =0.18.4` — `just deny` +
  lean build green. Key design pinned: `Operation` is not `Send`, so each rayon task
  REBUILDS its pipeline from the (Sync) recipe + registry; mirror `run_pixel_op`'s
  fan-out + exit-6. Recipe load/validation reused from SPEC-006. Design + DEC-033
  pushed to `main` before build. **Build runs on Sonnet** (prescriptive prompt).
- [x] build (2026-06-19, PR #35) — **first Sonnet 4.6 build** (new policy: build=Sonnet,
  verify=Opus). Metered subagent, 108k tok / ~7 min / ~$0.59. Rewrote `run_apply` into a
  rayon-parallel batch + `apply_one` worker (per-task pipeline rebuild — `Operation` not
  `Send`), `-j` bounded local thread pool, indicatif progress (stderr, hidden on
  `--quiet`), name-template output, exit-6 partial failure. Single-input path preserved.
  10 new tests (2 unit + 8 integration). No new op, no new DEC, lean build green.
- [x] verify (2026-06-19) — independent read-only Explore subagent on **Opus**: ✅
  APPROVED, no concerns. Thoroughly validated concurrency correctness (no shared
  `Box<dyn Operation>`, locally-scoped thread pool, no data races/unsafe), all exit codes
  (2/6/3/1), `-j` determinism, preserved single-input path. Orchestrator re-ran gates:
  `cargo test` 367 ok (0 failed), clippy/fmt/deny clean, `cargo build --no-default-features`
  (lean) clean. The Sonnet build cleared Opus verify cleanly — model split validated.
- [x] ship (2026-06-19, PR #35 squash-merged → `99f51cf`) — reflections + cost totals
  filled (build 108469 real Sonnet / verify ~55k est Opus; totals 163469 / $1.09 / 4),
  STAGE-005 backlog flipped, archived to `specs/done/`, `just cost-audit` green +
  cost-capture + lean build confirmed on main CI.
