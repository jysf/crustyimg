# SPEC-152 — SHIP prompt: `trim` as a recipe step

Cycle: **ship**. Bookkeeping only. Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-152-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. ⛔ **Proceed only if both hold.** Otherwise stop and report.
   - PR #258 is merged: `gh pr view 258 --json state,mergeCommit,mergedAt`. It was merged at
     `6fed684` on 2026-11-10 when this was written; record the SHA and the `mergedAt` date you see.
   - The timeline
     (`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-152-trim-as-a-recipe-step-timeline.md`)
     has build `[x]` and verify `[x]` (APPROVED).
3. Confirm the merge brought the build's work onto `main`: `src/operation/trim.rs` and
   `tests/trim.rs` exist, and the spec's `cost.sessions` has a `build` entry. If not, stop and report.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full: Frame, Design (its eight corrections), the match-rule table, the mutation table,
  `## Build Completion`, and the cost block.
- The timeline, all of it. Its verify entry is the orchestrator's summary of the verify report;
  the details you need from that report are below.
- STAGE-058's `## Spec Backlog`, all of it
  (`projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`).
- SPEC-151's `## Reflection (Ship)` (in `specs/done/`). It proposed that design drive the mutation
  rows and that an "expected all green" row name its search. SPEC-152 is the first spec to run
  under both.

## The facts to reflect on (from design, build and verify)
- **Verify approved with no punch list,** the first spec in PROJ-017 to do so. Its own oracles: every
  message compared by script with a planted edit caught; a Python oracle written from the rulings
  agreed with all 36 match rows; the 2,357,504-case count re-derived independently; ImageMagick
  `-trim` at AE 0 on 8-bit RGB, 16-bit RGBA and grey; a float TIFF drove the `f32(1/255)` edge on the
  binary itself.
- **Design drove all 21 mutation rows on a prototype and marked none "expected all green".** Build
  found two loose cells; verify confirmed both are wording:
  - **M2:** the table reasoned AC-13's binary no-op row red. Its input has nothing to trim, so `<`
    and `<=` both give the no-op; no assertion could turn it red.
  - **M3:** the build drove a **different mutant** (`ceil`) from the spec's `f64 / 255` maths, so the
    spec's row went unconfirmed until verify drove it as written. As written, the 210-case matrix
    stays green and the match-rule table is the float guard (the spec's claim, now confirmed). The
    cell's `alpha_zero_…` red depends on how the mutant writes the alpha test (with `alpha == 0.0`
    it stays green), and its row names `f32 over`/`f32 under` are the code's `f32 over range`/
    `f32 under range`.
- **The build recovered AC-3's case breakdown by search** because the spec named the groups but not
  their sizes. Verify's derivation: 14 × 65,792 + 14 × 65,280 + 7 × 65,280 + 65,536 = 2,357,504,
  with each group carrying its own range test (not one "per in-range (c, t, sign)" rule).
- **The frame's Python model was right, but not all of the frame was.** Design corrected eight
  things, including the inverse's colour-type condition, the corpus dims (right by accident), clamping
  out-of-range floats, and a whole-match message that printed a hex the image may not hold.
- **Design found and folded a crash:** `parse_color` panicked (exit 101) on a non-ASCII colour in
  `pad`, `watermark` and `watermark --color`. Verify confirmed the panic on `main` and the fix on the PR.
- **The overlap with SPEC-150 was smaller than the design prompt guessed,** and design's own map
  missed one file: PR #257 also edited `src/cli/ops.rs`, which reads the param tables. The build
  prompt added a `UInt {` grep after each rebase; SPEC-150 merged mid-build and the grep found
  nothing to fix.
- **Process:** the build wrote one hand-rolled CI wait loop despite the prompt forbidding it, and
  stopped it at once. Its mutation driver died as a detached process before M12; the build finished
  M12 by hand and verify re-drove it (24 red, matching).
- **Costs so far:** frame $4.38, design $14.98, build $9.58 (measured before its last turn),
  verify $6.23 (not recorded; read-only).

## Do, in order
1. **Mark ship `[~]`** in the timeline, and read its `merge` line: the orchestrator marked it `[x]`
   with the SHA. If the SHA or date differs from what `gh` reports, fix it and say so.
2. **Fold verify's wording items into the spec,** each marked `(corrected at ship)` in place. These
   are the only edits to the spec body besides the reflection and cost:
   - the **M2** cell: drop AC-13's binary row from "reasoned" and say why it can't go red;
   - the **M3** cell: name the alpha form the red/green result depends on, and use the code's row
     names (`f32 over range`, `f32 under range`);
   - the match-rule table's header "`t = 8` unless given": state `t` on each row that doesn't, using
     what `match_rule_table` in `src/operation/trim.rs` actually uses (read it; verify says `t = 0`);
   - "the scan allocates nothing" → "allocates nothing per pixel" (one boxed closure per apply), in
     every place the spec says it (grep);
   - Build Completion's "per in-range (c, t, sign)" breakdown → each group with its own range test,
     with verify's derivation.
3. **Mark the folded entry `[x]`:** "Pin `resize`'s cap message text" shipped inside SPEC-152 (AC-54).
   Mark it the way STAGE-058 marked SPEC-151's folded entries (find them by content), naming SPEC-152
   and PR #258.
