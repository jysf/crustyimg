---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-131
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-050
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: null                # chosen at build dispatch
  created_at: 2026-10-24

references:
  decisions:
    - DEC-005
    - DEC-014
    - DEC-031
    - DEC-036
    - DEC-058
    - DEC-061
    - DEC-064
    - DEC-099
    - DEC-100
    - DEC-101
  constraints:
    - untrusted-input-hardening
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-127
    - SPEC-128
    - SPEC-129
    - SPEC-133
    - SPEC-134

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-050's <capability>". Optional; null is acceptable.
value_link: >
  STAGE-050's recipe-reach outcome is "a recipe can express the whole job"; that only holds if a
  recipe that says something wrong is refused rather than silently run with defaults. Closes the
  silent-default path the watermark recipe surface (SPEC-128, unreleased) would otherwise ship in
  0.8.0.

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
      tokens_total: 5329474
      tokens_breakdown:
        input: 94
        output: 24567
        cache_creation: 116763
        cache_read: 5188050
      estimated_usd: 3.94
      duration_minutes: null
      recorded_at: 2026-10-24
      note: >
        Interactive frame session in its own worktree (docs/spec-131-frame). Measured from the
        session's transcript (cf81d232-….jsonl, identified by content), deduped by `.message.id`
        (47 unique ids), input/cache_creation/cache_read once per id, MAX output_tokens per id.
        Priced by component at Opus $5/$25 per MTok, cache_creation x1.25 ($6.25), cache_read
        x0.10 ($0.50). Measured before the commit/push turns, so it slightly under-counts.
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 10991748
      tokens_breakdown:
        input: 134
        output: 64574
        cache_creation: 201405
        cache_read: 10725635
      estimated_usd: 8.24
      duration_minutes: null
      recorded_at: 2026-10-24
      note: >
        Interactive design session in its own worktree (docs/spec-131-design). Measured from the
        session's transcript (62f96852-….jsonl, identified by content), deduped by `.message.id`
        (67 unique ids), input/cache_creation/cache_read once per id, MAX output_tokens per id.
        Priced by component at Opus $5/$25 per MTok, cache_creation x1.25, cache_read x0.10.
        Includes a release build of main and the driven measurement table. Measured before the
        commit/push turns, so it slightly under-counts.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-131: typed per-operation parameter structs

## Context

STAGE-050 backlog item "[M] **Typed per-operation parameter structs**" (split out of SPEC-127 by
its design, Call 4). PROJ-011 ships as one release, 0.8.0.

`OperationParams` is a `BTreeMap<String, toml::Value>` plus a resolved-asset side channel
(`src/operation/mod.rs:53`), read through `get_str` / `get_u32` / `get_f32` / `get_bool`
(`:73-99`), each of which returns `None` for a missing key **and** for a wrong-typed one. Each op
validates its own keys in its constructor. Five ops are registered
(`src/operation/registry.rs:90-98`): `identity`, `invert`, `auto-orient` (no params), `resize`
(`mode`, `width`, `height`, `percent`) and `watermark` (`image`, `text`, `font`, `size`, `color`,
`gravity`, `opacity`, `scale`, `margin`, `tile`). The terminal `optimize` marker is not an op.

