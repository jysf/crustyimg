# SPEC-160 — PUNCH LIST prompt: two comments and the saved --pad N sentence

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. Wording only: verify
found no behaviour or assertion to fix. You did not design, build or verify SPEC-160.

```
just wt fix/spec-160-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-160-edit-pad-trim:refs/remotes/origin/feat/spec-160-edit-pad-trim
git reset --hard origin/feat/spec-160-edit-pad-trim
```
Push to the PR branch with `git push origin HEAD:feat/spec-160-edit-pad-trim`. Draft PR #264's head
was `23a5c0e` when this was written. If it has moved, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

## Why
Verify drove 66 message rows, `parse_color` on seven routes, the pad fills with ImageMagick, the
nine gravities from its own model and six mutations, and found the code and tests sound. It
found three things to fix, all wording:

1. **A stale comment.** `src/cli/ops.rs:~353-356`, `run_pixel_op`'s forced-format check, says "The
   only caller whose pipeline can carry such a step forces its format; a caller that resolves the
   format per input must check there." That is no longer true: `edit --pad` carries an
   alpha-adding step without forcing a format, and `run_pixel_op` itself checks it further down
   (the gated one-input, batch-wide and kept-format checks).
2. **A DEC id in a comment.** `tests/edit_pad_trim.rs:~262`, in
   `replay_pad_and_trim_on_six_colour_types`: "… gains alpha for a translucent one (DEC-095)."
   No comment may name a spec, stage or DEC id.
3. **The saved `--pad N` replays as a fraction.** Verify saved `--pad 7` on 60×40 (`border =
   0.11666666666666667`) and replayed it on a 120×80 copy: a 14 px border, 148×108. The docs say
   the border is saved as a fraction of the long edge but don't say what that means on a
   different-sized image. The maintainer chose to say it. The sentences, found by content:
   - `docs/recipes.md:~221`: "… a border `--pad N` is saved as a fraction of that image's long edge."
   - `recipes/README.md:~54`: "… a pixel `--crop` or a border `--pad N` is saved as fractions of the
     image at that step."

## The fixes (settled, do not re-decide)
1. Rewrite comment 1 to say what the code now does: a forced format is the same for every input,
   so it is checked here once; an unforced format is checked below, once it is known, when a step
   adds alpha. Keep it short, and name no spec, stage or DEC.
2. Drop "(DEC-095)" from comment 2. Keep the rest of the sentence.
3. In each of the two doc sentences, add one clause that a replay scales the border with the
   image: for example, 7 px saved from a 60 px long edge replays as 14 px on a 120 px one. Use
   that file's voice, and keep it to one clause in each. If the sentence also covers `--crop`,
   the clause may cover both, because a pixel crop is saved as fractions too, but don't add a
   crop example.
4. No spec, stage or DEC id in any changed line.

Stay in these files: `src/cli/ops.rs` (that comment only), `tests/edit_pad_trim.rs` (that comment
only), `docs/recipes.md`, `recipes/README.md`, and the SPEC-160 spec (Build Completion and
`cost.sessions` only). `git diff --stat 23a5c0e` must show nothing else.

**Not yours:**
- DEC-115 and its replacement text, which ship applies on `main` after the merge;
- the design's mutation-table cells, which ship corrects;
- the CHANGELOG.

Don't edit them.

## Check it
On a release build of the PR (a fresh `CARGO_TARGET_DIR`, never a binary on `PATH`):
- Run `edit IN --pad 7 --pad-color 000000 -o e.png -y --save-recipe r.toml` on a 60×40 PNG. Then
  run `apply --recipe r.toml` on a 120×80 PNG. Quote the saved `border` line and the output size
  (148×108), read with ImageMagick. Your doc clause must match what you saw.
- Run `cargo test --test docs_ops --test edit_pad_trim`, with stdout redirected and `$?` read. It
  must be green.
- Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`. They must pass (the
  `ops.rs` edit is a comment, but check anyway).
- Run `/usr/bin/grep -n 'SPEC-\|STAGE-\|DEC-\|PROJ-'` on your changed lines. It must find nothing.
- There are no code changes, so no mutation runs and no matrix. CI runs on push.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- Rebase onto `main` only if the push is rejected. Never force-push over someone else's commit.
- **Never poll CI.** Run `gh pr checks 264 --watch` once in the background after your push, and
  don't write your own wait loop. When it exits, take one snapshot at the PR's head SHA, every leg.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect and read `$?`. zsh does not
  word-split unquoted parameters, and `"$R:docs/…"` needs braces (`${R}:`) or zsh reads `:d` as a
  modifier.
- Never share a `CARGO_TARGET_DIR` with another session, and delete yours when you're done. Write
  scratch files in your session's scratchpad, never `/tmp` or the system temp dir.
- **Budget ~20 exchanges.**

## When you finish
- Add a short `### Punch list (verify)` subsection to the spec's `## Build Completion`. It holds
  the two comments and two doc sentences as they now read, the replay drive with its quoted
  output, and the CI snapshot.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `/usr/bin/grep -l "SPEC-160 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
    `output_tokens`.
  - Price at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation
    is ×1.25 the input rate and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions` with `tokens_total`, a
    `tokens_breakdown`, `estimated_usd` and the rate anchors.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-160 punch list: PR #264 at <head SHA> — CI <green | red: legs>`.

Then, briefly:
- `git diff --stat 23a5c0e`;
- each changed comment and sentence, quoted as it now reads;
- the replay drive, with quoted output;
- anything that did not go as written;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
