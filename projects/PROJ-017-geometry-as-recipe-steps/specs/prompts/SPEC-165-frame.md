# SPEC-165 — FRAME prompt: the watermark's fit on a small or thin crop

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-165-frame -- --model opus
```

**Reserved: SPEC-165.** The highest spec id on `main` is SPEC-164. SPEC-154 to SPEC-158 are reserved by
PROJ-011's STAGE-063. Before you allocate, confirm SPEC-165 is unused:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-165' "$b" -- . ; done
```

The only expected hit is this prompt. If a spec file or another reservation uses SPEC-165, stop and
report, and don't pick another number. ⚠ `just new-spec` skips any id named in prose, so it may offer
SPEC-166 or later. Take SPEC-165 anyway (see "Allocate the id"). If you recommend a DEC, it takes the
next free id, which is DEC-119 on `main` today. Recommend its content; don't write it.

Your item is the STAGE-060 Spec Backlog entry "**The watermark's fit on a small or thin crop.**" Read it
in full, including "Found at SPEC-164's design" and "Ruled 2026-11-27". Touch no other item except as
this prompt says.

## Why this spec exists
The page sizes the watermark itself, in `recipe.js`'s `watermarkSize`, called from `planEdit`. Without a
crop, every output is at least 400 px and the engine never refuses the mark. A cropped box can be small
or thin, and then the engine refuses it in three ways, all driven at SPEC-164's frame and design:
- **coverage over 5 %:** "© 2026 Jane Doe" at Medium is refused on square boxes of 1–50 px;
- **the margin on the short side:** a 1600×20 strip at Large is refused on its 32 px margin;
- **the mark's width plus one margin on a thin vertical strip resized smaller:**
  `P 64x1600 @400 -> 16x400` is refused even at the 64 px minimum box.

The maintainer ruled that `MIN_CROP` stays 64, as a usability floor rather than a watermark guard. No
copy-pixel minimum covers every text, because the output is resized after the crop. **This entry makes
the page predict the engine's refusal instead of meeting it**, and it lands before the crop mode's DOM
entry, the first change that lets a user draw such a box.

## Settled; don't re-open
- **The recipe model** (SPEC-161, SPEC-164 and DEC-118): crop after `auto-orient`, before `resize`; the
  crop state model and `planEdit(photo, form, crop)`; `MIN_CROP = 64`.
- **Parity** (DEC-110): native and wasm agree on dimensions exactly. The page's predictions are checked
  against native `apply` with literals (HANDOFF trap 19), never against the page's own code.
- **The page is a static add-on** on the slim wasm (`pkg-slim`). It gets no save flow, no ddphotos code
  change, and no flip, pad or trim.
- **The engine's rules stay as they are.** The 5 % cap, the fit check and the margin are crustyimg's
  behaviour for every caller. This spec changes what the page asks for, not what the engine accepts. If
  you think the engine is wrong, file it as a finding rather than planning it here.
- **Never the maintainer's browser.** Any browser check runs headless, through
  `tests/ddphotos_page.mjs` and `scripts/lib/chrome.mjs` (SPEC-164).

## What already exists (read it; don't assume)
- **The engine's check:** `src/operation/mod.rs`, `Refusal` (about line 240) and the watermark's
  `refusal()` (about line 1894).
  - Fit is checked first: the mark's drawn size, plus the margin on the axes the gravity insets.
  - Then coverage: the mark's **visible** pixel count against 5 % of the area, for a mark with no
    `scale` that isn't tiled.
  - Read where `visible` and the mark's size come from for a text mark: which font, which rasteriser,
    and whether antialiased pixels count.
- **The page's model:** `integrations/ddphotos/edit/recipe.js`.
  - `textEms` uses the bundled Go font's advance widths, with 2240 units for anything outside Latin-1.
  - `watermarkSize` caps the size to 90 % of `outW - margin` and to a tenth of the short side.
  - `planEdit` sets `margin = round(outLong × 0.02)` and returns `watermark.capped`.
- **What the page does with a refusal today:** `edit.js` shows the engine's message as "Preview
  failed: …", and Make fails with "Could not make the copy: …". Confirm this, and say what a photo
  owner sees and can do about it.
- **The harnesses:** `tests/ddphotos_parity.mjs`, with its table, crop, replay, orientation, alpha and
  crop-mode rows and `checkModules`, and `tests/ddphotos_page.mjs`. Read HANDOFF trap 22 before you
  plan any page row.
- **SPEC-164** in `specs/done/`: "The minimum box (driven; a finding)", the design's
  watermark-fit finding, and the reflection.

