---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-164
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
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
  created_at: 2026-11-27

references:
  decisions:
    - DEC-110
    - DEC-112
    - DEC-113
    - DEC-118
  constraints:
    - test-before-implementation
  related_specs:
    - SPEC-161
    - SPEC-163

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-060's <capability>". Optional; null is acceptable.
value_link: "The tested half of STAGE-060's \"crop on the ddphotos edit page\" crop mode: the crop state the page's buttons and box drive, the CLI panel's smaller-file warning (DEC-118 rule 5), and the first check that loads the edit page in a browser in CI"

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
      date: 2026-11-27
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 7564921
      tokens_breakdown:
        input: 102
        cache_creation: 189694
        cache_read: 7317842
        output: 57283
      estimated_usd: 6.28
      rates: "Opus $5/$25 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "Measured after the frame push, from the session transcript (found by content), deduped by message.id (max output_tokens); 50 messages, all claude-opus-5-5, no subagents. Excludes this cost commit's own turn."
    - cycle: design
      date: 2026-11-27
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 14386045
      tokens_breakdown:
        input: 146
        cache_creation: 256751
        cache_read: 14016181
        output: 112967
      estimated_usd: 11.44
      rates: "Opus $5/$25 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "Measured after the design push, from the session transcript (found by content), deduped by message.id (max output_tokens); 73 messages, all claude-opus-5-5, no subagents. Excludes this cost commit's own turn."
    - cycle: build
      date: 2026-11-27
      model: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 17895068
      tokens_breakdown:
        input: 174
        cache_creation: 252184
        cache_read: 17556625
        output: 86085
      estimated_usd: 7.5
      rates: "Sonnet $3/$15 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "Measured before the final cost commit, from the session transcript (found by content), deduped by message.id (max output_tokens); 86 messages, all claude-sonnet-5-5, no subagents. Excludes the turns after this measurement (the cost commit, the cycle advance and the report)."
    - cycle: punchlist
      date: 2026-11-27
      model: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 2577337
      tokens_breakdown:
        input: 66
        cache_creation: 66263
        cache_read: 2499687
        output: 11321
      estimated_usd: 1.17
      rates: "Sonnet $3/$15 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "Measured just before the docs commit, from the session transcript (found by content), deduped by message.id (max output_tokens); 32 messages, all claude-sonnet-5-5, no subagents. Excludes the docs commit's own turn and the final report."
    - cycle: punchlist-2
      date: 2026-11-27
      model: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 2059972
      tokens_breakdown:
        input: 56
        cache_creation: 60333
        cache_read: 1987878
        output: 11705
      estimated_usd: 1.0
      rates: "Sonnet $3/$15 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "Measured just before the commit, from the session transcript (found by content), deduped by message.id (max output_tokens); 27 messages, all claude-sonnet-5-5, no subagents. Excludes the commit, the push, the CI check and the report."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-164: Crop on the ddphotos edit page: the crop state model, the CLI sentence and a headless page check

## Context

STAGE-060's backlog entry "**Crop on the page: the crop mode**" is the second half of the crop
entry, split at SPEC-161's frame. SPEC-161 (PR #273) shipped everything Node could test: the crop
model in `integrations/ddphotos/edit/recipe.js` (`aspectSize`, `followOrientation`, `presetCrop`,
`swapCrop`, `cropBox`, `planEdit(photo, form, crop)`), the upright photo size from `loadPhoto`, and
36 crop rows plus 17 replay rows within DEC-118. But nothing on the page calls the crop model:
`edit.js` calls `planEdit` with no crop, so a photo owner still cannot crop.

This frame finds the remaining entry larger than [M] once its verification is counted, and splits
it (see "Verdict"). This spec is the first half: everything the crop mode needs that can be proven
without the crop mode's DOM, plus the browser check the second half will add its rows to.

## Goal

Today a photo owner who copies the page's CLI recipe is never told that running it on a file
smaller than the published copy gives a smaller image, or a refused watermark, with no warning.
When this ships, the CLI panel says so, the crop mode's state machine and box geometry exist as
pure, Node-tested functions in `recipe.js`, and CI loads the edit page in headless Chrome for the
first time.

## Frame

*Framed 2026-11-27 on `main` at `ab033e4`, Opus. Every number below was driven in this session
against a fresh release binary and a fresh slim wasm built by `just ddphotos-parity` into
`target/ddphotos-parity/` (deleted at the end), unless it says otherwise. Probes lived in the
session scratchpad and are not committed.*

### Verdict: SPLIT, then GO on this half at [M]

The whole entry is more than [M]. Counted (AGENTS §2, size the verification):
- **Code:** the crop mode in `edit.js` (about 200–250 lines: the mode switch, the box, pointer,
  touch and keyboard handlers, presets, swap, Reset, Done, the size line), about 30–40 lines of
  `index.html` and 80–100 of `edit.css`, and about 60–90 lines of pure functions in `recipe.js`.
- **Verification:** 4 states × about 10 transitions × 4 photo shapes (landscape, portrait, square,
  EXIF-rotated) as a Node table (about 40–50 rows); plus a headless-browser check that does not
  exist yet (a fake ddphotos site, a CDP driver, a render counter, pointer, touch and key
  dispatch, about 250–300 lines), its CI step and its negative controls; plus the mutants below,
  each driven with its failure message (trap 13).

That is SPEC-161's size again on top of new browser infrastructure. Split by what can be proven
before the DOM exists:
- **SPEC-164 (this spec), [M]: the crop state model, the CLI sentence and the headless page
  check.** Pure `recipe.js` functions for the crop mode's state and box (below), the Node table
  that drives every transition and the recipe text each state emits, the CLI panel's
  smaller-file sentence, and an in-repo headless-Chrome check of today's page (load, render count,
  the debounce, the CLI panel's text). It ships on its own: the warning DEC-118 rule 5 asks for
  reaches users, and CI gains its first browser check of the edit page, with negative controls.
- **Second entry, [M]: "Crop on the page: the crop mode's DOM".** The Crop button, the box over
  the oriented base, presets, swap, Reset and Done as buttons, Pointer Events and touch, the
  keyboard, ARIA, the live size line and the live watermark note, all calling this spec's
  functions; the README rows; and its rows added to this spec's headless check (drag, keys, the
  render count across a drag and Done, the drag mutants). Filed right after this entry.

### Premise check (evidence)

- **Nothing on the page calls the crop model.** `edit.js:12` imports only `cliRecipe`,
  `pixelRecipe` and `planEdit`; `readEdit` (`edit.js:188–199`) calls `planEdit(photo, {...})` with
  no third argument. `/usr/bin/grep -n -E 'presetCrop|swapCrop|cropBox' edit.js` finds nothing.
- **The worker's `load` returns the upright base, and the page discards it.** `worker.js:40` is
  `return loadPhoto({ info, transform }, source);`. `loadPhoto` (`recipe.js:217–226`) renders
  `ORIENTED_BASE` (`auto-orient` at preview quality, JPEG) at the copy's full size and returns
  `{ width, height, format, bytes }` from `info` of that render, so the size is upright; the
  reply's `bytes` are transferred (`worker.js:103`). The page reads only `meta.width` and
  `meta.height` (`edit.js:327–328`); `meta.bytes` is dropped. Until the first preview arrives the
  page shows the published file itself (`edit.js:316`).
- **Parity on `main`:** `just ddphotos-parity`, exit 0:
  `16 table rows, 36 crop rows, 17 replay rows, 26 orientation rows, 2 alpha rows; 0 row failures, 0 other failures, K=1 (cap 78)`.
  Self-test: B1–B19, N1 to N5 all PASS on their expected rows.
- **No CI check loads the edit page in a browser.** The only headless-browser check in this repo is
  `tests/demo_smoke.mjs` (`just demo-smoke`, CI's `pages.yml` "build + browser smoke"). It drives
  the demo, not the edit page, over the DevTools Protocol with no driver, and `pages.yml` does not
  trigger on `integrations/**`. CI's `ddphotos-parity` job runs the page's recipes in Node only.
  The test bed (`~/claudia/web/jysites/fogcityphotosandbeyond/crustyimg-ddphotos-testbed`, no git
  remote) drives the page in Playwright's headless Chromium (`scripts/browser-check.mjs`, about 40
  checks on dev, static and apache), but only when the maintainer runs `just retest`; it is not in
  CI.

### The coordinate chain, driven

