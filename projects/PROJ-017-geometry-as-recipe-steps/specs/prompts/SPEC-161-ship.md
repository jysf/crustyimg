# SPEC-161 — SHIP prompt: crop on the ddphotos edit page, the recipe model and the 4x replay

Cycle: **ship**. Bookkeeping, plus folding verify's cell corrections into the spec on `main`.
DEC-118 was written and corrected on the PR, so no DEC changes here. Fresh CLI session, own
worktree. Sonnet is enough:

```
just wt docs/spec-161-ship -- --model sonnet
```

The maintainer pastes this prompt into a fresh session and will paste your final message back to the
orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

## Step 0 — check the merge, then get onto current `main`
1. Run `git fetch origin && git checkout --detach origin/main`.
2. ⛔ **Go on only if all of these hold.** Otherwise, stop and report.
   - **PR #273 is merged.** Run `gh pr view 273 --json state,mergeCommit,mergedAt,headRefOid`. The
     orchestrator saw merge `409f492`, merged head `8d1f1cb` (an update-branch merge of `main` onto
     the punch-listed head `a0bc1a9`), at 2026-11-27T00:11:38Z. Confirm that
     `git log --first-parent --no-merges --oneline a0bc1a9..8d1f1cb` lists nothing and
     `git show --remerge-diff --format= 8d1f1cb` is empty.
   - **The timeline**
     (`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-161-crop-on-the-ddphotos-edit-page-the-recipe-model-and-the-4x-replay-timeline.md`)
     has build, verify and punch list all `[x]`.
   - `decisions/DEC-118-…md` and `tests/ddphotos_upscale.py` exist on `main`, and the spec's
     `cost.sessions` has `build` and `punchlist` entries.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- **The spec in full**, especially the Frame and both sets of rulings, Design's nine corrections,
  the ACs, the Failing Tests, the mutation table, `## Build Completion` with its
  `### Punch list (verify)` subsection, and the cost block.
- **The timeline, all of it.** Its verify and punch-list entries summarise those reports; the details
  you need are below.
- **DEC-118** on `main`, as the punch list left it.
- **STAGE-060's `## Spec Backlog`**, all of it, and its `**Count:**` line
  (`projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md`).
- **SPEC-163's `## Reflection (Ship)`** (in `specs/done/`), for its three proposals, none applied.

## The facts to reflect on (frame to punch list)
- **Frame (Opus, $6.34).** It split the entry: this half is what Node can test (the crop model, the
  upright size, the parity rows, the 4× replay); the crop mode is the next entry. It drove the
  premise and found that `aspect` misses at 4×, so it recommended that presets emit a box. The
  maintainer reversed that: a preset emits `aspect`, with the orientation key, which moved SPEC-163
  ahead of this spec.
- **Design (Opus, $11.58).** It drove the bounds on an independent model (aspect worst 1 px,
  exhaustive over 273,365,360 cases) and the mutation table on a prototype. It made nine
  corrections to the frame, including that a box isn't exact at a resize tie, and moved the
  worker's load into `recipe.js` as `loadPhoto` so the harness tests the production code (trap 14).
  Two calls went to the maintainer, and both were accepted as designed.
- **Build (Sonnet, $6.13, measured before its final cost commit).**
  - 13 of 13 ACs and CI green, but the ACs were left unticked and the three reflection answers
    left as `<answer>`.
  - M1–M11 and trap 14 matched the table's red sets. Two reasons differed from the design: N4 (four
    rows fail on `max`, not `dims`) and M4 (three rows throw rather than read a wrong size).
  - Its temp-folder accounting was complete: system temp `ddphotos-*` 5 before and 5 after.
- **Verify (Opus, $7.60): PUNCH LIST at `f9c81ae`, no claim items, no re-verify.**
  - Its own model: all 52 literals and 22,908 grid cases match; 738 cases on the real engine.
  - The 1,854,856-box readback reproduced independently.
  - N4: the build was right; C26 is natively 600×482, not 601×482.
  - M4: C27 also throws, and no row showed the stored-size bug as a wrong mark (trap 13).
  - Two mutants the table lacked: dropping `Math.max(1, …)` stayed green; `x + width` one ulp past 1
    went red on `model-fractions`, C28 and R13.
  - Wording punch items in DEC-118 rules 1 and 4 and in one comment.
