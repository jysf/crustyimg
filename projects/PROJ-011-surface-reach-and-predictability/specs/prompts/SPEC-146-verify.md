# SPEC-146 — VERIFY prompt: CI runs the wasm round-trip tests

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**. You did not design or build SPEC-146.
Read-only toward the PR under review.

```
just wt verify/spec-146 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-146-wasm-roundtrip-ci:refs/remotes/origin/feat/spec-146-wasm-roundtrip-ci
git reset --hard origin/feat/spec-146-wasm-roundtrip-ci
```

Draft PR #245's head was `5098e4f` when this prompt was written, with CI green (19 pass, the
release-pipeline jobs skipped). Record the SHA you verify. The maintainer pasted this prompt into a
fresh session and will paste your final message back to the orchestrator, so it must stand on its
own.

⚠ **Another orchestrator thread owns PROJ-017 and is running at the same time.** Its `crop` build
edits `src/operation/`, `src/recipe/`, `docs/recipes.md`, `tests/recipe_corpus_output.rs`,
`tests/fixtures/recipe_corpus/` and may add wasm tests. Do not touch those. Use your own
`CARGO_TARGET_DIR`; never share one with another session.

## What the build claims
- The spec is
  `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-146-ci-runs-the-wasm-round-trip-tests.md`.
  Read it all: `## Frame`, `## Design`, the 11 ACs, `## Failing Tests` (C1 to C12 and the 17
  self-test rows), `## Implementation Context`, the mutation table M1 to M17, and
  `## Build Completion`.
- **Commits:** `e295dc7` (script and classifier rows), `1f9728e` (job, docs, comments), `739f364`
  (a shape-check fix), `5098e4f` (Build Completion, advance to verify). All four claim DCO sign-off.
- **No mutation was pushed:** `git diff origin/main...HEAD -- src tests` is empty.
- **The build did not rebase** onto `main` (it says the sandbox refused the force-push). `main` has
  moved on by docs commits. Judge whether that matters.
- C1, C3 and C10 RED on `main`, GREEN at the head. C4 (stub runner `0.2.125`: exit 1; no runner on
  `PATH`: exit 1), C7 local (`#[cfg(any())]` on `tests/wasm_roundtrip.rs:30` → `1 passed`, exit 0,
  then the floor exits 1) and C12 as specified.
- **Live run `42080605578`:** the `wasm-roundtrip` job printed `47 passed` and `45 passed`, each
  followed by the floor's "as expected" line; cold 4m04s, warm 1m31s (run `42084476692`).
- Local mutations M1, M2, M3, M5, M8, M12 to M17 all went red for their reason.
- **One design call changed:** assertions 10 and 11 of the shape check matched a substring that the
  error line also contains, so M5 stayed green. The build changed them to match the invocation line
  (`^\s*just wasm-test(-slim)?\s*>`) in `739f364`.
- **Unlisted additions:** a `compare-runner` subcommand in `scripts/wasm-ci-check.sh` (so self-test
  rows 14 to 17 can drive the version compare with strings; `runner-version` calls the same
  function), and one comment-only edit to the script header made to force a warm run.

## ⛔ Read-only
No merge, no `gh pr ready`, no commits or pushes **to PR #245's branch**. No tag, no publish, no
`gh workflow run`, no branch-protection change, no change to npm or crates.io settings. Do not fix
what you find; report it. Do not edit the spec or its timeline.

**One allowed exception: throwaway live rows.** The spec's live checks (C6, C7 live, C8 live, C9)
need a CI run on mutated code. Do not push them to PR #245. Cut a scratch branch off the head
(`verify-scratch/spec-146-<row>`), push **only that branch**, open a **draft** PR from it against
`main` titled `DO NOT MERGE: SPEC-146 verify row <M#>`, read the run, then close the PR and delete
the branch (`gh pr close <n> --delete-branch`). Push one row at a time (the workflow cancels the
previous run on a push to the same PR), and record each run id. Never put a throwaway commit on
`main` or on `feat/spec-146-wasm-roundtrip-ci`. Confirm at the end that no `verify-scratch/*`
branch remains: `git ls-remote --heads origin 'verify-scratch/*'`.

