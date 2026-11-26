# SPEC-161 — VERIFY prompt: crop on the ddphotos edit page, the recipe model and the 4x replay

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-161. Your job is to find what the build's own evidence can't show. CI green and
13 met ACs are claims until you drive them.

```
just wt verify/spec-161 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-161-page-crop-model:refs/remotes/origin/feat/spec-161-page-crop-model
git reset --hard origin/feat/spec-161-page-crop-model
```

Draft PR #273's head was `f9c81ae` when this was written. CI was green there, as the orchestrator
checked: 17 passed, 8 release jobs skipped, `ddphotos-parity` among the passes. The head is based on
`main`'s tip at `d7ba2f9`. Record the SHA you verify. The maintainer pastes your final message back
to the orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

⛔ **Don't merge PR #273 until this verify has reported.**

## What the build claims
- **ACs:** 13 of 13 met.
- **Red baseline on `main`:** the new harness against `main`'s page files exits 1 at the module
  check: six `recipe.js lacks …` lines and the `worker.js` line. Nothing else red.
- **Harness, main mode:** `16 table rows, 35 crop rows, 17 replay rows, 26 orientation rows, 2 alpha
  rows; 0 row failures, 0 other failures, K=1 (cap 78)`. Self-test: B1–B19 and N1–N5 PASS.
- **Replay PSNR:** lowest aspect R9 53.95 dB (floor 53.5), lowest box R12 59.46 dB (floor 59). All 17
  equal the design's numbers to two decimals.
- **Controls:** N1, N2, N4, N5 red on exactly their lists. Two reasons differ from the design:
  - **N4:** C26, C31, C33 and C34 fail on the comparator's `max` (equal sizes, pixels differ); only
    C32 fails on `dims`. The design said all five fail on `dims`, with C26 natively 601×482.
  - **M4:** C30, R12 and R15 fail as "threw", because the engine refuses the stored-size fractions
    (`'height' must be … <= 1, got 1.4939…`). The design said C30 reads 560×1071 with mark 43.
- **Mutations:** M1–M11 and trap 14's three rows, each driven in a scratch worktree; every red set
  matches the table.
- **Scope:** 6 files, matching `## Outputs`. The do-not-change paths (`src`, `tests/fixtures`,
  `edit.js`, `justfile`, `CHANGELOG.md`, `recipes`) have an empty diff. Slim wasm brotli 833,619 B,
  0 B of growth.

## Gaps the orchestrator saw in Build Completion
Check each and classify it; don't fix them.
- The 13 ACs are still `- [ ]` in the spec. SPEC-163's were ticked at build.
- The three build-phase reflection answers are still `— <answer>`. The build prompt asked for them.
- A `--self-test` run leaves a `ddphotos-orientation-*` folder every time, because N3 is red by
  design and the harness keeps red runs' folders. Say whether that happens on `main` too (SPEC-163
  added N3), so it belongs to that spec's loose ends, not this PR.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes, no CI reruns. Report; don't fix. Run any mutation
in a scratch `git worktree` under your scratchpad, with its own `CARGO_TARGET_DIR`, and restore it
before you finish.

## Temp folders (set up before the first harness run)
The harness makes its folders with `mkdtempSync(tmpdir())` and keeps them whenever a run fails, so
every red mutant run leaves one behind.
- Before the first run, count the system temp folder's `ddphotos-*` folders and their size
  (`du -sm`). Don't delete any of them: some belong to other sessions.
- Run every harness command with `TMPDIR` set to a folder in your scratchpad. After the first run,
  confirm the folders landed there; stop if they didn't.
- Empty that folder after you've read each red run.
- At the end, count the system temp folder's `ddphotos-*` again and report both counts.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-161-crop-on-the-ddphotos-edit-page-the-recipe-model-and-the-4x-replay.md`.
   Read these sections:
   - `## Frame`, with both sets of maintainer rulings (2026-11-22 and 2026-11-26);
   - `## Design`: the nine corrections, "What was driven", the model, `worker.js` and `edit.js`, the
     replay mode, the rows and DEC-118's content;
   - the ACs, the Failing Tests, the mutation table and Build Completion.
2. `decisions/DEC-118-…md` on the branch, and DEC-110, DEC-112 (rule 8), DEC-113 and DEC-114 rule 5
   on `main`.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for", especially traps 7, 10, 13 and 14.
4. AGENTS §15's rules: a bound or "can't happen" is a claim until its worst input is driven, and
   every finding is classified.

## Maintainer rulings the build had to honour
1. A preset emits `aspect = "a:b"`; a dragged box emits four fractions.
2. A preset that follows the photo adds `orientation = "match"`. A preset on a square photo, and
   `1:1` on any photo, write no key.
