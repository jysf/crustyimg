---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-136
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-055
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-opus-5        # usually same Claude, different session
  created_at: 2026-10-28

references:
  decisions:
    - DEC-100
    - DEC-101
    - DEC-095
    - DEC-091
    - DEC-088
    - DEC-018
  constraints:
    - single-image-library
    - no-new-top-level-deps-without-decision
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
  related_specs:
    - SPEC-128
    - SPEC-129
    - SPEC-121
    - SPEC-122
    - SPEC-131
    - SPEC-132

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-XXX's <capability>". Optional; null is acceptable.
value_link: >
  Delivers all three of STAGE-055's `delivers` items — the op, the linear
  application at the source's depth, and the `.cube` content in the cache key.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-136: A recipe can express a `.cube` LUT

## Context

This is the only spec in `STAGE-055`, and the missing consumer of four shipped
ones.

`DEC-100` built crustyimg's asset-bearing-op registration seam for `watermark`
and stated it was generalized "enough for a SECOND parameter-rich,
asset-bearing op (the `.cube` LUT, a decided but unscheduled STAGE-050 backlog
item) to register through it with no further change to `registry.rs`". It
proved that with a fixture, not the real op —
`tests/registry_seam.rs`'s `a_lut_shaped_op_is_strict_and_resolvable_through_one_table()`,
documented as "a `.cube`-LUT-shaped op — a file to read plus a 0..=1 strength".
`SPEC-129`/`DEC-101` then put asset **content** in the build cache key, which is
the actual differentiator here. `SPEC-121`/`SPEC-122` (`DEC-095`) made ops
preserve bit depth and gave the operation module its linear-light helpers.

So the capability is a hole in the middle of infrastructure built to its shape.
The feature was never in question — `docs/backlog.md`'s entry records that "the
feature was never in question; only the dependency advice was, and the
maintainer reversed that on 2026-08-10 in favour of in-housing."

Two things changed that made this writable now rather than earlier:

1. **The blocker cleared.** The backlog entry carried "⚠ Blocked on the
   colour-space question" — a `.cube` applied in a non-linear pipeline is baked
   against the wrong transfer function. The 8-bit half became conditional on an
   8-bit source when `SPEC-121` shipped; the transfer-function half closed when
   `SPEC-122` shipped the linear machinery.
2. **The placement question was settled** (2026-10-28) against a sibling
   proposal to host the engine in `irradiance`. It stays here. See
   `STAGE-055`'s Design Notes for the four reasons, the load-bearing one being
   that `single-image-library` is a **blocking** constraint.

## Goal

Add a `lut` operation that applies a `.cube` LUT at a declared strength, in
linear light, at the input's own bit depth — reachable from a recipe and from
the CLI — and fold the `.cube` file's content into the build cache key so a
grade change invalidates exactly the outputs that used it.

## Inputs

- **Files to read:**
  - `src/operation/registry.rs` — `register_with_assets` / `asset_keys` / the
    `Constructor` signature. The whole point is that this file does not change.
  - `src/operation/mod.rs` — the `Operation` trait, `is_sixteen_bit`,
    `is_luma`, `narrow_rgba8` / `narrow_rgba16`, `srgb_to_linear`,
    `linear_to_srgb`, `srgb8_linear_table`, `encode_srgb8`, `encode_srgb16`.
    `Watermark` is the worked example of an asset-bearing op end to end.
  - `src/build/cache.rs` — `absorb` (`pub(crate)` for this), `compute_key`,
    `recipe_hash`.
  - `src/cli/build.rs` — `target_recipe_hash`, where the resolved-asset
    fingerprint composes in.
  - `src/cli/common.rs` — `resolve_recipe_assets`, the recipe IO boundary.
  - `tests/registry_seam.rs` — the fixture this spec replaces with the real op.
  - `tests/build_watermark_cache.rs` — the model for the cache-invalidation
    test.
  - `tests/linear_light_resize.rs` and `tests/colour_type_preservation.rs` —
    the models for the linear and depth assertions.
  - `docs/backlog.md`, the "`.cube` LUT op — build the reader in-house" entry —
    the measured budget and the two corrections to the triage's dependency note.
