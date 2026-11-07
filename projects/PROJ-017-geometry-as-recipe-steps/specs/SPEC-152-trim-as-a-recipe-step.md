---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-152
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
  created_at: 2026-11-06

references:
  decisions:
    - DEC-095
    - DEC-102
    - DEC-110
    - DEC-112
    - DEC-113
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - single-image-library
    - every-public-fn-tested
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-141
    - SPEC-142
    - SPEC-145
    - SPEC-151

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-058's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-058's `trim` step: remove a uniform border within a stated tolerance, the inverse of `pad`, and the last of the five ops the `edit` flags spec needs"

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

# SPEC-152: trim as a recipe step

## Context

A recipe can't remove a border today. The registry holds `identity`, `invert`, `resize`,
`auto-orient`, `watermark`, `crop`, `rotate`, `flip` and `pad` (`with_builtins`,
`src/operation/registry.rs:305`), and only `crop` removes pixels, at a box fixed by its keys. A box
that fits one scan or screenshot is wrong for the next. So "cut the white margin off every scan in
this folder", or "undo the frame a `pad` step added", needs another tool (ImageMagick's `-trim`).

This is STAGE-058's `trim` item, PROJ-017 build-order line 6 of 13. `crop` (SPEC-145), `rotate` and
`flip` (SPEC-147) and `pad` (SPEC-151) have shipped. `trim` is `pad`'s inverse and the last of the five
ops that line 8 (`edit` flags) needs.

`trim` is the one geometry op whose output box depends on the pixels, not on its keys. That brings
three questions the others did not meet: what happens when the whole image matches; how a tolerance
written as a number compares against 8-bit, 16-bit, float and grey samples; and where "deterministic"
ends and "smart" begins (the brief keeps content-aware trimming out of this wave).

## Goal

A recipe can say `op = "trim"`, optionally with a `color` and a `tolerance`, and get the input's
smallest sub-rectangle outside which every pixel matches that colour (by default the top-left pixel).
The rectangle is copied exactly, at the input's colour type and bit depth, with the same rule on every
image and every target.

## Frame

### Verdict: GO at [S], one spec, no DEC

Framed 2026-11-06 on `main` at `613bf23` (crustyimg 0.8.2). `trim` is a scan for the bounding box,
then a `crop_imm` of it, which crop already does for all ten `DynamicImage` variants. The one hard
question, the tolerance's scale, has an exact answer that reuses `pad`'s ×257 (driven below), so it
needs no DEC. The verification is broad but mechanical, as `pad`'s was. `cycle` moves to `design`.

### Premise check (release build of `main` in a scratch `CARGO_TARGET_DIR`, deleted after)

