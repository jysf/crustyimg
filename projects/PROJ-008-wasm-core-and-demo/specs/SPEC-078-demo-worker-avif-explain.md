---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-078
  type: story
  cycle: verify                    # frame | design | build | verify | ship
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
    - cycle: build
      interface: claude-code
      tokens_total: 210000          # order-of-magnitude estimate: main-loop build, no metered subagent
      estimated_usd: 2.60
      note: >
        one worktree session. Worker spine first (it worked on the first run — the --target web
        package inits inside a module Worker unchanged), then AVIF/.avif-input/explain, then the
        smoke. The costly part was not the code: it was proving the responsiveness claim (the CDP
        client had to learn session routing to see the worker's own target, and the probe needed a
        negative control before its counts meant anything).
    - cycle: verify
      interface: claude-code
      tokens_total: 150000          # order-of-magnitude estimate: main-loop verify, no metered subagent
      estimated_usd: 1.85
      note: >
        adversarial pass in its own worktree. Wrote a fresh CDP client (Chrome), a WebDriver BiDi
        client (real Firefox) and a W3C WebDriver client (real Safari via safaridriver) rather than
        reusing the build's smoke — so a bug in one harness could not fake a pass in all three. The
        cost was the cross-browser leg, which the build had not attempted: Safari needed a human to
        tick "Allow remote automation", and proving the size-profile guard actually bites meant
        forging a non-profiled artifact and watching it get refused.
  totals:
    tokens_total: 360000
    estimated_usd: 4.45
    session_count: 2
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

