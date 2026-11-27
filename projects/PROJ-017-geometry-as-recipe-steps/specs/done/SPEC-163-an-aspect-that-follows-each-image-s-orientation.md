---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-163
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-017
  stage: STAGE-060
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: claude-opus-5-5     # usually same Claude, different session
  created_at: 2026-11-25

references:
  decisions:
    - DEC-102
    - DEC-110
    - DEC-112
    - DEC-113
    - DEC-114
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - clippy-fmt-clean
    - every-public-fn-tested
    - test-before-implementation
  related_specs:
    - SPEC-145
    - SPEC-151
    - SPEC-153
    - SPEC-160
    - SPEC-161

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-060's <capability>". Optional; null is acceptable.
value_link: "Lets one `aspect` recipe from STAGE-060's edit page (SPEC-161's presets) crop a folder of mixed landscape and portrait photos"

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
      date: 2026-11-25
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 5017163
      tokens_breakdown:
        input: 92
        cache_creation: 125273
        cache_read: 4857788
        output: 34010
      estimated_usd: 4.06
      note: >
        Measured from the session JSONL (found by the prompt path `SPEC-163-frame.md`), deduped by
        message.id (46 messages; max output_tokens per id), 0 subagent transcripts. Measured after the push,
        before this cost commit's own turns.
        Rates (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read
        (x0.10), $25 output.
    - cycle: design
      date: 2026-11-25
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 17880650
      tokens_breakdown:
        input: 186
        cache_creation: 253682
        cache_read: 17523055
        output: 103727
      estimated_usd: 12.94
      note: >
        Re-measured at ship (was 17,032,751 tokens, $12.37, taken before the blocked push). Measured from the session JSONL (found by its first user message naming `SPEC-163-design.md`), deduped by message.id (92 messages; max output_tokens per id), 0 subagent transcripts. Includes 17 driven mutants on a prototype.
        Rates (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read
        (x0.10), $25 output.
    - cycle: build
      date: 2026-11-26
      model: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 14654504
      tokens_breakdown:
        input: 180
        cache_creation: 214897
        cache_read: 14370569
        output: 68858
      estimated_usd: 6.15
      note: >
        Re-measured at ship (was 12,994,222 tokens, $5.50, taken before the last commit and the CI watch). Measured from the session JSONL (found by its first user message naming `SPEC-163-build.md`), deduped by message.id (85 messages; max output_tokens per id), 0 subagent transcripts.
        Rates (Sonnet): $3/MTok input, $3.75 cache_creation (x1.25), $0.3 cache_read (x0.10), $15 output.
    - cycle: verify
      date: 2026-11-26
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 9693572
      tokens_breakdown:
        input: 126
        cache_creation: 183771
        cache_read: 9457002
        output: 52673
      estimated_usd: 7.19
      note: >
        Measured at ship from the session JSONL (found by its first user message naming `SPEC-163-verify.md`), deduped by message.id (61 messages; max output_tokens per id), 0 subagent transcripts. The verify report said 9,269,764 tokens, $6.83, taken before its last turns.
        Rates (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read
        (x0.10), $25 output.
    - cycle: ship
      date: 2026-11-26
      model: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 3915965
      tokens_breakdown:
        input: 80
        cache_creation: 97042
        cache_read: 3798753
        output: 20090
      estimated_usd: 1.81
      note: >
        Measured after the ship push from the session JSONL (found by its first user message naming `SPEC-163-ship.md`), deduped by message.id (40 messages; max output_tokens per id), 0 subagent transcripts. Excludes the follow-up cost commit's own turns.
        Rates (Sonnet): $3/MTok input, $3.75 cache_creation (x1.25), $0.3 cache_read (x0.10), $15 output.
  totals:
    tokens_total: 51161854
    estimated_usd: 32.15
    session_count: 5
---

# SPEC-163: An aspect that follows each image's orientation

## Context

`aspect = "W:H"` on `crop` (SPEC-145) and `pad` (SPEC-151) always means W wide by H tall. A recipe
for "5×7 prints of this folder" wants 7:5 on landscape photos and 5:7 on portrait ones, which today
takes two recipes and a hand sort. STAGE-060's Spec Backlog entry "`crop`'s aspect can follow each
image's orientation" (filed 2026-11-05, moved from STAGE-058 on 2026-11-11) asks for one optional
key. The maintainer ruled on 2026-11-22 (SPEC-161's "Maintainer rulings", items 3 and 4) that the
ddphotos edit page's crop presets emit `aspect`, not a box, so a recipe copied from the page reads
`aspect = "3:2"` and works on any photo. Across a folder of mixed photos that only holds if the
aspect can follow each photo's orientation, so the page is this key's first user and this spec
builds before SPEC-161's design. SPEC-161 touches no Rust; this spec is the Rust half.

## Goal

Add one optional step key to `crop` and `pad` that puts an aspect's long side along the long side
of the buffer the step receives, so one `aspect = "3:2"` recipe gives 3:2 on landscape images and
2:3 on portrait ones. Leaving the key off keeps SPEC-145's and SPEC-151's meaning exactly.

## Frame

**Verdict: GO, [M], one spec (no split).** Framed 2026-11-25 (Opus) on `main` at `cd519f9`, release
build of 0.8.2+main in a scratch `CARGO_TARGET_DIR`. Probe recipes and images lived in the
scratchpad and are not committed.

### 1. Today's behaviour, driven

`crustyimg apply` with `aspect = "3:2"` (crop; pad with `color = "#ffffff"`), inputs made with
ImageMagick:
- landscape 1200×800: crop **1200×800**, pad **1200×800** (both already 3:2, unchanged)
- portrait 800×1200: crop **800×533**, pad **1800×1200**
- square 1000×1000: crop **1000×667**, pad **1500×1000**
- near-square 1001×1000: crop **1001×667**, pad **1500×1000**
- near-square 1000×1001: crop **1000×667**, pad **1502×1001**

EXIF-rotated JPEG (stored 1200×800, `exiftool -Orientation=6 -n`, displayed 800×1200):
- no `auto-orient`: crop **1200×800**, pad **1200×800**. The step sees the **stored** landscape
  buffer (DEC-113 rule 1). The output carries no Orientation tag (`exiftool -Orientation` on a JPEG
  output is empty, DEC-113 rule 2), so it displays landscape.
- `auto-orient` first: crop **800×533**, pad **1800×1200**. The step sees the oriented 800×1200
  portrait buffer.

So the premise holds: a portrait photo gets a landscape 3:2 cut out of it (800×533) or a landscape
canvas around it (1800×1200).

### 2. The key's name and values

Weighed:
- **Boolean** `follow_orientation = true`. Reads well, but `false` is a second spelling of the
  default, and it has no room for a later value.
- **Enum, one value now**: `orientation = "match"`. **Recommended.** A `ParamKind::OneOf(&["match"])`
  row in each op's table, so the existing table check refuses an unknown value or a wrong type with
  the list of accepted values, and a later value is additive.
- **Enum with forcing** (`"landscape"`, `"portrait"`). Declined: forcing is already written by the
  order of the ratio (`"3:2"` is landscape, `"2:3"` portrait), so it would be a second spelling of
  existing recipes, and `"landscape"` on `1:1` means nothing.

**The default has no spelling.** Off is the key's absence. There is no `"as-written"` value, so a
recipe has exactly one way to say "off", and `params()` round-trips the step as written (SPEC-145's
`params_round_trip_as_written`), so a loaded-then-saved recipe keeps the key iff it was there.
`edit --save-recipe` writes `orientation = "match"` only when its flag was typed (see 6) and never
writes a default.

⚠ The name sits next to EXIF Orientation and `auto-orient`. Docs must say the key reads the
**shape of the buffer the step receives** (DEC-113), so a camera JPEG needs `auto-orient` first, as
every geometry op's doc entry already says. Maintainer question 1.

### 3. Squares

**The rule:** with `orientation = "match"`, a landscape buffer (`width > height`) gets the aspect
with its larger side as W, a portrait buffer (`width < height`) with its larger side as H, and a
**square buffer gets the aspect as written**. It is a normalization, not a swap: `"2:3"` with the
key on a landscape image becomes 3:2.

- **3:2 on a square, key on: 3:2 (as written). Recommended.** Driven: crop **1000×667**, pad
  **1500×1000**, identical to key off. A square has no orientation to follow, so the writer's order
  is the only intent on record, and it is what SPEC-161's frame already chose for the page ("Square
  photos and 1:1 stay as they are"). Refusing would fail a whole folder batch on one square photo.
  Picking 2:3 would be an arbitrary tie-break that contradicts the written text.
- **`1:1` with the key on: accepted, a no-op. Recommended.** Driven: `1:1` gives 800×800 / 1200×1200
  on both 1200×800 and 800×1200, so the key cannot change it. Refusing would break the page: per
  SPEC-161 ruling 3 an untouched preset emits `aspect` with the key, and `1:1` is one of its four
  presets, so the page would have to special-case it.

Maintainer question 2.

### 4. Near-square

**Strictly wider is landscape, strictly taller is portrait, equal is square.** No tolerance: any
threshold is a number nobody can predict, and the page's port would have to carry it. One pixel
flips the result, and it should be documented with a driven example. Driven (key on = the swapped
aspect written by hand, since the key does not exist yet):
- 1001×1000 (landscape by 1 px), `3:2` on: crop **1001×667**, pad **1500×1000** (same as off).
- 1000×1001 (portrait by 1 px), `3:2` on → 2:3: crop **667×1001**, pad **1000×1500**. Off: crop
  **1000×667**, pad **1502×1001**.

So a one-pixel difference turns a 1000×667 cut into a 667×1001 one. That surprises only someone who
thinks of the image as square; anyone who thinks of it as portrait gets what they asked for. Crop
from a near-square photo is rare in practice (cameras and phones shoot 3:2, 4:3, 16:9).

### 5. Refusals

Recommended: all four refused **at load, exit 1**, through the existing DEC-102 shape
`recipe step N ('op'): …`. Existing load errors, driven on the 0.8.2+main binary:
- unknown key today: `error: recipe step 0 ('crop'): unknown key 'orientation'; expected one of: x, y, width, height, aspect, gravity` (exit 1) and the same for `pad` with `aspect, gravity, border, color`.
  So **a recipe the page emits after this ships is refused loudly by an older binary**, not
  silently run without the key. Crop and pad are both unreleased, so no shipped binary reads `crop`
  at all ("0.8.2 and earlier refuse it with `unknown operation 'crop'`", CHANGELOG).
- `gravity` with the box form: `error: recipe step 0 ('crop'): 'gravity' is only used with 'aspect'; a rectangle crop is placed by x and y`
- `gravity` with pad's border: `error: recipe step 0 ('pad'): 'gravity' is only used with 'aspect'; a border pad adds the same width on every side`
- unknown value: `error: recipe step 0 ('crop'): 'gravity' must be one of: center, north, …, southwest; got "up"`
- wrong type: `… got the integer 5` and `… got true`; case: `… got "North"` (case-sensitive)

For the new key:
- **with `crop`'s box: refuse**, mirroring `gravity` ("'orientation' is only used with 'aspect'; a
  rectangle crop …"). A box is already in the image's own fractions; ignoring the key would let a
  recipe say something it doesn't do.
- **with `pad`'s `border`: refuse**, the same way (a border has no aspect to turn).
- **unknown value** (`"landscape"`, `"Match"`): refuse via the table's `OneOf` message.
- **wrong type** (`true`, `1`): refuse via the same message. `true` is the likely typo from the
  boolean spelling, so a "must not refuse"-adjacent row: `orientation = "match"` with `gravity`
  builds, and with `1:1` builds.

Exact wording is design's; the order the checks run in (table first, then form rules) is the
existing one, so a wrong-typed key on a box crop reports the type error first. Design pins that.

### 6. `edit`'s flags

`edit` batches a folder today (driven: `edit <dir> --crop 3:2 --out-dir … --save-recipe s.toml`
→ L.png 1200×800, P.png **800×533**, exit 0, and `s.toml` holds `auto-orient` then
`op = "crop"`, `aspect = "3:2"`). That is the motivating "this folder" case on the CLI.

**Recommended: add `--crop-orientation match` and `--pad-orientation match`**, built exactly like
`--crop-gravity` / `--pad-gravity` (`src/cli/ops.rs` ~1527–1615): added to the aspect form's
params, refused as a usage error (exit 2) with a pixel `--crop WxH+X+Y` or a border `--pad N`, as
the gravity flags are, and saved by `--save-recipe` as `orientation = "match"` only when typed.
`edit --help` gains two lines. One vocabulary with the recipe (`match`), not a bare boolean flag,
so a later value needs no new flag. Recipe-only would keep the spec at [S]; maintainer question 3.

### 7. wasm and the page

- **wasm needs no change.** `src/wasm.rs::transform` parses with `Recipe::from_toml` and builds with
  `pixel_recipe.build_pipeline(&OperationRegistry::with_builtins())` (lines 209, 234, 243), the same
  loader and registry `apply` uses, so the key, its refusals and its rule reach wasm by construction.
  Build still runs `just wasm-check` and `just wasm-test` (AGENTS §15: crop/pad compile for wasm).
- **What SPEC-161's design ports into `recipe.js`:** the orientation test on the size of the
  **oriented** base (SPEC-161's fix, not `info()`'s stored size): landscape iff `w > h`, portrait iff
  `w < h`, square as written; then the normalization of `(a, b)` to `(max, min)` or `(min, max)`;
  then the existing port of `aspect_size` (crop) and `aspect_canvas` (pad, if the page ever pads,
  which STAGE-060 rules out). Integer comparisons only, so no `f64` question.
- **The rounding is shared.** The key only reorders `(a, b)` before the call; `crop`'s
  `aspect_size` and `pad`'s `aspect_canvas` run unchanged on both branches, so DEC-112's rounding
  and SPEC-161's driven copy/original bound apply to key-on rows as they are. (They are two
  functions, one per op, as today.)

### 8. The parity rows

Hand-written recipe text, like the harness's alpha rows (`tests/ddphotos_parity.mjs` ~548), since
`recipe.js` has no crop until SPEC-161; so `ROW_COUNT` and the self-test's expected-red lists for
TABLE do not move. Exact (no resample): PNG bytes equal native vs wasm. Inputs: W
`photo.webp` 1600×1071, R `photo_rot.jpg` (Orientation 6), and three new committed fixtures made
by ImageMagick from W (not by crustyimg): P 1071×1600, S 1071×1071, N 1071×1072. Aspect `16:9`
(no input is already at it, so every row cuts or grows).

Per op (crop, pad), **13 rows**:
- W: on, off (`16:9`); on with `9:16` written (catches swap-only-when-portrait) — 3
- P: on, off — 2
- S: on `16:9`, on `9:16`, off `16:9` — 3
- N: on, off — 2
- R with `auto-orient`: on, off — 2
- R without `auto-orient`: on — 1

Derivation: 3 + 2 + 3 + 2 + 2 + 1 = 13; × 2 ops = **26 rows**. Design may trim but must keep every
mutant below red.

**Negative control:** each "on" row asserts its predicted size as a constant **and** that it
differs from its "off" twin where the shape is not square. On P, crop `16:9` off is
`aspect_size(16, 9, 1071, 1600)` = **1071×602**; on is `aspect_size(9, 16, 1071, 1600)` =
**900×1600**. A harness that ignored the key gives 1071×602 on both and goes red. The self-test
feeds the "off" recipe text in place of the "on" row and requires red.

Mutants (design quotes each exact edit in its mutation table):
- the key read but not applied in `crop` → crop P/N/R+ao "on" rows red;
- the same in `pad` → pad rows red (one rule, two ops; a key wired into only one is the likely miss);
- swap when portrait instead of normalize → W "on `9:16`" red;
- `w >= h` as landscape → S "on `9:16`" red. ⚠ **S "on `16:9`" cannot catch it**: a square taken as
  landscape normalizes `16:9` to itself. A row whose written aspect is already in the mutant's
  order cannot fail (the "commuting ops" lesson, SPEC-160);
- `w <= h` as portrait → S "on `16:9`" red;
- a tolerance (`|w − h| ≤ 1` as square) → N "on" red.

**Goldens that must stay byte-identical:**
- `tests/fixtures/recipe_corpus/*.golden` and `*.output` for all 22 corpus and bundled recipes,
  including `v2_crop_aspect` (`2163x1730`, `1730x1384`, `2163x1730`) and `v2_pad_aspect`
  (`2600x3250`, `2080x2600`, `2600x3250`) (`tests/recipe_corpus_output.rs:83,97`);
- `tests/recipe_round_trip.rs`'s canonical TOML;
- the parity harness's `GOLDEN_*` texts, its 16 TABLE rows and its alpha rows;
- every existing expectation in `tests/crop.rs`, `tests/pad.rs`, `tests/edit_geometry.rs`,
  `tests/edit_pad_trim.rs`.

How build proves it: no `CRUSTYIMG_BLESS` run; `git diff --name-status origin/main -- tests/fixtures/`
shows only `A` lines (new fixtures, any new corpus recipe), never `M`; and those suites pass
unchanged. If design adds a corpus recipe with the key, the "22" count moves and the test's own
count assertion with it; derive the new count with `/bin/ls`.

### 9. Docs, and the sentences build must drive

Files: `docs/recipes.md` (crop ~60–79, pad ~98–130), `recipes/README.md` (op table 95/98, crop key
table ~231–232, pad key table), `docs/data-model.md` (crop table ~339, pad table ~396),
`docs/api-contract.md` (crop ~461, pad ~476, and ~571 if 6 adds flags), `skills/crustyimg/SKILL.md`
(236–237), `edit --help` (if 6), and a CHANGELOG line folded into the Unreleased crop/pad entries
(written at build).

Behaviour sentences build must drive (AGENTS §15), each on one input and a folder batch, with the
batch's exit code quoted:
- "With `orientation = "match"`, a portrait image gets the aspect turned" (P: 900×1600 not 1071×602);
- "A square image gets the aspect as written" (S both orders);
- "One pixel decides: 1071×1072 is portrait" (N);
- "The key reads the image the step receives, so put `auto-orient` first" (R with and without);
- "`1:1` is unchanged by the key";
- "The key is refused with a rectangle crop / a border pad", and "an unknown value is refused when
  the recipe loads (exit 1)", on a batch too (load errors fire before any input is read);
- if 6: "`--crop-orientation` with a pixel `--crop` is a usage error (exit 2)", and "`--save-recipe`
  writes the key only when typed".

### 10. The three handed-in items

- **`format = "bogus"`**: touches `src/recipe/mod.rs` and `src/sink/mod.rs` (where `unsupported
  output extension` lives). **No overlap.**
- **`quality = 101`**: touches `src/recipe/mod.rs` and, if `-q` is refused too, `src/cli/mod.rs`'s
  global `quality` arg (~93–95). **Overlap in `src/cli/mod.rs` only if question 6 adds flags**, and
  then only textual (the `Edit` variant ~479/498, a different struct). Schedulable in parallel.
- **wasm `validate()`**: touches `src/wasm.rs` and reads `registry.rs`'s `param_specs`. **No
  overlap** (this spec adds a `ParamSpec` row in `crop.rs`/`pad.rs`, which `validate()` would pick
  up for free).

This spec touches no file in `src/recipe/`, `src/sink/` or `src/wasm.rs`.

### 11. A DEC?

**Recommend an amendment to DEC-112, not a new DEC.** DEC-112 rule 5 already owns aspects and its
`affected_scope` already covers `crop.rs` and `integrations/ddphotos/edit/**`, the two places the
rule must agree. Recommended content, a rule 8 "Aspects can follow the image":
- `orientation = "match"` on an aspect `crop` or `pad` puts the aspect's larger side along the
  larger side of the buffer the step receives (DEC-113: after any `auto-orient`, never the tag);
  `width > height` is landscape, `width < height` portrait, equal is square and takes the aspect as
  written. No tolerance.
- Absent is off and has no other spelling; off is SPEC-145's and SPEC-151's meaning.
- The fit and rounding (rule 5) are unchanged: the key only orders the two sides.
- The key with a box or a border is refused at load.
- Add `src/operation/pad.rs` (and `src/cli/ops.rs` if 6) to `affected_scope`.
If the maintainer prefers a standalone record, it takes **DEC-118** (`/bin/ls decisions/` ends at
DEC-117 on `main` at `cd519f9`). Maintainer question 4.

### 12. Size

**[M], one spec.** The code is small (one table row and a few lines per op, two flags), but the
verification is not: 2 ops × on/off × 5 shapes, plus both written orders on the square and the
landscape, plus EXIF with and without `auto-orient` (26 parity rows), 4 load refusals × 2 ops plus
"must not refuse" rows, 2 flag refusals and 2 save-recipe rows if flags land, a mutation table of
6+, a DEC amendment and five doc files. SPEC-151 (`pad`, one op, no flags) was [S]; SPEC-153 and SPEC-160
(edit flags) were each [M]. Recipe-only (question 3 = no) would be [S]. **No split:** the flags
reuse the key's rule, its refusals and its docs rows, so a second spec would repeat the design and
verify fixed costs for about 40 lines of flag code.

### Scope

In:
- `orientation = "match"` on `crop` and `pad`'s aspect form, with the rule in 3–4;
- the load refusals in 5;
- `--crop-orientation` / `--pad-orientation` on `edit`, if question 3 says so;
- unit tests in each op, CLI tests, the 26 parity rows and the three new fixtures;
- the docs in 9 and the DEC-112 amendment in 11.

Out, and where it lives:
- the page emitting the key, its `recipe.js` port and its preset/swap UI → SPEC-161 and the
  "crop mode" entry (STAGE-060);
- named presets (`"5x7"`, `"square"`) in the recipe format → declined (settled); a docs table of
  common ratios may be added by this spec's docs only as prose, no key;
- forcing values (`"landscape"`, `"portrait"`) → declined here (2); a later value would be its own
  backlog entry;
- the key on `trim`, `rotate` or a future straighten → none has an aspect; no entry needed;
- the three handed-in items → PROJ-017 brief, "Handed in from PROJ-011";
- a wasm `ImageInfo.orientation` getter → SPEC-161 declined it; out.

### Rough build file list

- `src/operation/crop.rs` — `CROP_PARAMS` row, `Form::Aspect` carries the flag, normalization in
  `apply`, box-form refusal, unit tests
- `src/operation/pad.rs` — the same for `pad`'s aspect form and `border` refusal
- `src/operation/mod.rs` — only if the normalization becomes one shared helper (recommended: one
  function both ops call, so the rule can't drift between them)
- `src/cli/mod.rs`, `src/cli/ops.rs` — the two flags (if 3 = yes)
- `tests/crop.rs`, `tests/pad.rs`, `tests/edit_geometry.rs`, `tests/edit_pad_trim.rs`
- `tests/ddphotos_parity.mjs`, `tests/fixtures/ddphotos/` (P, S, N fixtures, README)
- possibly `tests/fixtures/recipe_corpus/` (a new `v2_crop_aspect_match` recipe) and
  `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs`
- `docs/recipes.md`, `recipes/README.md`, `docs/data-model.md`, `docs/api-contract.md`,
  `skills/crustyimg/SKILL.md`, `CHANGELOG.md`
- `decisions/DEC-112-…md` (amendment, written at design)

### Maintainer questions

1. **The key's name and values.** Recommend `orientation = "match"`, one value, absent = off, no
   spelling for off.
2. **A square image.** Recommend the aspect as written (3:2 stays 3:2); `1:1` with the key is a
   no-op, not refused.
3. **`edit` flags.** Recommend `--crop-orientation match` and `--pad-orientation match` in this spec
   ([M]); "recipe-only for now" keeps it [S].
4. **The DEC.** Recommend an amendment to DEC-112 (rule 8); a standalone record would be DEC-118.

### Maintainer rulings (2026-11-25)

The maintainer accepted all four recommendations:
1. **The key:** `orientation = "match"`, one value. Leaving it out means off, and off has no other
   spelling. `--save-recipe` writes the key only when its flag was typed.
2. **A square image** takes the aspect as written (3:2 stays 3:2). `1:1` with the key is accepted
   and changes nothing. Strictly wider is landscape, with no tolerance.
3. **`edit` flags are in this spec:** `--crop-orientation match` and `--pad-orientation match`, built
   like the gravity flags. The spec stays [M].
4. **The DEC:** amend DEC-112 with a rule 8, "Aspects can follow the image". Design writes the
   amendment's exact text; ship applies it after the merge.

### Findings outside this spec

None filed. (Load errors number steps from 0, `recipe step 0 ('crop')`, which is existing and
consistent behaviour, not a finding.)

## Design

Designed 2026-11-25 (Opus) on `main` at `1331fb7`. Everything below was driven on two release
builds in scratch `CARGO_TARGET_DIR`s (removed at the end): `main` as it is, and a prototype that is
`main` plus the code shape below (a scratch worktree, never pushed). Probe images and recipes lived
in the session scratchpad.

### Design corrections to the frame

1. **Existing test expectations do move, in five places, and only to list the new key.** The frame
   says every existing expectation in `tests/crop.rs` and `tests/pad.rs` stays. It can't: the
   unknown-key message lists the table's keys, so `tests/crop.rs:143`
   (`expected one of: x, y, width, height, aspect, gravity`) and `tests/pad.rs:284`
   (`const KEYS`) gain `orientation`, and three registry unit tests pin the tables:
   `with_builtins_contains_pad` (`src/operation/registry.rs:545` keys, 547–558 kinds),
   `every_builtin_publishes_a_param_schema` (783 crop keys, 790–806 crop kinds, 809 pad keys,
   826–841 pad kinds) and `one_of_values_are_accepted_by_their_constructors` (943–944, the count
   `6 + 9 + 9 + 2 + 9` becomes `6 + 9 + 9 + 1 + 2 + 9 + 1`). `tests/pad.rs`'s `KEYS` is read by
   two tests (`every_pad_refusal_names_its_reason` and
   `apply_refuses_a_pixel_width_key_and_writes_nothing`, `:1185`), so five edits move six tests.
   Driven (M17): with `main`'s text for these, exactly those six go red; with the edits the whole
   suite is green. Every other existing
   expectation is unchanged (see "What doesn't change").
2. **`tests/orientation.rs` already exists** (SPEC-110's EXIF-baking suite, 824 lines). The new
   integration suite is **`tests/aspect_orientation.rs`**. (The prototype overwrote it once by
   accident; the build must create a new file, never write over that one.)
3. **The frame's docs list missed `docs/cli-reference.md`** (`edit` synopsis :310, `--crop W:H`
   bullet :323–327, `--pad W:H` bullet :330–333). Added below.
4. **The key table's order.** The frame doesn't place the row. `crop` appends it (`x, y, width,
   height, aspect, gravity, orientation`); `pad` puts it after `gravity` so the aspect-form keys
   sit together (`aspect, gravity, orientation, border, color`). Both orders are what the
   unknown-key message prints, quoted below.
5. **Nothing in the frame was found wrong otherwise.** Driven and confirmed: the 26 parity sizes
   (all 26 match an independent fraction model, on the prototype; `main` refuses each "on" row
   with `unknown key 'orientation'` and gives the same "off" sizes), the near-square row, one
   shared helper, the table's messages, the refusals' placement, the flags' clap behaviour, batch.

### What was checked before building on it

- **One shared helper.** `crop.rs`'s `aspect_size(a, b, iw, ih)` (u32 → u32, `crop.rs:212`) and
  `pad.rs`'s `aspect_canvas(a, b, w, h)` (u64 sides, `pad.rs:246`) both take the aspect as two
  `u32`s first. A helper `follow_orientation(a, b, w, h) -> (u32, u32)` in `src/operation/mod.rs`
  reorders `(a, b)` before either call and touches neither function's body, so neither rounding
  changes. Driven: with the key absent, the full prototype suite (`cargo test --release
  --no-fail-fast`, 1463 `ok` lines, exit 0) passes, which includes every existing crop and pad test.
- **The key table.** `ParamKind::OneOf(&["match"])` (`registry.rs:83`). `check_params`
  (`registry.rs:246–267`) runs before any form rule, over the keys in sorted order, and
  `OneOf` uses `;` before `got` (`registry.rs:257–261`). Quoted from the prototype: unknown value
  `'orientation' must be one of: match; got "landscape"`, wrong case `… got "Match"`, wrong type
  `… got true` and `… got the integer 1` (`describe_value`, `registry.rs:221`). The box-form and
  border-form refusals come from the ops.
- **Placement of the form refusals.** `crop`'s `gravity` refusal is `crop.rs:135–140`, after the
  aspect branch returns and before the "needs a rectangle" check; `pad`'s is `pad.rs:131–137`,
  after the "needs an aspect or a border" check. The `orientation` refusal goes **immediately
  after** each. So: a box crop with both `gravity` and `orientation` reports the **gravity**
  message (driven); a border pad with both reports the **gravity** message (driven); a crop with
  only `orientation` reports the orientation message (driven; same as `gravity` alone today); a pad
  with `orientation` and no form reports `pad needs an aspect … or a border` (driven, the existing
  message). A wrong-typed key on a box crop reports the type error (the table runs first; driven).
- **`edit`'s flags.** `--crop-gravity` has `requires = "crop"` (`src/cli/mod.rs:478`), so without
  `--crop` clap refuses it (exit 2); with a pixel `--crop`, `build_edit_ops` refuses it
  (`src/cli/ops.rs:1551–1557`). `--pad-gravity` the same (`mod.rs:497`, `ops.rs:1604–1610`). A bad
  value goes through the op and `build_flag_op` swaps `'gravity'` for the flag name
  (`ops.rs:1433–1450`). The new flags copy this exactly: `requires`, a refusal in the pixel/border
  branch placed **after** the gravity one (so a pixel crop with both flags gets the gravity message,
  driven), and a `("orientation", "--crop-orientation")` / `("orientation", "--pad-orientation")`
  entry in the swap list. clap's own repeated-flag and missing-value errors apply unchanged (driven:
  `the argument '--crop-orientation <MATCH>' cannot be used multiple times`, `a value is required
  for '--crop-orientation <MATCH>' but none was supplied`, exit 2).
- **Batch.** `edit <dir> --crop 3:2 --crop-orientation match --out-dir … --save-recipe c.toml`
  over L 1600×1071, P 1071×1600, S 1071×1071, N 1071×1072: exit 0, outputs **1600×1067, 1067×1600,
  1071×714, 715×1072**; key off: 1600×1067, 1071×714, 1071×714, 1071×714. The aspect form already
  allows a batch `--save-recipe` (only a pixel `--crop` or border `--pad N` is refused,
  `ops.rs:1706–1727`, which keys on `StepRecord::Resolved`); confirmed, exit 0, and `apply` of the
  saved `c.toml` over the same folder gives byte-identical files (4 of 4). Pad over the same folder:
  1607×1071, 1071×1607, 1607×1071, 1071×1607.
- **The near-square row**, by hand. N = 1071×1072, aspect `16:9`.
  - crop off: `aspect_size(16, 9, 1071, 1072)`: 1071·9 = 9639 ≤ 1072·16 = 17152, so
    (1071, (2·1071·9 + 16) / 32 = 19294/32 = 602) → **1071×602**.
  - crop on: N is portrait (1071 < 1072), so (9, 16); `aspect_size(9, 16, 1071, 1072)`:
    1071·16 = 17136 ≤ 1072·9 = 9648 is false, so ((2·1072·9 + 16) / 32 = 19312/32 = 603, 1072) →
    **603×1072**.
  - pad off: `aspect_canvas(16, 9, 1071, 1072)`: 1071·9 = 9639 ≥ 1072·16 = 17152 is false, so
    ((2·1072·16 + 9) / 18 = 34313/18 = 1906, 1072) → **1906×1072**.
  - pad on: `aspect_canvas(9, 16, 1071, 1072)`: 1071·16 = 17136 ≥ 1072·9 = 9648, so
    (1071, (2·1071·16 + 9) / 18 = 34281/18 = 1904) → **1071×1904**.
  On differs from off for both ops. Driven: the prototype gives the same four sizes.
- **The parity harness.** `tests/ddphotos_parity.mjs`: the alpha rows (`:548–616`) are a function
  `alphaRows(ctx)` that writes the recipe text by hand, runs `transform` and native `apply` on one
  fixture, and returns failure strings; `main` concatenates them after `tableChecks`
  (`:641`). `ROW_COUNT = 16` (`:156`) counts TABLE only, and the self-test (`:526–546`) mutates the
  native text of TABLE rows and compares the red row numbers to fixed lists. `INPUTS` (`:159`) is
  read only by TABLE rows and `alphaRows`. So 26 hand-written rows fit as a third function,
  `orientationRows(ctx, mutate)`, with their own self-test case; TABLE, `ROW_COUNT`, `N1/N2`'s lists
  and the alpha rows don't move.
- **wasm reads no orientation of its own.** `src/wasm.rs:206–251` (`transform`) decodes, loads with
  `Recipe::from_toml`, builds with `OperationRegistry::with_builtins()` and runs; it bakes no EXIF
  orientation, and neither does native `apply`. So the R-without-`auto-orient` row sees the stored
  1600×1071 buffer on both sides (driven native: crop 1600×900, pad 1904×1071).

### The code shape

`file:line` is on `main` at `1331fb7`.

- **`src/operation/mod.rs`**, inserted before `impl FromStr for Gravity` (`:1243`), after
  `GRAVITY_NAMES` (`:1231–1241`):
  - `const ORIENTATION_NAMES: &[&str] = &["match"];` (private, like `GRAVITY_NAMES`; the registry's
    tests reach it as `crate::operation::ORIENTATION_NAMES`).
  - `fn follow_orientation(a: u32, b: u32, w: u32, h: u32) -> (u32, u32)`:
    `let (long, short) = (a.max(b), a.min(b)); if w > h { (long, short) } else if w < h {
    (short, long) } else { (a, b) }`. Doc comment: the larger side runs along the image's larger
    side; strictly wider is landscape, strictly taller portrait, a square keeps the aspect as
    written; integer comparisons only.
  - One unit test in the existing `mod tests` (`:2074`): FT-1.
- **`src/operation/crop.rs`**:
  - imports (`:18–21`): add `follow_orientation` and `ORIENTATION_NAMES`;
  - `CROP_PARAMS` (`:25–66`): append `ParamSpec { key: "orientation", kind:
    ParamKind::OneOf(ORIENTATION_NAMES) }` after `gravity`;
  - `Form::Aspect` (`:83–87`): add `follow: bool` with a one-line doc;
  - `from_params` aspect branch (`:120–132`): `let follow = params.contains("orientation");`
    (the table already limited it to `"match"`) and carry it into `Form::Aspect`;
  - after the gravity refusal (`:135–140`): `if params.contains("orientation") { return
    Err(invalid("'orientation' is only used with 'aspect'; a rectangle crop's shape is set by its
    width and height")) }`;
  - `apply`'s `Form::Aspect` arm (`:344–349`): `let (a, b) = if follow { follow_orientation(a, b,
    iw, ih) } else { (a, b) };` before `aspect_size`. The module doc (`:3–6`) gains "and an
    optional `orientation`".
- **`src/operation/pad.rs`**:
  - imports (`:52–55`): add `follow_orientation`, `ORIENTATION_NAMES`;
  - `PAD_PARAMS` (`:59–80`): insert the `orientation` row after `gravity` (`:64–67`);
  - `Form::Aspect` (`:84–88`): add `follow: bool`;
  - after the gravity-with-border refusal (`:131–137`): `if has_border &&
    params.contains("orientation") { … "'orientation' is only used with 'aspect'; a border pad adds
    the same width on every side" }`;
  - the aspect form (`:139–149`): read `follow` as crop does;
  - `layout`'s `Form::Aspect` arm (`:188–192`): `let (a, b) = if follow { follow_orientation(a, b,
    iw, ih) } else { (a, b) };` before `aspect_canvas`. The cap messages keep printing the aspect
    **as written** (`'aspect' = "16:9"`), which is what the user typed. The module doc (`:5–7`)
    gains the key.
- **`src/operation/registry.rs`**: no code change; the test edits in correction 1.
- **`src/cli/mod.rs`**, `Commands::Edit`:
  - after `crop_gravity` (`:476–479`): `/// Turn an aspect --crop W:H to follow each image: "match"
    puts the aspect's long side along the image's long side (a square image keeps it as written).`
    `#[arg(long, value_name = "MATCH", requires = "crop")] crop_orientation: Option<String>,`
  - after `pad_gravity` (`:495–498`): the same with `--pad W:H`, `requires = "pad"`,
    `pad_orientation`;
  - the `dispatch` destructure (`:1235–1252`) and the `EditFlags { … }` literal (`:1255–1266`):
    pass both through.
  - `value_name = "MATCH"`, not a clap `value_parser`: the value goes to the op's table, as the
    gravity flags' do, so the flag and the recipe give one message.
