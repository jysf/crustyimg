---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-063
  type: story
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # manifest schema + executor orchestration + BuildError + exit-code discipline; no new dep

project:
  id: PROJ-007
  stage: STAGE-020
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-08

references:
  decisions: [DEC-005, DEC-006, DEC-007, DEC-015, DEC-057]
  constraints:
    - decode-once-no-per-op-disk
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - ergonomic-defaults
  related_specs: [SPEC-006, SPEC-031, SPEC-035]

value_link: "STAGE-020's 'declare an image build and run it with one command' capability."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-08
      notes: >
        Framing/design cycle — main-loop, not separately metered → null-with-note per AGENTS §4.
        Included a firsthand probe: read run_apply/apply_one (the executor is apply_one looped over
        targets) + a serde/toml probe of the manifest schema (string-or-list source, deny_unknown_fields,
        version) — no new dependency.
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 260000
      estimated_usd: 2.34
      duration_minutes: 40
      recorded_at: 2026-07-08
      notes: >
        Build cycle run in the main loop (not a metered subagent), so tokens_total is a labelled
        ORDER-OF-MAGNITUDE ESTIMATE, not a harness reading (AGENTS §4). estimated_usd = 260k ×
        Opus 4.8 list ($5/$25 per MTok, ~80/20 in/out, no cache discount).
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-063: the `build` command + declared `crustyimg.build.toml` manifest

## Context

crustyimg runs a recipe over inputs ad hoc today (`apply --recipe web.toml assets/*.png
--out-dir dist`). PROJ-007 (Wave 2) turns that into a declared, cached, verifiable
**build**; this spec is the **skeleton**: a `crustyimg.build.toml` manifest declaring
targets (source × recipe → out/name) and a `crustyimg build` command that runs them all.
The executor is a thin orchestration over the SHIPPED per-input worker (`apply_one`)
looped over targets — no new machinery, no new dependency. It is the foundation the
content-addressed cache (STAGE-021), the reproducibility lockfile (STAGE-022), and
`--watch` (STAGE-023) all extend, plus the manifest-format contract (DEC-057). See the
parent `STAGE-020-build-command-and-manifest.md` for the probe result and framing.

## Goal

Add a `src/build/` manifest module + a `crustyimg build [FILE]` command that parses a
`crustyimg.build.toml` and runs every declared target end to end through the existing
recipe/pipeline/batch, with typed errors, `apply`-consistent exit codes, and a per-build
summary — no new dependency.

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — `run_apply` (the executor to generalize) + **`apply_one`** (the per-input
    decode→pipeline→sink worker to REUSE, signature: `apply_one(&Recipe, &OperationRegistry, &Input,
    &Path out_dir, &str template, Overwrite, Option<u8> quality)`), `build_sink`, `require_out_dir_for_batch`,
    the `CliError` exit-code map (~L505/539), `Commands` enum + dispatch (~L379/591).
  - `src/recipe/mod.rs` — `Recipe::from_toml`, `build_pipeline`, `RECIPE_MAX_BYTES` + the metadata
    size-guard pattern (mirror for the manifest), `RecipeError` (the `BuildError` analog).
  - `src/source/mod.rs` — `resolve` (glob/dir/path; ignore stdin for build targets).
  - `src/sink/mod.rs` — name templates (`{stem}`/`{ext}`), auto-created out dir (PATCH-001), `Overwrite`.
  - `src/pipeline/`, `src/operation/` (`OperationRegistry::with_builtins`).
  - `Cargo.toml` — confirm `serde`/`toml` present (they are); NO new dep.
- **External APIs:** none new.
- **Related code paths:** `src/lib.rs` (add `pub mod build;`), the `apply` integration tests
  (`tests/`) as the shape for `tests/build.rs`.

## Outputs

- **Files created:**
  - `src/build/mod.rs` — `BuildManifest { version: u32, target: Vec<Target> }`, `Target { source:
    SourceSpec, recipe: String, out: String, name: Option<String> }`, `SourceSpec` (untagged
    String|Vec<String>), `BuildManifest::from_toml(&str) -> Result<_, BuildError>` (+ size guard const),
    and `BuildError` (thiserror). Library-first: parsing/validation lives here, unit-tested.
  - `tests/build.rs` — integration tests driving the `crustyimg build` binary (see Failing Tests).
  - `decisions/DEC-057-*.md` — the build-manifest format + `build` semantics decision (emitted at build).
- **Files modified:**
  - `src/lib.rs` — `pub mod build;`.
  - `src/cli/mod.rs` — a `Commands::Build { file: Option<String> }` variant + a `run_build` executor
    (parse manifest → registry once → per target: resolve sources, parse recipe, probe pipeline, rayon
    fan-out `apply_one` with `Overwrite::Allow` → out/name), dispatch, summary, and `BuildError` → exit-code
    mapping (parse/unknown-field/version → 2; per-output failure → partial-batch 6).
