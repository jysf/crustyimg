# SPEC-143 — PUNCH LIST prompt: docs that match the refusal, and tests that can fail

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. Docs and tests only:
verify found no behaviour to fix. You did not design, build or verify SPEC-143.

```
just wt fix/spec-143-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-143-refuse-invisible-watermark:refs/remotes/origin/feat/spec-143-refuse-invisible-watermark
git reset --hard origin/feat/spec-143-refuse-invisible-watermark
```
You push to the PR branch with `git push origin HEAD:feat/spec-143-refuse-invisible-watermark`.
Draft PR #235's head was `c467490` when this was written. If it has moved, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own.

## Why
Verify drove every cause on the CLI, `apply`, `build` and `--watch`, plus the boundaries and 18
out-of-repo byte comparisons. The code is right on every path. Four things are not:
1. Two docs still promise the old behaviour, and two state a wrong order or exit code.
2. A recipe-side mutation survives: if `from_params` sets `max_alpha` to 65535, every test stays
   green. No recipe row refuses a low-alpha mark; FT-8's too-faint rows all use an opaque logo.
3. FT-6's golden can't see a small change on the 16-bit composite line. Multiplying the 16-bit
   opacity by 0.999 in the composite leaves everything green, because every 16-bit golden case
   runs at opacity 1 or fades to alpha16 = 1.
4. FT-11 and FT-12 failed on `main` because of the 5 % coverage cap, not because a mark drew
   nothing: FT-12's 64×48 base throws "the mark would cover 9.05 %", and FT-11's 64×64 inputs skip
   the mark at "6.79 %". The Build Completion says `transform` "returned Ok", which is false.

## The fixes (settled, do not re-decide)
1. **Docs** (no behaviour change):
   - `docs/cli-reference.md` (about `:289-290`) and `skills/crustyimg/SKILL.md` (about `:191`):
     delete the promise that a mark "is drawn as asked, even one too small or faint to see". Use
     the wording `recipes/README.md` now has as the model.
   - The same two places put the new refusal right after "exits 2 before any file is read". That
     holds only for **opacity 0** and **colour alpha `00`**. Say that blank text, a too-small
     size and a transparent image are refused after the `--font`/`--image` read and before any
     input is read (so `--text " " --font /no/f.ttf` exits 3).
   - `decisions/DEC-109-…md` (about `:58`) and `docs/api-contract.md` (about `:416`) say
     `--watch` exits 1. It reports both error lines and keeps watching. Say that.
   - `docs/cli-reference.md:283`: wrap the ~170-character line to the file's width.
2. **FT-8 gains two low-alpha recipe rows** (`tests/recipe_watermark.rs`):
   - `color = "ffffff01"` with `opacity = 0.0019` → the too-faint recipe line, exit 1;
   - `image = faint16` (the 16-bit alpha-4 fixture) with `opacity = 0.1` → the too-faint line,
     exit 1.
   Update the row-count assertion (9 → 11). Each row asserts no output and the input unchanged,
   as the others do.
3. **FT-6 gains a 16-bit mid-opacity case** (`tests/watermark_cli_params.rs`):
   `image_opacity_0_6_16bit`: `Gradient16(400,300)`, `--image logo.png --opacity 0.6` /
   recipe `opacity = 0.6`. The golden goes from 12 to 13 lines; update its count assertion.
   - **The new golden line must be `main`'s bytes, not the head's.** Bless it on the head, then
     prove it matches `main`: in a scratch worktree at `origin/main`, copy in the new
     `tests/watermark_cli_params.rs` and the 13-line golden, and run the test there (not the bless).
     It must pass. If it doesn't, stop and report: that would be a pixel change verify missed.
4. **FT-11 and FT-12 refuse on the right reason** (`tests/build_watermark_cache.rs`,
   `tests/wasm_roundtrip.rs`): enlarge the bases until the mark covers under 5 %, so that on
   `main` the mark is **drawn** (FT-11: exit 0 with output written; FT-12: `transform` returns
   `Ok`). Keep each test's refusal assertions as they are.
5. **Spec text** (the spec only; the timeline lives on `main` and is not yours):
   - Build Completion's FT-11 / FT-12 baseline lines: replace with what you measure in "Prove it
     can fail" below.
   - The mutation table: verify ruled all three build deviations to be spec errors. Fix the rows:
     M4a's red list adds FT-13; M5's red list adds FT-15 (it calls `apply`); AC-3 says `build`
     prints two lines (the target, then the step), as `docs/api-contract.md:547` already says.
   - The FT-8 description: 11 rows. The FT-6 description and Call 6: 13 cases.

Stay in these files: the four test files, the golden, `docs/cli-reference.md`,
`docs/api-contract.md`, `skills/crustyimg/SKILL.md`, DEC-109 and the SPEC-143 spec.
`git diff --stat` must show nothing else, and nothing in `src/`.

## Prove it can fail
Run each one at a time and record the result. For each mutation, check `git diff` is non-empty,
rebuild **clean** (a fresh `CARGO_TARGET_DIR`; reverting source does not rebuild the binary), run,
then restore and show green.
- **Item 2:** in `from_params`, set `max_alpha` to 65535. Both new FT-8 rows go red; before your
  change, nothing did.
- **Item 3:** in the 16-bit composite loop (not in `faded_alpha16`), multiply opacity by 0.999.
  FT-6 must go red **on a hash mismatch**, not on an exit code. Quote the failure.
- **Item 4:** run the enlarged FT-11 and FT-12 against `main`'s `src/` (a scratch worktree at
  `origin/main` with your test files copied in). FT-11 must fail on exit 0 with output written,
  and FT-12 on `Ok`, with **no** coverage message in either. Then on the head, apply M1 (drop the
  `invisible_mark()` call in `from_params`): both go red.
- **No regressions:** the full default leg, `--no-default-features` and `just wasm-test`, each in a
  fresh target dir, sequentially.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- Before pushing, run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.
- **Never poll CI.** Run `gh pr checks 235 --watch` in the background. When it exits, read one
  direct snapshot at the PR's true head SHA, every leg including windows-latest.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the pipe's exit code, so redirect and read `$?`.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`.
- Delete scratch target dirs and scratch worktrees when you're done.
- **Budget ~40 exchanges.**

## When you finish
- Add a short `### Punch list (verify)` subsection to the spec's `## Build Completion`: the four
  fixes, each "Prove it can fail" result with the quoted failure, and the matrix.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `grep -l "SPEC-143 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines
    that share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
    `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
    **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's
    output under-counts about 11×.
  - Price each component at the model that actually ran (read `.message.model`), never at a flat
    rate: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and
    cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions` with `tokens_total`, a
    `tokens_breakdown`, `estimated_usd` and the rate anchors.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-143 punch list: PR #235 at <head SHA> — CI <green | red: legs>`.

Then, short:
- `git diff --stat` against `c467490`;
- each "Prove it can fail" result, with the quoted failure;
- whether the 16-bit 0.6 golden line matched `main`;
- the matrix result, wasm included;
- anything that did not go as written;
- your measured cost.
