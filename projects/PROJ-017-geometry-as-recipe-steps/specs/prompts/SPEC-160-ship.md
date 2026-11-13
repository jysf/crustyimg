# SPEC-160 — SHIP prompt: edit --pad and --trim, and parse_color refuses a sign

Cycle: **ship**. Bookkeeping, plus DEC-115's replacement text, which design wrote and verify
checked against the code. Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-160-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. ⛔ **Proceed only if both hold.** Otherwise stop and report.
   - PR #264 is merged: `gh pr view 264 --json state,mergeCommit,mergedAt`. It was merged as
     `decf7f3` (`mergedAt` 2026-11-13T20:07:54Z) when this was written; record what you see.
   - The timeline
     (`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-160-edit-pad-and-trim-and-parse-color-refuses-a-sign-timeline.md`)
     has build, verify and punch list all `[x]`.
3. Confirm the merge brought the work onto `main`: `tests/edit_pad_trim.rs` exists, and the spec's
   `cost.sessions` has `build` and `punchlist` entries. If not, stop and report.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full: Frame with the maintainer rulings, Design (corrections 1–8, the mutation
  table, "DEC-115's ship-time replacement text"), `## Build Completion` with its punch-list
  subsection, and the cost block.
- The timeline, all of it. Its verify entry summarises the verify report; the details you need
  are below.
- DEC-115 (`decisions/DEC-115-a-step-whose-alpha-the-output-format-cannot-store-is-refused-before-the-pipeline-runs.md`).
- STAGE-058's `## Spec Backlog`, all of it
  (`projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`).
- SPEC-159's `## Reflection (Ship)` (in `specs/done/`). It proposed three template changes that
  were not applied:
  - build drives each doc sentence on every exit path, not just one;
  - design re-reads its DEC against its own corrections before it hands off;
  - the build's file list includes what `just wasm-test` touches when `src/wasm.rs` changes.
  Say whether SPEC-160 ran under each (the build and verify prompts asked for the first; design
  wrote DEC-115's replacement text, which is the second), and what happened.

## The facts to reflect on (from frame to punch list)
- **The frame ($4.79)** confirmed the premise (`edit --pad`/`--trim` were unknown flags, exit 2).
  It drove `round((N/long)·long) == N` for all 2.5e9 pairs up to 50,000, and showed `+f+f+f`
  loads on all four routes as `#0f0f0f`. It put three questions. The maintainer took every
  recommendation: entries 1–4 moved out and the nine-gravities pin folded in, so this spec closes
  STAGE-058; [M] uncut; one `--pad N|W:H`.
- **Design ($14.47)** corrected the frame eight ways and drove all 22 mutation rows on a
  prototype. The largest corrections:
  - `--pad N` past the long edge fails at apply;
  - no named colours;
  - the batch-wide format is `--format` or the template, never `-o`;
  - `edit`'s alpha line names the flag through a new `IntroducedAlpha.flag` (a library-only
    breaking change).
  - It also wrote DEC-115's replacement text for ship to apply.
- **Build ($7.27, measured before its CI-fix turns)**:
  - Drove every behaviour sentence on every exit path, and all 22 mutation rows.
  - It found four incomplete table cells (M6a, M9, M10a, M10b). Trap 14 stopped at the lib's
    compile error, so its integration rows didn't run.
  - CI's Windows leg failed once, on the build's own `/in/a.jpg` asserts; it normalised the
    separators.
  - All 13 files were exactly the spec's Outputs.
- **Verify ($8.36)** drove everything independently and found no behaviour or assertion defect.
  - It drove 66 message rows exactly, and `parse_color` on 113 rows across 7 routes.
  - It decoded pad fills at `c × 257` with ImageMagick.
  - It checked the nine gravities against its own model.
  - It drove six mutations, including trap 14 on the integration targets alone. There,
    `edit_geometry` was 21 green and 2 red, not "all 23 green".
  - Default 1350 and lean 1286 passed.
