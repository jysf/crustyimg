# SPEC-146 — SHIP prompt: CI runs the wasm round-trip tests

Cycle: **ship**. Bookkeeping only. Code is on `main` as `517acea` (PR #245, merged 2026-11-05).
Verify round 1 returned a punch list (P1 `err()` to stdout, P2 floor call matched as a substring);
the fix round (`df3aade`) closed both; round 2 returned ✅ APPROVED.

Fresh CLI session, own worktree, Sonnet is enough: `just wt docs/spec-146-ship -- --model sonnet`.
The maintainer pastes your final message back to the orchestrator, so it must stand on its own.

## Step 0
`git fetch origin && git rebase origin/main`. Proceed only if `gh pr view 245 --json state` is MERGED and
the timeline's verify-round-2 line is `[x]` APPROVED. Otherwise stop and report.

## Read first
AGENTS §4, §13, §15 "During ship"; the spec in full (Frame rulings, Design, 11 ACs, Build Completion incl.
the Fix round); the timeline; HANDOFF "Traps this wave paid for"; STAGE-053 and STAGE-054 backlogs.

## Facts to reflect on
- **Design call wrong in build:** shape-check assertions 10/11 matched a substring the error line also contains.
- **Verify found what CI green could not:** `err()` wrote to stdout, so the lock/runner errors vanished inside
  `$(…)` captures in CI; the floor-call assertion accepted a commented-out, `|| true` or `echo`ed call. The
  design's mutation table did not list either. Both passed 19 green checks. Name the lesson: a guard's failure
  message path and its neutering by decoy need their own rows.
- **Cross-thread:** the exact count floor (47/45) goes red when #244 (SPEC-145, +2 wasm tests) lands; the
  second-merging PR must bump to 49/47. Record who bumped, if known (`git log origin/main -- scripts/wasm-ci-check.sh`).
- **Frame premise out of date:** `publish-npm.yml`'s dry-run also reads `hasAlpha` (npm_smoke.mjs) and went red on
  a `src/wasm.rs` mutation when the PR touched `justfile`; on a `src/wasm.rs`-only PR it does not run. Correct the
  frame sentence in the spec.
- **Spec text edits to make (orchestrator-requested):** header comment "CI runs every subcommand" in
  `scripts/wasm-ci-check.sh` is loose (CI never calls `compare-runner`): reword it in this ship PR; and point the
  spec's C10 "on main" procedure at main's own regex rather than a full copy of the new script (the copy carries
  the fix, so it is green).
- Optional hardening landed (continue-on-error, slim `if:`, job `env:` assertions): shape check is 16 assertions,
  self-test 18 rows.
- Costs recorded: build ~$2.15, verify $6.57, fix $0.59, verify-2 ~$1.95 (+ frame, design: take from the spec's
  cost block). Compute the total.

## Do
1. Fill the spec's reflection and cost totals; set `status: shipped`; `just archive-spec SPEC-146`.
2. STAGE-054 backlog: tick/remove the wasm-CI line if present; STAGE-053: the wasm32 CI leg is folded here, note
   it. (The orchestrator already filed the "neutered floor call" and "failed `changes` + `needs`" items on
   STAGE-053: leave them.) Re-derive both Count lines with grep.
3. `docs/roadmap.md` / AGENTS mention only if build left a stale line.
4. Leave `.github/workflows/ci.yml` alone. Leave the timeline file to the orchestrator.
5. Commit `git commit -s` (Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>); docs-only: push to `main`
   directly per AGENTS §13. Measure your own cost (dedup by `.message.id`, MAX output, per-component pricing;
   find the transcript by searching `SHIP prompt: CI runs the wasm round-trip`).

## Report
First line: `SPEC-146 ship: archived, <commit>`. Then: what changed, totals, the two spec-text edits, counts,
anything out of spec, your cost.
