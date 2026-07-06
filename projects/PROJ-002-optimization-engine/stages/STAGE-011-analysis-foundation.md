---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.

stage:
  id: STAGE-011                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null

project:
  id: PROJ-002                      # parent project
repo:
  id: crustyimg

created_at: 2026-07-05
shipped_at: null

value_contribution:
  advances: >
    Builds the shared, computed-once `Analysis` layer that the whole PROJ-002
    thesis stands on — the format auto-decision, `explain`, and every later wave
    (planner, lint, manifest) read it. Infrastructure-forward, but the direct
    enabler of the "look at the image and decide" capability.
  delivers:
    - "A new `src/analysis/` module: an immutable, computed-once `Analysis` context
      (histogram, entropy, edge density, alpha coverage, capped unique-colour count,
      dominant colour) computed in one pass over the decoded buffer"
    - "Deterministic, no-ML internal `classification` (photo / graphic-logo / icon /
      document / ui-screenshot) collapsed to three optimization buckets"
    - "A typed, bounded, no-panic `AnalysisError` honoring STAGE-006 hardening"
  explicitly_does_not:
    - Change any CLI behaviour or command output — lands standalone so every existing
      test stays green (wired into a command only in STAGE-012)
    - Surface classification as a user-facing feature (internal enabler only)
    - Add a new default dependency, an `Analyzer` registry, or any recipe-serialized step
---

# STAGE-011: analysis foundation

## What This Stage Is

The foundational stage of PROJ-002: a new **`src/analysis/`** module that turns the single
decoded image into a shared, immutable **`Analysis`** context — computed once, in one pass, and
read by everything downstream. It extracts the cheap-but-decisive features (colour histogram,
luma entropy, edge density, alpha coverage, capped unique-colour count, dominant colour) and
runs a deterministic, no-ML **classification** (photo vs graphic/logo vs icon vs document vs
ui-screenshot) that collapses to three optimization buckets (`Lossy` / `LosslessFlat` /
`MixedSafe`). It lands **standalone** — no command reads it yet — so the whole stage is additive
and every existing test stays green. It is the base STAGE-012 (auto-decide) and the later
projects (planner, lint, manifest) all consume.

## Why Now

- **It is the dependency root.** Format auto-decision, `explain`, `lint`, the planner, and the
  manifest all read `Analysis`. Building it once, shared and correct, prevents each feature from
  re-scanning pixels or duplicating heuristics.
- **It's cheap and low-risk.** One extra linear pass on the already-decoded buffer (far cheaper
  than the up-to-8 candidate re-encodes the quality search already does), no new dependency, and
  because it's a sibling module that nothing calls yet, it can't regress existing behaviour.
- **The design is settled.** `docs/research/proj-002-design-analysis-layer.md` and
  `-classification.md` specify the trait shapes, the file:line integration points, the load-once
  invariant, and the migration order — this stage executes that plan.

## Success Criteria

- `Analysis::compute(&Image)` returns an immutable context with all features, computed in a
  single pass, never re-decoding and never touching disk.
- Classification routes photo→`Lossy`, logo/flat-graphic/icon/document→`LosslessFlat`,
  ui-screenshot/illustration→`MixedSafe` correctly on a labeled fixture corpus.
- `compute` is bounded and **never panics** on any input (0×0, 1-px, huge, degenerate alpha),
  returning a typed `AnalysisError` where a value is undefined; `unique_colors` is capped, never
  an unbounded collection (STAGE-006).
- The module lands with unit tests and is registered in `lib.rs`; **every existing test stays
  green**; no CLI behaviour changes; `just deny` stays green with no new default dependency.

## Scope

### In scope
- The `src/analysis/` module: `Analysis` (immutable, accessors-only), `Analysis::compute`, the
  feature extractors, `AnalysisError`. **(SPEC-046)**
- Deterministic internal classification → `ImageClass` + `OptBucket`, with a labeled fixture
  corpus and the tuned threshold constants recorded in a DEC. **(SPEC-047)**

### Explicitly out of scope
- Wiring `Analysis` into any command (STAGE-012 does that for `optimize`).
- An `Analyzer` trait-object registry (a plain `compute` is the decision; see the design brief).
- Lazy per-field `OnceCell` memoization — start eager single-pass; only split later if profiling
  shows a command pays for a field it doesn't read.
- Surfacing a `classify` command or any user-facing classification output.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-046 (shipped on 2026-07-06) — the `src/analysis/` layer: `Analysis` immutable context +
  single-pass feature extractors (histogram, entropy, edge density, alpha coverage, capped
  unique-colours, dominant colour) + bounded no-panic `AnalysisError`; landed standalone, all
  tests green (suite 440). PR #53 (f6c046e).
- [ ] SPEC-047 (design → build NEXT) — deterministic no-ML classification (`ImageClass` → three
  `OptBucket`s) on the SPEC-046 features + `source_format`/`has_exif` container priors + a labeled
  fixture corpus; thresholds + safe-fallback bias recorded in DEC-047

**Count:** 1 shipped / 1 in-design / 0 pending

## Design Notes

- **Layering:** `src/analysis/` is a peer of `src/operation/`, depending only on `::image`,
  `crate::image`, and `std` — never `clap`/`cli`/`sink`/`recipe`/disk (mirror the `operation/`
  module header). `Analysis` is NOT on the `Operation` trait and NEVER a recipe step (it's
  derived, not serialized — this preserves the byte-stable round-trip).
- **Load-once (DEC-002):** compute on the borrowed decoded buffer in one traversal; the 512 MiB
  decode cap already bounds the input, so `compute` is O(pixels) over a bounded array.
- **The minimal decisive subset:** four features carry the format decision — `has_exif` (camera
  prior), capped `unique_colors`, `edge_ratio`+`flat_ratio`, `entropy` — plus `has_alpha`.
  Everything else refines the label, not the decision (see `-design-classification.md`).
- Weighty decision → its own `DEC-*`: the classification thresholds + the safe-fallback bias
  (default to `Lossy`/photograph under uncertainty, bounded by the existing SSIMULACRA2 target).

## Dependencies

### Depends on
- STAGE-001 (PROJ-001) — the `Image`/decode-once model and `ImageInfo` the extractors borrow.
- DEC-002 (decode-once), DEC-034 (decode limits — the bound `compute` inherits).

### Enables
- STAGE-012 (auto-decide & explain) — the format decision + `explain` read `Analysis`.
- PROJ-003 (planner), PROJ-004 (lint), PROJ-005 (manifest) — all read this layer.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
