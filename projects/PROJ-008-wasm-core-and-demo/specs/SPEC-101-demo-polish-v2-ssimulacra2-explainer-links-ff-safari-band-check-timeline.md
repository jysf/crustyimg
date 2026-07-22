# SPEC-101 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-101-<cycle>.md`.

## Instructions
- [x] design — framed build-ready 2026-07-20. Batched demo polish (demo-files-only): (1) link the score
  panel's "SSIMULACRA2" to the metric explainer https://github.com/cloudinary/ssimulacra2 (WITH the "2")
  + a secondary link to the impl https://github.com/rust-av/ssimulacra2; (2) confirm the SPEC-081
  `color-mix()` score band renders on REAL Firefox+Safari (the load-bearing half — closes the SPEC-081
  launch carry). Links are href-only (zero-network holds). Logo swap DEFERRED (no logo yet). Sonnet build
  / Opus verify. Complexity S.
- [~] design (added 2026-07-20, maintainer feedback) — **third demo-polish item: a visible re-convert
  signal.** The Advanced-control re-convert already works (`demo/demo.js:520-536` — format/maxEdge/
  maxBytes/keepFull → debounced `convert()` on the kept `source`) but is SILENT ("it's working, but hard
  to tell"). Add a legible signal (surface the busy state on re-convert / an "Updated" pulse / an explicit
  Regenerate affordance) WITHOUT changing the auto-rerun behavior. Demo-files-only, still Complexity S.
  **Maintainer still testing the live demo — confirm the item is real (vs a discoverability miss) before
  the build.**
- [x] design (expanded 2026-07-22, maintainer: ONE spec for the whole demo pass) — batched because every
  item needs the same multi-browser setup and the demo is where the r/rust post lands. Now FOUR items:
  (1) SSIMULACRA2 explainer links; (2) the visible re-convert signal; (3) **the favicon set** — wire the
  7 files sitting untracked in `demo/` with RELATIVE hrefs + fix `site.webmanifest` (absolute icon `src`
  → relative, or they 404 under the `/crustyimg/` subpath; empty `name`/`short_name`; white theme colors
  on a dark demo); (4) **the DEVICE GATE** — real iOS Safari + real Android Chrome, checking the module
  Worker, `createImageBitmap` on `.avif` input, and large-photo memory. **(4) is a launch go/no-go, not
  polish** — the demo has only ever been proven on desktop. An honest documented degradation passes; a
  silent hang or crash is a blocker. Demo files only, no engine change, no wasm rebuild. Complexity M.
