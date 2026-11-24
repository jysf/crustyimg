# SPEC-157 — SHIP prompt: build destinations have one identity

Cycle: **ship**. This cycle is bookkeeping and a reflection; there are no code changes. Fresh CLI
session, own worktree. Sonnet is enough:

```
just wt docs/spec-157-ship -- --model sonnet
```

The maintainer pastes this prompt into a fresh session and will paste your final message back to the
orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

## Step 0: check the merge, then get onto current `main`
1. `git fetch origin && git checkout --detach origin/main`.
2. ⛔ **Go on only if both of these hold.** Otherwise, stop and report.
   - **PR #270 is merged.** Run `gh pr view 270 --json state,mergeCommit,mergedAt,headRefOid`.
     - The orchestrator approved it at head `afbee0c`, after punch list 2. Record the merge SHA, the
       date and the head that merged.
     - If the head that merged isn't `afbee0c`, list the extra commits
       (`git log --oneline afbee0c..<head>`) and stop if any of them is more than a merge from `main`.
   - **The timeline**
     (`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-157-build-destinations-have-one-identity-timeline.md`)
     has build, verify, punch list, re-verify and punch list 2 all `[x]`. The orchestrator ticks punch
     list 2 when it approves the merge.
3. **Confirm the merge brought the work onto `main`.**
   - `src/cli/build.rs` holds `check_destinations`, `READABLE_EXTENSIONS` and
     `readable_extensions_match_the_source_filter`.
   - `tests/build_destinations.rs` holds `macos_sharp_s_over_its_own_source_is_refused`.

   If either is missing, stop and report.

## Read first
- `/AGENTS.md`:
  - §4 (cost);
  - §13 (ship bookkeeping lands on `main`);
  - §15: "During design" (the coverage map) and "During ship".
- **The spec in full.** Especially:
  - Frame;
  - the design findings (the NFC/NFD residual);
  - the ACs;
  - the mutation table (M15–M19 were added at the punch list);
  - the message list;
  - the verbatim doc copies;
  - `## Build Completion`, with both punch-list subsections;
  - build's reflection;
  - the cost block.
- The timeline, all of it.
- **STAGE-063's `## Spec Backlog`**
  (`projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`).
  Read these entries:
  - the SPEC-157 entry;
  - the NFC/NFD entry, which the orchestrator extended to `ß`/`SS` and to the `--check` residual;
  - the `--check`-rewrites-outputs entry.
- **The SPEC-155 and SPEC-156 ship reflections**, in `specs/done/`, and **AGENTS §15's coverage-map
  rule**. The rule came from SPEC-155's ship and landed *after* SPEC-157's design. SPEC-157's verify
  was the first to apply it, after the fact.

## The facts to reflect on (from frame to punch list 2)
- **Frame and design.** Read both from the spec and the timeline.
  - The frame reproduced both review P1s on `main`:
    - `out = "src"` replaced the source, exit 0;
    - `alias/../dist` gave one file, two lock records and a passing `--check`.
  - Design built one resolver and one planning pass, re-resolved before each write, and added a
    `--watch` read carve-out. It had 34 tests and filed the NFC/NFD residual.
- **Build (Sonnet, $5.84 as measured before its last commit).**
  - It followed the spec; 24 ACs met.
  - M12 also turned FT-28 red. Verify confirmed this was only a mistake in the table cell.
- **Verify (Opus, $12.19)** applied the coverage map after the fact, and found:
  - **two claims**:
    - a source overwritten on APFS through `ß`/`SS` or NFC/NFD, exit 0;
    - a glob source writing into its own folder builds once, then refuses every later build. This
      PR introduced it.
  - **one assert**: UX1–UX3 turned no rows red, and six refusals had no row;
  - **wording**: exit 6, not 5.
- **The maintainer's rulings at the punch list:**
  - a same-file (device + inode) check, with no Unicode dependency, leaving the output-to-output
    collision as a residual;
  - the glob case is refused on the first build.
- **Punch list (Sonnet, $4.08 as measured before its last commit).**
  - Identity check: FT-35/36/37, M15.
  - Listing check: FT-38–42, M16.
  - Pins: FT-43–51, M17–M19.
  - FT-52 guards `READABLE_EXTENSIONS`, a copy of the source filter's list.
  - It added `skills/crustyimg/SKILL.md` to the scope.
- **Re-verify (Opus, $3.43): no claims.**
  - It drove both fixes on release builds of both sides, and found nothing wrongly refused.
  - Two asserts:
    - FT-52 missed an extension removed from the copy;
    - FT-35/36 could skip silently on CI.
  - Three wording items.
- **Punch list 2 (Sonnet, $1.30 as measured before its final commit and the CI wait).** It fixed all
  five. FT-52's first draft still missed the `xyz` mutant, and the session caught it.

## Do, in order
1. **Mark ship `[~]` in the timeline**, and add a `merge` line above it:
   `- [x] **merge** (<date>, maintainer) — PR #270 as <merge SHA>, head <head>.` Check it against `gh`.
2. **Read `main`'s CI at the merge commit once**:
   `gh api repos/jysf/crustyimg/commits/<merge SHA>/check-runs`. Add one line with the result to the
   spec's `### Punch list 2 (re-verify)` subsection. If legs are still running, say so; don't wait.
