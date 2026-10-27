# SPEC-133 — RE-VERIFY prompt: the punch-list fix on draft PR #206

Cycle: **verify** (re-verify of a punch-list fix). Fresh CLI session, own worktree, **Opus**,
**read-only**, small budget. You did not build SPEC-133, write its punch list, or fix it.

```
just wt verify/spec-133-fix -- --model opus
```

Then put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin && git reset --hard origin/feat/spec-133-watermark-damage-rule
```
PR #206 is a **draft**. Its head was `458254d` after the fix, and should be one docs commit later
(see item 6). Record the SHA you verify. The maintainer pasted this prompt into a fresh session and
will paste your final message back to the orchestrator, so it must stand on its own.

## What was verified before, and what the fix claims
SPEC-133 (a watermark refuses to damage an image; DEC-104) was verified at `3dd23ff`. The behaviour
was correct on every surface, and passing marks were byte-identical to `main`. The punch list:
- four mutations no test caught (M1, M2, M5, M7);
- three false or missing doc sentences.

Read `## Build Completion` and the verify line in the timeline, which is on `main`:
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-133-watermark-refuses-to-damage-an-image.md`.

The fix is `3dd23ff..HEAD`: commits `3da5780`, `5c32966`, `aeb52e1`, `458254d`, any merge of
`main`, and possibly one later docs commit. It claims:
1. **M2** (count the visible pixels after opacity is applied): AC-5 now uses `opacity = 0.001`
   (255 × 0.001 rounds to 0). At 0.01 the old row was hollow. AC-5's wording was corrected.
2. **M1** (count on `to_rgba8()`): `coverage_counts_a_sixteen_bit_overlay_at_its_own_depth`, with
   501 px at 16-bit alpha 128.
3. **M7** (judge the pipeline's input instead of the image entering the step):
   `rule_judges_the_image_entering_the_step_when_it_grew` (exit 0, mark drawn) and `…when_it_shrank`
   (exit 2). Only the first is expected to flip, because `apply`'s own guard backs up the second.
4. **M5** (count K per step): `skipped_count_is_per_image_not_per_step`.
5. The `watermark --help` sentence is scoped to a fixed-size mark (no `--scale`, no `--tile`).
6. The input count is taken after globs and directories resolve. This is now documented in
   `docs/api-contract.md` (plus the exit-code row), `recipes/README.md` and DEC-104. There is a new
   test, `a_directory_resolving_to_one_file_is_refused_with_exit_2`. The fix reported that
   `docs/data-model.md:248` still says "On one explicit input". If a later commit changed it, check
   that the change is right. If none did, list it as a punch item.
7. A CHANGELOG sentence reworded so it no longer claims an "instead of" that no release had.
8. Cleanups:
   - `optimize_decide_one` returns a named `DecideOne` struct, and the `type_complexity` allow is
     gone;
   - one `input_label()` helper, computed only when a line prints;
   - SPEC-131's `scale` range rows, and the `scale = true` type row, moved to an image step, with
     an assertion that the message is not the mode message;
   - `scale = 1` on an image step in the integer corpus recipe;
   - a DEC-104 line on faint 16-bit marks.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Do not fix what you find; report it. Do not
edit the spec or its timeline.

## Check, in order
1. **Scope.**
   - Run `git diff --stat 3dd23ff HEAD`.
   - Any merge of `main` must bring only `main`'s changes.
   - `src/` may change only in three places: unit tests in `src/operation/mod.rs`, the help text
     in `src/cli/mod.rs`, and `src/cli/optimize.rs` (items 8 and 9).
2. **Mutations, each yourself.**
   - Apply M1, M2, M5 and M7 one at a time. Check that `git diff` is non-empty, then rebuild in a
     **fresh** `CARGO_TARGET_DIR` (reverting source does not rebuild the binary).
   - Show the named test goes RED. Restore the source, rebuild, and show it green.
   - For M2, confirm by arithmetic, not by trusting the claim, that the composited alpha at
     0.001 really is 0 for every alpha the overlay uses.
   - For M1, confirm alpha 128/65535 truncates to 0 under `to_rgba8()`.
3. **Items 8 and 9 change no behaviour.**
   - Read the `optimize.rs` diff: the struct carries the same fields the tuple did, and every
     caller reads the field it read before.
   - `input_label()` gives the same label as the code it replaced, for a path, for stdin, and
     for a glob-expanded input. Drive one batch error and one skip, and compare their stderr
     with `3dd23ff`'s binary.
4. **Item 10 keeps SPEC-131's guarantees.** Each moved row fails with its range or type message
   on an image step, and still fails if the mode check is removed. Mutate the range check out
   and confirm the moved rows go RED.
5. **Docs: drive each new sentence on the binary**, not against the code:
   - the `--help` text (a tiled 100 % mark exits 0; a fixed-size 501 px mark is refused);
   - the glob/dir-of-one sentence (one image → exit 2; two → skip, exit 0);
   - the CHANGELOG line.
   Also check that other specs' CHANGELOG entries are untouched.
6. **Full suite and CI.**
   - Run the default leg once in a fresh target dir.
   - Run `gh pr checks 206` at the head you verified, and check every leg. Do not poll.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout
  to a file.
- A piped command reports the pipe's exit code: redirect output and read `$?`.
- zsh does not word-split unquoted parameters.
- macOS has no `timeout(1)`.
- Delete each scratch target dir when you are done with it.
- **Budget ~50 exchanges.**

## Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Find it by content:
`grep -l "SPEC-133 — RE-VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.

Dedup by `.message.id`: take input, cache_creation and cache_read once per id, and the MAX
output_tokens. Price each component, never at a flat rate: Opus $5/$25 per MTok, cache_creation
×1.25 the input rate, cache_read ×0.10. Report `tokens_total`, the breakdown and
`estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #206 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #206 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–6, one or two lines each with evidence;
- optional cleanups, listed separately;
- your cost.
