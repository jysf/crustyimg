# SPEC-165 — BUILD prompt: the engine decides whether the page's watermark fits

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**.

This build has a little Rust and wasm, and the rest is the page and its two harnesses:
- **The engine (`src/operation/mod.rs`, Rust).** Move the body of `Watermark::refusal`, unchanged,
  into a new `pub(crate) refusal_at(w, h)`. `refusal` then calls it. Add one unit test.
  **No rule changes.**
- **The wasm export (`src/wasm.rs`).** Add one export, `watermarkRefusal`, with rows E1–E5 in
  `tests/wasm_roundtrip.rs`.
- **`recipe.js`.**
  - Add `FIT_HEADROOM` (2) and `fitWatermark`.
  - `planEdit` takes a `refuses` callback and returns `blocked`.
  - `pixelRecipe` throws on a blocked edit.
- **The page.**
  - `worker.js` plans the edit through a new `plan` message.
  - `edit.js` handles a blocked plan.
  - `index.html` gets the new `#wm-capped` text, plus `#make-blocked` and Make's `aria-describedby`.
- **The harnesses.**
  - The parity harness gets fit rows F1–F15 and replay rows R18–R20 (`REPLAY_ROW_COUNT` 17 → 20).
    `ctx.refuses` is backed by the real wasm. `checkModules` gets new lines, and N5 its new expected
    set.
  - The page check gets a third photo and rows H9–H11 (8 → 11).
- **A new fixture** `tests/fixtures/ddphotos/photo_thin.webp` (64×1600), plus its README line.
- **Docs.** Change the README's Watermark row and write DEC-119.

The design drove every row, message and mutant on a prototype. Most of this build is typing its
literals in exactly. Be thorough across every row: Sonnet's known weak spot is sweep thoroughness.

```
just wt feat/spec-165-watermark-fit -- --model sonnet
```

You didn't design this. The spec carries the context (AGENTS §15). The maintainer pastes this prompt
into a fresh session, then pastes your final message back to the orchestrator, so that message must
make sense on its own. **Write it as plain lists, not box-drawn tables.**

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check that:
- the spec is at `cycle: build`;
- design commit `258e49f` is in your history;
- the spec has `### Maintainer rulings (2026-11-28)` and `## Design`;
- `decisions/DEC-119-*` and `tests/fixtures/ddphotos/photo_thin.webp` don't exist yet.

If any check fails, stop and report.

Then run `gh pr list --state open --json number,title,files` and `git branch -r`, and compare them
with the spec's overlap map. Parked PR #237 shares no file with this spec. Don't touch it. Redo
the check after every rebase. If a rebase conflicts, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-165-the-watermark-s-fit-on-a-small-or-thin-crop.md`.
   Read it in full:
   - `## Frame` and its rulings;
   - `## Design`: the corrections, the rule, the blocked state, the plan's shape, the export, the
     sentences, every row family, the texts, the mutation table, "What pins nothing else changes",
     and DEC-119;
   - the Outputs, all 14 ACs, every Failing Test, and `## Notes for the Implementer`.

   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in your
   report. Don't quietly re-decide it.
