---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-161
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
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
    - DEC-114
    - DEC-118
  constraints:
    - test-before-implementation
  related_specs:
    - SPEC-142
    - SPEC-145
    - SPEC-163

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
  sessions:
    - cycle: frame
      date: 2026-11-22
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 8444912
      tokens_breakdown:
        input: 120
        cache_creation: 161105
        cache_read: 8235183
        output: 48504
      estimated_usd: 6.34
      note: >
        Measured from the session JSONL (found by its first message, `prompts/SPEC-161-frame.md`), deduped by
        message.id (60 messages; max output_tokens per id), 0 subagent transcripts. Measured after the push,
        before this cost commit's own turns.
        Rates (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read
        (x0.10), $25 output.
    - cycle: design
      date: 2026-11-26
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 14353882
      tokens_breakdown:
        input: 156
        cache_creation: 245617
        cache_read: 13986046
        output: 122063
      estimated_usd: 11.58
      note: >
        Measured from the session JSONL (found by its first message, `prompts/SPEC-161-design.md`),
        deduped by message.id (78 messages; max output_tokens per id), 0 subagent transcripts.
        Measured after the design push, before this cost commit's own turns.
        Rates (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read
        (x0.10), $25 output.
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

⚠ **Ruled 2026-11-22: presets emit `aspect`, not a box** (question 3 reversed), and the replay DEC's
number is set at design. Read "Maintainer rulings" at the end of this section before anything
below that says presets emit a box.

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

### Maintainer rulings (2026-11-22)

1. **Split: yes.** This spec is the recipe model and the 4× replay. The crop mode is the next
   STAGE-060 entry.
2. **Interaction: yes,** as recommended (crop mode on the oriented base, DOM box, one render on
   Done). It's the second half's to build.
3. **Presets emit `aspect`, not a box. Reversed: the recipe must be readable and reusable.** A
   recipe copied from the page should say `aspect = "3:2"` and work on any photo, not carry four
   fractions that fit only this one. What follows for design:
   - An untouched preset emits `aspect` (with the orientation key, see 4). Once the user drags or
     resizes the box, it emits a box. So the page has two crop forms, and the parity rows cover
     both.
   - The page's predicted size must equal what wasm makes from the copy for `aspect` too: port
     `aspect_size`'s rounding into `recipe.js` exactly, or read the size back from wasm. Native
     against wasm on the same input stays exact (same Rust).
   - The 4× replay loses "exact at an integer multiple" for `aspect` rows, because `aspect_size`
     rounds the short side on each image separately (the frame's 602 vs 603 and 534 vs 533). That
     difference is accepted as the price of readability. Design re-drives the worst case for
     `aspect` across the frame's ratios, sizes and copy/original pairs, and the DEC states the
     driven bound per form: box rows exact at integer multiples, `aspect` rows within the driven
     bound. Don't assume 1 px.
4. **The orientation-key entry stays in PROJ-017 STAGE-060 and moves ahead of this spec in build
   order.** With `aspect` presets, a reusable "3:2" recipe has to follow each photo's orientation,
   so the page is the key's first user again. This spec emits the key on a preset that follows
   the photo. The key's own spec touches Rust (`crop`, `pad`, the recipe loader); this spec still
   touches none.
5. **The replay DEC: yes.** Its number is allocated at design, not here: the STAGE-063 P1 designs
   run first and may take DEC-116. Read "DEC-116" in this stub as "the replay DEC".

Order: the STAGE-063 P1 fixes ship first, then the orientation key, then this spec's design.

## Design

*Designed 2026-11-26 on `main` at `9af9ab6`, Opus. Every number below was driven on a prototype
(a detached worktree of `main` in the session scratchpad, deleted at the end) against a fresh
release binary and a fresh slim wasm, both built by `just ddphotos-parity`, unless it says
otherwise. The replay DEC is **DEC-118** (`/bin/ls decisions` ends at DEC-117; `DEC-118` appears in
`projects/` only as SPEC-163's unused alternative, and nowhere in `guidance/`).*

### Design corrections to the frame

1. **The box form is not exact at an integer multiple.** `resize` mode `max` rounds `side · (n /
   long)` in `f64`, and at an exact `.5` the page's and the CLI's products land on opposite sides
   of it. Driven: copy 1220×1600 (an original of 7320×9600, 6×), box 1014×1248 at 1080: the page
   computes `1014 · (1080 / 1248)` = 877.5 → **878**; the CLI computes `6084 · (1080 / 7488)` =
   877.4999999999999 → **877**. In the model sweep (below) that is **4 of 1,521,000** integer-multiple
   box cases, all 1 px. The frame's "0 of 1,000,000 at exact 4×" was a sweep that never landed on a
   tie (HANDOFF trap 17). DEC-118 states box rows as "exact at an integer multiple except at an
   `f64` tie in the resize, then 1 px".
2. **A preset on a square photo emits no orientation key.** Driven: 19,206 originals with a long
   side of 1601–9600 and |W − H| ≤ 10 publish as a *square* 1600 copy. With the key, a 3:2 preset on
   that copy replays transposed: 3199×3200 → copy 1600×1600 → page 1600×1067, CLI 2133×3200 (portrait,
   because the original is portrait by one pixel). Without the key the replay is within 1 px. A
   square has no orientation to follow, so the page keeps the aspect as the user sees it. 1:1 never
   needs the key either. Every other untouched preset emits `orientation = "match"` (ruling 4).
3. **`aspect` at replay is within 1 px in every band, exhaustively.** The frame's 602-vs-603 and
   534-vs-533 are the worst case, not a sample of a wider spread. Driven below over 273,365,360
   cases.
4. **`planEdit` drops `resize`'s `.max(1)`.** `src/operation/mod.rs:1007–1008` clamps each side to
   at least 1 px; `recipe.js:68–69` does not. A crop makes thin outputs reachable (a 1-px-tall box at
   a small size), so the port adds `Math.max(1, …)`. Driven in the copy-size sweep below.
5. **The worker's `load` logic moves into `recipe.js` as `loadPhoto(wasm, source)`.** The frame put
   it inline in `worker.js`, which Node can't run, so the harness would test a copy of it (HANDOFF
   trap 14). `loadPhoto` takes the engine's `{ info, transform }` as an argument, so `recipe.js`
   still imports nothing and reads no page, and the harness calls the production function. The
   worker's `load` becomes one line, and a module check pins that line. **`edit.js` needs no
   change:** it already builds `photo` from `meta.width`/`meta.height` (`edit.js:326–327`), which
   now carry the oriented size.
6. **The frame's N3/N4 are N4 and N5 here** (SPEC-163's self-test owns N3). N4 keeps the frame's
   edit (`width = 0.5\n` → `width = 0.500625\n`).
7. **The PSNR floor is per form, and much lower than the frame's 64.8 dB.** An aspect crop is
   centred on each image separately (`(W − cw) / 2` floored on the copy, exact on the 4× original),
   so its framing differs by up to half a copy pixel by design, and its rows read 53.95–73.41 dB. Box
   rows read 59.46–75.69 dB. The frame's 64.8 dB was one box row on an ImageMagick upscale.
8. **N5 can't see a shift in one direction on two aspect rows.** The half-pixel centring offset
   always points the same way (the copy's floor rounds left). N5 shifts the content left, which adds
   to it, and turns all 17 replay rows red. Driven the other way, the shift cancels the offset on R1
   (W 1:1, 54.66 dB) and R9 (S 2:3, 54.05 dB), which stay above the 53.5 floor. Aspect rows'
   placement is therefore pinned by an exact native-free check instead (`placement`, below): the
   recipe text of an aspect carries no placement, so the only page-side placement code is `cropBox`,
   and the check proves its box is where the engine puts the aspect.
9. **Replay rows: 17, not ~22.** One per form × orientation × kind, not one per crop row (the
   derivation is under "Sizing").

### What was driven

**The `aspect` and box replay bound (DEC-118 rule 1).** An independent model, written line for line
from `crop.rs` (`rect_edges` :218, `aspect_size` :233, the `follow_orientation` of `mod.rs:1252`,
centre placement `mod.rs:1313`) and `resize` mode `max` (`mod.rs:999–1009`), never from `recipe.js`:
the copy is `resize max 1600` of the original; the page crops the copy and resizes to the edit's
size; the CLI crops the original and resizes to the page's `outLong`. Ratios 1:1, 4:5, 5:4, 3:2, 2:3,
16:9 and 9:16; sizes as published, 1200, 1080, 800 and 400; a following preset uses the key exactly
as `presetCrop` decides (correction 2).
- Pairs: exact 4× of W, P, S, N (6400×4284, 4284×6400, 4284×4284, 4284×4288); the camera sizes
  4032×3024, 3024×4032, 6000×4000, 4000×6000, 6000×4016, 5472×3648; the 1700 px case (1700×1138 and
  1138×1700); 6,000 random originals of 1601–9600 px (landscape, portrait, square and near-square
  in turn); 3,000 random copies at 2×–6× exactly. 2,000 random boxes per fixed pair and size, 100
  per random pair and size.
- **Worst difference per form and band** (n, cases that differ, worst):
  - aspect, below 2×: 41,230, 3,656 (8.87 %), **1 px** (1700×1138, 3:2: page 1600×1067, CLI 1600×1066).
  - aspect, 2× and up (non-integer): 169,085, 11,354 (6.71 %), **1 px**.
  - aspect, integer multiple: 105,140, 3,652 (3.47 %), **1 px** (6400×4284, 3:2 at 800: page 800×534,
    CLI 800×533).
  - box, below 2×: 608,000, 98,990 (16.28 %), **2 px** (1700×1138, box 318×465 at 400: page 274×400,
    CLI 272×400).
  - box, 2× and up: 2,501,000, 118,649 (4.74 %), **1 px**.
  - box, integer multiple: 1,521,000, **4** (0.0003 %), **1 px**, each an `f64` tie (correction 1).
- **Exhaustive for `aspect`** (trap 17: a seeded sweep can miss a tie): every landscape original with
  a long side of 1601–4800 and every short side from long/3 up, plus long side 6400, × 1:1, 4:5,
  3:2, 16:9 in both written orders × the five sizes: **273,365,360 cases, worst 1 px** (1601–3199:
  102,378,640, 6,185,350 at 1 px; 3200–4800: 170,816,040, 7,893,489 at 1 px; 6400: 170,680, 5,439
  at 1 px). Portrait is the mirror (the key and `aspect_size` treat both axes alike). Box below 2×
  is "measured over this sweep", not exhaustive.
- **Negative controls on the sweep:** with the page's short side floored (the `aspect_size` port's
  half-up removed) the integer band's differing count goes from 3,652 to 8,804 (8.37 %), still 1 px,
  so this sweep sees the rounding but its bound can't; the exact page-against-wasm check below is
  what catches it. With the key dropped on the CLI side only, aspect below 2× reaches **1,057 px**
  (12,176 transposed cases), so the sweep can see an orientation error. With box fractions written
  `toFixed(4)`, the integer band goes from 4 to 3,896 differing cases.

**The page's prediction equals wasm on the copy, both forms.** `planEdit` (with the `aspectSize`
port and `followOrientation`) against `info()` of `transform(copy, pixelRecipe(edit), "png")` on the
real slim wasm in Node:
- every preset 1:1, 4:5, 5:4, 3:2, 2:3, 16:9, 9:16 × preset and swapped × W, R, P, S, N × as
  published, 1080 and 400: **210 checked, 0 misses**;
- 304 copy sizes (every w×h from 1×1 to 12×12; 40 random L×L, L×(L+1), (L+1)×L and L×random for L of
  13–1600) × every preset, preset and swapped, × own size, 800, 7 and 1, plus 4 random boxes each at
  own size, 800 and 3: **20,672 checked, 0 misses**. Exact, as required.
- Control: with the `aspectSize` short side floored, **46 of 210** fixture checks and **3,878 of
  20,672** sweep checks miss.

**The oriented base** (the worker's `load` sequence, run as `loadPhoto` on the real slim wasm):
- W: `info` 1600×1071, base 1600×1071; **R: `info` 1600×1071, base 1071×1600**; P 1071×1600 both; S
  1071×1071 both; N 1071×1072 both. Render time 27–49 ms on the Mac (the frame's 31–34 ms).
- `planEdit` then needs no transpose: TABLE rows 15–16 pass with the transpose deleted, at **803×1200
  and 723×1080**, unchanged.

**W-refit and R-refit** with the new `planEdit` (rows C29, C30, R14, R15):
- W-refit: W, box x 600 w 400 (fractions 0.375 / 0.25), full height, watermark on, as published:
  **400×1071, mark 40, capped**. Uncropped W at the same size: 64, uncapped.
- R-refit: R, box x 348 w 375 of the oriented 1071 (fractions 0.32492997198879553 /
  0.35014005602240894), full height: **375×1600, mark 37, capped**. With the stored-size bug (mutant
  M4 below) the row reads **560×1071, mark 43, uncapped**: output size, mark size and `capped` all
  differ. The frame's fractions (0.325 / 0.35) become pixel-snapped boxes; the numbers are the
  frame's.

**Gravity.** A preset is always centred (a moved box is a box), so it **emits no `gravity`**: crop
defaults to `Center` (`crop.rs:134`). Readable because the recipe says only what the user chose:
`aspect = "3:2"` and, for a following preset, `orientation = "match"`. Pinned by `GOLDEN_CROP_CLI`
and `GOLDEN_CROP_SQUARE`. The model has no gravity field.

**The swap.** A swapped preset is the *other* orientation of what the page shows, written in that
order, **with no key**: the user chose an orientation, so a reused recipe keeps it. It is an aspect,
not a box. Recipe text per state, for the 3:2 button:
- landscape photo (W), untouched: `aspect = "3:2"` + `orientation = "match"` → 1600×1067;
- landscape photo, swapped: `aspect = "2:3"` → 714×1071;
- portrait photo (P, R), untouched: `aspect = "3:2"` + `orientation = "match"` → 1067×1600;
- portrait photo, swapped: `aspect = "3:2"` → 1071×714;
- square photo (S), untouched: `aspect = "3:2"` → 1071×714; swapped: `aspect = "2:3"` → 714×1071;
- 1:1 on any photo: `aspect = "1:1"`, and the swap is a no-op.

**Fractions** on the port, not only the arithmetic: every `(x, width)` on sides 400, 900, 1067,
1071, 1200, 1600 and 2048, written by `planEdit`'s crop text and read back with `rect_edges`'
rounding: **5,729,060 boxes, 0 read back to other edges, 0 with `x + width` > 1**. Control: with
`toFixed(4)`, still 0 edge misses on these sides but **132 boxes with `x + width` > 1** (116 on 1600,
16 on 2048), which DEC-112's load check refuses. The harness keeps a smaller version (sides 1071 and
1600, 1,854,856 boxes).

**The PSNR floor and N5.** Originals made by Pillow 12.3.0 (`LANCZOS`, the harness venv's pinned
Pillow), PSNR over RGB of the region both images share from the top-left (the replay rows' sizes are
equal, so that is the whole image):
- clean, aspect rows R1–R10, R17: 54.55, 58.07, 66.35, 55.10, 63.88, 63.22, 59.04, 55.17, **53.95**,
  73.41, 68.10 → floor **53.5 dB**;
- clean, box rows R11–R16: 66.54, **59.46**, 74.11, 72.73, 59.90, 62.92 → floor **59 dB**;
- N5 (content moved left one copy pixel, size kept): 46.92, 48.16, 50.21, 49.90, 51.71, 51.30, 48.84,
  49.51, 45.96, 48.53, 52.77, 50.72, 50.18, 48.12, 49.69, 52.00, 50.01. Every aspect row ≤ 51.71 <
  53.5, every box row ≤ 52.77 < 59. **All 17 red.**
- The comparator needs no alignment: DEC-118 allows a 1 px size difference, but none of the 17 rows
  has one (their CLI sizes are literals equal to the page's), and on a row that had one the PSNR runs
  over the shared top-left region.

**Size and the wasm.** No Rust change. `just ddphotos-parity`'s slim build on `main` (the same
`wasm-build` `just wasm-size` measures): raw 4,148,451 B, gzip 1,195,748 B, **brotli 833,619 B**;
the prototype changes no file under `src/`, so **0 B** of growth. `recipe.js` grows from 6,467 B to
**10,592 B (+4,125 B)**; `worker.js` shrinks from 4,037 B to 3,914 B (−123 B); `edit.js` is unchanged.

### Sizing: [M], recounted

- Crop rows: **35** = presets on W, P, N, R (4 × 4 = 16) + presets on S in both orders (7) + swaps (2)
  + boxes C26–C30 (5) + recipe and format variants C31–C34 (4) + an aspect `cli` row C35 (1).
- Replay rows: **17** = aspect at 4× R1–R10 (10) + box at 4× R11–R15 (5) + 3.75× R16–R17 (2).
- Pure checks: `model-aspect` 8 literals, `model-follow` 11 literals, `model-fractions` 1,854,856
  boxes (1071·1072/2 + 1600·1601/2); `placement` 3; goldens 3; comparator cases B11–B19 (9).
- Code, measured on the prototype (`git diff --numstat`): `recipe.js` +96 −9, `worker.js` +2 −5,
  `tests/ddphotos_parity.mjs` +457 −30, a new 29-line `tests/ddphotos_upscale.py`. About 380 of the
  harness lines are row tables, literals and goldens.
- Not [L]: no native dependency, no new UI surface (the crop mode is the next entry), and the PSNR
  limb is a test comparator the frame already counted.

### The model (`integrations/ddphotos/edit/recipe.js`)

The crop state is `null` (no crop), `{ aspect: [a, b], follow }` (a preset, centred), or
`{ box: { x, y, w, h } }` (a placed rectangle), always in the pixels of the **oriented** copy. Build
writes these functions as below (the prototype's text, driven green through the full harness).
Line numbers are the prototype's; `main`'s `planEdit` is `recipe.js:56–93`, `pixelRecipe` :96–109,
`cliRecipe` :118–123.
- `aspectSize(a, b, w, h)` → `[width, height]` (:61): `aspect_size` (`crop.rs:233`) line for line.
  Every product is below 2^53 for sides and ratio terms up to 65535, so `Number` maths is exact.
- `followOrientation(a, b, w, h)` → `[a, b]` (:68): SPEC-163's page port, as written.
- `presetCrop(a, b, photo)` → crop (:80): `follow` is true unless `a === b` or the photo is square.
- `swapCrop(crop, photo)` → crop (:86): the shown aspect reversed, `follow: false`.
- `cropBox(crop, photo)` → `{ x, y, w, h }` (:99): the whole photo for `null`; the box as given; or
  the aspect's size centred as `Gravity::Center` places it (`mod.rs:1313`, `(side − crop) / 2`
  floored). For the second half's display and drag start.
- `planEdit(photo, form, crop = null)` (:114): `photo` is the oriented size. Crop first, then
  `longEdge`, `outLong`, the sides (now with `Math.max(1, …)`), the margin and the watermark fit,
  all from the cropped size. Returns the old fields plus `crop`, the step's keys as text (or null).
- `cropStep(crop, photo)` (:153, not exported): `aspect = "a:b"` plus `orientation = "match"` when
  following; or `x`, `y`, `width`, `height` as `${n / side}` (JS's shortest round-trip text, as
  DEC-114 rule 5 writes `--save-recipe`).
- `pixelRecipe(edit)` (:164): the crop step goes right after `auto-orient`, before `resize`.
  `cliRecipe` and `previewRecipe` call `pixelRecipe`, so the order holds in all three.
- `loadPhoto(wasm, source)` → `{ width, height, format, bytes }` (:215): `format` from `info` of the
  stored file; the size from `info` of `ORIENTED_BASE` (`auto-orient` only, at the preview's JPEG
  quality, :209) rendered by `transform`; `bytes` is that render, for the crop mode to show.

```js
export function aspectSize(a, b, w, h) {
  if (w * b <= h * a) return [w, Math.floor((2 * w * b + a) / (2 * a))];
  return [Math.floor((2 * h * a + b) / (2 * b)), h];
}

export function followOrientation(a, b, w, h) {
  const long = Math.max(a, b);
  const short = Math.min(a, b);
  if (w > h) return [long, short];
  if (w < h) return [short, long];
  return [a, b];
}

export function presetCrop(a, b, photo) {
  return { aspect: [a, b], follow: a !== b && photo.width !== photo.height };
}

export function swapCrop(crop, photo) {
  const [a, b] = shownAspect(crop, photo);
  return { aspect: [b, a], follow: false };
}

function shownAspect(crop, photo) {
  const [a, b] = crop.aspect;
  return crop.follow ? followOrientation(a, b, photo.width, photo.height) : [a, b];
}

export function cropBox(crop, photo) {
  if (!crop) return { x: 0, y: 0, w: photo.width, h: photo.height };
  if (crop.box) return { ...crop.box };
  const [a, b] = shownAspect(crop, photo);
  const [w, h] = aspectSize(a, b, photo.width, photo.height);
  return { x: Math.floor((photo.width - w) / 2), y: Math.floor((photo.height - h) / 2), w, h };
}

// planEdit: the first lines change to
  const box = cropBox(crop, photo);
  const longEdge = Math.max(box.w, box.h);
  ...
  const outW = Math.max(1, Math.round(box.w * s));
  const outH = Math.max(1, Math.round(box.h * s));
  ...
  return { crop: cropStep(crop, photo), resize: ..., /* the rest as today */ };

function cropStep(crop, photo) {
  if (!crop) return null;
  if (crop.box) {
    const { x, y, w, h } = crop.box;
    return `x = ${x / photo.width}\ny = ${y / photo.height}\nwidth = ${w / photo.width}\nheight = ${h / photo.height}\n`;
  }
  const [a, b] = crop.aspect;
  return `aspect = "${a}:${b}"\n` + (crop.follow ? 'orientation = "match"\n' : "");
}

// pixelRecipe: one new line after the auto-orient line
  if (edit.crop) r += `\n[[step]]\nop = "crop"\n${edit.crop}`;

const ORIENTED_BASE = previewRecipe('version = "2"\n\n[[step]]\nop = "auto-orient"\n');

export function loadPhoto(wasm, source) {
  const stored = wasm.info(source);
  const format = stored.format;
  stored.free(); // wasm-side objects are not garbage-collected
  const bytes = wasm.transform(source, ORIENTED_BASE, "jpeg");
  const shown = wasm.info(bytes);
  const photo = { width: shown.width, height: shown.height, format, bytes };
  shown.free();
  return photo;
}
```

Each exported function gets a `///` comment that says what it does in plain words (the prototype's
are in the scratch diff's spirit: behaviour first, no spec, stage or DEC ids). `ORIENTED_BASE` must
be declared after `PREVIEW_QUALITY` (it calls `previewRecipe` at module load).

### `worker.js` and `edit.js`

- `worker.js:9–10`: import `loadPhoto` (and `previewRecipe`) from `./recipe.js`.
- `worker.js:38–44`, the `load` case, becomes:
  ```js
      source = new Uint8Array(req.bytes);
      return loadPhoto({ info, transform }, source);
  ```
  The reply now carries `bytes` (the oriented base JPEG). `self.onmessage` already transfers
  `reply.bytes` (`worker.js:106`), so it costs no copy.
- `edit.js`: **no change.** `edit.js:326–327` reads `meta.width` and `meta.height`, which are now the
  oriented size; `readEdit` (`edit.js:188–199`) calls `planEdit(photo, {...})` with no crop, so the
  default `crop = null` applies. The crop mode (next entry) adds the crop argument and shows
  `meta.bytes`.
- **What the user sees change:** only for a published JPEG whose orientation tag turns it (photogen
  publishes upright WebP, so most sites see nothing). The page now plans such a photo as the
  portrait it shows: the stated output size, and the watermark's fit. Driven on R, as published, with
  `main`'s `recipe.js`:
  - "© Jane Doe 2026" at Medium: 64, uncapped, either way (no visible change).
  - "© 2026 Jane Doe Photography, all rights reserved" at Medium: planned from the stored 1600×1071
    it is 63 (capped for 1600 px wide), and native `apply` **refuses** it on the real 1071×1600
    output (`watermark refused: the 1405×62 px mark plus its 32 px margin does not fit inside the
    1071×1600 px image`, exit 2). Planned from the oriented size it is 41, capped, and `apply`
    exits 0. So today the page's "Make" fails on such a photo with a long mark; after this spec it
    shows "Made smaller to fit this photo" and succeeds.

### The replay mode

- **The original:** `tests/ddphotos_upscale.py SRC DST FACTOR [SHIFT]`, a new 29-line script run
  by the harness's `--python` (the venv with Pillow 12.3.0 that `just ddphotos-parity` already
  builds). `round(side · FACTOR)` with `Image.LANCZOS`. A JPEG source keeps its EXIF block, so R's
  original is a 4284×6400 *oriented* photo stored 6400×4284 with Orientation 6. Written into the
  run's `mkdtemp` folder, one per input × factor, removed on a green run. Nothing large is committed.
- **Each row:** the page side is `transform(copy, pixelRecipe(edit), "png")` (wasm); the CLI side is
  native `apply --recipe <cliRecipe(edit)> <original> -o … -y -Q`. Both decode with
  `tests/ddphotos_decode.py`.
- **The limbs, in order:** (1) page size equals the row's literal; (2) CLI size equals the row's
  literal; (3) `planEdit`'s `outW × outH` equals the page literal, and the watermark plan equals the
  row's `[size, capped]`; (4) `judgeReplay`: the size difference within DEC-118's bound for the
  row's form and band, then PSNR at or above the form's floor.
- **`judgeReplay({ form, band, page, cli, psnr })`** (exported, pure): bound 2 for box below 2×, else
  1; `replay-dims` if the larger side difference exceeds it; then `psnr` unless `psnr >=
  PSNR_FLOOR[form]` (so NaN is red). `band` is `integer` for a whole-number factor, `2x` for ≥ 2,
  `under-2x` below.
- **`psnr(a, b)`** (exported, pure): RGB over the shared top-left region, `Infinity` when equal.
- **B-cases** (in `--self-test`, beside B1–B10): B11 box integer 1 px → green; B12 box integer 2 px →
  `replay-dims`; B13 box under-2x 2 px → green; B14 aspect under-2x 2 px → `replay-dims`; B15 aspect
  at exactly 53.5 → green; B16 aspect 53.49 → `psnr`; B17 box 58.99 → `psnr`; B18 NaN → `psnr`; B19
  `psnr` of two 2×1 images 100 vs 101 = 48.13 dB (10·log10(65025)).

### The rows

Inputs: W `photo.webp` 1600×1071; P `photo_portrait.webp` 1071×1600; S `photo_square.webp`
1071×1071; N `photo_near_square.webp` 1071×1072; R `photo_rot.jpg`, stored 1600×1071, Orientation 6,
oriented 1071×1600. **No new fixture:** every row above names one of these five, and SPEC-163's
three serve as they are (no row needs another shape).

Every size is a literal from the independent model (or the frame's and SPEC-163's driven numbers,
which agree), never from `recipe.js` or crustyimg. "pub" is as published: the form's size is the
crop's own long edge, so there is no `resize` step.

**Crop rows (native against wasm, same input; family `CROP_TABLE`, ids C1–C35).** Exact (0 px,
bytes equal for PNG and JPEG, as TABLE's rule) unless marked *DEC-110* (a resize, ≤ 1/255 on ≤ K px).
Recipe `pixel` and format `png` unless stated.
- C1–C4, W presets pub: 1:1 → 1071×1071; 4:5 → 1339×1071; 3:2 → 1600×1067; 16:9 → 1600×900.
- C5–C8, P presets pub: 1071×1071; 1071×1339; 1067×1600; 900×1600.
- C9–C12, N presets pub: 1071×1071; 858×1072; 715×1072; 603×1072.
- C13–C16, R presets pub: 1071×1071; 1071×1339; 1067×1600; 900×1600.
- C17–C23, S presets pub, both orders: 1:1 → 1071×1071; 4:5 → 857×1071; 5:4 → 1071×857; 3:2 →
  1071×714; 2:3 → 714×1071; 16:9 → 1071×602; 9:16 → 602×1071.
- C24, W 3:2 swapped pub → 714×1071. C25, P 3:2 swapped pub → 1071×714.
- C26, W box (160, 214, 800, 643) at 600 → 600×482, *DEC-110*.
- C27, R box (107, 320, 536, 960) at 800 → 447×800, *DEC-110*.
- C28, W box (0, 0, 1600, 1067) pub → 1600×1067 (writes `x = 0`, `width = 1`: integers, which crop
  accepts; driven).
- C29, W-refit: W box (600, 0, 400, 1071) pub, watermark on → 400×1071, mark [40, capped].
- C30, R-refit: R box (348, 0, 375, 1600) pub, watermark on → 375×1600, mark [37, capped].
- C31, C26's box, `cli`/png at 600 → 600×482, *DEC-110*.
- C32, C26's box, `preview`/jpeg pub → 800×643 (exact; JPEG bytes asserted, as rows 9 and 11).
- C33, C26's box, `pixel`/webp at 600 → 600×482, *DEC-110*. C34, the same as avif → 600×482,
  *DEC-110*.
- C35, P 3:2 preset, `cli`/png at 1080, watermark on → 720×1080, mark [43, uncapped], *DEC-110*.
- `CROP_ROW_COUNT = 35`; `CROP_NO_RESAMPLE_ROWS` = every C row except C26, C27, C31, C33, C34, C35.

**Goldens** (literal texts in the harness; HANDOFF trap 13: each has a mutant that turns it red by
text, below):
- `GOLDEN_CROP_BOX`, C26's text: `version = "2"`, `auto-orient`, then `op = "crop"` with `x = 0.1`,
  `y = 0.19981325863678806`, `width = 0.5`, `height = 0.6003734827264239`, then `resize` `mode =
  "max"` `width = 600`. Readable: four fractions of the photo, each the exact pixel edge.
- `GOLDEN_CROP_SQUARE`, C20's text: `auto-orient`, then `op = "crop"` with `aspect = "3:2"` and
  nothing else.
- `GOLDEN_CROP_CLI`, C35's text: `version = "2"`, `format = "png"`, `auto-orient`, `op = "crop"` with
  `aspect = "3:2"` and `orientation = "match"`, `resize` `width = 1080`, then the watermark exactly as
  `GOLDEN_PIXEL_BOTH`'s (`size = 43`, `margin = 22`).
  (Each in TOML with a blank line before every `[[step]]`, as `GOLDEN_PIXEL_BOTH`.)

**Pure checks** (no binary):
- `model-aspect`, `aspectSize` against literals: 3:2 on 1600×1071 → 1600×1067; 2:3 on 1071×1600 →
  1067×1600; 3:2 on 1071×1071 → 1071×714; 2:3 on 1071×1072 → 715×1072; 4:5 on 1071×1071 →
  857×1071 (856.8: floor gives 856); 4:5 on 1071×1072 → 858×1072 (857.6); 9:16 on 1600×1071 →
  602×1071; 16:9 on 3×1 → 2×1.
- `model-follow`, `followOrientation` against the 11 rows of `crop.rs`'s
  `follow_orientation_orders_the_aspect_by_shape` (`mod.rs:2097–2117`), copied as literals.
- `model-fractions`: every `(x, width)` on sides 1071 and 1600 (1,854,856 boxes), from
  `planEdit(photo, form, { box })`'s crop text: each reads back to the same edges and `x + width ≤ 1`;
  the box count is asserted.
- `placement` (wasm only): for W 1:1, W 4:5 and S 2:3 (centre margins 529, 261 and 357, all odd),
  `transform` of the preset's recipe and of `{ box: cropBox(preset) }`'s recipe give identical PNG
  bytes.

**Replay rows (family `REPLAY_TABLE`, ids R1–R17), within DEC-118.** Page size = CLI size in every
row (literals); factor 4 unless stated.
- R1–R4, W presets pub: 1:1 → 1071×1071; 4:5 → 1339×1071; 3:2 → 1600×1067; 16:9 → 1600×900.
- R5 P 3:2 → 1067×1600; R6 N 3:2 → 715×1072; R7 R 3:2 → 1067×1600; R8 S 3:2 → 1071×714; R9 S 2:3 →
  714×1071; R10 W 3:2 swapped → 714×1071. (All aspect, floor 53.5.)
- R11 W box (160, 214, 800, 643) at 600 → 600×482; R12 R box (107, 320, 536, 960) at 800 → 447×800;
  R13 W box (0, 0, 1600, 1067) pub → 1600×1067; R14 W-refit → 400×1071, mark [40, capped]; R15
  R-refit → 375×1600, mark [37, capped]. (All box, floor 59.)
- R16, factor 3.75 (original 6000×4016), C26's box at 600 → 600×482 (box, band `2x`).
- R17, factor 3.75, W 3:2 preset pub → 1600×1067 (aspect, band `2x`).
- `REPLAY_ROW_COUNT = 17`.

**Unchanged families:** TABLE's 16 rows (with the transpose deleted and `planEdit` fed by
`loadPhoto`), the alpha rows A and B, and SPEC-163's 26 orientation rows. The orientation rows
already cover `orientation = "match"` against literals on W, P, S, N and R through both builds; they
are hand-written texts, not `recipe.js`'s, so the C rows don't repeat them and they don't replace the
C rows. Their row shape (literal sizes, decoded RGBA equal) is reused.

**The summary line** prints every family's count on every run (taking (3) of
`aspect-orientation-loose-ends`):
`16 table rows, 35 crop rows, 17 replay rows, 26 orientation rows, 2 alpha rows; <n> row failures,
<m> other failures (<labels>), K=1 (cap 78)`, then the replay PSNR per row on its own line.

### DEC-118 (build writes `decisions/DEC-118-<slug>.md`; a new record, not an amendment)

Title: **"A crop recipe from the edit page replays on the original within a stated bound"**.
Frontmatter in DEC-117's shape (`insight.id: DEC-118`, `type: decision`, `project.id: PROJ-017`,
`created_at` the build date, `supersedes: null`, `superseded_by: null`, a comment line saying it
stands beside DEC-110 and DEC-112), with:

