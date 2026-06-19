---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-032
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-005
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet (prescriptive prompt)
  created_at: 2026-06-19

references:
  decisions: [DEC-005, DEC-015, DEC-007, DEC-031]
  constraints:
    - no-new-top-level-deps-without-decision
    - clippy-fmt-clean
    - every-public-fn-tested
    - no-unwrap-on-recoverable-paths
  related_specs: [SPEC-006, SPEC-007, SPEC-011, SPEC-031]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-005's <capability>". Optional; null is acceptable.
value_link: >
  Completes the project thesis — `edit` tunes an ordered op chain on one
  image and `--save-recipe` captures it as the exact TOML that `apply`
  replays across a directory: tune once → save → replay.

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
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: "edit + --save-recipe: clap op flags + run_edit + build_edit_ops (canonical order auto-orient→resize→invert) reusing run_pixel_op + registry + Recipe::from_ops/to_toml; recipe round-trips through apply; no new dep/op"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 1
---

# SPEC-032: edit one-shot multi-op and save-recipe

## Context

**The last STAGE-005 spec — and the half that completes the thesis.** SPEC-031
made `apply --recipe` a parallel batch *replay*. This spec adds the *creation*
counterpart: `edit` builds an ordered operation list from CLI flags, runs it
once on a single image (decode-once → ops → encode), and — with
`--save-recipe FILE` — serializes that exact chain to a TOML recipe via the
operation registry (DEC-005) so `apply --recipe FILE` reconstructs an identical
pipeline. With this, "tune an edit once on one image, save it, replay it across
a whole directory" is two commands sharing one recipe format.

Everything this needs already exists and is shipped: the registry round-trip
(`Recipe::from_ops` / `to_toml`, SPEC-006), the registered ops
(`identity`/`invert`/`resize`/`auto-orient`), and the single-input
load→pipeline→sink fan-out (`run_pixel_op`, SPEC-011/013). `edit` is currently a
clap stub (`Commands::Edit { input, save_recipe }` → `NotImplemented`). This
spec wires it. Parent: `STAGE-005-batch-and-recipes` (backlog item #1, the
remaining one). Governing: **DEC-005** (recipe round-trip via the registry),
**DEC-015** (sink/format/quality fan-out), **DEC-007** (typed errors).

**No new external crate, no new DEC expected** — it composes the registry ops +
recipe serialization, both already in the tree.

## Goal

Implement `edit <input> [op flags…] [-o OUT] [--save-recipe FILE]`: build an
ordered op list from the op flags (via the registry, in a canonical order), run
it once on the resolved image and write the result through the normal sink, and
— when `--save-recipe` is given — write the equivalent TOML recipe that `apply`
replays identically (DEC-005 round-trip).

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — `Commands::Edit` (the stub to wire), `dispatch` (the
    `NotImplemented("edit")` arm to replace), `run_pixel_op` (the
    single-input load→run→sink path to REUSE), `run_resize` / `resize_params`
    (how a `--resize-max N` maps to registry `resize` params — MIRROR exactly),
    `OperationRegistry::with_builtins`, `GlobalArgs`, `CliError`.
  - `src/recipe/mod.rs` — `Recipe::from_ops(&[Box<dyn Operation>]) -> Recipe`,
    `Recipe::to_toml() -> Result<String, RecipeError>`, `RecipeError::Serialize`.
  - `src/operation/mod.rs` — the `Operation` trait (`name`, `params`),
    `OperationParams::{empty, from_map}`, `Identity`/`Invert`/`Resize`/`AutoOrient`.
  - `src/operation/registry.rs` — `OperationRegistry::build(name, &params)`,
    `RegistryError::{Unknown, InvalidParams}`.
  - `src/sink/mod.rs` — `SinkError::Io` (for `--save-recipe` write failures → exit 5).
  - `decisions/DEC-005`, `DEC-015`, `DEC-007`, `DEC-031`.
