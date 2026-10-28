# SPEC-135 — SHIP prompt: `watermark` refuses a size or scale that is not a sensible number

Cycle: **ship**. This is bookkeeping only. The code is on `main` as `df91f7e` (PR #207,
squash-merged 2026-10-27). Re-verify approved it at `161b637`. Before merging, the maintainer
updated the branch with `main`, which brought in only orchestrator docs commits; the orchestrator
confirmed that `src/`, tests and docs are identical to `161b637`.

Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-135-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. Open the timeline:
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-135-watermark-refuses-a-size-or-scale-that-is-not-a-sensible-number-timeline.md`.
3. ⛔ **Proceed only if both of these hold.** Otherwise stop and report.
   - The re-verify line is `[x]` with ✅ APPROVED.
   - PR #207 is merged: `gh pr view 207 --json state`.

## Read first
- `/AGENTS.md` §4 (cost), §13 (verify and ship bookkeeping lands on `main`) and §15 "During
  ship".
- The spec in full, including `## Frame` with rulings 1–7, `## Design`, `## Build Completion`
  with the punch-list fix note, and the cost block.
- The timeline.
- `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for"
  1–10, and rule 6.
- STAGE-054's `## Spec Backlog`, all of it. SPEC-135's entry is at about line 69.

## Do, in order
1. **Mark ship `[~]`** in the timeline. The orchestrator may already have done it.
2. **Write `## Reflection (Ship)`.** Make it outcome-focused, with three answers drawn from the
   evidence: the Build Completion, the verify readouts in the timeline, and the merge history. Do
   not invent anything.
   1. **What would I do differently next time?** Facts to weigh:
      - Trap 10 struck again, in a new form. Every test value was exact in `f32`, so a message
        printing the value widened to `f64` passed every test (verify's mutation M-c). The design
        claimed "prints the value as parsed" without a row where "as parsed" and "widened"
        differ.
      - The design predicted that AC-3 would go RED when the cap row was removed. It cannot:
        AC-3 compares the table with itself. Another library-level test caught it instead.
      - Three reports stated something they had not observed:
        - the fix said CI was "green at `b11dd40`", but that run was cancelled when the next
          push replaced it;
        - the fix guessed that docs-only commits skip the heavy CI jobs. They do not: the full
          13-job run at `161b637` passed;
        - verify first reported a row whose binary had already been deleted (exit 127), then
          re-drove it and corrected the report itself.
      - The build left five scratch files in `~` after a failed `cd`, and deleted them.
      - What held: the goldens were committed before any `src/` change, every row was run on
        both binaries, and released 0.7.1's crash was driven on the release asset before the
        CHANGELOG claimed it.
   2. **Does any template, constraint or decision need updating?** Name it, or say none. The
      candidate to weigh: should trap 10 add "a test of printed or formatted output uses a value
      whose naive and correct renderings differ"? **Propose it; do not apply it.**
   3. **Is there a follow-up spec to write before I forget?** Read STAGE-054's backlog first.
      Already filed, so do not duplicate:
      - `render_text` panics for a library caller;
      - marks with no visible pixels;
      - the text raster memory bound;
      - the `--font` error wording.

      **File one new entry** at the end of STAGE-054's backlog, `- [ ] (not yet written) — [S]`,
      holding re-verify's two optional cleanups:
      - DEC-106 says the old cast "turned NaN and −inf into a 1×1 mark". The old code computed
        the height as `round(oh/ow).max(1)`, so a tall overlay became 1×N. Make it accurate;
      - the AC-4 table test in `tests/watermark_cli_params.rs` stops at its first failing row.
        Collect every row's failure and assert once, so a mutation that breaks several rows shows
        all of them.
3. **Cost.** `cost.sessions` holds frame ($3.93), design ($5.32), build ($4.52) and the fix
   (a `build` entry, $0.82).
   - **The build and the fix were measured before their final turns.** Re-measure both from
     their full transcripts and replace the numbers if they differ.
   - **Add two entries** (verify and re-verify), plus a third for your own `ship`.
   - Every transcript is under `~/.claude/projects/`. The `…` in each path is
     `-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg-`.

     | cycle | transcript | confirm with `grep -c` | reported |
     |---|---|---|---|
     | build (re-measure) | `…-claude-worktrees-feat-spec-135-watermark-flag-checks/f090942b-1546-4cd1-bba7-15dec9605228.jsonl` | `SPEC-135 — BUILD prompt` | $4.52 |
     | build, punch-list fix (re-measure) | `…-claude-worktrees-feat-spec-135-watermark-flag-checks/70cf330a-cd9a-40de-9ab6-bca4fcf8ee7b.jsonl` | `SPEC-135 — FIX prompt` | $0.82 |
     | verify (add) | `…-claude-worktrees-verify-spec-135/748a7643-a3ea-40c7-b275-7aaab0c08cb6.jsonl` | `SPEC-135 — VERIFY prompt` | $4.31 |
     | verify, re-verify (add) | `…-claude-worktrees-verify-spec-135-fix/c08fd840-d4cf-4683-9005-2a710dd08c79.jsonl` | `SPEC-135 — RE-VERIFY prompt` | $1.45 |

   - **Before measuring each transcript, confirm it by content** with the grep in the table.
     Read the model from each transcript's `.message.model`; don't assume it. The dollar figures
     above are the sessions' own estimates, for sanity-checking only. Do not copy them.
   - **Recompute `cost.totals` from every entry:**
     - `tokens_total` is the sum of the non-null totals;
     - `estimated_usd` is the sum of the non-null values;
     - `session_count` counts every entry.

     Run `just cost-audit` and confirm SPEC-135 passes.
4. **Mark ship `[x]`** in the timeline, with the merge (`df91f7e`, PR #207, 2026-10-27) and the
   total cost.
5. **Advance and archive.** Run `just advance-cycle SPEC-135 ship`, then
   `just archive-spec SPEC-135`. Then read STAGE-054's SPEC-135 entry back.
   - It should read `- [x] **SPEC-135** (merged 2026-10-27, PR #207) — …`. Fix it by hand if the
     script left it otherwise. Touch no other entry except the one you file in step 2.3.
   - Recount the `**Count:**` line with a grep you have just run, not by arithmetic. Expect
     1 shipped / 0 active / 11 pending, counting your new entry.
6. **Check what the tooling sees.**
   - Run `just status` and `just backlog`. Confirm SPEC-135 shows as shipped and your new entry
     appears.
   - Run `just roadmap`: STAGE-054 must stay **active**. ⛔ **Do not run Stage Ship**, and do not
     mark STAGE-054 shipped. Its other items are for 0.8.1.
7. **Commit to `main` directly.** Ship bookkeeping does not go through a PR (AGENTS §13).
   - Commit: `git commit -s -m "ship(SPEC-135): watermark refuses a size or scale that is not a sensible number — reflected, archived"`.
     **The DCO sign-off is mandatory.**
   - Push: `git fetch origin && git rebase origin/main && git push origin HEAD:main`.
   - Read back `git show --stat HEAD` and check it is what you meant.

## Guardrails
- ⛔ No version bump, no tag, no release and no CHANGELOG edit. Release prep runs after you, in
  separate sessions.
- Do not edit code, tests or DECs. That includes DEC-106: its wording fix is the backlog entry
  you file. If the reflection says a template, constraint or DEC needs changing, propose the edit
  in your report; the orchestrator decides.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~40 exchanges.**

## Measure cost (yours and the transcripts above)
- Your own transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Identify it **by content, not recency**:
  `grep -l "SPEC-135 — SHIP prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran, never at a flat rate: Sonnet $3/$15 per
  MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-135 ship: archived — <commit SHA on main>`. Then:
- the three reflection answers, one line each;
- the follow-up you filed;
- the re-measured build and fix entries, old → new;
- the three new cost entries, ship included (cycle, model, tokens and USD for each);
- `cost.totals` (tokens, USD, session count) and the `just cost-audit` result;
- the recounted `**Count:**` line, and confirmation that STAGE-054 is still active;
- any template, constraint or DEC change you propose;
- your measured cost.
