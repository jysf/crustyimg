# SPEC-164 — VERIFY prompt: the crop mode's state model, the CLI sentence and the headless page check

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-164. Your job is to find what the build's own evidence can't show. CI green and
17 met ACs are claims until you drive them.

```
just wt verify/spec-164 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-164-crop-state-model:refs/remotes/origin/feat/spec-164-crop-state-model
git reset --hard origin/feat/spec-164-crop-state-model
```

Draft PR #274's head was `b7fee27` when this was written. The orchestrator checked CI there: every
leg passed, the release jobs skipped, and `ddphotos page parity` and the `pages` "build + browser
smoke" were among the passes. The head is based on `main` at `a849bf4`. Record the SHA you verify.
The maintainer pastes your final message back to the orchestrator, so it must stand on its own.
**Write it as plain lists, not box-drawn tables.**

⛔ **Don't merge PR #274 until this verify has reported.**

## What the build claims
- **ACs:** 17 of 17 met, all ticked.
- **Red baseline on `main`'s page:** parity exits 1 with exactly the six `FAIL modules: recipe.js
  lacks <name>` lines. The page check reads `ddphotos-page: 8 rows, 2 failed`, with H4 and H7 on
  `<no #cli-note>`.
- **Parity:** `16 table rows, 36 crop rows, 17 replay rows, 26 orientation rows, 2 alpha rows, 153
  crop-mode rows; 0 row failures, 0 other failures, K=1 (cap 78)`. `--self-test` unchanged.
- **Page check:** `ddphotos-page: 8 rows, 0 failed`. `just demo-smoke`: 78 ✓, 0 ✗ on the shared lib.
- **Mutations:** every row in the spec's table driven and matching, except **M4**, which is also red
  on X21 (the S0 key-move). The spec's quoted ternary doesn't exist in the build's code; `toBox` is
  shared by the drag and key paths.
- **CI:** the new step took 32 s, inside a 5 min 42 s job. Slim wasm brotli 833,619 B on the Mac.
- **Scope:** 10 files plus the spec, matching `## Outputs`. The do-not-change paths have an empty
  diff.
