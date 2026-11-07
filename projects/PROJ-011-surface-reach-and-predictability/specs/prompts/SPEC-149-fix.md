# SPEC-149 — FIX prompt: the `--keep-gps` refusal must not promise PNG

Cycle: **build (fix round)**. Fresh CLI session, **Sonnet**, on the existing PR branch. Verify (Opus) returned a PUNCH
LIST at `e53e9f6`: the 135-row matrix, `-v`/`-vv`/`-Q`, the lean build, the font and overlay errors, 20 mutations and
every CI leg all passed. One required defect, R-1.

```
just wt fix/spec-149 -- --model sonnet
```

Then, in the worktree: `git fetch origin && git checkout -B fix/spec-149 origin/feat/spec-149-no-flag-accepted-and-ignored`.
You **push to that branch**: `git push origin HEAD:feat/spec-149-no-flag-accepted-and-ignored`. The PR (#256) stays a
**draft**; no `gh pr ready`, no merge, no tags. Use your own `CARGO_TARGET_DIR`. Every commit `git commit -s`, ending
`Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.

⚠ PROJ-017 owns `src/operation/`, `src/recipe/`, `docs/recipes.md`, `docs/api-contract.md`, `docs/data-model.md`,
`CHANGELOG.md`, `skills/crustyimg/SKILL.md`. Stay out of them (their text is fixed on `main` by the orchestrator).

## R-1 — the workaround the message names works on JPEG only (required)
The `--keep-gps` refusal (R6) says the user can run `crustyimg meta copy --from <in> --to <out> -y` "(JPEG and PNG)".
Verify drove it with exiftool 13.55: JPEG to JPEG restores the GPS (`50 deg 29' 17.16" N, 30 deg 30' 0.00" E`); PNG to
PNG exits 4 `metadata lane does not support meta copy supports JPEG only in v1 yet`; JPEG to PNG and PNG to JPEG exit 4;
WebP exits 4. `meta copy` is JPEG-only (`src/metadata/mod.rs:305`, DEC-030). The wording came from the spec's own
sweep table, from a frame claim nobody tried on a PNG.

Ruling (orchestrator): say **"(JPEG only)"**. Do not drop PNG some other way, do not reword the rest of the message,
and do not touch `meta copy`.

Change `(JPEG and PNG)` to `(JPEG only)` in exactly these places:
1. `R6_KEEP_GPS` in `src/cli/global_flags.rs`. The phrase is split across a line break, so `grep` misses it: read the
   constant, and join the lines in your head before searching. Keep every other word, including "pixel-lane".
2. The R6 text copied into `tests/cli_flags_honest.rs`.
3. `README.md:199`.
4. `docs/cli-reference.md:21` and `:42`.
5. `docs/moat.md:96`.

Then find the rest yourself. Do not trust this list: run `grep -rnE "JPEG (and|or|/) ?PNG|JPEG/PNG|JPEG, PNG" README.md
docs src tests SECURITY.md AGENTS.md skills integrations` and also search for the split-line form
(`grep -rn -B1 -A1 "PNG)" src tests`). For every hit that is about **`meta copy`** or the **`--keep-gps` workaround**,
fix it. Hits about other things (for example `meta clean` or `meta strip`, which really do handle PNG; check
`src/metadata/mod.rs`) stay as they are. List every hit with a keep/fix verdict and the reason in your report.

## Order (trap 13: RED first, for its reason)
1. In `tests/cli_flags_honest.rs`, the R6 text is a literal copy. Change **only** the test's copy to `(JPEG only)`.
   Run `cargo test --test cli_flags_honest`: it must go RED on the `--keep-gps` rows (message mismatch, code still says
   "JPEG and PNG"). Quote one failing line. This also proves the test compares the whole message.
2. Change `R6_KEEP_GPS`. The test goes GREEN.
3. Add one **truth** check so the claim is tested, not just worded: a test that builds a JPEG, runs
   `crustyimg resize … -o out.jpg -y`, then the exact `meta copy --from … --to … -y` command taken from the R6 text
   (parse it out of the constant, do not retype it), and asserts exit 0 and that the output now has the source's GPS
   (read it back with the repo's existing metadata test helpers, or with `exiftool` only if the test skips with a
   loud message and **fails** where `CI` is set, trap 11; prefer the repo's helpers). Also assert PNG to PNG exits 4,
   so a future lane that gains PNG makes this test say the message is now stale. Quote RED/GREEN for it by running it
   once against a deliberately wrong command (swap `--from`/`--to` in a scratch copy), then restore exactly (trap 14).
4. Docs: apply the other edits. `git diff --stat` per file.
5. Gates: `just check`, `cargo clippy --all-targets -- -D warnings` (default and `--no-default-features`),
   `cargo fmt --check`.
6. Push. Background `gh pr checks 256 --watch`; when it exits read one direct snapshot at the true head SHA, every leg
   including windows-latest. Never poll.

## Not in this round (rule 7)
Verify's optional items stay on the list for ship or SPEC-150: a `"` in a path printed unescaped; FT-1's `meta copy`
row copies PNG to PNG (your new test covers the JPEG path); the garbled `metadata lane does not support meta copy
supports JPEG only in v1 yet` message; `--keep-gps=false` getting clap's error. Do not fix them. If you see
something else, report it.

## Guardrails
- ⛔ Nothing publishes or tags. A piped command reports the pipe's exit code: redirect and read `$?`. zsh does not
  word-split unquoted parameters. Scratch files in a scratch directory, never in `~`.
- Files you may touch: `src/cli/global_flags.rs`, `tests/cli_flags_honest.rs` (or a new `tests/keep_gps_workaround.rs`),
  `README.md`, `docs/cli-reference.md`, `docs/moat.md`, `SECURITY.md`, `AGENTS.md`, and the SPEC-149 spec's Build
  Completion (addendum only). Anything else is a deviation; say why.
- **Budget ~25 exchanges.**

## Measure your own cost
Transcript JSONL under `~/.claude/projects/<slug>/` (absolute working directory with every `/`, `.`, `_` replaced by
`-`); identify **by content**: search `FIX prompt: the --keep-gps refusal must not promise PNG`. Dedup by
`.message.id` (input, cache_creation, cache_read once per id, **MAX** `output_tokens`); Sonnet $3/$15, cache_creation
×1.25, cache_read ×0.10.

## Report back (your final message)
First line: `SPEC-149 fix: PR #256 at <head SHA> — CI <green | red: legs>`. Then: the RED line and the GREEN line;
the truth test and its negative control; the full grep hit list with verdicts; `git diff --stat` for the round; gates;
the true-head CI snapshot (every leg); anything you found out of scope; measured cost.
