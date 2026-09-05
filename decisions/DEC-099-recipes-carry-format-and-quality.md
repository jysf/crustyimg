---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-099
  type: decision
  confidence: 0.88
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-09-04
supersedes: null
superseded_by: null

affected_scope:
  - src/recipe/mod.rs
  - src/cli/ops.rs
  - src/cli/optimize.rs
  - src/cli/build.rs
  - src/wasm.rs
  - docs/api-contract.md
  - docs/data-model.md

tags:
  - recipe
  - format
  - quality
  - apply
  - build
  - wasm
  - dec-015
  - dec-098
---

# DEC-099: `Recipe` gains `format`/`quality`, gated behind `version = "2"`; one precedence chain across `apply`, `build`, `wasm::transform`

## Decision

`Recipe` gains two optional fields, `format: Option<String>` and `quality: Option<u8>`. Setting
either **requires** `version = "2"`; a `"1"` recipe that sets one is rejected with a new typed
error (`RecipeError::NewFieldNeedsVersion2`) naming the field and the version found, not a generic
`deny_unknown_fields` TOML parse error. `"1"` stays fully valid, and `to_toml` keeps emitting
`version = "1"` for any recipe that uses neither new field — the version is never bumped
unconditionally.

One precedence chain, extending DEC-015: **`--format` > a recognized `-o` extension (`apply`,
single input only) > `recipe.format` > preserve the source format.** Quality follows the same
shape: **`-q` > `recipe.quality` > the format's own default.** Both rungs are resolved at the CALL
SITE in `apply`/`build` (never inside `encode_one`), matching the seam SPEC-126/DEC-098 already
established. `build` has neither `--format`/`-o` nor a per-target `-q` (DEC-098), so
`recipe.format`/`recipe.quality` are its only way to vary either per target — a target's global
`-q`, when given, still applies uniformly across every target.

A terminal-`optimize` recipe (the bundled `web`/`gallery`/`product` shape) that ALSO declares
`format` is a carve-out: the explicit field wins and the auto-decision is skipped entirely,
matching what a `--format`/`-o` pin (`apply`) or a literal-extension name template (`build`)
already do on that path (DEC-087). `wasm::transform` grows the same rung: its `out_format`
parameter is the CLI-flag equivalent and wins; an empty `out_format` (the same "no override"
sentinel `optimize`/`optimize_detailed` already use in that module) defers to `recipe.format`, and
`recipe.quality`, when set, reaches the encoder the same way — there is no CLI-flag equivalent for
quality on that surface, so it is the only rung below the format's own default.

## Context

Driven on `main` at `7181eed` (0.7.1 + SPEC-126), with a recipe emitted by `edit --save-recipe`:

| recipe | exit | message |
|---|---|---|
| valid `version = "1"` (control) | **0** | — |
| v1 **plus an unknown `format` key** | **1** | `could not parse recipe TOML: TOML parse error at line 2, column 1` |
| `version = "2"`, no unknown key | **1** | `unsupported recipe version '2' (supported: 1)` |

`Recipe` was `#[serde(deny_unknown_fields)]`, so the old binary rejected any recipe carrying a
`format` key with a TOML parse error pointing at an arbitrary line — no hint that a newer
crustyimg was needed. The version check, given a clean recipe, already produced the right
message. That asymmetry is what decides the version gate below: making `format`/`quality`
optional v1 fields (`skip_serializing_if`) would have kept existing recipes serializing
unchanged, but it is the WRONG answer — it throws away the one case (`version = "2"`) that
already produces an actionable message, in favor of the one case that doesn't.

`apply --recipe` and `build` only agree on what "format" means as of STAGE-049 (SPEC-126,
DEC-098): before that, adding a schema field for it would have baked a disagreement into the
schema itself. STAGE-050 unblocked this spec once that landed.

## Alternatives Considered

### Call 1 — the version gate

- **`format`/`quality` as plain optional v1 fields.** Rejected: measured above — an old binary
  handed a v1-plus-`format` recipe fails with a parse error at an arbitrary line, not a message
  that says "upgrade crustyimg." A recipe is a file people commit and share, and crustyimg has two
  shipped releases in the wild, so a forward recipe **will** meet an old binary eventually.
- **Bump `to_toml` to always emit `version = "2"`.** Rejected outright: this would strand every
  recipe currently in the wild the next time it is resaved via `--save-recipe`, and the
  correctness would look fine right up until an old binary tried to read the "upgraded" file.
- **Chosen: gate `format`/`quality` behind `version = "2"`, checked as a domain rule in
  `from_toml` (not via serde), leaving `to_toml` to serialize whatever the struct's `version`
  field already holds.** A v1 recipe that never touches the new fields is untouched byte-for-byte
  by this spec; a v1 recipe that sets one gets a message naming the field and the version.

### Call 2 — precedence

