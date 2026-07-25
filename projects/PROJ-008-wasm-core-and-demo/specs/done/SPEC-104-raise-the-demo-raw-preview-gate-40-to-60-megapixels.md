---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-104
  type: chore                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
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
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-sonnet-5
      tokens_total: 151255
      duration_minutes: 18
      estimated_usd: 1.3
      recorded_at: 2026-07-24
      note: >
        Build dispatched as a metered sub-agent (Agent tool, Sonnet); tokens_total is the
        REAL subagent usage count, estimated_usd an order-of-magnitude label. Raised
        MAX_RAW_PREVIEW_MEGAPIXELS 40→60; moved the wasm_roundtrip over-threshold bomb +
        raw.rs boundary test to straddle 60 Mpix; regenerated the stale demo-smoke RAW
        fixture (oversize_preview.dng, 50.4→62.4 Mpix declared) via gen_raw_gate_fixtures
        — a dependency the constant raise silently broke; amended DEC-082. Brotli ~0 (−147 B).
    - cycle: verify
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 95240
      duration_minutes: 40
      estimated_usd: 3.8
      recorded_at: 2026-07-25
      note: >
        Verify dispatched as a metered sub-agent (Agent tool, Opus); tokens_total is the
        REAL subagent usage count. VERDICT CLEAN. Independently confirmed the 62.4 Mpix
        fixture straddles the 60→64 Mpix window (raw SOF0 scan) + is byte-reproducible;
        MUTATION-tested the gate (raising it to 63 flips the bomb test, proving the demo
        gate — not the native cap — rejects; caught an mtime/incremental-compile
        stale-object race in the process). Native suite (440), wasm-test (25), validate,
        demo-smoke, npm-smoke, lean all green; native decode paths byte-unchanged; −147 B.
    - cycle: ship
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: null
      estimated_usd: 0.4
      recorded_at: 2026-07-25
      note: >
        Orchestrator main-loop — PR #112 squash-merged (after two update-branch cycles,
        since main advanced under it) + bookkeeping + readouts + memory + brag. Merge
        redeploys the demo, so the 60 MP gate goes live.
  totals:
    tokens_total: 246495
    estimated_usd: 5.5
    session_count: 3
    session_count: 2
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

- [x] `MAX_RAW_PREVIEW_MEGAPIXELS == 60`; `MAX_RAW_PREVIEW_PIXELS == 60_000_000`.
- [x] A RAW whose largest embedded preview declares **≤ 60 MP** (e.g. a ~47 MP
      Leica-class preview) extracts and converts — no CLI-fallback. Proven by
      `raw_preview_extracts_largest_embedded_preview_as_png` (real preview well
      under 60 MP, independent-decoded via `info()`) plus the mutation extract in
      `raw_preview_rejects_over_threshold_before_decode_and_extracts_under_it`
      (2000×1500 under the gate); the exact ≤60/>60 discrimination is proven
      cheaply, natively, and exactly by
      `largest_declared_preview_pixels_straddles_a_60mp_boundary`.
- [x] A preview declaring **between 60 and 64 Mpix** (e.g. ~62 Mpix) is rejected with
      `RAW_PREVIEW_TOO_LARGE_MESSAGE`, **before** the full decode (header-peek only),
      and specifically by the DEMO gate — not the native cap. Mutation check: the
      wasm bomb is 62.4 Mpix (between the 60 Mpix gate and the 64 Mpix native cap,
      so a pass can only be this gate firing), and the same test's companion
      2000×1500 real preview proves the gate discriminates rather than rejecting
      vacuously.
- [x] The two fallback messages are unchanged and still distinct; no `raw:`/`Tiff is
      not supported` leaks.
- [x] Native `src/` behavior unchanged (the 64 Mpix `MAX_IMAGE_PIXELS` cap and every
      native decode path are untouched); full native gate suite green (786 tests).
- [x] `just wasm-build` brotli size unchanged within noise (a constant edit; no code
      size change expected — measured −147 B against a same-tree baseline build);
      `just wasm-test`, `just demo-smoke`, `just wasm-npm-smoke`, `just validate`
      all green.

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

