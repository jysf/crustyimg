# SPEC-075 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] **design** (2026-07-12, orchestrator main loop) — framed build-ready. Grounded in a
  design-time probe of `just wasm-build`'s `pkg/`: wasm-pack emits a valid package.json + typed
  `.d.ts`; the npm names `crustyimg` / `crustyimg-wasm` / `@jysf/crustyimg` are ALL free (404); and
  the `--target web` package needs explicit `init()`/`initSync`. Spec = settle identity/target/
  versioning (DEC-067) + `npm pack` → fresh-install → run `info`/`transform` client-side smoke test.
  NO live publish (SPEC-076, gated).
- [x] **build** (2026-07-12, PR #84, ~130k tok / ~$1.20 est) — DEC-067 settled: **`crustyimg-wasm`**,
  `--target web` (one artifact), version in lockstep with the crate, publish gated. `just wasm-npm-pkg`
  (finalize, **depends on `wasm-build`** so the DEC-066 size profile can't be bypassed) +
  `just wasm-npm-smoke` (npm pack → fresh install → import the bare specifier → `initSync` → `info` +
  `transform` to png/jpeg/webp/avif, output decoded back). No native addon, no lifecycle script, zero
  transitive deps. Size guard mutation-tested against a real bare build (1,503,817 B vs the profiled
  1,394,313 B — it trips). Native gates + deny + validate green; `src/` untouched. No publish.
- [x] **verify** (2026-07-13, fresh adversarial session, isolated worktree) — **CLEAN, ready to
  ship**, with **one hardening applied on the branch** (test-only; `src/` still untouched). Every
  claim re-driven from a fresh build, not read.

  **The package packs, installs, and runs — reproduced end to end.** `just wasm-npm-smoke` green
  from a clean worktree build: `npm pack` → install THAT tarball into a fresh temp project → a
  **separate Node process** imports the **bare specifier** `crustyimg-wasm`, resolves the `.wasm`
  through the package's own `./crustyimg_bg.wasm` subpath export, `initSync`s, and runs `info` →
  64×48 png / hasAlpha false, then `transform` → png/jpeg/webp/avif. The resized PNG is confirmed
  **32×24 twice**: an independent JS IHDR parse *and* `info()` on the output bytes.

  **The "not sharp" pitch holds, proven from the tarball.** Packed tarball is **8 files** — 2
  LICENSEs, README, `crustyimg.js`, `crustyimg.d.ts`, `crustyimg_bg.wasm(.d.ts)`, `package.json`.
  **No `.node`/`.dylib`/`.so`/`.dll`/`binding.gyp`** anywhere in `node_modules` after a real install
  (run WITHOUT `--ignore-scripts`, so a postinstall would have fired); the installed `package.json`
  has **no `scripts` key at all** and **no `dependencies` key at all** — zero transitive deps.

  **The size guard trips — and the threshold was the right thing to distrust.** Reproduced both
  endpoints: profiled `just wasm-build` = **1,394,631 B** brotli, stock-profile `wasm-pack` (no
  DEC-066 env vars) = **1,503,485 B** — the +109 KB (108,854 B), and the shipped 1.45 MB ceiling
  does trip it. **But the band was load-bearing on 4% of daylight**: 55,369 B of headroom above the
  profiled build (4.0%) and 53,485 B of margin below the stock one (3.7%), for a total separation of
  only 7.8%. Any legitimate 4% growth false-trips it — and *misreports the cause*, since the message
  blames a bypassed recipe. Worse, the discrimination erodes as the artifact grows, because a stock
  build grows with it. The builder's own reflection called this "chosen by assumption, validated by
  luck", and it was right to.

  **Hardened it structurally instead (applied, mutation-tested).** `strip = true` — one of DEC-066's
  three levers — is **directly observable in the binary**: a stripped `.wasm` has no debug-name
  table. Measured: the profiled artifact carries a **42 B `name` custom section**, a stock-profile
  one carries **980,292 B**. Four orders of magnitude — **categorical, not a threshold**, and immune
  to however much the code legitimately grows. `tests/npm_smoke.mjs` now asserts *that* (a 4 KB
  ceiling on the `name` section) as the "came through `just wasm-build`" proof, and demotes the size
  band to a plain **regression baseline keyed to the measured 1,394,631 B ±5%**, so a real size
  regression now reads as a size regression rather than a bypassed recipe. Mutation-tested by
  swapping the stock `.wasm` into the finalized `pkg/`: the structural check fails **240× over the
  line**, not 3.7%.

  **The recipe edge is real.** `wasm-npm-pkg: wasm-build` and `wasm-npm-smoke: wasm-npm-pkg` — every
  path to a package rebuilds through the size profile; `pkg/` is regenerated from scratch each time,
  so there is no stale-artifact path to a non-profiled `.wasm`.

  **DEC-067 holds, each clause driven.** `npm view crustyimg-wasm` → **404** (nothing published).
  Name/target/version confirmed in the *packed* `package.json` (`crustyimg-wasm`, `type: module`,
  `0.4.0`). The **browser** init path exists and is typed, not just the Node one: `crustyimg.js`
  default-exports `__wbg_init`, falls back to `new URL('crustyimg_bg.wasm', import.meta.url)` and
  uses `WebAssembly.instantiateStreaming` on a `Response`; `InitInput = RequestInfo | URL | Response
  | BufferSource | WebAssembly.Module`. **Lockstep versioning is enforced, not hoped** — I tried to
  break it three ways and all three die at exit 1: a drifted `pkg/` version (0.4.1 vs Cargo.toml's
  0.4.0), an override that sets its own `version`, and an override smuggling in a `postinstall`.

  **AVIF's container-only check is honest — and the output is better than the build proved.**
  `info(avif)` genuinely **throws** ("AVIF decoding isn't available in the WebAssembly build") — the
  limitation is real (DEC-065), not a lazy assertion. And the wasm-encoded AVIF **decodes correctly
  in an independent decoder**: macOS `sips` reads it as **32×24 avif** — the class of decoder the
  browser actually uses, so `rav1e`-in-wasm is producing genuinely valid files, not just a valid
  `ftyp` box.

  **No publish, and none reachable.** Nothing in `justfile` / `scripts/` / `npm/` / CI invokes
  `npm publish` (the only `publish` hits are the pre-existing **cargo** crate release and homebrew
  jobs). The finalize script additionally *refuses* any lifecycle script, so the published artifact
  could not run anything on a consumer's machine either.

  **Native unaffected, driven not assumed.** `git diff origin/main -- src/` is **empty**. `cargo
  fmt --check`, `cargo build`, `cargo build --no-default-features` (lean), `cargo test` (**716
  passed, 29 suites**), `cargo clippy --all-targets -D warnings`, `just deny` (advisories/bans/
  licenses/sources ok), `just validate` — all green.

  **Carried to ship (not defects):** (1) **CI still does not run `just wasm-npm-smoke`** — the
  package is proven on one Mac; this folds into the wasm CI job STAGE-025 already owes, and that job
  MUST build through `just wasm-build`. (2) Two different "brotli" numbers are in circulation for
  the same artifact — `brotli -q 11` (the recipe) says **1,394,313 B**, Node's `brotliCompressSync`
  (the guard) says **1,394,631 B**; a 318 B tooling difference, harmless but worth not confusing.
- [x] **verify** (2026-07-13, fresh adversarial session, isolated worktree) — **CLEAN**, one
  hardening applied on-branch (commit 03b291f, `-s`). Re-drove `just wasm-npm-smoke` from a clean
  build: bare-specifier import in a separate Node process, `initSync` via the package's own subpath
  export, PNG resized to 32×24 confirmed by an independent JS IHDR parse AND `info()`. Tarball = 8
  files, no `.node`/lifecycle script/deps. **Hardened the size guard: dropped the fragile 1.45 MB
  ceiling (±4% daylight) for a STRUCTURAL `strip`-fingerprint assertion** (name section 42 B
  profiled vs 980,292 B stock — mutation-tested, fails 240× not 3.7%); size demoted to a ±5%
  regression baseline. DEC-067 holds (npm 404; lockstep broken 3 ways → all exit 1; browser init
  path typed). AVIF honest — `info(avif)` throws, `sips` decodes the wasm-encoded AVIF 32×24. Native
  untouched (716 tests, `src/` diff empty); PR #84 CI 52 pass / 0 fail.
- [x] **ship** (2026-07-13) — **SHIPPED.** Squash-merged **PR #84** (`125a590`, DEC-067). Cost
  225k tok / $2.10 (per-session usd recorded). Ship reflection appended (the structural-guard
  lesson); spec + timeline archived. STAGE-026: SPEC-075 shipped (1 shipped / 0 active / 1 pending —
  SPEC-076 publish, gated). Roadmap carries updated (wasm CI job now owes `wasm-npm-smoke` too; bare
  `crustyimg` npm name reserved for a future CLI). Shared checkout reconciled to `main`.
  `just validate` + `just cost-audit` green.
