---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-031
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

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
  decisions: [DEC-006, DEC-033, DEC-005, DEC-015, DEC-007]
  constraints:
    - no-async-runtime
    - no-new-top-level-deps-without-decision
    - clippy-fmt-clean
    - every-public-fn-tested
    - no-unwrap-on-recoverable-paths
  related_specs: [SPEC-006, SPEC-007, SPEC-013]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-005's <capability>". Optional; null is acceptable.
value_link: >
  Turns `apply --recipe` into the parallel batch replay that IS the project
  thesis — tune once, replay the same recipe across a whole directory.

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
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: >
        Main-loop orchestrator work, not separately metered. Authored the spec
        (Failing Tests + Implementation Context); emitted DEC-033 (indicatif).
        Added rayon (DEC-006) + indicatif; just deny + lean build green. Key
        design: each rayon task rebuilds its pipeline from the Sync recipe +
        registry (Operation is not Send). First STAGE-005 spec.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-031: parallel batch `apply --recipe` over a source list

## Context

**The first STAGE-005 spec — and the project's thesis made real:** "tune an edit once,
save it as a recipe, replay the same recipe across a whole directory in one parallel
command." The recipe round-trip (load + version/unknown-op validation) already exists
(SPEC-006: `Recipe::{from_toml, build_pipeline}`, version `"1"`), and `apply --recipe`
already works for a **single** input (`run_apply` in `src/cli/mod.rs`). This spec
upgrades `apply` to a **parallel batch**: resolve a source list (file / glob / dir /
stdin), replay the recipe across every input with **`rayon`** data parallelism
(DEC-006, honoring `--jobs`/`-j`), show an **`indicatif`** progress bar on stderr
(DEC-033), write each result through the name-template `Sink`, and exit **6** on partial
failure with a per-input stderr summary (DEC-015).

It folds STAGE-005 backlog items **#2** (recipe load/validation — already satisfied,
reused), **#3** (parallel batch replay), and **#4** (batch name-templating) into one
coherent command. Parent: `STAGE-005-batch-and-recipes`. Governing: **DEC-006**
(rayon, no async), **DEC-005** (recipe round-trip via the registry), **DEC-033**
(indicatif), **DEC-015** (partial-batch exit 6).

## Goal

Extend `apply --recipe RECIPE <inputs…>` to replay the recipe over a resolved source
list **in parallel** (`rayon`, bounded by `--jobs`), with an `indicatif` progress bar,
name-templated batch output, and exit-6 partial-failure semantics — keeping the
existing single-input behavior unchanged.

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — `run_apply` (the single-input path to extend), `run_pixel_op`
    (the multi-input fan-out + `--out-dir` + exit-6 pattern to MIRROR), `build_sink`,
    `GlobalArgs` (`jobs`, `out_dir`, `name_template`, `quality`, `yes`), `Overwrite`,
    `CliError` (`PartialBatch`, `Usage`, `RecipeIo`).
  - `src/recipe/mod.rs` — `Recipe::{from_toml, build_pipeline}`, `RecipeError`.
  - `src/operation/registry.rs` — `OperationRegistry::with_builtins` (fn-pointer
    constructors → `Sync`).
  - `src/source/mod.rs` — `source::resolve` (file/glob/dir/stdin → `Vec<Input>`).
  - `src/sink/mod.rs` — `Sink::Dir { dir, template, format }`, `SinkInput`, `safe_join`.
  - `decisions/DEC-006`, `DEC-033`, `DEC-015`.
- **External crates:** `rayon` `=1.12.0` (DEC-006) — `par_iter`, `ThreadPoolBuilder`;
  `indicatif` `=0.18.4` (DEC-033) — `ProgressBar`, `ProgressStyle`. Both added at design.
- **Related code paths:** `src/cli/mod.rs`, `tests/`.

## Outputs

- **Files modified:**
  - `src/cli/mod.rs` — rewrite `run_apply` to a batch fan-out (single-input path
    preserved). Add a per-input worker (`apply_one(recipe, registry, input, …) ->
    Result<(), CliError>`) and the rayon+indicatif driver. No new public exports
    required (internal `fn`s); unit-test the pure helpers.
  - `Cargo.toml` — `rayon = "=1.12.0"`, `indicatif = "=0.18.4"` (done at design).
  - `docs/api-contract.md` — extend the `apply` entry with batch/parallel/progress (done at design).
- **Files created:** `tests/apply_batch.rs` — integration tests for the batch path.
- **Database changes:** none.

## Command surface (PINNED)

```
crustyimg apply --recipe RECIPE <INPUTS...> [--out-dir DIR] [-j N] [-q Q] [-y]
```

