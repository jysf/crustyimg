# SPEC-133 — SHIP prompt: a watermark refuses to damage an image

Cycle: **ship**. Bookkeeping only: the code is on `main` as `75f82e7`, from PR #206, squash-merged
2026-10-27, including the verify punch-list fix. Fresh CLI session, own worktree; Sonnet is
enough:

```
just wt docs/spec-133-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. Open the spec's timeline:
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-133-watermark-refuses-to-damage-an-image-timeline.md`.
3. ⛔ **Proceed only if** both hold; otherwise stop and report:
   - the re-verify line is `[x]` with ✅ APPROVED;
   - PR #206 is merged (`gh pr view 206 --json state`).
4. **SPEC-134's ship may run before you** and commit to `main`. If the STAGE-050 file conflicts
   when you rebase, keep both sides' changes. Never drop SPEC-134's.

## Read first
- `/AGENTS.md` §4 (cost), §13 (verify and ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full, including `## Build Completion` and `## Verify Readout`.
- The timeline.
- `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`, "Traps this wave paid for" and
  rule 6.
- The STAGE-050 entries for SPEC-133. There are two: one at about line 84 and one at about line
  211.
- DEC-104.

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **Write `## Reflection (Ship)`.** Make it outcome-focused, with three answers drawn from the
   evidence (Build Completion, the verify readouts, the timeline and the merge history). Do not
   invent anything.
   1. **What would I do differently next time?** Facts to weigh:
      - The design's list of tests that would flip was measured on a prototype where a refusal was
        an *error*. Under the real *skip*, four of those tests passed hollow: the mark was skipped
        and they stayed green. Only an "abort on any skip" probe found them.
      - Verify's own mutations found four test gaps, M1, M2, M5 and M7. AC-5's premise was wrong:
        opacity 0.01 never rounds an alpha to 0.
      - The design's surface table missed the glob or directory that resolves to one file. It
        takes the exit-2 path, which was undocumented. SPEC-134 hit the same kind of edge.
      - Re-verify still left three optional test gaps. They are filed, not fixed.
   2. **Does any template, constraint or decision need updating?** Name it, or say none.
      Candidates to weigh:
      - Should the design template ask for the "abort on any skip" style probe whenever a change
        turns an error into a silent fallback?
      - Should a design's surface table always include "one input reached through a glob or
        directory, and stdin"?
   3. **Is there a follow-up spec to write before I forget?** Read STAGE-050's `## Spec Backlog`
      first. Re-verify's items are already filed there: "SPEC-133's three test gaps…" and
      "`watermark` prints absolute paths…". The `--size nan` item is updated. Do not duplicate
      any of them.
3. **Cost.** `cost.sessions` already holds the design and earlier cycles, a `build` entry
   ($22.18) and a fix entry ($2.72).
   - **Both of those were measured before their final turns.** Re-measure them from their full
     transcripts, and replace the numbers if they differ:
     - build: `…--claude-worktrees-feat-spec-133-watermark-damage-rule/7ade2af7-0cc3-48b8-a9f9-6ddf8605ce1f.jsonl`
     - fix: `…--claude-worktrees-feat-spec-133-watermark-damage-rule/e832cf11-e841-4f2a-b89b-102d7a362133.jsonl`.
       This also covers the follow-up turn that fixed `docs/data-model.md`.
   - **Then add three entries,** plus a fourth for your own `ship`:

     | cycle | model | transcript (under `~/.claude/projects/`) | reported |
     |---|---|---|---|
     | verify | Opus | `…--claude-worktrees-verify-spec-133/a446ba7b-e3b1-406c-9f79-07a2e1fceef2.jsonl` | $11.16 |
     | build (fix, aborted) | Opus | `…--claude-worktrees-verify-spec-133/e07e474f-02c1-47f0-bc0e-1c4796a3e0bb.jsonl`: the fix prompt pasted into the verify worktree by mistake, then `/exit` | small |
     | verify (re-verify) | Opus | `…--claude-worktrees-verify-spec-133-fix/07fa8377-e921-49c4-aeac-317440893ed4.jsonl` | $3.22 |

     The `…` is `-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg-`.
   - **Before measuring each transcript, confirm it by content:** `grep -c "SPEC-133 — <CYCLE>
     prompt"`. For the build, use `grep -c "SPEC-133-build.md exactly"`.
   - **Don't assume the model:** read it from each transcript's `.message.model`. The dollar
     figures above are the sessions' own estimates, listed only for sanity-checking. Do not copy
     them.
   - **Recompute `cost.totals` from every entry:**
     - `tokens_total` sums the non-null totals, counting a null as `0`;
     - `estimated_usd` sums the non-null values;
     - `session_count` counts every entry.
   - Run `just cost-audit` and confirm SPEC-133 passes.
4. **Mark ship `[x]`** in the timeline. Mark the re-verify line `[x]` if the orchestrator has not.
   Include the merge date (PR #206, 2026-10-27) and the total cost.
5. **Advance and archive:** `just advance-cycle SPEC-133 ship`, then
   `just archive-spec SPEC-133`. Then read both STAGE-050 entries for SPEC-133 back.
   - The design-era entry (about line 211) should read like SPEC-131's:
     `- [x] **SPEC-133** (merged 2026-10-27, PR #206) — …`.
   - The older framing entry (about line 84) is the same spec's history. Mark it `[x]` with a
     pointer to the shipped entry, unless the script already did.
   - Fix either entry by hand if the script left it otherwise. Touch no other spec's entry.
   - Recount the `**Count:**` line with a grep you have just run, not by arithmetic. It was
     stale before SPEC-132 shipped.
6. **Check what the tooling sees.** Run `just status` and `just backlog`, and confirm SPEC-133
   shows as shipped and any follow-up you filed appears. A file you wrote is not a file the
   tooling sees until you read it back.
7. **Commit to `main` directly.** Ship bookkeeping does not go through a PR (AGENTS §13).
   - Commit: `git commit -s -m "ship(SPEC-133): a watermark refuses to damage an image — reflected, archived"`.
     **The DCO sign-off is mandatory.**
   - Push: `git fetch origin && git rebase origin/main && git push origin HEAD:main`.
   - Read back `git show --stat HEAD` and check it is what you meant.

**Do not run Stage Ship.** It is the orchestrator's next step, in its own session, after both
SPEC-133 and SPEC-134 are archived.

## Guardrails
- ⛔ No version bump, no tag, no release.
- Do not edit code, tests or DECs. If the reflection says a DEC, constraint or template needs
  changing, propose the edit in your report; the orchestrator decides.

## Measure cost (yours and the transcripts above)
Your own transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Identify it **by content,
not recency**: `grep -l "SPEC-133 — SHIP prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
under-counts about 11×.

Price each component at the model that actually ran, never at a flat rate: Sonnet $3/$15 per
MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-133 ship: archived — <commit SHA on main>`. Then:
- the three reflection answers, one line each;
- follow-ups filed, or "none new";
- the re-measured build and fix entries, old → new;
- the four new cost entries, ship included (cycle, tokens and USD for each);
- `cost.totals` (tokens, USD, session count) and the `just cost-audit` result;
- the recounted `**Count:**` line;
- any template, constraint or DEC change you propose;
- your measured cost.
