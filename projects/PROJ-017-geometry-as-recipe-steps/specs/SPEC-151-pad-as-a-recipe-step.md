---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-151
  type: story                      # epic | story | task | bug | chore
  cycle: frame  # frame | design | build | verify | ship
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
  created_at: 2026-11-06

references:
  decisions:
    - DEC-095
    - DEC-102
    - DEC-110
    - DEC-112
    - DEC-113
    - DEC-109
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - single-image-library
    - every-public-fn-tested
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-131
    - SPEC-141
    - SPEC-142
    - SPEC-145
    - SPEC-147

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-058's <capability>". Optional; null is acceptable.
value_link: "The fourth of STAGE-058's lossless geometry ops and the first that adds pixels: pad to an aspect or by a border, so a recipe can make a square or framed image the same way at any size"

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

# SPEC-151: pad as a recipe step

## Context

A recipe can't add pixels today. The registry holds `identity`, `invert`, `resize`,
`auto-orient`, `watermark`, `crop`, `rotate` and `flip` (`with_builtins`,
`src/operation/registry.rs:305`), and none of them grows the canvas. No `resize` mode pads either:
`fit` stops inside the box, `fill` crops to it and `cover` overshoots it. So "make every photo in this
folder square for Instagram" or "put an even border round each print" needs another tool.

This is STAGE-058's `pad` item, PROJ-017 build-order line 5 of 13. `trim` (line 6) is its inverse and
the `edit` flags spec (line 8) needs both. It also takes the stage's fold-in entry ("SPEC-147's verify
cleanups that need a PR") and the refusal-hook doc entry (see the Frame's scope).

`pad` is the first geometry op that adds pixels rather than moving them, so it is the first to meet
three questions crop, rotate and flip did not: what colour type a fill forces, what `transparent`
means when the output format has no alpha, and whether a per-image condition is skippable.

## Goal

A recipe can say `op = "pad"` with an aspect (plus `gravity`) or a border, and an opaque `color`, and
get a larger canvas in which every input pixel is unchanged at its offset and every new pixel is the
fill. The same recipe gives the same relative result on a 1200 px copy and a 6000 px original.

## Frame

### Verdict: SPLIT. SPEC-151 is `pad` with opaque fills, [S]; `transparent` is its own entry

SPEC-151 is GO at [S] as **pad to an aspect or by a border, filled with an opaque colour**. The
`transparent` fill (and any `#RRGGBBAA` with alpha below `ff`) moves to a new STAGE-058 entry filed by
this frame, because it is the only part that reaches past the op: it widens RGB to RGBA, and the
recommended JPEG rule (refuse) has to live where the output format is resolved, in `apply` (one input
and a batch), `build` and wasm `transform`. Kept together, that is an [M].

