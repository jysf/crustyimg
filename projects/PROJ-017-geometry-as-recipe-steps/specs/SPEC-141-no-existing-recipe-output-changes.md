---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-141
  type: task                       # epic | story | task | bug | chore
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
  implementer: null
  created_at: 2026-10-30

references:
  decisions:
    - DEC-058   # the cache key: canonical recipe TOML is one of its seven inputs
    - DEC-059   # the lockfile pins the key, records output hashes per env; rejects cross-arch byte pins
    - DEC-094   # AVIF tile count pinned; the encoder is deterministic per machine, not across machines
    - DEC-095   # ops preserve colour type and bit depth; the guard records both
    - DEC-104   # watermark refuses a mark that doesn't fit or covers >5%; constrains the fixtures
    - DEC-008   # Lanczos3 via fast_image_resize; the frame's negative-control mutation
  constraints:
    - test-before-implementation
    - single-image-library
  related_specs:
    - SPEC-131  # wrote the corpus and `to_toml_is_byte_identical_to_main_over_the_corpus` (shipped)
    - SPEC-097  # wrote `scripts/cli-golden.py`, the before/after binary differential (shipped)
    - SPEC-122  # linear-light resize; the float path whose cross-leg stability is unproven (shipped)
    - SPEC-124  # pinned AVIF tile count (shipped)
    - SPEC-142  # native-vs-wasm parity harness (PROJ-017 line 3), framed beside this one

value_link: >
  Makes STAGE-058's "the existing recipe corpus gives byte-identical output before and after
  the stage" and PROJ-017's fourth success signal a test that runs on every PR, instead of a claim.

cost:
  sessions:
    - cycle: frame
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 5253953
      tokens_breakdown:
        input: 92
        output: 34538
        cache_creation: 123761
        cache_read: 5095562
      estimated_usd: 4.19
      duration_minutes: null
      recorded_at: 2026-10-30
      note: >
        Interactive frame session in its own worktree (docs/spec-141-frame). Measured from the
        session's transcript (8c652873-….jsonl, identified by content), deduped by `.message.id`
        (46 unique ids, all claude-opus-5-5), input/cache_creation/cache_read once per id, MAX
        output_tokens per id. Priced by component at Opus $5/$25 per MTok, cache_creation x1.25
        ($6.25), cache_read x0.10 ($0.50): $0.0005 + $0.77 + $2.55 + $0.86. Measured before
        the commit/push turns, so it slightly under-counts.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-141: No existing recipe's output changes: the PROJ-017 guard

## Context

PROJ-017 adds five new geometry ops (`crop`, `rotate`, `flip`, `pad`, `trim`). Its fourth
success signal is that no existing recipe's output changes. This spec is line 1 of 13 in the
build order, and the first item in STAGE-058's Spec Backlog. Lines 2 (`crop`) and 4
(`rotate`/`flip`) wait on it.

What is pinned today, measured at frame on `8ccc2d1`:

- **The recipe's canonical TOML is pinned.** `tests/recipe_round_trip.rs:319`
  (`to_toml_is_byte_identical_to_main_over_the_corpus`, from SPEC-131) compares `to_toml()`
  for 10 corpus recipes and 3 bundled recipes against committed `.golden` files. Under
  DEC-058/DEC-059 the lockfile pins the **cache key**, and canonical recipe TOML is one of the
  key's seven inputs. So the lockfile half of the success signal ("lockfiles stay valid") is
  **already guarded**.
- **What a recipe produces is not pinned anywhere.** That test calls `build_pipeline` (`:364`)
  but never runs the pipeline. No test runs a corpus recipe. The bundled recipes are run by
  `tests/build.rs:532` on a 64×64 input (`:501`), so their 2048/2560/1600 `max` resize never
  fires. `tests/wasm_roundtrip.rs:161` runs them on 1800×1200, so only `product` resizes. Both
  assert format or same-machine equality, never a pinned value.
