# SPEC-153 — PUNCH LIST prompt: two doc sentences about saved `edit` recipes

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. Wording only: verify
found no code or test to fix. You did not design, build or verify SPEC-153.

```
just wt fix/spec-153-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-153-edit-geometry-flags:refs/remotes/origin/feat/spec-153-edit-geometry-flags
git reset --hard origin/feat/spec-153-edit-geometry-flags
```
You push to the PR branch with `git push origin HEAD:feat/spec-153-edit-geometry-flags`. Draft PR
#261's head was `d917d60` when this was written. If it has moved, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

## Why
Verify drove every message by script (P1–P16, A1–A3, byte for byte), typed order and pixel crops
against ImageMagick at AE 0, the byte pin with its own Python decoder and hash, replay on four
colour types and two tagged inputs, the batch rules, and six mutations. The code and tests hold.
Two user-facing sentences don't:
1. **`docs/recipes.md:209-210`** says a pixel `--crop` "is saved as fractions of the image the step
   received, so the recipe replays on any size." That's false. Verify drove it: `--crop 1x1+0+0`
   saved from a 200×10 image, then `apply` on a 60×40 image, exits 1 with `the crop rounds to 0×4
   px`. On another image the recipe crops the same **proportions**, not the same pixels, and it
   fails if a side rounds to 0 px.
2. **`docs/recipes.md:208-209` and `recipes/README.md:52-53`** list `--auto-orient` among the ops
   recorded "in the order you typed them". It isn't: the saved recipe always starts with an
   `auto-orient` step, wherever `--auto-orient` is typed, and whether or not it is typed (DEC-114
   part 2).

## The fixes (settled, do not re-decide)
1. **`docs/recipes.md`**, the paragraph at about :208 (find it by content). Say:
   - the saved recipe starts with an `auto-orient` step, always, wherever `--auto-orient` is typed;
   - then the op flags it exposes (`--resize-max`, `--invert`, `--crop`, `--rotate`, `--flip`), in
     the order you typed them;
   - a pixel `--crop WxH+X+Y` is saved as fractions of the image that step received, so on another
     image it crops the same proportions, not the same pixels, and fails if a side rounds to 0 px.
   Keep it as short as the current text. No spec, stage or DEC id.
2. **`recipes/README.md`**, the sentence at about :52. Same two points about order and
   `auto-orient`, in that file's shorter voice. If it keeps the "saved as fractions" clause, add
   "of the image at that step" so it doesn't imply pixels carry over.
3. **Build Completion** (the spec on the PR branch; the timeline lives on `main` and isn't yours).
   Correct two lines verify found wrong, each marked `(corrected at punch list)`:
   - M2's red set gains **FT-8** (verify's run: 15 red, FT-1 by hash, FT-5, 6, 8, 9, 10, 11, 12, 14,
     24, 25, 30, FT-31, `edit_no_ops_exits_2`, `stub_command_returns_not_implemented`). Add one
     line: `stub_command_returns_not_implemented` runs `edit in.png` with no op flag, so it is an
     `edit` test; the order is collected only for `edit`.
   - Trap 14: "the unit tests don't compile" is wrong. With `main`'s two files restored verbatim,
     the `src/cli/` unit tests are `main`'s own, and they compile and pass.
   Don't edit the design's mutation table; ship corrects those cells.

Stay in these files: `docs/recipes.md`, `recipes/README.md` and the SPEC-153 spec.
`git diff --stat d917d60` must show nothing else.

## Check it
- Drive fix 1's claim yourself on a release build of the PR (a fresh `CARGO_TARGET_DIR`):
  `edit` a 200×10 PNG with `--crop 1x1+0+0 --save-recipe r.toml`, then `apply r.toml` to a 60×40
  PNG. Quote the exit code and message, and check your new sentence says that.
- Then save `--rotate 90 --auto-orient` and read the TOML: `auto-orient` is the first step. Quote it.
- `cargo test --test docs_ops` green (it resolves doc examples), stdout redirected, `$?` read.
- `/usr/bin/grep -n 'SPEC-\|STAGE-\|DEC-\|PROJ-'` on your changed lines: none.
- No code changes, so no mutation runs and no matrix. CI runs on push.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag, no CHANGELOG
  edit.
- Rebase onto `main` only if the push is rejected; never force-push over someone else's commit.
- **Never poll CI.** Run `gh pr checks 261 --watch` once in the background after your push, and
  don't write your own wait loop. When it exits, take one snapshot at the PR's head SHA, every leg.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect and read `$?`. zsh does not
  word-split unquoted parameters.
- Never share a `CARGO_TARGET_DIR` with another session; delete yours when you're done. Write
  scratch files in your session's scratchpad, never `/tmp`.
- **Budget ~20 exchanges.**

## When you finish
- Add a short `### Punch list (verify)` subsection to the spec's `## Build Completion`: the three
  fixes, the two drives with their quoted output, and the CI snapshot.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `/usr/bin/grep -l "SPEC-153 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
    `output_tokens`.
  - Price at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25; cache_creation is
    ×1.25 the input rate and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions` with `tokens_total`, a
    `tokens_breakdown`, `estimated_usd` and the rate anchors.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-153 punch list: PR #261 at <head SHA> — CI <green | red: legs>`.

Then, short:
- `git diff --stat d917d60`;
- each changed doc sentence, quoted as it now reads;
- the two drives, with quoted output;
- the two Build Completion corrections;
- anything that did not go as written;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
