---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-046
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-002
  stage: STAGE-011
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # usually same Claude, different session
  created_at: 2026-07-05

references:
  decisions: [DEC-002, DEC-034]
  constraints: [untrusted-input-hardening, no-agpl-default-deps, ergonomic-defaults]
  related_specs: [SPEC-016]

value_link: >
  Infrastructure enabling STAGE-011's shared Analysis layer — the computed-once
  context every later feature (format auto-decide, explain, lint, planner, manifest)
  reads.

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-046: the `src/analysis/` layer — computed-once image analysis

## Context

PROJ-002 turns crustyimg into an analysis-driven optimization engine. Every feature in the wave
— format auto-decision (STAGE-012), `explain`, and later `lint`/planner/manifest — needs the same
derived facts about an image (how many colours, is it a photo or a graphic, does it have
meaningful alpha, how much edge detail). Today there is **no** such layer: `Operation`s are pure
pixel transforms with no shared context, and `ImageInfo` (`src/image/mod.rs`) carries only cheap
metadata (dims, colour type, has_alpha). This spec builds the foundation — a new `src/analysis/`
module — and lands it **standalone**, wired into nothing, so it is purely additive and every
existing test stays green. Classification (the photo-vs-graphic verdict) is the very next spec
(SPEC-047) built on the features this one produces.

Design is fully specified in `docs/research/proj-002-design-analysis-layer.md` (architecture,
file:line integration points, migration order) and `-classification.md` (the feature set).

## Goal

Add a `src/analysis/` module exposing an immutable, computed-once `Analysis` context and
`Analysis::compute(&Image) -> Result<Analysis, AnalysisError>` that extracts the decisive image
features in a single bounded, no-panic pass over the already-decoded buffer — with no CLI
behaviour change and no new default dependency.

## Inputs

- **Files to read:**
  - `docs/research/proj-002-design-analysis-layer.md` — the authoritative architecture (trait
    shapes, load-once invariant, what NOT to do).
  - `docs/research/proj-002-design-classification.md` — the feature definitions (single-pass core
    + edge pass) this spec implements (classification itself is SPEC-047).
  - `src/image/mod.rs` — the `Image`/`ImageInfo` core to borrow from; the model for accessors +
    no-panic decode bounds (`:32-37`, `:91-113`, `:170-190`, `:251-257`).
  - `src/operation/mod.rs` — the module-header layering rule + `OperationError` pattern to mirror.
  - `src/quality/mod.rs` — a sibling module with the same "no clap/sink/files" self-containment.
- **Related code paths:** `src/lib.rs` (module registration), `src/operation/mod.rs:487-509`
  (the `MAX_EDGE`/`MAX_AREA` bounding pattern to mirror for `unique_colors`).

## Outputs

- **Files created:** `src/analysis/mod.rs` — the `Analysis` struct, `Analysis::compute`, the
  feature extractors, and `AnalysisError`.
- **Files modified:** `src/lib.rs` — register `pub mod analysis;`.
- **New exports:**
  - `pub struct Analysis` — immutable, accessors-only (no public fields, no `&mut self`).
    Fields (private, exposed via `pub fn`): dims, `ColorType`, `alpha_translucent: f32`,
    `alpha_transparent: f32`, `unique_colors: UniqueColors` (`Exact(u32)` | `Saturated(cap)`),
    `histogram` (luma 256-bin + quantized RGB), `entropy: f32`, `bimodality: f32`,
    `edge_ratio: f32`, `flat_ratio: f32`, `sat_low_ratio: f32`, `gray_ratio: f32`,
    `dominant_color: [u8;4]`. (Classification fields `class`/`opt_bucket` are added in SPEC-047.)
  - `pub fn compute(img: &Image) -> Result<Analysis, AnalysisError>`.
  - `pub enum AnalysisError` (thiserror; e.g. `DegenerateDimensions`) — typed, no-panic.
- **Database changes:** none.

## Acceptance Criteria

- [ ] `Analysis::compute(&img)` computes all features in a **single traversal** of the decoded
  buffer (plus the O(256) histogram-derived scalars and one edge pass); it never re-decodes and
  never touches disk (assert by construction / no `Image::load`/`from_bytes`/fs in the module).
- [ ] `Analysis` is immutable: no public fields, all reads via accessors, no `&mut self` method.
- [ ] `unique_colors` is **capped** (e.g. at 4096) and short-circuits — never an unbounded
  collection; memory stays O(1) in the histogram/scalars regardless of image size.
- [ ] `compute` **never panics** on any input: 0×0 / 1-px / degenerate → a typed `AnalysisError`
  (or a well-defined default), fully-opaque and fully-transparent alpha both handled, a
  512 MiB-bounded buffer handled (the decode cap already applies).
