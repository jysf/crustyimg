# SPEC-156 — SHIP prompt: `meta strip` keeps the Adobe APP14 colour-transform marker

Cycle: **ship**. Bookkeeping, plus the two DEC edits the build was told to leave for after the
merge. Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-156-ship -- --model sonnet
```

The maintainer pastes this prompt into a fresh session and will paste your final message back to the
orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. ⛔ **Proceed only if both hold.** Otherwise stop and report.
   - PR #269 is merged: `gh pr view 269 --json state,mergeCommit,mergedAt`. Record what you see. The
     orchestrator read its head as `3aaa700`, with every CI leg green there.
   - The timeline
     (`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-156-meta-strip-keeps-the-adobe-app14-colour-transform-marker-timeline.md`)
     has build, verify and punch list `[x]`.
3. Confirm the merge brought the work onto `main`:
   - `src/metadata/mod.rs` holds `ADOBE_APP14_MIN_LEN` and `strip_all_removes_an_adobe_cm_app13`;
   - the spec's `cost.sessions` has `frame`, `design`, `build`, `verify` and `punchlist` entries;
   - the fixture `tests/fixtures/metadata/` file the spec names exists, with the sha256 the spec
     records.

   If not, stop and report.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full, especially:
  - Frame, with the maintainer rulings;
  - Design and "Design corrections to the frame";
  - the ACs (AC-3 was widened at the punch list) and the mutation table (row MK);
  - "DEC text for ship";
  - `## Build Completion` with its `### Punch list (verify)` subsection and build's reflection;
  - the cost block.
- The timeline, all of it.
- `decisions/DEC-003-metadata-dual-lane.md` and the DEC-029 file, in full, before you edit them.
- STAGE-063's `## Spec Backlog`
  (`projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`):
  the SPEC-156 entry, and the entry "`meta strip` drops EXIF orientation, and no doc says so", which
  the frame filed.

## The facts to reflect on (from frame to punch list)
- **The frame ($4.16)** drove the premise on a release build: a stripped `cmyk_16x16.jpg` scored
  SSIMULACRA2 41.94, and djpeg's pixel went from `srgb(191,64,32)` to `srgb(93,134,30)`. It found a
  3-component Adobe transform-0 file with ids 1,2,3 breaks too, while transform 1, RGB-letter ids and
  4-component transform 0 don't. It recommended keeping an APP14 with the `Adobe` signature only. It
  filed the orientation finding as its own STAGE-063 entry.
- **Design ($10.02)** corrected the frame: the byte after `Adobe` is the version's high byte, so
  "starts with `Adobe\0`" would still drop `Adobe\x01…` and leave it rendering orange. The rule became
  "starts with `Adobe` and is at least 12 bytes". It wrote a reproducible fixture recipe
  (`cjpeg -rgb`, ids 1,2,3, sha256 recorded), 9 tests and 12 mutation rows, and the DEC text for ship.
  It is the most expensive cycle of this spec.
- **Build (Sonnet, $2.53 as measured):**
  - fixture hashes equal the spec's; the baseline was red on exactly the predicted rows;
  - every mutation row matched, none wrong;
  - one deviation: `enumerate` in a test helper, for clippy.
  - Its reflection: the red baseline needs the helpers present, and the spec didn't say so; the
    overlap list named #268 only for `CHANGELOG.md`, but #268 also touches both docs files.
- **Verify ($2.76):**
  - it rebuilt the fixture byte for byte with its own `cjpeg`, and drove nine more JPEG shapes through
    djpeg and ImageMagick (two APP14s in both orders, APP14 after DQT, progressive, an out-of-range
    transform). Every Adobe shape rendered the same after the PR's strip;
  - one **assert**: nothing tested the `marker == 0xEE` clause. A Photoshop `Adobe_CM` APP13 passes
    the length and signature checks, so without the clause strip would keep it (mutant MK, 1,357
    green);
  - out of spec, pre-existing: crustyimg's decoder refuses an Adobe APP14 with transform 5 where
    libjpeg decodes it → `guidance/ideas.yaml` (`decode-unknown-adobe-transform`);
  - #269 conflicts with #240 in `src/metadata/mod.rs`; #240 also conflicts with `main` in `ci.yml`.
- **Punch list (Sonnet, $0.76 as measured):** FT-10 `strip_all_removes_an_adobe_cm_app13`, test code
  only. MK turned exactly FT-10 red (1,357/1); unmutated 1,358/0. AC-3 widened. Two pushes: the code
  (`a691427`), then the CI snapshot and cost (`3aaa700`). No re-verify, because the item was an assert.

## Do, in order
1. **Mark ship `[~]`** in the timeline, and add a `merge` line above it:
   `- [x] **merge** (<date>, maintainer) — PR #269 as <merge SHA>.` Check it against `gh`.
2. **Apply the DEC text** from the spec's "DEC text for ship", byte for byte, with `<ship date>`
   filled in:
   - DEC-003: append the amendment before `## References`, and add `tests/fixtures/metadata/**` to
     its `affected_scope` in the block-list form;
   - DEC-029: add the note after the first probe bullet. Find the bullet by its text
     (`remove_segments_by_marker(0xE1..=0xEF, 0xFE)`); the spec's line numbers may have moved.
   - Change nothing else in either DEC. Run `just validate` and quote its result.
