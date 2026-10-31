---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-017
  status: proposed
  priority: high
  target_ship: null

repo:
  id: crustyimg

created_at: 2026-10-30
shipped_at: null

value:
  thesis: >
    Every geometry edit a photo owner reaches for — crop, straighten, quarter
    turn, flip, trim, pad — exists as a recipe step, so an edit made anywhere
    (the CLI, the ddphotos edit page, a hand-written recipe) can be saved and
    re-run on the original by the CLI with the same result. A geometry
    operation that exists only as a flag cannot be offered by a page that has
    to hand its edit to the CLI, so the recipe step is the deliverable and the
    flag is the convenience.
  beneficiaries:
    - "Anyone preparing photos for the web who today has to leave crustyimg to crop or straighten, then come back to resize and encode"
    - "The ddphotos edit page (`integrations/ddphotos/`), which can only resize and watermark until recipes can crop and rotate"
    - "Recipe and `build` users, whose pipelines gain the ops every other image tool has, inside the reproducible build"
    - "Wave 4 (PROJ-014) — art direction and an og-card preset both need a crop op before they can be framed"
  success_signals:
    - "`crop`, `rotate`, `flip`, `trim` and `pad` are each a registered recipe step, with a key table that refuses bad parameters at load (DEC-102), and each is reachable from `crustyimg edit`"
    - "`rotate` takes any angle: a 2.5° straighten produces either the largest crop that fits or an expanded canvas, resampled in linear light at the source's bit depth"
    - "The ddphotos edit page offers crop and straighten, and the recipe it emits, run by the native CLI on the full-size original, matches the page's own result at the same scale, asserted by a test rather than by eye"
    - "No existing recipe's output bytes change: every op is new, and a golden test over the existing recipe corpus proves the lockfiles stay valid"
    - "Quarter turns and flips are exact: a rotate-90 four times, or a flip twice, returns the input byte-for-byte in pixels"
  risks_to_thesis:
    - "⚠ The page edits a downscaled preview and the CLI runs on the full-size original. A crop written in pixels on the preview lands in the wrong place on the original, so crop and pad need size-independent units (fractions of the image). Getting this wrong breaks the thesis silently, because both halves still exit 0"
    - "⚠ Arbitrary-angle rotate resamples, so it inherits the linear-light working type that made `resize` 1.5-2.5x slower in 0.8.0 (PROJ-015's working-type stage). If that stage has not landed, straighten ships slow, or ships in gamma space and contradicts DEC-095's precedent"
    - "`edit` applies its flags in a fixed canonical order regardless of where they appear. Crop-then-rotate and rotate-then-crop give different images, so geometry flags either get an order rule written down or `edit` grows ordered steps. Either way it is a decision, not a build detail"
    - "`trim` (remove a uniform border) is the one op that looks at pixel content. It stays deterministic with a stated tolerance, but it is the op most likely to drift into 'smart' territory the roadmap explicitly keeps out of this wave"
---

# PROJ-017: Geometry as Recipe Steps

## What This Project Is

Roadmap Wave 5, framed: **crop** (rectangle, gravity, aspect), **rotate** (quarter turns
*and* any angle), **flip**, **trim** and **pad**, every one of them a recipe step first and an
`edit` flag second. Then the first real consumer: the ddphotos edit page gains crop and
straighten, and the recipe it hands the CLI reproduces the edit on the original.

The operation registry today holds five ops: `identity`, `invert`, `resize`, `auto-orient`,
`watermark` (`src/operation/registry.rs`, `with_builtins`). None of the five here exist.
`resize --fill` scale-and-crops to a box, which is not a crop op: it cannot cut a free
rectangle or hold an aspect without also resizing.

## Why Now

