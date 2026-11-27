# SPEC-164 — DESIGN prompt: the crop state model, the CLI sentence and a headless page check

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-164-design -- --model opus
```

Your spec is
`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-164-crop-on-the-ddphotos-edit-page-the-crop-state-model-the-cli-sentence-and-a-headless-page-check.md`,
at `cycle: design`. Read its `## Frame` in full, with "Maintainer rulings (2026-11-27)" at its end.
This spec is the half of the crop mode that can be proven before the DOM exists. The DOM is the next
STAGE-060 entry, "Crop on the page: the crop mode's DOM".

The maintainer pastes this prompt into a fresh session and will paste your final message back to the
orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

## Rulings (settled — do not re-open)
From SPEC-164's frame (2026-11-27):
1. **Split.** This spec has three parts, plus their docs:
   - the pure crop-mode functions in `recipe.js` and their Node table;
   - the CLI panel's smaller-file sentence;
   - an in-repo headless-Chrome check of today's page.
   The DOM is not here.
2. **`MIN_CROP = 64` copy pixels per side.** The watermark's fit on a small or thin crop is its own
   STAGE-060 entry.
3. **Swapping twice stays an explicit orientation with no key;** pressing the preset again restores
   following.
4. **In crop mode, form changes update only the size line and the note;** Done renders; "Make the
   copy" is disabled. (The DOM entry builds this; your reducer and `sizeLine` must make it possible.)
5. **The headless check lives in this repo,** as a step in CI's advisory `ddphotos-parity` job. It
   isn't a required check yet.
6. **The README's crop rows go with the DOM entry.** This spec adds only the CLI clause.

Also settled, from SPEC-161 and DEC-118: the recipe text comes only from `presetCrop`, `swapCrop`,
`cropBox` and `cropStep`. **The new functions add no recipe rule.**

## Things in the frame to check before you build on them
Each is a claim until you drive it (AGENTS §15; HANDOFF traps 13 and 17).
1. **`MIN_CROP = 64` is safe from watermark refusals.** The frame's sweeps found refusals at short
   sides 8–40 (grid) and up to 50 px (square boxes), and none above. But its own strip example, a
   1600 × 20 strip at Large, was refused on a 32 px margin that scales with the long edge.
   - Drive thin strips at exactly 64 px on the short side, at the full long edge (1600), at Large,
     with each of the frame's three texts, and at each output size the page offers.
   - If any is refused, `MIN_CROP = 64` doesn't hold as stated. Report the smallest safe value and
     put it to the maintainer; don't change the ruling yourself.
   - Write the result as "measured over this sweep", not as a bound.
2. **The coordinate chain's worst error** is 2 copy px for W on a 375 px phone and 1 px elsewhere,
   and the fraction round trip is exact (0 of 1,280,800 and 0 of 574,056). Re-drive the round trip
   on your own model of `rect_edges`, and the screen error at both display widths.
3. **One ratio serves both axes**, because `edit.css` sizes the image with `object-fit: contain` and
   no letterbox inside the rect. Check `edit.css` and say whether a portrait photo in a landscape
   viewport, or the reverse, still gives one ratio. M2 (the ratio taken from the finished preview)
   is the trap the frame's prototype fell into.
4. **The render count** (`load` 1, `preview` 1, coalesced form changes) was driven in headless
   Chrome on the Mac. Say what the CI step must assert, and confirm `ubuntu-latest` ships a Chrome
   the check finds, as `pages.yml` relies on.
5. **Swap is disabled in S0, S3 and on 1:1, and live on a square photo.** Drive `swapCrop` on each
   case and quote the recipe text.