- **External APIs:** none new. (`toml` is already a dependency via `recipe`.)
- **Related code paths:** `src/cli/mod.rs`, `tests/`.

## Outputs

- **Files modified:**
  - `src/cli/mod.rs` — extend `Commands::Edit` with the op flags; replace the
    `NotImplemented("edit")` dispatch arm with `run_edit(...)`. Add `run_edit`
    plus a pure helper `build_edit_ops(...) -> Result<Vec<Box<dyn Operation>>,
    CliError>` (unit-tested). No new public exports.
  - `docs/api-contract.md` — flesh out the `edit` entry (flag surface, canonical
    op order, `--save-recipe`, exit codes). (Done at design.)
- **Files created:** `tests/edit.rs` — integration tests for `edit` +
  `--save-recipe` + the round-trip-into-`apply`.
- **New exports:** none (internal `fn`s only).
- **Database changes:** none.

## Command surface (PINNED)

```
crustyimg edit <INPUT> [--auto-orient] [--resize-max N] [--invert] \
    [-o OUT | --out-dir DIR] [--format FMT] [-q Q] [-y] [--save-recipe FILE]
```

- **Op flags (v1 — only registry ops that round-trip through `with_builtins`):**
  - `--auto-orient` (bool) → registry op `"auto-orient"` (empty params).
  - `--resize-max N` (`Option<u32>`) → registry op `"resize"` with params
    `{ mode = "max", width = N }` — built **exactly** the way `resize_params(Some(N),
    …)` builds them, so the saved recipe is byte-identical to a hand-written
    `resize/max` step.
  - `--invert` (bool) → registry op `"invert"` (empty params).
- **Canonical op order (PINNED):** regardless of the order the flags appear on
  the command line, ops are appended in this fixed order:
  **`auto-orient` → `resize` → `invert`** (orientation normalization → geometry
  → color). This makes the op list — and therefore the saved recipe —
  deterministic and independent of flag position. Document it in the command
  help / api-contract.
- **At least one op flag is REQUIRED.** Zero op flags → `CliError::Usage`
  (exit 2): `"edit requires at least one operation flag (--auto-orient,
  --resize-max, --invert)"`. (An `edit` with no ops is just a re-encode — that's
  `convert`'s job — and a no-op `--save-recipe` is not worth a special case.)
- **Input resolution & output:** `edit` reuses `run_pixel_op` for the
  load→run→write path, so it inherits the established single-input behavior:
  resolve `INPUT` via `source::resolve`, load, run the pipeline, and write to
  `-o PATH` / `-o -` (stdout) / `--out-dir DIR` (templated) / else stdout, with
  per-input format via `output_format_for` (`--format` › `-o` extension ›
  preserve source) and `-q`/`-y` honored. The typical use is one image; a glob
  that resolves to many fans out exactly like the other pixel commands
  (`--out-dir` then required, exit 6 on partial failure) — no special-casing.
- **`--save-recipe FILE`:** after a successful edit+write, serialize the op
  chain and write it to `FILE`:
  - Build the recipe from the SAME ops the pipeline ran:
    `Recipe::from_ops(&ops)` → `recipe.to_toml()?` (serialization failure →
    `CliError::Recipe`, exit 1).
  - Write the TOML to `FILE` (`std::fs::write`); an I/O error →
    `CliError::Sink(SinkError::Io(e))` (exit 5 — output write failed). Overwrites
    `FILE` if it exists (recipe files are derived artifacts, not user images —
    no overwrite guard).
  - The recipe carries `version = "1"` and no `name`/`description` (DEC-005,
    `from_ops`).

## Round-trip guarantee (PINNED — DEC-005)

The whole point: a recipe saved by `edit` must reconstruct the identical op
list in `apply`. This holds **because `edit` builds its ops through the same
registry `apply` loads them through:**

- `edit` constructs each op via `OperationRegistry::with_builtins().build(name,
  &params)` (never by `new`-ing a concrete op type) — identical to how
  `run_resize` builds its op and how `Recipe::build_pipeline` rebuilds one.
- `Recipe::from_ops` records each op's intrinsic `name()` + `params()`.
- `apply` (`Recipe::from_toml` → `build_pipeline(&with_builtins())`) resolves
  those names+params back through the same constructors.

So `edit IN --auto-orient --resize-max 800 --invert --save-recipe r.toml` then
`apply --recipe r.toml IN -o out2.png` produces the same pixels as the direct
`edit` output. **An integration test pins this** (compare `edit` output bytes to
`apply`-of-the-saved-recipe output bytes).

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] `edit in.png --resize-max 8 -o out.png` writes `out.png` resized to a max
  edge of 8; exit 0.
