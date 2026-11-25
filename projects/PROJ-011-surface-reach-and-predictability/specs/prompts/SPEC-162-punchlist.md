# SPEC-162 — PUNCH LIST prompt: a gated cache step and honest wording

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. You did not frame,
design, build or verify SPEC-162. Verify found two **claims** and one **wording** item, and the
orchestrator has ruled on both claims below. A short re-verify follows your report.

```
just wt fix/spec-162-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin chore/spec-162-ci-runs-each-test-once:refs/remotes/origin/chore/spec-162-ci-runs-each-test-once
git checkout --detach origin/chore/spec-162-ci-runs-each-test-once
```
Push with `git push origin HEAD:chore/spec-162-ci-runs-each-test-once`. Draft PR #271's head was
`e54e5b5` when this was written. If it has moved for any reason other than a maintainer merge from
`main`, stop and report.

The maintainer pastes this prompt into a fresh session and will paste your final message back to the
orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

⛔ **Don't merge PR #271, and don't mark it ready.** A re-verify follows.

## What verify found (the evidence, re-checked by the orchestrator against the API)
- **AC-7 on macOS holds on one of two warm samples.** Build + Test against the 560 s baseline:
  434 s (−22.5%, run `45539781614`) and 473 s (−15.5%, run `45541262385` at `e54e5b5`). A second
  baseline, `main` run `45535869201`, took 848 s on the same leg. The Build step alone fell on every
  warm leg in both runs: macOS 136 → 39–40 s, ubuntu 149 → 19–25 s, windows 254 → 44–55 s. The Test
  step on macOS is noise (424 → 394, then 434 s).
- **The cache step runs on docs-only pushes.** It has no `if:`, unlike every step after it. A
  docs-only run restores about 1.75 GB across the three legs (about 53 s) for no gate.
- **The probe's per-checkout hash protects nothing.** With one `CARGO_TARGET_DIR` shared by two
  worktrees, cargo reuses the other checkout's outer test binary, with its manifest path baked in. So
  the second checkout's run can come back a 1 s false green whether or not the probe folder is
  hashed. The doc comment's last bullet is false.
- **The `ci.yml` comment is loose.** A PR reads caches saved on its own `refs/pull/N/merge` and on
  `main`, never another PR's. And `target/tmp/avif-probe*` survives rust-cache's cleanup only because
  the cleanup treats any folder without `build`, `.fingerprint` or `deps` as a nested target folder.
  The action documents nothing about that. The cleanup still removes the crustyimg crate itself, so
  the nested build recompiles crustyimg alone.
- **The spec text** marks every AC met while AC-5 failed as written, says the classifier skips a
  docs-only commit (on a PR it diffs `base...HEAD`, so it doesn't; `4270619` was cancelled by the
  concurrency group), and cites DEC-081 as lines 188–190 (the mentions run 187–191).

## The orchestrator's rulings (settled, do not re-decide)
1. **The cache stays.** AC-7's bar was badly chosen: the cache can only speed up the Build step,
   and the Test step on macOS swings by more than the 20% bar between runs. The Build step fell
   70–90% on every warm leg, in both warm runs. AC-7 is restated below to measure the Build step.
2. **The hash goes.** A mechanism that prevents nothing should not be in the code. The probe folder
   becomes plain `<target>/tmp/avif-probe`. A target folder shared between checkouts is unsupported
   for this test, as for every other test.

## The fixes, in order
1. **Gate the cache step** (claim). In `.github/workflows/ci.yml`'s `check` job, add
   `if: needs.changes.outputs.code == 'true'` to the `Swatinem/rust-cache@v2` step, spelled exactly
   as on the Build step below it. A skipped action's post step is skipped too, so nothing else is
   needed. Touch no other job's cache step.
2. **Rewrite the cache step's comment** (wording), in plain words with no spec or DEC references.
   It must say:
   - it restores the dependency build from this pull request's last run, or from `main`'s;
   - the AVIF probe's folder under `target/tmp` happens to survive the action's cleanup, which does
     not promise to keep it, and the probe then rebuilds only crustyimg itself;
   - the key includes the runner's OS and architecture, so each leg keeps its own entry;
   - it is skipped when no code changed, like the steps after it.
3. **Drop the hash** (claim, per ruling 2). In `tests/avif_tile_pin.rs`'s `probe_binary()`:
   - the folder becomes `PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("avif-probe")`;
   - remove the `DefaultHasher` import and use;
   - rewrite the doc comment's folder bullets: the same folder every run, so the nested build is
     incremental; under the target folder, so `cargo clean` removes it; and one new bullet saying
     that a `CARGO_TARGET_DIR` shared between checkouts is unsupported, because cargo then reuses the
     other checkout's test binary;
   - the doc comment also cites a memory note as `[[concurrent-differently-featured-…]]`. Replace
     that link with the plain reason: a differently featured build in the shared target folder would
     corrupt it. Leave the file's other comments alone.
