---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
stage:
  id: STAGE-025
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-12
shipped_at: 2026-07-12

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
- [x] SPEC-074 (shipped 2026-07-12, PR #83 `506df80`, DEC-066) — **WASM bundle size.** Shrink the shipped demo
  `.wasm` (1.52 MB brotli w/ avif; rav1e ~0.35 is a KEEP, ~1.19 core is the debt). Design-time
  twiggy probe: NO single whale — mass in the SVG text/font stack (usvg text + ttf_parser +
  rustybuzz + unicode_bidi = resvg `text` feature) + the raster-codec spread (image's decoder set);
  ssimulacra2 NOT a top contributor. Method = feature-ablation brotli-diffing on the wasm-opt'd
  artifact + a size-tuned wasm profile (opt-level="z"/lto/panic=abort) + `wasm-opt -Oz`. No-cost
  levers pulled unconditionally; capability-losing ones (drop SVG `<text>`, trim a codec) = explicit
  DEC-066 calls with measured savings. `just wasm-test` stays green (no silent capability loss);
  native unaffected. **Ship completes STAGE-025.**

**Count:** 3 shipped / 0 active / 0 pending — **STAGE-025 COMPLETE 2026-07-12** (SPEC-072 wasm seam + SPEC-073 AVIF encode + SPEC-074 bundle size, DEC-064/065/066).

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

*Shipped 2026-07-12.*

- **Did we deliver the outcome in "What This Stage Is"?** **Yes, fully.** The stage turned "the
  core *should* compile to WASM" into a proven, sized, honest artifact: SPEC-072 shipped the wasm
  build seam (a real decode→transform→encode round-trip in-browser, no backend, native unaffected);
  SPEC-073 landed the headline (AVIF *encode* runs in the browser — PNG→valid `.avif`, verified by
  decoding the wasm-produced bytes with two independent decoders); SPEC-074 sized it honestly
  (1.52→1.33 MB brotli, −12.6%, by ablation, with every capability-losing lever refused-with-data).
  Every "it works/it's small" claim was *driven*, not asserted — the throughline of the stage.
- **How many specs did it actually take?** **3, exactly as framed** (SPEC-072/073/074) — because
  each was grounded by a design-time probe *before* framing (does the tree compile to wasm32? does
  rav1e? where do the bytes go?), so no spec discovered a surprise that forced a split. The
  probe-then-frame discipline is why the count held.
- **What changed between starting and shipping?** The AVIF story clarified from "unknown" to a
  proven asymmetry — encode works on wasm, decode doesn't (deferred to the browser's own
  `createImageBitmap`) — and the size work *corrected* two hypotheses under measurement (ssimulacra2
  wasn't the whale; the two "free" size levers actually cost speed/nothing).
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - **Probe the load-bearing unknown at *design* time — including the test runner and the *timing*,
    not just the compiler.** SPEC-072's probe proved `cargo build --target wasm32` but not
    `wasm-pack test` (the actual hard part); SPEC-074 proved a size spec must *time* the artifact
    (encode speed is a capability), not just weigh it. Both are the [[probe-load-bearing-crates-at-design]]
    lesson, widened.
  - **On wasm a panic aborts the module / crashes the page** — "typed error, never panic" is a hard
    rule there, not a nicety (SPEC-072/073). Worth a wasm framing in `untrusted-input-hardening`.
  - **An adversarial verify must re-drive the build's incidental "aha" findings, not just its
    acceptance criteria** — SPEC-074's most quotable claim (wasm-opt "fails silently at exit 0") was
    false and had propagated into 4 files + the auto-memory before verify caught and corrected it. A
    striking claim written as durable guidance is the highest-value thing to re-drive.
  - **Enabling a cargo feature for a new target flips *every* `cfg(feature)` site at once** (SPEC-073:
    `avif` on silently re-routed `optimize` into a decoder-needing search) — audit the whole set.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - Yes — "**sniff ≠ valid: decode wasm output with an independent decoder**" ([[verify-wasm-output-with-an-independent-decoder]])
    and "**time the artifact, not just weigh it**" are the two most reusable, both now their own memories.
  - **Carried to the rest of PROJ-008:** a **wasm CI job must build through `just wasm-build`** (the
    size profile lives in the recipe's env vars, not `[profile.release]` which native shares → a bare
    `cargo build --target wasm32` silently ships +109 KB); STAGE-027 inherits rav1e-runs-*serial*
    (Web Worker + progress) and must decode `.avif` inputs page-side via `createImageBitmap`; and
    `optimize(_, "webp")` on wasm returns *lossless* WebP (no lossy-WebP encoder in the wasm set).
