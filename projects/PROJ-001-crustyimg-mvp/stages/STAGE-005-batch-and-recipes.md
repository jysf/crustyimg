---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-005                     # stable, zero-padded within the project
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: crustyimg

created_at: 2026-06-14
shipped_at: 2026-06-19

# What part of the project's value thesis this stage advances.
value_contribution:
  advances: >
    Proves the central thesis end to end: tune an edit once on one image,
    save it as a recipe, and replay the exact same recipe across a whole
    directory in one parallel command. This is the differentiator over
    flag-soup CLIs and click-through GUIs.
  delivers:
    - "`edit` — one-shot multi-op on a single image (the experiment-like-an-editor mode)"
    - "`--save-recipe` — capture the tuned operation chain as a reusable TOML recipe"
    - "recipe load + validation — round-trip a saved recipe back into operations"
    - "`apply --recipe` — run a recipe over a source list with rayon parallelism + indicatif progress"
    - "output name-templates honored across the batch (`{stem}_web.{ext}`)"
  explicitly_does_not:
    - Add new image operations (it composes the ones from STAGE-003/004)
    - Do the full security assessment / decode limits / traversal tests (STAGE-006)
    - Add a TUI editor (post-MVP, see docs/backlog.md)
---

# STAGE-005: batch and recipes

## What This Stage Is

The stage that makes the thesis real. `edit` runs an ordered list of
operations on a single image in one shot (the "experiment like an editor"
mode), building the op list from CLI flags (`--resize-max`, `--unsharp`,
`--watermark`, …). `--save-recipe` captures that exact chain as a TOML
recipe (DEC-005) via the operation registry. `apply --recipe` then loads,
validates, and replays that recipe over a source list — a single image, a
glob, or a directory — with `rayon` parallelism (DEC-006) and an
`indicatif` progress bar, writing results through the name-template Sink
(`{stem}_web.{ext}`). When this ships, "tune once, replay across many" is a
single command and the MVP's functional surface is complete.

## Why Now

Recipes and batch are the payoff that the foundation (STAGE-001
registry/recipe round-trip + Source/Sink), the transforms (STAGE-003), and
the compose/metadata ops (STAGE-004) were all built toward. They depend on
every prior stage existing — a recipe is only useful once there are real
operations to chain. The `--jobs` placeholder from STAGE-001 is finally
honored here.

## Success Criteria

- `edit in.jpg --resize-max 1200 --watermark logo.png -o out.jpg` applies
  the ops in order, single decode, single encode.
- `--save-recipe web.toml` writes a recipe that, reloaded, reconstructs the
  identical operation list (round-trips through the registry, DEC-005).
- Recipe load validates the recipe version and rejects unknown operations
  with a typed error (basic validation here; full hardening in STAGE-006).
- `apply --recipe web.toml "photos/*.jpg" --out-dir optimized/ -j 8` runs in
  parallel across inputs with a progress bar; output names follow the
  template; partial failures exit 6 with a stderr summary.
- The same recipe runs unchanged on one image and on a directory.

## Scope

### In scope
- `edit` one-shot multi-op + `--save-recipe` (build op list from flags → recipe).
- Recipe load + validation (version check, unknown-op rejection — basic here).
- `apply --recipe` over a Source list with rayon parallelism (DEC-006) + indicatif progress.
- Output name-template handling across a batch; partial-failure summary (exit 6).

