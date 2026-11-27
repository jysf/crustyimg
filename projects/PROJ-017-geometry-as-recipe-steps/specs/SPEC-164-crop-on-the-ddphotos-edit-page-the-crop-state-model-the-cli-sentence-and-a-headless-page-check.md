---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-164
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
  sessions: []
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

### Overlap with other open work

Design writes this map from the other work's actual file lists, never from their specs or from
memory: `gh pr list --state open --json number,title,files` and `git branch -r` (diff each pushed
feature branch against `main`). Record the date and time it was taken. Open PRs keep changing, so
the build re-runs both commands at its Step 0 and records any file the map missed.

- `PR #NNN` (`<branch>`) — touches `<file>` that this spec also touches; <how they meet>
- (or: none, as of <date time>)

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- The crop mode's DOM: the Crop button, the box over the oriented base, preset, Swap, Reset and
  Done buttons, pointer, touch and keyboard handlers, ARIA, and the size line and live watermark
  note on the page; and their headless rows. STAGE-060's "Crop on the page: the crop mode's DOM".
- The README's Crop row and the reworded "No crop …" limit: that same entry.
- The three-server browser round and the wasm delta: STAGE-060's "Test-bed round and README".
- `rotate`/straighten on the page: STAGE-060's "Straighten on the page".
- The watermark's fit on small or thin crops (the 5 % coverage limit and the margin against the
  short side): STAGE-060's watermark-fit finding entry. This spec adds no recipe logic.
- A size gate on the slim wasm bundle: its existing STAGE-060 entry.
- Any Rust or wasm change.
- Recipe sidecars, any save flow, flip, pad or trim on the page, and ddphotos code (STAGE-060
  `explicitly_does_not`).
- Pinch and two-finger gestures.

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
