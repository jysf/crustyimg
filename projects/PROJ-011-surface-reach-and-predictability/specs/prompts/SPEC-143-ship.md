# SPEC-143 — SHIP prompt: a watermark that draws nothing is refused

Cycle: **ship**. This is bookkeeping only. The code is on `main` as `06eb552` (PR #235,
squash-merged 2026-10-31).
- Verify returned a punch list (docs and tests only). The punch list landed as `36c3b66`, and the
  orchestrator confirmed it without a re-verify: no `src/` change, and each new test was proven red
  by its mutation. The orchestrator marked the PR ready, and the maintainer merged it.
- This spec opens **0.8.2**.

Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-143-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. Open the timeline:
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-143-a-watermark-that-draws-nothing-is-refused-timeline.md`.
3. ⛔ **Proceed only if both of these hold.** Otherwise stop and report.
   - The punch-list line is `[x]` and says PR #235 was marked ready.
   - PR #235 is merged: `gh pr view 235 --json state`.

## Read first
- `/AGENTS.md` §4 (cost), §13 (verify and ship bookkeeping lands on `main`) and §15 "During
  ship".
- The spec in full: `## Frame` with rulings 1–3, `## Design` with Calls 1–8, the mutation table,
  `## Build Completion` including `### Punch list (verify)`, and the cost block.
- The timeline: the verify and punch-list lines.
- `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for"
  (1–12) and the rules.
- STAGE-054's `## Spec Backlog`, all of it.

## What verify and the punch list reported (the facts to reflect on)
- **The code was right on every path:** 36 cause rows (six causes × CLI, `apply`, `build`, both
  binaries), every Call 5 boundary, 18/18 out-of-repo `cmp` identical, and `--watch` driven by
  hand (it reports both lines and keeps watching).
- **Two tests failed on `main` for the wrong reason.** FT-11's 64×64 inputs and FT-12's 64×48 base
  tripped the 5 % coverage cap, so `main` skipped or refused the mark for coverage, not because it
  drew nothing. The build recorded FT-12 as "returned Ok", which was false. The punch list enlarged
  the bases (256×256, 256×192) and showed both now fail on `main` with no coverage message.
- **A recipe-side mutation survived:** `max_alpha = 65535` in `from_params` left every test green,
  because FT-8's too-faint rows all used an opaque logo. Two low-alpha rows now catch it.
- **The golden was not a byte guard on the 16-bit composite line.** Every 16-bit case ran at
  opacity 1 or faded to alpha16 = 1, so `opacity × 0.999` in the 16-bit loop stayed green. The
  build's own golden mutations turned FT-6 red only by refusal. A 16-bit 0.6 case now catches it
  by hash, and its bytes match `main`.
- **Two stale doc sentences:** `docs/cli-reference.md` and `SKILL.md` still promised a faint mark
  "is drawn as asked", and said every refusal came before any file read (true only for opacity 0
  and colour alpha `00`).
- **All three of the build's mutation-table deviations were spec errors** (AC-3's two `build`
  lines; M5 turning FT-15 red; M4a turning FT-13 red). The punch list corrected the table.
- **No CI job runs `just wasm-test`,** so FT-12 runs only locally. Already filed in STAGE-054 by
  the orchestrator (2764a24); don't file it again.

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **Write `## Reflection (Ship)`.** Make it outcome-focused, with three answers drawn from the
   evidence above, the Build Completion and the timeline. Do not invent anything.
   1. **What would I do differently next time?** Weigh at least: a RED baseline recorded by exit
      code instead of by failure message (two tests passed review while failing for the coverage
      cap); and a golden whose mutations went red only through a refusal, never by hash.
   2. **Does any template, constraint or decision need updating?** For example, a HANDOFF trap:
      "a baseline row is RED for its reason only if its failure message says so", or "a golden
      guards the bytes only if one mutation turns it red by hash". Check traps 1–12 first; if one
      already covers it, say which. **Propose it; do not apply it.**
   3. **Is there a follow-up spec to write before I forget?** Read STAGE-054's backlog first. Weigh
      verify's leftover notes: `faint_opacity` and `faint_opacity_boundary` hash identically;
      `docs/api-contract.md` has one unwrapped line about `--watch`. Either file **one** small
      backlog entry that groups what is worth doing, or say why none is.
