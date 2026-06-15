---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-001                     # stable, zero-padded within the project
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: crustyimg

created_at: 2026-06-13
shipped_at: 2026-06-14

# What part of the project's value thesis this stage advances.
value_contribution:
  advances: >
    Lays the single-image-model + pluggable Operation pipeline + recipe
    keystone that the whole thesis ("tune once, replay across many") rests
    on. Mostly infrastructure — no user-facing image commands yet.
  delivers:
    - A runnable `crustyimg` binary with a real subcommand interface (help, dispatch)
    - An internal pipeline that decodes once, applies an ordered list of Operations, and emits to a sink
    - Recipes that round-trip to/from TOML via an operation registry
    - Source/sink abstractions (single file / glob / dir / stdin → file / dir+template / stdout / display)
    - Green multi-OS CI with clippy + fmt + a smoke test
  explicitly_does_not:
    - Implement any real image operations or commands (resize/view/etc. are later stages)
    - Touch the metadata lane (STAGE-004)
    - Wire up parallel batch execution (the `apply` command is STAGE-005; the abstractions land here)
---

# STAGE-001: foundation and pipeline core

## What This Stage Is

The structural keystone of the rebuild. When this stage ships, `crustyimg`
is a real Rust binary with a clean clap subcommand interface and an
internal pipeline that **loads an image once, applies an ordered list of
`Operation`s in memory, and writes the result to a sink** (file, directory
with a name template, stdout, or terminal display). The same pipeline is
driven either by command-line arguments or by a **recipe** — a serializable
(TOML) list of operations that round-trips through an operation registry.
Source and sink abstractions already understand single files, globs,
directories, and stdin/stdout, so later stages add operations and commands
without re-touching the core. CI is green on Linux/macOS/Windows with
clippy and fmt enforced.

This stage deliberately ships with **zero real image operations** — it
proves the architecture the original prototype lacked (which re-read each
file from disk per operation and mixed two image libraries).

## Why Now

Every other stage plugs into these abstractions: view/info are sinks +
read-only ops, transforms are `Operation`s, the metadata lane is a parallel
path, and batch is "run the recipe over many sources." Building any command
before the `Operation`/pipeline/recipe contract exists would mean rework.
This is the foundation, so it is first.

## Success Criteria

- `crustyimg --help` lists subcommands; `crustyimg <cmd>` dispatches into
  the pipeline (commands may be stubs that report "not yet implemented").
- The pipeline decodes an image once and applies an ordered `Vec<Operation>`
  in memory (no per-op disk round-trips).
- A recipe serializes to TOML and deserializes back into the identical
  operation list via the registry (round-trip test passes).
- Source abstraction resolves a single path, a glob, and a directory to a
  list of inputs; sink abstraction writes to a file, a dir+name-template,
  stdout, and the terminal.
- `cargo test`, `cargo clippy -D warnings`, and `cargo fmt --check` pass in
  CI on Linux, macOS, and Windows.

## Scope

### In scope
- Cargo project layout, edition, dependency baseline, `justfile`/CI.
- Single canonical image type wrapper over `image::DynamicImage` + robust
  load/decode with format detection and clear errors.
- `Operation` trait + `Pipeline` executor.
- `Source` (single/glob/dir/stdin) and `Sink` (file/dir+template/stdout/display).
- `Recipe` (de)serialization (serde + TOML) + operation registry.
- clap subcommand skeleton + global args (`-o/--output`, output dir,
  verbosity, `--jobs` placeholder) wired to dispatch.

### Explicitly out of scope
- Any concrete operation logic (resize, watermark, filters, metadata).
- Parallel batch execution loop (abstractions only; `apply` is STAGE-005).
- WebP/AVIF; effects catalog; TUI.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-001 (shipped on 2026-06-13) — Cargo project + multi-OS CI + clippy/fmt + smoke test [S] — PR #1
- [x] SPEC-002 (shipped on 2026-06-13) — Canonical `Image` type + load/decode + capture metadata bundle at load (for DEC-003 preserve policy) + error types + native-generated test fixtures [M] — PR #2
- [x] SPEC-003 (shipped on 2026-06-14) — `Operation` trait + `Pipeline` (decode once → ordered ops → result) [M] — PR #3
- [x] SPEC-004 (shipped on 2026-06-14) — `Source` abstraction (single file / glob / dir / stdin → ordered inputs) [M] — PR #4
- [x] SPEC-005 (shipped on 2026-06-14) — `Sink` abstraction (file / dir+name-template / stdout / viuer display) [M] — PR #5
- [x] SPEC-006 (shipped on 2026-06-14) — `Recipe` TOML (de)serialization + operation registry (round-trip) [M] — PR #6
- [x] SPEC-007 (shipped on 2026-06-14) — clap subcommand skeleton + global args, dispatch into pipeline [M] — PR #7