```yaml
affected_scope:
  - tests/ddphotos_parity.mjs
  - tests/ddphotos_upscale.py
  - integrations/ddphotos/edit/recipe.js

tags:
  - ddphotos
  - crop
  - replay
  - parity
```

(Block list, HANDOFF trap 7: `scripts/decisions-audit.sh` drops an inline array.)

Body:
- **Context.** The edit page edits the published copy; its CLI recipe (`cliRecipe`) runs on the
  original, which is larger. DEC-110 bounds native against wasm on the same input; nothing bounded
  the copy against the original. Crop is the first page step whose meaning depends on the image's
  size, and it comes in two forms: a preset is an `aspect` (readable and reusable), a placed box is
  four fractions.
- **Decision.**
  1. **Sizes.** The CLI's output from the original and the page's from the copy differ, per side, by
     at most:
     - **box:** 0 px when the original is a whole multiple of the copy, except where `resize`'s
       `side · (n / long)` falls on an exact half in one image and not the other in `f64`, then
       1 px (4 of 1,521,000 cases); 1 px for an original at least 2× the copy; 2 px below 2×.
     - **aspect:** 1 px at any ratio, because the short side is rounded on each image separately.
       Exhaustive over 273,365,360 cases (long sides 1601–4800 and 6400, every short side from a
       third of the long side up).

     These are driven maxima, with the original made from the copy by `resize` mode `max` to
     1600 px and ratios 1:1, 4:5, 3:2 and 16:9 in both orders. The box bound below 2× is measured
     over a 3.1M-case sweep, not proven.
  2. **Orientation.** A preset that follows the photo writes `orientation = "match"`. A preset on a
     square copy, and 1:1 on any copy, write the aspect as shown, with no key: an original that is
     square on the copy but not in fact (|W − H| small enough to round away) would otherwise
     replay in the other orientation.
  3. **Framing.** A box's edges are each within DEC-112's bound of their exact position on both
     images. An aspect is centred on each image separately, so its framing may differ by up to half
     a copy pixel.
  4. **Pixels.** On the harness's Pillow-made original, the replay's PSNR (RGB, shared region) is at
     least **53.5 dB for an aspect** and **59 dB for a box**: the lowest measured on each form's rows
     (53.95 and 59.46), rounded down to 0.5 dB, with no further headroom. A one-copy-pixel framing
     shift reads at most 51.71 dB on an aspect row and 52.77 dB on a box row, below both floors. The
     floors hold for Pillow 12.3.0's `LANCZOS` upscale; a real photo's original has detail the copy
     lacks and reads lower, so they are not a promise about real originals.
  5. **A smaller file.** `resize` mode `max` never upscales, so on a file smaller than the copy the
     CLI's output is smaller than the page's, or its watermark is refused. The bound applies to an
     original at least as large as the copy.