3. **Cost.** `cost.sessions` holds frame ($5.55), design ($4.63), build ($3.88) and punchlist.
   - **The build and punch-list entries were measured before their final turns.** Re-measure both
     from their full transcripts and replace the numbers if they differ.
   - **Add one entry for verify**, plus one for your own `ship`.
   - Every transcript is under `~/.claude/projects/`. The `…` in each path is
     `-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg--claude-worktrees-`.

     | cycle | transcript | confirm by content | reported |
     |---|---|---|---|
     | build (re-measure) | `…feat-spec-143-refuse-invisible-watermark/8e0c9f1c-fc5f-45db-8c28-70e743c36971.jsonl` | `BUILD prompt: a watermark` | $3.88 |
     | verify (add) | `…verify-spec-143/644d2ca8-758d-4a0e-8ce6-c9ed95540769.jsonl` | `VERIFY prompt: a watermark` | $9.85 |
     | punchlist (re-measure) | `…fix-spec-143-punch/6219b4c5-2950-44ac-a29e-3b1ddc831633.jsonl` | `PUNCH LIST prompt: docs that match` | $1.86 |

   - **Confirm each transcript by content before measuring it.** `grep -c` can come back empty
     through the `rtk` wrapper; count with Python (`open(f).read().count(...)`) or `/usr/bin/grep`
     run outside a command substitution. Read the model from each transcript's `.message.model`;
     don't assume it. The dollar figures above are the sessions' own estimates, for
     sanity-checking only. Do not copy them.
   - **Recompute `cost.totals` from every entry:**
     - `tokens_total` is the sum of the non-null totals;
     - `estimated_usd` is the sum of the non-null values;
     - `session_count` counts every entry.

     Run `just cost-audit` and confirm SPEC-143 passes. Quote the line that names it; if no line
     names it, say so.
4. **Mark ship `[x]`** in the timeline, with the merge (`06eb552`, PR #235, 2026-10-31) and the
   total cost. Add a `[x]` re-verify line only if the timeline has an open one; it should not.
5. **Advance and archive.** Run `just advance-cycle SPEC-143 ship`, then
   `just archive-spec SPEC-143`. Then read STAGE-054's SPEC-143 entry back.
   - It should read `- [x] **SPEC-143** (merged 2026-10-31, PR #235) — …`. Fix it by hand if the
     script left it otherwise. Touch no other entry.
   - Recount the `**Count:**` line with a grep you have just run, not by arithmetic. Today it
     reads `3 shipped / 1 active / 20 pending`. Expect `4 shipped / 0 active / 20 pending`, plus
     any entry you file in step 2.3.
6. **Check what the tooling sees.**
   - Run `just status` and `just backlog`. Confirm SPEC-143 shows as shipped.
   - Run `just roadmap`: STAGE-054 must stay **active**. ⛔ **Do not run Stage Ship**, and do not
     mark STAGE-054 shipped; the orchestrator decides that.
7. **Commit to `main` directly.** Ship bookkeeping does not go through a PR (AGENTS §13).
   - Commit:
     `git commit -s -m "ship(SPEC-143): a watermark that draws nothing is refused — reflected, archived"`,
     with the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. **The DCO
     sign-off is mandatory.**
   - Push: `git fetch origin && git rebase origin/main && git push origin HEAD:main`. Other
     sessions push to `main` too; if the push is rejected, rebase and push again.
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
- Identify it **by content, not recency**: it contains `SHIP prompt: a watermark`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran, never at a flat rate: Sonnet $3/$15 per
  MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-143 ship: archived — <commit SHA on main>`. Then:
- the three reflection answers, one line each;
- any follow-up you filed, or "none" and why;
- the re-measured build and punch-list entries, old → new;
- the two new cost entries (verify and ship), each with cycle, model, tokens and USD;
- `cost.totals` (tokens, USD, session count) and the `just cost-audit` result;
- the recounted `**Count:**` line, and confirmation that STAGE-054 is still active;
- any template, constraint or DEC change you propose;
- your measured cost.