**The backlog premise ("found on input 400 of 500") is wrong on today's `main`.** `apply`
builds the pipeline once before resolving any input (`src/cli/optimize.rs:133-136`, "fails BEFORE
we touch any inputs"), and `build` does the same per target (`src/cli/build.rs:297-298`). Every
error a constructor *does* raise already fails at load, exit 1, zero outputs. **The real defect
is what the constructors do not raise.** Driven on a release build of `main` (`crustyimg 0.7.1`,
commit `804236a`), 5 generated 200×150 PNGs, `apply --recipe r.toml in/*.png --out-dir out`:

| recipe step (after `[[step]]`) | exit | outputs | stderr | output vs reference |
|---|---|---|---|---|
| `op="resize"` `mode="max"` `width="100"` (wrong type) | 1 | 0 | `error: invalid operation 'resize': mode 'max' requires a positive integer 'width' param (the long-edge cap)` | — |
| `op="resize"` `mode="max"` `widht=100` (misspelt) | 1 | 0 | *same line* — never names `widht` | — |
| `op="resize"` `mode="max"` `width=100` `bogus="x"` (unknown key) | **0** | 5 | *(empty)* | **0/5 differ** from the same step without `bogus` |
| `op="watermark"` `text="hi"` `opacity="0.5"` (string) | **0** | 5 | *(empty)* | **0/5 differ** from `opacity = 1.0` — the value is silently replaced by the default |
| `op="watermark"` `text="hi"` `opactiy=0.5` (misspelt) | **0** | 5 | *(empty)* | **0/5 differ** from `opacity = 1.0` |
| `op="watermark"` `text="hi"` `margin=-5` `tile="yes"` `size="big"` | **0** | 5 | *(empty)* | **0/5 differ** from `text="hi"` alone |
| `op="optimize"` `quality="max"` (terminal marker) | **0** | 1 | *(normal report)* | key ignored |

Controls, each proving the variable moves when well-typed: `opacity = 0.5` (float) vs
`opacity = 1.0` → **5/5 differ**; `margin = 5` `size = 64.0` vs defaults → **5/5 differ**;
self-diff of `opacity = 1.0` → 0/5. `build` (a one-target manifest over the same 5 inputs)
reproduces both halves: wrong-typed resize → exit 1, `target #0 (recipe r.toml, out dist)` +
the same line, 0 outputs; watermark `opacity = "0.5"` → exit 0, `built 1 target, 5 outputs`,
**0/5 differ** from `opacity = 1.0`.

So the user sees three things today:
1. **A watermark recipe that says something wrong runs anyway, with defaults, exit 0** — on every
   surface. The cause is `.unwrap_or(default)` over an accessor that conflates absent with
   wrong-typed (`src/operation/mod.rs:1337, 1341, 1360, 1365, 1379, 1380`). This path is
   SPEC-128's, merged to `main` but **not yet released** — fixing it before 0.8.0 means no user
   ever meets it.
2. **An unknown or misspelt key on any step is silently ignored.** This is a recorded accepted risk
   (DEC-061 verdict #1, pinned by `from_toml_tolerates_unknown_step_param_by_design`,
   `src/recipe/mod.rs:597`), which itself says a strict-validation spec flips it deliberately.
3. **Errors that do fire name neither the step nor the offending key** — `invalid operation
   'resize'` for a two-resize recipe does not say which one, and a misspelling is reported as a
   missing *correct* key. Precedents for good wording already exist in the repo: the top-level
   `deny_unknown_fields` error (`unknown field 'bogus', expected one of …`, with line/column) and
   `wasm::transform`'s `recipe step {index} ('{op}') needs its '{key}' asset …`
   (`src/wasm.rs:233-238`).

## Goal

A recipe step with an unknown key, a wrong-typed value or an out-of-range value is refused when
the recipe is loaded — before any input is decoded on `apply --recipe` and `build`, and before
any pipeline runs on wasm `transform` — with one stderr line naming the step index, the op and the
key (e.g. `error: recipe step 2 ('watermark'): 'opacity' must be a number in 0.0..=1.0, got the
string "0.5"`), exit code unchanged from today's recipe errors. Every recipe that is valid today
keeps parsing, producing the same pixels and the same `to_toml()` bytes (confirmed at design, Call 2).

## Frame

**Verdict: GO, as one M spec** — with the premise corrected (above) and the scope pinned to the
user outcome, not to an internal storage refactor.

### Premise check
- **Late failure (input 400 of 500): WRONG.** Constructor errors already fire before any input on
  `apply` and `build` (cited above). Nothing in a malformed recipe fails per-input today.
- **Unknown key: silently ignored — confirmed**, on every op, the `optimize` marker included.
  This is the scope-changing answer the frame prompt asked for: the spec must reverse DEC-061
  verdict #1, which the design session records in DEC-102.
- **Wrong-typed value: split by op.** Required keys (`resize`'s `mode`/`width`/`height`/`percent`,
  watermark's `image`/`text`) fail at load with a misleading message. **Optional keys (watermark's
  `size`/`color`/`gravity`/`opacity`/`margin`/`tile`, and `scale` for the record) silently take
  their default** — the worst case in the table, wrong output at exit 0.
- **Out-of-range: mostly already caught at load** (resize `width = 0`, watermark `opacity`,
  `scale`, `size`), but `margin = -5` is wrong-typed-to-`u32` and therefore silently 0.

### Sizing — M, and why it is not L
Counting the verification, not the code:
- **Conditions × keys:** unknown key on 5 ops + the `optimize` marker (6 rows); wrong type on 14
  keys; out of range on the ~8 keys with a range. ≈ 30 rows — but they live in **one** function
  (op construction / step validation), so they are one table-driven unit test per op, not 30
  integration tests.
- **Surfaces:** `apply --recipe` (file path; bundled names are fixed text and need only a "still
  parses" row), `build`, wasm `transform`. All three already funnel through
  `Recipe::build_pipeline` (`src/cli/optimize.rs:73,136`, `src/cli/build.rs:298`,
  `src/wasm.rs:241`). So the surface axis needs **one integration test per surface** proving the
  error reaches the user with step + key, the documented exit code and zero outputs — not the full
  op × condition cross product per surface. The matrix is ~30 unit rows + ~4 surface rows +
  a round-trip corpus pin. That is M.
- **It becomes L only if** the design chooses to make typed structs the *serialized* form (struct
  → TOML), because then every recipe's `to_toml()` bytes, the cache key and the lockfile must be
  re-pinned across the corpus. **Recommendation: do not.** Validate into typed structs (or a
  per-op key schema) at load, and keep the ordered map as the canonical serialized form. That
  delivers the whole user outcome with zero DEC-005 round-trip risk. If the design finds typed
  storage necessary, that is the split point: "parse-time rejection" (this spec) vs "typed
  structs as storage" (a follow-up). No second stub is written here.

### DEC-005 round-trip risk
`Recipe::to_toml()` (`src/recipe/mod.rs:281`) serializes the *parsed* step map verbatim
(BTreeMap order, user-authored types), and `build` hashes that string into the cache key
(`src/cli/build.rs:107-137`, DEC-058) and the lockfile (DEC-101; SPEC-129 pinned that a
watermark-free recipe hashes exactly as before). If validation leaves the map untouched,
re-emission is byte-identical for every recipe that still parses — trivially achievable. What
does change: a recipe carrying an unknown key, which today parses, hashes (the key enters the
cache key: an inert `bogus` already forces a rebuild with identical output) and builds, will now
be refused — an intended break inside 0.8.0's single lockfile migration. If the design instead
re-emits from typed structs, three byte changes are expected and must be measured on the corpus:
integer-for-float (`opacity = 1` → `1.0`), default emission (absent `gravity` →
`gravity = "southeast"`?), and key order. `--save-recipe` (`src/cli/ops.rs:1089`) already emits
from `Operation::params()` and is a second, separate round-trip direction to keep green.

### Scope
**In:**
- Unknown key on any step → refused at load (all five ops and the terminal `optimize` marker).
- Wrong-typed value on any key → refused at load, never silently defaulted.
- Out-of-range values → refused at load (existing range checks keep firing; `margin` negative now
  caught).
- Error names step index (design picks 0- or 1-based and says why), op and key; wording for
  unknown / wrong-type / out-of-range.
- All three surfaces: `apply --recipe`, `build`, wasm `transform`. Bundled recipes keep parsing.
- `version = "1"` and `"2"` recipes both validated identically (params are not versioned today).
- Flip `from_toml_tolerates_unknown_step_param_by_design` deliberately; DEC-102 records the
  reversal of DEC-061 verdict #1.

**Out (and where it lives):**
- A wasm `validate(recipe_toml)` export returning structured errors — **does not exist today**;
  it is roadmap row #6 (recipe playground, `docs/roadmap.md:38`). **Needs filing** as a
  backlog item: "wasm `validate()` export returning structured, per-step recipe errors (playground
  prerequisite; consumes SPEC-131's errors)".
- "Did you mean `opacity`?" suggestions — nice-to-have, design's call whether it is cheap; if not,
  fold it into the `validate()` item above.
- Making typed structs the serialized form (see sizing) — follow-up only if design proves it
  necessary.
- Data-dependent runtime failures (e.g. an overlay larger than a given input) — not a malformed
  recipe; stays per-input.
- Watermark damage refusal / `--min-size` — SPEC-133.
- Asset files that do not exist — already refused at load by `resolve_recipe_assets`
  (`src/cli/common.rs:207`); unchanged.
- The direct CLI verbs (`resize`, `watermark` flags) — clap already types them.

### Rough build file list
- `src/operation/mod.rs` — `OperationParams` accessors (absent vs wrong-typed), `Resize::from_params`,
  `Watermark::from_params` (**collides with SPEC-133**).
- `src/operation/registry.rs` — per-op known-key schema / paramless ops reject keys.
- `src/recipe/mod.rs` — step index in errors, `RecipeError` variant(s), `optimize` marker keys,
  flipped pin test.
- `src/wasm.rs` — error text only, probably no logic.
- `src/cli/mod.rs` (where `CliError` wraps `RecipeError` and maps exit codes) — only if a new
  variant needs a mapping.
- Tests: `tests/recipe_round_trip.rs`, `tests/recipe_watermark.rs`, `tests/registry_seam.rs`,
  `tests/build.rs`, `tests/apply_batch.rs`, `tests/wasm_roundtrip.rs`, and a new strict-params test
  file.
- Docs: `recipes/README.md`, `docs/api-contract.md` (recipe error contract), `CHANGELOG.md`;
  `decisions/DEC-102-*.md` (new), a superseded-by note on `decisions/DEC-061-*.md`.

**Build order:** shares `src/operation/mod.rs` (`Watermark`) with SPEC-133 → **sequential with
SPEC-133**; recommend SPEC-131 first, so SPEC-133's `--min-size` recipe key (if it lands in
SPEC-133) is added under the strict schema rather than retrofitted. No file overlap expected with
SPEC-132 (`src/build/watch.rs`) or SPEC-134 (`src/sink/mod.rs`, CLI naming) — can run in
parallel with both, unless design routes the error through `src/cli/common.rs`.

### Corrections to `prompts/SPEC-131-design.md`
- It says a malformed recipe "is discovered when the op runs — on input 400 of 500". It is not:
  constructor errors already fire before any input (see Premise). The defect is silent
  acceptance, not late failure.
- It names "the wasm `validate()` surface in `src/wasm.rs`". **No such export exists** — the wasm
  surface that loads a recipe is `transform` (`src/wasm.rs:204`). `validate()` is out of scope
  (needs filing, above).
- "Before any image is decoded" holds for `apply`/`build` but not for wasm `transform`, which
  decodes `input` before it builds the pipeline (`src/wasm.rs:241`). Design decides whether to
  validate first there (cheap reorder) or to word that surface's AC as "before any pipeline runs".

### Maintainer rulings (2026-10-25) — both questions below are settled
1. **Yes:** unknown step keys become a load-time error. DEC-102 supersedes DEC-061's **accepted
   risk #1** ("Recipe step params stay tolerant"); the frame's "verdict #1" label below means that
   accepted risk.
2. **No** "new in 0.8.0" hint on the error; a **Breaking** CHANGELOG line is enough.

### Open maintainer questions (as asked at frame)
- **Reverse DEC-061 verdict #1 (unknown step keys become errors)?** Recommend **yes** — the stage
  item and DEC-061 itself anticipate it, and a warning would be ignored in CI. Only the maintainer
  can retire a recorded accepted risk.
- **Should `apply --recipe` with an unknown key in a recipe that worked on 0.7.x print a hint
  that this is new in 0.8.0?** Recommend **no** — a CHANGELOG line under "Breaking" is enough;
  the error already names the key.

## Design (2026-10-24)

### What the design measured on top of the frame

Driven on a release build of `main` at `ca19da3` (`crustyimg 0.7.1`), three generated PNGs
(200×150, its inverse, 300×120), `apply --recipe r.toml in/*.png --out-dir out`; each row's
outputs compared with `cmp` against a reference recipe over the same inputs. Script and inputs:
the design session's scratchpad (`drive.sh`), reproducible from this table alone.

| step (after `[[step]]`) | exit | outputs | stderr | vs reference |
|---|---|---|---|---|
| **CTRL** `resize` `mode="max"` `width=100`, self-diff | 0 | 3 | — | **0/3 differ** |
| `resize` max `width=100` **`height=50`** (key the mode never reads) | **0** | 3 | — | 0/3 differ from no `height` |
| `resize` `mode="percent"` `percent=50` **`width=10`** | **0** | 3 | — | 0/3 differ from no `width` |
| `resize` max `width=100.0` (float for an integer key) | 1 | 0 | `invalid operation 'resize': mode 'max' requires a positive integer 'width' param …` | — |
| `resize` percent **`percent = nan`** | **0** | 3 | — | **every output is 1×1** |
| `resize` percent **`percent = inf`** | **6** | 0 | per input: `target dimensions 4294967295×4294967295 exceed …` | fails per input, not at load |
| `invert` / `identity` / `auto-orient` + `bogus = 1` | **0** | 3 each | — | 0/3 differ |
| **CTRL** watermark `text="hi"` `opacity=0.5` vs `1.0` | 0 | 3 | — | **3/3 differ** (the key moves output) |
| watermark `opacity = 1` (integer) vs `1.0` | 0 | 3 | — | 0/3 differ — **an integer for a number key is valid and must stay valid** |
| watermark **`margin = 5.0`** (float for an integer key) vs no margin | **0** | 3 | — | 0/3 differ — silently 0 |
| **CTRL** watermark `margin = 5` vs no margin | 0 | 3 | — | **3/3 differ** |
| watermark **`size = nan`** vs an `identity` recipe | **0** | 3 | — | **0/3 differ — no watermark at all, exit 0** |
| watermark **`size = inf`** vs `identity` | **0** | 3 | — | **0/3 differ — no watermark** |
| watermark **`scale = nan`** vs `scale = 0.0001` | **0** | 3 | — | 0/3 differ — overlay shrunk to 1 px |
| watermark `opacity = nan` | 1 | 0 | `'opacity' must be in 0.0..=1.0, got NaN` | already caught |
| watermark `image=…` **`size = 64.0`** / **`color = "ff0000"`** (text-only keys on an image step) | **0** | 3 | — | 0/3 differ from the bare image step |
| watermark `image = 5` (wrong type, required key) | 1 | 0 | `watermark step must set exactly one of 'image' or 'text'` — never names `image`'s type | — |
| watermark `gravity = 3` vs default | **0** | 3 | — | 0/3 differ |
| `version = "2"`, watermark `opacity = "0.5"` vs `1.0` | **0** | 3 | — | 0/3 differ — v2 behaves exactly like v1 |
| `… [[step]] op = "optimize"` `quality = "max"`, `-o o1.png` | **0** | 1 | — | key ignored |

`build` (one target, same 3 inputs): `bogus = "x"` on a resize step → exit 0, `built 1 target, 3
outputs`, output `cmp`-identical to the control; watermark `size = nan` → exit 0, 3 outputs.
Wrong-typed resize `width = "100"` → exit 1, 0 outputs, two stderr lines:
`error: target #0 (recipe r.toml, out dist)` / `error: invalid operation 'resize': mode 'max'
requires a positive integer 'width' param (the long-edge cap)`.

**What this adds to the frame:**
1. **Non-finite numbers are a silent-wrong-output path the frame did not list.** TOML has `nan`
   and `inf`. `size = nan|inf` drops the watermark entirely at exit 0; `percent = nan` writes
   1×1 images at exit 0; `percent = inf` fails **per input** (exit 6) — the only case in the repo
   that really is "found on input N", because `percent <= 0.0` is false for `inf`.
2. **A key the step's mode never reads is silently ignored** — resize's `height` under `max`,
   `width` under `percent`; watermark's `size`/`color`/`font` on an `image` step. Same defect
   class as an unknown key (the user wrote something; nothing happened), so it is **in scope**
   (Call 4). This is a widening of the frame's three conditions to four; it is named here, not
   slipped in.
3. **A float for an integer key splits by op**: resize refuses it with a misleading message;
   watermark's `margin = 5.0` silently becomes 0.
4. **DEC-014 already promised this.** Its Decision says the accessors "return typed errors on
   missing/wrong-type". They return `Option` (`src/operation/mod.rs:73-99`) and conflate the two.
   This spec makes DEC-014's sentence true, at the registry rather than in the accessors.
5. 📌 **Not in scope, filed:** `size = 100000.0` is accepted and took **52 s** of wall time for 3
   inputs at exit 0 (the text raster is enormous). That is a resource-limit question (DEC-036's
   family, and the CLI's `--size` has the same lack of a cap), not a malformed recipe; it gets a
   backlog line, not a range here. Inventing an upper bound is a product decision.

### Call 1 — validate at load against a schema the registry publishes; the map stays the stored form

**Settled: the frame's recommendation holds. No L split.** `OperationParams` stays
`BTreeMap<String, toml::Value>` (+ the resolved-asset side channel) and stays the serialized form,
so `to_toml()` cannot change for any recipe that still parses — validation reads the map, never
rewrites it.

Each op **publishes a param schema** at registration, and the registry checks a step's map
against it **before** calling the constructor:

```rust
// src/operation/registry.rs (illustrative — names may differ, behaviour may not)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamSpec { pub key: &'static str, pub kind: ParamKind }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamKind {
    Text,                              // any string
    OneOf(&'static [&'static str]),    // a string from a fixed list (resize `mode`, `gravity`)
    Asset,                             // a string naming a file, resolved at the IO boundary
    UInt { min: u32 },                 // a TOML integer in min..=u32::MAX (a float is refused)
    Number { min: f64, min_inclusive: bool, max: Option<f64> }, // integer OR float, always finite
    Bool,
}

impl OperationRegistry {
    /// An op that takes keys. `params` is the op's whole accepted-key table.
    pub fn register_with_params(&mut self, name: &'static str, ctor: Constructor,
                                params: &'static [ParamSpec]);
    /// The published table (None for an unregistered name). The read side a future
    /// wasm `validate()` and the docs consume.
    pub fn param_specs(&self, name: &str) -> Option<&[ParamSpec]>;
    /// Unchanged meaning; now DERIVED from the schema (every `Asset` key, in table order).
    /// Return lifetime narrows from 'static to &self — every caller copies `&key` out, so
    /// no caller changes (src/cli/common.rs:213, src/cli/build.rs:175, src/wasm.rs:95,
    /// tests/registry_seam.rs:73,92 all compile as-is).
    pub fn asset_keys(&self, name: &str) -> &[&'static str];
}
```

- `register(name, ctor)` (unchanged signature) now means **"takes no keys"**: an empty schema, so
  any key on `identity`/`invert`/`auto-orient` is refused.
- `register_with_assets(name, ctor, keys)` (unchanged signature) becomes shorthand for a schema
  of those keys as `Asset`. `tests/registry_seam.rs::a_second_asset_op_registers_without_seam_change`
  must stay green **unedited** — it is now this spec's regression guard on the old entry point.
- `OperationRegistry::build` runs the schema check, then the constructor. Every path that
  constructs an op from a map goes through it (recipe steps; `src/cli/ops.rs:974`'s CLI-built
  resize params, which only ever set mode-relevant keys — `resize_params`, `src/cli/ops.rs:117`).
- **The schema tables live next to their ops** in `src/operation/mod.rs` (`RESIZE_PARAMS`,
  `WATERMARK_PARAMS` consts), registered in `with_builtins`. **Adding a recipe key is one line in
  that const plus the constructor's read** — that is SPEC-133's (and the `--min-size` item's)
  whole seam cost.
- **Where each rule lives:** the registry check owns *unknown key*, *type* and *range* (every
  range here is mode-independent: `width`/`height` ≥ 1, `percent`/`size`/`scale` > 0,
  `opacity` in 0..=1, `margin` ≥ 0, all numbers finite). The constructor keeps what depends on
  other keys: required keys per mode, `image` XOR `text`, keys unused by the mode (Call 4), and
  `color`'s hex syntax. Every constructor message names its key.
- **Why the LUT op fits without another widening:** `.cube` LUT = `[ParamSpec { key: "file",
  kind: Asset }, ParamSpec { key: "strength", kind: Number { min: 0.0, min_inclusive: true,
  max: Some(1.0) } }]` through `register_with_params`. Asset resolution (via the derived
  `asset_keys`) and strictness come from the one table. AC-12 registers exactly that shape as a
  fixture op and touches nothing else.

**Rejected:** typed structs as the stored form (the L split — three measured byte changes on the
corpus per the frame, a cache-key/lockfile re-pin, for no user-visible gain); per-op hand-written
key checks in each constructor without a published table (no read side for `validate()`/docs, and
a new op could forget it — the registry check cannot be forgotten); validating inside
`Recipe::from_toml` (it has no registry, and custom registries in tests would lose their ops).

### Call 2 — `to_toml()` is byte-identical, pinned by a golden corpus captured on `main`

With the map untouched, re-emission cannot change; the pin proves it rather than asserting it.
**Corpus boundary:** `recipes/*.toml` (3 bundled) + a new `tests/fixtures/recipe_corpus/` of
hand-written recipes that together use **every op and every one of the 14 keys at least once**,
in both versions: a v1 resize per mode (6), a v2 recipe with `format`/`quality` and a terminal
`optimize`, an image watermark with every placement key, a text watermark with `font`/`size`/
`color`, and an integer-for-number recipe (`opacity = 1`, `size = 24`). Each has a `.golden`
sibling holding `Recipe::from_toml(src)?.to_toml()?` **captured from unmodified `main` before any
`src/` change** (the build's first commit; generated by an `#[ignore]`d test gated on
`CRUSTYIMG_BLESS=1`, so verify can re-run it on `main` and diff to zero). Test-only recipe
literals scattered across `tests/*.rs` are **outside** the boundary; they stay covered by their
own tests going green.

Consequences, stated so nobody re-measures them: SPEC-129's
`target_recipe_hash_matches_recipe_hash_for_watermarkfree_recipe` keeps holding; **no
`CACHE_SCHEMA_VERSION` change, no lockfile change** for any recipe that still parses. A recipe
carrying a now-refused key no longer builds at all — the intended break, one line under
**Breaking** in the CHANGELOG (maintainer ruling 2).

### Call 3 — error shape, numbering, exit codes

**One line, two shapes**, both prefixed with the step:

```
recipe step 0 ('resize'): unknown key 'widht'; expected one of: mode, width, height, percent
recipe step 0 ('invert'): unknown key 'bogus'; 'invert' takes no keys
recipe step 2 ('optimize'): unknown key 'quality'; the terminal 'optimize' step takes no keys (set 'format'/'quality' at the top of the recipe, with version = "2")
recipe step 1 ('watermark'): 'opacity' must be a number in 0.0..=1.0, got the string "0.5"
recipe step 1 ('watermark'): 'margin' must be an integer >= 0, got -5
recipe step 0 ('resize'): 'percent' must be a finite number > 0, got nan
recipe step 0 ('resize'): 'mode' must be one of: max, exact, percent, fit, fill, cover; got "big"
recipe step 0 ('resize'): 'height' is not used by mode 'max'
recipe step 1 ('watermark'): 'size' only applies to a text watermark (this step sets 'image')
```

- **Wrong type and out of range share one shape** (`'<key>' must be <expectation>, got <actual>`)
  — the user's fix is the same either way; a third shape would add nothing. Unknown key gets the
  `expected one of` list, mirroring the top-level `deny_unknown_fields` wording users already see.
- **"Did you mean…?" — no.** The `expected one of` list already shows the right spelling (≤10
  keys), and edit distance is new code with its own test surface. Folded into the wasm
  `validate()` backlog line, where a playground can show it inline.
- **Steps are numbered from 0** — matching every existing step/target index in the repo
  (`RecipeAssetUnreadable`'s `recipe step {step}`, `src/cli/mod.rs:614`; wasm's
  `recipe step {index}`, `src/wasm.rs:233`; `build`'s `target #0`). Two numbering bases in one
  `build` error (`target #0` above `recipe step 1`) would be worse than 0-based. `recipes/README.md`
  says so in one line.
- **Ordering:** keys are checked in the map's (sorted) order and the **first** failure is
  reported — deterministic, one line. Schema checks run before the constructor, so a misspelt
  required key (`widht`) is reported as unknown, not as "requires 'width'".
- **Types:** `RecipeError::InvalidOperation` gains `step: usize`; its Display becomes
  `recipe step {step} ('{name}'): {reason}`. No new `RecipeError`/`RegistryError` variant, no
  `CliError` change: **exit code stays 1** on `apply` and `build` (`src/cli/mod.rs:756`), and
  `build` keeps its `error: target #N (…)` line above. The terminal-marker check raises the same
  variant from `Recipe::from_toml` (step = its index, name `optimize`). Existing matches on
  `InvalidOperation { name, .. }` (`tests/recipe_round_trip.rs:241`,
  `tests/recipe_watermark.rs:191,208`) compile unchanged; no test pins the old
  `invalid operation '…'` wording (grepped `src tests docs`).
- **No "new in 0.8.0" hint** (maintainer ruling 2).

### Call 4 — keys the step's mode never reads are refused (widening, named)

Resize: `mode = "max"` reads only `width`; `percent` reads only `percent`; the other four read
`width`+`height`. Watermark: an `image` step refuses `font`/`size`/`color`. Checked in the
constructors (they know the mode). **Line drawn:** only the *mode-selecting* keys (`mode`;
`image`/`text`) make another key unused. `tile = true` ignoring `gravity`/`margin`, and `center`
ignoring `margin`, are documented interplay the CLI verb accepts too — **not** refused.
Safe for crustyimg's own output: `Resize::params()` emits only mode keys (`src/operation/mod.rs:778`)
and `Watermark::params()` emits `size`/`color` only in text mode (`:1399`); AC-11 pins that.

### Call 5 — the terminal `optimize` marker takes no keys, checked in `from_toml`

The marker is recipe syntax, not a registry op — it is stripped by `split_terminal_optimize`
(`src/recipe/mod.rs:410`) before `build_pipeline` ever sees it, on all three surfaces. So its
check lives in `Recipe::from_toml`, which every surface calls (`src/cli/common.rs:166,171`,
`src/wasm.rs:207`). Only a **terminal** marker is checked there; an `optimize` step anywhere else
keeps today's `UnknownOperation` from `build_pipeline` (SPEC-112's AC-5, unchanged).

### Call 6 — surfaces, and what wasm `transform` actually guarantees

All three surfaces reach the check through `Recipe::from_toml` (marker) and
`Recipe::build_pipeline` (`src/cli/optimize.rs:73,136`, `src/cli/build.rs:298`,
`src/wasm.rs:241`).

- `apply --recipe` (single input, and the batch fan-out): refused before any input is resolved,
  exit 1, zero outputs.
- `build`: refused while preparing targets, before any target writes, exit 1, zero outputs.
- **wasm `transform`: no reorder.** Its AC is worded to what it guarantees: *a bad step is refused
  with the same message before any pipeline step runs; the input is decoded first, so a bad input
  AND a bad recipe report the decode error.* Reordering would make `src/wasm.rs` a shared file with
  SPEC-134 for a guarantee no caller needs — a caller that wants recipe-only checking needs the
  `validate()` export (filed), which is image-free by construction. **This spec does not edit
  `src/wasm.rs`.**
- Bundled recipes (`web`/`gallery`/`product`) are fixed text; AC-11 covers them.

### Call 7 — `version = "1"` and `"2"` are validated identically

Step params are not versioned (DEC-099 versioned only `format`/`quality`). Nothing in this spec
reads `recipe.version`; AC-9 pins it.

### Call 8 — DEC-102 is written at **build**, not here

DEC-102 records three things: the reversal of DEC-061 **accepted risk #1** (maintainer ruling 1),
the schema seam of Call 1 (registry publishes accepted keys/types/ranges; asset keys derived from
it), and that this is what makes DEC-014's "typed errors on wrong-type" true. It is written in the
build PR so it describes the code as merged (DEC-101 was written at build for the same reason).
Format: `decisions/_template.md`, block-list `affected_scope` (`src/operation/registry.rs`,
`src/operation/mod.rs`, `src/recipe/mod.rs`, `docs/data-model.md`), `supersedes: null` (it
supersedes one clause, not DEC-061), and its Decision says "**supersedes DEC-061's accepted risk
#1**" the way DEC-101 says "amends DEC-058's clause 4". In `DEC-061`, accepted risk #1 and the
"A `deny_unknown_fields`-equivalent for recipe step params" alternative each get a one-line
**"Superseded by DEC-102 (2026-10-25): unknown step keys are now refused at load."** marker;
DEC-061's `superseded_by` stays `null` (the rest of DEC-061 stands).

## Inputs

- **Files to read:** `src/operation/registry.rs` (whole file, 327 lines — the seam);
  `src/operation/mod.rs:33-140` (`OperationParams`), `:609-800` (`Resize`), `:1070-1090`
  (`Gravity::from_str` — `OneOf` list must match it), `:1180-1450` (`Watermark`);
  `src/recipe/mod.rs:86-180` (`RecipeError`), `:302-375` (`from_toml`, `build_pipeline`),
  `:392-420` (`split_terminal_optimize`), `:588-612` (the pin test that flips);
  `src/text/mod.rs:51` (`parse_color`).
- **Related code paths (read, do not edit):** `src/cli/common.rs:156-228` (`load_recipe`,
  `resolve_recipe_assets`), `src/cli/optimize.rs:55-140`, `src/cli/build.rs:150-180,290-300`,
  `src/wasm.rs:85-102,204-245`, `src/cli/ops.rs:117-170,960-990`.
- **External APIs:** none.

## Outputs

- **Files created:** `tests/recipe_strict_params.rs`; `tests/fixtures/recipe_corpus/*.toml` +
  `*.golden`; `decisions/DEC-102-recipe-step-params-are-validated-at-load.md`.
- **Files modified:** see Implementation Context → "Files the build touches".
- **New exports:** `operation::registry::{ParamSpec, ParamKind}` (re-exported from
  `crate::operation` beside `OperationRegistry`), `OperationRegistry::register_with_params`,
  `OperationRegistry::param_specs`. `RecipeError::InvalidOperation` gains `step: usize`.
  `OperationRegistry::asset_keys` returns `&[&'static str]` (was `&'static [&'static str]`).
- **Database changes:** none. No `CACHE_SCHEMA_VERSION` change, no lockfile format change.

## Acceptance Criteria

Surfaces are named per AC. "Refused" = `Recipe::build_pipeline` (or `from_toml` for the marker)
returns `RecipeError::InvalidOperation { step, name, reason }` whose `reason` names the key and the
class phrase (`unknown key` / `must be` / `is not used by` / `only applies to`).

- [ ] **AC-1. Unknown key, every op** (library; reaches all surfaces). A `bogus = 1` key on each of
      the five registered ops — `resize`, `watermark`, `identity`, `invert`, `auto-orient` — is
      refused; `reason` contains `unknown key 'bogus'`; the param ops list their keys
      (`expected one of: …`), the paramless ops say `takes no keys`. A misspelt required key
      (`widht = 100`) is reported as `unknown key 'widht'`, not as a missing `width`.
- [ ] **AC-2. The terminal `optimize` marker with any key is refused at `Recipe::from_toml`**
      (library; all surfaces): `quality = "max"` → `InvalidOperation { name: "optimize", step: <its
      index> }`, reason names `quality`. A keyless marker still parses (the three bundled recipes).
- [ ] **AC-3. Wrong type, every key** (library). One row per key — 14 rows: resize `mode`,
      `width`, `height`, `percent`; watermark `image`, `text`, `font`, `size`, `color`, `gravity`,
      `opacity`, `scale`, `margin`, `tile` — each given a value of the wrong TOML type, including
      **a float for an integer key** (`width = 100.0`, `margin = 5.0`). Each is refused naming its
      key with `must be`; none falls back to a default. **And the positive half:** an integer for
      a number key (`opacity = 1`, `size = 24`, `percent = 50`) is accepted and builds.
- [ ] **AC-4. Out of range, every ranged key** (library). `width = 0`, `height = 0`,
      `percent = 0`, `percent = nan`, `percent = inf`, `size = 0`, `size = nan`, `size = inf`,
      `scale = 0`, `scale = nan`, `opacity = 1.5`, `opacity = -0.1`, `opacity = nan`,
      `margin = -5`, `mode = "big"`, `gravity = "up"`, `color = "xyz"` — each refused naming its
      key. (The non-finite rows are the measured silent-wrong-output and per-input-failure paths.)
- [ ] **AC-5. A key the step's mode never reads is refused** (library). Resize: `max` + `height`;
      `percent` + `width`; `exact` + `percent`. Watermark: `image` + `size`; `image` + `color`;
      `image` + `font`. Each refused naming the unused key. **Not** refused (positive rows):
      `tile = true` with `gravity`/`margin`; `gravity = "center"` with `margin`.
- [ ] **AC-6. The error names the step index, op and key** (library Display). A recipe with two
      `resize` steps where only step **1** has `bogus = 1` displays exactly
      `recipe step 1 ('resize'): unknown key 'bogus'; expected one of: mode, width, height, percent`.
- [ ] **AC-7. `apply --recipe` refuses before writing anything — single input AND batch**
      (CLI, two tests). (a) Single input, watermark `opacity = "0.5"`: exit **1**, no output
      file, stderr is one `error:` line containing `recipe step 0 ('watermark')` and `'opacity'`.
      (b) Batch of ≥3 inputs with `--out-dir`, resize `bogus = "x"`: exit **1**, the out dir
      holds **zero** files, stderr names `recipe step 0 ('resize')` and `'bogus'`.
- [ ] **AC-8. `build` refuses before writing anything** (CLI). One target over ≥3 inputs, a
      watermark step with `size = nan`: exit **1**, no output file, stderr carries the
      `error: target #0 (…)` line and a line naming `recipe step 0 ('watermark')` and `'size'`.
- [ ] **AC-9. `version = "1"` and `"2"` are validated identically** (library). The same bad step
      under each version yields an equal `InvalidOperation` (step, name, reason).
- [ ] **AC-10. wasm `transform` refuses a bad step with the same message** (wasm, under
      `just wasm-test`). A valid PNG + a recipe whose resize step has `bogus = 1` → `Err`, whose
      text contains `recipe step 0 ('resize')` and `unknown key 'bogus'`; a following valid call
      on the same module succeeds. (Guarantee as worded in Call 6: before any pipeline step runs,
      after input decode.)
- [ ] **AC-11. Nothing valid stops working** (library + CLI). (a) Every recipe in the Call 2
      corpus parses, builds a pipeline, and its `to_toml()` equals its `.golden` byte for byte.
      (b) A recipe crustyimg itself writes is never refused: `Recipe::from_ops` over `resize` in
      all six modes, image-mode and text-mode `watermark` (with and without `scale`/`font`), and
      the three paramless ops → `to_toml` → `from_toml` → `build_pipeline` succeeds. (c) The
      existing `edit --save-recipe` replay tests (`tests/edit.rs`) and SPEC-129's
      watermark-free hash test stay green, unedited.
- [ ] **AC-12. The schema is published through the registry and generalises** (library).
      (a) `param_specs` returns a table for every built-in: resize's 4 keys, watermark's 10,
      empty for the paramless 3; `None` for an unregistered name. (b) `asset_keys("watermark")`
      is still `["image", "font"]`, now derived. (c) A fixture op registered through
      `register_with_params` with the LUT shape (`file: Asset`, `strength: Number 0..=1`) is
      strict (refuses `bogus`, refuses `strength = 2.0`) **and** its `file` is listed by
      `asset_keys` and reaches the constructor through `set_resolved_bytes` — with no other
      registry change. (d) `a_second_asset_op_registers_without_seam_change` passes **unedited**.
      (e) every `OneOf` value in `RESIZE_PARAMS`/`WATERMARK_PARAMS` is accepted by the constructor
      (the `gravity` list cannot drift from `Gravity::from_str`).
- [ ] **AC-13. The accepted risk is retired on purpose, and the docs say so.**
      `from_toml_tolerates_unknown_step_param_by_design` is replaced by
      `build_pipeline_rejects_unknown_step_param` (same two inputs, now refused). DEC-102 exists
      (Call 8); DEC-061's accepted risk #1 and its matching alternative carry the superseded
      marker. `recipes/README.md` documents strict keys and 0-based step numbers;
      `docs/data-model.md`'s resize/watermark key tables gain types/ranges and the "unused by
      mode" rule, and its sentence claiming a wrong-typed key is already a typed error is
      corrected; `docs/api-contract.md` states the recipe-step error contract (exit 1, one line,
      step/op/key) for `apply`/`build` and the wasm wording; `CHANGELOG.md` has one **Breaking**
      line (unknown/wrong-typed/unused step keys now refuse the recipe) and one **Fixed** line
      (watermark keys no longer silently default; `nan`/`inf` refused).
- [ ] **AC-14. Negative controls, one revert per independent condition** (AGENTS §15). Six
      conditions; reverting each alone flips only its tests (table under Failing Tests). Evidence
      is the behavioural flip, never a hash.
- [ ] **AC-15. Clean matrix.** Default, `--no-default-features`, `--features webp-lossy`, each in
      a fresh `CARGO_TARGET_DIR`, **sequential**; `clippy -D warnings` + `fmt --check`;
      `just wasm-check` and `just wasm-test`; **`just demo-build`** (the wasm bundle-size gate —
      `just check` never runs it, and this change does reach wasm through `src/operation/**`).
      `src/operation/**` gains no `std::fs`/`std::path` (grep stays empty).

## Failing Tests

Written at design, made to pass at build. "RED on `main`" is the reason each one fails today;
tests marked **guard** are green on `main` and must stay green.

- **`tests/recipe_strict_params.rs`** (new; library tests through `Recipe::from_toml` +
  `build_pipeline(&OperationRegistry::with_builtins())`, CLI tests drive `env!("CARGO_BIN_EXE_crustyimg")`
  the way `tests/apply_batch.rs` does)
  - `unknown_key_is_refused_on_every_op` — AC-1; 5 ops + the `widht` row. RED: all build `Ok`.
  - `terminal_optimize_marker_with_a_key_is_refused` — AC-2. RED: parses `Ok`.
  - `keyless_terminal_optimize_marker_still_parses` — AC-2's positive half. **Guard.**
  - `wrong_type_is_refused_for_every_key` — AC-3; 14 rows. RED: watermark's 7 optional rows build
    `Ok` with defaults; the required rows' `reason` lacks `must be`.
  - `integer_for_a_number_key_is_accepted` — AC-3's positive half. **Guard.**
  - `out_of_range_is_refused_for_every_ranged_key` — AC-4; 17 rows. RED: `margin = -5`,
    `percent = nan|inf`, `size = nan|inf`, `scale = nan` build `Ok`.
  - `key_unused_by_the_mode_is_refused` — AC-5; 6 rows. RED: all build `Ok`.
  - `documented_interplay_is_not_refused` — AC-5's positive rows. **Guard.**
  - `error_display_names_step_op_and_key` — AC-6. RED: step 1's `bogus` builds `Ok`.
  - `versions_1_and_2_validate_identically` — AC-9; the bad step is the unknown key
    `opactiy = 0.5`. RED: both build `Ok`.
  - `recipes_crustyimg_writes_are_never_refused` — AC-11(b). **Guard.**
  - `apply_single_input_refuses_a_wrong_typed_watermark_key` — AC-7(a). RED: exit 0, 1 output.
  - `apply_batch_refuses_an_unknown_key_before_any_output` — AC-7(b). RED: exit 0, 3 outputs.
  - `build_refuses_a_non_finite_watermark_size_before_any_output` — AC-8. RED: exit 0, 3 outputs.
- **`tests/recipe_round_trip.rs`**
  - `to_toml_is_byte_identical_to_main_over_the_corpus` — AC-11(a). **Guard** (goldens captured
    from `main` in the build's first commit).
  - `bless_recipe_corpus_goldens` — `#[ignore]`, writes the goldens only when
    `CRUSTYIMG_BLESS=1`. Not an assertion; the generator verify re-runs on `main`.
- **`src/recipe/mod.rs`** (unit)
  - `build_pipeline_rejects_unknown_step_param` — AC-13; replaces
    `from_toml_tolerates_unknown_step_param_by_design`. RED: today's pin asserts the opposite.
- **`src/operation/registry.rs`** (unit)
  - `every_builtin_publishes_a_param_schema` — AC-12(a). RED: `param_specs` does not exist.
  - `asset_keys_are_derived_from_the_schema` — AC-12(b). RED: does not compile (no schema).
  - `register_without_params_refuses_any_key` — AC-1's paramless mechanism. RED: builds `Ok`.
  - `one_of_values_are_accepted_by_their_constructors` — AC-12(e). RED: no `OneOf`.
- **`tests/registry_seam.rs`**
  - `a_lut_shaped_op_is_strict_and_resolvable_through_one_table` — AC-12(c). RED: no
    `register_with_params`.
  - `a_second_asset_op_registers_without_seam_change` — AC-12(d). **Guard, unedited.**
- **`tests/wasm_roundtrip.rs`** (wasm half)
  - `transform_refuses_an_unknown_step_key` — AC-10. RED: returns `Ok`.

**Negative controls (AC-14) — revert one condition, expect exactly these RED:**

| # | condition (revert only this) | flips RED | must stay GREEN |
|---|---|---|---|
| C1 | registry: unknown-key check | `unknown_key_…`, `register_without_params_…`, `apply_batch_…`, `build_pipeline_rejects_…`, `error_display_…`, `versions_1_and_2_…`, `transform_refuses_…`, `a_lut_shaped_op_…` | all C2–C6 tests |
| C2 | registry: type check | `wrong_type_…`, `apply_single_input_…` | C1, C3–C6 |
| C3 | registry: range + finiteness check | `out_of_range_…`, `build_refuses_a_non_finite_…`, `a_lut_shaped_op_…` (its `strength = 2.0` row) | C1, C2, C4–C6 |
| C4 | constructors: unused-by-mode check | `key_unused_by_the_mode_…` | the rest |
| C5 | `from_toml`: terminal-marker check | `terminal_optimize_marker_…` | the rest |
| C6 | `InvalidOperation`'s Display step prefix | `error_display_…`, the three CLI tests and `transform_refuses_…` (their prefix assertions) | every library table (they assert `reason`, not Display) |

Derived `asset_keys` is structural, not a behaviour condition — reverting it breaks every
asset-bearing test at once — so it is guarded directly by AC-12(b)/(c), not by a revert row.
Two rules make that table hold: the library tables assert on the `reason` field, never on
Display; and each CLI/wasm test uses the one condition its row names (the range test uses
`size = nan`, the single-input test a string `opacity`), so they are independent.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Files the build touches (best estimate — the orchestrator schedules on this)

- `src/operation/registry.rs` — `ParamSpec`/`ParamKind`, the schema check in `build`,
  `register_with_params`, `param_specs`, derived `asset_keys`; unit tests.
- `src/operation/mod.rs` — `RESIZE_PARAMS`/`WATERMARK_PARAMS` consts; re-export; `Resize::from_params`
  and `Watermark::from_params` (unused-by-mode checks, key-naming messages; the
  `.unwrap_or(default)` reads become absence-only by construction); unit tests. ⚠ **Shared with
  SPEC-133** (`Watermark`) — SPEC-131 builds first.
- `src/recipe/mod.rs` — `InvalidOperation.step`, Display, `build_pipeline` passes the index,
  terminal-marker check in `from_toml`, module doc's "Accepted" paragraph rewritten, pin flipped.
- Tests: `tests/recipe_strict_params.rs` (new), `tests/recipe_round_trip.rs`,
  `tests/fixtures/recipe_corpus/` (new), `tests/registry_seam.rs` (one new test appended),
  `tests/wasm_roundtrip.rs` (one new test). Possibly `tests/recipe_watermark.rs` only if an
  existing test turns out to rely on a now-refused key (none found by grep).
- Docs: `recipes/README.md`, `docs/data-model.md`, `docs/api-contract.md`, `CHANGELOG.md`.
- Decisions: `decisions/DEC-102-recipe-step-params-are-validated-at-load.md` (new),
  `decisions/DEC-061-proj007-threat-model-verdicts.md` (two one-line markers).
- **Not touched:** `src/wasm.rs`, `src/cli/**` (incl. `common.rs`, `mod.rs`, `build.rs`,
  `optimize.rs`, `ops.rs`), `src/build/**`, `tests/apply_batch.rs`, `tests/build.rs`. The CLI
  surface tests live in the new file on purpose, to stay off SPEC-134's files. The only overlap
  with SPEC-134 is `tests/wasm_roundtrip.rs` (an appended test — a trivial merge) and
  `CHANGELOG.md`/`docs/api-contract.md` (separate paragraphs). → **Can build in parallel with
  SPEC-132 and SPEC-134; must precede SPEC-133.**

### Decisions that apply

- `DEC-061` — accepted risk #1 is what this spec retires (maintainer ruling 1); DEC-102 records it.
- `DEC-014` — params stay an ordered map with per-op constructors; its promise of typed
  wrong-type errors is what the schema finally delivers.
- `DEC-005` — recipes round-trip through the parsed form; Call 2 pins byte stability.
- `DEC-031` / `DEC-064` — `src/operation/**` stays filesystem-free and wasm32-clean; `Asset` is a
  declaration, never a read.
- `DEC-100` — the asset seam this spec folds into the schema; its two entry points keep their
  signatures.
- `DEC-058` / `DEC-101` — cache key and lockfile hash `to_toml()`; unchanged for every valid recipe.
- `DEC-099` — versions gate `format`/`quality` only; step params are unversioned (Call 7).
- `DEC-036` — recipe resource limits; the `size` upper bound is filed against this family, not
  invented here.

### Constraints that apply

- `untrusted-input-hardening` — a recipe is untrusted input; refuse, never default, never panic
  (including on `nan`/`inf` and on `u32` overflow of a TOML `i64`).
- `test-before-implementation` — the goldens and the RED tests land before the `src/` change.
- `no-unwrap-on-recoverable-paths` — the schema check returns errors; no `expect` on user values.
- `decode-once-no-per-op-disk` — `Asset` keys are still resolved only at the CLI IO boundary.
- `one-spec-per-pr`.

### Prior related work

- `SPEC-068` / DEC-061 — added top-level `deny_unknown_fields` and recorded the step-level risk.
- `SPEC-128` / DEC-100 (PR #189) — registered `watermark`, added the asset seam and the
  `.unwrap_or(default)` reads this spec closes.
- `SPEC-129` / DEC-101 (PR #190, #191) — the cache key over `to_toml()` + asset content.
- `SPEC-127` / DEC-099 — the version-2 gate (shape for a field-naming refusal).

### Out of scope (for this spec specifically)

- A wasm `validate(recipe_toml)` export returning structured per-step errors (and "did you
  mean…?" suggestions) — **filed on STAGE-050 by this design**; consumes `param_specs`.
- An upper bound on watermark `size` (measured: `100000.0` → 52 s, exit 0) — **filed on
  STAGE-050 by this design** as a resource-limit item; the CLI `--size` shares it.
- Reordering wasm `transform` to check the recipe before decoding (Call 6).
- Typed structs as the serialized form (Call 1).
- `tile`/`center` interplay with `gravity`/`margin` (Call 4's line).
- Data-dependent per-input failures (e.g. `percent = 400` exceeding the edge limit on a large
  input) — not a malformed recipe.
- Watermark damage refusal and `--min-size` — SPEC-133 and its split-out item; they add keys
  under this schema (one line each).
- Missing asset files — still `RecipeAssetUnreadable`, unchanged. Order note: on `apply`/`build`
  the asset resolver runs before `build_pipeline`, so a step with both a missing `image` file
  and a bad key reports the missing file first. Not a guarantee anyone needs; unchanged.
- The direct CLI verbs' flags — clap types them.

## Notes for the Implementer

- **Start with the goldens** (Call 2), on an untouched tree, in their own commit. Verify will
  re-run the bless test on `main` and diff.
- **TOML integers are `i64`.** `UInt { min }` must refuse negatives and values above `u32::MAX`
  with the `must be an integer >= N` shape — `get_u32`'s silent `try_from(..).ok()` is exactly how
  `margin = -5` became 0.
- **`Number` accepts `Integer` and `Float`, and refuses non-finite floats.** `opacity = 1` must
  keep building and keep serializing as `1` (Call 2's corpus has it).
- **Describe the got-value:** `the string "0.5"`, `the float 100.0`, `the integer 5`, `true`,
  `an array`, `a table`, and bare `nan`/`inf`/`-5` for out-of-range numbers.
- **Check order inside a step:** iterate the map in its (sorted) order; for each key, unknown →
  type → range; first failure wins. Then the constructor.
- **Constructor messages must name the key** — `color` currently surfaces `TextError`'s text
  without saying `'color'`; wrap it as `'color' must be a hex colour RRGGBB or RRGGBBAA, got
  "xyz"`.
- **Keep `register`/`register_with_assets` signatures.** `tests/registry_seam.rs`'s existing test
  is the guard that the old entry points still work; do not edit it.
- **Run the clean matrix sequentially, each leg in its own `CARGO_TARGET_DIR`**, and redirect test
  output to a file and read `$?` — never `cargo test | tail` (a pipe reports the pipe's code).
- **Push a WIP commit once it compiles**, before the matrix.
- **Comments and user-facing text are behaviour-first** — no spec/DEC ids in error strings or doc
  comments.
- Every commit `git commit -s`.

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