- **Recipe load** (unchanged, SPEC-006): read `--recipe` file (io error → exit **3**
  via `CliError::RecipeIo`); `Recipe::from_toml` (bad version → its typed error, exit
  1; the recipe references an op not in `with_builtins` → `build_pipeline` unknown-op
  error, exit 1).
- **Source resolution:** resolve EACH positional input via `source::resolve` (flatten
  into one `Vec<Input>`, mirroring `run_pixel_op`). A resolution error (missing path /
  empty glob) is a HARD error (exit 3/2), **not** partial-batch.
- **Single resolved input** → keep the existing behavior exactly: write to `-o`/
  `--out-dir`/stdout; no progress bar needed (or a trivial one).
- **Multiple resolved inputs** → require `--out-dir` (else `CliError::Usage`, exit
  **2**, "multiple inputs require --out-dir"). Replay the recipe across inputs **in
  parallel**; write each to `Sink::Dir { dir, template, format }` using the global
  `--name-template` (default `{stem}.{ext}`). A per-input load/run/write failure is
  caught, printed to stderr, and counts toward exit **6** (`PartialBatch`); a clean run
  exits 0.
- **`--jobs`/`-j N`** (`GlobalArgs.jobs`, already parsed) bounds parallelism: build a
  local `rayon::ThreadPool` with `num_threads(N)` and run the batch inside
  `pool.install(..)`; if `None`, use rayon's default pool.
- **Progress:** an `indicatif::ProgressBar::new(total)` rendered to **stderr**, `inc(1)`
  per completed input, `finish_and_clear()` at the end. **Suppressed when `--quiet`**
  (use `ProgressBar::hidden()`); indicatif also auto-hides on a non-TTY. stdout stays
  clean.
- `-q/--quality` and `-y/--yes` apply per-output as today (`Overwrite::Allow` iff `-y`).

## Parallel design (PINNED — `Operation` is NOT `Send`)

The `Operation` trait has **no `Send`/`Sync` bound**, so a built `Pipeline`
(`Vec<Box<dyn Operation>>`) **cannot cross a thread**. Do **NOT** build one pipeline and
share it. Instead:

- Build the `OperationRegistry::with_builtins()` **once** and share `&registry` (its
  constructors are `fn` pointers → `Sync`). Share `&recipe` (`Recipe` is plain data →
  `Sync`).
- In each rayon task, **rebuild a local pipeline**: `let pipeline =
  recipe.build_pipeline(&registry)?;` then load → `pipeline.run(img)` → encode → write.
  Building from the registry is cheap; this keeps everything `Send`-free across threads.
- Collect results into a `Vec<Result<(), CliError>>` (or count failures atomically);
  after the parallel section, sum failures → `CliError::PartialBatch { failed, total }`
  (exit 6) or `Ok(())`. Print each failure to stderr as it happens (label = input path).

## Acceptance Criteria

- [ ] `apply --recipe r.toml a.png b.png c.png --out-dir out/` writes 3 outputs, all
  with the recipe applied; exit 0.
- [ ] The SAME recipe runs unchanged on a single image (`apply --recipe r.toml a.png
  -o out.png`) — single-input behavior is preserved.
- [ ] `-j 1` and `-j 4` both produce identical correct outputs (determinism of results,
  not order); `-j` bounds the worker count.
- [ ] A batch with one unreadable/corrupt input + two good ones → the two good outputs
  are written, a stderr line names the failure, exit **6**.
- [ ] Multiple inputs without `--out-dir` → exit **2**.
- [ ] Output names follow the template (`{stem}.{ext}`, and a custom `--name-template
  {stem}_web.{ext}` is honored).
- [ ] A recipe with an unsupported `version` → exit 1; a recipe naming an unknown op →
  exit 1 (reused SPEC-006 validation).
- [ ] `--quiet` suppresses the progress bar; stdout stays clean (no progress on stdout).
- [ ] `cargo deny` green; **lean build** compiles; no async runtime (DEC-006 / constraint
  `no-async-runtime`).

## Failing Tests

Written during **design**, BEFORE build. Generate fixtures natively (small PNGs via the
`image` crate; recipes as inline TOML strings written to a tempdir).

- **`src/cli/mod.rs` (unit, `#[cfg(test)] mod tests`)**
  - `"apply_batch_requires_out_dir_for_multi"` — building the batch with 2 inputs and no
    `--out-dir` yields `CliError::Usage` (exit 2). (Test the guard helper directly.)
  - `"apply_worker_applies_recipe_to_one"` — `apply_one` on a fixture with a simple
    recipe (e.g. `resize max 8`) produces the expected dimensions. (Pure-ish worker.)