- **`scripts/cli-golden.py` (SPEC-097) is a before/after tool, not a gate.** It is referenced in
  no `justfile` recipe and no workflow. Its oracle goes to gitignored `target/cli-golden`. It
  runs only two recipes: an inline `max 16` resize (`:83`) and bundled `web` on 64×48
  (`:273`), which never resizes.

So a change that alters what an existing recipe draws, without touching its TOML, passes every
test in the repo today. The frame drove exactly that change (see Frame, premise check).

## Goal

A `cargo test` integration test runs every existing recipe on in-test-generated fixtures and
compares each output against a committed, per-recipe golden, so a PROJ-017 PR that changes any
existing recipe's output goes red in CI on every leg. A PROJ-017 builder runs
`cargo test --test recipe_corpus_output` and, on a regression, sees a failure that names the
recipe, the fixture, and the field that moved, for example
`v1_resize_max × rgb8_2600x1730: pixels changed (dims 800x532 unchanged)`.

## Frame

### Verdict: **GO**, [S]

The premise holds. The gap is a single test file plus goldens, built on the existing corpus.
Nothing needs a DEC, because DEC-059 already chose what may be pinned across machines. The one
real risk is measurement, not code: nobody has shown that resize's float path gives the same
pixels on all three CI legs. Design must measure that first (see "Pin rule").

### Premise check (release build of `main` @ `8ccc2d1`, fresh `CARGO_TARGET_DIR`, deleted after)

```
cargo build --release                                        # exit 0
cargo test --test recipe_round_trip                          # 10 passed, 1 ignored, exit 0
python3 scripts/cli-golden.py capture --bin <main binary>    # 27 cases, exit 0
python3 scripts/cli-golden.py check   --bin <main binary>    # 27/27 byte-identical, exit 0
```

**Negative control, driven at frame** (uncommitted; reverted with `git checkout`, tree clean
afterwards). Mutation: `src/operation/mod.rs:584`, `FilterType::Lanczos3` → `CatmullRom`, the
filter every resize uses.

| pin | result under the mutation |
|---|---|
| `recipe_round_trip` corpus test | **green**: 10 passed. It cannot see a pixel change. |
| `cli-golden.py check` | red on `resize`, `thumbnail_square`, `apply_recipe_plain`, `build` |
| `cli-golden.py`, case `apply_web_json` (bundled `web`) | **green**: 64×48 is under 2048, so the resize never ran |

A fifth case, `resize_usage_error`, also went red, but that's an artifact: clap prints argv[0],
and the oracle binary was named `crustyimg-main` (5 bytes longer). **`cli-golden.py` needs the
same binary filename for `capture` and `check`.**

So the premise holds, and so does question 4's warning: a fixture under the recipe's bound
makes the step a no-op, and a pin over a no-op passes under any change to that step.

### Inventory: every recipe that exists today

| source | recipes | run by a test? | output pinned? |
|---|---|---|---|
| `tests/fixtures/recipe_corpus/` | 10: `v1_resize_{cover,exact,fill,fit,max,percent}`, `v1_watermark_{image_placement,text_font}`, `v2_format_quality_optimize`, `v2_watermark_integer_numbers` | **no**: parsed, serialized, `build_pipeline`'d only (`tests/recipe_round_trip.rs:319-366`) | TOML only. Pixels: **gap** |
| bundled (`recipes/{web,gallery,product}.toml`, `include_str!` at `src/recipe/bundled.rs:42-50`, handed out by `resolve` `:56`) | 3, each `auto-orient` → `resize max` → `optimize` | yes, at sizes that skip the resize: `tests/build.rs:532` (64×64), `tests/wasm_roundtrip.rs:161` (1800×1200; only `product` resizes), `cli-golden.py:273` (64×48, `web` only, not in CI) | TOML only. Output: **gap** |
| `docs/recipes.md`, `README.md` | **none** (0 TOML blocks, 0 `op =` lines) | — | — |
| `docs/cli-reference.md:344` | none. Its one TOML block is a build manifest naming `recipes/web.toml`, i.e. bundled `web` | — | — |
| build fixtures | no recipe files. Manifests are written inline in tests and point at bundled names or inline `r.toml` | see bundled | — |
| ddphotos page (`integrations/ddphotos/edit/edit.js:266` `pixelRecipe`, `:288` `cliRecipe`) | generated: v2, `auto-orient` → optional `resize max` → optional text `watermark`, plus `format` and terminal `optimize` in `cliRecipe` | parity owned by SPEC-142 (native vs wasm), not a pinned output | **gap**. No corpus entry has this shape (text watermark *after* a resize, with a gravity) |
| agent skill (`skills/crustyimg/SKILL.md:210`, `:233`) | 2: `avatar` (`resize fill 256×256` + `optimize`), and a v2 `webp`/`quality 80` resize + image watermark | no | **gap**, but each is covered by shape (`v1_resize_fill`; `v2_watermark_integer_numbers` + `v2_format_quality_optimize`) |
| inline test recipes (`r.toml` strings in `tests/*.rs`) | many | yes, with assertions local to each test | not a corpus. They test behaviour, not stability. Out |

