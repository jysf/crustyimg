# SPEC-032 build prompt — `edit` + `--save-recipe`

Start a **fresh session**. You are the IMPLEMENTER for SPEC-032 in the `crustyimg`
repo. The architect (Opus) wrote the spec + failing tests. **No new dependency, no new
DEC** — this composes existing code (the registry ops + recipe serialization, both
already in the tree). Make the spec's `## Failing Tests` pass with the smallest correct
change, then open a PR and STOP. Follow this prompt exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-032-edit-one-shot-multi-op-and-save-recipe.md`
   — especially `## Command surface (PINNED)`, `## Round-trip guarantee (PINNED)`,
   `## Failing Tests`, `## Notes for the Implementer`.
2. `decisions/DEC-005` (recipe round-trip via the registry), `decisions/DEC-015`
   (sink/format/quality fan-out + exit 6), `decisions/DEC-031` (watermark deferred —
   why `edit` has no `--watermark` yet).
3. `src/cli/mod.rs` — the `Commands::Edit` clap stub, the `dispatch`
   `NotImplemented("edit")` arm, **`run_pixel_op`** (the single-input load→run→sink path
   you REUSE), and **`run_resize` / `resize_params`** (the exact `--resize-max →
   {mode,width}` mapping you MIRROR). Also `GlobalArgs`, `CliError`.
4. `src/recipe/mod.rs` (`Recipe::{from_ops, to_toml}`), `src/operation/registry.rs`
   (`OperationRegistry::with_builtins`, `build`), `src/operation/mod.rs`
   (`OperationParams::{empty, from_map}`; ops `Invert`/`Resize`/`AutoOrient`).

## What to build
- **Clap:** extend `Commands::Edit` with three op flags alongside the existing `input`
  and `save_recipe`:
  - `#[arg(long)] auto_orient: bool` (`--auto-orient`)
  - `#[arg(long, value_name = "N")] resize_max: Option<u32>` (`--resize-max`)
  - `#[arg(long)] invert: bool` (`--invert`)
  Keep using the GLOBAL `-o`/`--out-dir`/`--format`/`-q`/`-y` (no local shadowing).
- **Dispatch:** replace `Commands::Edit { .. } => Err(CliError::NotImplemented("edit"))`
  with a destructure that calls `run_edit(input, *auto_orient, *resize_max, *invert,
  save_recipe.as_deref(), &cli.global)`.
- **`build_edit_ops` (pure, UNIT-TESTED core):**
  `fn build_edit_ops(auto_orient: bool, resize_max: Option<u32>, invert: bool) ->
  Result<Vec<Box<dyn Operation>>, CliError>`.
  - Make a `OperationRegistry::with_builtins()`.
  - Push ops in the **canonical order** (independent of arg order):
    1. if `auto_orient` → `registry.build("auto-orient", &OperationParams::empty())`
    2. if `let Some(n) = resize_max` → params via `resize_params(Some(n), None, None,
       None, None, None)?`, then `registry.build("resize", &params)` mapping
       `RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason)` (and
       `Unknown` defensively) — mirror `run_resize`.
    3. if `invert` → `registry.build("invert", &OperationParams::empty())`
  - Empty list (no op flag) → `Err(CliError::Usage("edit requires at least one operation
    flag (--auto-orient, --resize-max, --invert)".into()))`.
  - `build("auto-orient"/"invert", …)` can't realistically fail, but propagate any
    `RegistryError` as `CliError::Usage` rather than unwrapping.
- **`run_edit` (flow — order matters):**
  1. `let ops = build_edit_ops(auto_orient, resize_max, invert)?;`
  2. Capture the recipe object NOW, before the ops are moved (only if saving):
     `let recipe = save_recipe.map(|_| Recipe::from_ops(&ops));`
     (`Recipe::from_ops` borrows `&[Box<dyn Operation>]`, so this must precede the fold.)
  3. `let pipeline = ops.into_iter().fold(Pipeline::new(), |p, op| p.push(op));`
  4. `run_pixel_op(pipeline, std::slice::from_ref(input), global, global.quality, None,
     None)?;` — this does resolve→load→run→write with all the established
     single/multi-input, format, `-q`/`-y`, and exit-code behavior. Do NOT re-implement it.
  5. On success, if `--save-recipe` was given: `let toml = recipe.unwrap().to_toml()?;`
     (serialize failure → `CliError::Recipe`, exit 1) then
     `std::fs::write(path, toml).map_err(|e| CliError::Sink(SinkError::Io(e)))?;`
     (write failure → exit 5). Overwrites if present (no overwrite guard for recipe files).
  Use `if let Some(recipe) = recipe` rather than `.unwrap()` to stay panic-free.

## Hard rules
- **Smallest correct change.** Reuse `run_pixel_op`, `resize_params`, the registry, and
  `Recipe::{from_ops, to_toml}` — do NOT touch the recipe layer, the registry's
  `with_builtins`, or add any op. **No new dependency.**
- **No `unwrap`/`expect`/`panic!`** off test paths (DEC-007). Diagnostics to stderr.
- The saved recipe MUST round-trip: build ops via the REGISTRY (not by `new`-ing concrete
  op types) so `apply --recipe` reconstructs the identical pipeline (DEC-005). An
  integration test pins `edit` output bytes == `apply`-of-the-saved-recipe output bytes.
- Native fixtures (small PNGs via `image`; outputs/recipes to a tempdir — mirror
  `tests/apply_batch.rs`'s `write_png` / `env!("CARGO_BIN_EXE_crustyimg")`). Every named
  test in `## Failing Tests` (4 unit in `src/cli/mod.rs`, 7 integration in `tests/edit.rs`)
  must EXIST and PASS. Confirm before claiming green.
- Run clippy right after writing doc comments (SPEC-031's `doc_lazy_continuation` lesson).

## Gates (all must pass — INCLUDING the lean build)
```
cargo fmt && git add -u
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --no-default-features
cargo deny check licenses
```

## Git / PR
- Branch `feat/spec-032-edit-save-recipe` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked `reports/*.md`.
- PR title: `feat(cli): edit + --save-recipe (SPEC-032)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-005, DEC-015, DEC-007, DEC-031 /
  Constraints checked / New decisions — "No new DEC").
- Fill the spec's `## Build Completion` + 3 reflection answers; append the build cost
  session (numerics null; agent `claude-sonnet-4-6`).

## Cost
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-19
  notes: "edit + --save-recipe: clap op flags + run_edit + build_edit_ops (canonical order auto-orient→resize→invert) reusing run_pixel_op + registry + Recipe::from_ops/to_toml; recipe round-trips through apply; no new dep/op"
```

## When done
`just advance-cycle SPEC-032 verify`, open the PR, and **stop** — the orchestrator
pauses for the user before any merge.
