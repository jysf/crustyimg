---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-141
  type: task                       # epic | story | task | bug | chore
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
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 6735818
      tokens_breakdown:
        input: 90
        output: 71913
        cache_creation: 189133
        cache_read: 6474682
      estimated_usd: 6.22
      duration_minutes: null
      recorded_at: 2026-10-30
      note: >
        Interactive design session in its own worktree (docs/spec-141-design). Measured from the
        session's transcript (1cdb629e-….jsonl, identified by content), deduped by `.message.id`
        (45 unique ids, all claude-opus-5-5), input/cache_creation/cache_read once per id, MAX
        output_tokens per id. Priced by component at Opus $5/$25 per MTok, cache_creation x1.25
        ($6.25), cache_read x0.10 ($0.50): $0.0004 + $1.18 + $3.24 + $1.80. Measured before
        the commit/push turns, so it slightly under-counts.
    - cycle: build
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 10792771
      tokens_breakdown:
        input: 152
        output: 49445
        cache_creation: 158838
        cache_read: 10584336
      estimated_usd: 4.51
      duration_minutes: null
      recorded_at: 2026-10-30
      note: >
        Build session in worktree test-spec-141-recipe-output-guard. Measured from the session
        transcript (84d491ec-….jsonl, identified by content), deduped by `.message.id` (73 unique
        ids, all claude-sonnet-5-5), input/cache_creation/cache_read once per id, MAX output_tokens
        per id. Sonnet $3/$15 per MTok, cache_creation x1.25, cache_read x0.10. Measured before the
        final commit/push turns, so it slightly under-counts.
    - cycle: verify
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 10590053
      tokens_breakdown:
        input: 130
        output: 47860
        cache_creation: 186613
        cache_read: 10355450
      estimated_usd: 7.54
      duration_minutes: null
      recorded_at: 2026-10-30
      note: >
        Read-only verify session; measured by the verify and recorded here by the punch-list
        session. Opus $5/$25 per MTok, cache_creation x1.25, cache_read x0.10.
    - cycle: punchlist
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 5200322
      tokens_breakdown:
        input: 92
        output: 25365
        cache_creation: 109720
        cache_read: 5065145
      estimated_usd: 2.31
      duration_minutes: null
      recorded_at: 2026-10-30
      note: >
        Punch-list session in worktree fix-spec-141-punch. Measured from its transcript
        (f6d20d96-….jsonl, identified by content), deduped by `.message.id` (43 unique ids, all
        claude-sonnet-5-5), input/cache_creation/cache_read once per id, MAX output_tokens per id.
        Sonnet $3/$15 per MTok, cache_creation x1.25, cache_read x0.10. Measured before the final
        doc commit, so it slightly under-counts.
    - cycle: verify
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 3429171
      tokens_breakdown:
        input: 62
        output: 17722
        cache_creation: 103188
        cache_read: 3308199
      estimated_usd: 2.74
      duration_minutes: null
      recorded_at: 2026-10-31
      note: >
        Re-verify of punch list 1 (read-only; measured by that session, recorded here by
        punchlist-2). Opus $5/$25 per MTok, cache_creation x1.25, cache_read x0.10.
    - cycle: punchlist-2
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 1362074
      tokens_breakdown:
        input: 36
        output: 6132
        cache_creation: 54513
        cache_read: 1301393
      estimated_usd: 0.69
      duration_minutes: null
      recorded_at: 2026-10-31
      note: >
        Punch-list-2 session in worktree fix-spec-141-punch2. Measured from its transcript
        (71542bc3-….jsonl, identified by content), deduped by `.message.id` (18 unique ids, all
        claude-sonnet-5-5), MAX output_tokens per id. Sonnet $3/$15 per MTok, cache_creation
        x1.25, cache_read x0.10. Measured before the final doc commit, so it slightly under-counts.
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

## Design

*(design, 2026-10-30, Opus.) Measured on `main` @ `a448a61` in a fresh `CARGO_TARGET_DIR`
(deleted after), with a throwaway probe test (`tests/zz_probe_141.rs`, never committed,
deleted) that used the library API exactly as the guard will. Profile: `test` (the dev
profile, `opt-level = 1`, `Cargo.toml:335-336`), which is what CI's `cargo test` runs.*

### What design measured

**1. The frame's RGBA fixtures make six rows pin a no-op.** On `rgba8_1100x700` and
`rgba16_1100x700`, all three bundled recipes' `resize max` (2560/2048/1600) left the image
unchanged (probe: `[1:resize:NOOP]`, output hash = input hash). That is six pins over a step
that never ran: the frame's own question-4 warning, and it contradicts the accepted rule
"every recipe step must actually do something on each fixture". **Design changes the two RGBA
fixtures** (see Call 1). With the new fixtures every `resize`, `watermark` and `invert` step
changed the image on every fixture (42/42 rows, each step checked one at a time). Every
watermark row was accepted (no DEC-104 refusal: the tiled and scaled marks are exempt from the
5% cap, `src/operation/mod.rs:1725`, and the two unscaled text marks are small).

The only steps that never change a fixture are `auto-orient` (the fixtures carry no
orientation; the frame put that out of scope) and `identity` (by definition). The guard exempts
exactly those two by name.

**2. Cost (dev profile, Mac).** Each pixel row took 3–362 ms; all 42 rows ≈ 7.5 s, plus
≤ 0.23 s per PNG/WebP encode. The bundled recipes' `optimize` decision is not reachable from the
library (see Call 3), so its cost was measured through the dev binary:
`apply --recipe <web|gallery|product> <fixture>.png --out-dir … --json` took 5–23 s per row, 107 s
for all nine. Every one of the nine picked **AVIF**, by a wide margin: on `rgba16`, AVIF 61,656
bytes vs lossless WebP 1,960,536 vs PNG 8,223,863 (`--json` report).

**3. The library seam writes the bytes `apply` writes.** On `rgb8_2600x1730`, the dev binary's
`apply --recipe <corpus file> rgb8.png --out-dir o` wrote `rgb8.png` with SHA-256 prefix
`a8584eba` for `v2_watermark_integer_numbers`, and `rgb8.webp` = `72991b30` for
`v2_format_quality_optimize`. The probe's `sink::encode_to_bytes(img, Png, None)` and
`encode_to_bytes(img, WebP, None)` on the in-process pipeline output gave the same two prefixes.
This also shows that `v2_format_quality_optimize`'s `quality = 80` never reaches the WebP
encoder on `apply`'s path (see Call 3).

**4. Both required mutations, driven against the probe** (applied with `sed`, reverted with
`git checkout`, tree clean afterwards):

| mutation | pixel rows changed | pixel rows unchanged |
|---|---|---|
| `src/operation/mod.rs:584` `FilterType::Lanczos3` → `CatmullRom` | **33**: the 11 recipes that resize × 3 fixtures. Their PNG/WebP encodes changed too | **9**: `v1_watermark_image_placement`, `v1_watermark_text_font`, `v2_format_quality_optimize` × 3 fixtures |
| `src/operation/mod.rs:1629` `opacity … unwrap_or(1.0)` → `unwrap_or(0.9)` | **6**: `v1_watermark_text_font` and `v2_watermark_integer_numbers` × 3 (the two recipes with a watermark step that has no `opacity` key) | **36**: every other row, including `v1_watermark_image_placement` (`opacity = 0.6`) and the ddphotos shape (`opacity = 0.85`) |

