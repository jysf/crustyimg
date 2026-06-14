---
insight:
  id: DEC-008
  type: decision
  confidence: 0.75
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
  - src/operation/**
  - Cargo.toml

tags:
  - performance
  - resize
  - dependencies
---

# DEC-008: Resize backend is `fast_image_resize` (SIMD), behind the `Resize` operation

## Decision

The default resize backend is **`fast_image_resize`** (SIMD-accelerated),
used inside the `Resize`/`thumbnail`/`shrink` `Operation`s. It converts in
and out of `image::DynamicImage` at its boundary so the canonical model
(DEC-002) is unchanged. `image::imageops::resize` remains the
correctness/fallback reference (and a likely test oracle).

## Context

Resize is the dominant cost in thumbnail and web-batch workloads, and
`fast_image_resize` is commonly 5–15× faster than `image::imageops` thanks
to SIMD. Since the thesis is partly about being fast for batch web-prep,
the hot path deserves the fast backend. This is the least-locked of the
five decisions — it depends on `fast_image_resize`'s current API and on the
conversion cost at the `DynamicImage` boundary being negligible relative to
the resize win (feature-exploration.md § "Technical considerations" —
Resize speed; "Decisions to formalize" #5).

## Alternatives Considered

- **Option A: `image::imageops::resize` only**
  - Why rejected: simplest (zero conversion, already a dep) but materially
    slower on the dominant cost. Kept as the correctness reference/fallback.

- **Option B: `fast_image_resize` only, drop the `image` resize**
  - Why rejected: losing a simple reference implementation removes a useful
    test oracle and a fallback if a `fast_image_resize` edge case bites.

- **Option C (chosen): `fast_image_resize` as default, `image` as reference/fallback**
  - Why selected: speed where it matters, with a correctness oracle and an
    escape hatch.

## Consequences

- **Positive:** Large speedups on resize-heavy batch work; the headline
  `shrink` path is fast.
- **Negative:** A conversion step at the `DynamicImage` boundary, an extra
  dependency, and API churn risk in `fast_image_resize` (the reason
  confidence is 0.75). Filter-type / color-space parity with `imageops`
  must be verified by test.
- **Neutral:** Backend is an internal detail of the `Resize` operation; the
  recipe/CLI surface doesn't expose it.

## Validation

Right if: `Resize` produces visually-equivalent output to the `imageops`
reference (within an SSIM/tolerance threshold test) and is meaningfully
faster on a benchmark. Revisit if: conversion overhead dominates for small
images, or `fast_image_resize` API/maintenance becomes a liability — then
fall back to `image::imageops` (the operation boundary makes the swap local).

## References

- Related specs: STAGE-003 (`resize`, `thumbnail`, `shrink`)
- Related decisions: DEC-002 (operation boundary that localizes the backend)
- External docs: https://docs.rs/fast_image_resize, https://docs.rs/image
- Open question: `resize-backend-api-stability` in `/guidance/questions.yaml`