**If the maintainer rules "flatten" for Q4 instead of "refuse"**, the transparent half needs no change
outside the op (today's encoder already does what the flatten rule would say, driven below). Then
re-merge it into SPEC-151 as one [S] and drop the new entry.

`cycle` stays at `frame` until the maintainer accepts the split (Q1).

### Premise check (on `main` at `31f061c`, crustyimg 0.8.2, release build in a scratch `CARGO_TARGET_DIR`, deleted after)

- **No `pad` op exists.** `/usr/bin/grep -rn -i '\bpad\b\|"pad"\|letterbox' src/` finds one comment
  in `src/recipe/mod.rs:525` (TOML padding in a test). A recipe with `op = "pad"` on a 4×2 PNG:
  `error: unknown operation 'pad'`, exit 1.
- **No resize mode pads.** `ResizeMode` is `Max | Exact | Percent | Fit | Fill | Cover`
  (`src/operation/mod.rs:748`). `docs/cli-reference.md:100-104`: `--fit` fits inside the box, `--fill`
  crops to it, `--cover` overshoots it with no crop. `pad` complements `fit` (below); it doesn't
  duplicate any mode.
- **The colour parser is hex only.** `parse_color` (`src/text/mod.rs:51`) reads `RRGGBB`, `#RRGGBB`
  (alpha 255) and `RRGGBBAA` / `#RRGGBBAA`, and returns `[u8; 4]`. It has no names (`"red"`,
  `"white"`, `"transparent"` are all errors). `watermark`'s key is `color`, and its load message is
  `'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "…"` (`src/operation/mod.rs:1665`).
- **Gravity and the odd pixel.** `Gravity::placement` (`src/operation/mod.rs:1246`) centres with
  `(bw - ow) / 2`, a floor. Crop uses it with margin 0: SPEC-145's `aspect_gravity_places_all_nine`
  pins a 2600×1730 image cropped to 5:4 (2163 wide, remainder 437) at `left = 218`, so the odd column
  is cut from the right.

### Driven: transparent pixels into JPEG today (the flatten nobody chose)

Input made with Python stdlib (not crustyimg, and not ImageMagick, which zeroes the RGB under alpha
0): a 48×16 RGBA8 PNG with three 16 px blocks: `(255,0,0,0)` transparent red, `(0,255,0,128)` half
green, `(0,0,255,255)` opaque blue. A 16-bit twin uses the same values ×257. Recipe `version = "2"`,
`format = "jpeg"`, `invert` twice. Read with ImageMagick at each block's centre:

| input | path | exit | (8,8) transparent red | (24,8) half green | (40,8) opaque blue |
|---|---|---|---|---|---|
| RGBA8 | `apply -o out.jpg` | 0 | `srgb(254,0,0)` | `srgb(0,255,1)` | `srgb(0,0,254)` |
| RGBA16 | `apply -o out.jpg` | 0 (stderr: the existing 16→8-bit warning) | `srgb(254,0,0)` | `srgb(0,255,1)` | `srgb(0,0,254)` |
| RGBA8 | `quality = 90` (the `JpegEncoder` path) | 0 | `srgb(254,0,0)` | | |
| RGBA8 ×2 | batch `--out-dir`, JPEG | 0 | `srgb(254,0,0)` | | |

**Today's rule is "drop alpha, keep the colour underneath", on every JPEG path, with no warning.** A
pixel the source marked fully transparent comes out as its hidden colour. So a pad that wrote
`transparent` as `#00000000` would give a **black** border in a JPEG at exit 0, and one written as
`#ffffff00` a white one. BMP keeps alpha (`magick identify`: `srgba`), so of the formats `recipes/
README.md:80` lists, JPEG is the one with no alpha channel at all. GIF has 1-bit transparency
only and wasn't driven; the new entry's frame drives it.

**When the pipeline can know.** The op never sees the output format (`Operation::apply(img)` has no
format). The format is resolved per input by `output_format_for` (`src/cli/ops.rs:212`) from
`--format`, the output name's extension, the recipe's `format` and the source format. Every rung is
known once the input is decoded, before the pipeline runs, but today the call sits after
`pipeline.run` (`ops.rs:384-399` and the batch arm at `:497`). So a refusal can be per input, after
decode and before any op runs, at no cost to a valid run. It can't be at load, because the source
format and `-o` aren't in the recipe.

### Driven: a tagged input (DEC-113)

A 60×40 PNG and JPEG (ImageMagick), tagged with `exiftool -Orientation#=6
-ExifIFD:ExifImageWidth=60 -ExifIFD:ExifImageHeight=40 -Artist=probe`, through `crop aspect = "1:1"`
(the nearest shipped op), read with `exiftool -s -Orientation# -ExifImageWidth -ExifImageHeight
-Artist -ImageSize`: both outputs are `40x40` with **no** Orientation, ExifImageWidth/Height or Artist.
**DEC-113 covers `pad` as written**: its rule 1 names `pad` among the geometry steps, and its
Consequences say `pad`/`trim` "follow the same rule without a new decision". `pad` keeps the bundle
with `Image::with_pixels`, writes no tag, and its docs entry says to put `auto-orient` first. It
misses nothing: a pad changes the dimensions, and DEC-113 rule 4 already obliges a future EXIF-writing
lane to rewrite `PixelXDimension`/`PixelYDimension`.

### Driven: pad to an aspect after `resize fit` gives an exact pixel box

The Instagram case, "1080×1080", is the one where the pixel count is the point. It doesn't need pixels
in `pad`: `resize mode = "fit"` to the box, then `pad aspect` of the box's ratio. Driven on 18 input
sizes (6000×4000 down to 5000×1 and 7×5000, ImageMagick-made) × 2 boxes (1080×1080 and 1200×630, so
40:21): crustyimg's `--fit` output read with `magick identify`, then the pad canvas computed in Python
with crop's half-up formula mirrored (**the pad step itself is modelled, since it doesn't exist**).
36/36 land on exactly 1080×1080 or 1200×630, including `1201x631 → fit 1199x630 → pad 1200x630`, where
`fit` itself misses the box by a pixel. Caveat: `fit` never upscales, so a 700 px image gives a
700×700 square. That's `resize`'s rule, and the docs entry says so.

