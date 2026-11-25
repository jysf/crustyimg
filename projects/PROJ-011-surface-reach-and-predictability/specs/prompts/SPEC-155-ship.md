# SPEC-155 — SHIP prompt: a write that fails must not report success or destroy the old file

Cycle: **ship**. Bookkeeping, one DEC wording fix the re-verify found, and the CI line the punch list
left as a placeholder. Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-155-ship -- --model sonnet
```

The maintainer pastes this prompt into a fresh session and will paste your final message back to the
orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. ⛔ **Proceed only if both hold.** Otherwise stop and report.
   - PR #268 is merged: `gh pr view 268 --json state,mergeCommit,mergedAt`. The orchestrator read it as
     squash commit `7819db6` (head `41971af`), merged 2026-11-23. Record what you see.
   - The timeline
     (`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-155-a-write-that-fails-must-not-report-success-or-destroy-the-old-file-timeline.md`)
     has build, verify, punch list and re-verify `[x]`.
3. Confirm the merge brought the work onto `main`: `src/sink/publish.rs` exists and holds
   `is_symlink()` and the test `the_old_mode_is_on_the_staged_file_before_the_first_write`;
   `decisions/DEC-116-a-file-write-is-all-or-nothing.md` exists. If not, stop and report.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full, especially Frame and the maintainer rulings, "Design corrections", the ACs, the
  mutation table (M12–M15 were added at the punch list), the DEC's verbatim copy, `## Build
  Completion` with `### Punch list (verify)`, build's reflection, and the cost block.
- The timeline, all of it.
- DEC-116 in full.
- STAGE-063's `## Spec Backlog`
  (`projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`):
  the SPEC-155 and SPEC-157 entries. SPEC-157 builds on this spec's `publish_file`.

## The facts to reflect on (from frame to re-verify)
- **The frame ($4.18)** drove the premise on a `main` release build under `ulimit -f 1` with XFSZ
  ignored: outputs under 8 KiB exited **0** with a truncated 512-byte PNG; larger ones exited 5 with
  the old file destroyed; `--save-recipe` overwrote without `-y`. It recommended a new DEC and put four
  rulings to the maintainer (read-only directory, fsync, `--save-recipe` guard, full disk), all
  accepted.
- **Design ($16.03)**, the most expensive cycle of any STAGE-063 spec so far: one hand-rolled helper,
  `src/sink/publish.rs`, under the four sink arms, `--save-recipe` and `write_lock`; 22 ACs, 39 tests,
  15 mutation rows driven on a prototype. It corrected the frame three ways (a rename replaces a
  `0444` file, so a write-probe keeps it refused; std's Windows rename tries `MoveFileExW` first; the
  `.tmp` suffix, not the leading dot, keeps staged files out of inputs). **But the two bugs verify
  found were in design's own `publish` steps**: the mode was read through `symlink_metadata` with no
  symlink case, and applied after the bytes were written.
- **Build (Sonnet, $7.61 as measured):** followed the spec exactly; baseline exactly the 15 E rows red
  on `main`; 22 ACs met; mutations matched except M1/M2/M3 also reddened E23, and M11 was 48, not 46.
- **Verify ($4.47):** drove every failure path on release builds; the writer sweep found no missed
  writer. **Two claims** (the symlinked lockfile: a `0600` target became a `0755` regular file, a
  dangling link exited 5 where `main` exited 0; and a killed `-y` over `0600` left a `0644` staged file
  holding the new bytes), **two asserts** (the char-boundary cut and the `.tmp` suffix had no row), and
  three wording items. Out of spec: `optimize`'s "kept the source unchanged" note →
  `guidance/ideas.yaml`.
- **Punch list (Sonnet, $2.56 measured before its last commits):** both claims fixed (FT-E24, FT-E25,
  FT-U14), both pins added (FT-U9b, FT-U13), M12–M15 each red on its row, DEC-116 narrowed. It left
  `CI_SNAPSHOT_PLACEHOLDER` in the spec.
- **Re-verify (Opus, $2.52):** ✅ APPROVED, but **retroactively**: the maintainer merged #268 before it
  ran. It drove four lockfile shapes on `main`, the pre-punch head and the merge; the merge matches
  `main` on all three symlink cases and keeps `0600` on a regular file (where `main` never kept the
  mode). It drove fix 2 on both sides (`0644` → `0600`, `0640` kept, `0400` still refused by the
  probe), M12, M13 and a new M16. One **wording** item (below). Out of spec: `build --check`'s
  `load_lock` reads a symlinked lockfile through the link (read-only, pre-existing).

## Do, in order
1. **Mark ship `[~]`** in the timeline, and add a `merge` line above it:
   `- [x] **merge** (<date>, maintainer) — PR #268 as <merge SHA>, before the re-verify ran.` Check it
   against `gh`.