- **`src/cli/ops.rs`**:
  - `EditFlags` (`:1103–1116`): `crop_orientation: Option<String>` after `crop_gravity`,
    `pad_orientation` after `pad_gravity` (the struct derives `Default`, so the unit tests'
    `..EditFlags::default()` literals compile unchanged);
  - `PAD_FLAGS` (`:1418–1422`): `[(&str, &str); 4]` with `("orientation", "--pad-orientation")`
    after gravity;
  - a small helper `fn with_text(params: OperationParams, key: &str, value: &str) ->
    OperationParams` beside `one_param` (`:1458`), used for the two new keys (the gravity code at
    `:1535–1543` and `:1588–1596` may keep its inline copy; refactoring it is optional and must
    not change any message);
  - crop aspect branch (`:1533–1549`): add the key when `flags.crop_orientation` is `Some`, and add
    `("orientation", "--crop-orientation")` to the swap list;
  - crop pixel branch (`:1550–1557`): after the gravity refusal, `if
    flags.crop_orientation.is_some()` → `CliError::Usage("--crop-orientation only turns an aspect
    --crop W:H; a pixel --crop WxH+X+Y keeps the size it gives")`;
  - pad aspect branch (`:1586–1601`): add the key when `flags.pad_orientation` is `Some`;
  - pad border branch (`:1603–1610`): after the gravity refusal, `if flags.pad_orientation.is_some()`
    → `CliError::Usage("--pad-orientation only turns an aspect --pad W:H; a border --pad N adds the
    same width on every side")`.
  - **The recipe writer needs no change.** `--save-recipe` writes `StepRecord::Built(op.params())`
    (`ops.rs:1450–1453`, written at `:1751–1777`), and `params()` returns the step's keys as built,
    so `orientation = "match"` is saved exactly when the flag put it there. Driven (FT-13 and its
    mutants M9 and M10).
  - `run_edit`'s doc (`:1660–1681`) needs no change.

