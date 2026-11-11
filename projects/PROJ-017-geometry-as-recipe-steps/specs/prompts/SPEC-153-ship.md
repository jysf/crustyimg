# SPEC-153 — SHIP prompt: `edit --crop`, `--rotate` and `--flip`, run in the order typed

Cycle: **ship**. Bookkeeping only. Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-153-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. ⛔ **Proceed only if both hold.** Otherwise stop and report.
   - PR #261 is merged: `gh pr view 261 --json state,mergeCommit,mergedAt`. It was merged as
     `70e2da7` (`mergedAt` 2026-11-11T02:10:43Z) when this was written; record what you see.
   - The timeline
     (`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-153-edit-flags-for-the-five-geometry-ops-and-the-ordering-rule-timeline.md`)
     has build, verify, punch list and merge all `[x]`.
3. Confirm the merge brought the work onto `main`: `tests/edit_geometry.rs` exists,
   `decisions/DEC-114-*.md` exists, and the spec's `cost.sessions` has `build` and `punchlist`
   entries. If not, stop and report.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full: Frame, the maintainer rulings, Design (corrections 1–6, the mutation table),
  `## Build Completion` with its punch-list subsection, and the cost block.
- The timeline, all of it. Its verify entry summarises the verify report; the details you need from
  that report are below.
- DEC-114 (`decisions/DEC-114-edit-runs-its-op-flags-in-the-order-typed-and-saves-what-ran.md`).
- STAGE-058's `## Spec Backlog`, all of it
  (`projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`).
- SPEC-152's `## Reflection (Ship)` (in `specs/done/`). It proposed that a mutation row quote the
  exact mutant edit, that a count carry its derivation, and that overlap maps come from
  `gh pr view --json files`. Say whether SPEC-153 ran under each.

## The facts to reflect on (from frame to punch list)
- **The frame split the spec** and put the ordering rule to the maintainer. The maintainer first
  objected that order shouldn't matter, or the result stops being deterministic; both rules are
  deterministic, and the maintainer chose typed order (DEC-114). Today's `--invert --resize-max 30`
  output changes (byte pin 0x62d0525855cff6a7 → 0x5d248c136b8b3ad7).
- **Design ($13.88) drove 15 mutation rows plus a trap-14 control on a prototype.** Two cells were
  still wrong, both found by build and confirmed by verify:
  - **M1:** FT-10 is red, not green (the build also saw FT-9 red).
  - **M6:** FT-14 reds on both the Orientation 6 and Orientation 5 rows (the saved step list), not
    "the Orientation 5 row" only.
  - **M2** red more than the table: FT-8, 10, 11, 12, 30 and `tests/cli.rs`'s
    `stub_command_returns_not_implemented`, which runs `edit in.png` with no op flag and so is an
    `edit` test. The punch list recorded this in Build Completion.
- **Verify drove everything independently:** every message P1–P16 and A1–A3 byte for byte by
  script, pixel crops and both orders against ImageMagick at AE 0, the byte pin with its own Python
  decoder and FNV-1a, replay on four colour types and two tagged inputs, the batch rules, six
  mutations.
- **The only defects were two user-facing doc sentences the build wrote:** `docs/recipes.md` said a
  saved pixel crop "replays on any size" (it crops the same proportions and fails if a side rounds to
  0 px; verify drove exit 1), and both recipe docs put `--auto-orient` in typed order (DEC-114 part 2
  puts it first, always). Neither sentence was driven by build. The punch list fixed both.
- **The punch list ran in two sessions.** The second found the work already pushed and re-reported
  the first's drives and cost without re-running them.
- **Out of spec, pre-existing:** `edit`'s help and docs call it "one-shot multi-op on a single
  image", but `edit` takes a batch (design correction 1). Sites today:
  `src/cli/mod.rs:453` (the `--help` doc comment), `docs/api-contract.md:536`, and the `edit`
  section of `docs/cli-reference.md` (grep it; the phrase may wrap).
- **Costs so far:** frame $4.77, design $13.88, build $5.29, verify $5.64 (not recorded; read-only),
  punch list $0.53.

## Do, in order
1. **Mark ship `[~]`** in the timeline. Check the `merge` line against `gh`; fix it and say so if it
   differs.
2. **Fold the table fixes into the spec,** each marked `(corrected at ship)` in place, in the
   design's mutation table (find M1 and M6 by content, about :1143 and :1187). Only these, the
   reflection and the cost block change in the spec body:
   - **M1:** FT-10 red, and FT-9 red, with the build's reason from its Build Completion M1 line;
   - **M6:** FT-14 red on both the Orientation 6 and Orientation 5 rows;
   - **M2:** add the extra reds with the one-line `stub_command_returns_not_implemented` reason, or
     point to the Build Completion line that already has it.
