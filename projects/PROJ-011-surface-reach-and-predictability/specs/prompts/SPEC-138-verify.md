# SPEC-138 — VERIFY prompt: `build` refuses the output flags its manifest owns

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not design or
build SPEC-138.

```
just wt verify/spec-138 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-138-build-refuses-output-flags:refs/remotes/origin/feat/spec-138-build-refuses-output-flags
git reset --hard origin/feat/spec-138-build-refuses-output-flags
```

Draft PR #221's head was `5118362` when this prompt was written, with CI green on every
non-skipped leg. Record the SHA you verify. The maintainer pasted this prompt into a fresh session
and will paste your final message back to the orchestrator, so it must stand on its own. This is
the second of the two specs that open **0.8.1**. The first, SPEC-137, is merged (`71b4f38`), and
this branch contains it.

## What the build claims
- The spec is
  `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-138-build-refuses-the-output-flags-its-manifest-owns.md`.
  Read it all:
  - `## Frame` with rulings 1–6;
  - `## Design` with Calls 1–8, including `compare-builds.sh` in Call 6;
  - the 17 ACs;
  - `## Failing Tests` (FT-1 to FT-15) and the mutation table;
  - the build's `## Build Completion`.
- 17 of 17 ACs met. Commit 1 is `277391d`; its only `src/` hunk is inside `src/cli/mod.rs`'s
  `mod tests`.
- Baseline on `main`: 12 RED, 2 GREEN (FT-13, FT-14), and FT-15 does not compile. Each is RED for
  the spec's reason.
- Every mutation went red where the table says, with one difference. The build applied "guard
  after the manifest loads" by moving the guard into `run_build`. That is off the watch path, so
  it turned FT-11 and FT-12 red as well. The spec's row is a different mutation.
- Byte identity: an empty `git diff --stat` for `src/build/` and `src/cli/build.rs`, and
  `compare-builds.sh` printed `IDENTICAL: lockfile and 3 outputs`.
- The matrix was green: default 1115, `--no-default-features` 1053, `webp-lossy` 1120.
- `README.md` is unchanged; the design found no flag line there for these four.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits and no pushes. Do not fix what you find; report it. Do not
edit the spec or its timeline.

## Check, in order
1. **Scope and commit order.**
   - Run `git diff --stat origin/main...HEAD` and compare it with the spec's file list and Build
     Completion.
   - `git show 277391d -- src/` must touch only the `#[cfg(test)]` module.
   - Every commit is signed off.
   - No `Cargo.toml`, `Cargo.lock`, version or `CACHE_SCHEMA_VERSION` change.
   - Re-run the structural check yourself:
     `git diff --stat $(git merge-base origin/main HEAD)...HEAD -- src/build/ src/cli/build.rs`
     must be empty.
2. **The baseline.** In a scratch worktree at `277391d` (tests added, code unmodified), run
   `cargo test --test build_refuses_output_flags --no-fail-fast`. Confirm 12 RED and 2 GREEN,
   each **for the spec's reason**: read each failure message, not only the count. Confirm FT-11
   fails **at its 20 s deadline** with the "still running" message. That proves the waiter can
   block, so its pass at the head is not a harness that never waits.
3. **The guard, read whole.** Read `dispatch` in `src/cli/mod.rs` from the top to the verb
   `match`, and `refused_build_flags`.
   - The guard sits after the existing non-`build` `--watch` guard and before the `match`.
   - It applies only to `Commands::Build`, and is reached by both the plain path and the watch
     path.
   - Detection is `is_some()`. None of the four fields has a clap `default_value`, and clap's
     `env` feature is off (`Cargo.toml`).
   - The messages match Call 3 character for character, and `CliError::Usage` maps to exit 2.
4. **The lean leg: what FT-11 and FT-12 prove without `watch`.** `--no-default-features` drops the
   `watch` feature, and the tests carry no `cfg` gate, so they run in the lean leg too.
   - On a lean build of `main`, drive `build --watch --format png` by hand. Report the exit code
     and stderr. Is `--watch` refused there for another reason?
   - Then apply the "drop `format`" mutation to a lean build, rebuilt clean, and run FT-11.
   - **If FT-11 stays green in the lean leg with the guard gone, it is hollow there.** Report
     that as a finding and judge whether it matters: the default legs still cover it.
5. **CI exercised the rows.** Green is not proof that a row ran. Read the matrix legs of the CI
   run at the head you verify, once each, with `gh run view <run> --log --job <job>`. Do not poll.
   - On ubuntu, macos and windows, and on the lean leg, all 14 integration tests and FT-15 appear
     as `ok`, with none `ignored`.
   - **windows-latest:** FT-11 and FT-12 finish well inside 20 s. Killing a watching child on
     Windows is the riskiest path in this suite. Quote the line and its timing if the log shows
     one.
