---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
stage:
  id: STAGE-025
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-12
shipped_at: null

value_contribution:
  advances: >
    The load-bearing leg of the WASM wave: prove the pure-Rust engine runs a real
    decode → transform → encode round-trip in a browser with no backend, and resolve
    the one thing the design-time probe found blocking (AVIF codecs on wasm32) — so
    STAGE-026 (npm library) and STAGE-027 (demo page) are packaging over a proven core,
    not a hope.
  delivers:
    - "A `wasm` build of the pure engine (Image::from_bytes → operations/pipeline → sink::encode_to_bytes + analysis/decide) compiling to wasm32 behind a feature/cfg, with the native-only deps gated out"
    - "A thin wasm-bindgen surface (transform(bytes, recipe_toml) → bytes, info(bytes), an optimize/auto-format entry) that runs a real round-trip headlessly"
    - "A resolved, DEC-recorded strategy for AVIF on wasm — encode (rav1e) and decode (re_rav1d) — with an honest scope of what converts in-browser at ship"
    - "A measured binary-size number + budget (release + wasm-opt), and a stable/MSRV toolchain recipe (`just wasm-build` or equivalent)"
  explicitly_does_not:
    - "Package to npm or build the demo page (STAGE-026 / STAGE-027)"
    - "Add engine features, ops, or formats — this re-hosts the shipped engine"
    - "Enable threads/SharedArrayBuffer or a WASI-polyfill demo unless the AVIF decision forces it (single-threaded browser path is the default)"
    - "Touch the native default/lean builds — every wasm concession lives behind the `wasm` feature / cfg(target_arch = wasm32)"
---

# STAGE-025: WASM core build (the load-bearing probe)

## What This Stage Is

The stage that turns "the core *should* compile to WASM" into "the core *does* run
in a browser, and here is exactly what it can convert." It compiles crustyimg's
already-I/O-agnostic pure path (`Image::from_bytes` → `operation`/`pipeline` →
`sink::encode_to_bytes`, plus `analysis`/`decide`) to `wasm32` behind a `wasm`
feature / `cfg(target_arch = "wasm32")`, gates the native-only dependencies out, adds
a thin `wasm-bindgen` surface, and proves a decode → transform → encode round-trip
runs headlessly. Its center of gravity is a decision the design-time probe forced:
**AVIF codecs on wasm32.** When all its specs ship, STAGE-026 and STAGE-027 are
packaging and UI over a proven, size-measured core.

## Why Now

- **First stage of the wave, and everything else depends on it.** The npm library and
  the demo page are both thin layers over the WASM build; if the build isn't proven and
  sized, they're speculative. This stage is the probe made real.
- **The design-time probe (2026-07-12) already de-risked most of it and found the one
  hard spot.** `cargo build --lib --target wasm32-unknown-unknown --no-default-features
  --keep-going` compiled the *entire* tree except **`re_rav1d`** (the AVIF decoder):
  `resvg`/`usvg`/`tiny-skia` (SVG), `fast_image_resize`, `image` (PNG/JPEG/GIF/WebP/BMP/
  TIFF), `avif-parse`, `rayon`, `ssimulacra2`, `skrifa`, `zeno` all built. So SVG +
  raster decode/encode + resize/optimize/transform already work in-browser; the only
  open question is AVIF, and it's worth resolving deliberately rather than stumbling on.
- **The headline is AVIF *encode*, not decode — and encode is still untested.** The
  compelling demo moment is "drop a PNG, get a smaller AVIF" (encode = `rav1e`, behind
  the `avif` feature), not "read an `.avif` input" (decode = `re_rav1d`). `rav1e`'s
  wasm32 status is currently *unknown* because `re_rav1d` blocks the build before it's
  reached. Establishing the build with AVIF-decode gated out unblocks testing the encode
  path that actually carries the headline.

## Success Criteria

- The pure path compiles to `wasm32-unknown-unknown` behind the `wasm` feature/cfg with
  `notify`/`viuer`/`clap`/`clap_complete`/`rayon`-batch/`indicatif` and the C codecs
  (`libheif`, lossy-`webp`) gated out; the native default and lean builds are unchanged
  (`cargo build` / `--no-default-features` still green; `just deny` unchanged).
- A `wasm-bindgen` surface exposes at least `transform(bytes, recipe_toml) → bytes`,
  `info(bytes)`, and an `optimize`/auto-format entry, and a **headless harness runs a real
  decode → transform → encode round-trip** (Node or `wasm-bindgen-test` in a headless
  browser) producing correct output bytes — not just a compile.
