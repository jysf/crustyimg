# 0.8.0 — RELEASE NOTES VERIFY prompt: draft PR #208

Cycle: **verify** (docs). Fresh CLI session, own worktree, **Opus**, **read-only**. You did not
write these docs.

```
just wt verify/release-0-8-0-notes -- --model opus
```

Then put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin && git reset --hard origin/docs/release-0-8-0-notes
```

Draft PR #208's head was `3db96e2` when this prompt was written. Record the SHA you verify. The
maintainer pasted this prompt into a fresh session and will paste your final message back to the
orchestrator, so it must stand on its own.

## Why this verify exists
These are the words users read for 0.8.0: the CHANGELOG (which also becomes the GitHub Release
body), the recipe guide, the agent skill and the README. Documentation has no green, so a sentence
is checked only when someone runs the command it describes. The writing session ran most of its
examples. Your job is to check them **independently**. Above all, check the claims it says it
did **not** drive, and the claims that are easy to get subtly wrong (precedence orders, cache
behaviour, ranges).

## What the PR claims
The five commits are `bcdccbb` (CHANGELOG), `acdac0d` (recipes), `02b40f5` (skill), `03c7d0b`
(RELEASING) and `3db96e2` (README and docs). They touch 7 files, docs only.
- **CHANGELOG `[Unreleased]`:**
  - a new `### Added` section with three entries: a recipe sets format and quality (SPEC-127); a
    recipe can watermark (SPEC-128); `build` rebuilds when a watermark's image or font changes
    (SPEC-129);
  - a new `Changed` entry: `apply --recipe` keeps the source format at every input count
    (SPEC-126);
  - SPEC-133's Fixed lead is narrowed to marks "too big for its image", and says tiny or faint
    marks (`--size 0.01`, `--opacity 0`) are still drawn as asked.
- **`recipes/README.md`:**
  - `version` "1"/"2", `format` (9 names) and `quality` in the file-format table;
  - an "Output format and quality" section with the precedence list and the v1 error;
  - a `watermark` row in the ops table, plus a section with every key, its mode and its range;
  - asset paths read from the **working directory**, not the recipe's folder;
  - the 5 % and "scaled mark does not fit" refusals.
- **`skills/crustyimg/SKILL.md`:**
  - the WebP batch caveat is removed, and "WebP output is lossless only" is kept;
  - the version is bumped to 0.8.x;
  - new: the output-format rule, the `meta` format refusal (exit 4), the watermark refusal rules,
    a v2 recipe example, load-time key checks (exit 1), `--json` refused when the format is
    pinned, and exit-code rows 1, 2 and 4;
  - one broken example fixed: `meta copy` now has `-o merged.jpg`.
- **`RELEASING.md`:** the stale batching note is replaced with a standing "diff `[Unreleased]`
  against the code commits since the last tag" rule.
- **`README.md`:** recipes mention watermark and format/quality; `--name-template`'s literal
  extension sets the format; the browser claim now says a recipe needing an image or font file is
  refused there. **That last one was read from `src/wasm.rs:233`, not driven.**
- **`docs/cli-reference.md`:** `--scale` is a fraction of the image's **width**, plus the `--size`
  range, the watermark refusals and `apply`'s format rule. **`docs/recipes.md`:** hand-written
  recipes list watermark and format.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits and no pushes. Do not fix what you find; report it. Do not
edit anything outside your scratch directory.

## Check, in order
Build a release binary of the PR head in a fresh `CARGO_TARGET_DIR`. The code equals `main`'s:
confirm `git diff origin/main...HEAD -- src/ Cargo.toml Cargo.lock` is empty. Make your inputs
with ImageMagick or Python, never with crustyimg.

1. **Scope.** The 7 files are docs only, and every commit is signed off. `[Unreleased]` is not
   rolled, the link references at the bottom are untouched, and no version was bumped.