4. **File verify's out-of-spec finding** as **one** STAGE-058 backlog entry, `[S]`, kind **claim**:
   `parse_color` accepts a sign, because `u8::from_str_radix` allows `+`: `color = "+f+f+f"` is read
   as `#0f0f0f` (exit 0) on `main` and on the PR, in `pad`, `watermark` and `trim`, against a message
   that says it must be a hex colour. Proposed fix: refuse any byte that isn't a hex digit (which
   also covers the non-ASCII case the fold-in fixed). First drive it yourself on a release build of
   `main` in a scratch `CARGO_TARGET_DIR` (delete it after), and quote the command and output in the
   entry. Before filing, search STAGE-058 and `guidance/questions.yaml` for an entry that already
   covers it; if one does, extend it instead and say so.
5. **Don't file** verify's wasm cleanup (`transform_trims_exactly` compares `to_rgba8`, so it doesn't
   pin the colour type): the native tests pin it. Mention it in the reflection only if it bears on an
   answer.
6. **Write `## Reflection (Ship)`**, outcome-focused, three answers drawn only from the evidence:
   1. **What would I do differently?** Weigh at least: design driving every mutation row (two loose
      cells out of 21, both wording, and a first clean verify; did the $14.98 design buy that?); a
      build that substitutes its own mutant for the spec's leaves the spec's row unconfirmed; a spec
      that states a count without its breakdown; an overlap map written from the other spec's text
      rather than its PR's file list.
   2. **Does any template, constraint or decision need updating?** For example: a mutation row should
      quote the exact mutant edit so build drives the same one; a count in a spec should carry its
      derivation; cross-project overlap maps should come from `gh pr view --json files`. Check AGENTS
      §15–16, HANDOFF, `guidance/` and SPEC-151's reflection first, and say which already covers
      each. **Propose it; don't apply it.**
   3. **A follow-up spec?** Beyond step 4, file **one** more small entry only if the evidence names a
      gap no entry covers, or say why none.
7. **Cost.** Every session's entry must be in `cost.sessions`: frame, design, build, verify, and your
   own `ship`.
   - **verify is not recorded** (it was read-only). It reported input 112, cache_creation 172,515,
     cache_read 8,008,049, output 45,988, total 8,226,664, **$6.23**. Measure it from its transcript
     and record that.
   - **Re-measure build**: its note says it was measured before the final bookkeeping turn. Read the
     frame and design notes too; both say they exclude their own final cost commit's turn, so
     re-measure them as well.
   - Markers are each prompt's title on `main` (`SPEC-152 — FRAME prompt`, `— DESIGN prompt`,
     `— BUILD prompt`, `— VERIFY prompt`); read each prompt's first line to confirm. Use the
     transcript whose **first user message** holds the title; the orchestrator's transcript also
     contains every title.
   - Dedup by `.message.id`: the three input counts once per id, the **MAX** `output_tokens`. Include
     any subagent transcripts under the session's `subagents/` folder. Price at the model in
     `.message.model`: Sonnet $3/$15, Opus $5/$25, cache_creation ×1.25, cache_read ×0.10.
   - Recompute `cost.totals` from every entry. Run `just cost-audit` and quote the line that names
     SPEC-152. (It fails today on PROJ-011's SPEC-149; that one isn't yours, so say so if it still
     does.)
8. **Mark ship `[x]`** with the merge SHA, PR #258, the date and the total cost.
9. **Advance and archive.** `just advance-cycle SPEC-152 ship`, then `just archive-spec SPEC-152`.
   - Read STAGE-058's SPEC-152 entry back. It should read `- [x] **SPEC-152** (merged <date>, PR
     #258) — …`. Fix it by hand if not.
   - Touch no other entry except steps 3–4's. Recount the `**Count:**` line with greps you just ran
     (today `7 shipped / 1 active / 8 pending`), and show the greps.
10. **Check what the tooling sees.** `just status`, `just backlog`, `just roadmap`. They show only
    the active project, PROJ-011, so say so rather than treat a missing SPEC-152 as an error.
    ⛔ Do not run Stage Ship or change STAGE-058's status; the orchestrator decides that.
11. **Commit to `main` directly** (AGENTS §13):
    `git commit -s -m "ship(SPEC-152): trim as a recipe step — reflected, archived"`,
    trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is mandatory.
    Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`; on a reject,
    rebase and push again. Read back `git show --stat HEAD`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, `docs/`, DECs, HANDOFF or templates; propose in your report. The spec's
  body changes only by step 2's corrections, the reflection and the cost block.
- Another session may push to `main` while you work. Rebase; never force.
- A piped command reports the pipe's exit code. zsh does not word-split unquoted parameters.
  `rtk` can truncate `grep`, so count with `/usr/bin/grep` or Python. Write scratch files only in
  your session's scratchpad, never `/tmp`.
- **Budget ~40 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-152 ship: archived — <commit SHA on main>`. Then:
- the merge SHA and date;
- step 2's five corrections, one line each;
- the folded entry as it now reads;
- the backlog entry filed in step 4, quoted, with your own driven output;
- the three reflection answers, one line each;
- any further follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- the recounted `**Count:**` line;
- any template, constraint or DEC change you propose;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on
  its own line.
