---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-165
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: high
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
  decisions: [DEC-032, DEC-045, DEC-104, DEC-109, DEC-110, DEC-112, DEC-118]
  constraints: []
  related_specs: [SPEC-128, SPEC-143, SPEC-161, SPEC-164]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-060's <capability>". Optional; null is acceptable.
value_link: "STAGE-060's crop: a box the user draws never meets an engine refusal of the page's own watermark, on the copy or on the original"

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
      tokens_total: 8719751
      tokens_breakdown:
        input: 124
        cache_creation: 179096
        cache_read: 8468574
        output: 71957
      estimated_usd: 7.15
      rates: "Opus $5/$25 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "Measured after the frame push, from the session transcript (found by content), deduped by message.id (max output_tokens); 62 messages, all claude-opus-5-5, no subagents. Excludes this cost commit's own turn."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-165: The watermark's fit on a small or thin crop

## Context

STAGE-060's Spec Backlog entry "The watermark's fit on a small or thin crop", in PROJ-017. The edit
page sizes the watermark itself (`watermarkSize` in `integrations/ddphotos/edit/recipe.js`, called by
`planEdit`). It caps the size to 90 % of `outW - margin` and to a tenth of the short side, but it does
not know the mark's real drawn size or its visible pixel count, which only the engine's rasteriser
knows (DEC-045). Without a crop that never mattered on the fixture shapes. A cropped box can be thin
or small, and then the engine refuses the page's own recipe (DEC-104).

**What a photo owner sees today.** The preview shows "Preview failed: watermark refused: the 9×2 px
mark plus its 8 px margin does not fit inside the 16×400 px image" (`edit.js` `runPreview`), and Make
the copy fails with "Could not make the copy: …" (`makeCopy`). The message is in pixels, names
nothing they can change, and the only way out is to guess: turn the watermark off, change the size or
the box. The CLI recipe on a 3.75× original can also refuse a mark that the copy accepted (driven
below), so the README's "run it on the original" sentence is not true at the edge.

MIN_CROP stays 64 (ruled 2026-11-27). This lands before the crop mode's DOM entry, the first change
that lets a user draw such a box.

## Goal

The page never asks the engine for a watermark it will refuse, on the copy or on the original. It asks
the engine whether its mark fits: a smaller mark gets the existing "Made smaller" note. When the mark
can't fit at any size, the page doesn't make the copy. It blocks Make with a plain sentence saying why
and what to change, including turning the watermark off. A copy the page makes with the watermark
on always carries it (ruled 2026-11-28).

## Frame

*Frame cycle, 2026-11-27. Every number below was driven with scratch scripts that call the release
binary and `recipe.js`; none of them is committed. "Model" means the engine's `refusal()` rule (DEC-104)
transcribed into JS and fed with marks the engine itself rendered (`crustyimg::text::render_text`,
sizes 1–200). The model agreed with native `apply` on **420 of 420** sampled runs (300 seeded runs over
W, P, R, S, plus 40 per refusal kind), and the backlog's three literals reproduced natively.*

### Verdict: GO, one [M] spec

Not [S]: the recommended fix adds a wasm export, its Rust test, and a change to how `planEdit` gets
its size. See "Sizing" for why it isn't split.

### The premise, driven

The sweep: W (1600×1071), P (1071×1600), R (stored 1600×1071 with an orientation tag, upright 1071×1600)
and S (1071×1071). Five texts: "© 2026 Jane Doe" (15 characters), a 67-character Latin-1 text, a
69-character text with characters outside Latin-1 (— and –), a 120-character text and "© 2026 山田太郎".
The three sizes and seven gravities. Every size the page offers (as published, 1200, 1080, 800, 400).
Boxes with each side 64–160 in 1 px steps, then 16 px steps up to full size, plus no crop. In all,
**37,709,805 planned runs**. R plans exactly as P, and its native runs matched P's.

- **Refusals per kind** (boxes of 64 px and up):
  - the mark wider than the output even at size 1 ("fit-w"): **288,078**;
  - the mark's width plus one margin on a thin strip ("fit-w-margin"): **166,680**;
  - coverage over 5 %: **11,193**;
  - the margin on the short (height) side: **0**. It needs a box under 64 px: the backlog's
    1600×20 at Large reproduced natively ("the 16×3 px mark plus its 32 px margin does not fit inside
    the 1600×20 px image"). With `MIN_CROP = 64` on a 1600 px copy the ratio is at most 25, so the margin
    (2 % of the long side) is at most half the short side. That argument holds only while the copy's long
    side is at most 1600 px.
- **Without a crop: 0 refusals** over 1,575 runs (W, P and S; R plans as P). That holds for the fixture
  shapes only. An *uncropped* 64×1600 photo plans exactly like the P 64×1600 box and would be refused.
  That follows from `planEdit`'s arithmetic and wasn't driven on a file.
- **The smallest clean box, per text** (the short side in copy pixels above which nothing refused,
  over the whole sweep, the same for all three sizes because the refusals happen at size 1):
  - "© 2026 Jane Doe": 66;
  - 67 characters: 158;
  - 69 characters, outside Latin-1: 161;
  - 120 characters: 241;
  - "© 2026 山田太郎": 64 (none refused).

  These are measured over this sweep, not a bound (trap 17).
- **Shrinking rarely helps.** Of the 465,951 refused runs, only the 11,193 coverage runs have any
  size that the engine accepts, and every such size is 4–7 px. The other **454,758 (97.6 %)** have no
  size ≥ 1 that fits with the page's margin.
- **The literals, native.** W 50×50 at Medium: "the mark would cover 5.08 % of the 50×50 px image (127
  of 2500 px)". P 64×1600 @400 → 16×400: "the 9×2 px mark plus its 8 px margin does not fit inside the
  16×400 px image", and the same for R. The backlog's "23×2" was a different text.

### Can the page predict each kind?

- **The mark's width.** `textEms × size` against the engine's rendered width over sizes 1–200 is off
  by **−12.1 to +1.26 px** on ordinary text and **+2.29 px** at worst ("jjj gyp |||"), so it
  *under*-estimates by up to 2.29 px. Outside Latin-1 it overestimates by up to **300 px**: unmapped
  characters draw `.notdef` at 1,536 units, not `GO_WIDEST`'s 2,240. 2,240 is the font's widest
  advance over all 709 mapped glyphs. The mark is the union of ink boxes, not the sum of advances.
- **The mark's height** isn't modelled at all today. It reaches 2× the size at size 1 and about 1.17×
  above that. A glyph probe over every mapped glyph found ink up to 0.05 em past the advances on each
  side for Latin-1 and 0.22 em for other glyphs, and an ink height up to 1.17 em (1.40 em for all
  glyphs).
- **Coverage** is a visible-pixel count of an antialiased raster. Only the rasteriser knows it.
- **(a) exact in `recipe.js`: no**, for any kind. Fit needs the ink box, which depends on per-glyph
  bearings at each size and float pen positions, and coverage needs the raster.
- **(b) a conservative bound: safe, but too costly.** The bound uses the mark's width with ink slack
  (`ceil(ems × size + k × size + 2)`, k = 0.11 Latin-1 / 0.45 other), its height (`ceil(1.17 ×
  size + 2)`, or 1.40) and its bounding box as the coverage bound. That bound is a theorem, because
  every visible pixel is inside the box. The glyph probe confirmed the constants at every size from 1
  to 200, and it let **0** refusals through over the sweep. But it changes **11,895,108** marks that
  the engine accepts today, **903 of them with no crop**, so existing outputs and goldens would move.
- **(c) ask the engine: recommended for every kind.** A wasm export that builds the real text
  `Watermark` and returns its `refusal()` on a given output size is exact by construction. It is the
  engine's rule, not a JS copy of it, so the 5 % cap and the fit rule have one owner.
  - **Cost, measured** with `just wasm-build-slim`: an export `markMetrics(text, size)` returning
    width, height and visible count adds **+1,040 B raw** (4,148,451 → 4,149,491) and **+697 B
    brotli** (833,619 → 834,316) to `pkg-slim`. A verdict export calls the same code plus `refusal()`,
    which is already in the bundle. Design re-measures the chosen signature.
  - **DEC-110** binds native and wasm to exact dimensions on the same recipe. For the export, its
    verdict must equal native `apply`'s verdict on the same output size, checked with literal rows
    against native, never against the page's own code (trap 19).

### What the page does

**The rule (recommended).** `watermarkSize`'s size stays the ceiling, so every mark accepted today
keeps its size. The page then descends from it to the largest size the engine accepts **on the
output and on the output 2 px smaller per side**. The 2 px is DEC-118's worst per-side bound for a box
on an original under 2×, so the CLI recipe on the original takes the same mark (see "The CLI recipe").
If no size ≥ 1 passes, the mark is left off the preview, the copy and both recipes.

Over the sweep that changes only the 465,951 runs refused today plus the 68,907 runs at the replay edge
(below). **No run without a crop changes.** Every changed run's new size is under 8 px or off.