**The canvas rule, swept** (Python, exact rationals): for 13 aspects including `1:1`, `40:21`,
`239:100`, `65535:1` and `1:65535`, and every image from 1×1 to 160×160 (332,800 cases), the canvas
is never smaller than the image, its short side is within 0.5 px of exact (0.5 reached at ties,
rounded up, as crop's is), and 1,373 cases are already at the aspect, so the pad is a no-op.

### Recommended keys and size modes (content for design; no DEC)

One op, two forms told apart by their keys, as `crop` does (no `mode` key):

- **Aspect:** `aspect = "W:H"` (required for the form), `gravity` (optional, default `center`). Reuse
  crop's `parse_aspect` and its messages verbatim (whole numbers 1 to 65535, decimals refused with
  their whole-number spelling). The canvas is the smallest `W:H` rectangle that holds the image; its
  long side is the image's, and its short side is `round(long · b / a)` half up, in `u64`, mirroring
  crop's `aspect_size`. An image already at the aspect is a **no-op**, written at exit 0.
- **Border:** `border = f`, a number with `0 < f <= 1` (a `Number` row like crop's `width`, so `0`,
  `1e-50` as an f32, nan and `800` are refused by the table). It adds `round(f · long edge)` px on
  **every** side, so the border is the same width all round. `gravity` is refused with a border.
  Recommended IN: it is the most common pad after aspect, it costs one key and one form, and it is
  exactly the "fraction of the image the step receives" DEC-112 rule 6 expects.
- **Fill:** `color`, required, read by `parse_color` and refused at load with watermark's message.
  In SPEC-151 the alpha must be `ff` (`RRGGBB`, `#RRGGBB`, `RRGGBBff`); anything else is refused at
  load: `'color' must be opaque, such as "#ffffff"; got "#ff000080"`. The new entry lifts that.
- **Rejected: an exact pixel size** (`width = 1080, height = 1080`). DEC-112 rule 6 says a pad length
  is a fraction of the input and an exact pixel size is "refused in a recipe, or a DEC amends this
  one". The driven `fit` + `aspect` pair covers the exact-box case with no amendment, so **pad extends
  DEC-112 and needs no DEC-114**. A recipe with `width` is refused as an unknown key.
