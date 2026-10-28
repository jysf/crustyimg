# SPEC-135 — FIX prompt: the verify punch list on draft PR #207

Cycle: **build** (a punch-list fix). Fresh CLI session in the build's worktree, **Sonnet**. The
work is small and fully specified.

```
just wt feat/spec-135-watermark-flag-checks -- --model sonnet
```

`just wt` reuses the existing worktree. Then sync with the remote:
`git fetch origin && git status`. The branch must be at `8922f31`, matching
`origin/feat/spec-135-watermark-flag-checks`, with a clean tree. If not, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own. **This spec gates 0.8.0.**

## What verify found
Verify at `8922f31` returned ⚠ PUNCH LIST with **one** item. Everything else held:
- all 14 ACs, driven on the release binary;
- all six negative controls;
- byte identity with `main` (ImageMagick AE = 0);
- the matrix and CI.

**The punch item: mutation M-c survives every test.**
- Call 1 and DEC-106 both claim the CLI message prints the value as parsed: `got 1.1`, not the
  `f32` widened to `f64`, which prints `got 1.100000023841858`.
- Every value in the test rows (`nan`, `inf`, `1025`, `-1`, `100000`, `1.5`, `2`) is exact in
  `f32`. So a `flag_value_text` that widens to `f64` before printing still passes every test.

## Do, in order
1. **Add rows that `f32` cannot represent exactly** to AC-4's table in
   `tests/watermark_cli_params.rs` (`a_bad_flag_value_is_a_usage_error_naming_the_flag`). Add
   both:
   - `--image logo --opacity 1.1` → exit 2, stderr exactly
     `error: --opacity must be a number in 0.0..=1.0, got 1.1`;
   - `--text . --size 1024.1` on the 1200×1000 input → exit 2, stderr exactly
     `error: --size must be a number > 0 and <= 1024, got 1024.1`.

     `1024.1` reads as `1024.0999755859375` in `f32`, so it lies above the cap: refused, not the
     residual. **Drive it first** on a release build. If it turns out to be accepted, say so,
     and use `--size 1025.3` instead.

   Then:
   - update the table's row-count assertion from 10 to 12;
   - **Before changing `src/`:** confirm both new rows pass on the current code. The fix is
     already in the code; the row pins it.
2. **Prove the rows can fail (mutation M-c).**
   - Change `flag_value_text` to print `f64::from(v)`.
   - Check `git diff` is non-empty, then run `cargo clean -p crustyimg` and rebuild.
   - Show **both** new rows go RED with `got 1.100000023841858` and `got 1024.0999755859375`, and
     that no older row does.
   - Restore the code, check `git diff -- src/` is empty, rebuild, and show the test green.
3. **Correct three false or loose sentences** that verify found. These are wording changes only,
   with no behaviour change:
   - **DEC-106:** it says `mark_size` returns `None` "exactly where the old saturating cast
     produced 4294967295". Reword it so it is true: `mark_size` returns `None` wherever a side
     would not fit a `u32`. That covers the old saturation cases plus NaN and −inf, which the old
     cast turned into a 1×1 mark. Only a direct library caller can reach those two, because the
     CLI and the schema refuse a scale of 0 or less first.
   - **The spec's Build Completion:** `fits_u32` is not "the same semantics" as the design's
     `!(w < u32::MAX as f32)`. They differ at −inf, which the design form treats as a fit and
     `fits_u32` refuses. Say that, and that only a library caller can reach it.
   - **`docs/data-model.md`:** restore "finite" in the `size` row, matching the other ranged rows:
     finite, `> 0` and `<= 1024`.
4. **Update the spec.**
   - AC-4's table gains the two rows (12 rows).
   - In `## Failing Tests`, the AC-4 row count changes from 10 to 12.
   - The negative-control list gains **M-c** (`flag_value_text` widens to `f64`), with the two
     rows it turns RED.
   - Add a short "Verify punch list (fix)" note under `## Build Completion`, with the commit SHAs.
5. **Matrix, sequentially, each leg in a fresh `CARGO_TARGET_DIR`:** default,
   `--no-default-features` and `--features webp-lossy`. Run `fmt --check`, clippy `-D warnings`
   and tests on each, with `--no-fail-fast` and stdout redirected to a file. Read `$?`. Delete
   each target dir afterwards.
6. **Commit and push to the PR branch:**
   `git commit -s`, then `git push origin HEAD:feat/spec-135-watermark-flag-checks`. Use one test
   commit and one docs commit, or a single commit. Then start
   `gh pr checks 207 --watch` **in the background**. When it exits, read one snapshot at the head
   SHA and check every leg, including windows-latest. Never poll.

## Guardrails
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- ⛔ The only `src/` change is the temporary M-c mutation, and it is reverted. The final diff
  against `8922f31` touches `src/` nowhere. Verify will check
  `git diff 8922f31 HEAD -- src/` is empty.
- Do not edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code. zsh does not word-split unquoted
  parameters. macOS has no `timeout(1)`.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~40 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "SPEC-135 — FIX prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Take input, cache_creation and cache_read once per id, and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`): Sonnet $3/$15 per
  MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Append a `build` entry, noted as the punch-list fix, to `cost.sessions`, with `tokens_total`, a
  `tokens_breakdown` and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-135 fix: PR #207 at <head SHA> — CI <green | red: legs>`.

Then, short:
- the two new rows, and whether `1024.1` was refused as expected;
- M-c: the RED output for each row, and green after the revert;
- `git diff 8922f31 HEAD -- src/` is empty;
- the three sentences corrected;
- the matrix result;
- your cost.
