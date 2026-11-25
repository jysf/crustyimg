# SPEC-161 — FRAME prompt: crop on the ddphotos edit page

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-161-frame -- --model opus
```

**Reserved: SPEC-161.** PROJ-011's STAGE-063 reserves SPEC-154 to SPEC-158, and SPEC-160 shipped on
2026-11-13. Before you allocate, confirm SPEC-161 is unused as a spec id:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-161' "$b" -- . ; done
```

The expected hits are this prompt, the PROJ-017 brief's reservation line, and SPEC-159's and
SPEC-160's frame prompts (which mention "SPEC-161 or later" in passing). If a spec file of its own or
another reservation uses SPEC-161, stop and report. Don't pick another number. ⚠ `just new-spec`
skips any id named in prose, so it will likely offer SPEC-162 or later; take SPEC-161 anyway (see
"Allocate the id").

Your item is the STAGE-060 Spec Backlog entry "**Crop on the page.**" Read it in full, with both ⚠
paragraphs: the watermark refit after a crop, and the EXIF-rotated size that `info()` reports. Two
more STAGE-060 entries bear on it, and question 9 asks you to recommend fold or leave for each:
- "**Label the parity harness's table checks.**" Its own text says to do it alongside this spec's
  first harness rows;
- "**`crop`'s aspect can follow each image's orientation.**" The page's aspect presets are its first
  user.

Touch no other item except as this prompt says.

## Why this spec exists
STAGE-060 is the proof half of PROJ-017's thesis. An edit made in a page that can't run the CLI is
handed to the CLI as a recipe, and it reproduces on the full-size original. SPEC-142 built the
native-vs-wasm parity harness on today's resize and watermark recipes. STAGE-058 shipped `crop` as a
recipe step, in fractions of the image (DEC-112). **This spec puts crop on the page.** It is the
first geometry the page emits, so it is the first real test of "the recipe means the same thing at
4× the size".

## Settled; don't re-open
- **Units** (DEC-112): a recipe carries crop as fractions of the image it receives, and each edge is
  rounded. The page emits fractions, never pixels.
- **Step order on the page** (STAGE-060 design note): `auto-orient` → `rotate` → `crop` → `resize` →
  `watermark` → (`optimize`). `rotate` arrives with "Straighten on the page", not here.
- **A geometry step acts on the buffer it receives and writes no metadata** (DEC-113).
- **Parity** (DEC-110): native and wasm agree on dimensions exactly, and on resampled pixels within
  1/255. Crop doesn't resample, so the crop rows must match exactly *before* the resize.
- **Out of the stage** (`explicitly_does_not`): no save flow or sidecar, no ddphotos code change, and
  no flip, pad or trim on the page.
- **The page is a static add-on** built from `integrations/ddphotos/edit/` and installed by
  `just ddphotos-edit-install`. It uses the slim wasm (`pkg-slim`).

The frame's job is everything those leave open.

## What already exists (read it, don't assume it)
- **The page**: `integrations/ddphotos/edit/`, meaning `edit.js` (the form, `readEdit`, `runPreview`,
  `makeCopy`, `showEditor`), `recipe.js` (`planEdit`, `pixelRecipe`, `cliRecipe`, `previewRecipe`,
  `watermarkSize`), `worker.js`, `index.html`, `edit.css`, and the README. Note what the preview
  shows today: the **finished** image, rendered by wasm from `pixelRecipe`. A free-form crop needs
  something to drag on. Say what that is.
- **`planEdit`** takes `photo = { width, height }` from wasm `info()` (the stored size), predicts the
  output's sides as `resize` mode `max` computes them, and fits the watermark to them.
  `cliRecipe` always resizes to `outLong`, so the CLI's output on the original matches what the page
  made.
- **The parity harness**: `tests/ddphotos_parity.mjs` (`just ddphotos-parity`, under
  `target/ddphotos-parity/`), with SPEC-142's spec and its re-verifies in `specs/done/`. Read its
  rows, its 4×-copy replay, the transposed-prediction rows 15–16, and how it was made to fail
  ("two recipes that differ by one pixel of crop must go red").