4. **Re-run the probe's ACs on the new folder**, locally, with one fresh `CARGO_TARGET_DIR` under your
   scratchpad:
   - AC-2: run `cargo test --test avif_tile_pin` twice; quote both wall times and the second run's
     nested `Fresh`/`Compiling` lines. The second run compiles nothing in the probe folder.
   - AC-3: count `${TMPDIR}avif-probe*` and `${TMPDIR}crustyimg-spec124-probe-target-*` before and
     after; both unchanged.
   - AC-4: M1 (at `src/sink/mod.rs:782`) and M2 (at `src/quality/mod.rs:426`), each alone, on the
     warm folder. Record what goes red, then restore and confirm green.
   - AC-6: two concurrent runs, one checkout, one target folder; both pass.
   - Confirm `<target>/tmp/avif-probe` exists and no `avif-probe-<hex>` folder was created.
5. **Edit the spec** (wording), on the PR branch:
   - **AC-5** becomes: "**AC-5: a target folder shared between checkouts is unsupported.** Cargo
     reuses one checkout's test binary in the other, with the first checkout's manifest path built
     in, so a run there can be a false green or a false red whatever the probe does. Each checkout
     uses its own `CARGO_TARGET_DIR`, as for every other test." Mark it met by the doc comment.
   - **AC-7** becomes: the cache is kept if the **Build step** falls at least 20% on each `check`
     leg in two warm runs, and the repository's cache total stays under 6 GB. Add a line recording
     ruling 1 and why the Build + Test bar was dropped (the macOS Test step's spread).
   - **The M3 row:** record what the build saw (A's second run red, reusing B's binary), and that
     this is cargo's behaviour, not the probe's.
   - **The M4 row:** record verify's drive (B with the pin removed red, A touched green, B again a
     1 s false green), and that the hash was dropped per ruling 2.
   - **`## Outputs`:** the probe folder line names the plain `avif-probe` folder.
6. **Edit `## Build Completion`** (wording). Change nothing above it but the items in fix 5.
   - "All acceptance criteria met?" says AC-5 was restated at the punch list, and AC-7 met on the
     restated bar.
   - Replace the "a docs-only commit does not measure the cache" finding: on a PR the classifier
     diffs `base...HEAD`, so a docs-only commit on a code PR re-runs everything; `4270619` was
     cancelled by the concurrency group.
   - The DEC-081 line range becomes 187–191.
   - Add the third run `45541262385` at `e54e5b5` (68.5 runner-min, slowest leg heic macOS 12.1 min)
     and the second baseline `45535869201` (106.2 runner-min, macOS Build + Test 848 s).
   - Add a `### Punch list` subsection: each fix, its commit, the AC-2/3/4/6 numbers from fix 4, and
     the measurement from fix 7.
7. **Measure the gated cache on CI.**
   - Push the `ci.yml` and test changes as their own commit, then the spec edits as a second commit.
   - Start one background `gh pr checks 271 --watch` per push you need to read. Never poll, and never
     write your own wait loop.
   - On the last run that ran the heavy jobs: each `check` leg's cache step ran and restored
     (quote "Cache restored from key" or its equivalent, and whether it was a full match), and give
     each leg's Build step against both earlier warm runs. The probe folder's name changed, so its
     nested build is cold once; say how long `avif_tile_pin` took.
   - A docs-only push on a PR still runs everything (fix 6), so the `if:` can't be shown skipping
     on this PR. Show instead that its expression is byte-identical to the Build step's.
   - Every leg green, including the three feature legs' `1 filtered out` for `avif_tile_pin`.

## Guardrails
- Scope: `ci.yml`, `tests/avif_tile_pin.rs` and the SPEC-162 spec only. No changes to the skip, to
  other jobs, to DECs, or to `guidance/`. The orchestrator files the out-of-spec findings.
- Commit with `git commit -s` and the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
  DCO sign-off is mandatory. Run `cargo fmt --check` before each push.
- Run the local matrix before the first push: `fmt --check`, `clippy --all-targets -D warnings` and
  the full test, on default, `--no-default-features` and `--features webp-lossy`, one fresh
  `CARGO_TARGET_DIR` per leg under your scratchpad. Redirect `cargo test` stdout to a file: an
  interactive terminal fails `display_sink_refuses_non_tty`.
- **Shell pitfalls:** a piped command reports the pipe's exit code, so redirect and read `$?`. zsh
  doesn't word-split unquoted parameters. Brace zsh variables before a colon (`${B}:`). macOS has no
  `timeout(1)`. Count with `/usr/bin/grep`, `/usr/bin/git` or Python.
- Write scratch files only in your session's scratchpad, never `/tmp`. Delete your target folders
  when you finish, and say how much space that freed.
- **Budget ~30 exchanges.** Push after each fix that passes, so the work is never only local.

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`. Find it by content:
  `/usr/bin/grep -l "SPEC-162 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any `subagents/` transcripts.
- Price at Sonnet: $3 per MTok input and $15 output; cache_creation is ×1.25 the input rate, and
  cache_read ×0.10.
- Add a `punchlist` entry to the spec's `cost.sessions` before your last push.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-162 punch list: pushed — PR #271 at <SHA>`. Then:
- each fix with its commit SHA;
- AC-2, AC-3, AC-4 (M1, M2) and AC-6 on the plain folder, with numbers;
- fix 7's CI evidence: the run ID, each leg's cache restore and Build step, `avif_tile_pin`'s time,
  and the `if:` comparison;
- the local matrix results;
- anything you could not do, and why;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
