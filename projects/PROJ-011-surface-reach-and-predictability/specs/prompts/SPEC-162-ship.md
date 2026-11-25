# SPEC-162 — SHIP prompt: CI runs each test once

Cycle: **ship**. This cycle is bookkeeping, two spec wording fixes and a reflection; there are no code
changes. Fresh CLI session, own worktree. Sonnet is enough:

```
just wt docs/spec-162-ship -- --model sonnet
```

The maintainer pastes this prompt into a fresh session and will paste your final message back to the
orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

## Step 0: check the merge, then get onto current `main`
1. `git fetch origin && git checkout --detach origin/main`.
2. ⛔ **Go on only if both of these hold.** Otherwise, stop and report.
   - **PR #271 is merged.** Run `gh pr view 271 --json state,mergeCommit,mergedAt,headRefOid`. The
     re-verify approved head `793dc0c`. Record the merge SHA, the date and the merged head. If the
     head differs from `793dc0c`, confirm that
     `git log --first-parent --no-merges --oneline 793dc0c..<head>` lists nothing and that
     `git show --remerge-diff <head>` is empty; stop if either disagrees.
   - **The timeline**
     (`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-162-ci-runs-each-test-once-timeline.md`)
     has build, verify, punch list and re-verify all `[x]`.
3. **Confirm the merge brought the work onto `main`.**
   - `.github/workflows/ci.yml` has no job named `avif`, and the `check` job's `Swatinem/rust-cache@v2`
     step carries `if: needs.changes.outputs.code == 'true'`.
   - `tests/avif_tile_pin.rs` joins `"avif-probe"` onto `CARGO_TARGET_TMPDIR`, with no `DefaultHasher`.

   If either is missing, stop and report.

## Read first
- `/AGENTS.md`: §4 (cost), §13 (ship bookkeeping lands on `main`), §15 (the coverage map, "a bound
  is a claim until its worst input is driven", the finding classes, "During ship").
- **The spec in full**, especially the ACs, the mutation table, `## Build Completion` with its
  `### Punch list` subsection, build's reflection and the cost block.
- **The timeline**, all of it. It summarises verify and re-verify, whose reports live only there.
- **STAGE-063's SPEC-162 entry** and its `**Count:**` line
  (`projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`).

## The facts to reflect on
- **Frame and design were folded into the spec by the orchestrator** (Opus, 2026-11-24), with no
  separate sessions. The spec measured baseline run `45501714010` (102.6 runner-minutes) and the 179
  leaked probe folders (229 GB) that started it.
- **Build (Sonnet, $2.17 as measured before its last edits).** 102.6 → 62.3 runner-minutes, cache
  kept. M3 did not match AC-5: with one `CARGO_TARGET_DIR` shared by two worktrees, cargo reused the
  other checkout's outer test binary. The orchestrator's AC-5 was wrong, against a lesson already in
  its own memory (a target folder shared across worktrees serves a stale library).
- **Verify (Opus, $3.37).** Two claims: macOS's Build + Test saving held on one of two warm runs
  (−22.5%, then −15.5%), and the ungated cache step restored ~1.75 GB on every docs-only run. One
  wording item: the probe hash protected nothing (a driven 1 s false green), plus loose comments and
  spec text. Three out-of-spec findings, filed by the orchestrator in `guidance/ideas.yaml`.
- **The orchestrator's rulings:** keep the cache, with AC-7 restated to the Build step (−71–87% on
  every warm leg; the macOS Test step swings by more than the 20% bar); drop the hash.
- **Punch list (Sonnet, $1.34 as measured before its last push).** Gated the cache step, dropped the
  hash, restated AC-5/AC-7. Its first M1/M2 attempt deleted the line and measured compile failures;
  the session caught it and redid both.
- **Re-verify (Opus, $1.70): approved at `793dc0c`, no claims.** It confirmed from rust-cache's
  source that a full key match saves nothing, and that `main`'s first run after the merge saves a
  fresh `check` entry including `target/tmp/avif-probe` (only if that run is green). Two wording items
  left for ship.

## Do, in order
1. **Mark ship `[~]` in the timeline**, and add a `merge` line above it:
   `- [x] **merge** (<date>, maintainer) — PR #271 as <merge SHA>, head <head>.` Check it against `gh`.
2. **Read `main`'s CI at the merge commit once**:
   `gh api repos/jysf/crustyimg/commits/<merge SHA>/check-runs`. If legs are still running, say so;
   don't wait.
   - If the run has finished green, list the caches
     (`gh api repos/jysf/crustyimg/actions/caches --jq '.actions_caches[] | "\(.key) \(.ref) \(.size_in_bytes)"'`)
     and confirm three `check` entries now sit on `refs/heads/main`. Give their sizes and the total.
   - Add one line with the result to the spec's `### Punch list` subsection.
3. **The re-verify's two wording fixes**, in the spec on `main`:
   1. In `## Build Completion`, the AC-5 row still reads "partly as written" and names the old hashed
      folders. Keep that text as history, and add one line after it: AC-5 was restated at the punch
      list and is met by the probe's doc comment.
   2. In `## Acceptance Criteria`, only AC-5 is ticked. Tick AC-1–4 and AC-6–10 too, since
      `## Build Completion` gives evidence for each.