## Questions this frame must answer
1. **The premise, driven.** Run every refusal kind through native `apply` and through `planEdit`, on W
   (1600×1071), P (1071×1600), R (stored landscape, upright portrait) and a square photo. Use the
   three sizes, a short text and a long one (at least 60 characters, and one with a character outside
   Latin-1), and boxes from 64 px to full size, with and without a resize. Report:
   - how many refusals each kind has;
   - the smallest clean box per text and size;
   - whether any refusal happens **without** a crop. The entry says none; drive it.

   A bound is a claim until its worst input is driven (trap 17).
2. **Can the page predict each kind exactly?**
   - **Fit** depends on the mark's drawn size. Does `textEms × size` equal the engine's rendered
     width and height, or is it off by kerning, hinting or rounding? Measure the error over the
     sweep.
   - **Coverage** depends on the visible pixel count, which only a rasteriser knows.
   - For each kind, say which of these the page can do:
     - (a) predict it exactly in `recipe.js`;
     - (b) bound it conservatively, never letting a refusal through, at the cost of some marks
       capped below what the engine would allow; measure how many;
     - (c) ask the engine, for example through a wasm export that returns the mark's size and visible
       count, or a dry-run of the step.

     (c) adds Rust and wasm. Measure what it costs with `just wasm-size`, and say what DEC-110
     requires of it.
   - Recommend one approach per kind, with the numbers.
3. **What the page does when the mark can't fit at the chosen size.** The options:
   - shrink the mark further, and to what size;
   - drop the mark, with a note;
   - keep the box and block Make with a plain message;
   - something else.

   Today's "Made smaller to fit this photo" note covers the width and height caps. Say whether it also
   covers the new caps or needs its own wording, and write each user-visible sentence you recommend.
   Recommend one option; this is the main question for the maintainer.
4. **The CLI recipe.** The page writes the size it chose into the recipe. Under DEC-118 rule 5, the
   recipe runs on the original or on a file at least as large as the copy. Drive whether a mark the
   page capped for a small box is still accepted when the same recipe runs on the original. Say
   whether the CLI panel's sentence still holds, or needs a clause.
5. **Gravity and margin.** The engine insets the margin only on the axes the gravity names. The page
   offers some set of gravities; list them. For each, say which axes take the margin. Say whether
   `watermarkSize`'s `outW - margin` is right for every gravity, and for the height axis, which it
   doesn't check today.
6. **Rows and literals.** Plan parity rows for each refusal kind and each fix: the predicted size, or
   the recommended outcome, against native `apply`, with literals.
   - Write thin strips in both orientations (trap 20).
   - Name one mutant that both sides share and the literal that catches it (trap 19).
   - If any row needs the page, say how it proves which document it's reading (trap 22).
   - Say the row count, and where each row lives.
7. **Mutants.** Name five or more that a working check must catch. For example: the margin not checked
   on the short side; coverage checked against the copy instead of the output; the cap applied before
   the resize; a refused mark shown as a success. Say which row catches each. Traps 12, 13 and 19–22
   apply.
8. **The wasm size.** If you recommend (a) or (b), confirm that no Rust or wasm changes. If you
   recommend (c), give the measured delta against the slim bundle's budget.
9. **Docs.** List the page README's watermark wording, the CLI panel, and any doc that states the
   page's watermark sizes. List each behaviour sentence the build must drive on every exit path it
   names (AGENTS §15).
10. **Size.** The backlog says [S]. A wasm export, or a third refusal kind with its own wording, may
    make it [M]. If it does, recommend a split, and say what each half ships on its own.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is a go/no-go plus a correctly sized, correctly scoped spec
stub. You don't write acceptance criteria, failing tests, Implementation Context or any DEC. You
recommend a DEC's content.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables.**

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you are in the
maintainer's main checkout rather than a worktree, stop and say so.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 backlog, §12, §13 git, §15 (its rules bind every cycle) and
   §16 habit 6 (no CI wait loops).
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`.
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md`:
   your item, every open entry, the `delivers` lines and the Success Criteria.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`, "Traps this wave paid for",
   especially 11–14, 17 and 19–22. Traps 13, 14 and 22 changed on 2026-11-27.
5. DEC-110, DEC-112, DEC-113 and DEC-118. Find every DEC that mentions the watermark's cap or margin.
6. SPEC-161 and SPEC-164 in `specs/done/`, with their ship reflections. SPEC-128 and SPEC-143 cover the
   watermark's engine side; find them and read their reflections.
7. `/guidance/constraints.yaml`.

### What a good frame does here
- **Drive question 1 before you answer anything else.** Use a scratch script that calls the release
  binary and `recipe.js`. Never commit it.
- **Size the verification, not just the code** (AGENTS §2): refusal kinds × orientations × sizes ×
  texts × gravities.
- **Scope boundary:** what's in, what's out, and where each out item lives.
- **Rough build file list** (paths).
- **Questions only the maintainer can answer:** one line each, with your recommendation.

### Temp folders and build dirs
- Before your first harness run, count the system temp folder's `ddphotos-*` folders and their size.
  Don't delete any of them: some belong to other sessions.
- Run every harness command with `TMPDIR` set to a folder in your scratchpad, and confirm the folders
  landed there.
- Use your own `CARGO_TARGET_DIR` in the scratchpad, never shared, and delete it at the end.
- Report both `ddphotos-*` counts and the space you freed.

### Allocate the id
Run `just new-spec "<title>" STAGE-060 PROJ-017`. If it gives anything other than SPEC-165, `git mv`
the spec and its timeline to SPEC-165, and fix the id inside both. Then run
`just advance-cycle SPEC-165 frame`.

### What to write into the stub
- **Front matter:** `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (DEC-110, DEC-118, SPEC-161, SPEC-164, and any DEC you found in "Read first" step 5), and
  `cycle: frame`.
