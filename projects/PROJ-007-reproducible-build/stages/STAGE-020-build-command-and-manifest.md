---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-020
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-007
repo:
  id: crustyimg

created_at: 2026-07-08
shipped_at: 2026-07-08

value_contribution:
  advances: >
    Delivers the skeleton of the project's "Makefile for images" thesis: a declared
    `build` (a TOML manifest of targets = source globs × a recipe → an output dir +
    name template) and a `crustyimg build` command that resolves and runs every
    target through the SHIPPED decode-once pipeline / rayon batch. This is the
    foundation the cache (STAGE-021), lockfile (STAGE-022), and `--watch` (STAGE-023)
    all build on — declare your asset build once and run it with one command.
  delivers:
    - "A `crustyimg.build.toml` manifest format (versioned; `[[target]]` = source × recipe → out/name)"
    - "`crustyimg build [FILE]` runs every target end to end (reusing recipes / pipeline / batch); defaults to `crustyimg.build.toml`"
    - "A per-build summary (targets run, outputs written, per-output errors) and correct exit codes"
    - "A recorded decision (DEC-057) fixing the build-manifest format + `build` command semantics"
  explicitly_does_not:
    - "Cache anything / skip unchanged work — a re-run redoes all targets (that is STAGE-021)"
    - "Write or check a lockfile (STAGE-022) or watch files (STAGE-023)"
    - "Emit a web-asset manifest / placeholders / favicons (Wave 4)"
    - "Add inline ops in the manifest (targets reference a recipe FILE in v1), or run arbitrary shell tasks"
    - "Add any new dependency (reuses serde/toml + the shipped recipe/pipeline/source/sink)"
---

# STAGE-020: the `build` command + declared build manifest

## What This Stage Is

The stage that gives crustyimg a **declared build**. Today you run a recipe over inputs
ad hoc (`apply --recipe web.toml assets/*.png --out-dir dist`); this stage lets you
*declare* that mapping once — a `crustyimg.build.toml` listing targets (each: a source
glob/dir, a recipe file, an output dir, an optional name template) — and run the whole
thing with `crustyimg build`. The executor is a thin orchestration layer over the
SHIPPED machinery: for each target it resolves sources (`source::resolve`), builds the
pipeline from the recipe (`Recipe::from_toml` + `build_pipeline`), and fans out the
existing per-input worker (`apply_one`) over the inputs with rayon — exactly what
`apply` already does, looped over declared targets. No cache, no lockfile, no watch yet
(those are STAGE-021/022/023); this is the skeleton they all extend, plus the manifest
format contract.

## Why Now

- **It's the foundation the rest of the wave requires.** The cache keys on a target's
  (input, recipe, config); the lockfile pins a build's outputs; `--watch` re-runs
  affected targets. All three need the declared-build + executor to exist first.
- **It's almost pure reuse.** The design probe confirmed the executor is `apply_one`
  looped over targets, and the manifest is a small serde/toml schema — no new
  dependency, low risk, high daily-driver value ("declare my asset build and run it").

## Success Criteria

- `crustyimg build` (defaulting to `crustyimg.build.toml`, or `build FILE`) parses the
  manifest and runs every target end to end, producing the declared outputs with correct
  dimensions/formats.