**Because it has a consumer that is blocked on it.** The ddphotos edit page shipped in #218
and was hardened through two test-bed rounds (#224, #227). It can only resize and watermark,
because recipes have no crop or rotate, and the roadmap row was rewritten in #219 to say so.
Straightening is the first thing a photo owner reaches for after crop.

**Because Wave 4 waits on it.** Two items from the 2026-10-25 feature review (art direction
with per-breakpoint crops, and a 1200×630 og-card preset) are crops. Framing PROJ-014 first
would mean framing them against an op that does not exist.

**Because it changes no existing bytes.** Every op is new, so unlike PROJ-015 this project
pays no lockfile migration. That is a property to protect, which is why a success signal
asserts it.

**Because it is close to ideal agent work.** It's code-bound, deterministic, and each op has
an exact or independently checkable answer. The roadmap's sequencing rationale names it
alongside build/cache as the work velocity actually compresses.

## Success Criteria

- `crop`, `rotate`, `flip`, `trim`, `pad` are registered recipe steps with key tables, and
  each is reachable from `crustyimg edit` and from wasm `transform` without wasm-specific code.
- Quarter turns and flips are exact and lossless in pixels. A 4× rotate-90 round trip is the
  identity.
- Any-angle rotate supports `crop-to-fit` (the largest axis-aligned rectangle inside the
  rotated image) and `expand` (a canvas that holds all of it, filled with a stated colour or
  transparent). It's resampled in linear light at the source's bit depth, and checked
  against an independent implementation (`vips rotate` / ImageMagick `-distort SRT`), not
  against itself.
- Crop and pad accept size-independent units, so a recipe authored on a 1200px preview
  replays correctly on a 6000px original.
- The ddphotos edit page offers crop (free and aspect presets) and straighten, and a
  native-vs-wasm parity test runs the recipe the page emits through both.
- The existing recipe corpus produces byte-identical output before and after.

## Scope

### In scope
- The five ops as recipe steps: key tables, validation at load, `Operation` impls that keep
  colour type and bit depth (DEC-095's widen-to-work, narrow-to-write)
- `edit` flags for each op, and the rule for how `edit` orders them
- Size-independent crop and pad units (fractions), and pixel units for the CLI
- Reusing `Gravity` (`src/operation/mod.rs`, nine compass points, from `watermark`) for crop
  and pad anchoring
- Arbitrary-angle rotate with `crop-to-fit` and `expand`, and its resampling filter
- The ddphotos edit page: crop and straighten controls, the recipe it emits, and the parity
  test
- `docs/recipes.md` and `docs/cli-reference.md` entries for each op
- The DECs this needs: the `edit` ordering rule, the size-independent unit rule, and the
  rotate resampler choice if it adds a dependency

### Explicitly out of scope
- **Smart crop, saliency, face-aware anything.** Post-1.0 beta per the roadmap. `trim`
  removes a uniform border with a stated tolerance and nothing more.
- **Perspective / keystone correction and lens distortion.** Not "rotate", and judgment-bound.
- **Art direction and the og-card preset.** They need this project's crop, and they belong
  with the manifest (PROJ-014). Logged in `guidance/ideas.yaml`.
- **The editor saving its edit** (roadmap row 6b, recipe sidecars). A separate wave; this
  project only makes the edit expressible.
- **A new verb.** The verb set was frozen at 14 (STAGE-030). Geometry goes through `edit`
  and recipes.
- **The working-type performance fix.** PROJ-015 owns it. This project depends on it (see
  Dependencies) and does not duplicate it.

## Stage Plan

Ordered by dependency. Specs inside each stage are listed in build order.

- [ ] STAGE-058 (proposed) — Lossless geometry: crop, quarter-turn rotate, flip, pad, trim, and their `edit` flags
- [ ] STAGE-059 (proposed) — Straighten: rotate by any angle, resampled in linear light, crop-to-fit or expand
- [ ] STAGE-060 (proposed) — The ddphotos edit page crops and straightens, and its recipe replays on the original

**Count:** 0 shipped / 0 active / 3 pending

STAGE-060's crop half can start once STAGE-058's crop spec ships. Its straighten half
waits for STAGE-059.

### Build order

One line per spec, in the order to frame and build them. Each spec still runs frame →
design → build → verify → ship in its own sessions (PROJ-011's HANDOFF rules). SPEC numbers
are given at each spec's frame, not here. The "after" column is the hard dependency; anything
without one can run beside the previous line (at most two builds at once, never on the same
files).

| # | stage | spec | size | after |
|---|---|---|---|---|
| 1 | 058 | No-bytes-changed guard over the existing recipe corpus | S | — |
| 2 | 058 | `crop` as a recipe step, with the size-independent unit DEC | M | 1 |
| 3 | 060 | Native-vs-wasm parity harness, green on today's recipes | S | — |
| 4 | 058 | `rotate` quarter turns and `flip` | S | 1 |
| 5 | 058 | `pad` | S | 2 |
| 6 | 058 | `trim` | S | 2 |
| 7 | 060 | Crop on the ddphotos page | M | 2, 3 |
| 8 | 058 | `edit` flags for the five, with the ordering DEC | S | 2, 4, 5, 6 |
| 9 | 059 | Any-angle `rotate`, `expand` mode, the resampler | M | 4, and ⚠ PROJ-015's working-type stage |
| 10 | 059 | `crop-to-fit` mode | S | 9 |
| 11 | 059 | Linear-light control and depth/alpha tests | S | 9 |
| 12 | 060 | Straighten on the ddphotos page | S | 10, 7 |
| 13 | 060 | Test-bed round, README, wasm size delta | S | 12 |

Total: 13 specs (3 M, 10 S). Lines 1–8 need nothing outside this project.

**Reserved ids (2026-10-30):** line 1 is **SPEC-141** and line 3 is **SPEC-142**. Their frame
prompts are `specs/prompts/SPEC-141-frame.md` and `SPEC-142-frame.md`, and the two can run at
the same time. SPEC-139 is taken on another branch, and SPEC-140 is left for PROJ-016 (DEC-107).
Later lines get their ids at frame.

## Dependencies

### Depends on
- **PROJ-011 / STAGE-050 + SPEC-131** (shipped): the parameter-rich op registry and typed
  per-op parameter structs every new op registers through.
- **DEC-102** (recipe step params validated at load) and **DEC-095** (ops preserve colour
  type and bit depth): both bind every op here.
- **SPEC-122** (shipped): linear-light resampling, which straighten reuses.
- ⚠ **PROJ-015's working-type stage** (not yet defined, flagged there as a shipped
  regression to pull forward). Straighten resamples, so its speed is that stage's speed.
  It doesn't block STAGE-058, which never resamples. It should land before STAGE-059 is
  built, or STAGE-059's design states the measured cost it accepts.
- **The ddphotos test bed** (`integrations/ddphotos/`, #227 green on 2026-10-29): STAGE-060's
  evidence runs there.

### Enables
- **PROJ-014 (Wave 4):** art direction (`<picture>` with per-breakpoint crops) and an og-card
  preset.
- **Roadmap row 6b:** an editor that saves a recipe has something worth saving.
- **The recipe playground (Wave 6):** a recipe format with geometry is one worth a
  playground.

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <yes/no + notes>
- **How many stages did it actually take?** <number, compare to plan>
- **What changed between starting and shipping?** <one or two sentences>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **What did we defer to the next project?**
  - <one-line items>
