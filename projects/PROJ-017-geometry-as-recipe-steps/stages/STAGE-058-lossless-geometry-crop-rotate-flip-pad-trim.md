---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-058
  status: proposed
  priority: high
  target_complete: null

project:
  id: PROJ-017
repo:
  id: crustyimg

created_at: 2026-10-30
shipped_at: null

value_contribution:
  advances: >
    The core of PROJ-017's thesis: the geometry edits that move pixels without
    resampling them (crop, quarter turns, flip, pad, trim) become recipe steps
    and `edit` flags, so every one is expressible, saveable and replayable.
  delivers:
    - "A `crop` recipe step: a rectangle, an aspect ratio with a gravity, in pixels or in fractions of the image"
    - "A `rotate` step for quarter turns (90/180/270) and a `flip` step (horizontal/vertical), exact in pixels"
    - "A `pad` step: to a size or an aspect, anchored by gravity, filled with a colour or transparent"
    - "A `trim` step: remove a uniform border within a stated tolerance"
    - "`edit` flags for all five, with a written ordering rule, and `--save-recipe` that writes them"
  explicitly_does_not:
    - "Resample. Any-angle rotate is STAGE-059"
    - "Detect content. `trim` compares border pixels to a colour and nothing else"
    - "Change the output of any existing recipe"
---

# STAGE-058: Lossless geometry: crop, quarter-turn rotate, flip, pad, trim

## What This Stage Is

When this stage ships, a recipe can say `op = "crop"`, `"rotate"`, `"flip"`, `"pad"` or
`"trim"`, `crustyimg edit` can do the same from flags, and none of the five resample: every
output pixel is an input pixel or a fill colour. That property makes them exact, which makes
their tests exact.

## Why Now

It's the first stage of PROJ-017 and depends on nothing unshipped. The registry seam
(SPEC-131's typed parameter structs, DEC-102's load-time validation) is in place, and
`Gravity` already exists in `src/operation/mod.rs` from `watermark`. The ddphotos edit page's
crop control (STAGE-060) waits only for this stage's crop spec.

## Success Criteria

- Each op is registered in `OperationRegistry::with_builtins` with a key table. A bad key or
  value is refused at recipe load, naming the step.
- A rotate-90 applied four times, and a flip applied twice, returns the input's pixels
  exactly, for 8-bit and 16-bit, with and without alpha.
- A fractional crop recipe produces the same relative region on a 1200px and a 6000px copy
  of the same image (within one pixel of rounding, the rounding rule stated).
- `edit` accepts each op as a flag, applies them in the documented order, and
  `--save-recipe` writes steps that `apply` replays to identical pixels.
- The existing recipe corpus gives byte-identical output before and after the stage.

## Scope

### In scope
- The five ops, their key tables, their `Operation` impls, and their docs entries
- The size-independent unit rule (a DEC)
- The `edit` ordering rule for geometry flags (a DEC)
- Colour type and bit depth preserved per DEC-095

### Explicitly out of scope
- Any-angle rotate (STAGE-059)
- The ddphotos page (STAGE-060)
- Content-aware trim or crop

## Spec Backlog

In build order. Each is framed in its own session first.

- [ ] **SPEC-141** (verified with a punch list 2026-10-30, PR #233) — [S] **The no-bytes-changed guard, first.** A golden test that runs
  the repo's existing recipe corpus (bundled recipes, `docs/recipes.md` examples, the build
  fixtures) and pins output hashes, so every later spec in this project can prove it moved
  nothing. ⚡ Check `scripts/cli-golden.py` first: it may already be most of this, and the spec
  then shrinks to "point it at the recipe corpus and gate on it". It's cheap, and it makes
  PROJ-017's fourth success signal checkable instead of claimed.
- [ ] (not yet written) — [M] **`crop` as a recipe step, with size-independent units.**
  Keys for a rectangle (`x`, `y`, `width`, `height`) or an aspect (`aspect = "16:9"` plus
  `gravity`). Every length is accepted in pixels or as a fraction of the image (the unit
  syntax is the design call). Refuses a zero-area or out-of-bounds rectangle at load where it
  can, and at apply where it needs the image size. ⚠ **Writes the unit DEC.** The ddphotos
  page edits a downscaled preview and the CLI replays on the original, so a pixel crop from the
  page is wrong by construction. The DEC says which units a recipe may carry and how a
  fraction rounds to pixels. Reuse `Gravity` (`src/operation/mod.rs`) rather than adding a
  second anchor type.
