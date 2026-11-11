---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-153
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
  created_at: 2026-11-10

references:
  decisions:
    - DEC-005
    - DEC-102
    - DEC-106
    - DEC-112
    - DEC-113
    - DEC-114
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - single-image-library
    - every-public-fn-tested
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-032
    - SPEC-110
    - SPEC-111
    - SPEC-145
    - SPEC-147
    - SPEC-151
    - SPEC-152

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-058's <capability>". Optional; null is acceptable.
value_link: >
  Delivers STAGE-058's "`edit` flags for all five, with a written ordering rule, and
  `--save-recipe` that writes them" for crop, rotate and flip, and the ordering rule all five
  share; the `--pad`/`--trim` half (filed after this entry) closes the stage.

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
      interface: claude-code
      model: claude-opus-5-5
      date: 2026-11-10
      tokens_total: 5378693
      tokens_breakdown:
        input: 84
        cache_creation: 147483
        cache_read: 5180694
        output: 50432
      estimated_usd: 4.77
      note: >
        Measured from the session JSONL after the frame push, deduplicated by message.id
        (42 messages; max output_tokens per id). Rates: Opus $5/$25 per MTok, cache_creation
        x1.25 input, cache_read x0.10 input. Excludes this cost commit's own turn.
    - cycle: design
      interface: claude-code
      model: claude-opus-5-5
      date: 2026-11-10
      tokens_total: 18545033
      tokens_breakdown:
        input: 194
        cache_creation: 266110
        cache_read: 18153020
        output: 125709
      estimated_usd: 13.88
      note: >
        Measured from the session JSONL after the design push (3be1af0), deduplicated by
        message.id (95 messages; max output_tokens per id); no subagents. Rates: Opus $5/$25 per
        MTok, cache_creation x1.25 input, cache_read x0.10 input. Excludes this cost commit's
        own turn. Includes the prototype build, the 548 s round-trip probe and 16 mutation runs.
    - cycle: build
      interface: claude-code
      model: claude-sonnet-5-5
      date: 2026-11-10
      tokens_total: 12089621
      tokens_breakdown:
        input: 172
        cache_creation: 199256
        cache_read: 11823833
        output: 66360
      estimated_usd: 5.29
      note: >
        Measured from the session JSONL at the last turn before the final commit, deduplicated by
        message.id (66 messages; max output_tokens per id); no subagents. Rates: Sonnet $3/$15 per
        MTok, cache_creation x1.25 input, cache_read x0.10 input. Excludes the final commit's turn.
    - cycle: punchlist
      interface: claude-code
      model: claude-sonnet-5-5
      date: 2026-11-10
      tokens_total: 898658
      tokens_breakdown:
        input: 24
        cache_creation: 51308
        cache_read: 841919
        output: 5407
      estimated_usd: 0.53
      note: >
        Measured from the session JSONL before this commit, deduplicated by message.id (12
        messages; max output_tokens per id); no subagents. Rates: Sonnet $3/$15 per MTok,
        cache_creation x1.25 input, cache_read x0.10 input. Excludes this commit's turn and the
        push and CI turns after it.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-153: edit flags for the five geometry ops, and the ordering rule

## Context

STAGE-058 line 8 of 13 (PROJ-017 brief, build order), the last STAGE-058 entry: "**`edit` flags for
the five, and the ordering rule.**" All five geometry ops have shipped as recipe steps: `crop`
(SPEC-145, DEC-112), `rotate` and `flip` (SPEC-147, DEC-113), `pad` (SPEC-151) and `trim`
(SPEC-152). The stage still owes "`edit` flags for all five, with a written ordering rule, and
`--save-recipe` that writes them", and the brief's success signal says each op "is reachable from
`crustyimg edit`".

Today a CLI user can't crop, turn or flip from `edit`. `crustyimg edit in.png --crop 30x20+0+0` and
`--rotate 90` both exit 2 with clap's `error: unexpected argument '--crop' found` (0.8.2, release
build of `main`). The only route is to write a recipe by hand, in fractions of the image.

## Goal

`crustyimg edit` takes `--crop`, `--rotate` and `--flip`, runs its op flags in the order they're typed
(DEC-114, written at design), and `--save-recipe` writes the steps it ran, in that order, so `apply`
replays them to the same pixels. `--pad` and `--trim` follow in the split-off entry, which closes
STAGE-058.

## Frame

*Framed 2026-11-10 on `main` at `b2c36d1` (0.8.2). Every probe ran in this session's scratchpad
(a fresh `CARGO_TARGET_DIR`, a clap 4.6.1 crate, a `toml` 0.8.23 crate), and none is committed.*

### Verdict: SPLIT

- **This spec (SPEC-153) [M]:** the ordering DEC, typed-order machinery for every op flag, `--crop`
  (both forms) with `--crop-gravity`, `--rotate`, `--flip`, `--save-recipe` built from what ran, the
  change to today's three flags, and the docs.
- **The split-off entry [S]:** `--pad` (`--pad-color`, `--pad-gravity`) and `--trim`
  (`--trim-color`, `--trim-tolerance`), on the machinery this spec builds. **It closes STAGE-058.**
  Filed in STAGE-058's backlog right after this entry.

Each half ships on its own. This half leaves `edit` with three new flags and a stated order; the
second adds two ops to a rule that already exists.

### Premise check: confirmed

- `crustyimg edit in.png --crop 30x20+0+0 -o a.png` → exit 2,
  `error: unexpected argument '--crop' found`. `--rotate 90` gives the same refusal. Neither flag exists.
- `edit` has three op flags (`src/cli/mod.rs:457-472`). `build_edit_ops` (`src/cli/ops.rs:1045`) appends
  `auto-orient` → `resize` → `invert`, whatever order they're typed in, and refuses zero flags (exit 2).
  Each op is built through `OperationRegistry::with_builtins`. A bad param is `CliError::Usage`, exit 2.
- `run_edit` (`ops.rs:1130`) captures the recipe **before** the run (`Recipe::from_ops(&ops)`), inserts
  `auto-orient` at step 0 unless `--auto-orient` was given (SPEC-111), and folds the ops onto
  `auto_orient_prefix()` (SPEC-110). So orientation is **always** baked first, and `--auto-orient` is
  an accepted no-op.
- `--save-recipe` writes `version = "1"`. A `version = "1"` recipe holding `crop` and `rotate` loads and
  runs (driven, exit 0), so the new steps need no version bump.
- **DEC-005 never states the canonical order.** It says a recipe is "an ordered list" and that the CLI
  and the loader build ops through one registry. The canonical order comes from SPEC-032 and lives
  only in code comments (which cite DEC-005 loosely) and `docs/cli-reference.md:306`. The ordering DEC
  **stands beside DEC-005**, and it supersedes SPEC-032's canonical-order statement, not a DEC.

### The order problem, driven

Input: a 60×40 RGB PNG. Recipes run with `apply` (`[[step]]` tables, crop `x = 0, y = 0, width = 0.5,
height = 1`):

- `crop` (left half) then `rotate angle = 90` → **40×30**
- `rotate angle = 90` then `crop` (left half) → **20×60**

A user typing `edit --rotate 90 --crop …` expects the second: turn the image, then crop what they see.
Under a fixed order that puts `crop` first, they would silently get the first, with exit 0.

**Today's three flags don't commute either.** On a 61×41 noise PNG (samples drawn from {0, 30, 255},
seed 7):

- recipe `[invert, resize max 30]` vs `[resize max 30, invert]`: 1789 of 1800 samples differ, max 124
- `edit --invert --resize-max 30` equals `[resize, invert]` byte for byte, which is the canonical order

A smooth gradient gave 0 differences in the same comparison, so it can't tell the orders apart.
`tests/edit.rs`'s `edit_flag_order_independent` uses a **solid-colour** 32×32 PNG, where the two orders
agree exactly, so it **can't fail**. Under any order rule, that test must be rebuilt on a fixture where
the orders differ (HANDOFF trap 10).

### The ordering rule: recommendation and DEC content

**Recommended: typed order for every op flag.** `edit` runs its op flags in the order they're typed,
like a recipe's steps, and `--save-recipe` writes them in that order.

Why:
- It's the order a user reads off their own command (`--rotate 90 --crop …` crops the turned image).
- `edit`'s command line maps one-to-one onto the recipe it saves. That's the brief's thesis ("the recipe
  step is the deliverable and the flag is the convenience"): the saved recipe reads like the command.
- A pixel length in a flag can then be stated in one sentence: it is in pixels of the image as it is at
  that point in the command.
- ImageMagick (`convert` / `magick`) runs its operators in sequence, so that's what users bring from
  other tools.

The rule's parts (DEC-114's content):

1. **Op flags run in typed order.** That covers `--resize-max`, `--invert`, `--crop`, `--rotate`, `--flip`,
   and later `--pad` and `--trim`. The index is clap's `ArgMatches::indices_of`, **filtered by
   `value_source() == ValueSource::CommandLine`**: driven, `indices_of` also returns an index for an
   **unset** flag that has a default (`auto_orient=false` at index 8), which would put defaulted flags
   into the order.
2. **`auto-orient` is not ordered. It's always first, and that is a rule, not a special case.** `edit`
   always bakes orientation through `auto_orient_prefix()` (SPEC-110), and the saved recipe always
   starts with `auto-orient` (SPEC-111). `--auto-orient` stays an accepted no-op wherever it's typed.
   Ordering it would break replay: `--rotate 90 --auto-orient` saved as `[rotate, auto-orient]` replays
   a mirrored tag (2, 4, 5, 7) differently from what `edit` ran (DEC-113 rule 3).
3. **Each op flag appears at most once.** Clap's default `ArgAction::Set` refuses a repeat:
   `error: the argument '--once <ONCE>' cannot be used multiple times`, kind `ArgumentConflict`, exit 2.
   A repeat (`--rotate 90 --crop … --rotate 90`) is a recipe's job; `--save-recipe` and editing the
   TOML is the escape hatch. Allowing repeats would make a modifier flag (`--crop-gravity`) ambiguous
   about which `--crop` it belongs to.
4. **Modifier flags have no position.** `--crop-gravity` (and, in the split-off half, `--pad-color`,
   `--pad-gravity`, `--trim-color`, `--trim-tolerance`) attach to their op wherever they're typed. Each
   `requires` its op (clap, exit 2). Rule 3 makes the attachment unambiguous.