- **`crop`** (`src/operation/crop.rs`) and SPEC-145 in `specs/done/`: the `box` and `aspect` forms,
  `gravity`, the rounding rule, the minimum size, and the refusals. Also the wasm `transform` path
  that runs the same op.
- **The test bed** lives outside this repo
  (`~/claudia/web/jysites/fogcityphotosandbeyond/crustyimg-ddphotos-testbed`, no git remote; see
  `specs/prompts/testbed-editor-clean.md` for its rules). Its browser round is a **separate**
  STAGE-060 entry. Don't run it here. Read its `CLAUDE.md` only if you need to know what its checks
  can reach.

## Questions this frame must answer
1. **What the user drags on.** The preview shows the finished image, so a crop box drawn on it moves
   as the crop changes. Recommend the interaction: a crop mode that shows the uncropped
   (auto-oriented, resized-for-screen) image with a box over it, then returns to the finished
   preview, or another shape. Say what wasm renders for each view and how often (a drag must not
   queue a wasm render per mouse move; `runPreview` already debounces at 200 ms). Keyboard and touch
   matter: say what the minimum is.
2. **Presets and free-form.** At least 1:1, 4:5, 3:2 and 16:9 (Success Criteria). Does a preset emit
   `aspect` + `gravity` or a `box`? Recommend one. A preset the user then drags is a `box`. Say
   whether a preset follows the photo's orientation (a 3:2 preset on a portrait photo), which is the
   orientation entry's question. Recommend: fold that entry into this spec, or ship the page's
   presets on the page side (swap W:H in JS) and leave the recipe key for later.
3. **Orientation, driven.** The ⚠ paragraph says `info()` reports the stored size, so an EXIF-rotated
   JPEG reads 1600×1071 when its output is 1071×1600. **Drive it** with a JPEG you tag yourself
   (`exiftool -Orientation=6 -n`). Run wasm `info()` and native `crustyimg info`, then run
   `transform` with `auto-orient` + a crop on both builds. Recommend how the page gets the oriented
   size (swap on the EXIF orientation it reads, a second wasm call, or the decoded preview's
   `naturalWidth`), so crop fractions live in the space `auto-orient` produces. Say what changes in
   the harness's rows 15–16, which pin the transposed prediction today.
4. **The size the page predicts after a crop.** `planEdit` predicts `outW`/`outH` for the watermark
   fit and for "what it made". After a crop, the CLI computes the crop box on the **original** and
   then resizes to `outLong`. The page computes it on the published copy. Do the two give the same
   output sides? **Drive it** over many crop fractions with a 1×/4× (and a non-integer ratio, such
   as 1200 vs 4032) pair of sizes. Use DEC-112's rounding on each edge, then `resize` mode `max`'s
   rounding. Report the worst case. If a side can differ by one pixel, say what the page shows and
   what the parity test asserts (AGENTS §15: a bound is a claim until its worst input is driven).
5. **The watermark refit.** The ⚠ paragraph says a crop that shrinks the image must re-trigger "Made
   smaller to fit this photo". Read `watermarkSize` and say where the crop enters `planEdit` so the
   fit uses the cropped sides. Name a crop that flips the `capped` state, for a parity row.
6. **`cliRecipe` on the original.** It resizes to `outLong`. With a crop, `outLong` is the cropped
   long edge on the copy. Show the recipe the page emits for one crop, and say whether replaying it
   on a 4× original gives the same framing and the same output size the page made. If the original
   is *smaller* than `outLong` after the crop (a heavily cropped copy of a small original), say what
   happens: `resize` mode `max` doesn't upscale.
7. **Parity rows.** List the rows this spec adds: each preset, a free-form box, an off-centre
   gravity, an EXIF-rotated input, the watermark refit, and the 4× replay for each. List the
   negative control the harness must keep: a one-pixel crop difference goes red. Say which rows
   match exactly and which match within DEC-110's bound (crop + resize resamples).