3. A swapped preset writes the aspect as shown, with no key.
4. The replay bound is a new DEC (DEC-118), not an amendment.
5. No comment or message names a spec, stage or DEC id.

## Check, in order
1. **Scope.**
   - Compare `git diff --name-only origin/main...HEAD` with `## Outputs`. Give the real count and
     name every file outside the list.
   - Confirm the do-not-change paths have an empty diff, including `integrations/ddphotos/README.md`.
   - Check every commit is signed off. Check the PR merges cleanly onto current `main`.
   - Run `gh pr list --state open --json number,files` once. The build named only #237, with no
     file in common.
2. **The sizes, against an independent model.**
   - Write your own Python model from `src/operation/crop.rs` (`rect_edges`, `aspect_size`) and
     `src/operation/mod.rs` (`resize` mode `max`, `follow_orientation`, the placement), never from
     `recipe.js` or the spec's literals.
   - Check every C1–C35 and R1–R17 literal in the harness against it.
   - Then go past the table: import `recipe.js` in Node and compare `planEdit`, `aspectSize`,
     `followOrientation` and `cropBox` with your model over a grid that reaches the edges:
     - the five inputs W, P, S, N, R;
     - presets 1:1, 4:5, 5:4, 3:2, 2:3, 16:9 and 9:16, followed and swapped;
     - sizes published, 1200, 1080, 800, 400 and 1;
     - a few thousand random boxes, including 1-px-wide and 1-px-tall ones.
   - Drive a sample of those on the real slim wasm and the native binary too, and compare output
     sizes. Say how many cases you ran and give every mismatch.
   - Correction 4 added `Math.max(1, …)` to `planEdit`. Find an input where it matters, and say
     whether any harness row reaches it.
3. **The box fractions (AC-4).**
   - Re-drive the `model-fractions` readback independently: every box on sides 1071 and 1600 reads
     back, through your port of DEC-112's rounding, to the same pixel edges, with `x + width ≤ 1`.
     Say whether you reproduce the 1,854,856 count.
   - Check "the shortest text that reads back" against DEC-114 rule 5 on a sample: no shorter
     decimal reads back to the same number.
4. **The rulings, driven on the model.**
   - Call `presetCrop` and `swapCrop` on W, P, S, N and R for every preset, and show the recipe text
     each writes. A square photo and `1:1` must write no key; a swapped preset must write the aspect
     as shown and no key; nothing writes `gravity`.
   - N is 1071×1072, so it follows. Say whether a near-square original whose copy rounds to a
     square is handled as DEC-118 rule 2 says.
5. **The upright size (AC-1, trap 10).**
   - Read R's Orientation tag with ImageMagick or `exiftool`.
   - Confirm `loadPhoto` returns 1071×1600 and the stored file's format, and that `worker.js`'s
     `load` is exactly the one line the module check pins.
   - The design says `edit.js` needs no change, because it builds `photo` from
     `meta.width`/`meta.height`. Read every place in `edit.js` and `worker.js` that reads a width,
     height or size, and say whether any still sees the stored size.
6. **The replay (AC-9, DEC-118 rules 1, 4 and 5).**
   - Re-run `tests/ddphotos_upscale.py` with the pinned Pillow (12.3.0, `justfile`'s
     `_ddphotos_pillow`) for one 4× row and one 3.75× row. Recompute the PSNR with your own code
     (numpy or ImageMagick `compare -metric PSNR`, not the harness's `psnr`), and compare it with
     the harness's number.
   - Drive B15–B18's edges on `judgeReplay` yourself: exactly at each floor, and just below.
   - Correction 8 says a right shift cancels the centring offset on R1 and R9, which stay above
     53.5 dB. Drive that, and say whether `placement` really covers what N5 misses.
   - Drive DEC-118 rule 5 once: an original smaller than the copy. Say what the CLI does.
   - Spot-check rule 1's aspect bound with your model over at least one full slice (every short
     side for a few long sides, including 6400). Say what range you drove.
   - Say whether the floors' "no further headroom" can flake in CI. Does CI install the pinned
     Pillow?
7. **The controls (AC-10).**
   - Drive N4 with a print of each red row's failure reason. Settle the dispute: `dims` or `max`
     for C26, C31, C33 and C34, and what C26's native size really is. Say which cell ship corrects.
   - Drive N1, N2 and N5 once and compare their red lists, in order, with the spec.
   - Confirm N3's code and list are byte-for-byte as on `main`.
