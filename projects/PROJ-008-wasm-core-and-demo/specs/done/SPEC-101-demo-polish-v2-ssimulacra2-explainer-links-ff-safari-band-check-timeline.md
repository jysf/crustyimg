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
- [x] build — Sonnet, 2026-07-22 on `spec-101-demo-pass`. Three code items landed: SSIMULACRA2 explainer
  links (metric + Rust impl, `href`-only), the "Updated" re-convert pulse (auto-rerun behavior itself
  unchanged — still debounced, still guarded on a loaded source), and the favicon set + the three
  `site.webmanifest` fixes. **Verified the manifest against a real `/crustyimg/` subpath server, not a
  root-served dir** — the distinction is the whole point, since a root server makes the absolute paths
  look fine and hides the bug. Smoke extended to cover all three. **PAUSED before the device gate (no
  device access) — see the process note below.** ~$2.0.
- [x] verify (finalize + verify) — Opus 1M, 2026-07-22. ✅ **CLEAN on all three code items.** Finalized
  two loose ends: gitignored `demo/_*` (confirming the harness was never committed) and recorded the
  maintainer-decided device gate. Verified against the committed diff, not the build's prose: both link
  targets curl 200 and the metric link is the `ssimulacra2` repo (not v1); the re-convert signal fires
  and no new path triggers a convert; **the favicon check carried a NEGATIVE CONTROL — on a real subpath
  server the relative paths return 200 while the root-absolute paths a leading slash would produce
  return 404**, proving the trap was real and the fix clears it. Zero off-origin requests during
  conversion; no `src/` change; validate + smoke green. ~$6.5.
- [x] ship — squash-merged PR #109 (**fdd4447**) 2026-07-22, CI CLEAN first try (all 5 commits signed).
  Merge triggers a Pages redeploy, so the live demo gains the favicons, the score links and the Updated
  signal. **DEVICE GATE = PASS (maintainer-decided), recorded honestly:** iOS WebKit solid (real iPhone
  Safari + DuckDuckGo, incl. a real Photos-library batch — the OS transcodes HEIC→JPEG on export so the
  demo receives JPEG); desktop DuckDuckGo same batch; desktop Chrome/FF/Safari via SPEC-078; **Android
  Chrome NOT tested — accepted on judgment** (static, no backend, degrades gracefully). "Could be faster"
  on detailed photos = the known single-threaded AVIF encode, already the top post-launch item.
  ~$8.8 / 3 sessions. **Two carries out: (1) the demo can't open RAW `.dng` — extension-based routing vs
  the bytes-only wasm surface; probe written. (2) the logo swap, pending the outsourced mark.**
  Process lesson banked: [[never-drive-the-maintainers-live-browser]] — a gate needing human hardware
  belongs on the launch-readiness track, not in a build spec's acceptance criteria.