- **Branch:** `feat/spec-078-demo-worker` (worktree `../crustyimg-wt-spec078`)
- **PR:** #86
- **All acceptance criteria met?** Yes, with ONE deviation on the intent controls (below).
  *(Verify's correction: **not quite** — the table below omits the **cross-browser + mobile**
  criterion, and the build drove only headless Chrome. Verify drove Safari and Firefox and recorded
  the matrix; mobile is still unverified. See ## Verify.)*

  | Criterion | Verdict |
  |---|---|
  | All conversions in a Web Worker; main thread responsive during a slow AVIF encode | ✅ driven in headless Chrome: during a real 562 ms PNG→AVIF encode the page ran **28 timer callbacks and 68 animation frames**. And the probe is calibrated: a deliberate 400 ms main-thread block reads **0 / 0**, so the counts are evidence rather than noise. The worker is also structurally visible — it attaches as its own CDP target, and the `.wasm` fetch now happens there. |
  | AVIF output works, off-thread, verified independently | ✅ 800×600 PNG → a 2632 B AVIF, confirmed by **three decoders the crate never met**: an ISOBMFF `ftyp`/`ispe` parse written from the spec, Chrome's own libavif (`createImageBitmap`), and macOS `sips` (skipped, loudly, on Linux CI). The engine *cannot* check this itself — it encodes AVIF and cannot decode it — which is exactly why an outside opinion is the only proof. Option enabled; no longer "coming". |
  | `.avif` input works | ✅ the 16×16 fixture decodes via `createImageBitmap` → `OffscreenCanvas` → PNG → engine → 16×16 WebP. The page reports the input as `avif` (not "png"), says *whose* decoder did it, and a browser that cannot decode AVIF gets a specific message naming the browser as the refuser. |
  | The decision is shown | ✅ bytes in→out, % saved, format chosen **and who chose it**, dimensions, how quality was decided, and where it ran — asserted off the DOM. Intent controls: output format (Auto / AVIF / WebP / JPEG / PNG) + max long edge. |
  | Honest surface | ✅ WebP still labeled lossless; a spinner, never a %; the "bigger, not smaller" case still says why. |
  | 100% client-side, no SAB/COOP-COEP, profiled `.wasm`, live gate | ✅ zero network requests during conversion — and the *worker's* traffic is in that log now, so a conversion phoning home from the worker could not hide. No SharedArrayBuffer, no wasm threads. The assembly guard still gates the deployed `.wasm`. |
  | Native/engine untouched; gates green | ✅ `git diff origin/main -- src/ Cargo.toml Cargo.lock` is **0 bytes**. `just check` (fmt + clippy + build + test), `just deny`, `just validate` all green. No framework, no bundler, no backend. |

- **New decisions emitted:** none. The build used DEC-064/065/067 as shipped; nothing needed a recorded tradeoff that those don't already carry.
- **Deviations from spec:**
  - **No quality/byte-budget slider.** The acceptance criteria ask for "a quality/byte-budget where the format supports it", but the shipped wasm surface (DEC-064) takes **no quality argument** — `transform` encodes at the format default and `optimize` decides quality itself. A slider would therefore control nothing, and adding one to the surface is explicitly out of scope for this spec. So the page shows how quality *was* decided instead (JPEG: searched with SSIMULACRA2; AVIF: encoder default, **not** searched, because a perceptual search must decode each candidate and this build cannot decode AVIF; WebP/PNG: lossless), and I added **Auto** — real intent, wired to the engine's own analysis + format shortlist — as the control that does exist. See the follow-up below.
  - **The `.avif` decode happens in the worker, not the page.** `createImageBitmap`/`OffscreenCanvas` are available to workers, so keeping it there means the page's thread never touches image data at all.
- **Follow-up work identified:**
  1. **A quality / byte-budget argument on the wasm surface** (`optimize(bytes, format, { target, maxBytes })`) — the missing half of "intent". It is an engine-surface change, so it needs its own spec; the demo can wire a slider to it in an afternoon once it exists. The engine already has the byte-budget search (the only one AVIF can use, since the perceptual one needs a decoder).
  2. **AVIF encode is slow enough to want cancellation.** A superseded job is currently dropped on the floor — the page ignores its result, but the worker still finishes it. A second worker, or an abort flag checked between engine calls, would stop a stale 5-second encode from hogging the thread.

### Build-phase reflection

1. **What was unclear in the spec that slowed you down?** — Nothing about the worker or AVIF: the spec's grounded assumptions were all correct and stated precisely enough to build from. The one gap was the **quality/byte-budget control**, which the acceptance criteria ask for and the shipped wasm surface cannot express. That is a design-time contradiction the spec could have caught by reading `src/wasm.rs`'s signatures (`optimize(input, out_format)` — no third argument).
2. **Was there a constraint or decision that should have been listed but wasn't?** — DEC-064 is listed, but only as "the worker calls the shipped surface, don't change it". What actually mattered is a *consequence* of it: the surface exposes no quality knob, so "intent" in the browser is limited to format + size. Worth stating in the spec rather than discovering in the build.
3. **If you did this task again, what would you do differently?** — I'd write the **negative control first**. The responsiveness probe is the whole spec's claim, and for an hour it was just a number I believed: 28 timer ticks *looks* like proof, but it only becomes proof once you've shown the same probe reads 0 against a thread you froze deliberately. Same lesson as the wave's green-poll-loop bug, from the other side — this time I checked before shipping the belief, but only because the spec's own history told me to.

---

## Verify

*2026-07-13, worktree `crustyimg-wt-verify078`, adversarial. Verdict: **the engineering is CLEAN —
every claim the build made reproduces. One acceptance criterion (cross-browser + mobile) was NOT met
by the build; verify met the desktop half of it and leaves mobile as an open launch item.***

**Method.** The build's own smoke was not trusted as the proof of the build. Three fresh clients were
written for this pass — CDP (Chrome), WebDriver BiDi (real Firefox), W3C WebDriver (real Safari via
`safaridriver`) — driving the real page over HTTP with a real 1600×1200 noisy photo (5.25 MB; a
worst-case for rav1e, giving a ~3.1 s encode). A bug in one harness cannot fake a pass in all three.

| Criterion | Verdict |
|---|---|
| Conversions in a Worker; main thread alive during a slow AVIF encode | ✅ **reproduces in all three engines.** Chrome **311** timer callbacks / **295** animation frames; Firefox **274**/**392**; Safari **260**/**187** — all during a real ~3.1 s PNG→AVIF encode. **The control holds:** the same probe against a deliberately frozen thread reads **0 / 0** in every engine, so the counts are evidence, not noise. The worker is structurally real too — it attaches as its own CDP target and the `.wasm` fetch happens *there*, not on the page. |
| AVIF output valid, off-thread, judged independently | ✅ 1600×1200 PNG → 509,060 B AVIF, agreed by **three decoders the crate never met**: macOS `sips` (`format: avif, 1600×1200`), Chrome's libavif via `createImageBitmap` in a *blank* page, and an ISOBMFF walk written here from the spec (major brand `avif`, `ispe` 1600×1200). And the asymmetry that makes an outside opinion *necessary* was confirmed by driving it: the engine's own `info()` **refuses** its own AVIF output. |
| `.avif` input | ✅ in all three engines: 286 B AVIF → 160 B WebP at 16×16; the page reports the input as `avif` (not "png") and names *whose* decoder did it. |
| The decision is shown | ✅ bytes in→out, % saved (**recomputed independently — the DOM's 69% is arithmetic, not decoration**), format, dimensions, how quality was decided, where it ran. |
| Honest surface (the build's deviation) | ✅ **and the deviation is the honest call.** There are **zero** range inputs and nothing labelled "quality" on the page — no dead slider pretending to steer an engine that takes no quality argument (DEC-064). Instead the page explains how quality *was* decided per format. **"Auto" is real, not a stub:** given a noisy photo it chose **jpeg**, not the headline avif — that is the engine's own analysis talking. The surface follow-up (a quality/byte-budget arg on the wasm surface) **is** filed under *Follow-up work identified*, item 1. |
| 100% client-side | ✅ **zero** network requests during conversion from the page **or the worker** — the worker's own CDP target is in the log, so a conversion phoning home from the worker could not hide. |
| Native/engine untouched; profiled `.wasm`; gates | ✅ `git diff origin/main -- src/ Cargo.toml Cargo.lock` = **0 bytes**. `just check` / `just deny` / `just validate` green; `just demo-smoke` green. The vendored `.wasm` is gitignored and rebuilt in CI, so the "profiled" claim rests entirely on the assembly guard — **so the guard was attacked**: a forged artifact carrying a fat 60 KB `name` section was refused, exit 1. The guard bites; it is not decoration. Local rebuild reproduces 1.33 MB brotli (SPEC-074's baseline), `name` section 42 B. |
| **Cross-browser + mobile** | ⚠️ **PARTIALLY MET — this criterion was not met by the build.** Desktop is now **driven and green**: **Chrome 150** (17/17), **Firefox 150** real Gecko (9/9), **Safari 26.5** real WebKit (8/8) — each confirmed for module Worker, `instantiateStreaming`, *and* `createImageBitmap` decoding AVIF (the risk the spec flagged: all three do decode it). Graceful degradation was **driven, not assumed** — an AVIF the browser's decoder refuses produces a clear message naming the browser as the refuser, in both Firefox and Safari; no hang, no stack. **Mobile (iOS Safari, Android Chrome) is UNVERIFIED** — undrivable on this machine (no iOS simulator, no Android SDK). Chrome device emulation proves *layout* only (page fits 390×844 and 412×915, no horizontal scroll, controls reachable) and is **not** evidence about a mobile engine. The matrix — including what was *not* driven — is recorded in `demo/README.md`, as the criterion asks. |

**Punch list (none block the code; all block the ship):**

1. **Mobile is still a launch blocker.** iOS Safari + Android Chrome remain undriven. Keep the
   cross-browser box in `docs/launch-readiness.md` **open**, narrowed to mobile-only, and clear it by
   loading https://jysf.github.io/crustyimg/ on a real phone after this merges. (Desktop's half of
   that box can be ticked.)
2. **The branch is 2 commits behind `main`** (the launch-readiness + STAGE-028 docs). Main requires
   up-to-date branches → `gh pr update-branch` before merge, not `--admin`.
3. **The build's completion table omitted a criterion it had not met** while answering "all criteria
   met? Yes". Not a code defect, but the reason cross-browser nearly shipped unverified.

**Verify-phase reflection.** The build's negative control is what made this pass fast and its own
claim believable — it reproduced exactly, in three engines. The gap was elsewhere, and it was a
*bookkeeping* gap that behaved like an engineering one: a criterion that no row in the completion
table denied, because no row mentioned it. A criterion nobody claims is a criterion nobody checks.

---

## Reflection (Ship)

1. **What would I do differently next time?** —
2. **Does any template, constraint, or decision need updating?** —
3. **Is there a follow-up spec I should write now before I forget?** —