Gaps: all 10 corpus recipes, all 3 bundled recipes, and the ddphotos shape. The two skill
recipes are covered by shape.

### Pin rule per format

DEC-059 already decided this for the lockfile. It rejected pinning encoded output hashes across
machines ("Option A", because the encoders promise run-to-run determinism on one machine, not
across arch, OS or codec patch). The guard follows that ruling, separating **what the recipe
draws** from **how the encoder writes it**. PROJ-017 changes the first and never the second.

| what | pin | why |
|---|---|---|
| **Pipeline output, every row** (the `Image` after the last pixel step, before encode) | width, height, colour type, bit depth, SHA-256 of the raw samples | This is what PROJ-017 can change. It is format-independent. |
| **PNG** encoded bytes | SHA-256 of the file | `image`'s PNG encoder is pure Rust and integer-only. |
| **Lossless WebP** encoded bytes (`image-webp`) | SHA-256 of the file | Pure Rust, integer-only. |
| **JPEG** | none needed | No corpus or bundled recipe writes JPEG. If design adds a JPEG row, it gets the same pixel pin and no byte pin until the CI legs are shown to agree. |
| **AVIF (rav1e) and the `optimize` search** | **no byte pin.** Pin the pre-encode pixels, the format `optimize` chose, and the output dimensions | DEC-059 Option A. rav1e is built without asm (`Cargo.toml:267`), so the per-CPU assembly worry doesn't apply, but its rate control and the ssimulacra2 quality search use `f64` maths with platform `libm`. DEC-058/059's experiment proved stability per machine only, and no test in the repo pins an AVIF hash. |

