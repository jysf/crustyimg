# SPEC-164 — BUILD prompt: the crop mode's state model, the CLI sentence and a headless page check

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**.

There's no Rust in this build. It is JavaScript, the parity harness, a new headless-Chrome check and
one CI step:
- **`recipe.js`** gains six pure functions: `MIN_CROP`, `toCopyDelta`, `moveBox`, `resizeBox`,
  `cropReducer` and `sizeLine`.
- **The parity harness** gains `modeChecks` with 153 literal rows (114 reducer, 31 geometry, 8 size
  line), the six names in `checkModules`, and a `153 crop-mode rows` count.
- **The CLI panel** gains one sentence (`index.html`), filled by one line in `edit.js`. The README's CLI
  row gains one clause.
- **`tests/ddphotos_page.mjs`** (new) runs the edit page in headless Chrome against a fake site, with
  rows H1–H8. Its Chrome driver moves out of `tests/demo_smoke.mjs` into a new `scripts/lib/chrome.mjs`.
- **CI's `ddphotos-parity` job** gains the "Edit page in headless Chrome" step. The `justfile` gains
  `ddphotos-page`.

The design drove every row, message and mutant on a prototype. The work is mostly typing its literals
in exactly and wiring the check. Be thorough across every row: Sonnet's known weak spot is sweep
thoroughness.

```
just wt feat/spec-164-crop-state-model -- --model sonnet
```

You didn't design this; the spec carries the context (AGENTS §15). The maintainer pastes this prompt
into a fresh session and pastes your final message back to the orchestrator, so that message must
stand on its own. **Write it as plain lists, not box-drawn tables.**

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`;
- design commit `2f95a2d` is in your history;
- the spec has `### Maintainer rulings on the design (2026-11-27)`;
- `scripts/lib/chrome.mjs` and `tests/ddphotos_page.mjs` don't exist yet.

If any check fails, stop and report.

⚠ **Overlap.** Parked PR #237 touches `.github/workflows/ci.yml` and the `justfile`, in different
places from this spec. Don't touch #237. After every rebase, run
`gh pr list --state open --json number,title,files` and compare; name any new overlap in Build
Completion. If a rebase conflicts, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-164-crop-on-the-ddphotos-edit-page-the-crop-state-model-the-cli-sentence-and-a-headless-page-check.md`.
   Read it in full:
   - `## Frame`, and both sets of rulings: the frame's (2026-11-27, before `## Design`) and the
     design's (before `## Inputs`);
   - `## Design`: the corrections, the frame checks, the six functions, the CLI sentence, the
     headless check, the CI step, the Node rows, the mutation table, and "Nothing else changes";
   - the Outputs, every AC (17), every Failing Test, and `## Notes for the Implementer`.

   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in your
   report; don't quietly re-decide it.
2. **DECs:** DEC-118 (rules 2 and 5), DEC-112, DEC-113 and DEC-110.
3. **The page code in full:** `integrations/ddphotos/edit/recipe.js`, `edit.js`, `worker.js`,
   `index.html` and `edit.css`.