- **External APIs:** none. Adding one is a failure of this spec.
- **Related code paths:** `src/operation/`, `src/build/`, `src/cli/`, `fuzz/`.

## Outputs

- **Files created:**
  - `src/operation/lut.rs` — the `.cube` parser, the interpolator, and the
    `Lut` op. New module rather than more of `mod.rs`, which is already 125 KB;
    `SPEC-097`'s split of `src/cli/` is the precedent.
  - `tests/lut.rs` — the op's own tests.
  - `tests/fixtures/lut/*.cube` — fixtures: an identity cube, a known non-identity
    cube with hand-computed expected values, a 1D cube, and malformed ones.
  - `fuzz/fuzz_targets/cube_parse.rs` — parser fuzz target.
- **Files modified:**
  - `src/operation/mod.rs` — `pub(super)` (or `pub(crate)`) on the linear
    helpers the new module needs, and `pub mod lut;`. **Widening their
    visibility is the one invasive change here** — `DEC-095` already did
    exactly this for `color_type_bit_depth` and recorded the path in its
    `affected_scope`; do the same.
  - `src/operation/registry.rs` — **one line**: the `register_with_assets`
    call. If anything else in this file has to change, `DEC-100`'s central
    claim is false and that is a finding worth reporting, not patching around.
  - `src/cli/ops.rs` — the CLI verb.
  - `src/cli/build.rs` — nothing, if `resolve_recipe_assets` is already
    generic over `asset_keys`. Verify rather than assume.
  - `docs/recipes.md` (or wherever the op reference lives) — the `lut` op.
- **New exports:** `operation::lut::Lut`, `operation::lut::CubeLut`, and a
  typed `CubeError`. Nothing else leaves the crate.

## Acceptance Criteria

- [ ] **AC-1.** A recipe with a `lut` step naming a `.cube` file and a
  `strength` applies the grade; the CLI verb reaches the same code with the
  same result on the same inputs.
- [ ] **AC-2.** `registry.rs` changes by exactly one `register_with_assets`
  call and nothing else. (Discharges `DEC-100`'s generalization claim against
  the real op.)
- [ ] **AC-3.** An `Rgb16` input produces an `Rgb16` output whose values carry
  more than 8 bits of information, and the grade is computed in linear light —
  demonstrated against an independently computed expectation, not against this
  code's own output.
- [ ] **AC-4.** `strength = 0.0` is **byte-exact** identical to the input, and
  `strength = 1.0` on the same non-identity LUT is **not** — both asserted in
  one test, so neither can pass vacuously.
- [ ] **AC-5.** An identity `.cube` at `strength = 1.0` is byte-exact
  identical to the input, while the non-identity cube at the same strength is
  not.
- [ ] **AC-6.** Editing the `.cube` file's **contents** (same path, same mtime
  where the platform allows) changes the build cache key and rebuilds that
  target; editing an unrelated file does not. (`build_watermark_cache.rs` is
  the model.)
- [ ] **AC-7.** `build --check` fails when a `.cube` has changed since the
  lockfile was written, naming the target.
- [ ] **AC-8.** Every malformed input is a typed `CubeError` at exit 1, never a
  panic and never a silently applied partial grade: missing size directive,
  size out of range, wrong entry count, non-numeric entry, out-of-domain value,
  truncated mid-table, both `LUT_1D_SIZE` and `LUT_3D_SIZE` present, empty
  file, and a 1 GB declared size.
- [ ] **AC-9.** `strength` outside `0.0..=1.0`, and non-finite, is refused in
  the recipe schema and on the CLI — per `SPEC-131`'s typed params and
  `SPEC-135`'s ruling on non-finite numbers.
- [ ] **AC-10.** The parser survives the fuzz target with no panics.
- [ ] **AC-11.** `just lint-all` and `just check` clean; `--no-default-features`
  builds; `just wasm-check` passes (the module is engine-side and must stay
  `std::fs`-free).
- [ ] **AC-12.** A benchmark row for the op at 8-bit and 16-bit, reported
  rather than gated — `STAGE-055`'s Design Notes explain why the number
  matters and why acting on it is a different spec.

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass.