- **Consequences.** The harness asserts exact sizes on its integer-4× rows (none of them is at a
  tie), rule 1's bound and rule 4's floors on every replay row, and a framing-shift control that
  must turn every replay row red. Changing the floors, the bound, or the rule in 2 needs a new
  measurement and an edit here. The copy-maker caveat stands: ddphotos' photogen makes the real copy,
  not crustyimg's `resize`; if it rounds differently, rule 1's non-integer bounds must be re-driven
  against it.


## Inputs

- **Files to read:**
  - `integrations/ddphotos/edit/recipe.js`, `worker.js`, `edit.js` — the page code this changes
    (`edit.js` read-only: it needs no change).
  - `tests/ddphotos_parity.mjs`, `tests/ddphotos_decode.py`, the `ddphotos-parity` recipe
    (`justfile:714–729`) — the harness and how CI runs it.
  - `src/operation/crop.rs:218–243` (`rect_edges`, `aspect_size`), `src/operation/mod.rs:999–1009`
    (`resize` mode `max`), `:1252–1261` (`follow_orientation`), `:1310–1332` (`placement`),
    `:2097–2117` (the follow table the harness copies) — the arithmetic the model ports. Read only.
  - `decisions/DEC-110`, `DEC-112` (rule 8), `DEC-114` (rule 5) and `DEC-117` (the frontmatter
    shape).
  - `specs/done/SPEC-163-…md` "The page port" — `followOrientation` as written.
