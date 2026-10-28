# 0.8.0 — RELEASE NOTES FIX prompt: the verify punch list on draft PR #208

Cycle: **release prep, punch-list fix** (docs only). Fresh CLI session in the release-notes
worktree, **Sonnet**. The work is small and fully specified.

```
just wt docs/release-0-8-0-notes -- --model sonnet
```

`just wt` reuses the existing worktree. Run `git fetch origin && git status`. The branch must be at
`3db96e2`, matching `origin/docs/release-0-8-0-notes`, with a clean tree. If not, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## What verify found
Verify at `3db96e2` returned ⚠ PUNCH LIST with one defect. Every other claim held on the binary.

**`build` ignores `--format`.**
- `build --format png` exits 0 and writes the target's own format: GIF, WebP or JPEG, with or
  without a name extension or a recipe `format`.
- `--format` is a global flag, so `build --help` lists it, but `build` never reads it.
- `apply --recipe` does honour it.

**The maintainer ruled (2026-10-27): reword the docs for 0.8.0.** The code bug is filed for 0.8.1
and is not yours. So the docs must say what is true today:
- **on `apply --recipe`:** `--format` wins, then the `-o` or template extension, then the
  recipe's `format`, then the source format;
- **on `build`:** the target's name extension wins, then the recipe's `format`, then the source
  format. `--format` has no effect on `build`.

Do not promise a future fix in user-facing text.

## Do, in order
1. **Fix the five false lines.** Use the line numbers to find each place; the wording must follow
   the rule above.
   - `CHANGELOG.md:29`: "`apply --recipe` (at one input or many) and `build` both honour them …
     `--format` still wins". Scope `--format` to `apply --recipe`, and state `build`'s order.
   - `CHANGELOG.md:53`: "`--format` still wins over the extension". This sits in the
     `--name-template` Breaking entry, in the context of `build` targets. Make it true: `--format`
     wins on the verbs that take it, and does not apply to `build`.
   - `recipes/README.md:126-130`: "The flags still win, in this order: 1. `--format` …". Give
     `apply --recipe` and `build` their own order.
   - `skills/crustyimg/SKILL.md:58` (the general output-format rule, next to the `build` usage)
     and `:244`: say plainly that on `build` the target name or the recipe's `format` decides,
     and that `--format` does nothing there. An agent must not reach for it.
2. **Fold in the cheap cleanups verify listed:**
   - `CHANGELOG.md:36` quotes 0.7.1's error as `unknown field 'format'`. 0.7.1 prints
     ``unknown field `format`, expected one of …``. Quote it exactly; check the wording on
     the released 0.7.1 if it is installed (`brew list --versions crustyimg`), or at the `v0.7.1`
     tag's source.
   - `SKILL.md:86-87` show `optimize photo.jpg … -o photo.jpg`, which exits 5 without `-y`.
     Make the example run as written: add `-y`, or write to a different file, whichever suits the
     surrounding text.
3. **Drive every line you changed** on a release build of the PR head (fresh `CARGO_TARGET_DIR`,
   deleted at the end; inputs made with ImageMagick or Python, never crustyimg). Cover:
   - `apply --recipe` with `--format png` vs a `.gif` name: PNG wins;
   - `build` with `--format png`: ignored, so the target's format is written;
   - `build` with a `.gif` name vs a recipe `format = "webp"`: GIF wins;
   - `build` with no name extension and a recipe `format`: the recipe's format;
   - the two `optimize` examples, run exactly as written: exit 0.

   Record each command and exit code. Then grep all 7 files this PR touches for `--format`, and
   check that no other line claims `build` honours it.
4. **Commit and push to the PR branch:** `git commit -s`, then
   `git push origin HEAD:docs/release-0-8-0-notes`. Run `gh pr checks 208 --watch` **in the
   background**. When it exits, read one snapshot at the head SHA. Never poll.

## Guardrails
- ⛔ Docs only. No `src/` or test changes, no version bump, no tag. The PR stays a **draft**: no
  `gh pr ready`, no merge.
- No spec or DEC ids in user-facing text.
- Do not edit the backlog. The `build --format` bug and the wasm message's spec id are already
  filed on STAGE-054.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`. If a binary you drive no longer
  exists, the shell exits 127; that is not a result.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~35 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content:
  `grep -l "0.8.0 — RELEASE NOTES FIX prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Take input, cache_creation and cache_read once per id, and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`): Sonnet $3/$15 per
  MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line: `0.8.0 notes fix: PR #208 at <head SHA> — CI <green | red: legs>`. Then, short:
- each changed line, as before → after (one line each);
- every driven command and its exit code;
- the `--format` grep across the 7 files: the hits and why each is true;
- your cost.
