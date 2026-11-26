# SPEC-163 — SHIP prompt: an aspect that follows each image's orientation

Cycle: **ship**. This is bookkeeping, plus three edits on `main`: DEC-112's rule 8 (design wrote it
and verify checked it against the code), one docs sentence, and the AC ticks. Fresh CLI session, own
worktree. Sonnet is enough:

```
just wt docs/spec-163-ship -- --model sonnet
```

The maintainer pastes this prompt into a fresh session and will paste your final message back to the
orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

## Step 0 — check the merge, then get onto current `main`
1. Run `git fetch origin && git checkout --detach origin/main`.
2. ⛔ **Go on only if both of these hold.** Otherwise, stop and report.
   - **PR #272 is merged.** Run `gh pr view 272 --json state,mergeCommit,mergedAt,headRefOid`. Verify
     approved head `fccb368`. Record the merge SHA, the date and the merged head. If the head differs
     from `fccb368`, confirm that
     `git log --first-parent --no-merges --oneline fccb368..<head>` lists nothing. Stop if it does.
   - **The timeline**
     (`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-163-an-aspect-that-follows-each-image-s-orientation-timeline.md`)
     has build and verify both `[x]`.
3. Confirm the merge brought the work onto `main`: `tests/aspect_orientation.rs` exists, and the
   spec's `cost.sessions` has a `build` entry. If not, stop and report.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- **The spec in full.** In particular:
  - Frame, with the maintainer rulings;
  - Design's corrections, "DEC-112's rule 8 (for ship to apply after the merge)" and "The page port";
  - the ACs and the mutation table;
  - `## Build Completion` and the cost block.
- **The timeline, all of it.** Its verify entry summarises the verify report; the details you need are
  below.
- **DEC-112** (`decisions/DEC-112-a-recipe-carries-geometry-lengths-as-fractions-of-the-image-and-rounds-each-edge.md`).
- **STAGE-060's `## Spec Backlog`**, all of it, and its `**Count:**` line
  (`projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md`).
- **SPEC-160's `## Reflection (Ship)`** (in `specs/done/`), for its proposals and what came of them.

## The facts to reflect on (frame to verify)
- **Frame (Opus, $4.06).** It drove the premise: `aspect = "3:2"` crops a portrait 800×1200 to
  800×533. It proposed the rule, and the maintainer took all four recommendations (`"match"`; a square
  as written; both `edit` flags at [M]; DEC-112 rule 8).
- **Design (Opus, $12.37).** It drove the whole design on a prototype, with 17 mutants. It made three
  corrections:
  - six existing key-list tests had to move;
  - `tests/orientation.rs` already existed;
  - the frame missed `docs/cli-reference.md`.

  It reasoned the 26 parity rows but didn't run them (no wasm build). Its push to `main` was refused by
  a permission check, and the maintainer pushed it by hand, on the second try.
- **Build (Sonnet, $5.50, measured before its last commit).**
  - The RED baseline matched.
  - M0 to M17 were all driven, and every red set matched the table.
  - The parity rows ran for the first time and were all right; N3 was exactly the eight.
  - Only M5 was driven against parity, and native-only at that.
  - 18 of 18 ACs met, CI green.
  - Its Build Completion wrongly says the build prompt wasn't on `main`.
  - It didn't record the space it freed.
- **Verify (Opus, $6.83): APPROVED at `fccb368`, no punch items.**
  - An independent model over 9324 rows, 0 mismatches; the 4662 key-off rows equal `main`'s binary.
  - Every message compared with `==`; AC-5 on the Orientation-6 fixture; 12 save/replay rows at AE 0.
  - The fixtures are byte-identical under ImageMagick 7.1.2-34.
  - M1 built into wasm: in both sides, and in wasm only, exactly `O-crop-S-on-9:16` and
    `O-pad-S-on-9:16` go red. When both sides carry the mutant, only the literal sizes catch it.
  - M1, M5, M9, M14, M15 and M16 were driven, each as tabled.
  - DEC-112 rule 8: "apply as written".
  - Five optional cleanups. The orchestrator gave two to ship (steps 3 and 4) and filed the other
    three as `aspect-orientation-loose-ends` in `guidance/ideas.yaml`.