**Count:** 7 shipped / 0 active / 0 pending — ✅ STAGE COMPLETE

> Backlog refined after PROJECT DESIGN: former SPEC-004 (Source+Sink) split
> into SPEC-004 (Source) and SPEC-005 (Sink) — Sink folds viuer display +
> name-template logic and was the largest item. Metadata capture-at-load
> made explicit in SPEC-002 (DEC-003) rather than adding a separate spec.

## Design Notes

- The `Operation` trait is the extension point for the entire project;
  later stages add implementations only. Keep it small: a name, typed
  params (serde-friendly), and `apply(image) -> Result<image>`.
- The registry maps `name -> constructor(params)` so recipes parse back into
  operations. New operations register here; nothing else changes.
- Metadata operations are a **separate lane** (container-level, see
  `feature-exploration.md`) — do not force them through the pixel
  `Operation` trait. Flagged here so SPEC-003 doesn't over-generalize.
- Weighty choices (single model + trait, codec policy, recipe format,
  resize backend) become `DEC-*` files during the project design pass
  (Prompt 2a). See `docs/feature-exploration.md` § "Decisions to formalize".

## Dependencies

### Depends on
- External: Rust toolchain; crates `image`, `clap`, `serde`, `toml`,
  `anyhow`/`thiserror`, `viuer` (sink display), `glob`/`walkdir` (source).
  Exact set/versions pinned during design.

### Enables
- STAGE-002 (view/info) — sinks + read-only paths.
- STAGE-003 (transforms) — first real `Operation`s.
- STAGE-004 (compose/metadata) — watermark `Operation` + the metadata lane.
- STAGE-005 (batch/recipes) — running a recipe over many sources in parallel.

## Stage-Level Reflection

*Shipped 2026-06-14.*

- **Did we deliver the outcome in "What This Stage Is"?** **Yes.** `crustyimg` is a
  runnable Rust binary with a real clap subcommand interface over the full
  `Source → Image::load → Pipeline(+Recipe) → Sink` spine — decode-once, single
  image library, no async. A recipe drives the whole chain end-to-end through the
  CLI (`apply --recipe`). Green 3-OS CI with clippy/fmt enforced; ~97 tests. The
  architecture the prototype lacked is proven, with zero real image ops shipped (by design).

- **How many specs did it actually take?** **7** (SPEC-001..007), exactly as planned
  after the SPEC-004 split (Source/Sink) decided during project design. No scope creep.

- **What changed between starting and shipping?** We adopted a build-on-Sonnet /
  design+verify-on-Opus split (SPEC-003 onward) that cut cost with no loss of
  correctness, and hardened the orchestration: verify is read-only, all
  build/verify/ship bookkeeping lands on `main`, and security was added to the plan
  mid-stage (the `untrusted-input-hardening` constraint + a STAGE-006 gate).

- **Lessons that should update AGENTS.md, templates, or constraints?**
  - Already applied: §13 "verify/ship bookkeeping on main, not the branch"; build
    prompts must use accurate build-mark wording (no "merged" at build time — caught
    on SPEC-006, fixed by SPEC-007).
  - `just advance-cycle`/`archive-spec` mis-glob when a spec has multiple `SPEC-NNN*`
    files; we do ship bookkeeping by hand. Worth fixing the scripts (deferred).

- **Should any spec-level reflections be promoted to stage-level lessons?**
  - "Build security in during the build, then adversarially verify it" (SPEC-004/005
    traversal guards, empirically bypass-probed) — adopt for every untrusted-input spec.
  - Prescriptive build prompts are what make Sonnet builds pass Opus verify first time.

- **DECs emitted this stage:** DEC-010 (glob source), DEC-011 (viuer behind `display`
  feature), DEC-012 (clap) — on top of the project-design DEC-002..009.

- **Follow-ups carried forward:** decode limits, glob escape-check tightening, recipe
  fuzzing, `cargo audit`/`deny`, `--features display` CI job → all owned by STAGE-006.
