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
    - "A `crop` recipe step: a rectangle, an aspect ratio with a gravity, in fractions of the image (pixels come through `edit --crop`, which converts them)"
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

- [x] **SPEC-141** (merged 2026-10-31, PR #233) — [S] **The no-bytes-changed guard, first.** A golden test that runs
  the repo's existing recipe corpus (bundled recipes, `docs/recipes.md` examples, the build
  fixtures) and pins output hashes, so every later spec in this project can prove it moved
  nothing. ⚡ Check `scripts/cli-golden.py` first: it may already be most of this, and the spec
  then shrinks to "point it at the recipe corpus and gate on it". It's cheap, and it makes
  PROJ-017's fourth success signal checkable instead of claimed.
- [x] **SPEC-145** (merged 2026-11-05, PR #244) — [M] **`crop` as a recipe step, with size-independent units.**
  Keys for a rectangle (`x`, `y`, `width`, `height`) or an aspect (`aspect = "16:9"` plus
  `gravity`). A recipe carries every length as a fraction of the image (DEC-112); pixels come
  through `edit --crop`, which converts them to fractions. Refuses a zero-area or out-of-bounds rectangle at load where it
  can, and at apply where it needs the image size. ⚠ **Writes the unit DEC.** The ddphotos
  page edits a downscaled preview and the CLI replays on the original, so a pixel crop from the
  page is wrong by construction. The DEC says which units a recipe may carry and how a
  fraction rounds to pixels. Reuse `Gravity` (`src/operation/mod.rs`) rather than adding a
  second anchor type.
- [x] **SPEC-147** (merged 2026-11-06, PR #252) — [S] **`rotate` (quarter turns) and `flip`.** `rotate` with `angle`
  restricted to multiples of 90 in this stage. A non-multiple is refused with a message naming
  the any-angle stage, until STAGE-059 lifts it on the same op and key table. `flip` with
  `axis = "horizontal" | "vertical"`. Exact: `image::imageops` rotate90/180/270 and flips, at
  the source's colour type. ⚠ Decide and test what happens to an EXIF orientation tag on an
  image that was not auto-oriented first. SPEC-110 bakes orientation at the start of `edit`;
  a hand-written recipe may not.
- [x] **SPEC-151** (merged 2026-11-06, PR #255) — [S] **`pad`.** To an exact size or to an aspect, anchored by
  `gravity`, filled with `color` (a value the `watermark` colour parser already reads) or
  `transparent`. ⚠ `transparent` into a format with no alpha (JPEG) is either refused or
  flattened against a stated colour. That is decided here, not left to the encoder.
- [x] **SPEC-152** (merged 2026-11-10, PR #258) — [S] **`trim`.** Removes a uniform border: the colour is the
  corner pixel or a stated `color`, matched within a stated `tolerance` per channel. Refuses
  (rather than producing a 0×0 image) when the whole image matches. Deterministic, with no
  content detection, and its doc entry says so. ⚠ Test on a 16-bit source with an 8-bit
  tolerance, so the tolerance's scale is pinned.
- [x] **SPEC-153** (merged 2026-11-10, PR #261) — [M] **`edit` flags for the five, and the ordering rule.**
  `--crop`, `--rotate`, `--flip`, `--pad`, `--trim`. ⚠ **Writes the ordering DEC.** `edit`
  applies flags in a fixed canonical order today (`auto-orient` → `resize` → `invert`,
  `src/cli/mod.rs`'s `Edit` doc), and crop-then-rotate is not rotate-then-crop. Either geometry
  slots into the canonical order at a stated position, or `edit` honours flag order for
  geometry. Design picks one and writes it down. `--save-recipe` must write steps that `apply`
  replays to the same pixels: that is the test.
  **Framed as a split (2026-11-10):** SPEC-153 takes the ordering DEC (recommended: typed order for
  every op flag, `auto-orient` always first, no repeats), `--crop` (pixels `WxH+X+Y` and `W:H` with
  `--crop-gravity`), `--rotate` and `--flip`, [M]. `--pad` and `--trim` are the entry below.
- [ ] (not yet written) — [S] **`edit --pad` and `--trim`, on SPEC-153's ordering rule.** Split out of
  SPEC-153 at its frame (2026-11-10); after SPEC-153. `--pad N|W:H` (N whole pixels of the long edge of
  the buffer the step receives, saved as a `border` fraction) with a required `--pad-color` and an
  optional `--pad-gravity` (aspect form only). `--trim` is a plain switch with optional `--trim-color`
  and `--trim-tolerance`, written only when given. Not an optional-value `--trim[=COLOR]`: clap then
  swallows the input path (driven). `trim`'s whole-match message names `'color'`/`'tolerance'`, so
  `edit` words it with the flag names. Consider folding the `parse_color` sign entry, since this adds
  two CLI routes into `parse_color`. **This entry closes STAGE-058.** SPEC-153's Frame records the
  syntax, the sizing and the replay matrix.
  Also reword the stale "single image" text, since `edit` takes one image or a batch (SPEC-153 design
  correction 1; a pixel `--crop` with `--save-recipe` needs one input): `src/cli/mod.rs:453` (the `--help`
  doc comment: "One-shot multi-op on a single image; optionally saves the recipe."),
  `docs/api-contract.md:536` ("One-shot multi-op on a single image — the \"experiment like an editor\"
  mode.") and `docs/cli-reference.md:301` ("Chain an ordered op list on a **single** image in one
  decode → ops → encode pass."). Found at SPEC-153's verify; no other STAGE-058 or `questions.yaml` entry covers it.

- [x] (folded into SPEC-145; shipped in SPEC-145) — [S] **`docs/recipes.md` still advertises a `crustyimg crop` verb.**
  Found at SPEC-141's frame. `docs/recipes.md:60-65` lists "Crop to an arbitrary rect / gravity
  / aspect **[planned — PROJ-006]**" as `crustyimg crop …`, and smart crop the same way. The verb
  set is frozen at 14 (STAGE-030), and PROJ-017 delivers crop as a recipe step plus an `edit`
  flag. Fold it into the `crop` spec's docs entries, or fix it on its own if `crop` slips.
- [ ] (not yet written) — [S] **Recipe-output guard: report the file-set and line-count failures together.**
  Found at SPEC-141's ship. `every_golden_line_is_checked_and_no_golden_is_stray` stops at the
  file-set assertion, so a change that also breaks the golden line count (`GOLDEN_LINES`) reports
  only the first (M4 in the spec's Build Completion). Test-only; optional, and only worth doing if
  a later spec trips both at once.

- [x] (folded into SPEC-151; shipped in SPEC-151, PR #255) — [S] **SPEC-147's verify cleanups that need a PR: a stricter `orientation_of`,
  and the corpus README's stale literals.** Found at SPEC-147's ship (verify's optional cleanup 1
  and its out-of-spec finding); fold into the next STAGE-058 spec's build. Two test-side fixes, no
  product change. (1) `orientation_of` in `tests/rotate_flip.rs` (:592) uses `.ok()?`, so any
  kamadak-exif error reads as "no tag"; a malformed `eXIf` chunk from a future encoder path would
  pass. Only `Err(exif::Error::NotFound(_))` should mean "no tag"; any other error should fail the
  test. (2) The "Adding a recipe" table in `tests/fixtures/recipe_corpus/README.md` quotes old
  literals and no test reads it: `14 → 15` and "all 14" (now 18), `ROWS (48)` (now 60),
  `GOLDEN_LINES (51)` (now 63), "the `42`" (now 54), and the `14` in `pairs.len()` (now 18).
  Cut the table's numbers to "see `recipe_corpus_output.rs`" where the file is the source, or have
  a test read them. This is a different gap from the entry above (that one is about reporting two
  failures together), so it is a separate entry.

- [ ] (not yet written) — [S] **`crop`'s aspect can follow each image's orientation.** Filed by the
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

- [ ] (not yet written) — [S] **The batch summary says "watermark skipped" for any refusing op.**
  Found at SPEC-145's verify. `report_skipped_images` (`src/cli/common.rs:40`) and `build.rs:864`
  print "watermark skipped on K of N images" for every step that refuses through the refusal hook.
  Only `watermark` uses the hook today, so nothing shipped is mislabelled. `pad` or `trim` may be
  the first other op to use it: name the op that refused, before either ships.
- [x] (folded into SPEC-151; shipped in SPEC-151, PR #255) — [S] **State when an op uses the refusal hook and when it fails at apply.**
  Found at SPEC-145's ship (mutation M4b). A refusal through `Operation::refusal` is skipped per
  image in a batch: with the 0 px check moved there, one input exits 2 but two inputs write both
  images uncropped and exit 0. An apply failure is not skipped. So the hook suits only a condition
  that is safe to skip (`watermark` too large for the image), and a condition that means the
  output is wrong (`crop` rounding to 0 px) must fail at apply. Today that rule sits in DEC-112
  and SPEC-145 only. Write it where an op author chooses: the `Operation::refusal` doc comment
  (`src/operation/mod.rs`) and the line that points a new op at it. Doc-comment-only, plus a test
  only if the wording is made checkable. Distinct from the entry above, which is about the
  batch message's text. Before `pad` or `trim` ships.

- [ ] (not yet written) — [S] **`apply` and `build` warn when a recipe with no `auto-orient` step meets a tagged input.**
  Found at SPEC-147's frame. The pixel lane writes no EXIF (DEC-003), so a recipe that doesn't start
  with `auto-orient` drops the tag without applying it: an `invert` recipe on a 60×40 PNG tagged
  Orientation 6 wrote the stored 60×40 pixels with no tag, so a portrait displays sideways, exit 0.
  Every op does this today, `crop` included. With `rotate` it reads as "nothing happened": `rotate
  angle = 90` on that input gives exactly the pixels the source already displayed as (0 px against
  `auto-orient`). SPEC-110 fixed the verbs and scoped `apply` out; `edit --save-recipe` already
  records `auto-orient`. Design decides warn vs refuse vs stay silent, and whether a geometry step
  alone triggers it. A warning changes no output bytes. After SPEC-147.

- [ ] (not yet written) — [S] **`pad` with a transparent fill, and what `transparent` does in a format with no alpha.**
  Split out of SPEC-151 at its frame (2026-11-06); after SPEC-151. Lifts SPEC-151's "opaque only"
  refusal so `color` takes `transparent` (and `#RRGGBBAA` below `ff`), widening RGB to RGBA and grey
  to grey-with-alpha (DEC-095's narrow rule already keeps a channel with non-opaque samples). ⚠ The
  JPEG rule is decided here, not left to the encoder. Driven at SPEC-151's frame on 0.8.2: JPEG output
  drops alpha and keeps the colour under it on every path (`-o`, `quality`, batch), exit 0, no warning
  (transparent red came out `srgb(254,0,0)`), so `#00000000` would give a black border. The format is
  known per input after decode and before the pipeline runs (`output_format_for`, `src/cli/ops.rs`),
  so a refusal can live there, in `apply` (one input and a batch), `build` and wasm `transform`.
  **Ruled 2026-11-06 (SPEC-151's Q1 and Q4): split, and refuse.** `transparent` into a format with
  no alpha is refused per input, exit 4, naming the step and the format. Drive GIF (1-bit
  transparency) too.

- [ ] (not yet written) — [S] **Pin `watermark`'s nine gravities, with a margin.** Found at SPEC-151's
  design (mutation M2): with `north` placed as `south` in `Gravity::placement`
  (`src/operation/mod.rs`), every `watermark` test stays green. `crop` and `pad` pin all nine at
  margin 0, but the margin path (`x_left`/`x_right`/`y_top`/`y_bottom` inset by `margin`) has no
  test that names an absolute offset. Add one row per gravity with a non-zero margin, expected
  offsets as literals, and drive the M2 swap plus a margin-dropped mutation. Test-only, no product
  change. Any time; pairs naturally with `trim` (line 6), which may place by gravity too.

- [x] (folded into SPEC-152; shipped in SPEC-152, PR #258, AC-54) — [S] **Pin `resize`'s cap message text.** Found at SPEC-151's verify (its
  optional cleanup), filed at SPEC-151's ship; fold into the next STAGE-058 spec's build. No test pins
  the text of `resize`'s two cap errors: `resize_oversize_is_typed_error`,
  `resize_apply_percent_rejects_oversized_output` and `apply_recipe_with_oversized_resize_exits_1`
  match only the error kind or exit 1. SPEC-151 hoisted `MAX_EDGE` and `MAX_AREA` to module-level
  constants shared with `pad`, so a `contains` on both lines guards the hoist. The binary prints, on
  `main` today (`resize --exact 60000x10` and `resize --exact 20000x20000` on a 4×4 PNG, both exit 1):
  `error: operation 'resize' failed: target dimensions 60000×10 exceed the maximum edge limit of 50000 px`
  and `error: operation 'resize' failed: target area 20000×20000 = 400000000 px exceeds the maximum area of 134217728 px`.
  Test-only, no product change.

- [ ] (not yet written) — [S] **kind: claim. `parse_color` accepts a sign, so `+f+f+f` is read as `#0f0f0f`.**
  Found at SPEC-152's verify; driven again at ship on a release build of `main` (0.8.2, scratch
  `CARGO_TARGET_DIR`, deleted after). `u8::from_str_radix` allows a leading `+`, so each two-character
  pair `+f` parses as 15, while the message says the value "must be a hex colour RRGGBB, #RRGGBB or
  RRGGBBAA". Verify drove it in `pad`, `watermark` and `trim`; ship drove `pad` and `trim`, with a 4×4 PNG
  of (200, 100, 50) padded by `border = 0.25`:
  ```
  pad  color = "#0f0f0f"  -> exit 0, top-left pixel (15, 15, 15)
  pad  color = "+f+f+f"   -> exit 0, top-left pixel (15, 15, 15)
  pad  color = "zzzzzz"   -> exit 1, recipe step 0 ('pad'): 'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "zzzzzz"
  trim color = "+f+f+f", tolerance = 0, on a 4×4 image with a #0f0f0f border and a red centre
                          -> exit 0, written 2×2 (the border was read as #0f0f0f)
  ```
  (`crustyimg apply --recipe r.toml -o out.png in.png`, recipe `version = "2"`.) The reading is harmless
  to a hand-written recipe that means a colour, but it lets a typo through as a different colour with
  exit 0, and the same parser backs the `watermark --color` flag. Proposed fix: refuse any byte that
  is not an ASCII hex digit, in one place, `parse_color` (`src/text/mod.rs`). That also covers the
  non-ASCII case SPEC-152 fixed, so the explicit non-ASCII check can go. Test: a refusal row for `+f+f+f` in `pad`, `trim` and `watermark`, and a mutation that puts
  `from_str_radix` back on the raw pair; the refusal message stays as it is.
  Search first found no entry in STAGE-058 or `guidance/questions.yaml` that covers it.

**Count:** 10 shipped / 0 active / 8 pending — re-derived 2026-11-10 at SPEC-153's ship, with `/usr/bin/grep -c '^- \[x\]'` (10: 6 `**SPEC-` entries and 4 folded), `/usr/bin/grep -c '^- \[ \]'` (8), `/usr/bin/grep -c '^- \[ \] \*\*SPEC-'` (0 active), `/usr/bin/grep -c '^- \[ \] (not yet written)'` (8) and `/usr/bin/grep -c '^- \[~\]'` (0)

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