2. **The DEC wording fix (re-verify's item 1).** DEC-116's bullet reads "…on every output except the
   build lockfile, which has no symlink guard: a symlinked lockfile is replaced by a regular file, and
   nothing is read from or written through the link." The lockfile has **neither** guard, and
   `build --check` does read through the link. Replace that span with:

   > …on every output except the build lockfile, which has neither guard: it is always rewritten, a
   > symlinked lockfile is replaced by a regular file, and nothing is written through the link.

   Make the same change to the spec's verbatim copy, byte for byte, marked `(corrected at ship)`.
   Change nothing else in the DEC. Run `just validate` and quote its result.
3. **Fill the CI placeholder.** Replace `CI_SNAPSHOT_PLACEHOLDER` in `### Punch list (verify)` with
   the PR's last snapshot, which the orchestrator and the re-verify both read: at `41971af`, 18 passed,
   8 skipped (release-only), none failed, Windows and lean included. Then read `main`'s CI for the
   merge commit once (`gh api repos/jysf/crustyimg/commits/<merge SHA>/check-runs`) and add one line
   with its result. If legs are still running, say so; don't wait.
4. **Write `## Reflection (Ship)`**, outcome-focused, three answers drawn only from the evidence:
   1. **What would I do differently?** Weigh at least these:
      - design cost $16.03 and drove 15 mutation rows on a prototype, yet both claims verify found
        were in design's own steps. A mutation table only tests the code against the rows someone
        wrote; neither "a symlinked destination with no guard" nor "a process killed between write and
        chmod" was a row. What would have found them at design?
      - the build followed the spec exactly and couldn't have caught them;
      - the PR merged before its re-verify. It came back clean, but nothing stopped the merge.
   2. **Does any template, constraint or decision need updating?** Candidates:
      - "a step that reads metadata of a destination says what it does when the destination is a
        symlink";
      - "an ordering of security-relevant steps (mode, write, rename) gets a row that observes the
        state between steps, not just the end state" (FT-U14's spy shape);
      - "a PR with a pending re-verify carries a 'do not merge' marker" (it stayed a draft; was that
        enough?).
      Check AGENTS §15–16, HANDOFF and recent ship reflections (SPEC-154's and SPEC-156's) first, and
      say which already covers each. **Propose it; don't apply it.**
   3. **A follow-up spec?** `optimize`'s note is in `guidance/ideas.yaml`. The `--check` read-through
      is read-only and now outside the DEC's claim: file it in `guidance/ideas.yaml` only if you think
      it should be refused, and say why. File nothing else unless the evidence names a gap.
5. **Cost.** Every session's entry must be in `cost.sessions`: frame, design, build, verify,
   punchlist, a new `reverify` entry, and your own `ship`.
   - **Re-measure build and punch list.** Both were measured before their last turns.
   - Check verify's entry (recorded by the punch list from verify's report) against its transcript.
   - **The re-verify:** its report gives input 70, cache_creation 84,434, cache_read 3,054,291, output
     18,618, total 3,157,413, $2.52 (Opus, transcript `0ee4d00f…`). Its worktree also holds an earlier
     transcript (`353633f7…`) that contains the prompt; check whether that one did any work, and if it
     did, add it to the entry and say so.
   - **Find each transcript by its marker.** The markers are each prompt's title on `main`:
     - `SPEC-155 — FRAME prompt`
     - `SPEC-155 — DESIGN prompt`
     - `SPEC-155 — BUILD prompt`
     - `SPEC-155 — VERIFY prompt`
     - `SPEC-155 — PUNCH LIST prompt`
     - `SPEC-155 — RE-VERIFY prompt`

     Read each prompt's first line to confirm its title. Use the transcripts whose **first user
     message** holds the title, because the orchestrator's transcript also contains every title.
   - **Dedup by `.message.id`.** Count the three input counts once per id, and take the **MAX**
     `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
   - **Price** at the model in `.message.model`: Sonnet $3/$15, Opus $5/$25. cache_creation is
     ×1.25 the input rate and cache_read ×0.10.
   - `cost.totals` reads 0 today. Recompute it from every entry, your own included, measured **after**
     your push (SPEC-156's ship recorded a figure taken too early, and the orchestrator had to correct
     it). Run `just cost-audit` and quote its line.
6. **Mark ship `[x]`** with the merge SHA, PR #268, the date and the total cost. Then run
   `just advance-cycle SPEC-155 ship` and `just archive-spec SPEC-155`.
   - Read STAGE-063's SPEC-155 entry back. It should read
     `- [x] **SPEC-155** (merged <date>, PR #268) — [M] …`, in the style of the SPEC-156 entry. Fix it
     by hand if not.
   - Touch no other entry.
   - Recount the `**Count:**` line with greps you just ran, and show them. Today it reads
     `5 shipped / 0 active / 18 pending`; expect `6 shipped / … / 17 pending`, unless another session
     changed the stage meanwhile (SPEC-157's build may be running; it doesn't tick its own entry).
   - ⛔ Don't change STAGE-063's `status` or its reflection.
7. **Commit to `main` directly** (AGENTS §13).
   - Commit with
     `git commit -s -m "ship(SPEC-155): a file write is all or nothing — reflected, archived"` and the
     trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is mandatory.
   - Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`. On a
     reject, rebase and push again.
   - Read back `git show --stat HEAD`. Then measure your own cost and, if it moved, amend nothing:
     push one more commit that updates only your `ship` entry and `cost.totals`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, `docs/`, HANDOFF or templates. The only DEC edit is step 2's. Propose
  anything else in your report.
- The spec's body changes only by step 2's copy, step 3's CI line, the reflection and the cost block.
- Another session may push to `main` while you work. Rebase; never force.
- A piped command reports the pipe's exit code. zsh does not word-split unquoted parameters, and it
  reads `$R:r…` as a modifier, so brace it (`${R}:`). `rtk` can truncate `grep`, so count with
  `/usr/bin/grep` or Python.
- Write scratch files only in your session's scratchpad, never `/tmp`.
- **Budget ~30 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-155 ship: archived — <commit SHA on main>`. Then:
- the merge SHA and date;
- DEC-116's sentence as it now reads, with the `just validate` result;
- the CI line as it now reads, and `main`'s CI at the merge;
- STAGE-063's SPEC-155 line as it now reads, and the recounted `**Count:**` line with its greps;
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- any template, constraint or DEC change you propose;
- your measured cost after your last push: input, cache_creation, cache_read, output, total tokens
  and dollars, each on its own line.