- **This is the first PROJ-017 spec with no punch list since SPEC-152.** Say what you think made the
  difference, from the evidence only.

## Do, in order
1. **Mark ship `[~]`** in the timeline, and add a `merge` line above it:
   `- [x] **merge** (<date>, maintainer) — PR #272 as <merge SHA>, head <head>.` Check it against
   `gh`.
2. **Apply DEC-112's rule 8** from the spec's "DEC-112's rule 8 (for ship to apply after the merge)",
   exactly as written. That section has four parts:
   - the rule-8 text, added after rule 7;
   - one sentence appended to the Consequences bullet "Orientation is not decided here";
   - one line appended to `## Context`;
   - five paths added to `affected_scope`.

   `supersedes` stays null.
   - Find each target by content. If one no longer reads as the spec quotes it, stop that edit and
     report the text as it stands.
   - Check `affected_scope` against
     `git diff --name-only <merge SHA>^1 <merge SHA> -- src/ docs/ recipes/ skills/`, and say if
     anything is missing.
   - Run `just decisions-audit` and quote its line.
3. **One docs sentence (verify's cleanup 1, wording).** In `docs/data-model.md`, the `crop` section
   reads, about line 333: "mixing them is refused at load, and so is `gravity` on a rectangle, and so
   is `orientation`." Read alone, the last clause sounds as if the key is always refused. Change the
   end to: "and so is `gravity` or `orientation` on a rectangle." Find it by content. If the `pad`
   section has the same construction, make the matching change ("on a border") and say so. Touch
   nothing else in `docs/`.
4. **Fold verify's results into the spec**, and mark each change `(corrected at ship)` in place:
   - **Tick AC-1 to AC-18.** Verify confirmed each, and Build Completion gives evidence.
   - **M1's parity cell:** "reasoned" becomes "driven by verify". Record that native and wasm both
     mutated, and wasm only, each give exactly `O-crop-S-on-9:16` and `O-pad-S-on-9:16`. In
     both-sided mode, only the literal sizes catch it.
   - **M1's FT-13 note:** driven by verify. On a 48×48 input, `--crop 3:2` gives 48×32 both mutated
     and clean, and a square row written `2:3` would catch it.
   - **M5's parity cell:** driven by build (native only).
   - **M13, M14 and M15's parity cells** stay reasoned. Say so in one line under the table.
   - **Build Completion's deviation** "the build prompt is not on `main`": add one line saying it was
     on `main` from `eaf76c3`. Keep the original text as history.

   Nothing else in the spec's body changes, apart from the reflection and the cost block.
5. **Write `## Reflection (Ship)`.** Outcome-focused: three answers, drawn only from the evidence.
   1. **What would I do differently?** Weigh at least these:
      - The design's parity cells were reasoned, and only verify drove a wasm-side mutant. It showed
        that a mutant in both sides passes the native-against-wasm comparison and is caught only by
        the literal sizes. Should design or build drive one wasm-side mutant whenever a spec adds
        parity rows?
      - Design's and build's pushes stopped at a permission check (design's) or claimed something
        false (build's "not on `main`"). What should a prompt tell a cycle to do when it can't push?
      - Build measured its cost before its last commit again, and didn't record the space it freed.
   2. **Does any template, constraint or decision need updating?** Candidates:
      - SPEC-160's proposals (read its reflection), now with this spec's evidence;
      - "parity literals are the check that catches a mutant both sides share; a spec that adds
        parity rows drives one mutant into both sides";
      - "a test row whose aspect is already in the order a mutant produces can't catch that mutant;
        write each square row in both orders".

      Check AGENTS §15–16, HANDOFF, `guidance/` and the reflections of SPEC-152, SPEC-153, SPEC-159
      and SPEC-160 first, and say which already covers each. **Propose; don't apply.**
   3. **A follow-up spec?** `aspect-orientation-loose-ends` is already filed. File nothing else
      unless the evidence names a gap it doesn't cover.
6. **Cost.** Every session needs an entry in `cost.sessions`: frame, design, build, verify, and your
   own `ship`.
   - **Re-measure build** (taken before its last commit and CI watch), and **design** (taken before
     its blocked push).
   - **Measure verify**: it reported 9,269,764 tokens, $6.83 (Opus).
   - **Re-measure frame** only if its note says it was taken early.
   - **The rulings and the merge are the maintainer's.** No entry.
   - **Find each transcript by its marker**, the prompt's title on `main`: `SPEC-163 — FRAME prompt`,
     `SPEC-163 — DESIGN prompt`, `SPEC-163 — BUILD prompt` and `SPEC-163 — VERIFY prompt`.
     - Read each prompt's first line to confirm its title.
     - Use the transcripts whose **first user message** holds the title, because the orchestrator's
       transcript contains every title.
     - If a cycle has two such transcripts, measure both and record one entry with their sum; its note
       names both.
   - **Dedup by `.message.id`.** Count the three input counts once per id, and take the **MAX**
     `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
   - **Price** at the model in `.message.model`: Sonnet $3/$15, Opus $5/$25. cache_creation is ×1.25
     the input rate and cache_read ×0.10.
   - **Recompute `cost.totals`** from every entry, your own included. Measure your own entry **after**
     your push. Run `just cost-audit` and quote the line that names SPEC-163.
7. **Mark ship `[x]`** with the merge SHA, PR #272, the date and the total cost.
8. **Advance and archive.** Run `just advance-cycle SPEC-163 ship`, then `just archive-spec SPEC-163`.
   - Read STAGE-060's SPEC-163 entry back. It should read
     `- [x] **SPEC-163** (merged <date>, PR #272) — [M] …`, in the style of the other shipped entries.
     Fix it by hand if not. Touch no other entry.
   - Recount the `**Count:**` line with greps you just ran, and show them, as the line itself
     describes. Today it reads `2 resolved (1 shipped spec, 1 folded) / 2 active / 4 pending`. Expect
     `3 resolved (2 shipped specs, 1 folded) / 1 active / 4 pending`, unless another session changed
     the stage meanwhile.
   - ⛔ Don't change STAGE-060's `status` or its reflection.
9. **Commit to `main` directly** (AGENTS §13).
   - Commit with
     `git commit -s -m "ship(SPEC-163): an aspect that follows each image's orientation — DEC-112 rule 8, reflected, archived"`
     and the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is
     mandatory.
   - Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`. On a reject,
     rebase and push again. A "Changes must be made through a pull request" notice can still mean the
     push landed, so check with `git ls-remote origin refs/heads/main`.
   - ⛔ **If a permission check refuses the push, stop and report.** Name the commits and the command
     for the maintainer to run, and don't claim the push happened.
   - Read back `git show --stat HEAD`.
   - Then measure your own cost. If it moved, amend nothing: push one more commit that updates only
     your `ship` entry and `cost.totals`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, HANDOFF, `guidance/` or templates.
- The only DEC you edit is DEC-112, and only with step 2's text.
- In `docs/`, change only step 3's sentence. Propose anything else in your report.
- Another session may push to `main` while you work. Rebase; never force.
- **Never poll CI,** and never write your own wait loop.
- A piped command reports the pipe's exit code.
- zsh does not word-split unquoted parameters, and it reads `$R:r…` as a modifier, so brace it
  (`${R}:`).
- `rtk` can truncate `grep`, so count with `/usr/bin/grep` or Python.
- Write scratch files only in your session's scratchpad, never `/tmp`.
- **Budget ~40 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-163 ship: archived — <commit SHA on main>`. Then:
- the merge SHA, date and merged head;
- DEC-112's four changes, each quoted as it now reads, and the `just decisions-audit` line;
- the `docs/data-model.md` sentence(s) as they now read;
- step 4's changes, one line each;
- STAGE-060's SPEC-163 line as it now reads, and the recounted `**Count:**` line with its greps;
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- any template, constraint or DEC change you propose;
- your measured cost after your last push: input, cache_creation, cache_read, output, total tokens and
  dollars, each on its own line.
