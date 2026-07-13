# SPEC-077 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] **design** (2026-07-13, orchestrator main loop) — framed build-ready. Grounded in a read of
  the `crustyimg-wasm` package surface: exports `.`→crustyimg.js + `./crustyimg_bg.wasm`; default
  `init()` resolves the wasm as `new URL('crustyimg_bg.wasm', import.meta.url)` + `instantiateStreaming`
  → **the page MUST be served over HTTP (application/wasm MIME), not `file://`**. Spec = a static,
  no-bundler, single-threaded demo skeleton (drop → convert to WebP/PNG → bytes/dims/info → download),
  served + GitHub-Pages-deployable, size-profiled `.wasm`, browser-driven smoke. AVIF/worker/explain
  = SPEC-078.
- [ ] **build** — through a PR, in a WORKTREE (not the shared checkout). Assemble the demo (vendor
  pkg/ via `just wasm-build`), serve recipe, GitHub Pages deploy, browser-driven smoke. Keep AVIF off
  the main thread. Commit with `-s`.
- [ ] **verify** — fresh adversarial session (worktree): drive the served page in a real browser —
  init succeeds, drop→convert→download works client-side, deployed `.wasm` is the profiled one,
  no backend/network for conversion, native untouched.
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals (per-session usd), reflection, memory + brag.