3. **Tidy STAGE-058's SPEC-153 line.** It reads `(designed 2026-11-10, [M]) — [S] **…`; the old
   `[S]` is stale. Step 9 rewrites the status part; make sure the size reads `[M]` once.
4. **Fold the stale "single image" wording into the `--pad`/`--trim` entry** (the `(not yet written)`
   entry that "closes STAGE-058"), as one added sentence: that spec rewords `edit`'s `--help`,
   `docs/api-contract.md` and `docs/cli-reference.md` to say `edit` takes one image or a batch. Quote
   the current text of each site in the entry. First search STAGE-058 and `guidance/questions.yaml`
   for an entry that already covers it; if one does, extend that instead and say so. Don't edit the
   sites themselves.
5. **Write `## Reflection (Ship)`**, outcome-focused, three answers drawn only from the evidence:
   1. **What would I do differently?** Weigh at least: a $13.88 design that drove every mutation row
      still left two wrong cells (both wording, both in rows whose reds depend on which tests type
      which flags); the only real defects were doc sentences nobody drove, so should build drive
      every behavioural claim it writes into `docs/`, the way it drives messages?; a punch list that
      ran twice and reported without re-running.
   2. **Does any template, constraint or decision need updating?** For example: build's checklist
      should include "drive each new doc sentence that states behaviour"; a mutation row's red set
      should be read off the prototype's run, not reasoned. Check AGENTS §15–16, HANDOFF, `guidance/`
      and SPEC-151's and SPEC-152's reflections first, and say which already covers each.
      **Propose it; don't apply it.**
   3. **A follow-up spec?** Beyond step 4, file **one** small entry only if the evidence names a gap
      no entry covers, or say why none.
6. **Cost.** Every session's entry must be in `cost.sessions`: frame, design, build, verify,
   punchlist, and your own `ship`.
   - **verify is not recorded.** It reported input 88, cache_creation 188,682, cache_read 6,888,052,
     output 40,600, total 7,117,422, **$5.64**. Measure it from its transcript and record that.
   - **Re-measure frame, design, build and punchlist**: each note says it excludes its own last
     turn(s).
   - **The punch list has two transcripts** whose first user message holds its title. Measure both
     and record one `punchlist` entry with their sum, the note naming both.
   - Markers are each prompt's title on `main` (`SPEC-153 — FRAME prompt`, `— DESIGN prompt`,
     `— BUILD prompt`, `— VERIFY prompt`, `— PUNCH LIST prompt`); read each prompt's first line to
     confirm. Use the transcripts whose **first user message** holds the title; the orchestrator's
     transcript also contains every title.
   - Dedup by `.message.id`: the three input counts once per id, the **MAX** `output_tokens`. Include
     any subagent transcripts under the session's `subagents/` folder. Price at the model in
     `.message.model`: Sonnet $3/$15, Opus $5/$25, cache_creation ×1.25, cache_read ×0.10.
   - Recompute `cost.totals` from every entry. Run `just cost-audit` and quote the line that names
     SPEC-153. If it fails on another spec, say which and that it isn't yours.
7. **Mark ship `[x]`** with the merge SHA, PR #261, the date and the total cost.
8. **Advance and archive.** `just advance-cycle SPEC-153 ship`, then `just archive-spec SPEC-153`.
   - Read STAGE-058's SPEC-153 entry back. It should read `- [x] **SPEC-153** (merged <date>, PR
     #261) — [M] …`. Fix it by hand if not.
   - Touch no other entry except step 4's. Recount the `**Count:**` line with greps you just ran
     (today `9 shipped / 1 active / 8 pending`; expect 10 / 0 / 8), and show the greps.
9. **Check what the tooling sees.** `just status`, `just backlog`, `just roadmap`. They show only the
   active project, so say so rather than treat a missing SPEC-153 as an error.
   ⛔ Do not run Stage Ship or change STAGE-058's status; the `--pad`/`--trim` spec is still to come.
10. **Commit to `main` directly** (AGENTS §13):
    `git commit -s -m "ship(SPEC-153): edit crop, rotate and flip in typed order — reflected, archived"`,
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
Plain lists only. First line: `SPEC-153 ship: archived — <commit SHA on main>`. Then:
- the merge SHA and date;
- step 2's corrections, one line each;
- STAGE-058's SPEC-153 line as it now reads;
- the `--pad`/`--trim` entry's added sentence, quoted;
- the three reflection answers, one line each;
- any further follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- the recounted `**Count:**` line;
- any template, constraint or DEC change you propose;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on
  its own line.