- **No `trim` op exists.** `/usr/bin/grep -rn -i trim src/` finds only `.trim()` string calls and two
  comments (`src/metadata/mod.rs:578` "NUL-trimmed", `src/wasm.rs:28` "Trimmed from the wasm build's
  feature set"). The registry registers `pad` at `registry.rs:328` and nothing after it.
- **A recipe with a `trim` step is refused at load.** `version = "2"`, `[[step]] op = "trim"` (with and
  without `color = "#ffffff"`, `tolerance = 8`), `apply --recipe trim.toml -o out.png b8.png`:
  `error: unknown operation 'trim'`, exit 1.
- **There is an integer key kind.** `ParamKind::UInt { min }` (`registry.rs:89`) takes a TOML integer in
  `min..=u32::MAX` and refuses a float rather than truncating it. It has **no `max`**, so a
  `tolerance` capped at 255 needs either a `max` on `UInt` (three existing rows get `max: None`:
  `mod.rs:814`, `:818`, `:1356`) or a constructor check. Design picks; the first gives the table's
  standard message.
- **`parse_color`** (`src/text/mod.rs:51`) reads `RRGGBB`, `#RRGGBB` (alpha 255) and `RRGGBBAA` /
  `#RRGGBBAA` into `[u8; 4]`, with no names. `pad` widens a channel to 16 bits as `(c << 8) | c` (×257,
  `wide`, `pad.rs`) and to float as `c / 255.0` in `f32` (`unit`).
- **The refusal hook's doc comment** (`src/operation/mod.rs:318-330`, rewritten by SPEC-151) already
  says a condition that would give a wrong output fails at `apply`. Nothing in `trim` uses the hook
  (below).

### Driven: ImageMagick's `-trim` and `-fuzz` (7.1.2-34 Q16-HDRI)

Inputs made with Python stdlib (zlib + struct PNG writer), 12×10, content inset by a border of left 1,
top 2, right 3, bottom 1 unless stated:

- `b8.png` (RGB8, white border, red/blue checker content): `-trim` → `8x7+1+2`. Sides trimmed
  independently to the bounding box.
- `b8near.png` (as `b8`, but border pixel (0,5) is `(250,255,255)`, one channel 5 off): no fuzz →
  `9x7+0+2` (the left column is kept for one pixel). `-fuzz 1284`, `1285`, `2%` → `8x7+1+2`. Bisected:
  it trims at **`-fuzz 525`** and not at 524.
- `b8near3.png` (the pixel is `(250,250,250)`, all three channels 5 off): bisected, trims at **`-fuzz
  909`** and not at 908.
- So **`-fuzz` is a colour distance, not a per-channel threshold**, in quantum units (0–65535 at Q16;
  `N%` is of 65535). One channel 1285 off needs 525 and three channels 1285 off need 909: both ≈
  Euclidean distance ÷ √6 (1285/525 = 2.448, 2225.7/909 = 2.449).
- `b16_{8,2056,2057}.png` (RGB16, black border, left border column `(v,v,v)` with `v` = 8, 2056,
  2057): no fuzz → `9x8+0+1` for all three (exact match only). `-fuzz 2056` → `8x7+1+2` for **all
  three**, including 2057, because the distance is scaled. **ImageMagick can't pin a per-channel edge
  at 16 bits.**
- `a0.png` (RGBA8, border alpha 0 with random hidden colours, content opaque green): no fuzz →
  `8x7+1+2`. **ImageMagick matches fully transparent pixels on alpha alone**, whatever colour is under
  them.
- `all.png` (all white): exit 0, stderr `geometry does not contain image 'all.png' @
  warning/attribute.c/GetImageBoundingBox/556`, and the output is a **1×1** image with page
  `12x10-1-1`. It neither refuses nor gives 0×0.
- `none.png` (4×3, every pixel different): `4x3+0+0`, unchanged.
- **Which colour:** `mixed.png` (2 px white top and left, 2 px black bottom and right) → `10x8+0+0`, so
  it trimmed the black and kept the white. `tlodd.png` (2 px white border all round, only the
  top-left pixel red) → `10x8+0+0`: it trimmed the right and bottom against white and kept the top and
  left because of the red corner. **ImageMagick does not trim against the top-left pixel.** It picks
  the background from the edges by its own rule.

**ImageMagick as an oracle:** usable only for `tolerance = 0` (`-fuzz 0` is exact), on an image whose
four corners share the trim colour, at 8 or 16 bits, alpha-0 borders included. It is not an oracle
for any `tolerance > 0` (distance vs per channel), for differing corners, or for the whole-match case.
Design may add an ImageMagick-checked row for the exact cases; the tolerance edges need literals.

### Driven: the tolerance's scale (Python model, the op doesn't exist)

- **8-bit:** `|s − c| <= t`, per channel, inclusive.
- **16-bit: `t × 257`**, so `tolerance = 8` means `0x0808` = 2056: a sample 2056 off matches and 2057
  off does not. `t × 257 == (t << 8) | t` for every `t` in 0..=255 (checked), so it is exactly `pad`'s
  `wide` applied to the tolerance. The alternative, 8 raw 16-bit steps, makes the same recipe trim a
  16-bit scan by about 1/257 as much as its 8-bit copy, so it is rejected.
- **Float: the obvious rule is not exact.** `|s − c/255| <= t/255` in `f64` on an `f32` sample (or
  all in `f32`) puts the edge on `f32` rounding. Searched every `c` and `t` in 0..=255: the `f32` image
  of a value exactly `t` steps away is **rejected** in 18,081 of 32,896 cases with `f64` maths and
  10,668 with all-`f32` maths. A ±3 ulp walk round every edge found 145,255 inputs where the two
  disagree; the first is `c = 0`, `t = 1`, sample `f32(1/255)` = 0.003921568859…
- **Recommended float rule: read a float sample as the nearest 16-bit value**,
  `round(s × 65535)` in `f64` with halves up, then use the 16-bit rule. Searched exhaustively: an
  exactly-`t`-away value is rejected in **0** cases, a `(t+1)`-away value is accepted in **0**, and every
  16-bit value `k` survives `f32(k/65535)` → round → `k` (all 65,536). Out-of-range floats (below 0 or
  above 1) need no clamp, because the difference is taken as a signed integer. `NaN` is design's (an
  `as` cast gives 0).
