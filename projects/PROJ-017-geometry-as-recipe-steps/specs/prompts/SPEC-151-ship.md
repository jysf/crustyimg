# SPEC-151 — SHIP prompt: `pad` as a recipe step

Cycle: **ship**. Bookkeeping only. Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-151-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables**: the last two reports lost lines in transit.

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. ⛔ **Proceed only if both hold.** Otherwise stop and report.
   - PR #255 is merged: `gh pr view 255 --json state,mergeCommit,mergedAt`. It was merged at
     `f1c521b` when this was written; record the SHA and the `mergedAt` date you see.
   - The timeline
     (`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-151-pad-as-a-recipe-step-timeline.md`)
     has verify `[x]` (punch list) and punch list `[x]`.
3. Confirm the merge brought the build's work onto `main`: `src/operation/pad.rs` exists,
   `tests/pad.rs` has `apply_reads_the_border_as_f64`, and the spec's `cost.sessions` has `build`
   and `punchlist` entries. If not, stop and report.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full: Frame, Design (its six corrections), the mutation table, `## Build Completion`
  including `### Punch list (verify)`, and the cost block.
- The timeline, all of it. Its verify entry is the orchestrator's summary of the verify report.
- STAGE-058's `## Spec Backlog`, all of it.
- `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: rule 7 and trap 17.
- SPEC-147's `## Reflection (Ship)` (in `specs/done/`): its first answer asked whether design should
  drive the cheap mutation rows. SPEC-151 is the first spec where design did.

## The facts to reflect on (from build, verify and the punch list)
- **The thesis held, and verify proved it with its own oracles.** The canvas matched an exact-fraction
  oracle written from the spec's prose over 332,800 exhaustive and 200,000 random cases; pixels matched
  ImageMagick at AE 0 in 12 cases, 16-bit samples compared one by one; all 37 load messages and 5 apply
  reasons matched by script, with a planted edit caught. No existing corpus output changed.
- **The frame's canvas rule was wrong, and design caught it by driving.** "The long side is kept"
  gives a square image padded to `16:9` a canvas smaller than the image. Design restated it per axis
  and swept it.
- **Design drove 12 of the 14 mutation rows on a prototype, and build found no wrong cell.** Compare
  SPEC-147, where a reasoned table was wrong in two cells. Design cost $13.61, SPEC-147's design less.
- **The one "expected all green" row was reasoned, and verify refuted it.** M14 (`f32` read for
  `border`) said no `f32` split exists in the load range. `0.01` on a 50 px edge is exactly ½ px:
  `f64` gives 1 px, `f32` 0 px. The canvas got an exhaustive sweep; the border's `f32` question got
  none. The punch list pinned it (`apply_reads_the_border_as_f64` and two unit rows); M14 now goes red.
- **Two counts in the spec were wrong**: the build's "32 files" (31, spec included) and the design's
  "three `orientation_of` callers" (`main` had two).
- **The build left a CI wait loop running for 3.5 hours.** It put the run id and SHA in one
  variable; zsh didn't split it, so `gh run view` failed forever. The punch-list prompt then said to
  use `gh pr checks --watch` only.
- **Verify's out-of-spec finding:** a padded `Rgba32F` canvas at the area cap is 2 GiB, but `resize`
  already reaches 2 GiB from any input on `main`, so it isn't new to `pad`.

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **Read the timeline's `merge` line**: the orchestrator marked it `[x]` with the SHA. If the SHA
   differs from what `gh` reports, fix it and say so.
3. **File verify's optional cleanup** as **one** STAGE-058 backlog entry, `[S]`, to be folded into
   the next STAGE-058 spec's build: no test pins `resize`'s cap message text
   (`resize_oversize_is_typed_error`, `resize_apply_percent_rejects_oversized_output` and
   `apply_recipe_with_oversized_resize_exits_1` match only the error kind or exit 1). The caps are now
   shared with `pad`, so a `contains` on both resize lines guards the hoisted constants. Quote both
   lines as the binary prints them today.
4. **The 2 GiB finding.** Search `guidance/`, `decisions/` (DEC-095), the `MAX_AREA` comment and every
   stage backlog for an entry on a byte-based area cap. If one covers it, add one line naming `pad` as
   a second route and say where. If none does, **don't file it**; say so and recommend where it
   belongs (the orchestrator decides).
