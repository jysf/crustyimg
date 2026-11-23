# SPEC-155 — BUILD prompt: a file write is all or nothing

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design gives
the change in code terms: a new `src/sink/publish.rs` (a `StageFs` seam, `RealFs`, a cleanup guard,
a per-process counter, and staged names `.<name>.crustyimg-<pid>-<n>.tmp`), the four sink arms,
`--save-recipe`'s guard, `write_lock` on the helper. It also gives every message verbatim, 22 ACs,
39 failing tests (12 unit, 23 end-to-end, 4 Windows), and a mutation table. **Every row except the
Windows ones was driven at design on a prototype.** The code is about 170 lines of helper and 110 of
call sites; most of the work is the tests, the mutations and the doc sentences. Be thorough across
every row: Sonnet's known weak spot is sweep thoroughness.

```
just wt feat/spec-155-atomic-writes -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that message
must stand on its own. Keep it plain text with short lists; a wide box-drawn table can garble when
copied out of the terminal.

## Before you start: SPEC-154 must have merged
STAGE-063 lands SPEC-154 (the CI path classifier) first, so this PR runs under it. At Step 0, check
that SPEC-154's PR is merged into `main` (`gh pr list --state merged --search "SPEC-154"`). If it
isn't, stop and report; don't start.

**SPEC-157 builds after this spec** and puts its sink change on your helper. Keep `publish_file` and
`publish_file_io`'s signatures as the spec gives them; if you have to change one, say so in your
report, because SPEC-157's design reads them.

## What you are building
- **`src/sink/publish.rs`** (new). Order inside `publish`:
  1. probe an existing destination by opening it for writing without truncating (this keeps today's
     refusal of a read-only file under `-y`);
  2. `create_new` the staged file in the destination's own directory (up to 16 retries on
     `AlreadyExists`, with a per-process atomic counter);
  3. the write loop, then flush, then close;
  4. copy the old file's permission bits;
  5. rename over the destination;
  6. disarm the cleanup guard.

  The cleanup guard removes the staged file on every early return. `FailingFs` writes at most
  1,024 B per call, so "the second write fails" is a real state. The pid is 0 on wasm32
  (`std::process::id()` panics there), and `publish_file_io` is gated to non-wasm.
- **`src/sink/mod.rs`:** the four arms (`:499`, `:533`, `:645`, `:665`) call `publish_file`. New
  `WriteFailed` and `DirNotWritable` variants. `guard_overwrite` becomes `pub(crate)`. The module docs
  are updated.
- **`src/cli/ops.rs`:** `run_edit` guards the recipe path **before the run** (otherwise a refused
  recipe still writes the image). `--save-recipe`'s write uses the helper.
- **`src/cli/build.rs`:** `write_lock` calls `publish_file_io`. Touch nothing else in this file:
  SPEC-157 and SPEC-158 own its other functions.
- **`src/cli/mod.rs:511`:** `--save-recipe` help gains "An existing FILE needs --yes."
- **A batch where every write fails** exits 6 and keeps every old file (`main` exits 0 with every
  output cut to 512 B).
- **The new DEC,** created from the spec's final text as
  `decisions/DEC-<next>-a-file-write-is-all-or-nothing.md`. Take the next free id **when you create
  it**: `ls decisions/ | sort` on current `main`, and again after each rebase. Other specs' builds may
  take ids too. Replace every `DEC-<next>` in your diff with the number you took.
- **Docs, `--help` and CHANGELOG,** verbatim from the spec.

The spec's `## Outputs` is the complete file list. Anything outside it is a deviation; say why.

### One correction from the orchestrator
The CHANGELOG "Fixed" line says a failed write "exits 5". That's true for one output, but AC 16 makes
a batch with failed writes exit 6. Write "exits 5 (6 for a batch, after the other inputs are
written)", and check the same phrase anywhere a doc sentence names the failed-write exit code.
Drive the sentence on both paths and quote both. If the spec's docs already name exit 6 for the
batch path, leave them as written.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- SPEC-154 has merged (above);
- the spec is at `cycle: build`;
- design commit `c16a202` is in your history;
- the four sink arms are still at about `src/sink/mod.rs:499`, `:533`, `:645` and `:665`. Find them by
  content if they've moved.

If any check fails, stop and report.

⚠ **Other open work touches nearby lines.** The spec's "Overlap with open PRs" lists it from the PRs'
file lists:
- #240: `src/cli/ops.rs`, in the `use super::common` block next to your `use crate::sink` line;
  expect a trivial adjacent conflict.
- #237: `CHANGELOG.md`; keep both entries.
- #226: `src/cli/mod.rs`.

