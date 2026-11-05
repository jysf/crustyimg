# SPEC-145 — PUNCH LIST prompt: four small cleanups before merge

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. Small: one test
bound, one test assert, one docs line, and the spec's mutation table. You did not design, build
or verify SPEC-145.

```
just wt fix/spec-145-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-145-crop:refs/remotes/origin/feat/spec-145-crop
git reset --hard origin/feat/spec-145-crop
```
Push to the PR branch with `git push origin HEAD:feat/spec-145-crop`. Draft PR #244's head was
`6611e79` when this was written. If it has moved, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own.

## Why
Verify **approved** #244 at `6611e79` with no punch items. It confirmed:
- all 36 refusal messages, byte for byte, through the binary;
- the size-independence thesis on 6000 px and 1200 px images, with pixels exact against `vips`;
- M1, M3b, M8, M12 and M4b, each driven;
- no existing corpus output changed;
- the docs, wasm and every CI leg.

It listed four optional cleanups, and the maintainer chose to fix them before merging.

## The fixes (settled, do not re-decide)
1. **Tighten the size-independence bound.** In `crop_fraction_agrees_at_1200_and_6000`, change
   the bound from **0.6** to **0.4**.
   - 0.4 is both the measured maximum over the seeded 200,000 cases and the analytic one: drift
     comes in multiples of 0.2, and halves round up.
   - At 0.6, a mutation that drifts exactly 0.6 would pass.
   - Update the assert's message and any comment that states the bound. Keep the seed and the
     case count.
   - If the spec's AC or DEC-112 states 0.6 as the bound, see fix 4.
2. **Assert the A-table's row count.** The R table asserts `rows.len() == 32`. The A table's count
   rests only on its `[_; 4]` array type. Add an explicit `assert_eq!(rows.len(), 4, …)` in the
   same style as the R table's.
3. **Fix the exit code in `recipes/README.md`'s crop section.** It says a bad crop "fails
   (exit 1)". That's true only for a single input. A batch where every input fails exits 6. A
   partial failure also exits 6, and the valid outputs are still written. Reword it to match
   `docs/api-contract.md`, which already says "for one input".
   - Before you write the batch sentence, drive both cases on this branch's binary, a two-input
     batch with one bad and one good input, and quote each exit code.
   - Check every other crop doc surface the spec lists (`docs/recipes.md`, `docs/data-model.md`,
     `skills/crustyimg/SKILL.md`, the ddphotos README) for the same claim, and fix each one you
     find.
4. **Bring the spec's mutation table up to date.**
   - **M4b's row:** with the check moved into the refusal hook, a single-input `apply` exits 2
     (`error: crop refused: …`) and writes nothing. A two-input batch exits 0 and writes both
     images uncropped. Today the table says the single input exits 0.
   - **The extra red rows** the build and verify both saw:
     - M2: `apply_crops_a_png_end_to_end` and the second case of `halves_round_up…`;
     - M4: `refusal_hook_is_not_used`;
     - M6: `aspect_never_exceeds_the_image`, with A3 and the portrait corpus row printing the
       apply-side bounds message;
     - M12: `every_op_named_in_data_model_resolves`.
   - Where the spec or DEC-112 states the 0.6 bound, change it to 0.4.

   DEC-112 is on `main`, not only on this branch. If it states 0.6, change it **on this branch**,
   so the change merges with the PR, and say so in your report.

**Not in scope:** the "watermark skipped" wording in `src/cli/common.rs:40`. Verify filed it for
STAGE-058's backlog; the orchestrator files it. No `src/` change at all.

Stay in these files: the crop test file(s) holding the two tests above, `recipes/README.md`, any
other crop doc surface that makes the exit-1 claim, the SPEC-145 spec, and DEC-112 only if fix 4
needs it. `git diff --stat 6611e79` must show nothing else.

## Prove it can fail
Run each one at a time, rebuilt clean after each mutation, and record the result. Redirect output
and read `$?`.
- **The tighter bound bites.** Apply a mutation that drifts up to 0.6 but not 0.8. One option:
  round only the right edge's **half** case down, so 0.5 goes to 0 on that edge. Find one, show
  the test red at 0.4, and say whether it was green at 0.6. If no such mutation exists, say so
  with your reasoning. Then restore.
- **The A-table assert bites.** Comment out one A row. It must fail at the new count assert, not
  only at a type error. If the `[_; 4]` type makes that impossible, say so and show what fails
  instead. Restore.
- **M1 still goes red** at the new bound.
- Then run the whole crop test file and the docs test (`tests/docs_ops.rs`) green.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag. No `src/`
  change.
- Before pushing, run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`, and do
  the same with `--no-default-features`.
- **Never poll CI.** Run `gh pr checks 244 --watch` in the background. When it exits, read one
  direct snapshot at the PR's true head SHA, and say whether every leg is green.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect and read `$?`. zsh does not
  word-split unquoted parameters. `rtk` can truncate `grep`, so use `/usr/bin/grep` for anything
  you count.
- Reverting source doesn't rebuild the binary, so rebuild clean after each mutation. Never share a
  `CARGO_TARGET_DIR` with another session, and delete scratch target dirs when you're done.
- **Budget ~25 exchanges.**

## When you finish
- Add a `### Punch list (verify cleanups)` subsection to the spec's `## Build Completion`. List
  the four fixes, each control with its result, the two exit codes you drove, and the CI run id.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `grep -l "SPEC-145 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
    `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
  - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
    MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions`.
  - Also append the **verify** session's entry, which it measured but couldn't write (it was
    read-only): input 122, output 29,837, cache_creation 163,088, cache_read 8,432,984, total
    8,626,031, Opus, **$5.98**.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-145 punch list: PR #244 at <head SHA> — CI <green | red: legs>`.

Then, short:
- `git diff --stat 6611e79`;
- the bound mutation you used, and its result at 0.4 (and at 0.6);
- the A-table control's result;
- the two batch exit codes, and each doc line you changed, quoted;
- whether DEC-112 changed;
- the CI run id;
- anything that did not go as written;
- your measured cost.