- **Grey:** a luma sample `L` compares as `(L, L, L)` against the colour, so `#808080` matches it and
  a chromatic colour compares per channel as for RGB. No type change (below).
- **Alpha takes part** as a fourth channel, with one exception (the alpha rule). An image with no alpha
  channel compares as if its alpha were the maximum.
- **Range** 0..=255, an integer, default 0 = exact match. 255 matches everything, which is the
  whole-match failure below, not a special case.
- **The `f32`/`f64` split (SPEC-151's M14) can't happen in the key**: an integer is read exactly.
  It moves to the float comparison, where the rule above closes it. Design pins it with the `c = 0`,
  `t = 1`, `s = f32(1/255)` row: it must match, and a mutant using `f64` `/255` maths fails it.

### Recommended keys (content for design; no DEC)

- **`color`**, optional, read by `parse_color`, with `pad`'s message for a bad value. **Absent means the
  top-left pixel**, compared in the image's own samples (no 8-bit round trip, so a 16-bit corner is
  exact). It differs from `pad`, where `color` is required, because a trim already has a natural colour
  and a pad does not. DEC-102's "refuse rather than guess" isn't broken: the default is a stated rule,
  not a guess.
- **Translucent colours are allowed** (`#RRGGBBAA` with any alpha, unlike `pad`): trimming only
  removes pixels, so the JPEG question that split `pad` doesn't arise. `#00000000` is how a recipe says
  "trim the transparent margin".
- **`tolerance`**, optional integer 0..=255, default 0, in 8-bit steps at every depth (above). A float
  (`8.0`, `8.5`) is refused as the wrong type, as `UInt` refuses it today. Spelling: `tolerance`
  (the backlog's word, and what `--fuzz`-free tools call it). Rejected: `fuzz` (ImageMagick's word for
  a different measure, a distance), `threshold` (reads as a binarisation).
- **Rejected: a fraction** (`tolerance = 0.03`). A fraction of what? It would have to mean the full
  sample range, which is the same rule as 8-bit steps with a worse key, and it reintroduces an `f64`
  read at the edge.
- **Rejected: a colour distance** (ImageMagick's). Per channel is the backlog's word, is exact in
  integers, and a recipe author can predict it from a colour picker.
- **Rejected: `sides`, per-side tolerance, `margin`/`keep`** (leave N px of border). None is asked
  for; each is a key that can come later without changing a default.

### Which corner, which sides, what "uniform" means

- **"Uniform" means one colour, not one width.** Each side is trimmed independently to the bounding
  box of pixels that don't match (ImageMagick's shape). An equal-width rule would fail the commonest
  case, a `pad` to an aspect, whose borders differ by side.
- **The colour is the top-left pixel**, and all four sides trim against it. Rejected: "all four
  corners must agree, else refuse", which refuses a scan with a margin on two sides only; and "each side
  against its own corners", which trims a gradient or a two-coloured frame side by side and is the first
  step toward "smart". A recipe that wants another colour states `color`. The doc entry says: put
  `auto-orient` first, since the top-left is the stored buffer's (DEC-113).
- If the top-left pixel is content (no border there), a trim against it usually trims nothing (a no-op,
  below). That is deterministic and the doc entry says so.

### The alpha rule

**A pixel with alpha 0 matches a colour with alpha 0, whatever its hidden colour.** Otherwise every
channel, alpha included, is compared. Driven on `a0.png`: ImageMagick trims it to `8x7+1+2` on alpha
alone. Under a plain four-channel rule, the default (top-left) colour would be one random hidden colour
and `a0.png` would trim almost nothing, so the rule is needed for the "trim the transparent margin of a
logo" case. It's exact (alpha 0 is a value, not a tolerance), and with `tolerance > 0` a nearly
transparent pixel still compares all four channels, so it stays per channel.

