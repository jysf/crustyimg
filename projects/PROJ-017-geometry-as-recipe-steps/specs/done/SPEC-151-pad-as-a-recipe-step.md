---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-151
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
  sessions:
    - cycle: frame
      date: 2026-11-06
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 9400987
      tokens_breakdown:
        input: 130
        cache_creation: 176884
        cache_read: 9167823
        output: 56150
      estimated_usd: 7.09
      note: "Re-measured at ship from the session JSONL (65 message ids, deduped by .message.id, max output_tokens per id), after the session's final turns; the build-time figure (8,576,051 tokens, $6.54) was taken before them. Rates: Opus $5/$25 per MTok, cache_creation x1.25 input, cache_read x0.10 input."
    - cycle: design
      date: 2026-11-06
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 18596691
      tokens_breakdown:
        input: 166
        cache_creation: 302125
        cache_read: 18171173
        output: 123227
      estimated_usd: 14.06
      note: "Re-measured at ship from the session JSONL (82 message ids, deduped by .message.id, max output_tokens per id), after the session's final turns; the earlier figure (17,921,519 tokens, $13.61) was taken before them. Includes a throwaway prototype and a 14-row mutation sweep over the full test suite. Rates: Opus $5/$25 per MTok, cache_creation x1.25 input, cache_read x0.10 input."
    - cycle: build
      date: 2026-11-06
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 31954502
      tokens_breakdown:
        input: 300
        cache_creation: 362475
        cache_read: 31487929
        output: 103798
      estimated_usd: 12.36
      note: "Re-measured at ship from the session JSONL (135 message ids, deduped by .message.id, max output_tokens per id), after the session's final turns; the earlier figure (28,368,198 tokens, $10.85) was taken at the last build turn. The session left a hand-written CI wait loop running for about 3.5 hours; that is in these numbers. Rates: Sonnet $3/$15 per MTok, cache_creation x1.25 input, cache_read x0.10 input."
    - cycle: verify
      date: 2026-11-06
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 12762619
      tokens_breakdown:
        input: 156
        cache_creation: 196816
        cache_read: 12502621
        output: 63026
      estimated_usd: 9.06
      note: "First recorded at ship (verify was read-only and never wrote a cost entry). Measured from the session JSONL (77 message ids, deduped by .message.id, max output_tokens per id). The orchestrator's paste read cache_read 12,043,800, output 56,820, total 12,295,899, about $8.66, with the input and cache_creation lines cut off. Excludes a duplicate verify session (f7535b39) the maintainer stopped after its first step because another was already running: 8 ids, 599,774 tokens, $0.63, no verification work; it is not in the totals. Rates: Opus $5/$25 per MTok, cache_creation x1.25 input, cache_read x0.10 input."
    - cycle: punchlist
      date: 2026-11-06
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 2372382
      tokens_breakdown:
        input: 62
        cache_creation: 64844
        cache_read: 2296625
        output: 10851
      estimated_usd: 1.10
      note: "Re-measured at ship from the session JSONL (28 message ids, deduped by .message.id, max output_tokens per id), after the session's final turns; the earlier figure (1,437,289 tokens, $0.72) was taken before its last two turns. Rates: Sonnet $3/$15 per MTok, cache_creation x1.25 input, cache_read x0.10 input."
    - cycle: ship
      date: 2026-11-06
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 3719694
      tokens_breakdown:
        input: 74
        cache_creation: 94808
        cache_read: 3608710
        output: 16102
      estimated_usd: 1.68
      note: "Measured from the session JSONL (37 message ids, deduped by .message.id, max output_tokens per id) before the closing commit, so it excludes the final turns (the commit, push and report). Includes one background `cargo build` of main in a scratch target dir, which cost no tokens. Rates: Sonnet $3/$15 per MTok, cache_creation x1.25 input, cache_read x0.10 input."
  totals:
    tokens_total: 78806875
    estimated_usd: 45.35
    session_count: 6
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


## Design

Designed 2026-11-06 on `main` at `9f59e25` (crustyimg 0.8.2), against the maintainer's five rulings of
2026-11-06 (split, opaque fills only; `color` required; `border` of the long edge; `transparent` will be
refused into a format with no alpha by the later entry; a grey image with a chromatic fill widens to
RGB). **No DEC**: `pad` stays inside DEC-095, DEC-112 and DEC-113, as the frame found. One op, one spec,
**[S]** in code; the cost is verification (see Sizing).

### Design corrections to the frame

