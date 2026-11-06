# SPEC-148 — VERIFY prompt: `lint` fixes what it names

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**. You did not design or build SPEC-148. Read-only toward
the PR under review.

```
just wt verify/spec-148 -- --model opus
```

Put the worktree on the PR's code:
```
git fetch origin feat/spec-148-lint-fixes-what-it-names:refs/remotes/origin/feat/spec-148-lint-fixes-what-it-names
git reset --hard origin/feat/spec-148-lint-fixes-what-it-names
```
Draft PR #253's head was `8f61167` when this prompt was written (3 commits; CI green on every leg incl.
windows-latest). Record the SHA you verify. The maintainer pastes this prompt into a fresh session and pastes your
final message back to the orchestrator, so it must stand on its own.

⚠ **Another orchestrator thread owns PROJ-017 and builds SPEC-147 (rotate/flip) now.** Do not touch `src/operation/`,
`src/recipe/`, `docs/recipes.md`, `tests/common/mod.rs`, `tests/wasm_roundtrip.rs`, `scripts/wasm-ci-check.sh`,
`README.md`, `CHANGELOG.md`. Use your own `CARGO_TARGET_DIR`; never share one.

## What the build claims
- Spec: `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-148-lint-fixes-what-it-names.md`. Read it
  all (`## Frame`, `## Design`, 11 ACs, FT-1 to FT-12, Implementation Context, the 16-row mutation table,
  `## Build Completion`). HANDOFF traps 1 to 17 and rules 1 to 7 bind you (rule 7: optional cleanups go in one
  punch list; trap 17: a bound worked out on paper is a claim until its maximising input is driven).
- Commits `4304607` (red checkpoint), `2a0a0ae` (fix), `8f61167` (Build Completion, advance). All three claim DCO.
- No mutation was pushed. The build did not touch `tests/common/mod.rs`, `src/cli/mod.rs`, or any PROJ-017 file.
- RED on `main` then GREEN: FT-1 (13 rows) etc. Local mutations M-1 to M-11, M-13, M-14 run; **M-12 (Windows forced to
  POSIX quoting) was not driven anywhere**. The Windows leg passed, but the build did not quote that leg's FT-1
  output.
- Deviations: helpers live in `tests/lint.rs`; `adoption_glue.rs` has its own coordinate fixture, so the shared
  `jpeg_with_gps` is unused by lint tests (check for dead-code warnings and clippy `-D warnings` in every test
  binary that includes `common`); FT-12 checks the §5 slice of the doc; M-10 differs from the spec (FT-9 stays green
  because stdin returns early).

## Checks (do all; report each)
1. **Scope and commit order.** `git diff origin/main...HEAD --name-only` is only the files above; nothing under
   PROJ-017's paths or `tests/common/mod.rs`; all commits signed off; no `src/cli/mod.rs`.
2. **Merge against current `main`.** In a scratch tree `git merge --no-commit --no-ff origin/main`; if SPEC-147 has
   merged, the merge must be clean and `just check` green on the merged tree. Report conflicts, do not resolve.
3. **Read the code.** `src/lint/mod.rs` formatter in full (`fix_command_for`, `shell_arg`, the safe set, the `./`
   guard, stdin and non-UTF-8 cases), `rules.rs` fragments, `report.rs` (JSON `fix:null`, SARIF without the suffix,
   human line). Look for: a path or fragment that reaches the shell unquoted; quoting for `$`, backtick, `!`, `*`,
   newline, tab, unicode, empty string; the PowerShell branch (`'` doubling and the curly quotes) vs the POSIX
   branch; anything that makes a printed fix pair `--format` with a different `-o` extension.
4. **Drive the headline claim yourself, not through the build's test.** Make fresh fixtures (ImageMagick/exiftool;
   say which are available). For each rule, run `lint`, copy the printed `fix:` line exactly, run it in `sh -c`
   on a scratch copy, then re-run `lint` on the result. Report per rule: stdout bytes (must be 0), file changed,
   finding cleared, format kept. Include names with a space, a `'`, a `$HOME`, a backtick, a leading `-`, a unicode
   name, and a name `stdin`. Also the subdirectory case.
5. **Trap 17: the byte budget.** `oversized-bytes` prints `optimize --max-size <budget>`. The frame showed plain
   `optimize` missing a 100 KB budget. Drive the *maximising* inputs: a file just over the budget, a file whose
   `optimize` result lands barely under or over it, a PNG, a WebP, an already-optimal file that cannot get smaller.
   Does every printed fix clear the finding, and what happens when it genuinely cannot (does `optimize` exit
   non-zero and leave the file unchanged, or truncate)? A fix that cannot clear its finding must say so; report the
   behaviour either way.
