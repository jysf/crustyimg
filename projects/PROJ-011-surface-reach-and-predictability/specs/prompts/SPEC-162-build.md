# SPEC-162 — BUILD prompt: CI runs each test once

Cycle: **build**. Start a fresh CLI session in its own worktree, with **Sonnet**:

```
just wt chore/spec-162-ci-runs-each-test-once -- --model sonnet
```

The maintainer will paste this prompt into that session, then paste your final message back to the
orchestrator. So your final message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

There was no frame or design session. The spec's `## Context` is the orchestrator's measurement of
run `45501714010`, and its `## Outputs` and ACs are the design. Read them as settled. If you find a
fact that contradicts them, stop and report it.

## What you are building
Three changes:
1. **Remove the `avif` CI job.** It runs exactly the default feature set again.
2. **Move `avif_tile_pin`'s probe build.** It now builds into a per-checkout folder under
   `CARGO_TARGET_TMPDIR`, so it builds incrementally and never leaves anything in the temp folder.
   Skip the probe test on the `webp-lossy` and `heic` legs.
3. **Try `Swatinem/rust-cache@v2` on the three `check` legs.** Keep it only if a measured run shows
   it pays, using AC-7's rule.

## Step 0: get onto current `main`
1. Run `git fetch origin && git rebase origin/main`.
2. Check that:
   - the spec (`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-162-ci-runs-each-test-once.md`)
     is at `cycle: build`;
   - `probe_binary()` in `tests/avif_tile_pin.rs` still uses
     `crustyimg-spec124-probe-target-{pid}`;
   - both pins still exist (`/usr/bin/grep -n with_num_threads src -r`);
   - the `avif` job is still in `.github/workflows/ci.yml`.

   If any check fails, stop and report.
3. Re-take the overlap map with `gh pr list --state open --json number,title,files`. The spec names
   #237 (`heic` job) and #240 (`ddphotos-parity`). Record any change.

## Read first
- The spec in full, especially:
  - the ACs;
  - the mutation table;
  - "Out of scope";
  - "Notes for the Implementer".
- `tests/avif_tile_pin.rs` in full, and DEC-094.
- `.github/workflows/ci.yml`: the `check`, `avif`, `webp-lossy` and `heic` jobs, and the comments at
  the top of the file.
- AGENTS §13 (PRs) and §15 (a bound or a "can't happen" is a claim until its worst input is driven).

## Order, with a pushed checkpoint after each stage
1. **Baseline.** Before you change anything:
   - Read run `45501714010`'s jobs from the API.
   - Write down each leg's wall time, each Test step and the total runner-minutes. You'll compare
     against these at AC-9.
   - Run `cargo test --test avif_tile_pin` once on `main`, timed, with stdout redirected to a file.
   - Count `${TMPDIR}crustyimg-spec124-probe-target-*` before and after, and quote both counts.
     After this baseline, delete only the folder you just created.
2. **The probe folder** (`tests/avif_tile_pin.rs`). Make the change in `## Outputs`.
   - Then drive, in this order:
     - AC-2;
     - AC-3;
     - AC-4: M1 and M2;
     - AC-5: M3, with a second worktree from `git worktree add --detach` in your scratchpad;
     - M4;
     - AC-6.
   - Use one fresh `CARGO_TARGET_DIR` per drive, under your scratchpad, unless the AC asks for a
     shared one.
   - Commit and push.
3. **The workflow.**
   - Remove the `avif` job.
   - Add the `--skip` to the `webp-lossy` and `heic` Test steps.
   - Add `Swatinem/rust-cache@v2` to `check`, after the toolchain step, with `key: check`.
   - Fix every comment these changes make wrong, including the `changes` job's list of heavy jobs,
     which names `avif`.
   - Open the **draft** PR with this push.
4. **The cache measurement (AC-7).**
   - The PR's first run saves the cache. Push one more commit to trigger the second run. A commit
     that fills in part of `## Build Completion` will do.
   - Read both runs from the API, and apply AC-7's rule.
   - If the cache doesn't pay, remove it in its own commit, saying why, and quote the numbers.