**Options weighed.**
- Shrink further: covered by the rule. It finds every size that exists, but they are all 4–7 px.
- **Drop the mark with a note: recommended** for the 97.6 % with no size.
- Keep the box and block Make: safer for a mark meant as protection, but it blocks a crop the user
  chose, over a mark that would be 1 px tall if it fitted.
- Shrink the margin: not recommended, because the margin is the page's own constant and changing it
  per box makes the recipe harder to read.

**The sentences** (each a recommendation; design fixes the wording):
- The existing note (`#wm-capped`) also covers the new caps, reworded because the reason is no longer
  only width: "**Made smaller to fit this photo: at the chosen size the watermark would not fit, or
  would cover too much of the image.**"
- A new note where the mark is left off: "**No watermark: this crop is too small or too narrow for the
  text. Make the box larger, or use shorter text.**"
- The result summary, when a copy is made with the mark left off, gains: "**No watermark (it does not
  fit this crop).**" A user then never downloads an unmarked copy without the page saying so.
- "Preview failed: watermark refused: …" should no longer appear for the page's own mark. If it does,
  the prediction is wrong, which is a bug, and the message stays as it is.

### The CLI recipe on the original

Driven. `cliRecipe` resizes to the page's long side, but DEC-118 lets a box's output differ by up to
1 px per side on an original at least 2×, and 2 px under 2×. Among cropped edits the engine accepts
today, **33,177** are refused 1 px smaller per side and **68,907** 2 px smaller. None are uncropped,
and every one is at a mark size under 8 px.

Natively, on a Pillow-made **6000×4016** original (3.75×, `tests/ddphotos_upscale.py`): W box
333,0,77,976 @400, the 67-character text at Small, gravity center. The copy makes 32×400 and accepts
the size-1 mark. The CLI makes 31×400 and refuses it: "the 32×2 px mark does not fit inside the 31×400
px image". **30 of 30** edge candidates refused on that original. On the 4× original, 0 of 40 did,
because a whole multiple gives 0 px (DEC-118 rule 1).

So the CLI panel's sentence ("Run it on the original or on a file at least as large as the web copy")
is **false at the edge today**. With the 2 px rule it holds by construction within DEC-118's bound. That
bound is measured, not proven, below 2×, and portrait originals were not driven (DEC-118). The sentence
needs no new clause after this spec. The design should say in DEC-119 that the guarantee is only as
strong as DEC-118's bound.

### Gravity and margin

The page offers **seven** gravities, and no `east` or `west` (`index.html` `#wm-gravity`):
- `southeast`, `southwest`, `northeast`, `northwest`: the margin applies on **both** axes;
- `south`, `north`: **y only**;
- `center`: **neither**.

`watermarkSize`'s `outW - margin` is right for the four corners. For `north`, `south` and `center` it
over-caps, because the engine adds no x margin there, which is safe and only costs size. The height axis
is not checked at all: `fitsHeight` is a tenth of the short side, with no margin and no ink height, and
it holds over the sweep only because of the ratio argument above. With (c) the engine checks both axes
for every gravity, and `watermarkSize` stays only as the ceiling. Driven: P 64×1600 @400 at `center` takes
a size-1 mark (9×2 in 16×400), while `southeast` leaves it off.

### Rows and literals

Native literals came from the model, which agreed with native on every sampled run. Design re-drives
each one natively before writing it.

**Fit rows, in `tests/ddphotos_parity.mjs`, a new section.** Each row runs the page's planned
`pixelRecipe` through native `apply` and wasm `transform` on the copy. Both must succeed at the literal
dimensions, and the planned watermark size, or `off` with no watermark step, is a literal. Each row also
runs a **control**: the same edit at today's size, natively, must be refused with the literal kind. That
proves the row reaches a refusal (trap 13: quote the message).
- F1 coverage: W 0,0,224,64 at 1600, 67 characters, Small, `center` → 224×64, margin 4. Today size 6
  is refused on coverage; the new size is **5** (mark 157×6, 493 px).
- F2 the same at `southeast` → size **5**, because coverage ignores gravity.
- F3 fit-w-margin, vertical: P 0,0,64,1600 at 400, "© 2026 Jane Doe", Medium, `southeast` → 16×400,
  margin 8 → **off**.
- F4 the same, horizontal (trap 20): W 0,0,1600,64 at 400 → 400×16 → size **1** (9×2). Today it is
  accepted, and stays unchanged.
- F5 R 0,0,64,1600 at 400 → 16×400 → **off**. The upright frame is read, not the stored one.
- F6 and F7, square in both orders (trap 20): S 0,0,64,1071 at 400 → 24×400, size **1**; S 0,0,1071,64
  at 400 → 400×24, size **2**.
- F8 gravity: P 0,0,64,1600 at 400, `center` → size **1**, against F3's `off`.
- F9 fit-w (wider at size 1): a 64-wide box at 400 with the 120-character text → **off**. Design picks
  the literal.
- F10 and F11, unchanged: W no crop (1600×1071, size **64**, mark 496×58), and P no crop at 400 with the
  120-character text (268×400, size **4**). The new rule must not move either.

**A replay row, in `REPLAY_ROWS`.** W 333,0,77,976 at 400, 67 characters, Small, `center`, on the 3.75×
original → page 32×400 → **off**. The CLI recipe then has no watermark step and succeeds. Its control is
today's recipe on the same original, which natively refuses "the 32×2 px mark does not fit inside the
31×400 px image".

**Export rows.** A Rust test, and a wasm test in `tests/wasm_roundtrip.rs`, assert the export's
verdict against literal native refusals, at least one per kind plus one accepted case.

**A page row, in `tests/ddphotos_page.mjs`.** The crop DOM doesn't exist yet, so this row uses an
uncropped thin photo (64×1600) in the check's album. It asserts the "No watermark" note is visible and
the preview did not fail. Like every navigation (trap 22), it marks the old window, waits for the new
document at the expected URL, and runs once against a server that delays the body.

**The shared mutant** (trap 19): set the headroom to 0 in `recipe.js`. Native and wasm read the same
recipe, so F1–F11 stay equal on both sides. Only the replay row's literal `off` catches it: the mutated
page plans size 1, and the CLI on 6000×4016 refuses it.

**Count:** 11 fit rows, 1 replay row (`REPLAY_ROW_COUNT` 17 → 18), about 5 export cases (Rust and
wasm) and 1 page row. Design fixes the final count.

### Mutants a working check must catch

- M1, the headroom at 0 (exact on the copy only): caught by the replay row.
- M2, the margin not passed to the engine check (0): caught by F3 and F5, where native refuses at
  today's sizes.
- M3, the check against the crop box instead of the resized output: caught by F3 and F5 (64×1600 copy
  pixels against 16×400).
- M4, width and height swapped in the call: caught by F3 against F4, and F6 against F7.
- M5, `center` treated as a corner: caught by F8.
- M6, the descent starting from `wanted` instead of `watermarkSize`'s cap: caught by F10, F11 and the
  existing golden rows (`GOLDEN_PIXEL_BOTH`, table rows 3, 7, 11), whose sizes would move.
- M7, a refused mark shown as a success (a mark left in the recipe, or no note): caught by the `off`
  rows asserting no watermark step, and by the page row's note.
- M8, coverage checked against the copy box's area instead of the output's: caught by F1 and F2.

Each design row names where its mutant is applied and every row expected red, and quotes each red
reason from a run (traps 12 and 13). A control edits a scratch copy of `recipe.js`, never the real
file (trap 14).

### The wasm size

(c) adds Rust and wasm, measured at +697 B brotli on `pkg-slim` for a `markMetrics` export. The slim
build has **no size gate** (STAGE-060's pending "Gate the slim wasm bundle's size" entry), so this spec
records the delta and asserts nothing.

### Docs

- `integrations/ddphotos/README.md`, the Watermark row: "(scaled to the output, and made smaller
  when the text would not fit a portrait or narrow photo)". This needs "or left off when the crop is
  too small for it".
- The same README's CLI row: "Run it on the original or on a file at least as large as the web copy: a
  smaller file gives a smaller image, or a refused watermark, with no warning." It stays, and now holds
  (see above).
- `integrations/ddphotos/edit/index.html`, the `#wm-capped` hint, and a new hint for the mark left off.
- `recipe.js`'s doc comment on `watermarkSize`, which names only the width and height caps.
- DEC-118 rule 5, which mentions a refused watermark. It needs no edit, but DEC-119 cites it.

**Behaviour sentences the build must drive on every exit path it names** (AGENTS §15):
- "made smaller when it would not fit": the width, height, margin, coverage and replay-headroom caps;
- "left off when the crop is too small": on the preview, on Make, and in both recipes;
- "run it on the original or a file at least as large": at a whole-multiple original (4×) and a
  fractional one (3.75×).

### Recommended DEC (DEC-119, content only; design writes it)