### Explicitly out of scope
- New image operations (composes STAGE-003/004 ops only).
- The full security assessment, decode limits, deep recipe/path hardening, cargo-audit-in-CI (STAGE-006).
- A ratatui TUI live-preview editor (post-MVP, docs/backlog.md).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-031 (shipped 2026-06-19, PR #35) — `apply --recipe` parallel batch over a Source list (rayon, DEC-006) + indicatif progress (DEC-033) + name-template output + exit-6 partial failure; bundles backlog items #2 (recipe load/validation, reused from SPEC-006) / #3 / #4
- [x] SPEC-032 (shipped 2026-06-19, PR #36) — `edit` command: one-shot ordered multi-op on a single image from CLI flags (`--auto-orient`/`--resize-max`/`--invert`, canonical order auto-orient→resize→invert) + `--save-recipe` serializing the chain to a round-trippable TOML recipe (DEC-005, byte-equal `edit`↔`apply` pinned by test); reuses `run_pixel_op` + the registry; no new dep/DEC

**Count:** 2 shipped / 0 active / 0 pending  (STAGE-005 COMPLETE. SPEC-031 = parallel batch `apply` [bundled backlog #2/#3/#4]; SPEC-032 = `edit` + `--save-recipe` — the recipe-creation half. "Tune once → save → replay" is now two commands sharing one recipe format.)

## Design Notes

- Recipes are TOML and round-trip through the operation registry built in
  STAGE-001 (DEC-005): `edit --save-recipe` serializes, `apply` deserializes
  back into the identical op list. Both paths construct ops via the registry
  so they cannot drift.
- Parallelism is `rayon` across inputs, not async (DEC-006, constraint
  `no-async-runtime`); bound by memory (~W×H×4 bytes per decoded image).
  The `--jobs`/`-j` placeholder from STAGE-001 is honored here.
- Recipe validation here is the functional baseline (version + unknown-op
  reject); the security-grade validation, decode limits, and path/symlink
  traversal hardening are consolidated and assessed in STAGE-006 (constraint
  `untrusted-input-hardening`).
- Partial batch failure exits 6 with a per-input summary on stderr (api-contract).

## Dependencies

### Depends on
- STAGE-001 — recipe (de)serialization, operation registry, Source/Sink, `--jobs` placeholder.
- STAGE-003 + STAGE-004 — the real operations a recipe chains and replays.
- External: `rayon` (parallel), `indicatif` (progress), `serde`/`toml` (recipe).

### Enables
- STAGE-006 — the recipe/batch surfaces are a hardening target (untrusted recipes/paths).
- Completes the MVP functional surface; post-MVP TUI editor builds on recipes.

## Stage-Level Reflection

*Filled in at ship (2026-06-19).*

- **Did we deliver the outcome in "What This Stage Is"?** **Yes.** "Tune an edit
  once on one image, save it as a recipe, and replay that exact recipe across a
  whole directory in one parallel command" is now real: `edit … --save-recipe
  r.toml` (SPEC-032) writes the recipe; `apply --recipe r.toml <inputs…>
  --out-dir … -j N` (SPEC-031) replays it with rayon parallelism + an indicatif
  progress bar + exit-6 partial-failure. The round-trip is byte-pinned by a test
  (`edit` output == `apply`-of-the-saved-recipe output). The `--jobs` placeholder
  from STAGE-001 is finally honored. The MVP's functional surface is complete.
- **How many specs did it actually take?** **2** (SPEC-031 replay + SPEC-032
  create) vs. a notional backlog of ~4 items — because SPEC-031 *bundled* backlog
  items #2 (recipe load/validation, already shipped in SPEC-006 and reused
  as-is), #3 (parallel replay), and #4 (batch name-templating) into one coherent
  command, and SPEC-032 carried `edit` + `--save-recipe` together. Recipe
  load/validation needed no new spec at all — STAGE-001's registry/recipe
  round-trip (DEC-005) paid off exactly as designed.
- **What changed between starting and shipping?** Almost nothing structural — the
  STAGE-001 foundation (registry round-trip, Source/Sink, `--jobs` placeholder)
  and the STAGE-003/004 ops were sufficient; the only design tension was the
  *replay* half (concurrency — `Operation` is not `Send`) which we resolved by
  rebuilding the pipeline per rayon task from the `Sync` recipe+registry.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - No template/constraint change needed. Two deps added (rayon DEC-006,
    indicatif DEC-033), both pure-Rust/permissive.
  - **Reinforced:** run the lean (`--no-default-features`) build in BOTH build and
    verify — CI-only otherwise ([[verify-includes-lean-no-default-features-build]]).
  - **New, small:** when promoting a clap *stub* to a real command, the
    stub-list test (`tests/cli.rs`) must be updated — name it in the build prompt.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - **The load-bearing concurrency invariant:** `Operation: !Send` → each parallel
    task rebuilds its pipeline from the shared `&recipe` + `&registry`; never share
    a `Box<dyn Operation>` or one `Pipeline` across threads. Carry into STAGE-006+
    (if a future op becomes expensive to rebuild, cache or add `Send` bounds).
  - **The model-split is validated:** build=Sonnet (prescriptive prompt),
    verify=Opus (independent Explore). Both stage builds ran clean on Sonnet,
    passed Opus verify with no concerns, at ~40% lower build cost (~$0.60 each).
  - **Pin the invariant, not just the surface:** both specs landed first-try
    because the *non-obvious* correctness rule was PINNED in the spec — the
    rebuild-per-task design (SPEC-031) and the build-via-registry +
    capture-recipe-before-move round-trip rule (SPEC-032) — rather than left to
    the build to infer.

**Stage cost (recorded):** SPEC-031 $1.09 / 163k + SPEC-032 $1.10 / 165k =
**$2.19 · 329k** across 2 shipped specs.
