# 0.8.1 — RELEASE NOTES VERIFY prompt: draft PR #228

Cycle: **verify** (docs). Fresh CLI session, own worktree, **Opus**, **read-only**. You did not
write these docs.

```
just wt verify/release-0-8-1-notes -- --model opus
```

Then put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin docs/release-next-notes:refs/remotes/origin/docs/release-next-notes
git reset --hard origin/docs/release-next-notes
```

Draft PR #228's head was `fa02c7d` when this prompt was written. Record the SHA you verify. The
maintainer pasted this prompt into a fresh session and will paste your final message back to the
orchestrator, so it must stand on its own.

## Why this verify exists
These are the words users read for 0.8.1: the CHANGELOG (which also becomes the GitHub Release
body) and the agent skill. Documentation has no green, so a sentence is checked only when someone
runs the command it describes. The writing session drove most of its claims. Your job is to check
them **independently**, starting with the two places where its own report and its text disagree.

The maintainer ruled the version **0.8.1** (a patch, despite the `### Breaking` entry). Do not
reopen that.

## What the PR claims
Two commits, two files, docs only: `173aa50` (CHANGELOG) and `fa02c7d` (skill).
- **CHANGELOG `[Unreleased]`:**
  - a new Fixed entry: `optimize --ssim` / `--target` now search for AVIF output. It says 0.8.0,
    for "an AVIF output (`--format avif` or `-o x.avif`)", skipped the search, warned "no avif
    decoder is built", and wrote one file whatever the score. It gives sizes and scores for
    `--ssim 50` and `--ssim 90` on an 800×532 photo, a second bug (an AVIF graphic with `--max`
    and `--ssim` but no `--format` exited 1), and says the browser build still writes AVIF at a
    fixed quality;
  - SPEC-137's Fixed entry is narrowed to `font-family` naming "only families other than `Go`",
    adds "(Text with no `font-family` did draw.)", and says a 0.8.0 `build` cache "re-renders
    once" on the next `build` with this release.
- **`skills/crustyimg/SKILL.md`:**
  - `build` refuses the four output flags (exit 2), and a Project builds paragraph says where the
    manifest sets folder, name and format;
  - SVG needs the default-on `svg` feature; SVG `<text>` always draws in Go, upright and regular;
  - "`--target` and `--ssim` search for the lowest quality that meets the score on JPEG, AVIF and
    lossy WebP outputs", with a new example `optimize photo.jpg --ssim 80 -o photo.avif`;
  - a directory input reads only its top-level images ("a folder of only subfolders exits 3"),
    and the two `lint` examples now use `'assets/**/*'`.

## ⚠ Two things to settle first
1. **`-o x.avif` on 0.8.0.** The writer reported: "AVIF chosen by `-o x.avif` instead of
   `--format` behaves the same on both binaries." The CHANGELOG entry says the opposite: that
   `-o x.avif` also skipped the search on 0.8.0. Download or build the **released 0.8.0**
   binary (`v0.8.0`, fresh target dir) and the PR head. Run `optimize <photo> --ssim 50 -o a.avif`
   and `--ssim 90 -o b.avif` on each. Record sizes, exit codes and warnings. If `-o x.avif`
   already searched on 0.8.0, the entry's parenthesis is false, and the skill's new example
   shows a path that was never broken (that is fine for the skill; it is wrong for the
   CHANGELOG).
2. **"lossy WebP" in the skill.** `webp-lossy` is not a default feature, and the skill elsewhere
   says "WebP output is lossless only". On the default release binary, run
   `optimize <photo> --ssim 80 -o x.webp` and `--target high -o y.webp`. Report the exit code,
   any warning, and whether the output is VP8L (lossless) by a header read. If shipped binaries
   cannot search WebP, the sentence needs "(with the `webp-lossy` feature)" or must drop WebP.
   Check that the two skill sentences do not contradict each other.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits and no pushes. Do not fix what you find; report it. Do not
edit anything outside your scratch directory.

