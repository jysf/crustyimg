---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-161
  type: story                      # epic | story | task | bug | chore
  cycle: frame  # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-017
  stage: STAGE-060
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: claude-opus-5-5     # usually same Claude, different session
  created_at: 2026-11-22

references:
  decisions:
    - DEC-110
    - DEC-112
    - DEC-113
  constraints: []
  related_specs:
    - SPEC-142
    - SPEC-145

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-060's <capability>". Optional; null is acceptable.
value_link: "The recipe half of STAGE-060's \"crop on the ddphotos edit page\", and its \"recipe from the page that the CLI replays on the full-size original\" proven by a 4× replay"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-161: crop on the ddphotos edit page: the recipe model and the 4x replay

## Context

STAGE-060 is the proof half of PROJ-017's thesis: an edit made on the ddphotos edit page, which
can't run the CLI, is handed to the CLI as a recipe and reproduces on the full-size original.
SPEC-142 built the native-vs-wasm parity harness on today's resize and watermark recipes, and
STAGE-058 shipped `crop` as a recipe step in fractions of the image (DEC-112, SPEC-145). Crop is
the first geometry the page emits, so it's the first real test of "the recipe means the same
thing at 4× the size". This spec is the first half of the STAGE-060 backlog entry "Crop on the
page", split at frame (see the Frame's verdict). The second half, the crop mode a person drags in,
is filed right after this one in STAGE-060's backlog.

Today a photo owner on the edit page can resize and watermark a copy, but can't crop it. For an
EXIF-rotated JPEG, the page also plans the watermark for the stored (sideways) size, because wasm
`info()` reports the stored size.

## Goal

The page's recipe code (`recipe.js`) and its worker can express a crop, a box in fractions of the
oriented image, placed ahead of `resize` in both `pixelRecipe` and `cliRecipe`, and they plan the
output and the watermark from the oriented, cropped size. The parity harness proves those recipes
match native-vs-wasm on the copy, and replays `cliRecipe` on a 4× original with the same framing
and the same output size.

## Frame

*Framed 2026-11-22 on `main` at `7d0e7fd` + the SPEC-161 reservation (`b08b241`), Opus.*

### Verdict: SPLIT, then GO on this half at [M]

The whole backlog entry is more than [M] once the verification is counted: about 25 new
native-vs-wasm rows, a new 4× replay mode with its own bound, two new fixtures, a new negative
control, the orientation fix, and a drag UI that Node can't test. Split it:
- **SPEC-161 (this spec), [M]: the recipe model and the 4× replay.** Everything Node can test. It
  ships on its own: the page plans EXIF-rotated photos from their oriented size (a user-visible fix
  to the watermark fit), and CI proves a crop recipe from `recipe.js` replays on a 4× original.
  The page shows no crop control yet.
- **Second entry, [M]: "Crop on the page: the crop mode".** The crop mode, the box, presets as
  buttons, keyboard and touch, and the CLI panel's wording. It's DOM-only and calls this spec's
  `recipe.js` functions. Its browser check is the existing "Test-bed round and README" entry.

### Premise check (evidence)

Run on a fresh release build of `main` and a fresh slim wasm (both built by `just ddphotos-parity`
into `target/ddphotos-parity/`, deleted at the end):
- `just ddphotos-parity`: **16 rows, 0 row failures, 0 table failures, K=1 (cap 78)**; the
  largest differing-pixel count on a resampling row is 1; the alpha rows pass; the self-test
  passes B1–B10, and N1 and N2 are red on exactly their expected rows.
- Crop runs through both builds, byte-identical. `auto-orient` + `crop` (box `x=0.1 y=0.2
  width=0.5 height=0.6`, and `aspect="3:2" gravity="northwest"`) on a JPEG made with
  ImageMagick from the fixture: wasm `transform` → PNG and native `apply` → PNG, `magick compare
  -metric AE` = 0 and `cmp` equal on all four outputs (plain and EXIF-rotated input × box and
  aspect).
- Crop isn't feature-gated. `crop.rs` and its `register_with_params("crop", …)` line in
  `registry.rs` carry no `#[cfg(feature)]`, so the slim build already has it.
