---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-147
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
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
    - DEC-113
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
  sessions:
    - cycle: frame
      date: 2026-11-05
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 5126748
      tokens_breakdown:
        input: 88
        cache_creation: 126143
        cache_read: 4964654
        output: 35863
      estimated_usd: 4.17
      note: "Re-measured at ship from the full JSONL (44 message ids, claude-opus-5-5, deduped by .message.id, max output_tokens per id). Rates: Opus $5/$25 per MTok, cache_creation x1.25 input, cache_read x0.10 input."
    - cycle: design
      date: 2026-11-05
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 9337224
      tokens_breakdown:
        input: 116
        cache_creation: 198364
        cache_read: 9068306
        output: 70438
      estimated_usd: 7.54
      note: "Re-measured at ship from the full JSONL (the first figure was taken before the session's final turn) (57 message ids, all claude-opus-5-5, deduped by .message.id, max output_tokens per id). Rates: Opus $5/$25 per MTok, cache_creation x1.25 input, cache_read x0.10 input."
    - cycle: build
      date: 2026-11-05
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 20821686
      tokens_breakdown:
        input: 212
        cache_creation: 244408
        cache_read: 20497072
        output: 79994
      estimated_usd: 8.27
      note: "Re-measured at ship from the full JSONL (identified by its first user message) (106 message ids, all claude-sonnet-5-5, input/cache once per .message.id, max output_tokens per id). Rates: Sonnet $3/$15 per MTok, cache_creation x1.25 input, cache_read x0.10 input. Includes the turns after the first measurement."
    - cycle: verify
      date: 2026-11-05
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 9880552
      tokens_breakdown:
        input: 126
        cache_creation: 171120
        cache_read: 9671117
        output: 38189
      estimated_usd: 6.86
      note: "Measured at ship from the verify session's full JSONL (62 message ids, all claude-opus-5-5, deduped by .message.id, max output_tokens per id). The report's own figure was 9,466,001 / ~$6.51, cut at the report turn; this one includes the turns after it. Rates: Opus $5/$25 per MTok, cache_creation x1.25 input, cache_read x0.10 input."
    - cycle: ship
      date: 2026-11-06
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 2067047
      tokens_breakdown:
        input: 48
        cache_creation: 69518
        cache_read: 1986937
        output: 10544
      estimated_usd: 1.02
      note: "Measured from this session's JSONL (24 message ids, all claude-sonnet-5-5, deduped by .message.id, max output_tokens per id) just before the ship commit; the commit and push turns after it are not counted. Rates: Sonnet $3/$15 per MTok, cache_creation x1.25 input, cache_read x0.10 input."
  totals:
    tokens_total: 47233257
    estimated_usd: 27.86
    session_count: 5
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

## Design

Designed 2026-11-05 on `main` at `85fb481` (crustyimg 0.8.2), against the maintainer's four rulings
of 2026-11-05 (clockwise; `horizontal`/`vertical` kept; no internal id in the refusal; `0`/`±360` a
no-op). **The metadata rule is DEC-113.** Two ops, one spec, **[S], no split**.

### What was driven at design (release build of `85fb481`, scratch `CARGO_TARGET_DIR`, deleted after)

Inputs: `base.png`, a 60×40 blue PNG with a red 10×10 block at the stored top-left (ImageMagick),
and `tag6.png`, the same file with `exiftool "-Orientation#=6"`. Outputs read with
`exiftool -s -s -s -ImageSize` / `-Orientation#`.

