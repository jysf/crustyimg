# SPEC-159 — SHIP prompt: `pad` with a transparent fill, and the no-alpha output rule

Cycle: **ship**. Bookkeeping, plus two settled wording fixes to DEC-115. Fresh CLI session, own
worktree; Sonnet is enough:

```
just wt docs/spec-159-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. ⛔ **Proceed only if both hold.** Otherwise stop and report.
   - PR #262 is merged: `gh pr view 262 --json state,mergeCommit,mergedAt`. It was merged as
     `a090a75` (`mergedAt` 2026-11-12T05:24:24Z) when this was written; record what you see.
   - The timeline
     (`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-159-pad-with-a-transparent-fill-and-the-no-alpha-output-rule-timeline.md`)
     has build, verify, punch list and merge all `[x]`.
3. Confirm the merge brought the work onto `main`: `src/operation/alpha.rs` exists, and the spec's
   `cost.sessions` has `build` and `punchlist` entries. If not, stop and report.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full: Frame with the maintainer rulings, Design (corrections 1–8, the mutation
  table), `## Build Completion` with its punch-list subsection, and the cost block.
- The timeline, all of it. Its verify entry summarises the verify report; the details you need from
  that report are below.
- DEC-115 (`decisions/DEC-115-a-step-whose-alpha-the-output-format-cannot-store-is-refused-before-the-pipeline-runs.md`).
- STAGE-058's `## Spec Backlog`, all of it
  (`projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`).
- SPEC-153's `## Reflection (Ship)` (in `specs/done/`). It proposed that build drive every doc
  sentence that states behaviour, and that a mutation row's red set be read off a run, not
  reasoned. Say whether SPEC-159 ran under each, and what happened.

## The facts to reflect on (from frame to punch list)
- **The frame** found the premise held (`transparent` refused at load) and drove the format table:
  JPEG drops alpha silently, GIF keeps only alpha 0. It put four questions; the maintainer took
  every recommendation (GIF allows `00`/`ff`, `trim` shares the helper, source alpha stays out,
  [M] uncut).
- **Design ($12.84) corrected the frame eight ways.** The largest: the auto-decided format *could*
  be JPEG (black border, exit 0), fixed by one line OR-ing `introduces_alpha`. It drove 17 of 18
  mutation rows on a prototype.
- **Build ($6.92)** drove all 17 mutations plus M18 by search, confirmed the three reasoned cells
  (M3's FT-11, M9's single input, M11 via the parity harness), and drove every doc sentence on a
  release build. Two files outside the spec's list (`tests/wasm_roundtrip.rs`,
  `scripts/wasm-ci-check.sh`) were caught by CI's wasm leg, which `just check` does not run
  (HANDOFF trap 5). It said "no spec cell was wrong" while its own account listed FT-2 as an extra
  red in M1–M3.
- **Verify ($7.47)** drove everything independently: messages with extra rows (upper-case alpha,
  no `#`, a three-step recipe), 12 widening cells and the four byte-pin hashes with its own PNG
  codec and FNV-1a, every format rung, GIF 00/01/80/fe/ff, batch and `build` exits 4 and 6,
  auto-decide through `apply` and a fresh `build`, six mutations. Default 1334 and lean 1270 passed.
- **The only defects were wording,** two kinds:
  - three docs (`docs/recipes.md`, `recipes/README.md`, `CHANGELOG.md`) said "exit 4" and skipped
    the per-input exit 6 when each input keeps its format. **Build had driven those sentences**, but
    only the exit-4 case; `docs/api-contract.md` already had both. The punch list ($0.56) fixed
    them and tightened the AVIF "approximately" sentence;
  - DEC-115, written at design, overstates `run_pixel_op` and misses two files in
    `affected_scope`. Build saw the scope gap; verify found the `run_pixel_op` sentences.
- **Verify's optional cleanups A and B** (wasm asserts: `transform` with an empty `out_format` and
  `recipe.format = "jpeg"`; the PNG half of the new wasm test asserts only non-empty output) were
  declined by the orchestrator: A would move the wasm test counts again, and parity row A covers
  B's bytes.

## Do, in order
1. **Mark ship `[~]`** in the timeline. Check the `merge` line against `gh`; fix it and say so if it
   differs.
2. **Fix DEC-115** (settled, do not re-decide; wording only):
   - `affected_scope`: add `src/cli/mod.rs` (the `CliError::UnstorableAlpha` arm) and
     `src/operation/trim.rs` (`parse_fill_color`). Check the list against
     `git diff --name-only a090a75^1 a090a75 -- src/` and say if anything else is missing.
   - Part 4 (about :73), "One input (`apply`, the pixel verbs through `run_pixel_op`): exit **4**":
     `run_pixel_op` checks only a **forced** format, once, before any input is read
     (`src/cli/ops.rs`, find it by content). Say that, and that the per-input check for an
     unforced format arrives with `edit --pad`, which is the first caller that can reach it.
   - The Neutral consequence (about :142), "single-input `apply` and `run_pixel_op` now resolve the
     output format before the pipeline runs": only single-input `apply` moved; `run_pixel_op`
     still runs the pipeline first and resolves the format after. Read the code to confirm both
     before you write.
   - Read :156 too and fix it if it says the same thing.
   - No new decision, no change to any part's rule. Run `just decisions-audit` and quote its line.