### Every message, with its exit code

Load refusals (exit **1**, `apply`, one input or a batch; nothing read or written). Each line is
what `apply` prints; wasm `transform` throws the same text without `error: `.

1. `error: recipe step 0 ('crop'): 'orientation' is only used with 'aspect'; a rectangle crop's shape is set by its width and height`
   (a box crop with the key; also a crop with only the key). New wording, modelled on `crop.rs:137`.
2. `error: recipe step 0 ('crop'): 'orientation' must be one of: match; got "landscape"`
3. `error: recipe step 0 ('crop'): 'orientation' must be one of: match; got "Match"`
4. `error: recipe step 0 ('crop'): 'orientation' must be one of: match; got true` (and
   `… got the integer 1`)
5. `error: recipe step 0 ('pad'): 'orientation' is only used with 'aspect'; a border pad adds the same width on every side`
   (tail reused from `pad.rs:133–134`).
6. `error: recipe step 0 ('pad'): 'orientation' must be one of: match; got "landscape"`
7. `error: recipe step 0 ('pad'): 'orientation' must be one of: match; got "Match"`
8. `error: recipe step 0 ('pad'): 'orientation' must be one of: match; got true` (and
   `… got the integer 1`)

Changed by the new row (exit 1), the unknown-key list:
- `error: recipe step 0 ('crop'): unknown key 'bogus'; expected one of: x, y, width, height, aspect, gravity, orientation`
- `error: recipe step 0 ('pad'): unknown key 'bogus'; expected one of: aspect, gravity, orientation, border, color`