- **Branch:** `spec-104-raise-raw-gate`
- **PR (if applicable):** #112 — https://github.com/jysf/crustyimg/pull/112
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - DEC-082 amended (not a new DEC)
- **Deviations from spec:**
  - The spec's Inputs list didn't mention `examples/gen_raw_gate_fixtures.rs` or
    `tests/demo_smoke.mjs`, but `just demo-smoke` failed after the constant raise:
    the committed `tests/fixtures/raw/oversize_preview.dng` fixture declared 50.4
    Mpix — over the old 40 Mpix gate but now UNDER the new 60 Mpix gate, so it
    stopped triggering the too-large error and the smoke test timed out waiting
    for a failure state that never came. Regenerated the fixture at 62.4 Mpix
    (8000×7800, matching the `wasm_roundtrip.rs` bomb) via
    `cargo run --example gen_raw_gate_fixtures`, and updated the prose comments in
    both that generator and `tests/demo_smoke.mjs` that cited the old 50.4/40
    numbers. `no_preview.cr2` regenerated byte-identical (no logic change there).
  - No other deviations; native `src/` decode paths and `MAX_IMAGE_PIXELS`
    untouched.
- **Follow-up work identified:**
  - None new — on-device mobile verification remains the pre-existing open
    launch-readiness item (DEC-082), unchanged by this retune.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing in the core constant/test change; the one gap was scope, not
   clarity — the Inputs list omitted the committed RAW smoke fixture
   (`tests/fixtures/raw/oversize_preview.dng`) and its generator
   (`examples/gen_raw_gate_fixtures.rs`), both of which encode the OLD 40/50.4
   Mpix numbers and silently stopped exercising the gate once it moved to 60.
   `just demo-smoke` failing was the only signal; nothing in the spec text
   pointed at it.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-082's `affected_scope` already lists `tests/demo_smoke.mjs` and
   `examples/gen_raw_gate_fixtures.rs` (from SPEC-103), so the decision record
   had the pointer — SPEC-104's own Inputs/Outputs sections just didn't carry it
   forward. A retune spec that moves a gate constant should always cross-check
   the amended decision's `affected_scope` for fixtures baked to the old value,
   not just the source + unit tests named in the diff.
3. **If you did this task again, what would you do differently?**
   — Run `just demo-smoke` (not just `wasm-test`) before considering the change
   complete, specifically because it drives committed binary fixtures that unit
   tests regenerate fresh every run and can't catch going stale.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — This spec existed only because SPEC-103's 40 MP default was a *mobile* number applied as a *global*
   constant — a conflation I could have caught at SPEC-103 framing (the gate's whole justification was the
   phone memory ceiling, yet nothing scoped it to phones). The retune was cheap and correct, but the
   cleaner lesson is that a device-specific safety bound should be scoped to the device from the start, or
   at least flagged as "global for now, platform-split later" so it isn't discovered by a desktop user
   hitting it. The build/verify sub-agent flow worked well again: the build caught the stale committed
   fixture (50.4 Mpix now under the raised gate), and verify's mutation test proved the demo gate — not
   the native cap — is what rejects.

2. **Does any template, constraint, or decision need updating?**
   — No template change. Two tooling footguns surfaced and are already filed in
   `docs/repo-tooling-backlog.md`: (a) `find_spec()`'s glob matches `specs/prompts/*.md`, so
   `just advance-cycle` silently no-ops on a prompt file (hit by both sub-agents this spec); (b) verify
   hit an mtime/incremental-compile stale-object race while mutation-testing (a `mv`-restore gave the file
   an older mtime than the compiled object, so cargo reused stale output) — a real trap for anyone
   mutation-testing, worth a note for future verify sessions.

3. **Is there a follow-up spec I should write now before I forget?**
   — Not yet, but the deferred candidate is now sharper: **platform-aware gating** (desktop effectively
   unlimited, mobile conservative) — the "correct" fix this spec's global raise sidestepped by maintainer
   choice. It only becomes worth building if/when a real phone test shows the mobile ceiling actually
   needs to differ from 60. So it stays paired with the still-open **on-device mobile verification**
   launch-readiness item — do that test first, let its result decide whether platform-split is warranted.