- [ ] `edit in.png --auto-orient --resize-max 8 --invert -o out.png` applies all
  three ops in the canonical order (auto-orient → resize → invert); exit 0.
- [ ] Flag order on the command line does NOT change the result:
  `edit … --invert --resize-max 8` and `edit … --resize-max 8 --invert` produce
  identical output bytes (canonical order is positional-independent).
- [ ] `edit in.png --resize-max 8 --save-recipe r.toml -o out.png` writes BOTH
  `out.png` and a valid `r.toml`; `r.toml` parses via `Recipe::from_toml` and
  contains a `resize`/`max`/`width=8` step and `version = "1"`.
- [ ] **Round-trip:** the bytes from `apply --recipe r.toml in.png -o out2.png`
  equal the bytes from the original `edit … -o out.png` (same ops, same order).
- [ ] `edit in.png -o out.png` with NO op flag → exit 2 with the "requires at
  least one operation flag" usage message.
- [ ] `edit missing.png --invert -o out.png` (nonexistent input) → exit 3
  (input not found — inherited from `run_pixel_op`).
- [ ] `--save-recipe` to an unwritable path (e.g. a nonexistent directory) →
  exit 5 (output write failed); the recipe op-building itself never panics.
- [ ] `edit` appears in `crustyimg --help` and `crustyimg edit --help` lists the
  op flags + `--save-recipe`.
- [ ] `cargo deny` green; the **lean build** (`--no-default-features`) compiles;
  no new dependency added.

## Failing Tests

Written during **design**, BEFORE build. Generate fixtures natively (small PNGs
via the `image` crate; outputs/recipes in a tempdir). Mirror the fixture style
in `tests/apply_batch.rs` (`write_png`, `env!("CARGO_BIN_EXE_crustyimg")`).

- **`src/cli/mod.rs` (unit, `#[cfg(test)] mod tests`)**
  - `"edit_ops_canonical_order"` — `build_edit_ops` with all three flags set
    (in any caller arg order) returns ops named `["auto-orient", "resize",
    "invert"]` in that order.
  - `"edit_ops_subset_order"` — `build_edit_ops(resize_max=Some(8), invert=true,
    auto_orient=false)` returns `["resize", "invert"]` (only the requested ops,
    still canonical order).
  - `"edit_ops_requires_at_least_one"` — `build_edit_ops` with no flags set →
    `Err(CliError::Usage(_))` (the "requires at least one operation" path).
  - `"edit_ops_resize_params_match_resize_command"` — the `resize` op built by
    `build_edit_ops(resize_max=Some(16), …)` has the same `params()` as the op
    `run_resize` would build for `--max 16` (i.e. `{mode:"max", width:16}`),
    pinning the round-trip equivalence.