Deterministic overlaps (existing messages, exit 1): a box crop with `gravity` and `orientation` →
`'gravity' is only used with 'aspect'; a rectangle crop is placed by x and y`; a border pad with
both → `'gravity' is only used with 'aspect'; a border pad adds the same width on every side`; a box
crop with `orientation = true` → message 4.

`edit` flag refusals (exit **2**, nothing written, no recipe written):
9. `error: --crop-orientation only turns an aspect --crop W:H; a pixel --crop WxH+X+Y keeps the size it gives`
10. `error: --pad-orientation only turns an aspect --pad W:H; a border --pad N adds the same width on every side`
11. `error: --crop-orientation must be one of: match; got "landscape"` (and `got "Match"`)
12. `error: --pad-orientation must be one of: match; got "landscape"` (and `got "Match"`)
13. `--crop-orientation` without `--crop` (clap): `error: the following required arguments were not
    provided:` / `  --crop <WxH+X+Y|W:H>`, then clap's usage line.
14. `--pad-orientation` without `--pad` (clap; `--pad` itself requires `--pad-color`):
    `error: the following required arguments were not provided:` / `  --pad-color <COLOR>` /
    `  --pad <N|W:H>`.
15. A pixel `--crop` with both `--crop-gravity` and `--crop-orientation`: the existing
    `error: --crop-gravity only places an aspect --crop W:H; a pixel --crop WxH+X+Y is placed by its +X+Y`.

A batch: `edit <dir> --crop 300x200+0+0 --crop-orientation match --out-dir …` exits 2 with
message 9 and creates no output directory (driven). `apply --recipe bad.toml <dir> --out-dir …`
with `orientation = "landscape"` exits 1 with message 2 and creates no output directory (driven).
No new batch line.

No message names a spec, stage or DEC id.

### The replay rows

FT-13 runs, for each op and each of a landscape 64×48, portrait 48×64 and square 48×48 noise PNG:
`edit IN --crop 3:2 --crop-orientation match --save-recipe r.toml -o e.png -y`, then `apply
--recipe r.toml IN -o b.png -y`, and asserts both exit 0, `e.png` and `b.png` have the same colour
type and bytes, the size equals a literal, and `r.toml` equals a literal text. The same four rows
without the flag pin that the key is absent when not typed. Sizes (hand-derived, halves up):
- crop on: 64×43, 43×64, 48×32; crop off: 64×43, 48×32, 48×32;
- pad on: 72×48, 48×72, 72×48; pad off: 72×48, 96×64, 72×48.
Saved text, crop on (pad likewise with `color = "#ffffff"` before `orientation`):

```
version = "1"

[[step]]
op = "auto-orient"

[[step]]
op = "crop"
aspect = "3:2"
orientation = "match"
```

(`version = "1"` is what `edit --save-recipe` writes on `main` today for every recipe it saves;
not this spec's to change.) Driven on the prototype, also at full size on P/L/S photos: pixel
difference 0 (ImageMagick `compare -metric AE`) for all six.

### What doesn't change, and what pins it

- The 22 recipe-corpus goldens and outputs (`tests/fixtures/recipe_corpus/`), pinned by
  `tests/recipe_corpus_output.rs` and its count assertion; no corpus recipe is added (the key's
  sizes are pinned by FT-2/FT-3 and the parity rows), so the count stays 22.
- `tests/recipe_round_trip.rs`'s canonical TOML: pinned by that test; no recipe there carries the
  key.
- The parity harness's `GOLDEN_*` texts, TABLE (16 rows, `ROW_COUNT`), `N1/N2_EXPECTED_RED` and the
  alpha rows: pinned by the harness itself (P4, P5, the self-test); the new rows are a separate
  function.
- Every crop, pad and `edit` geometry expectation in `tests/crop.rs`, `tests/pad.rs`,
  `tests/edit_geometry.rs`, `tests/edit_pad_trim.rs` **except** the two unknown-key lists in
  correction 1. Pinned by those suites, unchanged.
- The `crop`/`pad` unit tests (`crop.rs` `mod tests`, `pad.rs` `mod tests`), unchanged: FT-1 is the
  only new unit test.
- No new pin is needed: each item above already has one.

**The build's proof:** no `CRUSTYIMG_BLESS` run; `git diff --name-status origin/main --
tests/fixtures/` shows only `A` lines (the three parity fixtures, their README is `M` — see below);
`git diff origin/main -- tests/crop.rs tests/pad.rs src/operation/registry.rs` shows only the
correction-1 lines; and the four suites pass.

⚠ `tests/fixtures/ddphotos/README.md` is an `M` (it gains three table rows and the commands). That
is the one fixture-tree modification allowed; the proof line above excepts it by name.

### Fixtures

Three parity inputs, made with ImageMagick 7.1.2-34 from the committed `photo.webp`, never with
crustyimg, from `tests/fixtures/ddphotos/`:

```sh
magick photo.webp -rotate 90 -strip -quality 82 -define webp:lossless=false photo_portrait.webp
magick photo.webp -gravity center -crop 1071x1071+0+0 +repage -strip -quality 82 \
  -define webp:lossless=false photo_square.webp
magick photo.webp -rotate 90 -gravity center -crop 1071x1072+0+0 +repage -strip -quality 82 \
  -define webp:lossless=false photo_near_square.webp
```

Sizes: P **1071×1600**, S **1071×1071**, N **1071×1072** (check with `magick identify -format
%wx%h`). On the design machine they were 20,006, 13,358 and 13,166 bytes; the build records its
own bytes and SHA-256 in the README's table (a different ImageMagick gives different bytes, which is
fine: the rows compare native with wasm on the same bytes and assert sizes, not hashes). The README
gets three rows and the commands, and the same "do not regenerate to make a row pass" line covers
them.

### The parity rows (`tests/ddphotos_parity.mjs`)

`INPUTS` (`:159`) gains `P: "photo_portrait.webp", S: "photo_square.webp", N:
"photo_near_square.webp"`. A new section after the alpha rows, `orientationRows({ transform, bin,
python }, mutate = (t) => t)`, builds each row's recipe text by hand:

```
version = "2"
[[step]]
op = "auto-orient"          (R+ao rows only)
[[step]]
op = "crop"                 (or "pad", with color = "#ffffff")
aspect = "16:9"             (or "9:16")
orientation = "match"       ("on" rows only)
```

runs `transform(bytes, mutate(text), "png")` and native `apply --recipe … -o X.png -y`, decodes
both with the existing Pillow `decode`, and fails the row (label `O-<op>-<input>-<on|off>-<aspect>`)
unless: both sizes equal the row's literal, and the RGBA bytes are equal (no resample, so exact,
DEC-110 limb 2). It returns failure strings, and `main` (`:641`) becomes `[...tableChecks(table),
...alphaRows(ctx), ...orientationRows(ctx)]`. The final summary line is unchanged in shape.

The 26 rows and their literal sizes (input, key, written aspect → crop, pad), from an independent
fraction model and driven on the prototype's native binary (26/26 equal):

- W 1600×1071: on 16:9 → 1600×900, 1904×1071; off 16:9 → 1600×900, 1904×1071;
  on 9:16 → 1600×900, 1904×1071
- P 1071×1600: on 16:9 → 900×1600, 1071×1904; off 16:9 → 1071×602, 2844×1600
- S 1071×1071: on 16:9 → 1071×602, 1904×1071; on 9:16 → 602×1071, 1071×1904;
  off 16:9 → 1071×602, 1904×1071
- N 1071×1072: on 16:9 → 603×1072, 1071×1904; off 16:9 → 1071×602, 1906×1072
- R with `auto-orient`: on 16:9 → 900×1600, 1071×1904; off 16:9 → 1071×602, 2844×1600
- R without `auto-orient`: on 16:9 → 1600×900, 1904×1071

Count: per op 3 + 2 + 3 + 2 + 2 + 1 = 13; × 2 ops = **26**.

**Negative control (self-test N3).** `selfTest` gains a case `"N3 the orientation key stripped"`
that runs `orientationRows(ctx, (t) => t.replace('orientation = "match"\n', ""))` and requires the
red labels to be exactly the eight rows where the key changes the size:
`O-crop-W-on-9:16, O-crop-P-on-16:9, O-crop-N-on-16:9, O-crop-R+ao-on-16:9` and the same four for
`pad` (count: per op, W on 9:16 + P on + N on + R+ao on = 4; × 2 = 8; every other "on" row is a
square, already landscape, or the stored landscape R). A harness that ignored the key gives the
"off" size there and goes red.

### Docs (each surface, and what it will say)

- `docs/recipes.md` crop entry (`:60–79`): after the `gravity = "north"` example, one sentence and
  a TOML block: "Add `orientation = "match"` and the aspect follows each photo: one `aspect =
  "3:2"` recipe gives 3:2 on landscape photos and 2:3 on portrait ones. A square photo keeps the
  aspect as written. The key reads the image as the step receives it, so put `op =
  "auto-orient"` first for camera JPEGs."
- `docs/recipes.md` pad entry (`:98–118`): one sentence, "`orientation = "match"` turns the aspect
  to follow each photo, as for `crop`."
- `recipes/README.md`: the op table rows `:95` and `:98` gain "`+ orientation`"; the crop key table
  (`:225–232`) gains a row `| `orientation` | aspect | `"match"`: the aspect's long side follows
  the image's long side; a square image keeps it as written |`; a bullet after the
  `auto-orient` bullet (`:246–247`): "With `orientation = "match"`, a portrait image (taller than
  wide) gets the aspect turned, so `"3:2"` cuts a 2:3 region from it; a landscape image gets the
  larger side as the width; a square image gets the aspect as written. One pixel decides:
  1071×1072 is portrait. `"1:1"` is unchanged by the key. The key is refused on a rectangle crop,
  and a value other than `"match"` is refused when the recipe loads." The pad section
  (`### The pad step`, `:272–303`) gets one bullet saying the same for `pad`, and that it is
  refused with `border`.
