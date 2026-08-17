---
task:
  id: SPEC-122
  type: bug
  cycle: design
  blocked: false
  priority: high
  complexity: M

project:
  id: PROJ-010
  stage: STAGE-046
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5
  created_at: 2026-08-16

references:
  decisions:
    - DEC-092
    - DEC-019
    - DEC-058
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-120
    - SPEC-121

value_link: >
  STAGE-046's last defect, and the only one whose premise was measured before it
  was specced. `Resize::apply` resamples non-linear sRGB as if it were linear;
  against an independent reference the shipped downscale scores 70.45 and 84.45
  where a linear-light prototype scores ~100.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Scoped down from the
        backlog entry on two SPEC-120 findings: the premultiplied-alpha half is
        false, and the migration the entry worried about already exists.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-122: `resize` resamples in linear light

## Context

`Resize::apply` converts to RGBA8 and hands `PixelType::U8x4` to
`fast_image_resize` with Lanczos3 (`src/operation/mod.rs:395-527`). No gamma
handling anywhere. So crustyimg resamples **non-linear sRGB values as if they
were linear** — high-contrast edges darken on downscale, worst on thin bright
features against dark backgrounds.

**This spec exists because SPEC-120 measured it first, and the gate fired.**
DEC-092:

| case | source → target | mean signed luma err | today's SS2 | linear prototype | Δ |
|---|---|---|---|---|---|
| synthetic worst case (positive control) | 2048²→256² | −0.104350 (−88.07%) | −63.85 | 100.00 | **+163.85** |
| `graphic_large.png` | 512²→128² | −0.001386 (−0.44%) | **70.45** | 100.00 | +29.55 |
| `photo_forest_cc0.jpg` | 800×532→200×133 | −0.004920 (−2.63%) | **84.45** | 99.41 | +14.96 |

**Read the load-bearing numbers correctly.** The prototype's ~100 and the Δ
column are partly self-fulfilling — a linear-light Lanczos prototype scored
against a linear-light Lanczos reference should agree. **The claim that survives
is the first column: today's shipped path scores 70.45 and 84.45 against an
independent reference** (ImageMagick 7.1.2-29 Q16-HDRI, `-colorspace RGB`).

The positive control is what makes those readable rather than an uninterpretable
null: an −88% physical error registered as a 163.85-point swing, so **the
instrument can see this defect.**

### ⚡ Scoped down by SPEC-120 — this is ONE premise, not two

The backlog entry paired linear light with premultiplied alpha. **The alpha half
is FALSE.** `fast_image_resize` 6.0.0's `ResizeOptions::default()` sets
`mul_div_alpha: true` (`src/resizer.rs:52-60`) and `ResizeOptions::new()` *is*
`Default::default()` (`:63-64`) — and `Resize::apply` overrides only the
algorithm. **It has always premultiplied.** Do not add alpha handling.

## The design calls — settled here

### Call 1 — convert inside `Resize::apply`; the pipeline stays as it is

Linearize → resample → re-encode to the output transfer function, **inside the
op**. No pipeline-wide colour management, no new `Image` field, no 16-bit-
throughout project. `fast_image_resize` 6.0.0 ships `F32x4` and `U16x4`
(`src/pixels.rs:26,21`), so the backend is already in place.

### Call 2 — the reference implementation already exists in this repo

**SPEC-120 committed a working linear-light prototype and its harness:**
`examples/spec120_linear_probe.rs` and `scripts/spec120_linear_light.py`.

**Start from the prototype.** It is the code that produced the numbers above, so
starting anywhere else means re-earning them. It is an `examples/` throwaway —
productionizing it is this spec's work.

### Call 3 — the measurement is the acceptance test, and it reuses SPEC-120's harness

Do not invent a new oracle. Re-run `scripts/spec120_linear_light.py` against the
**branch** binary and show the shipped path's score moving toward the reference on
the same three cases. **The independent reference stays independent** — regenerate
it with the same external tool, do not substitute crustyimg's own output.

### Call 4 — sRGB is assumed; ICC-aware conversion is NOT this spec

crustyimg keeps ICC profiles but does not interpret them. Assume the sRGB transfer
function, and **say so in the DEC**. An image with a non-sRGB profile is resampled
under an assumption that is wrong for it — better than today's, still an
assumption. Full colour management is its own project.

### Call 5 — the migration already exists (shared with SPEC-121)

