# SPEC-162 — VERIFY prompt: CI runs each test once

Cycle: **verify**. Run it in a fresh CLI session, in its own worktree, with **Opus**, and
**read-only**. You did not frame, design or build SPEC-162. The orchestrator wrote the spec without
separate frame and design sessions, so nobody has checked its design. Your job is to find what the
build's evidence can't show. The build reports 10 of 10 ACs met and a measured saving. Treat both as
claims until you have checked them.

```
just wt verify/spec-162 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin chore/spec-162-ci-runs-each-test-once:refs/remotes/origin/chore/spec-162-ci-runs-each-test-once
git checkout --detach origin/chore/spec-162-ci-runs-each-test-once
```

When this was written, draft PR #271's head was `e54e5b5`. The last commit that changed code was
`ecf0dc7`, and its run `45539781614` passed. `e54e5b5` adds only spec text, so its heavy jobs were
skipped. Record the SHA you verify.

The maintainer pastes your final message back to the orchestrator, so it must stand on its own.
**Write it as plain lists, not box-drawn tables.**

⛔ **Don't merge PR #271 until this verify has reported, along with any punch list and re-verify it
leads to.**

## What the build claims
- **Runner-minutes:** 102.6 → 62.3, comparing run `45501714010` with run `45539781614`.
  - The slowest leg went from 17.9 to 10.7 min (heic on macOS).
  - The Test step got faster on every leg.
- **The cache was kept.** Build + Test fell 42.9% on Ubuntu, 22.5% on macOS and 46.6% on Windows.
  - The repository's caches total 3.48 GB across 9 entries; each `check` entry is 563–641 MB.
  - The probe's nested build was warm in run 2: `avif_tile_pin` took 20.5 s, against 149.9 s at
    baseline.
- **The probe is incremental.**
  - AC-2: the second local run took 1 s and compiled nothing.
  - AC-3: no temp folders were left behind.
  - AC-6: two concurrent runs both passed, and the second waited on cargo's lock.
- **Mutants:**
  - M1 turned both tests red.
  - M2 turned `both_encode_paths_set_the_thread_count` red, and only that test.
  - M4 found nothing wrong, so the per-checkout hash is "cheap insurance, not proven necessary".
- **M3 didn't match the table.**
  - With two worktrees sharing one `CARGO_TARGET_DIR`, A's second run reused the library and test
    binary that B had just built. Those still had B's manifest path, so the run went red.
  - `both_encode_paths_set_the_thread_count`, which never uses the probe, went red too.
  - The orchestrator's AC-5 expected A to stay green. That expectation was the design's mistake:
    cargo's outer build goes stale across worktrees that share a target folder, whatever the
    probe does.
- **Skips:** the three `check` legs ran the probe test, and the three feature legs show it as
  `1 filtered out`. `--list` matches the pattern on exactly one test.

## Read first
- **The spec on the PR branch:** the ACs, the mutation table, `## Outputs`, "Out of scope" and
  `## Build Completion`.
- **The PR's diff in full:** `git diff origin/main...HEAD`.
- **AGENTS §15:**
  - the coverage map;
  - "a bound or 'can't happen' is a claim until its worst input is driven";
  - the finding classes.
- **HANDOFF traps 11 and 15** (`projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`).

## Check, in order
1. **Scope.**
   - `git diff --stat origin/main...HEAD` shows only `ci.yml`, `tests/avif_tile_pin.rs` and the
     spec.
   - Every commit is signed off.
   - The required checks are still the three `check` legs:
     `gh api repos/jysf/crustyimg/branches/main/protection/required_status_checks --jq .contexts`.
2. **Re-derive the numbers from the API, not from the report.**
   - For runs `45501714010` and `45539781614`, sum each job's `completed_at - started_at`, and read
     each leg's Build and Test steps (`gh api repos/jysf/crustyimg/actions/runs/<id>/jobs`).
   - Quote the totals and per-leg figures. Name any figure that differs from the report by more
     than 5%.
   - Look at run 1 too (the cache-cold run, reported as 86.9 min), and say whether it is the run
     the report means.
   - Say whether macOS's −22.5% is one sample against a 20% bar. If another warm run exists on the
     branch, say what it shows. You're read-only, so don't trigger a rerun.
3. **The cache on a docs-only push.** The `rust-cache` step has no
   `if: needs.changes.outputs.code == 'true'`, unlike every step after it.
   - From the `e54e5b5` run's three `check` jobs, give how long the cache restore took, how much
     it downloaded, and whether the post step saved anything.
   - A docs-only push used to skip the heavy steps for almost nothing. Say what it costs now, per
     leg and in total.
   - Classify it. If a docs-only run now does real work for no gate, that is a **claim** about
     the PR's saving and about the `changes` job's comment.