The scaled image watermark resizes its mark with `image`'s own `imageops::resize`
(`src/operation/mod.rs:1773`, `:1829`), not the `fast_image_resize` call at `:584`. That is why
`v1_watermark_image_placement` stays unchanged under the filter mutation.

### Call 1 — fixtures: three, every one over every bound

| name | colour | size | why |
|---|---|---|---|
| `rgb8_2600x1730` | `Rgb8` | 2600×1730 landscape | unchanged from the frame |
| `rgba8_1730x2600` | `Rgba8` | 1730×2600 **portrait** | was 1100×700. Long edge 2600 > 2560, so every bundled resize runs. Portrait also drives `max`/`fit`/`cover` on the height-bound branch, which no other fixture does |
| `rgba16_2600x1730` | `Rgba16` | 2600×1730 | was 1100×700, for the same reason. Keeps DEC-095's 16-bit path (`resize_and_crop16`, `src/operation/mod.rs:670`, called at `:1069`) |

**Report to the orchestrator:** this departs from an accepted frame line (the 1100×700
sizes). The accepted rule "every step must do something on each fixture" cannot hold with
them, so design kept the rule and changed the sizes. Cost: about +2 s per run.

The generator is part of the spec, so the goldens can be reproduced (copy it verbatim):

```rust
struct XorShift(u64);
impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// A seeded gradient-plus-noise image, built from raw samples (no encoder or decoder involved).
fn fixture(kind: &str, w: u32, h: u32) -> Image {
    let mut r = XorShift(0x5EED_0141 ^ (u64::from(w) << 20) ^ u64::from(h));
    let px = match kind {
        "rgb8" => DynamicImage::ImageRgb8(ImageBuffer::from_fn(w, h, |x, y| {
            let n = (r.next() & 31) as u32;
            Rgb([((x * 255 / w + n) % 256) as u8, ((y * 255 / h + n) % 256) as u8,
                 (((x + y) / 7 + n) % 256) as u8])
        })),
        "rgba8" => DynamicImage::ImageRgba8(RgbaImage::from_fn(w, h, |x, y| {
            let n = (r.next() & 31) as u32;
            Rgba([((x * 255 / w + n) % 256) as u8, ((y * 255 / h + n) % 256) as u8, 90,
                  (128 + (x + y) % 128) as u8])
        })),
        "rgba16" => DynamicImage::ImageRgba16(ImageBuffer::from_fn(w, h, |x, y| {
            let n = (r.next() & 2047) as u32;
            Rgba([((x * 65535 / w + n) % 65536) as u16, ((y * 65535 / h + n) % 65536) as u16,
                  20000, (32768 + (x * 7 + y) % 32768) as u16])
        })),
        other => panic!("unknown fixture kind {other}"),
    };
    Image::from_parts(px, ImageFormat::Png, None)
}
```

SHA-256 of `pixels().as_bytes()`, as measured at design (first 8 hex digits). The build must
reproduce them, which proves it copied the generator: `rgb8_2600x1730` `94daa4af…`,
`rgba8_1730x2600` `7df0cef0…`, `rgba16_2600x1730` `75dfcd86…`. The test pins all 64 digits.

**Assets.** The image watermark's `image` key gets a seeded 64×32 RGBA logo, encoded to PNG
bytes in the test (`XorShift(0x1060)`, each pixel `[v, v>>8, v>>16, 128 | v>>24]` as `u8`, from
one `next()` per pixel). This asset is an input to the watermark step, not a fixture, and the
watermark decodes it. A `font` key gets `crustyimg::text::DEFAULT_FONT`. Both go in through
`OperationParams::set_resolved_bytes`, as `tests/recipe_round_trip.rs:354-362` already does
(the DEC-100 seam).

### Call 2 — the new corpus recipe: `v2_ddphotos_cli_shape.toml`

The exact shape `cliRecipe` builds (`integrations/ddphotos/edit/edit.js:266-292`) for a
1000-px output at the page's defaults: format `jpeg` (selected by default,
`integrations/ddphotos/edit/index.html:119`), size `round(1000 × 0.04)` = 40
(`edit.js:227`), and margin `round(1000 × 0.02)` = 20 (`edit.js:246`). Byte for byte:

```toml
# The ddphotos edit page's CLI recipe: auto-orient, a long-edge resize, then a
# text watermark placed with gravity, size, margin, colour and opacity, a
# pinned format and the terminal optimize step.
version = "2"
format = "jpeg"

[[step]]
op = "auto-orient"

[[step]]
op = "resize"
mode = "max"
width = 1000

[[step]]
op = "watermark"
text = "© ddphotos"
gravity = "southeast"
size = 40
margin = 20
color = "ffffff"
opacity = 0.85

[[step]]
op = "optimize"
```

JPEG is not byte-pinned (frame pin rule), so this recipe gets pixel rows only.

### Call 3 — the seam: in-process library for pixels and bytes; the binary for what only it can do

How `apply` runs a recipe (`src/cli/optimize.rs:45-203`): it loads and resolves assets
(`:56`, `:65`). A recipe that ends in `optimize` has the marker popped (`:73`, via
`split_terminal_optimize`, `src/recipe/mod.rs:441`, which is `pub(crate)`). Then:
- with a pinned format (the recipe's `format`, `:89-91`): `run_pixel_op(…, quality: None,
  Some(fmt), Some(AutoQuality::Fast), false)` (`:95-110`). The Fast plan's quality is `None`
  (`:437-440`), so the encode is `sink::encode_to_bytes(img, fmt, None)`;
- without one: `run_optimize_autodecide` (`:113`). That encodes every shortlisted candidate
  (`:1217-1224`, each through `encode_to_bytes`, `:1025`) and picks by size against the source
  file's bytes (`decide::pick_winner`, `:1229`). It is `pub(super)`, and so is everything it
  calls (`optimize_decide_one`, `encode_one_optimize_decided`, `:1413`);
- a plain recipe with no marker: `pipeline.run` (`:172`), then `Sink::write` (`:195`) with
  quality `global.quality.or(recipe.quality)` (`:194`). `Sink::write` encodes through the same
  public `sink::encode_to_bytes` (`src/sink/mod.rs:492`, `:526`, defined at `:698`).

**The seam, therefore:**
- **Pixel rows (42) and byte rows (6): in-process.** `Recipe::from_toml`, pop a terminal
  `optimize` exactly as `recipe_round_trip.rs:351-353` does, attach the asset bytes,
  `build_pipeline`, run, pin the `Image`. Byte rows call the same public
  `crustyimg::sink::encode_to_bytes` that `Sink::write` and the candidate solver call, with the
  quality `apply` passes: `None` for both byte-pinned recipes (for `v2_format_quality_optimize`
  because of the pinned path, `:95-110`; for `v2_watermark_integer_numbers` because neither
  `-q` nor `recipe.quality` is set, `:194`). Any change to the encoder settings inside
  `encode_to_bytes` therefore reds a byte row. This is negative control M6.
- **The seam check (2 rows): the binary.** For the two byte-pinned recipes on
  `rgb8_2600x1730`, run `env!("CARGO_BIN_EXE_crustyimg") apply --recipe <copy of the corpus
  file> rgb8_2600x1730.png --out-dir out` and assert the output file's SHA-256 equals the
  golden byte pin. This shows the in-process call is the call `apply` makes. It catches a change
  in `apply`'s routing, for example a quality starting to flow to `encode_to_bytes`, that the
  in-process rows alone cannot see. Design measured both rows equal (finding 3).
- **The bundled format rows (3): the binary.** The `optimize` decision is CLI-private (above),
  and the frame said no `src/` change, so the chosen format can only be read from
  `apply --recipe <name> … --json`: `candidates[winner].format`. The dimensions and the
  pre-encode pixels of these rows are already pinned in-process. **The rows run on
  `rgb8_2600x1730` only.** All nine took 107 s on the Mac in the dev profile; three take
  about 32 s, and the test may run them on three threads. **This is a cut, reported:** the
  ruling says "pin the chosen format", and design pins it for one fixture per bundled recipe,
  not three. The other six measured AVIF too.
  The decision depends on which codecs are built (`BuiltCodecs`, `src/cli/optimize.rs:1207-1210`).
  So this test is `#[cfg(all(feature = "avif", not(feature = "webp-lossy")))]`. It is compiled
  and runs on the six default-codec legs (test ×3, avif, heic ×2), and is absent on webp-lossy
  and lean, where a different answer would be correct. That is a compile-time scope, not a
  runtime skip: where the test exists it always runs, and the lines it checks are still parsed
  and counted on every leg (AC-8).