- **The only defects were wording, three of them:**
  - `run_pixel_op`'s comment still said only a forced-format caller can carry an alpha step. That
    has been false since this spec. Design wrote replacement text for the DEC that said the same
    thing, but not for the code comment.
  - A test comment named DEC-095.
  - The docs didn't say a saved `--pad N` scales on replay (7 px on 60 → 14 px on 120). The
    sentence was true but omitted the consequence; the maintainer chose to add it.
  - The punch list (~$0.50) fixed all three. No re-verify was needed (AGENTS §15).
- **Verify's optional cleanup 2** (DEC-115 wording) is accepted; see step 2.

## Do, in order
1. **Mark ship `[~]`** in the timeline, and add a `merge` line above it:
   `- [x] **merge** (2026-11-13, maintainer) — PR #264 as <merge SHA>.` Check it against `gh`.
2. **Apply DEC-115's replacement text** from the spec's "DEC-115's ship-time replacement text",
   exactly as written, with one settled change. In Part 4's new text, write "the format resolved
   for each input" in place of "each input's own format". For one input with `-o x.jpg`, the
   format checked comes from `-o`, not the source.
   - Find each target sentence by content. If one no longer reads as the spec quotes it, stop that
     edit and report the sentence as it stands.
   - Read the code at the lines verify cited (`src/cli/ops.rs`: the forced check, the gate, the
     batch-wide check, the two per-input checks; find each by content) and confirm the new text
     says what they do.
   - `affected_scope` needs no new path (the spec says why). Check it against
     `git diff --name-only decf7f3^1 decf7f3 -- src/`, and say if anything is missing.
   - No new decision, and no change to any part's rule. Run `just decisions-audit` and quote its
     line.
3. **Fold the table fixes into the spec.** Find each row in the design's mutation table by its
   `**Mn` label, and mark each fix `(corrected at ship)` in place. Only these, the reflection and
   the cost block change in the spec body:
   - **M6a:** add `tests/trim.rs` `a_non_ascii_colour_is_refused_not_a_panic` and
     `every_trim_refusal_names_its_reason` to the red set.
   - **M9:** add FT-4 and FT-9 (both build and verify drove them red).
   - **M10a:** add `tests/pad.rs` `every_colour_type_pads_exactly` and
     `gravity_corners_land_by_absolute_position` (build drove these; verify did not re-drive M10a).
   - **M10b:** add `every_recipe_output_matches_its_golden` (the `v2_ddphotos_cli_shape` recipe).
   - **Trap 14:**
     - the lib's unit tests don't compile (E0063), so `cargo test` stops there;
     - run alone, the integration targets give FT-1 to FT-13 red and FT-17 green;
     - `edit_geometry` is 21 green and 2 red (`edit_help_states_typed_order`,
       `no_op_flag_message_lists_every_flag`, both FT-18), not all 23 green.
   - **The table's preamble:** `no_op_flag_message_lists_every_flag` "is red in every row" held on
     the prototype only. On the build it was red in no row.
4. **Write `## Reflection (Ship)`**, outcome-focused, three answers drawn only from the evidence:
   1. **What would I do differently?** Weigh at least these:
      - Design wrote DEC replacement text for a rule its own change made stale, but left the code
        comment that said the same thing. A design's "what this change makes false" sweep should
        cover comments as well as DECs.
      - The mutation table again had four incomplete cells and a trap-14 row nobody could run as
        written.
      - The docs were true but omitted the replay consequence, which no "drive each sentence"
        rule catches.
      - The Windows leg caught a separator assumption only after the push.
   2. **Does any template, constraint or decision need updating?** Candidates:
      - SPEC-159's three proposals, now with a second spec of evidence;
      - "when a change makes a rule stale, grep comments as well as DECs for the old rule";
      - a mutation row's red set is read off a full `--no-fail-fast` run, every target, not the
        FT list;
      - a test that asserts a path compares it after separator normalisation.
      Check AGENTS §15–16, HANDOFF, `guidance/` and SPEC-152's, SPEC-153's and SPEC-159's
      reflections first, and say which already covers each. **Propose it; don't apply it.**
   3. **A follow-up spec?** Check the entries this spec moved out (three in `guidance/ideas.yaml`,
      one in STAGE-060) and the frame's earlier ideas. File **one** small entry only if the
      evidence names a gap nothing covers, or say why none.