6. **The GPS rule.** Re-derive from tags: version-only (no finding); latitude only; longitude only; dest-lat;
   dest-long; `GPSAreaInformation` alone; altitude/timestamp/datum only; real coordinates; a GPS block in a PNG or
   WebP/HEIC if the rule covers them. Use `exiftool` to build them *and* inspect with an independent reader
   (not crustyimg's own metadata code). Also: `meta clean --gps` on each clears the finding. Verify the exit codes
   (7 on a flag, 0 clean).
7. **The Windows leg.** Open the PR's windows-latest job log and quote FT-1's output (rows, counters) so we know it
   ran in real PowerShell and did not skip. Then drive **M-12** live: a throwaway draft PR from a
   `verify-scratch/spec-148-m12` branch (base: the PR branch, so the diff holds only the mutation) with Windows
   forced to POSIX quoting; the Windows leg must go red on the quote row for its reason. One live row at a time;
   close with `gh pr close --delete-branch`. Also read FT-1's Windows command line in the test source and say what
   would make it pass wrongly (PATH resolving a different `crustyimg`; `-Command` swallowing a nonzero exit).
8. **Re-run the checks and the mutation table.** `just check`, `cargo clippy --all-targets -- -D warnings`
   (default and lean), `cargo fmt --check`. Re-apply a representative set of mutations in a scratch tree, each
   with a non-empty `git diff`, reverted to empty: M-1 (old formatter), M-2, M-4, M-6, M-7, M-8, M-11. Each must
   go red for its reason, with the failing line quoted (trap 13). Control for the fix is the old code restored
   exactly (trap 14).
9. **Mutations the spec did not list** (try at least four): e.g. `shell_arg` safe set widened to include space;
   `-y` replaced by `--yes`-less variant; the budget rounded down; `GPSDestLatitude` removed from the predicate; the
   CMYK fix using `-o` with a different extension; a fix emitted for stdin when the path is `-`; SARIF keeping the
   `— fix:` suffix on a no-fix finding. Which are caught, which are gaps?
10. **Contracts and docs.** `fix:` in the JSON schema (`crustyimg.lint/v1`) is `null` when absent: confirm the
    documented schema allows null. `docs/github-actions.md` fix table matches the real output, sentence by sentence.
    List any other doc or `--help` string that shows a `fix:` line (SPEC-150 owns `lint --help`'s sentence).
11. **The fixtures.** `cmyk_16x16.jpg` size and provenance in its README; hand-built GPS bytes decoded by an
    independent reader.
12. **CI legs.** Every leg at the head, incl. windows-latest, macOS, lean, msrv, DCO.

## Guardrails
- ⛔ Push nothing to #253. Throwaway PRs only for live rows, from `verify-scratch/*`, closed with
  `gh pr close --delete-branch`; `git ls-remote --heads origin 'verify-scratch/*'` must be empty at the end.
- ⛔ Nothing publishes or tags. A piped command reports the pipe's exit code: redirect and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`. Scratch files in a scratch directory, never in `~`.
- **Budget ~60 exchanges.** Optional findings go in one punch list; the orchestrator will not re-verify them
  (rule 7). Only a required defect triggers a fix round and a narrow re-verify.

## Measure your own cost
Transcript JSONL under `~/.claude/projects/<slug>/` (absolute working directory with every `/`, `.`, `_` replaced
by `-`); identify **by content**: search `VERIFY prompt: lint fixes what it names`. Python or `/usr/bin/grep`.
⚠ Dedup by `.message.id`: input, cache_creation, cache_read once per id, **MAX** `output_tokens`. Price per
component at the model that ran (Opus $5/$25, Sonnet $3/$15; cache_creation ×1.25, cache_read ×0.10).

## Report back (your final message)
First line, one of:
- `✅ APPROVED — merge PR #253 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #253 at <SHA> needs: <required fixes>.` (separate required items from optional ones)
- `❌ REJECTED — <reason>.`
Then checks 1 to 12 with evidence (quoted lines), the Windows FT-1 output and the M-12 result, the merged-tree
result, cleanup confirmation (no `verify-scratch/*` branches), anything only the maintainer can do, and your
measured cost.