- `docs/data-model.md` crop (`:329–341`): the intro sentence gains "and optionally `gravity` and
  `orientation`" and "so is `orientation` on a rectangle"; the table gains `| `orientation` |
  string | `"match"` only; absent means the aspect as written |` and a sentence stating the rule
  (strictly wider is landscape, strictly taller portrait, square as written; it reads the buffer
  the step receives). Pad (`:392–405`): the table gains the same row ("aspect form only"), and
  `:401` reads "an aspect (`aspect`, `gravity`, `orientation`)".
- `docs/api-contract.md`: "Crop in a recipe" (`:460–467`) gains "an optional `orientation =
  "match"` turns the aspect to follow each image (a bad value, or the key on a rectangle, is
  refused when the recipe loads, exit **1**)"; "Pad in a recipe" (`:476–479`) the same with
  "border"; the `edit` synopsis (`:543`) becomes `[--crop WxH+X+Y|W:H [--crop-gravity G]
  [--crop-orientation match]]` and `[--pad N|W:H --pad-color C [--pad-gravity G]
  [--pad-orientation match]]`; the exit-2 list (`:557–559`) adds "`--crop-orientation` without
  `--crop` (or with a pixel crop)" and "`--pad-orientation` without `--pad` (or with a border
  `--pad N`)".
- `docs/cli-reference.md` (missed by the frame): synopsis `:310` as in api-contract; the `--crop
  W:H` bullet (`:323–327`) gains "`--crop-orientation match` turns the aspect to follow each image
  (its long side along the image's long side; a square keeps it as written)"; the `--pad W:H`
  bullet (`:330–333`) the same with `--pad-orientation`.
- `skills/crustyimg/SKILL.md` (`:236–237`): `crop (… or aspect = "W:H" + gravity + an optional
  orientation = "match")` and the same for `pad`.
- `edit --help`: the two doc comments in the code shape (FT-15 asserts both lines).
- No other surface: `/usr/bin/grep -rln -e crop-gravity -e pad-gravity -e 'aspect = "W:H"' README.md docs
  recipes skills integrations` on `main` lists `docs/api-contract.md`, `docs/cli-reference.md`
  and `skills/crustyimg/SKILL.md`, all above; `README.md` has no crop/pad key list.

**Behaviour sentences and every exit path each names** (the build drives each on one input and a
folder batch, quoting the exit code; verify drives them again):
1. "A portrait image gets the aspect turned" — one input exit 0 (P 16:9 crop 900×1600, not
   1071×602); `apply` batch exit 0 (FT-11); `edit` batch exit 0 (FT-14).
2. "A landscape image gets the larger side as the width" (`"9:16"` on landscape → 16:9) — one
   input exit 0; batch exit 0.
3. "A square image gets the aspect as written" — S both orders, one input exit 0; batch exit 0.
4. "One pixel decides: 1071×1072 is portrait" — N, one input exit 0.
5. "`1:1` is unchanged by the key" — one input exit 0 on landscape, portrait, square.
6. "The key reads the image as the step receives it; put `auto-orient` first" — R with and
   without, one input exit 0; `edit` always bakes first (both give the portrait result).
7. "Refused on a rectangle crop / a border pad" — one input exit 1; batch exit 1, nothing written.
8. "A value other than `"match"` is refused when the recipe loads" — one input exit 1; batch
   exit 1, no output directory.
9. "`--crop-orientation` with a pixel `--crop` is a usage error" (and `--pad-orientation` with
   `--pad N`) — one input exit 2; batch exit 2, nothing written.
10. "`--crop-orientation` without `--crop` is a usage error" — exit 2 (clap).
11. "`--save-recipe` writes the key only when typed" — one input exit 0 (both texts); batch exit 0
    (FT-14).

### CHANGELOG (under Unreleased → Added, after the `edit --pad` entry)

```
- **An aspect can follow each image's orientation.** `orientation = "match"` on an aspect `crop`
  or `pad` puts the aspect's long side along the image's long side, so one `aspect = "3:2"` recipe
  gives 3:2 on landscape photos and 2:3 on portrait ones; a square image keeps the aspect as
  written. It reads the image as the step receives it, so put `auto-orient` first for camera
  JPEGs. Any other value, and the key on a rectangle crop or a border pad, is refused when the
  recipe loads (exit 1).
- **`edit --crop-orientation match` and `--pad-orientation match`** do the same for an aspect
  `--crop W:H` or `--pad W:H`, and `--save-recipe` saves `orientation = "match"` only when the
  flag was typed. With a pixel `--crop` or a border `--pad N` they exit 2.
```

### DEC-112's rule 8 (for ship to apply after the merge)

Re-read DEC-112 in full against this design. Rule 5 ("the fit is exact integer maths in `u64`, with
the short side rounded half up") is unchanged: the key only reorders `(a, b)` before
`aspect_size`/`aspect_canvas`. The Consequences bullet "**Orientation is not decided here.**" talks
about EXIF orientation and the fractions; it stays true and gets one added sentence. Nothing in
DEC-112 names a call site this design changes.

Add after rule 7:

> **8. Aspects can follow the image.** An aspect `crop` or `pad` takes an optional `orientation =
> "match"`. With it, the aspect's larger side runs along the larger side of the buffer the step
> receives (DEC-113: after any earlier `auto-orient`, never the EXIF tag): a buffer with
> `width > height` is landscape and takes the aspect as `max:min`, one with `width < height` is
> portrait and takes `min:max`, and a square buffer takes the aspect as written. There is no
> tolerance: 1071×1072 is portrait. `"1:1"` is unchanged by the key. The key only orders the two
> sides; the fit and its rounding are rule 5's. `"match"` is the only value, and leaving the key
> out is the only way to say "off", which is rule 5's meaning. The key with a rectangle crop or a
> border pad, or any other value, is refused at load. `edit --crop-orientation match` and
> `--pad-orientation match` write it, and `--save-recipe` saves it only when typed.

Append to the Consequences bullet "Orientation is not decided here": "Rule 8's `orientation` key is
about the shape of the buffer, not the tag: it reads the buffer the step receives, like the
fractions."

Append to `## Context` one line: "Rule 8 was added 2026-11-25 (SPEC-163, the maintainer's ruling
on its frame): one recipe from the ddphotos page's aspect presets must crop a folder of mixed
landscape and portrait photos."

`affected_scope` additions: `src/operation/pad.rs`, `src/cli/mod.rs`, `src/cli/ops.rs`,
`docs/api-contract.md`, `docs/cli-reference.md`. (`src/operation/mod.rs`, `crop.rs`,
`registry.rs`, `recipes/README.md`, `docs/data-model.md`, `docs/recipes.md`, `SKILL.md` and
`integrations/ddphotos/edit/**` are already in it.) `supersedes` stays null: an amendment, not a
new record.

### The page port (for SPEC-161's design)

What `recipe.js` must implement when a preset emits `orientation = "match"` and the page shows the
crop it will produce. Integer comparisons only, so there is no `f64` question:

```
// (a, b): the aspect as written; (w, h): the size of the ORIENTED base the crop step receives
// (after auto-orient), not info()'s stored size.
function followOrientation(a, b, w, h) {
  const long = Math.max(a, b), short = Math.min(a, b);
  if (w > h) return [long, short];   // landscape
  if (w < h) return [short, long];   // portrait
  return [a, b];                     // square: as written
}
```

then the page's existing port of `aspect_size` (largest fit, short side `(2·long·b + a) / (2·a)` in
integers) on the reordered pair. Worked examples, `aspect = "3:2"`:
- landscape 1600×1071: (3, 2); 1600·2 = 3200 ≤ 1071·3 = 3213, so (1600, (2·1600·2 + 3) / 6 =
  6403/6 = 1067) → **1600×1067**.
- portrait 1071×1600: (2, 3); 1071·3 = 3213 ≤ 1600·2 = 3200 is false, so ((2·1600·2 + 3) / 6 =
  1067, 1600) → **1067×1600**.
- square 1071×1071: (3, 2) as written; 1071·2 = 2142 ≤ 1071·3, so (1071, (2·1071·2 + 3) / 6 =
  4287/6 = 714) → **1071×714**.
- near-square 1071×1072 (portrait by one pixel): (2, 3); 1071·3 = 3213 ≤ 1072·2 = 2144 is false,
  so ((2·1072·2 + 3) / 6 = 4291/6 = 715, 1072) → **715×1072**.
All four driven on the prototype (`edit … --crop 3:2 --crop-orientation match` batch above).

### Cache and wasm

- **`CACHE_SCHEMA_VERSION` (`src/build/cache.rs:81`, `1`) and the lockfile format don't change.**
  The key is new and optional: a recipe without it has the same canonical TOML as before (FT-13's
  "off" texts; `tests/recipe_round_trip.rs` and the corpus goldens unchanged), so its cache key is
  unchanged. A recipe with it could never have been cached: every earlier binary refuses it at load
  (driven on `main`: `unknown key 'orientation'; expected one of: x, y, width, height, aspect,
  gravity`, exit 1), and `crop`/`pad` are themselves unreleased.
- **wasm: no code change and no new `just wasm-test` row.** `transform` reaches the key through the
  same loader and registry (`src/wasm.rs:209, 234, 243`), and the 26 parity rows run the real
  `.wasm` through `transform` on every row, which is the wasm proof. So
  **`scripts/wasm-ci-check.sh` is not in the file list.** Build still runs `just wasm-check` and
  `just wasm-test` (crop/pad compile for wasm; AGENTS §15).

### Size