4. **Write `## Reflection (Ship)`.** Outcome-focused: three answers, drawn only from the evidence.
   1. **What would I do differently?** Weigh at least these:
      - **Folding frame and design into the orchestrator.** Two of the three findings were design
        errors: AC-5 contradicted a known lesson, and AC-7 put a 20% bar on Build + Test when the cache
        can only touch Build. Would a separate design session have caught them, or only a different
        rule?
      - **One warm sample against a bar.** The build measured once and called it met; verify found the
        second sample under the bar, and a second baseline 50% slower than the first.
      - **A guard nobody drove.** The probe hash shipped with a doc sentence claiming a protection
        that M4 never showed.
   2. **Does any template, constraint or decision need updating?** Candidates:
      - "an AC that measures a saving names the step the change can affect, not a sum that includes
        steps it can't";
      - "a CI timing bar needs two samples on each side, or a stated noise band";
      - "an orchestrator that folds design into the spec checks its ACs against its own saved
        lessons".

      Check AGENTS §15–16, HANDOFF and the recent ship reflections (SPEC-155, 156, 157) first, and say
      which already covers each. **Propose; don't apply.**
   3. **A follow-up spec?** Three ideas are already filed (`ci-classifier-pr-diffs-the-whole-branch`,
      `native-lean-plus-avif-build`, `rust-cache-on-feature-legs`). File nothing else unless the
      evidence names a gap none of them covers.
5. **Cost.** Every session must have an entry in `cost.sessions`:
   - **frame + design:** folded into the orchestrator's own session, which is not metered per spec.
     Add one `design` entry with `tokens_total: null` and a note saying so, as AGENTS §4 allows for
     un-metered main-loop cycles.
   - **build** and **punchlist** are present; re-measure both (each was taken before its last turn).
   - **verify** and **reverify** are missing; measure and add both.
   - your own **ship** entry.

   How to measure:
   - **Find each transcript by its marker**, the prompt's title line on `main`:
     `SPEC-162 — BUILD prompt`, `SPEC-162 — VERIFY prompt`, `SPEC-162 — PUNCH LIST prompt`,
     `SPEC-162 — RE-VERIFY prompt`, `SPEC-162 — SHIP prompt`. `VERIFY prompt` is a suffix of
     `RE-VERIFY prompt`, so match the full title with the `SPEC-162 — ` prefix and the colon after it.
     Read each prompt's first line to confirm. Use the transcripts whose **first user message** holds
     the title, because the orchestrator's transcript contains every title.
   - **Dedup by `.message.id`.** Count the three input counts once per id, and take the **MAX**
     `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
   - **Price** at the model in `.message.model`: Sonnet $3/$15, Opus $5/$25. cache_creation is ×1.25
     the input rate and cache_read ×0.10.
   - **Recompute `cost.totals`** from every entry, your own included. Measure your own entry **after**
     your push. Run `just cost-audit` and quote its line.
6. **Mark ship `[x]`** with the merge SHA, PR #271, the date and the total cost.
   - Run `just advance-cycle SPEC-162 ship` and `just archive-spec SPEC-162`.
   - Read STAGE-063's SPEC-162 entry back. It should read
     `- [x] **SPEC-162** (merged <date>, PR #271) — [S] …`, in the style of the SPEC-157 entry. Fix it
     by hand if not. Touch no other entry.
   - Recount the `**Count:**` line with greps you just ran, and show them. Today it reads
     `7 shipped / 0 active / 17 pending`; expect `8 shipped / 0 active / 16 pending`, unless another
     session changed the stage meanwhile.
   - ⛔ Don't change STAGE-063's `status` or its reflection.
7. **Commit to `main` directly** (AGENTS §13).
   - `git commit -s -m "ship(SPEC-162): CI runs each test once — reflected, archived"` with the
     trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is mandatory.
   - Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`. On a reject,
     rebase and push again. A "Changes must be made through a pull request" notice can still mean the
     push landed: check with `git ls-remote origin refs/heads/main`.
   - Read back `git show --stat HEAD`.
   - Then measure your own cost. If it moved, amend nothing: push one more commit that updates only
     your `ship` entry and `cost.totals`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, `docs/`, HANDOFF, `guidance/` or templates. Propose anything else in
  your report.
- The spec's body changes only by step 2's CI line, step 3's two fixes, the reflection and the cost
  block.
- **Never poll CI,** and never write your own wait loop.
- Another session may push to `main` while you work. Rebase; never force.
- A piped command reports the pipe's exit code. zsh does not word-split unquoted parameters, and it
  reads `$R:r…` as a modifier, so brace it (`${R}:`). `rtk` can truncate `grep`, so count with
  `/usr/bin/grep` or Python.
- Write scratch files only in your session's scratchpad, never `/tmp`.
- **Budget ~25 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-162 ship: archived — <commit SHA on main>`. Then:
- the merge SHA, date and merged head;
- `main`'s CI at the merge, and the `check` cache entries on `main` (or "still running");
- the two wording fixes, done;
- STAGE-063's SPEC-162 line as it now reads, and the recounted `**Count:**` line with its greps;
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- any template, constraint or DEC change you propose;
- your measured cost after your last push: input, cache_creation, cache_read, output, total tokens
  and dollars, each on its own line.
