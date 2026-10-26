# SPEC-132 — SHIP prompt: `build --watch` watches recipe asset paths

Cycle: **ship** (bookkeeping only; the code is on `main` as `7eca94b`, PR #201, squash-merged
2026-10-26, including the verify punch-list fix). Fresh CLI session, own worktree, Sonnet is
enough:

```
just wt docs/spec-132-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
`git fetch origin && git rebase origin/main`. Then open the spec's timeline
(`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-132-build-watch-watches-recipe-asset-paths-timeline.md`).
⛔ **Proceed only if the re-verify line is `[x]` with ✅ APPROVED** and PR #201 is merged
(`gh pr view 201 --json state`). Otherwise stop and report.

## Read first
`/AGENTS.md` §4 (cost), §13 (verify + ship bookkeeping lands on `main`), §15 "During ship"; the
spec in full, including `## Build Completion` and `## Verify Readout`; the timeline;
`projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md` "Traps this wave paid for"; the
STAGE-050 entry for SPEC-132; DEC-060 (its two 2026 amendments).

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **`## Reflection (Ship)`**: outcome-focused, three answers, drawn from evidence (Build
   Completion, the verify readouts, the timeline, the merge history), not invented:
   1. What would I do differently next time? Facts to weigh:
      - The build found a bug outside the spec, in released 0.7.x: on Linux, inotify reported
        the build's own reads, so `--watch` rebuilt in a loop while idle (207 builds in 45 s).
        It fixed it in the same PR without recording it. Verify then required a CHANGELOG entry
        and a DEC-060 amendment.
      - Verify's mutations survived three tests (AC-8(d), AC-7(d), AC-5), and the fix tightened
        them.
      - Two verify sessions ran on the same head; the first never reported back.
      - The frame found the bug by accident: it shows only when an asset sits beside its recipe.
   2. Does any template, constraint, or decision need updating? Name it, or say none. One
      candidate to weigh: should build prompts say how to handle an out-of-spec bug found
      mid-build? (Record it, or file it as its own item.)
   3. Is there a follow-up spec to write before I forget? Each one becomes a `- [ ]` in
      STAGE-050's `## Spec Backlog` (AGENTS §10). Read the backlog first; do not duplicate.
3. **Cost.** `cost.sessions` already holds frame, design, build and one fix session (labelled
   `build`, $0.96). Add **five** entries: four sessions nobody has recorded, plus your own `ship`.
   Measure each from its transcript (method below). Do not copy the dollar figures here; they
   are the sessions' own estimates, listed so you can sanity-check.
   | cycle | model | transcript (under `~/.claude/projects/`) | reported |
   |---|---|---|---|
   | verify | Opus | `…--claude-worktrees-verify-spec-132/c50ceccb-4ada-45f8-a87a-05c1eac900b0.jsonl` (never reported back) | ~$4.79 |
   | verify | Opus | `…--claude-worktrees-verify-spec-132/613a8d01-65cf-467a-9ca1-0911bacd9ce0.jsonl` | $5.22 |
   | build (fix) | Sonnet | `…--claude-worktrees-feat-spec-132-watch-recipe-assets/3aba645b-9712-4f1e-97eb-a9c50a945716.jsonl` (a second fix session that found the work done and changed nothing) | <$0.10 |
   | verify (re-verify) | Opus | `…--claude-worktrees-verify-spec-132-fix/9358f573-7bd2-4b1b-8b4c-8ebd16412686.jsonl` | $1.31 |
   The `…` is `-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg-`. Confirm
   each by content (`grep -c "SPEC-132 — <CYCLE> prompt"`) before measuring it. Read the model
   from each transcript's `.message.model`; don't assume it. Then compute `cost.totals` from
   every entry. `tokens_total` sums the non-null totals (use `0`, never `null`). `estimated_usd`
   sums the non-null values. `session_count` counts every entry. Run `just cost-audit` and
   confirm SPEC-132 passes.
4. **Mark ship `[x]`** in the timeline, and mark the re-verify line `[x]` if the orchestrator has
   not. Include the merge date (PR #201, 2026-10-26) and the total cost.
5. `just advance-cycle SPEC-132 ship`, then `just archive-spec SPEC-132`. Read the STAGE-050
   entry back afterwards. It should read like SPEC-131's (`- [x] **SPEC-131** (merged
   2026-10-25, PR #200; fix #203 2026-10-26) — …`); for SPEC-132 that's `- [x] **SPEC-132**
   (merged 2026-10-26, PR #201) — …`. Fix it by hand if the script left it otherwise. Touch no
   other spec's entry.
6. `just status` and `just backlog`: confirm SPEC-132 shows as shipped and any follow-up you filed
   appears. A file you wrote is not a file the tooling sees until you read it back.
7. Commit to `main` directly (AGENTS §13: ship bookkeeping does not go through a PR):
   `git commit -s -m "ship(SPEC-132): build --watch watches recipe asset paths — reflected, archived"`
   (**DCO sign-off is mandatory**), then `git fetch origin && git rebase origin/main && git push
   origin HEAD:main`. Read back `git show --stat HEAD` and check it is what you meant.

STAGE-050 is **not** finished (SPEC-133/134 are open), so do not run Stage Ship.

## Guardrails
- ⛔ No version bump, no tag, no release.
- Do not edit code, tests or DECs. If the reflection says a DEC, constraint or template needs
  changing, propose the edit in your report; the orchestrator decides.

## Measure cost (yours and the four above)
Your own transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Identify it **by content,
not recency**: `grep -l "SPEC-132 — SHIP prompt" ~/.claude/projects/<slug>/*.jsonl`.
⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's
output under-counts about 11×. Price each component at the model that actually ran: Sonnet
$3/$15 per MTok, Opus $5/$25; cache_creation is ×1.25 the input rate and cache_read ×0.10. Never
use a flat rate.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-132 ship: archived — <commit SHA on main>`. Then:
- the three reflection answers, one line each;
- follow-ups filed (or "none new");
- the five new cost entries (cycle, tokens, USD each);
- `cost.totals` (tokens, USD, session count) and the `just cost-audit` result;
- any template, constraint or DEC change you propose;
- your measured cost.