**[M] holds.** Count: code 5 source files (~140 changed lines in the prototype: `git diff --stat`
135 insertions, 10 deletions before tests); tests 1 new suite (14 tests, FT-2 to FT-15) + 1 unit test
(FT-1) + 5 edited expectations moving 6 tests; 26 parity rows + 1 self-test case + 3 fixtures; 7
doc surfaces (`docs/recipes.md`, `recipes/README.md`, `docs/data-model.md`,
`docs/api-contract.md`, `docs/cli-reference.md`, `SKILL.md`, `edit --help`) + CHANGELOG; an
18-row mutation table (M0 to M17). Against SPEC-153 and SPEC-160 ([M] each, `edit` flags) it is the same
shape. Not [L]: one rule, one key, two flags; the parity rows are a copy of the alpha rows' shape.

### Overlap with other open work

Taken 2026-11-25 ~17:00 local from `gh pr list --state open --json number,title,files`:
- **PR #237** (heic via `heifer`, parked): touches `.github/workflows/ci.yml`, `CHANGELOG.md`,
  `Cargo.lock`, `Cargo.toml`, `dist-workspace.toml`, `docs/licensing.md`, `fuzz/Cargo.toml`,
  `fuzz/fuzz_targets/heic_decode.rs`, `justfile`, `src/image/heic.rs`, `src/image/mod.rs`,
  `src/image/sniff.rs`, `src/source/mod.rs`, `tests/fixtures/heic/cmyk_icc_16x16.heic`,
  `tests/fuzz_regressions.rs`, `tests/input_heic.rs`. **Overlap: `CHANGELOG.md` only**, both
  adding lines under Unreleased → Added; a textual conflict at worst, resolved by keeping both.
- No other open PR.

The build re-runs the command at its Step 0, **rebases onto `main` before each push**, and
re-checks the map after each rebase, recording any file the map missed.

## Inputs

- **Files to read:** `src/operation/crop.rs`, `src/operation/pad.rs`, `src/operation/mod.rs`
  (`GRAVITY_NAMES`, `Gravity`), `src/operation/registry.rs` (`ParamKind`, `check_params`, the
  table tests), `src/cli/mod.rs` (`Commands::Edit`, `dispatch`), `src/cli/ops.rs` (`EditFlags`,
  `build_flag_op`, `build_edit_ops`, `run_edit`), `tests/ddphotos_parity.mjs`,
  `tests/fixtures/ddphotos/README.md`, `decisions/DEC-112-…md`, `DEC-113`, `DEC-114`.
- **External APIs:** none.
- **Related code paths:** `src/operation/`, `src/cli/`.

## Outputs

- **Files created:** `tests/aspect_orientation.rs` (FT-2 to FT-15);
  `tests/fixtures/ddphotos/photo_portrait.webp`, `photo_square.webp`, `photo_near_square.webp`.
- **Files modified:** `src/operation/mod.rs`, `src/operation/crop.rs`, `src/operation/pad.rs`,
  `src/operation/registry.rs` (tests only), `src/cli/mod.rs`, `src/cli/ops.rs`, `tests/crop.rs`,
  `tests/pad.rs` (one expected string each; `KEYS` feeds two pad tests), `tests/ddphotos_parity.mjs`,
  `tests/fixtures/ddphotos/README.md`, `docs/recipes.md`, `recipes/README.md`,
  `docs/data-model.md`, `docs/api-contract.md`, `docs/cli-reference.md`,
  `skills/crustyimg/SKILL.md`, `CHANGELOG.md`.
- **New exports:** none public. `ORIENTATION_NAMES` and `follow_orientation` are private to
  `crate::operation`.
- **Database changes:** none.

## Acceptance Criteria

All 18 ticked at ship (corrected at ship): verify confirmed each, and Build Completion carries the evidence.
Each names its tests; "driven" means a mutant in the table below turns the named test red.

- [x] **AC-1** `crop` with `orientation = "match"`: a landscape image gets the aspect's larger
  side as its width, a portrait image as its height, a square image the aspect as written, and
  48×49 / 1071×1072 count as portrait (49×48 as landscape), for both written orders. Sizes are
  literals. — FT-1, FT-2, FT-4
- [x] **AC-2** The same rule on `pad`. — FT-1, FT-3, FT-4
- [x] **AC-3** Without the key, `crop` and `pad` give exactly today's sizes (each FT-2/FT-3 row's
  "off" twin), and every existing crop, pad and `edit` test passes unchanged except the two
  unknown-key lists. — FT-2, FT-3, the existing suites
- [x] **AC-4** `aspect = "1:1"` gives the same size with and without the key, on landscape,
  portrait, square and near-square images, for both ops. — FT-5
- [x] **AC-5** The key reads the buffer the step receives: on a stored-landscape image tagged
  Orientation 6, without `auto-orient` the result is landscape, after `auto-orient` portrait. —
  FT-6, FT-19 (R rows)
- [x] **AC-6** `crop` refuses at load (exit 1, nothing written), with messages 1–4 verbatim: the
  key on a rectangle (and alone), an unknown value, the wrong case, the wrong type (`true`, `1`).
  A rectangle with `gravity` and `orientation` gets the gravity message; a rectangle with
  `orientation = true` gets the type message. — FT-7
- [x] **AC-7** `pad` refuses at load with messages 5–8 verbatim: the key with `border`, an unknown
  value, the wrong case, the wrong type. A border with `gravity` and `orientation` gets the gravity
  message. — FT-8
- [x] **AC-8** The unknown-key message lists `orientation` (crop last; pad after `gravity`), and
  the registry's published tables carry the row as `OneOf(["match"])`. — FT-16, FT-17, FT-18
- [x] **AC-9** Must not refuse: the key with an aspect loads and runs (exit 0) on both ops, alone,
  with `gravity`, and with `"1:1"`, on a landscape, a square and a near-square image; and from
  `edit` on a square image. — FT-9, FT-13
- [x] **AC-10** A batch: a recipe with a bad value exits 1 with message 2 and writes no output
  directory; a good recipe over a folder of landscape, portrait and square images turns each one
  by its own shape (exit 0). — FT-10, FT-11
- [x] **AC-11** `edit --crop W:H --crop-orientation match` and `--pad W:H --pad-color C
  --pad-orientation match` run the rule; the refusals 9–15 exit 2 verbatim and write neither the
  image nor the recipe. — FT-12, FT-13
- [x] **AC-12** `--save-recipe` writes `orientation = "match"` exactly when the flag was typed
  (both literal texts), and `apply` of the saved recipe reproduces the `edit` output byte for byte
  on landscape, portrait and square inputs, for both ops. A batch `edit` with an aspect crop and
  the key saves one recipe (exit 0) and turns each image. — FT-13, FT-14
- [x] **AC-13** `edit --help` shows both flags with their one-line descriptions. — FT-15
- [x] **AC-14** Native `apply` and wasm `transform` agree exactly (size and RGBA bytes) on the 26
  parity rows, each at its literal size; stripping the key from the text turns exactly the eight
  listed rows red. — FT-19, FT-20
- [x] **AC-15** Nothing else moves: the 22 recipe-corpus goldens, `recipe_round_trip`, the parity
  harness's `GOLDEN_*`, TABLE, `ROW_COUNT`, `N1/N2` lists and alpha rows; `git diff --name-status
  origin/main -- tests/fixtures/` shows only `A` lines plus `M tests/fixtures/ddphotos/README.md`;
  no `CRUSTYIMG_BLESS` run. — the existing suites, the diff
- [x] **AC-16** The docs surfaces listed in Design say what Design writes, and each of the eleven
  behaviour sentences is driven on one input and a batch with its exit code quoted in Build
  Completion. — checklist, verify re-drives
- [x] **AC-17** CHANGELOG has the two "Added" entries verbatim. — checklist
- [x] **AC-18** `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
  the lean `--no-default-features` build and tests, `just wasm-check` and `just wasm-test` pass. —
  CI legs

## Failing Tests

Written during **design**, BEFORE build. All of FT-1 to FT-15 were written and run on the design
prototype (green there, and red on the mutants below); the build may copy them from this text and
the prototype's shape. Sizes are hand-derived literals, never read from the code under test.

- **`src/operation/mod.rs`** (`mod tests`)
  - FT-1 `follow_orientation_orders_the_aspect_by_shape` — asserts, for (a, b, w, h): (16,9,64,48)
    and (9,16,64,48) → (16,9); (16,9,48,64) and (9,16,48,64) → (9,16); (16,9,48,48) → (16,9);
    (9,16,48,48) → (9,16); (16,9,48,49) → (9,16); (9,16,49,48) → (16,9); (1,1,64,48) and
    (1,1,48,64) → (1,1).
- **`tests/aspect_orientation.rs`** (new; noise PNGs written by the test; `mod common` for
  `tiff_with_orientation`)
  - FT-2 `crop_follows_each_image_shape` — 10 rows (aspect, w×h → on, off): 16:9 64×48 → 64×36,
    64×36; 9:16 64×48 → 64×36, 27×48; 16:9 48×64 → 36×64, 48×27; 9:16 48×64 → 36×64, 36×64;
    16:9 48×48 → 48×27, 48×27; 9:16 48×48 → 27×48, 27×48; 16:9 48×49 → 28×49, 48×27;
    9:16 48×49 → 28×49, 28×49; 16:9 49×48 → 49×28, 49×28; 9:16 49×48 → 49×28, 27×48.
  - FT-3 `pad_follows_each_image_shape` — the same 10 rows: 85×48, 85×48; 85×48, 64×114;
    48×85, 114×64; 48×85, 48×85; 85×48, 85×48; 48×85, 48×85; 48×85, 87×49; 48×85, 48×85;
    85×48, 85×48; 85×48, 49×87.
  - FT-4 `the_rows_can_tell_on_from_off` — the table's own negative control: on ≠ off in exactly 4
    crop rows and 4 pad rows (landscape 9:16, portrait 16:9, 48×49 16:9, 49×48 9:16).
  - FT-5 `one_to_one_is_unchanged_by_the_key` — `1:1`, on and off: 64×48 → crop 48×48, pad 64×64;
    48×64 → 48×48, 64×64; 48×48 → 48×48, 48×48; 48×49 → 48×48, 49×49.
  - FT-6 `the_key_reads_the_image_the_step_receives` — a stored 64×48 PNG tagged Orientation 6:
    crop 16:9 on, no `auto-orient` → 64×36; on after `auto-orient` → 36×64; off after
    `auto-orient` → 48×27; pad: 85×48, 48×85, 114×64.
  - FT-7 `crop_load_refusals` — 8 rows, each exit 1 and the whole stderr: messages 1 (rectangle; and
    the key alone), 2, 3, 4 (`true` and `1`), a rectangle with `orientation = true` (message 4),
    and a rectangle with `gravity` + `orientation` (the gravity message).
  - FT-8 `pad_load_refusals` — 6 rows: messages 5, 6, 7, 8 (`true` and `1`), and a border with
    `gravity` + `orientation` (the gravity message).
  - FT-9 `the_key_with_an_aspect_is_never_refused` — on 64×48, 48×48 and 48×49: crop `3:2` on,
    crop `3:2` + `gravity = "north"` on, crop `1:1` on, and the same three for pad; each exit 0.
  - FT-10 `batch_load_refusal_writes_nothing` — `apply` over a 2-image folder with
    `orientation = "landscape"`: exit 1, stderr is message 2 exactly, `--out-dir` not created.
  - FT-11 `batch_follows_each_image` — `apply` over l 64×48, p 48×64, s 48×48: crop 16:9 on →
    64×36, 36×64, 48×27; pad 16:9 on → 85×48, 48×85, 85×48; exit 0.
  - FT-12 `orientation_flag_refusals` — on a 48×64 input with `--save-recipe`, each exit 2, nothing
    written: refusal 13 (prefix), 9, 15, 11 (`landscape`, `Match`), 14 (prefix), 10, 12
    (`landscape`, `Match`).
  - FT-13 `save_recipe_writes_the_key_only_when_typed_and_replays` — the replay rows in Design (4
    flag sets × 3 shapes): exit 0, literal size, literal saved text, `apply` replay equal in colour
    type and bytes.
  - FT-14 `batch_edit_with_the_key_saves_one_recipe` — `edit <dir> --crop 3:2 --crop-orientation
    match --out-dir … --save-recipe r.toml` over 64×48 and 48×64: exit 0, 64×43 and 43×64, the
    crop-on text.
  - FT-15 `edit_help_lists_the_orientation_flags` — `edit --help` contains
    `--crop-orientation <MATCH>`, `--pad-orientation <MATCH>`, `Turn an aspect --crop W:H to follow
    each image`, `Turn an aspect --pad W:H to follow each image`.