## Check, in order
1. **Scope and commit order.**
   - `git diff --stat origin/main...HEAD` against the spec's `## Outputs`/file list and Build
     Completion. Every commit is signed off. No `src/`, `tests/`, `Cargo.toml`, `Cargo.lock` or
     version change.
   - None of PROJ-017's paths is touched.
   - The unlisted header-comment edit in `scripts/wasm-ci-check.sh`: comment only, no behaviour.
2. **Does the merge still hold on current `main`?**
   - `git fetch origin`; try `git merge --no-commit --no-ff origin/main` in a scratch worktree and
     report conflicts (abort afterwards).
   - Run `git log origin/main --oneline -- tests/wasm_roundtrip.rs src/wasm.rs` and re-run
     `just wasm-test` and `just wasm-test-slim` **on the merged tree**. Do 47 and 45 still hold?
     The floor is an exact count: a wasm test added on `main` by another thread turns this job red.
     If the numbers moved, that is a punch item (or a note for the merge order); say which.
3. **Read the job whole** (`.github/workflows/ci.yml`, job `wasm-roundtrip`), against the spec:
   - the action tags and `fallback: none`, `package-manager-cache: false`, Node `'22'`, the order
     of steps, and **no `env:` anywhere in the job** (trap 16);
   - `needs: changes` and the `if:` on the code output;
   - both test steps use `>log 2>&1` with no pipe and exit with `just`'s own code; the slim step
     runs when `wasm-check` passed, so one red run reports both sets;
   - the job `name:` is stable and is the name the maintainer will require.
4. **Read `scripts/wasm-ci-check.sh` whole.**
   - `lock-version` reads exactly one `wasm-bindgen` entry from `Cargo.lock`; what does it do with
     zero or two?
   - `runner-version` compares the whole `--version` string, not a prefix; `compare-runner`
     is the same function the workflow path uses (confirm, do not trust the claim).
   - `count` is an exact match; the failure message names `FULL_COUNT`/`SLIM_COUNT` and the file.
   - What does each subcommand do with empty input, a missing log, and a log with two
     `test result:` lines (for example a test binary plus doc tests)?
5. **The two `metadata` steps and the classifier.**
   - They live in `metadata`, not in `wasm-roundtrip` or `changes`. Confirm.
   - `scripts/ci-code-paths.sh`: `\.cargo/` is in `CODE_PATHS`; the table has the three new rows;
     the summary counts mismatched, crashed and junk. Run `--self-test` and quote the summary.
     Does `.github/workflows/ci.yml` itself count as a code path?
6. **Run every check yourself.**
   - Against the head: `scripts/wasm-ci-check.sh workflow`, `--self-test`, `ci-code-paths.sh
     --self-test`, and `actionlint .github/workflows/ci.yml` (the only finding allowed is the
     baseline SC2001 note in `changes`). GREEN.
   - Against a scratch worktree of `origin/main` with the new scripts copied in: C1 and C10 RED.
     Read each RED line: is it red for the check's reason (trap 13), not a missing tool or a script
     bug?
7. **Judge the changed assertions 10 and 11.** The build made them regex the invocation line. Can
   a YAML that runs the wrong recipe, or runs it in a comment or in an `echo`, still satisfy them?
   Try: the right string inside a comment on the line above a wrong command; `just wasm-test-slim`
   used by the full-set step. Name the check that catches each, or report a gap.
8. **Re-apply the mutation table yourself,** one row at a time in a scratch worktree, local rows
   first (M1, M2, M3, M5 shape, M8, M12 to M17): apply, check `git diff` is non-empty, run C1, C3,
   C10 and C11 as the row says, record every red check **with its quoted line**, revert, confirm
   green (trap 12: apply on every path the row names). Compare with the build's record.
