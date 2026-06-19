---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-031
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-18
supersedes: null
superseded_by: null

affected_scope:
  - src/operation/**
  - src/cli/mod.rs

tags:
  - architecture
  - operation
  - watermark
  - compositing
---

# DEC-031: Multi-image Operations carry their overlay as in-memory pixels, loaded at the IO boundary

## Decision

`watermark` is the first `Operation` that composes a **second** image (the overlay).
The `Watermark` op holds its overlay as **in-memory `DynamicImage` pixels**, injected
by the **construction boundary** — for direct CLI invocation that is `run_watermark`
in `src/cli/`, which loads the overlay file with `Image::load`. The op's `apply()`
**never reads a file** (it composites already-decoded pixels), preserving the rule
that `src/operation/**` does not depend on files (AGENTS.md §11). The op also stores
the overlay's source **path** purely so `params()` can serialize it for future recipe
round-trip. Watermark is **NOT registered in `OperationRegistry::with_builtins()`** in
this iteration, because the registry constructor is a pure
`fn(&OperationParams) -> Result<Box<dyn Operation>>` that cannot (and must not) load
the overlay file; recipe support for watermark is a **STAGE-005** concern handled by
the recipe loader (the IO boundary for recipes).

## Context

Every prior `Operation` (`resize`/`invert`/`auto-orient`) is a pure single-image
transform whose params are TOML scalars. `watermark` breaks that mold: it needs a
whole second image. Two constraints pull against each other:

- **`Operation::apply(Image) -> Image` is single-input and pure**, and
  `src/operation/**` must not depend on `clap`, files, or terminals (AGENTS.md §11).
- **The overlay is a file** the user names with `--image LOGO`, and for recipes it
  must round-trip as a path through the registry/`OperationParams` (TOML).

A design-time probe confirmed the compositing itself needs no new dependency —
`image::imageops::overlay` (source-over alpha), alpha-scaling for `--opacity`, and
`image::imageops::resize` for `--scale` cover gravity/opacity/scale/margin/tile. So
the only real decision is **where the overlay file is loaded**.

## Alternatives Considered

- **Option A: `apply()` loads the overlay from a path param**
  - Why rejected: puts file IO inside `src/operation/**` (violates AGENTS.md §11) and
    re-reads the file for every base in a batch.

- **Option B: register `watermark` in `with_builtins()` and load the overlay in the
  constructor**
  - Why rejected: the registry `Constructor` is `fn(&OperationParams) -> Result<..,
    RegistryError>` — a bare fn with no IO error channel; loading there still puts file
    IO in `src/operation/**`. Recipe construction is genuinely a recipe-loader (IO
    boundary) job, which lands in STAGE-005.

- **Option C (chosen): overlay loaded at the IO boundary (CLI now, recipe-loader
  later); op holds in-memory pixels; `apply()` stays file-free; path kept only for
  `params()`**
  - Why selected: honors the pure-pixel-core rule, loads the overlay **once** per
    command (shared across the batch), and leaves a clean seam for STAGE-005 recipes.

## Consequences

- **Positive:** `watermark` ships now with no new dependency and no constraint
  violation; the overlay is decoded once; the pattern generalizes to future composite
  ops (montage/append). `apply()` remains unit-testable with an in-memory overlay.
- **Negative:** `watermark` is **not recipe-round-trippable yet** — a recipe naming
  `watermark` won't reconstruct until STAGE-005 wires the recipe loader to resolve the
  overlay path. The op stores both the `DynamicImage` and the path (slight
  redundancy) so `params()` stays meaningful.
- **Neutral:** Gravity becomes a shared `operation`-level concept (a `Gravity` enum +
  parser), reusable by a future `crop` (AGENTS.md §14).

## Validation

Right if: `watermark --image logo.png --gravity southeast` composites the overlay at
the right anchor with opacity/scale/margin/tile behaving as documented, `apply()`
touches no files, and the overlay loads once per invocation (SPEC-029 tests). Revisit
when STAGE-005 adds recipes: extend the recipe loader to resolve `watermark`'s overlay
path (the deferred half of this decision), and update this DEC.

## References

- Related specs: SPEC-029 (`watermark` image overlay); STAGE-005 (recipes — the
  deferred recipe round-trip)
- Related decisions: DEC-002 (single pixel library / `Operation` extension point),
  DEC-008 (resize backend, reused for `--scale`)
- Constraints: pixel core must not depend on files (AGENTS.md §11),
  `no-new-top-level-deps-without-decision` (none added here)
- External docs: https://docs.rs/image/0.25.10 (`imageops::overlay`, `imageops::resize`)
