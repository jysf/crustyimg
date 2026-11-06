---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-147
  type: story                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-017
  stage: STAGE-058
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: claude-opus-4-7     # usually same Claude, different session
  created_at: 2026-11-05

references:
  decisions:
    - DEC-095
    - DEC-102
    - DEC-110
    - DEC-112
    - DEC-003
    - DEC-017
  constraints:
    - untrusted-input-hardening
    - single-image-library
    - every-public-fn-tested
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-131
    - SPEC-141
    - SPEC-142
    - SPEC-145
    - SPEC-110

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-058's <capability>". Optional; null is acceptable.
value_link: "STAGE-058's quarter-turn `rotate` and `flip` steps, exact in pixels, and the one rule for orientation tags after any geometry step"

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

# SPEC-147: rotate by quarter turns and flip, as recipe steps

## Context

PROJ-017 makes every geometry edit a recipe step, so the ddphotos edit page, the CLI and a
hand-written recipe share one saved recipe. This is STAGE-058's backlog item "`rotate` (quarter
turns) and `flip`", build-order line 4 of 13, after SPEC-141 (the corpus output guard) and
SPEC-145 (`crop`, DEC-112). A recipe author today can't turn or mirror an image at all: on 0.8.2
`op = "rotate"` and `op = "flip"` both fail at load with `unknown operation`. The only rotation
crustyimg performs is `auto-orient`'s, driven by the EXIF tag, so a scan stored on its side or a
selfie that needs mirroring has to leave crustyimg and come back. STAGE-059's any-angle
straighten extends this spec's `rotate` op and key table, and line 8 (`edit` flags) needs both.

SPEC-145 deferred one question to this spec, which owns it in full: what an EXIF orientation tag
and `PixelXDimension`/`PixelYDimension` mean after a geometry step on an input that was not
auto-oriented.

## Goal

A recipe can say `op = "rotate"` with `angle` a multiple of 90, or `op = "flip"` with
`axis = "horizontal" | "vertical"`, and get exactly the input's pixels moved, at the input's own
colour type and bit depth, the same on native and wasm. The rule for orientation tags after any
geometry step is written down once, so crop, rotate, flip, pad, trim and straighten all follow it.

## Frame

*Framed 2026-11-05 on `main` at `17605b3` (crustyimg 0.8.2, release build in a fresh scratch
`CARGO_TARGET_DIR`, deleted after). Every probe below ran in a scratch directory and none of it is
committed.*

### Verdict: GO, [S], one spec, two ops

### Premise check: confirmed

- `with_builtins` (`src/operation/registry.rs:296-309`) registers `identity`, `invert`, `resize`,
  `auto-orient`, `watermark`, `crop`. No `rotate`, no `flip`.
- Driven: `[[step]] op = "rotate"` / `angle = 90` → `error: unknown operation 'rotate'`, exit 1.
  `op = "flip"` / `axis = "horizontal"` → `error: unknown operation 'flip'`, exit 1.
- **`AutoOrient` already does the pixel work.** `AutoOrient::apply` (`src/operation/mod.rs:1123`)
  calls `DynamicImage::apply_orientation`, which in `image` 0.25.10 (`dynimage.rs:1161`) is
  `rotate90()`, `rotate180_in_place()`, `rotate270()`, `fliph_in_place()` and `flipv_in_place()`.
  `rotate` and `flip` should call the same `DynamicImage` methods. They match on every
  `DynamicImage` variant, so colour type and bit depth are kept by construction (DEC-095 needs no
  widen/narrow here). `AutoOrient` then returns `Image::from_parts(pixels, fmt, None)`: it
  **drops the whole metadata bundle**, not just the tag (DEC-017). `crop` instead uses
  `img.with_pixels(…)` (`src/operation/crop.rs:383`) and keeps the bundle in memory.

### Orientation and metadata: what I drove

Inputs made with ImageMagick, tags written with `exiftool -Orientation#=6
-ExifIFD:ExifImageWidth=60 -ExifIFD:ExifImageHeight=40 -Artist=probe` (60×40 JPEG and PNG), and a
60×40 PNG with a 10×10 red block at the stored top-left, tagged 6. Every output read with
`exiftool -s -Orientation# -ExifImageWidth -ExifImageHeight -ImageSize -Artist`:

