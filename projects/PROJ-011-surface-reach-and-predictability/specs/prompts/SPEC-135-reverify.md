# SPEC-135 — RE-VERIFY prompt: the punch-list fix on draft PR #207

Cycle: **verify** (re-verify of a punch-list fix). Fresh CLI session, own worktree, **Opus**,
**read-only**, small budget. You did not build SPEC-135, write its punch list, or fix it.

```
just wt verify/spec-135-fix -- --model opus
```

Then put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin && git reset --hard origin/feat/spec-135-watermark-flag-checks
```

Draft PR #207's head should be `161b637`. Record the SHA you verify. The maintainer pasted this
prompt into a fresh session and will paste your final message back to the orchestrator, so it
must stand on its own. **This spec gates 0.8.0.**

## What was verified before, and what the fix claims
SPEC-135 makes `watermark` check `--size`, `--scale` and `--opacity` through the recipe key table,
before any file is read. It caps `size` at 1024, and fixes the saturated scaled-mark message.

It was verified at `8922f31`. Everything held: 14/14 ACs, all six negative controls, and byte
identity with `main`. There was **one punch item**:
- mutation **M-c** survived every test. M-c makes `flag_value_text` print the `f32` widened to
  `f64`;
- it survived because every test value was exact in `f32`.

There were also three wording cleanups.

The spec is on `main`, with the verify line in its timeline:
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-135-watermark-refuses-a-size-or-scale-that-is-not-a-sensible-number.md`.
The PR's copy also has the fix's notes under `## Build Completion`.

The fix is `8922f31..HEAD`: commits `2aa78a4` (test), `b11dd40` (docs) and `161b637` (cost
entry). It claims:
1. AC-4's table gains two rows that `f32` cannot hold exactly, giving 12 rows:
   - `--image logo --opacity 1.1` → exit 2,
     `error: --opacity must be a number in 0.0..=1.0, got 1.1`;
   - `--text . --size 1024.1` on 1200×1000 → exit 2,
     `error: --size must be a number > 0 and <= 1024, got 1024.1`.

   Both passed on unchanged code.
2. Under M-c, the `--opacity 1.1` row went RED with `got 1.100000023841858`. The table test stops
   at its first failure, so the `--size 1024.1` row was checked by driving the mutated binary
   directly: `got 1024.0999755859375`. No older row failed.
3. `git diff 8922f31 HEAD -- src/` is empty.
4. Three sentences were corrected:
   - DEC-106: `mark_size` returns `None` wherever a side would not fit a `u32`, NaN and −inf
     included;
   - the Build Completion: `fits_u32` differs from the design form at −inf;
   - `docs/data-model.md`: the `size` row says "finite" again.
5. The matrix is green on three legs. The orchestrator checked CI: the full CI run at `161b637`
   passed all 13 jobs, windows-latest included.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits and no pushes. Do not fix what you find; report it. Do not
edit the spec or its timeline.

## Check, in order
1. **Scope.**
   - Run `git diff --stat 8922f31 HEAD`.
   - `git diff 8922f31 HEAD -- src/` must be empty. Only `tests/watermark_cli_params.rs`, the
     spec, DEC-106 and `docs/data-model.md` may change.
   - Every commit is signed off.
2. **The two rows.**
   - Build the PR head as a release binary in a fresh target dir. Drive both commands, and check
     the exit code and the exact stderr line.
   - Confirm `1024.1` really is above the cap at `f32`, not the documented residual: it is
     `1024.0999755859375` in `f32`.
   - Run `main`'s binary as well. It gives exit 2 with the old wording for `--opacity 1.1`, and
     exit 0 for `--size 1024.1`. Record both.
3. **M-c yourself, on both rows.**
   - Apply the widening in `flag_value_text`, check `git diff` is non-empty, and rebuild after
     `cargo clean -p crustyimg`.
   - Run the AC-4 test and show it goes RED.
   - Because the table test stops at its first failure, **prove the second row on its own** too.
     Either temporarily remove the `--opacity 1.1` row in your scratch copy, or drive the
     binary. The `--size 1024.1` row must fail by itself.
   - Restore the code, rebuild, and show the test green.
   - **Judge whether the table test stopping at its first failure hides anything else.** If a
     later row's assertion could be masked, say so as an optional cleanup (collect every row's
     failures, then assert once). It is not a punch item unless it hides a real gap.
4. **The three sentences are true.** Check each against the code:
   - DEC-106 against `mark_size` and `fits_u32`;
   - the `fits_u32` sentence at −inf: is it reachable only by a library caller?
   - the `data-model.md` row against the schema's actual range check.
5. **Nothing else moved.**
   - Run the default leg once in a fresh target dir, with `--no-fail-fast`, stdout redirected to
     a file. Read `$?`.
   - Run `gh pr checks 207` **once** at the head you verified, and check every leg. Do not poll.
   - Check the fix's `cost.sessions` entry is present and its `cost.totals` add up.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`.
- If a binary you drive no longer exists, the shell exits **127**. That is not a result. Check
  each binary exists before you record its output.
- Delete each scratch target dir when you're done with it. Keep scratch files in a scratch
  directory, never in `~`.
- **Budget ~35 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "SPEC-135 — RE-VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Take input, cache_creation and cache_read once per id, and the
  **MAX** `output_tokens`.
- Price each component at the model that actually ran (read `.message.model`): Opus $5/$25 per
  MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #207 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #207 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–5, one or two lines each with evidence;
- optional cleanups, listed separately;
- your cost.
