---
insight:
  id: DEC-002
  type: decision
  confidence: 0.95
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

created_at: 2026-06-13
supersedes: null
superseded_by: null

affected_scope:
  - src/image/**
  - src/operation/**
  - src/pipeline/**

tags:
  - architecture
  - pixel-core
  - keystone
---

# DEC-002: Single canonical image model + pluggable `Operation` trait

## Decision

There is exactly **one** in-memory image type — `Image`, a thin wrapper
over `image::DynamicImage` — and exactly **one** pixel library (`image`).
All pixel transforms implement a small `Operation` trait
(`name`, `params`, `apply(Image) -> Result<Image>`), and a `Pipeline`
decodes once, folds an ordered list of `Operation`s over the `Image` in
memory, and encodes once.

## Context

The earlier prototype's fatal flaw was using two pixel libraries (`image`
and `photon-rs`) inconsistently, converting between them ad hoc, and
re-reading each file from disk for every operation. That produced
duplicated logic, surprising conversions, and slow batch behavior. The
whole rebuild rests on not repeating this. The `Operation` trait is also
the project's extension point: every later stage adds transforms as
`Operation` impls without touching the core (feature-exploration.md §
"Workflow model" and § "Decisions to formalize" #1).

## Alternatives Considered

- **Option A: Keep two libraries (`image` + `photon-rs`)**
  - What it is: the prototype's status quo.
  - Why rejected: forces lossy conversions between models, doubles the
    dependency surface, and is the documented root cause of the rebuild.

- **Option B: One library but no `Operation` abstraction (free functions)**
  - What it is: each transform is a standalone fn the CLI calls directly.
  - Why rejected: nothing to serialize into a recipe, no uniform pipeline,
    and every new command re-touches dispatch. Recipes (DEC-005) need a
    uniform `name + params` shape.

- **Option C (chosen): one `Image` over `DynamicImage` + `Operation` trait + `Pipeline`**
  - What it is: a single canonical model and a small trait that the
    pipeline folds over a decoded image.
  - Why selected: one mental model, one dependency, decode-once/encode-once
    by construction, and a clean extension point that recipes serialize.

## Consequences

- **Positive:** Decode-once/encode-once is structural, not a discipline.
  New transforms are isolated additions. Recipes have a uniform shape to
  serialize. CI stays simple (one image stack).
- **Negative:** Operations that want a different representation (e.g. a
  fast SIMD resize) must convert in and out of `DynamicImage` at their
  boundary (handled in DEC-008). The trait must stay small or it ossifies.
- **Neutral:** `image::DynamicImage` discards metadata on encode — which is
  exactly why metadata is a separate lane (DEC-003), not a flaw here.

## Validation

Right if: later stages add resize/watermark/etc. as `Operation` impls with
no core changes, and the pipeline never round-trips through disk per op.
Revisit if: an operation fundamentally cannot express itself as
`apply(Image) -> Result<Image>` (e.g. multi-input compositing) — then
extend the trait deliberately rather than adding a second model.

## References

- Related specs: SPEC-002, SPEC-003
- Related decisions: DEC-003 (metadata lane), DEC-005 (recipes), DEC-008 (resize backend)
- External docs: https://docs.rs/image
- Feature research: `docs/feature-exploration.md`