- **External APIs:** none. **Tools:** Node, Python with Pillow 12.3.0 (the harness venv), wasm-pack,
  a Rust toolchain; all already used by `just ddphotos-parity`.
- **Related code paths:** `integrations/ddphotos/edit/`, `tests/`.

## Outputs

- **Files created:**
  - `tests/ddphotos_upscale.py` — makes a replay original with Pillow (and the N5 shifted one).
  - `decisions/DEC-118-a-crop-recipe-from-the-edit-page-replays-on-the-original-within-a-stated-bound.md`.
- **Files modified:**
  - `integrations/ddphotos/edit/recipe.js` — the crop model, `planEdit` on the oriented and cropped
    size, the crop step in `pixelRecipe`, `loadPhoto`.
  - `integrations/ddphotos/edit/worker.js` — `load` returns `loadPhoto({ info, transform }, source)`.
  - `tests/ddphotos_parity.mjs` — `loadWasm` returns `{ transform, info }`; `recipeFor` and `runTable`
    take the context and a row list; the oriented size via `loadPhoto`; the transpose deleted; the P5
    relabels; the module checks; CROP_TABLE, REPLAY_TABLE, the model, placement and golden checks;
    `judgeReplay`, `psnr`, B11–B19, N4, N5; the summary line.
  - `projects/PROJ-017-…/stages/STAGE-060-…md` — at ship, as usual.