- **New exports:** `crustyimg::build::{BuildManifest, Target, BuildError}` (+ `from_toml`). Keep the
  executor (`run_build`) in `cli` (mirrors `run_apply`).

## Acceptance Criteria

- [x] `crustyimg build` with no arg parses `./crustyimg.build.toml`; `crustyimg build FILE` uses `FILE`;
  a missing default file is a clear typed error (exit 3), not a panic.
- [x] A valid manifest runs **every** target: each target's sources are resolved, its recipe applied,
  and outputs written to its `out` dir under its `name` template (default `{stem}.{ext}`), with correct
  dimensions/format. A 2-target manifest produces both target's outputs.
- [x] `source` accepts a single glob/dir/path **or a list**; `out` dirs are auto-created; unknown TOML
  fields and an unsupported `version` are rejected with a typed `BuildError` (exit 2) BEFORE any input
  is touched; a bad/missing recipe fails before that target writes anything.
- [x] Re-running the same build is idempotent (same outputs) and does NOT require `--yes` — `build`
  overwrites its own declared outputs (`Overwrite::Allow`), unlike `apply`.
- [x] A per-output decode/encode failure is a **partial-batch** error (exit 6, reported per output),
  not a hard abort of the whole build; a build with all-good targets exits 0 with a summary (targets
  run, outputs written).
- [x] **No new dependency**; `cargo build --no-default-features` (lean) still succeeds; `just deny`
  unchanged and green.
- [x] `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` clean; every new public fn tested.

## Failing Tests

Written during **design**, BEFORE build. Build makes these pass.

- **`src/build/mod.rs`** (in a `#[cfg(test)] mod tests`)
  - `"parses_valid_manifest"` — version + two `[[target]]` (one `source` string, one `source` list, one
    with `name`, one without) → `Ok`, 2 targets, source forms resolve to the expected lists.
  - `"rejects_unknown_field"` — a target with a `bogus = 1` key → `Err(BuildError::…)` (`deny_unknown_fields`).
  - `"rejects_unsupported_version"` — `version = 999` → `Err(BuildError::UnsupportedVersion { .. })`.
  - `"rejects_oversize_manifest"` — text over the size cap → typed `BuildError`, never an unbounded read.
  - `"missing_required_field_is_error"` — a target without `recipe`/`out` → typed `BuildError`.
- **`tests/build.rs`** (integration, drive the binary)
  - `"build_runs_all_targets"` — temp project: two source PNGs + a recipe (e.g. resize→webp) + a
    `crustyimg.build.toml` with a target → `crustyimg build` → exit 0; both outputs exist in `out` with
    the expected format + dims.
  - `"build_discovers_default_manifest"` — `crustyimg build` (no arg) run in the temp dir finds
    `crustyimg.build.toml`; a run with an explicit `FILE` path also works.
  - `"build_missing_recipe_fails_before_writing"` — a target referencing a nonexistent recipe → exit ≠0,
    typed message, no output files created.
  - `"build_reruns_idempotently_without_yes"` — run the build twice (no `--yes`) → both exit 0, outputs
    present + identical dims (overwrite allowed for owned outputs).
  - `"build_reports_summary"` — a successful build prints a summary naming the targets + output count.
  - `"build_partial_failure_is_exit_6"` — a target whose source set includes one undecodable file →
    exit 6, the good outputs still written, the bad one reported (mirror `apply`).

## Implementation Context

*Read this section (and the files it points to) before starting build. The probe below was verified
firsthand during design — trust it, but re-confirm signatures against the current tree.*

### Decisions that apply
- `DEC-005` — recipes are versioned TOML that round-trip via the registry; the build manifest is the
  **sibling contract** (a dedicated file that BINDS recipes to source→output targets — do NOT overload
  the portable recipe format). Reuse `Recipe::from_toml` + the size-guard pattern.
- `DEC-006` — no async; batch parallelism is rayon. `build` fans targets' inputs out with rayon exactly
  like `apply` (registry is `Sync` via fn pointers; each task rebuilds its pipeline).
- `DEC-007` — typed `thiserror` in the library (`BuildError`), `anyhow`/exit-code mapping only at the
  `cli` boundary.
- `DEC-015` — partial-batch semantics: a per-output failure is exit 6 (collected), not a hard abort.
- **`DEC-057` (NEW — emit at build)** — the `crustyimg.build.toml` format (schema + versioning +
  `deny_unknown_fields` + recipe-file reference) and `build` semantics (default-file discovery,
  overwrite-owned-outputs, exit codes). The stable build contract, analogous to DEC-005 for recipes.

