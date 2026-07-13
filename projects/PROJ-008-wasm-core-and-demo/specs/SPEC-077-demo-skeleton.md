---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-077
  type: story
  cycle: build                     # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L

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
    - DEC-064    # the wasm-bindgen surface (info/transform/optimize/version) the page calls
    - DEC-066    # the size profile lives in `just wasm-build` — the deployed .wasm must be THAT
    - DEC-067    # crustyimg-wasm, --target web, explicit init() — how the page loads it
  constraints:
    - pure-rust-codecs-default
  related_specs:
    - SPEC-072   # the wasm surface + `just wasm-build`
    - SPEC-075   # the crustyimg-wasm package the demo consumes

value_link: >
  The first end-to-end proof that crustyimg runs as a real in-browser web page — drop an image,
  get an optimized result, download it — the skeleton STAGE-027's "watch it just work" demo is built on.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      note: >
        framed build-ready in the orchestrator main loop; grounded in a design-time read of the
        crustyimg-wasm package surface (2026-07-13): exports `.`→crustyimg.js + a
        `./crustyimg_bg.wasm` subpath; the default `init()` resolves the wasm as
        `new URL('crustyimg_bg.wasm', import.meta.url)` + `instantiateStreaming` — so the page
        MUST be served over HTTP (correct `application/wasm` MIME), not opened as `file://`.
    - cycle: build
      interface: claude-code
      tokens_total: 145000
      duration_minutes: 55
      estimated_usd: 1.35
      note: >
        ran in the build session's main loop, not a metered subagent — tokens_total is an
        order-of-magnitude ESTIMATE (~80/20 in/out at Opus 4.8 list rates, no cache discount),
        not a harness-reported number. Includes the wasm rebuilds and ~6 headless-Chrome smoke
        runs.
  totals:
    tokens_total: 145000
    estimated_usd: 1.35
    session_count: 1
---

# SPEC-077: demo skeleton (in-browser, single-threaded)

## Context

First spec of STAGE-027 (the demo page). STAGE-026 shipped `crustyimg-wasm` — proven to install
and run in Node. This spec proves the other half: **the package runs as a real web page in a
browser**, and builds the minimal end-to-end skeleton the full demo (SPEC-078) polishes. It's
deliberately the skeleton: drop an image → convert → see result + bytes → download, over the fast
synchronous formats. AVIF-encode-in-a-Web-Worker, `.avif`-input via `createImageBitmap`, the
`explain` readout, and intent controls are **SPEC-078**.

A **design-time read of the package (2026-07-13)** pinned how the page loads it:
- `import init, { info, transform, optimize } from '<vendored>/crustyimg.js'; await init();`
- With no argument, `init` resolves the wasm as `new URL('crustyimg_bg.wasm', import.meta.url)` and
  `WebAssembly.instantiateStreaming` — which needs the server to send `application/wasm` MIME (there
  is a slower `instantiate` fallback). **So the page must be SERVED over HTTP, not opened as
  `file://`** (streaming/fetch of `file://` fails). GitHub Pages serves correct MIME; local dev
  needs a static server.
- The only browser-unproven assumption in the wave: everything so far ran in Node (`initSync`
  with bytes). This spec drives the **browser** `init()`/fetch path for real.

## Goal

A static, client-side demo page — served over HTTP — that loads `crustyimg-wasm`, lets you drop an
image, converts it in-browser to a smaller/modern-format result (fast synchronous formats), shows
input→output bytes + dimensions + format, and downloads it. No backend, no bundler, single-threaded.
Deployable to GitHub Pages, with the **size-profiled** `.wasm`.

## Inputs