4. **The harness and the smoke:** `tests/ddphotos_parity.mjs` (`checkModules`, `modelChecks`, the
   main flow and the summary line), `tests/demo_smoke.mjs` (`findChrome`, `CDP`, Chrome's start-up),
   `scripts/serve.mjs`, `scripts/ddphotos-edit-install.mjs`, the `ddphotos-parity` recipe in the
   `justfile`, and the `ddphotos-parity` job in `.github/workflows/ci.yml` and `pages.yml`.
5. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for".
   Traps 11, 12, 13, 14, 19, 20 and 21 bind this build.

## Maintainer rulings (settled — do not re-open)
1. Split: this spec is the state model, the CLI sentence and the headless check. There's no crop
   control on the page yet; the DOM is STAGE-060's next entry.
2. **`MIN_CROP = 64`.** It's the box's usability floor, not a watermark guard. The design found thin
   resized strips still refuse a watermark at 64; the maintainer kept 64, and the watermark-fit
   entry handles it. Don't raise it, and don't add any watermark logic.
3. Swap twice stays explicit (`aspect = "3:2"` with no key); the preset again restores the key.
4. Crop mode updates only the size line; Done renders; Make is disabled during a crop. (The DOM
   entry wires this. Here it shapes the reducer.)
5. The headless check lives in this repo and runs in the **advisory** `ddphotos-parity` job. Don't
   make the job required, and don't rename it.
6. The README gets only the CLI clause. The Crop row and "How it was tested" belong to later entries.

No new DEC. No Rust or wasm change. No comment in `recipe.js`, `edit.js`, `index.html`, the
harness or the new files names a spec, stage, DEC or project id.

## ⚠ Temp folders — set this up before your first run
The parity harness keeps its `mkdtempSync(tmpdir())` folders on every failing run, and your first
checkpoint is a deliberate red run. The headless check makes `ddphotos-page-site-*` and
`ddphotos-page-chrome-*` folders.
- **Before the first run,** count the system temp folder's `ddphotos-*` folders and their size
  (`node -p "require('os').tmpdir()"` gives the folder). Don't delete any: some belong to other
  sessions. Design counted 5 folders, 1,723,616 KB.
- **Run every harness and headless command with `TMPDIR` set to a folder in your scratchpad**
  (create it first). After your first run of each, confirm its folders landed there. If they
  didn't, stop and work out why before running anything else.
- **Empty that folder after you've read each red run.** A folder kept for debugging must be named
  in Build Completion.
- **Never run a harness while you're editing the files it reads.**
- **At the end,** count the system `ddphotos-*` again and report both counts. If it grew, say
  which run leaked.

## Order, with a pushed checkpoint after each stage
These are the spec's six checkpoints. Commit with `git commit -s` and push after each one. A long
build with no pushed commit has been lost before. **If a push is refused, stop and report the
command; check `git ls-remote` before saying anything landed** (trap 21).
1. **The failing tests first.**
   - Add `modeChecks` with the 153 literal rows, the six names in `checkModules` (required from
     `recipe.js`, listed in `owned`), and the summary count. Add `tests/ddphotos_page.mjs` with
     H1–H8.
   - Run both against `main`'s page, and quote the RED baseline: the six
     `FAIL modules: recipe.js lacks <name>` lines, and H4 and H7 with `<no #cli-note>`. Say if
     anything else is red.
   - Open the PR **as a draft** with this commit.
2. **`scripts/lib/chrome.mjs`.** Move `findChrome`, `CDP` and Chrome's start-up out of
   `demo_smoke.mjs`, as `launchChrome(chromePath, profileDir) → { chrome, cdp, exited }`. Add the
   one fix: a `CHROME` that doesn't exist is reported, not spawned. `demo_smoke.mjs`'s checks and
   output don't change, and the long comments move with the code. `just demo-smoke` goes green.
3. **The six functions in `recipe.js`,** after `loadPhoto`, under a `// ── the crop mode ──` rule.
   `just ddphotos-parity` goes green in both modes, with the summary line the spec quotes.
4. **The sentence, the `edit.js` line and the README clause,** each exactly as the spec quotes
   them. `just ddphotos-page` goes green with H1–H8 PASS. If you reword the sentence, change H4,
   H7 and the README together, and keep "the web copy this page edits".
5. **The CI step and `just ddphotos-page`,** exactly as the spec quotes them: right after "Parity
   harness", no `if:`. Push, then read both jobs' logs once CI finishes (see Guardrails):
   - `ddphotos-parity`: the summary line, the H1–H8 PASS lines, and the new step's time;
   - `pages`' "build + browser smoke": green on the shared lib.
6. **The mutation table, re-driven** (traps 12, 13 and 14).
   - Run it in a scratch `git worktree` of your pushed head, never in the build worktree, with its
     own `CARGO_TARGET_DIR` and the scratch `TMPDIR`. Build the slim wasm and binary once and
     reuse them: every mutant is JS.
   - For each of M1, M3, M4, M5, M6, M9, M10, E1–E11, M11, M12, the wording change, the
     paragraph-only removal, trap 14 (`main`'s page restored, `main`'s `recipe.js` only, and an
     `edit.js` that defines `sizeLine`) and no Chrome:
     - apply **the spec's quoted edit**, not your own version. If the quoted line doesn't exist in
       your code, make the closest edit, quote it, and say why;
     - confirm the edit is live (`git diff` is non-empty, and the run reads the mutated file);
     - run the full harness or check, with output redirected and `$?` read;
     - read the red set **off that run**, compare it with the spec's, and quote the first failure
       line. Call out every row the spec doesn't list, and any row that's red for a different
       reason, such as a throw where the spec expects a value (trap 13).
   - Restore after each and show it green. Empty the scratch `TMPDIR` between rows.
   - **E5 and E10 have history:** E5's first version was an equivalent mutant, and E10 only went
     red once X30 existed. Confirm both are red on your rows.

   Paste each red set into Build Completion, trimmed to the lines that name rows. Where a cell is
   wrong, correct it in Build Completion and say so; don't edit the design's table.

## The things most likely to go wrong
- **Expected values from the code under test.** Both sides of the harness read `recipe.js`, so a
  mutant there agrees with itself. Every expected value is the spec's literal, never computed by
  `recipe.js` (trap 19).
- **Square and orientation rows in one order only.** The spec writes each square-photo swap row in
  both orders (X28S, X29S). Type every one in (trap 20).
- **`toCopyDelta`'s rounding.** Half away from zero on both signs, never `-0`, 0 for a ratio that
  isn't a positive finite number. `Math.round` alone is E1.
- **The reducer returns the same object when nothing changes,** and never mutates its input. The
  page disables Swap with that check.
- **`resizeBox` never flips,** and clamps at `min(MIN_CROP, side)` from the opposite edge.
- **The fake site** carries only what `edit.js` reads. Don't copy a real ddphotos `index.json`.
- **H5 depends on H3** leaving `#size` at 800 and the format at JPEG. Keep the order.
- **Cleanup:** await Chrome's `exit` before deleting its profile (deleting earlier failed with
  `ENOTEMPTY`). Delete the fake site unless `--keep`.