8. **wasm size.** Crop is already in the slim build if `transform` runs it. **Check** whether it is
   (feature gates, `pkg-slim`'s op list) and run `just wasm-size`. If the bundle doesn't change, say
   so with the numbers. The Success Criterion asks for the growth stated.
9. **Fold or leave.** For each of the two entries named above, recommend: **fold** (only if it
   doesn't change the size), **leave** in STAGE-060, or **move**. For the orientation entry, note
   it changes the recipe format for `crop` **and** `pad` (SPEC-151's frame named it), so it's
   CLI work, not page work. Don't move anything. Recommend.
10. **The three handed-in items** (PROJ-017 brief, "Handed in from PROJ-011"): `format = "bogus"`,
    `quality = 101`, and the wasm `validate()` export. They touch recipe load. Does this spec touch
    any file they'd touch? Say yes or no with the files, so the orchestrator can schedule them
    alongside. Don't frame them.
11. **Docs.** The page README's description of the controls, the CLI panel's wording on the page,
    and any doc that lists what the page can do (`integrations/ddphotos/README.md`, the roadmap row,
    `docs/`). STAGE-060 has a separate "Test-bed round and README" entry. Recommend which wording
    belongs here and which there. Per AGENTS §15, list each behaviour sentence the build must
    **drive on every exit path it names**.
12. **How the build is checked without a browser.** The parity harness runs in Node. The drag UI
    doesn't. Say what is testable in Node (`recipe.js` is pure: `planEdit` with a crop, the emitted
    recipes, the orientation swap), what needs the test bed's headless Chromium (the separate
    entry), and what a verify session can drive on its own. Never the maintainer's browser.
13. **Size.** The backlog says [M]. A crop UI with presets and a crop mode, an orientation fix, the
    size-prediction rounding, the watermark refit, the harness rows and possibly a recipe key may be
    more than [M]. If it is, recommend a split (for example: the `recipe.js` model and the harness
    rows first, then the drag UI), and say what each half ships on its own.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context, and
you do **not** write any DEC. You recommend its content. Separate design, build and verify sessions
follow. The orchestrator only sequences.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables**: wide tables have lost lines in transit.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather than
a worktree, stop and say so.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 backlog, §12, §13 git, §15 (its rules bind every cycle,
   including the new "During design" section and "a bound or 'can't happen' is a claim until its
   worst input is driven") and §16 habit 6 (no CI wait loops).
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, the build order, "Handed in
   from PROJ-011", and "Explicitly out of scope".
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md`:
   your item, every open entry, the `delivers` lines, the Success Criteria and the design notes.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for". The
   file exists at that path on `main`. If it isn't in your checkout, say so.
5. DEC-110, DEC-112, DEC-113, DEC-114.
6. SPEC-142 and SPEC-145 in `projects/PROJ-017-…/specs/done/`, with their ship reflections, and the
   STAGE-058 Stage-Level Reflection. Apply its lessons in your stub where they arise: a mutation row
   quotes its exact mutant edit, a count carries its derivation, build drives each doc sentence on
   every exit path, and design re-reads its DEC against its own corrections.
7. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code.** On a release build of `main` (fresh `CARGO_TARGET_DIR`,
  deleted at the end) and a slim wasm build: run a crop recipe through native `apply` and wasm
  `transform` on one fixture and compare. Run `just ddphotos-parity` and record its count.
- **Drive the orientation case (question 3) and the size bound (question 4).**
- **Size the verification, not just the code** (AGENTS §2): presets × orientations (landscape,
  portrait, square, EXIF-rotated) × watermark on/off and capped × 1×/4× × output formats.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths), and any fold-in's files.
- **Questions only the maintainer can answer:** one line each, with your recommendation. The
  interaction shape (question 1) is probably one.

### Allocate the id
Run `just new-spec "<title>" STAGE-060 PROJ-017`. If it gives anything other than SPEC-161, `git mv`
the spec and its timeline to SPEC-161 and fix the id inside both. Then run
`just advance-cycle SPEC-161 frame`. If you recommend a DEC, it takes the next free id (DEC-116 on
`main` today). Recommend its content; don't write it.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references` (DEC-110,
  DEC-112, DEC-113, SPEC-142, SPEC-145), and `cycle: frame`.
- `## Context` and `## Goal`: what a photo owner can't do on the page today, and what they can when
  this ships.
- A `## Frame` section directly after `## Goal`. It holds:
  - the verdict (go / no-go / split);
  - the premise check, with evidence;
  - the interaction you recommend, and what wasm renders when;
  - presets: aspect or box, and how orientation is handled;
  - the orientation drive and the page's fix;
  - the size-prediction bound, driven, and what the page and the harness do with it;
  - the watermark refit;
  - `cliRecipe` on the original, including the smaller-original case;
  - the parity rows and the negative control;
  - the wasm size;
  - fold/leave for the two entries, and the handed-in items' file overlap;
  - what is testable where;
  - sizing and why, and any split;
  - scope in/out, including fold-ins;
  - the rough build file list;
  - open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design
  (`prompts/SPEC-161-design.md`), build, verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-161 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-060's backlog, change your item's `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-161** (framed <date>)`. Fold another entry only if you recommend it **and** it
   doesn't change the size: mark it `- [x] (folded into SPEC-161)`. Otherwise leave it and put it to
   the maintainer. On a split, file the second half as a new `- [ ] (not yet written)` entry right
   after yours. File any out-of-spec finding as a new `- [ ] (not yet written)` entry at the end of
   the stage it belongs to. Update the `**Count:**` line with greps you've just run, and show them.
   Something with no stage goes in `guidance/ideas.yaml` (AGENTS §10).
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-161): frame crop on the ddphotos edit page`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-060 diff.
   ⚠ **PROJ-017 is deliberately `proposed`** (ruled 2026-11-13, until the status tooling can show
   more than one active project; `guidance/ideas.yaml`, `status-tooling-many-active-projects`). So
   `just status`, `just backlog` and `just roadmap` show PROJ-011, not this stage. That's expected.
   Don't change any project's status. Use a grep instead.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag, no CHANGELOG edit. A probe you drive at frame
  lives in a scratch directory and is never committed.
- Don't move or close any other STAGE-060 entry (question 9 is a recommendation), and don't change
  STAGE-060's status. Don't touch the test bed.
- **Never drive the maintainer's browser.** Any browser check runs in headless Chromium (Playwright),
  and at frame you shouldn't need one.
- Other sessions may be working at the same time. Never share a `CARGO_TARGET_DIR`. If
  `git rebase origin/main` conflicts in a file outside PROJ-017, stop and report.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters, and reads `$R:r…` as a modifier, so brace it (`${R}:`). macOS has
  no `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and
  `/usr/bin/git` for anything you count.
- Keep scratch files in your session's scratchpad, never in `~`, `/tmp` or the system temp dir.
- **Budget ~45 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`/usr/bin/grep -l "SPEC-161 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`.

Price each component at the model that actually ran (read `.message.model`), never at a flat rate:
Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
Measure at your last turn, after the push, and append a `frame` entry to the spec's `cost.sessions`
with `tokens_total`, a `tokens_breakdown`, `estimated_usd` and the rate anchors in a final small
commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-161 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`.
Then, short:
- the premise check (confirmed or wrong, with one line of evidence) and the parity harness count;
- the interaction you recommend, and what wasm renders when;
- presets: aspect or box, and how orientation is handled;
- the orientation drive, quoted, and the page's fix;
- the size-prediction bound: driven over what, and the worst case;
- the watermark refit, and a crop that flips `capped`;
- `cliRecipe` on the original, including the smaller-original case;
- the parity rows, and the negative control;
- the wasm size, with numbers;
- fold/leave for the two entries, and the handed-in items' file overlap;
- what is testable in Node, what needs the test bed, and what verify can drive;
- size and why, scope in/out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
