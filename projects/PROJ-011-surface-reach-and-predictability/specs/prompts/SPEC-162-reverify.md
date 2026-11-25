# SPEC-162 — RE-VERIFY prompt: the gated cache and the plain probe folder

Cycle: **verify (re-verify)**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not
frame, design, build, verify or punch-list SPEC-162. Keep it short: check the punch list's two code
changes and its spec edits, and nothing the first verify already settled.

```
just wt verify/spec-162-re -- --model opus
```

Put the worktree on the PR's code:
```
git fetch origin chore/spec-162-ci-runs-each-test-once:refs/remotes/origin/chore/spec-162-ci-runs-each-test-once
git checkout --detach origin/chore/spec-162-ci-runs-each-test-once
```
Draft PR #271's head was `793dc0c` when this was written. The code changed last at `9897c9a`, and its
run `45556625777` passed. Record the SHA you verify.

The maintainer pastes your final message back to the orchestrator, so it must stand on its own.
**Write it as plain lists, not box-drawn tables.**

⛔ **Don't merge PR #271 until this re-verify has reported.**

## Settled (do not re-open)
- The cache stays, and AC-7 measures the Build step, not Build + Test (orchestrator's ruling).
- The per-checkout hash is gone, and a target folder shared between checkouts is unsupported.
- The skip, the removed `avif` job and the three ideas filed in `guidance/ideas.yaml` were all
  checked by the first verify.

## What the punch list claims
- `9897c9a`: the `check` job's rust-cache step has `if: needs.changes.outputs.code == 'true'`, with a
  rewritten comment; the probe folder is `<target>/tmp/avif-probe`; the doc comment drops the hash
  bullet and the memory-note link.
- Locally: AC-2 second run 2 s with nothing compiled; AC-3 counts unchanged; M1 turned both tests red,
  M2 only `both_encode_paths_set_the_thread_count`; AC-6 two concurrent runs passed.
- CI run `45556625777`: every leg green, 66.9 runner-minutes; each `check` leg restored with a full
  match; Build step ubuntu 27 s, macOS 21 s, windows 50 s; `avif_tile_pin` took 120–216 s, a cold
  probe build because the folder was renamed.
- **The punch list raised one point itself.** On a full key match rust-cache saves nothing, so this
  PR's cache never gains the new `avif-probe` folder, and each later run on the PR pays the cold probe
  build again. The orchestrator's reading: `main` has no `check` entry yet, so `main`'s first run after
  the merge saves one that includes `avif-probe`, and PRs then restore `main`'s. Check that reading.

## Check, in order
1. **Scope.** `git diff e54e5b5..HEAD --stat` touches only `ci.yml`, `tests/avif_tile_pin.rs` and the
   spec. Both new commits are signed off. Within `ci.yml`, only the `check` job's cache step and its
   comment changed.
2. **The `if:`.** It is byte-identical to the Build step's. Say whether a skipped action's post step
   is skipped too (from the runner's behaviour or the action's `action.yml`), so a docs-only run saves
   nothing either.
3. **The comment and the doc comment are true.** Hold each sentence against what verify measured:
   which caches a PR reads, the cleanup keeping `target/tmp` without promising to, the key's OS and
   architecture, and the shared-target sentence. Name any sentence that still overstates.
4. **The plain folder, driven yourself.** One fresh `CARGO_TARGET_DIR` under your scratchpad:
   - AC-2: run `cargo test --test avif_tile_pin` twice and quote both times;
   - M1 at `src/sink/mod.rs:782` on the warm folder: the probe rebuilds and goes red; restore, green.
     Replace the argument, don't delete the line, so the mutant compiles.
5. **The cache after the merge.**
   - From `gh api repos/jysf/crustyimg/actions/caches`, confirm there is no `check` entry on
     `refs/heads/main`.
   - Read rust-cache v2's save logic (its source or README): does it skip the save on an exact key
     hit, and is the key built from the lockfile, toolchain and job, not from the test file?
   - Then say whether `main`'s first run after the merge will save an entry that includes
     `target/tmp/avif-probe`, and so whether the warm ~20 s probe comes back for PRs from then on.
     If not, that's a **claim** against the saving.
   - Say whether this PR's own cold probe runs matter for anything but this PR's CI time.
6. **The spec edits.** Read AC-5, AC-7, the M3 and M4 rows, `## Outputs` and `## Build Completion`
   (including `### Punch list`). Each matches the evidence above and verify's report. Every AC marked
   met has evidence next to it. The DEC-081 range reads 187–191.
7. **The latest run.** Read the run(s) at `793dc0c` once: green, and whether its heavy jobs ran
   (a docs-only commit on a code PR still runs them).
8. **Local matrix:** `fmt --check` and `clippy --all-targets -D warnings` on default only. The punch
   list ran the full three-leg matrix; skip it unless check 4 or 1 turns something up.

## Classify every finding (AGENTS §15)
- **wording:** a doc, comment or spec line that misstates what the code already does.
- **assert:** a missing or loose test row, where the behaviour is already right.
- **claim:** a fix that would change what a doc or test claims is true, what the code does, or what
  the PR's measured saving is.

## Guardrails
- ⛔ **Read-only, and the PR stays a draft.** No merge, no `gh pr ready`, no commits, no pushes, no
  CI reruns. Report; don't fix.
- Builds and mutants go in a scratch worktree with their own `CARGO_TARGET_DIR` under your
  scratchpad. Delete both when you finish, and say how much space that freed (measure before you
  delete).
- **Never poll CI,** and never write your own wait loop.
- A piped command reports the pipe's exit code, so redirect and read `$?`. zsh doesn't word-split
  unquoted parameters; brace zsh variables before a colon (`${B}:`). macOS has no `timeout(1)`.
  Count with `/usr/bin/grep`, `/usr/bin/git` or Python.
- Redirect `cargo test` stdout to a file: an interactive terminal fails
  `display_sink_refuses_non_tty`.
- Write scratch files only in your session's scratchpad, never `/tmp`.
- **Budget ~20 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`. Find it by content:
  `/usr/bin/grep -l "SPEC-162 — RE-VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any `subagents/` transcripts.
- Price at Opus: $5 per MTok input and $25 output; cache_creation ×1.25 the input rate, cache_read
  ×0.10. Report the numbers; ship writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is exactly one of:
- `✅ APPROVED — merge PR #271 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #271 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- checks 1–8, one or two lines each, with the evidence;
- the punch items, if any, numbered, each with its class, evidence and expected fix;
- out-of-spec findings, if any, with where each belongs;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