### Where each per-image condition fails, and what a batch gets

| condition | outcome | where | batch gets |
|---|---|---|---|
| nothing to trim (top row, bottom row, left and right column each hold a non-matching pixel) | **no-op**, the input returned unchanged (same type, same bytes), as `pad` already at its aspect | — | exit 0, written |
| the whole image matches | **fails**: a trim would leave 0×0. Message shape: `every pixel of this W×H px image is within 'tolerance' = t of #rrggbb (the top-left pixel); trimming it would leave nothing` | **apply** | that input fails, nothing written for it, the others written, **exit 6**; one input exits 1 |
| a 1×1 image | always the whole-match case (its top-left is itself) unless `color` is stated and differs, which is a no-op | apply | as above |

**Apply, not the refusal hook.** SPEC-145's M4b showed a hook refusal is skipped per image in a
batch: an all-white page in a scan batch would be written untrimmed at exit 0, which is the "skipping
gives a wrong output" case the hook's doc comment (`mod.rs:323-330`) sends to `apply`. The hook would
also scan the image twice. SPEC-151's frame drove the apply path in a batch (crop's 0 px case: both
`operation 'crop' failed` lines, `2 of 2 inputs failed`, exit 6). **So no `trim` condition uses the
hook**, and the entry "The batch summary says 'watermark skipped' for any refusing op" is neither a
prerequisite nor folded. It stays open for the first second op that uses the hook.

### Colour type, bit depth and metadata

- **DEC-095: no case can change either.** The output is `img.pixels().crop_imm(left, top, w, h)` of
  the input, which keeps the variant for all ten types (crop relies on it). No pixel is computed, so
  nothing widens or narrows. A grey image compared against a chromatic colour is widened only in the
  comparison, never in the buffer.
- **DEC-113 covers `trim` as written.** Its rule 1 names `trim` ("and later `pad`, `trim` and
  straighten") and its Consequences say `pad`/`trim` "follow the same rule without a new decision".
  `trim` keeps the bundle with `Image::with_pixels`, as `crop` does. SPEC-151's frame drove the tagged
  input on `crop aspect = "1:1"` (Orientation 6, ExifImageWidth/Height, Artist all gone from both PNG
  and JPEG outputs). `trim` reaches the file through the same `with_pixels` and the same pixel lane, so
  that evidence carries over and isn't re-driven. Design adds the one unit row `pad` has
  (`…_writes_no_orientation_tag_and_keeps_the_bundle`).

### The `pad`-then-`trim` inverse, and its condition

`pad` with fill `F`, then `trim` with `color = F` and the same `tolerance`, gives back the original
box **if and only if each of the original's four outermost lines (top row, bottom row, left column,
right column) holds at least one pixel that does not match `F` within the tolerance**. This holds on
every side, padded or not: an aspect pad adds on two sides only, and trim still eats an unpadded side
whose edge line matches. Driven on a Python model of both ops: 19,764 random cases (1–6 px sides, a
palette that includes near-fill colours, tolerances 0, 4 and 8, pads 0–2 px per side). The condition
predicted the result in all of them, 14,914 recovered and 4,850 not, with **0 mismatches**.

- **The intended failure:** an image whose top row is entirely white, padded with white, trims past the
  pad and loses that row. A test pins it.
- **The default colour needs care:** with no `color`, trim uses the padded image's top-left pixel. That
  is the fill after any `border` pad, but after an aspect pad that adds only right/bottom (gravity
  `north-west`) it is the image's own pixel. The round-trip test states `color`.

### Refusals at load (DEC-102: `recipe step N ('trim'): <reason>`, exit 1, no image read)

Shapes only; design writes the exact text. None names a spec, stage or DEC id.

