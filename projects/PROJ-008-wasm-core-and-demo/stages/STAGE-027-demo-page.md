---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-027
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-13
shipped_at: 2026-07-13

value_contribution:
  advances: >
    The payoff of the WASM wave: a zero-install, shareable "watch it just work" demo — drop an
    image, watch it become the smallest modern-format artifact in your browser, with the decision
    explained. The highest-ROI adoption artifact the roadmap names, and the thing a Show HN points at.
  delivers:
    - "A static, client-side single-page demo: drop/paste an image → declare an intent → see the optimized modern-format result + format chosen + bytes saved + an explain readout → download — 100% in-browser, no backend"
    - "In-browser conversion over the proven WASM core (SVG + PNG/JPEG/GIF/WebP decode; AVIF/WebP/PNG encode), consuming crustyimg-wasm"
    - "AVIF encode off the main thread (Web Worker + visible progress — rav1e runs serial on wasm), and .avif INPUTS decoded page-side via createImageBitmap (the wasm build can't decode AVIF)"
    - "Static hosting (e.g. GitHub Pages), single-threaded (no SharedArrayBuffer → no COOP/COEP headers a static host can't set)"
  explicitly_does_not:
    - "Add a backend, server, or hosted service (no-service / no-CDN guardrail) — it's a static page"
    - "Become a web app or absorb the maintainer's separate site-builder/content tool (the manifest is the seam; NO HTML-generation product here)"
    - "Publish to npm (SPEC-076) or change the engine/WASM surface (STAGE-025/026 shipped those)"
    - "Require threads/SharedArrayBuffer — stay single-threaded so a plain static host works"
---

# STAGE-027: demo page

## What This Stage Is

The **"watch it just work" artifact.** A static, client-side single-page demo where you drop an
image, declare an intent (a quality target / byte budget / output format), and watch crustyimg —
running entirely in your browser over the shipped WASM core — turn it into the smallest
modern-format artifact, showing the **format it chose, the bytes it saved, and why** (the
`explain`/`info` readout), with a download. No upload, no backend, no install. It consumes
`crustyimg-wasm` (STAGE-026) and is the flagship the Track-B funnel finally points at — the Show HN
moment. When it ships, PROJ-008's public face is done.

## Why Now

- **STAGE-025 + STAGE-026 delivered the substance** — the engine is proven, sized (1.33 MB brotli),
  and packaged. This stage is the payoff: making it *visible and shareable*.