3. **Drive the shipped behaviour once on `main`.** Build `main` `--release` in a fresh
   `CARGO_TARGET_DIR`. Run `meta strip` on the spec's new fixture and on `cmyk_16x16.jpg`, and quote
   djpeg's pixel (8,8) for input and output of each, plus the APP14 count before and after. This is
   the shipped binary's evidence, not the PR's.
4. **Write `## Reflection (Ship)`**, outcome-focused, three answers drawn only from the evidence:
   1. **What would I do differently?** Weigh at least these:
      - design cost 2.4× the frame and ~4× the build, for an [S] spec. What did that cost buy (the
        version-byte correction, the reproducible fixture, the mutation table that left verify one
        gap)? Was any of it avoidable?
      - every guard in `is_adobe_colour_transform` but one had a mutation row; the one without was
        the marker check. The table was built from the clauses design thought of as risky.
      - the build's overlap list was incomplete (#268 touches both docs files).
   2. **Does any template, constraint or decision need updating?** Candidates:
      - "a predicate with N clauses needs a mutation row per clause, each with an input that passes
        every other clause" (FT-10's shape);
      - "a red baseline that depends on new test helpers must say so";
      - "the overlap list comes from `gh pr list --json files`, not from memory".
      Check AGENTS §15–16, HANDOFF, `guidance/` and recent ship reflections (SPEC-154's included)
      first, and say which already covers each. **Propose it; don't apply it.**
   3. **A follow-up spec?** The orientation entry is already in STAGE-063 and the transform-5 decode
      is in `guidance/ideas.yaml`. File nothing new unless the evidence names a gap neither covers,
      and say why.
5. **Cost.** Every session's entry must be in `cost.sessions`: frame, design, build, verify,
   punchlist, and your own `ship`.
   - **Re-measure build and punch list.** Both were measured before their last turns.
   - Check that verify's entry (recorded by the punch list from verify's report) matches its
     transcript.
   - **Find each transcript by its marker.** The markers are each prompt's title on `main`:
     - `SPEC-156 — FRAME prompt`
     - `SPEC-156 — DESIGN prompt`
     - `SPEC-156 — BUILD prompt`
     - `SPEC-156 — VERIFY prompt`
     - `SPEC-156 — PUNCH LIST prompt`

     Read each prompt's first line to confirm its title. Use the transcripts whose **first user
     message** holds the title, because the orchestrator's transcript also contains every title.
   - **Dedup by `.message.id`.** Count the three input counts once per id, and take the **MAX**
     `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
   - **Price** at the model in `.message.model`: Sonnet $3/$15, Opus $5/$25. cache_creation is
     ×1.25 the input rate and cache_read ×0.10.
   - `cost.totals` reads 0 today. Recompute it from every entry. Run `just cost-audit` and quote the
     line that names SPEC-156. If it fails on another spec, say which and that it isn't yours.
6. **Mark ship `[x]`** with the merge SHA, PR #269, the date and the total cost. Then run
   `just advance-cycle SPEC-156 ship` and `just archive-spec SPEC-156`.
   - Read STAGE-063's SPEC-156 entry back. It should read
     `- [x] **SPEC-156** (merged <date>, PR #269) — [S] …`, in the style of the SPEC-154 entry. Fix it
     by hand if not.
   - Touch no other entry.
   - Recount the `**Count:**` line with greps you just ran, and show them. Today it reads
     `4 shipped / 0 active / 19 pending`; expect `5 shipped / … / 18 pending`, unless another session
     changed the stage meanwhile (SPEC-155 may ship close to this; say so if one did).
   - ⛔ Don't change STAGE-063's `status` or its reflection.
7. **Commit to `main` directly** (AGENTS §13).
   - Commit with
     `git commit -s -m "ship(SPEC-156): meta strip keeps the Adobe APP14 colour-transform marker — reflected, archived"`
     and the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is
     mandatory.
   - Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`. On a
     reject, rebase and push again.
   - Read back `git show --stat HEAD`.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, fixtures, CI, `docs/`, HANDOFF or templates. The only DEC edits are step 2's.
  Propose anything else in your report.
- The spec's body changes only by the reflection and the cost block.
- Another session may push to `main` while you work. Rebase; never force.
- A piped command reports the pipe's exit code. zsh does not word-split unquoted parameters, and it
  reads `$R:r…` as a modifier, so brace it (`${R}:`). `rtk` can truncate `grep`, so count with
  `/usr/bin/grep` or Python.
- Write scratch files only in your session's scratchpad, never `/tmp`.
- **Budget ~30 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-156 ship: archived — <commit SHA on main>`. Then:
- the merge SHA and date;
- the two DEC edits, with the front-matter result;
- step 3's pixels and APP14 counts;
- STAGE-063's SPEC-156 line as it now reads, and the recounted `**Count:**` line with its greps;
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- any template, constraint or DEC change you propose;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its
  own line.
