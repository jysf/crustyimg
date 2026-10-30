# SPEC-138 — SHIP prompt: `build` refuses the output flags its manifest owns

Cycle: **ship**. This is bookkeeping only. The code is on `main` as `fc05ef9` (PR #221,
squash-merged 2026-10-30).
- Verify approved `5118362` with no punch list (✅). The orchestrator marked the PR ready, and the
  maintainer merged it.
- This is the second of the two specs that open **0.8.1**. SPEC-137 shipped first.

Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-138-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. Open the timeline:
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-138-build-refuses-the-output-flags-its-manifest-owns-timeline.md`.
3. ⛔ **Proceed only if both of these hold.** Otherwise stop and report.
   - The verify line is `[x]` and says the PR was marked ready.
   - PR #221 is merged: `gh pr view 221 --json state`.

## Read first
- `/AGENTS.md` §4 (cost), §13 (verify and ship bookkeeping lands on `main`) and §15 "During
  ship".
- The spec in full: `## Frame` with rulings 1–6, `## Design` with Calls 1–8, the mutation table,
  `## Build Completion` and the cost block.
- The timeline, especially the verify line.
- `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for"
  and the rules.
- STAGE-054's `## Spec Backlog`, all of it. SPEC-138's entry is at about line 210.

## What verify reported (the facts to reflect on)
- Baseline at `277391d`: 12 red / 2 green, each for the spec's reason. FT-11 failed at its 20 s
  deadline, which proves its waiter can block.
- Every AC row exits 2 on the PR binary, and gives the spec's baseline on `main`.
  `compare-builds.sh` printed IDENTICAL; the one-byte key mutant printed LOCKFILE DIFFERS.
- **The after-manifest mutation differed between build and verify.** The build moved the guard
  into `run_build` only (off the watch path), so FT-11 and FT-12 went red as well. Verify applied
  it as the spec words it, on both paths: FT-8, 9 and 12 went red, and FT-11 stayed green. FT-12
  goes red because the `--watch`/`--check` conflict check runs before the manifest loads.
- **Lean leg:** a lean build refuses `build --watch` anyway, because `watch` is not compiled in.
  FT-11 still goes red there through its exact-stderr assertion, so it is not hollow, but in that
  leg it never exercises the wait-for-a-watcher path. The default legs cover that.
- **M-a to M-c were caught. M-d is a gap:** if `snapshot()` skipped dotfiles, no test would notice
  a write into `.crustyimg/`. The risk is low, because a build that slips past the guard writes its
  outputs too.
- Uncovered rows behaved correctly: `build other.toml --format png` exits 2;
  `build -- --format` treats it as a file name (exit 3, as on `main`); `-Qox.png` exits 2;
  `convert` and `optimize` still honour all four flags. `build --help` still lists the four flags,
  which is mildly misleading and out of scope.
- Verify's optional cleanups:
  1. FT-13 could assert that its snapshot diff includes a `.crustyimg/` path (closes M-d);
  2. `snapshot()` records files only, so a new empty directory is invisible to it;
  3. the DEC-105 rule-2 line and the `docs/api-contract.md:626` line run past the wrap width;
  4. the spec's mutation table should list FT-12 on the "after the manifest loads" row and FT-15
     on "also refuse quality";
  5. Build Completion gives `277391d`'s hunk header as `+1517,55`; it is `+1517,59`.

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **Spec text fixes (cleanups 4 and 5 only).** Correct the mutation table rows and the hunk
   header in the spec, and add one line under Build Completion saying verify found them. Change
   nothing else in the spec body. Do not touch DECs, `docs/` or tests: cleanups 1–3 are for step
   3.3.
3. **Write `## Reflection (Ship)`.** Make it outcome-focused, with three answers drawn from the
   evidence above, the Build Completion and the timeline. Do not invent anything.
   1. **What would I do differently next time?** Weigh at least: the mutation row the spec worded
      one way and the build applied another (should a design's mutation table name *where* each
      mutation is applied?); and the lean-leg question that only verify could answer.
   2. **Does any template, constraint or decision need updating?** Name it, or say none.
      **Propose it; do not apply it.**
   3. **Is there a follow-up spec to write before I forget?** Read STAGE-054's backlog first.
      Weigh cleanups 1–3 and the `build --help` text. Either file **one** small backlog entry that
      groups what is worth doing, or say why none is. Do not file one entry per cleanup.