To feed the binary, the test writes the fixture to a `tempfile::tempdir()` as PNG (with
`image`'s encoder; `tempfile` is already a dev-dependency, `Cargo.toml:316`). It then asserts
that `Image::load` of that file has the fixture's SHA-256: a check on the transport, so the
binary provably sees the same pixels. Asset paths resolve against the working directory
(`std::fs::read(&path)`, `src/cli/common.rs:237-258`), so the test runs the binary with
`current_dir(tempdir)` and writes `logo.png` there for `v2_watermark_integer_numbers`.

### Call 4 — the golden: one readable file per recipe

`tests/fixtures/recipe_corpus/<stem>.output` (bundled: `bundled_<name>.output`). It uses LF,
and the reader normalizes `\r\n`, because the repo has no `.gitattributes` and Windows may
check out CRLF. `#` lines and blank lines are comments. Every other line is one row, keyed by
`<fixture> <kind>`:

```
# v2_watermark_integer_numbers: what this recipe produces on each fixture.
# Written by `CRUSTYIMG_BLESS=1 cargo test --test recipe_corpus_output -- --ignored bless`.
# A changed line means an existing recipe's output changed.
rgb8_2600x1730 pixels dims=1024x681 colour=Rgb8 depth=8 sha256=<64 hex>
rgb8_2600x1730 png sha256=<64 hex>
rgba8_1730x2600 pixels dims=681x1024 colour=Rgba8 depth=8 sha256=<64 hex>
rgba8_1730x2600 png sha256=<64 hex>
rgba16_2600x1730 pixels dims=1024x681 colour=Rgba16 depth=16 sha256=<64 hex>
rgba16_2600x1730 png sha256=<64 hex>
```

The kinds are `pixels` (every recipe), `png` and `webp` (the byte rows), and `optimize`, which
appears only in the three bundled files, as `rgb8_2600x1730 optimize format=avif`. `colour` is
`{:?}` of `image::ColorType`. `depth` is bits per channel
(`bytes_per_pixel() * 8 / channel_count()`). `sha256` is over `pixels().as_bytes()` for
`pixels`, and over the encoded file for `png`/`webp`. **51 lines in all:** 42 `pixels` + 3
`png` + 3 `webp` + 3 `optimize`.

**Failure message.** The test checks every row before it fails, then panics once with every
mismatch. Each mismatch is one line naming recipe × fixture × the first field that moved, in the
order dims, colour, depth, pixels. Exactly:

```
v1_resize_max × rgb8_2600x1730: dims changed: golden 800x532, actual 801x532
v1_resize_max × rgb8_2600x1730: colour changed: golden Rgb8, actual Rgba8
v1_resize_max × rgb8_2600x1730: depth changed: golden 8, actual 16
v1_resize_max × rgb8_2600x1730: pixels changed (dims 800x532, colour Rgb8, depth 8 unchanged)
v2_watermark_integer_numbers × rgb8_2600x1730: png bytes changed (pixels unchanged)
v2_watermark_integer_numbers × rgb8_2600x1730: png bytes changed (pixels also changed)
bundled_web × rgb8_2600x1730: optimize chose webp, golden avif
v1_resize_max × rgb8_2600x1730: no golden line (bless to add it)
v1_resize_max × rgb8_2600x1730: step 1 ('resize') changed nothing
```

After the lines comes `N of 48 rows changed`, then the **actual** content of every recipe file
for this target, between marker lines:

```
----- BEGIN tests/fixtures/recipe_corpus/v1_resize_max.<os>-<arch>.output -----
…every line the test computed, plus the golden's `optimize` lines carried over unchanged…
----- END -----
```

The marker always uses the per-target file name (`<os>-<arch>` from `std::env::consts::OS`
and `ARCH`), in either layout, so outcome (ii) can write those files straight from a CI log.
The dump is part of the panic message, and libtest prints a failing test's output, so it
reaches the CI log. (HANDOFF trap 11: a passing test's output does not.)

### Call 5 — cross-leg: both outcomes specified; the build's first checkpoint decides

Nothing pins resize pixels across machines today, and design pushed nothing. The CI legs that
run `cargo test` (`.github/workflows/ci.yml`): `test` on `[ubuntu-latest, macos-latest,
windows-latest]` (`:72-95`), `avif` (`:129`, `:148`), `webp-lossy` (`:157`, `:175`), `heic` on
`[ubuntu-latest, macos-latest]` (`:196-223`), and `lean` (`:232`, `:249`). That is **eight legs
on three targets**: `linux-x86_64` (ubuntu-latest), `macos-aarch64` (macos-latest),
`windows-x86_64` (windows-latest).

**Checkpoint 1 (first push of the build):** commit the test and goldens blessed on the Mac
(shared layout), push, and open a **draft** PR. Wait with a backgrounded
`gh pr checks <n> --watch` (Bash `run_in_background`), never a poll loop. Then read the eight
legs:
- **(i) every leg green → shared layout.** One `<stem>.output` per recipe, 14 files. The test's
  `const LAYOUT: Layout = Layout::Shared;`.
- **(ii) any leg red on `recipe_corpus_output` → per-target layout.**
  `const LAYOUT: Layout = Layout::PerTarget;` with
  `const TARGETS: &[&str] = &["linux-x86_64", "macos-aarch64", "windows-x86_64"];`.
  - **Naming:** `<stem>.<os>-<arch>.output`, for **every** recipe on **every** target (42
    files), even recipes whose lines agree. One selection rule, no fallback to a shared file.
    The shared files are deleted.
  - **Selection:** `format!("{stem}.{}-{}.output", std::env::consts::OS, std::env::consts::ARCH)`.
  - **No golden for this target → fail, never skip.** If the running target is not in `TARGETS`,
    or its file is missing, the test panics with
    `no recipe golden for <os>-<arch>: goldens exist for linux-x86_64, macos-aarch64, windows-x86_64; bless on this target and add it to TARGETS`.
  - **Blessing a leg without that machine: from the CI log.**
    `gh run view <run-id> --log-failed > leg.log`. Strip GitHub's per-line prefix
    (`<job>\t<step>\t<timestamp> `) with `sed -E 's/^[^\t]*\t[^\t]*\t[^ ]+ //'`, then write each
    `----- BEGIN <path> -----` … `----- END -----` block to `<path>`. The Mac's own files come
    from a local bless with `LAYOUT = PerTarget`. Then push again. **The goldens are accepted only
    when that second CI run is green on all eight legs.** A log-copied golden that is wrong goes
    red there.
  - **Not designed: two legs on the same target disagreeing** (e.g. `lean` ≠ `test` on
    ubuntu). Features touch codecs, not pixel ops, so design expects this not to happen. If it
    does, **stop and report**: it would need per-feature goldens, which is a frame-level change.

Either way, the build records the checkpoint's run URL and each leg's result for
`recipe_corpus_output` in Build Completion, plus each leg's wall-clock time for the test.

**AVIF bytes (ruling 2's exception).** The format rows print nothing about AVIF bytes. The build
does not add an AVIF byte pin. If the build wants to show the legs agree on AVIF bytes, that is
a verify question, per the ruling.

### Call 6 — blessing, and why CI can't do it

`#[test] #[ignore] fn bless_recipe_corpus_outputs()`. Run it with
`CRUSTYIMG_BLESS=1 cargo test --test recipe_corpus_output -- --ignored bless`.
- `CRUSTYIMG_BLESS` unset: writes nothing and returns. This copies the existing convention
  (`tests/recipe_round_trip.rs:374-378`), so a bare `cargo test -- --ignored` stays harmless.
- **`CI` set to any non-empty value: panics before writing a byte**, with
  `refusing to bless recipe goldens in CI (CI is set)`. A panic, not a printed skip, so the
  attempt is red (trap 11). GitHub Actions always sets `CI=true`.
- Built without the default codecs (`not(all(feature = "avif", not(feature = "webp-lossy")))`):
  panics with `bless needs the default codec set (avif on, webp-lossy off)`, because it cannot
  compute the `optimize` lines.
- Writes this target's files only: `<stem>.output` in (i), `<stem>.<os>-<arch>.output` in (ii).
  The gate is one small function, `bless_gate(bless: Option<&OsStr>, ci: Option<&OsStr>) ->
  Result<bool, String>`, unit-tested directly (FT-7).

### Call 7 — the row count is a literal, and who updates it

The pixel test asserts `assert_eq!(rows_checked, 48, "14 recipes × 3 fixtures + 6 byte rows")`.
The completeness test asserts 51 golden lines and the exact file set. All three numbers are
**typed literals**, never `RECIPES.len() * 3`, so a recipe dropped from the list goes red.
**Each later spec that adds a corpus recipe** (the PROJ-017 op specs) adds its stem to `RECIPES`,
its expected dims to `EXPECTED_DIMS`, and its `.output` files. In the same diff it raises every
literal the test counts with: `ROWS` (48), `GOLDEN_LINES` (51), the `[…; 14]` length of
`EXPECTED_DIMS`, the `42` in FT-1's `EXPECTED_DIMS.len() * FIXTURES.len()` assert, and the
doc comments that quote them; `recipe_round_trip.rs` has its own `CORPUS` and `14`. It names
the new numbers in an AC. The README lists them by identifier (grep the test for numeric
literals and array lengths).

The two targets the author cannot run come from CI: bless the local target, push, and the run
fails on the others, printing each missing file's dump (FT-9). Copy each `BEGIN`/`END` block
from `gh run view <run> --log-failed` into its file and push; the second run is green on every
leg. A red on one target alone with no code change is libm drift: re-bless that target only,
in its own commit.
Verify recomputes the numbers from `RECIPES` and the golden files, and checks the old lines are
unchanged (the frame's blessing rule).

### Size

[S] holds. No `src/` change. About 350 lines of test, more than the frame's ~250: the dims table
is 42 lines, plus the two binary tests and the message unit test. Plus 14 golden files (42 in
(ii)), one corpus recipe, its `.golden`, a 4-line change to `recipe_round_trip.rs` and a new
README. Runtime per leg: pixel test ≈ 15 s on the Mac (the rows run twice; see FT-1), binary
tests ≈ 35 s, or ≈ 15 s with the three format rows on threads. Windows is likely 2–3× that.

### Where design departs from the frame (for the orchestrator)

1. **The RGBA fixtures are 1730×2600 and 2600×1730, not 1100×700** (Call 1). Measured: the
   frame's sizes left 6 rows pinning a resize that never ran.
2. **The chosen-format pin runs through the binary, on one fixture.** The library cannot reach
   the decision without a `src/` change (Call 3).
3. **`tests/fixtures/recipe_corpus/README.md` does not exist.** The frame's file list says
   "document … in README.md", so the build creates it.
4. **The pin is exact per target, but not per feature set** (Call 5). If two legs on one target
   disagree, the build stops.


## Inputs

- The 10 corpus recipes (`tests/fixtures/recipe_corpus/*.toml`), the 3 bundled recipes
  (`recipes/{gallery,product,web}.toml`), and the new `v2_ddphotos_cli_shape.toml` (Call 2).
- Three fixtures generated in the test from raw samples (Call 1), a seeded 64×32 logo, and
  `crustyimg::text::DEFAULT_FONT`.
- The library API: `Recipe::from_toml`, `OperationParams::set_resolved_bytes`,
  `Recipe::build_pipeline`, `Pipeline::run`, `Image::pixels`, `crustyimg::sink::encode_to_bytes`.
- The built binary (`env!("CARGO_BIN_EXE_crustyimg")`), for FT-4 and FT-5 only.

## Outputs

- `tests/recipe_corpus_output.rs`: the guard (FT-1 to FT-7, FT-8 in outcome (ii) only) and the
  `#[ignore]` bless.
- 14 golden files `tests/fixtures/recipe_corpus/{<stem>,bundled_<name>}.output` in outcome (i),
  or 42 `….<os>-<arch>.output` files in outcome (ii). 51 non-comment lines per target.
- `tests/fixtures/recipe_corpus/v2_ddphotos_cli_shape.toml` and its `.golden`.
- `tests/recipe_round_trip.rs`: `CORPUS` gains `"v2_ddphotos_cli_shape"`, and the count goes
  13 → 14.
- `tests/fixtures/recipe_corpus/README.md` (new).
- No `src/` change. No change to `CACHE_SCHEMA_VERSION`, the lockfile format or the version.

## Acceptance Criteria

Notation: **the 11 resizers** = `v1_resize_{cover,exact,fill,fit,max,percent}`,
`v2_watermark_integer_numbers`, `bundled_{gallery,product,web}`, `v2_ddphotos_cli_shape`.
**The 3 non-resizers** = `v1_watermark_image_placement`, `v1_watermark_text_font`,
`v2_format_quality_optimize`. **F** = the three fixtures `rgb8_2600x1730`, `rgba8_1730x2600`,
`rgba16_2600x1730`. The guard is new, so nothing in it is "RED on `main`". Its failing tests
are the mutation rows under Failing Tests, each run against the built test.

**The corpus**
- [ ] **AC-1. The ddphotos shape joins the corpus.** `tests/fixtures/recipe_corpus/v2_ddphotos_cli_shape.toml`
      is byte-identical to Call 2. Its `.golden` is `to_toml()` of it, written by the existing
      bless (`tests/recipe_round_trip.rs:374`). `CORPUS` lists it in sorted position (after
      `v1_watermark_text_font`, before `v2_format_quality_optimize`). `:332` reads
      `assert_eq!(pairs.len(), 14, "11 corpus recipes + 3 bundled recipes")`.
      `to_toml_is_byte_identical_to_main_over_the_corpus` passes. The 13 existing `.golden` files
      are unchanged byte for byte.

**The fixtures**
- [ ] **AC-2. The fixtures are generated, not decoded, and pinned.** The test builds F with
      Call 1's generator verbatim, from raw samples, with no crustyimg decode or encode. FT-2
      asserts each fixture's dims, colour type and full 64-digit SHA-256 as literals. Their first
      8 digits are `94daa4af`, `7df0cef0` and `75dfcd86`, the values measured at design.

**The pin**
- [ ] **AC-3. Every existing recipe is pinned on every fixture, at the pre-encode image.** For each
      of the 14 recipes × F (42 rows): parse, pop a terminal `optimize` as
      `tests/recipe_round_trip.rs:351-353` does, attach assets (Call 1), `build_pipeline`, run.
      Pin `dims`, `colour`, `depth` and `sha256` of `pixels().as_bytes()`. Plus 6 byte rows:
      `v2_watermark_integer_numbers` × F → `png`, and `v2_format_quality_optimize` × F → `webp`.
      Each is the SHA-256 of `crustyimg::sink::encode_to_bytes(&out, fmt, None)` (Call 3 says why
      `None`). FT-1 asserts `rows_checked == 48` as a literal.
- [ ] **AC-4. The pin cannot pass over a guard that ran nothing.** For every one of the 42
      rows, FT-1 also asserts the following. A failure is a mismatch line, not a skip:
      (a) **dims** equal a literal table in the test, keyed by recipe and fixture, with exactly
      42 entries (FT-1 checks the count). The table is the "dims" column of the golden as
      measured at design:

      | recipe | `rgb8_2600x1730` | `rgba8_1730x2600` | `rgba16_2600x1730` |
      |---|---|---|---|
      | `v1_resize_cover` | 721x480 | 640x962 | 721x480 |
      | `v1_resize_exact` | 640x480 | 640x480 | 640x480 |
      | `v1_resize_fill` | 640x480 | 640x480 | 640x480 |
      | `v1_resize_fit` | 640x426 | 319x480 | 640x426 |
      | `v1_resize_max` | 800x532 | 532x800 | 800x532 |
      | `v1_resize_percent` | 488x325 | 325x488 | 488x325 |
      | `v1_watermark_image_placement` | 2600x1730 | 1730x2600 | 2600x1730 |
      | `v1_watermark_text_font` | 2600x1730 | 1730x2600 | 2600x1730 |
      | `v2_ddphotos_cli_shape` | 1000x665 | 665x1000 | 1000x665 |
      | `v2_format_quality_optimize` | 2600x1730 | 1730x2600 | 2600x1730 |
      | `v2_watermark_integer_numbers` | 1024x681 | 681x1024 | 1024x681 |
      | `bundled_gallery` | 2560x1703 | 1703x2560 | 2560x1703 |
      | `bundled_product` | 1600x1065 | 1065x1600 | 1600x1065 |
      | `bundled_web` | 2048x1363 | 1363x2048 | 2048x1363 |

      (b) the output's SHA-256 ≠ the fixture's. (c) **Each step acts:** the recipe runs one step
      at a time (a one-step pipeline per step, each fed the previous output), and every step
      except `auto-orient` and `identity` changes the SHA-256. (d) The step-by-step result equals
      `build_pipeline(..).run(fixture)` on the whole recipe, the real pipeline. (e) A refusal or
      error is a mismatch line naming it. The exact control lines are
      `<recipe> × <fixture>: dims <actual>, expected <literal> (expected-dims table)`,
      `<recipe> × <fixture>: output equals the fixture`, and
      `<recipe> × <fixture>: step <i> ('<op>') changed nothing`.
- [ ] **AC-5. A red row names recipe, fixture and field.** FT-1 checks all 48 rows before it fails,
      then panics once with one line per mismatch, using the exact wording of Call 4. The fields
      are checked in the order dims, colour, depth, pixels, then bytes. The lines are followed
      by `<N> of 48 rows changed` (N counts rows with at least one mismatch line, not lines) and by the actual content of every recipe file for this
      target, between `----- BEGIN tests/fixtures/recipe_corpus/<stem>.<os>-<arch>.output -----`
      and `----- END -----`. FT-3 checks the nine message lines of Call 4, the three control lines of AC-4 and one dump block
      exactly, as strings, against a pure function the test exposes to itself.
- [ ] **AC-6. The library seam is the call `apply` makes.** FT-4 runs the binary
      `apply --recipe <copy of the corpus file> rgb8_2600x1730.png --out-dir out`, in a temp dir
      holding `logo.png` (the test's logo bytes), for `v2_watermark_integer_numbers` and
      `v2_format_quality_optimize`. It asserts the single output is named
      `rgb8_2600x1730.png` (respectively `.webp`) and that its SHA-256 equals the golden's
      `rgb8_2600x1730 png` (respectively `webp`) line. Before running, it asserts `Image::load`
      of the written fixture PNG has the fixture's SHA-256 (the transport check). It asserts 2
      rows checked.
- [ ] **AC-7. The bundled recipes' chosen format is pinned.** FT-5,
      `#[cfg(all(feature = "avif", not(feature = "webp-lossy")))]`, runs
      `apply --recipe <gallery|product|web> rgb8_2600x1730.png --out-dir out --json` and reads
      `candidates[winner].format` from stdout. It asserts that equals the golden's
      `rgb8_2600x1730 optimize format=…` line (`avif` for all three, as measured), and that the
      output file's extension matches. It asserts 3 rows checked. It runs on the test ×3, avif
      and heic ×2 legs, and does not exist on webp-lossy or lean (Call 3).

**The goldens**
- [ ] **AC-8. Each golden line is checked, and no golden is stray.** FT-6, on every leg: the
      `.output` files on disk are exactly the set `LAYOUT` requires (14 shared, or 14 × `TARGETS`),
      derived from `RECIPES`. Every non-comment line parses as Call 4's grammar. Keys are unique
      per file. Each recipe has exactly 3 `pixels` lines. `png` lines exist only for
      `v2_watermark_integer_numbers`, `webp` only for `v2_format_quality_optimize`, and
      `optimize` only for the three bundled files. Total non-comment lines per target is
      **51**, a literal. CRLF is normalized on read. Bless writes LF.
- [ ] **AC-9. The CI layout is the one the CI run picked.** The build's checkpoint 1 is a draft PR
      with the test and the Mac goldens (Call 5). The PR's Build Completion records the run URL
      and each of the 8 `cargo test` legs' result for `recipe_corpus_output`. Verify re-reads
      the legs from `.github/workflows/ci.yml` (`:72`, `:129`, `:157`, `:196`, `:232`), not from
      this list. Then, whichever lands:
      **(i)** `LAYOUT = Shared`, 14 files, and the final head's CI run is green on all 8 legs; or
      **(ii)** `LAYOUT = PerTarget`, `TARGETS` = exactly `linux-x86_64`, `macos-aarch64`,
      `windows-x86_64` (the `runs-on` labels above map to those), 42 files, FT-8 green, and
      the run *after* the log-blessed goldens were pushed is green on all 8 legs.
      Two legs on one target disagreeing is a **stop and report**, not a landing.

**Blessing**
- [ ] **AC-10. A bless cannot happen in CI.** `CRUSTYIMG_BLESS=1 cargo test --test recipe_corpus_output -- --ignored bless`
      rewrites this target's goldens (a second run leaves `git status` clean). With `CI` set
      to a non-empty value it panics with `refusing to bless recipe goldens in CI (CI is set)`
      before writing anything. Without `CRUSTYIMG_BLESS` it writes nothing and returns. Without
      the default codec set it panics with `bless needs the default codec set (avif on,
      webp-lossy off)`. FT-7 unit-tests `bless_gate` on all four cases, and control M5 drives the
      real command.

**Docs and scope**
- [ ] **AC-11. The corpus README states the rules.** The new `tests/fixtures/recipe_corpus/README.md`
      covers: what `.toml`/`.golden`/`.output` each pin; the bless command and its CI refusal;
      the layout in force; and the rule that **a changed `.output` line in a PROJ-017 PR is a
      verify finding unless that spec's ACs name the recipe and say why**. A new file or line
      for a new recipe is not a re-bless. Verify checks with
      `git diff --name-only origin/main -- 'tests/fixtures/recipe_corpus/*.output'`.
      It also covers the literal-count update rule (Call 7). Behaviour-first wording, no spec or
      DEC ids.
- [ ] **AC-12. Nothing else moves.** `git diff --name-only origin/main` on the build branch
      lists only Outputs' files. No `src/` path. `scripts/cli-golden.py`, `docs/recipes.md`,
      `.github/workflows/` and the 13 existing `.golden` files are untouched.
- [ ] **AC-13. Every negative control below gives exactly its listed red and green sets.** The
      build drives M1–M7 and pastes each red output into Build Completion. Verify re-drives at
      least M1 (required), M3 and M5, on a clean build.

## Failing Tests

Written during **design**. The guard is new: its "RED" is the mutation table below, each row run
against the built test.

- **`tests/recipe_corpus_output.rs`** (new)
  - FT-1 `every_recipe_output_matches_its_golden`: AC-3, AC-4, AC-5. 48 rows; literal
    `rows_checked == 48`; the dims table has 42 entries; collects every mismatch, then panics
    once with the dump.
  - FT-2 `the_fixtures_are_the_ones_the_goldens_were_blessed_on`: AC-2. 3 × (dims, colour,
    full SHA-256) literals.
  - FT-3 `a_row_mismatch_names_the_field_that_moved`: AC-5. The 9 lines of Call 4 plus the AC-4
    control lines, each built from a constructed golden/actual pair, compared as exact strings.
    Also one dump block (markers, per-target name, and an `optimize` line carried through).
    Covers trap 10: one case has the dims unchanged and the pixels changed, so a message that
    always says "dims" fails.
  - FT-4 `the_library_seam_writes_the_bytes_apply_writes`: AC-6. Binary, 2 rows, plus the
    transport check.
  - FT-5 `optimize_picks_the_pinned_format_for_each_bundled_recipe`: AC-7. Binary,
    cfg-gated, 3 rows; may run the three on threads.
  - FT-6 `every_golden_line_is_checked_and_no_golden_is_stray`: AC-8. Literal 51.
  - FT-7 `bless_refuses_in_ci_and_without_the_switch`: AC-10. Covers
    `bless_gate(None, None)` → `Ok(false)`, `(Some("1"), None)` → `Ok(true)`,
    `(Some("1"), Some("true"))` → `Err` containing `refusing to bless`, and
    `(Some("1"), Some(""))` → `Ok(true)`.
  - FT-8 `a_target_without_goldens_fails`: AC-9 (ii), **outcome (ii) only**.
    `golden_path("v1_resize_max", "linux", "riscv64")` is `Err` with exactly Call 5's message,
    and `("macos", "aarch64")` is `Ok(…/v1_resize_max.macos-aarch64.output)`. This is the
    trap-10 row: the derived path differs from the naive shared one.
  - FT-9 `a_missing_golden_fails_and_still_prints_its_dump`: AC-9. Drives the report builder
    as a pure function with one recipe's golden absent: exactly one missing-file line naming
    the expected path, counted in the failures, and every recipe's `BEGIN … END` block present
    and complete, the one with no golden included.
  - `bless_recipe_corpus_outputs`: `#[ignore]`, Call 6.
- **`tests/recipe_round_trip.rs`**
  - `to_toml_is_byte_identical_to_main_over_the_corpus`: AC-1 (`CORPUS` + 14). RED with the new
    `.toml` present but `CORPUS` not updated (`CORPUS must list every corpus recipe`).

### Negative controls (HANDOFF trap 12: location, every red, every green)

Apply each on the build branch, rebuild (the binary too: `cargo test` rebuilds it), run
`cargo test --test recipe_corpus_output > out.txt 2>&1; echo $?` (redirect, never pipe), record,
and revert with `git checkout <paths>`. Then confirm `git status --short` is empty. "Lines"
means FT-1's mismatch lines. FT-2, FT-3, FT-6 and FT-7 are green under every row unless listed.

| # | mutation (exact location) | RED | GREEN |
|---|---|---|---|
| M1 **required at verify** | `src/operation/mod.rs:584` `FilterType::Lanczos3` → `FilterType::CatmullRom` | FT-1 with exactly **36 lines**: 33 × `pixels changed (dims … unchanged)` (the 11 resizers × F) + 3 × `png bytes changed (pixels also changed)` (`v2_watermark_integer_numbers` × F), and `36 of 48 rows changed`. FT-4's `png` row | FT-1's 9 pixel rows of the 3 non-resizers × F and its 3 `webp` rows. FT-4's `webp` row. FT-5 **expected** green: AVIF wins by about 30×, but this was not driven at design, so a red here is reported, not explained away. `recipe_round_trip` stays green, which is the gap this guard closes |
| M2 | `src/operation/mod.rs:1629` `params.get_f32("opacity").unwrap_or(1.0)` → `unwrap_or(0.9)` | FT-1 with exactly **9 lines**: 6 × `pixels changed` (`v1_watermark_text_font`, `v2_watermark_integer_numbers` × F) + 3 × `png bytes changed (pixels also changed)`. FT-4's `png` row | FT-1's other 36 pixel rows (including `v1_watermark_image_placement`, `opacity = 0.6`, and `v2_ddphotos_cli_shape`, `opacity = 0.85`) and its 3 `webp` rows. FT-4's `webp` row. FT-5 |
| M3 positive control | `src/pipeline/mod.rs:105` `self.ops.iter().enumerate()` → `self.ops.iter().enumerate().take(0)`. Then **re-bless under the mutation** (local, `CI` unset), so the goldens describe "nothing ran". Then run | FT-1 on **all 42 pixel rows** through AC-4's controls alone: 33 × `dims …, expected …` (the resizers) + 9 × `output equals the fixture` (the non-resizers), plus a `step <i> ('<op>') changed nothing` line for every non-exempt step | The golden comparisons themselves (they were blessed on the mutation). FT-4 and FT-5 (the binary and the goldens are mutated alike). Revert with `git checkout src/pipeline/mod.rs tests/fixtures/recipe_corpus/` |
| M4 row count | delete `"v1_resize_fit"` from `RECIPES` in the test | FT-1: `rows checked 45, expected 48`. FT-6: `v1_resize_fit.output` is a stray golden, and 48 lines ≠ 51 | FT-2, FT-3, FT-4, FT-5, FT-7 |
| M5 bless in CI | none. Run `CI=true CRUSTYIMG_BLESS=1 cargo test --test recipe_corpus_output -- --ignored bless > b.txt 2>&1; echo $?`, with `shasum -a 256 tests/fixtures/recipe_corpus/*.output` before and after | exit ≠ 0; `b.txt` contains `refusing to bless recipe goldens in CI (CI is set)` | the two `shasum` listings are identical, and `git status --short` is empty. **The scenario's control:** the same command with `CI` unset exits 0 and rewrites the files (mtimes change), leaving `git status` clean. So the CI variable is what stopped it |
| M6 encoder settings | `src/sink/mod.rs:841-845`, the default `write_to` arm: for `ImageFormat::Png` only, encode with `image::codecs::png::PngEncoder::new_with_quality(&mut cursor, CompressionType::Best, FilterType::Adaptive)` instead | FT-1 with exactly **3 lines**: `png bytes changed (pixels unchanged)` × F. FT-4's `png` row | all 42 pixel rows, the 3 `webp` rows, FT-4's `webp` row, FT-5 (PNG never wins the decision; it lost by about 130× at design) |
| M7 `apply` routing | `src/cli/optimize.rs:91` `let pinned_format = cli_pin.or(resolve_format(recipe.format.as_deref())?);` → `let pinned_format = cli_pin;` | FT-4's `webp` row: `apply` no longer writes `rgb8_2600x1730.webp`. The recipe falls through to the auto-decision | **All of FT-1** (the in-process rows cannot see `apply`'s routing, which is why FT-4 exists). FT-4's `png` row (the plain path, `:172`, is untouched). FT-5 (bundled recipes have no `format`) |

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle. It is the
equivalent of a handoff document, folded into the spec since there is no separate receiving
agent.*

### Decisions that apply

- `DEC-059`: Option A. Pin the robust, record the fragile. That is why AVIF and the `optimize`
  search get no byte pin, and why the bundled rows pin pixels + format.
- `DEC-058`: canonical recipe TOML is a cache-key input. `recipe_round_trip.rs` already pins it,
  and this spec adds one recipe to that pin.
- `DEC-094`/`DEC-096`: the AVIF encoder is deterministic per machine, with the tile count pinned.
  The format rows lean on the decision's wide margin, not on AVIF bytes.
- `DEC-095`: ops preserve colour type and bit depth. Hence the `colour`/`depth` fields and the
  `Rgba16` fixture.
- `DEC-100`: assets are resolved at the IO boundary. The test attaches bytes with
  `set_resolved_bytes`, as the CLI's resolver does.
- `DEC-102`: params are validated at load. Every corpus recipe must still load (FT-1 fails a
  row on a load error).
- `DEC-104`: watermark refusals. Every watermark row was accepted on the new fixtures; a refusal
  is a red row.
- `DEC-008`: Lanczos3 via `fast_image_resize`, the target of M1.

### Constraints that apply

- `test-before-implementation`: the guard is the test. Its failing set is the mutation table.
- `single-image-library`: the fixtures and logo use `image`. No second imaging crate.

### Prior related work

- `SPEC-131` (shipped): the corpus and the TOML golden this spec extends.
- `SPEC-097` (shipped): `scripts/cli-golden.py`, a same-machine before/after tool. It stays out
  of CI and untouched.
- `SPEC-122` (shipped): the linear-light resize whose cross-leg stability checkpoint 1 measures.
- `SPEC-124` (shipped): AVIF tile count pinned.
- `SPEC-142` (framed/designing): native-vs-wasm parity of the ddphotos recipes. A different
  question, different files.

### Out of scope

See `## Frame` → "Out of scope". Also: the `docs/recipes.md` crop-verb finding (filed on
STAGE-058); an AVIF byte pin (a verify question only, ruling 2); per-feature goldens (Call 5
says stop and report).

## Notes for the Implementer

- **Push a WIP draft PR as checkpoint 1** as soon as the test compiles and passes on the Mac
  with blessed goldens, before anything else (Call 5). That push is also the long-build
  checkpoint. Wait on CI with `gh pr checks <n> --watch` run in the background, never a poll
  loop. Read failures with `gh run view <id> --log-failed`.
- **Bless order.** Add the `.toml`, update `CORPUS` and the count, then run
  `CRUSTYIMG_BLESS=1 cargo test --test recipe_round_trip -- --ignored bless`. `git status` must
  show **only** the new `v2_ddphotos_cli_shape.golden`; any other `.golden` changing is a stop.
  Then bless the `.output` files.
- Bless computes and writes; it does not run AC-4's controls. M3 depends on that: goldens can be
  blessed over a mutated pipeline and the controls still catch it.
- The probe's per-step fold is the cheap way to do AC-4 (c). Build a one-step `Recipe` clone per
  step, as the probe did. Then run the whole recipe once more through `build_pipeline` for
  (d).
- `cargo test` in an interactive terminal fails `display_sink_refuses_non_tty`, so redirect
  stdout to a file. A piped command reports the pipe's exit status. zsh does not word-split
  unquoted parameters. macOS has no `timeout`. Use `/usr/bin/grep` and `/usr/bin/git` for
  anything you count (`rtk` can truncate).
- On Windows, `tempfile` paths and `current_dir` work as on Unix. Compare file names, not full
  paths, in FT-4/FT-5.
- 16-bit inputs to the lossless-WebP and AVIF encoders print an 8-bit downgrade warning on
  stderr. That is expected, not a failure.
- `git commit -s` on every commit (DCO). Comments read behaviour-first, with no spec or DEC ids.
- Record in Build Completion: the checkpoint run URL, each leg's result and wall-clock time for
  this test, the outcome (i)/(ii), and M1–M7's red output.

---


## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `test/spec-141-recipe-output-guard`
- **PR (if applicable):** #233 (draft)
- **All acceptance criteria met?** yes, under outcome (ii) (per-target goldens). See below.
- **New decisions emitted:** none
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`):
  - `tests/recipe_corpus_output.rs` — new: the guard (FT-1 to FT-8) and the `#[ignore]` bless
  - `tests/recipe_round_trip.rs` — `CORPUS` gains `v2_ddphotos_cli_shape`; count 13 → 14
  - `tests/fixtures/recipe_corpus/v2_ddphotos_cli_shape.toml` — new corpus recipe (Call 2, byte for byte)
  - `tests/fixtures/recipe_corpus/v2_ddphotos_cli_shape.golden` — its canonical TOML
  - `tests/fixtures/recipe_corpus/README.md` — new: the rules
  - `tests/fixtures/recipe_corpus/<stem>.{linux-x86_64,macos-aarch64,windows-x86_64}.output` — 42 new goldens
- **`src/` diff:** `git diff --stat origin/main...HEAD -- src/` is empty. No `Cargo.toml`/`Cargo.lock` change.
- **Outcome: (ii), per-target.** Checkpoint 1 run
  https://github.com/jysf/crustyimg/actions/runs/41134312226 (head `5fdf103`, Mac goldens in the
  shared layout). macos-latest `test` passed. Red on a pixel/byte row: ubuntu `test`, avif, heic-ubuntu,
  webp-lossy, lean (26 of 48 rows changed on the Linux legs), and windows.
  The macOS heic leg died earlier on `avif_tile_pin` (unrelated), so it never reached this test. All five
  Linux legs printed byte-identical dumps, so no two legs on one target disagreed. Linux, Windows and
  Mac differ pairwise on 12 of 14 recipes (the two that agree are the ones with no resize). Goldens for
  Linux and Windows were copied from that log's marker blocks.
  Second run (head `886d388`, the per-target layout):
  https://github.com/jysf/crustyimg/actions/runs/41136772264, green on all 8 `cargo test` legs plus msrv.
  Wall time of the corpus-output test per leg: macos 34.0 s, ubuntu 50.4 s, windows 29.8 s, avif 52.3 s,
  heic-macos 41.9 s, heic-ubuntu 33.9 s, webp-lossy 22.1 s (7 tests), lean 16.9 s (7 tests).
- **Fixture hashes** (FT-2, full 64 digits): `94daa4af…`, `7df0cef0…`, `75dfcd86…`, matching AC-2 on every leg.
- **Mutations** (each applied by content, built clean per target dir, reverted and shown green):
  - M1 (`src/operation/mod.rs:584` Lanczos3→CatmullRom): FT-1 36 lines = 33 `pixels changed` + 3 `png bytes
    changed (pixels also changed)`, `36 of 48 rows changed`; FT-4 png red; FT-5 green. Matches the table.
  - M2 (`src/operation/mod.rs:1629` unwrap_or(1.0)→0.9): 9 lines = 6 pixels + 3 png; FT-4 png red. Matches.
  - M3 (`src/pipeline/mod.rs:105` `.take(0)`, re-blessed under it): 33 `dims …, expected …` + 9 `output equals
    the fixture` + 54 `changed nothing` (= 18 non-exempt steps × 3). Matches after one test fix (below).
    Goldens restored with `git checkout`; status clean.
  - M4 (delete `v1_resize_fit` from `RECIPES`): FT-1 `rows checked 45, expected 48`; FT-6 reports the stray
    golden set. It stops at the file-set assert, so the "48 ≠ 51 lines" half is not reported separately.
  - M5 (`CI=true` bless): exit 101, `refusing to bless recipe goldens in CI (CI is set)`; golden shasums
    identical before/after; the same command with `CI` unset exits 0 and rewrites files, `git status` clean.
  - M6 (`src/sink/mod.rs` ~841 PNG encoder settings): 3 lines `png bytes changed (pixels unchanged)`; FT-4 png red;
    FT-5 green. Matches.
  - M7 (`src/cli/optimize.rs:91` `pinned_format = cli_pin`): FT-4 red on the webp row
    (`rgb8_2600x1730.avif` written, expected `.webp`); all of FT-1 and FT-5 green. Matches.
  M1–M7 were driven on the shared layout (before the checkpoint result); the per-target switch only changed
  which golden file is read, and a final clean run of the whole test is green.
- **Matrix** (each in a fresh `CARGO_TARGET_DIR`, fmt `--check` and clippy `--all-targets -D warnings` clean):
  default 8 passed; `--no-default-features` 7 passed (FT-5 compiled out); `--features webp-lossy` 7 passed.
- **Deviations from spec:**
  - AC-4 (b) "output equals the fixture" is reported only when the dims check held, which is the only way
    M3's listed 33 + 9 lines come out.
  - The first push failed the lean leg on a dead-code error (`BUNDLED` is used only where FT-5 exists). Fixed by
    gating it with the same cfg.
  - `N of 48 rows changed` counts mismatch lines, which exceeds 48 under M3 (96).
- **Follow-up work identified:** none. The macOS heic leg's `avif_tile_pin` failure on the first run passed
  on rerun; it is worth a look as possibly flaky, but it is outside this spec.

### Punch list (verify)

Test-only; head `f6d297b` on PR #233 (draft). No `.output` file and no `src/` change.

1. **A missing golden fails and still prints every dump.** `check_recipes` (a pure function over
   a golden loader) turns a missing `.output` into one line naming the file, carries on with an
   empty golden, and prints that recipe's `BEGIN`/`END` block. `golden_path_for` now errors only
   for a target outside `TARGETS` (FT-8 unchanged), so the "goldens exist for …" message can no
   longer contradict a file that is merely missing. FT-9 added.
2. **FT-3 covers AC-4's control lines.** `dims_control_line`, `equals_fixture_line` and
   `step_unchanged_line`; `compute` calls them.
3. **`<N> of 48 rows changed` counts rows** (distinct recipe, fixture, kind with a mismatch line).
4. **FT-4 collects both rows** and prints `png row red/green`, `webp row red/green`.
5. **README and Call 7** name the literals (`ROWS`, `GOLDEN_LINES`, the `[…; 14]` length, the
   `42`, `CORPUS` and its `14` in `recipe_round_trip.rs`), the CI bless procedure and the
   libm-drift rule.

Controls (each on a clean build): missing `v1_resize_max.macos-aarch64.output` moved aside:
exit 101, one `no golden file …` line, 14 `BEGIN` blocks, the `v1_resize_max` block equals the
moved file (diff empty), restored and green. FT-9 with the old panic back: red. FT-3 with one
character changed in each of the three formatters: red each time. M3 re-driven: `42 of 48 rows
changed` (96 lines: 33 + 9 + 54). M1 re-driven: `36 of 48`, FT-4 `png` red and `webp` green.
During the work FT-9 first read the real goldens and went red under M1; it now uses empty
goldens for the other recipes so it is independent of what the pipeline draws.

CI run 41152445637 at `f6d297b`: green on every leg. `recipe_corpus_output` ran 9 tests on
windows, ubuntu, macos and avif; 8 on lean and webp-lossy.

### Punch list 2 (re-verify)

Test-only; PR #233 head was `6b90d71`, now `6376d65`. Diff touches `tests/recipe_corpus_output.rs`,
`tests/fixtures/recipe_corpus/README.md` and this spec only.

1. **The real lookup is pinned.** `a_target_without_goldens_fails` (extended, not a new test; the
   count stays 9) now calls `golden_path_for("__no_such_recipe__", "macos", "aarch64")` and
   requires `Ok` with the expected path and no file on disk. FT-8 (unknown target, Call 5's
   message) unchanged.
2. **The stray-file check names files**: `missing: [...]; extra: [...]`.
3. **FT-4's message** reads `apply wrote bytes that differ from the golden's <kind> row`.
4. **README comment list** gains the `GOLDEN_LINES` doc comment.

Controls (clean builds, stdout redirected, `$?` read):
- Mutation (A), `|| !path.exists()` put back in `golden_path_for`: the test goes red, exit 101,
  `a known target must return Ok even when the file is absent: "no recipe golden for
  macos-aarch64: ..."`. Restored: green.
- `bundled_web.linux-x86_64.output` moved aside: red, `missing: ["bundled_web.linux-x86_64"];
  extra: []`. A copy named `zz_extra.linux-x86_64.output`: red, `missing: []; extra:
  ["zz_extra.linux-x86_64"]`. Both restored; the tree held only the intended edits.
- M1 re-driven (`src/operation/mod.rs:584` Lanczos3 to CatmullRom): FT-4 `png row red:
  v2_watermark_integer_numbers: apply wrote bytes that differ from the golden's png row`; the
  `webp` row did not fire. Reverted with `git checkout`.
- Whole `recipe_corpus_output`: 9 passed, 1 ignored (default); 8 passed, 1 ignored (lean).
  clippy and fmt clean, default and `--no-default-features`.

CI run 41229750086 at `6376d65`: green on every leg. `recipe_corpus_output` ran 9 tests on
ubuntu and avif, 8 on lean.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Whether M3's `output equals the fixture` line applies to resizers too; the listed counts only work if it does not.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The lean/webp-lossy dead-code rule: anything used only by the cfg-gated tests must be gated too.

3. **If you did this task again, what would you do differently?**
   — Run clippy on `--no-default-features` before the first push, so checkpoint 1 measures pixels and nothing else. No design call turned out wrong; the per-target outcome was the one the spec anticipated.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
