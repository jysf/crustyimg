---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-001                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: crustyimg

created_at: 2026-06-13
shipped_at: null

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
- [ ] SPEC-005 (build) — `Sink` abstraction (file / dir+name-template / stdout / viuer display) [M]
- [ ] SPEC-006 (design) — `Recipe` TOML (de)serialization + operation registry (round-trip) [M]
- [ ] SPEC-007 (design) — clap subcommand skeleton + global args, dispatch into pipeline [M]

**Count:** 4 shipped / 0 active / 3 pending

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

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
