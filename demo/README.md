# The crustyimg demo

A static page that runs the crustyimg engine **in your browser**. Drop an image, convert it,
download the result — nothing is uploaded, and there is no backend to upload it to. The engine is
the same pure-Rust code the CLI runs, compiled to WebAssembly and shipped as
[`crustyimg-wasm`](../npm/README.md).

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

No COOP/COEP headers are needed: the engine is single-threaded, so there is no `SharedArrayBuffer`
to unlock. (Which is fortunate — GitHub Pages cannot set headers.)

## What it does

| | |
|---|---|
| **In** | PNG, JPEG, GIF, WebP, SVG (rasterized by resvg, `<text>` and all) |
| **Out** | WebP (lossless) and PNG |
| **Resize** | optional max long edge — runs a real recipe, the same TOML the CLI reads |
| **Runs** | entirely on your machine: no server, no upload, no network call for the conversion |

**AVIF output is not here yet.** The engine can encode it — that is the headline of this wave — but
`rav1e` is serial and takes seconds, and this page calls the engine synchronously on the main
thread, so an AVIF encode would freeze the tab. It lands in the next spec, behind a Web Worker.

Lossy WebP is absent for a different reason: it is a C library, and this build is pure Rust. So the
WebP here is **lossless** — which means re-encoding an already-lossy JPEG can legitimately produce a
*bigger* file. The page says so when it happens instead of hiding it. Resize it, or wait for AVIF.

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
| `index.html` | the page (and the classic script that explains the `file://` failure) |
| `demo.js` | the whole app: load the engine, convert, render, download |
| `demo.css` | the whole stylesheet |
| `vendor/` | the vendored `crustyimg-wasm` build output — generated, gitignored |
