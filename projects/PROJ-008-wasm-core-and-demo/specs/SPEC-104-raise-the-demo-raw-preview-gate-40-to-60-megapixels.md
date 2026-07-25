---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-104
  type: chore                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-008
  stage: STAGE-029
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-5     # one-constant retune; verify on Opus
  created_at: 2026-07-24

references:
  decisions: [DEC-082, DEC-063]
  constraints: [untrusted-input-hardening, ergonomic-defaults]
  related_specs: [SPEC-103]

value_link: >
  On-device tuning of SPEC-103's RAW gate: the 40 MP default wrongly blocked a
  real Leica DNG on desktop, where memory is ample — raise the gate so real RAW
  files convert in the demo.

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-104: raise the demo RAW preview gate 40 → 60 megapixels

## Context

SPEC-103 shipped the demo's RAW support behind `MAX_RAW_PREVIEW_MEGAPIXELS = 40`
(`src/wasm.rs:534`) — a deliberately conservative default, to be tuned on real
hardware post-ship. The maintainer immediately hit it: a real **Leica `.DNG`** (an
85 MB file, ~47 MP embedded preview) fell back to the "convert with the CLI" message
**on a desktop Mac**.

That exposes a conflation the 40 MP number carried: it was a **mobile** memory bound
(iOS Safari's per-tab ceiling), but it's a **single global constant** applied to every
visitor — including desktop, where a ~320 MB preview decode is trivial. So on desktop
the gate blocks for no reason. The maintainer chose to **raise the single global gate**
(accepting that this also lets phone visitors attempt larger previews — an unverified
but gracefully-degrading risk) rather than build platform detection now.

There is a natural ceiling: the engine already enforces a hard **64 Mpix native cap**
(`MAX_IMAGE_PIXELS`, DEC-063 — rejects any preview whose decode would exceed ~1 GiB
peak, on the RAW path too). The demo gate must stay **below** that, or the "convert
with the CLI instead" fallback becomes a lie (the CLI can't decode >64 Mpix either).

## Goal

Raise `MAX_RAW_PREVIEW_MEGAPIXELS` from 40 to **60**, so real Leica-class RAW files
(embedded previews up to 60 MP) convert in the demo, while staying below the 64 Mpix
native cap so the CLI-fallback message stays honest. One constant, plus the boundary
tests and a DEC-082 amendment. No other behavior change.

## Inputs

- **Files to read:**
  - `src/wasm.rs:526-597` — the gate: `MAX_RAW_PREVIEW_MEGAPIXELS` (534),
    `MAX_RAW_PREVIEW_PIXELS` (538), and the `raw_preview` export's pre-decode check
    (582-586).
  - `src/image/mod.rs:54-94` — `MAX_IMAGE_PIXELS` (64 Mpix = 67,108,864 px) +
    `check_pixel_budget`, the native backstop the demo gate sits below.
  - `src/image/raw.rs` — `largest_declared_preview_pixels` (~490) and its boundary
    unit test (SPEC-103 named it `…straddles_a_40mp_boundary`); the test's boundary
    value moves to 60.
  - `tests/wasm_roundtrip.rs` — SPEC-103's gate tests (the "rejects an over-threshold
    preview before decoding it" case, whose bomb declared 50.4 MP — now UNDER the new
    gate, so it must move).
  - `decisions/DEC-082-raw-preview-on-wasm-behind-a-demo-pixel-gate.md` — amend.

## Outputs

- **`src/wasm.rs`:** `MAX_RAW_PREVIEW_MEGAPIXELS: u64 = 40` → `60`. Update the doc
  comment: it's no longer a "framing default" — record that 60 was chosen to clear
  real Leica-class previews (≤60 MP) while staying below the 64 Mpix native cap so the
  fallback stays honest, that it applies globally (desktop + mobile), and that the
  mobile ~400 MB-peak exposure is an accepted, gracefully-degrading tradeoff.
- **`tests/wasm_roundtrip.rs`:** the over-threshold bomb must now declare a size
  **between the new 60 MP gate and the 64 Mpix native cap** (e.g. ~62–63 Mpix, i.e.
  62–63 million px) so it still tests the DEMO gate in isolation — above 64 Mpix and
  the native cap would be catching it, not this gate (the SPEC-103 straddle lesson).
  The "extracts normally under the gate" case stays valid (a real small preview is
  still well under 60 MP).
- **`src/image/raw.rs`:** update the boundary unit test to straddle 60 Mpix (rename
  from `…40mp…` → `…60mp…` if named for the value).