- **Two things the frame prompt treats as existing don't exist yet.** SPEC-142's harness has **no
  4× replay**, and **no crop negative control** (its self-test mutates `opacity` and `width`).
  SPEC-142 deferred both to this spec ("(b) replay parity at 4×: STAGE-060's crop spec"; "two
  recipes that differ by one pixel of crop waits for crop"). They're in scope here.

### The interaction (for the second entry; recorded here so design of both halves agrees)

Recommend a **crop mode**:
- A "Crop" button switches the preview to the **oriented base image**, the whole photo after
  `auto-orient` only, with a DOM box over it (shaded outside, corner and edge handles).
- "Done" switches back to the finished preview and schedules **one** `runPreview`.
- While the box moves, wasm renders nothing. The box is a DOM element in the image's natural
  pixel space, scaled by CSS. Dragging never queues a render, so the 200 ms debounce isn't
  involved.
- What wasm renders, and when:
  - **The oriented base:** once, at load. It's `transform(source, auto-orient, "jpeg")` at the
    copy's full size, not resized for the screen: the browser scales it, and box pixels then
    equal copy pixels with no second rounding. Driven in Node: **31–34 ms** for 1600×1071. When
    "Straighten on the page" lands, the base becomes `auto-orient` + `rotate`, re-rendered on an
    angle change under the same debounce.
  - **The finished preview:** on leaving crop mode, and on any form change outside crop mode, as
    today.
- Minimum for keyboard and touch:
  - Pointer Events (one path for mouse, touch and pen), `touch-action: none` on the box, and
    handles with a hit area of at least 44 px.
  - The box is focusable. Arrow keys move it 1 copy-pixel (Shift: 10), and Alt+arrow resizes from
    the bottom-right corner.
  - Presets, Reset and Done are real `<button>`s. A live text line reads the box's size in pixels
    ("1067 × 711 of 1600 × 1071").

### Presets: emit a `box`, and follow the photo's orientation on the page

Recommend that **a preset emits a box** (`x y width height`), not `aspect` + `gravity`:
- The page computes the preset's box in copy pixels with a JS port of `aspect_size` (exact
  integer maths, centred like `Gravity::Center`) and emits `pixel / side` fractions. That's the
  box the user then drags, so the page has one crop form.
- **Driven:** `aspect` doesn't match its own page prediction at an exact 4× replay, and `box`
  does. `aspect_size` rounds the short side on each image separately:
  - 9:16 on the 1600×1071 copy is 602×1071 on the page, but 603×1071 from the CLI on 6400×4284
    (2409.75 → 2410, then scaled to 1071, gives 602.5 → 603).
  - 3:2 at 800 is 800×534 on the page and 800×533 from the CLI.
  - Over 7 ratios × 5 sizes × 9 copy/original pairs, `aspect` gave 1–7 mismatches per pair, 4 of
    them at exact 4×.
  - A pixel-snapped box gave **0 mismatches in 1,000,000 cases at each exact 4× pair**.
- Orientation: **page-side.** A preset follows the oriented photo (3:2 on a portrait photo
  becomes 2:3), with a swap button. Square photos and 1:1 stay as they are.
- So the recipe needs no orientation key, and the STAGE-060 entry "`crop`'s aspect can follow each
  image's orientation" loses its first user (see Fold or leave).
- Fractions are written at full `f64` precision, the shortest text that reads back. That's
  `String(n / side)` in JS, as DEC-114 rule 5 writes `edit --save-recipe`. **Driven:** for every
  box on sides 400, 900, 1067, 1071, 1200, 1600 and 2048 (**5,729,060 boxes**), the written
  fractions round back to the same four pixel edges under DEC-112's rule, and `x + width` never
  exceeds 1.

### Orientation, driven, and the page's fix

Driven on a JPEG made with ImageMagick from the fixture and tagged with `exiftool -n
-Orientation=6`:
- `mine_rot.jpg: wasm info 1600x1071 jpeg`, and native `crustyimg info` gives `dimensions:
  1600x1071`. Both report the stored size, as the backlog's ⚠ paragraph says.
- `auto-orient` + box `0.1/0.2/0.5/0.6` → **536×960** on both builds, so the crop ran in oriented
  (1071×1600) space. The unrotated JPEG gives 800×643.
- `auto-orient` + `aspect "3:2"` → 1071×714 on both builds.
- `transform(bytes, auto-orient, "jpeg")` then `info()` → **1071×1600**.

