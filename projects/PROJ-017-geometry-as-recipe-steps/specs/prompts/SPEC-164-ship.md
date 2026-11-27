# SPEC-164 — SHIP prompt: the crop state model, the CLI sentence and a headless page check

Cycle: **ship**. This is bookkeeping, plus the spec corrections verify asked for and three findings to
file. Fresh CLI session, own worktree, Sonnet:

```
just wt docs/spec-164-ship -- --model sonnet
```

The maintainer will paste your final message back to the orchestrator, so it must stand on its own.
**Write it as plain lists, not box-drawn tables.**

## Step 0 — check the merge, then get onto current `main`
1. Run `git fetch origin && git checkout --detach origin/main`.
2. ⛔ **Go on only if both of these hold.** Otherwise, stop and report.
   - **PR #274 is merged.** Run `gh pr view 274 --json state,mergeCommit,mergedAt,headRefOid`. The
     approved head is `81ea04f`. Record the merge SHA, the date and the merged head. If the head isn't
     `81ea04f`, confirm that `git log --first-parent --no-merges --oneline 81ea04f..<head>` lists
     nothing, and stop if it lists anything.
   - **The timeline**
     (`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-164-crop-on-the-ddphotos-edit-page-the-crop-state-model-the-cli-sentence-and-a-headless-page-check-timeline.md`)
     has build, verify and punch list 1 ticked `[x]`. Punch list 2 is still `[ ]`; step 1 ticks it.
3. Confirm the merge brought the work onto `main`. `tests/ddphotos_page.mjs` must exist, and its
   `open()` must set `window.__leaving`. The spec's `## Build Completion` must have both punch-list
   subsections. If any of this is missing, stop and report.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- **The spec in full.** In particular:
  - the Frame, with its rulings;
  - the Design: "The six functions", "The mutation table, driven", and "Maintainer rulings on the
    design";
  - the ACs and the Failing Tests;
  - `## Build Completion`, with both punch-list subsections;
  - the cost block.
- **The timeline**, all of it.
- **STAGE-060's `## Spec Backlog`**, all of it, and its `**Count:**` line
  (`projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md`).
- **The reflections of SPEC-161 and SPEC-163** (in `specs/done/`), for their proposals and what came
  of them.

## The facts to reflect on (frame to ship)
- **Frame (Opus, $6.28).** It split the STAGE-060 entry: this spec covers the half that can be proven
  before the DOM exists, and the DOM entry comes next.
- **Design (Opus, $11.44).** It tabled 153 crop-mode rows, 18 Node mutants and 5 headless controls,
  all driven red, plus a page check H1–H8 driven green in a prototype. It found that `MIN_CROP = 64`
  is refused on thin resized strips. The maintainer ruled to keep 64 and to land the watermark-fit
  entry before the DOM entry.
- **Build (Sonnet, $7.50, measured before its last commits).** 17 of 17 ACs, CI green, and every
  mutant as tabled except M4, which was also red on X21.
- **Verify (Opus, $9.18): PUNCH LIST at `b7fee27`, wording only, no re-verify.**
  - Its own model agreed with all 153 literals and with 187,096 fuzzed reducer steps.
  - **M4's cell is wrong.** The build's edit turns X10 and X21 red, 28 failure lines, because the
    drag and key paths share `toBox`. An edit to the drag path alone turns only X10 red.
  - **AC-4 and the design sentence overclaim.** "An action that changes nothing returns the same
    object" isn't true of two actions. `preset` always returns a new state, even when that preset is
    already active with no drag. `drag-start` always returns a new state too.
- **Punch list 1 (Sonnet, $1.17).** It fixed three comments. It added X31 (the active preset pressed
  again, same-object `false`) on all four shapes, so 153 rows became 157. Under the negative control,
  X31 went red on all four shapes, and so did X7 (`p3:2 sw p3:2`). That's expected, since X7 also
  presses an already-active preset. The session ran the control on its working copy of `recipe.js`,
  not a scratch copy, and restored it with `git checkout`.