Three spaces: **screen** (CSS px, `clientX`/`clientY`), **displayed image** (the `<img>`'s box),
and **copy** (pixels of the upright copy, `photo.width × photo.height`, which is also the oriented
base's natural size). The model and the recipe work in copy pixels only.

1. **Screen → displayed:** `dCss = clientX − startClientX` (and y). No rounding. The page measures
   the ratio `r = img.getBoundingClientRect().width / img.naturalWidth` at each `pointerdown`
   (the rect changes when a phone rotates). `edit.css:217–222` sizes the image with `max-width:
   100%`, `max-height: 75vh` and `object-fit: contain` with auto width and height, so the box keeps
   the image's ratio and the rect is the content (no letterbox inside the rect): one `r` serves
   both axes. Owner: `edit.js` (DOM reads).
2. **Displayed → copy:** `dCopy = Math.round(dCss / r)`, once per pointer move, applied to the box
   as it was at `pointerdown` (delta from the start, never accumulated per move). **This is the
   only place screen rounding happens.** Owner: a pure function in `recipe.js` (below).
3. **Copy box → recipe:** the box is integers `{ x, y, w, h }` in copy pixels; `cropStep`
   (`recipe.js:107–118`) writes `x / width` and so on, and the engine's `rect_edges`
   (`src/operation/crop.rs:218–225`) rounds each edge back. No rounding on the page. **Driven
   exhaustively: 0 of 1,280,800 (x, length) pairs on a 1600 side and 0 of 574,056 on a 1071 side
   come back to a different edge**, so the recipe always lands on the box the user saw.
4. **Copy → screen (drawing the box):** the box is drawn in copy pixels inside an overlay sized to
   the image's natural size and scaled by CSS, as SPEC-161's frame ruled, so drawing never rounds.

Driven over W (1600×1071), P (1071×1600) and R (stored 1600×1071, upright 1071×1600), at a 375 px
phone (viewport 375×812) and a 1200 px desktop (1200×900), with the CSS limits above:
- W on the phone: `r = 0.23438`, 4.27 copy px per CSS px. With integer screen pixels a drag reaches
  every 4th or 5th copy pixel, so **the worst copy-pixel error from screen rounding is 2 px**
  (nearest reachable position); on a DPR-3 touch screen (1/3 CSS px events) it is 0–1 px.
- W on the desktop: `r = 0.63025`, worst 1 px. P and R, both views: `r` 0.350 or 0.422, worst 1 px.
- **A mapping that converts the absolute pointer position instead of the delta** (a mutant) moves a
  box that is only clicked, not dragged, by up to 2 copy px on the phone (1 px elsewhere).
- Keyboard moves are exact (one copy pixel per arrow), so any pixel the drag cannot reach the
  keyboard can.

**`recipe.js` should gain** (pure, Node-testable; the parity harness's `checkModules` must add
each name to `owned` so `edit.js` cannot redefine it):
- `toCopyDelta(dCss, ratio)`: step 2, `Math.round(dCss / ratio)`.
- `moveBox(box, dx, dy, photo)`: the box moved, clamped inside the photo, size unchanged.
- `resizeBox(box, handle, dx, dy, photo)`: one of 8 handles (4 corners, 4 edges) moved, clamped
  inside the photo and to `MIN_CROP` per side, with the opposite edge fixed. Alt+arrow is
  `resizeBox(box, "se", ±step, ±step...)` on one axis.
- `MIN_CROP`: the smallest side, in copy pixels (see "The minimum box").
- `cropReducer(state, action, photo)` → state: the state machine below, returning the `crop`
  argument `planEdit` takes, built only through `presetCrop`, `swapCrop` and `cropBox`.
- `sizeLine(edit, photo, crop)`: the size line's text from `planEdit`'s output and `cropBox`.
None of these adds a recipe rule: the recipe text still comes only from `cropStep`.

### The state machine, and the recipe each state emits

The `crop` value `planEdit` takes is the state. Four states:
- **S0 no crop:** `crop = null`. Emits no `crop` step.
- **S1 untouched preset** `a:b`: `presetCrop(a, b, photo)`. Emits `aspect = "a:b"`, plus
  `orientation = "match"` unless the photo is square or the preset is 1:1 (DEC-118 rule 2).
- **S2 swapped preset:** `swapCrop(S1 or S2, photo)`. Emits the aspect as shown, with no key.
- **S3 box:** `{ box: { x, y, w, h } }`. Emits four fractions.

Transitions (crop mode is a view, not a state: entering or leaving it never changes the state):
- **Preset button `a:b`** from any state → S1 for `a:b`. After a drag (S3) the box is discarded
  and the preset's centred box is shown.
- **Swap** from S1 or S2 → S2 with the shown aspect reversed. **Disabled in S0, S3 and on 1:1**:
  `swapCrop` of 1:1 is 1:1 with no key, which S1 already emits, so the button would do nothing.
  On a **square photo** swap is live (4:5 → 5:4 changes the box) and emits the aspect as shown;
  S1 on a square photo already has no key, so only the aspect changes.
  **Swap twice** stays S2 (`swapCrop` gives `follow: false` again): the user chose an orientation,
  and the recipe keeps it. Pressing the preset button again is the way back to S1. (Maintainer
  question 3.)
- **Drag, resize, arrow key or Alt+arrow** from any state → S3, starting from `cropBox(state,
  photo)` (S0 starts from the whole photo). **A drag that ends exactly on a preset's box stays a
  box** (ruling 1 of SPEC-161's design). A `pointerdown` with no movement, or a move whose
  `toCopyDelta` is 0 on both axes, does not change the state (an S1 stays an aspect).
- **Reset** from any state → S0.
- **Done** leaves crop mode with the state unchanged, clears the in-progress pointer state
  (capture, start position, the active handle), and schedules one finished preview. **Escape** is
  Done.
- **Re-entering crop mode** shows `cropBox(state, photo)`; S1 stays S1 until the box moves.
- **A form change** (size, watermark, format) keeps the crop. Outside crop mode it schedules the
  debounced preview, as today. Inside crop mode the form stays usable but nothing renders; the
  size line and the watermark note update from `planEdit(photo, form, crop)`, and Done renders.
  "Make the copy" is disabled in crop mode (Done first), so a copy is never made mid-drag.
- **The box made smaller than the minimum:** `resizeBox` clamps at `MIN_CROP`; the state is S3 at
  the minimum. A box can never leave the photo: in the frame's prototype an unclamped box gave
  `height = 1.1194029850746268`, which the engine refuses.

On an EXIF-rotated photo (R) every state is in upright pixels: `photo` comes from `loadPhoto`, the
base is the upright render, and the `crop` step runs after `auto-orient` (DEC-113, SPEC-161).

### The minimum box (driven; a finding)

The engine's own minimum is 1 px per side after rounding (`crop.rs:342–400` refuses a crop that
rounds to 0). A 1-copy-pixel box is also safe for DEC-118 rule 5's domain: on any file at least as
large as the copy, `round(a + s) ≥ round(a + 1) = round(a) + 1` for `s ≥ 1`, so no edge pair
collapses. **But a small box with the watermark on is refused by the engine,** which the page
cannot predict:
- Square boxes 1–200 px on W, Medium, native `apply`: "© 2026" is refused for sides 1–7; "© 2026
  Jane Doe" for 1–13, 20–25, 30–35, 40, 41 and 50 (non-monotonic). The largest:
  `watermark refused: the mark would cover 5.08 % of the 50×50 px image (127 of 2500 px); the limit is 5 %`.
- A 1600 × 20 strip, Large: `the 16×3 px mark plus its 32 px margin does not fit inside the 1600×20 px image`
  (the margin scales with the long edge).
- A grid (short side 8–200 step 8, long side ×1, ×2, ×4 or full, both orientations, three texts,
  three sizes; 1,800 runs): refusals at short sides 8–40 only.
- None above 50 px on either sweep. **This is measured over these sweeps, not a bound** (trap 17).

`watermarkSize` caps the mark to 90 % of the width and a tenth of the short side, but not to the
engine's 5 % coverage limit, and the margin is not checked against the short side. Without a crop
the outputs are at least 400 px and these refusals were not reached; a dragged box reaches them.
Recommend: **`MIN_CROP = 64` copy pixels per side** on the page now (above every refusal measured,
and about 15 CSS px on a phone), and the watermark fit filed as its own entry (filed at the end of
STAGE-060). If a refusal still happens, the preview fails with the engine's message through the
existing `Preview failed:` path (`edit.js:227`).

### What wasm renders, and when

One session (photo opened, crop mode entered, a drag, Done, one form change):
- **Load:** 1 `load` (the worker renders the oriented base once, `transform(source, ORIENTED_BASE,
  "jpeg")`, 31–34 ms for 1600×1071 per SPEC-161's frame) and 1 `preview`. **Driven in headless
  Chrome:** `{"load":1,"preview":1}` once the first preview is on screen.
- **Enter crop mode:** 0. The page shows `meta.bytes` as one blob URL made at load.
- **Drag, keys, presets, swap, Reset:** 0. **Driven:** a 10-move touch drag left the count at
  `{"load":1,"preview":2}`, unchanged from before it.
- **Done:** 1 `preview`.
- **A form change outside crop mode:** 1 `preview` per 200 ms debounce window. **Driven:** three
  size changes in one tick posted one preview (`preview` 1 → 2).
- **Make:** 1 `make`, as today.
So a session is `load` 1, `preview` 1 + (1 per Done) + (1 per settled form change), `make` per
click. A drag cannot queue a render because no drag handler calls `schedulePreview`.

**The base is shown at copy size, scaled by CSS.** A ddphotos copy is a 1600 px long-edge WebP, so
the base decodes to at most 1600 × 1600 × 4 B = 10.2 MB in the browser, and 6.9 MB for 1600×1071,
held once for the session beside today's preview image. The worker already holds the source and
the wasm heap; nothing new is held there. The base JPEG's byte size was not measured here.

### Keyboard, touch and screen readers (minimum that ships, in the second entry)

- **Keyboard:** the box is focusable (`tabindex="0"`). Arrow keys call `moveBox` by 1 copy px
  (Shift: 10); Alt+arrow calls `resizeBox` on the bottom-right corner by 1 (Shift+Alt: 10). Crop,
  the four presets, Swap, Reset and Done are real `<button>`s; Done returns focus to Crop.
  ⚠ **Alt+Left and Alt+Right are Back and Forward in Chrome on Windows and Linux.** CDP's key
  dispatch reached the page's `keydown` with `alt: true` in the prototype, but CDP enters below the
  browser's own shortcuts, so a headless check cannot prove the page wins. Design checks whether
  `preventDefault` on the box stops the navigation, and the test bed's round confirms it; if it
  does not, Alt+Up/Down stay and Left/Right need another chord (maintainer, if it comes to that).
- **Roles:** the box is `role="group"` with `aria-roledescription="crop box"`, an `aria-label`,
  and `aria-describedby` pointing at a one-line instruction ("Arrow keys move it, Alt+arrow
  resizes, Shift for 10 pixels"). No `slider`: the box moves in two dimensions. The preset buttons
  carry `aria-pressed` (true only in S1 or S2 for that preset; a drag clears it). Crop carries
  `aria-pressed` for the mode.
- **Live region:** the visible size line updates on every change. A separate visually hidden
  `role="status"` (polite) repeats it only when a change settles: `pointerup`, a key press after
  300 ms without another, a preset, Swap or Reset. Announcing every pointer move would flood a
  screen reader. Not `assertive`: nothing here is urgent.
- **Touch:** Pointer Events only (one path for mouse, touch and pen), `setPointerCapture` on
  `pointerdown`, and `touch-action: none` on the box and its handles only, so a drag that starts
  outside the box still scrolls the page past a 75vh image. Near the image's edge the box stops at
  the edge while the finger keeps going (clamped, delta from the start); moving back brings it
  back only once the finger passes the point where it stopped. Handles get a 44 px hit area;
  `.frame` has `overflow: hidden` (`edit.css:212`), so a handle's hit area must sit inside the
  image or the frame needs padding in crop mode. At `MIN_CROP` on a phone (about 15 CSS px) the
  handles overlap the box: the box's middle moves it, and the handles win on their own area.
  **Driven:** a CDP touch drag on a `touch-action: none` element delivered 10 `pointermove`s of
  type `touch`. **Pinch and two-finger gestures: out of scope.** iOS's edge-swipe Back cannot be
  prevented; the test bed's phone round looks at it.

### The size line and the watermark note

- **The size line** reads `planEdit(photo, form, crop)` for the current box and form, through
  `sizeLine`: "Crop 1067 × 711 of 1600 × 1071 · the copy will be 1067 × 711" (the second part from
  `outW × outH`, so a size below the crop's long edge reads as the resized size). It shows in crop
  mode only, and updates on every box change and form change.
- **"Made smaller to fit this photo"** (`#wm-capped`) shows when
  `planEdit(...).watermark?.capped` is true. Today it is set only in `runPreview` (`edit.js:219`).
  In crop mode it is set with the size line on every change, so it appears while the box shrinks
  and disappears when it grows back; outside crop mode it stays with `runPreview`. Driven example
  from SPEC-161: on R, "© 2026 Jane Doe Photography, all rights reserved" at Medium is capped to
  41 on the full photo; a narrower box caps it further. The note cannot say when the engine will
  refuse a mark for coverage (the finding above).

### The CLI sentence (this spec)

Today the `<details>` panel says (`index.html:147–152`): "This page edits the web copy the site
publishes. The same recipe, run on the original file, makes the same edit from a better source.
File sizes will differ: …". Recommend adding, as its own `<p class="hint">` straight after that
paragraph, with the copy's size filled in by `edit.js` from `photo.width × photo.height`:

> Run it on the original, or on a file at least as large as this copy (1600 × 1071). On a smaller
> file the CLI makes a smaller image than this page did, and may refuse the watermark, without
> warning.

It shows on every result, crop or not: `cliRecipe` always resizes to the page's long edge and
`resize` mode `max` never upscales (DEC-118 rule 5), so it applies to a plain resize too. Evidence
from SPEC-161's verify: an 800×536 original with W, 3:2 at 1080 and the watermark gave the page
1080×720 and the CLI 800×533, exit 0, with no warning. The size is filled in on the upright size,
so on R it reads 1071 × 1600.

### What is testable where

- **Node (`tests/ddphotos_parity.mjs`, this spec):** the new pure functions with literal expected
  values (trap 19: both sides of the page harness read `recipe.js`, so only a literal catches a
  mutant in it); the reducer's table (every transition × W, P, square, R, with square and
  orientation rows in both orders, trap 20) and the recipe text each state emits through
  `planEdit` and `pixelRecipe`/`cliRecipe`; `toCopyDelta` at the driven ratios; `MIN_CROP`
  clamping; `sizeLine`'s text; `checkModules`' new `owned` names.
- **Headless Chrome (a new in-repo check):** this spec: the page loads against a fake site, the
  worker posts `load` 1 and `preview` 1, the debounce coalesces, the CLI panel shows the new
  sentence with the photo's upright size (W and R), and `cliRecipe`'s text in the panel. The second
  entry adds: pointer and touch drags (the delta and ratio wiring), arrow and Alt+arrow keys, the
  render count across a drag and Done, the size line, `aria-pressed`, focus return.
- **Only the test bed's round:** the three real servers (dev, static, apache) and their MIME
  types, a real photogen-built site, Playwright's Chromium on the maintainer's machine, the 👁
  checklist (how it looks), Alt+arrow against the browser's shortcuts, and any real phone or
  screen reader. No CI check reaches those.

**The headless check: recommend in-repo**, `tests/ddphotos_page.mjs` behind `just ddphotos-page`,
run as a step in CI's existing `ddphotos-parity` job (it already builds `pkg-slim`, and
`ubuntu-latest` ships Chrome, as `pages.yml` relies on). It assembles a fake ddphotos site in a
temp folder (`albums/config.json`, `albums.json`, `<slug>/index.json`, the fixtures, and the page
with `pkg-slim` as `edit/vendor/`, as `ddphotos-edit-install` lays it out), serves it with
`scripts/serve.mjs`, and drives Chrome over the DevTools Protocol with no driver, as
`tests/demo_smoke.mjs` does (reuse its `findChrome` and `CDP` client; design decides whether to
move them into `scripts/lib/`). The render counter is `Page.addScriptToEvaluateOnNewDocument`
wrapping `Worker.prototype.postMessage`, so the page needs no test hook.
- **Prototype, driven** (scratchpad only): loads the page at 375×812 DPR 3, reads
  `{"load":1,"preview":1}`, three size changes → one more preview, a 10-move touch drag of 3 CSS
  px steps → 70 copy px at `r = 0.42875`, and reads back `pixelRecipe`'s text built from
  `recipe.js`. Chrome up and page ready in **5.9 s** on the Mac; the whole run about 7 s. CI cost:
  estimate 15–30 s on `ubuntu-latest` (Chrome's cold start dominates; not measured on CI).
- **Negative control, driven:** the prototype's drag with the display ratio dropped (`MUTANT=noscale`)
  went red: `FAIL drag box x=230, expected 270`, exit 1. For this spec's own rows design drives:
  `schedulePreview` calling `runPreview` directly (the debounce removed) must turn the coalescing
  row red on the preview count, and the CLI sentence removed from `index.html` must turn the panel
  row red on its text.
- **Trap 11:** the check never skips. With no Chrome it exits non-zero (as `demo_smoke.mjs` does),
  and CI's step runs it unconditionally, so a missing browser is red, not a printed skip.
- ⚠ **Incidental, from the prototype:** it measured the ratio against the *finished preview*,
  which after a size change is 800 px wide, and emitted `width = 1` and `height = 1.119…`. The crop
  mode must measure against the oriented base (copy size). That is mutant M2 below.

### Mutants a design can table

Groups: **N** = Node table, **H** = headless check, **T** = test bed only. Reasons marked "driven"
were read off a run in this frame; the rest are "not driven" and design drives them (trap 13).
- **M1 drag delta not divided by the display ratio** (`toCopyDelta` returns `dCss`, or `edit.js`
  passes ratio 1). N for the function, H for the wiring. Driven (H): `FAIL drag box x=230, expected 270`.
- **M2 the ratio taken from the finished preview** (`#preview`'s `naturalWidth` after a resize)
  instead of the base. H only. Driven incidentally: fractions `width = 1`, `height = 1.1194…`.
- **M3 swap emits the orientation key** (`swapCrop` returns `follow: true`). N (literal recipe
  text on S2, on W and on P). Not driven.
- **M4 a `pointerdown` with no move turns S1 into S3.** N (reducer: drag with delta 0), H (click on
  the box, then read the CLI recipe: still `aspect`). Not driven.
- **M5 Done does not clear the pointer state,** so a stray `pointermove` after Done moves the box
  (and turns S1 into S3). H only. Not driven.
- **M6 an arrow key moves one screen pixel** (`toCopyDelta(1, r)`, 4 copy px on the phone) instead
  of one copy pixel. N (`moveBox` by key) and H (ArrowRight at `r = 0.234` moves x by exactly 1).
  Not driven.
- **M7 per-move rounding accumulated** instead of the delta from the start. H, with moves small
  enough to differ: 30 moves of 1 CSS px at `r = 0.42875` give 30 × round(2.33) = 60 against
  round(30 / 0.42875) = 70 (arithmetic, not driven on the page); the prototype's 3 px steps give
  14 per move and might not separate them (trap 20's lesson: pick the input where they differ).
- **M8 a drag queues a render** (`schedulePreview` on `pointermove`). H (the render counter, which
  must read 0 new previews during the drag and exactly 1 after Done). Not driven.
- **M9 the box not clamped** to the photo. N (`moveBox` past the edge), H (drag past the edge).
  Driven incidentally: an unclamped box emitted `height = 1.1194…`.
- **M10 `MIN_CROP` ignored** (`resizeBox` allows 1 px). N. Not driven.
- **M11 the debounce removed** (this spec's row). H (the coalescing row). Not driven.
- **M12 the CLI sentence's size from the stored size** (`info` of the source) instead of the
  upright size. H on R (reads 1600 × 1071 instead of 1071 × 1600). Not driven.
None is T-only; T adds coverage (real servers, real devices), not mutants.

### The wasm size

This spec and the second entry add no Rust and no wasm. `just wasm-size` on `main` (`ab033e4`),
slim build (`--no-default-features --features avif`), as built by `just ddphotos-parity`:
**raw 4,148,451 B, gzip 1,195,748 B, brotli 833,619 B.** (SPEC-161's frame read raw 4,147,159 B
and brotli 833,239 B; the 1,292 B raw growth is SPEC-163's orientation key, not page work.) The
build should show the same three numbers.

### Fold or leave

- **"Test-bed round and README": leave.** It runs in another repository, on three servers, by the
  maintainer, and it also covers straighten, which waits for STAGE-059 (status `proposed`).
  Folding it would add a whole round to this [M]. Two of its wording items move out by this
  frame's recommendation, without moving the entry: the CLI panel's sentence is this spec's (the
  backlog entry already said so), and the README's crop rows go with the crop mode's DOM entry,
  because a behaviour sentence belongs with the change that makes it true (AGENTS §15). It keeps
  the round, the wasm delta and straighten's README text.
- **"Straighten on the page": leave.** It waits for STAGE-059, and it changes the base render
  (`auto-orient` + `rotate`, re-rendered on an angle change), which neither half builds.

### Docs

- **This spec:** `index.html`'s CLI panel gains the sentence above. `integrations/ddphotos/README.md`'s
  "What it can do" **CLI** row gains one clause: "Run it on the original or a file at least as
  large as the copy: a smaller file gives a smaller image, or a refused watermark." Nothing else:
  the page offers no crop yet, so the "No crop … yet" limit (`README.md:127`) stays true.
- **The crop mode's DOM entry:** a **Crop** row in "What it can do" (presets 1:1, 4:5, 3:2, 16:9
  following the photo, swap, drag, keyboard), the "No crop, rotate, flip, pad, trim …" limit
  reworded to drop crop, and `index.html`'s lede ("Make a resized, watermarked or re-encoded copy")
  gaining "cropped".
- **The test-bed entry:** the round's results and the wasm delta.
- **Behaviour sentences this spec's build must drive on every exit path** (AGENTS §15):
  1. "On a smaller file the CLI makes a smaller image than this page did": run `cliRecipe` from W
     at 1080 on an 800×536 original, plain resize and with 3:2; read the size (`800×533` for 3:2).
  2. "… and may refuse the watermark": run it with the watermark on a file small enough to refuse,
     and quote the refusal and the exit code.
  3. "without warning": quote the exit code (0) and stderr (empty) of path 1.
  4. "at least as large as this copy (W × H)": the panel on W and on R shows the upright size.
  The second entry's build drives its Crop row the same way.

### Sizing

[M] for this half: about 60–90 lines of pure `recipe.js`, a few lines of `index.html` and `edit.js`
(the sentence and its size), a Node table of about 40–50 reducer and geometry rows plus literals
for the new functions, a new headless check of about 250–300 lines with a fake-site builder,
three negative controls, one CI step and a `just` recipe. No DEC, no Rust, no new fixture
(the existing `tests/fixtures/ddphotos/` photos cover W, P, square and R).

The second entry is also [M]: about 250–350 lines of DOM and CSS, and about 10–15 headless rows
on this spec's check, with M1, M2, M4–M9 driven there.

### Scope

In (this spec):
- `recipe.js`: `toCopyDelta`, `moveBox`, `resizeBox`, `MIN_CROP`, `cropReducer`, `sizeLine`.
- The Node table and literals for them in `tests/ddphotos_parity.mjs`, and their names in
  `checkModules`' `owned` list.
- The CLI sentence in `index.html`, its size filled in by `edit.js`, and the README's CLI clause.
- `tests/ddphotos_page.mjs` and `just ddphotos-page`: the fake site, the CDP driver, the render
  counter, the rows on today's page, and the negative controls; a step in CI's `ddphotos-parity`
  job.

Out, and where each lives:
- The crop mode's DOM (Crop button, the box, presets, swap, Reset, Done, pointer, touch and
  keyboard handlers, ARIA, the size line and the live watermark note on the page) and its headless
  rows: the new STAGE-060 entry "Crop on the page: the crop mode's DOM", right after this one.
- The README's Crop row and the reworded limit: that entry.
- The three-server round and the wasm delta: "Test-bed round and README".
- `rotate` on the page: "Straighten on the page".
- The watermark's fit on small or thin crops (coverage and margin): the new STAGE-060 finding entry.
- A size gate on the slim bundle: its existing STAGE-060 entry.
- Save flow, sidecars, flip, pad or trim on the page, and ddphotos code: the stage's
  `explicitly_does_not`.
- Pinch and two-finger gestures: not planned.

### Rough build file list

- `integrations/ddphotos/edit/recipe.js`: the six new names
- `integrations/ddphotos/edit/index.html`: the CLI sentence
- `integrations/ddphotos/edit/edit.js`: fill the sentence's size (a few lines)
- `integrations/ddphotos/README.md`: the CLI row's clause
- `tests/ddphotos_parity.mjs`: the reducer and geometry rows, literals, `owned` names
- `tests/ddphotos_page.mjs`: new, the headless page check
- `scripts/lib/` (maybe): `findChrome` and the CDP client moved out of `tests/demo_smoke.mjs`, if
  design shares them (then `demo_smoke.mjs` changes too)
- `justfile`: `ddphotos-page`
- `.github/workflows/ci.yml`: a step in the `ddphotos-parity` job

### Open questions for the maintainer

1. **Split as above (state model, CLI sentence and headless check first; the crop mode's DOM
   second)?** Recommend yes.
2. **The minimum box: `MIN_CROP = 64` copy pixels per side, and the watermark fit filed as its own
   entry?** Recommend yes. Measured refusals stop at 50 px on the sweeps run; the fit fix touches
   `watermarkSize`, which is recipe logic this spec must not add.
3. **Swap twice stays an explicit orientation with no key; pressing the preset again restores
   following?** Recommend yes (it is what `swapCrop` already does).
4. **Form changes in crop mode update only the size line and the note; Done renders; "Make the
   copy" is disabled in crop mode?** Recommend yes.
5. **The headless check lives in this repo, as a step in the advisory `ddphotos-parity` job (not a
   required check yet)?** Recommend yes; make it required once both halves have shipped and it has
   run green for a while.
6. **The README's crop rows go with the crop mode's DOM entry, not the test-bed entry?** Recommend
   yes.

### Maintainer rulings (2026-11-27)

All six recommendations above were accepted as written:
1. **Split.** This spec is the pure state model and box functions with the Node table, the CLI
   sentence with its README clause, and the headless page check proven on today's page. The crop
   mode's DOM is the next STAGE-060 entry.
2. **`MIN_CROP = 64` copy pixels per side.** The watermark's fit on a small or thin crop is its own
   entry.
3. **Swapping twice stays an explicit orientation with no key;** pressing the preset again restores
   following.
4. **In crop mode, form changes update only the size line and the note;** Done renders; "Make the
   copy" is disabled.
5. **The headless check lives in this repo,** as a step in the advisory `ddphotos-parity` job. It
   becomes required once both halves have shipped and it has run green for a while.
6. **The README's crop rows go with the crop mode's DOM entry,** not the test-bed entry.

## Design

*Designed 2026-11-27 on `main` at `de19854`, Opus. Every number below was driven in this session
against a release binary and a slim wasm built into the session scratchpad
(`cargo build --release` and `just wasm-build` with the slim features, each with its own
`CARGO_TARGET_DIR`). The slim build read raw 4,148,451 B, gzip 1,195,748 B, brotli 833,619 B,
the frame's numbers. The prototypes (a candidate `recipe.js`, an independent model, a row
runner, a headless page check) lived in the scratchpad and are not committed.*

### Design corrections to the frame

1. **`MIN_CROP = 64` does not keep a watermarked crop safe from refusals.** This is the frame
   check the prompt put first, driven with native `apply` on the page's own `pixelRecipe`. A 64 px
   short side was swept at the full long edge (W 1600 × 64, P 64 × 1600), plus W 64 × 1071 and
   64 × 64. Each was run at every size the page offers (as published, 1200, 1080, 800 and 400),
   with the frame's three texts, Small, Medium and Large, and all 7 positions: **69 of 1,260 runs
   were refused.** Every refusal is a thin *vertical* strip resized below the box's long edge, for
   example `P 64x1600 @400 -> 16x400 … "© 2026 Jane Doe Photography, all rights reserved"`, which
   gives `watermark refused: the 23×2 px mark plus its 8 px margin does not fit inside the 16×400 px image`.
   `watermarkSize` caps the size to 1 px, but a 1 px font still lays the long text out 23 px
   wide, and a corner mark needs its width plus one margin inside the output's width. Horizontal
   strips (1600 × 64) were never refused. Short sides of 72, 80, 88, 96, 104, 112 and 120 were
   refused 45, 33, 21, 12, 12, 12 and 12 times; 121 was refused 3 times out of 180 runs
   (southeast only); **122 was refused 0 times in 1,260 runs**, and 128, 136, 144, 160, 176 and 192
   were each refused 0 times in 1,260. *Measured over this sweep, not a bound.* The floor depends
   on the text: a 120-character text (the field's `maxlength`) was refused at 192
   (`the 61×2 px mark plus its 8 px margin does not fit inside the 48×400 px image`) and passed at
   320. **No minimum in copy pixels covers every text**, because the output is resized after the
   crop. The ruling stands (the design does not change it): this spec builds `MIN_CROP = 64` and
   puts the finding to the maintainer (question 1 below). Rows that carry the 64 literal: G20,
   G21, G24, G25, G30, X23, X25 and X27.
2. **The display ratios are not the frame's.** Measured in headless Chrome on the page itself
   (`img.getBoundingClientRect()` over `naturalWidth`), the ratios were: W 0.21438 on a 375 × 812
   phone (the frame said 0.23438), W 0.51500 on a 1200 × 900 desktop (frame: 0.63025), P and R
   0.32026 on the phone and 0.42188 on the desktop, and on an 812 × 375 phone held sideways,
   W 0.26260 and P/R 0.17576. The worst copy-pixel error from integer CSS pixels is **2 px on the
   phone for W and for P/R** (the frame said 1 for P/R), **3 px for P/R on the sideways phone**
   (5.69 copy px per CSS px; a case the frame did not drive), and 1 px on the desktop. With 1/3 CSS
   px steps (a DPR-3 touch screen) the worst is 1 px on the phones and 0 on the desktop. Keys stay
   exact.
3. **"at least as large as this copy" reads as the downloaded copy.** The page calls the file it
   makes "the copy" ("Make the copy"), and the panel calls the published file "the web copy". The
   sentence now says "the web copy this page edits".
4. **The drag's start box lives in the reducer's state,** so M5's function side (Done clears the
   drag) and a stray `drag-move` after Done are Node rows (X15), not H only. The DOM entry still
   drives the wiring.
5. **M12 has no stored size to read in `edit.js`.** The page only ever gets `meta` from the
   worker's `load`, so the drivable mutant is in `worker.js`: `load` returns the stored size
   (`info(source)`). H7 catches it.
6. **Removing the whole sentence from `index.html` is red for the wrong reason.** `edit.js` then
   writes into a missing `#cli-size`, `showEditor` throws, and no preview arrives. Driven:
   `FAIL timed out waiting for the first preview on W; the page's error box: Cannot set properties of null (setting 'textContent'); page exceptions: none`.
   The control for the panel row's text is a wording change (a clause dropped), and the "main
   restored" control (trap 14) removes both the paragraph and the line that fills it.
7. **`toCopyDelta` rounds symmetrically and ignores a ratio that is not a positive finite
   number.** The frame's `Math.round(dCss / ratio)` rounds −2.5 to −2 but +2.5 to 3, so a drag
   left and the same drag right moved different distances. It also turned a 0 ratio (an image not
   yet laid out) into `Infinity`.
8. **The thin-strip refusal is the mark's width against the output's width**, not the margin
   against the long edge. The frame's 1600 × 20 example is the other axis: there the mark's height
   plus its margin doesn't fit the height.

### Frame checks, driven

1. **`MIN_CROP = 64`:** does not hold. See correction 1. Put to the maintainer.
2. **The coordinate chain:** my own model of `rect_edges` (`src/operation/crop.rs:217–225`: each
   edge `round(f · side)` on its own, halves up for these non-negative values) read back
   `String(x / side)` for every (x, width): **0 of 1,280,800 boxes on 1600, 0 of 574,056 on 1071,
   0 of 575,128 on 1072.** The negative control writes the fractions with `toFixed(3)` and gives
   800,296 wrong of 1,280,800 on 1600 and 181,436 of 574,056 on 1071. (`toFixed(4)` was 0 wrong on
   1600, too weak to be a control.) Screen error: correction 2.
3. **One ratio serves both axes:** yes. `edit.css:217–222` gives `.frame img` `max-width: 100%`,
   `max-height: 75vh`, `object-fit: contain` and auto width and height, so a replaced element keeps
   its ratio under both limits and the rect is the content. Measured in 9 layouts (W, P and R at
   375 × 812, 1200 × 900 and 812 × 375): the x and y ratios agree to 1e-5 (layout units) in each,
   including a portrait photo in a landscape viewport (P at 1200 × 900, height-limited at 675 px:
   0.42188 both ways) and a landscape photo in a portrait one (W at 375 × 812: 0.21438 and
   0.21437). M2 (the ratio taken from the finished preview) stays the DOM entry's.
4. **The render count:** driven again in the prototype check. `{"load":1,"preview":1,"make":0}`
   once the first preview is on screen, on W and on R, and `{"load":1,"preview":2,"make":0}`
   after three size changes in one tick plus a 600 ms settle. The CI step asserts those exact
   strings (H2, H3, H6), not "at least". **`ubuntu-latest` is now the `ubuntu-26.04` image, and
   its Chrome is found:** `pages` run 45915298934 (2026-11-27, on `main`) logged
   `Image: ubuntu-26.04` and `✓ attached to headless Chrome (google-chrome)`, which is
   `findChrome`'s `/usr/bin/google-chrome`. Chrome took 11.2 s to come up there
   (`✓ serving demo/` at 00:13:48.74, attached at 00:13:59.89).
5. **Swap:** driven through the prototype and the model, which agree. Disabled (the same state
   object comes back): S0 (X8, crop stays null, no crop step); 1:1 (X9, `aspect = "1:1"\n`); S3
   (X17, the box's fractions unchanged). Live on a square: 3:2 → `aspect = "2:3"\n` (X4S),
   2:3 → `aspect = "3:2"\n` (X5S), 4:5 → `aspect = "5:4"\n` (X28S) and 5:4 → `aspect = "4:5"\n`
   (X29S), none with the orientation key. On W, 3:2 then Swap is `aspect = "2:3"\n`; Swap again is
   `aspect = "3:2"\n` with no key (X6W); the preset again restores `aspect = "3:2"\norientation = "match"\n`
   (X7W).

### The six functions (in `recipe.js`, after `loadPhoto`, under a `// ── the crop mode ──` rule)

- **`MIN_CROP`:** `export const MIN_CROP = 64;`. The smallest side, in copy pixels, that a
  resize can make.
- **`toCopyDelta(dCss, ratio) → integer`:** `0` unless `ratio > 0` and finite. Otherwise
  `Math.sign(dCss) * Math.round(Math.abs(dCss) / ratio)`, and never `-0`. `ratio` is the shown
  image's width over its natural width, read by the DOM at `pointerdown`.
- **`moveBox(box, dx, dy, photo) → box`:** a new `{ x, y, w, h }` with `x` clamped to
  `[0, photo.width − w]` and `y` to `[0, photo.height − h]`, size unchanged. It assumes `box` is
  inside the photo, which every box the reducer makes is.
- **`resizeBox(box, handle, dx, dy, photo) → box`:** `handle` is one of `"n"`, `"s"`, `"e"`, `"w"`,
  `"ne"`, `"nw"`, `"se"` and `"sw"`; anything else throws `unknown crop handle <handle>`.
  - Each edge the handle names moves by `dx` (east or west edge) or `dy` (north or south). The
    other edges stay where they are.
  - An east edge is clamped to `[left + minW, photo.width]`, a west edge to `[0, right − minW]`,
    and likewise vertically, where `minW = Math.min(MIN_CROP, box.w)` and
    `minH = Math.min(MIN_CROP, box.h)`.
  - **Past the opposite edge it clamps, it never flips:** a flip would change which handle is
    being dragged mid-drag, and the keyboard has no flip.
  - **A box already smaller than `MIN_CROP`** (a preset on a photo under 64 px on a side, such as
    16:9 on 100 × 100, which is 100 × 56) can grow but cannot shrink further, and a zero delta
    leaves it as it is (G24–G26). On a photo smaller than 64 px the box is the photo's side, so
    that side cannot shrink at all.
- **`cropReducer(state, action, photo) → state`:**
  - **The state is `{ crop, preset, drag }`.** `crop` is the value `planEdit` takes (null, a
    `presetCrop`/`swapCrop` result or `{ box }`). `preset` is the `[a, b]` of the preset button
    that made the crop (null after a drag, a key or Reset), which the DOM uses for `aria-pressed`.
    `drag` is `null` or `{ handle, start }`, where `handle` is `"move"` or a resize handle and
    `start` is the box when the drag began.
  - **`cropReducer(null, { type: "reset" }, photo)` is the starting state**
    `{ crop: null, preset: null, drag: null }`.
  - **It never mutates `state`** (the table deep-freezes every state, so a write throws).
  - **An action that changes nothing returns the same object** (`===`). That is how the DOM
    disables Swap (`cropReducer(state, { type: "swap" }, photo) === state`), with no rule of its
    own.
  - **The actions:**
    - `{ type: "preset", a, b }`: `crop: presetCrop(a, b, photo)`, `preset: [a, b]`, `drag: null`.
    - `{ type: "swap" }`: the same object in S0, on a box and on a 1:1 aspect. Otherwise
      `crop: swapCrop(crop, photo)` with `preset` and `drag` kept.
    - `{ type: "drag-start", handle }`: `drag: { handle, start: cropBox(crop, photo) }`, with the
      crop unchanged.
    - `{ type: "drag-move", dx, dy }`: `dx` and `dy` are copy pixels from the drag's start,
      already through `toCopyDelta`. It returns the same object with no drag in progress.
      Otherwise the new box is `moveBox(start, …)` for `"move"`, or `resizeBox(start, handle, …)`.
      The same object comes back when the crop is a preset or null and the new box equals
      `start`, or when the crop is already that box. Otherwise the result is
      `{ crop: { box }, preset: null, drag }`. So a press with no movement keeps S1 (M4), and a
      drag that comes back to where it began stays a box (SPEC-161's ruling 1).
    - `{ type: "drag-end" }` and `{ type: "done" }`: `drag: null`, or the same object when there
      was no drag.
    - `{ type: "key-move", dx, dy }`: copy pixels (1 or 10), applied to `cropBox(crop, photo)`
      through `moveBox`. The same object when the box can't move (S0, or at an edge); otherwise a
      box with `preset: null`.
    - `{ type: "key-resize", dx, dy }`: the same through `resizeBox(…, "se", …)`, so Alt+Left is
      `{ dx: -1, dy: 0 }`.
    - `{ type: "reset" }`: `{ crop: null, preset: null, drag: null }`, or the same object when it
      is already that.
    - Any other `type` throws `unknown crop action <type>`.
  - **The pointer state is split by what it needs.** The copy-pixel start box lives in the
    reducer (`drag.start`), so Done clearing it is a Node row. The screen-side state stays in
    `edit.js`, because it is DOM: `pointerId`, `startClientX`/`startClientY`, the ratio read at
    `pointerdown`, and pointer capture. A stray `pointermove` that `edit.js` forwards after Done
    does nothing, because there is no `drag` (X15).
  - The reducer builds `crop` only through `presetCrop`, `swapCrop` and `cropBox`; it adds no
    recipe rule, and the recipe text still comes only from `cropStep`.
- **`sizeLine(edit, photo, crop) → string`:** `edit` is `planEdit(photo, form, crop)` for the same
  `crop`. It reads ` · the copy will be ${edit.outW} × ${edit.outH}` after:
  - no crop: `The whole photo, ${photo.width} × ${photo.height}`;
  - a box: `Crop ${w} × ${h} of ${photo.width} × ${photo.height}`;
  - a preset: `Crop ${a}:${b}, ${w} × ${h} of ${photo.width} × ${photo.height}`, with the aspect
    as shown (followed or swapped), so a swap shows in the line.
  The `×` is U+00D7 with a space each side and the `·` is U+00B7, as the page's result summary
  writes them. Literals are rows L1–L8.

Any private helper the build adds to `recipe.js` (the prototype used `clamp`, `sameBox` and
`HANDLE_EDGES`) also goes into `checkModules`' `owned` list.

### The CLI sentence

`index.html`, as its own paragraph straight after the "This page edits the web copy …" one
(`index.html:147–152`):

```html
<p class="hint" id="cli-note">
  Run it on the original, or on a file at least as large as the web copy this
  page edits (<span id="cli-size"></span>). On a smaller file the CLI makes a
  smaller image than this page did, and may refuse the watermark, without warning.
</p>
```

`edit.js` fills it in `showEditor`, on the line after `photo = { … }` (`edit.js:327`):
`$("cli-size").textContent = \`${photo.width} × ${photo.height}\`;`. It reads `photo`, which is
`meta` from the worker's `load`, the upright size (`loadPhoto`), not the stored size (M12). It is
filled once per photo, at load: the size is the web copy's, not the output's, so a size or crop
change never touches it. It shows on every result, crop or not, because `cliRecipe` always
resizes to the page's long edge and `resize` mode `max` never upscales (DEC-118 rule 5).

README, the **CLI** row of "What it can do" (`integrations/ddphotos/README.md:115`), gains at its
end: "Run it on the original or on a file at least as large as the web copy: a smaller file gives
a smaller image, or a refused watermark, with no warning."

### The headless page check

- **`tests/ddphotos_page.mjs`, run by `just ddphotos-page`.** Its arguments are
  `--pkg <slim pkg dir>` (required; there's no default that could point at a stale build) and
  `--keep` (optional).
- **The fake site** is a `mkdtempSync(join(tmpdir(), "ddphotos-page-site-"))` folder:
  - `albums/config.json`: `{ siteName, copyrightOwner: "Jane Doe", encrypted: false }`.
  - `albums/albums.json`: one album, `test`, with `count: 2`.
  - `albums/test/index.json`: two photos, 1 = `photo.webp` (W) and 2 = `photo_rot.jpg` (R), each
    with `src.full` and `src.grid` at `full/<name>`.
  - The two fixtures copied from `tests/fixtures/ddphotos/`.
  - `edit/`: every file of `integrations/ddphotos/edit/`, plus `edit/vendor/crustyimg.js` and
    `crustyimg_bg.wasm` from `--pkg`, as `scripts/ddphotos-edit-install.mjs` lays it out.
  - It is served with `scripts/serve.mjs`'s `startServer`.
- **The driver:** `findChrome`, the `CDP` client and Chrome's start-up (the spawn flags, the
  stderr drain, the 90 s `DevToolsActivePort` wait and its failure message) move out of
  `tests/demo_smoke.mjs` into a new `scripts/lib/chrome.mjs`. It exports `findChrome()`, `CDP`
  and `launchChrome(chromePath, profileDir) → { chrome, cdp, exited }`.
  - `demo_smoke.mjs` imports them. Its behaviour, checks and output don't change, and the long
    comments on the start-up budget move with the code.
  - Only `demo_smoke.mjs` uses them today
    (`/usr/bin/grep -rln -e findChrome -e 'class CDP' -e DevToolsActivePort tests scripts demo .github justfile integrations`).
  - `findChrome` gains one fix both checks need: a `CHROME` that doesn't exist is reported, not
    spawned (driven: without it the spawn's ENOENT crashed the process with a stack).
  - `pages.yml` runs on `scripts/**` and `tests/**`, so its "build + browser smoke" job runs
    `just demo-smoke` on the build's PR. That job must be green, and the build reads its log.
- **Profile:** `mkdtempSync(join(tmpdir(), "ddphotos-page-chrome-"))`, passed as
  `--user-data-dir`. The Chrome flags are `demo_smoke.mjs`'s, including `--no-sandbox` under
  `CI`.
- **The viewport** is `Emulation.setDeviceMetricsOverride` 1200 × 900, DPR 1.
- **The render counter** goes in through `Page.addScriptToEvaluateOnNewDocument`, before any
  navigation, so the page carries no test hook. It wraps `Worker.prototype.postMessage` and
  counts `msg.type` into `window.__renders = { load: 0, preview: 0, make: 0 }`.
- **Waiting:** every wait polls an expression at 100 ms, up to 60 s (120 s for a Make). It fails
  at once when `#error` is visible with class `error`. Its failure message quotes `#error`'s text
  and every `Runtime.exceptionThrown` so far. (Driven: a wait that only timed out took 60 s and
  could not say why.)
- **Cleanup**, on every exit path:
  - Close the CDP socket, kill Chrome and **await its `exit`**, then close the server.
  - Delete the profile with `rmSync(…, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 })`.
    If that still fails, print a warning naming the folder; the warning never changes the exit
    code. (Driven: deleting before Chrome exits failed with `ENOTEMPTY … /Default`, and so did a
    5 × 100 ms retry after a timed-out run.)
  - **Delete the fake site unless `--keep`**, which prints `fake site kept in <path>`. The site is
    rebuilt from committed files in under a second, so nothing in it is evidence, and deleting it
    keeps a failing CI or local run from leaving folders behind.
- **Exit:** 0 only when every row passed. Any failed row, wait or start-up exits 1, including no
  Chrome (`FAIL no Chrome or Chromium found …`), a `CHROME` that doesn't exist, and a missing or
  incomplete `--pkg`. The last line is `ddphotos-page: <n> rows, <f> failed`.

Rows on today's page (the prototype ran all eight green: page ready in 5.2 s cold, the whole run
13.1 s on the Mac, 8–9 s warm):
- **H1 the page loads:** open `/edit/index.html?album=test&photo=1` and wait for
  `#preview.src` to start with `blob:`, `#busy` to be hidden and `#make` to be enabled. Then
  `#error` is hidden, `#editor` is shown and `#engine` reads `Engine ready`. Failure:
  `FAIL H1 page loads: error "<#error text>", engine "<#engine text>"`.
- **H2 renders at load:** `JSON.stringify(window.__renders)` is
  `{"load":1,"preview":1,"make":0}`. Failure:
  `FAIL H2 renders at load: renders <got>, expected {"load":1,"preview":1,"make":0}`.
- **H3 three quick size changes add one preview:** in one evaluation, set `#size` to `1200`,
  `1080` and `800`, each followed by a bubbling `input` event. Wait for a new `blob:` preview
  with `#busy` hidden, then 600 ms (past the 200 ms debounce). The counter is
  `{"load":1,"preview":2,"make":0}`. Failure:
  `FAIL H3 three quick size changes add one preview: renders <got>, expected {"load":1,"preview":2,"make":0}`.
- **H4 the CLI sentence on W:** click `#make` and wait for `#result` to be shown. Then
  `#cli-note`'s text, with whitespace collapsed to single spaces and trimmed, is
  `Run it on the original, or on a file at least as large as the web copy this page edits (1600 × 1071). On a smaller file the CLI makes a smaller image than this page did, and may refuse the watermark, without warning.`
  The read is `textContent`, or `<no #cli-note>` when the element is missing. Failure:
  `FAIL H4 the CLI sentence on W: "<got>", expected "<literal>"`.
- **H5 the panel's recipe on W:** `#recipe`'s `textContent` is exactly (a literal, trap 19):
  `version = "2"\nformat = "jpeg"\n\n[[step]]\nop = "auto-orient"\n\n[[step]]\nop = "resize"\nmode = "max"\nwidth = 800\n\n[[step]]\nop = "optimize"\n`.
  Failure: `FAIL H5 the panel's recipe on W: <got JSON>, expected <literal JSON>`.
- **H6 renders at load on R:** open `?album=test&photo=2`, wait as H1, and the counter is
  `{"load":1,"preview":1,"make":0}` (it starts again on the new document).
- **H7 the CLI sentence on R:** click `#make`, wait for `#result`, and the sentence reads as in
  H4 with `(1071 × 1600)`. Failure: `FAIL H7 the CLI sentence on R: "<got>", expected "<literal>"`.
- **H8 no page errors:** no `Runtime.exceptionThrown` during the run. Failure:
  `FAIL H8 no page errors: <the exceptions, joined by " | ">`.

The DOM entry adds its rows to this file (drags, keys, the render count across a drag and Done,
M2 and M5's wiring).

### The CI step

In `.github/workflows/ci.yml`'s `ddphotos-parity` job, straight after "Parity harness" and before
"Clippy (wasm32 library)":

```yaml
      # The edit page in a real headless Chrome (tests/ddphotos_page.mjs), against a fake
      # ddphotos site, with the slim wasm the parity step just built. It counts the worker's
      # renders and reads the CLI panel. ubuntu-latest ships Chrome; with none, the check
      # fails rather than skips. It runs only after a green parity step: target/ comes back
      # from the cache, so after a failed build pkg-slim could be an older commit's.
      - name: Edit page in headless Chrome
        run: node tests/ddphotos_page.mjs --pkg target/ddphotos-parity/pkg-slim
```

- **No `if:`:** it runs whenever the parity step passed, and nothing in it can skip. Missing
  Chrome, a missing package or any failed row exits 1. Node prints every row line directly, so
  the PASS lines reach the job log (trap 11). The build quotes them from the PR's log.
- **The job stays advisory** (ruling 5). Its `name:` is unchanged.
- **The runner's Node:** this job has no `setup-node`. The CDP client uses Node's global
  `WebSocket` (Node 22+), which the `ubuntu-26.04` image's Node already serves to `pages.yml`'s
  smoke. The step adds nothing to install.
- **Cost:** Chrome's cold start was 11.2 s in the pages job. The rest is two page loads and two
  Makes, 8–13 s on the Mac and probably 2× on the runner. **Estimate 25–45 s** on a job that took
  3 min 10 s to 4 min 59 s in its last four green runs on `main` (not measured on CI; the build
  records the step's real time).

`justfile`:

```just
# The ddphotos edit page in a real headless Chrome (tests/ddphotos_page.mjs): a fake site,
# the worker's render count, the debounce and the CLI panel. Builds the slim wasm into
# target/ddphotos-parity/pkg-slim, as ddphotos-parity does.
ddphotos-page:
    CARGO_TARGET_DIR="target/ddphotos-parity/wasm-target" just --set _wasm_features " --no-default-features --features avif" \
        --set _wasm_out "$PWD/target/ddphotos-parity/pkg-slim" wasm-build
    node tests/ddphotos_page.mjs --pkg target/ddphotos-parity/pkg-slim
```

(The build may share the wasm line with `ddphotos-parity` through a private recipe; CI does not
call `just ddphotos-page`, so it builds the wasm once.)

### The Node rows (in `tests/ddphotos_parity.mjs`)

A new `modeChecks(ctx)` runs after `modelChecks`, and its failures join `failures` with three
labels: `mode-reducer:`, `mode-geometry:` and `mode-size-line:`. The summary line names each
failure kind (trap 15), and gains one count of what ran, not the constant:
`…, 2 alpha rows, 153 crop-mode rows; 0 row failures, 0 other failures, K=1 (cap 78)`.
`modeChecks` also fails `mode-count: <n> crop-mode rows ran, expected 153` when the count is off.
Every expected value is a literal from this design's independent model (trap 19); none is computed
from `recipe.js`.

- **Photos:** W, P, S and R come from `photoFor(ctx, input)` (`loadPhoto` on the fixture through
  the slim wasm). So R's rows also prove the upright size: 1071 × 1600, the same literals as P.
- **Running a row:** start from `cropReducer(null, { type: "reset" }, photo)` and apply each
  action in turn, deep-freezing every state. Compare four things with the row's literals:
  - the crop, in a compact form: `none`, `aspect a:b` or `aspect a:b follow`, or
    `box x,y,w,h`. Any other key set prints `odd <json>`;
  - the preset, as `a:b` or null;
  - the drag, as `handle@x,y,w,h` or null;
  - whether the last action returned the same object.

  Then compare `planEdit(photo, FORM_NO_WM, state.crop).crop`, the crop step's text, with the
  row's literal. `FORM_NO_WM` is the harness's `FORM` with `size: ""` and `wmOn: false`.
- **Failure messages:**
  - `mode-reducer: <id> "<actions>": <crop|preset|drag|same state> <got>, expected <want>`
  - `mode-reducer: <id> "<actions>": crop step <got JSON>, expected <want JSON>`
  - `mode-reducer: <id> "<actions>" threw: <message>`
  - `mode-geometry: <id>: <got JSON>, expected <want JSON>`
  - `mode-size-line: <id>: "<got>", expected "<want>"`

The action shorthand in the rows:
- `pA:B` is `{type:"preset",a:A,b:B}` and `sw` is swap.
- `ds:H` is drag-start with handle H, and `dm:X,Y` is drag-move.
- `de` is drag-end and `dn` is done.
- `km:X,Y` is key-move and `kr:X,Y` is key-resize.
- `rs` is reset.

**Reducer rows (114):** 28 transitions × 4 shapes (W, P, S, R) = 112, plus 2 square-only rows
(X28S, X29S), so the 4:5/5:4 swap is driven in both orders (trap 20). The orders are also covered
by 3:2/2:3 in X1/X2 and X4/X5 on every shape. Columns: id, shape, actions, crop, preset, drag,
same state, crop step text.

```js
const MODE_ROWS = [
  ["X1W","W","p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X1P","P","p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X1S","S","p3:2","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X1R","R","p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X2W","W","p2:3","aspect 2:3 follow","2:3",null,false,"aspect = \"2:3\"\norientation = \"match\"\n"],
  ["X2P","P","p2:3","aspect 2:3 follow","2:3",null,false,"aspect = \"2:3\"\norientation = \"match\"\n"],
  ["X2S","S","p2:3","aspect 2:3","2:3",null,false,"aspect = \"2:3\"\n"],
  ["X2R","R","p2:3","aspect 2:3 follow","2:3",null,false,"aspect = \"2:3\"\norientation = \"match\"\n"],
  ["X3W","W","p1:1","aspect 1:1","1:1",null,false,"aspect = \"1:1\"\n"],
  ["X3P","P","p1:1","aspect 1:1","1:1",null,false,"aspect = \"1:1\"\n"],
  ["X3S","S","p1:1","aspect 1:1","1:1",null,false,"aspect = \"1:1\"\n"],
  ["X3R","R","p1:1","aspect 1:1","1:1",null,false,"aspect = \"1:1\"\n"],
  ["X4W","W","p3:2 sw","aspect 2:3","3:2",null,false,"aspect = \"2:3\"\n"],
  ["X4P","P","p3:2 sw","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X4S","S","p3:2 sw","aspect 2:3","3:2",null,false,"aspect = \"2:3\"\n"],
  ["X4R","R","p3:2 sw","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X5W","W","p2:3 sw","aspect 2:3","2:3",null,false,"aspect = \"2:3\"\n"],
  ["X5P","P","p2:3 sw","aspect 3:2","2:3",null,false,"aspect = \"3:2\"\n"],
  ["X5S","S","p2:3 sw","aspect 3:2","2:3",null,false,"aspect = \"3:2\"\n"],
  ["X5R","R","p2:3 sw","aspect 3:2","2:3",null,false,"aspect = \"3:2\"\n"],
  ["X6W","W","p3:2 sw sw","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X6P","P","p3:2 sw sw","aspect 2:3","3:2",null,false,"aspect = \"2:3\"\n"],
  ["X6S","S","p3:2 sw sw","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X6R","R","p3:2 sw sw","aspect 2:3","3:2",null,false,"aspect = \"2:3\"\n"],
  ["X7W","W","p3:2 sw p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X7P","P","p3:2 sw p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X7S","S","p3:2 sw p3:2","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X7R","R","p3:2 sw p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X8W","W","sw","none",null,null,true,null],
  ["X8P","P","sw","none",null,null,true,null],
  ["X8S","S","sw","none",null,null,true,null],
  ["X8R","R","sw","none",null,null,true,null],
  ["X9W","W","p1:1 sw","aspect 1:1","1:1",null,true,"aspect = \"1:1\"\n"],
  ["X9P","P","p1:1 sw","aspect 1:1","1:1",null,true,"aspect = \"1:1\"\n"],
  ["X9S","S","p1:1 sw","aspect 1:1","1:1",null,true,"aspect = \"1:1\"\n"],
  ["X9R","R","p1:1 sw","aspect 1:1","1:1",null,true,"aspect = \"1:1\"\n"],
  ["X10W","W","p4:5 ds:move dm:0,0","aspect 4:5 follow","4:5","move@130,0,1339,1071",true,"aspect = \"4:5\"\norientation = \"match\"\n"],
  ["X10P","P","p4:5 ds:move dm:0,0","aspect 4:5 follow","4:5","move@0,130,1071,1339",true,"aspect = \"4:5\"\norientation = \"match\"\n"],
  ["X10S","S","p4:5 ds:move dm:0,0","aspect 4:5","4:5","move@107,0,857,1071",true,"aspect = \"4:5\"\n"],
  ["X10R","R","p4:5 ds:move dm:0,0","aspect 4:5 follow","4:5","move@0,130,1071,1339",true,"aspect = \"4:5\"\norientation = \"match\"\n"],
  ["X11W","W","p4:5 ds:move dm:50,50","box 180,0,1339,1071",null,"move@130,0,1339,1071",false,"x = 0.1125\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X11P","P","p4:5 ds:move dm:50,50","box 0,180,1071,1339",null,"move@0,130,1071,1339",false,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X11S","S","p4:5 ds:move dm:50,50","box 157,0,857,1071",null,"move@107,0,857,1071",false,"x = 0.1465919701213819\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X11R","R","p4:5 ds:move dm:50,50","box 0,180,1071,1339",null,"move@0,130,1071,1339",false,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X12W","W","p4:5 ds:move dm:5000,5000","box 261,0,1339,1071",null,"move@130,0,1339,1071",false,"x = 0.163125\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X12P","P","p4:5 ds:move dm:5000,5000","box 0,261,1071,1339",null,"move@0,130,1071,1339",false,"x = 0\ny = 0.163125\nwidth = 1\nheight = 0.836875\n"],
  ["X12S","S","p4:5 ds:move dm:5000,5000","box 214,0,857,1071",null,"move@107,0,857,1071",false,"x = 0.19981325863678806\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X12R","R","p4:5 ds:move dm:5000,5000","box 0,261,1071,1339",null,"move@0,130,1071,1339",false,"x = 0\ny = 0.163125\nwidth = 1\nheight = 0.836875\n"],
  ["X13W","W","p4:5 ds:move dm:50,50 dm:0,0","box 130,0,1339,1071",null,"move@130,0,1339,1071",false,"x = 0.08125\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X13P","P","p4:5 ds:move dm:50,50 dm:0,0","box 0,130,1071,1339",null,"move@0,130,1071,1339",false,"x = 0\ny = 0.08125\nwidth = 1\nheight = 0.836875\n"],
  ["X13S","S","p4:5 ds:move dm:50,50 dm:0,0","box 107,0,857,1071",null,"move@107,0,857,1071",false,"x = 0.09990662931839403\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X13R","R","p4:5 ds:move dm:50,50 dm:0,0","box 0,130,1071,1339",null,"move@0,130,1071,1339",false,"x = 0\ny = 0.08125\nwidth = 1\nheight = 0.836875\n"],
  ["X14W","W","p4:5 ds:move dm:50,50 de","box 180,0,1339,1071",null,null,false,"x = 0.1125\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X14P","P","p4:5 ds:move dm:50,50 de","box 0,180,1071,1339",null,null,false,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X14S","S","p4:5 ds:move dm:50,50 de","box 157,0,857,1071",null,null,false,"x = 0.1465919701213819\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X14R","R","p4:5 ds:move dm:50,50 de","box 0,180,1071,1339",null,null,false,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X15W","W","p4:5 ds:move dm:50,50 dn dm:100,100","box 180,0,1339,1071",null,null,true,"x = 0.1125\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X15P","P","p4:5 ds:move dm:50,50 dn dm:100,100","box 0,180,1071,1339",null,null,true,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X15S","S","p4:5 ds:move dm:50,50 dn dm:100,100","box 157,0,857,1071",null,null,true,"x = 0.1465919701213819\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X15R","R","p4:5 ds:move dm:50,50 dn dm:100,100","box 0,180,1071,1339",null,null,true,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X16W","W","p4:5 ds:move dm:50,50 p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X16P","P","p4:5 ds:move dm:50,50 p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X16S","S","p4:5 ds:move dm:50,50 p3:2","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X16R","R","p4:5 ds:move dm:50,50 p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X17W","W","p4:5 ds:move dm:50,50 sw","box 180,0,1339,1071",null,"move@130,0,1339,1071",true,"x = 0.1125\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X17P","P","p4:5 ds:move dm:50,50 sw","box 0,180,1071,1339",null,"move@0,130,1071,1339",true,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X17S","S","p4:5 ds:move dm:50,50 sw","box 157,0,857,1071",null,"move@107,0,857,1071",true,"x = 0.1465919701213819\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X17R","R","p4:5 ds:move dm:50,50 sw","box 0,180,1071,1339",null,"move@0,130,1071,1339",true,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X18W","W","p4:5 ds:move dm:50,50 rs","none",null,null,false,null],
  ["X18P","P","p4:5 ds:move dm:50,50 rs","none",null,null,false,null],
  ["X18S","S","p4:5 ds:move dm:50,50 rs","none",null,null,false,null],
  ["X18R","R","p4:5 ds:move dm:50,50 rs","none",null,null,false,null],
  ["X19W","W","rs","none",null,null,true,null],
  ["X19P","P","rs","none",null,null,true,null],
  ["X19S","S","rs","none",null,null,true,null],
  ["X19R","R","rs","none",null,null,true,null],
  ["X20W","W","p4:5 km:1,1","box 131,0,1339,1071",null,null,false,"x = 0.081875\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X20P","P","p4:5 km:1,1","box 0,131,1071,1339",null,null,false,"x = 0\ny = 0.081875\nwidth = 1\nheight = 0.836875\n"],
  ["X20S","S","p4:5 km:1,1","box 108,0,857,1071",null,null,false,"x = 0.10084033613445378\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X20R","R","p4:5 km:1,1","box 0,131,1071,1339",null,null,false,"x = 0\ny = 0.081875\nwidth = 1\nheight = 0.836875\n"],
  ["X21W","W","km:1,1","none",null,null,true,null],
  ["X21P","P","km:1,1","none",null,null,true,null],
  ["X21S","S","km:1,1","none",null,null,true,null],
  ["X21R","R","km:1,1","none",null,null,true,null],
  ["X22W","W","p4:5 kr:-1,0","box 130,0,1338,1071",null,null,false,"x = 0.08125\ny = 0\nwidth = 0.83625\nheight = 1\n"],
  ["X22P","P","p4:5 kr:-1,0","box 0,130,1070,1339",null,null,false,"x = 0\ny = 0.08125\nwidth = 0.9990662931839402\nheight = 0.836875\n"],
  ["X22S","S","p4:5 kr:-1,0","box 107,0,856,1071",null,null,false,"x = 0.09990662931839403\ny = 0\nwidth = 0.7992530345471522\nheight = 1\n"],
  ["X22R","R","p4:5 kr:-1,0","box 0,130,1070,1339",null,null,false,"x = 0\ny = 0.08125\nwidth = 0.9990662931839402\nheight = 0.836875\n"],
  ["X23W","W","p4:5 kr:-5000,-5000","box 130,0,64,64",null,null,false,"x = 0.08125\ny = 0\nwidth = 0.04\nheight = 0.059757236227824466\n"],
  ["X23P","P","p4:5 kr:-5000,-5000","box 0,130,64,64",null,null,false,"x = 0\ny = 0.08125\nwidth = 0.059757236227824466\nheight = 0.04\n"],
  ["X23S","S","p4:5 kr:-5000,-5000","box 107,0,64,64",null,null,false,"x = 0.09990662931839403\ny = 0\nwidth = 0.059757236227824466\nheight = 0.059757236227824466\n"],
  ["X23R","R","p4:5 kr:-5000,-5000","box 0,130,64,64",null,null,false,"x = 0\ny = 0.08125\nwidth = 0.059757236227824466\nheight = 0.04\n"],
  ["X24W","W","ds:nw dm:100,100","box 100,100,1500,971",null,"nw@0,0,1600,1071",false,"x = 0.0625\ny = 0.09337068160597572\nwidth = 0.9375\nheight = 0.9066293183940243\n"],
  ["X24P","P","ds:nw dm:100,100","box 100,100,971,1500",null,"nw@0,0,1071,1600",false,"x = 0.09337068160597572\ny = 0.0625\nwidth = 0.9066293183940243\nheight = 0.9375\n"],
  ["X24S","S","ds:nw dm:100,100","box 100,100,971,971",null,"nw@0,0,1071,1071",false,"x = 0.09337068160597572\ny = 0.09337068160597572\nwidth = 0.9066293183940243\nheight = 0.9066293183940243\n"],
  ["X24R","R","ds:nw dm:100,100","box 100,100,971,1500",null,"nw@0,0,1071,1600",false,"x = 0.09337068160597572\ny = 0.0625\nwidth = 0.9066293183940243\nheight = 0.9375\n"],
  ["X25W","W","p4:5 ds:w dm:5000,0","box 1405,0,64,1071",null,"w@130,0,1339,1071",false,"x = 0.878125\ny = 0\nwidth = 0.04\nheight = 1\n"],
  ["X25P","P","p4:5 ds:w dm:5000,0","box 1007,130,64,1339",null,"w@0,130,1071,1339",false,"x = 0.9402427637721755\ny = 0.08125\nwidth = 0.059757236227824466\nheight = 0.836875\n"],
  ["X25S","S","p4:5 ds:w dm:5000,0","box 900,0,64,1071",null,"w@107,0,857,1071",false,"x = 0.8403361344537815\ny = 0\nwidth = 0.059757236227824466\nheight = 1\n"],
  ["X25R","R","p4:5 ds:w dm:5000,0","box 1007,130,64,1339",null,"w@0,130,1071,1339",false,"x = 0.9402427637721755\ny = 0.08125\nwidth = 0.059757236227824466\nheight = 0.836875\n"],
  ["X26W","W","p4:5 ds:se dm:5000,5000","box 130,0,1470,1071",null,"se@130,0,1339,1071",false,"x = 0.08125\ny = 0\nwidth = 0.91875\nheight = 1\n"],
  ["X26P","P","p4:5 ds:se dm:5000,5000","box 0,130,1071,1470",null,"se@0,130,1071,1339",false,"x = 0\ny = 0.08125\nwidth = 1\nheight = 0.91875\n"],
  ["X26S","S","p4:5 ds:se dm:5000,5000","box 107,0,964,1071",null,"se@107,0,857,1071",false,"x = 0.09990662931839403\ny = 0\nwidth = 0.900093370681606\nheight = 1\n"],
  ["X26R","R","p4:5 ds:se dm:5000,5000","box 0,130,1071,1470",null,"se@0,130,1071,1339",false,"x = 0\ny = 0.08125\nwidth = 1\nheight = 0.91875\n"],
  ["X27W","W","p4:5 ds:s dm:0,-5000","box 130,0,1339,64",null,"s@130,0,1339,1071",false,"x = 0.08125\ny = 0\nwidth = 0.836875\nheight = 0.059757236227824466\n"],
  ["X27P","P","p4:5 ds:s dm:0,-5000","box 0,130,1071,64",null,"s@0,130,1071,1339",false,"x = 0\ny = 0.08125\nwidth = 1\nheight = 0.04\n"],
  ["X27S","S","p4:5 ds:s dm:0,-5000","box 107,0,857,64",null,"s@107,0,857,1071",false,"x = 0.09990662931839403\ny = 0\nwidth = 0.800186741363212\nheight = 0.059757236227824466\n"],
  ["X27R","R","p4:5 ds:s dm:0,-5000","box 0,130,1071,64",null,"s@0,130,1071,1339",false,"x = 0\ny = 0.08125\nwidth = 1\nheight = 0.04\n"],
  ["X30W","W","p3:2 rs","none",null,null,false,null],
  ["X30P","P","p3:2 rs","none",null,null,false,null],
  ["X30S","S","p3:2 rs","none",null,null,false,null],
  ["X30R","R","p3:2 rs","none",null,null,false,null],
  ["X28S","S","p4:5 sw","aspect 5:4","4:5",null,false,"aspect = \"5:4\"\n"],
  ["X29S","S","p5:4 sw","aspect 4:5","5:4",null,false,"aspect = \"4:5\"\n"],
];
```

**Geometry rows (31).** `W` is `{ width: 1600, height: 1071 }`, and boxes are written `x,y,w,h`.
- **`toCopyDelta`** (8):
  - G1 `(3, 0.42875)` is 7.
  - G2 `(1, 0.234375)` is 4.
  - G3 `(-1, 0.4)` is −3: the exact half rounds away from zero.
  - G4 `(1, 0.4)` is 3.
  - G5 `Object.is(toCopyDelta(-0.1, 0.5), 0)` is true (no `-0`).
  - G6 `(5, 0)` is 0.
  - G7 `(5, NaN)` is 0.
  - G8 `(30, 0.42875)` is 70: M7's arithmetic, which the DOM entry drives.
- **`moveBox`** (3), each on box `264,0,1071,1071` in W:
  - G9 `(50, 0)` gives `314,0,1071,1071`.
  - G10 `(-5000, 0)` gives `0,0,1071,1071`.
  - G11 `(5000, 5000)` gives `529,0,1071,1071`.
- **`resizeBox`** (17), on box `100,100,500,400` in W unless a row says otherwise:
  - **Each handle by (10, 10)** (G12–G19):
    - G12 `n` gives `100,110,500,390`.
    - G13 `s` gives `100,100,500,410`.
    - G14 `e` gives `100,100,510,400`.
    - G15 `w` gives `110,100,490,400`.
    - G16 `ne` gives `100,110,510,390`.
    - G17 `nw` gives `110,110,490,390`.
    - G18 `se` gives `100,100,510,410`.
    - G19 `sw` gives `110,100,490,410`.
  - **Clamps:**
    - G20 `e (-5000, 0)` gives `100,100,64,400`.
    - G21 `n (0, 5000)` gives `100,436,500,64`.
    - G30 `w (5000, 0)` gives `536,100,64,400`.
    - G22 `nw (-5000, -5000)` gives `0,0,600,500`.
    - G23 `se (5000, 5000)` gives `100,100,1500,971`.
  - **Small boxes:**
    - G24: box `0,0,40,30` in a 40 × 30 photo, `se (-10, -10)`, stays `0,0,40,30`.
    - G25: box `0,22,100,56` in a 100 × 100 photo, `s (0, -10)`, stays `0,22,100,56`.
    - G26: the same box, `s (0, 10)`, gives `0,22,100,66`.
  - G27: handle `"middle"` throws `unknown crop handle middle`.
- **The rest** (3):
  - G28: `MIN_CROP` is 64.
  - G29: `cropReducer(null, { type: "zoom" }, W)` throws `unknown crop action zoom`.
  - G31: `JSON.stringify(cropReducer(null, { type: "reset" }, W))` is
    `{"crop":null,"preset":null,"drag":null}`.

**Size-line rows (8).** Each is `sizeLine(planEdit(photo, { ...FORM_NO_WM, size }, crop), photo, crop)`.
- L1 W, no crop, size 1600: `The whole photo, 1600 × 1071 · the copy will be 1600 × 1071`
- L2 W, `presetCrop(3, 2)`, 1200: `Crop 3:2, 1600 × 1067 of 1600 × 1071 · the copy will be 1200 × 800`
- L3 P, `presetCrop(3, 2)`, 1600: `Crop 2:3, 1067 × 1600 of 1071 × 1600 · the copy will be 1067 × 1600`
- L4 S, `presetCrop(4, 5)`, 1600: `Crop 4:5, 857 × 1071 of 1071 × 1071 · the copy will be 857 × 1071`
- L5 S, `presetCrop(5, 4)`, 800: `Crop 5:4, 1071 × 857 of 1071 × 1071 · the copy will be 800 × 640`
- L6 R, box `0,314,1071,1071`, 800: `Crop 1071 × 1071 of 1071 × 1600 · the copy will be 800 × 800`
- L7 W, box `0,0,1067,711`, 1600: `Crop 1067 × 711 of 1600 × 1071 · the copy will be 1067 × 711`
  (the frame's example)
- L8 W, `swapCrop(presetCrop(3, 2))`, 1080: `Crop 2:3, 714 × 1071 of 1600 × 1071 · the copy will be 714 × 1071`

**Count: 114 + 31 + 8 = 153 crop-mode rows.** The prototype ran all 153 green against the
candidate `recipe.js`.

### The mutation table, driven

The N rows were driven by applying each edit to the candidate `recipe.js` and running the
prototype row runner, whose failure messages are the format above. Every row not listed as red
stayed green. Each mutant is in `recipe.js`, which both sides of the page harness read, so every
guard is a literal (trap 19). The literal that catches each one is in the red list.

- **M1 `toCopyDelta` returns `dCss`.** Edit: the body becomes `return dCss;`. Red: G1–G8.
  First message: `FAIL mode-geometry: G1: 3, expected 7`. Driven.
- **M3 swap emits the orientation key.** Edit, in `swapCrop`:
  `return { aspect: [b, a], follow: false };` becomes `follow: true`. Red: X4, X5 and X6 on all
  four shapes, X28S, X29S and L8 (29 lines). First message:
  `FAIL mode-reducer: X4W "p3:2 sw": crop aspect 2:3 follow, expected aspect 2:3`, then
  `… crop step "aspect = \"2:3\"\norientation = \"match\"\n", expected "aspect = \"2:3\"\n"`.
  Driven. (The real harness's existing swapped CROP_TABLE rows also go red on it.)
- **M4 a zero move turns S1 into S3.** Edit: `toBox` drops its
  `!s.crop?.box && sameBox(box, from) ? s :` guard. Red: X10 on all four shapes. First message:
  `FAIL mode-reducer: X10W "p4:5 ds:move dm:0,0": crop box 130,0,1339,1071, expected aspect 4:5 follow`.
  Driven.
- **M5 Done keeps the drag** (the function side; the DOM entry drives the wiring). Edit:
  `case "done": return s;`. Red: X15 on all four shapes. Message:
  `FAIL mode-reducer: X15W "p4:5 ds:move dm:50,50 dn dm:100,100": crop box 230,0,1339,1071, expected box 180,0,1339,1071`.
  Driven.
- **M6 a key step taken as screen pixels.** Edit, in `key-move`:
  `moveBox(from, toCopyDelta(action.dx, 0.21438), toCopyDelta(action.dy, 0.21438), photo)`.
  Red: X20 on all four shapes. Message:
  `FAIL mode-reducer: X20W "p4:5 km:1,1": crop box 135,0,1339,1071, expected box 131,0,1339,1071`.
  Driven.
- **M9 `moveBox` not clamped.** Edit: `x: box.x + dx, y: box.y + dy`. Red: G10, G11, and X11,
  X12, X14, X15, X17, X20 and X21 on all four shapes (62 lines). First message:
  `FAIL mode-reducer: X11W "p4:5 ds:move dm:50,50": crop box 180,50,1339,1071, expected box 180,0,1339,1071`.
  Driven.
- **M10 `MIN_CROP` ignored.** Edit: `minW = Math.min(1, box.w)`, and likewise `minH`. Red: G20,
  G21, G24, G25, G30, and X23, X25 and X27 on all four shapes. First message:
  `FAIL mode-reducer: X23W "p4:5 kr:-5000,-5000": crop box 130,0,1,1, expected box 130,0,64,64`.
  Driven.
- **E1 `toCopyDelta` rounds −2.5 to −2 and has no ratio guard** (not in the frame). Edit:
  `return Math.round(dCss / ratio);`. Red: G3, G5, G6 and G7. First message:
  `FAIL mode-geometry: G3: -2, expected -3`. Driven.
- **E2 `moveBox` clamps `x` to the photo, not the box** (not in the frame). Edit:
  `clamp(box.x + dx, 0, photo.width)`. Red: G11 and 18 X rows. First message:
  `FAIL mode-reducer: X11P "p4:5 ds:move dm:50,50": crop box 50,180,1071,1339, expected box 0,180,1071,1339`.
  Driven.
- **E3 `resizeBox` flips past the opposite edge** (not in the frame). Edit: a `w` edge clamped to
  `[0, photo.width]` and swapped with `right` when it passes it. Red: G30 and X25 on all four
  shapes. First message:
  `FAIL mode-reducer: X25W "p4:5 ds:w dm:5000,0": crop box 1469,0,131,1071, expected box 1405,0,64,1071`.
  Driven.
- **E4 the `nw` corner moves only its top edge** (not in the frame). Edit: `nw` maps to `"n"`.
  Red: G17, G22 and X24 on all four shapes. First message:
  `FAIL mode-reducer: X24W "ds:nw dm:100,100": crop box 0,100,1600,971, expected box 100,100,1500,971`.
  Driven.
- **E5 swap live on a box** (not in the frame). Edit: the swap guard becomes
  `if (!s.crop || (s.crop.aspect && s.crop.aspect[0] === s.crop.aspect[1])) return s;`.
  Red: X17 on all four shapes. Message:
  `FAIL mode-reducer: X17W "p4:5 ds:move dm:50,50 sw" threw: undefined is not iterable (cannot read property Symbol(Symbol.iterator))`.
  Driven. (A first version with `aspect?.[0] === aspect?.[1]` was an equivalent mutant: undefined
  equals undefined. It stayed green and was rewritten.)
- **E6 a preset keeps the drag** (not in the frame). Edit: `drag: s.drag` in `preset`. Red: X16 on
  all four shapes. Message:
  `FAIL mode-reducer: X16W "p4:5 ds:move dm:50,50 p3:2": drag move@130,0,1339,1071, expected null`.
  Driven.
- **E7 drag-end writes to the state** (not in the frame). Edit: `if (s.drag) s.drag = null; return s;`.
  Red: X14 on all four shapes. Message:
  `FAIL mode-reducer: X14W "p4:5 ds:move dm:50,50 de" threw: Cannot assign to read only property 'drag' of object '#<Object>'`.
  Driven.
- **E8 `sizeLine` reads the box, not the copy** (not in the frame). Edit: the out size from
  `cropBox`. Red: L2, L5 and L6. Message:
  `FAIL mode-size-line: L2: "Crop 3:2, 1600 × 1067 of 1600 × 1071 · the copy will be 1600 × 1067", expected "Crop 3:2, 1600 × 1067 of 1600 × 1071 · the copy will be 1200 × 800"`.
  Driven.
- **E9 `sizeLine` shows the aspect as stored** (not in the frame). Edit: `crop.aspect` for
  `shownAspect(crop, photo)`. Red: L3. Message:
  `FAIL mode-size-line: L3: "Crop 3:2, 1067 × 1600 of 1071 × 1600 · the copy will be 1067 × 1600", expected "Crop 2:3, 1067 × 1600 of 1071 × 1600 · the copy will be 1067 × 1600"`.
  Driven.
- **E10 reset keeps the preset** (not in the frame). Edit: `preset: s.preset` in `reset`. Red: X30
  on all four shapes. Message: `FAIL mode-reducer: X30W "p3:2 rs": preset 3:2, expected null`.
  Driven. It was green until X30 was added; X18 resets after a drag, when the preset is already
  null.
- **E11 a drag-move with no drag acts** (not in the frame). Edit: `if (!s.drag)` moves the current
  box instead of returning. Red: X15 on all four shapes. Message:
  `FAIL mode-reducer: X15W "p4:5 ds:move dm:50,50 dn dm:100,100": crop box 261,0,1339,1071, expected box 180,0,1339,1071`.
  Driven.
- **M11 the debounce removed** (H). Edit, in `edit.js`'s `schedulePreview`:
  `previewTimer = setTimeout(runPreview, 200);` becomes `runPreview();`. Red: H3 only. Message:
  `FAIL H3 three quick size changes add one preview: renders {"load":1,"preview":4,"make":0}, expected {"load":1,"preview":2,"make":0}`.
  Green: H1, H2 and H4–H8. Driven.
- **M12 the size from the stored file** (H). Edit, in `worker.js`'s `load`: return `loadPhoto`'s
  result with `width` and `height` from `info(source)`. Red: H7 only. Message:
  `FAIL H7 the CLI sentence on R: "Run it … edits (1600 × 1071). …", expected "Run it … edits (1071 × 1600). …"`.
  Green: H1–H6 and H8. Driven.
- **The sentence's wording changed** (H, the control for the panel's text). Edit: drop
  `and may refuse the watermark, ` from the paragraph. Red: H4 and H7. Message:
  `FAIL H4 the CLI sentence on W: "Run it on the original, … this page did, without warning.", expected "… this page did, and may refuse the watermark, without warning."`.
  Driven.
- **The paragraph removed from `index.html` only** (H). Red on the load, not on the text
  (correction 6):
  `FAIL timed out waiting for the first preview on W; the page's error box: Cannot set properties of null (setting 'textContent'); page exceptions: none`.
  Driven (on the 60 s timeout; the build's fail-fast on `#error` reports it at once).
- **Trap 14, `main`'s `recipe.js`, `index.html` and `edit.js` restored verbatim:**
  - The module check: driven on a scratch copy of the real harness with the new `checkModules`,
    pointed at the restored page. It exits 1 with exactly these lines:
    - `FAIL modules: recipe.js lacks toCopyDelta`
    - `FAIL modules: recipe.js lacks moveBox`
    - `FAIL modules: recipe.js lacks resizeBox`
    - `FAIL modules: recipe.js lacks MIN_CROP`
    - `FAIL modules: recipe.js lacks cropReducer`
    - `FAIL modules: recipe.js lacks sizeLine`
  - The headless check: H4 and H7 red with `"<no #cli-note>"`, and H1–H3, H5, H6 and H8 green.
  - With only `main`'s `recipe.js` restored, the same six module lines.
  - With `function sizeLine() { return ""; }` added to `edit.js`:
    `FAIL modules: edit.js defines sizeLine, which belongs to recipe.js`. Driven.
- **No Chrome** (trap 11). Edit: `CHROME=/nonexistent/chrome`. Exit 1, with the line
  `FAIL no Chrome or Chromium found at CHROME=/nonexistent/chrome; install one or set CHROME=/path/to/chrome`.
  Driven.
- **Not driven here:** M1's wiring and M2, M4's click, M5's wiring, M6's key wiring, M7, M8 and
  M9's drag. They are the DOM entry's, as the prompt splits them.

### Nothing else changes: what pins it

- **The parity summary line keeps its counts:** `16 table rows, 36 crop rows, 17 replay rows, 26 orientation rows, 2 alpha rows`,
  with `153 crop-mode rows` added after them, and `0 row failures, 0 other failures, K=1 (cap 78)`.
- **`--self-test`'s output is unchanged:** B1–B19 and N1–N5 PASS on their expected rows.
- **`TABLE`, `CROP_TABLE`, `REPLAY_TABLE`, `ORIENTATION_ROWS`, the alpha rows and the golden
  texts are unchanged.** The build's `git diff` of `tests/ddphotos_parity.mjs` adds only
  `modeChecks`, its rows, the `owned` names, the required-export names and the summary count.
- **No file under `src/`, `tests/fixtures/` or `recipes/` changes**
  (`git diff --name-only origin/main...HEAD -- src tests/fixtures recipes` prints nothing).
- **The slim wasm is unchanged:** brotli 833,619 B on the Mac (measured this session, raw
  4,148,451 B, gzip 1,195,748 B). CI reads 836,219 B, a number from earlier CI logs that this
  session did not measure. The build records both from its own `just wasm-size` and the CI log.
- **`recipe.js` grows by about 118 lines** (+5.4 KB unminified in the prototype). It isn't in the
  wasm, and the page loads it as a separate module.

### Open questions for the maintainer (from design)

1. **`MIN_CROP = 64` does not prevent watermark refusals on thin strips (correction 1).** Over
   this sweep, 122 was the smallest short side with no refusal, and a 120-character text was still
   refused at 192, so no copy-pixel minimum is safe for every text. **Recommend: keep 64 as the
   box's usability floor**, which is what this spec builds, as ruled. Have the watermark-fit entry
   say it covers the mark's *width plus margin* against the output's width at sizes below the
   crop's long edge, the case this sweep found. Schedule it before the crop mode's DOM entry ships,
   or with it, since the DOM is the first thing that lets a user make such a box. Until then a
   refused mark shows the engine's message through `Preview failed:` on Done. If you'd rather
   raise the minimum, 122 is the measured value, and rows G20, G21, G24, G25, G30, X23, X25 and X27
   change with it.

### Maintainer rulings on the design (2026-11-27)

1. **`MIN_CROP` stays 64** (question 1, as recommended). It is the box's usability floor, not a
   watermark guard. G20, G21, G24, G25, G30, X23, X25 and X27 keep their 64 literals.
2. **The watermark-fit entry lands before the crop mode's DOM entry.** It takes the case this
   design found (the mark's width plus one margin against a resized strip's width) along with the
   frame's two. STAGE-060 now orders it ahead of the DOM entry. Until it lands, no page control
   can draw a thin box, so this spec ships no user-visible refusal.

## Inputs

- **Files to read:** `integrations/ddphotos/edit/recipe.js` (the crop model: `presetCrop`,
  `swapCrop`, `cropBox`, `cropStep`, `planEdit`, `loadPhoto`); `edit.js:185–199` (`readEdit`),
  `edit.js:208–231` (`schedulePreview`, `runPreview`) and `edit.js:300–343` (`showEditor`, where
  `photo` is set at line 327); `worker.js:38–41` (`load`); `index.html:141–168` (the CLI panel);
  `edit.css:208–222` (`.frame`, `.frame img`).
- **Tests:** `tests/ddphotos_parity.mjs`: `checkModules` (lines 409–470), `photoFor` (502),
  `modelChecks` (862), the main flow and summary line (1360–1420). `tests/demo_smoke.mjs:106–192`
  (`findChrome`, `CDP`) and `:265–356` (Chrome's start-up). `scripts/serve.mjs` (`startServer`).
  `scripts/ddphotos-edit-install.mjs` (the `edit/` + `vendor/` layout).
- **CI:** `.github/workflows/ci.yml:80–109` (the `ddphotos-parity` job); `.github/workflows/pages.yml`
  (paths, and the "build + browser smoke" job); `justfile:714–729` (`ddphotos-parity`).
- **External APIs:** the Chrome DevTools Protocol (`Page.addScriptToEvaluateOnNewDocument`,
  `Page.navigate`, `Runtime.evaluate`, `Runtime.exceptionThrown`,
  `Emulation.setDeviceMetricsOverride`), as `demo_smoke.mjs` uses it. No auth.

## Outputs

- **Files created:**
  - `tests/ddphotos_page.mjs`: the headless page check, rows H1–H8.
  - `scripts/lib/chrome.mjs`: `findChrome`, `CDP` and `launchChrome`, moved out of `demo_smoke.mjs`.
- **Files modified:**
  - `integrations/ddphotos/edit/recipe.js`: the six crop-mode names.
  - `integrations/ddphotos/edit/index.html`: the `#cli-note` paragraph.
  - `integrations/ddphotos/edit/edit.js`: one line filling `#cli-size` from `photo`.
  - `integrations/ddphotos/README.md`: the CLI row's clause.
  - `tests/ddphotos_parity.mjs`: `modeChecks` and its 153 rows, the `owned` and required-export
    names, and the summary count.
  - `tests/demo_smoke.mjs`: imports from `scripts/lib/chrome.mjs`; no behaviour change.
  - `justfile`: `ddphotos-page`.
  - `.github/workflows/ci.yml`: the "Edit page in headless Chrome" step.
- **New exports** (from `recipe.js`): `MIN_CROP`, `toCopyDelta(dCss, ratio)`,
  `moveBox(box, dx, dy, photo)`, `resizeBox(box, handle, dx, dy, photo)`,
  `cropReducer(state, action, photo)` and `sizeLine(edit, photo, crop)`. From
  `scripts/lib/chrome.mjs`: `findChrome()`, `CDP` and `launchChrome(chromePath, profileDir)`.
- **Database changes:** none.

## Acceptance Criteria

- [x] **AC-1** `recipe.js` exports `MIN_CROP = 64` and `toCopyDelta`, which rounds half away from
  zero on both signs, never returns `-0`, and returns 0 for a ratio that isn't a positive finite
  number (G1–G8, G28).
- [x] **AC-2** `moveBox` keeps the box inside the photo with its size unchanged (G9–G11).
- [x] **AC-3** `resizeBox` moves exactly the named edges for each of the 8 handles. It clamps
  each edge at the photo and at `min(MIN_CROP, side)` from the opposite edge, never flips, lets a
  box under 64 px grow but not shrink, and throws on an unknown handle (G12–G27, G30).
- [x] **AC-4** `cropReducer` implements the nine actions and the `{ crop, preset, drag }` state as
  designed. It starts from `cropReducer(null, { type: "reset" })`, returns the same object when
  nothing changes, never mutates its input, and throws on an unknown action (X1–X30, G29, G31).
- [x] **AC-5** Every reducer row's crop step text, through `planEdit`, equals its literal. Swap is
  disabled in S0, on a box and on 1:1, live on a square, and twice stays explicit (X4–X9, X17,
  X28S, X29S). `cropStep`, `presetCrop`, `swapCrop` and `cropBox` are unchanged, so no recipe rule
  is added.
- [x] **AC-6** `sizeLine` gives the designed text for no crop, a preset (with the shown aspect)
  and a box, with the output size from `planEdit` (L1–L8).
- [x] **AC-7** `checkModules` requires the six names from `recipe.js` and lists them, and every
  private helper the build adds, in `owned`. `main`'s `recipe.js` restored gives the six
  `recipe.js lacks …` lines, and an `edit.js` that defines one of them is refused.
- [x] **AC-8** The parity summary line keeps its five counts and adds `153 crop-mode rows`. Every
  crop-mode failure is counted under "other failures" with its `mode-reducer`, `mode-geometry`,
  `mode-size-line` or `mode-count` label. `--self-test` is unchanged.
- [x] **AC-9** The CLI panel shows the designed sentence. `edit.js` fills its size from `photo` at
  load: `1600 × 1071` on W and `1071 × 1600` on R (H4, H7).
- [x] **AC-10** The README's CLI row ends with the designed clause, and nothing else in the README
  changes.
- [x] **AC-11** `tests/ddphotos_page.mjs` builds the fake site, serves it, drives headless Chrome
  through `scripts/lib/chrome.mjs`, and runs H1–H8 green. It exits 1 on any failed row, on a
  failed wait, with no Chrome, with a `CHROME` that doesn't exist and with a missing `--pkg`; it
  never skips.
- [x] **AC-12** The negative controls are red with the designed messages: M11 on H3, M12 on H7, the
  wording change on H4 and H7, and `main`'s page restored on H4 and H7 with `<no #cli-note>`.
- [x] **AC-13** The check leaves no temp folder behind on a pass or a fail without `--keep`, and
  awaits Chrome's exit before deleting its profile.
- [x] **AC-14** `just demo-smoke` passes locally with the shared lib, and `pages`'
  "build + browser smoke" is green on the PR.
- [x] **AC-15** CI's `ddphotos-parity` job runs "Edit page in headless Chrome" after the parity
  step, with no `if:`, and is green on the PR. Its log shows H1–H8 PASS and the step's time.
- [x] **AC-16** Nothing else changes: no file under `src/`, `tests/fixtures/` or `recipes/`; the
  parity counts and `--self-test` as above; the slim wasm at brotli 833,619 B on the Mac.
- [x] **AC-17** Each of the four behaviour sentences is driven and quoted in Build Completion
  (see Notes).

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in **build** is to make these pass.

- **`tests/ddphotos_parity.mjs`** (run by `just ddphotos-parity`)
  - `"modules: the crop-mode names"`: asserts `recipe.js` exports `toCopyDelta`, `moveBox`,
    `resizeBox`, `MIN_CROP`, `cropReducer` and `sizeLine`, and that neither `edit.js` nor
    `worker.js` defines them (AC-7). Red on `main` with the six `recipe.js lacks …` lines.
  - `"mode-reducer X1W … X30R, X28S, X29S"`: the 114 reducer rows above, each comparing crop,
    preset, drag, same-object and crop-step text with its literals (AC-4, AC-5). Red on `main` (no
    `cropReducer`).
  - `"mode-geometry G1 … G31"`: the 31 geometry rows (AC-1, AC-2, AC-3, AC-4).
  - `"mode-size-line L1 … L8"`: the 8 size-line rows (AC-6).
  - `"mode-count"`: 153 crop-mode rows ran (AC-8).
  - `"summary line"`: it reads `16 table rows, 36 crop rows, 17 replay rows, 26 orientation rows, 2 alpha rows, 153 crop-mode rows; 0 row failures, 0 other failures, K=1 (cap 78)`
    (AC-8, AC-16).
- **`tests/ddphotos_page.mjs`** (run by `just ddphotos-page`; new)
  - `"H1 page loads"` (AC-11).
  - `"H2 renders at load"`: `{"load":1,"preview":1,"make":0}` (AC-11).
  - `"H3 three quick size changes add one preview"`: `{"load":1,"preview":2,"make":0}` (AC-11,
    AC-12 with M11).
  - `"H4 the CLI sentence on W"`: the literal with `(1600 × 1071)` (AC-9). Red on `main`.
  - `"H5 the panel's recipe on W"`: the literal CLI recipe at 800 (AC-11).
  - `"H6 renders at load on R"` (AC-11).
  - `"H7 the CLI sentence on R"`: the literal with `(1071 × 1600)` (AC-9, AC-12 with M12). Red on
    `main`.
  - `"H8 no page errors"` (AC-11).
- **`tests/demo_smoke.mjs`** (run by `just demo-smoke`): unchanged checks, all green through the
  shared lib (AC-14).

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle. It is the
equivalent of a handoff document, folded into the spec since there is no separate receiving
agent.*

### Decisions that apply

- `DEC-118`: a crop recipe from the page replays on the original within a stated bound. Rule 5
  (a smaller file gives a smaller output) is the CLI sentence's reason. Rule 2 (a following preset
  writes `orientation = "match"`, a swapped one does not) is what X1–X7 pin.
- `DEC-112`: a recipe carries geometry as fractions and rounds each edge. This is why the page
  never rounds a box: the fraction round trip is exact (driven again here).
- `DEC-113`: a geometry step acts on the buffer it receives. The crop runs after `auto-orient`, so
  every box is in upright pixels (R's rows).
- `DEC-110`: native and wasm agree on dimensions exactly. Nothing here changes the engine; the
  parity rows stand as they are.
- No new DEC. The design adds no rule that outlives this spec: the reducer's shape is page code,
  and `MIN_CROP` is ruled.

### Constraints that apply

- `test-before-implementation`: the rows and H checks above come first. The build's first commit
  is the failing tests, red on `main`'s page with the quoted lines.
- `one-spec-per-pr`: the PR names SPEC-164 only.
- `cost-captured-per-cycle`: the build records its measured cost.
- `no-new-top-level-deps-without-decision`: none is added. Chrome is driven over CDP with Node's
  built-ins, as `demo_smoke.mjs` does.

### Overlap with other open work

Taken 2026-11-27, with `gh pr list --state open --json number,title,files`:
- `PR #237` (heifer swap): it touches `.github/workflows/ci.yml` and `justfile`, both of which
  this spec also touches. They meet in different places: #237 edits HEIC jobs and recipes, and
  this spec edits the `ddphotos-parity` job and adds `ddphotos-page`. There's no shared line, but
  whichever merges second rebases. #237 is parked (memory: watching, not merging).
- `git branch -r` (88 refs, 2026-11-27 01:07 EST): 16 pushed branches touch a file this spec
  touches. Each was diffed three-dot against `origin/main`. None has an open PR (only #237 is
  open), so they are left over from merged or closed work:
  - `feat/spec-161-page-crop-model`: `recipe.js`, `worker.js`, `ddphotos_parity.mjs`
  - `feat/spec-163-…` and `pr272`: `ddphotos_parity.mjs`
  - `spec-080-…` and `spec-105-…`: `scripts/lib/png.mjs` and `tests/demo_smoke.mjs`
  - the rest: `ci.yml` or `justfile` only

  The build re-runs both commands at its Step 0 and records any branch with an open PR.

### Prior related work

- `SPEC-161` (shipped, PR #273): the crop model this spec drives, the upright `loadPhoto`, and the
  36 crop and 17 replay rows.
- `SPEC-163` (shipped, PR #272): the orientation key (`orientation = "match"`).
- `SPEC-142` (shipped, PR #234): the parity harness and its `checkModules`.
- `SPEC-077`/`SPEC-078`: `tests/demo_smoke.mjs`, the CDP client and the Chrome start-up this spec
  moves into `scripts/lib/chrome.mjs`.

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel necessary during build,
create a new spec rather than expanding this one.

- The crop mode's DOM: the Crop button, the box over the oriented base, the preset, Swap, Reset and
  Done buttons, pointer, touch and keyboard handlers, ARIA, and the size line and live watermark
  note on the page; and their headless rows. STAGE-060's "Crop on the page: the crop mode's DOM".
- Alt+Left/Right against the browser's Back/Forward: that entry's design.
- The README's Crop row and the reworded "No crop …" limit: that same entry.
- The README's "How it was tested" mentioning `just ddphotos-page`: the test-bed entry, which
  rewrites that section (ruling 6 keeps this spec to the CLI clause).
- The three-server browser round and the wasm delta: STAGE-060's "Test-bed round and README".
- `rotate`/straighten on the page: STAGE-060's "Straighten on the page".
- The watermark's fit on small or thin crops (the 5 % coverage limit, the margin against the
  short side, and the mark's width plus margin against a resized strip's width, found here):
  STAGE-060's watermark-fit finding entry. This spec adds no recipe logic.
- A size gate on the slim wasm bundle: its existing STAGE-060 entry.
- Making `ddphotos-parity` a required check: ruling 5, once both halves have shipped.
- Any Rust or wasm change.
- Recipe sidecars, any save flow, flip, pad or trim on the page, and ddphotos code (STAGE-060
  `explicitly_does_not`).
- Pinch and two-finger gestures.
- A CHANGELOG line. The CHANGELOG records the crate; no page change has had one (SPEC-142 and
  SPEC-161 had none), and the page has no release channel of its own.

## Notes for the Implementer

- **Order and checkpoints** (long cycles must checkpoint; push after each):
  1. The failing tests: `modeChecks` with the 153 literal rows, the `checkModules` names, and
     `tests/ddphotos_page.mjs` with H1–H8. Run both against `main`'s page and quote the red lines
     (the six module lines; H4 and H7 `<no #cli-note>`). Commit and push.
  2. `scripts/lib/chrome.mjs` and `demo_smoke.mjs` on it; `just demo-smoke` green. Commit and push.
  3. The six functions in `recipe.js`, green on `just ddphotos-parity`. Commit and push.
  4. The sentence, the `edit.js` line, the README clause; `just ddphotos-page` green. Commit and
     push.
  5. The CI step and `just ddphotos-page`; open the PR as a draft and read both jobs' logs (never
     poll in a loop: one `gh pr checks --watch` in the background).
  6. The negative controls (M11, M12, the wording, `main`'s page, no Chrome), each driven on a
     scratch copy of the page and reverted. Quote each red line in Build Completion.
- **Temp folders.** The parity harness keeps its `mkdtempSync(tmpdir())` folders on any failing
  run, and a red run is expected at checkpoint 1.
  - Before the first run, count the system temp folder's `ddphotos-*` folders and their size, and
    don't delete any (some belong to other sessions). At design: **5 folders, 1,723,616 KB** in
    `/var/folders/r7/…/T`, before and after.
  - Run every harness and headless command with `TMPDIR` set to a folder in the session
    scratchpad, and confirm after the first run that the folders landed there.
  - `ddphotos_page.mjs` makes `ddphotos-page-site-*` and `ddphotos-page-chrome-*`, and deletes
    both unless `--keep`. Empty the scratch `TMPDIR` after reading each red run.
  - Never run a harness while editing the files it reads.
  - At the end, count the system `ddphotos-*` again and report both counts.
- **Never share a `CARGO_TARGET_DIR`** with another session. The recipes build under
  `target/ddphotos-parity/` in the build's own worktree.
- **Never drive the maintainer's browser.** The check starts its own headless Chrome with its
  profile in a temp folder.
- **Comments are plain:** they say what the code does and why, with no spec or DEC numbers in
  `recipe.js`, `edit.js` or `index.html` (the page is user-facing code).
- **The behaviour sentences to drive, on every exit path** (AGENTS §15). Run each with the release
  binary, on the build's branch, and quote it. Design's run on the candidate gave:
  1. "On a smaller file the CLI makes a smaller image than this page did". Run `cliRecipe` from W
     at 1080 (watermark off, JPEG) on an 800 × 536 file made from `photo.webp`: `800 × 536`. With
     3:2: `800 × 533`. The page made 1080 × 723 and 1080 × 720.
  2. "… and may refuse the watermark". The same recipe with "© 2026 Jane Doe" at Medium and 3:2,
     on a 100 × 67 file: exit 2,
     `error: watermark refused: the 334×39 px mark plus its 22 px margin does not fit inside the 100×67 px image`.
     (On the 800 × 536 file it was not refused: exit 0, 800 × 533.)
  3. "without warning": path 1 exits 0 with 0 bytes on stderr.
  4. "at least as large as the web copy this page edits (W × H)": H4 and H7.
- **Wording you may improve, but then change the literals in H4, H7 and the README together.**
  Keep "the web copy this page edits": the page's own word for the published file.
- **The fake site's `index.json`** needs only what `edit.js` reads (`fileName`, `src.full`,
  `src.grid`, and optional `description`, `version` and `kind`). Don't copy a real ddphotos
  `index.json`, which would bring fields this check doesn't stand behind.
- **H5 depends on H3 leaving `#size` at 800 and the format at JPEG.** Keep the order.
- **Record in Build Completion:** both jobs' step times from the CI log, the `ddphotos-*`
  counts, the wasm sizes, and the H lines from the PR's log.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-164-crop-state-model`
- **PR (if applicable):** #274 (draft)
- **All acceptance criteria met?** yes (17 of 17; each is ticked above)
- **New decisions emitted:** none
- **Files this diff touches** (`git diff --name-only origin/main...HEAD`: 10 files, plus this spec = 11):
  - `.github/workflows/ci.yml`: the "Edit page in headless Chrome" step in `ddphotos-parity`, after "Parity harness", no `if:`.
  - `integrations/ddphotos/README.md`: the CLI row's closing clause.
  - `integrations/ddphotos/edit/edit.js`: one line (plus a two-line comment) filling `#cli-size` from `photo`.
  - `integrations/ddphotos/edit/index.html`: the `#cli-note` paragraph after "This page edits the web copy …".
  - `integrations/ddphotos/edit/recipe.js`: `MIN_CROP`, `toCopyDelta`, `moveBox`, `resizeBox`, `cropReducer`, `sizeLine` and four private helpers (`clamp`, `sameBox`, `toBox`, `HANDLE_EDGES`).
  - `justfile`: the `ddphotos-page` recipe.
  - `scripts/lib/chrome.mjs` (new): `findChrome`, `CDP`, `launchChrome`.
  - `tests/ddphotos_page.mjs` (new): the headless page check, H1–H8.
  - `tests/ddphotos_parity.mjs`: `modeChecks` with 153 literal rows, the six names in `checkModules` (required and `owned`), the four helpers in `owned`, and the summary count.
  - `tests/demo_smoke.mjs`: imports the shared lib; its start-up code moved out; checks and output unchanged.
  - This spec (timeline, ACs, cost, Build Completion).
- **Unchanged, as required:** `git diff --stat origin/main...HEAD -- src tests/fixtures recipes integrations/ddphotos/edit/worker.js CHANGELOG.md` prints nothing.

**RED baseline** (checkpoint 1, `main`'s page, quoted):
- Parity, exit 1, six lines and nothing else:
  - `FAIL modules: recipe.js lacks toCopyDelta`, then `moveBox`, `resizeBox`, `MIN_CROP`, `cropReducer`, `sizeLine`.
- Page check, exit 1, `ddphotos-page: 8 rows, 2 failed`:
  - `FAIL H4 the CLI sentence on W: "<no #cli-note>", expected "Run it on the original, … (1600 × 1071). …"`
  - `FAIL H7 the CLI sentence on R: "<no #cli-note>", expected "… (1071 × 1600). …"`
  - H1, H2, H3, H5, H6 and H8 PASS. Nothing else was red.

**Results, green:**
- `just ddphotos-parity`, normal mode: `16 table rows, 36 crop rows, 17 replay rows, 26 orientation rows, 2 alpha rows, 153 crop-mode rows; 0 row failures, 0 other failures, K=1 (cap 78)`.
- `--self-test`: B1–B19 and N1–N5 PASS, unchanged.
- `just ddphotos-page`: `ddphotos-page: 8 rows, 0 failed`.
- `just demo-smoke`: 78 ✓, 0 ✗, exit 0, through the shared lib.

**The four behaviour sentences** (release binary built on this branch, recipes from the branch's `cliRecipe`, originals made from `photo.webp` with Pillow):
1. "On a smaller file the CLI makes a smaller image than this page did": an 800 × 536 original, W at 1080, JPEG, no watermark. Plain resize: CLI 800 × 536 (the page made 1080 × 723). With 3:2: CLI 800 × 533 (the page made 1080 × 720).
2. "… and may refuse the watermark": the same recipe with "© 2026 Jane Doe", Medium, 3:2, on a 100 × 67 file: exit 2, `error: watermark refused: the 334×39 px mark plus its 22 px margin does not fit inside the 100×67 px image`. On the 800 × 536 file it was not refused: exit 0, 800 × 533.
3. "without warning": both runs of path 1 exit 0 with 0 bytes on stderr.
4. "at least as large as the web copy this page edits (W × H)": H4 reads `(1600 × 1071)` on W and H7 reads `(1071 × 1600)` on R, both PASS locally and in CI.

**The mutation table, re-driven** (a scratch `git worktree` of the pushed head, never the build worktree; the slim wasm and binary built once and reused; each edit confirmed live with a non-empty `git diff`; the full harness run with output redirected; restored and shown green after every row; scratch `TMPDIR` emptied between rows). Every red set was read off its own run:
- M1 `toCopyDelta` returns `dCss`: red G1–G8. First: `FAIL mode-geometry: G1: 3, expected 7`. Matches.
- M3 swap keeps the key: red X4, X5, X6 on all four shapes, X28S, X29S, L8 (29 mode lines), plus the existing swapped rows C24, C25 and R10, as the spec says. First mode line is on X4. Matches.
- M4 a zero move turns S1 into S3: red X10 on all four shapes. First: `FAIL mode-reducer: X10W "p4:5 ds:move dm:0,0": crop box 130,0,1339,1071, expected aspect 4:5 follow`. **Mismatch: X21 on all four shapes is also red** (28 lines in all). The spec's quoted guard is a ternary that does not exist in this code; the closest edit removes the `if (!s.crop?.box && sameBox(box, from)) return s;` line, and `toBox` is shared by the drag and key paths, so the S0 key-move that cannot move (X21) now makes a box. The design's prototype guarded only the drag path. Both rows guard real behaviour.
- M5 Done keeps the drag: red X15 on all four shapes. First: `… crop box 230,0,1339,1071, expected box 180,0,1339,1071`. Matches.
- M6 key step as screen pixels: red X20 on all four. First: `crop box 135,0,1339,1071, expected box 131,0,1339,1071`. Matches.
- M9 `moveBox` not clamped: red G10, G11 and X11, X12, X14, X15, X17, X20, X21 on all four (62 lines). First: `X11W … crop box 180,50,1339,1071, expected box 180,0,1339,1071`. Matches.
- M10 `MIN_CROP` ignored: red G20, G21, G24, G25, G30 and X23, X25, X27 on all four. First: `X23W … crop box 130,0,1,1, expected box 130,0,64,64`. Matches.
- E1 red G3, G5, G6, G7; first `G3: -2, expected -3`. Matches.
- E2 red G11 and 18 X rows (X11P/R, X12 ×4, X14P/R, X15P/R, X17P/R, X20P/R, X21 ×4). First `X11P … crop box 50,180,1071,1339, expected box 0,180,1071,1339`. Matches.
- E3 red G30, X25 ×4; first `X25W … crop box 1469,0,131,1071, expected box 1405,0,64,1071`. Matches.
- E4 red G17, G22, X24 ×4; first `X24W … crop box 0,100,1600,971, expected box 100,100,1500,971`. Matches.
- E5 red X17 ×4; message `X17W "p4:5 ds:move dm:50,50 sw" threw: undefined is not iterable (cannot read property Symbol(Symbol.iterator))`, as the spec quotes. Red, not an equivalent mutant.
- E6 red X16 ×4; first `drag move@130,0,1339,1071, expected null`. Matches.
- E7 red X14 ×4; first `threw: Cannot assign to read only property 'drag' of object '#<Object>'`. Matches (see Deviations: `drag-end` and `done` are separate cases so this row stays X14 only).
- E8 red L2, L5, L6; E9 red L3; both with the spec's messages. Match.
- E10 red X30 ×4; first `X30W "p3:2 rs": preset 3:2, expected null`. Red on my rows.
- E11 red X15 ×4; first `crop box 261,0,1339,1071, expected box 180,0,1339,1071`. Matches.
- M11 debounce removed: red H3 only; `renders {"load":1,"preview":4,"make":0}, expected {"load":1,"preview":2,"make":0}`. Matches.
- M12 stored size in `worker.js`: red H7 only, reading `(1600 × 1071)`. Matches.
- Wording change (clause dropped): red H4 and H7. Matches.
- Paragraph removed from `index.html` only: the wait fails at once on `#error`, not on a timeout: `FAIL the page failed waiting for the first preview on W; the page's error box: Cannot set properties of null (setting 'textContent'); page exceptions: none`; `ddphotos-page: 0 rows, 1 failed`. Matches the spec's text.
- Trap 14: `main`'s `recipe.js`, `index.html` and `edit.js` restored: parity prints exactly the six `recipe.js lacks …` lines (exit 1); the page check has H4 and H7 red with `<no #cli-note>` and H1–H3, H5, H6, H8 green. `main`'s `recipe.js` only: the same six lines. `function sizeLine() { return ""; }` added to `edit.js`: `FAIL modules: edit.js defines sizeLine, which belongs to recipe.js`.
- No Chrome: `CHROME=/nonexistent/chrome` gives `FAIL no Chrome or Chromium found at CHROME=/nonexistent/chrome; install one or set CHROME=/path/to/chrome`, exit 1. A missing `--pkg` and a `--pkg` with no package also exit 1 with their own `FAIL` lines. `demo_smoke.mjs` with the same bad `CHROME` exits 1 with its own message.

**Slim wasm:** unchanged. On the Mac: raw 4,148,451 B, gzip 1,195,748 B, brotli 833,619 B. In CI (`ubuntu-26.04`): raw 4,174,347 B, gzip 1,196,820 B, brotli 836,219 B, the number the design recorded from earlier CI logs.

**CI at head `416a240`, every leg green** (read from the PR, not polled):
- `ddphotos page parity (native vs wasm)`, 5 m 42 s. "Parity harness" took 4 m 23 s (14:14:49 to 14:19:12). The new step "Edit page in headless Chrome" took 32 s (14:19:12 to 14:19:44), against the design's 25–45 s estimate. Its log has the summary line `16 table rows, 36 crop rows, 17 replay rows, 26 orientation rows, 2 alpha rows, 153 crop-mode rows; 0 row failures, 0 other failures, K=1 (cap 78)` and `PASS H1 page loads`, `PASS H2 renders at load`, `PASS H3 three quick size changes add one preview`, `PASS H4 the CLI sentence on W`, `PASS H5 the panel's recipe on W`, `PASS H6 renders at load on R`, `PASS H7 the CLI sentence on R`, `PASS H8 no page errors`, then `ddphotos-page: 8 rows, 0 failed`.
- `pages` "build + browser smoke", 3 m 31 s: success, `attached to headless Chrome (google-chrome)` and the demo-smoke ✓ line, on the shared lib.
- The other legs (build/test/clippy/fmt on three OSes, msrv, lean, heic ×2, webp-lossy, wasm round-trip, cargo-deny, DCO, front-matter, cost audit) pass; the release and Pages legs skip.

**Overlap, after the last rebase** (`gh pr list --state open`): only PR #237, which touches `.github/workflows/ci.yml` and `justfile`, in other places (HEIC jobs and recipes). Not touched. `origin/main` had not moved since the build began, so no rebase changed anything.

**Temp folders:** the system temp folder had 5 `ddphotos-*` folders, 1,723,616 KB, before the first run and has the same 5 folders, 1,723,616 KB, after the last. Nothing leaked there: every run used a scratch `TMPDIR`, which I emptied after each red run and which is empty now. One leak, in the scratch folder only: `--self-test` keeps a `ddphotos-orientation-*` folder because its N3 row is deliberately red (existing harness behaviour); I deleted it by hand each time. Before the retry fix below, one local `just ddphotos-page` left a `ddphotos-page-chrome-*` profile (deleted by hand, 0 bytes freed from the system folder).

- **Deviations from spec:**
  - `launchChrome` returns `{ chrome, cdp, exited, port }`, one field more than the spec. `demo_smoke.mjs`'s `file://` check reads the DevTools port.
  - `findChrome` also accepts `CHROME` as a bare name on `PATH` (as `spawn` did before), and reports one that does not exist.
  - The page check's profile delete is retried for about 15 s, not `maxRetries: 10` at 200 ms. The design's version failed once locally with `ENOTEMPTY … /Default` while Chrome's helper processes were still flushing, and left a folder (AC-13). After the change four local runs and the CI run left nothing.
  - `drag-end` and `done` are two reducer cases with the same body, so M5 and E7 each change one action. Merged, E7's edit also turned X15 red.
  - A first version of `cropReducer` returned the same object when the same preset was pressed twice. The design does not ask for it, and under M3 it turned X7P and X7R red, which the spec does not list. Removed.
  - M4's quoted ternary does not exist in this code (see the table above); the closest edit also turns X21 red.
  - Checkpoint 1 pushed a page check that imports `scripts/lib/chrome.mjs`, which arrived in checkpoint 2, so that one commit alone cannot run it. It was red on `main` by design.
- **Follow-up work identified:**
  - The design's open question stands: `MIN_CROP = 64` is a usability floor and does not stop a watermark refusal on a thin strip. STAGE-060's watermark-fit entry is ordered before the DOM entry.
  - `--self-test` keeps `ddphotos-orientation-*` and the parity harness keeps `ddphotos-*` folders on any red run; both are existing and unaddressed.
  - M4's cell should say X10 and X21; the design's table is not edited.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The spec was clear: its literals made the 153 rows mechanical, and all 153 passed on the first run. What slowed me was the ordering of the checkpoints. The page check at checkpoint 1 needs `scripts/lib/chrome.mjs`, which the prompt schedules for checkpoint 2, so I wrote the lib first and committed it second. The spec's M4 and E7 edits also quote code shapes (a ternary guard, a lone `done` case) that I had to match by structure.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Two things. First, Chrome's helper processes can keep writing into the profile after the browser exits, so "await the exit, then delete" is not enough on its own; the delete needs a longer retry than the design's 2 s. Second, the table's red sets depend on code structure (shared helpers, merged cases), so the spec should say which cases are separate.

3. **If you did this task again, what would you do differently?**
   — I would write the reducer from the mutation table first: split `drag-end` and `done`, keep `toBox` separate for the drag and key paths, and leave out the same-object shortcut on `preset`. Those three choices cost me a re-run each. I would also build the scratch mutation worktree before the first push, since a 25 s harness makes the full table cheap. No design call turned out wrong; the one cell that needs a correction is M4's (X10 and X21).

### Punch list (verify)

- **Comments as they now read:**
  - `scripts/lib/chrome.mjs`: "`sessionId` addresses an auto-attached target — the engine's Web Worker is one, and it is a genuinely separate target with its own Runtime and its own Network domain."
  - `scripts/lib/chrome.mjs`: "Chrome's start-up gets 90 s because a cold Chrome on a loaded CI runner can take well over 10 s to open its DevTools port." (`CHROME_STARTUP_MS` unchanged.)
  - `tests/ddphotos_page.mjs`, `waitFor`: ends "... and after `timeoutMs` otherwise."
- **X31 rows** (`p3:2 p3:2`, same-object `false`), literals copied from the X1 row of the same shape (the `p3:2` press from `reset`):
  - X31W from X1W and X31P from X1P: `aspect 3:2 follow`, preset `3:2`, no drag, step `aspect = "3:2"` + `orientation = "match"`.
  - X31R from X1R: same literals as X1W.
  - X31S from X1S: `aspect 3:2`, preset `3:2`, no drag, step `aspect = "3:2"`.
  - Counts: crop-mode rows 153 to 157 (118 reducer rows); `MODE_ROW_COUNT` and the two comments holding the count updated.
- **Negative control** (`return s` added to `case "preset"` when the preset is already active and no drag; run on the worktree's `recipe.js`, then restored with `git checkout`): red on all four. First failure line: `FAIL mode-reducer: X31W "p3:2 p3:2": same state true, expected false`. X7W/P/S/R (`p3:2 sw p3:2`) also went red, three failures each; no other row. Restored: green.
- **Parity summary:** `16 table rows, 36 crop rows, 17 replay rows, 26 orientation rows, 2 alpha rows, 157 crop-mode rows; 0 row failures, 0 other failures, K=1 (cap 78)`. Page check last line: `ddphotos-page: 8 rows, 0 failed`. Demo smoke last line: `demo-smoke: the demo loads crustyimg 0.8.2 in a real browser over HTTP, runs the web flow client-side, and funnels to the CLI. ✓`.
- **CI at 3852b45:** all legs pass (the release/publish legs skipped); the CI log's parity line and page-check line match the above.

### Punch list 2 (CI race)

- **Cause:** `open()` sent `Page.navigate` and returned, and the navigation command returns once the response has begun, not once the new document has parsed, so the first `waitFor` poll could run in a document with no `#preview` (CI at 10ae6e5: `Cannot read properties of null (reading 'src')`).
- **The new `open()`** (only `tests/ddphotos_page.mjs` changed):
  ```js
  /// Navigate and wait until the new document is the one being read. The navigation
  /// command returns before the new page exists, so without this wait the first poll can
  /// run in the previous document (a blank page, or the last photo's page).
  async function open(path) {
    const url = `${baseUrl}${path}`;
    // Marks the document we are leaving; the new document will not have it.
    await cdp.eval("window.__leaving = true");
    await cdp.send("Page.navigate", { url });
    const deadline = Date.now() + 30_000;
    while (Date.now() < deadline) {
      // A poll can land mid-navigation, so a throw here means "not there yet".
      const arrived = await cdp
        .eval(`window.__leaving !== true && location.href === ${JSON.stringify(url)} && document.readyState !== "loading"`)
        .catch(() => false);
      if (arrived) return;
      await sleep(50);
    }
    throw new Stop(`timed out waiting for ${url} to load`);
  }
  ```
  `waitFor` still lets predicate throws fail.
- **(a) The CI failure.** A scratch preload wrapper (not committed) answers `/edit/index.html` with its headers at once and its body 2 s later. A delay on the headers alone did **not** reproduce it: `Page.navigate` waits for the response to begin, so that run passed (8 rows, exit 0).
  - Without the fix, body delayed: exit 1, `FAIL Error: TypeError: Cannot read properties of null (reading 'src')` ... `at async waitForFirstPreview ... run`, then `ddphotos-page: 0 rows, 1 failed`.
  - With the fix, same delay: exit 0, `ddphotos-page: 8 rows, 0 failed`.
- **(b) The stale page.** A scratch copy logged `location.search` when `waitForFirstPreview("R")` returned, with the delay on `photo=2` only.
  - Without the fix: **not reproduced**. Body delayed: exit 1, the same `reading 'src'` failure on R (5 rows, 1 failed), the new empty document and not the old W page. Headers delayed: `location.search = ?album=test&photo=2`, 8 rows, 0 failed. The stale-W form is a hypothesis from reading the code; I could not make it happen with a server delay.
  - With the fix, body delayed: `PROBE ... location.search = ?album=test&photo=2`, exit 0, 8 rows, 0 failed.
- **Five page-check runs, fix, no delay:** exit codes 0, 0, 0, 0, 0. Last line: `ddphotos-page: 8 rows, 0 failed`.
- **Parity:** `16 table rows, 36 crop rows, 17 replay rows, 26 orientation rows, 2 alpha rows, 157 crop-mode rows; 0 row failures, 0 other failures, K=1 (cap 78)` (exit 0). **Demo smoke** (exit 0): `demo-smoke: the demo loads crustyimg 0.8.2 in a real browser over HTTP, runs the web flow client-side, and funnels to the CLI. ✓`.
- **Temp folders:** `ddphotos-*` in the system temp folder: 5 folders, 1,723,616 KB before and after; none deleted.

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
