---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-078
  type: story
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (may split if the UX balloons)

project:
  id: PROJ-008
  stage: STAGE-027
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # separate build session
  created_at: 2026-07-13

references:
  decisions:
    - DEC-065    # AVIF encode is IN the wasm build (one artifact) — already compiled into the deployed .wasm
    - DEC-064    # the wasm surface (transform/optimize/info) the worker calls
    - DEC-067    # crustyimg-wasm / --target web / explicit init() — how the worker loads it
  constraints:
    - pure-rust-codecs-default
  related_specs:
    - SPEC-077   # the shipped demo skeleton (main-thread, WebP/PNG, AVIF disabled)
    - SPEC-073   # AVIF encode on wasm (rav1e serial); decode deferred to createImageBitmap

value_link: >
  Completes STAGE-027: makes the demo's headline (drop a PNG → tiny AVIF) real without freezing the
  tab, reads `.avif` inputs, and shows the decision (bytes saved / format / why) — the full "watch it
  just work" experience. Its ship completes STAGE-027.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      note: >
        framed build-ready in the orchestrator main loop; grounded in the shipped SPEC-077 demo +
        the wave's facts: AVIF encode is ALREADY compiled into the deployed .wasm (DEC-065, one
        artifact) so this enables + off-loads it; rav1e is serial on wasm (must go off the main
        thread); the wasm build can't decode AVIF (DEC-073) so `.avif` INPUTS need createImageBitmap.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-078: demo — Web Worker, AVIF, and the explain readout

## Context

