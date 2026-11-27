---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-060
  status: active
  priority: medium
  target_complete: null

project:
  id: PROJ-017
repo:
  id: crustyimg

created_at: 2026-10-30
shipped_at: null

value_contribution:
  advances: >
    The proof half of PROJ-017's thesis: an edit made in a page that cannot run
    the CLI (the ddphotos edit page) is handed to the CLI as a recipe and
    reproduces on the original. Without this stage the thesis is asserted;
    with it, it is demonstrated by its first consumer.
  delivers:
    - "Crop on the ddphotos edit page: free-form and aspect presets"
    - "Straighten on the ddphotos edit page: an angle control with crop-to-fit"
    - "A recipe from the page that the CLI replays on the full-size original, proven by a native-vs-wasm parity test"
  explicitly_does_not:
    - "Save the edit next to the photo (roadmap row 6b, recipe sidecars)"
    - "Add anything to ddphotos itself. The page stays a static add-on; an Edit button in the lightbox is Doug's call"
    - "Add flip, pad or trim to the page. They're available in the recipe, but a photo owner rarely reaches for them, and each control is UI to maintain"
---

# STAGE-060: The ddphotos edit page crops and straightens, and its recipe replays on the original

## What This Stage Is

When this stage ships, the edit page (`integrations/ddphotos/edit/`) has a crop control and a
straighten control beside its resize and watermark controls. The preview, the finished copy
and the "run this on the original" recipe all agree, and a test proves the recipe means the
same thing to the native CLI as to the page.

## Why Now