9. **Live rows, on throwaway PRs, one at a time** (see the exception above):
   - **M6** (`src/wasm.rs:152`, `self.has_alpha` → `!self.has_alpha`): the `wasm-roundtrip` job is
     RED on **both** steps with `info_reports_png_dimensions` / "the RGBA fixture carries alpha",
     and **every native leg is green** on the same commit. Quote the failure line and run id.
     Revert to the exact original text; `git diff` against the head is empty.
   - **M4** (`#[cfg(any())]` at `tests/wasm_roundtrip.rs:30`): both steps red with the **floor's**
     message, not a compile error or a test panic. Quote it.
   - **M7 / C6** (install `tool: wasm-bindgen@0.2.125`): the install or `runner-version` step goes
     red with the mismatch message. If `fallback: none` makes the install itself fail on a version
     the manifest lacks, say which step failed and what it printed. The frame could not drive this
     one, so it is the check most worth driving.
   - **M5 live** (full step swapped for slim, or the reverse): red with
     `slim set: 47 passed, expected 45` (or the mirror), not a pass.
   - **M12 live**: a PR whose only change is a `.cargo/config.toml` comment runs the job (the
     `changes` output is `true`). The same PR with the `\.cargo/` line reverted skips it.
   Confirm that a docs-only change skips the job and that a skipped job reports under its own
   name (read the check list).
10. **Mutations the spec did not list.** Apply each and name the check that catches it, or report a
    gap:
    - **M-a:** `lock-version` returns the version of a different crate (`wasm-bindgen-futures`, say);
    - **M-b:** the floor reads the count from the wrong `test result:` line when a log holds two;
    - **M-c:** `cargo test` output piped through `tee` in a test step (the exit code is lost): does
      anything notice, and does the job still go red on a failing test?
    - **M-d:** the slim step is made `continue-on-error: true`;
    - **M-e:** a docs sentence in `AGENTS.md` §6 is reverted: C12 red with a message naming it.
11. **Docs and comments**, against the files: `AGENTS.md` §6 (one line), `docs/roadmap.md`,
    `docs/research/proj-008-wasm-build.md`, the `justfile` hint (reads the version from
    `Cargo.lock`; it prints `0.2.126` with the runner off `PATH`), and the `.cargo/config.toml`
    comment. No behaviour change in the comment-only edits (`git diff` shows comments only).
12. **CI.** Run `gh pr checks 245` once at the head you verified and read every leg, including
    windows-latest. Quote the `wasm-roundtrip` job's two result lines and the cold and warm times.
13. **What only the maintainer can do.** Making the job a required check, after its first green run
    on `main`. Does the PR body or the spec say how, with the exact check name? Is the name the
    same in the check list and in the job's `name:`?

## Guardrails
- zsh does not word-split unquoted parameters; loop with `while IFS= read -r`. In zsh, `$B:t` is a
  modifier, so write `"${B}:path"`. macOS has no `timeout(1)`.
- A piped command reports the pipe's exit code: redirect output and read `$?`.
- Restore every mutation exactly; show `git diff` empty afterwards.
- Delete scratch worktrees, stub directories and downloaded logs when you are done. Keep scratch
  files in a scratch directory, never in `~`.
- **Budget ~70 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: search for `VERIFY prompt: CI runs the wasm round-trip`. `grep -c` can come
  back empty through the `rtk` wrapper; count with Python or `/usr/bin/grep`.
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
- `✅ APPROVED — merge PR #245 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #245 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1 to 13, one or two lines each with evidence;
- item 2: whether the merged tree still has 47 and 45 wasm tests, and any merge-order note;
- item 7: the verdict on assertions 10 and 11;
- the mutation table as you ran it, local and live, with run ids, and M-a to M-e, each caught (by
  which check) or a gap;
- confirmation that no throwaway PR or `verify-scratch/*` branch remains;
- what only the maintainer can do (the required check), and the exact name to require;
- optional cleanups, listed separately from punch items;
- your cost.