After every rebase, run `gh pr list --state open --json number,title,files` and compare; name any new
overlap in Build Completion. **If a SPEC-156 or SPEC-157 build branch or PR has appeared, list its
files too.** If a rebase conflicts in a file outside the spec's list, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-155-a-write-that-fails-must-not-report-success-or-destroy-the-old-file.md`.
   Read it in full:
   - `## Frame` with the maintainer rulings;
   - `## Design`, especially "Design corrections to the frame" and "Things the frame asked design to
     check";
   - every AC, the Failing Tests, the baseline and the mutation table;
   - `## Implementation Context`: the code shape, messages, output bytes, the DEC, docs, CHANGELOG,
     verification, and the Windows rows;
   - `## Notes for the Implementer`.

   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in your
   report; don't quietly re-decide it.
2. **DECs:** DEC-035 (symlinked destinations), DEC-044 (out-dir auto-create) and DEC-059 (the
   lockfile). Read them yourself. Leave DEC-059's ship-time pointer alone: ship applies it.
3. The code the spec cites at `file:line`, before you change it: `src/sink/mod.rs:480-690`,
   `src/cli/ops.rs:1681-1764`, `write_lock` in `src/cli/build.rs`, `has_image_extension` in
   `src/source/mod.rs`.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 10 to 14). Also `/guidance/constraints.yaml`.

## Maintainer rulings (settled — do not re-open)
1. Read-only directory with `-y`: refused, exit 5, the directory named; no in-place fallback.
2. No `fsync`, file or directory.
3. `edit --save-recipe` needs `-y` to replace an existing recipe.
4. Full disk: the loud failure is accepted.
5. Hand-rolled on `std`, no new dependency. A replaced file keeps its permission bits but not its
   owner, ACLs, xattrs or hard links.
6. No message, help line or comment names a spec, stage or DEC id.

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push after
each numbered stage. Stages 2–6 are the spec's five checkpoints.
1. **The RED baseline.** Write `tests/atomic_write.rs` and run
   `cargo test --test atomic_write --no-fail-fast` on `main`'s code, stdout redirected to a file and
   `$?` read. Expected red, per the spec's baseline: E1–E9, E11, E15, E16, E19, E22, E23 (15 rows),
   each **for the reason the spec gives** (exit code, old bytes, or message). A row red for another
   reason (a precondition panic, a missing fixture) isn't a valid baseline; fix it first. Quote each
   red row's failure line. Open the PR **as a draft** with this commit.
2. **The helper and its unit tests** (U1–U12 green).
3. **The four sink arms.**
4. **`--save-recipe` and `write_lock`.**
5. **The end-to-end file green** (E1–E23), and the Windows rows (W1–W4) written.
   - Assertions in each row go in the spec's order: exit code, old bytes, staged file, then message
     (trap 13).
   - Expected messages and bytes come from the spec's literals, never from `Display` or the run.
   - Tests that need a non-root user **assert** it and fail with "running as root?". They never skip.
6. **Docs, CHANGELOG and the DEC.**
   - Write each doc row at the `file:line` the spec gives, found by content.
   - **Drive every behaviour sentence on every exit path it names**, using the spec's per-sentence
     path list. Quote each drive in Build Completion.