- The **AVIF-on-wasm strategy is decided and DEC-recorded**: (a) whether `rav1e` (encode)
  compiles/runs on wasm32 and ships in the demo, and (b) which path resolves `re_rav1d`
  (decode) — gate-out-for-now / port-shim / wasm32-wasi / browser-native — with the
  in-browser conversion scope stated honestly (no overclaim; the wave's earned-verdict rule).
- A **measured `.wasm` size** (release + `wasm-opt`) against a stated first-load budget,
  with lazy-loading identified for any heavy codec that blows it.
- A **repeatable, stable-toolchain build recipe** (`just wasm-build` or similar) that
  works past the Homebrew-vs-rustup `rustc` gotcha, on a stable/MSRV toolchain (not just
  the nightly the probe used).

## Scope

### In scope
- The `wasm` feature + `cfg(target_arch = "wasm32")` boundary; gating native-only deps
  out; the `wasm-bindgen` surface; the headless round-trip harness; the AVIF decision +
  DEC; size measurement + budget; the build recipe + toolchain install (wasm-pack /
  wasm-bindgen-cli / wasm-opt / binaryen).

### Explicitly out of scope
- npm packaging (STAGE-026), the demo page (STAGE-027), new engine features/formats,
  threads/SharedArrayBuffer (unless the AVIF decision forces WASI), and any change to the
  native build paths.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-072 (shipped 2026-07-12, PR #80 `c3813a5`, DEC-064) — **WASM build seam + baseline (AVIF-decode gated out).**
  `cfg(target_arch="wasm32")` boundary + target-scoped dep tables (move `re_rav1d`/`avif-parse` — the sole
  compile blockers — + the fs/CLI-shell deps to not-wasm32; `wasm-bindgen` to wasm32); gate `src/image/avif.rs`
  (mod at `image/mod.rs:29`, dispatch `353-355`) + the `cli`/`source`/`build` modules out; `crate-type=["cdylib","rlib"]`;
  a thin `src/wasm.rs` `wasm-bindgen` surface (transform/info/optimize) over the existing
  `from_bytes → build_pipeline → encode_to_bytes` path; `#[wasm_bindgen_test]` round-trip; `just wasm-build`
  fixing the RUSTC toolchain gotcha on a STABLE toolchain. Output: a real `.wasm` that decodes/encodes
  SVG + PNG/JPEG/GIF/WebP + resizes in a browser. Native default + lean builds unaffected. Likely emits DEC-064.
- [x] SPEC-073 (shipped 2026-07-12, PR #82 `f027d79`, DEC-065) — **AVIF-on-wasm decision.** Design-time
  probe RESOLVED the central question: **rav1e 0.8.1 + ravif 0.13.0 compile to wasm32** (`--features
  avif`, exit 0) → AVIF *encode* is achievable (the "convert to AVIF in-browser" headline); AVIF
  *decode* (re_rav1d) stays gated (SPEC-072), reading `.avif` inputs deferred to demo-side
  `createImageBitmap`. Spec: wire `out_format="avif"` into `src/wasm.rs`, enable `avif` for the wasm
  build per a size-measured strategy, PNG→AVIF `#[wasm_bindgen_test]`, **measure the .wasm size delta**
  (rav1e is large — the decisive SPEC-074 input), DEC-065 (encode in / decode deferred). Native unaffected.
- [ ] SPEC-074 (not yet framed, may fold into SPEC-072) — **size budget + optimization.**
  Release + `wasm-opt`, measure, set the first-load budget, identify lazy-loaded codecs;
  record the number. *(Fold into SPEC-072 if the baseline number is already within budget.)*

**Count:** 2 shipped / 0 active / 1 pending (SPEC-072 wasm seam + SPEC-073 AVIF-encode both SHIPPED, DEC-064/065; only SPEC-074 bundle size left, then STAGE-025 completes)

## Design Notes

- **Probe findings this stage is grounded in (2026-07-12, design-time, no repo edits):**
  only `re_rav1d` fails wasm32 (libc POSIX types `off_t`/`ptrdiff_t`/errno + threading
  const-eval `E0080`); everything else compiles, incl. `rayon` (wasm fallback) and the SVG
  stack. Full detail in the [[proj-008-wasm-core-and-demo-framed]] memory; a proper run
  record lands with SPEC-073.
- **AVIF priority = encode over decode.** The demo headline is "convert *to* AVIF" (rav1e);
  reading `.avif` inputs (re_rav1d) is secondary, so gating decode out first (SPEC-072) is a
  legitimate MVP, not a failure — SVG conversion still lands, and the encode headline becomes
  testable. State the in-browser conversion scope honestly at ship.
- **Toolchain gotcha (cost a debug cycle in the probe):** cargo invokes bare `rustc` → on
  this machine PATH resolves Homebrew's stable rustc (no wasm std) → a misleading "can't find
  crate for core". Fix: force `RUSTC` to the rustup toolchain's rustc, or install a rustup
  stable toolchain + its wasm32 target and use it consistently. Bake this into `just wasm-build`.
- **Feature-boundary discipline:** every wasm concession is `wasm`-feature / `cfg(wasm32)`
  gated; the native default + lean builds must stay byte-for-byte unaffected (a lean-build
  check belongs in verify — the [[verify-includes-lean-no-default-features-build]] lesson).
- **`deny`/license:** no new *default* dep; wasm-bindgen + any wasm-only deps are gated and
  must pass `just deny` (or a scoped, justified exception — the [[license-watchlist-practice]]).

## Dependencies

### Depends on
- The shipped I/O-agnostic core (PROJ-008 brief): `Image::from_bytes`, `sink::encode_to_bytes`
  (`src/sink/mod.rs:573`), the `operation`/`pipeline`/`analysis::decide` modules; the existing
  `[lib]` target + the `display`/`watch` optional-dep feature-gating precedent.
- The design-time WASM probe (2026-07-12) that established the blocker set.
- External: the wasm toolchain (`wasm-pack` / `wasm-bindgen-cli` / `wasm-opt`/binaryen), not
  yet installed here; a rustup stable toolchain with the `wasm32-unknown-unknown` target.

### Enables
- STAGE-026 (npm library) — packaging over a proven, sized WASM build.
- STAGE-027 (demo page) — the in-browser "watch it just work" artifact.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