- [ ] (not yet written) — [S] **`rotate` (quarter turns) and `flip`.** `rotate` with `angle`
  restricted to multiples of 90 in this stage. A non-multiple is refused with a message naming
  the any-angle stage, until STAGE-059 lifts it on the same op and key table. `flip` with
  `axis = "horizontal" | "vertical"`. Exact: `image::imageops` rotate90/180/270 and flips, at
  the source's colour type. ⚠ Decide and test what happens to an EXIF orientation tag on an
  image that was not auto-oriented first. SPEC-110 bakes orientation at the start of `edit`;
  a hand-written recipe may not.
- [ ] (not yet written) — [S] **`pad`.** To an exact size or to an aspect, anchored by
  `gravity`, filled with `color` (a value the `watermark` colour parser already reads) or
  `transparent`. ⚠ `transparent` into a format with no alpha (JPEG) is either refused or
  flattened against a stated colour. That is decided here, not left to the encoder.
- [ ] (not yet written) — [S] **`trim`.** Removes a uniform border: the colour is the
  corner pixel or a stated `color`, matched within a stated `tolerance` per channel. Refuses
  (rather than producing a 0×0 image) when the whole image matches. Deterministic, with no
  content detection, and its doc entry says so. ⚠ Test on a 16-bit source with an 8-bit
  tolerance, so the tolerance's scale is pinned.
- [ ] (not yet written) — [S] **`edit` flags for the five, and the ordering rule.**
  `--crop`, `--rotate`, `--flip`, `--pad`, `--trim`. ⚠ **Writes the ordering DEC.** `edit`
  applies flags in a fixed canonical order today (`auto-orient` → `resize` → `invert`,
  `src/cli/mod.rs`'s `Edit` doc), and crop-then-rotate is not rotate-then-crop. Either geometry
  slots into the canonical order at a stated position, or `edit` honours flag order for
  geometry. Design picks one and writes it down. `--save-recipe` must write steps that `apply`
  replays to the same pixels: that is the test.

- [ ] (not yet written) — [S] **`docs/recipes.md` still advertises a `crustyimg crop` verb.**
  Found at SPEC-141's frame. `docs/recipes.md:60-65` lists "Crop to an arbitrary rect / gravity
  / aspect **[planned — PROJ-006]**" as `crustyimg crop …`, and smart crop the same way. The verb
  set is frozen at 14 (STAGE-030), and PROJ-017 delivers crop as a recipe step plus an `edit`
  flag. Fold it into the `crop` spec's docs entries, or fix it on its own if `crop` slips.

**Count:** 0 shipped / 0 active / 7 pending — re-derived 2026-10-30 with `grep -c '^- \[ \]'` (7) and `grep -c '^- \[x\]'` (0); SPEC-141 is framed, not active

## Design Notes

- **The units rule is load-bearing for the whole project.** If a recipe can only carry
  pixels, STAGE-060 cannot work. If it can only carry fractions, the CLI loses
  `--crop 800x600+10+10`. Carry both, and make the page emit fractions.
- **Every op preserves colour type and bit depth** (DEC-095): widen to work, narrow to write.
  These ops never widen, since they move pixels and don't compute them, so this is cheap. It
  still needs a 16-bit and an alpha test per op, because the precedent was paid for once.
- **No resampling in this stage, on purpose.** It keeps every test exact, and keeps this stage
  out of the working-type performance question PROJ-015 owns.
- **Metadata after geometry.** Crop and rotate change the dimensions; check what EXIF
  `PixelXDimension`/`PixelYDimension` and orientation say afterwards, and that `meta` lane
  behaviour (C2PA credentials dropped on the pixel lane) is unchanged.

## Dependencies

### Depends on
- PROJ-011 / STAGE-050, SPEC-131 (shipped): the registry and typed parameter structs
- DEC-095, DEC-102 (shipped)
- SPEC-110 (shipped): `edit` bakes orientation first

### Enables
- STAGE-059: `rotate` exists, and any-angle extends it
- STAGE-060: crop for the ddphotos page

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