1. **The aspect canvas formula is per axis, not "long side kept".** The frame's wording ("its long side
   is the image's") gives a 4000×4000 image padded to `16:9` a 4000×2250 canvas, smaller than the image.
   The rule, as integer maths in `u64` (crop's `aspect_size` with the comparison reversed):
   - if `w·b >= h·a` (the image is at least as wide as `a:b`), the **width binds**: `cw = w`,
     `ch = (2·w·b + a) / (2·a)`, which is `round(w·b/a)` with halves up;
   - else the **height binds**: `ch = h`, `cw = (2·h·a + b) / (2·b)`.

   Driven (Python, exact rationals, the formula above): 4000×4000 → `16:9` is **7111×4000** and → `9:16`
   is **4000×7111**; 6000×4000 → `4:5` (a landscape to a portrait aspect) is **6000×7500**; 1730×2600 →
   `4:5` is **2080×2600**; 1919×1080 → `16:9` (one column off) is **1920×1080**; 1200×801 → `3:2` (one row
   off) is **1202×801**.
2. **The no-op is "the computed canvas equals the image", and it is wider than an exact ratio.** An image
   within half a pixel of the aspect is a no-op: 1918×1079 → `16:9` is a no-op (exact width 1918.22
   rounds to 1918), while the near miss 1200×801 → `3:2` is not: its exact width is 1201.5, a tie,
   which rounds up to 1202, so the step adds two columns (one each side at `center`). A no-op returns
   the step's input unchanged, colour type included: a grey image with a chromatic fill that needs no
   padding stays grey, because no fill pixel is written (DEC-095's luma rule says the same).
3. **The sweep, re-driven against this formula** (trap 17). For 13 aspects (`1:1 16:9 9:16 4:5 3:2 2:3
   40:21 239:100 7:3 65535:1 1:65535 65535:65534 1:2`) and every image from 1×1 to 160×160 (332,800
   cases): 0 cases where the canvas is smaller than the image, 0 where the binding axis isn't the
   image's length, and the grown side is within ½ px of exact in every case. **½ px is reached**, at
   exact ties (1200×801 → `3:2` is one), so ½ is the bound, measured and reached, not a paper maximum.
   1,324 cases are no-ops. The frame's 1,373 can't be reproduced: it didn't record its aspect list, and
   the formula it states can't have produced its "never smaller" result. A further 136,000 near-tie
   cases (sides up to the 50,000 cap and past it to `u32::MAX`, aspects with 65535 sides) held the
   bound. **`f64` never enters the canvas:** the formula is integer arithmetic, so there are no `f64`
   near-ties to search, and no value overflows a `u64` (`2·u32::MAX·65535 < 2^50`). Only the `border`
   form uses `f64`, one product rounded once.
4. **Float images keep their variant.** `Rgb32F`/`Rgba32F` are padded per variant with the fill
   `f32::from(c) / 255.0` per channel and alpha `1.0`; the input's samples are copied bit for bit. The
   DEC-095 helpers (which would take the 8-bit route) are not used by `pad` at all. Driven in the design
   prototype: bit-equal at every offset.
5. **The corpus aspect recipe is `4:5` at `northwest`, not `1:1`.** `4:5` binds the width on the two
   landscape fixtures and the height on the portrait one, so one recipe exercises both branches;
   `northwest` differs from `center` on both axes.
6. **Messages.** The frame gave shapes; the exact text is below. Three differ in wording from the frame's
   shapes: gravity with a border mirrors crop's `'gravity' is only used with 'aspect'` line; the missing
   colour uses the registry's `missing required key '…'` wording plus a hint; and one message covers
   `transparent` and every alpha below `ff`.

### What was driven at design

All on a release build of `9f59e25` in a scratch `CARGO_TARGET_DIR` (deleted after), plus a throwaway
prototype of `pad` and its tests in a scratch `git worktree` (never committed, removed after).

| probe | result |
|---|---|
| crop's 0 px failure, one input (`width = 0.001` on 40×30) | `error: operation 'crop' failed: the crop rounds to 0×30 px on this 40×30 px image; …`, **exit 1**, no file |
| the same, two failing inputs, `--out-dir` | both `error: <path>: operation 'crop' failed: …` lines, `error: 2 of 2 inputs failed`, **exit 6**, nothing written |
| the same, one failing (40×30) and one passing (4000×30) input | one error line, `error: 1 of 2 inputs failed`, **exit 6**; only the passing input's file is written (4×30) |
| pad then crop, 10 odd cases (ImageMagick modelled the pad at this design's offsets; the real binary cropped with `x = left/CW`, `y = top/CH`, `width = w/CW`, `height = h/CH`) | **0 differing pixels in all 10**: 37×23 → `1:1` and `16:9 southeast`, 23×37 → `3:2 northwest`, 1201×801 → `3:2`, 7×501 → `1:1 east`, 1919×1080 → `16:9`, borders 0.013 on 4001×2999, 0.3 on 501×7, 1 on 1×1, 0.0015 on 333×777. Control: the same crop one pixel right differs in all 8 cases with room, and is refused at load in the 2 without |
| the canvas sweep | correction 3 above |
| the prototype: the 37 refusal rows, the 800-case colour-type matrix, the 16-bit `×257` row, the odd-pixel and gravity rows, pad-then-crop through the pipeline (rgba16), the binary 0 px / batch / cap rows | **all pass as written in this spec** (the messages below were checked by `==` against the prototype) |
| the prototype under the full `cargo test --no-fail-fast` | green: 56 test binaries, no existing test touched by adding the op |
| kamadak-exif 0.6.1 on a PNG whose `eXIf` chunk is `II*\0\x08\0\0\0\xff\xff` | `Err(Truncated IFD)`, which the old `orientation_of` (`.ok()?`) reads as "no tag" |
| `format!("{:?}", -0.1_f64)`, `1e-50`, `0.001` (the registry's `float_text`) | `-0.1`, `1e-50`, `0.001` (checked through the refusal rows) |

### The op, in code terms

- **Where:** a new `src/operation/pad.rs` (`Pad`, `PAD_PARAMS`), declared in `src/operation/mod.rs`
  beside `pub mod flip;` (`:31-34`) as `pub mod pad;` and `pub use pad::{Pad, PAD_PARAMS};`.
- **Table** (`PAD_PARAMS`, in this order, which is the order the unknown-key message lists):

  | key | `ParamKind` |
  |---|---|
  | `aspect` | `Text` |
  | `gravity` | `OneOf(GRAVITY_NAMES)` |
  | `border` | `Number { min: 0.0, min_inclusive: false, max: Some(1.0) }` (crop's `width` row) |
  | `color` | `Text` |

- **Registration** in `with_builtins` (`src/operation/registry.rs:305-322`), after `flip`:
  `reg.register_with_params("pad", |p| Ok(Box::new(Pad::from_params(p)?)), PAD_PARAMS);`. Add `pad` to
  the built-in lists in the doc comments at `:275-278` and `:302-304`.
- **`Pad::from_params(&OperationParams) -> Result<Pad, RegistryError>`**, never panics. Runs
  `registry::check_params("pad", PAD_PARAMS, params)` first (`crop.rs:109` does the same), then, in
  this order (each row in the refusal table below is single-fault, so the order only matters for a
  step with two faults):
  1. `aspect` and `border` both present → P20;
  2. neither present → P18 (a step with only `gravity` and `color` lands here too, P19);
  3. `border` and `gravity` both present → P21;
  4. the aspect form: `parse_aspect` (crop's, made `pub(super)` at `crop.rs:226`, no behaviour change)
     → crop's messages; `gravity` read with `Gravity::from_str`, absent → `Gravity::Center`;
  5. the border form: `params.get_f64("border")` (**never `get_f32`**, DEC-112 rule 6; the table has
     already refused a value that is 0 as an `f32`);
  6. `color` absent → P22/P23;
  7. `color` equal to exactly `"transparent"` → the transparency message (P29), checked before the hex
     parse so it gets its own message rather than the hex one;
  8. `crate::text::parse_color(color)` fails → watermark's message (`src/operation/mod.rs:1665-1669`);
  9. the parsed alpha is not `0xff` → the transparency message (P30–P32).
  `Pad` holds the form, the fill `[u8; 3]` and the step's map as written; `params()` returns the map.
- **`fn layout(&self, iw: u32, ih: u32) -> Result<(u32, u32, u32, u32), String>`** (private, pure, no
  allocation): `(canvas_w, canvas_h, left, top)` or the apply-failure reason.
  - Aspect form: the canvas per correction 1, in `u64`.
  - Border form: `px = (fraction * f64::from(iw.max(ih))).round() as u64` (the **long** edge, ruling
    3); `px == 0` → the 0 px reason (A1); else canvas `(w + 2·px, h + 2·px)`, image at `(px, px)`.
  - Then the caps, **in `u64`, before any cast to `u32`**: either side over `MAX_EDGE` → A2; else the
    area over `MAX_AREA` → A3. A canvas past the caps can be far beyond `u32` (6000×4000 → `65535:1` is
    262,140,000 wide), so the cast comes only after both checks.
  - Then the offset: aspect → `Gravity::placement(cw, ch, iw, ih, 0)` (`src/operation/mod.rs:1246`),
    which floors the centre (`(bw - ow) / 2`), so with `center` and an odd remainder the extra fill
    goes right or bottom, matching crop; border → `(px, px)`.
- **The caps are resize's, shared.** Hoist `MAX_EDGE` (50,000) and `MAX_AREA` (134,217,728) out of
  `Resize::apply` (`src/operation/mod.rs:1028`, `:1039`) to module-level `pub(crate) const`s with their
  comments, and use them in both ops. Resize's messages and behaviour are unchanged.
- **`Pad::apply`**: `layout` error → `OperationError::Apply { op: "pad", reason }`. Canvas equal to the
  image → `Ok(img)`, the same `Image` back (the no-op). Otherwise build the canvas **per variant**:
  `ImageBuffer::from_pixel(cw, ch, fill)` then `imageops::replace(&mut canvas, &src, left, top)`
  (exact copy, no blend), and `Ok(img.with_pixels(out))`, which keeps the metadata bundle (DEC-113).
  The fill and output variant follow the colour-type table. A grey input with a chromatic fill is
  widened first with `DynamicImage::to_rgb8` / `to_rgba8` / `to_rgb16` / `to_rgba16`, which copy the
  luma value to all three channels exactly (driven: the 800-case matrix compares replicated samples).
  `DynamicImage` is `#[non_exhaustive]`: an unknown variant is an apply error
  (`this image's colour type cannot be padded`), never a panic or a conversion.
- **Neither form uses `Operation::refusal`.** Both conditions that depend on the image (A1 0 px, A2/A3
  caps) mean the output would be wrong, so they fail at apply and a batch fails that input (exit 6)
  rather than skipping it.

### The colour-type rule (DEC-095)

"Grey fill" means `r == g == b`. Every row keeps the input's bit depth. The fill at 16 bits is `c × 257`
per channel (`0x12` → `0x1212`); at 32-bit float it is `c / 255.0`. A new pixel's alpha is the type's
maximum (255, 65535, 1.0). An input pixel is copied unchanged, alpha included.

| input | grey fill (`#808080`) | chromatic fill (`#123456`) |
|---|---|---|
| `L8` | `L8`, fill `0x80` | `Rgb8`, fill `(0x12, 0x34, 0x56)`; input `l` → `(l, l, l)` |
| `La8` | `La8`, fill `(0x80, 0xff)` | `Rgba8`, fill `(0x12, 0x34, 0x56, 0xff)`; input `(l, a)` → `(l, l, l, a)` |
| `Rgb8` | `Rgb8` | `Rgb8` |
| `Rgba8` | `Rgba8`, fill alpha `0xff` | `Rgba8`, fill alpha `0xff` |
| `L16` | `L16`, fill `0x8080` | `Rgb16`, fill `(0x1212, 0x3434, 0x5656)` |
| `La16` | `La16`, fill `(0x8080, 0xffff)` | `Rgba16`, fill `(0x1212, 0x3434, 0x5656, 0xffff)` |
| `Rgb16` | `Rgb16` | `Rgb16`, fill `(0x1212, 0x3434, 0x5656)` |
| `Rgba16` | `Rgba16`, fill alpha `0xffff` | `Rgba16`, fill alpha `0xffff` |
| `Rgb32F` | `Rgb32F`, fill `128/255` ×3 | `Rgb32F`, fill `(18/255, 52/255, 86/255)` |
| `Rgba32F` | `Rgba32F`, fill alpha `1.0` | `Rgba32F`, fill alpha `1.0` |

`#808081` is chromatic (the boundary row: `L8` + `#808081` → `Rgb8`). A no-op (canvas equals image) keeps
the input type in every row.

### Gravity, by absolute position (the expected offsets; tests hold them as literals)

A 5×2 image padded to `1:1` gets a 5×5 canvas (vertical remainder 3); a 2×5 image gets 5×5 (horizontal
remainder 3). Offsets `(left, top)`:

| gravity | 5×2 | 2×5 |
|---|---|---|
| `center` | (0, 1) | (1, 0) |
| `north` | (0, 0) | (1, 0) |
| `south` | (0, 3) | (1, 0) |
| `east` | (0, 1) | (3, 0) |
| `west` | (0, 1) | (0, 0) |
| `northeast` | (0, 0) | (3, 0) |
| `northwest` | (0, 0) | (0, 0) |
| `southeast` | (0, 3) | (3, 0) |
| `southwest` | (0, 3) | (0, 0) |

And the odd pixel: a 4×3 image → `1:1` at `center` is a 4×4 canvas with the image at `(0, 0)` and one fill
row below (remainder 1, `floor(1/2) = 0` above).

### Exact messages

**Load.** Each is a `RecipeError::InvalidOperation`, displayed `recipe step N ('pad'): <reason>`
(`src/recipe/mod.rs:124`), exit 1 on `apply`, nothing read or written. Table rows come from
`check_params` (`src/operation/registry.rs:237-262`), which walks keys in **sorted** order (`aspect`,
`border`, `color`, `gravity`, then any unknown key in its place), so each row below has exactly one
fault. Every body below follows `op = "pad"`; the message follows `recipe step 0 ('pad'): `.

Matched messages, cited: `Number` range and type lines `registry.rs:115-134` (`expectation`) and `:212-224` (`describe_value`); crop's
both-forms `crop.rs:115-118`, gravity-misused `crop.rs:136-138`, no-form `crop.rs:142-146`, aspect
`crop.rs:228-231` (generic) and `:257-259` (decimal); watermark's colour `src/operation/mod.rs:1665-1669`;
missing key `src/operation/mod.rs:817`; resize's caps `src/operation/mod.rs:1044-1059`.

| id | step body | who | message |
|---|---|---|---|
| P1 | `aspect = "1:1"` `color = "#ffffff"` `width = 1080` | table | `unknown key 'width'; expected one of: aspect, gravity, border, color` |
| P2 | `aspect = "1:1"` `color = "#ffffff"` `height = 1080` | table | `unknown key 'height'; expected one of: aspect, gravity, border, color` |
| P3 | `aspect = "1:1"` `color = "#ffffff"` `mode = "aspect"` | table | `unknown key 'mode'; expected one of: aspect, gravity, border, color` |
| P4 | `border = 0` `color = "#ffffff"` | table | `'border' must be a number > 0 and <= 1, got 0` |
| P5 | `border = -0.1` … | table | `… got -0.1` |
| P6 | `border = 1.5` … | table | `… got 1.5` |
| P7 | `border = 800` … | table | `… got 800` |
| P8 | `border = nan` … | table | `… got nan` |
| P9 | `border = inf` … | table | `… got inf` |
| P10 | `border = -inf` … | table | `… got -inf` |
| P11 | `border = 1e-50` … | table | `… got 1e-50` |
| P12 | `border = "0.05"` … | table | `… got the string "0.05"` |
| P13 | `border = true` … | table | `… got true` |
| P14 | `aspect = "1:1"` `gravity = "middle"` `color = "#ffffff"` | table | `'gravity' must be one of: center, north, south, east, west, northeast, northwest, southeast, southwest; got "middle"` |
| P15 | `aspect = "1:1"` `gravity = 1` `color = "#ffffff"` | table | as P14, `got the integer 1` |
| P16 | `aspect = 16` `color = "#ffffff"` | table | `'aspect' must be a string, got the integer 16` |
| P17 | `aspect = "1:1"` `color = 16777215` | table | `'color' must be a string, got the integer 16777215` |
| P18 | `color = "#ffffff"` | constructor | `pad needs an aspect (aspect, and optionally gravity) or a border` |
| P19 | `gravity = "north"` `color = "#ffffff"` | constructor | as P18 |
| P20 | `aspect = "1:1"` `border = 0.1` `color = "#ffffff"` | constructor | `'aspect' cannot be combined with 'border'; a pad is either an aspect (aspect, gravity) or a border` |
| P21 | `border = 0.1` `gravity = "north"` `color = "#ffffff"` | constructor | `'gravity' is only used with 'aspect'; a border pad adds the same width on every side` |
| P22 | `aspect = "1:1"` | constructor | `missing required key 'color'; a pad needs a fill colour such as "#ffffff"` |
| P23 | `border = 0.1` | constructor | as P22 |
| P24 | `aspect = "1:1"` `color = "red"` | constructor | `'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "red"` |
| P25 | `aspect = "1:1"` `color = "#ff00"` | constructor | as P24, `got "#ff00"` |
| P26 | `aspect = "1:1"` `color = "#gggggg"` | constructor | as P24, `got "#gggggg"` |
| P27 | `aspect = "1:1"` `color = ""` | constructor | as P24, `got ""` |
| P28 | `aspect = "1:1"` `color = "Transparent"` | constructor | as P24, `got "Transparent"` (the keyword is lowercase only, like the gravity names) |
| P29 | `aspect = "1:1"` `color = "transparent"` | constructor | `a fill with transparency is not supported yet; 'color' must be opaque, such as "#ffffff", got "transparent"` |
| P30 | `aspect = "1:1"` `color = "#ff000080"` | constructor | as P29, `got "#ff000080"` |
| P31 | `border = 0.1` `color = "ff000000"` | constructor | as P29, `got "ff000000"` |
| P32 | `aspect = "1:1"` `color = "#ffffffFE"` | constructor | as P29, `got "#ffffffFE"` (alpha one below opaque) |
| P33 | `aspect = "16x9"` `color = "#ffffff"` | constructor (crop's parser) | `'aspect' must be two whole numbers from 1 to 65535 written W:H, such as "16:9"; got "16x9"` |
| P34 | `aspect = "0:9"` … | constructor | as P33, `got "0:9"` |
| P35 | `aspect = "-1:1"` … | constructor | as P33, `got "-1:1"` |
| P36 | `aspect = "65536:1"` … | constructor | as P33, `got "65536:1"` |
| P37 | `aspect = "1.5:1"` … | constructor | `'aspect' must be two whole numbers written W:H; write "1.5:1" as "3:2"` |

Rows P4–P13 elide `color = "#ffffff"` and the message's `'border' must be a number > 0 and <= 1, ` prefix
only in this table; the test holds every message in full. **Trap 10 rows:** P5 (`-0.1` is inexact as an
`f32`; the message prints the value as written), P11 (`1e-50` passes as an `f64` and is 0 as an `f32`),
P32 (one step below opaque), P28 (a near-miss spelling of the keyword). The P29 message names no later
spec and promises nothing about formats.

**Accepted, as controls for the colour rows:** `"#FFFFFF"`, `"ffffff"`, `"#123456ff"` and `"123456FF"`
each load (`parse_color` reads either case), and so does `border = 1` (a TOML integer) and
`aspect = "65535:1"`.

**Apply.** `operation 'pad' failed: <reason>` (`src/operation/mod.rs:187`); exit 1 on one input; in a
batch that input fails, nothing is written for it, the others are written, and the run exits 6 with
`error: K of N inputs failed` (driven on crop's equivalent above). wasm `transform` throws the same line.

| id | case | reason |
|---|---|---|
| A1 | `border = 0.001` on 40×30 | `the border rounds to 0 px on this 40×30 px image ('border' = 0.001 of its 40 px long edge); it must add at least 1 px` |
| A2 | `aspect = "65535:1"` on 1×1 | `'aspect' = "65535:1" on this 1×1 px image needs a 65535×1 px canvas, which exceeds the maximum edge limit of 50000 px` |
| A2b | `border = 1` on 20000×1 | `'border' = 1 on this 20000×1 px image needs a 60000×40001 px canvas, which exceeds the maximum edge limit of 50000 px` |
| A2c | `aspect = "65535:1"` on 6000×4000 | `'aspect' = "65535:1" on this 6000×4000 px image needs a 262140000×4000 px canvas, which exceeds the maximum edge limit of 50000 px` |
| A3 | `aspect = "1:1"` on 1×11586 | `'aspect' = "1:1" on this 1×11586 px image needs a 11586×11586 px canvas = 134235396 px, which exceeds the maximum area of 134217728 px` |

The border value prints as written (`registry::written`), so `0.001` prints `0.001`, not its `f32`
spelling; the aspect prints as the string written. **Boundaries:** `border = 0.0124` on 40×30 is A1
(0.496 px); `border = 0.0125` adds 1 px (`0.0125 × 40` rounds to 1; driven in the prototype);
1×11585 → `1:1` (134,212,225 px) passes; `aspect = "50000:1"` on 1×1 passes and `"50001:1"` is A2-shaped.

### Sizing: [S] in code, broad in verification

About 200 lines of op code in one file (a table, a constructor, a pure `layout`, a per-variant copy), two
one-line moves (`parse_aspect` visibility, the caps), one registration. The verification is the cost: 37
refusal rows (data lines in one test), an 800-case equality matrix, 2 corpus recipes across three targets,
2 wasm tests with the count move, 8 doc surfaces, and the two folded entries. That is the same shape as
SPEC-147 ([S], 58 ACs), so it stays [S]. The prototype that drove this design was about 550 lines with
tests, which is a fair estimate of the build.

## Inputs

- **Files to read:** `src/operation/crop.rs` (the shape to copy: table :25-66, `from_params` :107-189,
  `aspect_size` :212-222, `parse_aspect` :226-263, `apply` :325-385, unit tests);
  `src/operation/flip.rs` and `rotate.rs` (the newest ops); `src/operation/mod.rs` (`Operation` trait
  and `refusal` :274-304, `get_f64` :117, the DEC-095 helpers :334-430 (read, not used), the resize caps
  :1024-1061, `Gravity` and `placement` :1174-1270, watermark's colour message :1663-1670);
  `src/text/mod.rs:45-73` (`parse_color`); `src/operation/registry.rs` (`ParamKind` :78-185,
  `float_text` / `written` / `describe_value` :190-224, `check_params` :237-262, `with_builtins`
  :302-322, tests :500-790); `src/image/mod.rs` (`from_parts` :389, `with_pixels` :421);
  `tests/crop.rs` and `tests/rotate_flip.rs` (the integration shape: seeded buffers, `apply` helper,
  refusal tables); `tests/common/mod.rs` (`tiff_with_orientation` :134);
  `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs:257-340`,
  `tests/fixtures/recipe_corpus/README.md`; `tests/wasm_roundtrip.rs:360-540`;
  `scripts/wasm-ci-check.sh:15-20,200-230`; `tests/docs_ops.rs`.
- **Decisions:** DEC-095, DEC-102, DEC-109, DEC-110, DEC-112, DEC-113, DEC-003.
- **Related code paths:** `src/operation/`, `tests/`, `scripts/`, `docs/`, `recipes/README.md`,
  `skills/`, `integrations/ddphotos/README.md`.

## Outputs

- **Files created:** `src/operation/pad.rs` (the op and its unit tests); `tests/pad.rs` (load
  refusals, the colour-type matrix, binary rows, pad then crop, metadata);
  `tests/fixtures/recipe_corpus/v2_pad_aspect.{toml,golden,linux-x86_64.output,macos-aarch64.output,windows-x86_64.output}`
  and the same five for `v2_pad_border`.
- **Files modified:** `src/operation/mod.rs` (`pub mod pad`, the re-export, the caps hoisted, the
  `Operation` and `refusal` doc comments); `src/operation/crop.rs` (`parse_aspect` → `pub(super)`, no
  other change); `src/operation/registry.rs` (one registration, doc lists, test extensions);
  `tests/common/mod.rs` (`orientation_of`, moved here and made strict); `tests/rotate_flip.rs` (uses
  `common::orientation_of`; two new tests); `tests/recipe_corpus_output.rs` and
  `tests/recipe_round_trip.rs` (stems and literals only); `tests/fixtures/recipe_corpus/README.md` (the
  "Adding a recipe" table); `tests/wasm_roundtrip.rs` (two tests); `scripts/wasm-ci-check.sh` (two
  constants and the self-test fixtures); `docs/recipes.md`, `recipes/README.md`, `docs/data-model.md`,
  `docs/api-contract.md`, `skills/crustyimg/SKILL.md`, `integrations/ddphotos/README.md`,
  `docs/backlog.md`, `CHANGELOG.md`.
- **New exports:** `crustyimg::operation::{Pad, PAD_PARAMS}`, with
  `pub fn Pad::from_params(&OperationParams) -> Result<Pad, RegistryError>` and its `Operation` impl.
- **Database changes:** none. No `CACHE_SCHEMA_VERSION` change, no lockfile change, no version bump.

## Acceptance Criteria

**The op**
- [ ] **AC-1** `OperationRegistry::with_builtins()` registers `pad`. `param_specs("pad")` is exactly
  `[aspect: Text, gravity: OneOf(GRAVITY_NAMES), border: Number { min: 0.0, min_inclusive: false, max:
  Some(1.0) }, color: Text]`, in that order. The eight existing ops keep their tables unchanged.
- [ ] **AC-2** **The aspect canvas, per axis.** `layout` gives exactly: 4000×4000 → `16:9` 7111×4000;
  → `9:16` 4000×7111; 6000×4000 → `4:5` 6000×7500; 1730×2600 → `4:5` 2080×2600; 1200×801 → `3:2`
  1202×801; 1919×1080 → `16:9` 1920×1080; 4×3 → `1:1` 4×4.
- [ ] **AC-3** **The canvas sweep.** For the 13 aspects of correction 3 and every image from 1×1 to
  64×64 (53,248 cases), through `layout`: the canvas is never smaller than the image; the binding axis
  (width when `w·b >= h·a`, else height) equals the image's; the grown side `c` satisfies
  `|2·c·a − 2·w·b| <= a` (width binds) or `|2·c·b − 2·h·a| <= b` (height binds), which is "within ½ px"
  in integers; at least one case reaches exactly ½ px (a tie); and the test asserts the case count.
- [ ] **AC-4** **No-op.** 1200×800 → `3:2` and 1918×1079 → `16:9` return the input: same bytes, same
  variant, same size, on rgb8 and rgba16 buffers; an `L8` 6×4 buffer → `3:2` with fill `#123456` stays
  `L8`, byte-equal. On the binary, `apply` with `aspect = "3:2"` on a 6×4 PNG writes a 6×4 PNG whose
  decoded samples equal the input's, exit 0.
- [ ] **AC-5** **Border of the long edge.** `px = round(border × max(w, h))` (`f64`, halves up), on every
  side: 40×30 at `0.1` → 48×38 at (4, 4); 30×40 → 38×48 at (4, 4); 1×1 at `1` (a TOML integer) → 3×3 at
  (1, 1); 4001×2999 at `0.013` → 4105×3103 at (52, 52); 40×30 at `0.0125` → 42×32 at (1, 1); 50×10 at `0.01` → 52×12 at (1, 1) (exactly ½ px in `f64`, so 1 px; `0.49999999` in `f32`, so 0 px); 5×3 at `0.7` → 13×11 at (4, 4) (3.5 px → 4; `f32` gives 3).
- [ ] **AC-6** **The odd pixel.** 4×3 → `1:1` at `center` is 4×4 with the image at (0, 0) and row 3 all
  fill; 3×4 → 4×4 with column 3 all fill; 5×2 → 5×5 with the image at (0, 1): one fill row above, two
  below.
- [ ] **AC-7** **Gravity by absolute position.** Each of the 18 offsets in the gravity table (9
  gravities × {5×2, 2×5} → `1:1`) is exactly as listed. On the same cases through `apply`, input pixel
  (0, 0) is at `(left, top)` and input pixel (w−1, h−1) is at `(left + w − 1, top + h − 1)`, with the
  offsets taken from the table literals.
- [ ] **AC-8** **Exactness in every colour type.** For each of the 10 colour types × the fills `#808080`
  and `#123456` × the shapes 7×4, 6×4, 4×7, 4×6 (odd and even remainders on each axis) × the 9
  gravities at `aspect = "1:1"` plus `border = 0.25` (800 cases, seeded buffers): the output variant is the
  colour-type table's, the size is as computed, every input pixel equals its sample (replicated to
  `(l, l, l)` / `(l, l, l, a)` where the table widens) at `(x + left, y + top)`, compared as raw samples
  (`f32` by bit pattern), and every other pixel equals the table's fill. The offset comes from a
  test-local function written from the gravity names (`rx / 2`, `0` or `rx`), never from
  `Gravity::placement`. The test asserts 800 cases ran.
- [ ] **AC-9** **The colour-type table, row by row.** For each of its 20 cells, on a 2×1 seeded buffer
  with `border = 1`: the output variant, the fill read at (0, 0), and the input pixel read at (2, 2)
  (its alpha unchanged where the input had alpha, including an input alpha of 0 on `Rgba8` and
  `La16`). Plus the boundary row `L8` + `#808081` → `Rgb8`.
- [ ] **AC-10** **16-bit fill is `×257`.** `#123456` on `Rgb16` gives the fill `(0x1212, 0x3434,
  0x5656)`; on `Rgba16` `(0x1212, 0x3434, 0x5656, 0xffff)`; `#ffffff` on `L16` gives 65535 and `#000000`
  gives 0.
- [ ] **AC-11** **Float images keep their variant.** `#ff8000` on `Rgb32F` gives the fill `(1.0,
  128.0/255.0, 0.0)` as `f32`; on `Rgba32F` `(1.0, 128.0/255.0, 0.0, 1.0)`; the input's samples are
  bit-equal at their offset; the output is still `Rgb32F` / `Rgba32F`.
- [ ] **AC-12** `params()` returns the step's map exactly as written: `border = 1` stays the integer `1`,
  `border = 0.05` the float, `color = "#FFFFFF"` keeps its case, an absent `gravity` stays absent.
- [ ] **AC-13** **No refusal hook; apply fails only for the named conditions.** `refusal(&img)` is `None`
  for both forms on 1×1, 1×7 and 7×1 rgb8 buffers, and `apply` is `Ok` on those shapes for `aspect`
  `1:1`, `16:9`, `9:16` and `border = 1`. `apply` fails only for A1–A3, each pinned below.
- [ ] **AC-14** **Metadata (DEC-113).** Input: a seeded 7×5 rgb8 PNG carrying an `eXIf` chunk with
  Orientation 6 (built as `tests/rotate_flip.rs`'s `tagged_png` builds it). On the binary: `pad border =
  0.2` (1 px) writes 9×7 with no Orientation tag (`common::orientation_of` gives `None` or `Some(1)`);
  `pad border = 0.2` then `auto-orient` writes **7×9** (the bundle survived the pad); control:
  `auto-orient` alone writes 5×7.

**Load refusals** (DEC-102)
- [ ] **AC-15 … AC-51** (one per row) Each row P1–P37 is refused by `Recipe::build_pipeline` with
  `with_builtins()` as a `RecipeError::InvalidOperation` whose `Display` is **exactly**
  `recipe step 0 ('pad'): ` + that row's message (full text, no elision).
- [ ] **AC-52** The accepted controls load: `color` `"#FFFFFF"`, `"ffffff"`, `"#123456ff"`,
  `"123456FF"`; `border = 1`; `aspect = "65535:1"`.
- [ ] **AC-53** On the binary, `apply --recipe <P30> in.png -o out.png` exits 1, prints exactly
  `error: recipe step 0 ('pad'): a fill with transparency is not supported yet; 'color' must be opaque,
  such as "#ffffff", got "#ff000080"`, and writes no `out.png`.
- [ ] **AC-54** On the binary, `apply --recipe <P1>` exits 1, prints exactly `error: recipe step 0
  ('pad'): unknown key 'width'; expected one of: aspect, gravity, border, color`, and writes nothing.

**Apply failures**
- [ ] **AC-55** **0 px, one input.** `apply` with `border = 0.001` on a 40×30 PNG exits 1, prints exactly
  `error: operation 'pad' failed: ` + A1, and writes no `out.png`.
- [ ] **AC-56** **0 px, a batch.** `apply --out-dir` with the same recipe on `a.png` (40×30) and `b.png`
  (4000×3): exit 6; stderr contains `a.png: operation 'pad' failed: ` + A1 and the line
  `error: 1 of 2 inputs failed`; `out/a.png` does not exist; `out/b.png` is 4008×11.
- [ ] **AC-57** **Over the edge cap, one input.** `apply` with `aspect = "65535:1"` on a 1×1 `L8` PNG
  exits 1, prints exactly `error: operation 'pad' failed: ` + A2, and writes nothing. (Cheap: under a
  removed cap check this allocates only 65,535 bytes.)
- [ ] **AC-58** **Over the area cap, a batch.** `apply --out-dir` with `aspect = "1:1"` on `a.png` (1×11586
  `L8`) and `b.png` (1×100 `L8`): exit 6; stderr contains `a.png: operation 'pad' failed: ` + A3 and
  `error: 1 of 2 inputs failed`; `out/a.png` does not exist; `out/b.png` is 100×100.
- [ ] **AC-59** **The caps and the 0 px rule, through `layout` (no allocation).** A2b, A2c and A3 exactly;
  1×11585 → `1:1` passes; 1×1 → `50000:1` passes and → `50001:1` fails with the edge message; `border =
  0.0124` on 40×30 is A1-shaped (`0.0124`) and `0.0125` passes with 1 px.
- [ ] **AC-60** **Pad then crop is the identity.** Through the pipeline API, on seeded rgba16 buffers:
  37×23 → `16:9 southeast`; 23×37 → `3:2 northwest`; 1201×801 → `3:2`; 7×501 → `1:1 east`; 501×7 →
  `border = 0.3`; 1×1 → `border = 1`. Each pad is followed by `crop` with `x = left/CW`, `y = top/CH`,
  `width = w/CW`, `height = h/CH` (`f64`, printed with `{:?}`, offsets from the test-local function or
  the border width), and the result's `as_bytes()` and variant equal the input's.

**End to end, corpus, wasm**
- [ ] **AC-61** On the binary, `apply` with `aspect = "1:1"`, `color = "#123456"` on a seeded 4×3 rgb8
  PNG writes a 4×4 PNG: the input's pixels at rows 0–2, row 3 all `(0x12, 0x34, 0x56)`, exit 0.
- [ ] **AC-62** The corpus gains `v2_pad_aspect` (one step: `op = "pad"`, `aspect = "4:5"`,
  `gravity = "northwest"`, `color = "#1e90ff"`) and `v2_pad_border` (one step: `op = "pad"`,
  `border = 0.05`, `color = "#ffffff"`), each with `.toml`, `.golden` and three per-target `.output`
  files, added by the README's procedure. `EXPECTED_DIMS`: `v2_pad_aspect`
  `["2600x3250", "2080x2600", "2600x3250"]`; `v2_pad_border` `["2860x1990", "1990x2860", "2860x1990"]`.
  The literals become (recounted at design from `main` at `9f59e25`): `RECIPES` **20** stems (the two
  inserted in sorted order, after `v2_format_quality_optimize`), the `EXPECTED_DIMS` array length **20**,
  `ROWS` **66**, `GOLDEN_LINES` **69**, the `54` in `EXPECTED_DIMS.len() * FIXTURES.len()` → **60**, the
  doc comments ("all 20", "20 recipes x 3 fixtures", "60 pixel rows"), `recipe_round_trip.rs` `CORPUS`
  +2 stems in sorted order, and `pairs.len()` **20** with the message "17 corpus recipes + 3 bundled
  recipes".
- [ ] **AC-63** For each new recipe, its three per-target `.output` files are byte-identical (no float
  maths on pixels), and the build says so with the `cmp` it ran.
- [ ] **AC-64** **No existing corpus output changes:** `git diff --diff-filter=MD --name-only origin/main
  -- tests/fixtures/recipe_corpus/` lists exactly `tests/fixtures/recipe_corpus/README.md`;
  `--diff-filter=A` lists exactly the ten new files; `cargo test --test recipe_corpus_output` and
  `--test recipe_round_trip` pass with no re-bless of an existing file.
- [ ] **AC-65** wasm `transform` (DEC-110 limb 2, exact): on `png_unique(120, 80)`, `aspect = "1:1"`,
  `gravity = "south"`, `color = "#123456"` gives exactly 120×120 with the input at (0, 40) and every
  other pixel `(0x12, 0x34, 0x56, 0xff)`; `border = 0.05`, `color = "#123456"` gives exactly 132×92
  with the input at (6, 6). Zero pixels differ from the hand map, in each.
- [ ] **AC-66** wasm `transform` throws, and the message contains exactly the native line, for P30, P22
  and A1 (`border = 0.001` on `png_unique(40, 30)`, so `operation 'pad' failed: ` + A1); an identity
  transform succeeds after each.
- [ ] **AC-67** `scripts/wasm-ci-check.sh`: **`FULL_COUNT=53`, `SLIM_COUNT=51`** (both new wasm tests run
  in both sets; main is 51/49 at `9f59e25`, re-read after the build's last rebase and add 2 to whatever
  it then says), and the `--self-test` fixtures move with them: the base line is
  `test result: ok. 53 passed; …` in a variable named `l53`; derived files `l1`, `l52` (one under full)
  and `l51` (slim); rows 1 `count full 53` → ok `full set: 53 passed, as expected`; 2 `count full 1` →
  fail `full set: 1 passed, expected 53`; 3 `count full 52` → fail `expected 53`; 4 `count slim 53` →
  fail `slim set: 53 passed, expected 51`; 5 `count slim 51` → ok `slim set: 51 passed, as expected`; 7
  (`two`) and 9 (`count fat`) built from `l53`. Row 8 (`FAILED. 48 passed; 1 failed`) is unchanged.
  `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh --self-test` and `LC_ALL=en_US.UTF-8
  scripts/wasm-ci-check.sh workflow` both exit 0, and CI's full and slim count steps pass.

**Folded entries**
- [ ] **AC-68** **A stricter `orientation_of`** (folded entry "SPEC-147's verify cleanups that need a PR",
  part 1). `orientation_of` moves from `tests/rotate_flip.rs:591-597` to `tests/common/mod.rs` as
  `pub fn orientation_of(bytes: &[u8]) -> Option<u32>`: `Err(exif::Error::NotFound(_))` → `None`; any
  other `Err(e)` → `panic!("kamadak-exif could not read the EXIF in these bytes: {e}")`. Its two
  callers in `tests/rotate_flip.rs` and AC-14's caller use it. New tests in `tests/rotate_flip.rs`:
  `orientation_of_fails_on_a_malformed_exif_chunk` (`#[should_panic(expected = "kamadak-exif could not
  read")]`, a PNG whose `eXIf` chunk is `II*\0\x08\0\0\0\xff\xff`, which kamadak-exif 0.6.1 reads as
  `Truncated IFD`) and `orientation_of_reads_no_tag_as_none` (a plain PNG → `None`). Ship marks the
  STAGE-058 entry `[x]`.
- [ ] **AC-69** **The corpus README's literals** (the same folded entry, part 2). The "Adding a recipe"
  table in `tests/fixtures/recipe_corpus/README.md` (`:62-71`) is replaced by the table in "README
  text" below, word for word, plus its lead sentence. Check: `sed -n '/^## Adding a recipe/,/^## One
  target/p' tests/fixtures/recipe_corpus/README.md | /usr/bin/grep -nE '\b[0-9]{2,}\b'` prints nothing
  (no recipe, row or line count is left in the section).
- [ ] **AC-70** **When an op uses the refusal hook** (folded entry "State when an op uses the refusal hook
  and when it fails at apply"). The `Operation::refusal` doc comment (`src/operation/mod.rs:296-300`)
  keeps its first paragraph and gains the paragraph in "Doc comment text" below; the `Operation` trait
  doc (`:274-280`) gains its one sentence; `pad.rs`'s module doc says `pad` fails at apply for both
  conditions and uses no hook. Verify checks by reading. Ship marks the STAGE-058 entry `[x]`.

**Code moves with no behaviour change**
- [ ] **AC-71** `MAX_EDGE` and `MAX_AREA` are module-level `pub(crate) const`s in `src/operation/mod.rs`
  (their comments move with them), used by `Resize::apply` and `Pad`. Resize's existing cap tests pass
  unchanged (`cargo test --lib operation::tests` with no edit to a resize test).
- [ ] **AC-72** `parse_aspect` is `pub(super)`; `crop.rs` changes in nothing else (`git diff
  origin/main -- src/operation/crop.rs` is that one line), and `cargo test --test crop` and
  `--lib operation::crop` pass unchanged.

**Docs** (verify checks each by reading; `file:line` is on `main` at `9f59e25`)
- [ ] **AC-73** `docs/recipes.md:98` (`- **Trim / pad** **[planned — PROJ-017]**`) becomes a `pad`
  entry marked `[today]` after the flip entry, then `- **Trim** **[planned — PROJ-017]**`. The entry:
  "**Pad to an aspect or add a border** **[today]** — a recipe step:", a TOML snippet (`op = "pad"`,
  `aspect = "1:1"`, `color = "#ffffff"`), "or the same border on every side:", a second snippet
  (`op = "pad"`, `border = 0.05`, `color = "#000000"`), and these sentences: "`color` is required: a hex
  colour such as `\"#ffffff\"`, with no default." / "`aspect` grows the canvas to the smallest `W:H`
  shape that holds the image, placed by `gravity` (default `center`); an image already at that aspect
  is written unchanged." / "`border` is a fraction of the image's long edge, added on every side: a
  6000×4000 photo with `border = 0.05` gets 300 px all round." / "A grey image padded with a colour
  that is not grey becomes RGB." / "A transparent fill is not supported yet." / "For an exact pixel box
  such as 1080×1080, resize first (`op = \"resize\"`, `mode = \"fit\"`, `width = 1080`, `height =
  1080`), then pad with `aspect = \"1:1\"`. `fit` never enlarges, so a smaller photo gives a smaller
  square." / "Put `op = \"auto-orient\"` first when the recipe is meant as the photo is displayed."
- [ ] **AC-74** `docs/recipes.md:260` reads: `**PROJ-017:** geometry as recipe steps — \`crop\`,
  \`rotate\` (quarter turns), \`flip\` and \`pad\` today; \`trim\` and straighten **planned**.`
- [ ] **AC-75** `recipes/README.md:95`: the op table gains a `pad` row after `flip`: "`aspect` + `gravity`,
  or `border`, and a required `color` (below)". A new `### The pad step` section before
  `### Step keys are checked` (`:268`) gives both forms as TOML and says, as bullets: the canvas grows to
  the smallest `W:H` shape that holds the image, and an image already at the aspect is unchanged;
  `gravity` places the image (default `center`), and with `center` an odd leftover pixel goes on the
  right or bottom; `border` is a fraction of the long edge, the same width on every side, and one that
  rounds to 0 px on an image fails that image (a batch exits 6); a canvas past 50,000 px on a side or
  134,217,728 px in all fails the same way; `color` is required, hex only, opaque; a transparent fill
  is not supported yet; a grey image with a colour fill becomes RGB, and the bit depth is kept; for an
  exact pixel box, `resize` with `mode = "fit"` first; put `op = "auto-orient"` first when the recipe is
  meant as the photo is displayed.
- [ ] **AC-76** `docs/data-model.md:162-164`: "eight operations" becomes **nine**, naming `pad`. A new
  `### \`pad\` step param keys` section after the rotate/flip section (before `### Step keys are checked
  at load`, `:392`) has a `| Key | Type | Range |` table (`aspect` / string / `"W:H"`, whole numbers 1 to
  65535; `gravity` / string / one of the nine names, default `center`, aspect form only; `border` /
  number / `> 0` and `<= 1`, a fraction of the long edge; `color` / string / required, `RRGGBB`,
  `#RRGGBB`, or `RRGGBBAA` with alpha `ff`), the sentence "A pad is an aspect (`aspect`, `gravity`) or a
  border (`border`), never both.", and a TOML example with `op = "auto-orient"`, `op = "resize"`
  (`mode = "fit"`, 1080×1080) and `op = "pad"` (`aspect = "1:1"`) steps, which `tests/docs_ops.rs`
  then checks resolve.
- [ ] **AC-77** `skills/crustyimg/SKILL.md:233-236`: the ops list gains `pad` (`aspect = "W:H"` +
  `gravity`, or `border` as a fraction of the long edge, with a required opaque `color`) after `flip`,
  and the frontmatter `description` (`:3`) says "crop, rotate, flip, pad" where it says "crop, rotate,
  flip".
- [ ] **AC-78** `docs/api-contract.md`: a "**Pad in a recipe.**" paragraph after the rotate/flip
  paragraph (`:468-472`) says `op = "pad"` grows the canvas to an aspect (`aspect`, `gravity`) or by a
  border (`border`, a fraction of the long edge), filled with a required opaque `color`; a bad key or
  value, a translucent colour and `transparent` are refused when the recipe loads (exit **1**, nothing
  read or written); a border that rounds to 0 px or a canvas past the size caps fails at apply
  (`operation 'pad' failed: …`, exit **1** for one input, exit **6** in a batch, with nothing written
  for that input); wasm `transform` throws the same line; and the pixel lane writes no Orientation tag
  after it.
- [ ] **AC-79** `CHANGELOG.md`: under `## [Unreleased]` → `### Added`, after the rotate/flip entry
  (`:16-18`), "**A recipe can pad.**" — `op = "pad"` with `aspect` (and `gravity`) or `border`, and a
  required opaque `color`; the input's pixels are unchanged and its bit depth kept; a grey image with
  a colour fill becomes RGB; 0.8.2 and earlier refuse it with `unknown operation 'pad'`.
- [ ] **AC-80** `integrations/ddphotos/README.md:127-129` says recipes can crop, rotate, flip and pad
  (`op = "pad"` added to the list) and the edit page offers none of them yet. `docs/backlog.md:194`'s
  `pad` / `extend` row says it shipped as a recipe step (`aspect` or `border`, opaque fill); a
  transparent fill is a later STAGE-058 entry. `README.md:244` and `npm/README.md` are unchanged (their
  "and more" / "…" covers it).
- [ ] **AC-81** No user-facing string this spec adds (messages, doc comments, docs) names a spec, stage
  or DEC id: `/usr/bin/grep -nE "SPEC-|STAGE-|DEC-" src/operation/pad.rs` finds nothing, and the
  `Operation` doc comment edits name none. (`docs/backlog.md` is internal and may name STAGE-058.)

**Hygiene**
- [ ] **AC-82** No version bump, no `CACHE_SCHEMA_VERSION` change (`src/build/cache.rs:81`), no
  lockfile-format change. Why none is needed: no existing recipe's canonical TOML or output changes
  (AC-64); the cache key already folds in the crustyimg version (`src/cli/build.rs:439`); and 0.8.2
  refuses a `pad` recipe at load (`unknown operation 'pad'`), so no cached entry or lockfile line for
  one can exist.
- [ ] **AC-83** `just check` is green; CI is green on every leg, including the lean
  (`--no-default-features`) build, both wasm count steps and the wasm bundle-size gate (the build
  records the raw and brotli sizes against `main`).

## Failing Tests

Written at design; the build makes them pass. **Baseline on `main`:** every integration row is RED with
`unknown operation 'pad'` (load rows report it as the wrong message; binary rows as exit 1 with that
line; wasm rows the same), the corpus rows RED with a missing `v2_pad_aspect` / `v2_pad_border` file,
the two new `orientation_of` rows RED (`orientation_of` is not in `common`; and, once moved but still
lenient, the malformed row fails with "test did not panic"), and the unit tests don't compile until
`pad.rs` exists. The build quotes one RED message per test file. **Wrong-reason trap (13):** every
refusal and apply row asserts the whole message with `==` (or `contains` the whole line for batch
stderr, where the path varies), so a row refused for another row's reason fails visibly. The likely
wrong-reason RED is AC-14: if the test's `eXIf` chunk never loads, control B gives 7×5, not 5×7, and the
test must say "the input's Orientation tag never reached the pipeline", not "pad failed".

- **`src/operation/pad.rs` (`#[cfg(test)] mod tests`)** — calls the private `layout` and
  `Pad::from_params` + `apply`, never a copy.
  - `canvas_follows_the_binding_axis` — AC-2, the 7 rows as data.
  - `canvas_sweep_never_smaller_and_within_half_a_pixel` — AC-3: 13 aspects × 1..=64 × 1..=64; asserts
    53,248 cases ran and at least one tie.
  - `no_op_returns_the_input` — AC-4 unit half (rgb8, rgba16, and `L8` with `#123456`).
  - `border_is_a_fraction_of_the_long_edge` — AC-5, 7 rows.
  - `odd_remainder_puts_the_extra_fill_right_or_bottom` — AC-6 through `layout`, and through `apply` on
    a seeded 4×3 rgb8 buffer (row 3 all fill).
  - `gravity_places_by_absolute_position` — AC-7, the 18 offsets as literals.
  - `caps_and_zero_px_fail_before_allocating` — AC-59, every row exact.
  - `params_round_trip_as_written` — AC-12.
  - `refusal_hook_is_not_used` — AC-13 (3 shapes × 4 steps).
- **`src/operation/registry.rs` (`mod tests`)**
  - `with_builtins_contains_pad` (new) — AC-1.
  - `every_builtin_publishes_a_param_schema` (extended) — `keys("pad") == ["aspect", "gravity",
    "border", "color"]`, kinds as AC-1; existing asserts unchanged.
  - `one_of_values_are_accepted_by_their_constructors` (extended) — all 9 pad gravities build with
    `aspect = "1:1"`, `color = "#000000"`; the literal becomes `6 + 9 + 9 + 2 + 9`, message "6 resize
    modes + 9 watermark gravities + 9 crop gravities + 2 flip axes + 9 pad gravities".
- **`tests/pad.rs` (new)** — through the public API (`Recipe::from_toml`, `build_pipeline`,
  `Pipeline::run`) and the binary (`env!("CARGO_BIN_EXE_crustyimg")`, stdout and stderr captured, exit
  code read directly). Seeded buffers come from an xorshift fill written in the test (copy
  `tests/rotate_flip.rs`'s `seeded`, all 10 kinds), never from the code under test. Expected pixels come
  from hand rules (the offset function, the table's fills), never from `imageops`.
  - `every_pad_refusal_names_its_reason` — AC-15…51: the 37-row table as data `(id, body, message)`,
    each recipe `version = "2"` with one step; runs **every** row, collects each mismatch as
    `"{id}: got …"` (a row that builds `Ok` is reported as such), and asserts 37 rows ran.
  - `accepted_colour_and_value_spellings_load` — AC-52.
  - `every_colour_type_pads_exactly` — AC-8, 800 cases, failures named by kind, fill, step and shape.
  - `colour_type_table_row_by_row` — AC-9, 21 rows.
  - `sixteen_bit_fill_is_times_257` — AC-10.
  - `float_images_keep_their_variant` — AC-11.
  - `gravity_corners_land_by_absolute_position` — AC-7 through `apply`.
  - `pad_then_crop_is_the_identity` — AC-60, 6 cases.
  - `apply_reads_the_border_as_f64` — AC-5 binary half (50×10 at `0.01` → 52×12).
  - `apply_pads_a_png_end_to_end` — AC-61. `apply_no_op_pad_writes_the_input_pixels` — AC-4 binary half.
  - `apply_refuses_a_translucent_fill_and_writes_nothing` — AC-53.
    `apply_refuses_a_pixel_width_key_and_writes_nothing` — AC-54.
  - `apply_fails_a_0px_border_with_exit_1` — AC-55.
    `batch_with_a_0px_border_fails_that_input_with_exit_6` — AC-56.
  - `apply_fails_a_canvas_past_the_edge_cap_with_exit_1` — AC-57.
    `batch_with_a_canvas_past_the_area_cap_fails_that_input_with_exit_6` — AC-58.
  - `pad_writes_no_orientation_tag_and_keeps_the_bundle` — AC-14 (control B first, then the two rows).
- **`tests/rotate_flip.rs`** — AC-68: `orientation_of_fails_on_a_malformed_exif_chunk`,
  `orientation_of_reads_no_tag_as_none`; the existing orientation rows now call
  `common::orientation_of`.
- **`tests/recipe_corpus_output.rs` / `tests/recipe_round_trip.rs`** — AC-62…64: the two stems, their
  `EXPECTED_DIMS` and goldens, the literal bumps; no other change. Both recipes change every fixture's
  size, so the corpus's own "expected dims", "equals fixture" and "every acting step acts" controls apply
  to them unchanged.
- **`tests/wasm_roundtrip.rs`** — AC-65…66, `#[wasm_bindgen_test]`, not feature-gated:
  `transform_pads_exactly` and `transform_refuses_a_bad_pad_with_the_native_message`.
- **`scripts/wasm-ci-check.sh --self-test`** — AC-67's rows.
- **`tests/docs_ops.rs`** — unchanged; it goes red if AC-76's example lands before `pad` is registered.

### Mutation table (traps 12, 13 and 14)

Apply one at a time, run `cargo test --no-fail-fast` (stdout redirected to a file, `$?` read directly)
and, where the row says, the corpus test or the script; record the flip, revert, confirm green. A row
whose red set doesn't match is reported, not edited to match.

**Driven** means applied at design to a prototype of this spec (scratch `git worktree` of `9f59e25`,
own `CARGO_TARGET_DIR`, full `cargo test --no-fail-fast`, prototype baseline fully green, the edit
checked to have changed the file before each run). The prototype's tests carry this spec's names where
they existed; tests this spec adds that the prototype didn't have are listed as *reasoned* in the same
cell. Every row's "green" column was observed green over the whole suite unless marked reasoned.

| # | mutation, and where | red (expected) | green (expected) |
|---|---|---|---|
| M1 | **gravity floor → ceil**: in `Gravity::placement` (`src/operation/mod.rs:1250`, `:1254`), `x_center` and `y_center` become `(… + 1) / 2` | *driven:* `every_colour_type_pads_exactly` (odd-remainder shapes); `odd_remainder_puts_the_extra_fill_right_or_bottom`; `gravity_places_by_absolute_position`; `pad_then_crop_is_the_identity` (the crop takes the floor offset); **crop's** `operation::crop::tests::aspect_gravity_places_all_nine` (shared function). *Reasoned:* `gravity_corners_land_by_absolute_position`; corpus `v2_pad_aspect` stays green (`northwest` uses no centre) | *driven:* every watermark test (none pins an odd centred remainder); corpus `v2_crop_aspect` (`southeast`); every refusal row; every border row |
| M2 | **gravity swapped**: in `Gravity::placement`, `Gravity::North => (x_center, y_bottom)` | *driven:* `gravity_places_by_absolute_position` (`north` → (0, 3) on 5×2); `every_colour_type_pads_exactly` (the `north` cases); crop's `aspect_gravity_places_all_nine`. *Reasoned:* `gravity_corners_land_by_absolute_position` | *driven:* `odd_remainder_…` (centre only); `pad_then_crop_is_the_identity` (no `north` case: it computes its offset with the same test-local function, so it is an equality row, not a position row); every watermark test (out-of-spec gap, see "Out of scope"); every border and refusal row; the corpus (no `north`) |
| M3 | **`×257` → `<< 8`**: in `Pad::apply`'s 16-bit fill, `u16::from(c) << 8` | *driven:* `sixteen_bit_fill_is_times_257` (`0x1200`, not `0x1212`); `every_colour_type_pads_exactly` (16-bit types, both fills). *Reasoned:* `colour_type_table_row_by_row` (16-bit rows); corpus `v2_pad_border` and `v2_pad_aspect` on `rgba16_2600x1730` **by pixel hash** (same dims, trap 13's golden-red-by-hash row) | *driven:* every 8-bit and float case; every refusal, position and cap row |
| M4 | **long edge → short edge**: in `layout`, `w.min(h)` for `w.max(h)` | *driven:* `border_is_a_fraction_of_the_long_edge` (40×30 at 0.1 → 46×36); `caps_and_zero_px_fail_before_allocating` (20000×1 at `border = 1` adds 1 px, no cap error; 40×30 at 0.0125 rounds 0.375 → 0); `apply_fails_a_0px_border_with_exit_1` and `batch_with_a_0px_border_…` (the 4000×3 input now rounds to 0 too: 2 of 2 failed); `every_colour_type_pads_exactly` (border cases); `colour_type_table_row_by_row` (2×1 at `border = 1` → 1 px; its (2, 2) read lands outside); `pad_then_crop_is_the_identity` (501×7 border). *Reasoned:* corpus `v2_pad_border` (dims `2776x1903`, not `2860x1990`); wasm `transform_pads_exactly` border row | *driven:* every aspect row; every refusal row |
| M5a | **opaque check removed**: in `from_params`, delete the `rgba[3] != u8::MAX` refusal | *driven:* `every_pad_refusal_names_its_reason`, rows P30, P31, P32 each **built Ok**. *Reasoned:* `apply_refuses_a_translucent_fill_and_writes_nothing` (exit 0, a file written); wasm P30 | *driven:* P29 `transparent` (still caught by the keyword check: it's the other independent condition, hence M5b); every other refusal |
| M5b | **`transparent` keyword check removed** | *driven:* `every_pad_refusal_names_its_reason`, row P29 only: `got … 'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "transparent"` | *driven:* P30–P32 (alpha check); every other test |
| M6 | **0 px check removed**: in `layout`, skip the `px == 0` refusal | *driven:* `caps_and_zero_px_fail_before_allocating` (A1 row returns `Ok`); `apply_fails_a_0px_border_with_exit_1` (**exit 0**); `batch_with_a_0px_border_fails_that_input_with_exit_6` (**exit 0**). **What is written instead:** `px = 0` makes the canvas equal the image, so `apply` takes the no-op path and writes the input **unpadded**, exit 0, no message. *Reasoned:* wasm A1 row (no throw) | *driven:* every border row with a non-zero border; every aspect row |
| M7 | **cap check removed**: in `layout`, both cap `if`s become `if false` | *driven:* `caps_and_zero_px_fail_before_allocating` (pure: no canvas is allocated, so the 262,140,000-wide row costs nothing); `apply_fails_a_canvas_past_the_edge_cap_with_exit_1` (exit 0: a 65535×1 `L8` canvas, 64 KB, is written). *Reasoned:* `batch_with_a_canvas_past_the_area_cap_…` (exit 0; allocates ~134 MB at `L8`) | *driven:* every other test |
| M8 | **widening removed**: `let chroma = false;` in `Pad::apply` (grey + chromatic fill keeps grey, fill channel `r`) | *driven:* `colour_type_table_row_by_row` (prototype: `grey_image_widens_only_for_a_chromatic_fill`, `L8` + `#808081` stays `L8`); `every_colour_type_pads_exactly` (`l8`/`la8`/`l16`/`la16` × `#123456`: variant `l8`, want `rgb8`) | *driven:* every grey-fill case; every RGB, RGBA and float case; the corpus (all fixtures are RGB/RGBA) |
| M8b | **float fill not scaled**: `f32::from(c)` for `f32::from(c) / 255.0` | *driven:* `every_colour_type_pads_exactly` (`rgb32f`/`rgba32f` cases). *Reasoned:* `float_images_keep_their_variant` | *driven:* every integer case |
| M9 | **wasm count**: `FULL_COUNT=52` in `scripts/wasm-ci-check.sh`, after AC-67's change | *driven:* `count full` on a 53-passed log → `full set: 53 passed, expected 52`, exit 1; `--self-test` rows **1** (wrong verdict), **2** (wrong message: lacks `expected 53`) and **3** (wrong verdict: `count full 52` passes) | *driven:* `workflow` (16 assertions, 0 failed); self-test rows 4, 5 and 6–17. AC-67's own literals (53/51, `l53`/`l52`/`l51`) pass self-test 18/18 and workflow, driven |
| M10 | **corpus guard** (scratch, never committed): append `[[step]]` `op = "pad"` `border = 0.05` `color = "#ffffff"` to `tests/fixtures/recipe_corpus/v1_resize_max.toml` | *driven:* `every_recipe_output_matches_its_golden`, naming `v1_resize_max × rgb8_2600x1730: dims 880x612, expected 800x532 (expected-dims table)`, the same for `rgba8_1730x2600` (`612x880`, expected `532x800`) and `rgba16_2600x1730`. *Reasoned:* `recipe_round_trip`'s golden and `to_toml_is_byte_identical_to_main_over_the_corpus` (cargo stopped after the first failing binary, so they did not run) | every pad test |
| M11 | **trap 14 for the folded fix, the old code verbatim**: restore `orientation_of`'s `.read_from_container(…).ok()?` | *driven:* `orientation_of_fails_on_a_malformed_exif_chunk` (`test did not panic as expected`) | *driven:* every other `tests/rotate_flip.rs` row, including `orientation_of_reads_no_tag_as_none` (15 of 16 pass) |
| M12 | **trap 14 for the op, the old registry verbatim**: restore `with_builtins` from `main` (no `pad` line) | *driven:* every integration row in `tests/pad.rs` (`unknown operation 'pad'` in each load row; exit 1 with that line in each binary row). *Reasoned:* `with_builtins_contains_pad`; `every_builtin_publishes_a_param_schema`; `one_of_values_are_accepted_by_their_constructors`; `pad_writes_no_orientation_tag_and_keeps_the_bundle` (control B passes, then the pad step fails at load); corpus `v2_pad_aspect`, `v2_pad_border` and `to_toml_is_byte_identical_to_main_over_the_corpus`; `every_op_named_in_data_model_resolves` (AC-76's example); both wasm tests | *driven:* the `pad.rs` unit tests (they call the module directly); every existing test |
| M13 | **bundle dropped**: `Pad::apply` returns `Image::from_parts(out, img.source_format(), None)` | *reasoned:* `pad_writes_no_orientation_tag_and_keeps_the_bundle`'s second row (`pad` then `auto-orient` gives 9×7, not 7×9) | its first row (no tag either way); every other test |
| M14 | **`f32` read**: `get_f32("border").map(f64::from)` for `get_f64` | *driven:* the two new `border_is_a_fraction_of_the_long_edge` rows and `apply_reads_the_border_as_f64`. Verify found `0.01` on a 50 px edge splits `f64` (1 px) from `f32` (0 px), so the read is now pinned | every other test |

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle. It is the
equivalent of a handoff document, folded into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-112` — a recipe carries lengths as fractions of the image the step receives, read as `f64`,
  rounded half up, zero pixels refused at apply; rule 6 names `pad` and leaves "a fraction of what" to
  it (the long edge, ruling 3) and refuses an exact pixel size (`width`/`height` are unknown keys).
  Aspects are whole numbers in integer maths (rule 5).
- `DEC-095` — keep the colour type and bit depth. `pad` doesn't use the widen/narrow helpers: it builds
  per variant, and widens a grey image only for a chromatic fill, the watermark-over-grey case this DEC
  already rules. Float images keep their variant.
- `DEC-102` — the key table runs before the constructor; the constructor holds the cross-key rules (the
  two forms, `gravity` with a border, `color`'s syntax and opacity). A number is also range-checked as
  an `f32`, which is what refuses `1e-50`.
- `DEC-113` — a geometry step acts on the buffer it receives and writes no metadata; `with_pixels` keeps
  the bundle for a later `auto-orient`.
- `DEC-109` — the precedent for refusing a step that would do nothing (the 0 px border) instead of
  writing the image unchanged.
- `DEC-110` — limb 2: no resampling, so native and wasm agree exactly, dimensions and pixels.
- `DEC-003` — the pixel lane writes no container metadata.

### Constraints that apply

- `untrusted-input-hardening` — the canvas is computed in `u64` and checked against resize's caps
  before any cast or allocation; nothing panics on any TOML value or image size.
- `no-unwrap-on-recoverable-paths` — no `unwrap`/`expect` in `pad.rs` outside tests.
- `single-image-library` — `image` only.
- `every-public-fn-tested` — `Pad::from_params` and the `Operation` impl.
- `test-before-implementation` — the tests above, red first, baseline quoted.
- `one-spec-per-pr` — one PR, `feat/spec-151-pad`.

### Prior related work

- `SPEC-145` (shipped) — `crop`: `parse_aspect`, `aspect_size`, the 0 px apply failure, `tests/crop.rs`,
  the corpus and wasm rows this spec copies.
- `SPEC-147` (shipped) — `rotate`/`flip`: the newest op shape, `tests/rotate_flip.rs`, the count move in
  `scripts/wasm-ci-check.sh`, and the two cleanups folded in here.
- `SPEC-141` (shipped) — the corpus guard; its README is AC-62's procedure and AC-69's target.
- `SPEC-131` (shipped) — the key tables and typed constructors.

### README text (AC-69, word for word)

Replace the paragraph's first sentence and the table under `## Adding a recipe` (`:62-71`) with:

> Add the recipe's `<stem>.toml` and `<stem>.golden` here (or a bundled recipe under `recipes/`),
> then raise every literal in the same diff. The current values are in the two test files; this
> table says only where each one is and how it moves for a new recipe with no byte row:
>
> | where | literal | change |
> |---|---|---|
> | `recipe_corpus_output.rs` | `RECIPES` | add the stem, in sorted order among the corpus stems |
> | `recipe_corpus_output.rs` | `EXPECTED_DIMS` | add one entry: the recipe's output size on each of the three fixtures |
> | `recipe_corpus_output.rs` | the array length in `EXPECTED_DIMS`'s type | +1 per recipe |
> | `recipe_corpus_output.rs` | `ROWS` | +3 per recipe, +3 per new `png` or `webp` byte row |
> | `recipe_corpus_output.rs` | `GOLDEN_LINES` | +3 per recipe, +3 per new byte row, +1 per new bundled recipe |
> | `recipe_corpus_output.rs` | the count in `assert_eq!(EXPECTED_DIMS.len() * FIXTURES.len(), …)` | +3 per recipe |
> | `recipe_corpus_output.rs` | the counts in the module, `ROWS` and `GOLDEN_LINES` doc comments | to match |
> | `recipe_round_trip.rs` | `CORPUS`, and the count and message in `assert_eq!(pairs.len(), …)` | add the stem; +1 for a corpus recipe |

Everything after the table (byte rows, the `grep`, blessing) is unchanged.

### Doc comment text (AC-70)

`Operation::refusal` (`src/operation/mod.rs:296-300`) keeps its paragraph and gains:

> A refusal here is **skippable**. A single input that is refused exits 2, but in a batch the step is
> skipped for that image, which is written without it, and the run exits 0. So refuse here only when
> leaving the step out is an acceptable result, such as a watermark too large for a small image. When
> running the step would give a wrong output (a crop or a pad border that rounds to 0 px, a canvas
> past the size caps), return an error from `apply` instead: one input exits 1, and in a batch that
> input fails, nothing is written for it, and the run exits 6.

The `Operation` trait doc (`:274-280`) gains one sentence at its end: "A new op that can decline to run
on some images reads [`Operation::refusal`] first: it says which conditions belong there and which
belong in `apply`." `pad.rs`'s module doc ends: "Both conditions that depend on the image (a border that
rounds to 0 px, a canvas past the size caps) are errors from `apply`, not refusals, because skipping
the step would write an unpadded image."

(The exit 2 / exit 0 behaviour of a hook refusal is SPEC-145's measured M4b, recorded on the STAGE-058
entry; the exit 1 / exit 6 behaviour of an apply failure is driven at this design.)

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- **`transparent` and any fill with alpha below `ff`, and the no-alpha-format rule.** The STAGE-058 entry
  "`pad` with a transparent fill …" (ruled: refused into a format with no alpha, exit 4). SPEC-151 refuses
  both at load with P29/P30's message, which promises nothing about formats.
- **An exact pixel size in a recipe.** Refused by DEC-112 rule 6 (`width`/`height` are unknown keys);
  `resize mode = "fit"` then `pad aspect` gives the exact box (driven at the frame, 36/36).
- **`edit --pad`.** STAGE-058's `edit` flags spec, with the ordering DEC.
- **`trim`.** Its own STAGE-058 entry.
- **An aspect that follows each image's orientation.** STAGE-058's "`crop`'s aspect can follow each
  image's orientation" entry, which adds the key to both ops.
- **The "watermark skipped" batch message.** Its own STAGE-058 entry; `pad` never uses the hook.
- **The ddphotos page.** STAGE-060; the page doesn't pad, so no parity row.
- **A colour name or a 16-bit fill.** `parse_color` is hex and 8-bit.
- **Watermark's gravity tests.** Out-of-spec observation from mutation M2: swapping `north` for `south`
  in `Gravity::placement` leaves every watermark test green; pad's AC-7 rows now pin all nine at margin
  0, but watermark's margin path is unpinned. Not fixed here.

## Notes for the Implementer

- **Checkpoint.** Commit and push after each of: (1) the red tests, with the baseline quoted; (2) the op
  green locally; (3) the corpus files for your own target; (4) the wasm tests and the script; (5) the
  folded entries; (6) the docs. A long build with no pushed commit has been lost before. Open the PR
  as a **draft**.
- **The red-test commit** can hold the integration, registry and `tests/rotate_flip.rs` tests only; the
  unit tests in `pad.rs` can't compile before the module exists (SPEC-147's build hit the same).
- **Corpus bless order.** Add the two `.toml` files (a two-line `#` comment each, as `v2_crop_aspect.toml`
  has, then `version = "2"`, `name = "corpus-pad-aspect"` / `"corpus-pad-border"`, one step), write their
  `.golden` by the README (canonical TOML: keys sorted, so `op` first, then `aspect`, `color`, `gravity`
  / `border`, `color`; copy the shape `to_toml()` prints), raise every literal in one commit, bless your
  own target, push, and copy the other two targets' files from the failing CI log's
  `----- BEGIN … -----` blocks. Then `cmp` all three per recipe (AC-63). Gate each commit on its test
  run; don't chain `git commit` after `cargo test` with `;`.
- **wasm.** Both new wasm tests run in the full and slim sets, so both constants move by 2. Re-read both
  on `main` after your last rebase (PROJ-011's STAGE-063 may move them) and move the self-test fixtures
  in the same commit as the constants. Run `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh --self-test`
  and `… workflow` locally, `just wasm-test` and `just wasm-test-slim` if you can, and read CI's two
  count steps.
- **Mutations.** Drive the table below in a scratch `git worktree` of your pushed HEAD with its own
  `CARGO_TARGET_DIR`, never in the build worktree. Every row except the corpus and wasm rows was driven
  at design over the full `cargo test --no-fail-fast` on a prototype; a row whose red set differs on
  the real code is reported in Build Completion, not edited to match. M7 under the area-cap batch row
  (AC-58) allocates about 134 MB; the edge rows allocate under 1 MB.
- **`cargo test` in a terminal** fails `display_sink_refuses_non_tty`: redirect stdout to a file and
  read `$?`, never through a pipe. Use your own `CARGO_TARGET_DIR`. Run the lean build
  (`cargo test --no-default-features`) too: `pad` has no feature gate.
- **The metadata test's input** must load its tag: check control B (5×7 after `auto-orient`) before
  trusting the pad rows. `tests/rotate_flip.rs`'s `tagged_png` (`:580-590`) is the recipe for it.
- **User-facing text** (messages, doc comments, docs) reads behaviour-first and names no spec, stage or
  DEC id.
- **Files the PROJ-011 orchestrator must keep clear of** while this builds: `src/operation/`,
  `src/recipe/`, `src/text/mod.rs` (read only here), `docs/recipes.md`, `recipes/README.md`,
  `docs/data-model.md`, `docs/api-contract.md`, `docs/backlog.md`, `skills/crustyimg/SKILL.md`,
  `integrations/ddphotos/README.md`, `CHANGELOG.md`'s Unreleased section,
  `tests/fixtures/recipe_corpus/`, `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs`,
  `tests/wasm_roundtrip.rs`, `tests/common/mod.rs`, `tests/rotate_flip.rs`, `scripts/wasm-ci-check.sh`.
  If STAGE-063 adds a wasm test, the counts move: the build re-reads them (above).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-151-pad`
- **PR (if applicable):** #255 (draft)
- **All acceptance criteria met?** yes (83 of 83; details below)
- **New decisions emitted:** none
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`: 31 files, including this spec (the 30 Outputs plus the spec)):
  - `src/operation/pad.rs` (new) — the op, `PAD_PARAMS`, `Pad::from_params`, pure `layout` and `aspect_canvas`, the per-variant `grow`, 9 unit tests
  - `src/operation/mod.rs` — `pub mod pad` and the re-export; `MAX_EDGE`/`MAX_AREA` hoisted to module-level `pub(crate)` consts (resize uses them, messages unchanged); `Operation` and `refusal` doc text
  - `src/operation/crop.rs` — `parse_aspect` is `pub(super)`, one line
  - `src/operation/registry.rs` — one registration, doc lists, three test extensions
  - `tests/pad.rs` (new) — 17 integration rows
  - `tests/common/mod.rs` — strict `orientation_of`
  - `tests/rotate_flip.rs` — uses `common::orientation_of`; two new rows
  - `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs` — two stems, `EXPECTED_DIMS` entries, literals bumped
  - `tests/fixtures/recipe_corpus/v2_pad_aspect.{toml,golden}` and `v2_pad_border.{toml,golden}`, plus three `.output` files each (10 new files)
  - `tests/fixtures/recipe_corpus/README.md` — "Adding a recipe" table without counts
  - `tests/wasm_roundtrip.rs` — `transform_pads_exactly`, `transform_refuses_a_bad_pad_with_the_native_message`
  - `scripts/wasm-ci-check.sh` — `FULL_COUNT=53`, `SLIM_COUNT=51`, self-test fixtures (`l53`, `l52`, `l51`)
  - `docs/recipes.md`, `recipes/README.md`, `docs/data-model.md`, `docs/api-contract.md`, `skills/crustyimg/SKILL.md`, `integrations/ddphotos/README.md`, `docs/backlog.md`, `CHANGELOG.md` — docs
  - No `Cargo.toml`, `Cargo.lock`, `CACHE_SCHEMA_VERSION` or version change (`git diff --stat origin/main...HEAD` on those paths is empty).
- **Deviations from spec:**
  - The canvas formula is its own private function, `aspect_canvas(a, b, w, h) -> (u64, u64)`, which `layout` calls. Reason: AC-3's sweep "through `layout`" cannot hold for the extreme aspects (`65535:1`, `1:65535`), because `layout` refuses a canvas past the caps. The sweep calls `aspect_canvas` instead, so all 53,248 cases are checked. `layout` is unchanged in behaviour.
  - The comments on the hoisted caps lost their `SPEC-`/`DEC-` ids (ruling 5).
  - `common::orientation_of` has two callers in `tests/rotate_flip.rs` and one in `tests/pad.rs` (the spec now says two plus AC-14's).
  - Two local type aliases (`Offset`, `Layout`) and `as_chunks::<3>()` in tests, for clippy `-D warnings`; MSRV 1.90 covers `as_chunks`.
- **Follow-up work identified:**
  - `pad` at the area cap on an `Rgba32F` image allocates 134,217,728 px x 16 B = 2 GiB, against 512 MiB at RGBA8 (the cap is in pixels, not bytes). Float inputs are rare and decode has its own limits, but a byte-based cap is a hardening question for a decision, not for this spec.
  - CI's wasm-size step is path-filtered, so main's run has no size line to diff against. Head (slim): raw 4,137,932 B, brotli 829,807 B. No delta was measurable from CI.

### Acceptance criteria

All 83 met. AC-1 to AC-14 by the unit rows and `tests/pad.rs` rows of the same names; AC-15 to AC-51 by `every_pad_refusal_names_its_reason` (37 rows, asserted); AC-52 to AC-54 and AC-55 to AC-58 by the named binary rows (exit codes 1, 1, 1, 6, 1, 6 as specified); AC-59 by `caps_and_zero_px_fail_before_allocating`; AC-60 by `pad_then_crop_is_the_identity` (6 cases); AC-61 to AC-64 below; AC-65, AC-66 by the two wasm rows (53 and 51 ran green locally and in CI); AC-67 below; AC-68 to AC-70 and AC-71 to AC-72 as worded (crop.rs diff is the one line); AC-73 to AC-80 by reading against the spec's text; AC-81: `grep -nE "SPEC-|STAGE-|DEC-" src/operation/pad.rs` finds nothing, and no added line in `mod.rs` names an id; AC-82: no version, cache-schema or lockfile change; AC-83: `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test --no-fail-fast` ran on the three matrix legs (not the `just check` wrapper itself), and CI is green on every leg.

- **Baseline (red tests, stage 1):** `tests/pad.rs` 17 of 17 red with `unknown operation 'pad'` (for example `"aspect = \"1:1\" color = \"#FFFFFF\"" was refused: unknown operation 'pad'`); registry rows `with_builtins_contains_pad`, `every_builtin_publishes_a_param_schema` and `one_of_values_are_accepted_by_their_constructors` red; with a lenient `orientation_of`, `orientation_of_fails_on_a_malformed_exif_chunk` red with `test did not panic as expected` and the other 15 `rotate_flip` rows green.
- **Corpus (AC-62 to AC-64):** the two recipes, with the literals `RECIPES` 20, `EXPECTED_DIMS` 20, `ROWS` 66, `GOLDEN_LINES` 69, `60`, `pairs.len()` 20. First CI run on the corpus commit's successor failed only on the missing linux and windows files (mac 1 failed test, others 2). I copied those four files from the failing log, then `cmp` of macos-aarch64 vs linux-x86_64 vs windows-x86_64 was identical for `v2_pad_aspect` and for `v2_pad_border`. The second CI run (head `746733c`) is green on every leg: macos, ubuntu, windows, lean, webp-lossy, avif, both heic, msrv, cargo-deny, wasm Node, ddphotos parity, browser smoke, DCO. `git diff --diff-filter=MD --name-only origin/main -- tests/fixtures/recipe_corpus/` lists only `README.md`; `--diff-filter=A` lists 10 files; no existing `.output` or `.golden` changed.
- **wasm and the script (AC-67):** `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh --self-test`: `18 rows, 18 passed, 0 wrong verdict, 0 wrong message`. `... workflow`: `16 assertions, 0 failed`. `./scripts/validate-frontmatter.sh`: `all front-matter blocks parse`. `just wasm-check` exit 0 (one pre-existing dead-code warning, `MANIFEST_DROPPED_WARNING`); `just wasm-test`: `53 passed`; `just wasm-test-slim`: `51 passed`. CI: `full set: 53 passed, as expected` and `slim set: 51 passed, as expected`. Main's constants at the last fetch were still 51/49, so base had not moved. Bundle (slim, from the parity job): raw 4,137,932 B, brotli 829,807 B; no main baseline in CI.
- **Matrix:** default, `--no-default-features` and `--features webp-lossy`, each in a fresh `CARGO_TARGET_DIR`: clippy `-D warnings` 0, `fmt --check` 0, `cargo test --no-fail-fast` 0 with 56 test binaries ok, on all three. (My first webp attempt failed on a quoting slip in my own loop, then re-ran correctly.)

### Mutations

Driven in a scratch `git worktree` of the pushed head (`746733c`), own `CARGO_TARGET_DIR`, `cargo clean -p crustyimg` before each run, full `cargo test --no-fail-fast` to a file, `$?` read, restored and diff shown empty. Native only: wasm rows are marked reasoned and were not run under mutation.

| row | applied at | red (observed) | against the table |
|---|---|---|---|
| M1 | `Gravity::placement` x_center, y_center `+ 1` | `pad::gravity_places_by_absolute_position`, `pad::odd_remainder_puts_the_extra_fill_right_or_bottom`, `crop::aspect_gravity_places_all_nine`; `tests/pad.rs`: `gravity_corners_land_by_absolute_position`, `every_colour_type_pads_exactly`, `pad_then_crop_is_the_identity`, `apply_pads_a_png_end_to_end` | matches; one extra red, `apply_pads_a_png_end_to_end` (4x3 to 1:1 at centre is an odd remainder). Corpus and watermark green, as expected. `gravity_corners_land_by_absolute_position` (reasoned) confirmed |
| M2 | `Gravity::North => (x_center, y_bottom)` | `pad::gravity_places_by_absolute_position`, `crop::aspect_gravity_places_all_nine`, `gravity_corners_land_by_absolute_position`, `every_colour_type_pads_exactly` | matches; reasoned cell confirmed |
| M3 | `pad.rs` `wide`: `u16::from(c) << 8` | `sixteen_bit_fill_is_times_257`, `colour_type_table_row_by_row`, `every_colour_type_pads_exactly`, corpus `every_recipe_output_matches_its_golden` | matches; both reasoned cells (table rows, corpus by hash) confirmed |
| M4 | `layout`: `iw.min(ih)` | `pad::border_is_a_fraction_of_the_long_edge`, `pad::caps_and_zero_px_fail_before_allocating`, `colour_type_table_row_by_row`, `apply_fails_a_0px_border_with_exit_1`, `batch_with_a_0px_border_fails_that_input_with_exit_6`, `every_colour_type_pads_exactly`, `pad_then_crop_is_the_identity`, corpus golden | matches; one extra red, `float_images_keep_their_variant` (its `border = 1` on 3x2 adds 2 px, not 3). Corpus reasoned cell confirmed; wasm border row not run |
| M5a | alpha check `if false && ...` | `every_pad_refusal_names_its_reason`, `apply_refuses_a_translucent_fill_and_writes_nothing` | matches; reasoned cell confirmed |
| M5b | keyword check `if false && ...` | `every_pad_refusal_names_its_reason` only | matches |
| M6 | `if false && px == 0` | `pad::caps_and_zero_px_fail_before_allocating`, `apply_fails_a_0px_border_with_exit_1`, `batch_with_a_0px_border_...` | matches |
| M7 | both cap `if`s become `if false` | `pad::caps_and_zero_px_fail_before_allocating`, `apply_fails_a_canvas_past_the_edge_cap_with_exit_1`, `batch_with_a_canvas_past_the_area_cap_...` | matches; reasoned cell confirmed |
| M8 | `let chromatic = false;` | `colour_type_table_row_by_row`, `every_colour_type_pads_exactly` | matches |
| M8b | `f32::from(c)` unscaled | `float_images_keep_their_variant`, `every_colour_type_pads_exactly`, `colour_type_table_row_by_row` | matches (the reasoned cell is confirmed; the table row is an extra red) |
| M9 | `FULL_COUNT=52` | `count full` on a 53 log: `full set: 53 passed, expected 52`, exit 1; self-test rows 1, 2, 3 red (`FAIL 1 ... verdict fail, expected ok`, `FAIL 2 ... message lacks`, `FAIL 3 ... verdict ok, expected fail`); `18 rows, 15 passed, 2 wrong verdict, 1 wrong message`; `workflow` 16/0 | matches exactly |
| M10 | append a pad step to `v1_resize_max.toml` | `every_recipe_output_matches_its_golden` (`v1_resize_max x rgb8_2600x1730: dims 880x612, expected 800x532 (expected-dims table)`), `to_toml_is_byte_identical_to_main_over_the_corpus` | matches; reasoned cell confirmed |
| M11 | lenient `orientation_of` | `orientation_of_fails_on_a_malformed_exif_chunk` only | matches |
| M12 | registration line removed | 23 red: the 3 registry rows, `docs_ops::every_op_named_in_data_model_resolves`, all 17 `tests/pad.rs` rows, corpus golden, `to_toml_is_byte_identical_...`. The `pad.rs` unit rows stay green | matches; every reasoned cell confirmed (wasm rows not run) |
| M13 | `Image::from_parts(out, img.source_format(), None)` | `pad_writes_no_orientation_tag_and_keeps_the_bundle` only | matches (reasoned, now driven) |
| M14 | `get_f32("border").map(f64::from)` | none (0 red, exit 0) | matches the "expected all green" cell (reasoned, now driven) |

No cell of the table was wrong. The design's table is not edited.

### Punch list (verify)

Fixes:
- Two rows in `border_is_a_fraction_of_the_long_edge` (now 7): 50×10 at `0.01` → 52×12 at (1, 1) and 5×3 at `0.7` → 13×11 at (4, 4). Checked in Python with `f64`: `0.01 * 50 = 0.5` → 1 px; `0.7 * 5 = 3.5` → 4 px. In `f32` the products are 0.49999999 and 3.49999994, which give 0 and 3.
- `apply_reads_the_border_as_f64` in `tests/pad.rs`: a seeded 50×10 rgb8 PNG, `border = 0.01`, `#ffffff`; exit 0, 52×12, input at (1, 1), frame all `#ffffff`.
- Spec text: file count, M14 row, AC-5, Failing Tests list, `orientation_of` caller count (two).
- `docs/data-model.md:164`: removed the spec id. The line before it needed a comma after `watermark` to keep the sentence a list.

Prove it can fail (each on a fresh `CARGO_TARGET_DIR`, stdout redirected, `$?` read, `git diff` non-empty):
- M14 (`get_f32("border").map(f64::from)`): exit 101, exactly two targets red. `border_is_a_fraction_of_the_long_edge`: `Err("the border rounds to 0 px on this 50×10 px image ('border' = 0.01 of its 50 px long edge); it must add at least 1 px")` against `Ok((52, 12, 1, 1))`. `apply_reads_the_border_as_f64`: exit code `left: Some(1) right: Some(0)`. The two unit rows are one test function, so two tests go red, not three; the second row is not reached after the first fails.
- M4 (`iw.min(ih)`): the new unit rows are red along with the existing set; `border_is_a_fraction_of_the_long_edge` fails on its first row, `left: Ok((46, 36, 3, 3)) right: Ok((48, 38, 4, 4))`; `apply_reads_the_border_as_f64` is red too.
- Restored: full default leg and `--no-default-features` leg, each in a fresh target dir, both exit 0.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — AC-3 says the sweep runs "through `layout`", but `layout` rejects the extreme aspects at the caps, so those cases cannot be swept there. I pulled the canvas formula into its own function and swept that. Otherwise the spec's literals (messages, offsets, fills) were enough to build from without a question.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The area cap is in pixels, so an `Rgba32F` canvas at the cap is 2 GiB against 512 MiB at RGBA8. Reached through float TIFFs only, and out of scope here, but `untrusted-input-hardening` is the constraint it touches.

3. **If you did this task again, what would you do differently?**
   — Start the CI watch with the run id in a variable, not a re-parsed string, and push the corpus files before the docs so the corpus-bless round finishes first. Two harness slips (a zsh unsplit `$F` in the matrix loop, a mis-quoted run id) cost reruns, not code.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Three things. (a) Design driving the mutation table paid: 12 of 14 rows were driven on a
   prototype and build found no wrong cell (SPEC-147's reasoned table had two). Design cost $14.06
   re-measured ($13.61 as first recorded). It would not have caught the one miss by itself: build
   did drive M14 (`f32` read for `border`), got all green, and wrote "matches", and verify refuted
   it anyway. A driven "all green" shows that no test splits the mutant, not that no input does.
   Design should have searched for a splitting input (a product of `border` and the edge that lands
   on exactly k+½ px, such as `0.01` on 50 px) the way it swept the canvas exhaustively, and pinned
   it. (b) "No `f32` split exists in the load range" was reasoned, not searched: trap 17 again, one
   spec after SPEC-145. (c) Two counts were written from memory and both were wrong ("32 files" was
   31, "three `orientation_of` callers" was two); a count in a spec should come from a command and
   name it.

2. **Does any template, constraint, or decision need updating?**
   — Proposed, not applied. (i) A mutation row marked "expected all green" names the search that
   found no splitting input, or is marked "undriven claim"; nothing covers this today (`guidance/`,
   AGENTS and HANDOFF were checked), and trap 17 is the nearest, for prompts and DECs rather than
   mutation tables. (ii) Hand-written CI wait loops: the rule exists ("Never poll CI", in
   `SPEC-151-build.md` and HANDOFF), and the punch-list prompt already adds "don't write your own
   loop"; build still wrote one and it ran 3.5 hours, so the build-prompt boilerplate should carry
   that second sentence. No new rule. (iii) SPEC-147 proposed promoting rule 7 and trap 17 to
   `guidance/` or AGENTS §15 "During verify"; neither has been, both are still only in HANDOFF, and
   both held again (rule 7: the punch list needed no re-verify; trap 17: M14). Re-proposed. No DEC
   changes.

3. **Is there a follow-up spec I should write now before I forget?**
   — Filed one STAGE-058 entry (step 3): pin `resize`'s cap message text, test-only, folded into the
   next spec's build. The 2 GiB `Rgba32F` cap finding is not filed: no backlog entry covers it, and
   it is the maintainer's decision (see the ship report). Nothing else in the evidence names a gap no
   entry covers, so no further entry.
