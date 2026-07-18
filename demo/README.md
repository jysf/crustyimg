# The crustyimg demo

A static page that runs the crustyimg engine **in your browser**. Drop a photo and it becomes
**web-ready** in one click — the `web` flow (SPEC-085): downscale the long edge to 2048, modernize
to AVIF (photos) or lossless WebP (graphics), never hand back a bigger file, and score the result.
Nothing is uploaded, and there is no backend to upload it to. The engine is the same pure-Rust code
the CLI runs, compiled to WebAssembly and shipped as [`crustyimg-wasm`](../npm/README.md).

The demo's job is not to convert one image — it is to **turn a visitor into a user**. Every result
shows the exact `crustyimg web <file>` command (with a copy button and the recipe verbatim), because
the CLI runs this same flow on whole folders at once. All the old knobs (format override, a size cap,
a byte budget) live behind a collapsed **Advanced** disclosure; the hero never needs them.

```
just demo-serve      # build it, serve it, print a URL
just demo-smoke      # prove it: drive the served page in a real headless browser
```

## ⚠ It must be SERVED. You cannot open `index.html` from your filesystem.

Double-clicking `demo/index.html` gives you a `file://` URL, and the page will not work — the
browser blocks it twice over:

1. **`demo.js` never runs.** It is an ES module, and module scripts are fetched under CORS. A
   `file://` origin is opaque, so the browser refuses the module before executing a line of it —
   no import, no `init()`, no error in the console.
2. **The WebAssembly could not stream anyway.** `init()` fetches `crustyimg_bg.wasm` and hands it
   to `WebAssembly.instantiateStreaming`, which requires the server to send it as
   `application/wasm`. A `file://` fetch has no MIME type to send.

The page detects this and tells you so, rather than sitting on "Loading the engine…" forever. Any
static HTTP server fixes it; `just demo-serve` runs one (it is 60 lines of `node:http` — see
`scripts/serve.mjs` — and the load-bearing line in it is the `.wasm → application/wasm` entry).
GitHub Pages serves the right MIME type too, which is why the deploy Just Works.

No COOP/COEP headers are needed. The conversions run in a **Web Worker** — a separate thread, not
shared memory — so there is no `SharedArrayBuffer` and no wasm-threads build to unlock. (Which is
fortunate: GitHub Pages cannot set headers.)

## What it does

| | |
|---|---|
| **In** | PNG, JPEG, GIF, WebP, SVG (rasterized by resvg, `<text>` and all), AVIF |
| **Default (the hero)** | the `web` flow: auto-orient → downscale long edge to 2048 → modernize (Auto: AVIF for photos, lossless WebP for graphics) → **never bigger than the original** → score |
| **Advanced** | a collapsed `<details>`: format override, a max-edge (incl. "keep full resolution"), a byte budget — none of it needed for the one-click path |
| **Downscale** | runs a real recipe — the same `resize` TOML + `fast_image_resize` backend the CLI's `web` runs, not a browser resampler |
| **Funnel** | every result shows `crustyimg web <file>` + a copy button + the recipe verbatim from `recipes/web.toml`, because the CLI does this to whole folders (`crustyimg web *.jpg`) |
| **Explains** | bytes in → out and % saved (or "kept your original"), the downscale stated honestly, and the raw SSIMULACRA2 score where the engine can measure it |
| **Runs** | entirely on your machine, in a Worker: no server, no upload, no network call for the conversion |

The in-browser result **approximates** the CLI — the wasm build encodes AVIF at q80, the `web` CLI at
q85 (DEC-069) — so it is close, not byte-identical, and the page says so.

### Why a Worker (SPEC-078)

`rav1e` — the AVIF encoder in this `.wasm` — is **serial and slow**, and a wasm call is synchronous.
Called from the page's thread it would freeze the tab for the whole encode, which is why SPEC-077
shipped with AVIF disabled. `demo/worker.js` runs the engine on its own thread instead, so the page
keeps painting while it works. Every conversion goes through it, not just AVIF.

The busy indicator is a **spinner, not a percentage**: one conversion is a single blocking call
inside the worker and rav1e reports nothing as it goes, so a progress bar would be a lie about
information we do not have. What is true — and what the browser smoke asserts — is that the main
thread keeps running throughout.

### AVIF goes in one direction only

This build **encodes** AVIF and cannot **decode** it (DEC-065 — the decoder is a different codec that
does not build for bare wasm32). So:

* an `.avif` **input** is decoded by *your browser* (`createImageBitmap` → an `OffscreenCanvas` →
  PNG bytes → the engine), and the page says so rather than implying the engine did it;
* an AVIF **output**'s dimensions are read back by your browser's decoder too — the engine cannot
  check its own work here, so an independent decoder does.

Neither borrows a codec into the wasm; both borrow the one already in the browser.

## Browsers

