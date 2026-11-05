# SPEC-145 — PUNCH LIST 2 prompt: 0.4 is what the sweep measured, not the maximum

Cycle: **build (punch list 2)**. Fresh CLI session, own worktree, **Sonnet**. Tiny, and text
only: correct one false claim in DEC-112 and the spec, one false control row, and one test
message. You did not design, build or verify SPEC-145.

```
just wt fix/spec-145-punch2 -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-145-crop:refs/remotes/origin/feat/spec-145-crop
git reset --hard origin/feat/spec-145-crop
```
Push to the PR branch with `git push origin HEAD:feat/spec-145-crop`. Draft PR #244's head was
`0c6f722` when this was written. If it has moved, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own.

## Why
The re-verify confirmed all four cleanups, and the full suite is green (1,156 passed). But the
first punch list wrote two false statements.

1. **"0.4 px is the maximum drift" is true only in exact maths.** DEC-112 now says "Drift comes in
   multiples of 0.2 and halves round up, so the largest value reached is 0.4 px." The spec's
   design note says "0.4 px is the real maximum". But `x * 1200` and `x * 6000` are rounded
   separately in `f64`, so near a half-pixel tie the two sizes can round in opposite directions.
   - A `nextafter` search around `(n+0.5)/1200` found 207 fractions with drift 0.6.
   - Driven on the binary: `width = 0.009583333333333333` gives a 12 px crop at 1200 and 57 px at
     6000. 57/5 = 11.4, a drift of 0.6.
   - 0.4 is what the seeded 200,000-case sweep measures; it never lands that close to a tie. The
     test's `0.4 + 1e-9` bound is still correct for that fixed seed, and it's deterministic.
     STAGE-058's "within one pixel" still holds. **The code and the test bound stay as they are.**
2. **The "+0.1 control" row is wrong.** The Punch list subsection says the "+0.1 before
   `.round()`" mutation is "red at 0.4 and green at 0.6". At `6611e79`'s bare `drift <= 0.6` it's
   **red too**, at drift 0.6000000000000227. No mutation tested lands between the two bounds.

## The fix (settled, do not re-decide)
1. **Restate the drift claim** wherever it appears. Say: **0.4 px is measured over the seeded
   sweep; 0.6 px is the bound in general, reached at `f64` near-ties.** Give the driven example
   (`width = 0.009583333333333333`: 12 px at 1200, 57 px at 6000) in DEC-112 and the spec's
   design note.
   - Find every place by content: `/usr/bin/grep -rn "0\.4" decisions/DEC-112-*.md <the spec>
     src/operation/crop.rs tests/crop.rs`, and read each hit. That covers the DEC, the design
     note, AC-5, the Failing Tests entry, the M1 row and the test's comment.
   - AC-5 and the test's comment must say the 0.4 bound holds **for this seed**, and that
     changing the seed or the case count can reach 0.6 on correct code.
   - Rewrap each line you touch to the file's width, and restore the backticks around
     `0.5 + 0.5/5 = 0.6` in DEC-112.
2. **Correct the +0.1 row.** It's red at both 0.4 and 0.6, so it shows the bound bites, not that
   0.4 is tighter than 0.6. Say so, with both drift values.
3. **The test message prints the bound it checks.** `crop_fraction_agrees_at_1200_and_6000`'s
   panic message hardcodes "> 0.4". Make it print the same bound value the assert uses, so the
   message and the check can't drift apart. Test module only; no behaviour change.

Stay in these files: DEC-112, the SPEC-145 spec, and `src/operation/crop.rs`'s `mod tests`.
`git diff --stat 0c6f722` must show nothing else. Any `src/operation/crop.rs` hunk must sit inside
`mod tests`.

## Prove it
- **Drive the near-tie yourself** on this branch's release binary, before you write it into the
  DEC: `width = 0.009583333333333333` (with `x = 0`, `y = 0`, `height = 1`) on a 1200 px-wide and
  a 6000 px-wide image made by a tool other than crustyimg. Quote both output widths.
- **The message fix:** temporarily set the bound to 0.3, run the test, and quote the panic. It must
  name 0.3. Restore, and show it green.
- Run `cargo test --lib operation::crop` and `--test crop` green.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag. No behaviour
  change.
- Before pushing, run `cargo clippy --all-targets -- -D warnings` and with
  `--no-default-features`, and `cargo fmt --check`.
- **Never poll CI.** Run `gh pr checks 244 --watch` in the background. When it exits, read one
  direct snapshot at the PR's true head SHA.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect and read `$?`. zsh does not
  word-split unquoted parameters. `rtk` can truncate `grep`, so use `/usr/bin/grep`.
- Never share a `CARGO_TARGET_DIR` with another session. Delete scratch dirs and images when
  you're done.
- **Budget ~15 exchanges.**

## When you finish
- Add a `### Punch list 2 (re-verify)` subsection to the spec's `## Build Completion`, listing the
  three fixes, the near-tie widths you drove, the 0.3 control, and the CI run id.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `grep -l "SPEC-145 — PUNCH LIST 2 prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
    `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
  - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
    MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read ×0.10.
  - Append a `punchlist-2` entry to the spec's `cost.sessions`.
  - Also append the **re-verify** session's entry, which it measured but couldn't write (it was
    read-only): input 38, output 12,955, cache_creation 64,057, cache_read 1,545,215, total
    1,622,265, Opus, **$1.50**.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-145 punch list 2: PR #244 at <head SHA> — CI <green | red: legs>`.

Then, short:
- `git diff --stat 0c6f722`;
- the restated sentence in DEC-112, quoted, and every other place you changed;
- the near-tie widths you drove;
- the corrected +0.1 row, quoted;
- the 0.3 control's panic, quoted;
- the CI run id;
- anything that did not go as written;
- your measured cost.
