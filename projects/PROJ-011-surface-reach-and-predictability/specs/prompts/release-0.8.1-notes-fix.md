# 0.8.1 — RELEASE NOTES PUNCH LIST prompt: fix on main after PR #228

Cycle: **release prep (punch list)**. Fresh CLI session, own worktree, **Sonnet**. Five text edits
in two files, with the wording settled. You did not write or verify these docs.

```
just wt fix/release-0-8-1-notes -- --model sonnet
```

**Revised 2026-10-30.** PR #228 was merged (`88d0afc`) before this punch list ran, so the false
text is on `main`. Work on a fresh branch off `main` and open a **new** PR:
```
git fetch origin && git reset --hard origin/main
```
Check that the two required sentences below are still on `main`. If either is already fixed,
skip it and say so.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own.

## Why
Verify drove every claim and found one false in two places. Release binaries are built with
default features, and `webp-lossy` is not one, so they write lossless WebP only.
`optimize --ssim 80 -o x.webp` and `--target high -o y.webp` both exit 0 with no warning and
write the same lossless (VP8L) file; the score is dropped silently
(`src/cli/optimize.rs:427-431`). Everything else held.

## The edits (wording settled; do not re-decide)
Line numbers are from `main` at `88d0afc`. Find each by its text.

**Required:**
1. `CHANGELOG.md` (about line 37), in the AVIF `--ssim`/`--target` entry:
   "It now searches as it does for JPEG and WebP, with no warning" →
   "It now searches as it does for JPEG, with no warning".
2. `skills/crustyimg/SKILL.md` (about lines 97-98):
   "`--target` and `--ssim` search for the lowest quality that meets the score on JPEG, AVIF and
   lossy WebP outputs." →
   "`--target` and `--ssim` search for the lowest quality that meets the score on JPEG and AVIF
   outputs. WebP output is lossless and ignores them, without a warning."

**Cleanups (also do these):**

3. `skills/crustyimg/SKILL.md` (about line 338): the sentence beginning "`-` reads from stdin;
   `-o -` writes to stdout…" got glued onto the end of the new SVG-text paragraph. Move it back to
   its own paragraph, where it stood before #228 (`git show 88d0afc^1:skills/crustyimg/SKILL.md`
   shows where), unchanged.
4. `CHANGELOG.md` (about line 41), the browser sentence in the same AVIF entry:
   "still writes AVIF at a fixed quality" → "still writes AVIF at a fixed quality when asked for
   a score". A byte budget still size-searches AVIF in the browser (`src/wasm.rs:535-545`).
5. `skills/crustyimg/SKILL.md` (about line 333): "only a source build with
   `--no-default-features` lacks it" → "only a source build with `--no-default-features`, and
   the slim browser build, lack it".

Change nothing else. `git diff --stat origin/main` must show only these two files.

## Check
- Grep both files for `WebP` and read every hit in the changed sections. No remaining sentence may
  say shipped binaries search WebP. "WebP output is lossless only" (about line 130) stays.
- On a release build of the head (fresh `CARGO_TARGET_DIR`, deleted at the end), run
  `optimize <photo> --ssim 80 -o x.webp` and check the output header reads `VP8L`, so the new skill
  sentence is true. Make the photo with ImageMagick, not crustyimg.
- No spec or DEC ids, and no version number, in the text you add.

## Land it
- One commit: `git commit -s -m "docs(release): WebP output is never quality-searched; tidy the skill"`,
  ending with `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. **The DCO sign-off is
  mandatory.**
- Push the branch and open a **draft** PR with `gh pr create --draft`, titled
  `docs(release): WebP output is never quality-searched; tidy the skill`, with the five edits in
  the body and the Claude Code footer.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- **Never poll CI.** Run `gh pr checks <PR> --watch` in the background. When it exits, read one
  snapshot at the head SHA.

## Guardrails
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~20 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content:
  `grep -l "RELEASE NOTES PUNCH LIST prompt: fix on main" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`): Sonnet $3/$15 per
  MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
First line: `0.8.1 notes punch list: draft PR #NNN at <head SHA> — CI <green | red: legs>`. Then, short:
- `git diff --stat origin/main`, and the five edits done (or skipped as already fixed);
- the `WebP` grep result and the VP8L header check;
- anything that did not go as written;
- your measured cost.