8. **Mutations, driven by you.** Don't re-run all of them. Use the **exact edit the spec quotes**
   and a full harness run each, then restore and re-run green.
   - **M4** (`loadPhoto` reads the stored size): confirm the red set. Then judge trap 13: C30, R12
     and R15 go red as "threw", for an engine refusal. Is there any row where the stored-size bug
     shows as a wrong mark or size, not a throw? If AC-7's guard against it rests only on a throw,
     say so and classify it.
   - **M5** (crop after resize, in `cliRecipe` only): is `golden-crop-cli` really the only guard on
     rows where crop and resize commute?
   - **M9** (a square follows): `golden-crop-square` only?
   - **M10** (centring `floor` → `ceil`): `placement` only?
   - **Two the table doesn't have:**
     - drop `Math.max(1, …)` from `planEdit`'s two lines;
     - let a box's `x + width` exceed 1 by one rounding step in `cropStep`.
     Say which rows go red, or that none do.
   - **Trap 14:** restore `main`'s three page files and confirm exit 1 at the module check with
     exactly the seven lines.
   - Confirm the tree is restored: `git status` clean and the harness green.
9. **The goldens and the recipe order (AC-5).** Compare `golden-crop-box`, `golden-crop-square` and
   `golden-crop-cli` with the spec's literals byte for byte. Read the pixel, CLI and preview recipes
   for a crop row and confirm `auto-orient`, then `crop`, then `resize`. Replay C35's CLI text with
   the native binary's `apply` on the 4× original and compare sizes with R's literal.
10. **Nothing else changed (AC-12).** Diff TABLE's 16 rows, its three `GOLDEN_*` texts,
    `ORIENTATION_ROWS` and the alpha code against `main`. Only the three P5 failure strings, the
    transpose's two code lines and its comment may differ. Confirm that N1's and N2's lists keep
    TABLE's entries in order and only add C rows. Read the wasm size from CI's ddphotos-parity log
    or `just wasm-size`.
11. **DEC-118 (AC-13).**
    - For each sentence that states a bound, count or behaviour, say what drove it: the design, the
      build, or you, and over what range.
    - Check `affected_scope` is a block list (trap 7). Run `just decisions-audit` and quote its last
      line.
    - Say whether the copy-maker caveat (photogen, not crustyimg's `resize`, makes the real copy)
      is stated clearly enough for a reader who never saw the spec.
12. **Comments.** Run `/usr/bin/grep` over every added line in `recipe.js`, `worker.js`,
    `tests/ddphotos_parity.mjs` and `tests/ddphotos_upscale.py` for `SPEC-`, `STAGE-`, `DEC-` and
    `PROJ-`. Read the new comments for ones that describe the spec's history rather than the code.
13. **Full run and CI.**
    - Run `just ddphotos-parity` once in each mode (it builds into a fresh target dir; give it your
      own `CARGO_TARGET_DIR`). Redirect output and read `$?`. Quote both summary lines.
    - No Rust source changed, so the cargo matrix isn't needed.
    - Run `gh pr checks 273` once at the head. Say whether every leg is green.

## Classify every finding (AGENTS §15)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment, spec cell or table cell that misstates something the code already
  does;
- **assert:** a missing or loose assertion, where the behaviour is already right;
- **claim:** a fix that would change what a doc, DEC or test claims is true, or the code's behaviour.

Only a **claim** needs a re-verify after its punch list.

For every bound, count or "can't happen" in the spec and DEC-118, say whether it was driven, by whom
and over what range. Examples:
- "aspect differs by at most 1 px, exhaustive over 273,365,360 cases";
- "0 mismatches at each exact 4× pair";
- "every box on sides 1071 and 1600 reads back";
- "N5 can't see a right shift on R1 and R9";
- "`edit.js` needs no change";
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
- **Never poll CI and never write your own wait loop** (AGENTS §16 habit 6).
- Never drive the maintainer's browser.
- Write scratch files only in your session's scratchpad. Delete the scratch target dirs, worktrees
  and images, and say how much space that freed (measure before you delete). Leave the tree clean.
- **Budget ~80 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. The `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-161 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
- Price each component at Opus: $5 per MTok input and $25 output, with cache_creation at ×1.25 and
  cache_read at ×0.10. Measure at your last turn. You're read-only, so report the numbers; ship
  writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only, no box-drawn tables. The first line is exactly one of:
- `✅ APPROVED — merge PR #273 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #273 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–13, one or two lines each, with the evidence;
- the three Build Completion gaps, each with its kind;
- the punch items, numbered. Give each its kind (wording / assert / claim), its evidence, the fix
  you'd expect, and whether it lands on the PR or on `main`;
- optional cleanups, listed separately, each with its kind;
- the mutation-table and control cells ship should correct, one line each;
- out-of-spec findings, each with where it belongs (a stage, or `guidance/ideas.yaml`);
- the system temp folder's `ddphotos-*` counts before and after, and the space freed;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