| bad trim | who checks | message shape |
|---|---|---|
| unknown key: `fuzz`, `threshold`, `sides`, `border`, `margin` | table | `unknown key 'fuzz'; expected one of: color, tolerance` |
| near-miss spelling (trap 10): `colour`, `tolerence` | table | the same unknown-key message |
| `tolerance` out of range: `256`, `-1`, `4294967296` | table (`UInt` with a max) or constructor | `'tolerance' must be an integer in 0..=255, got 256` |
| `tolerance` of the wrong type: `8.0`, `8.5`, `"8"`, `true` | table | `'tolerance' must be an integer …, got the float 8.5` (or `the string "8"`) |
| inexact as `f32` (trap 10): `tolerance = 16777217` | table | out of range (it is above 255); no `f32` read exists for an integer |
| bad colour: `"white"`, `"#fff"`, `"#ff00"`, `"transparent"` | constructor | `pad`'s: `'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "white"` |
| `color` of the wrong type: `color = 0xffffff` | table | `'color' must be a string, got the integer 16777215` |

The only apply failure is the whole-match case: `operation 'trim' failed: …`, exit 1 on one input,
exit 6 in a batch.

### Determinism and the "smart" line (for the doc entry)

**`trim` compares each pixel with one colour, the stated `color` or the top-left pixel, and removes
the outer rows and columns in which every pixel matches. It never looks at what the content is.**
Considered and kept out: ignoring noise or isolated specks (a filter, so content-aware; post-1.0 with
smart crop), "trim to content" or saliency (post-1.0, the brief's out of scope), per-side colour or
tolerance (a later key if asked for), a colour-distance mode (a later `metric` key if asked for), and
keeping a margin (`trim` then `pad border` does it today).

### Exactness, sized for verification

The matrix: 10 colour types (`L8 La8 Rgb8 Rgba8 L16 La16 Rgb16 Rgba16 Rgb32F Rgba32F`) × 2 colour
sources (top-left, stated) × tolerance at 0 and at an edge (`t`, `t+1` on each side) × alpha (alpha-0
rule, alpha in the tolerance) × border shapes × whole-match and no-op × native/wasm. Every assertion
is equality with a sub-rectangle of the input: no pixel is computed.

- **Border shapes:** different widths on each side (1, 2, 3, 1); one side only; 1×1 (whole match, and
  a stated other colour = no-op); a one-pixel-wide survivor (a single non-matching column); odd sizes;
  a border pixel one step outside the tolerance, on each side, so each side's scan is pinned.
- **Scale rows:** 8-bit `t = 8` at 8 and 9 off; 16-bit `t = 8` at 2056 and 2057 off (the backlog's ⚠);
  the float rows at `f32((c ± t)/255)` and one past, including the `c = 0, t = 1` splitting input;
  grey (`L8`, `L16`) against a grey and a chromatic colour.
- **Alpha rows:** an alpha-0 border with mixed hidden colours (the `a0.png` shape), default and
  `#00000000`; a nearly transparent border pixel (alpha 1) with `t = 0` (kept) and `t = 1` (trimmed if
  its colour also matches).
- **The inverse:** pad then trim gives the input back, plus the intended failure row.
- **Fixtures** are seeded in-test buffers made independently of the code under test.
- **Mutations design should drive** (each with its splitting input named, per SPEC-151's ship lesson):
  `t << 8` for `× 257` (16-bit sample 2056 off); `<` for `<=` (any sample exactly `t` off); the float
  rule replaced by `f64` `/ 255` maths (`c = 0`, `t = 1`, `f32(1/255)`); the alpha-0 rule removed
  (`a0` shape); top-left read as bottom-right (`tlodd` shape); one side's scan dropped (the per-side
  one-step-outside rows); the whole-match check removed (a 1×1 image).

### Corpus (SPEC-141) and wasm (DEC-110)

- **A `trim` step alone can't act on the corpus fixtures.** `fixture()` (`tests/recipe_corpus_output.rs:132`)
  adds per-pixel noise (`n` in 0..32 at 8 bits, 0..2048 at 16) to a gradient, so no edge line is
  uniform and a lone trim is a no-op, which the "every acting step acts" control (`:321`) reports. And
  `pad` then `trim` returns the fixture itself, which control (b) (`equals_fixture_line`) reports.