- **Insert `recipe.format` above `-o`/`--format`.** Rejected: a flag typed at the call site is a
  deliberate, immediate override of a file's default; ranking a file's default above it would be
  backwards from every other layered default in this CLI (DEC-098's own reasoning, one level
  down).
- **A second, independent chain for recipes, parallel to `ops::output_format_for`.** Rejected:
  this is exactly the class of mistake SPEC-126 fixed (two resolution paths that can drift). The
  existing function is widened with one new rung instead.
- **Chosen: extend DEC-015's existing chain with `recipe.format`/`recipe.quality` as new,
  lowest-but-one rungs**, resolved at each caller (`apply`'s single- and multi-input branches,
  `build`'s `prepare_target`) rather than inside `encode_one`.

### Call 2's carve-out — terminal `optimize` + an explicit `format`

- **Let the auto-decision always win, even over an explicit `format`.** Rejected: a recipe that
  both ends in `optimize` and declares `format` is asking for two contradictory things (auto-pick
  vs. a pin); ignoring the explicit field would silently discard user intent.
- **Chosen: the explicit field wins, the decision is skipped** — identical to what a `--format`/
  `-o` pin (`apply`) or a literal-extension template (`build`) already do on this exact path
  (DEC-087). One behavior, not a second one bolted on.

### Call 3 — `wasm::transform`

- **Leave `out_format` mandatory and ignore `recipe.format` entirely.** Rejected: the whole point
  of a precedence rule is that the wasm surface obeys the same sentence as the native one: a
  recipe tuned with a `format` should behave identically in the browser and at the terminal.
- **Chosen: `out_format` is the override and wins; an empty string (the same "no override"
  sentinel `optimize`/`optimize_detailed` already use in `src/wasm.rs`) defers to `recipe.format`.**
  Neither present is a typed error, not a silent guess. `recipe.quality` threads to the encoder
  the same way, with no CLI-flag equivalent to rank above it on this surface.

### Call 4 — typed per-operation parameter structs

Not built here. `OperationParams` stays a `BTreeMap<String, toml::Value>`; typing it touches every
op in the registry and has no dependency in either direction on this schema change. Filed back to
STAGE-050's backlog as its own `[M]` item (splitting an L into two, per AGENTS §2).

## Consequences

- **Positive:** `apply --recipe`, `build`, and the wasm `transform` binding now honor one
  precedence rule for format AND quality, including the terminal-`optimize` carve-out. A recipe
  can pin its own output, closing the gap the spec's `value_link` names (`apply --recipe` now
  reaches what `convert` reaches).
- **Byte-changing, but only for recipes that opt in (AC-8, verified — see Validation).** A v1
  recipe, and every existing verb this spec did not touch, produce byte-identical output to
  `main`. This still batches into PROJ-011's single lockfile migration with the rest of
  STAGE-050 — no crate version bump, no release cut in this PR.
- **`build`'s cache key needs no new code to stay correct.** `crate::build::cache::recipe_hash`
  hashes the recipe's own canonical TOML (`recipe.to_toml()`), and `format`/`quality` are now part
  of that struct — so a target's effective cache key already changes when either field changes,
  with zero additional cache-key plumbing. Verified structurally (the hash is computed over the
  post-`Recipe`-struct-change serialization) rather than by adding a redundant explicit component.
- **`ops::output_format_for` gained a fourth parameter** (`recipe_format: Option<&str>`),
  `pub(super)`. Its only recipe-aware caller is `optimize::run_apply`'s single-input branch; every
  other caller (`resize`/`thumbnail`/`watermark`/`edit`/`auto-orient`, via `run_pixel_op`) passes
  `None`, leaving its own resolution byte-for-byte unchanged.
- **`build.rs`'s `OutputFormatPlan::Pinned` variant is now reached by two independent routes** —
  a literal-extension name template (SPEC-111, unchanged) and a recipe's own `format` (new). Both
  collapse to the same `encode_one(..., Some(fmt), ...)` call, so there is no new code path to
  drift, only a new way to arrive at the existing one.

## Validation

- **AC-1** (round-trip): `tests/recipe_v2.rs::v2_round_trips_format_and_quality` and
  `::v1_still_round_trips_and_stays_v1` (the strand guard — `to_toml` must keep emitting
  `version = "1"` for a recipe using neither field). Both **driven RED against pristine `main`
  first** (the fields/gate did not exist), then made to pass.
- **AC-2** (`apply` honours `recipe.format` at 1 and N inputs): two separate tests —
  `tests/apply_batch.rs::apply_honours_recipe_format_at_one_input` and
  `::apply_honours_recipe_format_at_n_inputs` — split per SPEC-126's own re-approve finding (a
  combined test dies at the first failing arity). Asserted on the written bytes via
  `image::guess_format`, never the extension.