The page is the reason PROJ-017 is framed now (roadmap row 5, rewritten in #219), and its
test bed went green on 2026-10-29 (#227). The crop half can start as soon as STAGE-058's crop
spec ships; the straighten half waits for STAGE-059.

## Success Criteria

- The page offers crop with free-form drag and presets (at least 1:1, 4:5, 3:2, 16:9), and a
  straighten angle control (about ±15°) defaulting to `crop-to-fit`.
- The recipe the page shows carries crop in size-independent units (STAGE-058's unit DEC) and
  in the right place in the step order relative to `resize`.
- A parity test runs the page's emitted recipe through native `crustyimg apply` and through
  wasm `transform` on the same input. It asserts the outputs match (exactly for crop; within
  a stated bound for straighten), and runs it again on a 4× larger copy to prove the units
  are size-independent.
- The test bed's browser checks cover both controls on dev, static and apache, as #227's
  rounds did.
- The wasm bundle stays inside its size gate (`just wasm-size`), with the growth from the
  new ops stated.

## Scope

### In scope
- `edit.js` controls, preview, and the recipe text (`pixelRecipe` / `cliRecipe`)
- The parity test and the larger-copy replay
- The page README's description of the new controls
- A test-bed round covering them

### Explicitly out of scope
- Recipe sidecars and any save flow (row 6b)
- The prebuilt edit-page zip on each release (#225's proposal, logged in `guidance/ideas.yaml`)
- ddphotos code changes

## Spec Backlog

In build order.

- [x] **SPEC-142** (merged 2026-11-01, PR #234) — [S] **The parity harness, first.** Before any UI: a test that takes
  a recipe string, runs it through native `apply` and wasm `transform` on the same fixture,
  and compares. Run it on today's resize + watermark recipes so it's green before geometry
  arrives. Then each later spec adds a row. ⚠ Assert it can fail: feed it two recipes that
  differ by one pixel of crop and require red.
- [x] **SPEC-163** (merged 2026-11-26, PR #272) — [M] **`crop`'s aspect can follow each image's orientation.** Filed by the
  orchestrator on 2026-11-05 at the maintainer's request. `aspect = "7:5"` always means 7 wide by 5
  tall (SPEC-145). A batch recipe such as "5×7 prints of this folder" wants 7:5 on landscape
  photos and 5:7 on portrait ones, which today takes two recipes and a hand sort. Add one optional
  key (for example `orientation = "match"`). Leaving it off keeps SPEC-145's meaning, so no
  existing recipe or golden changes. Design decides what a square image gets, the key's name, and
  the refusals. ⚠ `pad` to an aspect has the same question: decide it here, or name it in `pad`'s
  frame. Named presets (`"5x7"`, `"square"`) stay out of the recipe format: a ratio is
  unambiguous, and presets belong to the page's buttons (STAGE-060) and a docs table. After
  SPEC-145. SPEC-151's frame (2026-11-06) named this entry for `pad`: its aspect means W wide by H
  tall as crop's does, and this entry's spec adds the key to both ops with one rule.
  **Moved here from STAGE-058 on 2026-11-11** (maintainer ruling at SPEC-160's frame): the page's aspect
  presets are this key's first user, and it changes no shipped output, so STAGE-058 closes without it.
  **Ahead of SPEC-161 (maintainer ruling 2026-11-22, at SPEC-161's frame):** the page's presets emit
  `aspect`, not a box, so a recipe copied from the page reads `aspect = "3:2"` and works on any
  photo. To work on a folder of mixed photos it needs this key, so SPEC-161 emits it and this
  entry builds first. Builds after the STAGE-063 P1 fixes ship.
- [x] **SPEC-161** (merged 2026-11-27, PR #273) — [M] **Crop on the page.** Free-form and aspect-preset crop over
  the preview, emitting a `crop` step ahead of `resize` in both `pixelRecipe` and `cliRecipe`
  (`integrations/ddphotos/edit/recipe.js`): an untouched preset as an `aspect` (with
  `orientation = "match"` unless the photo is square or the preset is 1:1), a placed box as four
  fractions. Parity rows for each preset, plus the
  4× replay. ⚠ The page's existing "Made smaller to fit this photo" watermark logic runs after
  the crop. Check that a crop which shrinks the image re-triggers it.
  ⚠ **The page takes the photo's size from wasm `info()`, which reports the stored size.** For
  an EXIF-rotated JPEG that is 1600×1071 when the output is 1071×1600, so `planEdit` predicts
  landscape and sizes the watermark for it. Crop coordinates must live in the space `auto-orient`
  produces. This is a page workaround and belongs to this spec, not an `info()` bug: native
  `crustyimg info` reports the stored size too, and wasm `info()` is its twin. Found in SPEC-142's
  design; rows 15–16 of its harness pin the transposed prediction. Harmless today, because
  photogen publishes WebP copies that are already oriented.
  **Split at frame (2026-11-22):** SPEC-161 is the half Node can test: `recipe.js`'s crop model,
  the oriented-size fix, the parity rows and the 4× replay. The crop mode is the next entry.
  **Designed 2026-11-26:** 35 crop rows, 17 replay rows within the new DEC-118 (box exact at an
  integer multiple except at an `f64` tie, aspect within 1 px; PSNR floors 53.5/59 dB that a
  one-pixel shift falls below), N4 and N5; no Rust change, 0 B of wasm.
  **Ruled 2026-11-22:** an untouched preset emits `aspect` plus the orientation key (readable and
  reusable), and a dragged box emits a box. See SPEC-161's "Maintainer rulings". After the
  orientation-key entry above.
- [x] **SPEC-164** (merged 2026-11-27, PR #274) — [M] **Crop on the page: the crop mode.** The second half of the entry
  above, split at SPEC-161's frame. A "Crop" button shows the oriented base image (rendered once at
  load by SPEC-161's worker) with a DOM box over it; dragging renders nothing, and "Done" schedules
  one finished preview. Preset buttons (1:1, 4:5, 3:2, 16:9) that follow the photo's orientation,
  with a swap, and emit `aspect` until the box is dragged; Pointer Events with `touch-action: none`; a focusable box that arrow keys move and
  Alt+arrow resizes; a live size line. The CLI panel says to run the recipe on the original or any
  file at least as large as the copy (a smaller one gives a smaller output, or a refused watermark).
  Calls SPEC-161's `recipe.js` functions and adds no recipe logic. Its browser check is the
  "Test-bed round and README" entry. After SPEC-161. See SPEC-161's `## Frame` for the interaction.
  **Split at frame (2026-11-27):** SPEC-164 is the half provable before the DOM exists: the crop
  mode's state machine and box geometry as pure `recipe.js` functions (`toCopyDelta`, `moveBox`,
  `resizeBox`, `MIN_CROP`, `cropReducer`, `sizeLine`) with a Node table, the CLI panel's
  smaller-file sentence, and an in-repo headless-Chrome check of the edit page (`tests/ddphotos_page.mjs`,
  in CI's `ddphotos-parity` job). The DOM is the next entry. See SPEC-164's `## Frame`.
  **Ruled 2026-11-27:** all six frame recommendations accepted (split; `MIN_CROP = 64`; swap twice
  stays explicit; crop mode updates only the size line; the check in the advisory job; README crop
  rows with the DOM entry). See SPEC-164's "Maintainer rulings".
  **Designed 2026-11-27:** six functions with a `{ crop, preset, drag }` reducer, 153 Node rows, a
  headless check H1–H8 with a shared `scripts/lib/chrome.mjs`. Finding: `MIN_CROP = 64` still lets
  thin resized strips refuse a watermark (69 of 1,260 runs); put to the maintainer, ruling kept.
  **Ruled on the design (2026-11-27):** keep `MIN_CROP = 64`; the watermark-fit entry takes the
  width-plus-margin case and lands before the DOM entry.
- [ ] (not yet written) — [S] **The watermark's fit on a small or thin crop.** `watermarkSize`
  (`integrations/ddphotos/edit/recipe.js`) caps the mark to 90 % of the width and a tenth of the short
  side, but not to the engine's 5 % coverage limit, and the margin (`round(outLong × 0.02)`) is never
  checked against the short side. Without a crop the outputs are at least 400 px and neither was
  reached; a dragged box reaches both. Driven at SPEC-164's frame with native `apply` on the 1600×1071
  fixture: "© 2026 Jane Doe" at Medium is refused on square boxes of 1–13, 20–25, 30–35, 40, 41 and
  50 px (`the mark would cover 5.08 % of the 50×50 px image … the limit is 5 %`), and a 1600×20 strip
  at Large is refused on its 32 px margin. None above 50 px on the sweeps run (measured, not a bound).
  SPEC-164 recommends a 64 px minimum box on the page as a stopgap; this entry makes the page predict
  the engine instead. Parity rows for each refusal kind, with literals.
  **Found at SPEC-164's design (2026-11-27): a third refusal kind.** A thin *vertical* strip resized
  below its long edge refuses on the mark's width plus one margin against the output's width, even
  at a 64 px box: `P 64x1600 @400 -> 16x400` gives `the 23×2 px mark plus its 8 px margin does not
  fit inside the 16×400 px image` (69 of 1,260 runs refused at 64; 122 the smallest clean short side
  over that sweep; a 120-character text still refused at 192). No copy-pixel minimum covers every
  text, because the output is resized after the crop, so this entry must predict it.
  **Ruled 2026-11-27:** `MIN_CROP` stays 64 (a usability floor, not a watermark guard), and this
  entry lands **before** the crop mode's DOM entry, the first change that lets a user draw such a box.
- [ ] (not yet written) — [M] **Crop on the page: the crop mode's DOM.** The second half of the entry
  above, split at SPEC-164's frame. The Crop button and the box over the oriented base (`meta.bytes`,
  shown at copy size and scaled by CSS), the preset, Swap, Reset and Done buttons, Pointer Events with
  `touch-action: none` on the box and handles, arrow keys and Alt+arrow, ARIA (`role="group"` box,
  `aria-pressed` presets, a polite `role="status"` on settle), the live size line and the live
  "Made smaller to fit" note, all calling SPEC-164's functions with no logic of their own. The
  README's Crop row and the reworded "No crop …" limit. Its rows go on SPEC-164's headless check:
  drags (pointer and touch), keys, the render count across a drag and Done, and the drag mutants
  (ratio not applied, ratio from the finished preview, per-move rounding, a drag that renders, Done
  not clearing the pointer state). ⚠ Alt+Left/Right are Back/Forward in Chrome on Windows and Linux:
  design checks the page can keep them. After SPEC-164 and the watermark-fit entry above.
  Found at SPEC-164's verify: a Swap or a key press during a drag leaves `drag.start` stale, so the DOM
  entry decides whether they end the drag or are ignored mid-drag.
- [ ] (not yet written) — [S] **Straighten on the page.** An angle control emitting a
  `rotate` step in `crop-to-fit` mode, ordered before `crop` so a straighten-then-crop is the
  natural flow. Parity rows within STAGE-059's stated bound. Waits for STAGE-059.
- [ ] (not yet written) — [S] **Test-bed round and README.** Rerun the test bed's browser
  checks on all three servers with the new controls. Update the README and the CLI panel's
  wording. Record the wasm size delta.
- [x] (folded into SPEC-161) — [S] **Label the parity harness's table checks.** The three P5 checks
  (rows 4, 8, 12) all print `P5:`, so the closing count line shows `P5, P5, P5` and cannot say
  which recipe text moved. A comment in `tests/ddphotos_parity.mjs` also says "P1-P5" though there
  is no P1. Give each check its own label and fix the comment; no behaviour change. Found by
  SPEC-142's re-verify. Do it alongside the crop spec's first harness rows, not on its own.
- [ ] (not yet written) — [S] **Gate the slim wasm bundle's size.** The page ships the slim build
  (`--no-default-features --features avif`, `pkg-slim`), but the CI size gate
  (`scripts/lib/wasm-artifact.mjs`, used by `demo-assemble.mjs` and `npm_smoke.mjs`) measures only
  the full build. `just wasm-size` on the slim build reports (raw 4,147,159 B, brotli 833,239 B at
  SPEC-161's frame) but asserts nothing, so this stage's "stays inside its size gate" criterion has
  no gate for the bundle the page actually loads. Add a slim baseline and tolerance, or say in the
  Success Criteria that the full build's gate stands in for it. Found at SPEC-161's frame.

**Count:** 5 resolved (4 shipped specs, 1 folded) / 0 active / 5 pending — re-derived 2026-11-27 at SPEC-164's ship, over the Spec Backlog section, with `/usr/bin/grep -c '^- \[x\] \*\*SPEC-'` (4), `/usr/bin/grep -c '^- \[x\] (folded'` (1), `/usr/bin/grep -c '^- \[ \] \*\*SPEC-'` (0) and `/usr/bin/grep -c '^- \[ \] (not yet written)'` (5)

## Design Notes

- **The page edits a preview, the CLI runs on the original.** This is why STAGE-058's unit
  DEC exists, and why the parity test replays at 4× size. A pixel crop that passes at 1× and
  fails at 4× is exactly the bug this stage must not ship.
- **Step order on the page is fixed:** `auto-orient` → `rotate` → `crop` → `resize` →
  `watermark` → (`optimize`). It's written once in `edit.js` and asserted by the parity
  harness.

## Dependencies

### Depends on
- STAGE-058's `crop` spec and unit DEC (for crop)
- STAGE-059 (for straighten)
- The ddphotos test bed (#227)

### Enables
- Roadmap row 6b: an editor whose edit is worth saving
- A first contact with Doug about an Edit button, with something concrete to show

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