4. **Cost.** `cost.sessions` holds frame ($2.91), design ($3.64) and build ($2.28).
   - **The build was measured before its final turns.** Re-measure it from its full transcript,
     and replace the number if it differs.
   - **Add one entry for verify**, plus one for your own `ship`.
   - Every transcript is under `~/.claude/projects/`. The `…` in each path is
     `-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg--claude-worktrees-`.

     | cycle | transcript | confirm with `grep -c` | reported |
     |---|---|---|---|
     | build (re-measure) | `…feat-spec-138-build-refuses-output-flags/8813cf27-7b2f-4c0f-8812-2a07acdbf08b.jsonl` | `BUILD prompt: ` | $2.28 |
     | verify (add) | `…verify-spec-138/5199f3ee-e832-492c-839d-837c9a3b3969.jsonl` | `VERIFY prompt: ` | $5.99 |

   - **Before measuring each transcript, confirm it by content** with the grep in the table; the
     backticks in the title may be escaped in JSONL, so match on the text shown. Read the model from
     each transcript's `.message.model`; don't assume it. The dollar figures above are the
     sessions' own estimates, for sanity-checking only. Do not copy them.
   - **Recompute `cost.totals` from every entry** (today they read 0):
     - `tokens_total` is the sum of the non-null totals;
     - `estimated_usd` is the sum of the non-null values;
     - `session_count` counts every entry.

     Run `just cost-audit` and confirm SPEC-138 passes. Quote the line that names it; if no line
     names it, say so.
5. **Mark ship `[x]`** in the timeline, with the merge (`fc05ef9`, PR #221, 2026-10-30) and the
   total cost.
6. **Advance and archive.** Run `just advance-cycle SPEC-138 ship`, then
   `just archive-spec SPEC-138`. Then read STAGE-054's SPEC-138 entry back.
   - It should read `- [x] **SPEC-138** (merged 2026-10-30, PR #221) — …`. Fix it by hand if the
     script left it otherwise. Touch no other entry.
   - Recount the `**Count:**` line with a grep you have just run, not by arithmetic. Today it
     reads `2 shipped / 1 active / 15 pending`. Expect `3 shipped / 0 active / 15 pending`, plus
     any entry you file in step 3.3.
7. **Check what the tooling sees.**
   - Run `just status` and `just backlog`. Confirm SPEC-138 shows as shipped.
   - Run `just roadmap`: STAGE-054 must stay **active**. ⛔ **Do not run Stage Ship**, and do not
     mark STAGE-054 shipped; the orchestrator decides that with 0.8.1 release prep.
8. **Commit to `main` directly.** Ship bookkeeping does not go through a PR (AGENTS §13).
   - Commit:
     `git commit -s -m "ship(SPEC-138): build refuses the output flags its manifest owns — reflected, archived"`,
     with the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. **The DCO
     sign-off is mandatory.**
   - Push: `git fetch origin && git rebase origin/main && git push origin HEAD:main`.
   - Read back `git show --stat HEAD` and check it is what you meant.

## Guardrails
- ⛔ No version bump, no tag, no release and no CHANGELOG edit. Release prep runs later, in
  separate sessions.
- Do not edit code, tests, CI, DECs, `docs/`, HANDOFF or templates. If the reflection says one
  needs changing, propose the edit in your report; the orchestrator decides.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~40 exchanges.**

## Measure cost (yours and the transcripts above)
- Your own transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Identify it **by content, not recency**:
  `grep -l "SHIP prompt: " ~/.claude/projects/<slug>/*.jsonl`. The transcript may store the
  em-dash and backticks escaped, so search for plain text.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran, never at a flat rate: Sonnet $3/$15 per
  MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-138 ship: archived — <commit SHA on main>`. Then:
- the two spec text fixes;
- the three reflection answers, one line each;
- any follow-up you filed, or "none" and why;
- the re-measured build entry, old → new;
- the two new cost entries (verify and ship), each with cycle, model, tokens and USD;
- `cost.totals` (tokens, USD, session count) and the `just cost-audit` result;
- the recounted `**Count:**` line, and confirmation that STAGE-054 is still active;
- any template, constraint or DEC change you propose;
- your measured cost.
