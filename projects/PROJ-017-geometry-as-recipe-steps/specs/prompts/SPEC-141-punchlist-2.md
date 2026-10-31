# SPEC-141 — PUNCH LIST 2 prompt: pin the real golden-file lookup

Cycle: **build (punch list 2)**. Fresh CLI session, own worktree, **Sonnet**. Small and
test-only: one test on the real lookup, plus three message and README cleanups. You did not
design, build or verify SPEC-141.

```
just wt fix/spec-141-punch2 -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin test/spec-141-recipe-output-guard:refs/remotes/origin/test/spec-141-recipe-output-guard
git reset --hard origin/test/spec-141-recipe-output-guard
```
Push to the PR branch with `git push origin HEAD:test/spec-141-recipe-output-guard`. Draft PR
#233's head was `6b90d71` when this was written. If it has moved, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own.

## Why
The re-verify confirmed the first punch list. The missing-golden path prints its dump, the
README's procedure alone takes a simulated new recipe green, and FT-3, the row count and FT-4 all
hold. The full suite is green (1124 passed).

One gap remains. **FT-9 never calls `golden_path_for`.** It hands its goldens in through a
closure. So the re-verify's mutation (A), which put the old `|| !path.exists()` panic back
verbatim in `golden_path_for`, left all 9 tests green. With a Mac file then moved aside, the real
test panicked at about `:491` and printed no dump. The first punch list's item 1 can come back,
and no test would see it.

## The fix (settled, do not re-decide)
1. **Pin the real lookup.** Extend `a_target_without_goldens_fails`, or add a test beside it,
   that calls `golden_path_for` for a **known** target with a stem that has **no file on disk**.
   - It must return `Ok(path)`, with the path it expected, so the caller can report it as
     missing.
   - For an unknown target it stays `Err` with Call 5's message (FT-8 is unchanged).
   - Use a stem that can never exist, such as `__no_such_recipe__`, so no fixture is moved or
     deleted.
2. **The stray-file check names the file.** When
   `every_golden_line_is_checked_and_no_golden_is_stray` fails, its message names each
   **missing** and each **extra** `.output` file. Don't print only the two 42-entry lists.
3. **FT-4's message says what it compares.** Today it reads "apply wrote different bytes than
   the library seam", but it compares against the golden. Reword it to name the golden. Under M1,
   both `apply` and the library change.
4. **The README's comment list** gains the `GOLDEN_LINES` doc comment ("42 pixel rows", about
   `:54`).

**Not in scope:** turning "no expected dims" into a mismatch line, and moving the `ROWS` and `42`
asserts after the dump. The README bumps those literals before blessing, and the re-verify drove
that order to green. Leave them as they are.

Stay in these files: `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/README.md`
and the SPEC-141 spec. `git diff --stat 6b90d71` must show nothing else, and
`git diff 6b90d71 -- 'tests/fixtures/recipe_corpus/*.output' src/ Cargo.toml Cargo.lock` must be
empty.

## Prove it can fail
Run each of these one at a time and record the result. Redirect stdout to a file and read `$?`.
- **Mutation (A), the one the re-verify used.** In `golden_path_for`, put back the old
  `|| !path.exists()` panic exactly. Rebuild clean. The new test must go red. Restore, and show
  it green.
- **The stray-file message.** Move `bundled_web.linux-x86_64.output` aside. The stray-file check
  must fail **naming that file** as missing. Then copy any `.output` to
  `zz_extra.linux-x86_64.output`: the check must name it as extra. Restore both, and show
  `git status --short` empty.
- **M1, re-driven** (rebuilt clean): FT-4 names `png` red and `webp` green, with the new wording.
  Restore.
- Then the whole `recipe_corpus_output` test is green. Count its tests: there should be one more
  than before only if you added a test rather than extending one. Say which you did.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag. No `src/`
  change committed.
- Before pushing, run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`, and do
  the same with `--no-default-features`. A constant only the format test uses needs its cfg gate.
- **Never poll CI.** Run `gh pr checks 233 --watch` in the background. When it exits, read one
  direct snapshot at the PR's true head SHA, and give `recipe_corpus_output`'s test counts on a
  default leg and the lean leg.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the pipe's exit code, so redirect and read `$?`.
- Reverting source doesn't rebuild the binary, so rebuild clean after each mutation.
- zsh does not word-split unquoted parameters. `rtk` can truncate `grep`, so use `/usr/bin/grep`
  for anything you count.
- Delete scratch target dirs when you're done.
- **Budget ~25 exchanges.**

## When you finish
- Add a `### Punch list 2 (re-verify)` subsection to the spec's `## Build Completion`, listing
  the four fixes, each control with its result, and the CI run id and test counts.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `grep -l "SPEC-141 — PUNCH LIST 2 prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
    `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
  - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
    MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read ×0.10.
  - Append a `punchlist-2` entry to the spec's `cost.sessions`.
  - Also append the **re-verify** session's entry, which it measured but couldn't write (it was
    read-only): input 62, output 17,722, cache_creation 103,188, cache_read 3,308,199, Opus,
    **$2.74**.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-141 punch list 2: PR #233 at <head SHA> — CI <green | red: legs>`.

Then, short:
- `git diff --stat 6b90d71`;
- mutation (A): the new test's red, then green;
- the stray-file messages for the missing and the extra file, quoted;
- M1's FT-4 lines, quoted;
- CI: the run id and the test counts;
- anything that did not go as written;
- your measured cost.