- **`tests/apply_batch.rs` (integration, drives the binary)**
  - `"apply_batch_writes_all_outputs"` — recipe + 3 PNGs + `--out-dir` → 3 files exist;
    exit 0.
  - `"apply_single_input_unchanged"` — recipe + 1 PNG + `-o out.png` → out exists; exit 0.
  - `"apply_batch_partial_failure_exits_6"` — 2 good + 1 bogus (non-image) input +
    `--out-dir` → 2 outputs written, exit 6.
  - `"apply_batch_multi_without_out_dir_exits_2"` — 2 inputs, no `--out-dir` → exit 2.
  - `"apply_batch_jobs_one_and_four_agree"` — same recipe+inputs with `-j 1` vs `-j 4`
    → identical output bytes (or dims) per file.
  - `"apply_batch_name_template_honored"` — `--name-template {stem}_web.{ext}` → outputs
    named `*_web.*`.
  - `"apply_batch_unknown_op_exits_1"` — recipe naming a bogus op → exit 1.
  - `"apply_batch_quiet_clean_stdout"` — `--quiet` run → stdout empty (progress only on
    stderr / hidden).
- **`tests/cli.rs`** — `apply` already in the subcommand lists; confirm, add only if missing.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-006` — **no async**; batch parallelism via `rayon` across inputs. The `--jobs`
  placeholder from STAGE-001 is honored here.
- `DEC-005` — recipes round-trip through the registry; the batch builds each task's
  pipeline via `recipe.build_pipeline(&registry)` (already validated by SPEC-006).
- `DEC-033` — `indicatif` progress bar (stderr; suppressed by `--quiet`).
- `DEC-015` — partial-batch fan-out semantics + exit 6 (mirror `run_pixel_op`).
- `DEC-007` — typed errors; no `unwrap`/`expect`/`panic!` off test paths.

### Constraints that apply

- `no-async-runtime` (blocking) — rayon only; no `tokio`/`async`.
- `no-new-top-level-deps-without-decision` — rayon (DEC-006) + indicatif (DEC-033).
- `clippy-fmt-clean`, `every-public-fn-tested`, `no-unwrap-on-recoverable-paths`.

### Prior related work

- `SPEC-006` (shipped) — `Recipe` + registry round-trip + version/unknown-op
  validation. **Reused as-is; do NOT reimplement recipe parsing/validation.**
- `SPEC-007` (shipped) — the `apply` clap variant + `run_apply` single-input path.
- `SPEC-013`/SPEC-015 — `run_pixel_op` is the multi-input fan-out + exit-6 template to
  mirror (resolution-error vs per-input-failure distinction).

### Out of scope (for this spec specifically)

- `edit` + `--save-recipe` (the recipe-*creation* command) — the next STAGE-005 spec.
- Registering `watermark` (or any STAGE-004 op) in `with_builtins` for recipe
  round-trip — deferred (DEC-031); recipes here chain the registered ops
  (identity/invert/resize/auto-orient).
- Recursive directory walking via `walkdir` — `source::resolve` (glob/dir) covers the
  in-scope cases; deep recursion + symlink/path hardening is STAGE-006.
- Security-grade recipe/path validation, decode limits — STAGE-006.

## Notes for the Implementer

- **Mirror `run_pixel_op`** for the fan-out skeleton (resolve-all → single vs multi →
  `--out-dir` requirement → per-input catch → exit 6), but the per-input body is
  "build pipeline from recipe + run" instead of a fixed op.
- **The `Send` gotcha is the crux:** build `OperationRegistry` once, share `&registry`
  + `&recipe` across `par_iter`, and **rebuild the pipeline inside each task**. Do not
  try to make `Box<dyn Operation>` `Send` or share one `Pipeline`.
- **`-j`:** `rayon::ThreadPoolBuilder::new().num_threads(n).build().map_err(..)?` then
  `pool.install(|| inputs.par_iter().map(...).collect())`. `None` → default pool.
- **Progress:** `ProgressBar::new(total as u64)` (or `::hidden()` when `--quiet`),
  `.set_style(ProgressStyle::with_template("...").unwrap())` — but avoid `unwrap` in
  non-test code; use a `.unwrap_or_else(|_| ProgressStyle::default_bar())` or a const
  style that is known-valid. `inc(1)` per task; `finish_and_clear()`. Render to stderr.
- **stdout stays clean** (no `println!` of diagnostics) so `apply ... -o -` style pipes
  and `--quiet` runs are usable. Per-input error lines go to **stderr**.
- Keep the single-input path behaviorally identical to today (a test pins it).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
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