- **Rejected: an exact size in fractions** (`width = 1.2` meaning 120 % of the input). Same key names
  as crop with a different range (crop's are ≤ 1), and per-axis growth gives unequal borders, which is
  rarely what's wanted. `border` covers the useful case.
- **Rejected: per-axis border** (`f · W` on the left and right, `f · H` on the top and bottom).
  Uneven on any non-square image. Q3 asks for the ruling between the long edge and the short edge.
- **Rejected: a default fill** (black, white or transparent). Each default is wrong for half the uses,
  and DEC-102's precedent is to refuse at load rather than guess.

### Where each per-image condition fails, and what a batch gets

The rule from SPEC-145's M4b: a refusal through `Operation::refusal` is **skipped** per image in a
batch (the image is written unchanged at exit 0), so only a condition that is safe to skip may use
it. A condition that means the output is wrong fails at apply. Driven for the apply path: crop's
0 px case on a two-input batch printed both `operation 'crop' failed` lines and `2 of 2 inputs
failed`, **exit 6**, and wrote nothing.

| condition | outcome | where | batch gets |
|---|---|---|---|
| image already at the aspect | no-op, written unchanged | — | exit 0, image written |
| target smaller than the image | can't happen: no pixel-size form, and the aspect canvas is never smaller (swept) | — | — |
| border rounds to 0 px (`border = 0.001` on a 40 px image) | refuse: the author asked for a border and gets none (DEC-112 rule 4/6, DEC-109) | **apply** | that input fails, exit 6 |
| canvas past resize's caps (50,000 px edge or 134,217,728 px area, `src/operation/mod.rs:1028`; e.g. `aspect = "65535:1"` on 6000×4000) | refuse; never clamp, never allocate | **apply**, before any allocation | that input fails, exit 6 |
| `transparent` into JPEG | the new entry (Q4) | after decode, before the pipeline | per input |

**No `pad` condition uses the refusal hook.** So neither hook entry is a prerequisite. The doc-comment
entry ("State when an op uses the refusal hook and when it fails at apply") is folded in: it is
doc-only, its reader is exactly the author of an op like this one, and it says "before `pad` or `trim`
ships". The "watermark skipped" entry is **not** folded: it matters only once a second op uses the
hook, and `pad` doesn't. `trim`'s frame meets it again.

### Aspect against orientation

Consistent with crop: `aspect = "4:5"` always means 4 wide by 5 tall. The `orientation = "match"` key
belongs to STAGE-058's entry "`crop`'s aspect can follow each image's orientation", which already says
pad has the same question. Recommendation: that entry's spec adds the key to **both** ops with one
rule. Noted on that entry; not built here.

### Gravity and the odd pixel

Reuse `Gravity::placement(canvas_w, canvas_h, w, h, 0)` for the image's offset in the canvas. With
`center` and an odd remainder, **the extra fill pixel goes on the right (or the bottom)**, because the
offset is `floor(remainder / 2)`. That matches crop, where the extra cut pixel comes off the right.
One test pins it: a 4×3 image padded to `1:1` at `center` sits at `(0, 0)` with one fill row below it.

### Colour type and bit depth (DEC-095): no DEC needed

DEC-095's shared narrowing rule (`narrow_rgba8` / `narrow_rgba16`, `src/operation/mod.rs:383`)
already answers this. It never returns a type narrower than the input, keeps a luma type only if
every pixel still has `r == g == b`, and states the watermark precedent: "a colour watermark over a
gray base genuinely adds chroma and stays RGB". Recommended, one rule per case:

- **Grey image, grey fill** (`#808080` on `L8`/`La8`/`L16`/`La16`): keeps the luma type.
- **Grey image, chromatic fill:** widens to RGB / RGBA at the same depth. Not a DEC-095 break: it's the
  watermark-over-grey case DEC-095 already rules. (Refusing it was the alternative; it would make
  "pad my scans with cream" impossible for no gain.)
- **RGB or RGBA image, any opaque fill:** keeps its type. An alpha channel the input had stays, and the
  border's alpha is the maximum.
- **Bit depth is always kept.** On a 16-bit image each 8-bit fill channel is widened ×257
  (`(c << 8) | c`), so `#ff` → 65535 and `#00` → 0 (checked for all 256 values). A test pins
  `#123456` on `Rgb16` as `(0x1212, 0x3434, 0x5656)`. A 16-bit fill can't be stated (`parse_color`
  returns bytes); recorded as a limit, not a gap.
- **So a fill that fits the image's type always keeps it.**
- ⚠ For design: `Rgb32F`/`Rgba32F` (a float TIFF can decode to it) takes the 8-bit route in the
  DEC-095 helpers (`is_sixteen_bit`, `:344`). Crop and rotate keep the variant through `image`'s own
  functions. Design states what `pad` does with it: a per-variant canvas (fill `c / 255.0`) keeps it,
  the helpers would truncate it. A per-variant build (`ImageBuffer::from_pixel` plus a copy at the
  offset), widening only for grey + chroma, is the likely shape for both exactness and memory.

### Refusals at load (DEC-102: `recipe step N ('pad'): <reason>`, exit 1, no image read)

Shapes only; design writes the exact text. None names a spec, stage or DEC id.