- **Not modified (asserted by the build's file list):** anything under `src/`, `tests/fixtures/`,
  `edit.js`, the `justfile`, `CHANGELOG.md`, `integrations/ddphotos/README.md`, the recipe corpus.
- **New exports** (`recipe.js`): `aspectSize(a, b, w, h)`, `followOrientation(a, b, w, h)`,
  `presetCrop(a, b, photo)`, `swapCrop(crop, photo)`, `cropBox(crop, photo)`, `loadPhoto(wasm,
  source)`; `planEdit(photo, form, crop = null)` gains its third argument and a `crop` field.
  (`tests/ddphotos_parity.mjs`): `judgeReplay`, `psnr`.
- **Database changes:** none.

## Acceptance Criteria

- [ ] **AC-1 The page takes a photo's size from its upright render.** `loadPhoto` returns R's size
  as 1071×1600 (its stored size is 1600×1071) and the format of the stored file; the worker's `load`
  returns `loadPhoto({ info, transform }, source)`. TABLE rows 15–16 pass at 803×1200 and 723×1080
  with the transpose deleted.
- [ ] **AC-2 An untouched preset is an aspect that follows the photo.** On a non-square photo a
  preset other than 1:1 writes `aspect = "a:b"` and `orientation = "match"`; on a square photo, and
  for 1:1, it writes the aspect alone; no preset writes `gravity`. Sizes as C1–C23.
- [ ] **AC-3 A swapped preset is the other orientation, as written, with no key** (C24, C25).
- [ ] **AC-4 A placed box is four fractions that read back to its pixel edges.** Each is the shortest
  text that reads back as the same number; every box on sides 1071 and 1600 reads back to the same
  edges with `x + width ≤ 1`; C26–C34 sizes.
- [ ] **AC-5 The crop runs right after `auto-orient` and before `resize`,** in the pixel, CLI and
  preview recipes (goldens; C31; C32).
- [ ] **AC-6 The page predicts exactly what wasm makes, in both forms**: `planEdit`'s `outW × outH`
  equals the output on every C and R row; `aspectSize` and `followOrientation` equal their literal
  tables; a preset's `cropBox` is where the engine places the aspect.
- [ ] **AC-7 A crop re-fits the watermark**: W-refit is 400×1071 with mark 40, capped; R-refit is
  375×1600 with mark 37, capped (C29, C30, R14, R15).
- [ ] **AC-8 Native and wasm agree on every crop row**: exact, bytes equal, without a resize;
  within DEC-110 with one (C26, C27, C31, C33, C34, C35).
- [ ] **AC-9 The CLI recipe replays on a 4× and a 3.75× original within DEC-118**: R1–R17 sizes,
  `judgeReplay`'s bound and floors; B11–B19.
- [ ] **AC-10 Every control still sees what it guards**: N1, N2 red on exactly their grown lists; N4
  red on exactly C26, C31, C32, C33, C34; N5 red on all 17 replay rows, each on `psnr`; N3 unchanged.
- [ ] **AC-11 The run says what ran**: the summary line counts all five families on a green run; the
  three P5 checks are `P5-pixel`, `P5-cli`, `P5-preview`.
- [ ] **AC-12 Nothing else changes**: TABLE's 16 rows and its three `GOLDEN_*` texts, SPEC-163's 26
  rows and N3, the alpha rows, no file under `src/` or `tests/fixtures/`, `edit.js`; 0 B on the slim
  wasm. Pinned by: TABLE's literals and goldens are byte-for-byte unchanged in the diff (the P5
  relabel changes only the three failure strings at `:406–408`; the transpose deletion removes
  `:359–362`'s two code lines and rewrites the `:142–143` comment, not the rows); N1 and N2's
  lists keep their TABLE entries in order and only append C rows; `ORIENTATION_ROWS`, N3 and the
  alpha code are untouched; `git diff --stat origin/main -- src tests/fixtures
  integrations/ddphotos/edit/edit.js recipes` is empty (`tests/fixtures` holds the recipe corpus goldens); the build's `just wasm-size` (or the
  ddphotos-parity build's size lines) reads brotli 833,619 B on the slim build, as on `main`.
- [ ] **AC-13 DEC-118 is written** with the bounds and floors above, `affected_scope` a block list.

## Failing Tests

All in `tests/ddphotos_parity.mjs`, run by `just ddphotos-parity` (both modes). On `main` the C, R,
model and placement families don't exist, so each "fails" by being absent; the build adds them with
the literals above, and they must fail against `main`'s `recipe.js` (trap 14 row below).

- **modules** (`checkModules`) — asserts: `recipe.js` exports `aspectSize`, `followOrientation`,
  `presetCrop`, `swapCrop`, `cropBox`, `loadPhoto`; `worker.js` contains `return loadPhoto({ info,
  transform }, source);`; `edit.js` and `worker.js` define none of those names, nor `cropStep` or
  `ORIENTED_BASE` (added to `owned`). AC-1, AC-6.
- **TABLE rows 15–16** — asserts: `planEdit(loadPhoto(R))` predicts 803×1200 and 723×1080 with no
  transpose. AC-1.
- **C1–C35** (`runTable` on `CROP_TABLE`) — asserts per row: wasm and native size equal the literal;
  `planEdit` predicts it; the comparator (exact or DEC-110); the mark where given. AC-2, 3, 4, 6, 7, 8.
- **`crop-count`, `crop-resize-set`** — 35 rows ran; the rows without a resize are exactly
  `CROP_NO_RESAMPLE_ROWS`. AC-8.
- **`golden-crop-box`, `golden-crop-square`, `golden-crop-cli`** — C26, C20, C35's texts equal the
  literals. AC-2, AC-4, AC-5.
- **`model-aspect`, `model-follow`, `model-fractions`** — the literal tables and the 1,854,856-box
  readback. AC-4, AC-6.
- **`placement`** — W 1:1, W 4:5, S 2:3: aspect bytes equal box-at-`cropBox` bytes. AC-6.
- **R1–R17** (`replayRows`) — page, CLI and `planEdit` sizes equal the literals; the mark where
  given; `judgeReplay` green. AC-7, AC-9.
- **B11–B19** (`--self-test`) — `judgeReplay` and `psnr` on synthetic input. AC-9.
- **N1, N2, N4** (`--self-test`, `runTable` on TABLE + CROP_TABLE, native-side text mutation) — red
  rows equal, in order: N1 `{3,4,7,8,11,12,13,14,16,C29,C30,C35}`; N2 `{4,8,12,13,14,16,C35}`; N4
  `{C26,C31,C32,C33,C34}` (driven). AC-10.
- **N5** (`--self-test`, `replayRows(ctx, { shift: true })`) — red rows `{R1…R17}`, each failure on
  `psnr`. AC-10.
- **N3** — unchanged. **summary line** — prints the five family counts. AC-11.
- **`P5-pixel`, `P5-cli`, `P5-preview`** — the relabelled checks (`ddphotos_parity.mjs:406–408`).
  AC-11.

### Mutation table

All driven on the prototype through the full harness (`node tests/ddphotos_parity.mjs` main mode
unless it says `--self-test`). "Red" lists every failing row or check; everything else in the run
stayed green. Each mutant is applied to `recipe.js` only, which both the native and the wasm side
read, so only a literal catches it (SPEC-163's verify rule).

- **M1** `recipe.js` `followOrientation`: `if (w > h) return [long, short];` → `if (w >= h) return
  [long, short];`. Red: `model-follow` only. *Driven.* Every page path that calls it has a non-square
  photo (a square never follows, correction 2), so the C and R rows can't see it; the literal table
  is the guard.
- **M2** the same function, the two returns swapped (`[short, long]` for `w > h`, `[long, short]` for
  `w < h`). Red: C2, C3, C4, C6, C7, C8, C10, C11, C12, C14, C15, C16, C24, C25, C35; R2, R3, R4, R5,
  R6, R7, R10, R17; `golden-crop-cli`, `model-follow`, `placement`. *Driven.*
- **M3** `aspectSize`: half-up → floor (`Math.floor((w * b) / a)` and `Math.floor((h * a) / b)`).
  Red: C2, C3, C6, C7, C10, C11, C14, C15, C18, C19; R2, R3, R5, R6, R7, R17; `model-aspect`,
  `placement`. *Driven.* (C18/C19 are S 4:5 and 5:4: the square rows in both orders catch it.)
- **M4** `loadPhoto`: `const shown = wasm.info(bytes);` → `const shown = wasm.info(source);` (the
  stored size; trap 10). Red: 15, 16, C14, C15, C16, C27, C30; R7, R12, R15. *Driven.* C30 reads
  560×1071 with mark 43, uncapped (size, mark and `capped` all differ). C13 (R 1:1) stays green:
  1:1 is the same either way.
- **M5** crop after resize, in `cliRecipe` only: `let r = pixelRecipe({ ...edit, resize:
  edit.outLong });` → `let r = pixelRecipe({ ...edit, crop: null, resize: edit.outLong });` plus
  `if (edit.crop) r = r.replace(/(op = "resize"\nmode = "max"\nwidth = \d+\n)/, `$1\n[[step]]\nop =
  "crop"\n${edit.crop}`);`. Red: C31; R1, R2, R10, R11, R12, R14, R16; `golden-crop-cli`. *Driven.*
  Green, because crop and resize commute on them: C35 (P 3:2 at 1080: 723×1080 then 3:2 is 720×1080
  either way) and R3–R9, R13, R15, R17 (a preset or a full-height box whose long side is the photo's,
  resized to that same long side). The golden and the box rows are the guard.
- **M6** box fractions `toFixed(4)` (all four in `cropStep`). Red: `golden-crop-box`,
  `model-fractions`. *Driven.* No size moves on these sides.
- **M7** an untouched preset emits a box: `presetCrop` returns `{ box: cropBox({ aspect: [a, b],
  follow: … }, photo) }`. Red: C24, C25 (`swapCrop` of a box throws, so the row fails as
  "threw"), R10; `golden-crop-square`, `golden-crop-cli`. *Driven.*
- **M8** the key dropped: `cropStep` returns `` `aspect = "${a}:${b}"\n` `` alone. Red: C2, C7, C8,
  C11, C12, C15, C16, C35; R2, R5, R6, R7; `golden-crop-cli`, `placement`. *Driven.* (P's 4:5 row C6
  stays green: 4:5 is already portrait order.)
- **M9** a square follows: `presetCrop`'s `follow: a !== b && photo.width !== photo.height` → `follow:
  a !== b`. Red: `golden-crop-square` only. *Driven.* Same pixels on a square copy; the golden is the
  guard (correction 2's transposition needs a non-square original that publishes square, which no
  row has).
- **M10** `cropBox` centring `Math.floor` → `Math.ceil` (both). Red: `placement` only. *Driven.*
- **M11** `planEdit` ignores the crop: `const box = cropBox(crop, photo);` → `cropBox(null, photo)`.
  Red: every C row except C17 (S 1:1, the whole square), so C1–C16, C18–C35; and R1–R17. *Driven.*
- **N4** (`--self-test`): `t.replace("width = 0.5\n", "width = 0.500625\n")` on the native text. Red
  exactly C26, C31, C32, C33, C34, on `dims` (C26 natively 601×482 at 600 → the comparator's
  `dims`). *Driven.*
- **N5** (`--self-test`): the original's content moved left `round(factor)` px, size kept. Red
  exactly R1–R17, each `psnr`; the shifted PSNRs are listed under "What was driven". *Driven.*
- **N1, N2** (`--self-test`): the grown lists above. *Driven.*
- **Trap 14, `main`'s three page files restored verbatim** (`git show origin/main:…` over
  `recipe.js`, `worker.js`, `edit.js`): the run stops at the module check, red on `recipe.js lacks
  aspectSize`, `… followOrientation`, `… presetCrop`, `… swapCrop`, `… cropBox`, `… loadPhoto`, and
  `worker.js's load does not return loadPhoto({ info, transform }, source)`; exit 1. **`worker.js`
  alone restored:** red on the `worker.js` line only. **`recipe.js` alone restored:** the six
  "lacks" lines and `worker.js imports loadPhoto, which recipe.js lacks`. *Driven.*
- **The likely wrong-reason failures** (trap 13): a row that fails as "threw" (a crop that rounds to
  0 px, or an `x + width` above 1 that the engine refuses) is red for the wrong reason unless the
  mutation meant it. The build quotes the failure message of every red row it records.

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-110` — native against wasm on the same input: exact without a resize, ≤ 1/255 on ≤ K px with
  one. The C rows use it unchanged; K stays 1 (the prototype's largest count was 1).
- `DEC-112` — crop's fractions and their rounding (each edge on its own), and rule 8 (the
  orientation key). The model ports both.
- `DEC-114` rule 5 — fractions written as the shortest text that reads back; `cropStep` does the same.
- `DEC-118` (new, written by this build) — the replay bound and floors.
- `DEC-113` — a geometry step acts on the buffer it receives: why `auto-orient` must run before
  the crop, and why the page plans from the oriented size.

### Constraints that apply

- `test-before-implementation` — the rows and checks above are the failing tests; build them first.
- `clippy-fmt-clean` — covers `tests/**`; the JS and Python files are not linted by it, but CI's
  ddphotos-parity job also runs `just wasm-clippy` (unaffected: no Rust change).
- `no-new-top-level-deps-without-decision` — none added: Pillow is already the harness's pinned dep.

### Prior related work

- `SPEC-142` (shipped, PR #234) — the harness, its comparator, B1–B10, N1–N2; deferred the replay and
  the one-pixel crop control to this spec.
- `SPEC-163` (shipped, PR #272) — the orientation key, its page port, P/S/N fixtures, the 26 rows
  and N3.
- `SPEC-145` — crop in fractions; the near-tie lesson (trap 17).

### Overlap with open work

From `gh pr list --state open --json number,title,files` on 2026-11-26: one open PR, **#237** (heic
via `heifer`, parked). Its files: `.github/workflows/ci.yml`, `CHANGELOG.md`, `Cargo.lock`,
`Cargo.toml`, `dist-workspace.toml`, `docs/licensing.md`, `fuzz/Cargo.toml`,
`fuzz/fuzz_targets/heic_decode.rs`, `justfile`, `src/image/heic.rs`, `src/image/mod.rs`,
`src/image/sniff.rs`, `src/source/mod.rs`, `tests/fixtures/heic/cmyk_icc_16x16.heic`,
`tests/fuzz_regressions.rs`, `tests/input_heic.rs`. **No overlap** with this build's files (it
touches neither the `justfile` nor `CHANGELOG.md`). **Rebase onto `main` before each push and
re-run the map after each rebase.**

### Out of scope (for this spec specifically)

- The crop mode, the draggable box, preset buttons, keyboard and touch handling, and the CLI
  panel's wording: STAGE-060's "Crop on the page: the crop mode" entry.
- The page README's controls text and the three-server browser round: STAGE-060's "Test-bed round
  and README".
- `rotate`/straighten on the page: STAGE-060's "Straighten on the page".
- A size gate on the slim wasm bundle: the STAGE-060 finding entry.
- Any Rust change, including a wasm `ImageInfo.orientation` getter.
- Recipe sidecars, any save flow, flip/pad/trim on the page, and ddphotos code (STAGE-060
  `explicitly_does_not`).
- (1) and (2) of `aspect-orientation-loose-ends` (a message and an `edit` test row). This spec takes
  only (3), the summary line; ship updates the idea's notes.

### Docs and CHANGELOG

- **No doc changes.** No other surface describes what the page plans for an EXIF-rotated photo:
  `/usr/bin/grep -rln -i "rotat\|orientation\|sideways\|stored size"` over `integrations/ddphotos`,
  `docs/*.md` and `README.md` finds `integrations/ddphotos/README.md` (only its "No crop, rotate…
  yet" note, `:127`, which stays true until the crop mode) and docs files about the CLI's own
  `auto-orient`, none about the page. The README's watermark sentence (`:112`, "made smaller when
  the text would not fit a portrait or narrow photo") becomes true for rotated JPEGs too, and needs
  no edit. This spec adds no behaviour sentence to any doc; its only new prose is code comments and
  DEC-118.
- **No CHANGELOG line.** The edit page is not a released surface: `CHANGELOG.md` has never logged a
  page change (its only page mention is the slim build's "See `integrations/ddphotos/` for the
  first user"). The oriented-size fix is user-visible only on the page. The crop itself gets its line,
  if any, with the crop mode.

## Notes for the Implementer

- **Build order and checkpoints** (long cycles must checkpoint): (1) `recipe.js` model + `worker.js`,
  with the module check, TABLE green and the transpose deleted → commit and push; (2) CROP_TABLE,
  goldens, model and placement checks → commit and push; (3) the upscale script, REPLAY_TABLE,
  `judgeReplay`, `psnr`, B11–B19 → commit and push; (4) N1/N2 lists, N4, N5, summary line, P5
  relabels → commit and push; (5) DEC-118; (6) the mutation table re-driven and recorded.
- **Run** `just ddphotos-parity` (it builds a fresh release binary and slim wasm into
  `target/ddphotos-parity/`, then runs both modes). Redirect output to a file and read `$?`; a pipe
  reports the pipe's code. The prototype's full main run took ~27 s after the builds.
- **`runTable` and `recipeFor` take the context**, and `recipeFor` runs inside the row's `try`, so a
  `recipe.js` that throws on one row fails that row ("threw: …"), not the run. Do the same per
  replay row. M7 depends on it.
- **`loadWasm` returns `{ transform, info }`**, and `ctx` carries `info`. `info()` returns a
  wasm-side object: call `.free()` (`loadPhoto` does).
- **"pub" rows** pass `size: String(Infinity)`: `Number("Infinity") || longEdge` is `Infinity`, so
  `outLong` is the crop's own long edge and there is no `resize`. (The page's own "as published" is
  the photo's long edge, which gives the same plan for every C and R row: the crop's long edge is
  never above the photo's.)
- **The replay's R original** is a JPEG with R's EXIF block: Pillow's `im.getexif().tobytes()` into
  `save(..., exif=...)`. Without it R7, R12 and R15 replay sideways.
- **N5's shift** moves content left by `round(factor)` px and repeats the last columns, so the size
  is kept. Don't shift right: on R1 and R9 that cancels the aspect's half-pixel centring offset
  (correction 8).
- **Comments** read behaviour-first and name no spec, stage or DEC id (the harness's existing
  comments are the model).
- **Verify includes** a clean `just ddphotos-parity` (fresh target dir), the mutation table
  re-driven, and the files list built from `git diff --name-only`. The CI job is
  `ddphotos-parity` (advisory, ubuntu-latest); its log must show the summary line with all five
  counts.
- **Space:** the 4× originals are written into the harness's own `mkdtemp` folder and removed on a
  green run; a red run keeps them (as TABLE's outputs).

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
