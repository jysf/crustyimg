# SPEC-031 build prompt — parallel batch `apply --recipe`

Start a **fresh session**. You are the IMPLEMENTER for SPEC-031 in the `crustyimg`
repo. The architect (Opus) wrote the spec + failing tests + DEC-033. `rayon` and
`indicatif` are already in `Cargo.toml`. Make the spec's `## Failing Tests` pass with
the smallest correct change, then open a PR and STOP. Follow this prompt exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-031-parallel-batch-apply-recipe-over-a-source-list.md`
   — especially `## Command surface (PINNED)`, `## Parallel design (PINNED)`,
   `## Failing Tests`, `## Notes for the Implementer`.
2. `decisions/DEC-006` (no async; rayon), `decisions/DEC-033` (indicatif),
   `decisions/DEC-015` (partial-batch exit 6).
3. `src/cli/mod.rs` — `run_apply` (the single-input path you extend), and
   **`run_pixel_op`** (mirror its resolve-all → single-vs-multi → `--out-dir` →
   per-input-catch → exit-6 skeleton). Also `build_sink`, `Overwrite`, `GlobalArgs`
   (`jobs`/`out_dir`/`name_template`/`quality`/`yes`), `CliError`.
4. `src/recipe/mod.rs` (`Recipe::{from_toml, build_pipeline}`),
   `src/operation/registry.rs` (`OperationRegistry::with_builtins`),
   `src/source/mod.rs` (`source::resolve`), `src/sink/mod.rs` (`Sink::Dir`).

## What to build
- Rewrite `run_apply` into a batch fan-out (KEEP single-input behavior identical):
  - Read `--recipe` file (io → exit 3 via `CliError::RecipeIo`); `Recipe::from_toml`
    (reuse SPEC-006 validation — do NOT reimplement). Build `OperationRegistry::with_builtins()` ONCE.
  - Resolve every positional input via `source::resolve` into one `Vec<Input>` (a
    resolution error is a HARD error, exit 3/2 — like `run_pixel_op`).
  - **1 input** → existing behavior: build pipeline, load, run, write to `-o`/`--out-dir`/stdout.
  - **>1 input** → require `--out-dir` (else `CliError::Usage`, exit 2). Replay in PARALLEL.
- **Parallel design (CRITICAL — `Operation` is NOT `Send`):** do NOT share a `Pipeline`
  across threads. Share `&registry` + `&recipe` (both `Sync`); in each `rayon` task
  **rebuild** `let pipeline = recipe.build_pipeline(&registry)?;` then load → `pipeline.run(img)`
  → write to `Sink::Dir { dir: out_dir, template, format }`. Collect
  `Vec<Result<(), CliError>>`; print each `Err` to stderr (label = input path); after the
  parallel section, `failed > 0` → `CliError::PartialBatch { failed, total }` (exit 6).
- **`-j N`** (`global.jobs`): if `Some(n)`, `rayon::ThreadPoolBuilder::new().num_threads(n)
  .build()` then `pool.install(|| all.par_iter()...)`; if `None`, use the default pool.
- **Progress:** `indicatif::ProgressBar` rendered to **stderr**; `--quiet` →
  `ProgressBar::hidden()`. `inc(1)` per finished input; `finish_and_clear()` at the end.
  Use a const/known-valid `ProgressStyle` (NO `unwrap` in non-test code).
- Extract a small `apply_one(&recipe, &registry, &input, ...) -> Result<(), CliError>`
  worker so it's unit-testable, plus a guard helper for the multi-without-out-dir case.

## Hard rules
- **No async** (DEC-006 / `no-async-runtime`) — rayon only. **No `unwrap`/`expect`/`panic!`
  off test paths.** Diagnostics + progress to **stderr**; stdout stays clean.
- Reuse the GLOBAL `--out-dir`/`-j`/`-q`/`-y`/`--name-template` flags (no local shadowing).
- Reuse SPEC-006 recipe validation + the registry; do not add ops or touch `with_builtins`.
- Native fixtures (small PNGs via `image`; recipes as inline TOML to a tempdir). Every
  named test in `## Failing Tests` (2 unit in src/cli/mod.rs, 8 integration in
  tests/apply_batch.rs) must EXIST and PASS. Confirm before claiming green.

## Gates (all must pass — INCLUDING the lean build)
```
cargo fmt && git add -u
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --no-default-features
cargo deny check licenses
```

## Git / PR
- Branch `feat/spec-031-apply-batch` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked `reports/*.md`.
- PR title: `feat(cli): parallel batch apply --recipe (SPEC-031)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-006, DEC-033, DEC-005, DEC-015,
  DEC-007 / Constraints checked / New decisions — "No new DEC" — DEC-033 at design).
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
  notes: "parallel batch apply --recipe: run_apply rewrite + apply_one worker + rayon (per-task pipeline rebuild, Operation not Send) + indicatif progress + exit-6; reuses SPEC-006 recipe/registry; no new op"
```

## When done
`just advance-cycle SPEC-031 verify`, open the PR, and **stop** — the orchestrator
pauses for the user before any merge.