5. **Cost.** Every session's entry must be in `cost.sessions`: frame, design, build, verify,
   punchlist, and your own `ship`.
   - **verify is not recorded.** It reported 12,117,906 tokens, $8.36 (Opus), measured before its
     last message. Measure it from its transcript and record that.
   - **Re-measure build and punchlist.** Each was measured before its own last turns, and build's
     omits its CI-fix turns. Re-measure frame and design too, if their notes say the same.
   - **Find each transcript by its marker.** The markers are each prompt's title on `main`:
     - `SPEC-160 — FRAME prompt`
     - `SPEC-160 — DESIGN prompt`
     - `SPEC-160 — BUILD prompt`
     - `SPEC-160 — VERIFY prompt`
     - `SPEC-160 — PUNCH LIST prompt`

     Read each prompt's first line to confirm its title. Use the transcripts whose **first user
     message** holds the title, because the orchestrator's transcript also contains every title.
     If a cycle has two such transcripts, measure both and record one entry with their sum; its
     note names both.
   - **Dedup by `.message.id`.** Count the three input counts once per id, and take the **MAX**
     `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
   - **Price** at the model in `.message.model`: Sonnet $3/$15, Opus $5/$25. cache_creation is
     ×1.25 the input rate and cache_read ×0.10.
   - **Check the build entry's date.** It says 2026-11-12, and the PR opened 2026-11-13T06:51Z.
     Use the first message's timestamp in local time, and say which you used.
   - Recompute `cost.totals` from every entry. Run `just cost-audit` and quote the line that names
     SPEC-160. If it fails on another spec, say which and that it isn't yours.
6. **Mark ship `[x]`** with the merge SHA, PR #264, the date and the total cost.
7. **Advance and archive.** Run `just advance-cycle SPEC-160 ship`, then
   `just archive-spec SPEC-160`.
   - Read STAGE-058's SPEC-160 entry back. It should read
     `- [x] **SPEC-160** (merged <date>, PR #264) — [M] …`. Fix it by hand if not.
   - Touch no other entry.
   - Recount the `**Count:**` line with greps you just ran, and show the greps. Today it reads
     `17 resolved (7 shipped specs, 6 folded, 4 moved) / 1 active (SPEC-160) / 0 pending`; expect
     `18 resolved (8 shipped specs, 6 folded, 4 moved) / 0 active / 0 pending`.
8. **Check what the tooling sees.** Run `just status`, `just backlog` and `just roadmap`. They show
   only the active project, so if SPEC-160 is missing, say so rather than treat it as an error.
   - ⛔ Do not run Stage Ship, and don't change STAGE-058's `status`, `shipped_at` or its
     Stage-Level Reflection. The orchestrator writes a separate Stage Ship prompt next.
   - Do report whether the backlog is complete by your count, and quote STAGE-058's five
     `## Success Criteria`. For each one, name the spec and test that meets it, or say it is
     unmet.
9. **Commit to `main` directly** (AGENTS §13).
   - Commit with
     `git commit -s -m "ship(SPEC-160): edit --pad and --trim, and a signed colour refused — reflected, archived"`
     and the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off
     is mandatory.
   - Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`. On a
     reject, rebase and push again.
   - Read back `git show --stat HEAD`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, `docs/`, HANDOFF, templates or any DEC but DEC-115 (step 2's text
  only). Propose anything else in your report.
- The spec's body changes only by step 3's corrections, the reflection and the cost block.
- Another session may push to `main` while you work. Rebase; never force.
- A piped command reports the pipe's exit code.
- zsh does not word-split unquoted parameters, and it reads `$R:r…` as a modifier, so brace it
  (`${R}:`).
- `rtk` can truncate `grep`, so count with `/usr/bin/grep` or Python.
- Write scratch files only in your session's scratchpad, never `/tmp`.
- **Budget ~45 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-160 ship: archived — <commit SHA on main>`. Then:
- the merge SHA and date;
- DEC-115's changes, each quoted as it now reads, and the `just decisions-audit` line;
- step 3's corrections, one line each;
- STAGE-058's SPEC-160 line as it now reads, and the recounted `**Count:**` line;
- the five success criteria, each met (spec and test) or unmet;
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- any template, constraint or DEC change you propose;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on
  its own line.