- **`tests/edit.rs` (integration, drives the binary)**
  - `"edit_resize_writes_output"` — `edit in.png --resize-max 8 -o out.png` →
    `out.png` exists, decodes, max edge == 8; exit 0.
  - `"edit_no_ops_exits_2"` — `edit in.png -o out.png` (no op flag) → exit 2.
  - `"edit_missing_input_exits_3"` — `edit nope.png --invert -o out.png` → exit 3.
  - `"edit_save_recipe_writes_parseable_toml"` — `edit in.png --resize-max 8
    --save-recipe r.toml -o out.png` → `r.toml` exists and contains
    `version = "1"`, `op = "resize"`, `width = 8`.
  - `"edit_save_recipe_round_trips_through_apply"` — run `edit in.png
    --auto-orient --resize-max 8 --invert --save-recipe r.toml -o edit_out.png`,
    then `apply --recipe r.toml in.png -o apply_out.png`; assert
    `edit_out.png` bytes == `apply_out.png` bytes.
  - `"edit_flag_order_independent"` — `edit … --invert --resize-max 8 -o a.png`
    and `edit … --resize-max 8 --invert -o b.png` produce identical bytes.
  - `"edit_save_recipe_unwritable_exits_5"` — `--save-recipe
    no_such_dir/r.toml` → exit 5.
- **`tests/cli.rs`** — `edit` already in the subcommand surface; confirm it's
  listed, add an op-flag-help assertion only if the file's style invites it.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-005` — recipes round-trip through the operation registry. `edit` MUST
  build ops via `OperationRegistry::with_builtins().build(name, &params)` (not by
  constructing concrete op structs) so the saved recipe replays identically under
  `apply`.
- `DEC-015` — single/multi sink fan-out, per-input format resolution, `-q`/`-y`
  semantics, exit-6 partial-batch. Inherited for free by reusing `run_pixel_op`.
- `DEC-007` — typed errors; no `unwrap`/`expect`/`panic!` off test paths. Map
  registry `InvalidParams` → `CliError::Usage` (exit 2, like `run_resize`); recipe
  serialize failure → `CliError::Recipe` (exit 1); recipe file write failure →
  `CliError::Sink(SinkError::Io)` (exit 5).
- `DEC-031` — watermark (and other STAGE-004 compose ops) are NOT in
  `with_builtins`, so they cannot round-trip through a recipe yet. `edit` does
  NOT expose `--watermark`/`--text` in v1 (would need registry wiring first; out
  of scope — a separate spec).

### Constraints that apply

These apply to the paths touched (see `/guidance/constraints.yaml`):

- `no-new-top-level-deps-without-decision` — none needed; composes existing code.
- `clippy-fmt-clean`, `every-public-fn-tested`, `no-unwrap-on-recoverable-paths`.

### Prior related work

- `SPEC-006` (shipped) — `Recipe::{from_ops, to_toml, from_toml, build_pipeline}`
  + the registry. **Reused as-is; do NOT modify the recipe layer.** `from_ops`
  already records `name()`+`params()`; `to_toml` already maps failures to
  `RecipeError::Serialize`.
- `SPEC-007` (shipped) — the `Commands::Edit` clap stub + `dispatch`.
- `SPEC-011`/`SPEC-013` (shipped) — `run_pixel_op`: the single-input
  load→run→sink path `edit` reuses; `run_resize`/`resize_params`: the exact
  `--resize-max → {mode,width}` mapping to mirror.
- `SPEC-031` (shipped) — the replay half (`apply --recipe`); the round-trip
  partner this spec completes.

### Out of scope (for this spec specifically)

- `--watermark`/`--text` (or any STAGE-004 compose op) in `edit` — needs the op
  registered in `with_builtins` first (DEC-031). Separate spec if wanted.
- Additional resize modes in `edit` (`--resize-exact WxH`, `--resize-percent`,
  fit/fill/cover) — additive later; v1 ships `--resize-max` only to keep the flag
  surface tight. (Each is just another flag mapping to the same registry `resize`
  op, so they extend cleanly.)
- A recipe `name`/`description` from the CLI (e.g. `--recipe-name`) — `from_ops`
  leaves them `None`; not needed for the round-trip.
- TUI live-preview editor (post-MVP, `docs/backlog.md`).
- Security-grade recipe/path hardening, decode limits — STAGE-006.

## Notes for the Implementer