5. **A pixel length in a flag is in pixels of the buffer that step receives.** It's converted to a
   DEC-112 fraction **when the step runs**, by `f64` division (`X / W`), and written to the recipe at
   full `f64` precision (`toml`'s shortest round-trip form, never a fixed number of decimals). The
   round trip back to pixels is exact (see below).
6. **`--save-recipe` writes what ran**: `auto-orient`, then each op step in run order, with the fractions
   step 5 resolved. The recipe is captured **after** the run, not before it as today, because the
   fractions need each step's buffer size.
7. **Stands beside DEC-005** (which never stated an order) and supersedes SPEC-032's canonical-order
   statement. DEC-112 is unchanged: no pixel unit enters a recipe.

**What it changes for today's flags:** only an invocation that types `--invert` **before**
`--resize-max`. Today that runs `resize` then `invert`; under the rule it runs `invert` then `resize`,
and the bytes differ (driven: 1789/1800 samples on noise). `--resize-max N --invert`, either flag
alone, and any `--auto-orient` placement keep their bytes. `--save-recipe` output is unchanged for
every invocation that keeps its bytes. This is a CHANGELOG "Changed" line, and
`edit_flag_order_independent` is rewritten to assert the opposite on a fixture that can tell.

**Alternative A, canonical positions** (no behaviour change):
`auto-orient → trim → rotate → flip → crop → resize-max → invert → pad`. That keeps today's
`auto-orient → resize → invert` relative order. It runs `trim` first so it strips an existing border,
crops in the turned image, pads after resize so `--pad 20` is 20 px in the output, and pads after
invert so the fill isn't inverted. It can't express directly:

- crop-then-rotate: reachable by transforming the rectangle, since quarter turns, flips and crops
  rewrite into one another
- resize-then-crop in output pixels
- pad-then-trim
- pad before resize
- invert after pad (an inverted fill)

None of these loses much. The cost is the silent wrong answer when typed order differs from the
canonical one (exit 0), and a seven-step order every user has to learn.

**Alternative B, typed order for the geometry flags only:** `invert` stays last. That keeps today's
bytes, but `--resize-max` is geometry, so it still changes `--invert --resize-max`, unless resize is
exempt too. The rule ends up hard to state.

**Alternative C, canonical positions but refuse out-of-order typing (exit 2):** no silent surprise, but
it refuses commuting pairs (`--invert --crop`) or needs a commutation table in the CLI.

If the maintainer refuses any behaviour change to today's three flags, take **A**.

### Each flag's syntax, and what `--save-recipe` writes

This spec:

- **`--crop WxH+X+Y`**: pixels in the buffer the step receives, all four required, `x` (U+0078) only.
  Writes `op = "crop"` with `x = X/W_img`, `y = Y/H_img`, `width = W/W_img`, `height = H/H_img` as
  `f64`. `WxH` with no offset, `0x…`, a sign, `%` or `×` are refused at parse, exit 2.
- **`--crop W:H`** (aspect), with optional **`--crop-gravity G`** (one of the nine `GRAVITY_NAMES`). The
  string is passed through `crop::parse_aspect`, so the op's message and its whole-number spelling of
  a decimal come along. Writes `aspect = "W:H"`, plus `gravity` only when given (crop keeps an absent
  key absent). `--crop-gravity` with the pixel form is refused at parse, exit 2 (the form is known
  from the string). `--crop-gravity` with no `--crop` is clap's `requires`, exit 2.
- **`--rotate DEGREES`**: checked by `ROTATE_PARAMS` and `Rotate::from_params`, so it takes what a recipe
  takes (a multiple of 90 from -360 to 360), and the op's message comes with the flag named.
  `--help` says "90, 180 or 270, clockwise". Writes `angle = N` as an integer.
- **`--flip horizontal|vertical`**: `FLIP_PARAMS`' `OneOf`. Writes `axis = "…"`. No `h`/`v` short
  forms: the flag value is the recipe value, so nothing has to be translated (cheap to add later).

The split-off half (recorded here so the rule covers it; that entry's frame confirms the details):

- **`--pad N|W:H`**: `N` whole pixels ≥ 1 of the buffer's long edge (writes `border = N/long`), or an
  aspect. **`--pad-color C` is required** (the op requires `color`; no CLI-only default). Optional
  `--pad-gravity` (aspect form only).
- **`--trim`** is a plain switch, with optional `--trim-color C` and `--trim-tolerance T` (0–255),
  written only when given.
- **Not** an optional-value `--trim[=COLOR]`. Driven, clap's `num_args = 0..=1` makes
  `edit --trim in.png` take `in.png` as the colour and fail with "the following required arguments
  were not provided", exit 2.

Driven in clap 4.6.1 (the repo's version):

- **Typed order:** `edit in.png --crop 1 --flip h --rotate -90 --invert` gave indices
  `3:crop, 5:flip, 7:rotate, 8:invert`.
- **Repeats:** an `ArgAction::Append` flag keeps both (`3:rotate=90, 5:crop, 7:rotate=90`). A default
  flag refuses a repeat, exit 2.
- **Conflicts:** `conflicts_with` gives `error: the argument '--pad-aspect <…>' cannot be used with
  '--pad-border <…>'`, kind `ArgumentConflict`, exit 2.
- **`requires`:** `MissingRequiredArgument`, exit 2.
- **A bad `OneOf`:** `error: invalid value 'x' for '--flip <FLIP>'`, exit 2.
- **A negative value** needs `allow_negative_numbers = true` on `--rotate`.

### The crop pixel round trip, driven

The chain: pixels → `x = X/W` (`f64`) → the `toml` 0.8.23 writer → read back → the DEC-112 load check
(`x + width > 1` refused) → `rect_edges`' `round(x·W)`, `round((x+width)·W)`. The same arithmetic
covers `y`/`height`, and `pad`'s `border = P/long` (`round(border·long)` is the left-edge case).

- **TOML text round trip:** every `X/n` for n = 1..4096 → 8,394,752 values, 0 changed a bit.
- **Full sweep:** every rectangle (X ≥ 0, W ≥ 1, X + W ≤ n) on every edge length n = 1..2048
  (Σ n(n+1)/2 = 1,433,753,600 rectangles). Load refusals 0, wrong left edges 0, wrong right edges 0.
- **Edge-touching (X + W = n), the load bound's worst case**, n = 1..50,000 (`MAX_EDGE`):
  Σ n = 1,250,025,000 rectangles. Load refusals 0, wrong right edges 0.
- **Random interior rectangles**, n uniform in 1..50,000 (xorshift, seed 153): 200,000,000. Bad 0.
  Largest `|edge − exact|` before rounding: **7.28e-12 px**, against the 0.5 px a wrong pixel needs.
- **Noise is present:** in the full sweep, `X/n + W/n` differs bitwise from `(X+W)/n` in 353,073,302
  cases. So the zero is not an untested path.
- **Mutant `floor` for `round`:** 199,815,286 wrong; the first is n = 7, X = 1, W = 4. The harness can fail.
- **Mutant "write 6 decimals"** (`format!("{:.6}")`): 228 rectangles in n ≤ 2048 replay as a **load
  refusal**. The smallest is **n = 640, X = 9, W = 631**, which writes `x = 0.014063`,
  `width = 0.985938`, sum `1.000001`. That's why rule 5 says full precision.
- **An `f32` conversion** (the control for the bound) also gave 0 load refusals on edge-touching
  rectangles to 50,000. So the `x + width > 1` check is not reachable by this conversion in either
  width, and only a lossy writer reaches it.

**Range stated:** exhaustive to 2048, edge-touching exhaustive to 50,000, random to 50,000. **No edge
input exists for the full-precision conversion.** The edge input for the trap is (640, 9, 631) at 6
decimals. The replay test should still carry a rectangle where `X/n + W/n ≠ (X+W)/n` bitwise and
`X + W = n` (any n ≥ 3 has one: 353M cases), so the test reaches the noisy path (trap 10).

### Where each failure happens, and its exit code

`edit` takes one input.

- **Parse, before any file is read (DEC-106), exit 2, `CliError::Usage`:**
  - `--crop` syntax
  - `--crop W:H` through `parse_aspect` (the op's message)
  - `--crop-gravity` against `GRAVITY_NAMES`, or with the pixel form
  - `--rotate` through `ROTATE_PARAMS` and `Rotate::from_params`
  - `--flip` through `FLIP_PARAMS`
  - a repeated op flag (clap)
  - a modifier with no op (clap `requires`)
  - zero op flags (today's rule, message extended to list the new flags)

  The message names the flag (DEC-106: `--rotate must be a multiple of 90, got 45; …`), not the key
  (`'angle' must be …`). The split-off half adds `--pad`, `--pad-color`, `--trim-color` and
  `--trim-tolerance` here. The only thing that needs the image is the pixel → fraction step.
- **Apply, when the step receives its buffer, exit 1 (`CliError::Operation(Apply)`,
  `operation 'crop' failed: …`):**
  - **A pixel rectangle past the edge of the buffer** (`X + W > W_img` or `Y + H > H_img`). This is
    checked in **pixels before** conversion. The conversion can't itself trip the op's fraction bound
    (driven above), but a rectangle that doesn't fit the image would. The message is `edit`'s own and
    names the flag and the buffer: `--crop 800x600+10+10 runs past the right edge of the 640×480 px
    image it receives`. The op's fraction message (`'x' + 'width' is 0.015625 + 1.25 …`) would mean
    nothing to a pixel user.
  - **An aspect crop that rounds to 0 px** (an extreme ratio on a small image): the op's own message
    (DEC-112 rule 4), which quotes `'aspect' = "W:H"` as the user typed it.
  - Exit 1, not 2: it follows DEC-112 rule 4 (an apply failure, never a skippable refusal), even though
    one input makes the refusal/failure split moot. Design may argue for 2 (DEC-104's "the input asked
    for something unusable"); a maintainer question below.
  - Split-off half: `pad` at a canvas past the size caps (the op's message), and `trim` on a whole match
    (the op's message quotes `'color'`/`'tolerance'`, which `edit` should word as
    `--trim-color`/`--trim-tolerance`).
  - **`pad`'s "rounds to 0 px" is unreachable from `--pad N`:** `N ≥ 1` is refused at parse, and the
    conversion is exact.
- **Load:** unchanged. An unreadable input is exit 3, an unknown format exit 4, as today.

### `--save-recipe` replays exactly: the test design should write

Compare **decoded pixels** (and colour type and dimensions), not file bytes. PNG output is
deterministic today, and `edit_save_recipe_round_trips_through_apply` compares bytes; pixels are the
claim, and bytes can stay as an extra assertion on PNG.

Matrix:

- **Each flag and form alone:** crop pixel, crop aspect, crop aspect with gravity, rotate 90/180/270,
  flip h/v.
- **Every non-commuting pair typed both ways:** crop × rotate, crop × flip, crop × resize-max (pixel
  crop after a resize converts against the **resized** buffer), resize-max × invert.
- **All of this half's flags together**, in two typed orders.
- **Colour types:** 8-bit RGB, 8-bit RGBA, 8-bit grey, 16-bit RGB.
- **A tagged input:** a non-square PNG with an `eXIf` Orientation 6 (`tests/rotate_flip.rs` already
  builds one). `edit --rotate 90 --crop …` crops the displayed (oriented) image, the saved recipe
  starts with `auto-orient`, and `apply` of it on the same tagged file gives the same pixels.

Each row also asserts the saved TOML's step list **in order** (not "contains"), so a run order and a
recipe order that drift apart fail on the recipe, not only on pixels. An input whose size at the crop
step differs from the decoded size (after `--resize-max` or `--rotate 90` on a non-square image) is
required: a converter that divides by the decoded size passes every other row (trap 10).

Mutation rows design should quote, each with its exact edit (SPEC-152's reflection):

- **M1:** in `build_edit_ops`, sort by a fixed table instead of the typed index → the reverse-order
  pair rows red.
- **M2:** drop the `value_source` filter → a defaulted flag gets an index. Design must find the row
  that reds, or say it's equivalent.
- **M3:** convert against the decoded size, not the step's buffer → the resize-then-crop row red.
- **M4:** write fractions with `{:.6}` → the (640, 9, 631) row red at `apply`'s load.
- **M5:** capture the recipe before the run → the saved TOML has no resolved fractions. It's a compile
  or logic error; design states which.
- **M6:** let `--auto-orient` take its typed position → the tagged Orientation-5/7 row red.

### Sizing: [M], and why it's split

The code is moderate: typed-order collection, three flags, a crop parser, a pixel-converting crop
wrapper, and a recipe captured after the run. The **verification** is what grows. Counted for this half:

- 6 flag forms (crop pixel, crop aspect, crop aspect + gravity, rotate, flip, plus `--resize-max`/
  `--invert` order)
- ~14 parse refusals (crop syntax ×6, crop-gravity ×3, rotate ×2, flip ×1, repeat ×1, zero-flags ×1)
- 4 non-commuting pairs × 2 typed orders = 8
- replay over 4 colour types × 6 forms = 24, plus 1 tagged row
- 2 apply failures
- 6 mutations

That's ~55 rows and a DEC, an [M].

All five at once adds pad's 2 forms, a required colour, gravity, and trim's 3 options: ~12 parse
refusals, 2 pairs × 2 orders, 4 × 5 replay rows, and 2 apply failures. That's ~40 rows more, which is
an [L]. Hence the split.

### Scope

**In:**
- the ordering rule (DEC-114, written at design)
- typed-order machinery for every op flag
- `--crop` (both forms) and `--crop-gravity`, `--rotate`, `--flip`
- the pixel → fraction conversion at the step
- `--save-recipe` captured after the run
- today's `--invert --resize-max` behaviour change, with its CHANGELOG line
- rewriting `edit_flag_order_independent` on a fixture that can fail
- a byte pin for today's three flags' surviving outputs (below)
- the docs

**Fold-ins: none recommended for this spec.** Each open STAGE-058 entry:

- **Recipe-output guard: report the file-set and line-count failures together** → leave. Test-only, and
  unrelated to `edit`.
- **`crop`'s aspect can follow each image's orientation** → leave. It adds a recipe key to two ops,
  which changes the recipe format and its own design questions. `--crop` can take it after it ships.
- **The batch summary says "watermark skipped"** → leave. None of the five uses the refusal hook (all
  fail at apply), and `edit` takes one input, so this spec doesn't make it reachable.
- **`apply` and `build` warn on a tagged input with no `auto-orient`** → leave. It changes `apply`/
  `build`, not `edit`, and `edit --save-recipe` already writes `auto-orient` first (unchanged here).
- **`pad` with a transparent fill** → leave. It's its own spec, and the split-off `--pad` flag will take
  `transparent` once it lands.
- **Pin `watermark`'s nine gravities** → leave. Test-only, and unrelated.
- **`parse_color` accepts a sign** → leave for this spec. **Recommend folding it into the split-off
  `--pad`/`--trim` half**, since that half opens two new CLI routes (`--pad-color`, `--trim-color`) into
  `parse_color`. That half's frame decides.

**What pins today's bytes, before and after.** Nothing pins `edit`'s output bytes today.
`scripts/cli-golden.py` has no `edit` row (grep: no match). `tests/edit.rs`' 9 tests compare one run
against another, on a solid-colour fixture. The recipe corpus guard (SPEC-141) doesn't run `edit`, so
it's untouched. Build should first record, on `main` before any change, the decoded-pixel hashes and
`--save-recipe` text for `--resize-max N`, `--invert`, `--resize-max N --invert` and `--auto-orient
--resize-max N` on a noise fixture, then assert them after.

**wasm:** `edit` is CLI only. `Commands::Edit`, `run_edit` and `build_edit_ops` appear only in
`src/cli/mod.rs`, `src/cli/ops.rs` and `tests/edit.rs` (grep), and `src/wasm.rs` doesn't mention them.
No wasm row, export or count changes.

**Docs, each place that states `edit`'s flags or order:**

- `src/cli/mod.rs:453-472`, the `Edit` doc and flag docs, which are `crustyimg edit --help`. The order
  sentence becomes the typed-order rule, with `auto-orient` always first. Add the three flags.
- `docs/cli-reference.md:300-314`. The heading's synopsis gains the flags. Rule "0." (the canonical
  order) becomes the typed-order rule, with the `--rotate 90 --crop` example.
- `docs/api-contract.md:535-545`. Synopsis and "ordered operation list (v1: …)" gain the flags and the
  rule.
- `docs/recipes.md:206-208`. "`edit` records only the ops it exposes" lists the new ones. Say that a
  pixel `--crop` is saved as fractions.
- `docs/USAGE.md:137`, `docs/data-model.md:187`, `README.md:262`: `--auto-orient --resize-max`
  examples that stay valid under typed order (orientation is positionless, and resize is the only op),
  so no change. The new `--rotate`/`--crop` example goes in `cli-reference.md`.
- `skills/crustyimg/SKILL.md`: has no `edit` section (grep: only "edit" as a verb in prose). Add a line
  only if the skill lists verbs' flags elsewhere; design checks.
- `CHANGELOG.md`: "Added" for the three flags, and "Changed" for typed order (`--invert --resize-max`).
- `docs/lab-plan-2026-08.md` and `docs/sessions/…`: historical, left as they are.

### Rough build file list

- `src/cli/mod.rs`: `Edit` gains `crop`, `crop_gravity`, `rotate`, `flip`, the help text, and the
  `Commands::Edit` dispatch, which passes the `ArgMatches` order.
- `src/cli/ops.rs`: `build_edit_ops` builds in typed order. It gains a `--crop` parser and a pixel-crop
  step that resolves fractions from the buffer it receives and records them. `run_edit` builds the
  saved recipe after the run.
- Possibly `src/operation/crop.rs`: make a small helper `pub(crate)` (for example `parse_aspect` is
  already `pub(super)`). No change to the op's behaviour or key table.
- `tests/edit.rs`: rewrite `edit_flag_order_independent`, and add the byte pin for today's flags.
- `tests/edit_geometry.rs` (new): the flag, refusal, order and replay matrix.
- `decisions/DEC-114-…md`: written at design.
- The docs listed above, and `CHANGELOG.md`.

### Questions only the maintainer can answer

1. **The ordering rule.** Typed order for every op flag (recommended), or canonical positions
   (Alternative A), which change nothing today? *Recommend typed order.*
2. **Today's three flags.** May `edit --invert --resize-max N` change its output, as typed order does?
   *Recommend yes. It's pre-1.0, it gets a CHANGELOG "Changed" line, and nothing pins the old bytes. If
   no, take Alternative A.*
3. **`--crop` syntax.** `WxH+X+Y` pixels and `W:H` aspect only, with no fraction or percent form on the
   flag? *Recommend yes. A recipe carries fractions; a flag user thinks in pixels or ratios.*
4. **A pixel crop past the image's edge: exit 1 or 2?** *Recommend 1 (an apply failure, as DEC-112
   rule 4), with the flag and the buffer size named.*
5. **Split.** This spec (DEC, crop, rotate, flip), then `--pad`/`--trim`, which closes STAGE-058?
   *Recommend yes.*

### Maintainer rulings (2026-11-10)

All five as recommended:

1. **Typed order.** Every op flag runs in the order typed; `auto-orient` is always first; each op flag
   appears at most once. Asked why typed order over fixed positions, the maintainer weighed
   Alternative A (flag order never matters) and chose typed order.
2. **`edit --invert --resize-max N` may change its output,** with a CHANGELOG "Changed" line.
3. **`--crop` takes pixels (`WxH+X+Y`) and aspect (`W:H`, optional `--crop-gravity`) only,** with no
   fraction or percent form on the flag.
4. **A pixel crop past the edge of the image it receives is exit 1,** with the flag and the image size
   named.
5. **Split as filed.** SPEC-153 is the DEC, `--crop`, `--rotate` and `--flip`. `--pad`/`--trim` is the
   entry after it and closes STAGE-058.

### Out of scope (for this spec specifically)

- `--pad` and `--trim` flags: the split-off STAGE-058 entry after this one, which closes the stage.
- `pad` with a transparent fill: its own STAGE-058 entry.
- `crop`'s aspect following each image's orientation: its own STAGE-058 entry.
- `apply`/`build` warning on a tagged input with no `auto-orient`: its own STAGE-058 entry.
- `parse_color`'s sign: its own STAGE-058 entry, recommended for the `--pad`/`--trim` half.
- Any-angle `--rotate`: STAGE-059. `--rotate 45` keeps the op's "not supported yet" message.
- The ddphotos page: STAGE-060.
- `edit` flags for ops outside the five (`watermark`, `optimize`): not planned. `watermark` and
  `optimize` have their own verbs, and a recipe holds them.
- Repeated op flags: refused (rule 3). A recipe is the route.
- Short forms `h`/`v` for `--flip`: not now.
- A recipe `version` bump: not needed (driven).


## Design

*Designed 2026-11-10 on `main` at `e611315` (0.8.2). Every probe ran in this session's scratchpad:
a release build of `main` and a prototype of this design, each in its own fresh
`CARGO_TARGET_DIR`. The prototype was never committed or pushed, and both were deleted at the end.
DEC-114 is written: `decisions/DEC-114-edit-runs-its-op-flags-in-the-order-typed-and-saves-what-ran.md`.*

### Design corrections to the frame

1. **`edit` does not take one input.** Its `input` goes through `source::resolve_all`
   (`src/cli/ops.rs:337`), so a directory or a glob runs a batch. Driven on `main`:
   `edit bdir --resize-max 30 --out-dir o --save-recipe b.toml` exits 0, writes 3 images and one
   recipe. A pixel `--crop` is converted per image, so one saved recipe can't record it for a batch.
   Decided (DEC-114 part 6): with `--save-recipe`, a pixel `--crop` needs one input image, and a
   batch is refused with exit 2 before any image is read (message P16). Without `--save-recipe`,
   a batch crops each image in its own pixels, and an image too small for the rectangle fails
   that input (exit 6 for the run, the others written). An aspect `--crop` has no pixel length
   and saves fine from a batch. Rows AC-18, FT-28 to FT-30.
2. **No edge-touching rectangle has `X/n + W/n ≠ (X+W)/n`.** The frame asked the replay test for
   one with `X + W = n` ("any n ≥ 3 has one: 353M cases"). Driven exhaustively in `f64`
   (`rustc -O` probe): for every n = 1..50,000 and every X, `X/n + (n−X)/n` is exactly `1.0`,
   1,250,025,000 rectangles, 0 exceptions. The frame's 353,073,302 counted interior rectangles
   (`X + W < n`). The detector can fail: on interior rectangles it finds 470 cases at n = 60
   (first `X = 1, W = 5`), 160 at n = 40. So the replay test carries an **interior** noisy
   rectangle (`--crop 5x40+1+0` on 60×40: `1/60 + 5/60 ≠ 6/60` bitwise) and a separate
   edge-touching one (`--crop 47x40+13+0`), each replayed (FT-12).
3. **The 6-decimal mutant has no refusal in n ≤ 512.** Through the crate's real path (below), the
   `{:.6}` writer gave 0 refusals over all 22,500,864 rectangles with n ≤ 512, and 0 over
   12,988,905 edge-touching ones. It refuses only the frame's (640, 9, 631). So that rectangle
   must be in the test literally (FT-12's first row), not reached by a sweep.
4. **`--auto-orient` no longer adds a second `auto-orient` step.** Today `build_edit_ops` pushes an
   `auto-orient` op when the flag is typed, and it runs a second time after the prefix (a no-op).
   Under DEC-114 part 2 it adds no step; the bytes don't change (byte pin row 4).
5. **An aspect crop that rounds to 0 px doesn't quote `'aspect'`.** The op's message
   (`src/operation/crop.rs:366-370`) is `the crop rounds to {cw}×{ch} px on this {iw}×{ih} px
   image; it must keep at least 1 px each way`, with no key in it, so `edit` passes it through as
   is (A3).
6. **`--crop -1x10+0+0` never reaches `edit`'s parser.** Clap reads `-1…` as a flag:
   `error: unexpected argument '-1' found`, exit 2. Only `--crop=-1x10+0+0` reaches the parser.
   Both are rows (P1, P3).

### Drives the prompt asked for

- **The crop round trip, through the crate.** The frame's 2.68e9 rectangles ran in a Python model
  of the arithmetic (its frame says "a `toml` 0.8.23 crate" for the text round trip, and Python for
  the sweep). Re-driven through the real code in a `#[test] #[ignore]` probe in the prototype:
  `crop_fractions` (the converter below) → `Recipe::to_toml` (`src/recipe/mod.rs:290`, the writer
  `--save-recipe` uses) → `Recipe::from_toml` → `OperationRegistry::build("crop")`, which runs
  `Crop::from_params`' `x + width > 1` check (`src/operation/crop.rs:163`) → `rect_edges`
  (`crop.rs:197`, made `pub(crate)` in the probe only). Results:
  - (640, 9, 631): 0 refused, 0 wrong edges.
  - n = 1..512 exhaustive (Σ n(n+1)/2 = 22,500,864 rectangles): 0 refused, 0 wrong, 5,447,156
    noisy (`X/n + W/n ≠ (X+W)/n`).
  - Edge-touching (`X + W = n`), n = 1, 98, 195, … (stride 97, 516 lengths) plus 49,999 and
    50,000: 12,988,905 rectangles, 0 refused, 0 wrong.
  - Through `Crop::apply` on real 8-bit grey pixels, n = 1..64 exhaustive (Σ = 45,760): 0 wrong
    (output width = W and first pixel = X).
  - Control, the `{:.6}` writer: (640, 9, 631) refused at load (`'x' + 'width' is 0.014063 +
    0.985938`), and 0 refusals elsewhere in both sweeps (correction 3).
  - Runtime: 548.6 s for the probe, `--release`.
- **Rule 2's replay claim.** 6×4 RGB noise PNGs with an `eXIf` Orientation tag, recipes run with
  `apply` on the release build of `main`. `[rotate 90, auto-orient]` vs `[auto-orient, rotate 90]`:
  Orientation 5: 62 of 72 samples differ; Orientation 7: 62 of 72; controls Orientation 1: 0;
  Orientation 6: 0. On the prototype, `edit o5.png --rotate 90 --auto-orient --save-recipe` saved
  `[auto-orient, rotate]`, and its replay differed from the edit in 0 samples, from
  `[rotate, auto-orient]` in 62. This is M6's splitting input (FT-14, Orientation 5 row).
- **The `value_source` filter (M2).** Not equivalent. Driven: with the filter removed, `--invert`
  (a `SetTrue` switch, default `false`) gets an index when not typed, and the order gains an
  `invert` step: `--rotate 90 --crop 30x40+0+0` saved `["auto-orient", "rotate", "crop", "invert"]`,
  and `edit in.png` with no op flag exited 0 instead of 2. 8 tests red (mutation table).
  `--auto-orient` also gets a default index, but it isn't in the order list. The `Option` flags
  (`--resize-max`, `--crop`, `--rotate`, `--flip`) have no default and get no index when absent.
- **Indices for valued flags.** Driven on the prototype (`typed_order_ignores_spelling_and_position`
  and `drive` rows b–f): `--rotate 90 --crop 30x40+0+0`, `--rotate=90 --crop=30x40+0+0`, both flags
  before the input path, and one on each side of it all saved `[auto-orient, rotate, crop]` and
  gave identical pixels (30×40). `--crop 30x40+0+0 <input> --rotate 90` saved
  `[auto-orient, crop, rotate]` (40×30).
- **`--save-recipe` on a failed run, on `main` today.** No recipe is written: `--resize-max 0` (exit 2,
  `'width' must be an integer >= 1, got 0`), an unwritable `-o /nonexistent_dir/x.png` (exit 5),
  an undecodable input (exit 4). `run_edit` writes the recipe only after `run_pixel_op` returns
  `Ok` (`src/cli/ops.rs:1178-1184`). The rule stays: **a failed run writes no recipe**, now also
  for the new exit-1 apply failures (FT-15). No change from today.
- **Negative angles.** `--rotate` gets `allow_negative_numbers = true` (an `Arg` setting, so it
  affects only `--rotate`'s own value). Driven: `--rotate -90` and `--rotate=-90` both run
  (40×60). An input named `-g.png` typed bare is refused the same way on `main` and on the
  prototype (`error: unexpected argument '-g' found`, exit 2), and `edit --rotate -90 -o x.png -y
  -- -g.png` runs on both. `--rotate -- -90` is clap's `a value is required for '--rotate
  <DEGREES>'`. Decided: keep `allow_negative_numbers` on `--rotate` only.
- **The byte pin.** Recorded from a release build of `main` at `e611315` (`crustyimg 0.8.2`). The
  fixture is 61×41 8-bit RGB, written by the test itself, seeded independently of `crustyimg`:
  xorshift64 with state `s = 7`, each sample `[0, 30, 255][s % 3]` after one step
  `s ^= s << 13; s ^= s >> 7; s ^= s << 17` (wrapping, `u64`), row-major, R then G then B. The
  hash is FNV-1a 64 (offset `0xcbf29ce484222325`, prime `0x100000001b3`) over the decoded samples
  (`to_rgb8().as_raw()`). The pins were computed by a stdlib-only Python PNG decoder in the
  scratchpad, not by `image` or `crustyimg`, and agree with the prototype's Rust test. Decoder
  control: a recipe `[invert, invert]` replayed the input's own hash, `0x25fdc2dd88e14331`;
  seed 8 gives `0xb38270fd6c257d4c`, so the seed is reaching the pixels.

  | Invocation (input `n.png`, then `-o OUT -y --save-recipe R`) | Output | Hash on `main` | Hash after |
  |---|---|---|---|
  | `--resize-max 30` | 30×20 RGB8 | `0x01a031512f8d2eaf` | same |
  | `--invert` | 61×41 RGB8 | `0xfd42fcdce5e3c92e` | same |
  | `--resize-max 30 --invert` | 30×20 RGB8 | `0x62d0525855cff6a7` | same |
  | `--auto-orient --resize-max 30` | 30×20 RGB8 | `0x01a031512f8d2eaf` | same |
  | `--invert --resize-max 30` (**Changed**) | 30×20 RGB8 | `0x62d0525855cff6a7` | `0x5d248c136b8b3ad7` |

  The "after" value for the Changed row is the hash of `apply` running the hand-written recipe
  `[auto-orient, invert, resize max 30]` on `main`, so it was fixed before the prototype existed.

  Saved recipe text on `main`, byte for byte (each line ends `\n`; the same after the change):
  - `--resize-max 30` and `--auto-orient --resize-max 30`:
    `version = "1"\n\n[[step]]\nop = "auto-orient"\n\n[[step]]\nop = "resize"\nmode = "max"\nwidth = 30\n`
  - `--invert`: `version = "1"\n\n[[step]]\nop = "auto-orient"\n\n[[step]]\nop = "invert"\n`
  - `--resize-max 30 --invert`:
    `version = "1"\n\n[[step]]\nop = "auto-orient"\n\n[[step]]\nop = "resize"\nmode = "max"\nwidth = 30\n\n[[step]]\nop = "invert"\n`
  - `--invert --resize-max 30`: on `main` the same text as `--resize-max 30 --invert`; after,
    `version = "1"\n\n[[step]]\nop = "auto-orient"\n\n[[step]]\nop = "invert"\n\n[[step]]\nop = "resize"\nmode = "max"\nwidth = 30\n`.

### The design

**Collecting the typed order (`src/cli/mod.rs`).**
- `run()` (`src/cli/mod.rs:869-870`) keeps the `ArgMatches`:
  `let matches = Cli::command().get_matches();` then `Cli::from_arg_matches(&matches)` (on `Err`,
  `e.format(&mut Cli::command()).exit()`, which is what `Cli::parse()` does). Add `FromArgMatches`
  to the `clap` import (`mod.rs:20`).
- A new `pub enum EditOp { ResizeMax, Invert, Crop, Rotate, Flip }` and
  `pub(crate) fn edit_op_order(m: &clap::ArgMatches) -> Vec<EditOp>`: for each `(id, op)` in a
  fixed list, keep it only if `m.value_source(id) == Some(ValueSource::CommandLine)`, take
  `m.index_of(id)`, sort by index, return the ops. `--auto-orient` is not in the list.
- `Commands::Edit` gains `crop: Option<String>` (`value_name = "WxH+X+Y|W:H"`),
  `crop_gravity: Option<String>` (`value_name = "GRAVITY"`, `requires = "crop"`),
  `rotate: Option<i64>` (`value_name = "DEGREES"`, `allow_negative_numbers = true`),
  `flip: Option<String>` (`value_name = "AXIS"`), and `#[arg(skip)] order: Vec<EditOp>`, which
  `run()` fills from `matches.subcommand()` when it is `("edit", sub)`.
- The `Commands::Edit` dispatch (`mod.rs:1083-1096`) passes an `EditFlags` struct
  (`auto_orient`, `resize_max`, `crop`, `crop_gravity`, `rotate`, `flip`, `order`) to `run_edit`.
  The `invert` field stays on the struct (clap needs it) but is read from `order` only.
- Why `Option<i64>` for `--rotate`: the flag takes whole degrees, and clap refuses `90.0` and `abc`
  itself (P13). The op's range and multiple-of-90 checks then run on the integer.

**Building the steps (`src/cli/ops.rs`).** `build_edit_ops(flags: &EditFlags) -> Result<Vec<EditStep>, CliError>`
replaces today's (`ops.rs:1045-1096`):
- Zero op flags and no `--auto-orient` → P15.
- For each `EditOp` in `flags.order`, in order, build its step. Presence comes from `order` only
  (one source): a value field that is `None` for an op in `order` is an internal `CliError::Usage`.
- Every op is built through `OperationRegistry::with_builtins().build(op, params)`, as today. A
  `RegistryError::InvalidParams { reason }` becomes `CliError::Usage(reason)` with the quoted key
  replaced by the flag (`'angle'` → `--rotate`, `'axis'` → `--flip`, `'aspect'` → `--crop`,
  `'gravity'` → `--crop-gravity`), first occurrence only (`str::replacen(…, 1)`). So the op's own
  text arrives with the flag named (DEC-106), from one helper, `build_flag_op(op, params, keys)`.
- `--rotate N` → `{angle = N}` (TOML integer). `--flip A` → `{axis = "A"}`.
- `--crop` → `parse_crop_flag(s)`:
  - contains `:` → the aspect form: `{aspect = s}` plus `{gravity = G}` only when `--crop-gravity`
    was given, built through the registry (the op runs `parse_aspect`, `crop.rs:226`, and the
    `GRAVITY_NAMES` table). No new `pub` in `crop.rs`.
  - else the pixel form: `W`, `x` (U+0078 only), `H`, `+`, `X`, `+`, `Y`, each number non-empty
    ASCII digits and fitting a `u32`; anything else is P1. `W = 0` or `H = 0` is P2.
    `--crop-gravity` with the pixel form is P7.
- A pixel crop becomes a CLI-only step, `PixelCrop { w, h, x, y, typed, resolved }`, with
  `name() = "crop"`. Its `apply(img)`:
  1. reads the size of the image it receives, `(iw, ih)`;
  2. if `x + w > iw` (in `u64`) → A2 (right edge); if `y + h > ih` → A1 (bottom edge), as
     `OperationError::Apply { op: "crop", reason }`;
  3. `params = crop_fractions(w, h, x, y, iw, ih)`: `x/iw`, `y/ih`, `w/iw`, `h/ih` as
     `f64::from(…) / f64::from(…)`, each a `toml::Value::Float`;
  4. builds `Crop::from_params(&params)` and applies it (any error → `Apply` with its text);
  5. stores `params` in `resolved: Rc<RefCell<Option<OperationParams>>>` and returns the image.
  It has no `refusal`. `Operation` has no `Send` bound (`src/operation/mod.rs:305`), so `Rc` is
  enough (driven: compiles clean with no warnings).
- `EditStep { op, record }`, where `record` is `Built(op.params())` for every step but the pixel
  crop, which holds a clone of the `Rc`.

**Running and saving (`run_edit`, `ops.rs:1130-1187`).**
1. `build_edit_ops` (all parse refusals, before any file is read).
2. If `--save-recipe` is given, a step is a pixel crop, and `input != "-"`: resolve the input
   (`source::resolve_all(&[input], &mut std::io::empty(), OnEmpty::Fail)`); more than one image →
   P16. This reads no image (a missing path is exit 3, as before).
3. Fold the steps onto `auto_orient_prefix()` (`src/cli/optimize.rs:887`), keeping each step's
   `(name, record)`.
4. `run_pixel_op(…)?` as today.
5. Only then, with `--save-recipe`: a recipe with `version = "1"`, an `auto-orient` step, then one
   step per record in order (`Built(p)` as is; `Resolved` read from the cell, and an empty cell is
   an internal `OperationError::Apply`, exit 1). Then the symlink guard, `to_toml`, write, as today
   (`ops.rs:1178-1184`).

**What `--help` says** (driven on the prototype; build may reflow lines but keeps the sentences):
`Op flags run in the order you type them, like a recipe's steps: --rotate 90 --crop 300x200+0+0
turns the image, then crops what it shows. EXIF orientation is always baked first, wherever
--auto-orient is typed. Each op flag may be given once. At least one op flag is required.`
Flag lines: `--crop <WxH+X+Y|W:H>  Crop to WxH+X+Y (pixels of the image at this point in the
command) or to the largest W:H aspect that fits`; `--crop-gravity <GRAVITY>  Where an aspect
--crop sits: center (default), north, south, east, west, northeast, northwest, southeast,
southwest`; `--rotate <DEGREES>  Rotate clockwise by DEGREES: 90, 180 or 270 (negative turns
counter-clockwise)`; `--flip <AXIS>  Mirror the image: horizontal or vertical`. `--resize-max`
loses "(canonical order: after auto-orient, before invert)". `--auto-orient` adds "Always runs
first, wherever it is typed".

### Every message, verbatim

Each is the whole of stderr, exit 2, nothing written (no output, no recipe), before any file is
read (DEC-106). No message names a spec, stage or DEC.

- **P1** (`--crop 10X10+0+0`, `10x10`, `10×10+0+0`, `--crop=-1x10+0+0`, `+10x10+0+0`,
  `10x10+0+0+0`, `99999999999x1+0+0`): `error: --crop must be WxH+X+Y in pixels, such as
  800x600+10+20, or an aspect W:H, such as 16:9; got "10X10+0+0"` (the `got` is `{:?}` of what was
  typed). New text.
- **P2** (`--crop 0x10+0+0`, `10x0+0+0`): `error: --crop must keep at least 1 px each way; got
  "0x10+0+0"`. New text.
- **P3** (`--crop -1x10+0+0`, space-separated): clap's `error: unexpected argument '-1' found`,
  then a blank line, `  tip: to pass '-1' as a value, use '-- -1'`, usage and help lines
  (`clap_builder-4.6.0/src/error/format.rs:358`). Assert the first line.
- **P4** (`--crop 0:1`): `error: --crop must be two whole numbers from 1 to 65535 written W:H, such
  as "16:9"; got "0:1"` (`src/operation/crop.rs:229`, key renamed).
- **P5** (`--crop 1.5:1`): `error: --crop must be two whole numbers written W:H; write "1.5:1" as
  "3:2"` (`crop.rs:258`, key renamed).
- **P6** (`--crop 16:9 --crop-gravity middle`): `error: --crop-gravity must be one of: center,
  north, south, east, west, northeast, northwest, southeast, southwest; got "middle"`
  (`src/operation/registry.rs:263` with `GRAVITY_NAMES`, `src/operation/mod.rs:1207`).
- **P7** (`--crop 10x10+0+0 --crop-gravity north`): `error: --crop-gravity only places an aspect
  --crop W:H; a pixel --crop WxH+X+Y is placed by its +X+Y`. New text.
- **P8** (`--crop-gravity north --invert`, no `--crop`): clap's `error: the following required
  arguments were not provided:` then `  --crop <WxH+X+Y|W:H>` (`format.rs:251`). Assert both lines.
- **P9** (`--flip h`): `error: --flip must be one of: horizontal, vertical; got "h"`
  (`registry.rs:263` with `AXIS_NAMES`, `src/operation/flip.rs:14`).
- **P10** (`--flip` with no value): clap's `error: a value is required for '--flip <AXIS>' but none
  was supplied` (`format.rs:219`).
- **P11** (`--rotate 45`): `error: --rotate must be a multiple of 90, got 45; rotating by any other
  angle is not supported yet` (`src/operation/rotate.rs:71`, key renamed).
- **P12** (`--rotate 450`): `error: --rotate must be a number in -360.0..=360.0, got 450`
  (`registry.rs:263` with `ParamKind::expectation`, `registry.rs:134`, and `ROTATE_PARAMS`,
  `rotate.rs:18`).
- **P13** (`--rotate 90.0`, `--rotate abc`): clap's `error: invalid value '90.0' for '--rotate
  <DEGREES>': invalid digit found in string` (`format.rs:323`).
- **P14** (`--rotate 90 --rotate 90`, `--crop 1:1 --crop 2:1`, `--flip vertical --flip vertical`):
  clap's `error: the argument '--rotate <DEGREES>' cannot be used multiple times`
  (`format.rs:158`). Already true of today's flags on `main` (`--invert --invert`,
  `--resize-max 30 --resize-max 20`, driven).
- **P15** (no op flag): `error: edit requires at least one operation flag (--auto-orient,
  --resize-max, --invert, --crop, --rotate, --flip)`. Today's text (`ops.rs:1090`) with three flags
  added.
- **P16** (`edit DIR --crop 10x10+0+0 --out-dir O --save-recipe R`, DIR holding 3 images):
  `error: --save-recipe with a pixel --crop needs one input image, and "DIR" names 3: the crop is
  saved as fractions of the image it ran on; use an aspect --crop W:H, or edit each image on its
  own` (`DIR` as typed, `{:?}`). New text.

Apply failures, exit 1, no output and no recipe written:

- **A1** (`--resize-max 30 --crop 30x21+0+0` on 60×40): `error: operation 'crop' failed: --crop
  30x21+0+0 runs past the bottom edge of the 30×20 px image it receives`. The size is the image
  the step receives (30×20), not the decoded 60×40.
- **A2** (`--rotate 90 --crop 41x10+0+0` on 60×40): `error: operation 'crop' failed: --crop
  41x10+0+0 runs past the right edge of the 40×60 px image it receives`. The `×` is U+00D7. Right
  is checked before bottom.
- **A3** (`--crop 65535:1` on 60×40): the op's own message, `error: operation 'crop' failed: the
  crop rounds to 60×0 px on this 60×40 px image; it must keep at least 1 px each way`
  (`crop.rs:366-370`). Driven on the prototype: exit 1, no output, no recipe.
- In a batch, A1/A2 fail that input: `error: <path>: operation 'crop' failed: --crop 30x30+0+0
  runs past the right edge of the 20×50 px image it receives`, then `error: 1 of 3 inputs
  failed`, exit 6, the other images written (driven, prototype).

### Size after design: [M] holds, at its upper edge

- **Code:** two source files. `src/cli/mod.rs` (the `Edit` fields, `EditOp`, `edit_op_order`, `run()`
  keeping the matches, the dispatch) and `src/cli/ops.rs` (`EditFlags`, `parse_crop_flag`,
  `crop_fractions`, `PixelCrop`, `build_flag_op`, `build_edit_ops`, `run_edit`). The prototype's
  diff for both (`git diff --stat -- src/` on the scratch worktree): 455 lines added and 180
  removed, of which about 75 added lines are the throwaway round-trip probe.
  No change in `src/operation/`.
- **Rows:** 26 refusal + 51 replay + 4 apply-failure + 5 pin = **86** (derivation under Failing
  Tests), against the frame's ~55. The growth is the colour-type replay (24, which the frame also
  counted) plus three batch rows the frame didn't know about (correction 1) and one row per
  refusal input. 77 of the 86 sit in table-driven tests that share one `replay` helper and one
  refusal loop, so the work grows less than the count.
- **Mutation rows:** 15 plus the trap-14 control, all driven here; build re-drives them.
- **Verdict:** [M]. Nothing else fits in it: the `--pad`/`--trim` half stays split.

## Inputs

- **Files to read:** `src/cli/mod.rs:453-473` (`Edit`), `:869-880` (`run`), `:1083-1096`
  (dispatch); `src/cli/ops.rs:324-470` (`run_pixel_op`), `:1027-1187` (`build_edit_ops`,
  `run_edit`), `:1870-1930` (the four unit tests to rewrite); `src/cli/optimize.rs:887`
  (`auto_orient_prefix`); `src/operation/crop.rs` (key table, `from_params`, `rect_edges`,
  `parse_aspect`, `apply`); `src/operation/rotate.rs:18-80`; `src/operation/flip.rs:14-60`;
  `src/operation/registry.rs:119-141, 246-268`; `src/recipe/mod.rs:269-292`; `tests/edit.rs`.
- **External APIs:** clap 4.6.1 / clap_builder 4.6.0: `ArgMatches::index_of`, `value_source`,
  `ValueSource::CommandLine`, `FromArgMatches`, `Arg::allow_negative_numbers`, `requires`.
- **Related code paths:** `src/cli/`, `src/operation/`, `src/recipe/`.

## Outputs

- **Files created:** `tests/edit_geometry.rs` (FT-1 to FT-31 below except those in `tests/edit.rs`
  and `src/cli/`); `decisions/DEC-114-…md` (written at design).
- **Files modified:** `src/cli/mod.rs`, `src/cli/ops.rs`, `tests/edit.rs`, `CHANGELOG.md`,
  `docs/cli-reference.md`, `docs/api-contract.md`, `docs/recipes.md`, `recipes/README.md`.
- **New exports:** none outside the binary crate's `cli` module (`EditOp`, `edit_op_order`,
  `EditFlags` are `pub(crate)` or narrower).
- **Database changes:** none.

## Acceptance Criteria

Each AC names its tests. "FT-n" is the Failing Tests list below.

- [ ] **AC-1 DEC-114.** `decisions/DEC-114-edit-runs-its-op-flags-in-the-order-typed-and-saves-what-ran.md`
  states the seven parts, the alternatives (A declined by the maintainer 2026-11-10) and the
  relations to DEC-005, DEC-112 and DEC-113. Verify reads it; `just decisions-audit` passes.
- [ ] **AC-2 Typed order, pairs.** Each non-commuting pair typed both ways runs in typed order, saves
  its steps in that order, replays through `apply` to the same decoded pixels, and the two orders
  differ: crop × rotate (40×30 vs 30×40), crop × flip, crop × resize-max, resize-max × invert.
  FT-6.
- [ ] **AC-3 All flags, two orders.** `--resize-max 50 --rotate 90 --flip vertical --crop 20x30+3+4
  --invert` and its reverse save `[auto-orient, resize, rotate, flip, crop, invert]` and
  `[auto-orient, invert, crop, flip, rotate, resize]`, give 20×30 and 30×20, and replay. FT-7.
- [ ] **AC-4 Spelling and position.** `--rotate 90 --crop 30x40+0+0`, the `=` spellings, both flags
  before the input, and one on each side save `[auto-orient, rotate, crop]` with identical pixels.
  FT-9. And `edit_op_order` returns the typed order from parsed matches. FT-31.
- [ ] **AC-5 Only typed flags add steps.** A flag clap defaults (the `--invert` switch left off) adds
  no step: `--resize-max 30` alone saves `[auto-orient, resize]` and keeps its pin (FT-1 row 1);
  `edit in.png` with no op flag is P15 (FT-24); `edit_op_order` on `edit in --resize-max 3` is
  `[ResizeMax]` (FT-31).
- [ ] **AC-6 `auto-orient` first and positionless.** The saved recipe always starts with one
  `auto-orient`. `--auto-orient` typed anywhere changes nothing: FT-1 row 4 keeps its pin; a
  tagged Orientation-6 **and** an Orientation-5 input with `--rotate 90 --crop 4x3+1+0
  --auto-orient` give 4×3, save `[auto-orient, rotate, crop]`, and replay (FT-14, two rows).
  `--auto-orient` alone exits 0 and saves `[auto-orient]` (FT-25).
- [ ] **AC-7 One of each.** A repeated `--rotate`, `--crop` or `--flip` is P14. FT-22 (three rows).
- [ ] **AC-8 `--crop` pixel form.** `WxH+X+Y` parses to four `u32`s; refusals P1 (seven inputs),
  P2 (two), P3, each its own row. FT-18, FT-32.
- [ ] **AC-9 `--crop` aspect form and gravity.** `--crop 1:1` gives 40×40 centred, with
  `--crop-gravity east` 40×40 at the right; saved `aspect` and, only when given, `gravity`.
  Refusals P4, P5, P6, P7, P8. FT-5 rows 2–3, FT-19, FT-8 (gravity in the saved recipe).
- [ ] **AC-10 `--rotate`.** 90, 180, 270 and -90 run and save `angle` as a TOML integer; P11, P12,
  P13 (two inputs). `--rotate=-90` runs too, and an input path that starts with `-` still works
  after `--` with `--rotate -90` (`allow_negative_numbers` is on `--rotate` only). FT-5 rows 4–7,
  FT-20, FT-23.
- [ ] **AC-11 `--flip`.** `horizontal` and `vertical` run and save `axis`; P9, P10. FT-5 rows 8–9,
  FT-21.
- [ ] **AC-12 Zero op flags.** P15 verbatim. FT-24 (and `tests/edit.rs`' `edit_no_ops_exits_2`
  stays green).
- [ ] **AC-13 Pixel lengths are of the image the step receives.** After `--resize-max 30` and after
  `--rotate 90` on 60×40, `--crop 7x3+11+5` saves `x = 11/rw`, `y = 5/rh`, `width = 7/rw`,
  `height = 3/rh` exactly (`f64` equality, with `(rw, rh)` = (30, 20) and (40, 60)), and replays.
  FT-11, FT-33.
- [ ] **AC-14 Full precision.** `--crop 631x2+9+0` on 640×2 replays (the 6-decimal trap); the
  interior noisy `--crop 5x40+1+0` and the edge-touching `--crop 47x40+13+0` on 60×40 replay.
  FT-12.
- [ ] **AC-15 Past the edge.** A1 and A2 verbatim, exit 1, no output, no recipe. FT-15 (two rows).
- [ ] **AC-16 Aspect that rounds to 0 px.** A3 verbatim, exit 1, nothing written. FT-17.
- [ ] **AC-17 A failed run writes no recipe.** FT-15 and FT-17 assert the recipe path doesn't exist;
  `edit_save_recipe_unwritable_exits_5` stays green. Same as `main`.
- [ ] **AC-18 Batch.** A pixel `--crop` on a directory crops each image in its own pixels (both
  15×15); one image narrower than the rectangle fails that input (exit 6, the other written);
  with `--save-recipe` it is P16 and nothing is written; an aspect `--crop 1:1` on the same
  directory with `--save-recipe` exits 0 and saves. FT-28, FT-29, FT-30.
- [ ] **AC-19 Each flag alone replays.** Pixel crop, aspect crop, aspect + gravity, rotate
  90/180/270/-90, flip horizontal/vertical: dims, saved steps in order, decoded pixels equal to
  the replay. FT-5 (9 rows).
- [ ] **AC-20 Colour types.** 8-bit RGB, 8-bit RGBA, 8-bit grey and 16-bit RGB, each through six
  forms (pixel crop, aspect + gravity, rotate 270, flip vertical, rotate-crop-flip, resize-crop),
  keep their colour type and replay. FT-10 (4 × 6 = 24 rows).
- [ ] **AC-21 Changed: `--invert --resize-max 30`.** Gives `0x5d248c136b8b3ad7`, differs from
  `--resize-max 30 --invert`, saves `[auto-orient, invert, resize]`. FT-2, replacing
  `edit_flag_order_independent`.
- [ ] **AC-22 Byte pin.** The four unchanged invocations give the hashes and recipe text above,
  before and after the change. FT-1. Build runs FT-1 against `main` first (it must pass there) and
  records that.
- [ ] **AC-23 `--help`.** `crustyimg edit --help` contains the order sentence, the four new flags
  and no "canonical order". FT-26.
- [ ] **AC-24 Docs.** Each surface in "Docs" below says what it lists. Verify reads them.
- [ ] **AC-25 No version, cache or lockfile change; no wasm change.** No `Cargo.toml` version,
  `CACHE_SCHEMA_VERSION` or `Cargo.lock` change: `edit` doesn't touch the cache, and a saved recipe
  is still `version = "1"` (driven: `crop` and `rotate` steps load in a `version = "1"` recipe).
  `git grep -n "run_edit\|build_edit_ops\|EditOp\|edit_op_order" src/wasm.rs` finds nothing.
  Verify runs both.
- [ ] **AC-26 Old unit tests rewritten.** `edit_ops_canonical_order`, `edit_ops_subset_order`,
  `edit_ops_requires_at_least_one` and `edit_ops_resize_params_match_resize_command`
  (`src/cli/ops.rs:1875`, `:1885`, `:1894`, `:1911`) call today's `build_edit_ops(bool, Option<u32>, bool)`
  and no longer compile; they become FT-27 (typed order of built steps, P15, resize params equal
  `run_resize`'s). `tests/orientation.rs`' `edit_*` tests stay green unchanged.
- [ ] **AC-27 Mutation table.** Build drives every row below with its quoted edit and records red
  and green per test, and the failure message of each expected-red test.

## Failing Tests

Written at design. Fixtures are written by each test: `noise_rgb8(w, h, seed)` as the byte pin
describes, and RGBA8 / L8 / RGB16 noise from the same xorshift (one `xorshift` value per sample,
truncated to the type). Each replay row runs `edit IN ARGS -o E -y --save-recipe R`, then
`apply --recipe R IN -o A -y`, and asserts: both exit 0; `E` and `A` decode to the same
`ColorType`, the same dimensions, and the same `as_bytes()`; the expected dimensions; and the
saved `[[step]]` `op` list **equals** the expected list in order (not "contains"). FT-3 to FT-31
are new; the decode is `image::open`.

**`tests/edit.rs`**
- **FT-1 `edit_byte_pin_unchanged_invocations`**: the four unchanged rows of the byte-pin table:
  exit 0, `fnv1a64(to_rgb8().as_raw())` equals the literal, saved recipe text equals the literal.
- **FT-2 `edit_invert_then_resize_runs_in_typed_order`** (replaces `edit_flag_order_independent`,
  `tests/edit.rs:219-267`, which used a solid-colour 32×32 fixture where the orders agree): on the
  61×41 noise fixture, `--invert --resize-max 30` hashes to `0x5d248c136b8b3ad7`, its decoded
  bytes differ from `--resize-max 30 --invert`'s, and it saves `[auto-orient, invert, resize]`.

**`tests/edit_geometry.rs`** (input 60×40 RGB noise, seed 11, unless stated)
- **FT-5 `replay_each_flag_alone`**: 9 rows: `--crop 31x7+29+33` 31×7; `--crop 1:1` 40×40;
  `--crop 1:1 --crop-gravity east` 40×40; `--rotate 90` 40×60; `180` 60×40; `270` 40×60; `-90`
  40×60; `--flip horizontal` 60×40; `--flip vertical` 60×40. Steps `[auto-orient, <op>]`.
- **FT-6 `replay_pairs_both_orders`**: 4 pairs × 2 orders (AC-2), with dims (40×30 / 30×40,
  20×40 / 20×40, 30×20 / 30×20, 30×20 / 30×20), steps in order, and `assert_ne!` on the two
  orders' decoded bytes.
- **FT-7 `replay_all_flags_two_orders`**: AC-3.
- **FT-8 `saved_recipe_keeps_crop_gravity`**: `--crop 1:1 --crop-gravity east` saves a crop step
  with `aspect = "1:1"` and `gravity = "east"`; `--crop 1:1` alone saves no `gravity` key.
- **FT-9 `typed_order_ignores_spelling_and_position`**: AC-4's four argv shapes.
- **FT-10 `replay_colour_types`**: AC-20, asserting `color()` equals the input's.
- **FT-11 `pixel_crop_uses_the_received_size`**: AC-13. Find the step **by `op = "crop"`**, not by
  index (driven: an index lookup panics `index not found` under M1, which is the wrong reason).
- **FT-12 `pixel_crop_writes_full_precision`**: AC-14's three rows, plus
  `assert_ne!((1.0/60.0 + 5.0/60.0).to_bits(), (6.0/60.0).to_bits())` so the noisy row is noisy.
- **FT-14 `replay_tagged_orientation_6_and_5`**: 6×4 RGB noise (seed 5) with an `eXIf` chunk
  (big-endian TIFF, one IFD entry, tag `0x0112`, type 3, count 1, value 6 or 5) inserted after
  `IHDR` with its CRC; AC-6's command; 4×3, steps `[auto-orient, rotate, crop]`, replay equal.
  `tests/rotate_flip.rs` already builds a tagged PNG; reuse its helper if it is reachable, else
  copy it.
- **FT-15 `past_the_edge_names_the_image_the_step_receives`**: A1 and A2 as the **whole** stderr
  (an exit-code check alone stays green under M10), exit 1, output and recipe absent.
- **FT-17 `aspect_crop_that_rounds_to_zero_fails`**: A3.
- **FT-18 `crop_pixel_form_refusals`**: P1 × 7, P2 × 2, P3 (first line), each exit 2, whole
  stderr (P3: first line), nothing written.
- **FT-19 `crop_aspect_and_gravity_refusals`**: P4, P5, P6, P7, P8 (both lines).
- **FT-20 `rotate_refusals`**: P11, P12, P13 × 2.
- **FT-21 `flip_refusals`**: P9, P10.
- **FT-22 `repeated_op_flags_are_refused`**: P14 × 3.
- **FT-23 `negative_rotate_and_dash_input`**: `--rotate -90` and `--rotate=-90` give 40×60; an input
  file named `-g.png` passed after `--` with `--rotate -90` exits 0.
- **FT-24 `no_op_flag_message_lists_every_flag`**: P15 verbatim.
- **FT-25 `auto_orient_alone_is_accepted`**: exit 0, saves `[auto-orient]`.
- **FT-26 `edit_help_states_typed_order`**: `crustyimg edit --help` stdout contains `in the order
  you type them`, `--crop <WxH+X+Y|W:H>`, `--crop-gravity <GRAVITY>`, `--rotate <DEGREES>`,
  `--flip <AXIS>`, and not `canonical`.
- **FT-28 `batch_pixel_crop_crops_each_image_in_its_own_pixels`**: directory with 60×40 and 20×50:
  `--crop 15x15+0+0 --out-dir O` → both 15×15, exit 0.
- **FT-29 `batch_pixel_crop_too_large_for_one_image_fails_that_input`**: same directory,
  `--crop 30x30+0+0` → exit 6, the 60×40 one written (30×30), the other absent, stderr names the
  20×50 image.
- **FT-30 `batch_save_recipe_with_pixel_crop_is_refused`**: P16 (with the directory's path as
  typed), exit 2, no out-dir, no recipe; and `--crop 1:1 --save-recipe` on the same directory exits
  0 and saves `[auto-orient, crop]`.

**`src/cli/ops.rs` / `src/cli/mod.rs` unit tests**
- **FT-27 `edit_steps_follow_typed_order`, `edit_steps_require_an_op_flag`,
  `edit_resize_params_match_resize_command`**: AC-26's rewrites on the new `build_edit_ops(&EditFlags)`.
- **FT-31 `edit_op_order_follows_indices`** (in `src/cli/mod.rs` tests): `Cli::command()
  .try_get_matches_from(["crustyimg", "edit", "in", "--flip", "vertical", "--resize-max", "3",
  "--crop=1:1"])` → `[Flip, ResizeMax, Crop]`; `["crustyimg", "edit", "in", "--resize-max", "3"]` →
  `[ResizeMax]` (no `Invert`: the default-index case).
- **FT-32 `parse_crop_flag_cases`**: `"800x600+10+20"` → `Pixels { w: 800, h: 600, x: 10, y: 20 }`,
  `"16:9"` → `Aspect`, and P1/P2's inputs → `Err` (unit level, beside FT-18).
- **FT-33 `crop_fractions_divide_by_the_given_size`**: `crop_fractions(631, 2, 9, 0, 640, 2)` gives
  `x = 9.0/640.0` and `width = 631.0/640.0` exactly, and `(7, 3, 11, 5, 30, 20)` gives `x =
  11.0/30.0`, `y = 5.0/20.0`.

FT-3, FT-4, FT-13 and FT-16 are not used (numbering kept stable against the mutation table).

**Counts, with derivation.** Test functions: 2 in `tests/edit.rs` (FT-1, FT-2) + 23 in
`tests/edit_geometry.rs` (FT-5 to FT-12 is 8; FT-14 and FT-15 is 2; FT-17 to FT-26 is 10; FT-28 to
FT-30 is 3) + 6 unit tests (FT-27's 3, FT-31, FT-32, FT-33) = **31**. Rows inside them: refusal
rows 10 (FT-18: P1 × 7, P2 × 2, P3) + 5 (FT-19) + 4 (FT-20) + 2 (FT-21) + 3 (FT-22) + 1 (FT-24) +
1 (FT-30's P16) = **26**; replay rows 9 (FT-5) + 8 (FT-6) + 2 (FT-7) + 24 (FT-10) + 2 (FT-11) + 3
(FT-12) + 2 (FT-14) + 1 (FT-30's aspect) = **51**; apply-failure rows 2 (FT-15) + 1 (FT-17) + 1
(FT-29) = **4**; pin rows 4 (FT-1) + 1 (FT-2) = **5**.

## Implementation Context

### Decisions that apply

- `DEC-114` — the rule this spec builds. Every part has a test above.
- `DEC-005` — recipes are ordered lists, ops built through one registry; `edit` keeps building
  through `OperationRegistry::with_builtins`, so a saved step loads in `apply`.
- `DEC-106` — values checked before any file is read, and the message names the flag. The one
  pre-run look at the input (P16's `resolve_all`) lists paths and decodes nothing.
- `DEC-112` — fractions in a recipe, each edge rounded on its own. No pixel unit enters a recipe.
- `DEC-113` — geometry acts on the buffer it receives; mirrored tags and `auto-orient` order.
- `DEC-035` — `--save-recipe` refuses a symlink destination (unchanged; keep the guard).
- `DEC-015` — single vs multi input fan-out in `run_pixel_op` (unchanged).

### Constraints that apply

- `untrusted-input-hardening` — `--crop` numbers are parsed digit by digit into `u32` (no sign, no
  overflow panic); the edge check is in `u64`.
- `no-unwrap-on-recoverable-paths` — no `unwrap`/`expect` in `build_edit_ops`, `PixelCrop`,
  `run_edit`; the empty-cell case is an error, not a panic.
- `single-image-library` — fixtures via `image`; no new dependency.
- `every-public-fn-tested`, `test-before-implementation` — FT-31 to FT-33 for the new functions;
  write the tests first and record them RED on `main` (most fail to parse the new flags).
- `one-spec-per-pr`.

### Prior related work

- `SPEC-032` (shipped) — `edit` and `--save-recipe`, the canonical order DEC-114 replaces.
- `SPEC-110` / `SPEC-111` (shipped) — `auto_orient_prefix`; the saved recipe's leading `auto-orient`.
- `SPEC-145` (shipped) — `crop`, DEC-112. `SPEC-147` — `rotate`/`flip`, DEC-113.
- `SPEC-151` / `SPEC-152` (shipped) — `pad`/`trim` steps; the split-off `--pad`/`--trim` entry
  builds on this spec's `EditOp`/`build_flag_op` machinery.

### Overlap with other open work (from the PRs' file lists, `gh pr list --state open --json number,title,files`, 2026-11-10)

- **PR #240** (`chore/wasm-clippy`): `src/cli/ops.rs` hunks at `@@ -25,6` (the `use super::common`
  import block, beside where this spec adds `use std::cell::RefCell;` and `use std::rc::Rc;`) and
  `@@ -799`, `-868`, `-984` (metadata lane, far from `edit`). Also `.github/workflows/ci.yml`,
  `justfile`, `src/metadata/mod.rs` (not touched here).
- **PR #226** (`docs/agents-md-gates`): `src/cli/mod.rs` hunks `@@ -2,12` (module doc) and
  `@@ -888,10` (the `dispatch` doc comment, about 10 lines below `run()`, which this spec changes).
  Also `AGENTS.md`.
- **PR #237** (`heifer-migration`): `CHANGELOG.md` adds `### Changed` at the top of
  `[Unreleased]`; this spec adds a "Changed" line and three "Added" lines there. Put this spec's
  "Changed" line under that heading if #237 has merged; otherwise create `### Changed` after
  `### Added`.
- **PR #229** (`docs/roadmap.md`) — no overlap.
- **No `feat/spec-15[4-8]*` branch is pushed** (`git branch -r`). SPEC-155 to SPEC-158 may change
  `src/cli/common.rs`, `src/cli/ops.rs` and `CHANGELOG.md` when they open.
- **Build: run `git fetch origin && git rebase origin/main` before each push, and after each
  rebase re-run `gh pr list --state open --json number,files` and recheck this list.**

### Docs (each an AC-24 row; verify reads them)

- `src/cli/mod.rs:453-473` (`crustyimg edit --help`): the text under "What `--help` says".
- `docs/cli-reference.md:300-315`: heading becomes `` `edit <INPUT> [--auto-orient] [--resize-max N]
  [--invert] [--crop WxH+X+Y|W:H [--crop-gravity G]] [--rotate DEG] [--flip AXIS] [--save-recipe
  FILE]` ``. Line 306-307's "fixed canonical order" sentence becomes the typed-order rule (op flags
  run in typed order; orientation always first; each once; a pixel `--crop` is in pixels of the
  image at that point, saved as fractions). Lines 309-312 ("The CLI-level orientation bake itself
  is not recorded … not yet fixed") are **stale since SPEC-111** and become "the saved recipe
  starts with `auto-orient` and lists the steps in the order they ran". Add the example
  `crustyimg edit photo.jpg --rotate 90 --crop 1200x800+0+100 -o out.jpg`.
- `docs/api-contract.md:535-560`: the synopsis gains the flags; "ordered operation list (v1:
  `--resize-max N`, `--invert` …)" lists all five; the "fixed canonical order" sentence
  (lines 544-546) becomes the typed-order rule with the three exit-2/exit-1 classes (parse
  refusals, P16, past the edge); add that a pixel `--crop` is saved as fractions of the image the
  step received and that `--save-recipe` with it needs one input image.
- `docs/recipes.md:205-208`: "`edit` records only the ops it exposes — `--auto-orient`,
  `--resize-max`, `--invert`" lists `--crop`, `--rotate`, `--flip` too, and says a pixel `--crop`
  is saved as fractions. Line 209-210 ("a crop to an exact box … is written directly as TOML")
  drops "a crop to an exact box".
- `recipes/README.md:52` (**missed by the frame**): same change as `docs/recipes.md:208`, and drop
  "a crop to an exact box" from line 53.
- `skills/crustyimg/SKILL.md`: **no change**. It has no `edit` section (`grep -n -i "edit\b"` finds
  only lines 145 and 156, both "edit metadata" prose) and lists no verb's flags for `edit`.
- `CHANGELOG.md` `[Unreleased]`: **Added** — "`edit` can crop, rotate and flip: `--crop WxH+X+Y`
  (pixels of the image at that point) or `--crop W:H` with `--crop-gravity`, `--rotate DEGREES`,
  `--flip horizontal|vertical`; `--save-recipe` saves them as recipe steps, a pixel crop as
  fractions." **Changed** — "`edit` runs its op flags in the order you type them. `edit --invert
  --resize-max N` now inverts, then resizes (0.8.2 resized first whatever the order), so its output
  changes; `--resize-max N --invert` and every other invocation of 0.8.2's flags are unchanged."
- Unchanged, re-checked: `docs/USAGE.md:137`, `docs/data-model.md:187`, `README.md:262`,
  `recipes/README.md:45`, `docs/recipes.md:206` (examples with `--auto-orient --resize-max` or
  `--resize-max` alone, same bytes and recipe under typed order). `tests/orientation.rs:789`'s
  assertion message says "canonical order" but asserts recipe text that doesn't change; leave it.
  `docs/lab-plan-2026-08.md`, `docs/sessions/`, `docs/backlog.md`, `CHANGELOG.md:797-799` (0.x
  history): historical, left.
- Outside the repo: the jysf.org project page may list `edit`'s flags; not a build AC, noted for
  ship.

### Mutation table

Driven on the prototype (`cargo test --release --test edit_geometry --test edit --no-fail-fast`),
one mutant at a time from a clean copy. The prototype's test names map to FTs as follows:
`byte_pin_today_flags` FT-1, `invert_then_resize_runs_in_typed_order` FT-2,
`replay_each_flag_alone` FT-5, `replay_pairs_both_orders` FT-6, `replay_all_flags_two_orders`
FT-7, `typed_order_ignores_spelling_and_position` FT-9, `replay_colour_types` FT-10,
`pixel_crop_uses_the_received_size` FT-11, `pixel_crop_writes_full_precision` FT-12,
`replay_tagged_inputs` FT-14, `past_the_edge_names_the_received_image` FT-15, `refusals` (one
table: FT-18 to FT-21 and FT-24), `batch_pixel_crop_and_save_recipe` (FT-28 + FT-30),
`auto_orient_alone_is_accepted` FT-25. FT-8, FT-17, FT-23, FT-26, FT-29, FT-27, FT-31 to FT-33 were
not in the prototype; their column is *reasoned*. Baseline: all 14 prototype tests and the 9
`tests/edit.rs` tests green. "Green" lists the driven tests that stayed green; every FT not named
red is expected green.

- **M1 — a fixed table instead of the typed index (Alternative A's order).** `src/cli/mod.rs`,
  `edit_op_order`: replace `typed.sort_by_key(|(i, _)| *i);` with
  `typed.sort_by_key(|(_, op)| match op { EditOp::Rotate => 0, EditOp::Flip => 1, EditOp::Crop => 2, EditOp::ResizeMax => 3, EditOp::Invert => 4 });`.
  Red (driven): FT-2 (hash `7120281550223046311`), FT-6, FT-7, FT-11 (with the index lookup it
  panicked `index not found`; FT-11 must find the step by `op`), FT-15 (exit 0: the crop ran
  before the resize). Green (driven): FT-1, FT-5, FT-9 (rotate-then-crop is Alternative A's order
  too), FT-10, FT-12, FT-14, FT-25, `refusals`, batch, all 9 `tests/edit.rs`. Reasoned red: FT-31
  (its first case is not in Alternative A's order).
- **M7 — ties and spelling: declaration order instead of the index.** Same location: delete
  `typed.sort_by_key(|(i, _)| *i);`. Red (driven): FT-2, FT-6, FT-7, FT-9 (saved
  `["auto-orient", "crop", "rotate"]`), FT-10, FT-11, FT-14 (`--crop 4x3+1+0 runs past the right
  edge of the 4×6 px image`), FT-15. Green (driven): FT-1 (its two-flag row is typed in
  declaration order), FT-5, FT-12, FT-25, `refusals`, batch, `tests/edit.rs`. Reasoned red: FT-31.
- **M2 — drop the `value_source` filter.** `src/cli/mod.rs`, `edit_op_order`: delete the line
  `.filter(|(id, _)| m.value_source(id) == Some(clap::parser::ValueSource::CommandLine))`. **Not
  equivalent.** Red (driven): FT-1 **by hash** (`pixel pin ["--resize-max", "30"]`: got
  `7120281550223046311` = `0x62d0525855cff6a7`, the resize-then-invert pin, because the defaulted
  `--invert` switch joins the order), FT-5 (`["auto-orient", "crop", "invert"]`), FT-6, FT-9 (`["auto-orient", "rotate",
  "crop", "invert"]`), FT-14, FT-24 (P15 row exits 0), FT-25, and `tests/edit.rs`'
  `edit_no_ops_exits_2` (exit 0). Green (driven): FT-2, FT-7, FT-10, FT-11, FT-12, FT-15 (every
  row types `--invert` or fails first), batch. Reasoned red: FT-31's `[ResizeMax]` case.
- **M3 — convert against the decoded size, not the image the step receives.** `src/cli/ops.rs`,
  `PixelCrop::apply`: replace `let (iw, ih) = (img.pixels().width(), img.pixels().height());` with
  `let (iw, ih) = ::image::image_dimensions(std::env::args().nth(2).unwrap_or_default()).unwrap_or((1, 1));`
  (an emulation: a step can't see the decoded size, so this reads the input file's header the way
  a converter run before the pipeline would; it takes `argv[2]` as the input). Red (driven): FT-6,
  FT-7, FT-10 (`runs past the bottom edge of the 60×40 px image`), FT-11, FT-15, batch (FT-28:
  argv[2] is the directory, so the size read fails). FT-9 also red, **an artifact of the
  emulation** (argv[2] is not the input when the flags come first). Green (driven): FT-1, FT-2,
  FT-5, FT-12 (no resize or turn before the crop), FT-14, FT-25, `refusals`, `tests/edit.rs`.
- **M4 — write fractions with `{:.6}`.** `src/cli/ops.rs`, `run_edit`'s recipe loop: replace
  `StepRecord::Resolved(cell) => cell.borrow().clone().ok_or_else(|| {` with
  `StepRecord::Resolved(cell) => cell.borrow().clone().map(|p| OperationParams::from_map(p.iter().map(|(k, v)| (k.to_owned(), toml::Value::Float(format!("{:.6}", v.as_float().unwrap()).parse().unwrap()))).collect())).ok_or_else(|| {`.
  Red (driven): FT-11 (`0.366667` vs `11/30`), FT-12 (`apply` refuses: `recipe step 1 ('crop'):
  'x' + 'width' is 0.014063 + 0.985938, past the right edge; together they must be at most 1`).
  Green (driven): every other test, including every replay row (no other rectangle hits the
  6-decimal trap, correction 3).
- **M5 — capture the recipe before the run.** `src/cli/ops.rs`, `run_edit`: move the
  `if let Some(path) = save_recipe { … }` block (recipe build and `to_toml`) above
  `run_pixel_op`, writing the file after. It **compiles**; it's a logic error, caught by the
  empty-cell guard. Red (driven): every test with a pixel `--crop` and `--save-recipe`: FT-5, FT-6,
  FT-7, FT-9, FT-10, FT-11, FT-12, FT-14, FT-15 (stderr `operation 'crop' failed: internal: the
  crop step never ran`). Green (driven): FT-1, FT-2, FT-25, `refusals`, batch, `tests/edit.rs`.
  If the build makes the empty cell a panic instead, the same tests red with a panic.
- **M6 — let `--auto-orient` take its typed position.** Four edits: `src/cli/mod.rs` add
  `AutoOrient,` as `EditOp`'s first variant and `("auto_orient", EditOp::AutoOrient),` to
  `edit_op_order`'s list (`[(&str, EditOp); 6]`); `src/cli/ops.rs` `build_edit_ops` add
  `EditOp::AutoOrient => { steps.push(built(build_flag_op("auto-orient", &OperationParams::empty(), &[])?)); }`;
  `run_edit` wrap the leading `recipe.steps.push(RecipeStep { op: "auto-orient" … })` in
  `if !flags.auto_orient { … }` (today's rule). Red (driven): FT-14 only, on its Orientation-5
  row (`dims … left: (4, 3)`: the replay of `[rotate, crop, auto-orient]` differs); its
  Orientation-6 row alone would stay green (rotations commute). Green (driven): everything else,
  including FT-1 row 4 (`--auto-orient` typed first saves the same text).
- **M8 — the converter divides in `f32`.** `src/cli/ops.rs`, `crop_fractions`: replace
  `toml::Value::Float(f64::from(x) / fw)` with `toml::Value::Float(f64::from(x as f32 / iw as f32))`
  (the `x` line). Red (driven): FT-11 (`0.36666667461395264`), FT-12 (edit refuses `--crop
  47x40+13+0`: `'x' + 'width' is 0.21666666865348816 + 0.7833333333333333`). Green (driven): all
  else. Reasoned red: FT-33.
- **M9 — `x` divided by the wrong axis.** Same line: `f64::from(x) / fh`. Red (driven): FT-5,
  FT-10, FT-11, FT-12. Green (driven): all else. Reasoned red: FT-33.
- **M10 — the past-the-edge check removed.** `src/cli/ops.rs`, `PixelCrop::apply`: both
  `if u64::from(self.x) + u64::from(self.w) > u64::from(iw) {` and the `y`/`h` line become
  `if false {`. What happens instead: `Crop::from_params` refuses the fraction, still exit 1, with
  the op's text: `operation 'crop' failed: invalid params for operation 'crop': 'height' must be a
  number > 0 and <= 1, got 1.05`. Red (driven): FT-15 only, on the message. Green (driven): all
  else. Reasoned red: FT-29's stderr check. An exit-code-only assertion stays green.
- **M11 — `--crop-gravity` dropped from the saved recipe.** `src/cli/ops.rs`, the aspect branch:
  build the op from `params` but record `StepRecord::Built` of a map holding only `aspect`. Red
  (driven): FT-5 (`pixels differ between edit and replay ["--crop", "1:1", "--crop-gravity",
  "east"]`), FT-10 (southwest rows). Green (driven): all else. Reasoned red: FT-8.
- **M13 — the flag name not substituted.** `src/cli/ops.rs`, `build_flag_op`: replace
  `reason = reason.replacen(&format!("'{key}'"), flag, 1);` with `let _ = (key, flag);`. Red
  (driven): `refusals` (`error: 'aspect' must be two whole numbers …` for `--crop 0:1`), i.e.
  FT-19, FT-20, FT-21 (P4, P5, P6, P9, P11, P12). Green (driven): all else.
- **M14 — the batch refusal removed.** `src/cli/ops.rs`, `run_edit`: `if count > 1 {` →
  `if count > 1 && false {`. Red (driven): FT-30 (exit 0, not 2). Green (driven): all else.
- **M15 — `--auto-orient` no longer counts as an op flag.** `src/cli/ops.rs`, `build_edit_ops`:
  `if flags.order.is_empty() && !flags.auto_orient {` → `if flags.order.is_empty() {`. Red
  (driven): FT-25. Green (driven): all else. (`tests/orientation.rs`' `edit_auto_orient_flag_still_exits_zero`
  also reds; reasoned, it ran outside the mutation loop.)
- **Trap 14 — `main`'s `src/cli/mod.rs` and `src/cli/ops.rs` restored verbatim (tests kept).** Red
  (driven): FT-2 (hash), FT-5, FT-6, FT-7, FT-9, FT-10, FT-11, FT-12, FT-14, FT-15, `refusals`
  (`error: unexpected argument '--crop' found`), batch — 12 tests. Green (driven): FT-1 (the pin
  holds on `main`, as it must), FT-25 (`--auto-orient` alone is accepted on `main`), and all 9
  `tests/edit.rs` tests, including `edit_flag_order_independent`, which FT-2 replaces. The
  `src/cli/` unit tests (FT-27, FT-31 to FT-33) don't compile against `main`'s signatures.
- **Golden by hash (trap 13).** FT-1 and FT-2 are the goldens, and each has a driven mutation that
  turns it red on a hash mismatch, not on a refusal or an exit code: M2 reds FT-1 (`--resize-max
  30` hashed `0x62d0525855cff6a7`), and M1 and M7 red FT-2 (`--invert --resize-max 30` hashed
  `0x62d0525855cff6a7`, the old order).

## Notes for the Implementer

- **Write FT-1 first and run it on `main`.** It must pass there, unchanged, before any source edit,
  and again after. Record both runs in Build Completion. FT-2's literal is the "after" value; on
  `main` FT-2 fails on the hash (`0x62d0525855cff6a7`), which is the expected RED and its reason.
- **RED baseline reasons.** On `main`, every FT in `edit_geometry.rs` fails because clap refuses the
  new flags (`error: unexpected argument '--crop' found`, exit 2) — that is a parse failure, not the
  test's reason. Quote it once, and for FT-24 (P15) quote the old three-flag message.
- **Checkpoints** (AGENTS long-build rule): commit and push after (1) FT-1/FT-2 and the typed-order
  machinery with `--resize-max`/`--invert` only, (2) `--rotate`/`--flip`, (3) `--crop` and
  `PixelCrop`, (4) batch + docs. Each push rebases first.
- **CI legs that matter:** the full native matrix (Linux/macOS/Windows), the lean
  `--no-default-features` leg (edit is in the binary there too), and wasm (no change expected;
  AC-25's grep). `cargo test` in a terminal fails `display_sink_refuses_non_tty`; redirect stdout.
- **`×` is U+00D7** in A1/A2 and in P1's third input; `x` (U+0078) is the only separator `--crop`
  takes.
- Comments and help text read behaviour-first, with no spec or DEC ids.
- Don't make `rect_edges` or `parse_aspect` public: the design needs neither (the probe widened
  `rect_edges` only for itself).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-153-edit-geometry-flags`
- **PR (if applicable):** #261 (draft)
- **All acceptance criteria met?** yes (AC-1 to AC-27; detail below)
- **New decisions emitted:** none (DEC-114 was written at design and is on `main`).
- **Files this diff touches** (`git diff --name-only origin/main...HEAD`; 10 files including this spec):
  - `src/cli/mod.rs` — `Edit` gains `--crop`, `--crop-gravity`, `--rotate`, `--flip` and the new help text; `EditOp`, `edit_op_order`; `run()` keeps the `ArgMatches`; dispatch passes `EditFlags`; FT-31
  - `src/cli/ops.rs` — `EditFlags`, `EditStep`/`StepRecord`, `parse_crop_flag`, `crop_fractions`, `PixelCrop`, `build_flag_op`, new `build_edit_ops`, `run_edit` (batch refusal, recipe built after the run); FT-27, FT-32, FT-33
  - `tests/edit.rs` — FT-1 byte pin, FT-2 (replaces `edit_flag_order_independent`)
  - `tests/edit_geometry.rs` — new; 23 tests (FT-5 to FT-12, 14, 15, 17 to 26, 28 to 30)
  - `CHANGELOG.md` — three Added lines (crop, rotate, flip), one Changed line
  - `docs/cli-reference.md`, `docs/api-contract.md`, `docs/recipes.md`, `recipes/README.md` — flags, typed-order rule, pixel crop saved as fractions; stale "canonical order" and "orientation bake not recorded" text replaced
  - this spec file — Build Completion, cost
- **Deviations from spec:**
  - Stages 2, 3 and 4 were committed as one checkpoint (one push), not three. Stage 1 (red tests) and the docs were pushed separately.
  - Mutation runs used the full suite (`cargo test --no-fail-fast`) with each mutant in a scratch worktree and its own target dir. T14 used only the three integration targets (`edit`, `edit_geometry`, `orientation`) (corrected at punch list: the `src/cli/` unit tests do compile and pass against `main`'s two files).
  - FT-6 uses `--crop 30x40+0+0` with `--rotate 90` for the crop x rotate pair, to land on the spec's 40x30 / 30x40 dimensions.
  - FT-14 uses `tests/common`'s little-endian `tiff_with_orientation` (the spec said big-endian; the helper was reachable, so I reused it).
- **Follow-up work identified:**
  - `--pad` / `--trim` flags (the split-off entry, closes STAGE-058).

**Baseline (before any source edit).** FT-1 on `main`'s code: `edit_byte_pin_unchanged_invocations ... ok` (9 passed, 1 failed in `tests/edit.rs`). FT-2 on `main`: `assertion left == right failed: got 0x62d0525855cff6a7` (the old order). `tests/edit_geometry.rs` on `main`: 22 of 23 failed; `auto_orient_alone_is_accepted` passed. Quoted clap refusal: `error: unexpected argument '--crop' found`. FT-24 on `main`: `error: edit requires at least one operation flag (--auto-orient, --resize-max, --invert)`.
**After:** FT-1 ok, FT-2 ok, 23/23 `edit_geometry`, 10/10 `tests/edit.rs`, 12/12 `tests/orientation.rs`, 6 unit tests ok.

**AC-1 to AC-27.** All met. AC-1: DEC-114 exists, `just decisions-audit` exits 0 (0 structural errors, 2067 scope warnings). AC-2 FT-6, AC-3 FT-7, AC-4 FT-9 + FT-31, AC-5 FT-1/FT-24/FT-31, AC-6 FT-1 row 4, FT-14 (two rows), FT-25, AC-7 FT-22, AC-8 FT-18 + FT-32, AC-9 FT-5/8/19, AC-10 FT-5/20/23, AC-11 FT-5/21, AC-12 FT-24 and `edit_no_ops_exits_2`, AC-13 FT-11 + FT-33, AC-14 FT-12, AC-15 FT-15, AC-16 FT-17, AC-17 FT-15/17 and `edit_save_recipe_unwritable_exits_5`, AC-18 FT-28 to 30, AC-19 FT-5, AC-20 FT-10, AC-21 FT-2, AC-22 FT-1, AC-23 FT-26, AC-24 docs edited, AC-25 `git grep` of `src/wasm.rs` finds nothing, `git diff --stat origin/main...HEAD -- Cargo.toml Cargo.lock` is empty, no `CACHE_SCHEMA_VERSION` change, AC-26 FT-27, AC-27 table below.

**Mutations** (full suite each, red lists are the failing tests; spec cell in brackets where it differs).
- M1: red FT-2, FT-6, FT-7, FT-11, FT-15, FT-31, plus FT-9 and FT-10. [Spec: FT-9 and FT-10 green. FT-9 has a fifth row, `--crop … --rotate 90` typed crop first, which Alternative A's order reverses, and FT-10's rotate-crop-flip and resize-crop rows put crop before the geometry op. FT-31 reasoned red: confirmed.]
- M7: red FT-2, 6, 7, 9, 10, 11, 14, 15, 31. Matches.
- M2: red FT-1 (by hash), 5, 6, 9, 14, 24, 25, `edit_no_ops_exits_2`, FT-31, plus FT-10, 11, 12, 30 and `tests/cli.rs` `stub_command_returns_not_implemented`. (corrected at punch list) Verify's run: 15 red, FT-1 by hash, FT-5, 6, 8, 9, 10, 11, 12, 14, 24, 25, 30, FT-31, `edit_no_ops_exits_2`, `stub_command_returns_not_implemented`; FT-8 is red too. `stub_command_returns_not_implemented` runs `edit in.png` with no op flag, so it is an `edit` test; the order is collected only for `edit`. [Spec: FT-10, 11, 12 green because every row types `--invert` or fails first; my rows for those tests don't type `--invert`, so the defaulted switch joins the order and adds a step. FT-30 and the stub test are extra reds for the same reason.]
- M3: red FT-6, 7, 9, 10, 11, 15, 28, 29. Matches (FT-9 an emulation artifact); FT-29 also red because `argv[2]` is the directory.
- M4: red FT-11, FT-12 only. Matches.
- M5: red FT-5, 6, 7, 9, 10, 11, 12, 14, 15. Matches.
- M6: red FT-14 only. Matches.
- M8: red FT-11, FT-12, FT-33. Matches.
- M9: red FT-5, 10, 11, 12, FT-33. Matches.
- M10: red FT-15 (message) and FT-29 (message; reasoned, confirmed). Matches.
- M11: red FT-5, 8, 10. Matches.
- M13: red FT-19, 20, 21. Matches.
- M14: red FT-30 only. Matches.
- M15: red FT-25, `unit edit_steps_require_an_op_flag`, `tests/orientation.rs` `edit_auto_orient_flag_still_exits_zero` (reasoned, confirmed) and `already_correct_verbs_do_not_double_rotate` (not in the spec). Matches plus the unit test and the extra orientation test.
- Trap 14 (corrected at punch list: the earlier "the unit tests don't compile" is wrong. With `main`'s two files restored verbatim, the `src/cli/` unit tests are `main`'s own, and they compile and pass): `main`'s `src/cli/mod.rs` and `ops.rs` restored: FT-2 red (hash), 22 of 23 `edit_geometry` tests red, FT-1 ok, FT-25 ok, the other 9 `tests/edit.rs` tests ok, 12 `tests/orientation.rs` ok.
- Reasoned cells confirmed: FT-8 (M11), FT-17 and FT-23 (green under every mutation, red only under trap 14), FT-26 (green in all but trap 14), FT-27 (M15's unit test), FT-29 (M3, M10), FT-31 (M1, M2, M7), FT-33 (M8, M9).

### Punch list (verify)

Wording only; no code or test changed.
- `docs/recipes.md`: the saved-recipe paragraph now says the recipe always starts with `auto-orient`, then the other flags in typed order, and that a pixel `--crop` replays as the same proportions, not the same pixels, and fails if a side rounds to 0 px.
- `recipes/README.md`: same order and `auto-orient` point; the pixel crop is "saved as fractions of the image at that step".
- This section: M2's red set gains FT-8, and trap 14's "unit tests don't compile" is corrected, each marked `(corrected at punch list)`.

Drives (release build of the PR head, fresh `CARGO_TARGET_DIR`):
- `edit a.png --crop 1x1+0+0 --save-recipe r.toml` on a 200×10 PNG saved `width = 0.005`, `height = 0.1`. `apply --recipe r.toml --out-dir o2 b.png` on a 60×40 PNG exited 1: `error: operation 'crop' failed: the crop rounds to 0×4 px on this 60×40 px image; it must keep at least 1 px each way`.
- `edit b.png --rotate 90 --auto-orient --save-recipe r2.toml` saved `[[step]] op = "auto-orient"` first, then `[[step]] op = "rotate"`, `angle = 90`.
- `cargo test --test docs_ops`: exit 0, 3 passed.
- CI snapshot: recorded in the PR report after the push.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Little. The prototype-driven messages and tables made the code mechanical. The FT-2 and FT-10/FT-11/FT-12 'green under M2' cells depend on whether rows type `--invert`, which the spec didn't say.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The clippy `type_complexity` and `ok_expect` lints hit new test code; nothing to add to the spec.

3. **If you did this task again, what would you do differently?**
   — Commit stages 2 to 4 separately (I merged them), and quote zsh arrays in the mutation runner from the start (T14 ran once with a one-string argument list). No design call turned out wrong.

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