- **`tests/lut.rs`**
  - `"identity_cube_is_byte_exact_and_a_real_cube_is_not"` — asserts AC-5 and
    its own positive control in one test: identity LUT at `strength = 1.0`
    leaves bytes unchanged, non-identity LUT at `strength = 1.0` changes them.
    A no-op implementation fails the second half.
  - `"strength_zero_is_byte_exact_and_strength_one_is_not"` — asserts AC-4,
    same structure. These two tests are deliberately paired: either alone can
    pass vacuously.
  - `"grade_is_applied_in_linear_light_not_srgb"` — asserts AC-3's hard half.
    Uses a cube that is a pure 0.5 linear gain, on a mid-grey input, and
    asserts the output equals the hand-computed linear-space answer **and is
    measurably different from the sRGB-space answer**. Both expected values
    written as literals in the test, derived outside this code. Without the
    second assertion the test cannot tell the two spaces apart.
  - `"sixteen_bit_input_stays_sixteen_bit_and_keeps_sub_8_bit_detail"` —
    asserts AC-3's depth half: output `ColorType` is `Rgb16`, and a gradient
    whose steps are smaller than one 8-bit level survives with distinct values.
    Asserting only the `ColorType` would pass on an 8-bit-widened buffer.
  - `"luma_input_narrows_per_dec_095"` — an `L16` input through a grey-preserving
    LUT narrows back to a luma type; through a colour LUT it does not. DEC-095's
    chroma rule, exercised on this op.
  - `"malformed_cube_is_typed_error_never_panic"` — table-driven over every
    case in AC-8, asserting the specific `CubeError` variant per row, not just
    `is_err()`. A single catch-all variant would make this test worthless.
  - `"strength_out_of_range_is_refused"` — AC-9, CLI and recipe paths.
- **`tests/registry_seam.rs`**
  - `"the_real_lut_op_registers_through_the_unchanged_table"` — AC-2's
    behavioural half: the real `lut` op resolves through the same
    `asset_keys` → `set_resolved_bytes` → `build` path the fixture op used.
    The "no change to `registry.rs`" half is a diff assertion for the build
    cycle to report, not a test.
- **`tests/build_cache.rs`** (or `tests/build_lut_cache.rs`, following
  `build_watermark_cache.rs`)
  - `"editing_a_cube_file_invalidates_only_its_targets"` — AC-6 with its
    negative control: the target using the `.cube` rebuilds, a target that does
    not use it stays cached.
  - `"build_check_fails_on_a_changed_cube"` — AC-7.