- **Smallest correct change.** The clap `Edit` variant already has `input` and
  `save_recipe`; add three op flags (`auto_orient: bool`, `resize_max:
  Option<u32>`, `invert: bool`). Replace the `Commands::Edit { .. } =>
  Err(CliError::NotImplemented("edit"))` arm with a destructure + `run_edit(...)`
  call.
- **`build_edit_ops` is the pure, unit-tested core.** Signature roughly:
  `fn build_edit_ops(auto_orient: bool, resize_max: Option<u32>, invert: bool) ->
  Result<Vec<Box<dyn Operation>>, CliError>`. Body: a `with_builtins()` registry,
  push ops in the canonical order (auto-orient, then resize if `Some`, then
  invert), each via `registry.build(...)`. For `resize`, reuse `resize_params(
  Some(n), None, None, None, None, None)?` to get the params, then
  `registry.build("resize", &params)` mapping `RegistryError::InvalidParams` →
  `CliError::Usage` (mirror `run_resize`). Empty list → `CliError::Usage`.
- **`run_edit` flow** (order matters for the round-trip + write-after-success):
  1. `let ops = build_edit_ops(auto_orient, resize_max, invert)?;`
  2. Build the recipe object NOW, before moving the ops into the pipeline (only
     if `--save-recipe` was given): `let recipe = save_recipe.as_ref().map(|_|
     Recipe::from_ops(&ops));`
  3. Fold the ops into a `Pipeline` (`ops.into_iter().fold(Pipeline::new(), |p,
     op| p.push(op))`).
  4. `run_pixel_op(pipeline, std::slice::from_ref(input), global, global.quality,
     None, None)?;` — applies + writes (single image; inherits all sink/format/
     exit-code behavior). The `input: &String` becomes a one-element slice.
  5. On success, if `(save_recipe, recipe)` are both `Some`, `recipe.to_toml()?`
     and `std::fs::write(path, toml).map_err(|e|
     CliError::Sink(SinkError::Io(e)))?`.
- **Do NOT** re-implement load/sink/format logic — `run_pixel_op` already does it
  and is well-tested. The new surface is just: flags → ops (canonical order) →
  optional recipe serialization.
- **`Recipe::from_ops` borrows `&[Box<dyn Operation>]`**, so capture the recipe
  before the `into_iter().fold` that consumes `ops`. (Building the recipe object
  is cheap and image-independent; serializing/writing happens after the edit
  succeeds so a failed edit leaves no orphan recipe.)
- Run clippy right after writing doc comments (the SPEC-031
  `doc_lazy_continuation` lesson): 3-space-indented continuation lines in doc
  bullet lists trip the lint.
- **Run the lean build** (`cargo build --no-default-features`) before finishing —
  it's CI-only otherwise.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-032-edit-save-recipe`
- **PR (if applicable):** opened (see PR for number/URL)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - No new DEC (as expected — composes existing code)
- **Deviations from spec:**
  - Updated `tests/cli.rs` `stub_command_returns_not_implemented` test: now
    asserts exit 2 + "requires at least one operation flag" (was exit 1 + "not
    yet implemented"), since `edit` is no longer a stub. The spec noted this
    test file but did not prescribe the exact update needed.
- **Follow-up work identified:**
  - None identified within STAGE-005 scope.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The spec said `run_pixel_op(pipeline, std::slice::from_ref(input), ...)` but
     `input` in `run_edit` is `&str` while `run_pixel_op` takes `&[String]`. A
     minor type mismatch requiring `[input.to_owned()]` instead. Trivial to fix,
     but not called out in the build prompt.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The stale `stub_command_returns_not_implemented` test in `tests/cli.rs` was
     not mentioned as needing an update. Adding that to the build prompt would
     prevent a confusing test failure at the gate step.

3. **If you did this task again, what would you do differently?**
   — Check for stale "not yet implemented" test references before running gates,
     rather than discovering the failure after all other gates passed.

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