## Check, in order
Build a release binary of the PR head in a fresh `CARGO_TARGET_DIR`. The code equals `main`'s:
confirm `git diff origin/main...HEAD -- src/ Cargo.toml Cargo.lock` is empty. Make your inputs
with ImageMagick or Python, never with crustyimg.

1. **Scope.** Two files, docs only, every commit signed off. `[Unreleased]` is not rolled, the
   link references are untouched, no version was bumped, and no `0.8.1` string was added.
2. **The two items above.**
3. **The AVIF entry, the rest of it.**
   - `--ssim 50` and `--ssim 90` with `--format avif` on both binaries: same file on 0.8.0 with
     the warning, different files and scores on the PR. The quoted numbers are for the writer's
     photo; on yours, check the shape, not the bytes.
   - The second bug: an AVIF graphic input, `--max 128 --ssim 80`, no `--format`. Exit 1 with
     "The image format Avif is not supported" on 0.8.0, exit 0 on the PR. If you cannot
     reproduce it on 0.8.0, say what input you tried.
   - The browser claim: read `src/wasm.rs`'s optimize path and say what it does with an AVIF
     output and a score. If you cannot drive it, say so plainly.
4. **The SVG entry.**
   - On 0.8.0 and the PR: `font-family="sans-serif"`, `"Arial, Helvetica"`, and **no**
     `font-family` at all. Count non-background pixels. The parenthesis "(Text with no
     `font-family` did draw.)" must hold on 0.8.0.
   - **The cache clause.** Read where the crate version enters the build cache key and quote the
     line. Then prove a version change re-renders: a 0.8.0-written cache read by a scratch copy
     with a bumped version reports "0 cached" for the SVG target and draws text, and the next run
     is cached. Delete the scratch copy.
5. **The Breaking and Changed entries.** Spot-check, do not re-run the whole matrix:
   `build --format png`, `build -o x.png --watch` and `build --out-dir d --check` exit 2 and
   write nothing (snapshot before and after); a `--no-default-features` build exits 4 on an SVG
   with "rebuild with --features svg".
6. **The overclaim pass.** Grep `[Unreleased]` and the skill's diff yourself for `never`,
   `always`, `every`, `all`, `any`, `whatever`, `only`. For each hit, say whether it is true on the
   binary. In particular: "System fonts are still never read" (check with a font installed on
   this Mac, like Arial, against the Go render), and "every release with SVG input (0.5.0
   through 0.8.0)".
7. **The directory-input sentence.** On the PR binary and on 0.8.0: `lint d/` and
   `optimize d/` where `d/` holds only subfolders of images (exit 3?), and where `d/` holds a top-level
   image plus a subfolder (reads only the top-level one?). Then the new `lint 'assets/**/*'`
   examples, plain and `--format sarif`.
8. **Examples, re-run independently.** Run every command line the PR **adds or changes** in the
   skill, plus at least 10 others, each in a fresh directory. Record each exit code against what
   the skill says. Every flag the skill names must exist in that verb's `--help`.
9. **No spec or DEC ids** in the text this PR adds: grep the diff for `SPEC-` and `DEC-`.
10. **CI.** Run `gh pr checks 228` **once** at the head you verify. Do not poll.

## Guardrails
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters; loop with `while IFS= read -r`. macOS has no `timeout(1)`.
- If a binary you drive no longer exists, the shell exits **127**. That is not a result. Check
  each binary exists before you record its output.
- A target dir shared across worktrees serves a stale library: use a fresh one per binary.
- Two binaries that both report 0.8.0 can differ. Label every run with the binary's path.
- Keep scratch files in a scratch directory, never in `~`. Delete scratch target dirs at the end.
- **Budget ~60 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content:
  `grep -l "RELEASE NOTES VERIFY prompt: draft PR #228" ~/.claude/projects/<slug>/*.jsonl`. The
  transcript may store the em-dash escaped, so search for the text after it.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`): Opus $5/$25 per
  MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #228 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #228 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- the two items to settle first, each with the runs that settle it;
- items 1 and 3–10, one or two lines each with evidence;
- every false or unverifiable sentence, with the file, the line, what the binary does, and the
  exact replacement wording you would use;
- optional wording cleanups, listed separately;
- your cost.