**The edit page asks the engine whether its watermark fits, and leaves it off when no size does.**
- The page's size formula is the ceiling.
- The page takes the largest size up to that ceiling that the engine's own `refusal()` accepts on the
  output and on the output 2 px smaller per side, which is DEC-118's box bound under 2×.
- If none fits, the mark is left off, and the page says so.
- The page never transcribes the engine's 5 % or fit rule.
- The guarantee for the CLI recipe on the original is as strong as DEC-118's bound.

### Sizing

[M], by the size of the verification: 3 kinds × 4 photo shapes × both orders × 7 gravities, a replay
edge on a fractional original, an export with native-literal rows on two targets, and a page row.

**No split recommended.** The export's only caller is the page, so a first half would ship a dead
export, and a second half that couldn't run without it. The design should also decide where the plan
runs. `planEdit` is synchronous on the main thread, and wasm lives in the worker, so either the worker
plans the edit (a `plan` message) or `planEdit` takes a `fits` callback and stays pure for the Node
harness.

### Scope

**In:**
- the wasm export and its Rust and wasm tests;
- `planEdit`'s size from the engine's verdict with 2 px of headroom, and the mark left off when nothing
  fits;
- the three page sentences;
- the fit, replay, export and page rows;
- the README Watermark row;
- DEC-119.

**Out** (each listed under Out of scope below, with where it lives).

### Rough build file list

- `src/wasm.rs`: the export.
- `src/operation/mod.rs`: possibly a `pub` constructor or helper for a text mark's verdict, with no
  rule change.
- `tests/wasm_roundtrip.rs`: the export rows.
- `integrations/ddphotos/edit/recipe.js`: `planEdit` and `watermarkSize`.
- `integrations/ddphotos/edit/worker.js`: the export call, or a `plan` message.
- `integrations/ddphotos/edit/edit.js`: the notes and the result summary.
- `integrations/ddphotos/edit/index.html`: the hints.
- `integrations/ddphotos/README.md`.
- `tests/ddphotos_parity.mjs`: the fit rows, the replay row and `checkModules`.
- `tests/ddphotos_page.mjs`: the page row.
- a 64×1600 fixture under `tests/fixtures/ddphotos/`, with its README line.
- `decisions/DEC-119-….md`.

### Questions only the maintainer can answer

1. Leave the mark off, or block Make, when no size fits? **Recommend leaving it off**, with the note
   and the summary line.
2. Accept (c), a wasm export, over (b), which needs no Rust but changes 11.9M accepted marks, 903 with
   no crop? **Recommend (c).**
3. One [M] spec, or a split into an export half and a page half? **Recommend one spec.**
4. A readable floor (no mark under 8 px) is **not** in this spec. Today a long text at 400 px already
   gets 4–6 px marks with no crop (F11), so a floor changes uncropped outputs. **Recommend filing it as
   an idea** (done: `guidance/ideas.yaml`).

### Maintainer rulings (2026-11-28)

1. **When no size fits, block Make; don't leave the mark off.** This overrides the frame's
   recommendation. The reason: if the watermark is on, every copy and recipe the page produces carries
   it. The page never decides on its own to make an unmarked copy. The user gets one without the mark
   by turning the watermark off, and the message names that as one way out, next to "make the box
   larger" and "use shorter text". Shrinking still comes first: the page takes the largest size up to
   today's ceiling that the engine accepts, with the 2 px headroom, and blocks only when no size ≥ 1
   passes. Every "off" row, the replay row's "off", the "left off" sentences and M7 in the Frame
   become "blocked" rows. Design re-derives them.
2. **(c): ask the engine through a wasm export.** (b) is rejected because it moves 11.9M accepted
   marks.
3. **One [M] spec.** No split.
4. **No readable floor here.** It stays the idea `watermark-readable-floor`.

DEC-119's sentence changes accordingly: "The edit page asks the engine whether its watermark fits, and
blocks Make when no size does."

## Design

*Design cycle, 2026-11-28, on Opus. Every literal below was driven: native `apply` from a release
build of a scratch prototype, the prototype's real slim wasm in Node, `recipe.js` in Node, and a
headless Chrome started by the session with its profile in the scratchpad. The prototype is not
committed. "Today" means `main` at 36defb9.*

### Design corrections to the frame