2. **DECs:** DEC-045, DEC-104, DEC-110 and DEC-118 (rules 1, 2 and 5).
3. **The engine:** `Watermark::from_params` and `refusal` in `src/operation/mod.rs`, plus
   `assert_refuses`. Then `src/wasm.rs` (its exports' conventions) and `tests/wasm_roundtrip.rs`.
4. **The page code in full:** `integrations/ddphotos/edit/recipe.js`, `edit.js`, `worker.js` and
   `index.html`.
5. **The harnesses:**
   - `tests/ddphotos_parity.mjs`: `checkModules`, `loadWasm`, `recipeFor`, `REPLAY_ROWS`,
     `replayRows`, N5 and the summary line;
   - `tests/ddphotos_page.mjs`: `buildSite`, `open` and `waitFor`;
   - `tests/ddphotos_upscale.py` and `scripts/serve.mjs`;
   - the `ddphotos-parity`, `ddphotos-page` and `wasm-*` recipes in the `justfile`;
   - the `ddphotos-parity` and `wasm-roundtrip` jobs in `.github/workflows/ci.yml`.
6. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for".
   Traps 11–14 and 19–22 bind this build.

## Maintainer rulings (settled — do not re-open)
1. **When no size fits, block Make.** Never leave the mark off.
   - If the watermark is on, every copy and recipe the page produces carries it.
   - The page renders nothing unmarked: while blocked, the preview is the published photo.
   - The message names the three ways out: make the crop or the size larger, use shorter text, or
     turn the watermark off.
2. **The engine decides through a wasm export.**
   - Nothing in JS restates the 5 % cap, the fit check, the margin rule or glyph widths.
   - If you find yourself writing `0.05`, a margin comparison or an em width for the new rule, stop.
     That is the rejected option (b).
3. **One spec.**
4. **No readable floor.**
5. **Out of scope:** `MIN_CROP` stays 64, there's no crop DOM, no slim-bundle gate and no change to
   any engine rule.

No comment in the code, the harnesses or the fixture README names a spec, stage, DEC or project id.
Only DEC-119's own file names DECs.

## ⚠ Temp folders and large files — set this up before your first run
- **Before the first run,** count the system temp folder's `ddphotos-*` folders and their size, with
  `/bin/ls -d "$(getconf DARWIN_USER_TEMP_DIR)"ddphotos-*`. Delete none of them: other sessions own
  some. Design counted 0.
- **Run every harness and page-check command with `TMPDIR` set to a folder in your scratchpad.**
  Create the folder first. After the first run of each command, confirm its folders landed there. If
  they didn't, stop.
- **Empty that folder after you've read each red run.** If you keep a folder for debugging, name it
  in Build Completion.
- **Never run a harness while you're editing files it reads.**
- **Large originals** (6000×4016 and 4016×6000), the Pillow venv and your `CARGO_TARGET_DIR` live in
  the scratchpad only. Measure them and delete them at the end.
- **At the end,** count the system `ddphotos-*` again and report both counts. If the count grew, say
  which run leaked.

## Order, with a pushed checkpoint after each stage
The spec's four checkpoints come after a red baseline. Commit with `git commit -s` and push after
each one: a long build with no pushed commit has been lost before. **If a push is refused, stop and
report the command. Check `git ls-remote` before saying anything landed** (trap 21).

0. **The failing tests first.**
   - Add the unit test, E1–E5, F1–F15, R18–R20, the `checkModules` lines, the
     `planEdit without the engine's check` row, N5's new set, the fixture and H9–H11.
   - Run them against `main`'s code and quote the RED baseline, one line per family. It's red
     because `refusal_at`, `watermarkRefusal`, `fitWatermark` and `#make-blocked` don't exist yet.
     Say if anything else is red, or if a row expected red passes.
   - Open the PR **as a draft** with this commit.
1. **The export.**
   - Add `refusal_at`, with `refusal` delegating to it, and the unit test.
   - Add `watermarkRefusal`. Pass `size` as a float, and return `Option<String>`, which is
     `undefined` in JS.
   - Run `cargo test --lib`, `just wasm-test`, `just wasm-test-slim` and `just wasm-clippy`. Also run a
     lean `--no-default-features` build and clippy: `refusal_at` must not be dead code there.
   - Paste `git diff origin/main -- src/operation/mod.rs` into Build Completion. It must show the body
     moved unchanged, with no rule edited (AC-2).
2. **`recipe.js` and the parity harness.** `just ddphotos-parity` must go green in both modes. Quote
   the summary line, which gains `, 15 fit rows` and reads `20` replay rows.
3. **The page.** Change `worker.js`, `edit.js` and `index.html`, then make `just ddphotos-page` green
   with `11 rows, 0 failed`.
   - Then run the check **once** against a scratch copy of `scripts/serve.mjs` that delays only the
     HTML body: call `res.flushHeaders()`, then pipe the file after 1,500 ms.
   - **Prove the delay applied:** a `fetch` sees the headers in under 50 ms and the body after about
     1,500 ms. Quote both times. Don't commit the switch.
4. **The docs, DEC-119 and the size.**
   - Write the README's Watermark row and the `recipe.js` doc comments exactly as the spec quotes them.
   - Write DEC-119 with the spec's content and `affected_scope`.
   - Record the `just wasm-build-slim` brotli size: the Mac figure (design measured 834,585 B), and
     CI's figure from the log.

Then **the mutation table, re-driven** (traps 12, 13 and 14):
- Run it in a scratch `git worktree` of your pushed head, never in the build worktree. Give it its
  own `CARGO_TARGET_DIR` and the scratch `TMPDIR`. Build the slim wasm and the binary once, and reuse
  them for every JS mutant.
- **M11 is a Rust mutant** (the export accepts everything). Rebuild the wasm for it. Also run E1–E5
  under `cargo`/`just wasm-test` against it. Design didn't drive that.
- For each of M1–M7, M7b, M7c, M7d and M9–M11:
  - apply **the spec's quoted edit**, not your own version. If the quoted line doesn't exist in your
    code, make the closest edit, quote it, and say why;
  - confirm the edit is live: `git diff` is non-empty, and the run reads the mutated file;
  - run the full harness or check with its output redirected, and read `$?`;
  - read the red set **off that run** and compare it with the spec's. Quote the first failure line.
    Call out every row red that the spec doesn't list, and any row that's red for a different reason,
    such as a throw where the spec expects a value (trap 13).
- Restore after each one and show it green. Empty the scratch `TMPDIR` between rows.
- **M6 has history.** The frame expected the golden rows to catch it, but design found that C29, C30,
  R14, R15, F4, F7 and F15 do. Confirm that on your rows.

Paste each red set into Build Completion, trimmed to the lines that name rows. Where the spec's table
is wrong, correct it in Build Completion and say so. Don't edit the design's table.

## The things most likely to go wrong
- **Expected values from the code under test.** Both sides of the parity harness read `recipe.js`,
  so a mutant there agrees with itself. Every expected size, side and message is the spec's literal
  (trap 19). Only the replay rows' literals catch the shared mutant M1.
- **A JS copy of the rule.** `ctx.refuses` in the harness is the real slim wasm's `watermarkRefusal`.
  It's never a function you write.
- **The headroom check on both sizes.** Each size must be accepted on (outW, outH) **and** on
  (max(1, outW − 2), max(1, outH − 2)). Descend from `watermarkSize`'s ceiling, never from `wanted`.
- **Thin strips and the square in both orders** (trap 20). Type every one of F1–F15 in.
- **Make fails closed.** `makeCopy` plans again and returns when the plan is blocked. A stale enabled
  button must not make a copy.
- **The preview while blocked** is the published photo, and no render is asked for. Showing the edit
  without the mark would be an unmarked copy the page made, which ruling 1 forbids.
- **`checkModules`' line match.** Pass the export itself as `refuses`, not a wrapper, so the
  `worker.js` line in the spec matches exactly.
- **The fixture.** Make it with Pillow 12.3.0 from the spec's pixel formula, `lossless=True`, in a venv
  in the scratchpad. Record the generator in the fixture README. Check its dimensions natively.
- **Trap 22.** H9–H11 navigate the way H1–H8 do, through `open()`, which waits for the new document.

## Guardrails
- **What must not change:**
  - every engine rule, so `refusal_at`'s body is byte-identical to the old `refusal` body;
  - `MIN_CROP`, `CHANGELOG.md` and `recipes/`;
  - `TABLE`, `CROP_TABLE`, R1–R17, the orientation and alpha rows, the 157 crop-mode rows, N1–N4 and
    the golden texts.

  Paste `git diff --stat origin/main...HEAD -- recipes CHANGELOG.md`; it must be empty. The files
  you change are the spec's `## Outputs` list, plus the spec itself, and no more.
- No version bump, no release, no tag.
- Don't edit the spec's timeline or STAGE-060. The orchestrator and the ship session keep them.
- **Every commit uses `git commit -s`** and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR that has an
  unsigned commit.
- **Never poll CI, and never write your own wait loop.** After a push you need CI for, run
  `gh pr checks <PR> --watch` once, in the background. When it exits, take one snapshot of every leg at
  the PR's head SHA. If it exits with legs still pending, say so; don't call that green.
- **Rebase onto `main` before each push**, and redo the overlap check after each rebase.
- **Shell traps:**
  - A piped command reports the **pipe's** exit code, so redirect output and read `$?`.
  - zsh doesn't word-split unquoted parameters, and `${H}:` needs braces.
  - macOS has no `timeout(1)`.
  - Use `/usr/bin/grep`, `/bin/ls` and `/usr/bin/git` for anything you count.
- **Shared state:**
  - Never share a `CARGO_TARGET_DIR` with another session, and delete yours when you're done.
  - Write scratch files only in your session's scratchpad.
  - Don't leave PR-body files behind.
- **Never drive the maintainer's browser.** The page check starts its own headless Chrome.
- **Budget ~120 exchanges.** If you're past budget with everything green, stop and report what's
  left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase
     on what changed. Give the count, and say whether it includes the spec;
   - the RED baseline, quoted;
   - every AC, one line each: met, or not and why. Tick the met ones (`- [x]`);
   - the moved-body diff (AC-2);
   - the parity summary line in both modes, and the `ddphotos-page` last line, plus the delayed-body
     run with its two timings;
   - the behaviour sentences, driven on every exit path the spec lists, with each one quoted;
   - every mutation row: the edit you applied, and its red set against the spec's, with every mismatch
     called out;
   - the slim wasm size, Mac and CI;
   - the CI legs, with `ddphotos-parity`'s summary line and its H9–H11 lines;
   - the overlap check after your last rebase;
   - the temp folders: the system `ddphotos-*` count before and after, and the space you freed, per
     item;
   - the build-phase reflection: write each answer, and name any design call that turned out wrong.
   - **Before your last push,** check that the spec has no `— <answer>` left and no unticked AC that
     you met.
2. **Measure your own cost at your last turn, after the last push.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - **Find it by its content, not by which file is newest:**
     `/usr/bin/grep -l "SPEC-165 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - **Dedup by `.message.id`.** Take the input and both cache counts once per id, and the **MAX**
     `output_tokens`. Include any `subagents/` transcripts.
   - **Price** at the model in `.message.model`: Sonnet is $3/$15 per MTok, Opus $5/$25.
     cache_creation costs ×1.25 the input rate, and cache_read ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-165 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks it
     ready. Never mark it ready yourself.
   - Title: `feat(ddphotos): the engine decides whether the edit page's watermark fits (SPEC-165)`.
   - Body, per AGENTS §13:
     - a summary saying that the page now shrinks a mark the engine would refuse, and blocks Make with
       the reason when no size fits. It never makes an unmarked copy;
     - the spec metadata;
     - the decisions referenced: DEC-045, DEC-104, DEC-110 and DEC-118;
     - the constraints checked;
     - the new decision: DEC-119;
     - breaking: none;
     - the Claude Code footer at the end.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-165 build: draft PR #NNN at <head SHA> — CI <green | red: legs | pending>`. Then,
briefly, as plain lists:
- the RED baseline on `main`;
- AC completion as a count, plus any AC not met;
- the moved-body diff, in one line: what moved and what didn't change;
- the parity summary line in both modes, the `ddphotos-page` last line, and the delayed-body timings;
- each mutation's red set against the spec's, including M11 under cargo, any edit that differed from
  the quoted one, and every cell that turned out wrong;
- the behaviour sentences, quoted;
- the slim wasm size, Mac and CI;
- the CI legs, with `ddphotos-parity`'s summary line and its H9–H11 lines;
- the overlap check after your last rebase;
- the files changed, against the spec's `## Outputs`, and the empty `git diff --stat` for the paths
  that must not change;
- the temp folders: the system `ddphotos-*` count before and after, and the space freed per item;
- any design call that turned out wrong, and any finding outside the spec;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its
  own line.
