# SPEC-145 — SHIP prompt: `crop` as a recipe step

Cycle: **ship**. Bookkeeping only. Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-145-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. ⛔ **Proceed only if both hold.** Otherwise stop and report.
   - PR #244 is merged: `gh pr view 244 --json state,mergeCommit`. Record the merge SHA.
   - The timeline
     (`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-145-crop-as-a-recipe-step-in-units-that-survive-a-resize-timeline.md`)
     has re-verify 2 `[x]` APPROVED.
3. Confirm the merge brought the build's work onto `main`: `src/operation/crop.rs` exists, and the
   spec's `cost.sessions` has more than the frame and design entries. If not, stop and report.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full: Frame, Design, the mutation table, `## Build Completion` with both punch-list
  subsections, and the cost block.
- The timeline, all of it.
- DEC-112, all of it.
- STAGE-058's `## Spec Backlog`, all of it.

## The facts to reflect on (from build, verify and the two punch lists)
- **The thesis held.** A crop in fractions agrees within one pixel at 1200 px and 6000 px; verify
  drove it on real images with pixels exact against `vips`. No existing corpus output changed.
- **The 0 px check lives in `Crop::apply`, as an op failure, by design.** Mutation M4b moves it
  into the skippable refusal hook: a single input then exits 2, but a batch writes both images
  uncropped and exits 0. So the hook would have hidden a bad crop in a batch, and the design's
  choice is what prevents that.
- **Verify approved with four optional cleanups,** and the maintainer chose to fix them before
  merge. That took two punch lists and two re-verifies: about $5.13, against a $9.37 build.
- **A false maximum came from the orchestrator.** Verify called 0.4 px the "analytic maximum"
  drift. The orchestrator's punch-list prompt passed it on as settled, and the punch list wrote it
  into DEC-112. Re-verify found 0.6 at `f64` near-ties (`width = 0.009583333333333333`: 12 px at
  1200, 57 px at 6000), and that cost the second round. The orchestrator has saved this as a
  memory; it's not a constraint yet.
- **The batch summary says "watermark skipped" for any refusing op.** Already filed in STAGE-058;
  don't file it again. Nor the orientation-following aspect entry, also filed.

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **Apply re-verify 2's two wording cleanups**, in the spec only:
   - The "+0.1" mutation row: add that it's **green at `0.6 + 1e-9`**, so nobody reads "red at 0.6"
     as the bound the test uses.
   - Punch list item 4's account (near lines 1145–1146) says DEC-112's line "now says 0.4,
     asserted by SPEC-145's test". Make it "0.4 for this seed", keeping it as history.
   Find both by content, not line number. Change nothing else in the spec's body.
3. **Write `## Reflection (Ship)`**, outcome-focused, three answers drawn only from the evidence:
   1. **What would I do differently?** Weigh at least: taking optional cleanups as two extra rounds
      that cost about half the build; a derived bound carried forward as settled; whether the
      refusal hook's batch-skip behaviour (M4b) should be stated where ops choose between the
      hook and an apply failure.
   2. **Does any template, constraint or decision need updating?** For example: "a verify prompt
      that carries a session's derived bound marks it as a claim to drive, not a fact", or
      "optional cleanups go in one punch list, with no re-verify unless a claim changed". Check the
      existing HANDOFF traps and `guidance/` first; if one covers it, say which. **Propose it;
      don't apply it.**
   3. **A follow-up spec?** Read STAGE-058's backlog first: two entries from this spec are already
      filed. File **one** new small entry only if the evidence names a gap they don't cover, or
      say why none.
4. **Cost.** Every session's entry must be in `cost.sessions`: frame, design, build, verify,
   punchlist, re-verify, punchlist-2, re-verify-2, and your own `ship`.
   - **re-verify-2 is not recorded anywhere** (it was read-only). Add it: input 62, output 8,850,
     cache_creation 61,819, cache_read 2,125,534, total 2,196,265, Opus, **$1.67**, measured
     mid-session. Then re-measure it from its transcript.
   - Find each transcript **by content**: `/usr/bin/grep -l "<marker>" ~/.claude/projects/*/*.jsonl`.
     Markers are each prompt's title on `main`, for example `SPEC-145 — BUILD prompt`,
     `SPEC-145 — VERIFY prompt`, `SPEC-145 — PUNCH LIST prompt`, `SPEC-145 — PUNCH LIST 2 prompt`,
     `SPEC-145 — RE-VERIFY prompt`, `SPEC-145 — RE-VERIFY 2 prompt`. Read each prompt's first line
     to confirm its exact title. A marker can match this orchestrator's own transcript too (it
     wrote the prompts), and the PUNCH LIST marker is a prefix of PUNCH LIST 2's: use the
     transcript whose **first user message** contains the full title.
   - Re-measure any entry that says it was measured before its session's final turn (frame,
     design and re-verify-2 say so), and add any missing one. Dedup by `.message.id`; the three
     input counts once per id, the **MAX** `output_tokens`. Include any subagent transcripts under
     the session's `subagents/` folder. Price at the model in `.message.model`: Sonnet $3/$15,
     Opus $5/$25, cache_creation ×1.25, cache_read ×0.10.
   - Recompute `cost.totals` from every entry (today it reads 0). Run `just cost-audit` and quote
     the line that names SPEC-145.
5. **Mark ship `[x]`** with the merge SHA, PR #244, the date and the total cost.
6. **Advance and archive.** `just advance-cycle SPEC-145 ship`, then `just archive-spec SPEC-145`.
   - Read STAGE-058's SPEC-145 entry back. It should read `- [x] **SPEC-145** (merged <date>, PR
     #244) — …`. Fix it by hand if not.
   - The `(folded into SPEC-145)` docs entry (`docs/recipes.md` advertised a `crustyimg crop`
     verb): check on `main` that the stale verb is gone, then mark it `[x]` with "(shipped in
     SPEC-145)". If the verb is still there, leave the entry open and say so.
   - Touch no other entry. Recount the `**Count:**` line with greps you just ran (today
     `1 shipped / 1 active / 7 pending`), and show the greps.
7. **Check what the tooling sees.** `just status`, `just backlog`, `just roadmap`. They show only
   the active project, PROJ-011, so say so rather than treat a missing SPEC-145 as an error.
   ⛔ Do not run Stage Ship or change STAGE-058's status; the orchestrator decides that.
8. **Commit to `main` directly** (AGENTS §13):
   `git commit -s -m "ship(SPEC-145): crop as a recipe step — reflected, archived"`, trailer
   `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is mandatory. Push
   with `git fetch origin && git rebase origin/main && git push origin HEAD:main`; on a reject,
   rebase and push again. Read back `git show --stat HEAD`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, DECs, `docs/`, HANDOFF or templates; propose in your report. The
  spec's body changes only by step 2's two cleanups.
- Another session (PROJ-011, SPEC-146) may push to `main` while you work. Rebase; never force.
- A piped command reports the pipe's exit code. zsh does not word-split unquoted parameters.
  `rtk` can truncate `grep`, so count with `/usr/bin/grep` or Python.
- **Budget ~35 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-145 ship: archived — <commit SHA on main>`. Then:
- the merge SHA;
- the two cleanups, each quoted after the change;
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- the folded docs entry's state, and the recounted `**Count:**` line;
- any template, constraint or DEC change you propose;
- your measured cost.