- **Punch list (Sonnet, $2.40 reported, $1.95 in the spec).** It added `model-min`, C36 (an R box
  that M4 turns red as 239×400, not a throw; it joined N1's list), and counted summary lines with
  `A-count`. It ticked the ACs and filled the reflection, marked as written at punch list.
- **The orchestrator marked the PR ready** after the punch list, because verify had asked for no
  re-verify.

## Do, in order
1. **Mark ship `[~]`** in the timeline, and add a `merge` line above it:
   `- [x] **merge** (<date>, maintainer) — PR #273 as <merge SHA>, head <head>.` Check it against
   `gh`.
2. **Fold verify's corrections into the spec**, and mark each change `(corrected at ship)` in place.
   Find each by content.
   - **M4's cell:** the red set is 15, 16, C14, C15, C16, C27, C30, C36, R7, R12 and R15. C27, C30,
     R12 and R15 fail as "threw" (the engine refuses the stored-size fractions); 15, 16, C14–C16
     and R7 fail on `planEdit predicts …`; C36 fails on its size (239×400 against 400×300). Drop
     "C30 reads 560×1071 with mark 43, uncapped".
   - **Build Completion's M4 line** names C27 among the `planEdit` failures. Add one line saying
     verify found it throws. Keep the original as history.
   - **N4's cell:** C26, C31, C33 and C34 fail on the comparator's `max`, at 600×482 for C26; only
     C32 fails on `dims` (801×643). Drop "C26 natively 601×482".
   - **M5's cell:** add that R14 fails as "threw" (the watermark is refused on 267×717).
   - **Two new mutation rows**, marked driven by verify:
     - `planEdit`'s two `Math.max(1, …)` dropped: green at `f9c81ae`; since the punch list,
       `model-min` only.
     - a box's `x + width` one ulp past 1: `model-fractions` (2,671 boxes), C28 and R13 as engine
       refusals.
   - **N1's list** in the Failing Tests gains C36 at the end. Check AC-10's wording still holds.
   - **The crop-row count:** where the spec's prose says 35 crop rows or C1–C35 as the whole table,
     add C36 (corrected at ship). Leave the design's historical numbers in "What was driven" alone.
   - **AC-12:** brotli 833,619 B is the Mac's figure; CI reads 836,219 B, the same as `main`, so
     0 B of growth either way. And TABLE's hunk also adds three fields in its `.map` (`family`,
     `crop`, `form`); its row literals are unchanged.

   Nothing else in the spec's body changes, apart from the reflection and the cost block.
3. **Write `## Reflection (Ship)`.** Outcome-focused: three answers, drawn only from the evidence.
   1. **What would I do differently?** Weigh at least these:
      - The design tabled two failure reasons it hadn't checked (N4's `dims`, M4's 560×1071), and
        verify found a mutant (`Math.max` dropped) that nothing caught. Should a design that
        tables a mutation also quote each red row's failure message, as trap 13 asks of the
        build?
      - The build ticked nothing and left its reflection empty, though its prompt asked for the
        reflection. What would make that hard to miss?
      - This spec's temp-folder instructions were followed in all three sessions, and none leaked
        into the system temp folder. Is that worth keeping as a standing instruction for any
        harness that keeps failed runs?
   2. **Does any template, constraint or decision need updating?** Weigh SPEC-163's three
      proposals with this spec's evidence:
      - a spec that adds parity rows drives one mutant into both sides (here every mutant was in
        `recipe.js`, which both sides read, so only literals could catch any of them);
      - write square rows in both orders (C18/C19 caught M3 that way);
      - prompts say what to do when a push is refused (this spec's prompts did; no push was
        refused).

      Check AGENTS §15–16, HANDOFF, `guidance/` and the reflections of SPEC-160 and SPEC-163 first,
      and say which already covers each. **Propose; don't apply.**
   3. **A follow-up spec?** None is expected. The crop-mode entry already says its CLI panel must
      tell the user to run the recipe on a file at least as large as the copy; that covers verify's
      rule-5 finding. The orchestrator has already updated `aspect-orientation-loose-ends` in
      `guidance/ideas.yaml` (the summary line resolved; the N3 temp folder added). File nothing else
      unless the evidence names a gap neither covers.