| run | output | Orientation | ExifImageWidth/Height | Artist |
|---|---|---|---|---|
| `apply` crop `x=0 y=0 width=0.5 height=1`, JPEG/PNG/WebP | 30×40 | absent | absent | absent |
| `apply` `auto-orient`, JPEG/PNG | 40×60 | absent | absent | absent |
| `apply` `invert`, JPEG/PNG | 60×40 | absent | absent | absent |
| `edit --invert`, JPEG/PNG | 40×60 (SPEC-110 bakes first) | absent | absent | absent |

So **the pixel lane writes no EXIF at all** (DEC-003, `docs/api-contract.md:209-213`). The
`crop` bundle kept by `with_pixels` never reaches the file. Three consequences:

1. **The trap the frame prompt describes can't happen today.** No output carries a stale
   Orientation tag, so no viewer rotates twice. `PixelXDimension`/`PixelYDimension` can't be stale
   either: they are absent after `crop`, `auto-orient` and every other op.
2. **The opposite trap is live, and it's not new.** The `invert` recipe on the tagged PNG wrote
   the stored 60×40 pixels with the tag gone (red block still top-left), so a photo that displayed
   as a 40×60 portrait now displays sideways, exit 0. Every recipe without an `auto-orient` step
   does this today, `crop` included. SPEC-110 fixed it for the verbs and scoped `apply` out.
   `edit --save-recipe` now records `auto-orient` as the first step (driven: `edit --invert
   --save-recipe` → `auto-orient`, `invert`; edit and replay both 40×60), so that SPEC-110
   divergence is closed.
3. **For `rotate` it reads as "nothing happened".** ImageMagick `-rotate 90` of the stored
   pixels against crustyimg's `auto-orient` output: `compare -metric AE` = **0** differing pixels.
   So on an orientation-6 input that was not auto-oriented, `rotate angle = 90` acting on the
   stored buffer produces exactly the image the source already displayed as. The author asked for
   a quarter turn and sees none.

C2PA: not driven here. On the pixel lane C2PA is dropped with the rest of the container metadata
(DEC-003), and `rotate`/`flip` add no path to the sink, so that behaviour is unchanged.

### Recommended rule (one paragraph, for DEC-113)