## Design calls that are yours
1. **The six functions' exact signatures and return shapes:** `toCopyDelta`, `moveBox`,
   `resizeBox`, `MIN_CROP`, `cropReducer` and `sizeLine`.
   - Name the reducer's actions (preset, swap, drag-start, drag-move, drag-end, key-move, key-resize,
     reset, done) and the state's shape.
   - Say whether the pointer's start state lives in the reducer's state or in `edit.js`. M5 (Done
     doesn't clear it) depends on that choice.
   - Each function must be pure and Node-testable, and `checkModules` must add each name to `owned`.
2. **`resizeBox`'s handles.** The frame names 8: 4 corners and 4 edges, with the opposite edge fixed.
   Say how a handle dragged past the opposite edge behaves (clamp at `MIN_CROP` or flip), and how
   `MIN_CROP` interacts with a photo smaller than 64 px on a side.
3. **`sizeLine`'s text.** Use the frame's proposal ("Crop 1067 × 711 of 1600 × 1071 · the copy will
   be 1067 × 711") or better, with literal expected values on W, P, square and R.
4. **The reducer table.** Every transition × W, P, square and R. Square and orientation rows go in
   both orders (trap 20). Each row's expected state and recipe text is a literal from your own
   model, never from `recipe.js` (trap 19). Give the row count and its derivation.
5. **The headless check's rows on today's page.** At least:
   - the page loads against the fake site;
   - the render counter reads `load` 1 and `preview` 1;
   - three quick form changes add one preview;
   - the CLI panel shows the new sentence with the upright size on W and on R;
   - the panel shows `cliRecipe`'s text.
   Give each row's literal and its failure message.
6. **The fake site and the driver.** Decide whether `findChrome` and the CDP client move out of
   `tests/demo_smoke.mjs` into `scripts/lib/`. If they move, `demo_smoke.mjs` changes too, and the
   demo's CI job must stay green. Say which job runs it. Name the temp folders the check makes, and
   whether it keeps them on failure (see "Temp folders" below).
7. **The CI step.** Its place in `ddphotos-parity`, its command, and that it runs unconditionally:
   with no Chrome it exits non-zero, never a printed skip (trap 11). Estimate its CI cost.
8. **The CLI sentence and the README clause.** Keep the frame's wording or improve it. Say where
   `edit.js` fills in the size, and that it reads `photo` (the upright size), not the stored size
   (M12).
9. **Docs and CHANGELOG.** Say whether a CHANGELOG line belongs here. The user-visible change is the
   sentence. List every behaviour sentence the build must drive on every exit path (the frame lists
   four).
10. **No DEC is expected.** If you find one is needed, say why and allocate DEC-119.

## Tests: each one must be able to fail
- **Drive the mutation table yourself on a prototype** in your scratchpad. Use the frame's M1–M12,
  keeping the ones that apply to this half. M1's function side, M3, M4's reducer side, M6's function
  side, M9's function side, M10, M11 and M12 apply here; M2, M5, M7 and M8 are the DOM entry's.
  - Each row quotes its exact edit, the red tests with **each red row's failure message from a run**,
    and the green tests (trap 13).
  - Mark any row you couldn't run "not driven".
  - Add at least one mutant per new function that the frame doesn't list.
- **Trap 14:** `main`'s `recipe.js`, `index.html` and `edit.js` restored verbatim must turn the
  module check and the CLI-sentence row red. Give the exact lines expected.
- **Trap 19:** both sides of the page harness read `recipe.js`, so every guard on a new function is
  a literal. Say which literal catches each mutant.
- **The headless check's negative controls:** M11 (`schedulePreview` calling `runPreview` directly)
  must turn the coalescing row red on its preview count. The sentence removed from `index.html`
  must turn the panel row red on its text. M12 must turn the R row red on its size. Drive all
  three.
- **Nothing else changes:** say what pins it. TABLE, CROP_TABLE, REPLAY_TABLE, the orientation and
  alpha rows, and N1–N5 stay as they are, and the parity summary line keeps its counts (16 table,
  36 crop, 17 replay, 26 orientation, 2 alpha). No file under `src/`, `tests/fixtures/` or
  `recipes/` changes, and the slim wasm stays at brotli 833,619 B on the Mac (CI reads 836,219 B;
  record both).

## Temp folders (set up before the first harness run)
The parity harness keeps its `mkdtempSync(tmpdir())` folders whenever a run fails, so every red
mutant run leaves one. Your new check will likely do the same, and Chrome makes profile folders.
- Before the first run, count the system temp folder's `ddphotos-*` folders and their size. Don't
  delete any of them: some belong to other sessions.
- Run every harness and headless command with `TMPDIR` set to a folder in your scratchpad. After
  the first run, confirm the folders landed there; stop if they didn't.
- Point Chrome's `--user-data-dir` at the scratchpad too.
- Never run the harness while you're editing files it reads.
- Empty the folder after you've read each red run.
- At the end, count the system temp folder's `ddphotos-*` again and report both counts.
- For the build: say whether the new check should take a `--keep` flag or always delete its fake
  site, and write the build's temp-folder instructions into Implementation Context.

## Rules every design session follows

**You are the design cycle only.** You write Acceptance Criteria, Failing Tests and Implementation
Context. You don't write the code. Separate build and verify sessions follow. One builds from your
text alone, and another verifies it, so everything the builder needs must be in the spec
(AGENTS §15).

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you're in the
maintainer's main checkout, not a worktree, stop and say so. Then build the overlap map with
`gh pr list --state open --json number,files`.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 (the coverage map in
   "During design", "a bound or 'can't happen' is a claim until its worst input is driven", the
   refusal rule) and §16 habit 6 (no CI wait loops).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for",
   especially 11, 13, 14, 15 (a summary line counts every failure kind) and 19–21.
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md`:
   your entry, the DOM entry after it, and the watermark-fit finding at the end.
4. `/guidance/constraints.yaml`, then DEC-110, DEC-112, DEC-113 and DEC-118.
5. **SPEC-161 and SPEC-163** in `specs/done/`: the model, its rulings, the harness's families and
   controls, and both ship reflections.
6. The page code in full: `integrations/ddphotos/edit/recipe.js`, `edit.js`, `worker.js`,
   `index.html` and `edit.css`. Then `tests/ddphotos_parity.mjs` (`checkModules` especially),
   `tests/demo_smoke.mjs`, `scripts/serve.mjs`, and the `ddphotos-parity` and `ddphotos-edit-install`
   recipes in the `justfile`. Then CI's `ddphotos-parity` job in `.github/workflows/ci.yml`, and
   `pages.yml`'s browser smoke job.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`.
- **Drive before you write.** Use Node against `recipe.js`, the real slim wasm and a headless Chrome
  you start yourself. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC that names two things needs a
  test for each.
- Expected values come from literals or an independent model, never from the code under test.
- No version bump.

### Ids
Don't run `just new-spec`. Fill in Inputs, Outputs, Acceptance Criteria, Failing Tests and
Implementation Context, and keep `## Frame`. If you find the frame wrong, say so in a short
`### Design corrections to the frame` subsection at the top of `## Design`, and in your report.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-164 build`.
2. In STAGE-060's backlog, change SPEC-164's marker from `(framed 2026-11-27)` to
   `(designed <date>)`, keeping `[M]` or the size you found.
3. Update the timeline: mark design `[x]` with a 2–4 line summary, and put `prompts/SPEC-164-build.md`
   on the build line.
4. Commit with `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-164): design the crop state model and the headless page check`, with the trailer
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
5. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
   - Check with `git ls-remote origin refs/heads/main`: a "Changes must be made through a pull request"
     notice can still mean the push landed.
   - ⛔ **If a permission check refuses the push, stop and report** (trap 21). Name the commits and
     give the exact command for the maintainer to run. Don't say it's on `main` until `ls-remote`
     shows it.
6. Read back `git show --stat HEAD`.

### Guardrails
- ⛔ **What you may change:** the spec, the timeline and the stage file only.
  - No code changes, and no DEC file.
  - No pushed probe branch.
  - No version bump, no tag, no CHANGELOG edit.
  - Never edit PROJ-011's files.
- **Never drive the maintainer's browser.** A headless Chrome you start yourself, with its profile
  in your scratchpad, is fine.
- **Shell traps:**
  - A piped command reports the pipe's exit code, so redirect output and read `$?`.
  - zsh doesn't word-split unquoted parameters, and it reads `$R:r…` as a modifier, so brace it
    (`${R}:`).
  - `git pull` overwrites `FETCH_HEAD`.
  - macOS has no `timeout(1)`.
  - `rtk` can truncate `grep`, `ls` and `git log` output, so use `/usr/bin/grep`, `/bin/ls` and
    `/usr/bin/git` for anything you count.
- **Scratch space:**
  - Never share a `CARGO_TARGET_DIR` with another session.
  - Remove the prototype, its target dir, the fake site and Chrome's profiles before you finish, and
    say how much space that freed (measure before you delete).
  - Write scratch files only in your session's scratchpad.
- **Budget ~70 exchanges.** The mutation rows and the headless prototype are the extra.

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- **Identify it by content, not recency:**
  `/usr/bin/grep -l "SPEC-164 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.
- **Dedup by `.message.id`.** Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any `subagents/` transcripts.
- **Price** at the model in `.message.model`: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is
  ×1.25 the input rate, and cache_read ×0.10.
- **Measure at your last turn, after the push.** Append a `design` entry to the spec's `cost.sessions`
  in a final small commit, and push it the same way.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-164 design: DONE | SPLIT | BLOCKED`, then the commit SHA on
`main` (or "not pushed" and why). Then, short:
- each of the five frame checks, with what you drove (`MIN_CROP` against the strip sweep first);
- any correction to the frame;
- the six functions, one line each, with their signatures;
- the reducer's actions and state shape, and where the pointer state lives;
- the AC list, one line each;
- the rows, by family (reducer, geometry literals, `sizeLine`, module check, headless), with counts
  and their derivations;
- the Failing Tests and the mutation table: each row with its quoted edit, the red tests with their
  failure messages, the green tests, and driven or not driven;
- the headless check: its rows, its negative controls, the shared-driver decision, the CI step and
  its cost;
- what pins "nothing else changes";
- docs and CHANGELOG decisions, and the behaviour sentences the build must drive;
- the wasm size and `recipe.js` growth;
- the overlap map, from the PRs' file lists;
- anything the build prompt must carry (checkpoints, CI legs, Chrome, the temp-folder rules);
- any maintainer question, with your recommendation;
- the system temp folder's `ddphotos-*` counts before and after, and the space freed;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own line.
