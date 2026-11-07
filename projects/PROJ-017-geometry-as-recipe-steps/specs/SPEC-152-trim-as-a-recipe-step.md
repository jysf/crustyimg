---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-152
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
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
  sessions:
    - cycle: frame
      date: 2026-11-06
      interface: claude-code
      model: claude-opus-5-5
      tokens_total: 5170027
      tokens_breakdown:
        input: 82
        cache_creation: 135007
        cache_read: 4993199
        output: 41739
      estimated_usd: 4.38
      note: "Measured from this session's transcript after the push (41 messages, deduped by message.id, max output_tokens per id). Rates: Opus $5/$25 per MTok, cache_creation x1.25, cache_read x0.10. Excludes this final cost commit's own turn." 
    - cycle: design
      date: 2026-11-06
      interface: claude-code
      model: claude-opus-5-5
      tokens_total: 20229426
      tokens_breakdown:
        input: 184
        cache_creation: 304244
        cache_read: 19798023
        output: 126975
      estimated_usd: 14.98
      note: "Measured from this session's transcript after the push (90 messages, deduped by message.id, max output_tokens per id). Rates: Opus $5/$25 per MTok, cache_creation x1.25, cache_read x0.10. Includes a prototype and 21 driven mutation rows (full suite each). Excludes this final cost commit's own turn."
    - cycle: build
      date: 2026-11-07
      interface: claude-code
      model: claude-sonnet-5-5
      tokens_total: 23926026
      tokens_breakdown:
        input: 220
        cache_creation: 270388
        cache_read: 23555280
        output: 100138
      estimated_usd: 9.58
      note: "Measured from this session's transcript at the last turn before the cost commit (104 messages, deduped by message.id, max output_tokens per id). Rates: Sonnet $3/$15 per MTok, cache_creation x1.25, cache_read x0.10. Excludes the final bookkeeping turn."
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

## Design

Designed 2026-11-06 on `main` at `b05d630` (crustyimg 0.8.2), against the maintainer's seven rulings of
2026-11-06. **No DEC.** One op, one spec, **[S]** in code (about 200 lines in `trim.rs`); the cost is
verification, as for `pad`. Two things are added to the frame's scope, both small and both on `trim`'s
own path (corrections 6 and 7).

### Design corrections to the frame

1. **The inverse condition misses colour type.** `pad` then `trim` gives back the input's box and
   pixels under the frame's condition, but **not its colour type when a grey image is padded with a
   colour that is not grey**: `pad` widens it to RGB (its DEC-095 rule) and `trim` never narrows.
   Driven on the prototype: `L8` 7×5 → `pad border = 0.3, color = "#123456"` → `trim color =
   "#123456"` gives back the 7×5 box as `Rgb8`, every pixel `(l, l, l)`. The full condition: **the box
   and samples come back if each of the input's four outermost lines holds a pixel that doesn't match
   the fill within the tolerance; the colour type comes back too unless `pad` widened it.** AC-45 pins
   both, and the doc entry says it.