| bad pad | who checks | message shape |
|---|---|---|
| unknown key (`width = 1080`) | table | `unknown key 'width'; expected one of: aspect, gravity, border, color` |
| no form (only `color`) | constructor | `pad requires an aspect (aspect, gravity) or a border` |
| both forms (`aspect` and `border`) | constructor | `'aspect' cannot be combined with 'border'; a pad is either an aspect (aspect, gravity) or a border` |
| `gravity` with a border | constructor | `'gravity' is not used by a border pad` |
| missing `color` | constructor | `pad requires 'color', a hex colour such as "#ffffff"` |
| bad colour (`"red"`, `"#ff00"`, `"transparent"` in SPEC-151) | constructor | watermark's: `'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "red"` |
| a translucent colour (`"#ff000080"`), SPEC-151 only | constructor | `'color' must be opaque, such as "#ffffff"; got "#ff000080"` |
| bad aspect (`"16x9"`, `"0:9"`, `"-1:1"`, `"1.5:1"`) | constructor (crop's parser) | crop's messages, e.g. `write "1.5:1" as "3:2"` |
| border zero, negative, over 1, nan, `1e-50` (0 as an f32: trap 9), wrong type | table | `'border' must be a number > 0 and <= 1, got 0` |
| bad gravity | table (`OneOf(GRAVITY_NAMES)`) | the existing gravity message |

Apply failures (`operation 'pad' failed: …`, exit 1 on one input, exit 6 in a batch): the border that
rounds to 0 px, and the canvas past the caps, each naming the image and canvas sizes.

### Exactness, sized for verification

The matrix: 2 forms × 9 gravities (aspect only) × odd/even remainders × 8 integer colour types
(`L8 La8 Rgb8 Rgba8 L16 La16 Rgb16 Rgba16`) × grey/chromatic fill × native/wasm. It's cheap because
pad computes no pixel: every assertion is equality.

- **Unit (in `pad.rs` or `tests/pad.rs`):** for each colour type, a seeded unique-pixel image padded
  with a grey and a chromatic fill, at all 9 gravities and an odd and an even remainder. Assert every
  input pixel is byte-equal at `(x + left, y + top)`, every other pixel equals the fill (×257 at 16
  bits), and the output colour type follows the rule above. Plus the 4×3 odd-pixel row, the no-op row
  (already at the aspect: byte-equal to the input), and the 0 px border and over-cap apply failures.
- **Pad then crop is the identity:** pad, then `crop` with `x = left / CW`, `y = top / CH`,
  `width = w / CW`, `height = h / CH` gives the input's pixels byte-equal. DEC-112's measured
  pixel-fraction round trip is what makes this exact; drive it, don't assume it.
- **Corpus (SPEC-141):** two recipes over the three existing fixtures (`rgb8_2600x1730`,
  `rgba8_1730x2600`, `rgba16_2600x1730`): `aspect = "1:1"` with a non-centre gravity, and a
  `border`. That's 6 new pixel rows, with no existing line changed
  (`git diff --diff-filter=M origin/main -- tests/fixtures/recipe_corpus/` empty). As with crop, the
  three per-target `.output` files of a pad recipe should be identical, which design can assert.
- **Fixtures** are the corpus's three plus seeded in-test buffers for the luma types (not in the
  corpus), made independently of the code under test.
- **Native vs wasm (DEC-110 limb 2, exact):** two `tests/wasm_roundtrip.rs` rows, as crop and
  rotate/flip each added: a pad transform equal to the expected pixels, and a load and an apply
  refusal giving the native message. `FULL_COUNT` 51 → 53 and `SLIM_COUNT` 49 → 51 in
  `scripts/wasm-ci-check.sh`, and its self-test fixtures that name 51 move in the same PR. No ddphotos
  parity row (the page doesn't pad; STAGE-060).
- **Mutation rows** (SPEC-147's ship): design drives the cheap ones (a removed load check, the gravity
  floor flipped to ceil, the ×257 replaced by `<< 8`, the cap check removed) rather than reasoning them.
  "Apply never fails" isn't claimed: it can fail twice, both named above.

### Sizing: [S], and why

The op is a canvas, a fill and a copy at an offset: smaller than crop (no fractional rectangle), and
both helpers it needs (`parse_aspect`, `Gravity::placement`) exist. No DEC. The verification is a
broad but mechanical equality matrix, 2 corpus recipes, 2 wasm rows, and about 9 load refusals. The
two folded entries are test-only and doc-only. Kept with `transparent` and a refuse rule at format
resolution it becomes an [M], hence the split.

### Scope

**In:** the `pad` op (aspect and border forms, opaque `color`, `gravity`), its key table and load
refusals, its two apply failures, colour type and depth per the rule above, unit tests, 2 corpus
recipes, 2 wasm rows and the count change, the docs entries, and two folded entries:

- **"SPEC-147's verify cleanups that need a PR"** (folded; no size change, both test-side):
  (1) `orientation_of` (`tests/rotate_flip.rs:591`) treats only `Err(exif::Error::NotFound(_))` as
  "no tag" and fails on any other error. (2) The corpus README's "Adding a recipe" table
  (`tests/fixtures/recipe_corpus/README.md:62-71`) quotes 14 / 48 / 51 / 42 / 14 where the test now
  says 18 / 60 / 63 / 54 / 18. **Keeping it right:** this spec's own recipes move those literals again
  (to 20 / 66 / 69 / 60 / 20), so the fix cuts the numbers out of the table and points at
  `recipe_corpus_output.rs` / `recipe_round_trip.rs` for the values. The table keeps the "what to
  raise and by how much" column. A test reading the README is the other option; the cut is cheaper and
  can't go stale.
- **"State when an op uses the refusal hook and when it fails at apply"** (folded; doc-only): the
  `Operation::refusal` doc comment (`src/operation/mod.rs:296`) says a hook refusal is skipped per
  image in a batch while an apply failure is not, so an op uses the hook only for a condition that is
  safe to skip. `pad.rs`'s module doc follows it.

**Out:** see "Out of scope (for this spec specifically)".

### Rough build file list

- `src/operation/pad.rs` (new): `PAD_PARAMS`, `Pad::from_params`, `Operation for Pad`
- `src/operation/mod.rs`: `mod pad`, the export, the `Operation::refusal` doc comment (folded entry),
  and sharing resize's caps (`:1028`) with `pad`
- `src/operation/crop.rs`: `parse_aspect` made `pub(super)` for reuse (no behaviour change)
- `src/operation/registry.rs`: register `"pad"`, and the registry's op-list tests
- `tests/pad.rs` (new), `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs`
- `tests/fixtures/recipe_corpus/v2_pad_aspect.{toml,golden}`, `v2_pad_border.{toml,golden}` and their
  three `.output` files each
- `tests/fixtures/recipe_corpus/README.md`, `tests/rotate_flip.rs` (folded entry)
- `tests/wasm_roundtrip.rs`, `scripts/wasm-ci-check.sh` (`FULL_COUNT`, `SLIM_COUNT`, self-test lines)
- Docs: `docs/recipes.md` (`:98` "Trim / pad [planned]" becomes a `pad` entry marked today, with
  `trim` still planned; `:260`'s PROJ-017 line), `recipes/README.md` (the op table), `docs/api-contract.md`
  and `docs/data-model.md` where they list recipe ops, `skills/crustyimg/SKILL.md`, `CHANGELOG.md`
  (Unreleased). Found with `/usr/bin/grep -rln 'op = "crop"\|`crop`'` over those paths; design
  confirms each lists ops. No verb: the set is frozen at 14 (STAGE-030).

### Open maintainer questions (each with a recommendation)

1. **Split `transparent` into its own entry?** Recommend **yes**: SPEC-151 = opaque fills [S], and a
   new [S] entry for `transparent` and its format rule. (If Q4 is "flatten", re-merge instead.)
2. **`color` required, or a default?** Recommend **required**, refused at load when missing.
3. **What a `border` fraction is of.** Recommend **the long edge**, the same px on all four sides.
   The short edge is the other even-border choice; per-axis is rejected (uneven).
4. **`transparent` into JPEG: refuse or flatten?** Recommend **refuse** per input, after decode and
   before the pipeline, naming the step and the format, exit 4 (`unsupported format`). The flatten
   that exists today (driven: alpha dropped, the hidden colour kept, exit 0, no warning) gives a black
   border for `#00000000`, and "decided here, not left to the encoder" reads as refusing it. Flatten
   is the cheap alternative: pin today's behaviour with a test and document "into JPEG a transparent
   border comes out as the colour under it, black for `transparent`".