- **`## Context` and `## Goal`:** what a photo owner sees today when a crop makes the mark unfit, and
  what they see when this ships.
- **A `## Frame` section directly after `## Goal`.** It holds:
  - the verdict (go, no-go or split);
  - the premise, driven;
  - predictability per kind, with your recommendation;
  - what the page does, with its sentences;
  - the CLI recipe;
  - gravity and margin;
  - the rows and literals;
  - the mutants;
  - the wasm size;
  - docs;
  - sizing and any split;
  - scope in and out;
  - the rough build file list;
  - the open maintainer questions.
- **`### Out of scope (for this spec specifically)`**, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- **The timeline:** mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design
  (`prompts/SPEC-165-design.md`), build, verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-165 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-060's backlog, change your item's `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-165** (framed <date>)`.
   - On a split, file the second half as a new `- [ ] (not yet written)` entry right after yours.
   - File any out-of-spec finding as a new `- [ ] (not yet written)` entry in the stage it belongs to,
     or in `guidance/ideas.yaml` if it has no stage (AGENTS §10).
   - Update the `**Count:**` line with greps you've just run, and show them. Today it reads
     `5 resolved (4 shipped specs, 1 folded) / 0 active / 5 pending`, so expect
     `… / 1 active / 4 pending` with no split.
3. Commit with `git commit -s`. **The DCO sign-off is mandatory.** Use the message
   `docs(SPEC-165): frame the watermark's fit on a small or thin crop`, with the trailer
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. A "Changes must
   be made through a pull request" notice can still mean the push landed, so check with
   `git ls-remote origin refs/heads/main`. ⛔ **If a permission check refuses the push, stop and
   report** (trap 21). Name the commits and the command for the maintainer, and don't claim the push
   happened.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-060 diff. ⚠ The status tooling
   shows only the highest-numbered project. If `just status` or `just backlog` doesn't show
   STAGE-060, that's expected; use a grep instead.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag, no CHANGELOG edit. Probes live in the
  scratchpad and are never committed.
- Don't move or close any other STAGE-060 entry, and don't change STAGE-060's status. Don't touch the
  test bed.
- **Never drive the maintainer's browser.** A headless Chromium you start yourself is fine.
- Never share a `CARGO_TARGET_DIR`. If `git rebase origin/main` conflicts in a file outside PROJ-017,
  stop and report.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh doesn't
  word-split unquoted parameters, and it reads `$R:r…` as a modifier, so brace it (`${R}:`). macOS has
  no `timeout(1)`. `rtk` can truncate `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git`
  for anything you count.
- Keep scratch files in your session's scratchpad, never in `~`, `/tmp` or the system temp folder.
- **Budget ~50 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working directory's
absolute path with every `/`, `.` and `_` replaced by `-`.
- **Find it by content, not recency:**
  `/usr/bin/grep -l "SPEC-165 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
- **Price** at `.message.model`: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the
  input rate, and cache_read ×0.10.
- Measure at your last turn, **after** the push. Append a `frame` entry to the spec's `cost.sessions`
  in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-165 frame: GO | NO-GO | SPLIT`, then the commit SHA on
`main`. Then, short:
- the premise: refusals per kind over the sweep, the smallest clean boxes, and whether any refusal
  happens without a crop;
- predictability per kind: the measured error of `textEms`, and your recommendation (a, b or c) with
  its numbers;
- what the page does, with each sentence you recommend;
- the CLI recipe on the original: driven, with the result;
- gravity and margin;
- the rows planned, by kind, with their count and where they live;
- the mutants, and which row catches each;
- the wasm size;
- docs, and the behaviour sentences;
- size and why, scope in and out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them;
- the `ddphotos-*` counts before and after, and the space freed;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
