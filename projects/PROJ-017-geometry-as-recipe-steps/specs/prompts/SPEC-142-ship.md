# SPEC-142 — SHIP prompt: the ddphotos parity harness

Cycle: **ship**. Bookkeeping only. Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-141-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. ⛔ **Proceed only if both hold.** Otherwise stop and report.
   - PR #234 is merged: `gh pr view 234 --json state,mergeCommit`. Record the merge SHA.
   - The timeline
     (`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-142-ddphotos-page-native-vs-wasm-parity-harness-timeline.md`)
     has re-verify 2 `[x]` APPROVED.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full: Frame, Design (Calls 1–7), the mutation table, `## Build Completion` with both
  punch-list subsections, and the cost block.
- The timeline, all of it.
- STAGE-060's `## Spec Backlog`, all of it.

## The facts to reflect on (from verify and the two punch lists)
- **Parity is bounded, not exact (DEC-110, rule B):** dimensions exact, no-resample rows 1, 3, 9
  and 11 exact, resampling rows within 1/255 on at most K pixels (cap 78). K = 1, measured on
  aarch64 and on three x86 CPUs (AMD EPYC 9V45, AMD EPYC 7763, Intel Xeon 8370C), all agreeing.
  Native hashes matched across the Mac and the Xeon on all 16 rows; the 1-pixel difference sits on
  the wasm side.
- **The cause is measured:** native `libm::powf` gives K = 0 on aarch64; both `powf` sites
  contribute. Filed as `native-wasm-resize-exactness` in `guidance/ideas.yaml`; don't file it
  again. K is guarded by review only; the summary line makes a raised K visible.
- **The build introduced a CI hole:** a crashing `scripts/ci-code-paths.sh` made the `changes`
  step report docs-only, so heavy jobs would skip and show green. Verify found it by driving the
  crash; punch list 1 fixed it.
- **The preview recipe had no golden text** until punch list 1 added row 12 to P5.
- **The summary line read `0 failed` while a table check failed and the run exited 1.** Two
  sessions tripped on it; punch list 2 fixed it.
- **One spec sentence misattributed a CPU to a run;** punch list 2 corrected it against all three
  logs.
- **Moving the recipe code to `recipe.js` broke the ddphotos test bed's `check.py`,** which read
  `GO_ADVANCE` from `edit.js`. A separate test-bed session fixed it; the merge waited for it.

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **Write `## Reflection (Ship)`**, outcome-focused, three answers drawn only from the evidence:
   1. **What would I do differently?** Weigh at least: a CI classifier path added without a
      crash test; a summary line that undercounted failures; a page refactor that a downstream
      test bed depended on, found only at build.
   2. **Does any template, constraint or decision need updating?** For example: "a CI step that
      branches on a command's output assigns it first, then validates it". Check the existing
      HANDOFF traps and `guidance/` first; if one covers it, say which. **Propose it; don't
      apply it.**
   3. **A follow-up spec?** Read STAGE-060's backlog first. Weigh the re-verify's leftovers: the
      three P5 checks all print `P5`, and a code comment says "P1-P5" with no P1. File **one**
      small backlog entry, or say why none.
3. **Cost.** Every session's entry must be in `cost.sessions`: frame, design, design-amend, build, verify,
   punchlist, re-verify, punchlist-2, re-verify-2 (the last measured $1.39, Opus: input 46,
   output 7,772, cache_creation 60,252, cache_read 1,627,815), and your own `ship`.
   - Find each transcript **by content**: `/usr/bin/grep -l "<marker>" ~/.claude/projects/*/*.jsonl`.
     Markers: `SPEC-142 — DESIGN`, `SPEC-142 — BUILD prompt`, `SPEC-142 — VERIFY prompt`,
     `SPEC-142 — PUNCH LIST prompt`, `SPEC-142 — PUNCH LIST 2 prompt`,
     `RE-VERIFY prompt: the punch list on the ddphotos parity harness`,
     `RE-VERIFY 2 prompt: the corrected CPU sentence`. Read each prompt's first line on `main` to
     confirm its exact title. A marker can match this orchestrator's own
     transcript too (it wrote the prompts): use the transcript whose **first user message** contains
     it.
   - Re-measure any entry that says it was measured before its session's final turn (punchlist-2 says so), and add any
     missing one. Dedup by `.message.id`; the three input counts once per id, the **MAX**
     `output_tokens`. Price at the model in `.message.model`: Sonnet $3/$15, Opus $5/$25,
     cache_creation ×1.25, cache_read ×0.10.
   - Recompute `cost.totals` from every entry. Run `just cost-audit` and quote the line that names
     SPEC-142.
4. **Mark ship `[x]`** with the merge SHA, PR #234, the date and the total cost.
5. **Advance and archive.** `just advance-cycle SPEC-142 ship`, then `just archive-spec SPEC-142`.
   Read STAGE-060's SPEC-142 entry back: it should read `- [x] **SPEC-142** (merged <date>, PR
   #234) — …`. Fix it by hand if not; touch no other entry. Recount the `**Count:**` line with a
   grep you just ran (today `0 shipped / 0 active / 4 pending`).
6. **Check what the tooling sees.** `just status`, `just backlog`, `just roadmap`. ⛔ Do not run
   Stage Ship or change STAGE-060's status; the orchestrator decides that.
7. **Commit to `main` directly** (AGENTS §13):
   `git commit -s -m "ship(SPEC-142): the ddphotos parity harness — reflected, archived"`, trailer
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
First line: `SPEC-142 ship: archived — <commit SHA on main>`. Then:
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- the recounted `**Count:**` line;
- any template, constraint or DEC change you propose;
- your measured cost.
