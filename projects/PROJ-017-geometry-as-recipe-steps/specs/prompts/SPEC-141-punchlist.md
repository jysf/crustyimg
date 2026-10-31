# SPEC-141 — PUNCH LIST prompt: make the recipe-output guard blessable for the next recipe

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. The work is
test-only: a failure path that must still print its CI-log dump, three message formatters that
FT-3 must check, a row count, and the README. You did not design, build or verify SPEC-141.

```
just wt fix/spec-141-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin test/spec-141-recipe-output-guard:refs/remotes/origin/test/spec-141-recipe-output-guard
git reset --hard origin/test/spec-141-recipe-output-guard
```
Push to the PR branch with `git push origin HEAD:test/spec-141-recipe-output-guard`. Draft PR
#233's head was `dfaa28b` when this was written. If it has moved, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own.

## Why
Verify confirmed that the guard pins output correctly on all three CI targets, and that every
mutation matched the spec. M4b also closed the build's open question: the 51 line-count literal
goes red on its own.

But the guard **can't be blessed from a CI log** in the situation every later PROJ-017 spec will
hit: a new recipe, whose Linux and Windows `.output` files don't exist yet.
- `check_all` calls `golden_path()`, which panics before any dump is built. So the
  `else { Golden::default() }` branch (about `tests/recipe_corpus_output.rs:605-607`) is dead
  code.
- Verify deleted one target's file. FT-1 panicked at about `:467`, and the output had **0**
  `BEGIN` blocks.
- The message also contradicts itself: `no recipe golden for macos-aarch64: goldens exist for
  linux-x86_64, macos-aarch64, windows-x86_64`.

Two more gaps:
- FT-3 checks 8 of Call 4's 9 lines. It doesn't check AC-4's three control lines. Those strings
  are built inline in `compute`, so no test pins their wording.
- The README doesn't name the literals a later spec must bump, or say where the new Linux and
  Windows files come from.

## The fix (settled, do not re-decide)
1. **A missing golden file fails, and still prints every dump.**
   - In `check_all`, a missing `.output` for this target becomes **one mismatch line** naming the
     file it expected. For example: `v2_new_recipe: no golden file
     tests/fixtures/recipe_corpus/v2_new_recipe.linux-x86_64.output (bless this target from
     this run's dump below)`.
   - The check then carries on with an empty golden, so the rows still compute and the
     `BEGIN`/`END` block for that recipe still prints. The test still fails (trap 11: a missing
     golden never skips).
   - **Keep FT-8 as it is.** `golden_path` for an unknown target is still `Err` with Call 5's
     message, and never falls back to another target's file.
   - Fix the contradictory message. Whatever lists "goldens exist for …" must list what is on
     disk for **this** target, or be dropped.
   - Add **FT-9 `a_missing_golden_fails_and_still_prints_its_dump`**. It drives the report
     builder with one recipe's golden absent, as a pure function, with no file deleted. It
     asserts:
     - exactly one missing-file line;
     - the failure count includes it;
     - that recipe's `BEGIN … END` block is present and complete;
     - the other recipes' blocks are present too.
2. **FT-3 covers the three control lines.**
   - Move AC-4's three strings out of `compute` into small formatter functions, the same way
     Call 4's lines are built:
     - `<recipe> × <fixture>: dims <actual>, expected <literal> (expected-dims table)`;
     - `<recipe> × <fixture>: output equals the fixture`;
     - `<recipe> × <fixture>: step <i> ('<op>') changed nothing`.
   - `compute` calls them, and FT-3 checks each against its exact wording, as a string.
3. **`<N> of 48 rows changed` counts rows, not lines.** Today N counts mismatch lines, so M3
   prints `96 of 48`. Make N the number of distinct rows with at least one mismatch line, and keep
   the wording.
   - M1 and M2 still read `36 of 48` and `9 of 48`, since each of their rows has one line.
   - Under M3 it must read `42 of 48`.
4. **FT-4 reports both rows before it fails.** It collects the `png` and `webp` results, then
   fails once naming every row that went red. Today it stops at the first, so M1's "webp green"
   can't be observed.