- **Existing tests, one expected string each** (they go red on the build's first run and green
  after the edit; this is the intended change, not a moved golden)
  - FT-16 `tests/crop.rs` `every_load_refusal_names_its_reason` (`:143`) — the unknown-key list
    ends `aspect, gravity, orientation`.
  - FT-17 `tests/pad.rs` `const KEYS` (`:284`, read by `every_pad_refusal_names_its_reason` and
    `apply_refuses_a_pixel_width_key_and_writes_nothing`) —
    `expected one of: aspect, gravity, orientation, border, color`.
  - FT-18 `src/operation/registry.rs` — `with_builtins_contains_pad` (keys and kinds),
    `every_builtin_publishes_a_param_schema` (crop keys and kinds, pad keys and kinds),
    `one_of_values_are_accepted_by_their_constructors` (count `6 + 9 + 9 + 1 + 2 + 9 + 1`, message
    names both orientation values).
- **`tests/ddphotos_parity.mjs`** (run by `just ddphotos-parity`, both modes)
  - FT-19 `orientationRows` — the 26 rows in Design, each native == wasm exactly at its literal
    size.
  - FT-20 self-test `N3 the orientation key stripped` — red labels exactly the eight listed rows.

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

The code shape, every message, the replay rows, the fixtures, the parity rows, the docs, the
CHANGELOG lines, DEC-112's rule 8, cache, wasm and the overlap map are in `## Design` above; this
section points at them and adds what the build needs on top.

### Decisions that apply

- `DEC-112` — aspects are whole numbers and the fit's rounding is rule 5's; this spec adds rule 8
  (text in Design). **Don't edit DEC-112 in the build**: ship applies the amendment after merge, so
  `main` stays true until the code lands.
- `DEC-113` — a geometry step acts on the buffer it receives and writes no metadata; the key reads
  that buffer, never the EXIF tag (FT-6, the R parity rows).
- `DEC-114` — `edit` runs its op flags in typed order and saves what ran; the new flags are
  modifiers with no position (like the gravity flags, not in `edit_op_order`), and `--save-recipe`
  saves the step's own params.
- `DEC-102` — load-time key tables; a bad key, value or type is exit 1 at load, with the table's
  wording.
