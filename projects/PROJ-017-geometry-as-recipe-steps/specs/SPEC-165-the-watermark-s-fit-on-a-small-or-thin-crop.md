---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-165
  type: story                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-017
  stage: STAGE-060
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-7
  implementer: claude-opus-4-7     # usually same Claude, different session
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
  sessions: []
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
the engine whether its mark fits: a smaller mark gets the existing "Made smaller" note, and a mark
that can't fit at any size is left off, with a plain sentence saying why and what to change.

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

- **Any change to the engine's rules** (the 5 % cap, the fit check, the margin). This spec changes what
  the page asks for, not what crustyimg accepts (DEC-104).
- **`MIN_CROP`**: stays 64 (ruled 2026-11-27).
- **The crop mode's DOM**: the next STAGE-060 entry. Its live "Made smaller" note reads this spec's
  verdict.
- **A readable floor** (no mark under 8 px): `guidance/ideas.yaml`, because it changes uncropped outputs.
- **A gate on the slim wasm bundle**: STAGE-060's pending "Gate the slim wasm bundle's size" entry.
- **A shrinking margin** for small boxes: considered and not recommended (see the Frame).
- **Save flow, ddphotos code, and flip, pad or trim on the page**: STAGE-060's `explicitly_does_not`.

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