5. **The README (`tests/fixtures/recipe_corpus/README.md`) and the spec's Call 7.**
   - **"Adding a recipe":** name every literal by its identifier and value:
     - `ROWS` (48);
     - `GOLDEN_LINES` (51);
     - `EXPECTED_DIMS` (one entry per recipe);
     - the `42` in FT-1's `EXPECTED_DIMS.len() * FIXTURES.len()` assert;
     - the `[…; 14]` array length;
     - anything else your grep finds.

     Find them by grepping the test for numeric literals and array lengths. Don't work from this
     list.
   - **The bless procedure for the targets you don't have.** Bless your own target locally. Push.
     The CI run fails on the other targets and prints each one's dump. Copy each block from
     `gh run view <run> --log-failed` into its file, push, and the second run must be green on
     every leg. New files and new lines for a new recipe aren't a re-bless.
   - **A red on one target, with no code change.** The goldens pin each runner image's `powf`
     (its libm), not only its OS and architecture. So a red on one target alone, with no code
     change, after a runner-image update, means libm drift. Re-bless **that target only** from
     its CI log, in its own commit whose message says so. Never fold it into a feature PR.
   - Update Call 7 in the spec to match, and keep the behaviour-first wording (no spec or DEC
     ids in the README).

Stay in these files: `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/README.md`,
and the SPEC-141 spec. **No golden file changes.** `git diff --stat` must show nothing else, and
`git diff --stat dfaa28b -- 'tests/fixtures/recipe_corpus/*.output'` must be empty.

## Prove it can fail
Run each of these one at a time and record the result. Redirect stdout to a file and read `$?`.
- **The missing-golden path, driven for real.**
  - Move `v1_resize_max.macos-aarch64.output` aside, and run
    `cargo test --test recipe_corpus_output`.
  - It must exit non-zero, with exactly one missing-file line naming that file.
  - Count the `BEGIN` blocks with `/usr/bin/grep -c`: there must be **14**.
  - The `v1_resize_max` block must equal the file you moved aside. Diff them, after stripping
    the markers. That proves the dump is the bless source.
  - Restore the file, and show the test green.
- **FT-9's own red.** Put the old panic back temporarily. FT-9 must go red. Restore it.
- **FT-3's new rows.** Change one character in each new formatter, one at a time. FT-3 must go
  red each time. Restore.
- **M3, re-driven.** Apply `.take(0)` at `src/pipeline/mod.rs` (find it by content), re-bless
  under it, rebuild clean, and run.
  - It must print `42 of 48 rows changed`.
  - Then restore with `git checkout -- src/ tests/fixtures/`, and show `git status --short`
    empty.
- **M1, re-driven.** Rebuild clean. FT-4 must now name its `png` row as red and leave its `webp`
  row green. Restore.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag. No `src/`
  change committed.
- Before pushing, run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`, and do
  the same with `--no-default-features`. The lean leg compiles the format test out, and a
  constant only that test uses needs the same cfg gate (the build hit this once).
- **Never poll CI.** Run `gh pr checks 233 --watch` in the background. When it exits, read one
  direct snapshot at the PR's true head SHA, and confirm `recipe_corpus_output` ran **9** tests on
  the default legs (8 on lean and webp-lossy).
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the pipe's exit code, so redirect and read `$?`.
- Reverting source doesn't rebuild the binary, so after each mutation rebuild clean (a fresh
  `CARGO_TARGET_DIR` or `cargo clean -p crustyimg`).
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate
  `grep` output, so use `/usr/bin/grep` for anything you count.
- Delete scratch target dirs when you're done.
- **Budget ~35 exchanges.**

## When you finish
- Add a `### Punch list (verify)` subsection to the spec's `## Build Completion`. It lists:
  - the five fixes;
  - each control above, with its result;
  - the CI run id and test counts.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `grep -l "SPEC-141 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
    `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
  - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
    MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions`.
  - Also append the **verify** session's entry, which the verify measured but couldn't write
    (it was read-only): 10,590,053 tokens (input 130, output 47,860, cache_creation 186,613,
    cache_read 10,355,450), Opus, **$7.54**.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-141 punch list: PR #233 at <head SHA> — CI <green | red: legs>`.

Then, short:
- `git diff --stat dfaa28b`;
- the missing-golden control: exit code, the line, `BEGIN` count = 14, and the diff result;
- FT-9's and FT-3's reds;
- M3's `42 of 48`, and M1's FT-4 rows;
- the literals the README now names, and the grep that found them;
- CI: the run id and the test counts per leg;
- anything that did not go as written;
- your measured cost.