- [ ] Feature values are correct on synthetic fixtures: a solid-colour image → `entropy ≈ 0`,
  `unique_colors = Exact(1)`, `edge_ratio ≈ 0`, `flat_ratio ≈ 1`; a smooth gradient → higher
  entropy + low edge_ratio; a sharp checkerboard → high edge_ratio; an RGBA image with holes →
  `alpha_transparent > 0`.
- [ ] Determinism: `compute` on the same pixels yields byte-identical feature values across runs
  and platforms (integer or fixed-order f32 accumulation; no RNG, no wall-clock).
- [ ] The module depends only on `::image`, `crate::image`, and `std` (no `clap`/`cli`/`sink`/
  `recipe`/fs) — enforced by the module header + review.
- [ ] `just deny` stays green (no new dependency); **all existing tests pass unchanged**; no CLI
  command output changes.

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in **build** is to make these
pass. Fixtures are generated in-test (crustyimg already has native solid/gradient/noise
generators — reuse them; see `src/cli` create path / test helpers).

- **`src/analysis/mod.rs` (unit tests)**
  - `"solid image → zero entropy, one unique colour, flat"` — a 64×64 solid RGB image:
    asserts `entropy` ≈ 0 (± ε), `unique_colors == Exact(1)`, `flat_ratio` ≈ 1.0,
    `edge_ratio` ≈ 0.0.
  - `"vertical gradient → nonzero entropy, low edges"` — asserts `entropy` above a threshold and
    `edge_ratio` below a threshold (smooth, not edgy).
  - `"checkerboard → high edge ratio"` — a hard 8×8 checkerboard: asserts `edge_ratio` above a
    high threshold and `flat_ratio` low.
  - `"few-colour graphic → capped unique colours stays Exact and small"` — a 4-colour image:
    asserts `unique_colors == Exact(4)`.
  - `"many-colour photo-like → unique colours Saturated at cap"` — a noise image: asserts
    `unique_colors == Saturated(4096)` (or the chosen cap), proving the short-circuit.
  - `"rgba with transparent region → alpha_transparent > 0"` — asserts `alpha_transparent` and
    `alpha_translucent` reflect the hole.
  - `"degenerate dimensions do not panic"` — a 0-area / 1-px input returns `Err(AnalysisError::…)`
    (or a defined default) and does **not** panic.
  - `"determinism: two computes are identical"` — `compute` twice on the same buffer → equal
    feature values.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply
- `DEC-002` — decode-once: compute on the single borrowed decoded buffer; never re-decode.
- `DEC-034` — decode limits (dims ≤ 65535, alloc ≤ 512 MiB): the input buffer is already bounded,
  so `compute` is O(pixels) over a bounded array. `unique_colors` adds its own cap (this spec).

### Constraints that apply
- `untrusted-input-hardening` — `compute` is a new untrusted-input surface: typed error, **no
  `unwrap`/`expect`/`panic!`** on recoverable paths; bound every accumulator (`unique_colors`
  capped). Mirror the `resize` op's `MAX_EDGE`/`MAX_AREA` discipline.
- `no-agpl-default-deps` — no new dependency at all; features are hand-computed on the `image`
  buffer (no `imageproc` — it drags sdl2/nalgebra).
- `ergonomic-defaults` — n/a to behaviour here (no CLI surface), but keep the API small.

### Prior related work
- `SPEC-016` (shipped) — established `src/quality/` as a self-contained pixel+metric module with
  no clap/sink deps; `src/analysis/` mirrors that layering exactly.

### Out of scope (for this spec specifically)
- Classification (`ImageClass`/`OptBucket`) — SPEC-047 (built on these features).
- Wiring `Analysis` into any command — STAGE-012.
- Lazy `OnceCell` per-field memoization — start eager single-pass.
- An `Analyzer` trait / registry — a plain `compute` is the decision (design brief §"What NOT to
  do").
- `serde` derive on `Analysis` / any JSON — not needed until `explain`/manifest.

## Notes for the Implementer

- Convert once to a working view (`to_rgba8()` / `to_luma8()`) exactly as existing ops do; fuse
  the histogram/alpha/unique-colour/dominant accumulation into one loop, then derive
  entropy/bimodality from the luma histogram (O(256)). The edge pass is a second linear sweep
  (Sobel-lite, integer, no kernel library). On very large images, stride-sample the edge pass
  (classification tolerates it) — but keep it deterministic (fixed stride, not sampled).
- Keep every threshold/const (cap size, edge threshold, stride) named in one place for testability
  and a future tuning DEC.
- The `unique_colors` cap value (4096) is a shared anchor with the classification/format-decision
  work — don't hardcode a second copy in SPEC-047; expose it.

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

1. **What was unclear in the spec that slowed you down?**
   — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>
3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>
2. **Does any template, constraint, or decision need updating?**
   — <answer>
3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