1. **The coverage rows' literals did not reproduce.** The frame's F1 (W 0,0,224,64 at 1600, "a
   67-character text", Small) never named its text. With this design's 67-character text the engine
   takes today's size 6 there; size 7 is refused at 5.97 %. The coverage rows are now **W 0,0,640,96 at
   400** with the 120-character text, Small: the copy is 400×60, today's size 6 is refused ("the mark
   would cover 5.05 % of the 400×60 px image (1213 of 24000 px); the limit is 5 %"), and the new size
   is 5. A row that resizes is also the only kind that can catch M3 (the box passed instead of the
   output): without a resize the box and the output are the same size.
2. **M8 is gone.** The page computes no area: it hands the output's sides to the engine. Checking
   coverage against the box is M3.
3. **The uncropped 64×1600 photo is refused only at 400 px, and only with a corner position.** Driven
   on a 64×1600 WebP with today's `recipe.js`: as published (64×1600) the page asks for size 3 and
   the engine takes it; at 1200, 1080 and 800 it takes size 2, 2 and 1; at 400 (16×400) `southeast`
   is refused with "the 9×2 px mark plus its 8 px margin does not fit inside the 16×400 px image", and
   `center` is taken at size 1. The page row therefore sets the size to 400.
4. **The replay row's literal depends on its text.** The frame's literal needs a size-1 mark exactly
   32 px wide. "© 2026 Jane Doe. All rights reserved. NOT for reuse without consent" (67 characters)
   draws 32×2. The same text with "Not" draws 31×2, and today's CLI takes that at 31×400. So the text
   is part of the row's literal, and the build must copy it byte for byte.
5. **Portrait originals were driven.** The frame didn't drive them. P box 0,0,126,1600 at 400, on a
   4016×6000 Pillow original (3.75×): the copy is 32×400, today's CLI makes 31×400 and refuses "the
   32×2 px mark does not fit inside the 31×400 px image", and the new rule blocks the edit. The CLI's
   sides minus the page's, with no watermark, over seeded random boxes and sizes:
   - W at 3.75× (6000×4016), 60 boxes: 0,0 on 58; +1,0 on 1; 0,−1 on 1.
   - P at 3.75× (4016×6000), 60 boxes: 0,0 on 59; 0,−1 on 1.
   - P at 2.5× (2678×4000), 60 boxes: 0,0 on all 60.
   - P at 1.5× (1606×2400, under 2×), 40 boxes: 0,0 on 37; +1,0 on 2; −1,0 on 1.

   No side differed by more than 1 px. These are sampled, not a bound. They sit inside DEC-118's
   2 px, so the 2 px headroom holds on the portrait originals driven.
6. **The height axis can be driven without a box under 64 px on the page.** `planEdit` takes any
   box. Only `cropReducer` holds `MIN_CROP`. So F14 uses W 0,0,1600,20 at 1600, Large, `south`, where
   the height check alone refuses: the mark's width is 16 ≤ 1600, and its height plus the margin is
   3 + 32 > 20. The export reports it exactly as native does.
7. **The export is a verdict, at +966 B brotli**, not the frame's `markMetrics` at +697 B. That buys
   one owner for the rule. See "The export".
8. **N5, the framing-shift control, needs a change.** It derives its expected red set from
   `REPLAY_ROWS` and requires every row to be red on `psnr` alone. A blocked row draws no image, so it
   stays green under the shift (driven: N5 went red on R18 and R19 in the prototype until this was
   fixed). The expected set leaves the blocked rows out (18 of 20), and the check asserts they stay
   green.
9. **The frame's "30 of 30" replay count was not re-derived**, because its text is unknown. The design
   drove the literal row and its portrait mirror instead. Over 140 edge candidates (boxes the engine
   takes on the copy but refuses 1 or 2 px smaller, on W and P at 3.75×, 2.5× and 4×), the new rule
   blocked all 140, and the CLI refused none of the edits it allowed. All 140 are size-1 marks, so the
   sweep can't show the rule choosing a size above 1 that the CLI takes. R20 shows that instead.

### The rule

`watermarkSize` (`recipe.js:47`) is unchanged and stays the ceiling. When the watermark is on,
`planEdit` asks the engine for each size from that ceiling down to 1. It takes the first size the engine
accepts on the output **and** on the output 2 px smaller per side, each side floored at 1. It never
goes above the ceiling, so every mark the engine takes today, 2 px smaller included, keeps its size.
When no size passes, the plan is **blocked**. The descent is linear on purpose: "the largest size that
passes" is exact only if every size is asked, and nothing guarantees that acceptance is monotone in the
size. Each size costs one or two questions (the second only when the first passes), so a plan asks at
most 2 × ceiling (192 at 1600 px Large). The blocked rows here ask 1–2, and plan in under 5 ms in Node
(F14's shape with T120: 2 questions, 4.4 ms).

### The blocked state (what the page shows)

- **The message.** `#make-blocked` sits directly above the Make button, `class="hint blocked"`,
  `role="status"`, and is `hidden` unless the plan is blocked. Its text (whitespace collapsed):
  "The watermark does not fit this copy at any size, so the copy cannot be made with it. Make the crop
  or the size larger, use shorter text, or turn the watermark off."
  "The size" is in the list because a larger output size is a way out with no crop at all (fix 3).
- **The preview** shows the **published photo, not a render.** The `<img>` goes back to the URL the
  page shows before the engine's first preview, and the worker is asked for no render. A preview of the
  edit *without* the mark was considered and rejected: the preview is a JPEG blob the user can save with
  "Save image as", so it would be an unmarked copy the page made, against ruling 1. The published photo
  is already public, so showing it gives nothing away. The `#error` box stays clear, and "Preview
  failed: watermark refused: …" never appears for the page's own mark. If it does, the plan is wrong:
  the page row fails on any `#error` failure (`waitFor`'s `ERROR_NOW`).
- **The Make button** is `disabled`, with `aria-describedby="make-blocked"` always set. The reason is
  visible next to it and is read with it. It is disabled, not refusing on click, so the user sees why
  before trying, and no click ends in an error. Disabling can lag a plan by the 200 ms debounce, so
  `makeCopy` re-plans and returns without asking the worker when the new plan is blocked. It also can't
  build a recipe from a blocked plan, because `pixelRecipe` throws on one.
- **The `#wm-capped` hint** is hidden while blocked: the mark wasn't made smaller, it can't be drawn.
- **The CLI and recipe panels** live in `#result`. `schedulePreview` already hides it on every input,
  and it is shown only by a successful Make, so a blocked plan never shows either panel. No panel can
  offer a refused mark, because `cliRecipe` goes through `pixelRecipe`, which throws on a blocked plan.
- **Leaving the blocked state.** Every form input re-plans through `schedulePreview` → `runPreview`,
  with no reload. When the new plan isn't blocked, `showFit` hides `#make-blocked`, re-enables Make and
  renders the preview. The page row drives the two exits the page has today: changing the position (to
  `center`) and turning the watermark off. Shorter text and a larger size are the same code path, and
  the fit rows give their verdicts. Changing the box needs the crop DOM, the next STAGE-060 entry,
  which passes its crop to the same `plan` message.

### The plan's shape and where it runs

**`planEdit(photo, form, crop = null, refuses = null)`: a callback, called in the worker.**
- `refuses(text, size, gravity, margin, color, width, height)` returns `undefined` when the engine
  takes the mark on an image of that size, and the refusal's text otherwise. The page passes the wasm
  export itself, `watermarkRefusal`. The Node harness passes the **same export from the real slim
  wasm package** it already loads (`loadWasm` returns it). That is the engine, never a JS copy of its
  rule (trap 19), and no literal rows stand in for it. `planEdit` throws "planEdit needs the engine's
  watermark check" when the watermark is on and `refuses` isn't a function, so a caller that forgets
  it fails loudly instead of getting an unchecked mark. With the watermark off it isn't needed.
- **It runs in the worker**, because the export is wasm and the wasm lives in the worker. A new `plan`
  message, `{ form, crop }`, returns `{ edit }`. The worker keeps the photo's upright size from `load`,
  which is the size `loadPhoto` returns. `edit.js` no longer imports `planEdit`. It asks the worker,
  then builds the recipes from the edit it gets back. A callback keeps `planEdit` pure and synchronous
  for the harness, and the `plan` message is the only way the page can satisfy it. The frame's two
  options are therefore one design, not alternatives.
- **The result.** Everything it returns today, plus `blocked`: `null`, or `{ reason: "watermark" }`.
  When blocked, `watermark` keeps its fields with `size: null` and `capped: false`. `capped` is now
  `size < wanted`, where `wanted` is `watermarkSize`'s requested size, which `watermarkSize` now
  returns as well. So "Made smaller" also covers sizes the engine lowered. `pixelRecipe` (and through
  it `cliRecipe` and `previewRecipe`'s input) throws "the watermark does not fit this copy at any
  size" on a blocked edit.

**Every caller of `planEdit`, and what it does with `blocked`** (lines on `main` at 36defb9):
- `integrations/ddphotos/edit/edit.js:189` (`readEdit`): **moves to `worker.js`'s `plan` case.**
  `readEdit` becomes `async` and asks the worker. `runPreview` (`edit.js:218`) shows the blocked state
  and asks for no render. `makeCopy` (`edit.js:249`) re-plans, applies `showFit`, and returns when
  blocked.
- `tests/ddphotos_parity.mjs:534` (`recipeFor`, the table and crop rows): passes `ctx.refuses`. No
  row there is blocked; a blocked edit would throw in `pixelRecipe` and fail the row.
- `tests/ddphotos_parity.mjs:766` (`replayRows`): passes `ctx.refuses`. A row whose literal is
  `"blocked"` asserts the plan is blocked and runs its control on the original, without page or CLI
  images (see "Rows").
- `tests/ddphotos_parity.mjs:917` (`minimumSideChecks`), `:940` (`fractionChecks`), `:978` and `:980`
  (`placementChecks`), `:1205` (`reducerRows`) and `:1306` (`sizeLineChecks`): all have the watermark
  off and are **unchanged**, which is itself the check that the watermark-off path needs no callback.
- The new fit rows: pass `ctx.refuses`.

### The export

**`watermarkRefusal(text, size, gravity, margin, color, width, height)`** in `src/wasm.rs`, with
`#[wasm_bindgen(js_name = watermarkRefusal)]`, returning `Option<String>` (`string | undefined` in JS).
- It builds the step's parameters (`text`, `size` as a float, `gravity`, `margin`, `color`) and calls
  `Watermark::from_params`, the same constructor a recipe's `watermark` step uses. On `Ok` it returns
  `refusal_at(width, height)` as text. On `Err` it returns the error's text, for example a mark
  invisible at that size, which `from_params` refuses. The page treats any text as "not this size".
- Opacity isn't a parameter. It changes neither the fit nor the visible count (`refusal`'s doc
  comment), and the page's opacity is at least 0.1. Colour is a parameter because the mark is rendered
  in it. Driven: `#000000`, `#ff0000` and `#ffffff` give the same verdict on E4.
- **`src/operation/mod.rs` gets no new `pub` item.** The body of `Watermark`'s `Operation::refusal`
  (`mod.rs:1903`) moves, unchanged, into `pub(crate) fn refusal_at(&self, bw: u32, bh: u32) ->
  Option<Refusal>` in the inherent `impl Watermark`. `refusal` becomes
  `self.refusal_at(img.width(), img.height())`. This lets the export ask about an output size without
  allocating an image of that size. **No rule changes**: the same comparisons in the same order, pinned
  by the existing refusal tests (`assert_refuses`, `mod.rs:3017`) and the new unit test.
- **Size, measured on the Mac** with `just wasm-build-slim` against `main` at 36defb9:
  - brotli 833,619 → **834,585 B (+966 B)**;
  - raw 4,148,451 → 4,150,504 B (+2,053 B).

  CI's figure differs; the build records CI's. There is no gate (STAGE-060's pending entry), so
  nothing asserts it.
- **DEC-110 for the export:** on the same output size, the export's verdict must equal native
  `apply`'s, in rows with literal expected values on both targets. Rows E1–E5 are below.

### The sentences

- **`#wm-capped`** (`index.html:95`): "Made smaller to fit this photo: at the chosen size the watermark
  would not fit, or would cover too much of the image."
- **`#make-blocked`**: "The watermark does not fit this copy at any size, so the copy cannot be made
  with it. Make the crop or the size larger, use shorter text, or turn the watermark off."
- **The README's Watermark row** (`integrations/ddphotos/README.md:112`): "Text, position, size
  (scaled to the output, made smaller when it would not fit the photo or would cover too much of it,
  and Make is blocked, with the reason, when it cannot fit at any size), colour, opacity. Uses
  crustyimg's bundled font."
- **`recipe.js`'s doc comments**: `watermarkSize` says it is the ceiling, and that the engine decides
  the size under it. `planEdit` documents `refuses` and `blocked`.
- **The CLI row** (`README.md:115`) and **`#cli-note`** (`index.html:153`) are unchanged. "Run it on
  the original or on a file at least as large as the web copy" now holds within DEC-118's bound,
  driven at 4× (R14, R15) and 3.75× (R18–R20).

**Behaviour sentences the build drives on every exit path** (AGENTS §15):
- "made smaller when it would not fit the photo or would cover too much of it":
  - width: F4, F6, F7, F9's ceiling;
  - width plus margin: F11;
  - height: F15;
  - coverage: F1, F2;
  - the headroom: R20.
- "Make is blocked, with the reason, when it cannot fit at any size", on each path:
  - the preview: H9 (published photo, no render, no `#error`);
  - Make: H9 (disabled, `aria-describedby`);
  - both recipes: the fit rows' "pixelRecipe made a recipe from a blocked edit" check, and `#result`
    hidden in H9;
  - the shapes: W (F14), P (F3, F9), R (F5), S (not blockable at 64 px, F6 and F7), and the
    uncropped thin photo T (F12, H9);
  - the originals: 3.75× (R18, R19) and 4× (no blocked 4× row exists, because a whole multiple gives
    0 px, DEC-118 rule 1; R14 and R15 pin it unchanged).
- "the block clears on the next plan": H10 (position) and H11 (watermark off).
- "run it on the original or a file at least as large": 4× (R14, R15) and 3.75× (R18, R19, R20).

### Rows, by family

**Fit rows** (a new `FIT_ROWS` section in `tests/ddphotos_parity.mjs`, `FIT_ROW_COUNT = 15`).
- Each row:
  - plans with the real export;
  - asserts the output's sides and the mark (`<size>[ capped]` or `blocked`) as literals;
  - when not blocked, runs `pixelRecipe` through wasm `transform` and native `apply`, and both must
    succeed at the literal size (decoded with Pillow, as `runTable` does);
  - when blocked, asserts that `pixelRecipe` throws.
- A row whose size changed, or that is blocked, also runs its **control**: the same edit with the mark
  forced to the size the page asks for today. Natively, its stderr must be exactly
  `error: watermark refused: <literal>`, and in wasm `transform` must throw a message containing the
  literal (trap 13).

The rows (texts: T15 is the harness's `WATERMARK_TEXT` "© Jane Doe 2026"; T67 and T120 are below):
- F1: W 0,0,640,96, size 400, T120, Small, `center` → 400×60, **5 capped**. Control size 6: "the
  mark would cover 5.05 % of the 400×60 px image (1213 of 24000 px); the limit is 5 %".
- F2: the same at `southeast` → **5 capped**, same control. Coverage ignores gravity.
- F3: P 0,0,64,1600, 400, T15, Medium, `southeast` → 16×400, **blocked**. Control size 1: "the 9×2 px
  mark plus its 8 px margin does not fit inside the 16×400 px image".
- F4: W 0,0,1600,64, 400, T15, Medium, `southeast` → 400×16, **1 capped** (unchanged). F3's other
  order (trap 20).
- F5: R 0,0,64,1600, 400, T15, Medium, `southeast` → 16×400, **blocked**, F3's control. The upright
  frame is read.
- F6: S 0,0,64,1071, 400, T15, Medium, `southeast` → 24×400, **1 capped** (unchanged).
- F7: S 0,0,1071,64, the same → 400×24, **2 capped** (unchanged). The square in both orders.
- F8: P 0,0,64,1600, 400, T15, Medium, `center` → 16×400, **1 capped** (unchanged), against F3.
- F9: P 0,0,64,1600, 400, T120, Medium, `center` → 16×400, **blocked**. Control size 1: "the 56×2 px
  mark does not fit inside the 16×400 px image".
- F10: W, no crop, 1600, T15, Medium, `southeast` → 1600×1071, **64** (unchanged).
- F11: P, no crop, 400, T120, Medium, `southeast` → 268×400, **4 capped** (unchanged).
- F12: T, no crop, 400, T15, Medium, `southeast` → 16×400, **blocked**, F3's control. The uncropped
  thin photo.
- F13: T, no crop, 1600, T15, Medium, `southeast` → 64×1600, **3 capped** (unchanged). As published.
- F14: W 0,0,1600,20, 1600, T15, Large, `south` → 1600×20, **blocked**. Control size 2: "the 16×3 px
  mark plus its 32 px margin does not fit inside the 1600×20 px image". The height alone.
- F15: P 0,0,20,1600, 1600, T15, Large, `south` → 20×1600, **1 capped** (unchanged). F14's other
  order.

**Replay rows** (`REPLAY_ROWS`, `REPLAY_ROW_COUNT` 17 → **20**), each an 8th and 9th element
(form overrides, control):
- R18: W box 333,0,77,976, 400, watermark on, T67, Small, `center`, 3.75× → 32×400, **blocked**.
  Control: `cliRecipe` with the mark at size 1, run natively on the 6000×4016 original, stderr exactly
  "error: watermark refused: the 32×2 px mark does not fit inside the 31×400 px image".
- R19: P box 0,0,126,1600, the same form, 3.75× → 32×400, **blocked**, same control text on the
  4016×6000 original. The portrait mirror.
- R20: W box 0,0,640,96, 400, T120, Small, `center`, 3.75× → page 400×60 and CLI 400×60, **5 capped**,
  PSNR 67.84 dB (box floor 59). A size the CLI takes under the new rule.

A blocked replay row skips the page and CLI images and the PSNR, and runs its control instead.

**Export rows** (literal, on both targets). The wasm side is `watermarkRefusal` in
`tests/wasm_roundtrip.rs`. The native side is a unit test in `src/operation/mod.rs` that asserts
`refusal_at(w, h)` equals `refusal(&img)` on a w×h image, with the literal text, and that `apply`
refuses with the same:
- E1: T15, 64, `southeast`, margin 32, 1600×1071 → accepted (`None`).
- E2: T15, 1, `southeast`, 8, 16×400 → "the 9×2 px mark plus its 8 px margin does not fit inside the
  16×400 px image".
- E3: T120, 1, `center`, 8, 16×400 → "the 56×2 px mark does not fit inside the 16×400 px image".
- E4: T120, 6, `center`, 8, 400×60 → "the mark would cover 5.05 % of the 400×60 px image (1213 of 24000
  px); the limit is 5 %".
- E5: T15, 2, `south`, 32, 1600×20 → "the 16×3 px mark plus its 32 px margin does not fit inside the
  1600×20 px image". The height alone.

Each was driven against native `apply` on a blank PNG of that size, and all five were equal. The fit
rows' controls check the same equality through the page's own recipes.

**Page rows** (`tests/ddphotos_page.mjs`, after H7 and before H8, on a third photo
`photo_thin.webp`, 64×1600):
- H9: with the watermark switched on and the size set to 400 in one tick, Make is blocked. Checked:
  - `#make-blocked` shows the literal;
  - Make is disabled with `aria-describedby="make-blocked"`;
  - `#error` is hidden;
  - `#result` and `#wm-capped` are hidden;
  - the preview's path is `/albums/test/full/photo_thin.webp`;
  - the render count is still `{"load":1,"preview":1,"make":0}`.
- H10: the position set to `center` clears the block (`#make-blocked` hidden, Make enabled). Make
  then runs, and the panel's recipe contains `op = "watermark"` and `gravity = "center"\nsize = 1\n`.
- H11: back to `southeast` blocks again, and turning the watermark off clears it.

**Counts.**
- The parity summary line gains ", 15 fit rows", and its replay count goes 17 → 20 (17 + R18–R20).
- The page summary goes from 8 rows to 11 (H1–H8 + H9–H11).
- Every other count stays as it is: 16 table, 36 crop, 26 orientation, 2 alpha, 157 crop-mode,
  `ddphotos-page` H1–H8.
- The fit rows' 15 is F1–F15, one per line of the table above.

### Texts

- T15: "© Jane Doe 2026" (the harness's existing `WATERMARK_TEXT`).
- T67: "© 2026 Jane Doe. All rights reserved. NOT for reuse without consent" (67 characters; size-1 mark
  32×2).
- T120: "© 2026 Jane Doe. All rights reserved. This photograph may not be copied, shared or reused in
  any way without my consent." (120 characters, the page's `maxlength`; size-1 mark 56×2).

### Mutation table

Every row was driven on the scratch prototype, never on the real files (trap 14). Each run copied
`tests/`, `integrations/` and `scripts/` into a scratch folder, applied the one edit, and ran
`tests/ddphotos_parity.mjs` and/or `tests/ddphotos_page.mjs` against the prototype's native
binary and slim wasm. The messages are quoted from those runs. "Green" means every other row of the
named harness passed, and the summary line's counts were unchanged. Because both sides read
`recipe.js`, every guard below is a literal (trap 19). The literal that catches each mutant is named.

- **M1 (the shared mutant), the headroom at 0.** `recipe.js`: `export const FIT_HEADROOM = 2;` →
  `export const FIT_HEADROOM = 0;`.
  - Red: R18, R19, each "planEdit's watermark is 1 capped, expected blocked".
  - Green: every fit row, R1–R17, R20, and every other family.
  - Caught by R18's and R19's literal `"blocked"`. Native against wasm stays equal, because both read
    the mutated `recipe.js`. Driven.
- **M2, the margin not passed to the engine.** `recipe.js`: `refuses(text, size, gravity, margin,
  color` → `refuses(text, size, gravity, 0, color` (both calls in `fitWatermark`).
  - Red: F3, F5 and F12, each "planEdit's watermark is 1 capped, expected blocked", plus "threw:
    watermark refused: the 9×2 px mark plus its 8 px margin does not fit inside the 16×400 px image".
  - Red: F14, "planEdit's watermark is 2 capped, expected blocked", plus "threw: watermark refused:
    the 16×3 px mark plus its 32 px margin does not fit inside the 1600×20 px image".
  - Green: everything else.
  - Caught by the literal `"blocked"` on F3, F5, F12 and F14. Driven.
- **M3, the crop box passed instead of the output.** `recipe.js`: `form.wmGravity, margin,
  form.wmColor, outW, outH)` → `form.wmGravity, margin, form.wmColor, box.w, box.h)`.
  - Red: F1 and F2, each "planEdit's watermark is 6 capped, expected 5 capped", plus "threw: watermark
    refused: the mark would cover 5.05 % of the 400×60 px image (1213 of 24000 px); the limit is 5 %".
  - Red: F3, F5, F9 and F12, each "planEdit's watermark is 1 capped, expected blocked" (plus their
    native refusal).
  - Red: R18 and R19, "planEdit's watermark is 1 capped, expected blocked".
  - Red: R20, "threw: watermark refused: the mark would cover 5.05 % of the 400×60 px image (1213 of
    24000 px); the limit is 5 %".
  - Green: everything else.
  - Caught by F1's and F2's literal 5 (it replaces the frame's M8), and by the `"blocked"` literals.
    Driven.
- **M4, width and height swapped.** `recipe.js`: `form.wmColor, outW, outH)` → `form.wmColor, outH,
  outW)`.
  - Red: F1, "planEdit's watermark is 1 capped, expected 5 capped".
  - Red: F2, F4 and F15, each "planEdit's watermark is blocked, expected …".
  - Red: F3, F5, F9, F12 and F14, each "… is 1 capped (2 capped for F14), expected blocked".
  - Red: F7, "planEdit's watermark is 1 capped, expected 2 capped".
  - Red: R18, R19 and R20.
  - Green: F6, F8, F10, F11, F13 and the other families.
  - Caught by each order's literal: F3 against F4, F14 against F15, F6 against F7 (trap 20). Driven.
- **M5, `center` treated as a corner.** `recipe.js`: the gravity passed to `fitWatermark` is
  `form.wmGravity === "center" ? "southeast" : form.wmGravity`.
  - Red: F8, "planEdit's watermark is blocked, expected 1 capped".
  - Green: everything else, including F1 and R18–R20: coverage ignores gravity, and their blocks
    don't depend on the margin.
  - Caught by F8's literal 1. Driven.
- **M6, the descent from `wanted` instead of the ceiling.** `recipe.js`: `size =
  fitWatermark(refuses, text, fit.size,` → `size = fitWatermark(refuses, text, Math.max(8,
  Math.round(outLong * Number(form.wmSize))),`.
  - Red: C29, "planEdit's watermark is 43, expected 40 capped".
  - Red: C30, "planEdit's watermark is 43 capped, expected 37 capped".
  - Red: R14 and R15, the same messages as C29 and C30.
  - Red: F4, "planEdit's watermark is 6 capped, expected 1 capped".
  - Red: F7, "… 10 capped, expected 2 capped".
  - Red: F15, "… 2 capped, expected 1 capped".
  - Green: the golden rows (`GOLDEN_PIXEL_BOTH`, rows 3, 7 and 11), whose ceiling is their wanted
    size, and F10 and F11.
  - The frame named F10, F11 and the goldens as the catchers. Driven, they stay green. The existing
    literals C29, C30, R14 and R15, plus the fit rows' 1 and 2, catch it.
- **M7 (the frame's M7, made "blocked"), a blocked plan offers the refused mark.** `recipe.js`:
  `size = fitWatermark(…, outW, outH);` → `size = fitWatermark(…, outW, outH) ?? fit.size;`.
  - Parity red: F3, F5, F9 and F12, each "planEdit's watermark is 1 capped, expected blocked", plus
    the engine's refusal on the copy (for example F3's "threw: watermark refused: the 9×2 px mark plus
    its 8 px margin does not fit inside the 16×400 px image").
  - Parity red: F14, "… 2 capped, expected blocked".
  - Parity red: R18 and R19, "planEdit's watermark is 1 capped, expected blocked".
  - Page red, as a Stop: "the page failed waiting for the blocked message on T; the page's error box:
    Preview failed: watermark refused: the 9×2 px mark plus its 8 px margin does not fit inside the
    16×400 px image; page exceptions: none". That is exactly the message the page must never show for
    its own mark.
  - Green: H1–H7, and every other parity row.
  - Caught by the `"blocked"` literals and by H9. Driven.
- **M7b, `pixelRecipe` builds a recipe from a blocked edit.** `recipe.js`: the line `if
  (edit.blocked) throw new Error("the watermark does not fit this copy at any size");` deleted.
  - Red: F3, F5, F9, F12 and F14, each "pixelRecipe made a recipe from a blocked edit".
  - Green: everything else.
  - Caught by the fit rows' throw check. Driven.
- **M7c, Make stays enabled while blocked.** `edit.js`: `$("make").disabled = edit.blocked !==
  null;` → `$("make").disabled = false;`.
  - Red: H9, "Make is enabled".
  - Then a Stop: "timed out waiting for the block again on T; the page's error box: none; page
    exceptions: none". H11's wait needs a disabled Make.
  - Green: H1–H7.
  - Driven.
- **M7d, no message while blocked.** `edit.js`: `$("make-blocked").hidden = edit.blocked ===
  null;` → `$("make-blocked").hidden = true;`.
  - Red, as a Stop: "timed out waiting for the blocked message on T; the page's error box: none;
    page exceptions: none".
  - Green: H1–H7.
  - Driven.
- **M9 (new), the block isn't cleared on the next plan.** `edit.js`: `$("make-blocked").hidden =
  edit.blocked === null;` → `$("make-blocked").hidden = $("make-blocked").hidden && edit.blocked ===
  null;`.
  - Red, as a Stop: "timed out waiting for the block to clear on T; the page's error box: none; page
    exceptions: none".
  - Green: H1–H7 and H9.
  - Driven. H11's watermark-off exit uses the same line, so the mutant reaches it the same way.
- **M10 (new), the preview shows "Preview failed" in place of the message.** `edit.js`: `if
  (edit.blocked) {` (the blocked branch in `runPreview`) → `if (false) {`.
  - Red: H9, 'error box "Preview failed: the watermark does not fit this copy at any size"'.
  - Then a Stop: "the page failed waiting for the block to clear on T; the page's error box: Preview
    failed: the watermark does not fit this copy at any size; page exceptions: none".
  - Green: H1–H7.
  - Caught by H9's `#error` check. `pixelRecipe`'s throw turns the missing branch into a visible
    failure, never an unmarked render. Driven.
- **M11 (new), the export says "accepted" for every input.** `src/wasm.rs`: `Ok(mark) =>
  mark.refusal_at(width, height).map(|r| r.to_string()), Err(e) => Some(e.to_string()),` → `Ok(_mark)
  => None, Err(_e) => None,`, built into its own slim package.
  - Parity red: F1 and F2, "planEdit's watermark is 6 capped, expected 5 capped".
  - Parity red: F3, F5, F9, F12 and F14, each "… expected blocked", each with the engine's refusal on
    the copy.
  - Parity red: R18 and R19, "planEdit's watermark is 1 capped, expected blocked".
  - Parity red: R20, "threw: watermark refused: the mark would cover 5.05 % of the 400×60 px image
    (1213 of 24000 px); the limit is 5 %".
  - Page red, as a Stop: "the page failed waiting for the blocked message on T; the page's error box:
    Preview failed: watermark refused: the 9×2 px mark plus its 8 px margin does not fit inside the
    16×400 px image".
  - Green: every row the rule doesn't move (F4, F6–F8, F10, F11, F13, F15, R1–R17, all other
    families).
  - Caught by the fit and replay literals, which come from native runs, not from the export. Driven.
- **The unit test and `wasm_roundtrip`'s E1–E5:** their literals were driven (export against native
  `apply`, all five equal; the prototype's unit test passed with the other 126 `operation::` tests).
  M11 makes E2–E5 red by construction, since each expects text and gets `None`. That cargo run is
  **not driven**: the design didn't run `wasm-bindgen-test-runner` on the mutant.

### What pins "nothing else changes"

- **The existing families are green on the prototype**, with unchanged counts and literals: 16 table,
  36 crop, 17 replay (R1–R17), 26 orientation, 2 alpha and 157 crop-mode rows. That includes
  `GOLDEN_PIXEL_BOTH` (row 4) and rows 3, 7 and 11, whose recipe text and size are literals. This
  re-drives frame check 1. Without a crop, F10 keeps size 64 and F11 size 4.
- **The self-test** (N1–N5) is green on the prototype with N5's expected set leaving out R18 and R19.
  N1–N4 are unchanged.
- **The table and crop rows' watermark literals** (C29 40 capped, C30 37 capped, R14, R15) didn't move.
  M6 turns exactly those red, so they can.
- **No engine rule changes**: `refusal_at` is the moved body; the existing refusal tests and the five
  export rows pin it.

### DEC-119 (content; the build writes the file)

**The edit page asks the engine whether its watermark fits, and blocks Make when no size does.**
- The page's size formula (`watermarkSize`) is the ceiling.
- The page takes the largest size up to that ceiling that the engine's own `refusal()` accepts, through
  the wasm export `watermarkRefusal`, on the output and on the output 2 px smaller per side. The 2 px
  is DEC-118's box bound for an original under 2×.
- When no size ≥ 1 passes, the plan is blocked:
  - the page renders nothing without the mark;
  - Make is disabled, with a sentence naming the ways out (a larger crop or size, shorter text, the
    watermark off);
  - no recipe is offered.

  The page never makes an unmarked copy on its own.
- The page never transcribes the engine's 5 % cap or its fit rule. The export and native `apply` agree
  on the same output size (DEC-110), in literal rows on both targets.
- The guarantee for the CLI recipe on the original is as strong as DEC-118's bound. Portrait originals
  were driven at 3.75×, 2.5× and 1.5× (at most 1 px per side over 160 sampled boxes), which is
  measured, not proven.
- `affected_scope`: `integrations/ddphotos/edit/recipe.js`, `integrations/ddphotos/edit/worker.js`,
  `integrations/ddphotos/edit/edit.js`, `integrations/ddphotos/edit/index.html`, `src/wasm.rs`,
  `src/operation/mod.rs`, `tests/ddphotos_parity.mjs`, `tests/ddphotos_page.mjs`,
  `tests/wasm_roundtrip.rs`.
- References DEC-045 (only the rasteriser knows the mark), DEC-104 (the refusal), DEC-110, DEC-118
  (rule 5 and the box bound).

## Inputs

- **Files to read:**
  - `integrations/ddphotos/edit/recipe.js`: `watermarkSize` (`:47`), `planEdit` (`:128`),
    `pixelRecipe` (`:165`), `cliRecipe` (`:188`), `loadPhoto` (`:217`).
  - `integrations/ddphotos/edit/edit.js`: `readEdit` (`:188`), `runPreview` (`:214`), `makeCopy`
    (`:247`), `showEditor` (`:300`).
  - `integrations/ddphotos/edit/worker.js`: the protocol and `handle` (`:36`).
  - `integrations/ddphotos/edit/index.html`: `#wm-capped` (`:95`), `#make` (`:135`), `#result`
    (`:138`).
  - `src/operation/mod.rs`: `Refusal` (`:237`), `Operation::refusal`'s contract (`:310–337`),
    `Watermark::from_params` (`:1644`), `Watermark`'s `refusal` (`:1903`), `assert_refuses`
    (`:3017`).
  - `src/wasm.rs`: the exports' conventions (`:119–206`).
  - `tests/wasm_roundtrip.rs`: `mod wasm` and its imports (`:31–35`).
  - `tests/ddphotos_parity.mjs`: `checkModules` (`:409`), `loadWasm` (`:496`), `recipeFor` (`:530`),
    `REPLAY_ROWS` (`:702`), `replayRows` (`:755`), N5 (`:1532` and on), main (`:1700` and on).
  - `tests/ddphotos_page.mjs`: `buildSite` (`:85`), `open` (`:150`), `waitFor` (`:130`), H1–H8.
  - `scripts/serve.mjs` (`:45`): how a page row's server answers.
- **External APIs:** none.
- **Related code paths:** `src/text/mod.rs` (the rasteriser `from_params` calls); DEC-118's
  harness (`tests/ddphotos_upscale.py`).

## Outputs

- **Files created:**
  - `decisions/DEC-119-the-edit-page-asks-the-engine-whether-its-watermark-fits-and-blocks-make-when-no-size-does.md`:
    the content under "DEC-119" above.
  - `tests/fixtures/ddphotos/photo_thin.webp`: a 64×1600 lossless WebP made with Pillow 12.3.0, with
    the generator recorded in `tests/fixtures/ddphotos/README.md`. Its content: pixel (x, y) =
    ((4x) mod 256, ⌊y / 7⌋ mod 256, (x·y) mod 256), saved `lossless=True`.
- **Files modified:**
  - `src/operation/mod.rs`: `refusal_at` (`pub(crate)`), `refusal` delegates, one unit test.
  - `src/wasm.rs`: `watermarkRefusal`.
  - `tests/wasm_roundtrip.rs`: the export rows E1–E5.
  - `integrations/ddphotos/edit/recipe.js`: `FIT_HEADROOM`, `fitWatermark`, `planEdit`'s `refuses`
    and `blocked`, `watermarkSize` returns `wanted`, and `pixelRecipe` throws on a blocked edit.
  - `integrations/ddphotos/edit/worker.js`: imports `watermarkRefusal` and `planEdit`, keeps
    `shown`, and adds the `plan` case.
  - `integrations/ddphotos/edit/edit.js`: `readEdit` asks the worker; `showFit`; `runPreview` and
    `makeCopy` handle `blocked`; `publishedUrl`.
  - `integrations/ddphotos/edit/index.html`: the `#wm-capped` text, `#make-blocked`, and Make's
    `aria-describedby`.
  - `integrations/ddphotos/README.md`: the Watermark row.
  - `tests/ddphotos_parity.mjs`: the fit rows, R18–R20, `ctx.refuses`, `checkModules`, N5.
  - `tests/ddphotos_page.mjs`: the third photo, H9–H11.
  - `tests/fixtures/ddphotos/README.md`: the new fixture's line.
- **New exports:**
  - wasm: `watermarkRefusal(text: string, size: number, gravity: string, margin: number, color:
    string, width: number, height: number): string | undefined`.
  - `recipe.js`: `FIT_HEADROOM` (2) and `fitWatermark(refuses, text, ceiling, gravity, margin, color,
    outW, outH)` (a size or `null`).
  - `planEdit`'s fourth parameter, `refuses`, and its `blocked` field.
- **Database changes:** none.

## Acceptance Criteria

- [ ] **AC-1 The export.** `watermarkRefusal` returns `undefined` exactly when native `apply` takes the
  same text mark on an image of that size. Otherwise it returns the refusal text native prints after
  `error: watermark refused: `. Rows E1–E5, literal, on wasm and native.
- [ ] **AC-2 No rule change, no new `pub`.** `Watermark::refusal` is `self.refusal_at(w, h)`, and
  `refusal_at` is the old body, unchanged and `pub(crate)`. Every existing refusal test passes
  unchanged.
- [ ] **AC-3 The rule.** With the watermark on, `planEdit` takes the largest size from
  `watermarkSize`'s ceiling down to 1 that `refuses` accepts on (outW, outH) and on
  (max(1, outW − 2), max(1, outH − 2)). `capped` is `size < wanted`. Fit rows F1–F15, literal.
- [ ] **AC-4 Blocked.** When no size passes, `blocked` is `{ reason: "watermark" }`, the mark's size is
  `null`, and `pixelRecipe` (and so `cliRecipe`) throws "the watermark does not fit this copy at any
  size". F3, F5, F9, F12 and F14.
- [ ] **AC-5 The callback is required when the watermark is on.** `planEdit` with the watermark on
  and no `refuses` throws "planEdit needs the engine's watermark check". With it off, no callback is
  needed, and the existing watermark-off calls are unchanged.
- [ ] **AC-6 The plan runs in the worker.** `worker.js` plans with `planEdit(shown, req.form,
  req.crop, watermarkRefusal)`, `edit.js` doesn't import `planEdit`, and `checkModules` pins both.
- [ ] **AC-7 The blocked page.** While blocked:
  - `#make-blocked` shows its literal;
  - Make is disabled with `aria-describedby="make-blocked"`;
  - the preview is the published photo and no render is asked for;
  - `#error`, `#wm-capped` and `#result` are hidden.

  H9.
- [ ] **AC-8 Leaving it.** The next plan that isn't blocked clears it with no reload: a changed
  position (H10) and the watermark turned off (H11). A copy made after the block carries the mark
  (H10's recipe).
- [ ] **AC-9 Make fails closed.** `makeCopy` re-plans, and returns without asking the worker for a
  copy when the plan is blocked.
- [ ] **AC-10 The replay edge.** R18 and R19 (landscape and portrait, 3.75×) are blocked, and their
  controls are refused natively on the original with the literal. R20 makes 400×60 with mark size 5 on
  both sides, above the box PSNR floor. `REPLAY_ROW_COUNT` is 20.
- [ ] **AC-11 Nothing else changes.** All 16 table, 36 crop, R1–R17, 26 orientation, 2 alpha and 157
  crop-mode rows pass with their literals unchanged. N1–N4 are unchanged. N5's expected set is the 18
  rows that draw, and R18 and R19 stay green under the shift.
- [ ] **AC-12 The sentences.** The `#wm-capped` text, the `#make-blocked` text and the README's
  Watermark row, each exactly as in "The sentences". The `#make-blocked` text is asserted by H9.
- [ ] **AC-13 DEC-119** is written with the content and `affected_scope` above.
- [ ] **AC-14 The wasm size** delta of the slim build is recorded in Build Completion, Mac and CI.

## Failing Tests

Written during **design**, before build. The build makes these pass. Every expected value is a
literal driven natively at design.

- **`src/operation/mod.rs`** (unit tests)
  - `"refusal_at is refusal on an image of that size"`: for E1–E5, builds the mark with
    `Watermark::from_params` (text, size, gravity, margin, color `#ffffff`). It asserts that
    `refusal_at(w, h)` equals `refusal(&img)` on a blank w×h image, that its `to_string()` is the
    literal (or `None` for E1), and that `apply` refuses with the same `Refusal`.
- **`tests/wasm_roundtrip.rs`** (`mod wasm`, both feature sets)
  - `"watermark_refusal_matches_native"`: E1–E5 through `watermark_refusal`, each equal to its
    literal (`None` for E1).
- **`tests/ddphotos_parity.mjs`**
  - `"fit F1"` … `"fit F15"`: the mark and the output's sides, literal. A row that isn't blocked must
    run in wasm and natively at that size. A blocked row must make `pixelRecipe` throw. Each row with a
    control: native stderr exactly `error: watermark refused: <literal>`, and the wasm message contains
    it. Labels: `fit <id> -- <what>`.
  - `"fit-count"`: 15 rows ran.
  - `"R18"`, `"R19"`: blocked, with the control literal on the 3.75× original. `"R20"`: page and CLI
    400×60, mark `5 capped`.
  - `"replay-count"`: 20.
  - `"modules"`: `worker.js` contains `return { edit: planEdit(shown, req.form, req.crop,
    watermarkRefusal) };`, `edit.js` doesn't import `planEdit`, and `FIT_HEADROOM` and `fitWatermark`
    are owned by `recipe.js`.
  - `"planEdit without the engine's check"` (in `modelChecks`): `planEdit` with the watermark on and
    no `refuses` throws "planEdit needs the engine's watermark check". With the watermark off and no
    `refuses` it returns a plan.
  - `--self-test` `"N5"`: the expected red set is R1–R17 and R20, each on `psnr`, with R18 and R19
    green.
- **`tests/ddphotos_page.mjs`**
  - `"H9 T at 400: Make is blocked, with the reason"`: AC-7's six facts, with the render count
    `{"load":1,"preview":1,"make":0}`.
  - `"H10 T at the centre: the block clears and the copy carries the mark"`.
  - `"H11 T: turning the watermark off clears the block"`.
  - The summary: `ddphotos-page: 11 rows, 0 failed`.

## Implementation Context

*Read this section, and the files it points to, before starting the build cycle. It is the handoff
document, folded into the spec because there is no separate receiving agent.*

### Decisions that apply

- `DEC-045`: only the engine's rasteriser knows the mark's drawn size and visible count, so the page
  asks it.
- `DEC-104`: the refusal itself. This spec changes what the page asks for, not what the engine refuses.
- `DEC-110`: native and wasm agree exactly on dimensions. Here, the export and `apply` agree on the
  verdict.
- `DEC-112`: the crop's fractions; unchanged.
- `DEC-118`: the box bound (2 px under 2×) the headroom comes from, and rule 5 (a smaller file).
- `DEC-119` (new): written by this build.

### Constraints that apply

- `no-unwrap-on-recoverable-paths`: the export returns text, never panics, on any input (a panic in
  wasm aborts the page's engine).
- `untrusted-input-hardening`: the text is user input; `from_params` already checks it against
  `WATERMARK_PARAMS`.
- `decode-once-no-per-op-disk`: unchanged; the export allocates no image.

### Overlap with other open work

Taken 2026-11-28 01:20 UTC with `gh pr list --state open --json number,title,files` and
`git branch -r`.
- `PR #237` (`heifer` swap, parked): touches `.github/workflows/ci.yml`, `justfile`, `Cargo.toml`,
  `Cargo.lock` and HEIC files. It shares no file with this spec. Don't touch it.
- `origin/feat/spec-164-crop-state-model`: SPEC-164's merged branch. Its diff against `main` is the
  pre-squash history (11 files). No open PR.
- No other open PR. The build re-runs both commands at its Step 0 and records any file this map
  missed.

### Prior related work

- `SPEC-128`, `SPEC-143` (shipped): the watermark's recipe step and its refusal rules.
- `SPEC-161` (shipped): the page's crop recipes and the parity harness's crop rows.
- `SPEC-163` (shipped): DEC-118 and the replay rows.
- `SPEC-164` (shipped): the crop state model, `MIN_CROP`, the headless page check and trap 22.

### Out of scope (for this spec specifically)

- **Any change to the engine's rules** (the 5 % cap, the fit check, the margin). This spec changes what
  the page asks for, not what crustyimg accepts (DEC-104).
- **`MIN_CROP`**: stays 64 (ruled 2026-11-27).
- **The crop mode's DOM**: the next STAGE-060 entry. It passes its crop to the `plan` message, and its
  live "Made smaller" note reads this spec's `capped` and `blocked`.
- **A readable floor** (no mark under 8 px): `guidance/ideas.yaml` (`watermark-readable-floor`).
- **A gate on the slim wasm bundle**: STAGE-060's pending "Gate the slim wasm bundle's size" entry.
- **A shrinking margin** for small boxes: considered in the frame and not recommended.
- **Save flow, ddphotos code, and flip, pad or trim on the page**: STAGE-060's `explicitly_does_not`.
- **A version bump, tag or CHANGELOG entry.**

## Notes for the Implementer

- **Build order and checkpoints** (push after each; a long build must checkpoint):
  1. `refusal_at` and its unit test, then `watermarkRefusal` and the wasm rows. Run `cargo test
     --lib`, `just wasm-test` and `just wasm-test-slim`, then commit and push.
  2. `recipe.js` and the fit rows, the replay rows, `checkModules` and N5. Run `just
     ddphotos-parity` (both modes), then commit and push.
  3. `worker.js`, `edit.js`, `index.html`, the fixture and H9–H11. Run `just ddphotos-page`, plus one
     run against a server that delays the body (below), then commit and push.
  4. README, DEC-119, `just wasm-build-slim` size, Build Completion.
- **The mutation table is your negative control.** Drive every row on a scratch copy of the files
  (copy `tests/`, `integrations/` and `scripts/` into a scratch folder and run from there), never on
  the real files (trap 14). Quote each red row's message from your run.
- **CI legs that run this:** `ddphotos-parity` (the parity step, then "Edit page in headless Chrome",
  then wasm clippy) and `wasm-roundtrip` (both feature sets). `wasm-clippy` lints `src/wasm.rs`, so
  run `just wasm-clippy` locally. The native `refusal_at` is used only by the wasm export and `refusal`;
  if native clippy flags it as dead code on a lean build, check `--no-default-features` too.
- **wasm-bindgen's `Option<String>`** returns `undefined` in JS for `None`, which is what `refuses`
  expects. Pass the export itself, not a wrapper, so `checkModules`' line match holds.
- **`size` in the export is a float.** `from_params` takes `size` as a float, and the page's sizes
  are whole numbers.
- **Temp folders.**
  - Before your first harness run, count `/bin/ls -d "$(getconf DARWIN_USER_TEMP_DIR)"ddphotos-*`
    and their size, and delete none: other sessions own some. Design counted 0 before and after.
  - Run every harness and headless command with `TMPDIR` set to a folder in your scratchpad. After the
    first run, confirm the `ddphotos-*` folders landed there.
  - The page check makes its Chrome profile under `TMPDIR`, so it lands in the scratchpad too.
  - Never run a harness while editing files it reads.
  - Empty the folder after reading each red run.
  - Make the large originals (6000×4016, 4016×6000) only in the scratchpad, and delete them at the
    end. Count again at the end and report both counts.
- **The delayed-body run (trap 22).** `startServer`'s `onRequest` can't delay a body. In a scratch
  copy of `scripts/serve.mjs`, for `.html` only, call `res.flushHeaders()`, then pipe the file after
  1,500 ms. **Prove it applied**: a `fetch` must see the headers in under 50 ms and the body after
  about 1,500 ms. Design measured 8 ms and 1,509 ms. Without `flushHeaders()` Node holds the headers
  too, which delays the whole response and is not the trap. Run the page check once against it. Don't
  commit the switch.
- **Nothing reads `recipe.js`'s 5 %, fit or margin rule.** If you find yourself writing `0.05`, a
  margin check or a glyph width in JS for the new rule, stop: that is the frame's rejected option (b).

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