- `DEC-106` — a CLI refusal names the flag (`build_flag_op`'s key swap).
- `DEC-110` — native and wasm agree exactly on a step that doesn't resample (the 26 rows).

### Constraints that apply

- `untrusted-input-hardening` — the key is a `OneOf` string checked by the table before use; the
  helper is integer comparisons on values already bounded (aspect sides ≤ 65535, image sides
  `u32`), so nothing overflows or allocates.
- `no-unwrap-on-recoverable-paths` — no `unwrap`/`expect` in the new op or flag code.
- `clippy-fmt-clean` — CI denies warnings.
- `every-public-fn-tested`, `test-before-implementation` — FT-1 to FT-15 first; record each red run.

### Overlap with other open work

See Design → "Overlap with other open work" (taken 2026-11-25 from `gh pr list --state open --json
number,title,files`). `git branch -r` lists 84 remote branches besides `main`; diffing each from
its merge-base, 41 touch a file this spec touches (a loop of `git diff --name-only
origin/main...$b` filtered to this spec's files) (for example `feat/spec-157-build-destinations`:
`CHANGELOG.md`, `docs/cli-reference.md`, `SKILL.md`, `src/cli/mod.rs`), and none has an open PR
(the open-PR list holds only #237), so they are leftovers, not live work). **PR #237 overlaps on `CHANGELOG.md` only.** The build re-runs both commands at Step 0,
**rebases onto `main` before each push**, re-checks the map after each rebase, and records any file
the map missed.

### Prior related work

- `SPEC-145` (shipped) — `crop`, `aspect_size`, the gravity refusal this key copies.
- `SPEC-151` (shipped) — `pad`, `aspect_canvas`, the border/gravity refusal.
- `SPEC-153` (shipped) — `edit --crop`, `--crop-gravity`, `--save-recipe`'s replay test shape.
- `SPEC-160` (shipped) — `edit --pad`, `--pad-gravity`, `PAD_FLAGS`, `FlagWorded`.
- `SPEC-161` (framed) — the page; its design reads "The page port" above.

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- The page emitting the key, `recipe.js`'s port of the rule, and the preset/swap UI: SPEC-161 and
  STAGE-060's "crop mode" entry.
- Named presets (`"5x7"`, `"square"`) in the recipe format: declined (settled at frame).
- Forcing values (`"landscape"`, `"portrait"`): declined at frame; a later value is its own entry.
- The key on `trim`, `rotate` or straighten: none has an aspect.
- The three items handed in from PROJ-011 (`format = "bogus"`, `quality = 101`, wasm `validate()`).
- A wasm `ImageInfo.orientation` getter.
- What version `edit --save-recipe` writes (`version = "1"` today): not this spec's.
- Refactoring the gravity flags' inline param-append code: optional, and only if no message moves.

### Mutation table

Every row **driven** on the design prototype: the mutant edit applied to the prototype (which
passes the full suite clean), then `cargo test --release --no-fail-fast --lib --test
aspect_orientation --test crop --test pad --test edit_geometry --test edit_pad_trim --test
recipe_corpus_output --test recipe_round_trip`, the red set read off that run, then reverted. The
baseline M0 (no edit) is all green. "Green" means every other test in those suites. FT numbers
are in Failing Tests. The parity rows (FT-19, FT-20) were not run at design (no wasm build); their
red sets are **reasoned** from the native sizes, which were driven (26/26 against the model).

- **M0 baseline** — no edit. Red: none. Driven.
- **M1 square taken as landscape** — `src/operation/mod.rs`, `follow_orientation`: `if w > h {` →
  `if w >= h {`. Red: FT-1, FT-2, FT-3 (the 48×48 `9:16` row). Green: all else, including FT-13
  (its square row writes `3:2`, already in landscape order: a row whose aspect is in the mutant's
  order cannot fail). Driven. Parity (driven by verify, corrected at ship): `O-crop-S-on-9:16`, `O-pad-S-on-9:16` red;
  `S-on-16:9` cannot catch it. Native and wasm both mutated, and wasm only, each give exactly those two rows.
  In both-sided mode only the literal sizes catch it. FT-13 note (driven by verify, corrected at ship): on a
  48×48 input `--crop 3:2` gives 48×32 both mutated and clean, and a square row written `2:3` would catch it.
- **M2 swap inverted** — same fn: the `w > h` arm returns `(short, long)` and the `w < h` arm
  `(long, short)`. Red: FT-1, FT-2, FT-3, FT-6, FT-11, FT-13, FT-14. Driven.
- **M3 crop skips the helper** — `src/operation/crop.rs`, `apply`: `let (a, b) = if follow {` →
  `let (a, b) = if follow && false {`. Red: FT-2, FT-6, FT-11, FT-13, FT-14. Green: FT-3 (pad
  still follows). Driven.
- **M4 pad skips the helper** — `src/operation/pad.rs`, `layout`: the same edit. Red: FT-3, FT-6,
  FT-11, FT-13. Green: FT-2, FT-14 (crop only). Driven.
- **M5 the key loads but is never read (negative control)** — both ops: `let follow =
  params.contains("orientation");` → `… && false;`. Red: FT-2, FT-3, FT-6, FT-11, FT-13, FT-14.
  Green: FT-4, FT-5, FT-7 to FT-10, FT-12, FT-15. Driven. Parity (driven by build, native only, corrected at ship): exactly N3's eight.
- **M6 box-form refusal removed** — `crop.rs`: `if params.contains("orientation") {` (the
  rectangle refusal) → `if params.contains("orientation") && false {`. Red: FT-7. Driven. (With it
  gone a rectangle crop silently ignores the key; the key alone falls to "crop needs a rectangle".)
- **M7 border-form refusal removed** — `pad.rs`: `if has_border && params.contains("orientation")
  {` → `… && false {`. Red: FT-8. Driven.
- **M8 unknown value accepted** — both tables: `kind: ParamKind::OneOf(ORIENTATION_NAMES)` →
  `kind: ParamKind::Text`. Red: FT-7, FT-8, FT-10, FT-12, FT-18 (all three registry tests). Driven.
- **M9 `--save-recipe` writes the key when the flag wasn't typed** — `src/cli/ops.rs`, crop aspect
  branch: `if let Some(o) = &flags.crop_orientation { params = with_text(params, "orientation",
  o); }` → `params = with_text(params, "orientation", "match");`. Red: FT-12 (a typed bad value is
  overwritten), FT-13 (the "off" text gains the key). Driven.
- **M10 `--save-recipe` drops the key when typed** — `ops.rs`, `fn built`: `StepRecord::Built(op.
  params())` → the same params with `"orientation"` filtered out. Red: FT-13, FT-14. Driven.
- **M11 `--crop-orientation` with a pixel `--crop` accepted** — `ops.rs`: `if
  flags.crop_orientation.is_some() {` → `… && false {`. Red: FT-12. Driven.
- **M12 `--pad-orientation` with a border `--pad N` accepted** — `ops.rs`: `if
  flags.pad_orientation.is_some() {` → `… && false {`. Red: FT-12. Driven.
- **M13 swap only when portrait** (the frame's "swap instead of normalize") — `follow_orientation`
  body → `if w < h { (b, a) } else { (a, b) }`. Red: FT-1, FT-2, FT-3 (landscape `9:16`, portrait
  `9:16`). Driven. Parity (reasoned): `O-*-W-on-9:16` red.
- **M14 a one-pixel tolerance** — `if w > h {` → `if w > h + 1 {` and `else if w < h {` → `else if
  w + 1 < h {`. Red: FT-1, FT-2, FT-3 (the 48×49 and 49×48 rows). Driven. Parity (reasoned):
  `O-*-N-on-16:9` red.
- **M15 square taken as portrait** — `} else if w < h {` → `} else if w <= h {`. Red: FT-1, FT-2,
  FT-3, FT-11, FT-13 (square rows written `16:9`/`3:2`). Driven. Parity (reasoned):
  `O-*-S-on-16:9` red.
- **M16 trap 14: `main`'s `src/operation/` and `src/cli/` restored verbatim** (`mod.rs`, `crop.rs`,
  `pad.rs`, `registry.rs`, `cli/mod.rs`, `cli/ops.rs`; tests kept). The unit tests compile (FT-1
  goes with `mod.rs`; the registry tests revert with `registry.rs`; lib 623 passed). Red: FT-2,
  FT-3, FT-5 to FT-15 (13 of the suite's 14), FT-16, FT-17 (both `KEYS` readers). Green: FT-4 only
  (a check on the test's own table, by design). Driven.
- **M17 the six moved expectations restored to `main`'s text** (`tests/crop.rs`, `tests/pad.rs`,
  `registry.rs` from `main`, code kept). Red: FT-16, FT-17 (`every_pad_refusal_names_its_reason`
  and `apply_refuses_a_pixel_width_key_and_writes_nothing`, both read `const KEYS`), FT-18 (all
  three). Driven. This is correction 1's evidence.

Parity cells of M13, M14 and M15 stay reasoned: no session drove them (corrected at ship).

## Notes for the Implementer

- **Don't name the new suite `tests/orientation.rs`**: that file exists (SPEC-110). Use
  `tests/aspect_orientation.rs`.
- Order matters twice: in each op the `orientation` form refusal goes **after** the `gravity` one,
  and in `build_edit_ops` the `--crop-orientation`/`--pad-orientation` refusal goes **after** the
  gravity flag's. FT-7, FT-8 and FT-12 pin the gravity message winning.
- `pad`'s table row goes **after `gravity`**, `crop`'s **last**; the unknown-key messages FT-16 and
  FT-17 pin that order.
- `follow_orientation` must get the **image** size the step receives (`iw, ih` in `crop::apply`;
  `iw, ih` in `pad::layout`, not the `u64` copies), and must run **before** `aspect_size` /
  `aspect_canvas`, which stay byte-for-byte as they are.
- The `"match"` value is never read: `params.contains("orientation")` is enough once the table has
  passed. Don't parse it a second time.
- `with_text` (or whatever the build calls it) must keep `OperationParams`' sorted order: build it
  from a `BTreeMap` via `from_map`, as the gravity code does, so the saved TOML lists keys
  alphabetically (`aspect`, `color`, `orientation`) — FT-13's literal texts depend on that.
- Parity: run `just ddphotos-parity` (both modes) locally; it needs `node`, `wasm-pack` and a Python
  with Pillow. Paste the summary lines (`N rows, 0 row failures, …` and every `PASS N3 …`) into Build
  Completion. The ddphotos-parity CI job is advisory; a local green is the gate.
- **Checkpoint:** commit and push after (1) the op change + FT-1 to FT-11 green, (2) the flags +
  FT-12 to FT-15 green, (3) the parity rows + fixtures, (4) docs + CHANGELOG. Open the PR as a draft
  after (1).
- CI legs to read before calling it green: the full matrix (Linux, macOS, Windows), the lean
  `--no-default-features` leg, clippy/fmt, wasm-check, and the advisory ddphotos-parity job.
- In an interactive terminal `cargo test` fails `display_sink_refuses_non_tty`; redirect stdout to
  a file and read `$?`, never a pipe's.
- Comments and messages read behaviour-first: no spec, stage or DEC id in code or output.

---

## Build Completion

- **Branch:** `feat/spec-163-aspect-follows-orientation`
- **PR (if applicable):** #272 (draft)
- **All acceptance criteria met?** yes (AC-1 to AC-18; the CI legs of AC-18 are reported in the build report)
- **New decisions emitted:** none (DEC-112's rule 8 follows at ship)
- **Files this diff touches** (21 from `git diff --name-only origin/main...HEAD`, plus this spec file = 22):
  - `src/operation/mod.rs` — private `ORIENTATION_NAMES` and `follow_orientation`; FT-1
  - `src/operation/crop.rs` — `orientation` table row (last), `follow` on the aspect form, rectangle refusal after gravity's, helper call before `aspect_size`
  - `src/operation/pad.rs` — `orientation` row after `gravity`, `follow`, border refusal after gravity's, helper call before `aspect_canvas`
  - `src/operation/registry.rs` — tests only: the three table tests list the new row (FT-18)
  - `src/cli/mod.rs` — `--crop-orientation` / `--pad-orientation` (`requires`), passed through `dispatch`
  - `src/cli/ops.rs` — `EditFlags` fields, `with_text`, `PAD_FLAGS` entry, key swap for crop, the two exit-2 refusals after the gravity ones
  - `tests/aspect_orientation.rs` — new suite, FT-2 to FT-15 (14 tests)
  - `tests/crop.rs`, `tests/pad.rs` — one expected unknown-key list each (FT-16, FT-17)
  - `tests/ddphotos_parity.mjs` — `INPUTS` gains P, S, N; `orientationRows` (26 rows); self-test N3
  - `tests/fixtures/ddphotos/photo_portrait.webp`, `photo_square.webp`, `photo_near_square.webp` — new, made with ImageMagick
  - `tests/fixtures/ddphotos/README.md` — three table rows and the commands
  - `docs/recipes.md`, `recipes/README.md`, `docs/data-model.md`, `docs/api-contract.md`, `docs/cli-reference.md`, `skills/crustyimg/SKILL.md` — the key and the flags
  - `CHANGELOG.md` — the two Added entries, verbatim
  - this spec file
- **Deviations from spec:**
  - FT-1 has one extra row, `(1,1,48,48)`. FT-13 and others match the spec's literals. No other deviation; `src/wasm.rs`, `scripts/wasm-ci-check.sh`, `tests/wasm_roundtrip.rs`, `Cargo.toml`, `Cargo.lock`, `CACHE_SCHEMA_VERSION` untouched.
  - The prompt's path `projects/.../SPEC-163-build.md` "on main" is not on `main`; it is on this branch (commit eaf76c3).
    - (corrected at ship) The build prompt was on `main` from `eaf76c3`; the line above is kept as history.
- **Follow-up work identified:** none.

**RED baseline (stage 1, `main`'s code):** the new suite 1 passed (FT-4, the table's own check) and 9 failed (FT-2/3, 5 to 11), every recipe row refused with `unknown key 'orientation'; expected one of: …` (exit 1); `tests/crop.rs` 1 red and `tests/pad.rs` 2 red (the moved lists); FT-1 and the registry tests did not compile (no helper, no `ORIENTATION_NAMES`). No row disagreed.

**Behaviour sentences, driven with the release binary (exit codes quoted):**
1. portrait turned: P `16:9` crop 900×1600 exit 0; `apply` folder L/P/S/N 1600×900, 900×1600, 1071×602, 603×1072 exit 0; `edit` folder 1600×1067, 1067×1600, 1071×714, 715×1072 exit 0.
2. landscape `9:16` → 1600×900 exit 0; folder exit 0 (L 1600×900, P 900×1600, S 602×1071, N 603×1072).
3. square: `16:9` → 1071×602, `9:16` → 602×1071, exit 0; folder exit 0.
4. N `16:9` → 603×1072 exit 0.
5. `1:1` on/off, L/P/S all 1071×1071, exit 0.
6. R without `auto-orient` 1600×900; with 900×1600; `edit` 900×1600; all exit 0.
7. rectangle crop and border pad with the key: exit 1 with messages 1 and 5, one input and folder; no output directory.
8. `orientation = "landscape"`: exit 1, message 2, one input and folder, no output directory.
9. pixel `--crop` / border `--pad N` with the flag: exit 2, messages 9 and 10, one input and folder, no output directory.
10. `--crop-orientation` without `--crop`: exit 2 (clap), one input and folder.
11. `--save-recipe`: key present when typed, absent when not (both texts quoted by FT-13); folder `edit` saves one recipe with the key, exit 0.

**Parity (`just ddphotos-parity`, both modes):** `16 rows, 0 row failures, 0 table failures, K=1 (cap 78)` (the 26 orientation rows add no failure; exit 0). Self-test: `PASS N1`, `PASS N2` and
`PASS N3 the orientation key stripped: red rows {O-crop-N-on-16:9,O-crop-P-on-16:9,O-crop-R+ao-on-16:9,O-crop-W-on-9:16,O-pad-N-on-16:9,O-pad-P-on-16:9,O-pad-R+ao-on-16:9,O-pad-W-on-9:16}, expected` the same eight. All 26 reasoned literals were right; no row disagreed.

**Mutations (scratch worktree, own target dir, clean rebuild each, full suite `--no-fail-fast`, red set read off the run):** M0 green. Every red set equals the spec's table, with no extra red test and no mismatched cell:
- M1: FT-1, FT-2, FT-3. M2: FT-1, 2, 3, 6, 11, 13, 14. M3: FT-2, 6, 11, 13, 14. M4: FT-3, 6, 11, 13.
- M5 (negative control): FT-2, 3, 6, 11, 13, 14; FT-4, 5, 7 to 10, 12, 15 green. M6: FT-7. M7: FT-8.
- M8: FT-7, 8, 10, 12 and all three registry tests. M9: FT-12, 13. M10: FT-13, 14. M11, M12: FT-12.
- M13, M14: FT-1, 2, 3. M15: FT-1, 2, 3, 11, 13.
- M16 (trap 14): lib unit tests compile and pass; the new suite is 1 passed (FT-4 only) and 13 red; `tests/crop.rs` 1 and `tests/pad.rs` 2 red.
- M17: crop 1, pad 2, registry 3.
- Parity: M5 applied to the native binary only (release build, unmutated wasm package): exactly N3's eight rows red. Other mutants' parity cells were not driven (each needs a release rebuild); their rows stay reasoned.

**Matrix:** default, `--no-default-features`, `--features webp-lossy`, each in a fresh `CARGO_TARGET_DIR`: test, `clippy -D warnings`, `fmt --check` all exit 0. `just wasm-check` exit 0; `just wasm-test` exit 0 (56 passed). Fixtures: `git diff --name-status origin/main -- tests/fixtures/` shows three `A` and `M tests/fixtures/ddphotos/README.md`; no bless run.

**Overlap after the last rebase:** open PRs are #237 (parked; overlaps `CHANGELOG.md` only) and this one. Nothing new.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Little. The prompt cites its own path "on main", but the file is only on the branch.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — clippy's `type_complexity` on the test row tables; avoided by letting the compiler infer the array types.

3. **If you did this task again, what would you do differently?**
   — Run the mutation script against a pre-warmed target dir; its first build cost most of the wall time.

No design call turned out wrong.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Three things, none a behaviour defect; verify approved with no punch items. (a) The design's parity
   cells were reasoned, build drove only M5 (native only), and verify drove the one wasm-side mutant: M1
   in both sides passes the native-against-wasm comparison, and only the literal sizes catch it
   (`O-crop-S-on-9:16`, `O-pad-S-on-9:16`). A spec that adds parity rows should drive one mutant into
   both sides and one into wasm only, at design or build. M13, M14 and M15's parity cells were never
   driven and stay reasoned. (b) The two pushes went wrong in different ways: design's stopped at a
   permission check and the maintainer pushed by hand, and build's Build Completion says the build
   prompt "is not on `main`" when it was there from `eaf76c3`. A prompt should say what to do when a
   push is refused (commit, stop, name the commits and the command) and should say "check
   `git ls-tree origin/main` before reporting a file missing". (c) Build measured its cost before its
   last commit, so ship re-measured it ($5.50 to $6.15); design's was taken before its blocked push
   ($12.37 to $12.94). Build also didn't record the space it freed. What I think made this the first
   PROJ-017 spec with no punch list since SPEC-152, from the evidence only: design drove 17 mutants on
   a prototype before build, and build's RED baseline and M0 to M17 red sets all matched the table
   with no extra red (SPEC-160's four missed reds were absent); the mutation table named the exact
   edit per row; the parity rows were written as literal sizes, so the comparison could not agree with
   itself; and verify's independent model over 9324 rows found 0 mismatches. I cannot separate these
   from the spec being small and having no wasm change.

2. **Does any template, constraint, or decision need updating?**
   — Proposed, not applied. *(Applied 2026-11-27 by the maintainer's approval: the parity-mutant, both-orders and refused-push rules are PROJ-011 HANDOFF traps 19–21, and trap 13 now covers a design's mutation table.)* (i) "A parity row's literal size is the check that catches a mutant both
   sides share; a spec that adds parity rows drives one mutant into both sides." Not covered: AGENTS
   §15 (about line 685) covers undriven bounds and all-green rows, and SPEC-159's and SPEC-160's
   reflections speak of the wasm leg, none of the shared-mutant case. Verify's M1 run is its first
   evidence. (ii) "A test row whose aspect is already in the order a mutant produces cannot catch it;
   write each square row in both orders." Not covered; the first sighting was design's M1 note on
   FT-13, now also in `aspect-orientation-loose-ends`. (iii) SPEC-160's proposals (comments swept with
   DECs, red set read off every target, separator normalisation): this spec's evidence is for the
   second, which held (M0 to M17 read off the full suite, no extra red). The first and third were not
   exercised. SPEC-152's and SPEC-153's "driven, not reasoned" point is borne out again: the one cell
   build left reasoned for wasm was the one verify drove. (iv) What a prompt should say when a push is
   refused: AGENTS and `guidance/` are silent; the ship prompt carries it ad hoc ("stop and report,
   don't claim the push happened"). I couldn't check HANDOFF (the repo tracks none). No DEC changes
   beyond DEC-112's rule 8, applied as written.

3. **Is there a follow-up spec I should write now before I forget?**
   — None. `aspect-orientation-loose-ends` in `guidance/ideas.yaml` already carries verify's cleanups
   2 to 4 (including the square-row note); nothing in the evidence names another gap. The page port is
   SPEC-161's, already in STAGE-060.