4. **Cost.** Every session needs an entry in `cost.sessions`: frame, design, build, verify,
   punchlist, and your own `ship`.
   - **Re-measure build** (taken before its final cost commit) and **punchlist** (taken before its
     commit and CI watches; it reported 5,498,685 tokens, $2.40 afterwards).
   - **Add verify**: it reported 10,710,415 tokens, $7.60 (Opus). Measure it yourself.
   - **Re-measure design** only if its note says it was taken before its push, and **frame** only if
     its note says it was taken early.
   - **The rulings and the merge are the maintainer's.** No entry.
   - **Find each transcript by its marker**, the prompt's title on `main`: `SPEC-161 — FRAME prompt`,
     `SPEC-161 — DESIGN prompt`, `SPEC-161 — BUILD prompt`, `SPEC-161 — VERIFY prompt` and
     `SPEC-161 — PUNCH LIST prompt`.
     - Read each prompt's first line to confirm its title.
     - Use the transcripts whose **first user message** holds the title, because the orchestrator's
       transcript contains every title.
     - If a cycle has two such transcripts, measure both and record one entry with their sum; its
       note names both.
   - **Dedup by `.message.id`.** Count the three input counts once per id, and take the **MAX**
     `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
   - **Price** at the model in `.message.model`: Sonnet $3/$15, Opus $5/$25. cache_creation is ×1.25
     the input rate and cache_read ×0.10.
   - **Recompute `cost.totals`** from every entry, your own included. Measure your own entry **after**
     your push. Run `just cost-audit` and quote the line that names SPEC-161.
5. **Mark ship `[x]`** with the merge SHA, PR #273, the date and the total cost.
6. **Advance and archive.** Run `just advance-cycle SPEC-161 ship`, then `just archive-spec SPEC-161`.
   - Read STAGE-060's SPEC-161 entry back. It should read
     `- [x] **SPEC-161** (merged <date>, PR #273) — [M] …`, in the style of the other shipped
     entries. Fix it by hand if not. Touch no other entry.
   - Recount the `**Count:**` line with greps you just ran, and show them, as the line itself
     describes. Today it reads `3 resolved (2 shipped specs, 1 folded) / 1 active / 4 pending`.
     Expect `4 resolved (3 shipped specs, 1 folded) / 0 active / 4 pending`, unless another session
     changed the stage meanwhile.
   - ⛔ Don't change STAGE-060's `status` or its reflection. The stage isn't done: four entries are
     still unwritten.
7. **Commit to `main` directly** (AGENTS §13).
   - Commit with
     `git commit -s -m "ship(SPEC-161): crop on the edit page, the recipe model and the 4x replay — reflected, archived"`
     and the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is
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
- Don't edit code, tests, CI, DECs, docs, HANDOFF, `guidance/` or templates.
- Another session may push to `main` while you work. Rebase; never force.
- **Never poll CI,** and never write your own wait loop.
- A piped command reports the pipe's exit code.
- zsh does not word-split unquoted parameters, and it reads `$R:r…` as a modifier, so brace it
  (`${R}:`).
- `rtk` can truncate `grep`, so count with `/usr/bin/grep` or Python.
- Write scratch files only in your session's scratchpad, never `/tmp`.
- **Budget ~40 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-161 ship: archived — <commit SHA on main>`. Then:
- the merge SHA, date and merged head, with the two merge checks;
- step 2's changes, one line each;
- STAGE-060's SPEC-161 line as it now reads, and the recounted `**Count:**` line with its greps;
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- any template, constraint or DEC change you propose;
- your measured cost after your last push: input, cache_creation, cache_read, output, total tokens and
  dollars, each on its own line.
