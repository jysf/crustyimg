# SPEC-141 — SHIP prompt: the recipe-output guard

Cycle: **ship**. Bookkeeping only. Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-141-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. ⛔ **Proceed only if both hold.** Otherwise stop and report.
   - PR #233 is merged: `gh pr view 233 --json state,mergeCommit`. Record the merge SHA.
   - The timeline
     (`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-141-no-existing-recipe-output-changes-timeline.md`)
     has re-verify 2 `[x]` APPROVED.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full: Frame, Design (Calls 1–7), the mutation table, `## Build Completion` with both
  punch-list subsections, and the cost block.
- The timeline, all of it.
- STAGE-058's `## Spec Backlog`, all of it.

## The facts to reflect on (from verify and the two punch lists)
- **Goldens are per target**, and what they really pin is each runner image's libm `powf`, not
  only OS and architecture. CPU dispatch was ruled out. A red on one target alone, with no code
  change, means libm drift; the README says to re-bless that target alone, in its own commit.
- **The first build couldn't be blessed from a CI log** for a new recipe: a missing golden
  panicked before any dump printed. Punch list 1 fixed it.
- **Punch list 1's own fix had no test on the real path.** FT-9 drove the report builder through a
  closure, so the old panic put back verbatim stayed green. Re-verify caught it by mutation; punch
  list 2 pinned `golden_path_for` with an exact-path `Ok` assertion.
- **A new recipe still prints no dump until three literals are bumped** (`EXPECTED_DIMS`, `ROWS`,
  the `42`). The README orders the bumps first, and the re-verify drove that order green. Left as
  is, by ruling.
- **The libm remedy is filed** as `native-wasm-resize-exactness` in `guidance/ideas.yaml`
  (`libm::powf`, measured K = 0 on aarch64). Don't file it again.

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **Write `## Reflection (Ship)`**, outcome-focused, three answers drawn only from the evidence:
   1. **What would I do differently?** Weigh at least: a fix proved through a test double, not the
      production path (FT-9's closure); and two verify rounds spent on a guard whose bless path
      for a *new* recipe wasn't driven at design.
   2. **Does any template, constraint or decision need updating?** For example, a lesson: "a test
      for a fixed path must call the production function, not a stand-in". Check the existing
      HANDOFF traps and `guidance/` first; if one covers it, say which. **Propose it; don't
      apply it.**
   3. **A follow-up spec?** Read STAGE-058's backlog first. Weigh the unbumped-literal no-dump
      behaviour and the stray-file check's ordering. File **one** small backlog entry, or say why
      none.
3. **Cost.** Every session's entry must be in `cost.sessions`: frame, design, build, verify,
   punchlist, re-verify, punchlist-2, re-verify-2 (the last measured $0.82, Opus: input 26,
   output 4,012, cache_creation 53,200, cache_read 772,116), and your own `ship`.
   - Find each transcript **by content**: `/usr/bin/grep -l "<marker>" ~/.claude/projects/*/*.jsonl`.
     Markers: `SPEC-141 — BUILD prompt`, `SPEC-141 — VERIFY prompt`, `SPEC-141 — PUNCH LIST prompt`,
     `SPEC-141 — PUNCH LIST 2 prompt`, `RE-VERIFY prompt: the punch list on the recipe-output guard`,
     `RE-VERIFY 2 prompt: the real golden-file lookup`. A marker can match this orchestrator's own
     transcript too (it wrote the prompts): use the transcript whose **first user message** contains
     it.
   - Re-measure any entry that says it was measured before its session's final turn, and add any
     missing one. Dedup by `.message.id`; the three input counts once per id, the **MAX**
     `output_tokens`. Price at the model in `.message.model`: Sonnet $3/$15, Opus $5/$25,
     cache_creation ×1.25, cache_read ×0.10.
   - Recompute `cost.totals` from every entry. Run `just cost-audit` and quote the line that names
     SPEC-141.
4. **Mark ship `[x]`** with the merge SHA, PR #233, the date and the total cost.
5. **Advance and archive.** `just advance-cycle SPEC-141 ship`, then `just archive-spec SPEC-141`.
   Read STAGE-058's SPEC-141 entry back: it should read `- [x] **SPEC-141** (merged <date>, PR
   #233) — …`. Fix it by hand if not; touch no other entry. Recount the `**Count:**` line with a
   grep you just ran (today `0 shipped / 0 active / 7 pending`).
6. **Check what the tooling sees.** `just status`, `just backlog`, `just roadmap`. ⛔ Do not run
   Stage Ship or change STAGE-058's status; the orchestrator decides that.
7. **Commit to `main` directly** (AGENTS §13):
   `git commit -s -m "ship(SPEC-141): the recipe-output guard — reflected, archived"`, trailer
   `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is mandatory. Push
   with `git fetch origin && git rebase origin/main && git push origin HEAD:main`; on a reject,
   rebase and push again. Read back `git show --stat HEAD`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, DECs, `docs/`, HANDOFF or templates; propose in your report.
- A piped command reports the pipe's exit code. zsh does not word-split unquoted parameters.
  `rtk` can truncate `grep`, so count with `/usr/bin/grep` or Python.
- **Budget ~35 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-141 ship: archived — <commit SHA on main>`. Then:
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- the recounted `**Count:**` line;
- any template, constraint or DEC change you propose;
- your measured cost.
