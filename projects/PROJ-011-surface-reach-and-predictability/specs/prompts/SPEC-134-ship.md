# SPEC-134 — SHIP prompt: a `--name-template` extension pins the output format

Cycle: **ship** (bookkeeping only; the code is on `main` as `7034d94`, PR #202, squash-merged
2026-10-27 UTC / 2026-10-26 PDT, including two punch-list fixes). Fresh CLI session, own worktree, Sonnet is
enough:

```
just wt docs/spec-134-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
`git fetch origin && git rebase origin/main`. Then open the spec's timeline
(`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-134-name-template-extension-pins-the-output-format-timeline.md`).
⛔ **Proceed only if the re-verify 2 line is `[x]` with ✅ APPROVED** and PR #202 is merged
(`gh pr view 202 --json state`). Otherwise stop and report.

## Read first
`/AGENTS.md` §4 (cost), §13 (verify + ship bookkeeping lands on `main`), §15 "During ship"; the
spec in full, including `## Build Completion` and `## Verify Readout`; the timeline;
`projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md` "Traps this wave paid for"; the
STAGE-050 entry for SPEC-134; DEC-105 (and DEC-087, which it extends).

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **`## Reflection (Ship)`**: outcome-focused, three answers, drawn from evidence (Build
   Completion, the verify readouts, the timeline, the merge history), not invented:
   1. What would I do differently next time? Facts to weigh:
      - Verify found the code sound but returned 3 doc fixes. The maintainer also ruled in a
        fourth item, a real bug on 0.7.x too: at N inputs the pin check read `-o` while the
        write used the template, so JPEG/AVIF bytes landed in `in.png`.
      - Fix 1 added `resolved_input_count`, but every N-input row used literal paths, so a
        mutation replacing it with `inputs.len()` stayed green. A second fix and re-verify were
        needed for 3 glob rows.
      - The rule being extended lived only in `docs/api-contract.md`, not in DEC-087; the
        orchestrator misattributed it at frame.
   2. Does any template, constraint, or decision need updating? Name it, or say none. One
      candidate to weigh: when a fix adds a function that turns arguments into a derived value
      (a count, a resolved list), should the fix prompt require a test whose input makes the two
      differ?
   3. Is there a follow-up spec to write before I forget? Each one becomes a `- [ ]` in
      STAGE-050's `## Spec Backlog` (AGENTS §10). Read the backlog first; do not duplicate.
      Already filed: "Encoding verbs resolve their inputs twice" and "`responsive` accepts
      `--name-template` and silently ignores it". Also: the backlog item "Absorbed into SPEC-134
      as AC-22" (`meta … -o x.png`) is done by this spec. Mark it `[x]` with "shipped in
      SPEC-134", wording only, and keep it where it is.
3. **Cost.** `cost.sessions` already holds frame, design, build and two fix sessions (both
   labelled `build`: $1.29 and $0.72). Add **four** entries: three sessions nobody has recorded,
   plus your own `ship`. Measure each from its transcript (method below). Do not copy the dollar
   figures here; they are the sessions' own estimates, listed so you can sanity-check.
   | cycle | model | transcript (under `~/.claude/projects/`) | reported |
   |---|---|---|---|
   | verify | Opus | `…--claude-worktrees-verify-spec-134/22adad68-e2c5-46b1-97e9-1317e56d6818.jsonl` | $5.14 |
   | verify (re-verify) | Opus | `…--claude-worktrees-verify-spec-134-fix/a9419ffd-5b32-437d-b3f5-693949453b3f.jsonl` | $1.99 |
   | verify (re-verify 2) | Opus | `…--claude-worktrees-verify-spec-134-fix2/d3cf82b6-036f-4c88-8a13-96f2bb65ea3b.jsonl` | $1.02 |
   The `…` is `-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg-`. Confirm
   each by content (`grep -c "SPEC-134 — <CYCLE> prompt"`) before measuring it. Read the model
   from each transcript's `.message.model`; don't assume it. Then compute `cost.totals` from
   every entry. `tokens_total` sums the non-null totals (use `0`, never `null`). `estimated_usd`
   sums the non-null values. `session_count` counts every entry. Run `just cost-audit` and
   confirm SPEC-134 passes.
4. **Mark ship `[x]`** in the timeline. Include the merge date (PR #202, 2026-10-26 PDT) and the
   total cost.
5. `just advance-cycle SPEC-134 ship`, then `just archive-spec SPEC-134`. Read the STAGE-050
   entry back afterwards. It should read like SPEC-131's (`- [x] **SPEC-131** (merged
   2026-10-25, PR #200; fix #203 2026-10-26) — …`); for SPEC-134 that's `- [x] **SPEC-134**
   (merged 2026-10-26, PR #202) — …`. Fix it by hand if the script left it otherwise. Touch no
   other spec's entry.
6. `just status` and `just backlog`: confirm SPEC-134 shows as shipped and any follow-up you filed
   appears. A file you wrote is not a file the tooling sees until you read it back.
7. Commit to `main` directly (AGENTS §13: ship bookkeeping does not go through a PR):
   `git commit -s -m "ship(SPEC-134): a --name-template extension pins the format — reflected, archived"`
   (**DCO sign-off is mandatory**), then `git fetch origin && git rebase origin/main && git push
   origin HEAD:main`. Read back `git show --stat HEAD` and check it is what you meant.

STAGE-050 is **not** finished (SPEC-133 is open), so do not run Stage Ship.

## Guardrails
- ⛔ No version bump, no tag, no release.
- Do not edit code, tests or DECs. If the reflection says a DEC, constraint or template needs
  changing, propose the edit in your report; the orchestrator decides.

## Measure cost (yours and the four above)
Your own transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Identify it **by content,
not recency**: `grep -l "SPEC-134 — SHIP prompt" ~/.claude/projects/<slug>/*.jsonl`.
⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's
output under-counts about 11×. Price each component at the model that actually ran: Sonnet
$3/$15 per MTok, Opus $5/$25; cache_creation is ×1.25 the input rate and cache_read ×0.10. Never
use a flat rate.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-134 ship: archived — <commit SHA on main>`. Then:
- the three reflection answers, one line each;
- follow-ups filed (or "none new");
- the four new cost entries (cycle, tokens, USD each);
- `cost.totals` (tokens, USD, session count) and the `just cost-audit` result;
- any template, constraint or DEC change you propose;
- your measured cost.
