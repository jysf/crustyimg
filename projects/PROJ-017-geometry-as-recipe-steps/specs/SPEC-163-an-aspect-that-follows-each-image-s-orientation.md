---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-163
  type: story                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
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
  constraints: []
  related_specs:
    - SPEC-145
    - SPEC-151
    - SPEC-153
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
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
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

### Findings outside this spec

None filed. (Load errors number steps from 0, `recipe step 0 ('crop')`, which is existing and
consistent behaviour, not a finding.)

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

### Overlap with other open work

Design writes this map from the other work's actual file lists, never from their specs or from
memory: `gh pr list --state open --json number,title,files` and `git branch -r` (diff each pushed
feature branch against `main`). Record the date and time it was taken. Open PRs keep changing, so
the build re-runs both commands at its Step 0 and records any file the map missed.

- `PR #NNN` (`<branch>`) — touches `<file>` that this spec also touches; <how they meet>
- (or: none, as of <date time>)

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

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