**Fix: use the size of the oriented base.** The worker's `load` renders the oriented base once,
for the crop mode, and reports `info()` of that base as the photo's size. `planEdit` and every
crop fraction use that size.
- No EXIF parser in JS (which would be another JPEG/WebP/TIFF reader to keep).
- No new wasm export: an `orientation` getter on `ImageInfo` would touch `src/wasm.rs`, which the
  handed-in `validate()` item also touches.
- `naturalWidth` of the displayed base is the same number, but the worker already holds the
  exact value.

What changes in the harness's rows 15–16:
- Their expected output sizes stay 803×1200 and 723×1080.
- `runTable`'s `rotated` transpose (`predicted = rotated ? [outH, outW] : …`) is deleted, and
  `planEdit` is fed the oriented size, obtained the way the worker obtains it. The page's
  prediction then equals the output as is.
- The watermark text of 15–16 doesn't move: for "© Jane Doe 2026" at Medium, both 1080×723 and
  723×1080 give size 43, uncapped (driven).
- So 15–16 alone can't tell the fix from the bug (HANDOFF trap 10). The new row R-refit (below)
  can.

### The size the page predicts after a crop: the bound, driven

There are two different comparisons:
1. **The page's prediction vs what wasm makes from the copy: exact.** `planEdit` ports
   `rect_edges` (or `aspect_size` for a preset's box) and `resize` mode `max`, all in `f64`.
   `Math.round` matches `f64::round` for non-negative values. The harness asserts it on every
   row.
2. **The page's output vs the CLI's output from the original.** The CLI computes the box on the
   original and then resizes to `outLong`. Driven in Node with a line-for-line port of
   `rect_edges` and `resize` mode `max`: random pixel-snapped boxes, × 5 sizes (as published,
   1200, 1080, 800, 400), with the copy made from the original by `resize` mode `max`.
   - Exact 4× (1600×1071 ↔ 6400×4284, its portrait, and a 1600 square): **0 of 1,000,000
     differ** in each.
   - 4032×3024 → 1200 copy: 1.20 % differ, worst 1 px.
   - 6000×4000 → 1600: 2.59 %, worst 1 px.
   - 6000×4016: 1.67 %, worst 1 px.
   - 5472×3648: 3.03 %, worst 1 px.
   - A 1700 px original (1.0625×): 25.9 %, **worst 2 px** (page 400×373, CLI 400×371).
   - Sweep over 4,000 random originals from 1601 to 9600 px long, 400 boxes each, by
     original/copy ratio: **worst 2 px below 2×, worst 1 px at 2× and above.**
   - In 38–575 cases per pair, the long edge flips on a near-square crop (868×830 vs 868×829
     style). It's still within the same 1 px.

   The ratio is the original's long edge over the published copy's.

**What the page shows:** its own output size, exactly, from `make`'s `info(edited)`, as today.
**What the parity test asserts:**
- Dimensions exactly equal on the integer-4× replay rows.
- On one non-integer replay row, each side within 1 px, with an original ≥ 2× the copy.
- The ±2 px case below 2× is recorded as a fact for the CLI panel, not asserted.

This is a new property, beyond DEC-110 (same input both sides), so **recommend DEC-116** (content
below).

Caveat carried from SPEC-142: ddphotos' real photogen resizer makes the copy, not crustyimg's
`resize`. If photogen rounds differently, the copy-vs-original rounding moves, and the non-integer
rule must be re-driven against it. The integer-4× rows are unaffected, because the harness makes
its own original.

### The watermark refit

`watermarkSize(text, outW, outH, margin, size)` reads only the output's sides. So the crop enters
`planEdit` **before** `longEdge`: oriented size → crop box in copy pixels (`cw × ch`) → `longEdge
= max(cw, ch)` → `outLong`, `outW`, `outH`, `margin` and the fit, as today. A crop re-triggers
"Made smaller to fit this photo" through `edit.watermark.capped` with no other change. Driven with
`recipe.js` as it is, feeding the cropped sides:
- **W-refit:** the 1600×1071 copy, box `x=0.375 width=0.25 y=0 height=1` → 400×1071, mark **40,
  capped**. Uncropped, it's 64, uncapped.
- **R-refit (the trap-10 row):** the EXIF-rotated fixture, box `x=0.325 width=0.35 y=0 height=1`.
  - Correct, in oriented space: 375×1600, mark **37, capped**.
  - With the stored-size bug, the same fractions give 560×1071, mark 43, uncapped.

  The fix and the bug differ in output size, watermark size and the `capped` flag.

### `cliRecipe` on the original

One crop, emitted for the 1600×1071 copy with a box of `0.1/0.2/0.5/0.6`, as published, with the
watermark at Medium:

```
version = "2"
format = "png"
[[step]] op = "auto-orient"
[[step]] op = "crop"     x = 0.1  y = 0.2  width = 0.5  height = 0.6
[[step]] op = "resize"   mode = "max"  width = 800
[[step]] op = "watermark" … size = 32  margin = 16 …
```

(Steps are shown one per line here; the page emits normal TOML.) Replayed natively: 800×643 from
the copy, and **800×643** from a 6400×4284 original (ImageMagick 4× upscale); `magick compare`
PSNR 64.8 dB.

The narrow W-refit crop: 400×1071 from the copy, and 400×1071 from 4×.

**A smaller original:** `resize` mode `max` never upscales, so the CLI's output is smaller than the
page's, with the watermark at the page's pixel size, proportionally bigger. Driven: the narrow
crop's `cliRecipe` on an 800×536 file is **refused**, `error: watermark refused: the 311×36 px mark
plus its 21 px margin does not fit inside the 200×536 px image`, exit 2, no file. A wider crop
would silently give a smaller output with a heavier mark.
- In the copy-from-original model this can't happen: the original's crop is never shorter than
  the copy's. Driven: 0 cases over all the pairs above.
- It happens only when the user hands the CLI a file smaller than the published copy.
- Recommend: no recipe change. The second entry's CLI panel says "run it on the original, or any
  file at least as large as this copy".

### Parity rows and the negative control (design finalises the count)

New fixtures, made outside crustyimg like the existing two (ImageMagick + exiftool, recipe in the
fixtures README):
- **P**, a stored-portrait 1071×1600 WebP;
- **S**, a square 1071×1071 WebP.

W and R exist. The 4× originals are made **at test time by Pillow** (already in the harness's
venv; independent of crustyimg), so nothing large is committed.

Native-vs-wasm rows (same input on both sides):
- **Each preset × each orientation:** 1:1, 4:5, 3:2, 16:9 × W, P, S, R, as `pixel/png` at "as
  published". There's no resize step, so they're **exact** (0 px, bytes equal). That's 16 rows.
- **A free-form box** off-centre (`0.1/0.2/0.5/0.6`) on W and on R, at 1080: crop + resize
  resamples, so **within DEC-110** (≤ 1/255 on ≤ K px).
- **An off-centre preset placement:** a 3:2 box pushed to the top-left edge, on W. Exact.
- **W-refit and R-refit**, as above, with the watermark on. Exact at "as published"; the
  assertion includes `capped`.
- **One `cli/png` row and one `preview/jpeg` row** of the free box. The `cli` row resamples. The
  `preview` row is exact if it has no resize, and JPEG bytes are asserted.
- **One `webp` row and one `avif` row** of the free box at 1080. Resamples.

Replay rows (new mode: wasm `pixelRecipe` on the copy vs native `cliRecipe` on the original):
- **The 4× replay of every crop row above:** dimensions **exact**, framing checked by a PSNR
  floor set from measurement (DEC-116). It isn't DEC-110's 1/255: the sources differ in
  resolution. On an ImageMagick-upscaled 4× the frame measured 64.8–65.9 dB. Design measures it
  on the Pillow-made original, and a real-detail original reads lower.
- **One non-integer replay** (an original ≥ 2× the copy, such as 3.75×): each side within 1 px,
  plus the PSNR floor.

Negative controls the self-test must keep or add:
- Keep **N1** and **N2**. Their expected-red lists grow with the rows.
- **N3 (new, required by SPEC-142): one pixel of crop.** Native-side mutation: the crop's
  `width` gets +1 copy pixel (the exact mutant edit, such as `width = 0.5\n` → `width =
  0.500625\n` on W, is quoted in design). It must turn red exactly the crop rows that carry it
  (`dims`).
- **N4 (new): a replay framing shift.** The original-side `x` moves by one copy pixel with the
  size kept, so the dimensions still match. It must turn the replay rows red **on the PSNR
  floor**. That proves the floor can see a 1 px shift. If it can't, the floor is too low, and
  design must say so before build.

Design checks each of these "must go red" claims against the measured numbers (AGENTS §15).

Which rows match how:
- **Exact:** every native-vs-wasm row with no resize step.
- **Within DEC-110:** native-vs-wasm rows that resize.
- **Within DEC-116:** replay rows. Dimensions are exact at 4× and within 1 px off-integer; pixels
  meet the PSNR floor.

### The wasm size

No Rust change, and crop is already linked into the slim build (above). **0 B of growth.**
- `just wasm-size` on the slim build (`--no-default-features --features avif`, `pkg-slim`):
  **raw 4,147,159 B (3.96 MB), gzip 1,195,240 B, brotli 833,239 B.**
- The size gate CI runs (`scripts/lib/wasm-artifact.mjs`, used by `demo-assemble.mjs` and
  `npm_smoke.mjs`) measures the full build, baseline 1,266,535 B brotli. It doesn't measure the
  slim bundle the page ships. Filed as an out-of-spec finding at the end of STAGE-060.
- The page's own JS grows: `recipe.js` gains the crop model. Design states the bytes.

### Fold or leave

- **"Label the parity harness's table checks": FOLD.** Its text says to do it with this spec's
  first harness rows. It's a relabel of `P5` × 3 plus one comment and doesn't change the size.
  Marked `[x] (folded into SPEC-161)`.
- **"`crop`'s aspect can follow each image's orientation": LEAVE** in STAGE-060 for now, and put
  the home to the maintainer.
  - It changes the recipe format for `crop` **and** `pad` (SPEC-151's frame named it), so it's CLI
    work, not page work.
  - With box presets, the page no longer needs it. The ground for moving it here ("the page's
    aspect presets are this key's first user") no longer holds, so a CLI stage or
    `guidance/ideas.yaml` may suit it better.
  - Not moved.

### The handed-in items' file overlap

**No overlap** under the recommended fix.
- This spec touches no Rust. `format = "bogus"` and `quality = 101` touch recipe load
  (`src/recipe*.rs`, `src/cli/*`), and `validate()` touches `src/wasm.rs`.
- They can be scheduled alongside.
- The overlap would be `src/wasm.rs` only if design rejects the oriented-base fix for an
  `ImageInfo.orientation` getter.

### What is testable where

- **Node (this spec, all of it):**
  - `recipe.js` is pure: `planEdit` with a crop and the oriented size, the preset box (the
    `aspect_size` port), fraction formatting, and `pixelRecipe`/`cliRecipe` with the crop step in
    order.
  - The harness's rows, the 4× and off-integer replays, N1–N4, and the golden recipe texts.
  - The worker's oriented-size sequence, run on the real slim wasm in Node, as the harness
    already loads it.
- **The test bed's headless Chromium (the separate "Test-bed round and README" entry):** the crop
  mode, drag, keyboard, touch emulation, Done → one preview, on dev, static and apache.
- **What verify can drive alone:** `just ddphotos-parity` and its self-test, mutation of
  `recipe.js` (for example, delete the oriented-size swap and require R-refit red), and Node
  probes of `recipe.js` against the native binary. Never the maintainer's browser.

### Sizing

[M] for this half. The verification is the bulk:
- About 25 native-vs-wasm rows (16 preset × orientation, 2 free, 1 off-centre, 2 refit, 2
  cli/preview, 2 formats), so `ROW_COUNT` goes from 16 to about 41.
- About 22 replay rows (one per crop row, plus one off-integer), with a new comparator limb (PSNR)
  and its B-cases.
- Two new negative controls, two fixtures, and goldens for a crop recipe text.
- A new DEC.
- Code: about 80–120 lines of `recipe.js`, about 15 of `worker.js`, and about 200 of harness.

### Scope

In:
- `recipe.js`: the crop model (box in oriented copy pixels → fractions), the preset box, `planEdit`
  on the oriented and cropped size, and the crop step in `pixelRecipe`/`cliRecipe` after
  `auto-orient` and before `resize`.
- `worker.js`: `load` renders the oriented base and reports its size.
- `edit.js`: only the oriented-size wiring. No crop control.
- The harness's rows, replay mode, N3, N4 and the folded labels; the fixtures and their README.
- DEC-116.

Out, and where each lives:
- The crop mode UI, presets as buttons, keyboard and touch, and the CLI panel's smaller-original
  wording: the new second entry.
- The README's controls description and the test-bed browser round: "Test-bed round and README".
- `rotate` on the page: "Straighten on the page".
- An orientation key for `crop`/`pad`: its own entry (left).
- A size gate for the slim bundle: the new finding entry.
- Save flow, sidecars, flip/pad/trim on the page, and ddphotos code: the stage's
  `explicitly_does_not`.

### Rough build file list

- `integrations/ddphotos/edit/recipe.js`: the crop model, preset box, and `planEdit`/recipes
- `integrations/ddphotos/edit/worker.js`: the oriented base and size at `load`
- `integrations/ddphotos/edit/edit.js`: use the oriented size (one call site)
- `tests/ddphotos_parity.mjs`: rows, replay mode, PSNR limb, N3/N4, labels (fold), removing the
  rows 15–16 transpose
- `tests/ddphotos_decode.py`: probably unchanged; the replay's Pillow upscale may live in a
  sibling script
- `tests/fixtures/ddphotos/`: `photo_portrait.webp`, `photo_square.webp`, README
- `decisions/DEC-116-*.md`: the replay bound
- `justfile`: only if `ddphotos-parity` needs a new argument

### DEC-116 (recommended content; design writes it)

"Replay parity: the page's output from the published copy and the CLI's `cliRecipe` output from
the original agree":
1. **Dimensions:** exact when the original is an integer multiple of the copy. When it isn't,
   each side is within 1 px for an original ≥ 2× the copy, and within 2 px below 2×. Both are
   driven bounds; this frame drove them over 8M cases.
2. **Framing:** each crop edge within DEC-112's 0.6 px once scaled.
3. **Pixels:** a PSNR floor measured on the harness's Pillow-made original, with no headroom,
   that N4 proves can see a 1-copy-pixel shift.

Its `affected_scope` is `tests/ddphotos_parity.mjs` and `integrations/ddphotos/edit/recipe.js`, in
block-list form (HANDOFF trap 7).

### Open questions for the maintainer

1. **Split as above (model and replay first, then the crop mode)?** Recommend yes.
2. **The interaction: a crop mode on the oriented base, the DOM box, one render on Done?**
   Recommend yes.
3. **Presets emit a box, not `aspect`?** Recommend yes. It's exact at 4×, there's one form on the
   page, and no recipe key is needed. The cost: the CLI panel shows `x/y/width/height`, not
   `aspect = "3:2"`.
4. **The orientation-key entry's home,** now that the page doesn't use it: leave it in STAGE-060,
   move it to a CLI stage, or move it to `guidance/ideas.yaml`. Recommend moving it out of
   STAGE-060 so the stage can close without it.
5. **Write DEC-116 for replay parity?** Recommend yes. It's the property the stage's third
   `delivers` line names, and no DEC covers it.

## Inputs

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth>
- **Related code paths:** `src/some/module/`

## Outputs

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New exports:** <names and signatures>
- **Database changes:** <migrations>

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

### Out of scope (for this spec specifically)

- The crop mode, the draggable box, preset buttons, keyboard and touch handling, and the CLI
  panel's wording: STAGE-060's "Crop on the page: the crop mode" entry (filed at this frame).
- The page README's controls text and the three-server browser round: STAGE-060's "Test-bed round
  and README".
- `rotate`/straighten on the page: STAGE-060's "Straighten on the page".
- A `crop`/`pad` key that follows each image's orientation: STAGE-060's orientation entry (left;
  maintainer question 4).
- A size gate on the slim wasm bundle: the STAGE-060 finding entry filed at this frame.
- Any Rust change, including a wasm `ImageInfo.orientation` getter.
- Recipe sidecars, any save flow, flip/pad/trim on the page, and ddphotos code (STAGE-060
  `explicitly_does_not`).

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Not "src/foo and tests": the enumeration is what a verify cycle diffs
  the real change against, and what tells you whether every affected `DEC-*` has
  the path in its `affected_scope`. SPEC-122's Deviations claimed "`src/operation`
  and `tests/` only" and was wrong by two `scripts/` files, which left
  `affected_scope` blind; SPEC-124's build added this section by hand after that
  lesson and still listed 8 of its 9 files. It is a checklist because it is easy
  to get wrong from memory — build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