- **Every wait fails fast** when `#error` shows, quoting its text and the page exceptions.

## Guardrails
- **What must not change:** anything under `src/`, `tests/fixtures/` or `recipes/`; `worker.js`;
  `CHANGELOG.md`; `TABLE`, `CROP_TABLE`, `REPLAY_TABLE`, the orientation and alpha rows, the golden
  texts and `--self-test`. Paste
  `git diff --stat origin/main...HEAD -- src tests/fixtures recipes integrations/ddphotos/edit/worker.js CHANGELOG.md`;
  it must be empty. The files you change are the spec's `## Outputs` list, no more.
- **The slim wasm is unchanged.** Read its size from your `just wasm-size` on the slim build and
  from the CI log, against brotli 833,619 B on the Mac.
- No version bump, no release, no tag.
- Don't edit the spec's timeline or STAGE-060: the orchestrator and ship keep them.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI, and never write your own wait loop.** After a push you need CI for, run
  `gh pr checks <PR> --watch` once, in the background, and take one snapshot at the PR's head SHA
  when it exits, every leg. If it exits with legs still pending, say so; don't call that green.
- **Rebase onto `main` before each push**, and redo the overlap check after each rebase.
- **Shell traps:**
  - A piped command reports the **pipe's** exit code, so redirect and read `$?`.
  - zsh doesn't word-split unquoted parameters, and it reads `$H:t…` as a modifier, so brace it
    (`${H}:`).
  - `git pull` overwrites `FETCH_HEAD`.
  - macOS has no `timeout(1)`.
  - `rtk` can truncate `grep`, `ls` and `git log`, so use `/usr/bin/grep`, `/bin/ls` and
    `/usr/bin/git` for anything you count.
- **Shared state:** never share a `CARGO_TARGET_DIR` with another session, and delete yours when
  you're done. Write scratch files only in your session's scratchpad, never `/tmp` or the system
  temp folder. Don't leave PR-body files behind.
- **Never drive the maintainer's browser.** The check starts its own headless Chrome, with its
  profile in a temp folder.
- **Budget ~120 exchanges.** If you're past budget with everything green, stop and report what's
  left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase
     on what changed. Give the count, and say whether it includes the spec;
   - the RED baseline, quoted;
   - every AC, one line each: met, or not and why. Tick the met ones (`- [x]`);
   - the parity summary line in both modes, and the `ddphotos-page` last line;
   - the four behaviour sentences, driven with the release binary on your branch and quoted (the
     spec's Notes list them);
   - every mutation row: the edit applied, and its red set against the spec's, with every mismatch
     called out;
   - the slim wasm size, both jobs' step times and the H lines from the PR's log, and the CI legs;
   - the overlap check after your last rebase;
   - the temp folders: the system `ddphotos-*` count before and after, and the space you freed,
     per item;
   - the build-phase reflection: write each answer, and name any design call that turned out wrong.
   - **Before your last push,** check that the spec has no `— <answer>` left in the build-phase
     reflection and no unticked AC you met.
2. **Measure your own cost at your last turn, after the last push.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - **Identify it by content, not recency:**
     `/usr/bin/grep -l "SPEC-164 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - **Dedup by `.message.id`.** Take the input and both cache counts once per id, and the **MAX**
     `output_tokens`. Include any `subagents/` transcripts.
   - **Price** at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation
     is ×1.25 the input rate, and cache_read ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-164 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks
     it ready. Never mark it ready yourself.
   - Title: `feat(ddphotos): the edit page's crop state model, a CLI size note and a headless page check (SPEC-164)`.
   - Body per AGENTS §13:
     - a summary, including that the CLI panel now says to run the recipe on a file at least as
       large as the web copy, and that no crop control appears yet;
     - the spec metadata;
     - decisions referenced: DEC-110, DEC-112, DEC-113, DEC-118;
     - constraints checked;
     - new decisions: none;
     - breaking: none;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-164 build: draft PR #NNN at <head SHA> — CI <green | red: legs | pending>`. Then,
briefly, as plain lists:
- the RED baseline on `main`;
- AC completion, as a count, plus any AC not met;
- the parity summary line in both modes, and the `ddphotos-page` last line;
- each mutation's red set against the spec's, any edit that differed from the quoted one, and
  every cell that turned out wrong;
- the four behaviour sentences, quoted;
- the slim wasm size; the CI legs, with `ddphotos-parity`'s summary line, its H1–H8 lines and the
  new step's time, and `pages`' smoke result;
- the overlap check after your last rebase;
- the files changed, against the spec's `## Outputs`, and the empty `git diff --stat` for the
  paths that must not change;
- the temp folders: the system `ddphotos-*` count before and after, and the space freed per item;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on
  its own line.