- **`tests/hostile_inputs.rs`**
  - `"a_cube_declaring_a_gigabyte_is_refused_before_allocating"` — the
    resource-limit case from AC-8. Asserts the refusal happens without the
    allocation, which is the actual property; a test that merely observes an
    error could be passing after a 1 GB alloc.

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-100` — the asset-op seam this registers through, and the source of AC-2.
  Its claim that a second asset-bearing op needs no `registry.rs` change is
  what this spec discharges.
- `DEC-101` — asset content in the cache key; `absorb` is `pub(crate)` for it.
- `DEC-095` — ops preserve colour type and bit depth; `Resize` resamples in
  linear light. Both the narrowing rules and the linear helpers come from here,
  and the "transfer function is assumed sRGB" statement is inherited rather
  than re-decided.
- `DEC-091` — Fence A puts a LUT path + strength in the workhorse (the
  parameters generalize across a batch); Fence B keeps *authoring* in lab.
- `DEC-088` — the external-integration triage. A `.cube` is a **file format**,
  so tier 1, which the DEC calls "Preferred … hashable — so it participates in
  the build cache key."
- `DEC-018` — the licence gate that would apply if a dependency were taken.
  None is.

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `single-image-library` (**blocking**) — exactly one pixel library (`image`).
  Hand-roll the interpolation; this is also the constraint that kept the engine
  out of `irradiance`.
- `no-new-top-level-deps-without-decision` — taking `lut-cube` or anything else
  needs a `DEC-*` first. The measured budget says not to.
- `no-unwrap-on-recoverable-paths` — typed `thiserror` in the library; `anyhow`
  only at the `main.rs`/`cli` boundary. The parser is all recoverable paths.
- `every-public-fn-tested` — `CubeLut`, `Lut`, and `CubeError`'s surface.
- `clippy-fmt-clean` — `just lint-all`, not `just lint`; CI runs
  `--all-targets`.

### Prior related work

- `SPEC-128` (shipped, PR #189) — built the seam; `Watermark` is the worked
  example of an asset-bearing op.
- `SPEC-129` (shipped, PRs #190/#191) — asset content in the cache key.
- `SPEC-121` + `SPEC-122` (shipped) — depth preservation and linear light.
  `SPEC-122`'s benchmark is the source of the working-type caveat.
- `SPEC-131` (shipped) — typed per-operation parameter structs; `strength`
  uses them. Also the source of the lesson that a schema checking `f64` while
  ops read `f32` silently loses small values — relevant to `DOMAIN_MIN`/`MAX`.
- `SPEC-132` (shipped) — `build --watch` already watches recipe asset paths, so
  a `.cube` edit triggers a rebuild with no work here. Verify it, don't rebuild it.
- `SPEC-135` (shipped, PR #207) — the ruling on non-finite numeric params;
  `strength` inherits it.

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- Changing the working buffer type (`F32x4` → `U16x4`). Measure and report
  (AC-12); the change is its own spec.
- A float working path, or promoting an 8-bit source. `Rgb32F` sources keep
  taking the 8-bit route.
- ICC interpretation. The transfer function stays assumed-sRGB.
- A wasm entry point that accepts LUT bytes.
- `.3dl` or any other LUT format.
- LUT authoring, chaining two LUTs, or a `lut` lint rule.
- Anything in `irradiance`.

## Notes for the Implementer

**Reuse, do not re-derive.** `Watermark` is the worked example for an
asset-bearing op: `register_with_assets` declares the key, `from_params` reads
back via `OperationParams::resolved_bytes`, and `to_toml` emits only the path.
Copy that shape. Likewise take the linear round trip from `resize_and_crop16`
rather than writing a second one — and note `srgb8_linear_table()` exists
because 256 `powf`s are free next to the image, while the 16-bit path calls
`srgb_to_linear` per sample.

**Pick the interpolation deliberately and write down why.** Trilinear (8 taps)
or tetrahedral (4 taps). Tetrahedral is what colour tools generally ship and it
avoids trilinear's error on the cube diagonals, which is where neutral greys
sit — so the artifact lands where it is most visible. Measure both on a real
`.cube` if the choice is close, ship one.

**The `.cube` format has sharp edges worth knowing before you parse it.** It
carries either `LUT_1D_SIZE` or `LUT_3D_SIZE` and never both; `DOMAIN_MIN`/
`DOMAIN_MAX` default to `0 0 0` / `1 1 1` and are frequently absent; the 3D
table is ordered red-fastest; comments are `#` to end of line; `TITLE` is
quoted. Sizes in the wild are 17, 25, 32, 33, 64 — a declared size is attacker
input, so bound it before allocating (AC-8's 1 GB row).

**`strength` blends in linear, not sRGB** — see `STAGE-055`'s Design Notes.
And `strength = 0.0` must be *byte-exact*, which means short-circuiting before
the linear round trip rather than relying on the round trip being lossless.
It is lossless (DEC-095 measured it exact for all 65,536 16-bit values), but
depending on that for an advertised no-op is a weaker guarantee than skipping
the work.

**Two repo lessons that bear directly on this spec's tests.** A paired
assertion is not optional here: this repo has shipped a test that passed
because its reference and candidate were wrong identically, and another that
stayed green because a removed guard's caller errored through a different path
that named the same words. The `identity`/`non-identity` and
`strength 0`/`strength 1` pairings above exist for that reason — do not
collapse them into single-sided tests because they look redundant.

**Diagnostics go to stderr**, so `-o -` stdout pipes stay clean.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Not "src/foo and tests": the enumeration is what a verify cycle diffs
  the real change against, and what tells you whether every affected `DEC-*` has
  the path in its `affected_scope`. SPEC-122's Deviations claimed "`src/operation`
  and `tests/` only" and was wrong by two `scripts/` files, which left
  `affected_scope` blind; SPEC-124's build added this section by hand after that
  lesson and still listed 8 of its 9 files. It is a checklist because it is easy
  to get wrong from memory — build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
