---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-153
  type: story                      # epic | story | task | bug | chore
  cycle: frame  # frame | design | build | verify | ship
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

*(Filled at design; the frame's out-of-scope list is under `## Frame`.)*

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