2. **The frame's corpus dims formula is right by accident.** "Fixture plus `2 × round(0.01 × long
   edge)`" describes the result, but the second `pad` sees the padded image: 2600 → 26 px (2652×1782),
   then 2652 → 27 px (2706×1836), and `trim` removes the 27 px frame. The output is the first pad's
   size, 2652×1782, which is what the formula gives. Driven: the blessed prototype outputs are
   `2652x1782` / `1782x2652` / `2652x1782` for both recipes.
3. **Both corpus recipes produce the same pixels** (each removes its inner frame and keeps the same
   26 px white one; driven, identical `sha256` on all three fixtures). So the corpus pins `trim`'s box
   by dims and its copy by hash, but the two recipes don't pin different pixels. That is fine for a
   guard (each recipe pins a different rule: the corner and the 16-bit edge), and M16 is the
   mutation that turns them red **by hash** (trap 13).
4. **Out-of-range floats are clamped, not left signed.** Ruling 2 says a float sample "is read as its
   nearest 16-bit value", and a 16-bit value lies in 0..=65535, so the reading is
   `round(s × 65535)` clamped to 0..=65535: `1.5` reads as 65535 (white), `-0.25` as 0. The frame's
   "no clamp needed, the difference is signed" would make `1.5` match nothing near white, which is not
   "the nearest 16-bit value". **NaN** (left to design): **a NaN sample never matches**, in the image
   or in the top-left reference. A NaN pixel is kept (trim stops at it), and an image whose top-left
   has a NaN sample and no `color` trims nothing (a no-op, not the whole-match failure). Reason: a
   NaN is not a colour, and reading it as 0 (what an `as` cast does) would trim it as black. Driven:
   AC-11's rows; M17 and M18 are the mutations.
5. **The whole-match message prints no hex for the default.** The frame's shape printed `#rrggbb`,
   which can't be true for a 16-bit or float top-left without rounding. The message names the source
   instead: `its top-left pixel` when `color` is absent, and `'color' = "<as written>"` when stated
   (the author's own text, true at every depth).
6. **Fold-in: `parse_color` panics on a non-ASCII colour (found here, unfiled).** It checks the byte
   length, then slices `hex[i*2..i*2+2]`, so a 6-byte string with a multi-byte character panics.
   Driven on the 0.8.2 release binary: `color = "aéaaa"` in a `pad` step, in a `watermark` step and as
   `watermark --color` each print `panicked at src/text/mod.rs:60:49: end byte index 2 is not a char
   boundary`, **exit 101**. `trim`'s `color` goes through the same function, so `trim` can't meet
   `untrusted-input-hardening` without the fix. It is one check (`!hex.is_ascii()` → the existing
   error) and three rows (AC-12). It changes `pad`'s and `watermark`'s behaviour only from a panic to
   their existing message. **If the orchestrator wants it split out instead, AC-12 and M13 move to a
   new STAGE-058 entry and `trim`'s refusal row T23 is dropped (correction 6); nothing else changes.**
7. **`ParamKind::UInt` gains `max`** (the design call the frame left open; below). That is a change
   to a public enum, recorded as a library break in the CHANGELOG.
8. **SPEC-150 shares less than the design prompt said.** Its spec (`Files the build touches`) and its
   PR #257 change neither `src/operation/**` nor `tests/wasm_roundtrip.rs`; the shared files are
   `docs/api-contract.md` and `src/wasm.rs` (which `trim` does not touch). Details under
   "Files shared with SPEC-150".

### What was driven at design

All on a release build of `b05d630` in a scratch `CARGO_TARGET_DIR` (deleted after), plus a throwaway
prototype of `trim`, its tests, the corpus rows and the script change in a scratch `git worktree`
(never committed or pushed, removed after).

- **Today:** `[[step]] op = "trim"` → `error: unknown operation 'trim'`, exit 1.
- **crop's 0 px stand-in** (`x = 0, y = 0, width = 0.001, height = 1` on a 40×30 PNG): one input →
  `error: operation 'crop' failed: the crop rounds to 0×30 px on this 40×30 px image; it must keep at
  least 1 px each way`, exit 1, no file. Batch with a 4000×30 second input, `--out-dir`: `error: a.png:
  operation 'crop' failed: …`, `error: 1 of 2 inputs failed`, exit 6, only `b.png` written.
- **The fold-in's lines**, on the binary: `resize --exact 60000x10` → `error: operation 'resize'
  failed: target dimensions 60000×10 exceed the maximum edge limit of 50000 px`; `--exact 20000x20000`
  → `… target area 20000×20000 = 400000000 px exceeds the maximum area of 134217728 px`; the
  `apply_batch` recipe (`exact` 50000×50000) → `… target area 50000×50000 = 2500000000 px exceeds the
  maximum area of 134217728 px`; `percent = 2000000` on 4×4 → `… target dimensions 80000×80000
  exceed …`. All exit 1.
- **`parse_color`'s panic** (correction 6), and its control `color = "#zzéaa"` (7 bytes, so the length
  check refuses it first): the hex message, exit 1.
- **The float rule, in Rust** (path: `f32` sample → `f64::from` → `× 65535.0` → `.round()` →
  `.clamp(0.0, 65535.0)` → `as i64`; the product of an `f32` and 65535 is exact in `f64`, so the only
  rounding is the final `round`). Every `c`, `t` in 0..=255 and both signs: the `f32` value exactly `t`
  steps off, built two ways (`(c ± t) as f32 / 255.0` and `((c ± t) as f64 / 255.0) as f32`), and every
  value within 3 ulp of each (`next_up`/`next_down`), must match; the value one 8-bit step past and the
  value one 16-bit level past (`(c·257 ± (t·257 + 1)) / 65535`), each with ±3 ulp, must not; and every
  16-bit level `k` survives `f32(k / 65535)` → level → `k` both ways. **2,357,504 cases, 0 wrong.**
  The frame's Python result holds. The splitting input of the `f64 /255` rule is confirmed:
  `f32(1/255) − 1/255 > 0` in `f64`, and the rule reads it as level 257 (AC-2, AC-3, M3).
- **The prototype** (op, refusal table, match rows, 210-case exactness matrix, scan rows, alpha, NaN,
  inverse, binary rows, metadata, fold-in, corpus with the literals, script): 15 integration tests
  and 3 unit tests green; the full `cargo test --release --no-fail-fast` green (59 test binaries); every
  mutation row below was applied to it.
- **The inverse through the real `pad`**: AC-45's cases through `Pipeline::run` with the shipped
  `pad` code: 0 differing samples; the white-top-row failure and the widened grey case as stated.

### The op, in code terms

- **Where:** a new `src/operation/trim.rs` (`Trim`, `TRIM_PARAMS`), declared in `src/operation/mod.rs`
  after `pub mod rotate;` (`:35-36`) as `pub mod trim;` and `pub use trim::{Trim, TRIM_PARAMS};`.
- **Table** (`TRIM_PARAMS`, in this order, which is the order the unknown-key message lists):
  - `color`: `ParamKind::Text`
  - `tolerance`: `ParamKind::UInt { min: 0, max: Some(255) }`
- **Registration** in `with_builtins` (`src/operation/registry.rs:306-328`), after `pad`:
  `reg.register_with_params("trim", |p| Ok(Box::new(Trim::from_params(p)?)), TRIM_PARAMS);`, `Trim`
  and `TRIM_PARAMS` added to the `use super::{…}` at `:16-20`, and `trim` added to the built-in lists
  in the doc comments at `:276-278` and `:304-305` ("`flip`, `pad` and `trim`").
- **`Trim::from_params(&OperationParams) -> Result<Trim, RegistryError>`**, never panics. Runs
  `registry::check_params("trim", TRIM_PARAMS, params)` first (as `pad.rs:96` does), then:
  1. `color` present → `crate::text::parse_color`; an error → pad's message (`pad.rs:157-161`, the text at `:159`):
     `'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got {color:?}`. Any alpha is accepted
     (ruling 7), and there is no `transparent` keyword (it gets the hex message, row T19).
  2. `tolerance`: `params.get_u32("tolerance")`, absent → 0. The table has already limited it to
     0..=255; the `u8::try_from` that follows maps a failure to
     `internal: 'tolerance' = {t} passed the key table` (a **different** text from the table's, so M15,
     the table's `max` ignored, is red rather than silently caught by an identical fallback; trap 9).
  `Trim` holds `color: Option<[u8; 4]>`, `tolerance: u8` and the step's map as written; `params()`
  returns the map.
- **The match rule** (a private `fn` per image, built once per `apply`):
  - Every sample becomes an integer **level**: 8-bit `s`; 16-bit `s`; float `round(f64::from(s) ×
    65535.0)` clamped to 0..=65535, or **none** for NaN. One 8-bit **step** is 1 level at 8 bits and
    **257** at 16 bits and float.
  - A pixel is four levels `(r, g, b, a)`: grey `L` is `(L, L, L)`; a type with no alpha has alpha at
    its maximum (255, or 65535 for 16-bit and float).
  - The **reference**: a stated `color` `[r, g, b, a]` is `c × step` per channel (so `#ffffff` is
    65535 at 16 bits and for float); with no `color`, the top-left pixel's own levels, so a 16-bit or
    float corner is used exactly, with no 8-bit round trip.
  - **Tolerance** in levels: `t × step` (8 → 2056 at 16 bits).
  - **A pixel matches** when (alpha rule) its alpha level and the reference's are both 0; otherwise
    when every one of the four channels has a level and `|level − reference| <= tolerance`. A NaN
    (no level) on either side fails that channel.
- **The scan:** `top` = the first row holding a pixel that doesn't match; none → the whole-match
  failure. `bottom` = the last such row (searched from the bottom). `left` = the first column holding
  a non-matching pixel among rows `top..=bottom`; `right` = the last. Each side is its own search. If
  `(left, top, right, bottom)` is `(0, 0, w − 1, h − 1)`, return `Ok(img)` (the no-op: same `Image`,
  same type and bytes). Otherwise
  `img.pixels().crop_imm(left, top, right − left + 1, bottom − top + 1)` and
  `Ok(img.with_pixels(out))`, which keeps the variant (all ten) and the metadata bundle (DEC-113).
- `DynamicImage` is `#[non_exhaustive]`: an unknown variant is the apply error
  `this image's colour type cannot be trimmed`, never a panic.
- **No `Operation::refusal`** (ruling 5). The module doc ends: "When every pixel matches, `apply`
  fails rather than refusing, because skipping the step would write an untrimmed image."
- **`unwrap`/`expect`:** none outside tests. The scan's `bottom`/`left`/`right` searches can't come
  up empty once `top` is found; write them with `unwrap_or` (as the prototype did), not `expect`.

### `ParamKind::UInt` gains `max` (design call)

**Decision: add `max: Option<u32>` to `UInt`**, not a constructor check.

- **Consumers found** (`/usr/bin/grep -rn "UInt\|ParamKind\|param_specs" src tests integrations demo
  scripts`): `registry.rs` itself (`expectation` `:121`, `check` `:149`, `check_params` `:248-258`);
  three `UInt` rows in `src/operation/mod.rs` (`resize` `width` `:814`, `height` `:818`, `watermark`
  `margin` `:1356`); `src/cli/ops.rs:1354-1366` (reads `WATERMARK_PARAMS` `Number` rows only);
  `tests/registry_seam.rs:118-152` (an outside-crate op table, `Asset` and `Number` only). **There is
  no wasm schema export** (`src/wasm.rs` exports `info`, `transform`, `optimize` and two getter
  structs) and **ddphotos reads no `ParamKind`** (it is JS). The `validate()` in `src/build/mod.rs:249`
  checks a build manifest, not step keys. crates.io lists **0 reverse dependencies** of `crustyimg`.
- **Why `max`:** the enum's own doc (`registry.rs:75-77`, "Every rule here depends only on the value itself") puts every rule that depends only on the
  value in the table. With a constructor check, one key would have three wordings, and one of them
  false: `-1` → `'tolerance' must be an integer >= 0, got -1` (table), `4294967296` → `… must be an
  integer >= 0, got 4294967296` (table, and untrue as written), `256` → the constructor's. With `max`,
  every out-of-range and wrong-type value gets one true line.
- **The cost:** `ParamKind` is public and exhaustive, so code outside the crate that builds
  `ParamKind::UInt { min }` must add `max: None`: a library break, written in the CHANGELOG under
  `### Breaking` (AC-61). It needs no version decision of its own: RELEASING.md says a patch release is
  bug fixes only, so the new ops already make the next release a minor one.
- **Shape:** `UInt { min: u32, max: Option<u32> }` with the doc line "The largest accepted value, if
  below `u32::MAX`." `expectation()`: `max: None` → `an integer >= {min}` (unchanged), `max:
  Some(max)` → `an integer in {min}..={max}`. `check()`: `*i >= i64::from(*min) && *i <=
  i64::from(max.unwrap_or(u32::MAX))`. The three existing rows get `max: None`; their messages are
  unchanged (AC-5).

### The match rule, as a table a test can check

Each row: a 2×1 buffer, pixel (0, 0) the top-left, pixel (1, 0) the sample; "match" is whether (1, 0)
matches. `#000000` stated, `t = 8` unless given.

- `Rgb8`, stated `#000000`, `t = 8`: sample `(8, 0, 0)` matches; `(9, 0, 0)` does not.
- `Rgb8`, `t = 0`, stated `#102030`: `(16, 32, 48)` matches; `(16, 32, 49)` does not.
- `Rgb16`, stated `#000000`, `t = 8`: `(2056, 0, 0)` matches; `(2057, 0, 0)` does not.
- `Rgb16`, stated `#ffffff`, `t = 0`: `(65535, 65535, 65535)` matches; `(65534, 65535, 65535)` does not
  (`#ff` is 65535, not 65280).
- `Rgb16`, default, `t = 0`, top-left `(4660, 22136, 39612)`: the same samples match; `(4661, …)`
  does not (no 8-bit round trip).
- `Rgb32F`, stated `#000000`, `t = 1`: `f32(1/255)` = `1.0f32 / 255.0` in channel 0 matches (the
  splitting input); `f32(2/255)` does not.
- `Rgb32F`, stated `#ffffff`, `t = 8`: `(f64(63479) / 65535) as f32` (2056 levels off) matches;
  `(f64(63478) / 65535) as f32` (2057) does not.
- `Rgba32F`, default, top-left `(0.25, 0.5, 0.75, 1.0)`, `t = 0`: the same `f32`s match;
  `0.25f32.next_up()` in channel 0 also matches (same level); `0.25 + 1/65535` as `f32` does not.
- `L8`, stated `#808080`, `t = 0`: `L = 128` matches. `L8`, stated `#800000`: `L = 128` does **not**
  (it compares as `(128, 128, 128)`).
- `L16`, stated `#808080`: `L = 32896` matches; stated `#800000`: does not.
- `La8`, stated `#80808080`, `t = 0`: `(128, 128)` matches; `(128, 255)` does not (alpha compared).
- `La16`, default, top-left `(1000, 0)`: `(54321, 0)` matches (alpha rule).
- `Rgba8`, stated `#00000000`, `t = 0`: `(200, 10, 90, 0)` matches (alpha rule); `(0, 0, 0, 1)` does
  not.
- `Rgba8`, stated `#00000000`, `t = 8`: `(200, 10, 90, 3)` does **not** match (alpha 3 is not 0, so
  all four channels are compared, and 200 is 200 off); `(5, 5, 5, 3)` matches (every channel within 8).
- `Rgba8`, stated `#000000` (alpha 255): `(0, 0, 0, 0)` does not match (alpha 255 off).
- `Rgba16`, default, top-left `(9, 9, 9, 0)`: `(60000, 1, 2, 0)` matches (alpha rule on the reference
  too).
- `Rgb32F`: a NaN in channel 0 does not match `#ffffff` at `t = 255`; `(1.5, 1.0, 7.0)` matches
  `#ffffff` at `t = 0` (clamped to 65535); `(-0.25, 0.0, -9.0)` matches `#000000` at `t = 0`.
- `Rgba32F`, stated `#00000000`: alpha `-0.0` and `1e-6` (level 0) match by the alpha rule; alpha
  `8e-6` (level 1) does not.

That is every one of the 10 colour types with the default and a stated colour, `t = 0` and an edge
(`t` and one past), the 16-bit 2056/2057 pair, the float splitting input, grey against a grey and a
chromatic colour, and the alpha rows.

### Exact messages

**Load.** Each is a `RecipeError::InvalidOperation`, displayed `recipe step 0 ('trim'): <reason>`
(`src/recipe/mod.rs:124`), exit 1 on `apply`, nothing read or written. Table rows come from
`check_params` (`registry.rs:238-263`), which walks keys in sorted order; every row has one fault.
Matched: unknown key `registry.rs:245`; type and range `:248-258` with `describe_value` `:213-225`;
the `UInt` expectation after this spec; the colour message `pad.rs:157-161` (the same text as
watermark's at `src/operation/mod.rs:1683-1687`). Every body follows `op = "trim"`.

- T1 `fuzz = 8` → `unknown key 'fuzz'; expected one of: color, tolerance`
- T2 `threshold = 8` → `unknown key 'threshold'; expected one of: color, tolerance`
- T3 `sides = "all"` → `unknown key 'sides'; expected one of: color, tolerance`
- T4 `border = 0.01` → `unknown key 'border'; expected one of: color, tolerance`
- T5 `margin = 2` → `unknown key 'margin'; expected one of: color, tolerance`
- T6 `colour = "#ffffff"` (trap 10) → `unknown key 'colour'; expected one of: color, tolerance`
- T7 `tolerence = 8` (trap 10) → `unknown key 'tolerence'; expected one of: color, tolerance`
- T8 `tolerance = 256` → `'tolerance' must be an integer in 0..=255, got 256`
- T9 `tolerance = -1` → `… got -1`
- T10 `tolerance = 4294967296` → `… got 4294967296`
- T11 `tolerance = 16777217` (trap 10: inexact as `f32`, printed exactly) → `… got 16777217`
- T12 `tolerance = 8.0` → `'tolerance' must be an integer in 0..=255, got the float 8.0`
- T13 `tolerance = 8.5` → `… got the float 8.5`
- T14 `tolerance = "8"` → `… got the string "8"`
- T15 `tolerance = true` → `… got true`
- T16 `color = "white"` → `'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "white"`
- T17 `color = "#fff"` → as T16, `got "#fff"`
- T18 `color = "#ff00"` → as T16, `got "#ff00"`
- T19 `color = "transparent"` → as T16, `got "transparent"` (no keyword; write `#00000000`)
- T20 `color = ""` → as T16, `got ""`
- T21 `color = 0xffffff` → `'color' must be a string, got the integer 16777215`
- T22 `color = "#gggggg"` → as T16, `got "#gggggg"`
- T23 `color = "aéaaa"` (correction 6) → as T16, `got "aéaaa"`

Rows T9–T15 elide `'tolerance' must be an integer in 0..=255, ` here only; the test holds every
message in full. **Accepted, as controls:** no keys; `tolerance = 0`; `tolerance = 255`;
`color = "#00000000"`; `color = "FFFFFF"`; `color = "#123456ff"`; `color = "#ffffff"` with
`tolerance = 8`.

**Apply.** `operation 'trim' failed: <reason>` (`src/operation/mod.rs:206`); exit 1 on one input; in a
batch that input fails, nothing is written for it, the others are written, and the run exits 6 with
`error: K of N inputs failed`. wasm `transform` throws the same line.

- W1 (no `color`): `every pixel of this {w}×{h} px image matches its top-left pixel within
  'tolerance' = {t}, so a trim would leave nothing`
- W2 (`color` stated): `every pixel of this {w}×{h} px image matches 'color' = {color:?} within
  'tolerance' = {t}, so a trim would leave nothing` (the colour as written, case kept)

`{t}` is the tolerance read (0 when absent). Examples the tests hold: a 1×1 `Rgb16` with no keys →
`every pixel of this 1×1 px image matches its top-left pixel within 'tolerance' = 0, so a trim would
leave nothing`; a 3×2 `Rgb8` of `(250, 255, 255)` with `color = "#FFFFFF"`, `tolerance = 5` → `every
pixel of this 3×2 px image matches 'color' = "#FFFFFF" within 'tolerance' = 5, so a trim would leave
nothing`. **Why this wording:** it names where the colour came from and never prints a colour the
image doesn't hold; a 16-bit or float top-left has no exact `#rrggbb`.

### Sizing: [S] in code, broad in verification

About 200 lines of op code in one file, a two-line `UInt` change with three `max: None` rows, a one-line
`parse_color` check, one registration. The verification: 23 refusal rows, a 210-case exactness matrix,
about 30 match-rule rows, 2 corpus recipes, 2 wasm tests with the count move, 9 doc surfaces, the
fold-in. The same shape as SPEC-151 ([S], 83 ACs). The prototype was about 600 lines with its tests.

## Inputs

- **Files to read:** `src/operation/pad.rs` (the shape to mirror: table `:41-62`, `from_params`
  `:95-171`, the colour message `:157-161`, `wide` `:267-269`, the no-op `:292-296`, unit tests);
  `src/operation/crop.rs` (`apply` and the 0 px failure `:324-372`, `crop_imm` `:370`);
  `src/operation/registry.rs` (`ParamKind` `:78-106`, `expectation` `:115-134`, `check` `:136-188`,
  `describe_value` `:213-225`, `check_params` `:238-263`, `with_builtins` `:306-328`, tests);
  `src/operation/mod.rs` (`OperationParams::get_u32` `:115`, the `Operation` trait and `refusal`
  `:303-333`, the three `UInt` rows `:814`, `:818`, `:1356`, resize's caps `:1055-1075`, the fold-in
  tests `:2351`, `:3565`); `src/text/mod.rs:45-73` (`parse_color`); `src/image/mod.rs:421`
  (`with_pixels`); `src/wasm.rs:205-262` (`transform`); `tests/pad.rs` (helpers `:36-230`, binary
  helpers `:1068-1100`, `tagged_png` `:1317`, the metadata row `:1341`); `tests/common/mod.rs`
  (`tiff_with_orientation`, `orientation_of`); `tests/recipe_corpus_output.rs` (`RECIPES` `:32-53`,
  `ROWS` `:58`, `GOLDEN_LINES` `:62`, `EXPECTED_DIMS` `:66`, `fixture` `:132`, controls `:321-385`,
  `:737`); `tests/recipe_round_trip.rs:257-340`; `tests/fixtures/recipe_corpus/README.md`;
  `tests/wasm_roundtrip.rs:364-372` (`png_unique`), `:543-622` (pad's two rows);
  `scripts/wasm-ci-check.sh:18-19`, `:204-225`; `tests/apply_batch.rs:461-500`; `tests/docs_ops.rs`.
- **Decisions:** DEC-095, DEC-102, DEC-110, DEC-112, DEC-113, DEC-003.
- **Related code paths:** `src/operation/`, `src/text/mod.rs`, `tests/`, `scripts/`, `docs/`,
  `recipes/README.md`, `skills/`, `integrations/ddphotos/README.md`, `CHANGELOG.md`.

## Outputs

- **Files created:** `src/operation/trim.rs` (the op and its unit tests); `tests/trim.rs` (load
  refusals, exactness, scan, grey, alpha, float, inverse, binary rows, metadata, the non-ASCII rows);
  `tests/fixtures/recipe_corpus/v2_trim_corner.{toml,golden,linux-x86_64.output,macos-aarch64.output,windows-x86_64.output}`
  and the same five for `v2_trim_tolerance`.
- **Files modified:** `src/operation/mod.rs` (`pub mod trim`, the re-export, `max: None` on the three
  `UInt` rows, the fold-in asserts in two tests); `src/operation/registry.rs` (`UInt`'s `max`, one
  registration, doc lists, tests); `src/text/mod.rs` (the ASCII check and a unit row);
  `tests/apply_batch.rs` (the fold-in assert); `tests/recipe_corpus_output.rs` and
  `tests/recipe_round_trip.rs` (stems and literals); `tests/wasm_roundtrip.rs` (two tests);
  `scripts/wasm-ci-check.sh` (two constants and the self-test fixtures); `docs/recipes.md`,
  `recipes/README.md`, `docs/data-model.md`, `docs/api-contract.md`, `skills/crustyimg/SKILL.md`,
  `integrations/ddphotos/README.md`, `docs/backlog.md`, `CHANGELOG.md`.
- **New exports:** `crustyimg::operation::{Trim, TRIM_PARAMS}`, with
  `pub fn Trim::from_params(&OperationParams) -> Result<Trim, RegistryError>` and its `Operation` impl.
  **Changed export:** `ParamKind::UInt { min: u32, max: Option<u32> }`.
- **Database changes:** none. No `CACHE_SCHEMA_VERSION` change, no lockfile change, no version bump.

## Acceptance Criteria

**The op**
- [ ] **AC-1** `OperationRegistry::with_builtins()` registers `trim`. `param_specs("trim")` is exactly
  `[color: Text, tolerance: UInt { min: 0, max: Some(255) }]`, in that order. The nine existing ops keep
  their tables, except that `resize`'s `width` and `height` and `watermark`'s `margin` read
  `UInt { min: 1, max: None }`, `UInt { min: 1, max: None }` and `UInt { min: 0, max: None }`.
- [ ] **AC-2** **The match rule, row by row.** Each of the 36 rows of "The match rule, as a table a test
  can check" (2×1 buffers built in the test; the top-left at (0, 0), the sample at (1, 0)) gives the
  stated match / no match, and the test asserts 36 rows ran.
- [ ] **AC-3** **The float rule is exact.** For every `c`, `t` in 0..=255 and both signs: the `f32`
  value `t` steps off (built as `(c ± t) as f32 / 255.0` and as `((c ± t) as f64 / 255.0) as f32`) and
  every value within 3 ulp of it (`next_up`/`next_down`) match `c` at `t`; the value one 8-bit step
  past and the value one 16-bit level past (`((c·257 ± (t·257 + 1)) as f64 / 65535.0) as f32`), each
  ±3 ulp, do not; every level `k` in 0..=65535 reads back as `k` from both `(k as f64 / 65535.0) as f32`
  and `k as f32 / 65535.0`. The test asserts **2,357,504** cases and 0 wrong, and that `1.0f32 / 255.0`
  reads as level 257.
- [ ] **AC-4** `params()` returns the step's map as written: an absent `tolerance` stays absent,
  `tolerance = 8` stays the integer 8, `color = "#FFFFFF"` keeps its case.
- [ ] **AC-5** **`UInt`'s `max`.** `ParamKind::UInt { min: 0, max: Some(255) }.expectation()` is
  `an integer in 0..=255`; `UInt { min: 1, max: None }` is `an integer >= 1`. Through the registry,
  `resize` with `mode = "exact"`, `width = 0`, `height = 10` is still refused with
  `'width' must be an integer >= 1, got 0`, and `watermark` `margin = -1` with
  `'margin' must be an integer >= 0, got -1` (both unchanged from `main`).
- [ ] **AC-6** **Exactness in every colour type.** For each of the 10 colour types × the colour sources
  {top-left (no `color`), `color = "#ffffff"`} × the shapes {4×3 content in a border (1, 2, 3, 1); 3×3 in
  (0, 0, 2, 0), stated colour only, since its top-left is content; 1×3 in (2, 1, 2, 1), a one-pixel-wide
  survivor; 5×7 in (3, 0, 1, 2)} × {`t = 0` on an exact white border; `t = 8` with the border pixel at
  (w−1, h−1) exactly 8 steps below white (8·257 levels at 8 bits, 2056 at 16 bits and float); `t = 8`
  with that pixel one past (9·257, or 2057)} (**210 cases**, seeded buffers): the output has the
  input's colour type and equals the input's sub-rectangle sample for sample (`f32` by bit pattern),
  at `(l, t, cw, ch)` for the first two and `(l, t, w − l, h − t)` for the third. The boxes are
  literals in the test; the content is kept below level 30000 so it never matches white. The test
  asserts 210 cases ran.
- [ ] **AC-7** **Each side on its own.** On a 7×6 `Rgb8` with 3×2 content and a 2 px `#808080` border,
  `color = "#808080"`, `tolerance = 8`, one border pixel 9 steps off in channel 0: at (0, 3) the box
  is `(0, 2, 5, 2)`; at (3, 0) `(2, 0, 3, 4)`; at (6, 3) `(2, 2, 5, 2)`; at (3, 5) `(2, 2, 3, 4)`. The
  same four with the pixel 8 steps off each give `(2, 2, 3, 2)`.
- [ ] **AC-8** **Odd shapes.** A 5×5 white `Rgb8` with one other pixel at (2, 2) → that pixel alone,
  `(2, 2, 1, 1)`; a 1×5 white column with one other pixel at y = 3 → `(0, 3, 1, 1)`; a 1×1 `(9, 9, 9)`
  with `color = "#ffffff"` → returned unchanged (a no-op).
- [ ] **AC-9** **Grey compares as three equal channels.** On a 3×3 `L8` and `L16` (centre `L = 16` /
  4112, border 128 / 32896): `color = "#808080"` → `(1, 1, 1, 1)`, still `L8`/`L16`; `color = "#800000"` →
  the input unchanged.
- [ ] **AC-10** **The alpha rule.** A 12×10 buffer, content opaque green at `(1, 2, 8, 7)`, every
  border pixel alpha 0 with a different seeded hidden colour, on `Rgba8`, `La16` and `Rgba32F`: no
  `color` and `color = "#00000000"` each give `(1, 2, 8, 7)` (6 cases). On a 3×3 `Rgba8` with a
  transparent border, an opaque centre and a corner pixel at (2, 2), `color = "#00000000"`,
  `tolerance = 8`: corner `(200, 10, 90, 3)` → `(1, 1, 2, 2)` (kept); `(5, 5, 5, 3)` → `(1, 1, 1, 1)`.
- [ ] **AC-11** **NaN and out-of-range floats.** A 3×3 `Rgb32F` with a white edge, centre
  `(0.25, 0.5, 0.75)` and a corner at (2, 2): corner `(NaN, 1, 1)` with `color = "#ffffff"` →
  `(1, 1, 2, 2)`; the same at `tolerance = 255` → `(2, 2, 1, 1)`; an all-NaN image with no keys → the
  input unchanged (`Ok`, no error); an edge of `(1.5, 1.0, 7.0)` with `color = "#ffffff"` →
  `(1, 1, 1, 1)`; an edge of `(-0.25, 0.0, -9.0)` with `color = "#000000"` → `(1, 1, 1, 1)`.
- [ ] **AC-12** **A non-ASCII colour is refused, not a panic** (correction 6). `parse_color("aéaaa")` and
  `parse_color("#éééé")` are `Err(TextError::Color(_))`; a recipe step with `color = "aéaaa"` is refused
  at load for `trim` (T23), `pad` (`border = 0.1`) and `watermark` (`text = "hi"`), each message
  containing `got "aéaaa"`; on the binary, `watermark --text hi --color "aéaaa" in.png -o out.png` exits
  **2** with exactly `error: invalid color 'aéaaa': expected RRGGBB, #RRGGBB, or RRGGBBAA`.
- [ ] **AC-13** **No-op.** A seeded `Rgb8`, `Rgba16` and `Rgba32F` 4×3 buffer with no border (its
  top-left is content) returns the same bytes and variant. On the binary, `apply` with `op = "trim"` on
  a seeded 4×3 PNG writes a 4×3 PNG whose decoded samples equal the input's, exit 0.
- [ ] **AC-14** **The whole-match failure, through the pipeline.** A 1×1 `Rgb16` `(7, 8, 9)` with no
  keys fails with exactly `operation 'trim' failed: every pixel of this 1×1 px image matches its
  top-left pixel within 'tolerance' = 0, so a trim would leave nothing`; a 3×2 `Rgb8` of
  `(250, 255, 255)` with `color = "#FFFFFF"`, `tolerance = 5` fails with `… every pixel of this 3×2 px
  image matches 'color' = "#FFFFFF" within 'tolerance' = 5, so a trim would leave nothing`; a seeded
  4×3 `Rgb8` with `tolerance = 255` fails with W1 at `4×3` and `'tolerance' = 255`.
- [ ] **AC-15** **No refusal hook.** `refusal(&img)` is `None` for a 1×1, a uniform 3×2 and a seeded
  4×3 `Rgb8` buffer, with and without `color`.
- [ ] **AC-16** **Metadata (DEC-113).** Input: a 7×5 `Rgb8` PNG, a 1 px white frame round 5×3 content,
  carrying an `eXIf` chunk with Orientation 6 (built as `tests/pad.rs:1317`'s `tagged_png`). On the
  binary: control B, `auto-orient` alone, writes 5×7; `trim` writes 5×3 with no Orientation tag
  (`common::orientation_of` gives `None` or `Some(1)`); `trim` then `auto-orient` writes **3×5** (the
  bundle survived the trim).

**Load refusals** (DEC-102)
- [ ] **AC-17 … AC-39** (one per row) Each row T1–T23 is refused by `Recipe::build_pipeline` with
  `with_builtins()` as a `RecipeError::InvalidOperation` whose `Display` is **exactly**
  `recipe step 0 ('trim'): ` + that row's message (full text).
- [ ] **AC-40** The accepted controls load: no keys; `tolerance = 0`; `tolerance = 255`;
  `color = "#00000000"`; `color = "FFFFFF"`; `color = "#123456ff"`; `color = "#ffffff"` with
  `tolerance = 8`.
- [ ] **AC-41** On the binary, `apply --recipe <T1> in.png -o out.png` exits 1, prints exactly
  `error: recipe step 0 ('trim'): unknown key 'fuzz'; expected one of: color, tolerance`, and writes
  no `out.png`.
- [ ] **AC-42** On the binary, `apply --recipe <T12>` exits 1, prints exactly `error: recipe step 0
  ('trim'): 'tolerance' must be an integer in 0..=255, got the float 8.0`, and writes nothing.

**Apply failure**
- [ ] **AC-43** **One input.** `apply` with `op = "trim"` on a 1×1 `Rgb8` PNG `(7, 8, 9)` exits **1**,
  prints exactly `error: operation 'trim' failed: every pixel of this 1×1 px image matches its
  top-left pixel within 'tolerance' = 0, so a trim would leave nothing`, and writes no `out.png`.
- [ ] **AC-44** **A batch.** `apply --out-dir` with the same recipe on `a.png` (that 1×1) and `b.png`
  (an 8×6 `Rgb8`, 4×3 content in a white border (1, 2, 3, 1)): exit **6**; stderr contains
  `a.png: operation 'trim' failed: ` + AC-43's reason and the line `error: 1 of 2 inputs failed`;
  `out/a.png` does not exist; `out/b.png` is 4×3.

**Inverse**
- [ ] **AC-45** **Pad then trim.** Through the pipeline API with the shipped `pad`: `Rgba16` 37×23 →
  `pad aspect = "16:9", gravity = "southeast"`; `Rgb8` 23×37 → `aspect = "3:2", gravity =
  "northwest"`; `L8` 7×5 → `border = 0.3`, fill `#808080`; `Rgba32F` 5×9 → `aspect = "1:1"`; `Rgb16`
  1×1 → `border = 1`; each with fill `#123456` unless stated and then `trim` with the same `color`,
  gives back the input's bytes and variant. `L8` 7×5 → `border = 0.3, color = "#123456"` → `trim
  color = "#123456"` gives the input widened to `Rgb8` (`(l, l, l)` per pixel, compared with
  `DynamicImage::to_rgb8` of the input). **The intended failure:** a 4×3 `Rgb8` whose top row is all
  white, `pad border = 0.5, color = "#ffffff"`, then `trim` with no keys, gives the input's
  `(0, 1, 4, 2)` (the white row went with the pad).

**End to end, corpus, wasm**
- [ ] **AC-46** On the binary, `apply` with `op = "trim"` on AC-44's 8×6 `b.png` writes a 4×3 PNG equal to
  the content, exit 0.
- [ ] **AC-47** The corpus gains `v2_trim_corner` (`pad border = 0.01 color = "#ffffff"`, `pad border =
  0.01 color = "#000000"`, `trim`) and `v2_trim_tolerance` (`pad border = 0.01 color = "#ffffff"`, `pad
  border = 0.01 color = "#080808"`, `trim color = "#000000" tolerance = 8`), each with `.toml` (a
  two-line `#` comment, `version = "2"`, `name = "corpus-trim-corner"` / `"corpus-trim-tolerance"`),
  `.golden` and three per-target `.output` files, added by the README's procedure. `EXPECTED_DIMS` for
  both: `["2652x1782", "1782x2652", "2652x1782"]`. The literals (recounted at design from `main` at
  `b05d630`: `sed -n '/^const RECIPES/,/^];/p' tests/recipe_corpus_output.rs | /usr/bin/grep -c '^    "'`
  → 20, and the same on `CORPUS` in `recipe_round_trip.rs` → 17) become: `RECIPES` **22** (the two
  inserted after `v2_rotate_90`), the `EXPECTED_DIMS` array length **22** (the two entries after
  `v2_rotate_90`'s), `ROWS` **72**, `GOLDEN_LINES` **75**, the `60` in
  `EXPECTED_DIMS.len() * FIXTURES.len()` → **66**, the doc comments ("all 22", "22 recipes x 3
  fixtures", "66 pixel rows"), `CORPUS` **19** stems, and `pairs.len()` **22** with the message
  "19 corpus recipes + 3 bundled recipes". Driven on the prototype: the corpus and round-trip tests
  pass with these literals.
- [ ] **AC-48** For each new recipe, its three per-target `.output` files are byte-identical, and the
  build says so with the `cmp` it ran.
- [ ] **AC-49** **No existing corpus output changes:** `git diff --diff-filter=MD --name-only
  origin/main -- tests/fixtures/recipe_corpus/` is **empty**; `--diff-filter=A` lists exactly the ten
  new files; `cargo test --test recipe_corpus_output` and `--test recipe_round_trip` pass with no
  re-bless of an existing file.
- [ ] **AC-50** **`v2_trim_tolerance` pins the 16-bit edge.** On `rgba16_2600x1730` the second frame is
  `0x0808` = 2056 levels from black, exactly the tolerance; the `<` for `<=` mutation (M2) and the
  `<< 8` mutation (M1) each turn this row red **by dims** (`2706x1836`, expected `2652x1782`) and with
  `step 2 ('trim') changed nothing`. Recorded in Build Completion from the mutation run.
- [ ] **AC-51** wasm `transform` (DEC-110 limb 2, exact), on `png_unique(120, 80)`: `pad border = 0.05,
  color = "#123456"` then `trim` (no keys) gives exactly 120×80 equal to the decoded input; `pad aspect
  = "1:1", gravity = "south", color = "#123456"` then `trim color = "#123456"` gives the same. Zero
  pixels differ in each.
- [ ] **AC-52** wasm `transform` throws, and the message contains exactly the native line, for T1
  (`fuzz = 8`), T12 (`tolerance = 8.0`) and the whole match on a 4×3 PNG of one colour (encoded in the
  test with `image`: `operation 'trim' failed: every pixel of this 4×3 px image matches its top-left
  pixel within 'tolerance' = 0, so a trim would leave nothing`); an identity transform succeeds after
  each.
- [ ] **AC-53** `scripts/wasm-ci-check.sh`: **`FULL_COUNT=55`, `SLIM_COUNT=53`** (both new tests run in
  both sets; `main` is 53/51 at `b05d630`; re-read after the build's last rebase and add 2 to whatever
  it then says). The `--self-test` fixtures move with them: the base line is `test result: ok. 55
  passed; …` in a variable named `l55`; derived files `l1`, `l54` (one under full) and `l53` (slim);
  rows 1 `count full 55` → ok `full set: 55 passed, as expected`; 2 `count full 1` → fail `full set: 1
  passed, expected 55`; 3 `count full 54` → fail `expected 55`; 4 `count slim 55` → fail `slim set: 55
  passed, expected 53`; 5 `count slim 53` → ok `slim set: 53 passed, as expected`; 7 (`two`) and 9
  (`count fat`) built from `l55`. Row 8 is unchanged. `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh
  --self-test` (18 rows, 18 passed) and `… workflow` (16 assertions, 0 failed) exit 0, and CI's full and
  slim count steps pass. Driven on the prototype script.

**Folded entry**
- [ ] **AC-54** **`resize`'s cap text is pinned.** `resize_oversize_is_typed_error`
  (`src/operation/mod.rs:2351`) asserts its first error's `Display` contains `target dimensions
  60000×10 exceed the maximum edge limit of 50000 px` and its second's contains `target area
  20000×20000 = 400000000 px exceeds the maximum area of 134217728 px`;
  `resize_apply_percent_rejects_oversized_output` (`:3565`) asserts `target dimensions 2000000×2000000
  exceed the maximum edge limit of 50000 px`; `apply_recipe_with_oversized_resize_exits_1`
  (`tests/apply_batch.rs:461`) asserts stderr contains `target area 50000×50000 = 2500000000 px exceeds
  the maximum area of 134217728 px`. The existing kind and exit-code asserts stay. Ship marks the
  STAGE-058 entry `[x]`.

**Docs** (verify checks each by reading; `file:line` is on `main` at `b05d630`)
- [ ] **AC-55** `docs/recipes.md:120` (`- **Trim** **[planned — PROJ-017]**`) becomes
  "**Trim a uniform border** **[today]** — a recipe step:", a TOML snippet (`op = "trim"`), "or against a
  stated colour, within a tolerance:", a second snippet (`op = "trim"`, `color = "#ffffff"`,
  `tolerance = 8`), and these sentences: "`trim` compares each pixel with one colour, the stated
  `color` or the image's top-left pixel, and removes the outer rows and columns in which every pixel
  matches. It never looks at what the content is." / "Each side is trimmed on its own, so a border
  can be wider on one side than another." / "With no `color`, the colour is the top-left pixel of the
  image as the step receives it: put `op = \"auto-orient\"` first when the recipe is meant as the photo
  is displayed." / "`tolerance` (default 0, exact) is in 8-bit steps per channel at every bit depth:
  `tolerance = 8` allows 8 off at 8 bits and 2056 (8 × 257) at 16 bits, and a float sample is read
  as its nearest 16-bit value." / "A fully transparent pixel matches a transparent `color` (such as
  `\"#00000000\"`) whatever colour is hidden under it; any other pixel is compared on all four
  channels, alpha included." / "An image with no border is written unchanged. An image in which
  every pixel matches fails, because trimming it would leave nothing." / "`pad` then `trim` with the
  same colour gives back the original, unless an edge row or column of the original already matches
  the colour; a grey image padded with a colour comes back as RGB." / "ImageMagick's `-fuzz` is a
  colour distance, not a per-channel step, so `magick -trim -fuzz` cuts differently whenever
  `tolerance` is above 0."
- [ ] **AC-56** `docs/recipes.md:282` reads: `**PROJ-017:** geometry as recipe steps — \`crop\`,
  \`rotate\` (quarter turns), \`flip\`, \`pad\` and \`trim\` today; straighten **planned**.`
- [ ] **AC-57** `recipes/README.md:96`: the op table gains a `trim` row after `pad`: "an optional
  `color` and `tolerance` (below)". A new `### The trim step` section after `### The pad step` (before
  `### Step keys are checked`, `:298`) gives both snippets and, as bullets: each pixel is compared with
  one colour, the stated `color` or the top-left pixel, and the outer rows and columns that match are
  removed, each side on its own; `tolerance` is 0..=255 (default 0) in 8-bit steps per channel, × 257 at
  16 bits, a float read as its nearest 16-bit value; a pixel with alpha 0 matches a `color` with alpha
  0 whatever is hidden under it; nothing to trim writes the image unchanged; an image that matches
  everywhere fails that image (a batch exits `6`); `color` is hex and may be translucent; the colour
  type and bit depth are kept; put `op = "auto-orient"` first when the recipe is meant as the photo is
  displayed.
- [ ] **AC-58** `docs/data-model.md:162-164`: "nine operations" becomes **ten**, naming `trim`
  ("`crop`, `rotate`, `flip`, `pad` and `trim`"). A new `### \`trim\` step param keys` section after the
  pad section (before `### Step keys are checked at load`, `:424`) has a `| Key | Type | Range |` table
  (`color` / string / optional, `RRGGBB`, `#RRGGBB` or `RRGGBBAA`, default the top-left pixel;
  `tolerance` / integer / 0 to 255, default 0, in 8-bit steps), and a TOML example with
  `op = "auto-orient"` and `op = "trim"` (`tolerance = 8`) steps, which `tests/docs_ops.rs` checks
  resolve.
- [ ] **AC-59** `skills/crustyimg/SKILL.md:236-237`: the ops list gains `trim` (an optional `color`,
  default the top-left pixel, and `tolerance` 0–255 in 8-bit steps) after `pad`, and the frontmatter
  `description` (`:3`) says "crop, rotate, flip, pad, trim" where it says "crop, rotate, flip, pad".
- [ ] **AC-60** `docs/api-contract.md`: a "**Trim in a recipe.**" paragraph after the pad paragraph
  (`:476-482`) says `op = "trim"` removes the outer rows and columns that match one colour (the stated
  `color` or the top-left pixel) within `tolerance`; a bad key or value is refused when the recipe
  loads (exit **1**, nothing read or written); an image in which every pixel matches fails at apply
  (`operation 'trim' failed: …`, exit **1** for one input, exit **6** in a batch, with nothing written
  for that input); an image with nothing to trim is written unchanged; wasm `transform` throws the same
  line; and the pixel lane writes no Orientation tag after it.
- [ ] **AC-61** `CHANGELOG.md` under `## [Unreleased]`: in `### Added`, after the pad entry (`:19-21`),
  "**A recipe can trim.**" — `op = "trim"` removes a uniform border, against a stated `color` or the
  top-left pixel, within a `tolerance` in 8-bit steps; the kept pixels are unchanged and the colour type
  and bit depth kept; 0.8.2 and earlier refuse it with `unknown operation 'trim'`. A new `### Fixed`
  entry: a colour with a non-ASCII character (`color = "aéaaa"`, `--color "aéaaa"`) is refused with the
  colour message instead of crashing (`pad`, `watermark`). A new `### Breaking` entry, for library users
  only: `ParamKind::UInt` has a `max` field; code that builds `ParamKind::UInt { min }` adds
  `max: None`. The CLI is unchanged by it.
- [ ] **AC-62** `integrations/ddphotos/README.md:127-129` says recipes can crop, rotate, flip, pad and
  trim (`op = "trim"` added to the list) and the edit page offers none of them yet.
  `docs/backlog.md:193`'s `trim` row says it shipped as a recipe step (`color`, `tolerance`; the
  top-left pixel by default). `README.md` and `npm/README.md` are unchanged.
- [ ] **AC-63** No user-facing string this spec adds names a spec, stage or DEC id:
  `/usr/bin/grep -nE "SPEC-|STAGE-|DEC-|PROJ-" src/operation/trim.rs` finds nothing, and no added line in
  `registry.rs`, `mod.rs` or `text/mod.rs` names one. If SPEC-150 has merged,
  `cargo test --test help_has_no_internal_ids` passes. (`docs/backlog.md` and `docs/recipes.md`'s
  roadmap line may name PROJ-017, as they do today.)

**Hygiene**
- [ ] **AC-64** No version bump, no `CACHE_SCHEMA_VERSION` change (`src/build/cache.rs:81`), no
  lockfile-format change. Why none is needed: no existing recipe's canonical TOML or output changes
  (AC-49); the cache key already folds in the crustyimg version; and 0.8.2 refuses a `trim` recipe at
  load (`unknown operation 'trim'`), so no cached entry or lockfile line for one can exist. The
  `UInt` change alters no recipe's accepted values (the three existing rows keep `max: None`).
- [ ] **AC-65** `just check` is green; CI is green on every leg, including the lean
  (`--no-default-features`) build, both wasm count steps and the wasm bundle-size gate (the build
  records the raw and brotli sizes).

## Failing Tests

Written at design; the build makes them pass. **Baseline on `main`:** every integration row in
`tests/trim.rs` is RED with `unknown operation 'trim'` (load rows report it as the wrong message;
binary rows as exit 1 with that line); the non-ASCII rows RED by a **panic** (`end byte index 2 is not
a char boundary`); the corpus rows RED with a missing `v2_trim_corner` file; the fold-in asserts RED
only if the text has drifted (they are green on `main`: say so, they pin, they don't fix); the unit
tests don't compile until `trim.rs` exists. The build quotes one RED message per test file. **Wrong-
reason trap (13):** every refusal and apply row asserts the whole message with `==` (or `contains` the
whole line in batch stderr). The likely wrong-reason RED: AC-16, where control B must say "the input's
Orientation tag never reached the pipeline" if the `eXIf` chunk didn't load, not "trim failed"; and
AC-12's binary row, which must fail on the exit code (101 on `main`), not on a missing file.

- **`src/operation/trim.rs` (`#[cfg(test)] mod tests`)** — calls the private matcher and
  `Trim::from_params` + `apply`, never a copy.
  - `match_rule_table` — AC-2, 36 rows as data, asserts 36 ran.
  - `float_rule_is_exact_at_every_edge` — AC-3 (prints the case count; asserts 2,357,504).
  - `params_round_trip_as_written` — AC-4.
  - `refusal_hook_is_not_used` — AC-15.
  - `whole_match_names_its_source` — AC-14's three messages through `apply` directly.
- **`src/operation/registry.rs` (`mod tests`)**
  - `with_builtins_contains_trim` (new) — AC-1.
  - `every_builtin_publishes_a_param_schema` (extended) — `keys("trim") == ["color", "tolerance"]`,
    kinds as AC-1; the three `UInt` rows' kinds asserted with `max: None`.
  - `uint_max_is_in_the_expectation` (new) — AC-5's two `expectation()` strings and the two
    unchanged messages.
- **`src/text/mod.rs` (`mod tests`)** — `parse_color_refuses_non_ascii` — AC-12 unit half.
- **`tests/trim.rs` (new)** — through the public API and the binary
  (`env!("CARGO_BIN_EXE_crustyimg")`, stdout and stderr captured, exit code read directly). Buffers
  are built in the test from 16-bit levels (8-bit sample = `level / 257`, float = `level / 65535` in
  `f64` then `f32`) with a seeded xorshift for content, never from the code under test. Expected boxes
  are literals; output is compared with the input's sub-rectangle by raw samples (the prototype's
  `sub_rect_mismatch`).
  - `every_trim_refusal_names_its_reason` — AC-17…39: the 23-row table as data `(id, body,
    message)`; runs every row, collects each mismatch as `"{id}: got …"` (a row that builds is
    reported as such), asserts 23 rows ran.
  - `accepted_spellings_load` — AC-40.
  - `a_non_ascii_colour_is_refused_not_a_panic` — AC-12, the three recipe rows and the binary row.
  - `every_colour_type_trims_exactly` — AC-6, 210 cases, failures named by kind, source, shape and
    offset.
  - `each_side_is_scanned_on_its_own` — AC-7, 8 rows.
  - `odd_shapes_and_survivors` — AC-8.
  - `grey_compares_as_three_equal_channels` — AC-9.
  - `alpha_zero_matches_whatever_is_hidden` — AC-10.
  - `nan_and_out_of_range_floats` — AC-11.
  - `whole_match_fails_at_apply` — AC-14 through the pipeline.
  - `no_op_returns_the_input` — AC-13 unit half; `apply_no_op_trim_writes_the_input_pixels` — binary
    half.
  - `pad_then_trim_is_the_identity` — AC-45.
  - `apply_trims_a_png_end_to_end` — AC-46.
  - `apply_refuses_an_unknown_key_and_writes_nothing` — AC-41;
    `apply_refuses_a_float_tolerance_and_writes_nothing` — AC-42.
  - `apply_fails_when_every_pixel_matches_with_exit_1` — AC-43;
    `batch_with_a_uniform_image_fails_that_input_with_exit_6` — AC-44.
  - `trim_writes_no_orientation_tag_and_keeps_the_bundle` — AC-16 (control B first).
- **`src/operation/mod.rs`, `tests/apply_batch.rs`** — AC-54's asserts in the three named tests.
- **`tests/recipe_corpus_output.rs` / `tests/recipe_round_trip.rs`** — AC-47…50: the stems,
  `EXPECTED_DIMS`, goldens, literal bumps; no other change.
- **`tests/wasm_roundtrip.rs`** — AC-51…52, `#[wasm_bindgen_test]`, not feature-gated:
  `transform_trims_exactly` and `transform_refuses_a_bad_trim_with_the_native_message`.
- **`scripts/wasm-ci-check.sh --self-test`** — AC-53's rows.
- **`tests/docs_ops.rs`** — unchanged; it goes red if AC-58's example lands before `trim` is
  registered.

### Mutation table (traps 12, 13 and 14)

Apply one at a time, run `cargo test --no-fail-fast` (stdout to a file, `$?` read directly) and, where the
row says, the script; record the flip, revert, confirm green. A row whose red set doesn't match is
reported, not edited to match.

**Driven** means applied at design to the prototype (scratch `git worktree` of `b05d630`, own
`CARGO_TARGET_DIR`, full `cargo test --release --no-fail-fast`; prototype baseline **59 test binaries,
all green**; each edit asserted to change its file and the restore asserted byte-equal). The prototype's
tests carry this spec's names (its extra `match_rule_rows` is a subset of `match_rule_table`); tests this spec adds that the prototype didn't have are listed as
*reasoned* in the same row. In every row, every test not listed red was **observed green** over the
whole suite. No row is "expected all green".

- **M1 `× 257` → `<< 8` for the tolerance** (the matcher's `tol = t × step`; at 16 bits and float,
  `t << 8`). Splitting input: a 16-bit sample 2056 off at `t = 8` (2056 > 2048). *Driven red:*
  `trim::tests::match_rule_table` (`rgb16 2056`, `f32 2056`), `every_colour_type_trims_exactly`
  (16-bit and float `t = 8` edge cases), corpus `every_recipe_output_matches_its_golden` with exactly
  one row: `v2_trim_tolerance × rgba16_2600x1730: dims 2706x1836, expected 2652x1782` **by dims**,
  plus `dims changed: golden 2652x1782` and `step 2 ('trim') changed nothing`. 8-bit corpus rows and
  `v2_trim_corner` green (expected: `t = 0` there).
- **M1b `× 257` → `<< 8` for a stated colour** (`#ff` → 65280). *Driven red:* `match_rule_table`
  (`rgb16 ff`, `l16 grey`), `grey_compares_as_three_equal_channels` (`L16`), `pad_then_trim_is_the_identity`
  (16-bit and float cases), `nan_and_out_of_range_floats`, `every_colour_type_trims_exactly` (stated
  colour, 16-bit and float). Corpus green (its stated colours are `#000000`, which `<< 8` keeps).
- **M2 `<=` → `<`** (the per-channel compare). Splitting input: any sample exactly `t` off (and every
  exact match at `t = 0`). *Driven red:* 13 tests: `match_rule_rows`, `match_rule_table`, corpus golden
  (all six `v2_trim_*` rows by dims `2706x1836` and `step 2 ('trim') changed nothing`, **including
  `v2_trim_tolerance × rgba16`**, AC-50), `grey_…`, `each_side_is_scanned_on_its_own`,
  `odd_shapes_and_survivors`, `pad_then_trim_…`, `whole_match_fails_at_apply`, `nan_…`,
  `every_colour_type_…`, `apply_fails_when_every_pixel_matches_with_exit_1`,
  `batch_with_a_uniform_image_…`, `trim_writes_no_orientation_tag_…`. *Reasoned:* AC-13's and AC-46's
  binary rows, both wasm rows.
- **M3 the float rule replaced by `f64` `/ 255` maths** (`|f64(s) − c/255| <= t/255` for a stated
  colour on `Rgb32F`/`Rgba32F`). Splitting input `c = 0`, `t = 1`, `s = f32(1/255)`. *Driven red:*
  `match_rule_rows` and `match_rule_table` (`f32 1/255`, `f32 over`, `f32 under`, the `Rgba32F` alpha
  rows), `alpha_zero_…` (`Rgba32F`), `nan_…`, `pad_then_trim_…` (`Rgba32F`). **Observed green:**
  `every_colour_type_trims_exactly`: its float edge (2056 below `1.0`) happens to fall on the accepted
  side in `f64`, which is why the matrix is not the float guard; `match_rule_table` is.
- **M4 the alpha-0 rule removed.** Splitting input: the `a0` shape. *Driven red:* `match_rule_rows`,
  `match_rule_table` (`la16 a0 tl`, `rgba8 a0`, `rgba16 a0 tl`, `rgba32f a -0`, `a 1e-6`),
  `alpha_zero_matches_whatever_is_hidden`.
- **M5 top-left read as bottom-right** (`read(w − 1, h − 1)`). Splitting input: the `tlodd` shape (a
  corner that differs from the top-left). *Driven red:* `match_rule_table` (`rgb16 tl+1`, `rgba32f tl
  level`: the sample at (1, 0) is the bottom-right of a 2×1), `every_colour_type_trims_exactly` (the
  `t = 8` one-past cases, whose bottom-right is the off pixel). Green: `alpha_zero_…` (every corner of
  its border is alpha 0, so any corner is a correct reference there).
- **M6a–d one side's scan dropped** (`top = 0`; `bottom = h − 1`; `left = 0`; `right = w − 1`). Splitting
  input: the one-step-outside rows of AC-7. *Driven red, each of the four:* corpus golden (both recipes,
  all three fixtures, by dims, e.g. M6a `2652x1809`), `each_side_is_scanned_on_its_own`,
  `odd_shapes_and_survivors`, `grey_…`, `alpha_zero_…`, `nan_…`, `pad_then_trim_…`,
  `every_colour_type_…`, `batch_with_a_uniform_image_…` (b.png's size), `trim_writes_no_orientation_tag_…`
  (10 tests each). *Reasoned:* AC-46, wasm `transform_trims_exactly`.
- **M7 the whole-match check removed** (`return Ok(img)` instead of the error). Splitting input: a 1×1
  image. **What is written instead:** the input, unchanged, exit 0, no message. *Driven red:*
  `whole_match_fails_at_apply`, `apply_fails_when_every_pixel_matches_with_exit_1` (exit 0, a file
  written), `batch_with_a_uniform_image_fails_that_input_with_exit_6` (exit 0). *Reasoned:*
  `whole_match_names_its_source` (unit), wasm W1 row.
- **M8 grey widened wrong** (`L` compared as `(L, 0, 0)`). Splitting input: `L = 128` against
  `#808080` and against `#800000`. *Driven red:* `match_rule_table` (`l8 grey`, `l8 chroma`, `l16 …`,
  `la8 alpha`), `grey_compares_as_three_equal_channels`, `pad_then_trim_…` (`L8`),
  `every_colour_type_…` (grey kinds).
- **M9 wasm count one lower** (`FULL_COUNT=54` after AC-53). *Driven* on the prototype script:
  `count full` on a 55-passed log → `full set: 55 passed, expected 54`, exit 1; `--self-test` → `18
  rows, 15 passed, 2 wrong verdict, 1 wrong message` (rows 1, 2, 3). AC-53's literals: self-test 18/18,
  `workflow` 16/0.
- **M10 corpus guard** (scratch, never committed): append `[[step]] op = "trim"` to
  `tests/fixtures/recipe_corpus/v1_resize_max.toml`. *Driven red:* `every_recipe_output_matches_its_golden`
  naming `v1_resize_max × rgb8_2600x1730: step 2 ('trim') changed nothing` (and the `rgba8`, `rgba16`
  rows; a lone trim on a noisy fixture is a no-op, so the "every acting step acts" control is what
  catches it, not dims), and `to_toml_is_byte_identical_to_main_over_the_corpus`.
- **M11a/b the fold-in, one character of each cap line** (`exceed` → `exceeds` in the edge line;
  `exceeds` → `exceed` in the area line). *Driven red:* M11a `resize_oversize_is_typed_error`,
  `resize_apply_percent_rejects_oversized_output`; M11b `resize_oversize_is_typed_error`,
  `apply_recipe_with_oversized_resize_exits_1`. So the three tests are red across the two lines; each
  line is pinned by two.
- **M12 trap 14 for the op: `main`'s `with_builtins` restored verbatim** (the whole function; it differs
  only by the `trim` line). *Driven red:* 17 tests: all 15 integration rows in `tests/trim.rs` (each loads a
  `trim` recipe, including `accepted_spellings_load`, and `a_non_ascii_colour_…` via its trim row),
  corpus golden, `to_toml_is_byte_identical_to_main_over_the_corpus`. *Reasoned:*
  `with_builtins_contains_trim`, `every_builtin_publishes_a_param_schema`,
  `every_op_named_in_data_model_resolves` (AC-58's example), the binary rows the prototype lacked, both
  wasm tests. Green: the `trim.rs` unit tests (they call the module directly).
- **M13 the non-ASCII check removed** (trap 14 for the fold-in fix: `parse_color` as on `main`).
  *Driven red:* `a_non_ascii_colour_is_refused_not_a_panic` (a panic). *Reasoned:*
  `parse_color_refuses_non_ascii`, T23 in `every_trim_refusal_names_its_reason`.
- **M14 bundle dropped** (`Image::from_parts(out, img.source_format(), None)`). *Driven red:*
  `trim_writes_no_orientation_tag_and_keeps_the_bundle` only (`trim` then `auto-orient` gives 5×3).
- **M15 the table's `max` ignored** (`check` uses `u32::MAX` whatever `max` says). *Driven red:*
  `every_trim_refusal_names_its_reason` (T8 and T11 reach the constructor's `internal:` line). T10 stays
  refused by the `u32` cap with the same text. *Reasoned:* `uint_max_is_in_the_expectation` stays green
  (it checks `expectation()`, which is unchanged): the refusal table is the guard.
- **M16 the crop shifted one pixel right, same size** (`crop_imm(left + 1, …)`; trap 13's golden-by-hash
  row). *Driven red:* corpus golden, all six `v2_trim_*` rows `pixels changed (dims … unchanged)`
  **by hash**; `every_colour_type_…`, `each_side_…`, `odd_shapes_…`, `grey_…`, `alpha_zero_…`, `nan_…`,
  `pad_then_trim_…`.
- **M17 float clamp removed.** *Driven red:* `match_rule_table` (`f32 over`), `nan_and_out_of_range_floats`.
- **M18 NaN read as 0** (the `is_nan` check removed, so `as i64` gives 0). *Driven red:*
  `match_rule_table` (`f32 nan` at `t = 255`), `nan_and_out_of_range_floats` (`t = 255` row, all-NaN row).

Not run under mutation: the wasm tests (native only at design, as SPEC-151's build did), and the unit and
binary rows the prototype didn't have (each named as reasoned above).

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle. It is the
equivalent of a handoff document, folded into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-095` — keep the colour type and bit depth. `trim` computes no pixel: the output is `crop_imm` of
  the input, which keeps all ten variants. A grey image compared with a chromatic colour is widened
  only inside the comparison, never in the buffer.
- `DEC-102` — the key table runs before the constructor; `tolerance`'s range is in the table (the new
  `UInt` `max`); the constructor holds `color`'s syntax.
- `DEC-112` — rule 6 puts a tolerance outside the units rule; `tolerance` is an integer in 8-bit steps,
  read exactly, so no `f32`/`f64` question arises at load. The `f32` question moves to the float
  comparison, which the level rule makes exact (AC-3).
- `DEC-113` — rule 1 names `trim`: it acts on the buffer it receives and writes no metadata;
  `with_pixels` keeps the bundle for a later `auto-orient`. The top-left is the stored buffer's.
- `DEC-110` — limb 2: no resampling, so native and wasm agree exactly.
- `DEC-003` — the pixel lane writes no container metadata.

### Constraints that apply

- `untrusted-input-hardening` — nothing panics on any TOML value or image: the colour parse refuses
  non-ASCII (AC-12), the scan allocates nothing, the crop is at most the input.
- `no-unwrap-on-recoverable-paths` — no `unwrap`/`expect` in `trim.rs` outside tests.
- `single-image-library` — `image` only.
- `every-public-fn-tested` — `Trim::from_params` and the `Operation` impl.
- `test-before-implementation` — the tests above, red first, baseline quoted.
- `one-spec-per-pr` — one PR, `feat/spec-152-trim`.

### Prior related work

- `SPEC-151` (shipped) — `pad`: the shape this op mirrors, its colour message, `tests/pad.rs`'s
  helpers, the corpus and wasm rows, the count move.
- `SPEC-145` (shipped) — `crop`: `crop_imm`, the 0 px apply failure and the batch exit 6.
- `SPEC-141` (shipped) — the corpus guard and its README procedure.
- `SPEC-131` (shipped) — the key tables (`ParamKind`) and typed constructors.

### Files shared with SPEC-150 (PROJ-011, PR #257, draft, building)

Read on 2026-11-06 from SPEC-150's `### Files the build touches` and `gh pr view 257 --json files`
(head `5996c6b`). The design prompt expected overlap in `src/operation/registry.rs`,
`src/operation/mod.rs`, `src/wasm.rs` and `tests/wasm_roundtrip.rs`; SPEC-150 lists all of
`src/operation/**` and `tests/wasm_roundtrip.rs` as **not touched**, and its PR agrees. The real
overlap:

- **`docs/api-contract.md`** — both. SPEC-150's hunk is the exit-code table at `:55-58` (row `3`);
  SPEC-152's is the new "Trim in a recipe" paragraph after `:482`. Different hunks.
- **`src/wasm.rs`** — SPEC-150 only (the asset refusal at `:236-240`). SPEC-152 doesn't edit it.
- **`CHANGELOG.md`** — SPEC-150's spec does not list it; if its build adds an Unreleased entry, both
  append to `## [Unreleased]`; rebase and keep both.
- **SPEC-150's new guard** `tests/help_has_no_internal_ids.rs` scans every production string literal
  in `src/` for `SPEC-<digit>` and `DEC-<digit>`. Every literal `trim.rs` adds is id-free (AC-63), so it
  stays green. Comments are not scanned, but AC-63 keeps them id-free too.

**Merge order:** whichever lands second rebases. Rebase onto `main` before each push; after the last
rebase, re-read `FULL_COUNT`/`SLIM_COUNT` on `main` (SPEC-150 adds no wasm test today, but re-read
anyway) and add 2 to whatever they say.

### Out of scope (for this spec specifically)

- **`edit --trim`** and the ordering rule: STAGE-058 line 8 ("`edit` flags for the five").
- **`pad` with a transparent fill**: its own STAGE-058 entry. `trim` accepts a translucent `color`
  because it only removes pixels; that doesn't touch `pad`'s refusal.
- **The ddphotos edit page** offering trim: STAGE-060.
- **Content-aware trimming**: ignoring noise or specks, trim to content, saliency. Post-1.0 with smart
  crop.
- **Per-side colour or tolerance, a colour-distance mode, a kept margin**: not asked for; a later key
  each. A kept margin is `trim` then `pad border`.
- **A softer alpha rule** (a nearly transparent pixel matching a transparent `color` within the
  tolerance on alpha alone). Ruling 3 compares all four channels unless alpha is exactly 0; AC-2's
  `a3 far` row pins that a pixel at alpha 3 with a bright hidden colour is kept by
  `#00000000, tolerance = 8`. A later key if an anti-aliased logo edge needs it.
- **The batch summary's "watermark skipped" text**, **watermark's nine gravities with a margin**,
  **the guard's file-set and line-count failures together**: their own STAGE-058 entries.
- **The `Rgba32F` area cap at 2 GiB**: unchanged; `trim` allocates no more than its input.
- **The `UInt` message for a value past `u32::MAX` on the existing rows** (`'width' must be an integer
  >= 1, got 4294967296`): unchanged here; those rows keep `max: None`.

## Notes for the Implementer

- **Checkpoint.** Commit and push after each of: (1) the red tests, with the baseline quoted; (2) the
  op green locally; (3) the corpus files for your own target; (4) the wasm tests and the script;
  (5) the fold-in and the `parse_color` fix; (6) the docs. Open the PR as a **draft**.
- **The red-test commit** can hold the integration, registry, `text` and fold-in tests; the unit tests
  in `trim.rs` can't compile before the module exists.
- **The prototype's shape** (a reader per variant giving four `Option<i64>` levels, a matcher closure,
  four separate side searches) is one way; any shape that passes the tests and keeps each side's
  search separate is fine. Keep the `u8::try_from` fallback message distinct from the table's (M15).
- **Corpus bless order.** Add the two `.toml` files, write their `.golden` by the README (canonical TOML:
  keys sorted, so `op` first, then `border`, `color`; the trim step `op`, `color`, `tolerance`; copy the
  shape `to_toml()` prints), raise every literal in one commit, bless your own target, push, and copy the
  other two targets' files from the failing CI log's `----- BEGIN … -----` blocks. Then `cmp` all three
  per recipe (AC-48). Expect both recipes' `.output` pixel lines to be identical to each other
  (correction 3). Gate each commit on its test run; don't chain `git commit` after `cargo test` with `;`.
- **wasm.** Both new tests run in the full and slim sets. Re-read both counts on `main` after your last
  rebase and move the self-test fixtures in the same commit. Run `LC_ALL=en_US.UTF-8
  scripts/wasm-ci-check.sh --self-test` and `… workflow`, `just wasm-test` and `just wasm-test-slim` if
  you can, and read CI's two count steps. **Never poll CI and never write your own wait loop**
  (AGENTS §16 habit 6): run `gh pr checks --watch` once in the background, or stop and report.
- **Mutations.** Drive the table in a scratch `git worktree` of your pushed HEAD with its own
  `CARGO_TARGET_DIR`, never in the build worktree; `cargo clean -p crustyimg` or a fresh target dir
  before each run, full `cargo test --no-fail-fast` to a file, `$?` read, the edit checked to have
  changed the file, the restore checked with `git diff`. A row whose red set differs is reported in
  Build Completion, not edited to match.
- **`cargo test` in a terminal** fails `display_sink_refuses_non_tty`: redirect stdout to a file and read
  `$?`, never through a pipe. Run the lean build (`cargo test --no-default-features`) too: `trim` has
  no feature gate.
- **User-facing text** (messages, doc comments, docs) reads behaviour-first and names no spec, stage or
  DEC id.
- **Files the PROJ-011 orchestrator must keep clear of** while this builds: `src/operation/`,
  `src/text/mod.rs`, `src/recipe/`, `docs/recipes.md`, `recipes/README.md`, `docs/data-model.md`,
  `docs/backlog.md`, `skills/crustyimg/SKILL.md`, `integrations/ddphotos/README.md`,
  `tests/fixtures/recipe_corpus/`, `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs`,
  `tests/wasm_roundtrip.rs`, `tests/apply_batch.rs`, `scripts/wasm-ci-check.sh`. Shared with SPEC-150:
  `docs/api-contract.md` and `CHANGELOG.md` (above).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-152-trim`
- **PR (if applicable):** #258 (draft), head `1627d5a` before this commit; CI green on every leg
- **All acceptance criteria met?** yes (65 of 65; notes below)
- **New decisions emitted:** none
- **Files this diff touches** (`git diff --name-only origin/main...HEAD`: 28 files, 29 with this spec file):
  - `src/operation/trim.rs` (new) — the op, match rule, scan, unit tests
  - `src/operation/mod.rs` — `pub mod trim`, re-export, `max: None` on three `UInt` rows, fold-in asserts
  - `src/operation/registry.rs` — `UInt` `max`, registration, doc lists, three tests
  - `src/text/mod.rs` — non-ASCII colour refused; unit test
  - `tests/trim.rs` (new) — 19 integration tests (refusals, 210-case matrix, scan, alpha, NaN, inverse, binary, metadata)
  - `tests/apply_batch.rs` — fold-in assert on the area cap text
  - `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs` — two stems, the literals
  - `tests/fixtures/recipe_corpus/v2_trim_{corner,tolerance}.{toml,golden,linux-x86_64.output,macos-aarch64.output,windows-x86_64.output}` — 10 new files
  - `tests/wasm_roundtrip.rs` — two wasm tests
  - `scripts/wasm-ci-check.sh` — 55/53 and the self-test fixtures
  - `docs/recipes.md`, `recipes/README.md`, `docs/data-model.md`, `docs/api-contract.md`, `skills/crustyimg/SKILL.md`, `integrations/ddphotos/README.md`, `docs/backlog.md`, `CHANGELOG.md` — docs
- **Deviations from spec:** none in files. The registration is one line (rustfmt), so M12's restore removes that line.
- **Follow-up work identified:** none.

**Evidence**
- Baseline on `main` before the op: `tests/trim.rs` 19 of 19 red (`unknown operation 'trim'`; the non-ASCII row panicked at `src/text/mod.rs:60:49`); the lib tests did not compile (`E0559 ... no field named 'max'`, 7 errors); the fold-in asserts were green on `main`, as the spec said.
- AC-1..16: met (unit `match_rule_table` runs 36 rows; `float_rule_is_exact_at_every_edge` asserts 2,357,504 cases, which is 14 match + 14 past-by-step + 7 past-by-level cases per in-range pair, plus 65,536 level read-backs; the matrix runs 210). AC-17..42: met (23 refusal rows, 7 controls, 2 binary rows). AC-43..46: met. AC-47..50: met. AC-51..53: met. AC-54: met. AC-55..62: met by content. AC-63: met (grep of `trim.rs` empty; `help_has_no_internal_ids` passes since SPEC-150 merged). AC-64: met (`git diff --stat origin/main...HEAD -- Cargo.toml Cargo.lock src/build/cache.rs` is empty). AC-65: met (CI green on every leg, including lean and both wasm count steps).
- Corpus: `git diff --diff-filter=MD --name-only origin/main -- tests/fixtures/recipe_corpus/` is empty; `--diff-filter=A` lists 10 files. First CI run failed only `every_golden_line_is_checked_and_no_golden_is_stray` and `every_recipe_output_matches_its_golden` (missing linux/windows files); the files were copied from the failing log and the second run is green on all legs. `cmp` of the three per-target files: both recipes identical across linux, macos and windows; the two recipes' pixel lines are identical to each other (correction 3). Dims `2652x1782` / `1782x2652` / `2652x1782`.
- wasm: local `just wasm-check` ok, `just wasm-test` 55 passed, `just wasm-test-slim` 53 passed; `--self-test` 18 rows, 18 passed; `workflow` 16 assertions, 0 failed; `validate-frontmatter` ok. CI: `full set: 55 passed, as expected`, `slim set: 53 passed, as expected`. `main` was 53/51 at the last rebase. Bundle size vs a `main` CI run (42417699527): raw 4,137,956 -> 4,150,967 B (+13,011), brotli 829,740 -> 832,650 B (+2,910).
- Matrix (fresh `CARGO_TARGET_DIR` each): default, `--no-default-features` (the `trim` tests ran), `--features webp-lossy`: `fmt --check`, clippy `-D warnings` and `cargo test --no-fail-fast` all exit 0.
- SPEC-150 merged (PR #257) before my last rebase; no `UInt` literal lacks `max`.
- Mutations (scratch worktree of the pushed head, own target dir, clean rebuild each, full suite; restored tree green):
  - M1: match_rule_table, every_colour_type_trims_exactly, corpus golden. M1b: match_rule_table, every_colour_type, grey, nan, pad_then_trim.
  - M2: 15 red: match_rule_table, float_rule_is_exact (unit), whole_match_names_its_source (unit), every_colour_type, each_side, odd_shapes, grey, nan, pad_then_trim, whole_match_fails_at_apply, apply_fails_when_every_pixel_matches, batch_with_a_uniform_image, trim_writes_no_orientation_tag, apply_trims_a_png_end_to_end, corpus golden. The reasoned binary no-op row stayed green (corrected: AC-13's binary half does not touch an edge); AC-46's row is red (confirmed).
  - M3 (float level as `ceil(f64(s) x 255 x 257)`): match_rule_table, float_rule_is_exact, every_colour_type, pad_then_trim. Note: the spec said the matrix stays green under its `/255` variant; mine, a different arithmetic, turned it red as well.
  - M4: alpha_zero, match_rule_table. M5: every_colour_type, match_rule_table.
  - M6a-d: each 11 red (alpha_zero, apply_trims_a_png_end_to_end, batch_with_a_uniform_image, each_side, every_colour_type, corpus golden, grey, nan, odd_shapes, pad_then_trim, trim_writes_no_orientation_tag). Reasoned AC-46 confirmed red.
  - M7: apply_fails_when_every_pixel_matches, batch_with_a_uniform_image, whole_match_names_its_source, whole_match_fails_at_apply.
  - M8: every_colour_type, grey, match_rule_table, pad_then_trim. M9 (script, FULL_COUNT=54): self-test 18 rows, 15 passed, 2 wrong verdict, 1 wrong message (rows 1, 2, 3), as the spec said.
  - M10: corpus golden, to_toml_is_byte_identical_to_main_over_the_corpus. M11a: resize_apply_percent_rejects_oversized_output, resize_oversize_is_typed_error. M11b: apply_recipe_with_oversized_resize_exits_1, resize_oversize_is_typed_error.
  - M12: 24 red, the integration rows that load trim plus every_op_named_in_data_model_resolves, corpus golden, to_toml_is_byte_identical, with_builtins_contains_trim, every_builtin_publishes_a_param_schema (reasoned cells confirmed); the `trim.rs` unit tests stayed green.
  - M13: a_non_ascii_colour_is_refused_not_a_panic, every_trim_refusal_names_its_reason, parse_color_refuses_non_ascii (reasoned cells confirmed). M14: trim_writes_no_orientation_tag_and_keeps_the_bundle only. M15: every_trim_refusal_names_its_reason only (uint_max_is_in_the_expectation stayed green, as reasoned).
  - M16: 9 red incl. corpus golden by hash. M17 and M18: nan_and_out_of_range_floats, match_rule_table.
  - Not run under mutation: the wasm tests.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — How AC-3 reaches 2,357,504: the text names the groups but not the counts or range checks. It is 14 match, 14 past-by-step and 7 past-by-level values per in-range pair, plus 65,536 read-backs; I recovered it by a search.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. One design cell is loose: M3's "matrix stays green" depends on how the f64 mutant is written.

3. **If you did this task again, what would you do differently?**
   — Launch the mutation driver as a tracked background task, not a detached process: it died before M12 and the restore check and had to be resumed by hand. No design call turned out wrong.

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