- **CI went red at punch list 1's head.** The page check failed with `Cannot read properties of null
  (reading 'src')`. The cause: `open()` returned when the navigation started, so the first poll could
  run on the previous document. The orchestrator wrote punch list 2 rather than re-running the leg.
- **Punch list 2 (Sonnet, $1.00).**
  - `open()` now marks the old window and polls until the new document is loaded.
  - The CI failure reproduced only when the page's body was delayed. Delaying the headers didn't
    reproduce it, because the navigate command waits for the response to start.
  - The fix is green, five local runs out of five, and CI is green at `81ea04f`.
  - The orchestrator also predicted a second form: R reading the stale W page. It wasn't reproduced.
- **Neither design's prototype nor build's run caught the race.** Both passed by timing. It first
  failed in CI, two pushes after verify. Verify checked the page check's rows and failure exits, but
  not how it navigates.

## Do, in order
1. **Update the timeline.**
   - Tick punch list 2 `[x]`: `(2026-11-27, Sonnet, prompts/SPEC-164-punchlist-2.md): 81ea04f; open()
     waits for the new document; CI failure reproduced (body delay) red then green; stale-page form
     not reproduced. $<re-measured>.`
   - Add a merge line: `- [x] **merge** (<date>, maintainer) — PR #274 as <merge SHA>, head <head>.`
     Check it against `gh`.
   - Mark ship `[~]`.
2. **Correct the spec**, and mark each change `(corrected at ship)` in place. Find each target by its
   content. If a target no longer reads as quoted here, skip that edit and report the text as it
   stands.
   - **The design sentence**, in "The six functions": "**An action that changes nothing returns the
     same object** (`===`)." Keep it, and add a sentence naming the two exceptions: `preset` and
     `drag-start` always return a new state (X31 pins `preset`).
   - **AC-4**: "returns the same object when nothing changes" becomes "returns the same object when
     nothing changes, except `preset` and `drag-start`, which always return a new state". Its row
     list "(X1–X30, G29, G31)" becomes "(X1–X31, G29, G31)".
   - **M4** in "The mutation table, driven": the red set becomes "X10 and X21 on all four shapes, 28
     failure lines, because the drag and key paths share `toBox`; a drag-only edit turns only X10
     red", with "(found by build, confirmed by verify)". Keep the first-message quote.
   - **The counts.** Change 153 to 157 wherever the spec states the current row count or the summary
     line: the Node-rows section, the count line (where 114 becomes 118, so
     `118 + 31 + 8 = 157`), "Nothing else changes", Outputs, AC-8, Failing Tests and Notes for the
     Implementer. Leave alone any 153 that records a past run, such as the prototype's run or the
     build's report. List every line you changed, and every 153 you left with its reason. Count with
     `/usr/bin/grep -n 153` before and after.
   - **Tick AC-1 to AC-17.** Verify confirmed each, and Build Completion gives the evidence.
   - **Front matter**: set `agents.implementer` to `claude-sonnet-5-5`.

   Nothing else in the spec's body changes, apart from the reflection and the cost block.
3. **File two findings in `guidance/ideas.yaml`**, with `status: proposed`, in the format of the
   entries around `build-output-link-to-source-message`:
   - `chrome-checks-leak-on-sigint`: Ctrl-C (SIGINT) during `tests/ddphotos_page.mjs` or
     `tests/demo_smoke.mjs` leaves Chrome running and both temp folders behind. The `finally` cleanup
     never runs on a signal. Source: SPEC-164 verify.
   - `chrome-checks-hang-without-timeout`: a Chrome that hangs hangs the whole run, because CDP
     calls have no timeout and the `ddphotos-parity` CI job has no `timeout-minutes`. Source: SPEC-164
     verify.

   Run whatever check validates `ideas.yaml` (look in the `justfile`), and quote its line.
4. **Add one finding to STAGE-060's DOM entry** (the `(not yet written) — [M] **Crop on the page: the
   crop mode's DOM.**` entry). Append one sentence: "Found at SPEC-164's verify: a Swap or a key press
   during a drag leaves `drag.start` stale, so the DOM entry decides whether they end the drag or are
   ignored mid-drag." Don't change any other entry's text.
5. **Write `## Reflection (Ship)`.** Outcome-focused: three answers, drawn only from the evidence.
   1. **What would I do differently?** Weigh at least these:
      - The page check's race passed design's prototype, build, a local run and two CI runs, then
        failed on the third. No cycle asked how the check knows it is reading the right document.
        What check, at which cycle, would have caught it? For example, a prompt line saying every wait
        after a navigation must prove which document it is in, or a run against a server that delays
        the body.
      - The orchestrator predicted the stale-page form, and it didn't reproduce. What does that say
        about filing a mechanism nobody has driven?
      - The M4 cell and AC-4 each claimed more than the code does. Design drove M4 but tabled the red
        set it expected, not the one it got.
      - Punch list 1 ran its negative control on the working copy, not a scratch copy.
   2. **Does any template, constraint or decision need updating?** Candidates:
      - "a headless check waits for the navigated document before its first poll";
      - "a mutation cell records the red set as driven, never as expected";
      - the proposals from SPEC-161's and SPEC-163's reflections, now with this spec's evidence.

      Check AGENTS §15–16, HANDOFF, `guidance/`, and the reflections of SPEC-160, SPEC-161 and
      SPEC-163 first, and say which already covers each. **Propose; don't apply.**
   3. **A follow-up spec?** Steps 3 and 4 file the known gaps. File nothing else unless the evidence
      names a gap they don't cover.
