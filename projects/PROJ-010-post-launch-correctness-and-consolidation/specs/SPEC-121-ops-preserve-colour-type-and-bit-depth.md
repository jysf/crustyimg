---
task:
  id: SPEC-121
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
    - DEC-058
    - DEC-090
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
    - no-unwrap-on-recoverable-paths
  related_specs:
    - SPEC-122
    - SPEC-090

value_link: >
  STAGE-046's largest correctness item. Three op bodies widen every image to
  RGBA8 and never narrow back, so `resize`, `thumbnail`, `edit` and the flagship
  `web` add an all-opaque alpha channel (+12.4% bytes, measured) and halve
  16-bit input — on a tool whose thesis is quality-per-byte.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Established that the
        migration story the backlog entries worried about ALREADY EXISTS —
        the cache key carries `crate::version()` and the lockfile never
        promised hash stability across versions — which removes this spec's
        largest assumed risk.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-121: ops preserve colour type and bit depth

## Context

Three `Operation::apply` bodies convert to RGBA8 and never narrow back:

| site | op |
|---|---|
| `src/operation/mod.rs:197` | `Invert::apply` |
| `src/operation/mod.rs:396` | `Resize::apply` |
| `src/operation/mod.rs:816` | `Watermark::apply` |

**Measured by driving the binary and reading the output IHDR** — a structural
assertion, not a byte guess. From a 64×64 PNG with `colour_type=2` (RGB, no alpha):

| verb | out `colour_type` | |
|---|---|---|
| `convert --format png`, `optimize`, `auto-orient` | 2 (RGB) | ✅ |
| `resize --max 32`, `thumbnail --size 32`, `edit --invert` | **6 (RGBA)** | ❌ |
| **`web`** | **6 (RGBA)** | ❌ **the flagship** |
| `watermark --text` | 6 (RGBA) | arguable — compositing genuinely needs alpha |

The clean verbs are exactly the ones that run **no `Operation`**. Every verb that
folds a real op through the pipeline comes out RGBA. **`web` is
`auto-orient → resize → optimize`, so fixing `Resize` fixes the flagship.**

**Cost of the wasted channel:** on a 512×512 representative PNG, same pixels —
RGB **377,132 B** vs RGBA **423,756 B**, **+46,624 B / +12.4%** for a channel
carrying no information.

### The same call also truncates 16-bit — one fix, two defects

`to_rgba8()` is *also* why 16-bit input is silently halved. Measured on a 32×32
`bit_depth=16` RGB PNG: `convert`/`auto-orient` preserve **16-bit RGB**;
`resize`/`edit --invert` return **8-bit RGBA** — halved *and* alpha-added.

**"crustyimg is 8-bit internally" is not accurate as stated.** Decode preserves
the `DynamicImage` variant, `Identity` and `AutoOrient` preserve it, and the
default encode path (`img.pixels().write_to(..)`, `src/sink/mod.rs`) preserves it.
**Only the three op bodies collapse it.** `DynamicImage` already has `ImageRgb16`
/ `ImageRgba16`, so **no type change is needed anywhere.**

Full evidence: `docs/backlog.md`, `## ⚠ Live defect — ops widen to RGBA and never
narrow back`. **Read it; do not re-derive it.**

## The design calls — settled here

### Call 1 — "widen to work, narrow to write"

An op preserves the input's colour type and bit depth **unless it genuinely needs
more**. Widening internally is fine; returning a widened image is the defect.

The narrowing must be **lossless-only**: narrow RGBA→RGB only when every alpha
sample is opaque, and 16→8 never (that is a downgrade, not a narrowing). An op
that was handed 8-bit returns 8-bit; one handed 16-bit returns 16-bit.

### Call 2 — `Watermark` is the exception, and it is explicit

Compositing a translucent overlay genuinely produces alpha. **`Watermark` may
return RGBA from an RGB input** — but only when the overlay actually contributed
non-opaque samples. If the composite is fully opaque, Call 1 applies and it
narrows like the others. **Decide this in code, not by exempting the op.**

### Call 3 — a lossy target's 8-bit downgrade is REPORTED, not silent

JPEG and lossy WebP are 8-bit. Feeding them 16-bit pixels is a real downgrade and
the user should be told, in the spirit of SPEC-090's honest size reporting. This
is a **one-line diagnostic at the sink**, not a new policy.

### Call 4 — ⚡ THE MIGRATION ALREADY EXISTS. Do not build one.

Both backlog entries flag "this invalidates every PROJ-007 build lockfile" as the
thing that makes the fix non-trivial, and the linear-light entry asks whether the
cache key needs a new colour-pipeline-version component.

**Established at design, by reading the code — the answer is no:**

- **`cache_key_for` already includes `crate::version()`**
  (`src/cli/build.rs:294`, via `cache::compute_key(crate::version(), …)`). A
  release changes every key, so **old and new renders cannot collide in the
  cache.** No new key component is needed.
- **The lockfile never promised output-hash stability across versions.**
  `src/build/lock.rs:32-36`: `hash` is *"recorded as observed under `[env]`, never
  promised"*, and is explicitly not stable *"across arch/OS/codec versions"*.
  `key` is a function of inputs — including the version — so a bump is *"an
  unambiguous, cross-machine drift signal"*, which is the designed behaviour.

