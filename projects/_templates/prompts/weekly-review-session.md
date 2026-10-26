# WEEKLY REVIEW — session prompt (reusable, any week)

Fresh CLI session, own worktree, **Opus**. You are not the orchestrator and you did not do any of
the work under review. That distance is the point: in this variant one agent plays every role, so
the weekly review is the only regular second look (`FIRST_SESSION_PROMPTS.md` Prompt 6,
"non-optional here").

```
just wt docs/weekly-review-<YYYY-WNN> -- --model opus
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0
`git fetch origin && git rebase origin/main`. Work out this ISO week's id: `date +%G-W%V` (e.g.
`2026-W44`); call it `WEEK`. Check `reports/weekly/` for an existing `WEEK-review.md`; if one
exists, stop and report it rather than overwrite it.

## Step 1 — gather, with the repo's own tools
1. `just report-weekly` — writes the **data** report to `reports/weekly/WEEK.md` (specs shipped,
   decisions touched, cost). Keep it; it is the second file you commit.
2. `just weekly-review` — prints the review prompt with the file list and the last 7 days of git
   activity. **Follow it**: read every file it lists, then produce the report it defines.
3. Also read, because they hold what the tooling cannot show: the active project's `HANDOFF.md`
   (rules, traps), each in-flight spec's `-timeline.md`, and the last 7 days of merged PRs
   (`gh pr list --state merged --search "merged:>=<7 days ago>"`) including each PR's verify
   readout where the spec records one.
4. `just status`, `just backlog`, `just cost-audit`, `./scripts/decisions-audit.sh` (lint mode).

Every claim in your report cites its evidence: a file and line, a commit SHA, a PR number, or a
command you ran and its output. A finding without evidence is a hunch; label it as one.

## Step 2 — write the review
Write `reports/weekly/WEEK-review.md` with exactly these sections, tight and actionable in 10
minutes of reading:

1. **Stale decisions** — DEC-* to supersede or amend, with the reason. Flag only; do not edit DECs.
2. **Low-confidence decisions** — DEC-* below 0.8 that recent work strengthened or weakened.
3. **Missing constraints** — patterns to formalise; propose the YAML entry.
4. **Resolved questions** — `guidance/questions.yaml` items recent work answered; name the DEC
   that would formalise each.
5. **AGENTS.md / HANDOFF drift** — anything the repo does that contradicts them; propose the edit.
6. **Template improvements** — specific: "add X to `projects/_templates/spec.md` because Y".
7. **Stage health** — each active stage: progressing, stalled, or needs rescoping.
8. **Cycle health** — skipped or out-of-order cycles (e.g. a PR merged before its verify), verify
   catching what build's own controls missed, mailed-in reflections, frame used as a real gate or
   not.
9. **Session hygiene** — any sign two cycles of one spec ran in one session; prompts that bundled
   steps.
10. **Cost** — this week's spend by cycle from `cost.sessions`; entries missing or flagged low.
11. **Top 3 actions** — the three things the orchestrator should do next, each one line with its
    evidence. This is the section the maintainer reads first; put it at the **top** of the file,
    above section 1.

End the file with a cost footer for this review session (method below).

## Step 3 — land it
- **Commit only the two report files** (`reports/weekly/WEEK.md`, `reports/weekly/WEEK-review.md`).
  Do **not** edit DECs, constraints, AGENTS.md, HANDOFF, specs, stages or code: every change is a
  proposal in the report, and the orchestrator decides.
- Reports go to `main` directly, like other docs (AGENTS §13):
  `git commit -s -m "docs(reports): weekly review WEEK"` (**DCO sign-off is mandatory**), then
  `git fetch origin && git rebase origin/main && git push origin HEAD:main`. Read back
  `git show --stat HEAD`.

## Guardrails
- Read-only apart from the two report files. Do not start, stop or message other sessions.
- zsh does not word-split unquoted parameters. A piped command reports the pipe's exit code.
- **Budget ~60 exchanges.** Breadth over depth: flag what needs a deeper look, do not do it.

## Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Identify it **by content, not
recency**: `grep -l "WEEKLY REVIEW — session prompt" ~/.claude/projects/<slug>/*.jsonl`. ⚠ **Dedup
by `.message.id`**: one JSONL line per *content block*; lines sharing an id repeat identical
`input_tokens` / `cache_creation_input_tokens` / `cache_read_input_tokens` while `output_tokens`
grows. Take those three once per id and the **MAX** `output_tokens`. Price each component at the
model that actually ran (Opus $5/$25 per MTok, Sonnet $3/$15; cache_creation ×1.25 the input rate,
cache_read ×0.10), never flat. Put `tokens_total`, the breakdown and `estimated_usd` in the
report's footer before committing (it will slightly under-count the final turns; say so).

## Report back (your final message — it is pasted to the orchestrator)
First line: `Weekly review WEEK: reports/weekly/WEEK-review.md — <commit SHA on main>`. Then the
**Top 3 actions**, one line each with evidence, then one line per section that has a finding
(skip empty sections), then your cost.