3. **Write `## Reflection (Ship)`.** Keep it outcome-focused: three answers, drawn only from the
   evidence.
   1. **What would I do differently?** Weigh at least these:
      - **Design never drove the two claims.** Both were cells of the coverage map that design
        didn't have yet: a name the filesystem merges, and a source that lists its own output on the
        next run. Verify applied the map after the fact and found them. Would the §15 rule have caught
        them at design? Say which cell each one is.
      - **The cost profile.** Verify cost more than build and both punch lists together. Was that the
        map doing its job, or a sign it belongs earlier?
      - **A drift guard that guarded only one direction.** FT-52 checked the copied list against
        itself. Two sessions wrote it, and both times it was half a guard.
   2. **Does any template, constraint or decision need updating?** Candidates:
      - "a test that guards a copied list probes a literal list it doesn't take from the copy";
      - "a test that skips on an environment condition fails instead under `CI`";
      - "a refusal added at a punch list needs a matching 'must not refuse' row".

      Check AGENTS §15–16, HANDOFF and the SPEC-155/156 ship reflections first, and say which of
      them already covers each candidate. **Propose; don't apply.**
   3. **A follow-up spec?** These are already filed:
      - STAGE-063's NFC/NFD + `ß`/`SS` entry (the output-to-output residual, and `--check` passing
        over it);
      - five `guidance/ideas.yaml` entries from verify and re-verify.

      File nothing else unless the evidence names a gap no entry covers.
4. **Cost.** Every session's entry must be in `cost.sessions`:
   - frame, design, build, verify, punchlist, reverify, punchlist-2, and your own `ship`.

   How to measure them:
   - **Re-measure build, punchlist and punchlist-2.** Each was measured before its last turn. The
     re-verify's entry was also measured before its final message, so re-measure it too.
   - **Check frame's and design's entries**, if present, against their transcripts. If either is
     missing, measure it.
   - **Find each transcript by its marker.** The markers are each prompt's title on `main`:
     - `SPEC-157 — FRAME prompt`
     - `SPEC-157 — DESIGN prompt`
     - `SPEC-157 — BUILD prompt`
     - `SPEC-157 — VERIFY prompt`
     - `SPEC-157 — PUNCH LIST prompt`
     - `SPEC-157 — RE-VERIFY prompt`
     - `SPEC-157 — PUNCH LIST 2 prompt`

     `PUNCH LIST prompt` is a prefix of `PUNCH LIST 2 prompt`, so match each title exactly, with the
     colon that follows it. Read each prompt's first line to confirm its title. Use the transcripts
     whose **first user message** holds the title, because the orchestrator's transcript contains
     every title.
   - **Dedup by `.message.id`.** Count the three input counts once per id, and take the **MAX**
     `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
   - **Price** at the model in `.message.model`: Sonnet $3/$15, Opus $5/$25. cache_creation is ×1.25
     the input rate and cache_read ×0.10.
   - **Recompute `cost.totals`** from every entry, your own included. Measure your own entry **after**
     your push. Run `just cost-audit` and quote its line.
5. **Mark ship `[x]`** with the merge SHA, PR #270, the date and the total cost.
   - Run `just advance-cycle SPEC-157 ship` and `just archive-spec SPEC-157`.
   - Read STAGE-063's SPEC-157 entry back. It should read
     `- [x] **SPEC-157** (merged <date>, PR #270) — [M] …`, in the style of the SPEC-156 entry. Fix
     it by hand if not. Touch no other entry.
   - Recount the `**Count:**` line with greps you just ran, and show them. Today it reads
     `6 shipped / 0 active / 17 pending`; expect `7 shipped / 0 active / 16 pending`, unless another
     session changed the stage meanwhile.
   - ⛔ Don't change STAGE-063's `status` or its reflection. All four P1 specs (SPEC-154 to 157) have
     now shipped. Say so in your report, but closing the stage or starting its release line is the
     orchestrator's call.
6. **Commit to `main` directly** (AGENTS §13).
   - Commit with
     `git commit -s -m "ship(SPEC-157): build destinations have one identity — reflected, archived"`
     and the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is
     mandatory.
   - Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`. On a
     reject, rebase and push again.
   - Read back `git show --stat HEAD`.
   - Then measure your own cost. If it moved, amend nothing: push one more commit that updates only
     your `ship` entry and `cost.totals`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, `docs/`, HANDOFF or templates. Propose anything else in your report.
- The spec's body changes only by step 2's CI line, the reflection and the cost block.
- Another session may push to `main` while you work. Rebase; never force.
- A piped command reports the pipe's exit code. zsh does not word-split unquoted parameters, and it
  reads `$R:r…` as a modifier, so brace it (`${R}:`). `rtk` can truncate `grep`, so count with
  `/usr/bin/grep` or Python.
- Write scratch files only in your session's scratchpad, never `/tmp`.
- **Budget ~30 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-157 ship: archived — <commit SHA on main>`. Then:
- the merge SHA, date and merged head;
- `main`'s CI at the merge;
- STAGE-063's SPEC-157 line as it now reads, and the recounted `**Count:**` line with its greps;
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- any template, constraint or DEC change you propose;
- your measured cost after your last push: input, cache_creation, cache_read, output, total tokens
  and dollars, each on its own line.