- **So each recipe pads twice and trims once.** Two recipes, both on all three fixtures:
  - `v2_trim_corner`: `pad border = 0.01` `#ffffff`, `pad border = 0.01` `#000000`, `trim` with no
    keys. The top-left is black, so the black frame goes and the white one stays.
  - `v2_trim_tolerance`: `pad border = 0.01` `#ffffff`, `pad border = 0.01` `#080808`, `trim`
    `color = "#000000"`, `tolerance = 8`. On `rgba16_2600x1730` the `#080808` fill is `0x0808` and the
    tolerance is 2056, so the 16-bit scale is pinned at its inclusive edge in the corpus too.
  - Every step acts, the output differs from the fixture (it carries the white frame), and the expected
    dims are the fixture plus `2 × round(0.01 × long edge)` per axis. Six new pixel rows; no existing
    line changes (`git diff --diff-filter=M origin/main -- tests/fixtures/recipe_corpus/` empty). The
    three per-target `.output` files of each should be identical.
- **Wasm:** two `tests/wasm_roundtrip.rs` rows, as `pad` added: a trim transform equal to the expected
  sub-rectangle, and a load refusal plus the whole-match apply failure with the native message.
  `FULL_COUNT` 53 → 55 and `SLIM_COUNT` 51 → 53 in `scripts/wasm-ci-check.sh` (read on `main` at the
  frame), with its self-test fixtures that name 53 and 51 moved in the same PR. Build re-reads both
  counts first in case PROJ-011 has moved them. No ddphotos parity row (the page doesn't trim;
  STAGE-060).

### Sizing: [S], and why

The op is a four-sided scan and a `crop_imm`. The match rule is one function per sample width (8-bit,
16-bit, float read as 16-bit), and `parse_color`, `crop_imm` and `with_pixels` all exist. No DEC:
DEC-112 rule 6 puts `tolerance` outside the units rule in so many words, the ×257 is `pad`'s
precedent, and the float rule is an op-local reading written in the doc entry. The verification is the
matrix above, 2 corpus recipes, 2 wasm rows and about 8 load refusals: the same breadth as `pad`,
which was [S]. One fold-in, test-only. It would grow to [M] if the maintainer wants the tolerance
scale written as a DEC (one more document in design, still no more code) **and** a second colour
rule (per-side corners or a distance mode); neither is recommended.

### Scope

**In:** the `trim` op (optional `color`, optional `tolerance`), its key table and load refusals, the
whole-match apply failure, the no-op, the scale, grey and alpha rules, unit tests, 2 corpus recipes,
2 wasm rows and the count change, the docs entries, and one fold-in:

- **"Pin `resize`'s cap message text"** (folded; test-only, no size change). It asks to be folded into
  the next STAGE-058 build, and its text is already recorded in the entry. A `contains` on both lines
  (`exceed the maximum edge limit of 50000 px`, `exceeds the maximum area of 134217728 px`) in the three
  tests it names.

**Not folded:**
- **"Pin `watermark`'s nine gravities, with a margin"**: it pairs with `trim` only "if `trim` places by
  gravity", and it doesn't (no gravity key). It belongs with the next spec that touches
  `Gravity::placement`.
- **"Recipe-output guard: report the file-set and line-count failures together"**: optional, and worth
  doing only when a spec trips both. Adding two recipes raises both constants on purpose, which is not
  that case.
- **"The batch summary says 'watermark skipped' for any refusing op"**: no `trim` condition uses the
  hook (above).

**Out:** see "Out of scope (for this spec specifically)".

### Rough build file list

- `src/operation/trim.rs` (new): `TRIM_PARAMS`, `Trim::from_params`, the match rule, the bounding-box
  scan, `Operation for Trim`
- `src/operation/mod.rs`: `mod trim` and the export
- `src/operation/registry.rs`: register `"trim"`, the op-list tests, and `UInt`'s `max` if design takes
  that route (with the three existing `UInt` rows in `mod.rs`)
- `tests/trim.rs` (new)
- `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs` (the two stems, `EXPECTED_DIMS`, the
  constants the tests own)
- `tests/fixtures/recipe_corpus/v2_trim_corner.{toml,golden}`, `v2_trim_tolerance.{toml,golden}` and
  three `.output` files each