3. **Fold the table fixes into the spec,** each marked `(corrected at ship)` in place in the
   design's mutation table (find each row by its `**Mn —` label). Only these, the reflection and
   the cost block change in the spec body:
   - **M1, M2, M3:** add FT-2 to the red set: `Display` reads the capacity, so the GIF/JPEG line
     changes (verify drove M1: the GIF line became "GIF output has no alpha channel…").
   - **M9:** the single-input `-o p.jpg` row is driven red (build), no longer reasoned.
   - **M11:** driven red by build (the parity harness exits 1 on `alpha-B`), no longer reasoned.
   - **M13:** `pad_then_trim_is_the_identity` is in `tests/trim.rs`, not `tests/pad.rs`.
   - **M16:** add `introduces_alpha_is_any_translucent_step` to the red set; FT-5 and FT-7 are driven
     red.
4. **Write `## Reflection (Ship)`**, outcome-focused, three answers drawn only from the evidence:
   1. **What would I do differently?** Weigh at least: build drove the exit-4 doc sentences and
      still missed the exit-6 case, so "drive each sentence" is necessary but checks only the case
      the driver thinks of; a DEC written at design described a call site the design itself then
      changed (design correction 2 moved `run_pixel_op` to forced-format-only); a build report that
      said "no cell wrong" next to its own extra reds; two files only CI's wasm leg caught.
   2. **Does any template, constraint or decision need updating?** Candidates: build's checklist
      should say "drive each doc sentence on every exit path the spec names, not one"; design
      should re-read its DEC against its own corrections before it hands off; "files outside the
      list" should include whatever `just wasm-test` touches when `src/wasm.rs` changes. Check
      AGENTS §15–16, HANDOFF, `guidance/` and SPEC-152's and SPEC-153's reflections first, and say
      which already covers each. **Propose it; don't apply it.**
   3. **A follow-up spec?** Verify's declined cleanups A and B, and the ideas the frame filed
      (`alpha-dropped-silently-on-encode`, `tiff-alpha-extrasamples` in `guidance/ideas.yaml`).
      File **one** small entry only if the evidence names a gap no entry covers, or say why none.
5. **Cost.** Every session's entry must be in `cost.sessions`: frame, design, build, verify,
   punchlist, and your own `ship`.
   - **verify is not recorded.** It reported input 128, cache_creation 194,394, cache_read
     10,413,890, output 42,067, total 10,650,479, **$7.47**, measured before its last message.
     Measure it from its transcript and record that.
   - **Re-measure frame, design, build and punchlist:** each was measured before its own last
     turn(s).
   - Markers are each prompt's title on `main`: `SPEC-159 — FRAME prompt`, `— DESIGN prompt`,
     `— BUILD prompt`, `— VERIFY prompt`, `— PUNCH LIST prompt`. Read each prompt's first line to
     confirm. Use the transcripts whose **first user message** holds the title; the
     orchestrator's transcript also contains every title. If a cycle has two such transcripts,
     measure both and record one entry with their sum, the note naming both.
   - Dedup by `.message.id`: the three input counts once per id, the **MAX** `output_tokens`. Include
     any subagent transcripts under the session's `subagents/` folder. Price at the model in
     `.message.model`: Sonnet $3/$15, Opus $5/$25, cache_creation ×1.25, cache_read ×0.10.
   - Recompute `cost.totals` from every entry. Run `just cost-audit` and quote the line that names
     SPEC-159. If it fails on another spec, say which and that it isn't yours.
6. **Mark ship `[x]`** with the merge SHA, PR #262, the date and the total cost.
7. **Advance and archive.** `just advance-cycle SPEC-159 ship`, then `just archive-spec SPEC-159`.
   - Read STAGE-058's SPEC-159 entry back. It should read `- [x] **SPEC-159** (merged <date>, PR
     #262) — [M] …`. Fix it by hand if not.
   - Touch no other entry. Recount the `**Count:**` line with greps you just ran (today
     `10 shipped / 1 active / 7 pending`; expect 11 / 0 / 7), and show the greps.
8. **Check what the tooling sees.** `just status`, `just backlog`, `just roadmap`. They show only the
   active project, so say so rather than treat a missing SPEC-159 as an error.
   ⛔ Do not run Stage Ship or change STAGE-058's status; SPEC-160 (`edit --pad`/`--trim`) is still
   to come and closes the stage.
9. **Commit to `main` directly** (AGENTS §13):
   `git commit -s -m "ship(SPEC-159): transparent pad and the no-alpha output rule — reflected, archived"`,
   trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is mandatory.
   Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`; on a reject,
   rebase and push again. Read back `git show --stat HEAD`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, `docs/`, HANDOFF, templates or any DEC but DEC-115 (step 2's wording
  only); propose in your report. The spec's body changes only by step 3's corrections, the
  reflection and the cost block.
- Another session may push to `main` while you work. Rebase; never force.
- A piped command reports the pipe's exit code. zsh does not word-split unquoted parameters, and
  reads `$R:r…` as a modifier, so brace it (`${R}:`). `rtk` can truncate `grep`, so count with
  `/usr/bin/grep` or Python. Write scratch files only in your session's scratchpad, never `/tmp`.
- **Budget ~45 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-159 ship: archived — <commit SHA on main>`. Then:
- the merge SHA and date;
- DEC-115's changes, each quoted as it now reads, and the `just decisions-audit` line;
- step 3's corrections, one line each;
- STAGE-058's SPEC-159 line as it now reads;
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- the recounted `**Count:**` line;
- any template, constraint or DEC change you propose;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on
  its own line.
