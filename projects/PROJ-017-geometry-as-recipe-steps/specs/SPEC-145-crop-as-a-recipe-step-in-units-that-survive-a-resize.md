---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-145
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-017
  stage: STAGE-058
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: claude-opus-4-7     # usually same Claude, different session
  created_at: 2026-11-04

references:
  decisions:
    - DEC-112
    - DEC-095
    - DEC-102
    - DEC-109
    - DEC-110
    - DEC-091
    - DEC-088
    - DEC-106
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

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-058's <capability>". Optional; null is acceptable.
value_link: "The first of STAGE-058's five geometry ops, and the unit rule (a DEC) that lets a crop made on the ddphotos page's preview replay on the full-size original"

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
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 7291832
      tokens_breakdown:
        input: 106
        output: 45408
        cache_creation: 160436
        cache_read: 7085882
      estimated_usd: 5.68
      duration_minutes: null
      recorded_at: 2026-11-04
      note: >
        Interactive frame session, worktree docs/spec-145-frame. Measured from transcript
        ba7db560-f24e-4992-a32c-37af6e74c570.jsonl, found by the prompt's title, deduped by
        `.message.id` (53 unique ids, all claude-opus-5-5): input/cache_creation/cache_read once
        per id, MAX output_tokens per id. Rates: Opus $5 in / $25 out per MTok, cache_creation
        x1.25 input, cache_read x0.10 input. Taken before the commit/push exchanges, so it
        slightly under-counts the session.
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 8136660
      tokens_breakdown:
        input: 127
        output: 77211
        cache_creation: 274596
        cache_read: 7784726
      estimated_usd: 7.54
      duration_minutes: null
      recorded_at: 2026-11-05
      note: >
        Interactive design session, worktree docs/spec-145-design. Measured from transcript
        a34d9c70-8a2c-4971-a998-fbcb33183f2f.jsonl, found by the prompt's title, plus its one
        Explore subagent (subagents/agent-ad77a099e630fca0a.jsonl, the docs inventory), deduped by
        `.message.id`: main 48 ids, subagent 16 ids, all claude-opus-5-5; input/cache_creation/
        cache_read once per id, MAX output_tokens per id. Main $6.45 + subagent $1.09. Rates: Opus
        $5 in / $25 out per MTok, cache_creation x1.25 input, cache_read x0.10 input. Taken before
        the commit/push exchanges, so it slightly under-counts the session.
    - cycle: build
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 24033759
      tokens_breakdown:
        input: 222
        output: 85576
        cache_creation: 261455
        cache_read: 23686506
      estimated_usd: 9.37
      duration_minutes: null
      recorded_at: 2026-11-05
      note: >
        Interactive build session, worktree feat-spec-145-crop. Measured from transcript
        84331158-b19d-4632-bf59-252f40e40a91.jsonl, found by the prompt's title, deduped by
        `.message.id` (108 unique ids, all claude-sonnet-5-5): input/cache_creation/cache_read
        once per id, MAX output_tokens per id. Rates: Sonnet $3 in / $15 out per MTok,
        cache_creation x1.25 input, cache_read x0.10 input. Taken before the final commit and
        push, so it slightly under-counts the session.
    - cycle: verify
      agent: claude-opus
      interface: claude-code
      tokens_total: 8626031
      tokens_breakdown:
        input: 122
        output: 29837
        cache_creation: 163088
        cache_read: 8432984
      estimated_usd: 5.98
      duration_minutes: null
      recorded_at: 2026-11-05
      note: >
        Read-only verify session; it measured its own cost but could not write it, so the punch-list
        session recorded it from the verify readout. Rates: Opus $5 in / $25 out per MTok,
        cache_creation x1.25 input, cache_read x0.10 input.
    - cycle: punchlist
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 3025187
      tokens_breakdown:
        input: 74
        output: 15149
        cache_creation: 68689
        cache_read: 2941275
      estimated_usd: 1.37
      duration_minutes: null
      recorded_at: 2026-11-05
      note: >
        Punch-list session, worktree fix-spec-145-punch. Measured from transcript
        fe38dbc6-9a70-4f27-b79e-db9e1d5830ce.jsonl, found by the prompt's title, deduped by
        `.message.id` (37 unique ids, all claude-sonnet-5-5), MAX output_tokens per id. Rates:
        Sonnet $3 in / $15 out per MTok, cache_creation x1.25, cache_read x0.10. Taken before the
        commit, push and CI exchanges, so it slightly under-counts the session.
    - cycle: reverify
      agent: claude-opus
      interface: claude-code
      tokens_total: 1622265
      tokens_breakdown:
        input: 38
        output: 12955
        cache_creation: 64057
        cache_read: 1545215
      estimated_usd: 1.50
      duration_minutes: null
      recorded_at: 2026-11-05
      note: >
        Read-only re-verify session; it measured its own cost but could not write it, so
        punch list 2 recorded it from the re-verify readout. Rates: Opus $5 in / $25 out per MTok,
        cache_creation x1.25 input, cache_read x0.10 input.
    - cycle: punchlist-2
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 998294
      tokens_breakdown:
        input: 26
        output: 7144
        cache_creation: 54768
        cache_read: 936356
      estimated_usd: 0.59
      duration_minutes: null
      recorded_at: 2026-11-05
      note: >
        Punch-list 2 session, worktree fix-spec-145-punch2. Measured from its transcript, found
        by the prompt's title, deduped by `.message.id` (13 unique ids, all claude-sonnet-5-5),
        MAX output_tokens per id. Rates: Sonnet $3 in / $15 out per MTok, cache_creation x1.25,
        cache_read x0.10. Taken before the commit, push and CI exchanges, so it slightly
        under-counts the session.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-145: crop as a recipe step, in units that survive a resize

## Context

A recipe can't crop today. The registry holds five ops (`identity`, `invert`, `resize`,
`auto-orient`, `watermark`; `with_builtins`, `src/operation/registry.rs:294`), and the only crop in the code
is `resize mode = "fill"`'s internal centre crop (`crop_center`, `src/operation/mod.rs:595`),
which always resizes first and is always centred. So a recipe can't cut a free rectangle or
take an aspect from one side of the frame, and the ddphotos edit page can't offer crop because
it can only hand the CLI a recipe.