4. **Where the cache lives, and how big it gets.**
   - List every cache entry with its key, ref and size:
     `gh api repos/jysf/crustyimg/actions/caches --paginate --jq '.actions_caches[] | "\(.key) \(.ref) \(.size_in_bytes)"'`.
   - Check the PR's comment that "the cache key already includes the OS". Name what in the key
     makes the three legs distinct.
   - Check the comment that it restores "from the last run of this branch, or of main". Say what a
     PR can read: caches on `refs/pull/N/merge` and on `main`, and nothing from other PRs.
   - After the merge, `main` saves its own three entries. Estimate the steady-state total, with a
     few PRs open at once, against GitHub's 10 GB repository limit, and say what happens at the
     limit (eviction, not failure).
   - The probe folder lives under `target/tmp`. Say whether rust-cache's cleanup keeps it on
     purpose or by accident: read the action's documented cleanup, or the `Post` step log. If it
     keeps it by accident, the 20.5 s depends on undocumented behaviour; say so.
5. **The probe's doc comment is a claim.** It says two worktrees sharing one `CARGO_TARGET_DIR`
   "would otherwise share a probe, and the second could be served the first one's binary".
   - M4 never showed that, and M3 shows the outer test binary is stale in that setup anyway.
   - Drive M4 yourself in the order most likely to expose it: build in B with the pin removed,
     then in A with the source touched, both on a plain `avif-probe` folder.
   - Then classify the sentence. Does the hash stay, as insurance with honest wording, or go?
6. **The skip.**
   - `-- --skip` is passed to every test binary and to the doctests. Confirm it filters exactly
     one test in the whole suite, on a feature build (`--features webp-lossy --list`).
   - Confirm the feature legs' logs show `1 filtered out` for `avif_tile_pin`, and `0 filtered out`
     for every other binary.
   - Then name the risk the other way round. If the test is renamed, the skip silently filters
     nothing, and the feature legs pay the probe build again. That is slower, never a lost check.
     Say whether that needs a guard. The orchestrator thinks it doesn't.
7. **Coverage moved, not lost.**
   - The `avif` job claimed to test avif "without the other defaults" and never did. Confirm this
     from its removed commands.
   - The probe test now runs on the three `check` legs only. Is there anything in it that the
     webp-lossy or heic features change? Check whether the probe's
     `cargo build --features image/rayon` takes any feature from the outer command.
8. **AC-5 and the spec text.**
   - Say what AC-5 should have said. The orchestrator suggests: a shared `CARGO_TARGET_DIR` across
     worktrees is unsupported for this test, as for every other.
   - Say whether the spec's AC-5 and M3 row are now **wording** items.
   - Say whether `## Build Completion` marks AC-5 "met" while M3 failed. If so, that's a wording
     item too.
9. **The docs sweep.** The build listed DEC-020:146, DEC-081:188–190 and two `docs/sessions` files.
   Say whether any of them states the `avif` job as current, outside a dated record. Report only;
   the spec leaves DECs as written.
10. **The local matrix:** `fmt --check`, `clippy --all-targets -D warnings` and the full test, on
    default, `--no-default-features` and `--features webp-lossy`. Use one fresh `CARGO_TARGET_DIR`
    per leg, under your scratchpad.

## Classify every finding (AGENTS §15)
- **wording:** a doc, comment or spec line that misstates what the code already does.
- **assert:** a missing or loose test row, where the behaviour is already right.
- **claim:** a fix that would change what a doc or test claims is true, what the code does, or what
  the PR's measured saving is.

## Guardrails
- ⛔ **Read-only, and the PR stays a draft.** No merge, no `gh pr ready`, no commits, no pushes, no
  CI reruns. Report; don't fix.
- Mutants and builds go in a scratch worktree, with their own `CARGO_TARGET_DIR` under your
  scratchpad. Delete both when you finish, and say how much space that freed.
- **Never poll CI,** and never write your own wait loop.
- **Shell pitfalls:**
  - A piped command reports the pipe's exit code, so redirect and read `$?`.
  - zsh doesn't word-split unquoted parameters, so pass flags explicitly, or loop with
    `while IFS= read -r`.
  - zsh reads `$B:r…` as a modifier, so brace it (`${B}:`).
  - macOS has no `timeout(1)`.
  - Count with `/usr/bin/grep`, `/usr/bin/git` or Python.
- Redirect `cargo test` stdout to a file. An interactive terminal fails
  `display_sink_refuses_non_tty`.
- Write scratch files only in your session's scratchpad, never `/tmp`.
- **Budget ~30 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content:
  `/usr/bin/grep -l "SPEC-162 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any `subagents/` transcripts.
- Price at Opus: $5 per MTok input and $25 output; cache_creation is ×1.25 the input rate, and
  cache_read is ×0.10.
- You're read-only, so report the numbers; ship writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is exactly one of:
- `✅ APPROVED — merge PR #271 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #271 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- checks 1–10, one or two lines each, with the evidence;
- the punch items, numbered, each with its class, evidence and expected fix;
- out-of-spec findings, each with where it belongs (STAGE-063 or `guidance/ideas.yaml`), and
  whether it was already there before this PR;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