2. **The claims that are easy to get subtly wrong.** Drive each one; reading is not enough.
   - **Format precedence:** `--format`, else the `-o` or template extension, else the recipe's
     `format`, else the source format. Run it on `apply --recipe` **and** `build`, at one input
     **and** at two inputs. Use one row per level where that level wins and the next one down
     differs.
   - **SPEC-129's cache claim:**
     - editing the watermark image rebuilds;
     - a recipe without a watermark keeps its cache key;
     - every cache rebuilds once on upgrade, because the key includes the crate version. Check
       this one by reading the key code, and say how you checked it.
   - **Asset paths come from the working directory, not the recipe's folder.** Check it on
     `apply --recipe` and on `build` with a manifest in a subfolder. If `build` resolves them
     another way, the guide is wrong.
   - **`--scale` is a fraction of the image's width** (`docs/cli-reference.md`). Measure the mark's
     width on two image widths.
   - **The browser claim** (README): a recipe that needs an image or font file is refused in the
     browser. Read `src/wasm.rs` in full around `:225-245` and say exactly what happens to such a
     recipe and with what message. If a wasm test exercises it, run that test. If you cannot
     drive it, say so plainly.
   - **The ranges** in the watermark key table match `WATERMARK_PARAMS` exactly: `size` finite,
     > 0 and ≤ 1024; `scale` finite and > 0; `opacity` 0–1; `margin`. Also check the `quality`
     range against what the binary does: `quality = 101` is accepted and treated as 100 today
     (filed for 0.8.1). The guide must not claim it is refused.
3. **The CHANGELOG, entry by entry.**
   - For each **new or edited** entry, run the command it describes and check the exit code and
     wording.
   - Then the overclaim pass. Grep `[Unreleased]` yourself for `never`, `always`, `every`,
     `invisible`, `no longer` and `all`. For each hit, ask whether it is true on the binary.
     0.8.0's ruling: no line may promise more than "a non-finite or oversized size or scale is
     refused". `--size 1`, `--size 0.01` and `--opacity 0` still exit 0.
   - Check SPEC-133's narrowed lead is true at both ends: a too-big mark is refused, and a tiny
     one is drawn.
4. **Examples, re-run independently.**
   - Run every TOML block in `recipes/README.md`, plus every TOML block and **at least 15**
     command lines in `SKILL.md`. Include the fixed `meta copy … -o merged.jpg`, the `--json`
     refusal, the exit-code rows 1, 2 and 4, and the v2 watermark recipe.
   - Record each exit code against what the doc says.
   - Every flag the skill names must exist in that verb's `--help`. Grep the skill's flags and
     diff them against `--help`.
5. **What was kept.**
   - "WebP output is lossless only" is still in the skill, and is true: `-q 40` to `.webp`
     writes VP8L. Check with `file` or a header read.
   - The removed WebP caveat really is fixed: a folder of `.webp` files exits 0.
6. **No spec or DEC ids** in the user-facing text this PR adds. Grep the diff for `SPEC-` and
   `DEC-`, excluding `RELEASING.md` if it legitimately keeps none.
7. **CI.** Run `gh pr checks 208` **once**. The feature-build legs were skipped. Confirm from
   `.github/workflows/` that a docs-only path filter is why, and say which file and which rule.
   Do not poll.

## Guardrails
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters; loop with `while IFS= read -r`. macOS has no `timeout(1)`.
- If a binary you drive no longer exists, the shell exits **127**. That is not a result. Check
  each binary exists before you record its output.
- Keep scratch files in a scratch directory, never in `~`. Delete your scratch target dir at the
  end.
- **Budget ~60 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content:
  `grep -l "0.8.0 — RELEASE NOTES VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`.
- Price each component at the model that actually ran (read `.message.model`): Opus $5/$25 per
  MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #208 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #208 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–7, one or two lines each with evidence;
- every false or unverifiable sentence, with the file, the line and what the binary does;
- optional wording cleanups, listed separately;
- your cost.