Last spec of STAGE-027 (its ship completes the stage). SPEC-077 shipped the demo skeleton — live at
https://jysf.github.io/crustyimg/ — with three things deliberately deferred: **all conversions run
on the main thread** (verify's finding — not just AVIF), **AVIF output is disabled** ("coming —
needs a Web Worker"), and there's **no `.avif` input, no explain readout, and no intent controls**.
This spec closes them.

The load-bearing move is the **Web Worker**: rav1e (AVIF encode) is **serial on wasm** (SPEC-073),
so a synchronous encode on the main thread would freeze the tab. Move the engine into a Worker and
all conversions run off-thread, keeping the UI responsive. AVIF encode is **already compiled into
the deployed `.wasm`** (DEC-065 — one artifact, `avif` ON; SPEC-077 only disabled the UI option), so
this **enables + off-loads** it, it does not rebuild the engine.

Grounded assumptions the build must DRIVE first (name them, don't assume):
- **The `--target web` package inits + runs inside a module Web Worker** (`new Worker(url, {type:
  'module'})`; `init()` resolves the `.wasm` relative to the worker script's URL). Standard, but
  prove it — it's the spec's spine.
- **`.avif` INPUT path** (the wasm can't decode AVIF): `createImageBitmap(avifBlob)` → draw to an
  `OffscreenCanvas` → `convertToBlob({type:'image/png'})` → feed those PNG bytes to the wasm
  `transform`. No new wasm entry needed. Confirm the browser actually decodes AVIF via
  `createImageBitmap`.
- **"Progress" = a responsive busy state, not a % bar.** A single rav1e encode is one blocking call
  in the worker with no native progress signal; the win is the main thread staying responsive (a
  spinner/indeterminate indicator + a cancel/disable), not a real percentage. Scope it honestly.

## Goal

Move the demo's conversions into a Web Worker so they never freeze the tab; enable AVIF output
(PNG/JPEG/… → `.avif`) off-thread; accept `.avif` inputs via `createImageBitmap`; and show the
decision — bytes in→out + saving, format chosen, dimensions — with intent controls (output format +
a quality/byte-budget where the format supports it). Its ship completes STAGE-027.

## Inputs

- **Files to read:** `demo/` (index.html, demo.js, demo.css — the shipped skeleton), the
  `scripts/demo-assemble.mjs` / `serve.mjs` / `demo-smoke` harness (SPEC-077), `pkg/`'s
  `crustyimg.js`/`.d.ts` (the `transform`/`optimize`/`info` surface + `init`).
- **External:** Web Worker (module type), `createImageBitmap`, `OffscreenCanvas`; the browser's
  native AVIF decode.

## Outputs

- **Files modified/created:** a worker module (e.g. `demo/worker.js`) that `init()`s the engine and
  runs `transform`/`optimize`; `demo.js` posts the input bytes + intent to the worker and renders the
  result (bytes/dims/format/saving) + a busy state; the AVIF output option enabled; the `.avif`-input
  `createImageBitmap`→canvas→PNG path; intent controls (format + quality/budget) in `index.html`/CSS;
  the `demo-smoke` extended to drive the worker + AVIF + `.avif` input.
- **No change to:** `src/` / the engine / the WASM surface (AVIF is already compiled in); the
  vendored `.wasm` stays the size-profiled build; native build untouched.

## Acceptance Criteria

- [ ] **All conversions run in a Web Worker** — during a slow AVIF encode the **main thread stays
      responsive** (assert it in the browser: the UI updates / a probe runs while the encode is in
      flight; the page does not freeze). Driven in a real browser, served over HTTP.
- [ ] **AVIF output works** — a PNG/JPEG → `.avif` conversion produces **valid AVIF** (verified by an
      independent decoder — `sips` or `createImageBitmap`), off the main thread. The option is enabled
      (no longer "coming").
- [ ] **`.avif` input works** — dropping an `.avif` decodes it via `createImageBitmap` (→ canvas →
      PNG → engine) and converts; a clear message if the browser can't decode AVIF.
- [ ] **The decision is shown** — input→output **bytes + % saved**, the **format** chosen, and
      **dimensions**; plus intent controls (output format + a quality/byte-budget where supported).
- [ ] **Honest surface** — WebP stays labeled lossless (no lossy-WebP on wasm); no faked progress %.
- [ ] Still **100% client-side** (zero network during conversion), no SharedArrayBuffer / no
      COOP-COEP (a Worker is a separate thread, not shared-memory threads); the deployed `.wasm` is
      the size-profiled one; the live Pages deploy still passes its gate.
- [ ] Native/engine untouched; `just deny`/`just validate` green; guardrail held (a thin demo page).

## Failing Tests

Written now (design). Browser-driven, extending SPEC-077's `demo-smoke`:

- **`demo-smoke` (headless Chrome, served over HTTP), new assertions:**
  - `"worker converts off the main thread"` — kick off a conversion; while it's running, confirm the
    main thread is responsive (a main-thread timer/probe fires during the encode; the page isn't
    blocked). Fails if conversion runs on the main thread.
  - `"png → avif is valid AVIF"` — convert a PNG to AVIF via the worker; pull the bytes out and
    confirm they're valid AVIF (independent decode / `sips`). No main-thread freeze.
  - `"avif input converts"` — feed an `.avif` fixture; assert it decodes (createImageBitmap path) and
    converts to a valid output of the expected dims.
  - `"the readout shows bytes + format + saving"` — after a conversion, the DOM shows input/output
    bytes, % saved, and the chosen format.

## Implementation Context

### Decisions that apply
- `DEC-065` — AVIF encode is already IN the deployed `.wasm` (one artifact); enable + off-load it,
  don't rebuild. rav1e is serial → the Worker is why it's usable.
- `DEC-064` — the worker calls the shipped `transform`/`optimize`/`info`; don't change the surface.
- `DEC-067` — `--target web` + explicit `init()`; the worker `init()`s the same package.

### Constraints that apply
- `pure-rust-codecs-default` — no new codec; `.avif` input uses the *browser's* decoder
  (createImageBitmap), not a bundled one; still no backend.

### Prior related work
- `SPEC-077` — the skeleton (demo/, the `demo-assemble`/`serve`/`smoke` harness, the CORS/`file://`
  lesson, the `waitFor` fix). Reuse its browser-smoke machinery.
- `SPEC-073` — AVIF encode on wasm (serial); the decode-deferred-to-createImageBitmap decision.

### Out of scope (for this spec)
- SharedArrayBuffer / wasm threads / COOP-COEP (a Worker is enough; don't reach for shared-memory
  threads); lossy-WebP on wasm; `npm publish` (SPEC-076); any engine/WASM-surface change; a
  framework/bundler; the maintainer's separate site tool.

## Notes for the Implementer

- **Build + verify in a WORKTREE, drive the real browser** (the SPEC-075/077 lessons). A page that
  looks right but can't `init()` the engine in a Worker, or freezes on an AVIF encode, is the failure
  this spec exists to prevent — prove responsiveness and a valid AVIF in a real browser.
- **The Worker is the spine — prove it first** (module worker + `init()` + one round-trip) before the
  AVIF/input/UX layers, so a wrong assumption there surfaces on day one, not at the end (the SPEC-077
  build-the-harness-first lesson).
- Keep it honest: no fake progress %, WebP labeled lossless, a clear message if `createImageBitmap`
  can't decode AVIF on the user's browser.
- Likely **no new DEC** (uses shipped decisions); add one only if the worker/`.avif`-input approach
  needs a recorded tradeoff. Commit `-s`; per-session `estimated_usd`. If the UX (intent controls +
  explain) balloons, split it from the worker/AVIF core into a follow-up rather than expanding here.

---

## Build Completion

*Filled in at the end of the build cycle.*

- **Branch:**
- **PR:**
- **All acceptance criteria met?**
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection

1. **What was unclear in the spec that slowed you down?** —
2. **Was there a constraint or decision that should have been listed but wasn't?** —
3. **If you did this task again, what would you do differently?** —

---

## Reflection (Ship)

1. **What would I do differently next time?** —
2. **Does any template, constraint, or decision need updating?** —
3. **Is there a follow-up spec I should write now before I forget?** —
