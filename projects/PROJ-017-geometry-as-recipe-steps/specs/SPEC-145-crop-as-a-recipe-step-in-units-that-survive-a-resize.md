---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-145
  type: story                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-017
  stage: STAGE-058
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-7
  implementer: claude-opus-4-7     # usually same Claude, different session
  created_at: 2026-11-04

references:
  decisions:
    - DEC-095
    - DEC-102
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
`|edge_6000 / 5 - edge_1200|` was **0.40 px**. The rule's bound is 0.5 + 0.5/5 = 0.6 px, so the
"within one pixel" criterion holds with room to spare.

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

- `edit --crop` and the geometry ordering DEC: STAGE-058's `edit`-flags item, which also owns
  the pixel→fraction conversion and its `--save-recipe` round trip.
- The ddphotos page's crop UI, crop rows in `tests/ddphotos_parity.mjs`, and replay parity (b):
  STAGE-060's crop-on-page item.
- What an EXIF orientation tag means after a geometry step, and `PixelXDimension`/`PixelYDimension`:
  STAGE-058's rotate/flip item.
- Pixel units in a recipe, unless the maintainer's Q1 ruling admits them.
- Smart or content-aware crop: the roadmap's post-1.0 beta row, outside PROJ-017.
- Art-direction and og-card crops: PROJ-014.
- Step indices in apply-time and unknown-op error messages: `guidance/ideas.yaml`
  (`recipe-errors-name-the-step-everywhere`).
- `pad`, `trim`, `rotate`, `flip`: their own STAGE-058 items, including their `docs/recipes.md`
  line.

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