- **DEC-082 amendment:** a dated note that the gate was retuned 40 → 60 after the
  maintainer hit it on a real desktop Leica DNG; that 40 conflated a mobile bound with
  a global one; that 60 sits below the native 64 Mpix cap to keep the CLI fallback
  honest; and that on-device *mobile* verification (does a phone survive a ~60 MP
  preview?) is still the open launch-readiness item.

## Acceptance Criteria

- [ ] `MAX_RAW_PREVIEW_MEGAPIXELS == 60`; `MAX_RAW_PREVIEW_PIXELS == 60_000_000`.
- [ ] A RAW whose largest embedded preview declares **≤ 60 MP** (e.g. a ~47 MP
      Leica-class preview) extracts and converts — no CLI-fallback. Proven by a
      `wasm_roundtrip` case with a preview declared just under 60 MP that returns valid
      PNG bytes (independent-decode the dims).
- [ ] A preview declaring **between 60 and 64 Mpix** (e.g. ~62 Mpix) is rejected with
      `RAW_PREVIEW_TOO_LARGE_MESSAGE`, **before** the full decode (header-peek only),
      and specifically by the DEMO gate — not the native cap. Mutation check: lowering
      the constant back toward 50 flips a ~55 MP case, proving the gate tracks the
      constant and is non-vacuous.
- [ ] The two fallback messages are unchanged and still distinct; no `raw:`/`Tiff is
      not supported` leaks.
- [ ] Native `src/` behavior unchanged (the 64 Mpix `MAX_IMAGE_PIXELS` cap and every
      native decode path are untouched); full native gate suite green.
- [ ] `just wasm-build` brotli size unchanged within noise (a constant edit; no code
      size change expected); `just wasm-test`, `just demo-smoke`, `just wasm-npm-smoke`,
      `just validate` all green.

## Failing Tests

- **`tests/wasm_roundtrip.rs`**
  - `"rawPreview rejects a preview between the demo gate and the native cap"` — a
    `raw_blob` whose only preview declares ~62 Mpix (via `jpeg_declaring`) with tiny
    entropy → `RAW_PREVIEW_TOO_LARGE_MESSAGE`, via the header peek, no full decode.
  - `"rawPreview extracts a ~sub-60 MP preview"` — the existing under-gate extract case
    still passes (a real small preview), confirming 60 didn't break the happy path.
- **`src/image/raw.rs`** — the `largest_declared_preview_pixels` boundary unit test now
  straddles 60 Mpix (a hair under extracts / declared count returned; a hair over is
  what the wasm gate rejects). Cheap, native, exact — the SPEC-103 pattern.

## Implementation Context

### Decisions that apply
- `DEC-082` — the demo RAW pixel gate (this spec amends it with the retune).
- `DEC-063` — the 64 Mpix `MAX_IMAGE_PIXELS` native cap the demo gate must stay below.

### Constraints that apply
- `untrusted-input-hardening` — the gate still rejects on the header peek before any
  large allocation; no panic on any input path; typed errors only.
- `ergonomic-defaults` — the fallback copy stays plain and honest.

### Prior related work
- `SPEC-103` (shipped, PR #111, `fe66a89`) — introduced the gate + the SOF-peek
  pattern + the "bomb must straddle the demo gate and the native cap to test the right
  one" lesson. Reuse its fixtures (`raw_blob`, `jpeg_declaring`).

### Out of scope
- **Platform-aware gating** (desktop-high / mobile-conservative) — the maintainer chose
  the simpler global raise; a per-platform gate stays a possible later refinement, not
  this spec.
- **On-device mobile verification** — still a launch-readiness checklist item.
- Any native cap or native RAW-routing change.

## Notes for the Implementer

- **Why 60, not "just clear it":** the maintainer's file is ~47 MP, but `L…DNG` could
  be a 60 MP M11; 60 clears any realistic Leica body in one shot while staying under
  the 64 Mpix native cap (so the "use the CLI" message stays truthful — above 64 the
  CLI can't do it either). Don't go to/over 64 or the fallback message becomes a lie.
- **Keep the demo-gate test in the [60, 64] Mpix window** — a bomb over 64 Mpix would
  be caught by the native cap, not this gate, silently making the test prove the wrong
  thing (the exact SPEC-103 straddle trap).
- Units: `MAX_RAW_PREVIEW_PIXELS = 60 * 1_000_000 = 60,000,000`; the native cap is
  `64 * 1024 * 1024 = 67,108,864`. The testable window is (60,000,000, 67,108,864].
- rtk footgun: cross-check any grep with raw `grep` + a positive control.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - DEC-082 amended (not a new DEC)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any]

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
