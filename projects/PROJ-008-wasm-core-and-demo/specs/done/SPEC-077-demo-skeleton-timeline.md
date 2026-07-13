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
- [x] **verify** (2026-07-13, worktree `crustyimg-wt-verify077`) — **CLEAN, ready to ship** after
  fixing 3 defects on the branch; all three were in the *test/docs*, the page itself needed no
  change. Re-drove every claim in real headless Chrome: init() over HTTP (version = crate version,
  `application/wasm`, no console errors), PNG/SVG/JPEG/GIF/WebP in → WebP/PNG out through the real
  file input, **ZERO network requests during conversion**, and the download's bytes decoded by
  **`sips`** — a decoder nobody here wrote (64×48 webp ✓). AVIF disabled + never invoked; `src/`
  diff empty; cargo build/test/clippy/fmt, `just deny`, `just validate` green. Assembly guard
  **mutation-tested** (forged a 1 MB `name` section → refused, exit 1). The `file://` correction is
  **independently confirmed**: `demo.js` is fetched and CORS-refused from origin `null`, and the
  `.wasm` is *never requested* — `init()` is genuinely never reached.
  **⚠ THE HEADLINE FINDING — the smoke's `waitFor()` was waiting for nothing.** `PAGE_STATE` was
  unparenthesized, so `${PAGE_STATE} === 'done'` parsed as `state ?? (null === 'done')` = `state ??
  false` — **truthy for every state**. Every wait returned on its first poll and each read RACED the
  conversion it was meant to await: **3 failures in 8 runs**, each check reading the *previous*
  file's result. This was the root cause of the two "races" the build fixed on CI — its fixes were
  symptoms-only, so the 22 green checks were partly luck and the deploy gate could have gone green on
  a page that never converted the file it was handed. Fixed (parens + a freshness token in `drop()`
  instead of waiting on an already-true `done`): **10/10 clean, 28 checks each**. Also fixed: the
  `justfile` still repeated the *disproven* `file://` story, and `pages.yml` did not trigger on
  `tests/**` — where the gate it blocks the deploy on actually lives.
- [x] **ship** (2026-07-13) — **SHIPPED.** Squash-merged **PR #85** (`9a61787`). Build + verify ran
  in worktrees (no collision). Cost 355k tok / $3.30 (per-session usd); ship reflection appended (the
  failure-mode-claims-are-unproven + the `waitFor`-waited-for-nothing lessons, both new memories);
  spec + timeline archived. STAGE-027: SPEC-077 shipped (1 shipped / 0 active / 1 pending — SPEC-078
  worker/AVIF/explain left). Roadmap carries: **GitHub Pages NOT enabled on the repo → deploy leg
  unproven end-to-end** (maintainer action); SPEC-078's Worker should take ALL conversions, not just
  AVIF. `pages.yml` = the repo's first CI-through-`just wasm-build` + browser smoke. `just validate`
  + `just cost-audit` green.