5. **The two folded entries** ("SPEC-147's verify cleanups that need a PR…" and "State when an op uses
   the refusal hook…") shipped inside SPEC-151. Mark each `[x]` the way STAGE-058 marked its earlier
   folded docs entry (find it by content), naming SPEC-151 and PR #255.
6. **Write `## Reflection (Ship)`**, outcome-focused, three answers drawn only from the evidence:
   1. **What would I do differently?** Weigh at least: design driving the mutation rows (did the
      $13.61 buy the correct table, and would driving M14 have caught the split?); a "no case exists"
      claim reasoned rather than searched (trap 17, again); counts in a spec written from memory.
   2. **Does any template, constraint or decision need updating?** For example: a mutation row
      marked "expected all green" must name the search that found no case; build prompts forbid
      hand-written CI wait loops; SPEC-147's proposal to promote rule 7 and trap 17. Check HANDOFF,
      `guidance/` and SPEC-147's reflection first, and say which already covers each. **Propose it;
      don't apply it.**
   3. **A follow-up spec?** Beyond steps 3 and 4, file **one** more small entry only if the evidence
      names a gap no entry covers, or say why none.
7. **Cost.** Every session's entry must be in `cost.sessions`: frame, design, build, verify,
   punchlist, and your own `ship`.
   - **verify is not recorded anywhere** (it was read-only). Its paste lost the input and
     cache_creation lines; it reported cache_read 12,043,800, output 56,820, total 12,295,899, about
     **$8.66**. Measure it from its transcript in full and record that.
   - Re-measure any entry whose note says it was measured before its session's final turns (build and
     punchlist say so; read frame and design too). Markers are each prompt's title on `main`
     (`SPEC-151 — FRAME prompt`, `— DESIGN prompt`, `— BUILD prompt`, `— VERIFY prompt`,
     `— PUNCH LIST prompt`); read each prompt's first line to confirm. Use the transcript whose
     **first user message** holds the title; the orchestrator's transcript also contains every title.
   - Dedup by `.message.id`: the three input counts once per id, the **MAX** `output_tokens`.
     Include any subagent transcripts under the session's `subagents/` folder. Price at the model in
     `.message.model`: Sonnet $3/$15, Opus $5/$25, cache_creation ×1.25, cache_read ×0.10.
   - Recompute `cost.totals` from every entry (today it reads 0). Run `just cost-audit` and quote the
     line that names SPEC-151.
8. **Mark ship `[x]`** with the merge SHA, PR #255, the date and the total cost.
9. **Advance and archive.** `just advance-cycle SPEC-151 ship`, then `just archive-spec SPEC-151`.
   - Read STAGE-058's SPEC-151 entry back. It should read `- [x] **SPEC-151** (merged <date>, PR
     #255) — …`. Fix it by hand if not.
   - Touch no other entry except steps 3–5's. Recount the `**Count:**` line with greps you just ran
     (today `4 shipped / 1 active / 10 pending`), and show the greps.
10. **Check what the tooling sees.** `just status`, `just backlog`, `just roadmap`. They show only
    the active project, PROJ-011, so say so rather than treat a missing SPEC-151 as an error.
    ⛔ Do not run Stage Ship or change STAGE-058's status; the orchestrator decides that.
11. **Commit to `main` directly** (AGENTS §13):
    `git commit -s -m "ship(SPEC-151): pad as a recipe step — reflected, archived"`,
    trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is mandatory.
    Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`; on a reject,
    rebase and push again. Read back `git show --stat HEAD`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, `docs/`, DECs, HANDOFF or templates; propose in your report. The spec's
  body changes only by the reflection and the cost block.
- Another session (PROJ-011, SPEC-149) may push to `main` while you work. Rebase; never force.
- A piped command reports the pipe's exit code. zsh does not word-split unquoted parameters.
  `rtk` can truncate `grep`, so count with `/usr/bin/grep` or Python.
- **Budget ~35 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-151 ship: archived — <commit SHA on main>`. Then:
- the merge SHA and date;
- the backlog entry filed in step 3, quoted;
- step 4's finding: the entry you extended, or "none covers it" and where you recommend it go;
- the two folded entries as they now read;
- the three reflection answers, one line each;
- any further follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- the recounted `**Count:**` line;
- any template, constraint or DEC change you propose;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on
  its own line.