**A geometry step acts on the pixel buffer it receives, and writes no metadata.** That buffer is
the stored orientation unless an earlier `auto-orient` step baked the tag; `apply` adds no
orientation step of its own (SPEC-145's rule, unchanged). The pixel lane carries no EXIF, so no
output of a geometry step has an Orientation tag or `PixelXDimension`/`PixelYDimension`. Neither
op resets, strips or composes the tag, because no tag reaches the output. A recipe meant in
display coordinates puts `auto-orient` first, and every geometry op's doc entry says so. **The
forward guard:** if the pixel lane ever carries EXIF through (a future keep-metadata option), it
must set Orientation to 1 and rewrite or drop the pixel dimensions on the way out, or it reopens
the double-rotation trap. The DEC should name that as the condition that reopens it.

- **Needs a DEC?** Yes, recommend **DEC-113**, short. STAGE-058's five ops, STAGE-059's straighten
  and STAGE-060's page all rely on it, and SPEC-145 deferred it explicitly. It reads DEC-003; it
  supersedes nothing.
- **Touches the metadata path?** No. No sink, metadata or pipeline code changes, so the DEC keeps
  this [S].
- **Changes anything `crop` shipped?** No. Crop's rule is this rule, and its outputs carry no tag.
  The DEC cites DEC-112 and SPEC-145's "defer, with a stated rule".
- Rejected: **`rotate`/`flip` auto-orient implicitly.** It gives a different meaning to `rotate`
  than to `crop` on the same buffer, and a recipe that already starts with `auto-orient` would be
  unaffected anyway. Rejected: **compose the tag** (rotate the stored pixels and write the tag
  back). There is no tag-writing path on the pixel lane to compose into.

### Keys, direction and naming

- **`angle`: a TOML number, integer or float, read as `f64`.** `90`, `90.0`, `-90` and `270` all
  load. `ParamKind::Number { min: -360.0, min_inclusive: true, max: Some(360.0) }` checks type,
  finiteness and range in the key table; the constructor checks "a multiple of 90" (`angle % 90.0
  == 0.0`, exact in `f64` for these values) and normalises to 0/90/180/270. `0`, `360` and `-360`
  load as a no-op, so a page slider at zero emits a valid step. STAGE-059 lifts only the
  multiple-of-90 check; the key's type and range don't change, so no recipe breaks then.
  - Rejected: **`UInt`** (no negatives, and `90.0` refused today but accepted after STAGE-059 is a
    type change on a published key). There is no signed-integer `ParamKind`.
  - Rejected: **a string enum** (`"90" | "180" | "270"`). It can't grow to `2.5`.
  - Rejected: **any magnitude** (`720`, `-450`). A bounded range catches typos; beyond ±360 has no
    use for a quarter turn or a straighten.
- **Direction: positive is clockwise.** Clockwise: `image::imageops::rotate90` (affine.rs:118 maps
  (x,y) → (h−1−y, x)), CSS `rotate()`, ImageMagick `-rotate` (driven: 0 px against
  `rotate90`-based auto-orient), sharp's `rotate()`, and EXIF orientation 6's own "rotate 90 CW".
  Anticlockwise: the maths convention and Pillow's `Image.rotate`. Every image tool a recipe author
  is likely to know turns clockwise. Doc sentence: *"A positive `angle` turns the image clockwise
  as it is displayed after any earlier step; `-90` turns it anticlockwise."*
- **`flip`: `axis = "horizontal" | "vertical"`, with "horizontal" = mirror left↔right.** That is
  `image`'s `fliph` (`flip_horizontal_in`, affine.rs:168: (x,y) → (w−1−x, y)), CSS `scaleX(-1)`,
  and every photo editor's "Flip Horizontal". The other reading (flip across the horizontal axis)
  exists, and ImageMagick names them backwards (`-flop` is left↔right, `-flip` is top↔bottom).
  Recommend keeping `horizontal`/`vertical` (the backlog's and `image`'s names), with the doc
  sentence *"`horizontal` mirrors left and right; `vertical` mirrors top and bottom."*
  Rejected: renaming to `"left-right" | "top-bottom"`, which is unambiguous but matches no other
  tool. A maintainer question (below).
- **One op or two: two ops, one spec.** `rotate` and `flip` are separate verbs-of-intent with
  separate key tables. Merging them (`op = "transform"`, `rotate` + `flip` keys) would give
  STAGE-059 a key table with keys it never reads. Splitting into two specs would duplicate the
  corpus bless, the wasm rows, the `wasm-ci-check.sh` counts and the docs pass for ~40 lines of op
  code each.
- **`rotate 180` = flip both ways** is an identity of the maths and a test: `rotate angle = 180`
  is pixel-equal to `flip horizontal` then `flip vertical`, on a non-square, non-symmetric fixture.

### Refusal table (all at load, DEC-102; none uses the refusal hook)

Shapes follow the messages driven on 0.8.2 for `crop` and `resize` (`recipe step 0 ('crop'):
'width' must be a number > 0 and <= 1, got 800`, `… got the string "a"`, `unknown key 'widht';
expected one of: …`, `'mode' must be one of: …; got "diag"`). Design pins the exact text.

| input | where | message shape |
|---|---|---|
| `angle` missing | constructor | `recipe step N ('rotate'): 'angle' is required` |
| `angle = "90"`, `true` | key table | `'angle' must be a number >= -360 and <= 360, got the string "90"` |
| `angle = 450`, `nan`, `inf` | key table | `… got 450` / `got nan` |
| `angle = 45`, `90.5` | constructor | `'angle' must be a multiple of 90, got 45; rotating by any other angle is not supported yet` (see Q3) |
| unknown key (`angel`, `axis` on rotate) | registry | `unknown key 'angel'; expected one of: angle` |
| `axis` missing | constructor | `recipe step N ('flip'): 'axis' is required (horizontal or vertical)` |
| `axis = "diagonal"`, `"h"` | key table (`OneOf`) | `'axis' must be one of: horizontal, vertical; got "diagonal"` |
| `axis = 1` | key table | `… got the integer 1` (whatever `describe_value` prints) |

None needs the image: every check is on the step's own keys. `apply` has **no failure path**: a
quarter turn or a flip is defined on every buffer, including 1×1 and 1×N. So the STAGE-058 entry
"State when an op uses the refusal hook" doesn't bind here, and neither op touches the batch
"watermark skipped" message.

### Exactness, sized for verification (AGENTS §2)

The matrix: 2 ops × (rotate: 0, 90, 180, 270, −90, 360, `90.0`; flip: 2 axes) × colour types
(L8, La8, Rgb8, Rgba8, L16, La16, Rgb16, Rgba16, plus Rgb32F/Rgba32F since `DynamicImage` keeps
them) × square / non-square × native / wasm. All exact, so it is unit-test loops over seeded
fixtures, not measurements. Rows:

- **Round trips:** rotate 90 four times, rotate −90 then 90, flip twice (each axis) return the
  input's pixels exactly and its `DynamicImage` variant, for every colour type above, on a
  **7×5** non-square buffer seeded independently (an XorShift fill like `crop.rs`'s tests, not
  output of the code under test).
- **Absolute positions, not only round trips** (a round trip passes for an op that does nothing):
  on the 7×5 buffer, `rotate 90` gives 5×7 with input (0,0) at output (4,0); `flip horizontal`
  puts (0,0) at (6,0); checked against `image::imageops` called directly *and* a hand-written
  index map. A negative control: `rotate 90` ≠ identity on that buffer.
- **Dimensions swap** on 90/270/−90 and stay on 0/180/360 and both flips.
- **180 = flip-h ∘ flip-v**, every colour type.
- **Orientation:** a tag-6 PNG through a `rotate 90` recipe gives `rotate90` of the *stored*
  pixels and an output with no Orientation tag (read by an independent EXIF reader, as SPEC-093's
  tests do with `kamadak-exif`); the same input through `auto-orient` + `rotate 90` gives the
  stored pixels turned 180°.
- **Corpus (SPEC-141):** two new recipes, `v2_rotate_90.toml` and `v2_flip_horizontal.toml` on
  the existing three fixtures (`rgb8_2600x1730`, `rgba8_1730x2600`, `rgba16_2600x1730`, which cover
  non-square and 16-bit with alpha). `RECIPES` 16 → 18, `ROWS` 54 → 60, `GOLDEN_LINES` 57 → 63,
  `EXPECTED_DIMS` gains `1730x2600 / 2600x1730 / 1730x2600` for rotate and the unchanged dims for
  flip. No existing line changes (the guard proves it).

### Native and wasm (DEC-110 limb 2)

No resampling, so dimensions and pixels exact. Two rows in `tests/wasm_roundtrip.rs`:
`transform` rotates and flips a seeded image to the same pixels as the native op, and refuses
`angle = 45` with the native message. That moves `scripts/wasm-ci-check.sh` **`FULL_COUNT` 49 → 51
and `SLIM_COUNT` 47 → 49**, and the `--self-test` fixture literal (`l49`, :204) with them, in the
same PR (SPEC-145 went red at merge on exactly this). The corpus rows cover native on all three CI
targets. `tests/ddphotos_parity.mjs` rows wait for the page to emit rotate (STAGE-060).

### Docs

`/usr/bin/grep -rn "rotate\|flip" docs/ recipes/ skills/ README.md` (session logs and probes
aside):
- `docs/recipes.md:80` "**Rotate / flip / trim / pad** **[planned — PROJ-017]**" → two step
  entries beside `crop`'s, each with the direction/axis sentence and "put `auto-orient` first".
  Trim and pad stay planned.
- `docs/recipes.md:242` "**PROJ-006:** `crop`/smart-crop, rotate/flip/trim/pad … **planned**" is
  stale twice (wrong project, crop shipped): rewrite to point at PROJ-017.
- `docs/backlog.md:191-192` (`rotate`, `flip` / `flop` rows): mark shipped as recipe steps.
- `docs/data-model.md`, `recipes/README.md`, `skills/crustyimg/SKILL.md`: add `rotate` and `flip`
  wherever `crop` was added (SPEC-145's surfaces; `tests/docs_ops.rs` checks the data-model names
  resolve).
- No verb anywhere promises `crustyimg rotate` or `crustyimg flip`; no verb line to convert. The
  verb set stays at 14.

### `edit --rotate` / `--flip`

Out. Line 8 owns every geometry flag with the ordering DEC.

### Sizing: [S]

About 80 lines of op code (two structs, two key tables, two registrations) on calls `AutoOrient`
already makes. The cost is the verification: the colour-type loops, the corpus bless across three
targets, two wasm rows with the count move, and five doc surfaces, plus a short DEC. The DEC
touches no metadata path, so it doesn't push this to [M]. No split.

### Scope

**In:** `rotate` (quarter turns, `angle` as above) and `flip` ops, key tables, registrations;
their unit, corpus and wasm rows; DEC-113 (geometry and orientation tags); the doc entries above.

**Out**, and where each lives:
- Any-angle `rotate`, `expand`, `crop-to-fit`, resampling → STAGE-059 (lines 9-11).
- `edit --rotate` / `--flip` and the ordering DEC → STAGE-058 line 8.
- Rotate/flip on the ddphotos page and `ddphotos_parity.mjs` rows → STAGE-060.
- A warning when a recipe without `auto-orient` meets a tagged input → filed in STAGE-058 (below).
- Carrying EXIF through the pixel lane → not planned; DEC-113 names it as the reopening condition.
- Transpose / transverse (rotate + flip in one op) → not needed; two steps compose.

### Rough build file list

- `src/operation/rotate.rs` (new: `Rotate`, `Flip`, `ROTATE_PARAMS`, `FLIP_PARAMS`, unit tests)
  — or one file each; design picks
- `src/operation/mod.rs` (module and re-exports)
- `src/operation/registry.rs` (two `register_with_params` lines; `with_builtins` tests)
- `tests/rotate_flip.rs` (new: load refusals through `Recipe`, CLI rows, the orientation row)
- `tests/recipe_corpus_output.rs`; `tests/fixtures/recipe_corpus/v2_rotate_90.toml`,
  `v2_flip_horizontal.toml`, their `.golden` and three `.output` files each; the corpus `README.md`
- `tests/wasm_roundtrip.rs`; `scripts/wasm-ci-check.sh` (`FULL_COUNT`, `SLIM_COUNT`, self-test)
- `docs/recipes.md`, `docs/data-model.md`, `docs/backlog.md`, `recipes/README.md`,
  `skills/crustyimg/SKILL.md`
- `decisions/DEC-113-*.md` (written at design)

### Questions only the maintainer can answer

1. **Direction:** positive `angle` = clockwise? *Recommend yes* (image crate, CSS, ImageMagick,
   EXIF 6).
2. **`flip` values:** keep `horizontal` = mirror left↔right, or rename to `left-right` /
   `top-bottom`? *Recommend keep*, with the one-line doc sentence.
3. **The non-multiple message:** the backlog says it "names the any-angle stage", but user-facing
   text reads behaviour-first with no internal ids. *Recommend* "rotating by any other angle is not
   supported yet" with no `STAGE-059` in it.
4. **`angle = 0` / `±360`:** load as a no-op, or refuse as pointless? *Recommend accept*, so the
   page can emit a zero straighten.

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

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- Any-angle `rotate`, `expand`, `crop-to-fit` and resampling: STAGE-059 (build-order lines 9-11),
  which lifts the multiple-of-90 check on this spec's `angle` key.
- `edit --rotate` / `--flip` and the geometry ordering DEC: STAGE-058, build-order line 8.
- Rotate/flip controls on the ddphotos page and `tests/ddphotos_parity.mjs` rows: STAGE-060.
- A warning when `apply`/`build` runs a recipe with no `auto-orient` step on a tagged input:
  its own STAGE-058 backlog entry.
- Carrying EXIF through the pixel lane, or rewriting Orientation / `PixelXDimension` there: not
  planned. DEC-113 names it as the condition that reopens the double-rotation trap.
- A combined transpose/transverse op: two steps compose.

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