So a user upgrading sees key changes, `--frozen` fails, and they regenerate the
lock. **That is the normal upgrade path, already specified.** This spec's job is
to **drive it and confirm it**, and make sure the release notes say outputs
change — not to invent machinery. If driving it shows something the contract does
not cover, **that is a finding: report it, do not design around it.**

### Call 5 — no new flag

"Preserve what you were given when the target format can hold it" is not
something a user should have to ask for.

## Inputs

- `src/operation/mod.rs:190-210` (`Invert`), `:390-530` (`Resize`), `:810-830`
  (`Watermark`).
- `src/sink/mod.rs` — the `write_to` path that already preserves the variant.
- `src/cli/build.rs:275-302` (`cache_key_for`) and `src/build/lock.rs:20-45` —
  Call 4's evidence.
- `docs/backlog.md`'s RGBA-widening entry.

## Outputs

- **Files modified:** `src/operation/mod.rs` (the three bodies), `src/sink/mod.rs`
  (Call 3's diagnostic), `tests/`.
- **New DEC expected:** yes — one covering the byte-change and its migration
  posture, **shared with SPEC-122** so the wave is one decision, not two.
  `affected_scope`: `src/operation/**`, `src/sink/**`.
- ⚠ **Correct the "8-bit internally" claim wherever it is written** — it outlives
  this stage. Part of this spec, not a follow-up.

## Acceptance Criteria

- [ ] **AC-1.** An RGB (`colour_type=2`) PNG through `resize`, `thumbnail`,
      `edit --invert` and **`web`** comes out **`colour_type=2`**. Asserted on the
      output **IHDR**, not on file size.
- [ ] **AC-2.** A 16-bit RGB PNG through the same verbs comes out **16-bit**.
- [ ] **AC-3.** **An RGBA input with a genuinely translucent pixel stays RGBA.**
      The narrowing is lossless-only — this is the control that stops "always
      narrow" from destroying real alpha.
- [ ] **AC-4.** **`Watermark` returns RGBA only when the overlay contributed
      non-opaque samples**, and narrows otherwise (Call 2). Both directions tested.
- [ ] **AC-5.** **A lossy target reports the 8-bit downgrade** (Call 3), pinned by
      a test asserting the message.
- [ ] **AC-6.** **The byte win is measured, not assumed** — assert the RGB output
      is materially smaller than the RGBA one on the same pixels.
- [ ] **AC-7.** **The verbs that were already correct are unchanged** —
      `convert`, `optimize`, `auto-orient` byte-identical to `main`.
- [ ] **AC-8.** **Call 4 driven, not reasoned.** Build a target with a committed
      lockfile on `main`'s binary, upgrade to the branch binary, and show: the
      cache key changes, `--frozen` fails, regeneration succeeds, and no stale
      cache entry is served. **If the contract does not hold, stop and report.**
- [ ] **AC-9.** **A negative control per op body** — revert `Invert`, `Resize` and
      `Watermark` independently; each turns only its own tests red.
      **The evidence is the behavioural flip, not a binary hash** (AGENTS §15).
- [ ] **AC-10.** Clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential,
      through `rtk proxy`: default, `--no-default-features`, `--features
      webp-lossy`. Clippy and `fmt --check` each. Own `main` baseline. Then read
      the CI legs individually.

## Failing Tests

Written during **design**, before build. AC-1 through AC-4 **fail on today's
`HEAD`** — this is a live defect with a genuine red-to-green transition.

- **`tests/colour_type_preservation.rs`** (new)
  - `"rgb_png_stays_rgb_through_resize_thumbnail_edit_and_web"` — AC-1. **RED.**
  - `"sixteen_bit_png_stays_sixteen_bit"` — AC-2. **RED.**
  - `"translucent_rgba_input_keeps_its_alpha"` — AC-3. Passes today; the control.
  - `"watermark_narrows_when_the_composite_is_opaque"` — AC-4. **RED.**
  - `"watermark_keeps_alpha_when_the_overlay_is_translucent"` — AC-4. Passes today.
  - `"rgb_output_is_smaller_than_rgba_for_the_same_pixels"` — AC-6.
  - `"convert_optimize_auto_orient_bytes_unchanged"` — AC-7.
- **`tests/sink.rs`** — `"lossy_target_reports_the_eight_bit_downgrade"` — AC-5. **RED.**

## Implementation Context

### Decisions that apply
- **DEC-058** — the cache-key composition Call 4 rests on.
- **DEC-090** — the diagnostic channel Call 3's message uses.

### Out of scope
- **Linear-light resampling — SPEC-122.** Same function, different premise.
  **Sequence the two together** so the byte change and its migration are paid once.
- A 16-bit-throughout pipeline as a goal. Only these three bodies collapse it.
- Any new user-facing flag.

## Notes for the Implementer

- **The narrowing must be lossless-only.** AC-3 exists because "always narrow"
  passes AC-1 and destroys real transparency.
- **Three op bodies are three claims** — AC-9 wants three reverts.
- **Do not build a migration.** Call 4 says one exists; your job is to drive it.
- **Budget in exchanges, not minutes** (~250). Cost tracks rebuilds and message
  count, not wall clock.
- macOS has no `timeout(1)`. `git commit -s`. **Own git worktree.** **Do not merge
  the PR. Do not bump the version.**
- Follow `projects/_templates/prompts/closing-steps-snippet.md`, including
  `just advance-cycle SPEC-121 verify` — and confirm the `cycle:` line moved.

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