7. **Mutations, one at a time** (HANDOFF traps 12 and 14), in a scratch `git worktree` of your
   pushed head with its own `CARGO_TARGET_DIR`, never in the build worktree. The spec quotes each
   mutant's exact edit: **apply that edit, not your own version of it.** If the quoted line doesn't
   exist in your code, make the closest edit, quote what you applied, and say why. For each row:
   - apply it, and check `git diff` is non-empty;
   - rebuild **clean** (`cargo clean -p crustyimg`), because a reverted source doesn't rebuild the
     binary;
   - run the full suite with `--no-fail-fast`, stdout redirected and `$?` read, and read the red set
     **off that run**. Compare it with the table and call out every test the table doesn't list;
   - **M7b** is red on U10 only, by design (E14 can't catch a collision a microsecond wide). Drive it
     and say whether that still holds;
   - **Trap 14a/14b:** restore `main`'s files as the table says and list what goes red;
   - restore, rebuild, and show it green.

   Paste each mutation's red output, trimmed to the lines that name tests, into Build Completion.
   Where a cell is wrong, correct it in Build Completion and say so; don't edit the design's table.
8. **Clean matrix:**
   - default, `--no-default-features`, and `--features webp-lossy`, one after another, each in a
     fresh `CARGO_TARGET_DIR`, with clippy `-D warnings` and `fmt --check` on each;
   - then `just wasm-check` and `just wasm-test` once. `src/sink` is in the wasm build, and
     `just check` runs neither.

## The things most likely to go wrong
- **The write probe.** Dropping it lets `-y` replace a 0444 file that `main` refuses (M10). It opens
  without `truncate`.
- **A short write reported as full** (M11) turns 46 tests red. The write loop must handle partial
  writes.
- **`ulimit -f` and stderr.** FT-E22 captures stderr through a pipe (`Command::output()`). A redirect to
  a file inside the `sh -c` is capped by `ulimit -f 0` too, and turns the row into a panic (exit 101).
- **Staged files as inputs.** The `.tmp` extension keeps them out; the leading dot does not.
  `resolve_directory` and `resolve_glob` don't skip dotfiles.
- **Windows.** W1–W4 are reasoned, not driven. **The Windows CI leg is their only evidence.** Quote
  the four rows' results from its log. If W2's exit code or message differs from the spec's
  reasoning, stop and report rather than loosen the assertion. A sharing violation (`os error 32`)
  instead of `os error 5` still passes, because the row asserts only the prefix.
- **Retyped messages.** Copy every message from the spec byte for byte.
- **A green leg on one OS isn't the matrix.** Read every CI leg, including lean, msrv 1.90.0, Windows
  and wasm.

## Guardrails
- No version bump, no release, no tag. No change to `CACHE_SCHEMA_VERSION`, the lockfile format,
  `Cargo.toml` or `Cargo.lock`. Paste `git diff --stat origin/main...HEAD` for those paths.
- `ulimit -f` runs only inside the test's subshell or your own subshell, never in your session's shell.
- Don't edit DEC-035, DEC-044, DEC-059 or the spec's timeline. They live on `main`, and ship and the
  orchestrator keep them.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI and never write your own wait loop** (AGENTS §16 habit 6). Run
  `gh pr checks <PR> --watch` once, in the background, after your last push, and take one snapshot
  when it exits. If it exits with legs still pending, say so; don't call that green.
- **Rebase onto `main` before each push** and redo Step 0's overlap check after each rebase. Don't
  chase `main` beyond that.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout to
  a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`. Gate each commit
  on its test run; don't chain `git commit` after `cargo test` with `;`. zsh does not word-split
  unquoted parameters, so loop with `while IFS= read -r`, and it reads `$R:r…` as a modifier, so brace
  it (`${R}:`). macOS has no `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use
  `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Delete each scratch target dir and the
  mutation worktree when you're done. Write scratch files in your session's scratchpad, never `/tmp`
  or the system temp dir, and don't leave PR-body files behind.
- **Budget ~110 exchanges.** If you're past budget with the matrix green, stop and report what's
  left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase on
     what changed; give the count and say whether it includes the spec file;
   - the RED baseline, quoted;
   - every AC, one line each: met, or not and why;
   - each behaviour sentence's drive on each exit path, quoted;
   - every mutation row, with the edit you applied, the tests that went red and those that stayed
     green, against the spec's table, with every mismatch called out;
   - the DEC id you took;
   - the matrix legs, the wasm check and test, and the CI legs, with W1–W4 quoted from the Windows
     leg;
   - the overlap check after your last rebase;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost** at your **last** turn, after the last push.
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `/usr/bin/grep -l "SPEC-155 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`. Include any `subagents/`
     transcripts.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per MTok,
     Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-155 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks it
     ready. Never mark it ready yourself.
   - Title: `fix(sink): a file write is all or nothing (SPEC-155)`.
   - Body per AGENTS §13:
     - Summary: a failed write used to exit 0 over a truncated file under 8 KiB, and destroy the old
       file above it; now every file is staged beside the destination and renamed into place, and a
       failure exits 5 (6 for a batch) with the old file kept;
     - Spec metadata;
     - Decisions referenced: DEC-035, DEC-044, DEC-059;
     - New decisions: DEC-<the id you took>, a file write is all or nothing;
     - Constraints checked;
     - Breaking: `-y` into a directory that can't take new files is refused; `edit --save-recipe`
       needs `-y` to replace a recipe; `-y` now makes a new file (permissions kept; owner, ACLs, xattrs
       and hard links not);
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-155 build: draft PR #NNN at <head SHA> — CI <green | red: legs | pending>`.
Then, briefly, as plain lists:
- the RED baseline on `main`;
- AC completion (as a count, plus any row not met);
- each behaviour sentence's drive, one line per exit path, including the batch exit-6 path;
- each mutation's flips against the table, the edit you applied where it differs from the quoted one,
  and every cell that turned out wrong;
- the DEC id you took;
- the matrix result, `just wasm-check`/`just wasm-test`, and the CI legs, including lean, msrv,
  Windows (W1–W4 quoted) and wasm;
- the overlap check after your last rebase;
- files changed against the spec's `## Outputs`;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its
  own line.
