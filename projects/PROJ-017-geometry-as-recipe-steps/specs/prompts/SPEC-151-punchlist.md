# SPEC-151 — PUNCH LIST prompt: a test that pins the `f64` border read

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. Tests and wording only:
verify found no behaviour to fix. You did not design, build or verify SPEC-151.

```
just wt fix/spec-151-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-151-pad:refs/remotes/origin/feat/spec-151-pad
git reset --hard origin/feat/spec-151-pad
```
You push to the PR branch with `git push origin HEAD:feat/spec-151-pad`. Draft PR #255's head was
`ef6befa` when this was written. If it has moved, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

## Why
Verify drove the canvas against its own rational oracle (332,800 exhaustive cases plus 200,000
random ones, 0 differ), every pixel against ImageMagick at AE 0, all 37 load messages and 5 apply
reasons by script, the metadata rule with `exiftool`, and five mutations. The code is right. Two
things are not:
1. **Nothing guards the `f64` border read DEC-112 requires.** Mutation M14
   (`get_f32("border").map(f64::from)` for `get_f64("border")`) leaves every test green. The spec's
   M14 cell says that's expected because no `f32` split exists in the load range. That is false:
   `border = 0.01` on a 50 px long edge is exactly ½ px, so `f64` gives 1 px and `f32` gives
   `0.49999999` → 0 px. Verify drove it: the real binary writes 52×12 from a 50×10 input; the M14
   mutant fails with the 0 px message and writes nothing. Other splits: `0.7` on 5 px (4 vs 3),
   `0.02` on 25 px (1 vs 0), `0.03` on 50 px (2 vs 1). The orchestrator re-checked all four in
   Python.
2. **Build Completion miscounts the diff.** It says "31 files, plus this spec file once committed =
   32". The PR is 31 files **including** the spec: the spec's 30 Outputs plus the spec.

## The fixes (settled, do not re-decide)
1. **Unit rows** in `border_is_a_fraction_of_the_long_edge` (`src/operation/pad.rs`, test module
   only): add two rows, each with its canvas and offset as literals:
   - `border = 0.01` on 50×10 → 52×12 at (1, 1);
   - `border = 0.7` on 5×3 → 13×11 at (4, 4).
   Compute each expected value yourself in Python with `f64` arithmetic (`round(b * L)`, halves
   up) before you write it, and quote it. Don't take the numbers above on trust.
2. **One binary row** in `tests/pad.rs`: `apply_reads_the_border_as_f64`. A seeded 50×10 rgb8 PNG
   (the file's own `seeded`, not the code under test), `border = 0.01`, `color = "#ffffff"`: exit
   0, output 52×12, the input at (1, 1) and the one-pixel frame all `#ffffff`. Same shape as
   `apply_pads_a_png_end_to_end`.
3. **Spec text** (the spec only; the timeline lives on `main` and isn't yours):
   - Build Completion's file line: "31 files, including this spec (the 30 Outputs plus the spec)".
   - The mutation table's **M14** row: the red set is now the two new unit rows and
     `apply_reads_the_border_as_f64`. Replace "expected all green" and its reasoning with one
     sentence: verify found `0.01` on a 50 px edge splits `f64` (1 px) from `f32` (0 px), so the
     read is now pinned. Don't edit any other row.
   - AC-5: append the two new rows to its list of cases. The Failing Tests line for
     `border_is_a_fraction_of_the_long_edge` says 7 rows, not 5. Add
     `apply_reads_the_border_as_f64` to the `tests/pad.rs` list.
   - The `orientation_of` caller count: verify confirmed `main` had 2 callers, so the build's
     count is right. Where the spec says three (find it by content), make it two.
4. **`docs/data-model.md:164`**: the line this PR reworded keeps `(SPEC-128)`. Remove the id and
   nothing else on that line (ruling 5: no doc line names a spec id). Leave the other `SPEC-128`
   mentions in that file alone; they're on lines this PR didn't touch.

Stay in these files: `src/operation/pad.rs` (its `#[cfg(test)]` module only), `tests/pad.rs`,
`docs/data-model.md` and the SPEC-151 spec. `git diff --stat ef6befa` must show nothing else, and
`git diff ef6befa -- src/` must touch only lines inside `mod tests`.

## Prove it can fail
For each mutation, check `git diff` is non-empty, rebuild **clean** (a fresh `CARGO_TARGET_DIR`;
reverting source does not rebuild the binary), run the full `cargo test --no-fail-fast` with stdout
redirected and `$?` read, then restore and show green.
- **M14** (`get_f32("border").map(f64::from)` in `Pad::from_params`): all three new tests go red,
  and nothing else. Quote each failure. Before your change, this mutation left the suite green.
- **M4** (long edge → short edge in `layout`): the two new unit rows go red too (50×10 at 0.01 on
  the short edge is 0.1 px → 0). Quote one failure.
- **No regressions:** the full default leg and `--no-default-features`, each in a fresh target dir,
  sequentially. No wasm run is needed: nothing wasm-facing changes.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag, no CHANGELOG
  edit.
- Before pushing, run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.
- **Never poll CI.** Run `gh pr checks 255 --watch` in the background, and don't write your own
  wait loop: the build's hand-written loop put the run id and SHA in one variable, zsh didn't split
  it, and it polled for 3.5 hours. When the watch exits, take one direct snapshot at the PR's true
  head SHA, every leg including windows-latest.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the pipe's exit code, so redirect and read `$?`.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`.
- Never share a `CARGO_TARGET_DIR` with another session; PROJ-011's SPEC-149 work may be running.
- Delete scratch target dirs and scratch worktrees when you're done.
- **Budget ~30 exchanges.**

## When you finish
- Add a short `### Punch list (verify)` subsection to the spec's `## Build Completion`: the four
  fixes, each "Prove it can fail" result with the quoted failure, and the matrix.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `/usr/bin/grep -l "SPEC-151 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
    `output_tokens`.
  - Price at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25; cache_creation is
    ×1.25 the input rate and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions` with `tokens_total`, a
    `tokens_breakdown`, `estimated_usd` and the rate anchors.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-151 punch list: PR #255 at <head SHA> — CI <green | red: legs>`.

Then, short:
- `git diff --stat ef6befa`;
- the two unit rows' expected values, with your Python check;
- each "Prove it can fail" result, with the quoted failure;
- the matrix result;
- anything that did not go as written;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