### PROBE — verified firsthand (2026-07-08)
- **The executor is `apply_one` looped over targets.** `run_apply` already does: read+size-guard the
  recipe, `Recipe::from_toml`, `OperationRegistry::with_builtins()` once, probe `build_pipeline` before
  touching inputs, `source::resolve` the inputs, then (multi-input) rayon `par_iter` over
  `apply_one(&recipe, &registry, input, &out_dir, &template, overwrite, quality)`. `run_build` generalizes
  this from ONE (recipe, inputs, out) to N declared targets: for each `Target`, resolve its `source`(s),
  parse its `recipe`, and fan out `apply_one` into its `out`/`name`. Build the registry ONCE and share it.
- **Manifest schema parses cleanly (serde/toml, no new dep):**
  ```rust
  #[derive(Deserialize)] struct BuildManifest { version: u32, #[serde(default)] target: Vec<Target> }
  #[derive(Deserialize)] #[serde(deny_unknown_fields)]
  struct Target { source: SourceSpec, recipe: String, out: String, name: Option<String> }
  #[derive(Deserialize)] #[serde(untagged)] enum SourceSpec { One(String), Many(Vec<String>) }
  ```
  A `source` string OR list both parse; a `bogus` key is rejected (`deny_unknown_fields = true`); a
  `version` field is required. Add a `BUILD_MANIFEST_MAX_BYTES` guard (mirror `RECIPE_MAX_BYTES`) with a
  metadata pre-check before reading. `SourceSpec::as_slice()`/`into_vec()` → the list `source::resolve`
  iterates.

### Executor shape (mirror run_apply)
```
fn run_build(file: Option<&str>, global: &GlobalArgs) -> Result<(), CliError> {
    let path = file.unwrap_or("crustyimg.build.toml");
    // size-guard + read + BuildManifest::from_toml  (typed BuildError → exit 2/3)
    let registry = OperationRegistry::with_builtins();
    let mut outputs = 0; let mut failures = 0;
    for target in &manifest.target {
        let recipe = /* size-guard + read + Recipe::from_toml(target.recipe) */;
        recipe.build_pipeline(&registry)?;                 // fail this target early
        let inputs = resolve_all(&target.source)?;          // source::resolve each, flatten
        let out_dir = PathBuf::from(&target.out);
        let template = target.name.as_deref().unwrap_or("{stem}.{ext}");
        // rayon par_iter over apply_one(&recipe, &registry, input, &out_dir, template, Overwrite::Allow, global.quality)
        // collect Results → count outputs / failures (partial-batch, exit 6 if any failed)
    }
    // print summary (targets, outputs, failures); Ok or partial-batch error
}
```
Honor `global.jobs` (bounded pool) + `global.quiet` (hidden progress) like `run_apply`. Overwrite is
`Overwrite::Allow` for build (owned outputs) — the deliberate difference from `apply`.

### Constraints that apply
- `decode-once-no-per-op-disk` (the pipeline already guarantees it; build just orchestrates),
  `untrusted-input-hardening` (size-guard the manifest; `deny_unknown_fields`; version-gate; the
  inherited sink/source/recipe/decode hardening), `no-unwrap-on-recoverable-paths`,
  `every-public-fn-tested`, `clippy-fmt-clean`, `ergonomic-defaults` (one short `crustyimg build` with a
  discovered default file — no required flags for the common case).

### Prior related work
- `SPEC-006` (recipe TOML + registry, DEC-005) — the sibling format to mirror (parse/version/size-guard).
- `SPEC-031` (rayon batch, DEC-006) + `apply_one` — the per-input worker + fan-out to reuse verbatim.
- `SPEC-035` (`RECIPE_MAX_BYTES` size guard, DEC-036) — the manifest size-guard pattern.
- `PATCH-001` (`--out-dir` auto-creates the target dir) — build reuses it for each `out`.

### Out of scope (for this spec specifically)
- Cache / incremental skip (STAGE-021); lockfile + `--check`/`--frozen` (STAGE-022); `--watch`
  (STAGE-023); a `--dry-run`/plan preview; inline ops in a target; per-target format/quality overrides
  beyond `--quality`; the Wave-4 web-asset manifest; arbitrary shell tasks.

## Notes for the Implementer

- Keep the manifest parsing/validation in `src/build/` (library, unit-tested); keep `run_build` in
  `cli` (mirrors `run_apply`) so `apply_one` is reused directly. Do NOT duplicate the per-input worker.