`cache_key_for` includes `crate::version()` (`src/cli/build.rs:294`), and the
lockfile explicitly never promised output-hash stability across versions
(`src/build/lock.rs:32-36`). A release changes every key; old and new renders
cannot collide. **Drive it, do not design it** — and share **one DEC** with
SPEC-121 so the wave is a single decision.

## Inputs

- `src/operation/mod.rs:390-530` — `Resize::apply`.
- `examples/spec120_linear_probe.rs`, `scripts/spec120_linear_light.py` — the
  prototype and the harness.
- **DEC-092** — the verdict, the numbers, and the alpha refutation.
- `docs/backlog.md`'s linear-light entry, **including SPEC-120's appended result**.

## Outputs

- **Files modified:** `src/operation/mod.rs`, `tests/`.
- **New DEC:** **shared with SPEC-121** — the byte change, the sRGB assumption
  (Call 4), and the migration posture. `affected_scope`: `src/operation/**`.
- The `examples/` prototype may be deleted once production carries it — say which
  you chose.

## Acceptance Criteria

- [ ] **AC-1.** **The shipped path's SSIMULACRA2 against the independent reference
      improves materially on all three SPEC-120 cases**, re-run through that
      harness. State the before/after per case.
- [ ] **AC-2.** **The physical quantity improves too** — mean signed luminance
      error against the reference moves toward zero. Both metrics reported, and
      **if they disagree, that disagreement is the finding**.
- [ ] **AC-3.** **The positive control still fires** — the synthetic worst case
      shows the large swing. A fix that fixes only realistic cases is suspicious.
- [ ] **AC-4.** **The reference is regenerated independently** — same external
      tool, not crustyimg's own output. Committing a crustyimg-derived reference
      would make this untestable forever
      [[fixtures-from-the-code-under-test-cannot-fail]].
- [ ] **AC-5.** **Alpha behaviour is unchanged** — translucent-edge error stays at
      SPEC-120's measured 27/255 band. Call 2's refutation says nothing should move
      here; prove it didn't.
- [ ] **AC-6.** **Upscale and no-op resize are unaffected**, byte-identical to
      `main` where no resampling occurs.
- [ ] **AC-7.** **A negative control** — revert the linearization; the three cases
      return to 70.45 / 84.45 / −63.85. **The behavioural flip is the evidence.**
- [ ] **AC-8.** **The migration driven** (Call 5), shared with SPEC-121: key
      changes, `--frozen` fails, regeneration succeeds, no stale cache hit.
- [ ] **AC-9.** **Performance is measured and reported.** f32 resampling is more
      work than u8. Not a gate — but `resize` is the most-used op and nobody asked
      at design, so report it with controls.
- [ ] **AC-10.** Clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential,
      through `rtk proxy`: default, `--no-default-features`, `--features
      webp-lossy`. Clippy and `fmt --check` each. Then read the CI legs.

## Failing Tests

- **`tests/linear_light_resize.rs`** (new)
  - `"downscale_scores_better_against_an_independent_reference"` — AC-1. **RED.**
  - `"mean_luminance_error_moves_toward_zero"` — AC-2. **RED.**
  - `"translucent_edge_error_is_unchanged"` — AC-5. Passes today; the control.
  - `"upscale_and_noop_resize_are_byte_identical_to_main"` — AC-6.
- **Harness re-run** (AC-1/AC-3, recorded not committed as a test): the three
  SPEC-120 cases before and after.

## Implementation Context

### Decisions that apply
- **DEC-092** — the premise verdict and the alpha refutation. Binding.
- **DEC-019** — SSIMULACRA2 as the oracle; SPEC-120 proved it can see this defect.
- **DEC-058** — the cache-key composition Call 5 rests on.

### Out of scope
- Premultiplied alpha (**refuted**), ICC-aware conversion (Call 4), a 16-bit
  pipeline, and the colour-type fix (**SPEC-121** — same function, sequence together).

## Notes for the Implementer

- **Do not re-litigate the premise.** It was measured; DEC-092 is the record.
- **Start from `examples/spec120_linear_probe.rs`.**
- **Keep the reference independent** — AC-4 is what stops this becoming a test
  that cannot fail.
- **Budget in exchanges (~250), not minutes.**
- macOS has no `timeout(1)`. `git commit -s`. **Own git worktree.** **Do not merge
  the PR. Do not bump the version.**
- Follow `closing-steps-snippet.md`, including `just advance-cycle SPEC-122 verify`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
2. **Was there a constraint or decision that should have been listed but wasn't?**
3. **If you did this task again, what would you do differently?**

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
