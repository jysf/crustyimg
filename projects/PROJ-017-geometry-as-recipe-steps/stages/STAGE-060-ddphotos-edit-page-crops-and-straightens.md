---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-060
  status: proposed
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
- [ ] (not yet written) — [M] **Crop on the page.** Free-form and aspect-preset crop over
  the preview, emitting a fractional `crop` step ahead of `resize` in both `pixelRecipe` and
  `cliRecipe` (`integrations/ddphotos/edit/edit.js`). Parity rows for each preset, plus the
  4× replay. ⚠ The page's existing "Made smaller to fit this photo" watermark logic runs after
  the crop. Check that a crop which shrinks the image re-triggers it.
  ⚠ **The page takes the photo's size from wasm `info()`, which reports the stored size.** For
  an EXIF-rotated JPEG that is 1600×1071 when the output is 1071×1600, so `planEdit` predicts
  landscape and sizes the watermark for it. Crop coordinates must live in the space `auto-orient`
  produces. This is a page workaround and belongs to this spec, not an `info()` bug: native
  `crustyimg info` reports the stored size too, and wasm `info()` is its twin. Found in SPEC-142's
  design; rows 15–16 of its harness pin the transposed prediction. Harmless today, because
  photogen publishes WebP copies that are already oriented.
- [ ] (not yet written) — [S] **Straighten on the page.** An angle control emitting a
  `rotate` step in `crop-to-fit` mode, ordered before `crop` so a straighten-then-crop is the
  natural flow. Parity rows within STAGE-059's stated bound. Waits for STAGE-059.
- [ ] (not yet written) — [S] **Test-bed round and README.** Rerun the test bed's browser
  checks on all three servers with the new controls. Update the README and the CLI panel's
  wording. Record the wasm size delta.
- [ ] (not yet written) — [S] **Label the parity harness's table checks.** The three P5 checks
  (rows 4, 8, 12) all print `P5:`, so the closing count line shows `P5, P5, P5` and cannot say
  which recipe text moved. A comment in `tests/ddphotos_parity.mjs` also says "P1-P5" though there
  is no P1. Give each check its own label and fix the comment; no behaviour change. Found by
  SPEC-142's re-verify. Do it alongside the crop spec's first harness rows, not on its own.

**Count:** 1 shipped / 0 active / 4 pending

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