- **Files to read:**
  - `pkg/` (from `just wasm-build`) — `crustyimg.js` (the `init`/`instantiateStreaming` path),
    `crustyimg.d.ts` (`info`/`transform`/`optimize`/`version`/`ImageInfo`), `package.json` exports.
  - `npm/package.overrides.json` — the package identity/exports (DEC-067).
  - `justfile` — `wasm-build`/`wasm-npm-pkg`; add the demo assemble/serve recipe here.
  - `src/wasm.rs` — the API being called (don't change it); `src/recipe/` for the recipe TOML shape
    if `transform` (vs `optimize`) is used.
- **External:** a static server for local dev; GitHub Pages for hosting; browser `fetch`/ESM/
  `WebAssembly.instantiateStreaming`.

## Outputs

- **Files created:**
  - A demo directory (e.g. `demo/` or `web/`): `index.html` + a module script + minimal CSS —
    drag-drop / file-picker → convert → before/after (bytes + dims + format) → download.
  - A **`just` recipe** to assemble the demo: build the wasm **through `just wasm-build`**
    (size-profiled, DEC-066), vendor the `pkg/` files into the demo (a copy — no bundler), and serve
    it locally over HTTP (correct `application/wasm` MIME).
  - A **GitHub Pages** deploy path (a workflow, or a committed `docs/`/`gh-pages` setup) that runs
    `just wasm-build` + assembles + publishes — deploying the profiled `.wasm`.
  - A browser-driven smoke (headless or the in-repo Browser tooling) that loads the served page,
    drops a fixture image, and asserts the converted result's dimensions/bytes.
- **No change to:** `src/` / the engine / the WASM surface; the native build; `crustyimg-wasm`'s
  package shape (consume it as-is).

## Acceptance Criteria

- [ ] The demo page, **served over HTTP**, loads `crustyimg-wasm`, `await init()` succeeds, and a
      dropped image is converted **in-browser** (no network call for the conversion) to a
      smaller/modern-format result, shown with input→output **bytes + dimensions + format**, and
      **downloadable**. Driven in a real browser (headless or the Browser tooling), not asserted from
      Node.
- [ ] Inputs: **SVG + PNG/JPEG/GIF/WebP** decode; outputs: **WebP + PNG** (the fast synchronous
      formats). AVIF output is **SPEC-078** (needs the Web Worker) — the skeleton may omit it or
      clearly mark it "coming"; it must NOT run AVIF encode on the main thread (it would freeze).
- [ ] **100% client-side**: no backend, no conversion network calls (only the page + wasm assets
      load). Single-threaded (no SharedArrayBuffer → no COOP/COEP headers needed).
- [ ] The deployed/served `.wasm` is the **size-profiled** one (~1.33 MB brotli, via
      `just wasm-build`) — not a bare `cargo build` (DEC-066).
- [ ] `file://` limitation documented (must be served); GitHub Pages deploy path exists and works.
- [ ] Native build / engine untouched; `just deny`/`just validate` green; guardrail held (a thin
      demo page — no backend, no routing/templating/CMS).

## Failing Tests

Written now (design). A browser demo is driven, not unit-tested — the "tests" are a driven smoke +
an assembly guard:

- **Browser smoke** (headless browser or the in-repo Browser tooling), via a `just demo-smoke` (or
  folded into `demo-serve`): serve the assembled demo → load it → confirm `init()` resolved (no
  console error, `version()` returns the crate version) → drop/inject a PNG fixture → convert to
  WebP → assert the result is non-empty, decodes to the expected dims (reuse the SPEC-075 JS IHDR/
  info approach), and a download is produced. Fails if the browser can't instantiate the wasm or the
  conversion doesn't run client-side.
- **Assembly guard:** assert the vendored `.wasm` in the demo is the size-profiled one (reuse
  SPEC-075's structural `strip`-fingerprint check, not a size band) — so a demo deploy can't ship a
  bare-build `.wasm`.

## Implementation Context

### Decisions that apply
- `DEC-067` — `crustyimg-wasm`, `--target web`, explicit `init()`. The page imports `crustyimg.js`
  and `await init()`s before calling the API. Serve over HTTP (MIME) — `file://` won't stream.
- `DEC-066` — the deployed `.wasm` MUST be the size-profiled `just wasm-build` output (the demo
  assemble recipe depends on it, same discipline as `just wasm-npm-pkg`).
- `DEC-064` — the API surface: `info(bytes) -> ImageInfo{width,height,format,hasAlpha}`,
  `transform(bytes, recipe_toml, out_format) -> bytes`, `optimize(bytes, out_format) -> bytes`,
  `version()`. (`optimize` on wasm is honest-but-partial — first-candidate, no perceptual search.)

### Constraints that apply
- `pure-rust-codecs-default` — the page runs pure-Rust→wasm; no backend, no service.

### Prior related work
- `SPEC-075` — the package + its Node smoke + the structural size guard (reuse the fingerprint check
  and the IHDR/info dimension-assert approach for the browser smoke).

### Out of scope (for this spec)
- **AVIF encode + the Web Worker + visible progress; `.avif` inputs via `createImageBitmap`; the
  `explain` readout; intent controls (quality/budget)** — all **SPEC-078**.
- `npm publish` (SPEC-076); a bundler/framework; threads/SharedArrayBuffer; lossy-WebP on wasm; any
  engine/WASM-surface change; the maintainer's separate site/content tool.

## Notes for the Implementer

- **Serve, don't `file://`.** The single biggest gotcha: `init()`'s `instantiateStreaming` needs
  `application/wasm` MIME over HTTP. Local dev + the smoke must serve the demo (a tiny static server
  in the recipe); GitHub Pages is fine. Document it in the demo README/recipe.
- **No bundler.** Vendor `pkg/`'s files next to `index.html` and `import` `crustyimg.js` directly
  (`<script type="module">`); `init()` finds `crustyimg_bg.wasm` by relative URL. Keeps the demo a
  plain static site — no build toolchain beyond `just wasm-build` + a copy.
- **Keep AVIF off the main thread.** The skeleton uses the fast synchronous formats (WebP/PNG). Do
  NOT call `transform(_, _, "avif")` synchronously in the page — rav1e is serial and would freeze it;
  that's SPEC-078's Web Worker.
- **Reuse SPEC-075's proofs:** the structural `strip`-fingerprint size guard, and the JS IHDR/`info`
  dimension assertion, both transfer to the browser smoke.
- Likely **no new DEC** (uses shipped decisions); add one only if the hosting approach needs a
  recorded tradeoff. Commit with `-s`. **Build/verify in a WORKTREE, not the shared checkout** (the
  SPEC-075 collision lesson). Drive the real browser — a page that "looks right" but can't
  `instantiateStreaming` is the failure mode.

---

## Build Completion

- **Branch:** `feat/spec-077-demo-skeleton` (built in a worktree, not the shared checkout)
- **PR:** #85
- **All acceptance criteria met?** Yes — all six, each proven by `just demo-smoke` driving real
  headless Chrome over HTTP (22 checks green), not by inspection:
  - `init()` resolves via `instantiateStreaming`; a PNG dropped into the real `<input type=file>`
    (CDP `DOM.setFileInputFiles` — the user path, not a test hook) converts in-browser; bytes +
    dims + format shown for input and output; a `blob:` download is produced. The output's bytes
    are pulled back out of the browser and decoded by parsers we wrote ourselves (VP8L header for
    WebP, IHDR for PNG), so the engine never grades its own homework.
  - Input reach driven through the page: **SVG** (the repo's hand-written 40×30 fixture →
    rasterized, reported `png`), **JPEG, GIF, WebP, PNG**. Output: **WebP + PNG**. AVIF is a
    `disabled` option labelled "coming (needs a Web Worker)" — never called.
  - 100% client-side, **measured, not asserted**: the CDP network log shows ZERO requests during
    the conversion, and nothing off-origin across the whole page load.
  - The vendored `.wasm` is the size-profiled one — the assembler refuses any other (structural
    `strip` fingerprint: 42 B `name` section vs ~980 KB, reused from SPEC-075).
  - `file://` limitation documented **and tested** (see below). Pages deploy path exists
    (`.github/workflows/pages.yml`), gated on the browser smoke.
  - `src/` diff is empty; `just deny`, `just validate`, `just wasm-npm-smoke` green.
- **New decisions emitted:** none — DEC-064/066/067 covered it, as the spec predicted.
- **Deviations from spec:**
  - **The `file://` failure mode is not what the spec said, and the spec's version is the
    dangerous one.** The design (and my first draft of the page) said `instantiateStreaming`
    fails on the MIME type. Measured in headless Chrome: it never gets that far. `demo.js` is an
    **ES module**, module scripts are fetched under CORS, and a `file://` origin is opaque — so
    the browser blocks the module before executing a line of it. No import, no `init()`, no
    `catch`, no console error: the page sits on "Loading the engine…" **forever**. The MIME
    problem is real and would bite second. Fixed by a classic (non-module) script in `index.html`
    that detects `file:` and says so — a classic script is not CORS-fetched, which is exactly why
    it survives to explain the failure. The smoke now asserts both halves (can't run, and says
    why), so the claim is tested rather than repeated.
  - **Added a CI gate the spec didn't ask for.** `pages.yml` runs `just demo-smoke` on every PR
    and blocks the deploy on it. This is the first CI job in the repo that builds through
    `just wasm-build` — it closes part of the standing "CI never runs the wasm smokes" carry
    (`wasm-npm-smoke` is still Mac-only).
  - **Refactored two SPEC-075 helpers into `scripts/lib/`** (`wasm-artifact.mjs`, `png.mjs`)
    instead of copy-pasting the fingerprint check and PNG fixture into a second smoke. There is
    now one definition of "this .wasm came through `just wasm-build`" in the repo. `npm_smoke.mjs`
    imports them and still passes unchanged.
  - **No browser driver.** Chrome is driven over the DevTools Protocol directly (~80 lines of
    WebSocket JSON-RPC). Adding Puppeteer/Playwright to prove a page that needs no toolchain would
    have undercut the pitch it exists to make.
- **Follow-up work identified:**
  - **SPEC-078 (already planned):** AVIF encode in a Web Worker; `.avif` inputs via
    `createImageBitmap`; `explain` readout; intent controls. The page's `avif` option is wired and
    disabled, waiting for it.
  - The conversion runs on the **main thread**, so a very large PNG will jank the tab even for
    WebP/PNG. The Worker SPEC-078 adds should take *all* conversions, not just AVIF.
  - **Lossless WebP can grow an already-lossy JPEG.** The page is honest about it (it says "N%
    bigger" and why), but the real fix is AVIF (SPEC-078) or lossy WebP — which is a C library,
    and so blocked on `pure-rust-codecs-default`. Worth a line on the license watchlist.
  - GitHub Pages must be enabled for the repo (Settings → Pages → Source: GitHub Actions) before
    the deploy job can publish; the workflow is correct but has never run.

### Build-phase reflection

1. **What was unclear in the spec that slowed you down?** — Nothing was unclear; one thing was
   *wrong*, and being handed it as settled fact is what made it costly. The spec stated the
   `file://` failure mode confidently ("instantiateStreaming fails"), so I wrote that into the
   page's error handler and into a doc comment before testing it — and the handler I wrote can
   never fire, because the module it lives in never loads. A design-time claim about *how*
   something fails is exactly as unproven as a claim about how it works, and the wave has now been
   bitten by this twice (SPEC-074's "wasm-opt fails silently at exit 0" was also false).
2. **Was there a constraint or decision that should have been listed but wasn't?** — Not a
   decision, but a *behaviour*: nothing warned that `optimize(bytes, "auto")` can shortlist AVIF,
   which on the main thread would freeze the tab — the one thing the spec explicitly forbade. The
   guardrail ("keep AVIF off the main thread") and the API's auto-format behaviour live in
   different documents, and the collision is only visible if you read `src/wasm.rs`. The page
   therefore always passes an explicit format. Worth stating in DEC-064's surface notes.
3. **If you did this task again, what would you do differently?** — Drive the browser *first*.
   The smoke was the last thing I wrote and it immediately found two real bugs (the CORS module
   block, and a `var status` at global scope silently aliasing `window.status` — a legacy
   string-coercing DOM property — so the error message assignment was a no-op while `dataset.state`
   worked). Both are invisible to reading the code and instant to a real browser. For a spec whose
   entire thesis is "we have never run this in a browser", the browser is the first tool to reach
   for, not the verification at the end.

---

## Reflection (Ship)

1. **What would I do differently next time?** —
2. **Does any template, constraint, or decision need updating?** —
3. **Is there a follow-up spec I should write now before I forget?** —
