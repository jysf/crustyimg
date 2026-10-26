# SPEC-131 — SHIP prompt: recipe step params are validated at load

Cycle: **ship** (bookkeeping only; the code is on `main` as `ba8b740`, PR #200, plus the
verify punch-list fix `13ed6d7`, PR #203). Fresh CLI session,
own worktree, Sonnet is enough:

```
just wt docs/spec-131-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
`git fetch origin && git rebase origin/main`. Then open the spec's timeline
(`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-131-typed-per-operation-parameter-structs-timeline.md`).
⛔ **Proceed only if the verify line is `[x]` with ✅ APPROVED**, or with a punch list whose
follow-up fix PR is named there as merged. If verify is still `[~]`, or a follow-up is open, stop
and report: shipping before verify is the mistake this project already made once.

The orchestrator has already recorded the verify verdict and its cost on `main`; you record ship.

## Read first
`/AGENTS.md` §4 (cost), §13 (verify + ship bookkeeping lands on `main`), §15 "During ship"; the
spec in full, including `## Build Completion`, its build-phase reflection, and the verify readout
the orchestrator recorded; `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`
"Traps this wave paid for"; the STAGE-050 entry for SPEC-131.

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **`## Reflection (Ship)`** — outcome-focused, three answers, drawn from evidence (Build
   Completion, the verify readout, the merge history), not invented:
   1. What would I do differently next time? (Include the merge-before-verify: #200 merged as
      `ba8b740` before its verify ran, because the gate lived only in prompts — builds now open
      draft PRs. And the regression the retroactive verify caught: the build removed the
      constructors' f32 range checks as redundant, so `1e-50` passed and read as 0; none of the
      build's six negative controls could see it.)
   2. Does any template, constraint, or decision need updating? Name it, or say none.
   3. Is there a follow-up spec to write before I forget? Each one becomes a `- [ ]` in
      STAGE-050's `## Spec Backlog` (AGENTS §10). Check that the two items SPEC-131's design filed
      (wasm `validate()` export; watermark `size` upper bound) are already there; do not duplicate.
3. **Cost.** Append your own `ship` entry to `cost.sessions` (method below). Then compute
   `cost.totals` from every session entry: `tokens_total` sums the non-null totals (use `0`, never
   `null`), `estimated_usd` sums the non-null values, `session_count` counts every entry. Then run
   `just cost-audit` and confirm it passes for SPEC-131.
4. **Mark ship `[x]`** in the timeline: merge dates (PR #200 on 2026-10-25, fix PR #203 on
   2026-10-26), total cost.
5. `just advance-cycle SPEC-131 ship`, then `just archive-spec SPEC-131` (moves the spec and its
   timeline into `specs/done/` and updates the stage backlog). Read the STAGE-050 entry back
   afterwards: it should read like SPEC-129's (`- [x] **SPEC-131** (merged 2026-10-25, PR #200; fix #203 2026-10-26) —
   …`). Fix it by hand if the script left it otherwise. Touch no other spec's entry.
6. `just status` and `just backlog`: confirm SPEC-131 shows as shipped and any follow-up you filed
   appears. A file you wrote is not a file the tooling sees until you read it back.
7. Commit to `main` directly (AGENTS §13: ship bookkeeping does not go through a PR):
   `git commit -s -m "ship(SPEC-131): strict recipe step params — reflected, archived"`
   (**DCO sign-off is mandatory**), then `git fetch origin && git rebase origin/main && git push
   origin HEAD:main`. Read back `git show --stat HEAD` and check it is what you meant.

STAGE-050 is **not** finished (SPEC-132/133/134 are open), so do not run Stage Ship.

## Guardrails
- ⛔ No version bump, no tag, no release.
- Do not edit code, tests or DECs. If the reflection says a DEC or constraint needs changing,
  propose the edit in your report; the orchestrator decides.

## Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Identify it **by content, not
recency**: `grep -l "SPEC-131 — SHIP prompt" ~/.claude/projects/<slug>/*.jsonl`. ⚠ **Dedup by
`.message.id`**: Claude Code writes one JSONL line per *content block*; lines sharing an id repeat
identical `input_tokens` / `cache_creation_input_tokens` / `cache_read_input_tokens` while
`output_tokens` grows. Take those three once per id and the **MAX** `output_tokens`. Summing every
line over-counts ~2×; taking the first line's output under-counts ~11×. Price each component at
the model that actually ran (Sonnet $3/$15 per MTok, Opus $5/$25; cache_creation ×1.25 the input
rate, cache_read ×0.10), never a flat rate.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-131 ship: archived — <commit SHA on main>`. Then: the three reflection answers
in one line each; follow-ups filed (or "none new"); `cost.totals` (tokens, USD, session count) and
the `just cost-audit` result; any template/constraint/DEC change you propose; your measured cost.