- **AC-3** (precedence both directions): `tests/apply_batch.rs::cli_format_overrides_recipe_format`
  — `--format` wins over a recipe pinning a different format.
- **AC-4** (`apply`/`build` byte-identical on a v2 recipe):
  `tests/apply_batch.rs::apply_and_build_agree_on_v2_recipe` — a recipe declaring BOTH `format`
  and `quality`, no CLI overrides anywhere, asserted on raw bytes.
- **AC-5** (new field without v2 is rejected): `tests/recipe_v2.rs::new_field_without_v2_is_rejected`
  — both the `format` and `quality` halves, asserted on the typed `RecipeError::NewFieldNeedsVersion2`
  variant, not just a non-zero exit.
- **AC-6** (terminal-`optimize` carve-out): `tests/optimize.rs::terminal_optimize_honours_an_explicit_recipe_format`
  — a bundled-shaped recipe (auto-orient + resize + terminal `optimize`) with an explicit
  `format = "png"` pin is asserted to sniff as PNG; the identical recipe with no `format` is
  asserted to NOT sniff as PNG (a lossless re-encode of real photographic content is essentially
  never the auto-decide winner on any feature leg, so this is robust across the AC-9 matrix, not
  a coincidence of which codecs happen to be compiled in).
- **AC-7** (negative controls, one revert per independent condition — see below).
- **AC-8** (nothing else changes bytes): driven end to end, not reasoned about. Built `main`
  (`a0cda1f`) and this branch as separate release binaries (fresh, isolated `CARGO_TARGET_DIR`s
  via a temporary detached worktree at `main`'s commit, removed after). Ran `resize --max 64`,
  `thumbnail`, `watermark --image <overlay>`, `convert --format webp`, `optimize`, `web`,
  `responsive --widths 64,128` (single input), a **v1** recipe through `apply` (batch), and the
  same v1 recipe through `build` (a 2-target manifest, `*.png`/`*.jpg` globs) — over a 4-file,
  2-format corpus (`tests/fixtures/c2pa/signed.png`,
  `tests/fixtures/classify/checker_graphic.jpg`, `tests/fixtures/classify/color_photo_fuji.png`,
  `tests/fixtures/optimize/already_compressed.jpg`). **All 9 checks matched byte-for-byte**
  (`diff -rq` empty) on both binaries. **Positive control:** a `version = "2"` recipe (declaring
  `format`/`quality`) handed to `apply` — `main` rejects it (`exit 1`,
  `could not parse recipe TOML: ... unknown field 'format'`), this branch accepts it (`exit 0`,
  real output) — confirming the comparison methodology can detect a real difference, on the exact
  axis this spec changes.
  ⚠ **Corpus boundary, stated rather than implied:** the run covers the seven named verbs plus
  `apply`/`build` on a v1 recipe, on 4 files in 2 formats (PNG, JPEG). It does **not** cover every
  `(verb × format × flag)` combination — the full conformance matrix stays PROJ-010's job
  (`brief.md`'s Out of Scope). The hypothesis under test was "did a shared seam move for a v1
  recipe or an unrelated verb", and the call graph bounds which seams could: `output_format_for`'s
  new parameter defaults to `None` at every non-recipe-aware call site, `build`'s
  `OutputFormatPlan::Preserve`/`Pinned`-via-template arms are unchanged when `recipe.format` is
  `None`, and `wasm::transform`'s `out_format` behavior for a non-empty string is unchanged.
- **AC-9** (clean matrix): see the spec's `## Build Completion` for the per-leg outcome, plus
  `just wasm-check`.

### AC-7's negative controls

Reverted each of the three conditions ALONE (on top of the same commit, restored via
`git checkout -- <file>` between each), rebuilt, and re-ran the affected suites. The evidence is
the behavioural flip, never a hash (AGENTS §15). **Measured, not assumed** — the actual results
below, including one genuine asymmetry that is worth stating rather than smoothing over:

- **Call 2 (precedence, `src/cli/ops.rs` + `src/cli/optimize.rs` + `src/cli/build.rs`) reverted
  alone** (`recipe.format`/`recipe.quality` made inert at every call site; Call 1's schema and
  Call 3's wasm code left untouched): `tests/apply_batch.rs::apply_honours_recipe_format_at_one_input`,
  `::apply_honours_recipe_format_at_n_inputs`, and `::apply_and_build_agree_on_v2_recipe`, plus
  `tests/optimize.rs::terminal_optimize_honours_an_explicit_recipe_format`, go **RED** (recipe.format
  is silently ignored: single-input writes `.png` when `.jpg` is expected — "one.png must be
  written: No such file or directory"; the terminal-`optimize` carve-out auto-decides AVIF instead
  of honouring the PNG pin). `cli_format_overrides_recipe_format` stays **GREEN** (`--format`
  already won unconditionally, so it cannot tell the two states apart — a correctly-scoped
  observation, not a gap). `tests/recipe_v2.rs`'s 3 tests stay **GREEN**, and all 41
  `wasm_roundtrip.rs` tests stay **GREEN** — confirming Call 2 is independent of Call 1's schema
  gate and of Call 3's wasm wiring.
- **Call 3 (wasm, `src/wasm.rs::transform` only) reverted alone** (restored to its pre-spec
  3-statement body: parse `out_format` unconditionally, `quality: None` always): exactly 2 of the 3
  new wasm tests go **RED** —
  `tests/wasm_roundtrip.rs::transform_falls_back_to_recipe_format_when_out_format_empty` and
  `::transform_honours_recipe_quality` — both call `transform` with an empty `out_format` (asking
  it to fall back to the recipe's own `format`/`quality`), and both now fail with the SAME clean
  typed error, `unsupported output extension` (from `parse_format("")`), rather than either falling
  back or panicking — the reverted code is byte-for-byte `main`'s pre-spec `transform`, so this is
  the expected, honest failure mode, not a new one this spec introduced.
  ⚠ **`::transform_errors_when_neither_format_is_given` stays GREEN in BOTH states** — an empty
  `out_format` is already an error with no `recipe.format` involved at all (a bare `parse_format("")`
  fails either way), so this one test cannot by itself discriminate Call 3 present vs. reverted;
  the other two tests are what carries the control. All native tests (`recipe_v2.rs`,
  `apply_batch.rs`, `optimize.rs`) stay **GREEN** — confirming Call 3 is independent of the two
  native calls.
  ⚠ `just wasm-test` runs in **no CI job** — this negative control, like the wasm assertions
  themselves, is not covered by the required matrix (AC-9). Driven manually in this build cycle.
- **Call 1 (version gate, `src/recipe/mod.rs::from_toml`'s version check) reverted alone**
  (restored to accepting only `SUPPORTED_VERSION = "1"`, no `NewFieldNeedsVersion2` rule; Calls 2
  and 3's code left untouched): `tests/recipe_v2.rs::new_field_without_v2_is_rejected` goes **RED**
  as its own direct guard (a v1-plus-`format` recipe now parses instead of erroring) — but so does
  **every other v2-dependent test**: `::v2_round_trips_format_and_quality`,
  `apply_batch.rs`'s four new tests (including `cli_format_overrides_recipe_format`, which passed
  under the Call-2 revert but not this one), and `optimize.rs`'s carve-out test, ALL fail with
  `unsupported recipe version '2' (supported: 1)` — because a `version = "2"` recipe can no longer
  even be constructed to exercise Calls 2/3 at all. Only `::v1_still_round_trips_and_stays_v1`
  stays **GREEN**. ⚠ **This is a real, worth-stating asymmetry, not a control that failed:** Call 1
  is a genuine PRECONDITION for Calls 2/3 being reachable through a real recipe file, not a
  parallel independent branch the way Calls 2 and 3 are independent of EACH OTHER (as the two
  reverts above demonstrate — reverting either one leaves the other's tests, and Call 1's own
  schema tests, green). The meaningful independence claim for Call 1 is narrower and still holds:
  reverting Call 2 or Call 3 alone never flips Call 1's OWN regression guards
  (`v1_still_round_trips_and_stays_v1`, `v2_round_trips_format_and_quality`,
  `new_field_without_v2_is_rejected` — all three stayed green under both other reverts), proving
  Calls 2/3 are downstream consumers of Call 1's schema, not entangled with its own validation
  logic.

## References

- Related specs: **SPEC-127** (this decision's spec), **SPEC-126** (the `apply`/`build` format
  agreement this spec's schema field depends on), **SPEC-111** (`OutputFormatPlan`, the terminal-
  `optimize` carve-out's build twin), **SPEC-112** (`split_terminal_optimize` living in
  `src/recipe/mod.rs` so both `cli` and `wasm` can reach it).
- Related decisions: **DEC-015** (the precedence chain this spec extends), **DEC-098** (`apply`
  moves, `build` does not — the seam this spec's new rung reuses rather than reimplements),
  **DEC-087** (`build`'s name template is the format pin — the carve-out's build-side twin),
  **DEC-005** (recipes round-trip through the registry), **DEC-058** (the build cache key — the
  recipe-hash mechanism that already covers `format`/`quality` with no new code).
- Code: `src/recipe/mod.rs` (`Recipe`, `RecipeError::NewFieldNeedsVersion2`, `SUPPORTED_VERSION_2`),
  `src/cli/ops.rs` (`output_format_for`), `src/cli/optimize.rs` (`run_apply`), `src/cli/build.rs`
  (`prepare_target`, `build_one`), `src/wasm.rs` (`transform`).
- Stage: `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-050-recipe-reach.md`.