This is STAGE-058's second backlog item and PROJ-017 build-order line 2 of 13, on the
critical path: `pad`, `trim`, crop on the page (STAGE-060) and the `edit` flags all wait on it.
It also absorbs the stage's later docs item ("`docs/recipes.md` still advertises a `crustyimg
crop` verb").

The risk is the one PROJ-017's brief names first. The page edits a downscaled copy and the CLI
replays on the full-size original, so a crop written in the copy's pixels lands in the wrong
place. Both sides exit 0, so nothing reports it.

## Goal

A recipe can say `op = "crop"` with a rectangle or an aspect plus `gravity`, in units that mean
the same region at any image size. The ddphotos page can then write a crop with no knowledge of
the original's size, and `apply` cuts the same region from the original, within half a pixel per
edge.

## Frame

### Verdict: GO, [M]

Go at M, as one spec with both forms. There's a pre-named split (below) if design's refusal
table grows past about 20 rows. **One maintainer ruling is needed before design** (Q1): this
frame recommends **fractions only** in a recipe, against STAGE-058's design note "carry both",
because DEC-091 already rules that absolute pixel coordinates are not admissible in the main
engine. Design can't write the unit DEC until that's ruled.

### Premise check (evidence, on `main` at `995098e`, crustyimg 0.8.2, release build in a scratch target dir)

- **No crop op or key exists.** `/usr/bin/grep -rn -i crop src/operation/` finds only `resize`'s
  `fill` path (`crop_center`, `resize_and_crop8/16`) and one doc line on `Gravity`
  (`mod.rs:1152`, "reusable by a future `crop`"). `with_builtins` registers five ops, no crop.
- **Today's refusal, driven:** a v2 recipe with `[[step]] op = "crop"`, `x = 0.1`, `y = 0.1`,
  `width = 0.5`, `height = 0.5`, run with `crustyimg apply --recipe crop.toml in.png -o out.png`,
  gives `error: unknown operation 'crop'`, **exit 1**. The message doesn't name the step index
  (filed below as an out-of-spec finding).
- **An apply-time op failure** is also exit 1: `resize mode = "exact"` at 60000×60000 gives
  `error: operation 'resize' failed: target dimensions 60000×60000 exceed the maximum edge
  limit of 50000 px`. That is the shape a crop apply-time refusal inherits.
- **Resize's internal crop:** `fill` scales with `s = max(W/w, H/h)` and then centre-crops with
  `x = (dw - target_w) / 2` (integer floor), via `imageops::crop_imm`. `cover` scales the same
  and **does not crop**. The code, its unit tests (`resize_fill_center_crops_exact`) and
  `docs/cli-reference.md:103-104` and `docs/recipes.md:54-56` agree today. (The 2026-08 audit's
  "documented backwards" finding is fixed in those two files. This frame did not re-read
  `recipes/README.md`'s mode table.) **`crop` does not reuse `crop_center`**: that helper is
  centre-only and typed to the resize working buffer. It does reuse `imageops::crop_imm` (already
  a dependency) and `Gravity::placement` (`mod.rs:1227`, margin 0) for the aspect form.
- **`percent` is the nearest size-independent unit.** Its table row is `ParamKind::Number { min:
  0.0, min_inclusive: false, max: None }`, read as `f32` (`get_f32`), and it rounds with
  `((w as f64 * p).round() as u32).max(1)` (`mod.rs:948-955`), so `percent` clamps a zero result
  up to 1 px. `crop` shares the `Number` kind and the `f64::round` but **not** the spelling
  (`percent` is a 0-100 scale, a single key, and has no upper bound) and **not** the `.max(1)`
  (see the refusal table). No `get_f64` accessor exists (`mod.rs:85-107`). Design adds one or
  reads `f32` (both give exact pixels; the round trip below was checked in f64).
- **The integer/float split is already live:** `resize mode = "max"` with `width = 800.0` is
  refused at load (`recipe step 0 ('resize'): 'width' must be an integer >= 1, got the float
  800.0`). So "an integer is pixels, a float is a fraction" would give `1` and `1.0` different
  meanings in a format where `resize` already treats `800.0` as an error.
- **wasm gets the op for free:** `src/wasm.rs:234` builds `OperationRegistry::with_builtins()`, so a
  registered `crop` runs in `transform` with the same table and code.
- **`apply` adds no orientation step.** The verbs build on `auto_orient_prefix()`
  (`src/cli/optimize.rs:883-896`), but the recipe lane runs only the recipe's steps. The page's
  recipes always start with `auto-orient` (`integrations/ddphotos/edit/recipe.js:97`).

### The trap, in numbers

The page edits ddphotos' published copy (1600 px long edge, `integrations/ddphotos/README.md:124`).
The original is 6000×4000, so the copy is **1600×1067**. A user drags a crop on the copy at
x=400, y=300, 800×500 px: the centre half of the width.

| recipe carries | lands on the 6000×4000 original | region |
|---|---|---|
| copy pixels (`x=400 y=300 w=800 h=500`) | columns 400-1200, rows 300-800 | **13.3 % × 12.5 % of the frame, top-left**; meant 50 % × 46.9 %, centred. Exit 0 |
| fractions (`x=0.25 y=0.28116… w=0.5 h=0.46860…`) | columns **1500-4500**, rows **1125-2999** (3000×1874) | the exact scaled edges are 1500, 4500, 1124.65 and 2999.06, so each edge is within 0.5 px |

The same fractions at STAGE-058's exit-criterion sizes: on 1200×800, edges 300/900/225/600, and on
6000×4000, edges 1500/4500/1125/2999. Over 200,000 random fractions, the largest
`|edge_6000 / 5 - edge_1200|` was **0.40 px**. The rule's loose bound is `0.5 + 0.5/5 = 0.6` px;
drift comes in multiples of 0.2 and halves round up, so in exact maths 0.4 px is the largest. In
`f64` it is not: `x * 1200` and `x * 6000` are rounded separately, so near a half-pixel tie the two
sizes can round in opposite directions. **0.4 px is what the seeded sweep measures; 0.6 px is the
bound in general, reached at `f64` near-ties.** Driven on the binary: `width = 0.009583333333333333`
(`x = 0`, `y = 0`, `height = 1`) gives a 12 px crop at 1200 and 57 px at 6000, and 57/5 = 11.4 is
a drift of 0.6. The test asserts 0.4 for its fixed seed only.

Today's page avoids this for the watermark only because `cliRecipe` always resizes to the page's
long edge first (`recipe.js:111-118`, "the watermark size is in pixels"). That workaround doesn't
work for crop. Crop must come **before** the resize, or the output loses the original's
resolution. And a pixel crop after a resize still drifts by a pixel, because the copy's and the
original's short sides round independently (SPEC-142's frame, 6400×4286 vs 1600×1072).

### Recommended unit syntax (content for the unit DEC; design writes it)

**Fractions only, written as TOML numbers from 0 to 1.** `x`, `y`, `width` and `height` are each a
fraction of the image's width (`x`, `width`) or height (`y`, `height`), as the buffer reaches the
step.

```toml
[[step]]
op = "crop"
x = 0.25
y = 0.28
width = 0.5
height = 0.47
```

- **Table rows:** `x` and `y` are `Number { min: 0.0, min_inclusive: true, max: Some(1.0) }`;
  `width` and `height` are `Number { min: 0.0, min_inclusive: false, max: Some(1.0) }`. TOML
  integers `0` and `1` are accepted as 0.0 and 1.0 (the `Number` kind already does this), so
  `x = 0` and `width = 1` aren't ambiguous: there are no pixels to confuse them with.
- **A pixel-minded author is refused, not misread:** `width = 800` →
  `recipe step 0 ('crop'): 'width' must be a number in 0.0..=1.0, got 800`. Design should
  consider a crop-specific hint ("crop lengths are fractions of the image"). The one silent
  case left is `width = 1` meaning "1 px": it's read as the full width. Name it in the docs.
- **Why it's DEC-091-clean:** Fence A says a workhorse parameter must mean the same thing on a
  different image. Fractions, aspects and gravities do. Absolute pixel coordinates don't, and
  "absolute rect → lab" is the fence's own worked example (DEC-091, Context §1).
- **The CLI user who wants pixels** gets them from `edit --crop 800x600+10+10` (STAGE-058's
  `edit`-flags spec). `edit` has the image, so it converts to `px / W`, and `--save-recipe` writes
  fractions. **This round trip is exact:** for every rectangle on every width W ≤ 300
  (4,545,100 rectangles, exhaustive), and for 2,000,000 random rectangles with W ≤ 50,000 (the
  `MAX_EDGE`), `round((a/W)·W) = a`, `round((a/W + b/W)·W) = a + b`, and `a/W + b/W ≤ 1.0`,
  with 0 mismatches. So `--save-recipe` replays to identical pixels, which is that spec's test.
  The page's own `x + width` never trips the `≤ 1` check for the same reason: for every
  `i/n + (n−i)/n` with n ≤ 3000 (4,498,500 pairs) the f64 sum is never above 1.0.

**Alternatives rejected:**
- **Both units, told apart by TOML type** (integer = pixels, float = fraction). This is what
  STAGE-058's design note asks for. It's rejected because `width = 1` and `width = 1.0` would mean
  different things, and `resize` already refuses `800.0` for a pixel key, so the convention would
  flip between ops. It needs a `ParamKind` whose type depends on the value, which is DEC-102's
  "Revisit if" trigger. And it puts lab-side parameters in the workhorse against DEC-091 unless
  that DEC is amended (Q1).
- **`"10%"` strings.** These need a new string-parsing `ParamKind`. A 0-100 scale also collides
  with a 0-1 aspect of the same image, and the page would format strings for no gain over a
  float.
- **A separate key family** (`x_px` … beside `x` …). This doubles the table and the
  exclusivity rules, and the pixel family is still lab-side under Fence A.

### Recommended rounding rule (content for the unit DEC)

**Round each edge, not the origin and the size.** For a W×H buffer:

```
left   = round(x · W)             right  = round((x + width) · W)
top    = round(y · H)             bottom = round((y + height) · H)
crop   = (left, top, right − left, bottom − top)
```

- The arithmetic is f64, with `f64::round`, which rounds halves away from zero (half-up for these
  non-negative values). It's IEEE multiply and round, so native and wasm32 give the same integers
  by construction. Like DEC-110's dimensions limb, crop agrees *exactly* across targets, and
  crop never resamples, so the pixels agree exactly as well.
- **It stays in bounds by construction, with no clamp:** load refuses `x + width > 1`, and
  `(x + width) · W ≤ W` for any f64 value ≤ 1, so `right ≤ W`.
- **Why edges and not size:** rounding the size separately lets the far edge drift by up to
  1 px, so two crops that abut in fractions would overlap or leave a gap. With edges, each edge
  is within 0.5 px of its exact position on every image size, and that is the testable claim.
- **Named tests** (design writes them): `crop_edges_within_half_a_pixel`, a property over every
  W in 1..=N and a fraction grid, asserting `|edge − f·W| ≤ 0.5`. Its mutations are floor
  instead of round, and size rounded instead of the right edge; each must go red.
  `crop_fraction_agrees_at_1200_and_6000`, the exit criterion, asserting `|e_6000/5 − e_1200| < 1`
  for every edge.
- **The aspect form** uses exact integer maths in u64 with no float. For `aspect = "a:b"` on W×H,
  if `W·b ≤ H·a`, the crop is the full width W and a height of `(2·W·b + a) div (2a)` (round
  half-up). Otherwise it's the full height H and a width of `(2·H·a + b) div (2b)`. Either side is
  ≤ its image side by construction. It's placed by `Gravity::placement(W, H, cw, ch, 0)`, so
  centring floors, as in `resize fill`.

### The two forms: one op, exclusive key groups, no `mode` key

- **Rectangle:** `x`, `y`, `width`, `height`, all four required. **Aspect:** `aspect` (required)
  and `gravity` (optional, default `center`).
- **The precedent is watermark's `image` XOR `text`, not resize's `mode`.** `resize` needs `mode`
  because its modes share keys (`width`/`height`) with different meanings. Crop's two groups share
  no key, so the keys present pick the form, and a `mode` key would be one more thing to get
  wrong.
- **Mixed keys** are refused at load by the constructor, naming one key from each group:
  `'aspect' cannot be combined with 'x'; a crop is a rectangle (x, y, width, height) or an aspect
  (aspect, gravity)`. `gravity` with a rectangle is refused as unused, which is the DEC-102 rule
  for a key the form never reads.

### Refusals: load vs apply

Load means `RecipeError::InvalidOperation`, displayed as `recipe step N ('crop'): <reason>`,
exit 1, with no image read (DEC-102). Apply means `operation 'crop' failed: <reason>`, exit 1, as
today's op failures are.

| bad crop | when | who checks | message shape |
|---|---|---|---|
| unknown key (`left = 0.1`) | load | table | `unknown key 'left'; expected one of: x, y, width, height, aspect, gravity` |
| wrong type (`x = "10%"`, `width = true`) | load | table | `'x' must be a number in 0.0..=1.0, got the string "10%"` |
| negative (`x = -0.1`) | load | table | `… got -0.1` |
| a fraction over 1 (`width = 1.5`, `width = 800`) | load | table | `… got 800` |
| nan / inf | load | table | `… got nan` |
| zero size (`width = 0`, `height = 0.0`, `width = 1e-50`, which is 0 as an f32: trap 9) | load | table (`min_inclusive: false`) | `'width' must be a number > 0 and <= 1, got 0` |
| out of bounds (`x + width > 1`, `y + height > 1`) | load | constructor | `x + width is 1.2, past the image's right edge (1.0)` |
| partial rectangle (no `height`) | load | constructor | `a rectangle crop requires 'height'` |
| no keys at all | load | constructor | `crop requires a rectangle (x, y, width, height) or an aspect` |
| mixed forms (`aspect` and `x`) | load | constructor | as above |
| `gravity` with a rectangle | load | constructor | `'gravity' is not used by a rectangle crop` |
| malformed aspect (`"16x9"`, `"16:"`, `"a:b"`, `"1.5:1"`) | load | constructor (a `Text` row, parsed like `color`'s hex) | `'aspect' must be two whole numbers W:H, got "16x9"` |
| zero or negative aspect (`"0:9"`, `"-1:1"`) | load | constructor | as above |
| bad gravity | load | table (`OneOf(GRAVITY_NAMES)`, as watermark has) | `'gravity' must be one of: …; got "middle"` |
| a rectangle that rounds to 0 px (`width = 0.0001` on a 1600 px image: right == left) | **apply** | `apply` | `the crop rounds to 0×1067 px on a 1600×1067 image` |
| an aspect whose short side rounds to 0 (`"10000:1"` on a 1600×1067 image) | **apply** | `apply` | same shape |
| **pixel out of bounds** | n/a under fractions only | — | if Q1 admits pixels: **refuse at apply, never clamp**. A clamp changes the region and the aspect at exit 0, which is this spec's own trap |

**Refuse, not `.max(1)`, at apply:** a 0 px crop has no meaning, and the precedent is DEC-109 (a
watermark that would draw nothing is refused), not `resize percent`'s 1 px floor. A consequence
to accept: a recipe valid on a 6000 px original can be refused on a 40 px thumbnail. Aspect
integers get an upper bound in the table or the parser (for example 1..=10,000 each), so the u64
maths can't overflow and a hostile `"18446744073709551615:1"` is a load refusal.

### Colour type and bit depth (DEC-095)

`crop` slices and never computes a pixel. `DynamicImage::crop_imm` keeps the variant, so this is
exact and needs no widen/narrow step. **Fixtures:** SPEC-141's corpus already builds the three
DEC-095 cases from seeded samples (`tests/recipe_corpus_output.rs:49`: `rgb8_2600x1730`,
`rgba8_1730x2600` portrait, `rgba16_2600x1730`). Two corpus recipes (one rectangle, one aspect
with a non-centre gravity) give 6 pixel rows that pin colour type, depth and a SHA-256 per
fixture. Because crop has no float maths, design can also assert that its three per-target
`.output` files are identical, which no resize recipe can claim. Add a unit row for `L8`/`La8`
(luma isn't in the corpus) and the identity crop (`0, 0, 1, 1` → the input's buffer, byte-equal).

**How the build proves no existing line changed:**
`git diff --diff-filter=M --name-only origin/main -- tests/fixtures/recipe_corpus/` is empty. Only
added (`A`) files appear there, and the literal bumps in `recipe_corpus_output.rs` /
`recipe_round_trip.rs` follow the README's table. The corpus test itself stays green with no
re-bless of an existing file.

### EXIF orientation: defer, with a stated rule

Crop doesn't decide orientation. **The rule for this spec:** `x` and `y` are in the pixel buffer
the step receives. That's the stored orientation unless an earlier `auto-orient` step baked it,
and `apply` adds none. The page always does (`recipe.js:97`), so its fractions are display
coordinates. The `crop` doc entry says to put `auto-orient` first for display coordinates. What a
surviving orientation tag means after a geometry step, and the stage's `PixelXDimension` note,
belong to the rotate/flip spec (STAGE-058's next item), which owns the full question.

### Native vs wasm (DEC-110)

Crop is a no-resample op, so DEC-110 limb 2 applies: dimensions and pixels exact. The rounding is
target-independent by construction (above). Design adds one row to `tests/wasm_roundtrip.rs` (run
by `just wasm-test` in a real wasm VM): the same fraction recipe on the same input gives the same
dimensions as native. **Replay parity** ("(b)" in SPEC-142's frame: the page's recipe on the copy
vs the CLI's on the original) and crop rows in `tests/ddphotos_parity.mjs` belong to **STAGE-060's
crop-on-page spec**, because the harness builds recipes only through `recipe.js`, and the page
emits no crop until then. ⚠ Trap 5: the wasm bundle-size gate isn't in `just check`. Crop adds a
little code, so the build reads the CI gate.

### Docs

- `docs/recipes.md:60-62`: "Crop to an arbitrary rect / gravity / aspect **[planned — PROJ-006]**"
  with `crustyimg crop …` becomes a **recipe-step** entry (`op = "crop"`, a TOML snippet, a
  fractions note, and an `auto-orient`-first note), tagged with PROJ-017. There's no verb: the set is
  frozen at 14 (STAGE-030).
- `docs/recipes.md:64-65`: "Smart (content-aware) crop **[planned — PROJ-006]**" with `crustyimg
  crop --smart …` loses the verb and points at the roadmap's post-1.0 beta row
  (`docs/roadmap.md:80`). PROJ-017 keeps smart crop out.
- **Every `crustyimg crop` promise:** `/usr/bin/grep -rn "crustyimg crop"` outside `target/` and
  `.git/` finds exactly `docs/recipes.md:61` and `:65`, plus the STAGE-058 backlog text. Both are
  covered above.
- The op tables gain a `crop` row. Design inventories them: `recipes/README.md` (op table near
  :88), `skills/crustyimg/SKILL.md`, `docs/api-contract.md`, `docs/data-model.md` and
  `docs/cli-reference.md`'s `apply` section all name `auto-orient` and are candidates. Fix
  `docs/recipes.md:63` ("Rotate / flip / trim / pad [planned — PROJ-006]") only for its PROJ id,
  if design chooses; each of those ops' own specs owns its entry.

### `edit --crop`: out

Recipe-only here. The stage backlog puts all five `edit` flags in one later spec with the ordering
DEC, and a crop flag is exactly where ordering bites (crop-then-resize vs resize-then-crop). That
spec also owns the pixel→fraction conversion and its `--save-recipe` round trip, which this frame
measured as exact.

### Sizing: M, and why

The code is small: a `CROP_PARAMS` table, a `Crop` struct with `from_params` / `apply` /
`params()`, one registration, an aspect parser and a `get_f64` accessor (about 250-350 lines with
unit tests). **The verification is what makes it M.** It covers 2 forms; about 17 load-refusal
rows and 2 apply rows (each with a mutation); the edge-rounding property and the two-size test;
9 gravities placed exactly; 3 corpus colour types plus luma unit rows; the identity crop; one wasm
VM row; a corpus addition with literal bumps on three targets; and about 6 doc surfaces. That's
at the top of M, not an L.

**Pre-named split, if design finds it heavier:** (a) the unit DEC and the rectangle form, then
(b) the aspect form with gravity. STAGE-060's page crop needs only (a), because the page drags a
rectangle, so (a) alone unblocks the critical path.

### Scope

**In:** the `crop` op (rectangle + aspect) in the registry, so it runs in native and wasm; its key
table and constructor refusals; apply-time zero-size refusals; the unit-and-rounding DEC; the
corpus recipes and goldens; the wasm row; the `docs/recipes.md` crop and smart-crop fixes and the
op-table rows.

**Out, and where each lives:**
- `edit --crop` and the geometry ordering DEC: STAGE-058's `edit`-flags item.
- The page's crop UI, crop rows in `ddphotos_parity.mjs`, and replay parity (b): STAGE-060's
  crop-on-page item.
- Orientation-tag semantics after geometry, and EXIF dimensions: STAGE-058's rotate/flip item.
- Pixel units in a recipe: only if Q1 rules them in (then this spec grows, or a follow-up carries
  them).
- Smart/content-aware crop: post-1.0 beta (roadmap), out of PROJ-017.
- Art direction / og-card crops: PROJ-014.
- Step indices in apply-time and unknown-op errors: filed to `guidance/ideas.yaml`.

### Rough build file list

- `src/operation/mod.rs`: `CROP_PARAMS`, `Crop` (or a new `src/operation/crop.rs` re-exported
  there), `OperationParams::get_f64`, an aspect parser, and `Gravity::placement` reachable from
  crop.
- `src/operation/registry.rs`: `register_with_params("crop", …)` in `with_builtins`, and its
  registry test.
- `tests/crop.rs` (new): refusal rows through `apply`, rounding property, two-size test,
  gravities, colour types.
- `tests/fixtures/recipe_corpus/v2_crop_rect.{toml,golden,<3 targets>.output}` and
  `v2_crop_aspect.*` (new). `tests/recipe_corpus_output.rs` and `tests/recipe_round_trip.rs` get
  literal bumps only.
- `tests/wasm_roundtrip.rs`: one crop row.
- `decisions/DEC-NNN-crop-units-are-fractions-and-round-per-edge.md` (written by design).
- `docs/recipes.md`, `recipes/README.md`, `skills/crustyimg/SKILL.md`, and the other op tables
  design finds. `CHANGELOG.md` under Added.

### Open maintainer questions

1. **Pixels in a recipe?** STAGE-058's note says "carry both"; DEC-091 Fence A says absolute pixel
   coordinates are lab-side. **Recommend fractions only** in recipes. Pixels live in `edit
   --crop`, which converts exactly. No DEC-091 amendment is needed. (The alternative is to amend
   Fence A for crop and take the int/float key ambiguity.)
2. **Aspect spelling:** whole numbers only (`"16:9"`, `"239:100"` for 2.39:1), or decimals too
   (`"2.39:1"`)? **Recommend whole numbers.** The maths stays exact integer arithmetic, and any
   decimal ratio has a whole-number spelling.
3. **A crop that rounds to 0 px at apply: refuse, or clamp to 1 px like `resize percent`?**
   **Recommend refuse** (the DEC-109 precedent).

## Design

Designed 2026-11-05 on `main` at `3a412d0` (crustyimg 0.8.2), against the maintainer's rulings of
2026-11-05 (fractions only; whole-number aspects; refuse 0 px). **The unit rule is DEC-112.** One
spec, both forms, **no split** (sizing at the end of this section).

### What was driven at design (release build, scratch `CARGO_TARGET_DIR`, deleted after)

| probe | result | control |
|---|---|---|
| `apply --recipe crop.toml in.png -o out.png` (rect crop, v2) | `error: unknown operation 'crop'`, exit 1 | `resize mode = "max" width = 80` on the same input: exit 0, 80×54 PNG |
| the same crop recipe, two inputs | `error: unknown operation 'crop'`, exit 1 (load fails before any input) | — |
| unknown key on a registered op (`left = 0.1` on resize) | `error: recipe step 0 ('resize'): unknown key 'left'; expected one of: mode, width, height, percent`, exit 1 | — this is the line shape every crop load refusal takes |
| apply-time op failure, one input (`resize exact 60000×60000`) | `error: operation 'resize' failed: …`, **exit 1**, nothing written | — |
| the same, two inputs with `--out-dir` | one `error: <input>: operation 'resize' failed: …` line per input, `error: 2 of 2 inputs failed`, **exit 6**, nothing written | — crop's apply failure inherits this; no new exit logic |

**Two code facts that shape the design:**
- `imageops::crop_imm` **clamps silently** (`image-0.25.10/src/imageops/mod.rs:67-83`,
  `crop_dimms`: `width = min(width, iwidth - x)`). So crop must never hand it an out-of-bounds
  rectangle, and the bounds mutation below cannot rely on a panic.
- Native recipe pipelines run with `OnRefusal::Skip` (`src/cli/common.rs:89`, used by `apply`;
  `src/cli/ops.rs:494`; `src/cli/optimize.rs:1433,1714`), and a skipped step still writes the image
  ("A skipped watermark is not a failure: the image was written", `src/cli/optimize.rs:305`). wasm
  runs `pipeline.run` (`OnRefusal::Fail`, `src/wasm.rs:243`). **So the 0 px check lives in
  `Crop::apply`, returning `OperationError::Apply`, and `Crop` does not override
  `Operation::refusal`.** Through `refusal()` a 0 px crop would be skipped natively (exit 0, the
  uncropped image written) and an error in wasm: the trap this spec exists to close, plus a
  native/wasm split.

### The op, in code terms

- **Where:** a new `src/operation/crop.rs`, declared `pub mod crop;` in `src/operation/mod.rs`, with
  `pub use crop::{Crop, CROP_PARAMS};`. A child module can call the private
  `Gravity::placement` (`src/operation/mod.rs:1227`) and read `GRAVITY_NAMES` (`:1168`), so neither
  is widened. `mod.rs` is already 3,637 lines.
- **`OperationParams::get_f64(&self, key) -> Option<f64>`** (`src/operation/mod.rs`, beside
  `get_f32` at `:98`): `Float(f) => Some(*f)`, `Integer(i) => Some(*i as f64)`, else `None`. Crop
  reads every fraction with it. (DEC-112 says why not `f32`: `x = 0.0009375` on 1600 px.)
- **`CROP_PARAMS`**, in this order (the order the unknown-key message lists):

  | key | `ParamKind` |
  |---|---|
  | `x` | `Number { min: 0.0, min_inclusive: true, max: Some(1.0) }` |
  | `y` | `Number { min: 0.0, min_inclusive: true, max: Some(1.0) }` |
  | `width` | `Number { min: 0.0, min_inclusive: false, max: Some(1.0) }` |
  | `height` | `Number { min: 0.0, min_inclusive: false, max: Some(1.0) }` |
  | `aspect` | `Text` (parsed by the constructor, like watermark's `color`) |
  | `gravity` | `OneOf(GRAVITY_NAMES)` |

  `ParamKind::expectation` (`src/operation/registry.rs:115-133`) renders these as
  `a number in 0.0..=1.0` and `a number > 0 and <= 1`. No registry change is needed, and **no
  crop-specific "these are fractions" hint is added**: the table check runs before the constructor
  (`registry.rs:423`), so a hint would need a registry change for a message that already says
  `<= 1, got 800`. The docs carry it.
- **Registration:** `reg.register_with_params("crop", |p| Ok(Box::new(Crop::from_params(p)?)), CROP_PARAMS);`
  in `with_builtins` (`registry.rs:294-312`), after `watermark`. Update the doc comments there and
  at `:271-273` that list the built-ins.
- **`Crop::from_params(&OperationParams) -> Result<Crop, RegistryError>`** runs
  `registry::check_params("crop", CROP_PARAMS, params)` first (as `Resize::from_params` does at
  `mod.rs:794`), then, in this order:
  1. `aspect` present **and** any of `x`, `y`, `width`, `height` present → the mixed-forms refusal,
     naming the first rectangle key present in table order.
  2. no `aspect`, `gravity` present → the gravity refusal (with or without rectangle keys).
  3. no `aspect`, no rectangle key → the no-keys refusal.
  4. no `aspect`, some rectangle keys → the first missing of `x`, `y`, `width`, `height` → the
     partial refusal.
  5. rectangle: `x + width > 1.0` → right-edge refusal; then `y + height > 1.0` → bottom-edge
     refusal. Both sums in `f64`.
  6. aspect: parse (below); `gravity` defaults to `center`.
- **`Crop` holds** the form (`Rect { x, y, width, height }` as `f64`, or `Aspect { a, b, gravity }`
  with `a`, `b` as `u32`) **and the step's original map**, so `params()` returns the keys exactly as
  written (an integer `x = 0` stays an integer, an absent `gravity` stays absent). `params()` is not
  what `to_toml()` serializes (the recipe keeps its own map, DEC-102), but `edit --save-recipe` will
  read it later, so it must round-trip.
- **The rectangle maths**, one private pure function both `apply` and the unit tests call:
  `fn rect_edges(x: f64, y: f64, w: f64, h: f64, iw: u32, ih: u32) -> (u32, u32, u32, u32)` returning
  `(left, top, right, bottom)` per DEC-112: `left = (x * iw as f64).round() as u32`,
  `right = ((x + w) * iw as f64).round() as u32`, and the same for `top`/`bottom` with `ih`.
- **The aspect maths**, one private pure function: `fn aspect_size(a: u32, b: u32, iw: u32, ih: u32)
  -> (u32, u32)`, in `u64`:
  - if `iw·b ≤ ih·a` (the image is at least as tall as a:b): `cw = iw`,
    `ch = (2·iw·b + a) div (2·a)`;
  - else: `ch = ih`, `cw = (2·ih·a + b) div (2·b)`.

  That is `round(iw·b/a)` with halves up, in integers. The short side is at most its image side:
  in the first branch `iw·b/a ≤ ih`, and rounding a value ≤ `ih` half up cannot pass the integer
  `ih`. It is **0 when `2·iw·b < a`** (first branch) or **`2·ih·a < b`** (second): `"10000:1"` on
  1600×1067 gives `ch = (3200 + 10000) div 20000 = 0`. Placement is
  `Gravity::placement(iw, ih, cw, ch, 0)`: margin 0, centring floors (`(iw − cw) / 2`), as in
  `resize fill`. Equivalent ratios give identical crops (`"32:18"` = `"16:9"`), because both
  expressions scale numerator and denominator by the same factor.
- **Worked example (numbers that don't divide evenly):** `aspect = "5:4"` on **2600×1730**:
  `2600·4 = 10400 > 1730·5 = 8650`, so the crop is full height. `cw = (2·1730·5 + 4) div 8 =
  17304 div 8 = 2163` (exact value 2162.5, half rounds up), `ch = 1730`. Slack `2600 − 2163 = 437`:
  `center`/`north`/`south` put it at x = 218 (218.5 floored), the three `…east` at 437, the three
  `…west` at 0; y is 0 for all nine. On the portrait **1730×2600**: `1730·4 = 6920 ≤ 2600·5 =
  13000`, so full width: `ch = (2·1730·4 + 5) div 10 = 1384`, slack 1216: `center`/`east`/`west` at
  y = 608, the three `north…` at 0, the three `south…` at 1216; x is 0 for all nine. Swapping W and H
  in the fit gives 1730×1384 on the landscape image, which is the aspect-maths mutation.
- **The aspect parser** (private `fn parse_aspect(&str) -> Result<(u32, u32), String>`):
  exactly one `:`; each side one or more ASCII digits and nothing else (no sign, no space, no `.`);
  each side's value 1..=65535 (parse with checked `u32` arithmetic over the digits, not
  `u32::from_str`, which accepts a leading `+`). On failure: if both sides are plain decimals
  (ASCII digits with exactly one `.` in at least one side, digits on both sides of it), scale both by
  10^d (d = the longer fractional part), divide by their gcd, and if both results are in 1..=65535
  give the decimal message with that spelling; otherwise (overflow included, with checked maths)
  give the generic message. The bound keeps the `u64` maths far from overflow
  (`2 · u32::MAX · 65535 < 2^64`).
- **`Crop::apply`**: read `iw`, `ih` from `img.pixels()`; compute the rectangle (`rect_edges`, or
  `aspect_size` + `placement`); **if `right > iw` or `bottom > ih`, return `OperationError::Apply`
  with `"internal: the crop's right edge {right} is past the image's {iw} px width"` (or the
  bottom/height wording)**: unreachable after `from_params`, and there so a removed load check can
  never reach `crop_imm`'s clamp; **if the width or height is 0, return the 0 px refusal**; else
  `img.pixels().crop_imm(left, top, cw, ch)` and `img.with_pixels(…)`. `DynamicImage::crop_imm`
  keeps the variant (DEC-095), with no widen and no narrow. No `.max(1)`, no `min`, no clamp.

### Exact messages

**Load** (every one is a `RecipeError::InvalidOperation`, displayed `recipe step N ('crop'): <reason>`,
`src/recipe/mod.rs:124`; exit 1 on `apply`, nothing read or written). Table rows come out of
`check_params` (`src/operation/registry.rs:227-251`) unchanged; the constructor rows are worded to
match `resize`'s (`src/operation/mod.rs:813-822`: `'{key}' is not used by mode '{mode}'`) and
`watermark`'s (`mod.rs:1573-1588`: `watermark step must set exactly one of 'image' or 'text'`).
Operands in the edge messages are printed **as written**: an integer by `to_string`, a float by the
registry's `float_text` (`registry.rs:191`, made `pub(crate)`), never by printing an `f64` sum or an
`f32`.

| id | step body (with `op = "crop"`) | who | message after `recipe step 0 ('crop'): ` |
|---|---|---|---|
| R1 | `x=0.1 y=0.1 width=0.5 height=0.5 left=0.1` | table | `unknown key 'left'; expected one of: x, y, width, height, aspect, gravity` |
| R2 | `x="10%" y=0 width=0.5 height=0.5` | table | `'x' must be a number in 0.0..=1.0, got the string "10%"` |
| R3 | `x=0 y=0 width=true height=0.5` | table | `'width' must be a number > 0 and <= 1, got true` |
| R4 | `x=-0.1 y=0 width=0.5 height=0.5` | table | `'x' must be a number in 0.0..=1.0, got -0.1` |
| R5 | `x=0 y=1.5 width=0.5 height=0.5` | table | `'y' must be a number in 0.0..=1.0, got 1.5` |
| R6 | `x=0 y=0 width=1.5 height=0.5` | table | `'width' must be a number > 0 and <= 1, got 1.5` |
| R7 | `x=0 y=0 width=800 height=0.5` | table | `'width' must be a number > 0 and <= 1, got 800` |
| R8 | `x=nan y=0 width=0.5 height=0.5` | table | `'x' must be a number in 0.0..=1.0, got nan` |
| R9 | `x=0 y=0 width=0.5 height=inf` | table | `'height' must be a number > 0 and <= 1, got inf` |
| R10 | `x=0 y=0 width=0 height=0.5` | table | `'width' must be a number > 0 and <= 1, got 0` |
| R11 | `x=0 y=0 width=0.5 height=0.0` | table | `'height' must be a number > 0 and <= 1, got 0.0` |
| R12 | `x=0 y=0 width=1e-50 height=0.5` | table (the `f32` reading is 0) | `'width' must be a number > 0 and <= 1, got 1e-50` |
| R13 | `x=0.7 y=0 width=0.5 height=0.5` | constructor | `'x' + 'width' is 0.7 + 0.5, past the right edge; together they must be at most 1` |
| R14 | `x=0 y=0.6 width=0.5 height=0.6` | constructor | `'y' + 'height' is 0.6 + 0.6, past the bottom edge; together they must be at most 1` |
| R15 | `x=0 y=0 width=0.5` | constructor | `a rectangle crop needs x, y, width and height; 'height' is missing` |
| R16 | `width=0.5 height=0.5` | constructor | `a rectangle crop needs x, y, width and height; 'x' is missing` |
| R17 | *(no keys)* | constructor | `crop needs a rectangle (x, y, width, height) or an aspect (aspect, and optionally gravity)` |
| R18 | `aspect="16:9" x=0 y=0 width=0.5 height=0.5` | constructor | `'aspect' cannot be combined with 'x'; a crop is either a rectangle (x, y, width, height) or an aspect (aspect, gravity)` |
| R19 | `x=0 y=0 width=0.5 height=0.5 gravity="north"` | constructor | `'gravity' is only used with 'aspect'; a rectangle crop is placed by x and y` |
| R20 | `gravity="north"` | constructor | same as R19 |
| R21 | `aspect="16x9"` | constructor | `'aspect' must be two whole numbers from 1 to 65535 written W:H, such as "16:9"; got "16x9"` |
| R22 | `aspect="16:"` | constructor | as R21, `got "16:"` |
| R23 | `aspect="a:b"` | constructor | as R21, `got "a:b"` |
| R24 | `aspect="+16:9"` | constructor | as R21, `got "+16:9"` |
| R25 | `aspect="0:9"` | constructor | as R21, `got "0:9"` |
| R26 | `aspect="-1:1"` | constructor | as R21, `got "-1:1"` |
| R27 | `aspect="70000:1"` | constructor | as R21, `got "70000:1"` |
| R28 | `aspect="2.39:1"` | constructor | `'aspect' must be two whole numbers written W:H; write "2.39:1" as "239:100"` |
| R29 | `aspect="1.5:1"` | constructor | `'aspect' must be two whole numbers written W:H; write "1.5:1" as "3:2"` |
| R30 | `aspect="1.00001:1"` | constructor (its whole-number spelling 100001:100000 is past 65535) | as R21, `got "1.00001:1"` |
| R31 | `aspect=1.78` | table | `'aspect' must be a string, got the float 1.78` |
| R32 | `aspect="16:9" gravity="middle"` | table | `'gravity' must be one of: center, north, south, east, west, northeast, northwest, southeast, southwest; got "middle"` |

Each row is single-fault, so its message can only come from its own check. R13 uses `0.7`, which is
inexact in both `f32` (`0.699999988…`) and `f64` sums, so printing a computed value instead of the
operands as written fails it (trap 10). R29 is the row where the naive spelling (`"15:10"`) and the
reduced one differ. R24 is the row `u32::from_str` gets wrong.

**Apply** (an `OperationError::Apply`, displayed `operation 'crop' failed: <reason>`,
`src/operation/mod.rs:168`; the CLI prefixes `error: `; exit 1 on a single-input `apply`, the existing
per-input line and exit 6 when every input of a batch fails). All on a **1600×1067** image (the
ddphotos copy size):

| id | step body | rounds to | message after `operation 'crop' failed: ` |
|---|---|---|---|
| A1 | `x=0.5 y=0 width=0.0001 height=1` | left 800, right `round(800.16)` = 800 | `the crop rounds to 0×1067 px on this 1600×1067 px image; it must keep at least 1 px each way` |
| A2 | `x=0 y=0.5 width=1 height=0.0001` | top `round(533.5)` = 534, bottom `round(533.6067)` = 534 | `the crop rounds to 1600×0 px on this 1600×1067 px image; it must keep at least 1 px each way` |
| A3 | `aspect="10000:1"` | `(3200 + 10000) div 20000` = 0 | `the crop rounds to 1600×0 px on this 1600×1067 px image; it must keep at least 1 px each way` |
| A4 | `aspect="1:10000"` | `(2134 + 10000) div 20000` = 0 | `the crop rounds to 0×1067 px on this 1600×1067 px image; it must keep at least 1 px each way` |

The messages carry no step index: apply-time errors don't today, for any op, and that is filed as
`recipe-errors-name-the-step-everywhere` (`guidance/ideas.yaml:337`). They name the op, the rounded
size and the image size, which is what ruling 3's "naming the step" can mean without that change.
**This is a reading of the ruling; the report flags it.**

### Sizing: M, one spec, no split

32 load rows and 4 apply rows is more than the frame's "about 20" split trigger. It stays one spec
because the rows are **data lines in one table-driven test each**, not separate test functions, and
the aspect form adds 12 of them (R21–R32 less R31's table half) plus one function of integer maths.
Splitting would duplicate the corpus bless, the wasm row, the docs pass and the CI round trip, which
are the expensive parts. Code: about 300–400 lines with unit tests. If the build finds the aspect
half costs more than a third of the cycle, it says so in its reflection; the split stays named for
the next op that grows this way.

## Inputs

- **Files to read:** `src/operation/mod.rs` (`OperationParams` :33-127, `OperationError` :160-183,
  `Operation` :255-286, `Resize::from_params` :780-865, `Gravity` :1147-1263, `WATERMARK_PARAMS`
  :1270-1315, `Watermark::from_params` :1561-1690); `src/operation/registry.rs` (`ParamKind`
  :76-185, `float_text` :191, `check_params` :227, `with_builtins` :294, tests :440-741);
  `src/recipe/mod.rs:124` and `:389-407`; `src/pipeline/mod.rs:79-126`; `src/cli/common.rs:72-95`;
  `src/wasm.rs:225-246`; `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs:259-333`,
  `tests/fixtures/recipe_corpus/README.md`; `tests/wasm_roundtrip.rs:311-366`; `tests/docs_ops.rs`.
- **Decisions:** DEC-112 (this spec's), DEC-091, DEC-095, DEC-102, DEC-109, DEC-110.
- **Related code paths:** `src/operation/`, `tests/`, `docs/`, `recipes/README.md`, `skills/`.

## Outputs

- **Files created:** `src/operation/crop.rs` (the op, its maths, its unit tests); `tests/crop.rs`
  (load and apply refusal tables, CLI rows, DEC-095 exactness);
  `tests/fixtures/recipe_corpus/v2_crop_rect.{toml,golden,linux-x86_64.output,macos-aarch64.output,windows-x86_64.output}`
  and the same five for `v2_crop_aspect`.
- **Files modified:** `src/operation/mod.rs` (`pub mod crop;`, re-export, `get_f64`);
  `src/operation/registry.rs` (registration, doc lists, `float_text` → `pub(crate)`, three tests);
  `tests/recipe_corpus_output.rs` and `tests/recipe_round_trip.rs` (stems and literals only);
  `tests/wasm_roundtrip.rs` (three rows); `docs/recipes.md`, `recipes/README.md`,
  `docs/data-model.md`, `skills/crustyimg/SKILL.md`, `docs/api-contract.md`,
  `integrations/ddphotos/README.md`, `README.md`, `CHANGELOG.md`.
- **New exports:** `crustyimg::operation::crop::Crop` (re-exported as `crustyimg::operation::Crop`)
  with `pub fn from_params(&OperationParams) -> Result<Crop, RegistryError>` and its `Operation`
  impl; `crustyimg::operation::CROP_PARAMS: &[ParamSpec]`;
  `OperationParams::get_f64(&self, &str) -> Option<f64>`.
- **Database changes:** none. No `CACHE_SCHEMA_VERSION` change, no lockfile change, no version bump.

## Acceptance Criteria

**The op**
- [ ] **AC-1** `OperationRegistry::with_builtins()` registers `crop`, and `param_specs("crop")` is
  exactly `x, y, width, height, aspect, gravity` in that order, with the kinds in the design table.
  `identity`, `invert`, `resize`, `auto-orient` and `watermark` keep their tables unchanged.
- [ ] **AC-2** A rectangle crop cuts `(left, top, right − left, bottom − top)` with each edge rounded
  on its own in `f64` per DEC-112: on 1200×800, `x=0.123 y=0.2345 width=0.5 height=0.4321` gives a
  600×345 crop at (148, 188); on 6000×4000 the same step gives 3000×1728 at (738, 938).
- [ ] **AC-3** Halves round up, and the fractions are read as `f64`: `x=0.0009375 y=0 width=0.5
  height=1` on 1600×1067 gives an 800×1067 crop at (2, 0) (1.5 → 2; an `f32` read gives 1);
  `x=0.1 y=0.25 width=0.5 height=0.4321` on 2600×1730 gives a 1300×747 crop at (260, 433)
  (432.5 → 433).
- [ ] **AC-4** Every edge is within 0.5 px of `f · W` for every W in 1..=64 over the fraction grids
  `i/97` and `i/8` (every `x`, `width` with `x + width ≤ 1`).
- [ ] **AC-5** The same fractions on 1200×800 and 6000×4000 give edges with
  `|e_6000 / 5 − e_1200| ≤ 0.4` for all four edges, over 200,000 cases from a fixed seed. The 0.4
  bound holds **for this seed**: changing the seed or the case count can reach 0.6 on correct code
  (the general bound, reached at `f64` near-ties).
- [ ] **AC-6** An aspect crop is the largest a:b rectangle that fits, its short side rounded half up
  in integer maths, placed by `gravity` (default `center`) with centring floored: the worked
  example's 18 placements (9 gravities × 2 orientations) exactly; `"16:9"` on 2600×1730 is 2600×1463
  at (0, 133); `"32:18"` gives the same crop as `"16:9"`.
- [ ] **AC-7** A crop moves pixels and computes none (DEC-095): on rgb8, rgba8 and rgba16 inputs
  every output pixel equals the input pixel at `(left + i, top + j)`, and the colour type and bit
  depth are the input's; the same holds for L8 and La8.
- [ ] **AC-8** `x=0 y=0 width=1 height=1` returns a buffer byte-equal to the input's, with the same
  colour type.
- [ ] **AC-9** `Crop::params()` returns the step's keys exactly as written, for both forms: integer
  values stay integers, and an absent `gravity` stays absent.

**Refusals**
- [ ] **AC-10 … AC-41** (one per row) Each load row R1–R32 is refused by `Recipe::build_pipeline`
  with `with_builtins()` as a `RecipeError::InvalidOperation` whose `Display` is **exactly**
  `recipe step 0 ('crop'): ` + that row's message.
- [ ] **AC-42 … AC-45** (one per row) Each apply row A1–A4 builds, and `Pipeline::run` on a
  1600×1067 image returns an `OperationError::Apply` whose `Display` is **exactly**
  `operation 'crop' failed: ` + that row's message.
- [ ] **AC-46** On the binary, `apply --recipe <A1> in.png -o out.png` exits 1, prints exactly
  `error: operation 'crop' failed: the crop rounds to 0×1067 px on this 1600×1067 px image; it must
  keep at least 1 px each way`, and writes no `out.png`. (The guard against routing the check
  through `refusal()`, which `apply` skips.)
- [ ] **AC-47** On the binary, `apply --recipe <R13> in.png -o out.png` exits 1, prints exactly
  `error: recipe step 0 ('crop'): 'x' + 'width' is 0.7 + 0.5, past the right edge; together they
  must be at most 1`, and writes no file.
- [ ] **AC-48** `Crop` does not override `Operation::refusal`: `refusal(&img)` is `None` for the A1
  step on the 1600×1067 image.

**End to end, corpus, wasm**
- [ ] **AC-49** On the binary, a rectangle recipe (`version = "2"`, AC-2's step) applied to a
  1200×800 PNG writes a 600×345 PNG whose pixels equal the input's at offset (148, 188), exit 0.
- [ ] **AC-50** The corpus gains `v2_crop_rect` (one step, AC-2's fractions) and `v2_crop_aspect`
  (one step, `aspect = "5:4"`, `gravity = "southeast"`), each with `.toml`, `.golden` and three
  per-target `.output` files, added by the README's procedure. `EXPECTED_DIMS`: `v2_crop_rect`
  `["1300x747", "865x1123", "1300x747"]`; `v2_crop_aspect` `["2163x1730", "1730x1384", "2163x1730"]`.
  The literals become: `RECIPES` +2 stems, `EXPECTED_DIMS` length **16**, `ROWS` **54**,
  `GOLDEN_LINES` **57**, the `42` → **48**, `recipe_round_trip.rs` `CORPUS` +2 and `pairs.len()`
  **16** (message "13 corpus recipes + 3 bundled recipes"), and the doc comments to match.
- [ ] **AC-51** For each new crop recipe, its three per-target `.output` files are byte-identical
  (crop does no float maths on pixels), and the build says so with the `cmp` it ran.
- [ ] **AC-52** **No existing output changes:** `git diff --diff-filter=M --name-only origin/main --
  tests/fixtures/recipe_corpus/` is empty, `git diff --diff-filter=D` on the same path is empty, and
  `cargo test --test recipe_corpus_output` and `--test recipe_round_trip` pass with no re-bless of
  an existing file.
- [ ] **AC-53** wasm `transform` (run in a real wasm VM by `just wasm-test`): the AC-2 rectangle
  step on a 1200×800 PNG gives exactly 600×345 and pixels equal to the input's at (148, 188), zero
  differing (DEC-110 limb 2, no 1/255 allowance), and `aspect = "5:4"`, `gravity = "southeast"`
  gives exactly 1000×800 with pixels equal at (200, 0).
- [ ] **AC-54** wasm `transform` throws, and the message contains exactly the native line, for R18
  (`recipe step 0 ('crop'): 'aspect' cannot be combined with 'x'; …`) and R28; and for A1 on a
  1600×1067 PNG (`operation 'crop' failed: the crop rounds to 0×1067 px …`). The module survives each
  (an identity transform succeeds after).

**Docs** (verify checks each by reading; `file:line` is on `main` at `3a412d0`)
- [ ] **AC-55** `docs/recipes.md:60-62`: the planned `crustyimg crop` entry becomes **"Crop to a
  rectangle or an aspect [today]"** with a recipe-step snippet (`[[step]] op = "crop"` with
  fractions, and an `aspect = "16:9"` / `gravity` variant), one line saying the numbers are fractions
  of the image (0–1), not pixels, so the same recipe crops the same region at any size, and one line
  saying to put `op = "auto-orient"` first when the fractions are meant as the photo is displayed.
  It names no `crustyimg crop` verb.
- [ ] **AC-56** `docs/recipes.md:64-65`: smart crop loses the `crustyimg crop --smart` line and is
  marked **post-1.0, beta**, pointing at `docs/roadmap.md`'s smart-crop row (`:80`).
  `docs/recipes.md:63`'s `[planned — PROJ-006]` for rotate/flip/trim/pad becomes
  `[planned — PROJ-017]`. Afterwards `/usr/bin/grep -rn "crustyimg crop" docs/ recipes/ README.md
  skills/ npm/ integrations/` finds nothing.
- [ ] **AC-57** `recipes/README.md:84-93`: the op table gains a `crop` row after `watermark` (before
  `optimize`): "`x`, `y`, `width`, `height` as fractions of the image, or `aspect` + `gravity`
  (below)". A new `### The crop step` section before `### Step keys are checked` (`:198`) gives both
  forms as TOML, a `| Key | Form | Value |` table, and notes saying: lengths are fractions 0–1, not
  pixels, and `width = 1` is the full width; each edge rounds to the nearest pixel (halves up), so
  the same recipe agrees within a pixel at any size; `aspect` is two whole numbers (`"239:100"`, not
  `"2.39:1"`); a crop that rounds to 0 px fails rather than being widened; the fractions are of the
  image as the step receives it, so put `auto-orient` first for display coordinates.
- [ ] **AC-58** `docs/data-model.md:162-166`: "five operations" becomes six, naming `crop`. A new
  `### crop step param keys` section after the resize section (before `:328`) has a
  `| Key | Type | Range |` table (`x`, `y`: number `0.0..=1.0`; `width`, `height`: number
  `> 0 and <= 1`; `aspect`: string, two whole numbers 1–65535 as `W:H`; `gravity`: one of the nine,
  default `center`), the two forms' exclusivity, and a TOML example containing `op = "crop"` (which
  `tests/docs_ops.rs` then checks resolves).
- [ ] **AC-59** `skills/crustyimg/SKILL.md:233-234`: the ops list gains `crop` (`x`/`y`/`width`/
  `height` as fractions 0–1, or `aspect = "W:H"` + `gravity`), and the frontmatter `description`
  (`:3`) mentions crop.
- [ ] **AC-60** `docs/api-contract.md`: a "**Crop in a recipe.**" paragraph beside the watermark
  one (`:392-412`) says `op = "crop"` is a registered operation taking a rectangle or an aspect in
  fractions, refused at load for bad keys and at apply when it rounds to 0 px, and points at
  `docs/data-model.md` for the keys.
- [ ] **AC-61** `integrations/ddphotos/README.md:127-129` ("No crop, rotate or colour … recipes have
  only `resize`, `auto-orient`, `watermark` and `invert`") is rewritten: recipes can crop now; the
  edit page doesn't offer crop yet; rotate and colour still need their own work. `README.md:244`'s
  "resize, watermark and more" names crop. `npm/README.md:102` already lists crop for `transform`;
  verify confirms it is now true and it is unchanged.
- [ ] **AC-62** `CHANGELOG.md`: under `## [Unreleased]` (`:8`), a new `### Added` with "**A recipe
  can crop.**" — `op = "crop"` with a rectangle in fractions or an aspect with gravity; the same
  recipe crops the same region at any image size; 0.8.2 and earlier refuse it with
  `unknown operation 'crop'`.
- [ ] **AC-63** Every user-facing string this spec adds (messages, doc comments, docs) names no
  spec or DEC id.

**Hygiene**
- [ ] **AC-64** No version bump, no `CACHE_SCHEMA_VERSION` change (`src/build/cache.rs:81`), no
  lockfile-format change. Why none is needed: an existing recipe's canonical TOML and output are
  unchanged (AC-52), the cache key already folds in the crustyimg version (`src/build/cache.rs:25`),
  and 0.8.2 refuses a crop recipe at load, so no cached entry or lockfile line for one can exist.
- [ ] **AC-65** `just check` is green, CI is green on every leg including the lean
  (`--no-default-features`) build and the wasm bundle-size gate, and `just wasm-test` is green
  locally with its result line quoted in Build Completion (CI does not run it yet; SPEC-146 adds it).

## Failing Tests

Written at design; the build makes them pass. **Baseline on `main`:** every integration row is RED
with `unknown operation 'crop'` (load rows, apply rows, CLI rows, wasm rows) or a missing corpus file
(`v2_crop_*`); the unit tests don't compile until `src/operation/crop.rs` exists. The build quotes
one RED message per test file. **The wrong-reason trap (13):** a load row that fails for another
row's reason is impossible to miss, because every row asserts the whole message with `==`.

- **`src/operation/crop.rs` (`#[cfg(test)] mod tests`)** — calls the private production functions
  `rect_edges`, `aspect_size` and `parse_aspect`, and `Crop::from_params` + `apply`, never a copy.
  - `crop_edges_within_half_a_pixel` — AC-4: for W in 1..=64, for grids `i/97` and `i/8`, every `x`
    and `width > 0` with `x + width ≤ 1`: `|left − x·W| ≤ 0.5` and `|right − (x+width)·W| ≤ 0.5`
    (and the same through `y`/`height`/`H`, by calling with the axes swapped). Asserts the number of
    cases checked equals a literal the build computes once and writes down.
  - `crop_fraction_agrees_at_1200_and_6000` — AC-5: xorshift64 (shifts 13, 7, 17, as in
    `tests/recipe_corpus_output.rs:106-114`), seed `0x5EED_0145`; per case draw `u1`, `u2` as
    `(next() >> 11) as f64 * 2^-53`, set `x = min`, `width = max − min`, skip `width == 0`, until
    **200,000** cases; the same for `y`/`height` from the next two draws. Asserts
    `|e_6000 / 5 − e_1200| ≤ 0.4` for all four edges and that 200,000 cases ran. Design measured
    the production rule at **0.40 px** max over this seed (not the general bound, which is 0.6 at
    `f64` near-ties), and both rounding mutations at **0.80 px**.
  - `rect_edges_exact_example` — AC-2: `(0.123, 0.2345, 0.5, 0.4321)` → 1200×800:
    `(148, 188, 748, 533)`; 6000×4000: `(738, 938, 3738, 2666)`.
  - `halves_round_up_and_fractions_read_as_f64` — AC-3: through `Crop::from_params` + `apply`
    (not `rect_edges` alone, so the `get_f64` read is in the path): `x=0.0009375 y=0 width=0.5
    height=1` on 1600×1067 → 800×1067, and the pixels at output (0, 0) equal input (2, 0);
    `x=0.1 y=0.25 width=0.5 height=0.4321` on 2600×1730 → 1300×747, output (0, 0) = input (260, 433).
  - `aspect_fit_worked_example` — AC-6: `aspect_size(5, 4, 2600, 1730) == (2163, 1730)`,
    `(5, 4, 1730, 2600) == (1730, 1384)`, `(16, 9, 2600, 1730) == (2600, 1463)`,
    `(32, 18, 2600, 1730) == (2600, 1463)`, `(10000, 1, 1600, 1067) == (1600, 0)`,
    `(1, 10000, 1600, 1067) == (0, 1067)`.
  - `aspect_gravity_places_all_nine` — AC-6: through `from_params` + `apply` on 2600×1730 and
    1730×2600 rgb8 images with a pixel pattern unique per position, the 18 offsets in the worked
    example, each checked by the output's (0, 0) pixel equalling the input at the expected offset and
    the output size; plus `"16:9"` centre on 2600×1730 at (0, 133). One table, every failing row
    reported by gravity and orientation, and a literal count of 19 rows.
  - `luma_crops_are_exact` — AC-7: L8 and La8 inputs, every output pixel equals the input at the
    offset; colour type preserved.
  - `identity_crop_is_byte_equal` — AC-8: rgb8 and rgba16, `as_bytes()` equal, colour type equal.
  - `params_round_trip_as_written` — AC-9: the rect step with `x = 0` (integer) and the aspect step
    without `gravity`: `params()` equals the input map; with `gravity = "west"` it is kept.
  - `refusal_hook_is_not_used` — AC-48.
- **`src/operation/registry.rs` (`mod tests`)**
  - `with_builtins_contains_crop` (new) — AC-1.
  - `every_builtin_publishes_a_param_schema` (extended) — AC-1: `keys("crop")` ==
    `["x", "y", "width", "height", "aspect", "gravity"]`; the existing asserts unchanged.
  - `one_of_values_are_accepted_by_their_constructors` (extended) — crop's nine gravities each build
    with `aspect = "16:9"`; the literal becomes `6 + 9 + 9`, message "6 resize modes + 9 watermark
    gravities + 9 crop gravities".
- **`tests/crop.rs` (new)** — through the public API (`Recipe::from_toml`, `build_pipeline`,
  `Pipeline::run`) and the binary (`env!("CARGO_BIN_EXE_crustyimg")`, stdout and stderr redirected
  to files, exit code read directly). Fixtures are built in the test from seeded samples, encoded
  with the `image` crate's PNG encoder; expected pixels come from indexing the source buffer, never
  from `crop_imm`.
  - `every_load_refusal_names_its_reason` — AC-10…41: the 32-row R table as data
    (`(id, body, message)`), each recipe `version = "2"` with one step; runs **every** row, collects
    each mismatch as `"{id}: expected …, got …"`, fails listing all of them, and asserts 32 rows ran.
  - `every_zero_pixel_crop_fails_at_apply` — AC-42…45: the 4-row A table on a 1600×1067 rgb8 image,
    same collect-all shape, 4 rows ran; each row also asserts `build_pipeline` succeeded (so a row
    can't pass by being refused at load).
  - `apply_refuses_a_zero_pixel_crop_and_writes_nothing` — AC-46 (binary).
  - `apply_refuses_a_crop_past_the_right_edge_at_load` — AC-47 (binary).
  - `apply_crops_a_png_end_to_end` — AC-49 (binary).
  - `crop_preserves_colour_type_and_every_pixel` — AC-7: rgb8, rgba8, rgba16 at 64×48 with
    `x=0.25 y=0.25 width=0.5 height=0.5` (all products integers, so this test guards copying, not
    rounding) → 32×24 at (16, 12); every output pixel equals the input's at the offset, colour type
    and depth equal.
- **`tests/recipe_corpus_output.rs` / `tests/recipe_round_trip.rs`** — AC-50…52: the two new stems,
  their `EXPECTED_DIMS`, their goldens, and the literal bumps; no other change. Both new recipes
  change dimensions on every fixture, so the corpus's own "every acting step acts" and "expected
  dims" controls apply to them unchanged.
- **`tests/wasm_roundtrip.rs`** — AC-53…54, `#[wasm_bindgen_test]`:
  - `transform_crops_exactly` — the rectangle and aspect rows, dimensions with `assert_eq!` and
    every pixel compared with the decoded input at the offset (decoded with the `image` crate inside
    the test).
  - `transform_refuses_a_bad_crop_with_the_native_message` — R18 and R28 (load) and A1 (apply),
    `msg.contains(<the exact native line>)`, and an identity transform succeeds after each.
- **`tests/docs_ops.rs`** — unchanged; it goes red if AC-58's `op = "crop"` example lands before the
  op is registered, which is the order guard for that doc.

### Mutation table (traps 12 and 14)

Each row: where it is applied, what goes **red** (with the number that moves), and what stays
**green**. Apply one at a time, run `cargo test` (stdout redirected) and, where the row says, `just
wasm-test` and the corpus test; record the flip, then revert and confirm green. A row whose red set
doesn't match is reported, not edited to match.

| # | mutation, and where | red (expected) | green (expected) |
|---|---|---|---|
| M1 | **floor the right edge**: `rect_edges`, `right = ((x + w) * iw).floor()` (`src/operation/crop.rs`) | `crop_edges_within_half_a_pixel` (0.99 px); `crop_fraction_agrees_at_1200_and_6000` (0.80 > 0.4); `rect_edges_exact_example` (right 747 on 1200, width 599); `halves_round_up_and_fractions_read_as_f64` first case (right 801, width 799); `apply_crops_a_png_end_to_end` (599×345); corpus `v2_crop_rect` on all three fixtures (1299x747, 864x1123, 1299x747 against `EXPECTED_DIMS`); wasm `transform_crops_exactly` rect row (599 wide) | every R row, A1–A4 (A1 still rounds to 0: floor(800.16) = 800), `crop_preserves_colour_type_and_every_pixel` (integer products), aspect tests, corpus `v2_crop_aspect`, every existing test |
| M2 | **round the size, not the far edge**: `rect_edges`, `right = left + (w * iw).round()` and `bottom = top + (h * ih).round()` | both property tests (0.99 / 0.80); `rect_edges_exact_example` (bottom 534 on 1200×800: height 346); `apply_crops_a_png_end_to_end`; `halves_round_up_and_fractions_read_as_f64` second case; corpus `v2_crop_rect` on `rgb8_2600x1730` and `rgba16_2600x1730` (1300x748) | corpus `v2_crop_rect` on `rgba8_1730x2600` (865x1123 either way); everything M1 lists green |
| M3 | **remove the right-edge load check**: delete step 5's `x + width > 1.0` branch in `Crop::from_params` | R13 in `every_load_refusal_names_its_reason` (builds Ok); `apply_refuses_a_crop_past_the_right_edge_at_load` (the binary now prints the apply line `operation 'crop' failed: internal: the crop's right edge … is past …`, not the load line; were the apply guard also missing, `crop_imm` would clamp and the test would see exit 0 and a written file, still red) | R14 (its own check), every other test |
| M3b | **remove the apply-side bounds guard too** (with M3 applied) | as M3; plus the binary test sees exit 0 and a written file (the silent clamp, measured in `crop_dimms`) | as M3 |
| M4 | **clamp to 1 px**: in `Crop::apply`, `cw.max(1)` / `ch.max(1)` before cropping, no refusal | all four rows of `every_zero_pixel_crop_fails_at_apply`; `refusal_hook_is_not_used`; `apply_refuses_a_zero_pixel_crop_and_writes_nothing` (exit 0, file written); wasm `transform_refuses_a_bad_crop_with_the_native_message` A1 part | every R row, every happy-path test, corpus |
| M4b | **route 0 px through the refusal hook**: move the 0 px check from `apply` into an `Operation::refusal` override | `refusal_hook_is_not_used`; `every_zero_pixel_crop_fails_at_apply` (`Pipeline::run` now returns `crop refused: …`, not `operation 'crop' failed: …`); `apply_refuses_a_zero_pixel_crop_and_writes_nothing` (with the check in the refusal hook, a single-input `apply` exits 2 (`error: crop refused: …`) and writes nothing; a two-input batch exits 0 and writes both images uncropped); wasm A1 part (message is `crop refused: …`) | everything else |
| M5 | **accept mixed forms, prefer the rectangle**: delete step 1 in `Crop::from_params` (the rectangle branch then runs and ignores `aspect`) | R18 | R19, R20 (gravity is checked separately), every other row and test |
| M6 | **swap W and H in the fit**: `aspect_size` computes with `(ih, iw)` | `aspect_fit_worked_example` (2163×1730 → 1730×1384); `aspect_gravity_places_all_nine` (all 19 rows: sizes differ); `aspect_never_exceeds_the_image`; A3 and A4 rows (the message's size reads `1067×0` / `0×1600`; A3 and the portrait corpus row print the apply-side bounds message); corpus `v2_crop_aspect` on all three fixtures; wasm aspect row | every rectangle test, R rows, corpus `v2_crop_rect` |
| M7 | **wasm-only rounding**: in `rect_edges`, `#[cfg(target_arch = "wasm32")]` floor the right edge | `just wasm-test`: `transform_crops_exactly` rect row (599×345) | every native test (`cargo test`, corpus): the `cfg` excludes them |
| M8 | **`f32` read**: replace `get_f64` with `get_f32(..).map(f64::from)` in `Crop::from_params` | `halves_round_up_and_fractions_read_as_f64` first case (left 1, not 2) | R13 (operands print from the TOML value), property tests (they call `rect_edges` with `f64`), corpus (its fractions don't sit on a half-pixel) |
| M9 | **unreduced decimal hint**: drop the gcd step in `parse_aspect` | R29 (`"15:10"`) | R28 (gcd 1), every other row |
| M10 | **`u32::from_str` per side** in `parse_aspect` | R24 (`"+16:9"` accepted, builds Ok) | every other row |
| M11 | **corpus guard** (scratch copy, never committed): append `[[step]] op = "crop"` `x = 0` `y = 0` `width = 0.5` `height = 1` after the resize step of `tests/fixtures/recipe_corpus/v1_resize_max.toml` | `recipe_corpus_output` red naming `v1_resize_max × <fixture>` on all three fixtures (dims `400x532` against `800x532` on the two landscape fixtures, `266x800` against `532x800` on the portrait one); `recipe_round_trip` red on `v1_resize_max`'s golden | every crop test |
| M12 | **trap 14, the old code verbatim**: restore `with_builtins` from `main` (no `crop` registration) | every row of `tests/crop.rs`, the corpus crop rows, both wasm rows, `with_builtins_contains_crop`, both registry extensions; message `unknown operation 'crop'`; `every_op_named_in_data_model_resolves` (`tests/docs_ops.rs`) | the `crop.rs` unit tests (they call the module directly), every existing test |

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle. It is the
equivalent of a handoff document, folded into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-112` — **this spec's.** Fractions only, `f64`, per-edge rounding with halves up, load
  refuses `x + width > 1`, 0 px refused at apply as an op failure, whole-number aspects.
- `DEC-091` — Fence A: absolute pixel coordinates are lab-side; why the recipe has no pixel keys.
- `DEC-102` — the key table is checked before the constructor; the constructor holds only cross-key
  rules (forms, partial rectangle, bounds, aspect syntax). Every number is also range-checked as an
  `f32`, which is what refuses `1e-50`.
- `DEC-095` — preserve colour type and depth. `crop_imm` on the `DynamicImage` keeps the variant;
  no widen, no narrow.
- `DEC-109` — the precedent for refusing a step that would do nothing, instead of clamping.
- `DEC-110` — limb 2: a recipe with no resampling step is exact across native and wasm, dimensions
  and pixels.
- `DEC-014` / `DEC-061` — `OperationParams` stays the ordered TOML map; checking never rewrites it.

### Constraints that apply

- `untrusted-input-hardening` — every recipe value is untrusted: the aspect parser uses checked
  arithmetic and a 65535 bound; nothing panics on any TOML value.
- `no-unwrap-on-recoverable-paths` — no `unwrap`/`expect` in `crop.rs` outside tests.
- `single-image-library` — `image` only (`crop_imm`); no new dependency.
- `every-public-fn-tested` — `Crop::from_params`, `get_f64`, the `Operation` impl.
- `test-before-implementation` — the tests above, red first, baseline quoted.
- `one-spec-per-pr` — one PR, `feat/spec-145-crop`.

### Prior related work

- `SPEC-131` (shipped) — the key tables and typed constructors this op plugs into.
- `SPEC-128` / `SPEC-133` (shipped, STAGE-050) — `watermark` in the registry: the shape of a
  registered op with a form choice (`image` XOR `text`), its refusals and its wasm rows.
- `SPEC-141` (shipped, PR #233) — the recipe corpus guard; its README is the procedure for
  AC-50.
- `SPEC-142` (shipped) — native/wasm parity and DEC-110.

### Out of scope (for this spec specifically)

- `edit --crop` and the geometry ordering DEC: STAGE-058's `edit`-flags item, which also owns
  the pixel→fraction conversion and its `--save-recipe` round trip.
- The ddphotos page's crop UI, crop rows in `tests/ddphotos_parity.mjs`, and replay parity (b):
  STAGE-060's crop-on-page item.
- What an EXIF orientation tag means after a geometry step, and `PixelXDimension`/`PixelYDimension`:
  STAGE-058's rotate/flip item.
- Pixel units in a recipe: ruled out (DEC-112).
- Smart or content-aware crop: the roadmap's post-1.0 beta row, outside PROJ-017.
- Art-direction and og-card crops: PROJ-014.
- Step indices in apply-time and unknown-op error messages: `guidance/ideas.yaml`
  (`recipe-errors-name-the-step-everywhere`).
- `pad`, `trim`, `rotate`, `flip`: their own STAGE-058 items, including their `docs/recipes.md`
  entries (only line 63's project id is fixed here).
- A wasm CI leg: SPEC-146. `docs/backlog.md:59`'s stale "exactly four" op count (a document no
  tool reads).

## Notes for the Implementer

- **Checkpoint.** Commit and push after each of: (1) the red tests, with the baseline quoted;
  (2) the op green locally; (3) the corpus files for your own target; (4) the docs. A long build
  with no pushed commit has been lost before.
- **Corpus bless order.** Add the two `.toml` files, write their `.golden` by the README, raise every
  literal in one commit, bless your own target, push, and copy the other two targets' files from
  the failing CI log's `----- BEGIN … -----` blocks. Before that, check AC-51 locally against your
  own file only; after CI, `cmp` all three.
- **`cargo test` in a terminal** fails `display_sink_refuses_non_tty`: redirect stdout to a file and
  read `$?`, never through a pipe. Use your own `CARGO_TARGET_DIR`. `just check` doesn't run the
  wasm bundle-size gate: read that CI leg, and run `just wasm-test` yourself (CI doesn't yet).
- **Run the lean build** (`cargo test --no-default-features`): crop has no feature gate, so it must
  pass there too.
- **Don't** reuse `crop_center` (`mod.rs:596`): it's centre-only and typed to resize's buffer.
  Do call `Gravity::placement` with margin 0.
- **User-facing text** (messages, doc comments, docs) reads behaviour-first and names no spec or
  DEC id.
- **Files the PROJ-011 orchestrator must keep clear of** while this builds: `src/operation/`,
  `src/recipe/`, `docs/recipes.md`, `recipes/README.md`, `docs/data-model.md`,
  `docs/api-contract.md`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md`'s Unreleased section,
  `tests/fixtures/recipe_corpus/`, `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs`,
  `tests/wasm_roundtrip.rs`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** feat/spec-145-crop
- **PR (if applicable):** #244 (draft)
- **All acceptance criteria met?** yes (65 of 65; two notes under Deviations)
- **New decisions emitted:**
  - none (DEC-112 was written at design)
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`, minus this spec file):
  - `src/operation/crop.rs` (new) — the op, `CROP_PARAMS`, edge/aspect maths, aspect parser, unit tests
  - `src/operation/mod.rs` — `pub mod crop` + re-export, `OperationParams::get_f64`
  - `src/operation/registry.rs` — registers `crop`, doc lists, `float_text` to `pub(crate)`, three tests extended
  - `tests/crop.rs` (new) — R1–R32, A1–A4, binary rows, colour-type exactness
  - `tests/wasm_roundtrip.rs` — `transform_crops_exactly`, `transform_refuses_a_bad_crop_with_the_native_message`
  - `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs` — stems and literals only (16 recipes, 54 rows, 57 golden lines, 48, pairs 16)
  - `tests/fixtures/recipe_corpus/v2_crop_rect.{toml,golden,linux-x86_64.output,macos-aarch64.output,windows-x86_64.output}` and the same five for `v2_crop_aspect` (all new)
  - `docs/recipes.md`, `recipes/README.md`, `docs/data-model.md`, `skills/crustyimg/SKILL.md`, `docs/api-contract.md`, `integrations/ddphotos/README.md`, `README.md`, `CHANGELOG.md` — crop documented
  - Nothing outside the spec's Outputs list. `Cargo.toml`, `Cargo.lock` and `src/build/cache.rs` untouched (`git diff --stat` on them is empty).
- **Evidence:**
  - Red baseline: every `tests/crop.rs` row failed with `unknown operation 'crop'`; unit tests failed to compile (`cannot find function rect_edges / aspect_size`, `cannot find type Crop`); three registry tests failed.
  - Corpus: first CI run red on every leg that runs the corpus test (only because the linux/windows files were missing); every existing `.output` in the dump matched the repo; the four new files were copied from the dump; second CI run (head da31660) green on all legs, including lean, msrv and the page build. `cmp` of the three per-target files: `v2_crop_rect` and `v2_crop_aspect` byte-identical (exit 0). `git diff --diff-filter=M --name-only origin/main -- tests/fixtures/recipe_corpus/` and `--diff-filter=D` are empty.
  - Local matrix, each in a fresh target dir, fmt --check and clippy -D warnings clean: default 1156 passed / 0 failed; `--no-default-features` 1093 / 0; `--features webp-lossy` 1160 / 0 (19 crop-named tests ran on each, including lean).
  - Wasm: `just wasm-check` finished clean; `just wasm-test` result line: `test result: ok. 49 passed; 0 failed; 0 ignored; 0 filtered out`.
  - Bundle size (the gate is `scripts/demo-assemble.mjs`, run by the "build + browser smoke" job): main (same code, run on a13d568) 6,065,799 B raw / 1,298,960 B brotli; this branch 6,083,069 B / 1,302,231 B: +17,270 B raw, +3,271 B brotli (+0.25 %), far inside the gate's 5 % of its 1,266,535 B baseline.
- **Mutations M1–M12** (each applied, `git diff` non-empty, crate cleaned, tests run with `--no-fail-fast`, restored; wasm legs via `just wasm-test`):
  - M1 floor right edge: red = both property tests, `rect_edges_exact_example` (747), `halves_round_up…`, `apply_crops_a_png_end_to_end` (599×345), corpus `v2_crop_rect` on all three fixtures (1299x747, 864x1123, 1299x747), wasm rect row (599). Matches the table.
  - M2 round the size: red = both property tests, `rect_edges_exact_example`, corpus `v2_crop_rect` on rgb8 and rgba16 (1300x748; portrait stays green). **Mismatch:** also red are `apply_crops_a_png_end_to_end` (346 tall) and the second case of `halves_round_up…` (748), which the table's M2 row omits; both follow from the spec's own AC-2/AC-3 numbers. wasm leg not run (the row doesn't call for it).
  - M3: red = R13, `apply_refuses_a_crop_past_the_right_edge_at_load` (binary prints `operation 'crop' failed: internal: the crop's right edge 77 is past the image's 64 px width`). R14 and everything else green. Matches.
  - M3b: as M3, and the binary exits 0 (the silent clamp). Matches.
  - M4: red = A1–A4, the binary test (exit 0), wasm `transform_refuses_a_bad_crop…` (A1). **Mismatch:** `refusal_hook_is_not_used` also goes red (it asserts `apply` errors).
  - M4b: red = A1–A4 (`crop refused: …`), `refusal_hook_is_not_used`, the binary test, wasm A1. **Mismatch in how:** a single-input `apply` exits 2 with `error: crop refused: …`, not exit 0. Driven separately: with two inputs and `--out-dir`, the same mutation exits 0, writes both uncropped 1600×1067 images and prints a "watermark skipped" warning. So the design's trap is real for batches only.
  - M5: red = R18 only. Matches. (Applied as "skip the mixed-forms check and route to the rectangle branch".)
  - M6: red = `aspect_fit_worked_example`, `aspect_gravity_places_all_nine` (all 19 rows reported after I made the test collect apply errors too), A3, A4, corpus `v2_crop_aspect` on all three fixtures, wasm aspect row (800×640). **Mismatches:** `aspect_never_exceeds_the_image` also red; with W and H swapped the crop overruns the image, so A3 and the portrait fixture trip the apply-side bounds guard and print `internal: the crop's … edge … is past …` rather than a `1067×0` size line (A4 does read `1067×0`).
  - M7: native `cargo test` fully green; wasm `transform_crops_exactly` red (599×345). Matches.
  - M8: red = `halves_round_up_and_fractions_read_as_f64` only ("left edge is 2, not 1"). Matches.
  - M9: red = R29 only. Matches. M10: red = R24 only. Matches.
  - M11 (in place, restored, `git status` empty afterwards): `every_recipe_output_matches_its_golden` red naming `v1_resize_max × <fixture>` (400x532 vs 800x532 on both landscape, 266x800 vs 532x800 portrait); `to_toml_is_byte_identical…` red. Crop tests green. Matches.
  - M12 (`with_builtins` restored from `origin/main`): red = all six `tests/crop.rs` rows, corpus and round-trip rows, three registry tests, both wasm rows (`unknown operation 'crop'`); `crop.rs` unit tests green. **Mismatch:** `every_op_named_in_data_model_resolves` (docs_ops) also red, since the data-model example now names `crop`; that is the order guard the spec describes, not a leak.
- **Deviations from spec:**
  - The wasm test file has two functions (as the Failing Tests section names them) holding 2 + 3 rows, not "three rows".
  - The rectangle golden lists keys alphabetically (`height, width, x, y`), the canonical `to_toml` order; the spec's example order was not what the golden pins.
  - The gravity unit test was changed during the mutation pass to report apply errors per row.
  - Pre-existing, not mine: `just wasm-check` prints a `MANIFEST_DROPPED_WARNING` dead-code warning.
- **Follow-up work identified:**
  - The M2, M4, M4b, M6, M12 rows of the mutation table need the extra red rows and the single-input exit 2 note above.
  - A refusing op in a native single-input run exits 2 but is warned about as "watermark skipped" in a batch; that wording is watermark-specific and wrong for any other refusing op.

### Punch list (verify cleanups)

Four cleanups after verify approved #244 at `6611e79` with no punch items. Worktree `fix-spec-145-punch`,
Sonnet. No behaviour change in `src/`; the one `src/operation/crop.rs` edit is inside its `#[cfg(test)]` module.

1. **Bound 0.6 -> 0.4** in `crop_fraction_agrees_at_1200_and_6000` (it lives in `src/operation/crop.rs`'s
   test module, not `tests/crop.rs`). The bound is `0.4 + 1e-9`: the unmodified rule's worst case computes
   as `0.4000000000000057` in f64, so a bare `<= 0.4` fails on the production code. Message now prints the drift.
2. **A table**: `assert_eq!(rows.len(), 4, "A1..A4 all ran")` in `every_zero_pixel_crop_fails_at_apply`.
3. **`recipes/README.md` crop section**: exit `1` is for one input; a batch exits `6`. Driven on this branch's
   binary, crop `x=0.5 width=0.03` (0 px on a 16x12 image, 8 px on a 256x256 one):
   one bad input -> exit 1; two bad inputs -> exit 6, nothing written; one bad + one good -> exit 6, the
   good image written (`error: 1 of 2 inputs failed`). No other crop doc surface makes the exit-1 claim
   (`docs/recipes.md`, `docs/data-model.md`, `skills/crustyimg/SKILL.md`, the ddphotos README checked;
   `docs/api-contract.md` already says "for one input").
4. **Mutation table**: M4b's single-input exit 2 / two-input exit 0 note, the extra red rows for M2, M4, M6
   and M12, and the 0.6 -> 0.4 bound (M1, AC-5 and the test's description). The design note's loose
   `0.5 + 0.5/5 = 0.6` derivation is kept, with the 0.4 the seeded sweep measures stated beside it (punch list 2: that is not the general maximum). **DEC-112 changed**
   on this branch (its size-independence line now says 0.4, asserted by SPEC-145's test).

CI on `1316ad7`: run 42090297892 (CI), plus DCO 42090297937, all 17 checks pass, 8 release jobs skipped.

Controls (each rebuilt clean, `cargo clean -p crustyimg`, exit read from `$?`):

| control | result |
|---|---|
| right edge `+ 0.1` before `.round()` | drift 0.6000000000000227: **red at 0.4 and red at 0.6** (at `6611e79`'s bare `drift <= 0.6`). It shows the bound bites; it does not show 0.4 is tighter than 0.6. No mutation tested lands between the two bounds. |
| right edge half rounds down (`ceil(v - 0.5)`) | `crop_fraction...` stays green at 0.4 (drift stays 0.4); `halves_round_up_...` red. Not a bound mutation. |
| A4 row removed, array type `[_; 3]` | red at the new count assert (`left: 3, right: 4`) |
| A4 row removed, type left `[_; 4]` | does not compile (E0308), so the type alone already catches it |
| M1 (floor the right edge) | `crop_fraction...` red (0.80 > 0.4), plus `crop_edges_within_half_a_pixel`, `rect_edges_exact_example`, `halves_round_up_...` |
| `cargo test --lib operation::crop`, `--test crop`, `--test docs_ops` | 11, 6, 3 passed |

### Punch list 2 (re-verify)

The re-verify confirmed the first four cleanups but found two false statements the first punch list
wrote. Worktree `fix-spec-145-punch2`, Sonnet. Text only, plus one test message; no behaviour change.

1. **The drift claim restated** in DEC-112, the design note, AC-5, the Failing Tests entry, the
   punch-list item 4 line and the test's comment: 0.4 px is measured over the seeded sweep; 0.6 px is
   the bound in general, reached at `f64` near-ties. The 0.4 bound holds for this seed only.
2. **The `+ 0.1` control row corrected:** drift 0.6000000000000227 is red at both 0.4 and 0.6.
3. **The test message prints the bound it checks:** `crop_fraction_agrees_at_1200_and_6000` now
   uses one `BOUND` const for the assert and the panic text.

Near-tie driven on this branch's release binary, images made by ImageMagick, recipe `x = 0`,
`y = 0`, `width = 0.009583333333333333`, `height = 1`: **12 px** wide at 1200 and **57 px** wide at
6000 (57/5 = 11.4, drift 0.6).

0.3 control: with `BOUND = 0.3` the test panics `... 111 at 1200, 553 at 6000 (drift
0.4000000000000057 > 0.3)`. Restored; `cargo test --lib operation::crop` (11) and `--test crop` (6)
green, clippy (default and `--no-default-features`) and `fmt --check` clean.

CI run id: CIRUNID

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Little. The exact-message table and mutation table made the build mechanical. The mutation table's red lists were incomplete for M2, M4, M6 and M12, and "the native apply skips it: exit 0" is true only for batches.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — That `to_toml` and the goldens order keys alphabetically, and that the corpus tests need all three targets' files before any leg is green. No design call turned out wrong.

3. **If you did this task again, what would you do differently?**
   — Gate every commit on the test exit code (I pushed a golden with a wrong key order once) and make the mutation script restore from a committed state from the start.

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