5. **The final run.** Collect AC-8 and AC-9 from the run at your final head.

## The things most likely to go wrong
- **A stale probe binary.** A shared target folder has served a stale library across worktrees
  before. AC-4 and AC-5 are the guard. A green probe test after a mutant is a finding: stop and
  report it, don't explain it away.
- **A skip that filters nothing.** `--skip` matches substrings. Show that the pattern matches exactly
  one test in the whole suite (AC-8), and quote the feature legs' `1 filtered out` line.
- **A cache that looks fast only because it's cold or warm.** Compare like with like:
  - the second run against the baseline leg, Build + Test only;
  - not the first run, which also pays to save the cache.
- **The required checks.** They are the three `check` legs by name. Don't rename the `check` job or
  change its `name:`.

## Guardrails
- Stay in these files:
  - `.github/workflows/ci.yml`;
  - `tests/avif_tile_pin.rs`;
  - the SPEC-162 spec.

  `git diff --stat origin/main...HEAD` must show nothing else. Leave `src/` alone; mutants are
  applied and restored, never committed.
- ⛔ No CHANGELOG edit, version bump, tag or release. The PR stays a **draft**: no `gh pr ready` and
  no merge.
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- Never force-push over someone else's commit.
- **Never poll CI.** After each push, run `gh pr checks <PR> --watch` once in the background, and
  don't write your own wait loop. If it returns before the checks register, run it once more.
- **Shell pitfalls:**
  - A piped command reports the pipe's exit code, so redirect and read `$?`.
  - zsh doesn't word-split unquoted parameters, so loop with `while IFS= read -r`.
  - zsh reads `$O:s…` as a modifier, so brace it (`${O}:`).
  - Count with `/usr/bin/grep` or Python, because `rtk` can truncate `grep`.
- Redirect `cargo test` stdout to a file: an interactive terminal fails
  `display_sink_refuses_non_tty`.
- Write scratch files only in your session's scratchpad, never `/tmp`. Remove your second worktree
  and every scratch `CARGO_TARGET_DIR` before you finish, and say how much space that freed.
- **Budget ~35 exchanges.**

## Check it
Run the local matrix, each leg in a fresh `CARGO_TARGET_DIR`:
- `fmt --check`;
- `clippy --all-targets -D warnings`;
- the full test.

Run all three on the default build, `--no-default-features` and `--features webp-lossy`. Then check
that every CI leg is green at your final head.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`;
   - every AC, one line each, with its quoted evidence;
   - every mutant against the table, with each mismatch called out;
   - the baseline and final-run numbers side by side;
   - the cache decision, with its numbers;
   - the sweep for docs that name the `avif` job (hits listed, none changed);
   - the overlap check after your last rebase;
   - the build-phase reflection.
2. **Measure your own cost** at your **last** turn, after the last push.
   - Find your transcript by content:
     `/usr/bin/grep -l "SPEC-162 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`. `<slug>` is
     your working directory's absolute path, with every `/`, `.` and `_` replaced by `-`.
   - Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
     `output_tokens`. Include any `subagents/` transcripts.
   - Price at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation is
     ×1.25 the input rate, and cache_read ×0.10.
   - Append a `build` entry to `cost.sessions`.
3. **Advance the cycle.** Run `just advance-cycle SPEC-162 verify`, then commit and push. The PR
   stays a draft.
   - PR title: `ci: run each test once — drop the duplicate avif job, build the AVIF probe incrementally (SPEC-162)`.
   - PR body, per AGENTS §13:
     - a summary with the measured before and after;
     - the spec metadata;
     - the decisions referenced: DEC-009, DEC-020, DEC-094;
     - new decisions: none;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
The first line is `SPEC-162 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`. Then, as
plain lists:
- the baseline, and the final run's runner-minutes, slowest leg and each Test step;
- AC completion (as a count, plus any row not met);
- each mutant's result against the table;
- the cache decision, with its numbers and the cache sizes;
- the probe test's lines from a `check` leg and from a feature leg;
- the matrix result and the CI legs;
- the docs sweep hits, and the overlap check;
- anything that didn't go as written, and any out-of-spec finding;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on
  its own line.