5. **A grey image padded with a colour: widen to RGB, or refuse?** Recommend **widen**, per DEC-095's
   watermark-over-grey precedent. No DEC needed.

### Out-of-spec findings

- **Transparent pixels go into JPEG as their hidden colour, with no warning** (driven above, every JPEG
  path). Not new and not pad's. A `transparent` pad would be its first deliberate producer, so it's
  recorded on the new transparent entry rather than filed on its own.

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

- **`transparent` and any fill with alpha below `ff`, and the JPEG rule.** The new STAGE-058 entry
  "`pad` with a transparent fill" (filed at this frame), unless Q4 is ruled "flatten" and it re-merges.
- **An exact pixel size in a recipe.** Refused by DEC-112 rule 6; `resize mode = "fit"` then
  `pad aspect` covers the exact box (driven). Pixels for people come through `edit --pad`.
- **`edit --pad`.** STAGE-058's `edit` flags spec (build-order line 8), with the ordering DEC.
- **`trim`.** STAGE-058's `trim` entry (line 6). It meets the "watermark skipped" entry if it uses
  the refusal hook.
- **An aspect that follows each image's orientation.** STAGE-058's "`crop`'s aspect can follow each
  image's orientation" entry, which adds the key to both ops.
- **The "watermark skipped" batch message.** Its own STAGE-058 entry; `pad` never uses the hook.
- **The ddphotos page.** STAGE-060; the page doesn't pad, so no parity row here.
- **A colour name or a 16-bit fill.** `parse_color` is hex and 8-bit; a name table is its own idea.

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