⚠ **The load-bearing unknown: resize pixels across legs.** Resize runs in linear light: `f32`
`powf` (`src/operation/mod.rs:520,529`, which calls the platform's `libm`) and
`fast_image_resize`'s `F32x4` convolution, which picks AVX2/SSE4.1 on the x86 legs and NEON
on macOS arm64. The repo's only cross-leg pixel-hash constant is resvg's
(`tests/svg_font_family.rs:20`). Nothing pins resize pixels. **Design measures this first**:
push the pins on a draft PR and read all three legs (Linux x86_64, macOS arm64, Windows
x86_64; `.github/workflows/ci.yml:72`). If they agree, there is one constant per row. If a leg
differs, the pin becomes a per-`(arch, os)` table, still **exact** on each leg, blessed from
that leg's CI output. Never add a tolerance, and never drop the row. (Maintainer question 1.)

### Where it lives

**The Rust integration test**: a new `tests/recipe_corpus_output.rs`, reusing
`recipe_round_trip.rs`'s `CORPUS`/`BUNDLED` lists and its asset stand-ins. `cargo test`
already runs on every CI leg, so the guard runs on every PR without anyone remembering it.
**Not `cli-golden.py`**: it is not in CI, its oracle is uncommitted by design (a
same-machine before/after tool), and wiring it in means a second release build per PR. It
stays as SPEC-097 left it.

### Fixtures

Generate them in the test as raw sample buffers handed straight to `Image`, so no decoder or
encoder under test produces an input. Each needs a non-flat, seeded pattern (a gradient plus
noise, like `cli-golden.py`'s `make_png`), so that resampling or compositing changes the
hash.

| fixture | why |
|---|---|
| `rgb8_2600x1730` | over every `max` bound (2560, 2048, 1600, 1024, 800) and every 640×480 box, so every resize fires |
| `rgba8_1100x700` | alpha through resize and watermark compositing; over 1024 and 640×480 |
| `rgba16_1100x700` | DEC-095's depth preservation; the 16-bit path calls `srgb_to_linear` per sample, not the 8-bit table |

**Assets:** the image watermark gets a seeded 64×32 RGBA logo built in the test. The text
watermark uses `crustyimg::text::DEFAULT_FONT`, as the corpus test already does. Every
watermark row must be **accepted** on its fixture. DEC-104 refuses a mark that does not fit or
that covers over 5% of the image, and the tiled `v1_watermark_image_placement` is the row most
likely to hit the coverage rule. Design checks each watermark × fixture pair. A refused row
counts as red, never as a skip (HANDOFF trap 11).

**Matrix:** 14 recipes (10 corpus, 3 bundled, plus the ddphotos shape) × 3 fixtures = **42
pixel rows**. Encoded-byte rows are only the PNG and lossless-WebP terminals (single digits).
The AVIF `optimize` encode is the debug-test cost risk: at 2560 px in a debug build it can take
seconds per row. Design times it, and may run the terminal encode only on the 1100×700
fixtures, with the 2600×1730 row pinned at the pixel stage.

### It must be able to fail

Design requires these controls. The build drives each on a scratch copy and records the red
output.

1. **Resize filter** `Lanczos3` → `CatmullRom` (`src/operation/mod.rs:584`). Driven at frame
   against today's pins (above). On the guard, it must go red on every resizing row, naming
   each recipe, and stay green on the watermark-only, `identity` and `invert` rows. That second
   half shows the guard is specific, not just broken.
2. **A watermark default** (e.g. the default margin or opacity). It must go red on the
   watermark rows only.
3. **Positive control: "runs no steps".** Make the pipeline skip its steps. It must go red,
   because every resizing row also asserts its expected **dimensions** as a literal, and every
   watermark row asserts its output hash ≠ the fixture's input hash. A guard that ran nothing
   cannot pass.
4. **The corpus list.** The new test asserts its row count (as `recipe_round_trip.rs:329-332` already does), so a
   recipe dropped from the list goes red.

Per HANDOFF trap 12 (added the day of this frame), design names the exact code location for
controls 2 and 3 and lists every row expected red **and** every row expected green. The guard has
one path (in-process library), so there is no second path to cover.

### Blessing

Extend the existing switch rather than adding a second one:
`CRUSTYIMG_BLESS=1 cargo test --test recipe_corpus_output -- --ignored bless` writes **one
golden file per recipe**: `tests/fixtures/recipe_corpus/<stem>.output` (and
`bundled_<name>.output`), one line per fixture. A re-bless then shows in `git diff` as the
named recipe's file changing, and the failure message names recipe × fixture × field.

**Rule for every PROJ-017 spec:** a changed `*.output` file in a PROJ-017 PR is a **verify
finding**, unless that spec's acceptance criteria name the recipe and say why its output
changes. Adding a **new** `.output` row (a new op's own corpus entry) is not a re-bless. Verify
checks this with `git diff --name-only origin/main -- 'tests/fixtures/recipe_corpus/*.output'`,
and every pre-existing path it lists must be named in an AC.

### Sizing: [S]

One test file of roughly 250 lines, about 14 small golden files, one new corpus recipe, and a
2-line change to the existing corpus test's count. Verification: 42 pixel rows plus a few byte
rows, four controls, and **one draft-PR push to read three CI legs**. That push is the only
step whose outcome is unknown. If the legs disagree, the per-leg table adds a column and one
more push, which is still [S]. If design finds the debug AVIF encode too slow on every
fixture, it trims the encode rows, not the pixel rows.

### Scope

**In:** the 10 corpus recipes, the 3 bundled recipes, one new corpus recipe in the ddphotos
`cliRecipe` shape (`auto-orient`, `resize max`, text `watermark` with `gravity`/`size`/`margin`/
`color`/`opacity`, v2 `format`; it then also joins the TOML golden, 13 → 14 rows), three
fixtures, per-recipe `.output` goldens, the bless switch, four controls, and the
cross-leg measurement.

**Out:** see below.

### Rough build file list

- `tests/recipe_corpus_output.rs`: **new**, the guard and its `#[ignore]` bless
- `tests/fixtures/recipe_corpus/*.output`, `bundled_*.output`: **new**, about 14
- `tests/fixtures/recipe_corpus/v2_ddphotos_cli_shape.toml` + `.golden`: **new**
- `tests/recipe_round_trip.rs`: `CORPUS` gains the new stem, and the count goes 13 → 14
  (`:259`, `:332`)
- `tests/fixtures/recipe_corpus/README.md`: document `.output` and the bless rule
- Possibly `tests/common/mod.rs`: a seeded fixture generator, if one doesn't exist already

No `src/` change.

### Open maintainer questions

1. **If CI legs give different resize pixels, accept per-`(arch, os)` exact pins?** Recommend
   **yes**. It stays exact on each leg. The alternative is a tolerance, which is the "pin
   loosened until it's always green" this frame was told to avoid.
2. **Pin AVIF/`optimize` encoded bytes at all?** Recommend **no**, per DEC-059 Option A: pin
   the pre-encode pixels plus the chosen format and dimensions. Revisit only if the draft PR
   shows all three legs agree on the AVIF bytes too.
3. **Add the ddphotos `cliRecipe` shape to the corpus now?** Recommend **yes**. It is the
   recipe STAGE-060 will extend with crop and rotate, it is the only shape in use with no
   corpus entry, and it costs one file.

### Maintainer rulings (2026-10-30)
All three recommendations above were accepted:
1. If the CI legs give different pixels, pin exactly per OS and architecture. Never a tolerance.
2. No byte pins for AVIF or the `optimize` search (DEC-059 Option A).
3. Add the ddphotos-shaped recipe to the corpus now.

### Out of scope (for this spec specifically)

- **Encoded-byte pins for AVIF and lossy encoders** (see the pin rule; maintainer question 2).
- **Wiring `cli-golden.py` into CI or extending its cases.** It stays a manual refactor tool
  (SPEC-097).
- **The skill's two recipes as corpus entries.** Covered by shape. Not filed; add them only if
  a later spec changes the skill's examples.
- **Inline test recipes** (`r.toml` strings in `tests/*.rs`). They test behaviour, not
  stability.
- **Native-vs-wasm parity of the ddphotos recipes.** SPEC-142.
- **`auto-orient` with a real orientation tag.** The fixtures carry no EXIF, so `auto-orient`
  is a no-op on every row. SPEC-110 already pins orientation baking, and PROJ-017's `rotate`
  spec (line 4) owns the orientation interaction. If the in-memory `Image` can carry an
  orientation cheaply, design may add one row. Otherwise it stays out.
- **Pinning any op PROJ-017 adds.** Each new op adds its own corpus row in its own spec.

## Inputs

*(design)*

## Outputs

*(design)*

## Acceptance Criteria

*(design)*

- [ ] Criterion 1 (testable)

## Failing Tests

*(design)*

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*(design)*

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>

### Constraints that apply

- `constraint-id-1` — <one-line summary>

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>

## Notes for the Implementer

*(design)*

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