| probe | result | control |
|---|---|---|
| `apply` `op = "rotate"`, `angle = 90` on `base.png` | `error: unknown operation 'rotate'`, exit 1 | — the baseline every new row is RED against |
| `apply` `op = "flip"`, `axis = "horizontal"` | `error: unknown operation 'flip'`, exit 1 | — |
| `apply` full-frame crop (`x=0 y=0 width=1 height=1`) on `tag6.png` | 60×40, Orientation absent | input reads Orientation **6**, 60×40 (same reader) |
| `apply` full-frame crop **then** `auto-orient` on `tag6.png` | **40×60**, Orientation absent | `auto-orient` alone: 40×60, Orientation absent |
| `apply` `invert` then `auto-orient` on `tag6.png` | **40×60** | — |
| `crop x = 450` / `-inf` / `nan` / `45.0` / `"90"` / `true` | `'x' must be a number in 0.0..=1.0, got 450` / `got -inf` / `got nan` / `got 45.0` / `got the string "90"` / `got true`, exit 1 | — the table line `angle` will print |
| `resize` with no keys | `error: recipe step 0 ('resize'): missing required key 'mode'`, exit 1 | — the missing-key wording matched below |
| `crop aspect="16:9" gravity = 1` | `'gravity' must be one of: center, …; got the integer 1` | — the `OneOf` wrong-type wording |
| `rustc` one-liner: `format!("a number in {:?}..={:?}", -360.0, 360.0)` | `a number in -360.0..=360.0` | `{}` of `45.0_f64` prints `45`, `{:?}` prints `45.0` (trap 10's split) |
| the same one-liner: `((a / 90.0) as i64).rem_euclid(4) * 90` for the nine angles and `-0.0` | `-360→0, -270→90, -180→180, -90→270, -0→0, 0→0, 90→90, 180→180, 270→270, 360→0`; `-0.0 % 90.0 == 0.0` is `true` | — the normalisation table below |

**Three code facts that shape the design:**
- **The registry renders a closed `Number` range as `a number in -360.0..=360.0`**
  (`src/operation/registry.rs:126`, `{min:?}..={max:?}`), not the frame's `>= -360 and <= 360`.
  The messages below use the real text. No registry change.
- **A bundle kept by an op reaches a later `auto-orient`.** `crop` keeps the metadata bundle
  (`img.with_pixels`, `src/operation/crop.rs:384`; `Image::with_pixels`, `src/image/mod.rs:421`),
  and so does `invert`; `auto-orient` after either still reads the tag (driven above, 40×60).
  `AutoOrient` drops it (`Image::from_parts(pixels, fmt, None)`, `src/operation/mod.rs:1140`).
  **`rotate` and `flip` keep the bundle, as `crop` does.** If they dropped it, they would be the only
  ops after which `auto-orient` silently does nothing, and `from_parts` also resets
  `truncated_jpeg`/`animated_input`/`source_container`, which `with_pixels` carries.
- **No pixel-lane encoder writes the bundle.** `encode_to_bytes_with` (`src/sink/mod.rs:725`) is the
  one place an `Image` becomes bytes, and it passes only `img.pixels()` to the encoder. `image`
  0.25.10's PNG, JPEG, WebP and AVIF encoders all have `set_exif_metadata`
  (`codecs/png.rs:785`, `jpeg/encoder.rs:757`, `webp/encoder.rs:101`, `avif/encoder.rs:134`), so
  carrying EXIF through is a one-call change there. That is the site DEC-113's "reopen" obligation
  is written at (AC-40).

### The ops, in code terms

- **Where:** two new files, `src/operation/rotate.rs` (`Rotate`, `ROTATE_PARAMS`) and
  `src/operation/flip.rs` (`Flip`, `FLIP_PARAMS`), declared in `src/operation/mod.rs` beside
  `pub mod crop;` (`:29-30`) with `pub use rotate::{Rotate, ROTATE_PARAMS};` and
  `pub use flip::{Flip, FLIP_PARAMS};`. Separate files because STAGE-059 grows `rotate.rs` (any
  angle, resampling, `expand`) and `flip` never changes with it.
- **Tables:**

  | op | key | `ParamKind` |
  |---|---|---|
  | `rotate` | `angle` | `Number { min: -360.0, min_inclusive: true, max: Some(360.0) }` |
  | `flip` | `axis` | `OneOf(&["horizontal", "vertical"])` (a `pub(crate) const AXIS_NAMES` in `flip.rs`) |

- **Registration** in `with_builtins` (`src/operation/registry.rs:294-310`), after `crop`:
  `reg.register_with_params("rotate", |p| Ok(Box::new(Rotate::from_params(p)?)), ROTATE_PARAMS);`
  and the same for `flip`. Update the built-in lists in the doc comments at `:265-267` and `:291-293`.
- **`Rotate::from_params(&OperationParams) -> Result<Rotate, RegistryError>`**: runs
  `registry::check_params("rotate", ROTATE_PARAMS, params)` first (as `Crop::from_params` does,
  `crop.rs:109`), then: `angle` absent → `missing required key 'angle'` (the wording of
  `src/operation/mod.rs:813` and `crop.rs:122`); read it with `params.get_f64("angle")`
  (`src/operation/mod.rs:113`), **never `get_f32`**; `quarter_turns(angle)` is `None` → the
  multiple-of-90 refusal, printing the angle **as written** (below).
- **`fn quarter_turns(angle: f64) -> Option<u8>`** (private, pure, one place): `None` when
  `angle % 90.0 != 0.0`; else `Some(((angle / 90.0) as i64).rem_euclid(4) as u8)`, the number of
  clockwise quarter turns, 0..=3. Exact in `f64` for every accepted value: each is a whole multiple
  of 90 in [-360, 360], and `% 90.0` and `/ 90.0` on such values are exact.
- **Printed as written:** move crop's private `fn written(params, key) -> String`
  (`src/operation/crop.rs:190-198`) into `src/operation/registry.rs` beside `float_text` (`:191`) as
  `pub(crate) fn written(...)`, and import it in both `crop.rs` and `rotate.rs`. No behaviour change
  for crop: its R13/R14 rows (`tests/crop.rs`) guard the move. An integer prints by `to_string`, a
  float by `float_text` (`{:?}`), so `45.0` prints `45.0` and `1.1` prints `1.1`.
- **`Rotate` holds** `turns: u8` and the step's map; `params()` returns the map exactly as written
  (`angle = -90` stays `-90`, not `270`; `90.0` stays a float), as `Crop` does.
- **`Rotate::apply`**: `turns == 0` → `Ok(img)`, the same `Image` back, nothing copied; `1` →
  `img.pixels().rotate90()`; `2` → `rotate180()`; `3` → `rotate270()`; then
  `Ok(img.with_pixels(turned))`. **Clockwise** is `image`'s `rotate90`
  (`imageops/affine.rs:118`, input (x, y) → output (h−1−y, x)).
- **`Flip::from_params`**: `check_params("flip", FLIP_PARAMS, params)`, then `axis` absent →
  `missing required key 'axis'`. The table has already limited it to the two names.
- **`Flip::apply`**: `horizontal` → `img.pixels().fliph()` (input (x, y) → (w−1−x, y), left↔right);
  `vertical` → `flipv()` (top↔bottom); `Ok(img.with_pixels(flipped))`.
- **Neither op overrides `Operation::refusal`, and neither `apply` can fail.** A quarter turn or a
  mirror is defined on every buffer, 1×1 and 1×N included. Every `DynamicImage` method used maps
  every variant (`dynamic_map!`), so colour type and bit depth are the input's (DEC-095), with no
  widen and no narrow.

### Normalisation (what `quarter_turns` returns, and what the output is)

| `angle` as written | turns | output (on a W×H input) |
|---|---|---|
| `-360` | 0 | the input, unchanged, W×H |
| `-270` | 1 | 90° clockwise, H×W |
| `-180` | 2 | 180°, W×H |
| `-90` | 3 | 270° clockwise (90° anticlockwise), H×W |
| `0` | 0 | the input, unchanged |
| `90` | 1 | 90° clockwise, H×W |
| `180` | 2 | 180° |
| `270` | 3 | 270° clockwise, H×W |
| `360` | 0 | the input, unchanged |
| `90.0` | 1 | as `90` |
| `-90.0` | 3 | as `-90` |
| `-0.0` | 0 | the input, unchanged |

**A no-op step** (turns 0) returns the `Image` it was given: the same pixels, colour type and bundle.
Its output is byte-equal to its input (AC-9). It is **not** a corpus recipe: the corpus refuses a
recipe whose output equals its fixture and a non-inert step that changes nothing
(`tests/recipe_corpus_output.rs:352-353`, `:366-368`), and loosening that guard for one op would
weaken it for every recipe. The no-op is pinned by a unit row and a binary row instead.

### Index maps (the expected pixels; tests compute them by hand, never with `image::imageops`)

On a W×H input, output pixel (ox, oy) is input pixel:

| op | output size | output (ox, oy) = input | input (0, 0) lands at, on 7×5 |
|---|---|---|---|
| rotate, 1 turn (90° cw) | H×W | (oy, H−1−ox) | (4, 0) |
| rotate, 2 turns | W×H | (W−1−ox, H−1−oy) | (6, 4) |
| rotate, 3 turns (90° acw) | H×W | (W−1−oy, ox) | (0, 6) |
| flip `horizontal` | W×H | (W−1−ox, oy) | (6, 0) |
| flip `vertical` | W×H | (ox, H−1−oy) | (0, 4) |

### Exact messages

**Load only.** Every refusal is a `RecipeError::InvalidOperation`, displayed
`recipe step N ('<op>'): <reason>` (`src/recipe/mod.rs:124`), exit 1 on `apply`, nothing read or
written. Table rows come out of `check_params` (`src/operation/registry.rs:227-251`) unchanged, in
the shapes driven above for `crop` (the range and type lines) and the `OneOf` line; the constructor
rows use `resize`'s `missing required key '…'` (`src/operation/mod.rs:813`). There are **no apply
rows**: neither op can fail at apply.

| id | step body (after `op = "rotate"`) | who | message after `recipe step 0 ('rotate'): ` |
|---|---|---|---|
| R1 | *(no keys)* | constructor | `missing required key 'angle'` |
| R2 | `angel = 90` | table | `unknown key 'angel'; expected one of: angle` |
| R3 | `angle = 90` `axis = "horizontal"` | table | `unknown key 'axis'; expected one of: angle` |
| R4 | `angle = "90"` | table | `'angle' must be a number in -360.0..=360.0, got the string "90"` |
| R5 | `angle = true` | table | `'angle' must be a number in -360.0..=360.0, got true` |
| R6 | `angle = 450` | table | `'angle' must be a number in -360.0..=360.0, got 450` |
| R7 | `angle = -450` | table | `'angle' must be a number in -360.0..=360.0, got -450` |
| R8 | `angle = 360.5` | table | `'angle' must be a number in -360.0..=360.0, got 360.5` |
| R9 | `angle = nan` | table | `'angle' must be a number in -360.0..=360.0, got nan` |
| R10 | `angle = inf` | table | `'angle' must be a number in -360.0..=360.0, got inf` |
| R11 | `angle = -inf` | table | `'angle' must be a number in -360.0..=360.0, got -inf` |
| R12 | `angle = 45` | constructor | `'angle' must be a multiple of 90, got 45; rotating by any other angle is not supported yet` |
| R13 | `angle = 45.0` | constructor | as R12, `got 45.0; …` |
| R14 | `angle = 90.5` | constructor | as R12, `got 90.5; …` |
| R15 | `angle = 1.1` | constructor | as R12, `got 1.1; …` |
| R16 | `angle = 1e-300` | constructor | as R12, `got 1e-300; …` |

| id | step body (after `op = "flip"`) | who | message after `recipe step 0 ('flip'): ` |
|---|---|---|---|
| F1 | *(no keys)* | constructor | `missing required key 'axis'` |
| F2 | `axis = "horizontal"` `angle = 90` | table | `unknown key 'angle'; expected one of: axis` |
| F3 | `axis = "diagonal"` | table | `'axis' must be one of: horizontal, vertical; got "diagonal"` |
| F4 | `axis = "h"` | table | `'axis' must be one of: horizontal, vertical; got "h"` |
| F5 | `axis = "Horizontal"` | table | `'axis' must be one of: horizontal, vertical; got "Horizontal"` |
| F6 | `axis = 1` | table | `'axis' must be one of: horizontal, vertical; got the integer 1` |
| F7 | `axis = true` | table | `'axis' must be one of: horizontal, vertical; got true` |

Each row is single-fault. R3 and F2 rely on `check_params` walking keys in sorted order: `angle`
before `axis`, so R3's valid `angle` passes and `axis` is the unknown key, and F2's `angle` is
unknown before `axis` is read. **Trap 10 rows:** R13 is where `{}` (`45`) and as-written (`45.0`)
differ; R15 is where an `f32` read prints `1.100000023841858`; **R16 is where an `f32` read is
wrong, not just ugly**: `1e-300 as f32` is `0.0`, which passes the table's `f32` range check and
would load as a silent no-op if the constructor read `get_f32`. R6–R8 and R9–R11 share a message
shape and differ only in the value; each is its own data line.

### Sizing: [S], one spec, no split

About 120 lines of op code across two files (two structs, two tables, two registrations, one pure
function, one helper moved), plus unit tests. 23 load rows, all data lines in one table-driven test
per op. The cost is verification: ten colour types, the corpus bless across three targets (both new
recipes byte-identical across targets), two wasm tests with the count move, and nine doc surfaces.
The DEC touches no metadata code (AC-40 is a comment), so it doesn't push this to [M].

## Inputs

- **Files to read:** `src/operation/crop.rs` (the newest op: table :24-66, `from_params` :107-189,
  `written` :190-198, `apply` :335-385, tests); `src/operation/mod.rs` (`get_f64` :113,
  `Resize::from_params` :797-865, `AutoOrient` :1095-1145, `orientation_from_exif_segment`);
  `src/operation/registry.rs` (`ParamKind` :78-185, `float_text` :191, `describe_value` :202,
  `check_params` :227-251, `with_builtins` :291-310, tests :440-790); `src/image/mod.rs`
  (`from_parts` :389, `with_pixels` :421, `MetadataBundle` :483); `src/sink/mod.rs` (module doc
  :18-29, `encode_to_bytes_with` :725); `src/recipe/mod.rs:124`; `tests/crop.rs` (the shape for
  `tests/rotate_flip.rs`); `tests/common/mod.rs:52-170` (EXIF helpers);
  `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs:257-336`,
  `tests/fixtures/recipe_corpus/README.md`; `tests/wasm_roundtrip.rs:375-450`;
  `scripts/wasm-ci-check.sh:18-19,190-230`; `tests/docs_ops.rs`.
- **Decisions:** DEC-113 (this spec's), DEC-003, DEC-017, DEC-095, DEC-102, DEC-110, DEC-112.
- **Related code paths:** `src/operation/`, `src/sink/mod.rs` (comment only), `tests/`, `scripts/`,
  `docs/`, `recipes/README.md`, `skills/`.

## Outputs

- **Files created:** `src/operation/rotate.rs`, `src/operation/flip.rs` (the ops and their unit
  tests); `tests/rotate_flip.rs` (load refusals, exactness across colour types, binary rows, the
  orientation rows); `tests/fixtures/recipe_corpus/v2_rotate_90.{toml,golden,linux-x86_64.output,macos-aarch64.output,windows-x86_64.output}`
  and the same five for `v2_flip_horizontal`.
- **Files modified:** `src/operation/mod.rs` (two `pub mod`, two re-exports);
  `src/operation/registry.rs` (two registrations, doc lists, `written` moved in, tests);
  `src/operation/crop.rs` (imports `written`, its copy deleted); `src/sink/mod.rs` (the
  module-doc paragraph of AC-40, no code); `tests/recipe_corpus_output.rs` and
  `tests/recipe_round_trip.rs` (stems and literals only); `tests/wasm_roundtrip.rs` (two tests);
  `scripts/wasm-ci-check.sh` (two constants, self-test fixtures); `docs/recipes.md`,
  `recipes/README.md`, `docs/data-model.md`, `docs/api-contract.md`, `skills/crustyimg/SKILL.md`,
  `integrations/ddphotos/README.md`, `README.md`, `docs/backlog.md`, `CHANGELOG.md`.
- **New exports:** `crustyimg::operation::{Rotate, ROTATE_PARAMS, Flip, FLIP_PARAMS}`, each op with
  `pub fn from_params(&OperationParams) -> Result<_, RegistryError>` and its `Operation` impl.
- **Database changes:** none. No `CACHE_SCHEMA_VERSION` change, no lockfile change, no version bump.

## Acceptance Criteria

**The ops**
- [ ] **AC-1** `OperationRegistry::with_builtins()` registers `rotate` and `flip`.
  `param_specs("rotate")` is exactly `[angle: Number { min: -360.0, min_inclusive: true, max:
  Some(360.0) }]` and `param_specs("flip")` exactly `[axis: OneOf(["horizontal", "vertical"])]`.
  The six existing ops keep their tables unchanged.
- [ ] **AC-2** Every row of the normalisation table (12 rows) loads, `quarter_turns` gives its
  turns, and `apply` on a seeded 7×5 rgb8 buffer gives that turn's index map exactly (size and
  every pixel).
- [ ] **AC-3** **Clockwise, by absolute position.** On a seeded 7×5 rgb8 buffer, `rotate angle = 90`
  gives 5×7 with input (0, 0) at output (4, 0) and every pixel per the 1-turn map; `angle = 270`
  puts input (0, 0) at (0, 6); `angle = 180` at (6, 4).
- [ ] **AC-4** **Mirror axis, by absolute position.** On the same buffer, `flip axis = "horizontal"`
  puts input (0, 0) at (6, 0) and every pixel per its map; `axis = "vertical"` puts it at (0, 4).
- [ ] **AC-5** Round trips return the input's bytes (`as_bytes()`) and its `DynamicImage` variant
  exactly, for each of the ten colour types (L8, La8, Rgb8, Rgba8, L16, La16, Rgb16, Rgba16,
  Rgb32F, Rgba32F) on a seeded 7×5 buffer: `rotate 90` four times; `rotate -90` then `rotate 90`;
  `flip horizontal` twice; `flip vertical` twice.
- [ ] **AC-6** `rotate 180` equals `flip horizontal` then `flip vertical`, bytes and variant, for each
  of the ten colour types.
- [ ] **AC-7** A quarter turn swaps the dimensions: 7×5 → 5×7 for `90`, `270`, `-90`, `-270`; 7×5
  stays 7×5 for `180`, `-180` and both flips.
- [ ] **AC-8** A single `rotate 90`, `rotate 180`, `flip horizontal` and `flip vertical` each keep the
  colour type and bit depth, for each of the ten colour types.
- [ ] **AC-9** **No-op.** `angle = 0`, `360`, `-360` and `-0.0` each return a buffer byte-equal to
  the input, same variant, same size, on 7×5 rgb8 and rgba16. On the binary, `apply` with
  `rotate angle = 0` on a 7×5 PNG writes a 7×5 PNG whose decoded samples equal the input's, exit 0.
- [ ] **AC-10** `params()` returns the step's map exactly as written: `angle = -90` stays the integer
  `-90`, `angle = 90.0` stays the float `90.0`, `axis = "vertical"` stays itself.
- [ ] **AC-11** Neither op uses the refusal hook, and `apply` never fails: `refusal(&img)` is `None`
  and `apply` is `Ok` for every turn (0–3) and both axes on 1×1, 1×7 and 7×1 rgb8 buffers, with the
  output size per the maps.

**Refusals** (all at load)
- [ ] **AC-12 … AC-27** (one per row) Each rotate row R1–R16 is refused by
  `Recipe::build_pipeline` with `with_builtins()` as a `RecipeError::InvalidOperation` whose
  `Display` is **exactly** `recipe step 0 ('rotate'): ` + that row's message.
- [ ] **AC-28 … AC-34** (one per row) Each flip row F1–F7, the same, with `recipe step 0 ('flip'): `.
- [ ] **AC-35** On the binary, `apply --recipe <R12> in.png -o out.png` exits 1, prints exactly
  `error: recipe step 0 ('rotate'): 'angle' must be a multiple of 90, got 45; rotating by any other
  angle is not supported yet`, and writes no `out.png`.
- [ ] **AC-36** On the binary, `apply --recipe <F3> in.png -o out.png` exits 1, prints exactly
  `error: recipe step 0 ('flip'): 'axis' must be one of: horizontal, vertical; got "diagonal"`, and
  writes no file.

**Orientation and metadata** (DEC-113)
- [ ] **AC-37** **A geometry step acts on the stored pixels and writes no Orientation tag.** Input: a
  seeded 7×5 rgb8 PNG carrying an `eXIf` chunk with Orientation 6, built in the test. On the binary,
  `apply` with one `rotate angle = 90` step (no `auto-orient`) writes a 5×7 PNG whose pixels are the
  1-turn map of the **stored** buffer, and **kamadak-exif** reads the output as having no
  Orientation field, or Orientation 1. Two controls in the same test: kamadak-exif reads
  Orientation **6** on the input bytes (the reader sees a PNG tag), and `apply` with only
  `auto-orient` on the input gives 5×7 equal to the 1-turn map (the tag reaches the pipeline).
- [ ] **AC-38** The same input through `auto-orient` then `rotate angle = 90` gives 7×5 equal to the
  2-turn map of the stored buffer.
- [ ] **AC-39** The same input through `rotate angle = 90` then `auto-orient` gives 7×5, pixel-equal
  to AC-38's output: `rotate` keeps the bundle, so a later `auto-orient` still applies the tag.
- [ ] **AC-40** `src/sink/mod.rs`'s module doc, under `## Encoding` (`:18-29`), gains a paragraph
  saying: the pixel lane writes no container metadata (no EXIF, no ICC); the `Image`'s metadata
  bundle may still hold the source's Orientation tag after a step has turned or flipped the pixels;
  so a change that writes the bundle here must set Orientation to 1 and rewrite or drop
  `PixelXDimension`/`PixelYDimension`, or a viewer turns the image a second time; and
  `tests/rotate_flip.rs` checks the Orientation half. Behaviour-first, no spec or DEC id. No code
  change in `src/sink/`.

**End to end, corpus, wasm**
- [ ] **AC-41** On the binary, `apply` with `flip axis = "horizontal"` on a seeded 7×5 PNG writes a
  7×5 PNG whose pixels are the horizontal map of the input, exit 0.
- [ ] **AC-42** The corpus gains `v2_rotate_90` (one step, `op = "rotate"`, `angle = 90`) and
  `v2_flip_horizontal` (one step, `op = "flip"`, `axis = "horizontal"`), each with `.toml`,
  `.golden` and three per-target `.output` files, added by the README's procedure.
  `EXPECTED_DIMS`: `v2_rotate_90` `["1730x2600", "2600x1730", "1730x2600"]`; `v2_flip_horizontal`
  `["2600x1730", "1730x2600", "2600x1730"]`. The literals become: `RECIPES` **18** stems (the two
  inserted in sorted order), `EXPECTED_DIMS` length **18**, `ROWS` **60**, `GOLDEN_LINES` **63**,
  the `48` in `EXPECTED_DIMS.len() * FIXTURES.len()` → **54**, the doc comments ("all 18", "18
  recipes x 3 fixtures", "54 pixel rows"), `recipe_round_trip.rs` `CORPUS` +2 stems in sorted order
  and `pairs.len()` **18** with message "15 corpus recipes + 3 bundled recipes".
- [ ] **AC-43** For each new recipe, its three per-target `.output` files are byte-identical (no
  float maths on pixels), and the build says so with the `cmp` it ran.
- [ ] **AC-44** **No existing corpus output changes:** `git diff --diff-filter=M --name-only
  origin/main -- tests/fixtures/recipe_corpus/` and the same with `--diff-filter=D` are empty;
  `--diff-filter=A` lists exactly the ten new files; `cargo test --test recipe_corpus_output` and
  `--test recipe_round_trip` pass with no re-bless of an existing file.
- [ ] **AC-45** wasm `transform`: on a 120×80 `png_unique` input, `rotate angle = 90` gives exactly
  80×120, `angle = -90` exactly 80×120, `flip horizontal` and `flip vertical` exactly 120×80, each
  with zero pixels differing from the index map of the decoded input (DEC-110 limb 2, no 1/255
  allowance).
- [ ] **AC-46** wasm `transform` throws, and the message contains exactly the native line, for R12,
  R6 and F3; an identity transform succeeds after each.
- [ ] **AC-47** `scripts/wasm-ci-check.sh`: `FULL_COUNT=51`, `SLIM_COUNT=49` (both new wasm tests run
  in both sets), and the `--self-test` fixtures move with them: the base line is
  `test result: ok. 51 passed; …` in a variable named `l51`; derived files `l1`, `l50` (one under
  full) and `l49` (slim); rows 1 `count full 51` → ok `full set: 51 passed, as expected`; 2
  `count full 1` → fail `full set: 1 passed, expected 51`; 3 `count full 50` → fail `expected 51`;
  4 `count slim 51` → fail `slim set: 51 passed, expected 49`; 5 `count slim 49` → ok
  `slim set: 49 passed, as expected`; 7 (`two`) and 9 (`count fat`) built from `l51`. Row 8
  (`FAILED. 48 passed; 1 failed`) is unchanged. `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh
  --self-test` and `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh workflow` both exit 0, and CI's
  `count full` and slim count steps pass.

**Docs** (verify checks each by reading; `file:line` is on `main` at `85fb481`)
- [ ] **AC-48** `docs/recipes.md:80`: `**Rotate / flip / trim / pad** **[planned — PROJ-017]**`
  becomes two `[today]` entries after the crop entry, each a recipe step with a TOML snippet. The
  **rotate** entry (`op = "rotate"`, `angle = 90`) says: a quarter turn, `angle` a multiple of 90
  from -360 to 360; **"A positive `angle` turns the image clockwise as it is displayed after any
  earlier step; `-90` turns it anticlockwise."** (ruling 1); any other angle is refused for now; put
  `op = "auto-orient"` first when the recipe is meant as the photo is displayed. The **flip** entry
  (`op = "flip"`, `axis = "horizontal"`) says **"`horizontal` mirrors left and right; `vertical`
  mirrors top and bottom."** (ruling 2) and the same `auto-orient` line. A third line keeps
  `**Trim / pad** **[planned — PROJ-017]**`.
- [ ] **AC-49** `docs/recipes.md:242` (`**PROJ-006:** \`crop\`/smart-crop, rotate/flip/trim/pad, …
  **planned**`) becomes two lines: `**PROJ-017:** geometry as recipe steps — \`crop\`, \`rotate\`
  (quarter turns) and \`flip\` today; \`trim\`, \`pad\` and straighten **planned**.` and
  `Smart crop, redaction, auto-color, upscaling — **planned**.` (the wrong project id dropped, no
  new one invented).
- [ ] **AC-50** `recipes/README.md:93`: the op table gains `rotate` ("`angle`, a multiple of 90 from
  -360 to 360, clockwise (below)") and `flip` ("`axis`, `horizontal` or `vertical` (below)") rows
  after `crop`. A new `### The rotate and flip steps` section before `### Step keys are checked`
  (`:243`) gives both as TOML and says: the ruling-1 sentence; the ruling-2 sentence; `0`, `360` and
  `-360` are accepted and change nothing; any angle that is not a multiple of 90 is refused when
  the recipe loads; both act on the image as the step receives it, so put `op = "auto-orient"` first
  when the recipe is meant as the photo is displayed; neither changes a pixel's value or the colour
  type, only its place.
- [ ] **AC-51** `docs/data-model.md:162-164`: "six operations" becomes **eight**, naming `rotate`
  and `flip`. A new `### \`rotate\` and \`flip\` step param keys` section after the crop section
  (before `### Step keys are checked at load`, `:364`) has a `| Op | Key | Type | Range |` table
  (`rotate` / `angle` / number, integer or float / `-360.0..=360.0`, a multiple of 90; `flip` /
  `axis` / string / `horizontal` or `vertical`), the ruling-1 and ruling-2 sentences, and a TOML
  example with `op = "auto-orient"`, `op = "rotate"` and `op = "flip"` steps (which
  `tests/docs_ops.rs` then checks resolve).
- [ ] **AC-52** `skills/crustyimg/SKILL.md:233-235`: the ops list gains `rotate` (`angle`, a
  multiple of 90, clockwise) and `flip` (`axis = "horizontal" | "vertical"`), and the frontmatter
  `description` (`:3`) says "crop, rotate, flip" where it says "crop".
- [ ] **AC-53** `docs/api-contract.md`: a "**Rotate and flip in a recipe.**" paragraph after the
  crop paragraph (`:460-467`) says `op = "rotate"` turns by a multiple of 90 (positive clockwise)
  and `op = "flip"` mirrors on `axis`; a bad key, value or angle is refused when the recipe loads
  (exit **1**, nothing read or written); neither fails at apply; wasm `transform` throws the same
  line; and the pixel lane writes no Orientation tag after either, so a recipe meant as the photo
  is displayed starts with `auto-orient`.
- [ ] **AC-54** `CHANGELOG.md`: under `## [Unreleased]` → `### Added` (`:10`), after the crop entry,
  "**A recipe can rotate and flip.**" — `op = "rotate"` with `angle` a multiple of 90 (positive is
  clockwise) and `op = "flip"` with `axis = "horizontal"` or `"vertical"`, exact in pixels at the
  input's colour type; 0.8.2 and earlier refuse them with `unknown operation`.
- [ ] **AC-55** `integrations/ddphotos/README.md:127-129` says recipes can crop, rotate and flip,
  the edit page offers none of them yet, and colour is planned in PROJ-015. `README.md:244` reads
  "resize, crop, rotate, watermark and more". `docs/backlog.md:191-192`: the `rotate` row says
  quarter turns shipped as a recipe step (any angle is STAGE-059), and the `flip` / `flop` row says
  shipped as `flip` with `axis`. `npm/README.md:102` is unchanged (its "…" covers them).
- [ ] **AC-56** Every user-facing string this spec adds (messages, doc comments, docs) names no spec,
  stage or DEC id. `/usr/bin/grep -nE "SPEC-|STAGE-|DEC-" src/operation/rotate.rs
  src/operation/flip.rs` finds nothing outside `#[cfg(test)]`.

**Hygiene**
- [ ] **AC-57** No version bump, no `CACHE_SCHEMA_VERSION` change (`src/build/cache.rs:81`), no
  lockfile-format change. Why none is needed: no existing recipe's canonical TOML or output changes
  (AC-44); the cache key already folds in the crustyimg version (`src/build/cache.rs:25`); and 0.8.2
  refuses a `rotate` or `flip` recipe at load, so no cached entry or lockfile line for one exists.
- [ ] **AC-58** `just check` is green; CI is green on every leg, including the lean
  (`--no-default-features`) build, both wasm count steps and the wasm bundle-size gate; and
  `crop`'s existing tests pass unchanged after `written` moves (`cargo test --test crop`).

## Failing Tests

Written at design; the build makes them pass. **Baseline on `main`:** every integration row is RED
with `unknown operation 'rotate'` or `unknown operation 'flip'` (load rows, binary rows, wasm rows,
the orientation rows), the corpus rows RED with a missing `v2_rotate_90` / `v2_flip_horizontal`
file, and the unit tests don't compile until the two modules exist. The build quotes one RED message
per test file. **Wrong-reason trap (13):** every refusal row asserts the whole message with `==`, so
a row refused for another row's reason fails visibly. The likely wrong-reason RED is the orientation
control: if the test's `eXIf` chunk never loads, control B gives 7×5, not 5×7, and the test must say
"the input's Orientation tag never reached the pipeline", not "rotate failed".

- **`src/operation/rotate.rs` (`#[cfg(test)] mod tests`)** — calls the private `quarter_turns` and
  `Rotate::from_params` + `apply`, never a copy.
  - `normalises_every_accepted_angle` — AC-2: the 12-row table as data `(written, turns)`; per row
    builds through `from_params`, asserts `quarter_turns` and the stored `turns`, and checks `apply`
    on a seeded 7×5 rgb8 buffer against the hand index map for that turn. Collects every failing
    row by its written angle, then fails; asserts 12 rows ran.
  - `rotate_90_is_clockwise_by_position` — AC-3: named pixel and full map for 90, 270, 180.
  - `rotate_refuses_non_multiples_of_90` — `quarter_turns` returns `None` for 45, 45.0, 90.5, 1.1,
    1e-300 (the unit half of R12–R16).
  - `no_op_angles_return_the_input` — AC-9 unit half: 0, 360, -360, -0.0 on rgb8 and rgba16.
  - `params_round_trip_as_written` — AC-10 for rotate.
  - `refusal_hook_is_not_used_and_apply_never_fails` — AC-11 for rotate (4 turns × 3 shapes).
- **`src/operation/flip.rs` (`#[cfg(test)] mod tests`)**
  - `horizontal_mirrors_left_and_right_by_position` — AC-4, both axes, named pixel and full map.
  - `params_round_trip_as_written` — AC-10 for flip.
  - `refusal_hook_is_not_used_and_apply_never_fails` — AC-11 for flip (2 axes × 3 shapes).
- **`src/operation/registry.rs` (`mod tests`)**
  - `with_builtins_contains_rotate_and_flip` (new) — AC-1.
  - `every_builtin_publishes_a_param_schema` (extended) — AC-1: `keys("rotate") == ["angle"]`,
    `keys("flip") == ["axis"]`, kinds as AC-1; existing asserts unchanged.
  - `one_of_values_are_accepted_by_their_constructors` (extended) — both `axis` values build; the
    literal becomes `6 + 9 + 9 + 2`, message "6 resize modes + 9 watermark gravities + 9 crop
    gravities + 2 flip axes".
- **`tests/rotate_flip.rs` (new)** — through the public API (`Recipe::from_toml`,
  `build_pipeline`, `Pipeline::run`, `crustyimg::operation::{Rotate, Flip}`) and the binary
  (`env!("CARGO_BIN_EXE_crustyimg")`, stdout and stderr redirected to files, exit code read
  directly). Seeded buffers come from an xorshift fill written in the test (as `tests/crop.rs`
  does), one per colour type, never from the code under test. Expected pixels come from the hand
  index maps, never from `image::imageops`.
  - `every_rotate_refusal_names_its_reason` — AC-12…27: the 16-row R table as data
    `(id, body, message)`, each recipe `version = "2"` with one step; runs **every** row, collects
    each mismatch as `"{id}: expected …, got …"` (a row that builds `Ok` is reported as such), and
    asserts 16 rows ran.
  - `every_flip_refusal_names_its_reason` — AC-28…34, the same shape, 7 rows.
  - `round_trips_are_exact_in_every_colour_type` — AC-5: 10 colour types × 4 round trips, each
    failure named by colour type and trip; asserts 40 cases ran.
  - `rotate_180_is_both_flips_in_every_colour_type` — AC-6, 10 cases.
  - `quarter_turns_swap_the_dimensions` — AC-7.
  - `single_steps_keep_colour_type_and_depth` — AC-8: 10 colour types × 4 single steps.
  - `apply_refuses_a_45_degree_rotate_and_writes_nothing` — AC-35 (binary).
  - `apply_refuses_a_diagonal_flip_and_writes_nothing` — AC-36 (binary).
  - `apply_rotate_0_writes_the_input_pixels` — AC-9 binary half.
  - `apply_flips_a_png_end_to_end` — AC-41 (binary).
  - `rotate_acts_on_stored_pixels_and_writes_no_orientation_tag` — AC-37 (binary). The input PNG
    is encoded with `image::codecs::png::PngEncoder` and `ImageEncoder::set_exif_metadata` given a
    bare little-endian TIFF with one Orientation entry (value 6): the bytes of
    `tests/common/mod.rs::wrap_with_orientation_app1`'s payload without the `Exif\0\0` prefix, which
    a new helper `tests/common/mod.rs::tiff_with_orientation(o: u8) -> Vec<u8>` returns (and
    `wrap_with_orientation_app1` then reuses). The output is read with
    `exif::Reader::new().read_from_container(&mut std::io::Cursor::new(bytes))`: `Err(NotFound)`, or
    no `Tag::Orientation` in `In::PRIMARY`, or a value of 1, passes; 6 fails with a message that
    names the obligation ("the pixel lane carried the source's Orientation tag through; it must
    write 1 or nothing"). Controls A and B as AC-37, A first.
  - `auto_orient_then_rotate_turns_twice` — AC-38 (binary).
  - `rotate_then_auto_orient_matches_auto_orient_then_rotate` — AC-39 (binary): asserts 7×5, the
    2-turn map, and equality with AC-38's output bytes after decode.
- **`tests/recipe_corpus_output.rs` / `tests/recipe_round_trip.rs`** — AC-42…44: the two new stems,
  their `EXPECTED_DIMS` and goldens, the literal bumps; no other change. Both new recipes change
  every fixture (rotate changes its size; flip changes its pixels on the seeded, asymmetric
  fixtures), so the corpus's own "expected dims", "equals fixture" and "every acting step acts"
  controls apply to them unchanged.
- **`tests/wasm_roundtrip.rs`** — AC-45…46, `#[wasm_bindgen_test]`, not feature-gated:
  - `transform_rotates_and_flips_exactly` — the four rows, size with `assert_eq!` and every pixel
    against the hand index map of the input decoded with the `image` crate inside the test.
  - `transform_refuses_a_bad_rotate_or_flip_with_the_native_message` — R12, R6, F3,
    `msg.contains(<the exact native line>)`, and an identity transform after each.
- **`scripts/wasm-ci-check.sh --self-test`** — AC-47's rows.
- **`tests/docs_ops.rs`** — unchanged; it goes red if AC-51's example lands before the ops are
  registered, which is the order guard for that doc.

### Mutation table (traps 12 and 14)

Apply one at a time, run `cargo test` (stdout redirected to a file, `$?` read directly) and, where the
row says, `just wasm-test`, the corpus test or the script; record the flip, revert, confirm green.
A row whose red set doesn't match is reported, not edited to match. **"Reasoned"** marks a cell
worked out from the maths or the code, not driven at design (no op code exists yet); the build
drives every row and corrects the table in Build Completion if a reasoned cell is wrong.

| # | mutation, and where | red (expected) | green (expected) |
|---|---|---|---|
| M1 | **direction**: in `Rotate::apply`, the 1-turn arm calls `rotate270()` instead of `rotate90()` (`src/operation/rotate.rs`) | `rotate_90_is_clockwise_by_position` (input (0,0) lands at (0,6), not (4,0)); `normalises_every_accepted_angle` rows `90`, `-270`, `90.0`; `round_trips_are_exact_in_every_colour_type`'s `-90 then 90` trip (two 270s = 180, all 10 types); `rotate_acts_on_stored_pixels_and_writes_no_orientation_tag` (the 3-turn map, not the 1-turn); `auto_orient_then_rotate_turns_twice` and `rotate_then_auto_orient_matches_auto_orient_then_rotate` (tag 6 and a 270 cancel: 7×5 but the stored pixels, not the 2-turn map; the two outputs still equal each other); corpus `v2_rotate_90` on all three fixtures **by pixel hash** (the dims are the same for 90 and 270: this is the golden's red-by-hash row, trap 13); wasm `transform_rotates_and_flips_exactly` row `90` | `four 90s` round trip (four 270s = identity); `180 = both flips`; `quarter_turns_swap_the_dimensions`; every refusal row; every flip test; rows `270`, `-90`, `-90.0`, `±180`, no-op rows; wasm row `-90`. This row is why AC-3 exists: the four-90s round trip alone passes an anticlockwise op. *Reasoned.* |
| M2 | **flip axis**: in `Flip::apply`, the `horizontal` arm calls `flipv()` (`src/operation/flip.rs`) | `horizontal_mirrors_left_and_right_by_position` (input (0,0) at (0,4), not (6,0)); `rotate_180_is_both_flips…` (flipv twice = identity ≠ 180, all 10 types); `apply_flips_a_png_end_to_end`; corpus `v2_flip_horizontal` on all three fixtures by pixel hash; wasm `horizontal` row | `flip horizontal twice` round trip (flipv twice = identity); vertical rows; every rotate test; every refusal row. *Reasoned.* |
| M3 | **normalisation**: in `quarter_turns`, return `Some(1)` when `angle == -90.0` (`src/operation/rotate.rs`) | `normalises_every_accepted_angle` rows `-90` and `-90.0`; `round_trips…` `-90 then 90` trip (two 90s = 180); wasm row `-90` | every other normalisation row, every refusal, corpus (uses `90`), every flip test. *Reasoned.* |
| M4 | **no-op**: in `quarter_turns`, return `Some(1)` when `angle == 0.0` | `normalises_every_accepted_angle` rows `0` and `-0.0` (`-0.0 == 0.0` in `f64`, driven); `no_op_angles_return_the_input` cases `0`, `-0.0`; `apply_rotate_0_writes_the_input_pixels` (5×7, not 7×5) | rows `360`, `-360` (they reach turns 0 through `rem_euclid`, not this branch); every other test. The corpus has no no-op row (see "Normalisation"). *Reasoned.* |
| M5 | **range**: in `ROTATE_PARAMS`, `max: Some(360.0)` → `max: None` | R6 (`450` now passes the table and is a multiple of 90: **builds `Ok`**, reported as such); R4, R5, R7, R9, R10, R11 (the expectation now reads `a finite number >= -360`, so the whole-message `==` fails); R8 (`360.5` passes the table, refused by the constructor's multiple-of-90 message instead); AC-1's schema assert in `every_builtin_publishes_a_param_schema`; wasm R6 | R1–R3, R12–R16, every flip row, every pixel test. The message carries the range, so any change to the bound reddens every table row: that is the guard working, not noise. *Reasoned from `registry.rs:124-131`.* |
| M6 | **multiple of 90**: in `quarter_turns`, delete the `angle % 90.0 != 0.0` line | R12–R16 each **build `Ok`**; `rotate_refuses_non_multiples_of_90`; `apply_refuses_a_45_degree_rotate…` (exit 0, a file written); wasm R12. **What `apply` then does with 45:** `(45.0 / 90.0) as i64` is `0`, so turns 0: the image is written **unrotated**, exit 0, no message. `90.5` turns once, `1.1` and `1e-300` not at all. | every accepted-angle test, every flip test, R1–R11. *Reasoned.* |
| M6b | **`f32` read**: in `Rotate::from_params`, `get_f32("angle").map(f64::from)` instead of `get_f64` | R16 (`1e-300` reads as `0.0`, loads as a no-op, **builds `Ok`**) | R15 (`1.1` as `f32` is `1.100000023841858`, still not a multiple, and the message prints the TOML value as written, so it stays green); every other row. *Reasoned.* |
| M7 | **metadata carried through**: in `encode_to_bytes_with` (`src/sink/mod.rs:725`), for `ImageFormat::Png` build a `PngEncoder`, call `set_exif_metadata(exif)` with the bundle's EXIF when `img.metadata()` has one, and `write_with_encoder` | `rotate_acts_on_stored_pixels_and_writes_no_orientation_tag` (kamadak-exif reads Orientation **6** on the 5×7 output; the failure names the obligation) | AC-38 and AC-39 rows (their outputs pass through `auto-orient`, which drops the bundle, so no tag; *reasoned*); the corpus PNG byte row (its fixtures carry no bundle); every other test. Revert before the next row. |
| M7b | **bundle dropped**: in `Rotate::apply`, return `Image::from_parts(turned, img.source_format(), None)` instead of `img.with_pixels(turned)` | `rotate_then_auto_orient_matches_auto_orient_then_rotate` (the later `auto-orient` finds no tag: 5×7, the 1-turn map) | AC-37 (still no tag on the output); every other test. *Reasoned.* |
| M8 | **wasm count**: `FULL_COUNT=50` in `scripts/wasm-ci-check.sh` | `scripts/wasm-ci-check.sh count full <a 51-passed log>` → `full set: 51 passed, expected 50`; `--self-test` rows 1 (verdict fail, expected ok), 2 (wrong message: lacks `expected 51`) and 3 (wrong verdict: passes under `FULL_COUNT=50`, should fail) | `workflow`; slim rows 4, 5. The self-test goes red because its fixtures name 51: it stays green only when the fixtures move with the constant, which is AC-47. |
| M9 | **corpus guard** (scratch copy, never committed, after the ops exist): append `[[step]]` `op = "rotate"` `angle = 90` to `tests/fixtures/recipe_corpus/v1_resize_max.toml` | `recipe_corpus_output` naming `v1_resize_max × <fixture>` on all three (dims `532x800` against `800x532` on the two landscape fixtures, `800x532` against `532x800` on the portrait one); `recipe_round_trip` on `v1_resize_max`'s golden | every rotate/flip test |
| M10 | **trap 14, the old code verbatim**: restore `with_builtins` from `main` (no `rotate`/`flip` lines) | every row of `tests/rotate_flip.rs` (`unknown operation 'rotate'` / `'flip'`; the orientation test's control A still passes, control B passes, the rotate step fails at load); corpus `v2_rotate_90` and `v2_flip_horizontal`; both wasm tests; `with_builtins_contains_rotate_and_flip`; both registry extensions; `every_op_named_in_data_model_resolves` (`tests/docs_ops.rs`); `to_toml_is_byte_identical_to_main_over_the_corpus` | the `rotate.rs` and `flip.rs` unit tests (they call the modules directly); every existing test |

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle. It is the
equivalent of a handoff document, folded into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-113` — **this spec's.** A geometry step acts on the buffer it receives and writes no
  metadata; the bundle stays in memory for a later `auto-orient`; whoever makes the pixel lane write
  EXIF must set Orientation to 1 and fix or drop the pixel dimensions.
- `DEC-003` — the pixel lane carries no container metadata; why no output can carry a stale tag.
- `DEC-017` — ops may read metadata; `AutoOrient` bakes the tag and drops the bundle.
- `DEC-095` — preserve colour type and depth. The `DynamicImage` turn and flip methods keep the
  variant; no widen, no narrow.
- `DEC-102` — the key table runs before the constructor; the constructor holds the multiple-of-90
  rule. Every number is also range-checked as an `f32`, which is why R16's constructor must read
  `f64`.
- `DEC-110` — limb 2: no resampling, so native and wasm agree exactly, dimensions and pixels.
- `DEC-112` — crop's units; not used here (angles are not lengths), cited because DEC-113 names
  crop as following the same metadata rule.

### Constraints that apply

- `untrusted-input-hardening` — nothing panics on any TOML value; `as i64` runs only on a value the
  table bounded to [-360, 360] and the constructor proved a multiple of 90.
- `no-unwrap-on-recoverable-paths` — no `unwrap`/`expect` in `rotate.rs`/`flip.rs` outside tests.
- `single-image-library` — `image` only; kamadak-exif is already a dependency and appears only in
  the test.
- `every-public-fn-tested` — both `from_params`, both `Operation` impls.
- `test-before-implementation` — the tests above, red first, baseline quoted.
- `one-spec-per-pr` — one PR, `feat/spec-147-rotate-flip`.

### Prior related work

- `SPEC-145` (shipped) — `crop`: the registered-op shape, `tests/crop.rs`, the corpus and wasm rows
  this spec copies.
- `SPEC-131` (shipped) — the key tables and typed constructors.
- `SPEC-141` (shipped) — the corpus guard; its README is AC-42's procedure.
- `SPEC-110` (shipped) — the verbs bake orientation first; `apply` was scoped out, which DEC-113
  keeps.
- `SPEC-093` (shipped) — the precedent for reading EXIF in tests with kamadak-exif.

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
- A no-op corpus recipe: the corpus guard refuses one by design (see "Normalisation").

## Notes for the Implementer

- **Checkpoint.** Commit and push after each of: (1) the red tests, with the baseline quoted; (2) the
  ops green locally; (3) the corpus files for your own target; (4) the wasm tests and the script;
  (5) the docs. A long build with no pushed commit has been lost before.
- **Corpus bless order.** Add the two `.toml` files (a two-line `#` comment each, as
  `v2_crop_rect.toml` has, then `version = "2"`, `name = "corpus-rotate-90"` /
  `"corpus-flip-horizontal"`, one step), write their `.golden` by the README (canonical TOML: keys
  sorted, so `op` then `angle` / `axis`), raise every literal in one commit, bless your own target,
  push, and copy the other two targets' files from the failing CI log's `----- BEGIN … -----`
  blocks. Then `cmp` all three per recipe (AC-43).
- **wasm.** Both new wasm tests run in the full and slim sets, so both constants move by 2. Move the
  self-test fixtures in the same commit as the constants (SPEC-145 went red in CI on exactly this).
  Run `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh --self-test` and `… workflow` locally; `just
  wasm-test` locally if you can, and read CI's two wasm count steps.
- **`cargo test` in a terminal** fails `display_sink_refuses_non_tty`: redirect stdout to a file
  and read `$?`, never through a pipe. Use your own `CARGO_TARGET_DIR`. Run the lean build
  (`cargo test --no-default-features`) too: neither op has a feature gate.
- **The orientation test's input** must load its tag: check control B (5×7 after `auto-orient`)
  before trusting the rotate row. If `image`'s PNG decoder doesn't hand the `eXIf` chunk to the
  bundle, stop and report; don't switch to a JPEG (lossy pixels can't be compared exactly).
- **User-facing text** (messages, doc comments, docs) reads behaviour-first and names no spec,
  stage or DEC id (ruling 3).
- **Files the PROJ-011 orchestrator must keep clear of** while this builds: `src/operation/`,
  `src/sink/mod.rs`, `src/recipe/`, `docs/recipes.md`, `recipes/README.md`, `docs/data-model.md`,
  `docs/api-contract.md`, `docs/backlog.md`, `skills/crustyimg/SKILL.md`, `README.md`,
  `integrations/ddphotos/README.md`, `CHANGELOG.md`'s Unreleased section,
  `tests/fixtures/recipe_corpus/`, `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs`,
  `tests/wasm_roundtrip.rs`, `tests/common/mod.rs`, `scripts/wasm-ci-check.sh`.

---

## Build Completion

- **Branch:** `feat/spec-147-rotate-flip`
- **PR (if applicable):** #252 (draft)
- **All acceptance criteria met?** yes (58 of 58). CI green on every leg at `f2f5b59`.
- **New decisions emitted:**
  - none in this cycle. DEC-113 was written at design and is on `main`.
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`, 32 files; the list below names 31, and the 32nd is this spec file, which it leaves out):
  - `src/operation/rotate.rs` — new: `Rotate`, `ROTATE_PARAMS`, private `quarter_turns`, 6 unit tests
  - `src/operation/flip.rs` — new: `Flip`, `FLIP_PARAMS`, `AXIS_NAMES`, 3 unit tests
  - `src/operation/mod.rs` — two `pub mod` and two re-exports
  - `src/operation/registry.rs` — two registrations, doc lists, `written` moved in, 3 test changes
  - `src/operation/crop.rs` — imports `written`; its private copy deleted
  - `src/sink/mod.rs` — module-doc paragraph under `## Encoding`, comment only (AC-40)
  - `tests/rotate_flip.rs` — new: 14 integration tests
  - `tests/common/mod.rs` — new `tiff_with_orientation`; `wrap_with_orientation_app1` reuses it
  - `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs` — two stems, `EXPECTED_DIMS` rows, literal bumps
  - `tests/fixtures/recipe_corpus/v2_rotate_90.{toml,golden,linux-x86_64.output,macos-aarch64.output,windows-x86_64.output}` — new
  - `tests/fixtures/recipe_corpus/v2_flip_horizontal.{toml,golden,linux-x86_64.output,macos-aarch64.output,windows-x86_64.output}` — new
  - `tests/wasm_roundtrip.rs` — two wasm tests
  - `scripts/wasm-ci-check.sh` — `FULL_COUNT` 51, `SLIM_COUNT` 49, self-test fixtures and rows 1–5, 7, 9
  - `docs/recipes.md`, `recipes/README.md`, `docs/data-model.md`, `docs/api-contract.md`, `skills/crustyimg/SKILL.md`, `integrations/ddphotos/README.md`, `README.md`, `docs/backlog.md`, `CHANGELOG.md` — the AC-48..55 surfaces
  - Nothing else. `Cargo.toml`, `Cargo.lock` and `src/build/cache.rs` are untouched (`git diff --stat` for those paths is empty).
- **Deviations from spec:**
  - `tests/common/mod.rs` is not in the spec's `## Outputs` list; the Failing Tests section asks for the `tiff_with_orientation` helper there, so it is in.
  - The red-test commit holds the integration, registry and wasm-free tests only. The unit tests in `rotate.rs` and `flip.rs` cannot compile before the modules exist, so they landed with the ops.
  - One extra integration test, `every_step_matches_its_index_map_in_every_colour_type`, checks the hand index maps for all 10 colour types (70 cases). The spec's list checks positions only on rgb8, in the unit tests.
  - The mutations ran in a scratch `git worktree` of the pushed HEAD with its own target dir, not in the build worktree, so a mutated file could never be committed. M9 ran there too.
  - The first corpus push was red on purpose (linux and windows `.output` missing); the second is green.
- **Follow-up work identified:**
  - none.

### Acceptance criteria
- AC-1 met: `with_builtins_contains_rotate_and_flip` and the extended `every_builtin_publishes_a_param_schema` assert both tables exactly; the six old tables are untouched.
- AC-2 met: `normalises_every_accepted_angle` runs the 12 rows (loads, `quarter_turns`, stored turns, 7×5 index map) and asserts 12 ran.
- AC-3 met: `rotate_90_is_clockwise_by_position` (90 → (4,0), 270 → (0,6), 180 → (6,4), plus the full maps).
- AC-4 met: `horizontal_mirrors_left_and_right_by_position` ((6,0) and (0,4), plus the full maps).
- AC-5 met: `round_trips_are_exact_in_every_colour_type`, 40 cases, bytes and variant.
- AC-6 met: `rotate_180_is_both_flips_in_every_colour_type`, 10 cases, with a not-the-input control.
- AC-7 met: `quarter_turns_swap_the_dimensions`, 8 rows.
- AC-8 met: `single_steps_keep_colour_type_and_depth`, 40 cases.
- AC-9 met: `no_op_angles_return_the_input` (0, 360, -360, -0.0 on rgb8 and rgba16) and `apply_rotate_0_writes_the_input_pixels` (binary, exit 0).
- AC-10 met: `params_round_trip_as_written` in both op files.
- AC-11 met: `refusal_hook_is_not_used_and_apply_never_fails` in both op files (12 and 6 cases).
- AC-12..27 met: `every_rotate_refusal_names_its_reason`, R1–R16, whole-message `==`, asserts 16 ran.
- AC-28..34 met: `every_flip_refusal_names_its_reason`, F1–F7, asserts 7 ran.
- AC-35 met: `apply_refuses_a_45_degree_rotate_and_writes_nothing` (exit 1, exact line, no `out.png`).
- AC-36 met: `apply_refuses_a_diagonal_flip_and_writes_nothing`.
- AC-37 met: `rotate_acts_on_stored_pixels_and_writes_no_orientation_tag`; control A reads Orientation 6, control B gives 5×7 (the PNG `eXIf` chunk does reach the bundle, so no stop was needed).
- AC-38 met: `auto_orient_then_rotate_turns_twice`.
- AC-39 met: `rotate_then_auto_orient_matches_auto_orient_then_rotate`.
- AC-40 met: paragraph added under `## Encoding` in `src/sink/mod.rs`; no code change there.
- AC-41 met: `apply_flips_a_png_end_to_end`.
- AC-42 met: both recipes, goldens and three `.output` files each; `RECIPES` 18, `EXPECTED_DIMS` 18, `ROWS` 60, `GOLDEN_LINES` 63, `* FIXTURES.len()` 54, doc comments, `CORPUS` +2, `pairs.len()` 18 with "15 corpus recipes + 3 bundled recipes".
- AC-43 met: `cmp` of linux/macos and macos/windows for each recipe exits 0 (see the corpus section).
- AC-44 met: `--diff-filter=M` 0 files, `--diff-filter=D` 0, `--diff-filter=A` exactly 10.
- AC-45 met: wasm `transform_rotates_and_flips_exactly` (80×120, 80×120, 120×80, 120×80; 0 differing pixels each).
- AC-46 met: wasm `transform_refuses_a_bad_rotate_or_flip_with_the_native_message` (R12, R6, F3; identity after each).
- AC-47 met: constants 51/49, fixtures `l51`/`l1`/`l50`/`l49`, rows 1–5, 7, 9 as specified; self-test and workflow exit 0; CI count steps pass.
- AC-48 met: `docs/recipes.md` has two `[today]` entries with the exact sentences; `Trim / pad` stays planned.
- AC-49 met: the PROJ-006 line is now the PROJ-017 line plus the "Smart crop, redaction…" line.
- AC-50 met: two op-table rows and `### The rotate and flip steps` before `### Step keys are checked` in `recipes/README.md`.
- AC-51 met: "eight operations", the new section with the table, both sentences and the TOML example; `tests/docs_ops.rs` passes.
- AC-52 met: SKILL.md ops list and the frontmatter `description` ("crop, rotate, flip").
- AC-53 met: "Rotate and flip in a recipe." paragraph after the crop paragraph in `docs/api-contract.md`.
- AC-54 met: CHANGELOG `### Added` entry after the crop entry.
- AC-55 met: ddphotos README, README.md line, both backlog rows; `npm/README.md` untouched.
- AC-56 met: `grep -nE "SPEC-|STAGE-|DEC-" src/operation/rotate.rs src/operation/flip.rs` finds nothing (not even in tests). The backlog row names STAGE-059 as the spec's wording asks; backlog is an internal document.
- AC-57 met: `git diff --stat origin/main...HEAD -- Cargo.toml Cargo.lock src/build/cache.rs` is empty; no version bump.
- AC-58 met: local matrix and CI green on every leg; `cargo test --test crop` passes unchanged (6 passed) after `written` moved.

### Red baseline (stage 1)
Against `main`'s code, `cargo test --test rotate_flip`: 0 passed, 14 failed. Quoted per file: `tests/rotate_flip.rs` → `assertion left == right failed: stderr: error: unknown operation 'rotate'` (binary rows), and the refusal tables report `R9: expected "recipe step 0 ('rotate'): …", got "unknown operation 'rotate'"` for every row. Controls A and B passed before the rotate step failed, so the red is the right reason. The corpus and wasm rows have no separate red commit: the recipes did not exist, and the wasm tests ran only after the ops did.

### Mutations (scratch worktree, clean `cargo clean -p crustyimg` rebuild before each, restored with `git checkout`, `git status` clean after each)
Baseline of the same test set on the unmutated tree: exactly one red, `every_golden_line_is_checked_and_no_golden_is_stray` (the linux/windows files were not yet copied in); it is red in every row below and not repeated. All pixel rows were **reasoned** in the table; every one is now **driven**.

| # | applied | red (driven) | vs. table |
|---|---|---|---|
| M1 | `rotate.rs` 1-turn arm → `rotate270()` | `rotate_90_is_clockwise_by_position`; `normalises_every_accepted_angle`; `round_trips_are_exact_in_every_colour_type`; `rotate_acts_on_stored_pixels_and_writes_no_orientation_tag`; `auto_orient_then_rotate_turns_twice`; `rotate_then_auto_orient_matches_auto_orient_then_rotate`; `every_step_matches_its_index_map_in_every_colour_type`; corpus `every_recipe_output_matches_its_golden`; wasm `transform_rotates_and_flips_exactly` ("rotate 90: pixels differing") | matches; four-90s trip and `180 = both flips`, dimension and refusal rows stay green. Confirmed. The extra red row is this build's own added test |
| M2 | `flip.rs` horizontal arm → `flipv()` | `horizontal_mirrors_left_and_right_by_position`; `rotate_180_is_both_flips_in_every_colour_type`; `apply_flips_a_png_end_to_end`; `every_step_matches_…`; corpus golden; wasm ("flip horizontal: pixels differing") | matches. Confirmed |
| M3 | `quarter_turns` → `Some(1)` for `-90.0` | `normalises_every_accepted_angle` (rows `-90`, `-90.0`); `round_trips_…` (the `-90 then 90` trip); `every_step_matches_…`; wasm ("rotate -90: pixels differing") | matches. Confirmed |
| M4 | `quarter_turns` → `Some(1)` for `0.0` | `normalises_every_accepted_angle` (`0`, `-0.0`); `no_op_angles_return_the_input`; `apply_rotate_0_writes_the_input_pixels`; `every_step_matches_…` (its `0` case) | matches; `360`/`-360` rows stay green. Confirmed |
| M5 | `ROTATE_PARAMS` `max` → `None` | `every_rotate_refusal_names_its_reason` (R6 "but it built"; R4, R5, R7, R9, R10, R11 read `a finite number >= -360`; R8 refused by the multiple-of-90 line instead); `every_builtin_publishes_a_param_schema`; wasm refusal test (R6 builds) | matches. Confirmed |
| M6 | delete the `% 90.0` check | `every_rotate_refusal_names_its_reason` (R12–R16 all "but it built"); `rotate_refuses_non_multiples_of_90`; `apply_refuses_a_45_degree_rotate…` (exit 0, not 1); wasm refusal test (R12 builds) | matches. Confirmed |
| M6b | `get_f64("angle")` → `get_f32(..).map(f64::from)` | `every_rotate_refusal_names_its_reason` with exactly one wrong row: **R16** ("but it built"); R15 stays green | matches. Confirmed |
| M7 | `encode_to_bytes_with` writes the bundle's EXIF for PNG | `rotate_acts_on_stored_pixels_and_writes_no_orientation_tag`, message "the pixel lane carried the source's Orientation tag through (6); it must write 1 or nothing" | matches; AC-38/39 stay green. The *reasoned* cell for them is confirmed |
| M7b | `Rotate::apply` → `Image::from_parts(.., None)` | `rotate_then_auto_orient_matches_auto_orient_then_rotate` only | matches; AC-37 stays green. Confirmed |
| M8 | `FULL_COUNT=50` | `count full` on a 51 log → `full set: 51 passed, expected 50`; self-test rows 1, 2, 3 red; `workflow` green; slim rows 4, 5 green | **one cell corrected**: row 3 (`count full 50`) goes red as a *wrong verdict* (it passes under `FULL_COUNT=50` but should fail), not as "message lacks `expected 51`". Rows 1 (wrong verdict) and 2 (wrong message) are as written |
| M9 | append a `rotate` step to `v1_resize_max.toml` (scratch) | `every_recipe_output_matches_its_golden`, naming `v1_resize_max × rgb8_2600x1730 / rgba8_1730x2600 / rgba16_2600x1730` ("dims 532x800, expected 800x532" on the two landscape fixtures, "800x532, expected 532x800" on the portrait one); `to_toml_is_byte_identical_to_main_over_the_corpus` | matches. Confirmed. Branch `git status` clean |
| M10 | `with_builtins` without `rotate`/`flip` | all 14 `tests/rotate_flip.rs` rows; corpus golden test; `with_builtins_contains_rotate_and_flip`; `every_builtin_publishes_a_param_schema`; `one_of_values_are_accepted_by_their_constructors`; `every_op_named_in_data_model_resolves`; both wasm tests; `rotate.rs`/`flip.rs` unit tests stay green | matches, plus `to_toml_is_byte_identical_to_main_over_the_corpus` (the new corpus recipes no longer load) which the table did not list. Control A and B pass; the rotate step is what fails. Confirmed |

### Corpus
- First CI run (`b6069b8`, macOS target only): red on 8 legs, in exactly two tests, `every_golden_line_is_checked_and_no_golden_is_stray` and, on linux/windows, `every_recipe_output_matches_its_golden`, for the missing files. Nothing else failed. The macOS legs failed only on the stray-file test.
- The linux `.output` text is identical across the five linux legs (one distinct content per recipe), and windows came from its own leg.
- Second run (`f2f5b59`): green on every leg: ubuntu, macOS, windows, avif, webp-lossy, lean, both heic legs, msrv, wasm round-trip, ddphotos parity, browser smoke, cargo-deny, front-matter, DCO.
- `cmp v2_rotate_90.linux-x86_64.output v2_rotate_90.macos-aarch64.output` and `cmp …macos-aarch64… …windows-x86_64…` exit 0; the same for `v2_flip_horizontal`. All six files per pair are byte-identical.
- No existing `.output` or `.golden` changed: `git diff --diff-filter=M --name-only origin/main -- tests/fixtures/recipe_corpus/` is empty; `--diff-filter=A` lists the ten new files.

### wasm and the script
- `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh --self-test` → `wasm-ci-check self-test: 18 rows, 18 passed, 0 wrong verdict, 0 wrong message`.
- `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh workflow` → `wasm-ci-check workflow: 16 assertions, 0 failed`.
- `./scripts/validate-frontmatter.sh` → `✓ validate-frontmatter: all front-matter blocks parse.` (345 blocks).
- Local: `just wasm-check` finished; `just wasm-test` → `test result: ok. 51 passed`; `just wasm-test-slim` → `test result: ok. 49 passed`.
- CI: `wasm-ci-check: full set: 51 passed, as expected` and `wasm-ci-check: slim set: 49 passed, as expected`.
- Bundle size (the `pages` workflow's "build + browser smoke", features avif,svg), main at `7fdee9f` against this branch: raw 6,083,069 → 6,098,538 B (+15,469 B, +0.25%); brotli 1,302,231 → 1,304,654 B (+2,423 B, +0.19%). The job and its size gate pass.

### Matrix (each in a fresh `CARGO_TARGET_DIR`, `cargo test` redirected to a file, exit read directly)
- default: 1180 passed, 0 failed; clippy `-D warnings` exit 0; `fmt --check` exit 0.
- `--no-default-features`: 1117 passed, 0 failed; clippy and fmt exit 0. The rotate/flip tests ran here (not skipped).
- `--features webp-lossy`: 1184 passed, 0 failed; clippy and fmt exit 0.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Little. Two small things: the M8 table cell says row 3 fails on its message, but it fails on its verdict; and the red-test checkpoint says "write every failing test", yet the unit tests in the two op files cannot compile before the modules exist.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The orientation rule (DEC-113) and the exact messages were enough to build from. `tests/common/mod.rs` is named in the Failing Tests but missing from `## Outputs`.

3. **If you did this task again, what would you do differently?**
   — Gate the corpus commit on the test run instead of chaining `git commit` after `cargo test` with `;`: the first corpus commit went out while `every_golden_line_is_checked…` was still red for the missing targets (expected, but ungated). I would also start the mutation sweep in a scratch worktree from the start, which is what kept the wasm and mutation runs from touching the branch. No design call turned out wrong.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Three things, all about claims made before they were driven. (a) Design reasoned the 12-row
   mutation table, and build corrected two cells (M8 row 3 fails on its verdict, not its message;
   M10 also reds `to_toml_is_byte_identical_to_main_over_the_corpus`). Both are cheap, one-command
   rows, so design should drive them and keep only the pixel rows reasoned. (b) The corpus README's
   literal table fell behind across SPEC-145 and SPEC-147 and no test reads it; any doc that
   restates a test literal should be read by a test or say "see the test". (c) "`apply` never fails"
   was stated as a universal and driven only over the AC-11 shapes; design should have written it
   as a claim over named shapes (1×1, 1×7, 7×1, every turn and both axes), which is all that was
   proven. DEC-113's tag 3 and 8 agreement had the same gap and is now worded as such.

2. **Does any template, constraint, or decision need updating?**
   — Proposed, not applied. (i) Rule 7 (optional cleanups need no re-verify) and trap 17 (a derived
   bound is a claim until driven) both held here: verify approved with no punch items, and the four
   optional cleanups went to ship and the next spec instead of costing rounds. They live only in
   `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md` (rule 7, trap 17); no file in
   `guidance/` covers either, so promote both to `guidance/` or to AGENTS §15 "During verify", and
   point HANDOFF at them. (ii) A one-line rule for design: a mutation table's cheap rows are driven
   at design, not reasoned. No `constraints.yaml` entry covers it. (iii) DEC-113 got the one
   allowed wording sentence (only tag 6 driven); no other DEC change.

3. **Is there a follow-up spec I should write now before I forget?**
   — Filed on STAGE-058, one entry: a stricter `orientation_of` (`Err(NotFound)` only) and the
   corpus README's stale literals, both test-side, for the next STAGE-058 spec's build. The README
   gap is separate from the existing "report the file-set and line-count failures together" entry,
   which covers failure reporting only. No other gap is named by the evidence, so none further.
