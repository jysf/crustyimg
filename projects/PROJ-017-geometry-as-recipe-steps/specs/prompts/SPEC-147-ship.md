# SPEC-147 — SHIP prompt: `rotate` by quarter turns and `flip`, as recipe steps

Cycle: **ship**. Bookkeeping only. Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-147-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. ⛔ **Proceed only if both hold.** Otherwise stop and report.
   - PR #252 is merged: `gh pr view 252 --json state,mergeCommit`. It was merged at `b90e568`
     when this was written; record the SHA you see.
   - The timeline
     (`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-147-rotate-by-quarter-turns-and-flip-as-recipe-steps-timeline.md`)
     has verify `[x]` APPROVED.
3. Confirm the merge brought the build's work onto `main`: `src/operation/rotate.rs` and
   `src/operation/flip.rs` exist, and the spec's `cost.sessions` has a `build` entry. If not, stop
   and report.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full: Frame, Design, the mutation table, `## Build Completion` and the cost block.
- The timeline, all of it. Its verify entry is the orchestrator's summary of the verify report.
- DEC-113, all of it.
- STAGE-058's `## Spec Backlog`, all of it.
- `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: rule 7 and trap 17.

## Recover the verify report first
The verify report reached the orchestrator cut off. Two parts are missing: its cost table's
`input` and `cache_creation` rows, and the end of an out-of-spec finding that began "wording,
STAGE-058 (the corpus guard's README). The 'Adding a recipe' table in …".
- Find the verify transcript **by content**:
  `/usr/bin/grep -l "SPEC-147 — VERIFY prompt" ~/.claude/projects/*/*.jsonl`. The marker also
  matches the orchestrator's transcript (it wrote the prompt). Use the one whose **first user
  message** contains the title.
- Read that session's final assistant message and quote the out-of-spec finding in full.
- What it's about, checked on `main`: `tests/fixtures/recipe_corpus/README.md` "Adding a recipe"
  still names the old literals (`; 14]`, `ROWS (48)`, `GOLDEN_LINES (51)`, `the 42`). On `main`
  they are 18, 60, 63 and 54. Confirm that against `tests/recipe_corpus_output.rs` yourself.

## The facts to reflect on (from build and verify)
- **The thesis held.** Every rotate and flip matches ImageMagick (`-rotate N`, `-flop`, `-flip`)
  at AE 0 on 8-bit RGB and 16-bit RGBA, with depth and colour type kept. No existing corpus output
  changed. Verify approved with **no punch items**, the first STAGE-058 spec to do so.
- **Rule 7 worked.** Verify labelled four optional cleanups (one assert, three wording) and said
  none needs a re-verify. Compare SPEC-145, where optional cleanups took two extra rounds at about
  half the build's cost.
- **Trap 17 was applied and showed its limits.** Angle exactness was driven (54 near-multiple
  inputs, all refused). "apply never fails" was driven only over the AC-11 shapes. DEC-113 §3's
  claim that both step orders agree for tags 3 and 8 is reasoned from rotations commuting; only
  tag 6 was driven.
- **The build's mutation table was wrong in two cells** (M8 row 3's failure kind, M10's missing
  `to_toml…` row). Build found both, verify confirmed both. The table was reasoned at design.
- **A count literal table outside the tests drifted.** The corpus README's literal table fell
  behind across SPEC-145 and SPEC-147, and no test reads it.

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **Apply verify's wording cleanups 2 and 3, in the spec only.** Find each by content:
   - Build Completion's "Files this diff touches … 31 files": count the list under it, and make the
     number match (verify counted 32 in the diff). If the list is missing a file, say which one;
     don't add it.
   - The mutation table (`## Failing Tests`): M8's red cell says rows "2 and 3 (message lacks
     `expected 51`)". Make it row 2 wrong message, row 3 wrong verdict (passes under
     `FULL_COUNT=50`, should fail). M10's red cell: add
     `to_toml_is_byte_identical_to_main_over_the_corpus`.
   Change nothing else in the spec's body.
3. **Apply verify's wording cleanup 4 in DEC-113.** In the section on step order (it says tags 3,
   6 and 8 agree because rotations commute), add one sentence: only tag 6 was driven (SPEC-147's
   AC-39 and its verify); 3 and 8 follow from rotations commuting. This is the one DEC edit
   allowed.
4. **Don't apply cleanup 1 or the README finding here.** One is a test change and the other is in
   `tests/fixtures/`, so both need a PR. File them as **one** STAGE-058 backlog entry, `[S]`, to
   be folded into the next STAGE-058 spec's build:
   - `orientation_of` (`tests/rotate_flip.rs`) uses `.ok()?`, so any kamadak-exif error reads as
     "no tag"; only `Err(exif::Error::NotFound(_))` should;
   - the corpus README's "Adding a recipe" literals are stale (quote the old and current values).
   First read the existing entry "Recipe-output guard: report the file-set and line-count failures
   together". If it already covers the README, add to it instead and say so.
5. **Write `## Reflection (Ship)`**, outcome-focused, three answers drawn only from the evidence:
   1. **What would I do differently?** Weigh at least: the mutation table reasoned at design and
      corrected at build (should design drive the cheap rows?); a literal table no test reads;
      whether "apply never fails" should have been stated as a claim over named shapes.
   2. **Does any template, constraint or decision need updating?** For example: a doc that
      restates test literals is checked by a test or cut to "see the test"; rule 7 and trap 17
      held, so promote them from HANDOFF to `guidance/`. Check HANDOFF and `guidance/` first; if
      one covers it, say which. **Propose it; don't apply it.**
   3. **A follow-up spec?** Beyond step 4's entry, file **one** more small entry only if the
      evidence names a gap no entry covers, or say why none.
6. **Cost.** Every session's entry must be in `cost.sessions`: frame, design, build, verify, and
   your own `ship`.
   - **verify is not recorded anywhere** (it was read-only). From the cut-off report: Opus,
     cache_read 9,263,318, output 32,855, total 9,466,001, about **$6.51**. Measure it from its
     transcript in full and record that.
   - Re-measure any entry whose note says it was measured before its session's final turn (design
     says so; read frame and build too). Markers are each prompt's title on `main`
     (`SPEC-147 — FRAME prompt`, `— DESIGN prompt`, `— BUILD prompt`, `— VERIFY prompt`); read each
     prompt's first line to confirm. Use the transcript whose first user message holds the title.
   - Dedup by `.message.id`: the three input counts once per id, the **MAX** `output_tokens`.
     Include any subagent transcripts under the session's `subagents/` folder. Price at the model
     in `.message.model`: Sonnet $3/$15, Opus $5/$25, cache_creation ×1.25, cache_read ×0.10.
   - Recompute `cost.totals` from every entry (today it reads 0). Run `just cost-audit` and quote
     the line that names SPEC-147.
7. **Mark ship `[x]`** with the merge SHA, PR #252, the date and the total cost.
8. **Advance and archive.** `just advance-cycle SPEC-147 ship`, then `just archive-spec SPEC-147`.
   - Read STAGE-058's SPEC-147 entry back. It should read `- [x] **SPEC-147** (merged <date>, PR
     #252) — …`. Fix it by hand if not.
   - Touch no other entry except step 4's. Recount the `**Count:**` line with greps you just ran
     (today `3 shipped / 1 active / 8 pending`), and show the greps.
9. **Check what the tooling sees.** `just status`, `just backlog`, `just roadmap`. They show only
   the active project, PROJ-011, so say so rather than treat a missing SPEC-147 as an error.
   ⛔ Do not run Stage Ship or change STAGE-058's status; the orchestrator decides that.
10. **Commit to `main` directly** (AGENTS §13):
    `git commit -s -m "ship(SPEC-147): rotate and flip as recipe steps — reflected, archived"`,
    trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is
    mandatory. Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`;
    on a reject, rebase and push again. Read back `git show --stat HEAD`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, `docs/`, HANDOFF or templates; propose in your report. The spec's
  body changes only by step 2, and DEC-113 only by step 3's sentence.
- Another session (PROJ-011, SPEC-148) may push to `main` while you work. Rebase; never force.
- A piped command reports the pipe's exit code. zsh does not word-split unquoted parameters.
  `rtk` can truncate `grep`, so count with `/usr/bin/grep` or Python.
- **Budget ~35 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-147 ship: archived — <commit SHA on main>`. Then:
- the merge SHA;
- the recovered out-of-spec finding, quoted in full;
- the three cleanups applied (spec ×2, DEC-113 ×1), each quoted after the change;
- the backlog entry filed or extended in step 4, quoted;
- the three reflection answers, one line each;
- any further follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- the recounted `**Count:**` line;
- any template, constraint or DEC change you propose;
- your measured cost.