6. **Drive the release binaries, without the test harness.** Build `main`'s merge-base and the PR
   head as release binaries, each in a fresh target dir. In a scratch project (the spec's
   three-target fixture works), snapshot with `find . -type f | sort | xargs shasum -a 256`
   before and after every run.
   - **Every AC-1 to AC-12 row on both binaries.** `main` gives the spec's baseline result, and the
     PR exits 2 with exactly Call 3's lines and an unchanged snapshot. A row that gives the same
     answer on both binaries, but is listed as RED on `main`, is hollow.
   - **Rows the tests do not cover.** Report each one:
     - an explicit manifest path: `build other.toml --format png` (positional `FILE`);
     - a flag after `--`: `build -- --format`;
     - `--format` with `-o` attached in a cluster: `-Qox.png`;
     - `build --help` exits 0 and still lists the flags. Say whether that misleads; it is not
       in scope;
     - one more verb takes all four flags unchanged: `convert` or `optimize` with
       `-o x.png --format png`.
   - **AC-14 on the PR binary:** `apply --recipe` honours all four flags. Read every output
     format by magic bytes (`file` or `xxd | head -1`), never by extension.
7. **Byte identity, re-run yourself.** Save Call 6's `compare-builds.sh` verbatim into your
   scratch directory.
   - Run it against the two release binaries from item 6. It must print
     `IDENTICAL: lockfile and 3 outputs`.
   - Then apply the `hasher.update(b"x");` mutation from Call 6 to a third build, rebuilt clean.
     It must print `LOCKFILE DIFFERS`, with one `key =` line.
   - Restore, and confirm the tree is clean.
8. **Re-apply the mutation table yourself, one row at a time.** For each one:
   - check `git diff` is non-empty;
   - rebuild **clean** (fresh `CARGO_TARGET_DIR` or `cargo clean -p crustyimg`; reverting
     source does not rebuild the binary);
   - run `cargo test --test build_refuses_output_flags --no-fail-fast` and FT-15;
   - record exactly which go red, then restore and show them green.

   Two notes:
   - **"Move the guard after the manifest is loaded"**: apply it as the spec words it. The guard
     moves to a point that **both** the plain and the watch paths reach after the manifest loads.
     Do not use `run_build` only. The spec predicts FT-8 (exit 3) and FT-9 (the parse message).
     Report what goes red, and whether FT-11 and FT-12 stay green under this form.
   - The table tests loop over several spellings, so a red one names only its first failing row.
     For "drop `output`" and "drop `format`", also drive the mutated binary over every spelling
     in FT-2 and FT-5, and confirm each one is accepted (exit 0, writes).
9. **Mutations the spec did not list.** Apply each, rebuild clean, and name the test that catches
   it, or report a gap:
   - **M-a:** the guard joins the lines with `\n` but without `error: `, so only the first line
     carries the prefix;
   - **M-b:** the guard returns exit 1 instead of 2, by using a different `CliError` variant;
   - **M-c:** the fixed order is replaced by command-line order (iterate the raw args);
   - **M-d:** `snapshot()` skips dotfiles and dot-directories. Does any "nothing written" row
     still prove that `.crustyimg/` is untouched?
10. **Code quality.** Read the whole diff:
    - no `unwrap` or `expect` in `src/**` outside tests;
    - no new `CliError` variant, and `code()` is unchanged;
    - comments and messages read behaviour-first, with no spec or DEC ids;
    - the test helpers kill the child on panic, through a `ChildGuard` or equivalent.
11. **Docs and record**, against the binary:
    - `docs/cli-reference.md`: the four global-option rows end "Refused by `build` (exit `2`).",
      and the `build [FILE]` paragraph matches the spec's Notes. Drive each claim;
    - `docs/api-contract.md:626` carries the clause;
    - `CHANGELOG.md` has Call 7's entry verbatim under `## [Unreleased]` → `### Breaking`.
      SPEC-137's `### Fixed` entry is still there, and the released 0.8.0 lines are unchanged;
    - DEC-057 (both bullets, including the stale "output format follows the source" one) and
      DEC-105 rule 2 match Call 8 word for word. `./scripts/decisions-audit.sh` reports no
      errors.
12. **Full suite.** Run the default leg **and** the `--no-default-features` leg once each,
    sequentially, each in a fresh target dir. Use `--no-fail-fast`, redirect stdout to a file, and
    read `$?`. Then run `gh pr checks 221` once at the head you verified.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- zsh does not word-split unquoted parameters; loop with `while IFS= read -r`. In zsh, `$B:t`
  is a modifier, so write `"${B}:path"` for a git rev and path. macOS has no `timeout(1)`.
- A target dir shared across worktrees serves a stale library. Use a fresh one per checkout, and
  never share one between differently-featured builds running at once.
- A watching child must never be left running. Kill any you start, and check `pgrep crustyimg`
  before you finish.
- Delete each scratch target dir and scratch worktree when you are done with it. Keep scratch
  files in a scratch directory, never in `~`.
- **Budget ~70 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content:
  `grep -l "VERIFY prompt: \`build\` refuses" ~/.claude/projects/<slug>/*.jsonl`. The transcript
  may store the em-dash escaped, so search for the text after it.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`), never at a flat
  rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and
  cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #221 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #221 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–12, one or two lines each with evidence;
- the lean-leg finding from item 4;
- the uncovered rows from item 6, each with what the PR binary does;
- the mutation table as you ran it: mutation, tests that went RED, and whether that matches the
  spec;
- M-a to M-d, each caught (by which test) or a gap;
- optional cleanups, listed separately from punch items;
- your cost.