6. **Cost.** Every session needs an entry in `cost.sessions`: frame, design, build, verify,
   punchlist, punchlist-2 and your own `ship`.
   - **Re-measure build**, which it took before its last commits; **punchlist**, taken before its docs
     commit; and **punchlist-2**, which excludes its commit, push and CI watch.
   - **Measure verify** and add its entry. It reported $9.18 (Opus), and the cost block has no verify
     entry yet.
   - **Re-measure frame and design** only if their notes say they were taken early. Both say they
     were taken after the push.
   - **The rulings and the merge are the maintainer's**, so they get no entry.
   - **Find each transcript by its marker**, the prompt's first line on `main`:
     - `SPEC-164 — FRAME prompt`
     - `SPEC-164 — DESIGN prompt`
     - `SPEC-164 — BUILD prompt`
     - `SPEC-164 — VERIFY prompt`
     - `SPEC-164 — PUNCH LIST prompt`
     - `SPEC-164 — PUNCH LIST 2 prompt`

     Use the transcript whose **first user message** holds the marker, because the orchestrator's
     transcript contains every marker. Note that `PUNCH LIST prompt` is also a prefix of
     `PUNCH LIST 2 prompt`, so match the whole first line. If a cycle has two such transcripts,
     measure both and record one entry with their sum, and its note names both.
   - **Dedup by `.message.id`.** Count the three input counts once per id, and take the **MAX**
     `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
   - **Price** at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation
     is ×1.25 the input rate and cache_read ×0.10.
   - **Recompute `cost.totals`** from every entry, your own included. Measure your own entry **after**
     your push. Run `just cost-audit` and quote the line that names SPEC-164.
7. **Mark ship `[x]`** with the merge SHA, PR #274, the date and the total cost.
8. **Advance and archive.** Run `just advance-cycle SPEC-164 ship`, then `just archive-spec SPEC-164`.
   - Read STAGE-060's SPEC-164 entry back. It should read
     `- [x] **SPEC-164** (merged <date>, PR #274) — [M] …`, in the style of the other shipped entries.
     Fix it by hand if not, and touch no other entry beyond step 4.
   - Recount the `**Count:**` line with the four greps it names, and show them. Today it reads
     `4 resolved (3 shipped specs, 1 folded) / 1 active / 5 pending`. Expect
     `5 resolved (4 shipped specs, 1 folded) / 0 active / 5 pending`, unless another session changed
     the stage meanwhile.
   - ⛔ Don't change STAGE-060's `status` or its reflection.
9. **Commit to `main` directly** (AGENTS §13).
   - Commit with
     `git commit -s -m "ship(SPEC-164): the crop state model — corrected, reflected, archived"` and
     the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is
     mandatory.
   - Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`. On a reject,
     rebase and push again. A "Changes must be made through a pull request" notice can still mean the
     push landed, so check with `git ls-remote origin refs/heads/main`.
   - ⛔ **If a permission check refuses the push, stop and report.** Name the commits and the command
     for the maintainer to run, and don't claim the push happened.
   - Read back `git show --stat HEAD`.
   - Then measure your own cost. If it moved, amend nothing: push one more commit that updates only
     your `ship` entry and `cost.totals`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, HANDOFF, templates or any DEC. In `guidance/`, edit only step 3's two
  `ideas.yaml` entries.
- Another session may push to `main` while you work. Rebase; never force.
- **Never poll CI,** and never write your own wait loop.
- A piped command reports the pipe's exit code.
- zsh doesn't word-split unquoted parameters, and it reads `$R:r…` as a modifier, so brace it
  (`${R}:`).
- `rtk` can truncate `grep`, so count with `/usr/bin/grep` or Python.
- Write scratch files only in your session's scratchpad, never `/tmp`.
- **Budget ~40 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-164 ship: archived — <commit SHA on main>`. Then:
- the merge SHA, date and merged head;
- step 2's changes, one line each, with the design sentence, AC-4 and M4 quoted as they now read;
- every 153 → 157 change, and every 153 you left with its reason;
- the two `ideas.yaml` entries' ids and the validation line;
- the DOM entry's added sentence;
- STAGE-060's SPEC-164 line as it now reads, and the recounted `**Count:**` line with its greps;
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- any template, constraint or DEC change you propose;
- your measured cost after your last push: input, cache_creation, cache_read, output, total tokens and
  dollars, each on its own line.