- **The roadmap names the demo the highest-ROI marketing artifact** ("zero-install try it,
  inherently shareable; time the Show HN here"). Adoption is the binding constraint; this is the
  artifact that converts interest.
- **It pairs with the launch** — `npm publish` (SPEC-076) is deliberately held to go out *with* the
  demo (demo live + package published + Show HN), not before it.

## Success Criteria

- A **static, client-side** page (no backend, no network calls for the conversion): drop/paste an
  image → pick an intent → get an optimized modern-format result + **format chosen + bytes saved +
  an explain/info readout** → **download** it. Works offline once loaded.
- **In-browser conversion** over the WASM core: decode SVG + PNG/JPEG/GIF/WebP; encode to WebP/PNG
  and **AVIF** (the "drop a PNG, get a tiny AVIF" headline). AVIF encode runs **in a Web Worker with
  visible progress** (rav1e is serial on wasm — must not freeze the page). `.avif` **inputs** are
  decoded page-side via **`createImageBitmap`** (the wasm build encodes but can't decode AVIF).
- **Honest capability surface** — WebP output is currently *lossless* on wasm (no lossy-WebP
  encoder in the wasm feature set); the demo must not offer a "lossy WebP" it can't produce, or must
  label it honestly. No faked results.
- **Static-hostable** (GitHub Pages or equivalent), **single-threaded** — no SharedArrayBuffer, so
  no COOP/COEP headers the host can't set. First load is reasonable given the 1.33 MB brotli core.
- Honors the guardrails: no service/CDN; no HTML-generation product; the demo is crustyimg's own
  thin marketing page, distinct from the maintainer's separate site/content tool.

## Scope

### In scope
- The demo page (HTML/CSS/JS, or a deliberately light framework) consuming `crustyimg-wasm`; drag-drop
  + intent controls (format / quality-or-budget) + result preview + format/bytes/explain readout +
  download; a Web Worker wrapping AVIF encode with progress; `createImageBitmap` for `.avif` inputs;
  a static build + hosting setup (GitHub Pages workflow); a size/first-load sanity pass.

### Explicitly out of scope
- A backend/service; a general web app; the site-builder/content tool; `npm publish` (SPEC-076);
  engine/WASM-surface changes; threads/SharedArrayBuffer; lossy-WebP encode on wasm (a separate
  future codec question).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-077 (shipped 2026-07-13, PR #85 `9a61787`) — **demo skeleton, single-threaded.** A static,
  no-bundler page that loads `crustyimg-wasm` (`import init … from crustyimg.js; await init()` —
  served over HTTP, NOT `file://`, per the wasm MIME/streaming grounding), drop-an-image →
  `optimize`/`transform` → result + `info` + bytes → download; SVG + PNG/JPEG/GIF/WebP in, **WebP/PNG
  out** (AVIF deferred to 078 — needs the worker); GitHub Pages deploy through `just wasm-build`
  (size-profiled `.wasm`); browser-driven smoke. The end-to-end "it works in a browser" proof.
- [x] SPEC-078 (shipped 2026-07-13, PR #86 `b568f82`) — **Web Worker + AVIF + explain.** Move ALL
  conversions into a module Web Worker (main thread stays responsive — rav1e serial would freeze it);
  ENABLE AVIF output (already compiled into the deployed `.wasm`, DEC-065 — just off-load + un-disable);
  `.avif` INPUTS via `createImageBitmap`→canvas→PNG→wasm; the bytes-in/out + %-saved + format readout;
  intent controls (format + quality/budget). "Progress" = a busy state, not a %. Ship completes STAGE-027.
  May split the UX from the worker/AVIF core if it balloons.

**Count:** 2 shipped / 0 active / 0 pending — **STAGE-027 COMPLETE 2026-07-13** (SPEC-077 skeleton + SPEC-078 Worker/AVIF/explain). The demo is live + full-featured on desktop; **mobile verification is the one carry → STAGE-028 launch-readiness** (real-device test before the Show HN). **✅ DEPLOY PROVEN LIVE 2026-07-13: GitHub Pages enabled; the demo is published at https://jysf.github.io/crustyimg/ — `pages.yml` deploy job green, the page loads, `vendor/crustyimg_bg.wasm` serves as `application/wasm`, and the engine initializes ("Engine loaded", version 0.4.0, no console errors). The end-to-end deploy leg is no longer unproven.**

## Design Notes

- **Consume the package, but dev against local `pkg/`.** The demo imports `crustyimg-wasm`; during
  dev that's the local `just wasm-build` `pkg/` (or `npm link`), so the demo doesn't block on
  SPEC-076's publish. At launch, repoint to the published package.
- **The carries from STAGE-025/026 are load-bearing here** (all recorded): rav1e runs *serial* on
  wasm → AVIF encode MUST be a Web Worker with progress or the page hangs; the wasm build can't
  decode AVIF → `.avif` inputs use `createImageBitmap`; `optimize(_, "webp")` returns *lossless* WebP
  → don't offer a lossy-WebP the engine can't make; the `--target web` package needs `await init()`.
- **Single-threaded on purpose** — SharedArrayBuffer/threads need COOP/COEP headers GitHub Pages
  can't set, so stay single-threaded (rav1e already runs serial). Don't reach for wasm threads.
- **Guardrail:** this is a thin marketing/demo page, not a product or the maintainer's site-builder.
  No routing/templating/CMS. If it grows those, it's out of scope.

## Dependencies

### Depends on
- STAGE-025 (WASM core) + STAGE-026/SPEC-075 (`crustyimg-wasm`, `just wasm-build`/`pkg/`).
- External: a static host (GitHub Pages); the browser `createImageBitmap` + Web Worker APIs.

### Enables
- The Show HN / adoption moment (Track B); pairs with SPEC-076's publish at launch.
- Cutting toward 1.0 (the demo + library are the last WASM-wave deliverables).

## Stage-Level Reflection

*Shipped 2026-07-13.*

- **Did we deliver the outcome in "What This Stage Is"?** **Yes** (desktop) — the crustyimg engine
  runs as a real, live web page (https://jysf.github.io/crustyimg/): drop → convert client-side →
  download, with AVIF both directions off the main thread, `.avif` input via `createImageBitmap`, and
  an honest explain readout. Every claim was *driven* in a real browser (SPEC-077's headless-Chrome
  smoke; SPEC-078's three separate CDP/BiDi/WebDriver clients across Chrome/Firefox/Safari, with a
  frozen-thread negative control). **The one gap: mobile** (iOS Safari / Android Chrome) is
  unverified — undrivable without a device here — carried to STAGE-028 as a launch blocker.
- **How many specs did it actually take?** **2** (SPEC-077 skeleton, SPEC-078 Worker/AVIF/explain),
  as framed. The split held: 077 proved "it runs as a web page" (and caught the `file://`/CORS
  reality); 078 added the headline (AVIF off-thread) + the UX.
- **What changed between starting and shipping?** The deploy went from theoretical to *proven live*
  (Pages enabled mid-stage), and "cross-browser" got sharpened from an afterthought into an explicit,
  driven criterion — which then exposed that only mobile remains.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - **A criterion nobody claims is a criterion nobody checks** — verify must diff the build's
    completion table against the spec's Acceptance list ([[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]).
  - **Don't edit an in-flight spec's own files on `main` while a build/verify branch holds them** —
    it caused SPEC-078's PR to go CONFLICTING. Put cross-cutting additions elsewhere (a separate
    stage/doc) or wait for the merge; fold launch-gating criteria into the spec *before* the build.
  - **Browser claims need real, separate per-engine drivers + a negative control** — a responsiveness
    count is decoration until the same probe reads 0 against a thread you froze on purpose.
    ([[a-green-poll-loop-may-be-waiting-for-nothing]], the failure-mode-unproven lesson.)
  - **Serve, don't `file://`** (SPEC-077): module scripts are CORS-fetched; `file://` is an opaque
    origin and blocks the module before `init()` — the error handler can't fire.
- **Should any spec-level reflections be promoted to stage-level lessons?** The two above
  (criterion-nobody-claims; separate-clients-plus-control) are the most reusable and are already
  their own memories.