- A malformed manifest (bad TOML, unknown field, unsupported version) fails with a typed
  error BEFORE any input is touched; a target with a missing recipe/source fails clearly;
  a per-output failure is a partial-batch error (mirror `apply`'s exit-code conventions).
- Re-running a build is idempotent (same outputs) — `build` owns its declared output
  tree and overwrites its own derived outputs without `--yes` (a deliberate difference
  from `apply`, documented).
- A per-build summary reports targets run + outputs written + errors; output dirs are
  auto-created (the `--out-dir` precedent, PATCH-001).
- **No new dependency**; pure-Rust default; `just deny` green; lean build unaffected.
- A **DEC-057** records the manifest format + `build` semantics as a stable contract.

## Scope

### In scope
- A `src/build/` module: the `crustyimg.build.toml` schema (`BuildManifest`/`Target`,
  serde/toml, `deny_unknown_fields`, version check) + a typed `BuildError`. A `run_build`
  executor + `Commands::Build` in `src/cli/`, reusing `apply_one` / `source::resolve` /
  `Recipe` / `OperationRegistry`. Manifest discovery (default `crustyimg.build.toml`),
  per-build summary, exit codes mirroring `apply`, overwrite-owned-outputs. DEC-057.
  **(SPEC-063)**

### Explicitly out of scope
- Content-addressed cache / incremental skip (STAGE-021); lockfile + `--check`/`--frozen`
  (STAGE-022); `--watch` (STAGE-023); the Wave-4 manifest/placeholders/favicons; inline
  ops in a target (recipe-file reference only in v1); arbitrary shell tasks; a
  `--dry-run`/plan preview (natural STAGE-021 companion — note as follow-up).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-063 (shipped on 2026-07-08) — the `build` command + `crustyimg.build.toml` manifest: `src/build/`
  (`BuildManifest`/`Target` serde schema + `BuildError`, versioned, `deny_unknown_fields`, size/target
  caps) + a `run_build` executor in `src/cli/` that prepares ALL targets then loops the shipped `apply_one`
  (source × recipe → out/name, rayon, fail-before-write), default-file discovery, summary + exit codes,
  overwrite-owned-outputs; DEC-057. **No new dependency.** PR #69 (a254fe8), 24/24 CI green. Known hazard
  carried to STAGE-021/022: non-injective source→output mapping (stem collision).

**Count:** 1 shipped / 0 active / 0 pending — single-spec stage complete.

## Design Notes

- **PROBE RESULT (2026-07-08) — the executor is pure reuse; the manifest is a small serde schema.**
  A firsthand read of `run_apply`/`apply_one` (src/cli) confirmed `build` is `apply_one` (the shipped
  per-input decode→pipeline→sink worker) looped over declared targets — same rayon fan-out, same sink
  templating, same exit-code conventions. A serde/toml probe confirmed the manifest schema parses:
  `version` + `[[target]]` with `source` as a **string OR list** (`#[serde(untagged)]`), a `recipe`
  path, an `out` dir, and an optional `name` template; `#[serde(deny_unknown_fields)]` rejects typos
  (the recipe-validation discipline). **No new dependency** (serde/toml are shipped).
- **DEC-057 (at build):** the `crustyimg.build.toml` format + `build` semantics as a stable contract
  (the recipes/DEC-005 analog for builds): a dedicated manifest file (NOT extending the portable recipe
  format — recipes stay reusable/input-agnostic; a build BINDS recipes to source→output mappings),
  targets reference a recipe FILE, and `build` overwrites its own declared outputs.
- **Manifest schema (v1):**
  ```toml
  version = 1

  [[target]]
  source = "assets/**/*.png"      # a glob / dir / path, OR a list of them (source::resolve each)
  recipe = "recipes/web.toml"     # path to a recipe TOML (Recipe::from_toml)
  out    = "dist/img"             # output directory (auto-created; PATCH-001)
  name   = "{stem}.webp"          # optional name template; default "{stem}.{ext}"
  ```
- **Reuse map:** parse manifest → build `OperationRegistry::with_builtins()` ONCE (shared across
  rayon) → per target: `source::resolve` each source, `Recipe::from_toml` the recipe (+ probe
  `build_pipeline` before touching inputs so a bad recipe fails early), then fan out `apply_one(recipe,
  &registry, input, out_dir, template, overwrite, quality)` over the inputs. This is `run_apply`
  generalized from one (recipe, inputs, out) to N declared targets.
- **Overwrite semantics — a deliberate difference from `apply`.** `apply`/batch default to
  `Overwrite::Forbid` (require `--yes`). A **build owns its output tree** and must be re-runnable
  ("make" overwrites its targets), so `build` uses `Overwrite::Allow` for its declared outputs. It only
  ever writes within a target's `out` dir (the sink already blocks name-template path escapes), never
  deletes unrelated files. STAGE-021's cache makes most re-run writes skip anyway; STAGE-022's lockfile
  makes them reviewable. Record this in DEC-057.
- **Errors + exit codes (mirror `apply`):** manifest parse / unknown-field / unsupported-version → a
  typed `BuildError` → exit 2 (usage). A target's missing recipe file → exit 3 (io) / bad recipe → exit
  1; an empty/missing source → the `source` error (exit 3/2). A per-output decode/encode failure is a
  **partial-batch** error (exit 6, DEC-015), collected and reported like `apply`, not a hard abort.
- **Security/hardening:** the manifest is config input — apply the recipe-file size guard pattern to the
  manifest read (`*_MAX_BYTES` + metadata pre-check); `deny_unknown_fields`; version-gate. The heavy
  lifting (path-traversal on outputs, symlink-escape on sources, recipe op validation, decode caps) is
  already enforced by sink/source/recipe/image and is inherited unchanged.

## Dependencies

### Depends on
- Shipped: `src/cli/mod.rs` (`run_apply` + `apply_one` + `build_sink` + exit-code map), `src/recipe/`
  (`Recipe::from_toml`/`build_pipeline`/`RECIPE_MAX_BYTES`, DEC-005), `src/pipeline/` (decode-once),
  `src/source/` (`resolve`), `src/sink/` (name templates + auto-created out dir, PATCH-001), `rayon`
  batch (SPEC-031, DEC-006), `serde`/`toml` (DEC-005).
- PROJ-009 (input reach) — a build ingests AVIF/SVG/RAW sources like any other now.

### Enables
- STAGE-021 (the cache keys on a target's resolved input+recipe+config), STAGE-022 (the lockfile pins a
  build's outputs), STAGE-023 (`--watch` re-runs affected targets) — all extend this executor + manifest.

## Stage-Level Reflection

- **Did we deliver the outcome in "What This Stage Is"?** Yes — `crustyimg build` runs a declared
  `crustyimg.build.toml` (targets = source × recipe → out/name) end to end over the shipped pipeline,
  with a two-phase fail-before-write executor, manifest hardening, and apply-consistent exit codes. PR
  #69, 24/24 CI green, **no new dependency** (455 lines src/build + 291 lines cli on shipped machinery).
- **How many specs did it actually take?** 1 (SPEC-063), as planned — a clean single-spec skeleton stage.
- **What changed between starting and shipping?** The executor improved on the spec sketch (prepare ALL
  targets — recipe parse + pipeline probe + source resolve — *before* executing any, so a bad target
  can't strand an earlier target's outputs), and two things surfaced by exercising the binary: a
  `.gitignore` near-miss (an unanchored `build/` silently excluded the whole new module from `git add`)
  and a real reproducibility hazard (a non-injective source→output mapping — stem collision — races the
  fan-out and over-counts). The hazard is out of scope for the skeleton but recorded in DEC-057 as a
  STAGE-022 blocker.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - Adding a new top-level `src/` module: **audit `.gitignore` for unanchored directory patterns** (and
    count staged files against the spec's Outputs list before committing) — a silent exclusion produces
    no error and would merge a broken PR.
  - When a project's whole thesis is verifiability, **correctness hazards found by hand-exercising the
    binary matter more than the tests passing** — the stem collision passed every gate green.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - Yes — the injective source→output constraint is a project-level invariant (the cache and especially
    the lockfile depend on it), so it lives in DEC-057's validation list, not just SPEC-063's follow-ups.