- Verify the lean build + `just deny` as part of build (no new dep, so both should be unchanged — confirm).
- Build the `OperationRegistry` ONCE and share it across all targets + rayon tasks (fn pointers → `Sync`).
- Probe each target's `build_pipeline` BEFORE resolving/writing its inputs, so a bad recipe fails that
  target early (exit 1) rather than mid-fan-out.
- `build` overwrites its declared outputs (`Overwrite::Allow`) — add a test that a second run needs no
  `--yes`. Confirm the sink still blocks name-template path escapes (build only writes within `out`).
- Reuse the `apply` partial-batch reporting (exit 6, per-output stderr) — do not invent a new scheme.
- Emit `DEC-057` with `affected_scope` covering `src/build/**`, `src/cli/mod.rs`, `src/lib.rs`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-063-build-command`
- **PR (if applicable):** #69
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - `DEC-057` — the `crustyimg.build.toml` manifest format + `build` command semantics
    (dedicated versioned file, recipe-file reference, overwrite-owned-outputs, exit codes,
    cwd-relative paths). `affected_scope`: `src/build/**`, `src/cli/mod.rs`, `src/lib.rs`.
- **Deviations from spec:**
  - **Two-phase executor, not per-target-in-a-loop.** The Implementation Context's sketch
    prepares each target inside the run loop. I prepare **all** targets first (parse recipe,
    probe `build_pipeline`, resolve sources), then execute. Same reuse, stronger guarantee:
    target #2's missing recipe can no longer strand target #1's outputs on disk — which is
    what `build_missing_recipe_fails_before_writing` actually asks for once a manifest has
    more than one target. Recorded in DEC-057 ("fail-before-write").
  - **`load_recipe` extracted** from `run_apply` (size-guard + read + parse) and shared with
    `run_build`, rather than duplicating those three steps. `apply_one` is reused verbatim,
    as specified.
  - **`BuildError::InvalidTarget` + `TooManyTargets` added** beyond the spec's variant list:
    a target with an empty `source` list / blank field, or a `-` (stdin) source, is rejected
    at parse time (a build reads declared files; stdin cannot feed N targets), and the
    target count is capped like `RECIPE_MAX_STEPS`.
  - **`CliError::BuildManifestIo { path, source }`** carries the path, unlike the older
    `RecipeIo`, so a missing default file says *which* file it looked for.
  - The spec's `build_runs_all_targets` sketch says "e.g. resize→webp". No registry op
    changes format and `Target` has no `format` key (out of scope), so outputs preserve the
    source format; the test asserts a resize + a `{stem}_web.{ext}` template on PNG instead.
- **Follow-up work identified:**
  - **Per-target `format` / `quality` keys** (`version = 2`, additive). Today a target's
    `name = "{stem}.webp"` would produce PNG bytes under a `.webp` name — the inherited
    `apply` batch behavior (format = source format). Worth closing before 1.0.
  - **Manifest-relative path resolution** (or a `--dir` flag). Paths are cwd-relative today
    (DEC-057); revisit if users trip over it.
  - **Flatten the fan-out** to one rayon pass over `(target, input)` pairs — targets run
    sequentially today, which under-uses the pool on wide, shallow builds.
  - `PartialBatch`'s message says "N of M **inputs** failed" for a build whose unit is an
    output. Cosmetic; shared with `apply` (DEC-015), so left alone deliberately.
  - A `--dry-run` / plan preview — the natural STAGE-021 companion (already noted in the stage).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Only one thing, and it wasn't in the spec at all: **what manifest paths are relative
   to**. The spec's tests all run in the project root, so cwd-relative and manifest-relative
   are indistinguishable there. I chose cwd-relative (the `make` / `apply` convention),
   documented it as a decision with its revisit trigger rather than leaving it implicit.
   Everything else — schema, executor shape, overwrite semantics, exit codes — was decided
   in the spec and the probe held up exactly as written.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — `DEC-044` (`--out-dir` auto-creates the output directory, cited in the spec as
   "PATCH-001") and `DEC-035` (the sink's symlink-destination guard). Both are load-bearing
   for `build`: the first is why `out` dirs just appear, the second is half of why
   `Overwrite::Allow` is safe. The overwrite argument in DEC-057 leans on DEC-035, and I had
   to go find it.
3. **If you did this task again, what would you do differently?**
   — Write the two-phase (prepare-all → execute-all) split from the start instead of
   following the per-target loop sketch and then noticing that the "fails before writing"
   test only truly holds for a one-target manifest. The multi-target case is the whole
   point of the command; the failure test should be written multi-target first, and the
   executor shape falls out of it.

---

## Reflection (Ship)

1. **What would I do differently next time?**
   — <answer>
2. **Does any template, constraint, or decision need updating?**
   — <answer>
3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
