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
- [x] **build** (2026-07-13, worktree `crustyimg-wt-spec077`, PR #85) — `demo/` (index.html +
  demo.js + demo.css, no bundler), `just demo-build`/`demo-serve`/`demo-smoke`, a 60-line static
  server with the `application/wasm` MIME, `.github/workflows/pages.yml` (deploy GATED on the
  browser smoke, which also runs on every PR — the repo's first CI job that builds through
  `just wasm-build`). Smoke drives REAL headless Chrome over CDP (no browser driver): init() +
  PNG/JPEG/GIF/WebP/SVG in → WebP/PNG out, download decoded by our own VP8L/IHDR parsers, zero
  network requests during conversion. Assembly guard reuses SPEC-075's structural strip
  fingerprint (now shared in `scripts/lib/`). AVIF wired but `disabled` — never called.
  **The spec's `file://` failure mode was WRONG** and the browser proved it: the ES module is
  CORS-blocked before it ever runs, so the page hangs on "Loading…" with no error rather than
  failing at `instantiateStreaming`; fixed with a classic script that explains it, and tested.
  `src/` diff empty. No new DEC.
- [ ] **verify** — fresh adversarial session (worktree): drive the served page in a real browser —
  init succeeds, drop→convert→download works client-side, deployed `.wasm` is the profiled one,
  no backend/network for conversion, native untouched.
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals (per-session usd), reflection, memory + brag.