- **Deviations the build named:**
  - Chrome's profile delete retries for about 15 s, not 2 s, because Chrome's helpers kept writing
    after exit.
  - `launchChrome` also returns `port` (`demo_smoke.mjs`'s `file://` check needs it), and
    `findChrome` accepts a bare `CHROME` name on `PATH`.
  - `drag-end` and `done` are separate reducer cases.
  - A same-object shortcut in `preset` was added and then removed (commit `416a240`, "a preset press
    always makes a new state, as designed"), because under M3 it turned two rows red that the spec
    doesn't list.
  - Checkpoint 1's commit can't run alone: the page check imports `scripts/lib/chrome.mjs`, which
    arrived at checkpoint 2.

## Gaps the orchestrator saw
Check each and classify it; don't fix them.
- **`scripts/lib/chrome.mjs:~72`** carries a `(SPEC-078)` reference in a comment moved from
  `demo_smoke.mjs`. `main`'s `demo_smoke.mjs` names SPEC-077/078/080 in several comments. Say
  whether moving it into a new file breaks the "no ids in comments" rule, or is history carried with
  the code.
- **The preset shortcut and AC-4.** AC-4 says `cropReducer` "returns the same object when nothing
  changes", and the page disables Swap with that check. Pressing the preset that's already active,
  with no drag: is that "nothing changes" by the design's own rows? Say which rows pin it, and
  whether the build's removal is the design or a departure.
- **The cost** was measured before the build's last commits. Ship records it; just say whether the
  build's note says so.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes, no CI reruns. Report; don't fix. Run any mutation
in a scratch `git worktree` under your scratchpad, with its own `CARGO_TARGET_DIR`, and restore it
before you finish. **Never drive the maintainer's browser.** The page check starts its own headless
Chrome, with its profile in a temp folder.

## Temp folders (set up before the first harness or page-check run)
The parity harness keeps its `mkdtempSync(tmpdir())` folders whenever a run fails. `--self-test`
always keeps a `ddphotos-orientation-*` folder, because N3 is red by design. The page check makes
`ddphotos-page-site-*` and `ddphotos-page-chrome-*` folders.
- Before the first run, count the system temp folder's `ddphotos-*` folders and their size
  (`du -sm`). Don't delete any of them: some belong to other sessions.
- Run every harness and page-check command with `TMPDIR` set to a folder in your scratchpad. After
  the first run of each, confirm the folders landed there; stop if they didn't.
- Empty that folder after you've read each red run.
- At the end, count the system temp folder's `ddphotos-*` again and report both counts.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-164-crop-on-the-ddphotos-edit-page-the-crop-state-model-the-cli-sentence-and-a-headless-page-check.md`.
   Read:
   - `## Frame`, with the frame's rulings and the design's rulings (both 2026-11-27);
   - `## Design`: the eight corrections, the frame checks, the six functions, the CLI sentence, the
     headless check, the CI step, the Node rows, the mutation table, and "Nothing else changes";
   - the ACs, the Failing Tests, the Notes and Build Completion.
2. DEC-118 (rules 2 and 5), DEC-112, DEC-113 and DEC-110 on `main`.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for",
   especially traps 11, 13, 14, 19 and 20.
4. AGENTS §15's rules: a bound or "can't happen" is a claim until its worst input is driven, and
   every finding is classified.

## Maintainer rulings the build had to honour
1. Split: no crop control on the page yet.
2. **`MIN_CROP = 64`, kept after the design's finding.** Thin resized strips still refuse a
   watermark at 64; STAGE-060's watermark-fit entry handles that, before the DOM entry. **Not a
   finding here** unless the build changed the value or added watermark logic.
3. Swap twice stays explicit; the preset again restores the key.
4. Crop mode updates only the size line; Done renders; Make is disabled during a crop.
5. The check runs in the **advisory** `ddphotos-parity` job, which isn't renamed or made required.
6. The README gets only the CLI clause.
7. No comment names a spec, stage, DEC or project id.

## Check, in order
1. **Scope.**
   - Compare `git diff --name-only origin/main...HEAD` with `## Outputs`. Give the real count and
     name every file outside the list.
   - Confirm an empty diff for `src`, `tests/fixtures`, `recipes`, `worker.js` and `CHANGELOG.md`.
   - Check every commit is signed off, and that the PR merges cleanly onto current `main`.
   - Run `gh pr list --state open --json number,files` once. The build named only #237.
2. **The functions, against your own model.**
   - Write your own model of the six functions **from the spec's `## Design` text**, never from
     `recipe.js` or the harness's literals.
   - Check every one of the 153 literals against it. Say how many agree, and give every mismatch.
   - Then go past the table. Import `recipe.js` in Node and compare it with your model over random
     action sequences (thousands, on all four shapes: W, P, R and a square), including:
     - drags and key moves past every edge;
     - resizes on all 8 handles past the opposite edge;
     - boxes under 64 px on a photo side under 64;
     - stray `drag-move`, `drag-end` and `done` actions with no drag;
     - unknown actions and handles.
   - On every step, assert these invariants:
     - the box stays inside the photo;
     - a resize never flips, and never shrinks a side below `min(64, side)`;
     - the input state is never mutated (freeze it);
     - the same object comes back exactly when the state's content is unchanged. Say which actions
       break that, and whether the design says they should.
   - `toCopyDelta`: drive the symmetry (`f(−d) === −f(d)` with no `-0`), the halves, and the bad
     ratios (0, negative, `NaN`, `Infinity`, a string).
3. **The recipe text (AC-5).** For every reducer state your sequences reach, run `planEdit`'s crop
   step text and check it against DEC-118 rule 2 and the rulings:
   - an untouched preset writes `aspect` plus `orientation = "match"`, except on a square or 1:1;
   - a swap writes the aspect as shown, with no key;
   - a box writes four fractions;
   - nothing writes `gravity`.

   Confirm `cropStep`, `presetCrop`, `swapCrop` and `cropBox` are byte-for-byte as on `main`.
4. **`sizeLine` (AC-6).** Check L1–L8 against your model, and the output size against `planEdit` on
   a few rows outside the table, including R and a 400 size.
5. **The M4 cell.** Re-drive M4 with the closest edit to the spec's quoted one, and say:
   - whether X21's red is a correct catch or a side effect of `toBox` being shared;
   - what the corrected cell should say. Ship corrects the table.
6. **The page check, driven by you.**
   - Run `just ddphotos-page` (own `CARGO_TARGET_DIR`), redirect output, read `$?`, and quote the
     last line.
   - Read `tests/ddphotos_page.mjs`:
     - Does the render counter go in only through `Page.addScriptToEvaluateOnNewDocument`, with no
       test hook in the page?
     - Does the fake site carry only what `edit.js` reads?
     - Does every wait fail fast on `#error`?
     - Does every exit path await Chrome's exit before deleting its profile?
     - Is the fake site deleted unless `--keep`?
   - Drive **M11** (the debounce removed) and **M12** (`worker.js`'s `load` returns the stored size)
     with the spec's quoted edits. Quote each red line, and confirm only H3 and only H7 go red.
   - Drive the failure exits: no Chrome, a `CHROME` that doesn't exist, a bare name not on `PATH`,
     a missing `--pkg`, and a `--pkg` folder with no package. Each must exit 1 with a FAIL line,
     never a skip or a stack trace (trap 11).
   - Kill the check mid-run once (for example `kill -INT` during H3). Say what it leaves behind in
     your `TMPDIR`.
   - The profile delete now retries for about 15 s. Say whether that can mask a Chrome that never
     exits, and whether a failed delete can change the exit code.
7. **The shared Chrome lib (AC-14).**
   - Diff `demo_smoke.mjs`'s checks and output on `main` against the branch. Confirm the moved code
     is the same apart from the named deviations.
   - Run `just demo-smoke` and quote its count.
   - Say whether `findChrome`'s new `PATH` lookup can pick up a different binary than before on the
     CI image or the Mac.
8. **The CI step (AC-15).**
   - Confirm it sits right after "Parity harness", has no `if:`, and the job's `name:` and advisory
     status are unchanged.
   - Read the PR's `ddphotos-parity` log once, at the head SHA. Quote the summary line, the H1–H8
     PASS lines, the last line, and the step's time.
   - The comment says a failed parity step skips this one, so a stale cached `pkg-slim` never runs.
     Confirm GitHub's default step behaviour gives that.
9. **The sentence (AC-9, AC-10, AC-17).**
   - Read `index.html`'s paragraph, `edit.js`'s one line and the README clause, against the spec's
     text.
   - Re-drive behaviour sentence 2 yourself, with the native binary: the page's CLI recipe with the
     watermark on, on a 100×67 file. Quote the exit code and stderr.
   - Re-drive sentence 1 on one file smaller than the copy.
   - Say whether "the web copy this page edits (W × H)" is right on R, where the stored file is
     1600×1071.
10. **Nothing else changed (AC-16).** Diff `TABLE`, `CROP_TABLE`, `REPLAY_TABLE`, the orientation and
    alpha rows, the golden texts, and N1–N5's code and lists against `main`. Run `just ddphotos-parity`
    once in each mode, with your own `CARGO_TARGET_DIR`, redirect output, read `$?`, and quote both
    summary lines.
11. **Trap 14.** Restore `main`'s `recipe.js`, `index.html` and `edit.js` in a scratch worktree, and
    confirm: the six module lines, and H4/H7 red on `<no #cli-note>` with the other six H rows green.
    Then restore and show green.
12. **Comments.** Run `/usr/bin/grep` for `SPEC-`, `STAGE-`, `DEC-` and `PROJ-` over every added line
    in the page files, `tests/ddphotos_parity.mjs`, `tests/ddphotos_page.mjs`,
    `scripts/lib/chrome.mjs`, `ci.yml` and the `justfile`. Read the new comments for ones that
    describe the spec's history rather than the code.
13. **CI.** Run `gh pr checks 274` once, at the head. Say whether every leg is green.

## Classify every finding (AGENTS §15)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment, spec cell or table cell that misstates something the code already
  does;
- **assert:** a missing or loose assertion, where the behaviour is already right;
- **claim:** a fix that would change what a doc or test claims is true, or the code's behaviour.

Only a **claim** needs a re-verify after its punch list.

For every bound, count or "can't happen" in the spec, say whether it was driven, by whom and over
what range. Examples:
- "the box never leaves the photo";
- "returns the same object when nothing changes";
- "never skips";
- "no temp folder left on a pass or a fail";
- "0 B of wasm growth".

## Guardrails
- A piped command reports the pipe's exit code, so redirect output and read `$?`.
- zsh doesn't word-split unquoted parameters, and it reads `$H:t…` as a modifier, so brace it
  (`${H}:`).
- `git pull` overwrites `FETCH_HEAD`, so refer to the branch by its remote ref or SHA.
- macOS has no `timeout(1)`.
- `rtk` can truncate `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything you
  count.
- Never share a `CARGO_TARGET_DIR` with another session.
- **Never poll CI and never write your own wait loop.**
- Write scratch files only in your session's scratchpad. Delete the scratch target dirs and
  worktrees, and say how much space that freed (measure before you delete). Leave the tree clean.
- **Budget ~80 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. The `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-164 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
- Price each component at Opus: $5 per MTok input and $25 output, with cache_creation at ×1.25 and
  cache_read at ×0.10. Measure at your last turn. You're read-only, so report the numbers; ship
  writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only, no box-drawn tables. The first line is exactly one of:
- `✅ APPROVED — merge PR #274 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #274 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–13, one or two lines each, with the evidence;
- the three gaps the orchestrator saw, each with its kind;
- the punch items, numbered. Give each its kind (wording / assert / claim), its evidence, the fix
  you'd expect, and whether it lands on the PR or on `main`;
- optional cleanups, listed separately, each with its kind;
- the mutation-table cells ship should correct, one line each;
- out-of-spec findings, each with where it belongs (a stage, or `guidance/ideas.yaml`);
- the system temp folder's `ddphotos-*` counts before and after, and the space freed;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