- `tests/wasm_roundtrip.rs`, `scripts/wasm-ci-check.sh` (`FULL_COUNT`, `SLIM_COUNT`, self-test lines)
- Fold-in: `src/operation/mod.rs` (`resize_oversize_is_typed_error` :2351,
  `resize_apply_percent_rejects_oversized_output` :3565), `tests/apply_batch.rs`
  (`apply_recipe_with_oversized_resize_exits_1` :461)
- Docs (found with `/usr/bin/grep -rn -i "\btrim\b" docs/ recipes/ skills/ README.md integrations/` and
  a grep for the files that list `pad` as an op):
  - `docs/recipes.md:120` "**Trim** **[planned — PROJ-017]**" becomes a `trim` entry marked today, with
    the determinism sentence; `:282`'s PROJ-017 line drops "`trim`" from planned (straighten stays).
  - `recipes/README.md` (the op table, `:96` area), `docs/api-contract.md` (a "Trim in a recipe" para
    after `:476`), `docs/data-model.md` (`:164`'s op list, which `tests/docs_ops.rs` checks, and a
    "`trim` step param keys" section after `:392`), `skills/crustyimg/SKILL.md:236`,
    `integrations/ddphotos/README.md:128` (the list of steps the page doesn't offer),
    `docs/backlog.md:193` (the `trim` row: shipped as a recipe step), `CHANGELOG.md` (Unreleased).
  - Left as history: `docs/research/*`, `docs/sessions/*`, `docs/feature-exploration.md`,
    `docs/video-tool-assessment-2026-08.md` (a video trim). `docs/roadmap.md:36` lists the wave and
    needs no edit.
  - No verb: the set is frozen at 14 (STAGE-030). `trim` is a recipe step; `edit --trim` is line 8.

### Open maintainer questions (each with a recommendation)

1. **The default colour:** top-left pixel, all sides against it? Recommend **yes**. (Alternatives:
   all four corners must agree or refuse; ImageMagick's edge census.)
2. **The tolerance's scale:** 8-bit steps, ×257 at 16 bits, float read as the nearest 16-bit value?
   Recommend **yes**, as an op rule in the doc entry, with no DEC.
3. **The alpha rule:** alpha 0 matches alpha 0 whatever the hidden colour? Recommend **yes**
   (ImageMagick agrees, driven).
4. **"Uniform":** one colour, each side trimmed independently? Recommend **yes** (one width fails
   the aspect-pad inverse).
5. **The whole image matches:** fail at apply (exit 1, a batch exits 6 with the others written)?
   Recommend **yes**, not the hook.
6. **`tolerance` an integer, so `8.0` is refused?** Recommend **yes**: exact, no `f32` read, same
   units as `color`.
7. **`color` optional and translucent allowed, unlike `pad`?** Recommend **yes** for both.

### Out-of-spec findings

None filed. Two things recorded for the doc entry and design, not as gaps: ImageMagick's `-fuzz` is a
scaled colour distance, so anyone checking `trim` against `magick -fuzz N -trim` will see different
edges for `tolerance > 0`; and `NaN` samples in a float TIFF need a stated reading (design).

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

- **`edit --trim`** and the ordering rule: STAGE-058 line 8 ("`edit` flags for the five").
- **`pad` with a transparent fill**: its own STAGE-058 entry (split out of SPEC-151). `trim` accepts a
  translucent `color` because it only removes pixels; that doesn't touch `pad`'s refusal.
- **The ddphotos edit page** offering trim: STAGE-060.
- **Content-aware trimming**: ignoring noise or specks, trim to content, saliency. Post-1.0 with smart
  crop (the brief's "Explicitly out of scope", the roadmap's smart-crop row).
- **Per-side colour or tolerance, a colour-distance mode, a kept margin**: not asked for; each would be
  a new optional key in a later entry, changing no default. A kept margin is `trim` then `pad border`.
- **The batch summary's "watermark skipped" text**: its own STAGE-058 entry; `trim` uses no refusal
  hook.
- **Pinning `watermark`'s nine gravities with a margin** and **reporting the guard's file-set and
  line-count failures together**: their own STAGE-058 entries (not folded; see the Frame's scope).
- **The `Rgba32F` area cap at 2 GiB** (SPEC-151's ship note): unchanged; `trim` allocates no more than
  its input.

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
