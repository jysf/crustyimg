# 0.8.0 — RESVG 0.49 FIX prompt: the verify punch list on draft PR #211

Cycle: **punch-list fix** (CHANGELOG only). Fresh CLI session, **Sonnet**. The work is two edits
to one file, and both are fully specified below.

```
just wt chore/resvg-0.49 -- --model sonnet
```

`just wt` reuses the branch's worktree if it exists. Run `git fetch origin && git status`. The
branch must be at `5dea0c2`, matching `origin/chore/resvg-0.49`, with a clean tree. If not, stop and
report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## What verify found
Verify at `5dea0c2` returned ⚠ PUNCH LIST, for the CHANGELOG wording only. The code, lockfile,
advisories, SVG guards and every CI leg passed. It drove each claim on a BASE build (resvg 0.47) and
a HEAD build (resvg 0.49):
- **The panic named the wrong bug.** "A panic when a filter region is smaller than its input" is a
  panic resvg introduced in 0.48, so 0.7.1 never had it. The part of that fix that does matter to
  us is **clipping** when a filter region is smaller than its input. What 0.49 does fix is a
  **crash**: an `feComposite` filter with `operator="arithmetic"` and an oversized filter region
  panics on 0.7.1 (exit 101, no file written) and renders on 0.49.
- **"Text may render with small differences from 0.7.1" is inaccurate.** Plain text was
  byte-identical in every test. The differences are 15–20 px corrections, and they appear only
  where one of the listed fixes applies.
- Every other claim held: filters in rotated or skewed groups, gradients inherited through `href`,
  nested `<tspan>` baselines, `text-anchor` with `dx`, and the sizes (+440,928 B native, +21,212 B
  wasm brotli).

## Do, in order
1. **Replace the entry.** In `CHANGELOG.md`, under `## [0.8.0]` → `### Changed`, replace the whole
   `- **SVG rendering updated to resvg 0.49.** …` bullet with exactly this text (wrap at the file's
   width, about 96 columns):

   ```markdown
   - **SVG rendering updated to resvg 0.49.** Text in SVGs is now laid out by the maintained
     `skrifa` + `harfrust` font stack instead of `ttf-parser` + `rustybuzz`, which are no longer
     maintained and are now gone from the dependency tree, along with the two advisory
     exceptions they needed. It also picks up resvg's SVG fixes. A hostile SVG with an
     `feComposite` filter (`operator="arithmetic"`) whose region extends past the image no
     longer crashes `crustyimg`; 0.7.1 panicked and wrote no file. Also fixed: filters inside
     rotated or skewed groups, clipping when a filter region is smaller than its input, gradients
     and patterns that inherit `gradientTransform` or `patternTransform` through `href`,
     `text-anchor` combined with `dx`, and nested `<tspan>` baselines under
     `dominant-baseline`. SVGs that use these features render differently from 0.7.1, now as the
     SVG specification describes. The native binary grows by about 440 KB and the WebAssembly
     module by about 21 KB compressed.
   ```

   Do not edit any other entry.
2. **Set the release date.** Change `## [0.8.0] - 2026-10-27` to today's local date (`date +%F`).
   The maintainer tags after this merges. If the tag lands on a later day, the orchestrator will
   correct the date.
3. **Check it.**
   - `git diff origin/chore/resvg-0.49 -- CHANGELOG.md` shows only the heading date and that one
     bullet.
   - `grep -n 'SPEC-\|DEC-' CHANGELOG.md` finds nothing in the new bullet.
   - Run `just release 0.8.0` (read-only checks). It must still print
     `✓ Cargo.toml + Cargo.lock == 0.8.0` and `✓ CHANGELOG has a '## [0.8.0]' section`.
4. **Commit and push to the PR branch.**
   - `git commit -s -m "docs(changelog): correct the resvg 0.49 entry; set the 0.8.0 date"`, with
     the line `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>` at the end of the
     message.
   - `git push origin HEAD:chore/resvg-0.49`.
   - Run `gh pr checks 211 --watch` **in the background**. When it exits, read one snapshot at the
     head SHA and check every leg. Never poll.

## Guardrails
- ⛔ `CHANGELOG.md` only. No `src/`, `Cargo.*` or `deny.toml` change, and no tag. The PR stays a
  **draft**: no `gh pr ready`, no merge.
- A piped command reports the pipe's exit code, so redirect output and read `$?`.
- **Budget ~15 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content:
  `grep -l "0.8.0 — RESVG 0.49 FIX prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Take input, cache_creation and cache_read once per id, and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`): Sonnet $3/$15 per
  MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line: `resvg fix: PR #211 at <head SHA> — CI <green | red: legs>`. Then, short, **with the
evidence inline**:
- the full `git diff` of `CHANGELOG.md`;
- the `just release 0.8.0` lines;
- the CI legs at the head SHA;
- your cost.