The demo leans on three things browser engines have historically disagreed about: **module Web
Workers** (`new Worker(url, {type:'module'})`), **`WebAssembly.instantiateStreaming`**, and
**`createImageBitmap` decoding AVIF** — the last one is what the `.avif`-input path rides on. So they
are not assumed; each engine below was **driven** (SPEC-078 verify, 2026-07-13): the real page, over
HTTP, converting a real 1600×1200 photo to AVIF, with the main-thread probe and its negative control.

| Engine | Module Worker + `instantiateStreaming` | `createImageBitmap` decodes AVIF | Main thread stays alive during a ~3 s AVIF encode |
|---|---|---|---|
| **Chrome 150** (macOS, headed + headless) | ✅ | ✅ | ✅ 311 timers / 295 frames (control: 0 / 0) |
| **Firefox 150** (macOS, real Gecko) | ✅ | ✅ | ✅ 274 timers / 392 frames (control: 0 / 0) |
| **Safari 26.5** (macOS, real WebKit) | ✅ | ✅ | ✅ 260 timers / 187 frames (control: 0 / 0) |
| **iOS Safari** | **not driven** — see below | **not driven** | **not driven** |
| **Android Chrome** | **not driven** | **not driven** | **not driven** |

The counts are only meaningful because of the **control**: the same probe, against a deliberately
frozen thread, reads 0 / 0. All three desktop engines also produced an AVIF that macOS `sips` — a
decoder from neither this crate nor the browser — read back as a 1600×1200 AVIF.

**Mobile is NOT verified.** It could not be driven on the verifying machine (no iOS simulator, no
Android SDK), and desktop-Chrome device emulation proves layout, not engine capability — so it is not
evidence about iOS Safari or Android Chrome. What *was* checked: the page lays out at 390×844 and
412×915 with no horizontal scroll and the controls reachable. **Driving a real phone remains an open
launch-checklist item** (`docs/launch-readiness.md`). The engine features are well-supported on both
(module workers: iOS 15+, Chrome 80+; AVIF decode: iOS 16+, Android Chrome 85+), and a browser that
*can't* decode AVIF gets a clear message rather than a hang — but "well-supported" is a claim from a
support table, not a drive, and this file does not pretend otherwise.

### Quality, honestly

There is no quality **slider**, because the shipped wasm surface takes no quality argument (DEC-064)
— a slider would control nothing. The engine decides, and the readout says which of these happened:

* **JPEG** — quality *searched* with SSIMULACRA2 until the result is visually lossless (the engine's
  real auto-quality: it decodes every candidate to score it);
* **AVIF** — the encoder's *default* quality, **not** searched: a perceptual search must decode each
  candidate, and this build cannot decode AVIF;
* **WebP / PNG** — *lossless*, so there is no quality to choose. Lossy WebP is a C library and this
  build is pure Rust, which means re-encoding an already-lossy JPEG to WebP can legitimately produce
  a **bigger** file. That is exactly when the demo's **never-bigger guard** hands your original back
  ("kept your file") instead of shipping the larger result — pure page logic on top of `web`, which
  itself reports size honestly and can go larger (use the CLI's `optimize` for an unconditional
  never-bigger that keeps dimensions).

The **score** shown is raw SSIMULACRA2 (~100 is visually identical; it can go **negative** on a bad
encode) — never a 0–100 percentage. The engine only produces it when it ran a perceptual search
(JPEG here); AVIF cannot be scored in-browser (encode-only — no decoder to measure against), and
lossless output has nothing to score. The page says which of those happened rather than showing a
blank.

## How it is built

There is no bundler, no framework, and no `npm install`. The whole build is:

```
just demo-build
  ├── just wasm-build          # the size-profiled .wasm (DEC-066) → pkg/
  └── scripts/demo-assemble.mjs
        ├── refuse any .wasm that did not come through that recipe  ← the assembly guard
        └── copy pkg/crustyimg.js + crustyimg_bg.wasm → demo/vendor/
```

`demo/vendor/` is a build output — regenerated every time, never committed. The guard is
structural, not a size band: a size-profiled `.wasm` is stripped, so its `name` debug section is
~42 B where an unprofiled one carries ~980 KB. That difference is categorical and does not decay as
the engine grows. (Same check the npm package uses — `scripts/lib/wasm-artifact.mjs`.)

`.github/workflows/pages.yml` runs exactly that, gates it on `just demo-smoke` in a real browser,
and publishes `demo/` to GitHub Pages.

## Files

| File | |
|---|---|
| `index.html` | the page: the one-click hero, the Advanced `<details>`, the funnel (and the classic script that explains the `file://` failure) |
| `demo.js` | the page's half: drop → post to the worker → render the result, the never-bigger guard, the funnel + copy, the honest score, and the slow-path timer |
| `worker.js` | the engine's thread: `init()`s the wasm, runs the `web` geometry (auto-orient + downscale) then `optimizeDetailed` at speed 10, and borrows the browser's AVIF decoder |
| `demo.css` | the whole stylesheet |
| `vendor/` | the vendored `crustyimg-wasm` build output — generated, gitignored |
