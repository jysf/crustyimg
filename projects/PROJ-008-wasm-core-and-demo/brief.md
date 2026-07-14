---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-008
  status: active                    # proposed | active | shipped | cancelled
  priority: high
  target_ship: null                 # optional: YYYY-MM-DD

repo:
  id: crustyimg

created_at: 2026-07-12
shipped_at: null

# Business value. Testable claim, not marketing copy.
value:
  thesis: >
    Compile crustyimg's already-I/O-agnostic pure-Rust core to WebAssembly and ship
    two artifacts over it: an **npm-packaged library** that runs the engine
    (decode → transform/optimize → encode) entirely **client-side, with no native
    addon**, and a **squoosh.app-style demo page** where you drop an image and watch
    it become the smallest modern-format artifact — in the browser, no upload, no
    backend. The demo is the highest-ROI adoption artifact the project has: a
    zero-install, inherently shareable "try it now" that finally gives the Track-B
    funnel something to point at, and it is where the "watch it just work" moment
    lives (in-browser AVIF/SVG conversion). The npm library is a sharp answer to
    `sharp`'s and the abandoned `@squoosh/cli`'s pain: image optimization in a JS
    toolchain with no ABI/native-build/CI-breakage friction. Both stay strictly
    client-side to honor the no-service / no-CDN guardrail (`docs/territory.md`).
  beneficiaries:
    - "The abandoned @squoosh/cli / libSquoosh audience and no-Node / Makefile shops (the warm leads named in the roadmap)"
    - "npm / JS-toolchain developers who want image optimization without sharp's native-addon ABI/CI friction"
    - "crustyimg's own adoption funnel — a shareable, zero-install 'try it' artifact to time a Show HN around"
    - "Evaluators who want to try the engine (formats, byte savings, explain) before installing a binary"
  success_signals:
    - "The pure transform core (from_bytes → operations → encode_to_bytes) compiles to wasm32 and runs in a browser with NO backend"
    - "In-browser decode of AVIF/SVG/PNG/JPEG/WebP and encode to WebP/PNG (+ AVIF where size permits) — the 'watch it just work' conversion"
    - "An installable npm package exposes a typed JS/TS API (transform/convert/resize/optimize/info) that runs client-side with no native addon"
    - "A public, static-hosted demo page: drop an image → pick an intent → see the optimized result + format chosen + bytes saved + explain → download, all in the browser"
    - "First-load bundle stays within a stated size budget (lazy-loaded codecs where needed) so 'instant try it' is actually instant"
  risks_to_thesis:
    - "Binary size — rav1d (AVIF decode) + resvg (SVG) + any AVIF *encoder* (rav1e) compiled to WASM can be multi-MB, which directly undercuts a 'zero-install, instant' demo (the load-bearing probe; DEC-054 already flagged WASM binary size)"
    - "Threads/SIMD — rayon batch parallelism and SharedArrayBuffer need COOP/COEP headers a static host (GitHub Pages) can't easily set; the single-image demo path must not depend on threads, and large-image perf on the single-threaded path is unproven"
    - "Scope creep into a 'web app' or the maintainer's separate site-builder tool — the demo is a thin client-side marketing artifact, not a product and not HTML generation (territory guardrail)"
    - "Codec-encode parity in the browser — if the heavy encoders don't fit the size budget, the in-browser story may be decode-broad but encode-narrower than the CLI; that gap must be scoped honestly, not overclaimed"
---

# PROJ-008: WASM core + demo page

## What This Project Is

The **reach wave** — roadmap Wave 3 (ships around 0.6.0). crustyimg's engine is
already a pure function — `input bytes + recipe → output bytes` — with the
filesystem, CLI, terminal, and batch concerns deliberately quarantined in the
`source` / `sink` / `cli` shell (the `operation`, `pipeline`, and `analysis::decide`
modules are documented as free of `std::fs` / `sink` / `cli`, and the seam functions
`Image::from_bytes` and `sink::encode_to_bytes` already take and return bytes). This
wave compiles that core to **WebAssembly** and builds two things on top of it: an
**npm-packaged library** (a `wasm-bindgen` surface + typed JS/TS wrapper that runs
the engine client-side) and a **squoosh-style demo page** (drop an image, declare an
intent, watch it become the smallest modern-format artifact — decoded, transformed,
and re-encoded entirely in your browser, with the `explain`-style record of what it
did). It is the local, client-side counterpart to the shipped CLI: the same engine,
no install, no upload, no service.

## Why Now

- **It's the highest-ROI adoption artifact left before 1.0, and PROJ-007 unblocked it.**
  The build-out waves (input reach, build/cache/lockfile) made the engine broad and
  trustworthy; adoption is the binding constraint (`docs/roadmap.md`, "adoption-first"),
  and the demo page is the marketing artifact the Track-B funnel has never had — a
  zero-install, inherently shareable "try it" that a Show HN can point at. Sequencing
  rationale (2026-07-07) explicitly moved the WASM core + demo up for exactly this reason.
- **The seam already exists and is architecturally enforced.** This is not a rewrite:
  `Image::from_bytes` / `sink::encode_to_bytes` are the bytes-in/bytes-out core, and
  the pure-Rust decode/rasterize choices were made *with this wave in mind* — DEC-053
  (`re_rav1d` AVIF decode) and DEC-054 (`resvg` SVG) both note "the decoder/rasterizer
  serves the Wave-3 WASM demo." The work is a thin shim + getting the dep tree to
  compile to `wasm32`, not new engine code.
- **The library is a sharp, differentiated pitch.** `sharp` (Node + native addon) and
  the **abandoned** `@squoosh/cli` leave a vacated category: image optimization in a JS
  toolchain that installs like any pure-JS package — no ABI mismatch, no node-gyp, no
  CI-breakage. A pure-Rust-to-WASM library *is* that, and it reuses the exact engine the
  CLI ships.

## Success Criteria

- The pure transform core compiles to `wasm32-unknown-unknown` and runs a full
  **decode → transform → encode** round-trip in a browser with **no backend** — proven
  in a headless harness first, then in-page.
- In-browser **decode** of AVIF, SVG, PNG, JPEG, WebP and **encode** to WebP + PNG (and
  AVIF where the size budget permits) — the in-browser AVIF/SVG conversion is the
  headline "watch it just work" moment.
- An **installable npm package** exposes a typed JS/TS API for the load-bearing commands
  (transform via recipe, convert, resize/thumbnail, the `optimize`/auto-format engine,
  `info`) that runs entirely client-side — **no native addon**.
- A **public, static-hosted demo page** (client-side only): drop or paste an image, pick
  an intent (a quality target / byte budget / format), and see the optimized result with
  the **format chosen, bytes saved, and a perceptual/explain readout**, downloadable —
  all in the browser, honoring no-service / no-CDN.
- **Bundle size** stays within a stated first-load budget (codecs lazy-loaded where
  needed) so the demo is genuinely instant; the number is set by the STAGE-025 probe and
  recorded in its DEC.
- No regression to the CLI or the pure-Rust-default posture: `just deny` unchanged, the
  native default/lean builds unaffected, and any WASM-only concessions live behind a
  `wasm` feature / `cfg(target_arch = "wasm32")`, never on the native path.

## Scope

### In scope
- **A WASM build of the core** behind a `wasm` feature / `wasm32` cfg: the
  `from_bytes → pipeline/operations → encode_to_bytes` path plus the `analysis`/`decide`
  optimization engine, with the filesystem/CLI/terminal/batch concerns and WASM-hostile
  deps (`notify`, `viuer`, `clap`, `rayon`-batch, the C codecs `libheif`/lossy-`webp`)
  gated out; a `wasm-bindgen` surface (`transform(bytes, recipe_toml) -> bytes`,
  `info(bytes)`, an `optimize`/auto-format entry). **(STAGE-025 — the load-bearing probe:
  compilation + binary size + single-threaded perf.)**
- **An npm-packaged JS/TS library** over the WASM build (`wasm-pack`/`wasm-bindgen`), with
  a small typed API mirroring the load-bearing CLI verbs, published (or dry-run/tagged) to
  npm; installs with no native addon. **(STAGE-026.)**
- **A client-side demo page**: a static single-page "drop an image → declare intent →
  optimized modern-format result + explain + download" experience, hosted statically
  (e.g. GitHub Pages), 100% in-browser. **(STAGE-027.)**

### Explicitly out of scope
- **Any backend, server, or hosted service** — the WASM core is more valuable as a
  **library** than a server, and a running service would violate the no-service / no-CDN
  guardrail. `serve` stays a demand-gated 2.0 item.
- **HTML generation / templating / routing / the maintainer's separate site-builder tool**
  — the demo page is crustyimg's own thin marketing artifact; the manifest (Wave 4) is the
  seam that keeps the two apart (`docs/territory.md`).
- **New image formats or engine features** — this wave *re-hosts* the shipped engine; it
  does not add ops, formats, or the web-asset manifest (Wave 4), geometry (Wave 5), or
  smart-crop (post-1.0 beta).
- **`heic` in WASM** — HEIC is a C system library (libheif) and stays a native opt-in
  feature only (DEC-052/DEC-056); it never enters the WASM build.
- **AVIF *encode* is best-effort, size-budget-gated** — if the rav1e encoder blows the
  bundle budget, in-browser encode leans on the pure-Rust WebP/PNG paths and AVIF encode
  is documented as a native-CLI capability, not overclaimed for the browser. (Decode of
  AVIF via `re_rav1d` is in scope and is the priority.)
- **PROJ-008 "stretch" frontier items** (AI super-res, face-aware crop, JPEG XL, RAW
  Tier-2) — those are the 2.0+ roadmap row, not this wave.

## Stage Plan

Ordered list of stages this project will produce. Update as work proceeds.

Format: `- [status] STAGE-ID — one-line summary`

- [x] STAGE-025 (shipped on 2026-07-12) — **WASM core build.** The pure engine runs in-browser with
  no backend: a `cfg(target_arch="wasm32")` boundary + a thin `wasm-bindgen` surface over
  `from_bytes → build_pipeline → encode_to_bytes`, gating the fs/CLI shell + `re_rav1d` (the one
  wasm32 blocker) out. **SPEC-072** (seam + baseline, DEC-064), **SPEC-073** (AVIF *encode* runs on
  wasm — the headline; decode deferred to `createImageBitmap`, DEC-065), **SPEC-074** (bundle size
  1.52→**1.33 MB brotli** by ablation, DEC-066). Every "works/small" claim was driven, not asserted.
  Three specs, exactly as framed — each grounded by a design-time probe so none forced a split.
- [ ] STAGE-026 (framed + active on 2026-07-12) — **npm-packaged library.** Package the shipped
  WASM build (`just wasm-build`'s `pkg/` — wasm-pack already emits a near-publishable package.json +
  typed `.d.ts`) into an installable npm module: settle identity (name/scope vs the crate), target
  (`web`/`bundler`), versioning, README; prove `npm pack` → fresh-install → `transform`/`info` runs
  client-side with no native addon; DEC for identity/target/versioning/publish. Publish is **gated
  on explicit maintainer approval** (outward-facing). The packaged `.wasm` must be the size-profiled
  build (the STAGE-025 +109 KB footgun). Specs: SPEC-075 (package shape + smoke test + DEC — frame
  first), SPEC-076 (publish/release, gated, foldable).
- [x] STAGE-027 (shipped on 2026-07-13) — **the demo page.** The crustyimg engine runs as a real,
  LIVE web page (https://jysf.github.io/crustyimg/): drop → convert client-side → download, AVIF both
  directions off the main thread (Web Worker), `.avif` input via `createImageBitmap`, an honest
  explain readout — driven CLEAN in Chrome/Firefox/Safari. SPEC-077 (skeleton) + SPEC-078
  (Worker/AVIF/explain). One carry: **mobile verification** → STAGE-028 (real-device test before launch).
- [ ] STAGE-028 (proposed on 2026-07-13) — **launch readiness.** The capstone: the README front
  door (CLI-only today), honest BENCHMARKS.md, and the Show HN go/no-go against
  `docs/launch-readiness.md`. Depends on SPEC-078 (demo, incl. cross-browser) + SPEC-076 (gated npm
  publish); times them into one launch. Docs + coordination, not code. Specs: SPEC-082 (README),
  SPEC-083 (BENCHMARKS).

**Count:** 2 shipped / 1 active / 1 proposed (STAGE-025 + STAGE-027 SHIPPED; STAGE-026 — SPEC-075
shipped, gated SPEC-076 publish parked; STAGE-028 launch readiness proposed). Only the LAUNCH remains:
launch-readiness (README + benchmarks + mobile test) + the gated npm publish → Show HN.

## Dependencies

### Depends on
- The shipped I/O-agnostic core: `Image::from_bytes` / `Image::decode_path`
  (`src/image/mod.rs`), the `operation` / `pipeline` / `analysis` / `recipe` modules
  (documented free of `std::fs`/`sink`/`cli`), and `sink::encode_to_bytes`
  (`src/sink/mod.rs:573`, already shared with the build cache).
- The pure-Rust decode/rasterize choices made with this wave in mind: **DEC-053**
  (`re_rav1d` no-asm AVIF decode) and **DEC-054** (`resvg`/`usvg`/`tiny-skia` SVG),
  both of which explicitly anticipate the WASM demo; `fast_image_resize` (has wasm32
  SIMD support); the `image` pure-Rust codec set (PNG/JPEG/GIF/WebP).
- The `[lib]` target that already exists (`crustyimg` lib + bin), and the feature-gating
  precedent (`display`/`watch` optional deps dropped from the lean build) — the same
  mechanism gates the native-only deps out of the WASM build.
- DEC-004 (pure-Rust default), DEC-006 (no async runtime), the `untrusted-input-hardening`
  posture (decode caps carry into the browser).

### Enables
- The **Track-B adoption funnel** — the flagship shareable "try it" artifact and a
  no-native-addon npm alternative to `sharp`/dead `@squoosh/cli`.
- A future **web-asset manifest** consumer story and any later WASM/HTTP frontend seam
  (the enabler the private commercial notes depend on — kept out of this repo).
- Upstreaming momentum (a pure-Rust AVIF-decode PoC in the browser strengthens the
  `image-rs` contribution tracked in the roadmap).

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <yes/no + notes>
- **How many stages did it actually take?** <number, compare to plan>
- **What changed between starting and shipping?** <one or two sentences>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **What did we defer to the next project?**
  - <one-line items>

---

*Lineage: this instantiates the roadmap's provisional "PROJ-008 (WASM seam)" as the
concrete WASM-core + npm-library + demo-page wave (Wave 3). The "stretch" frontier items
once parked under PROJ-008 (AI super-res, face-aware crop, JPEG XL, RAW Tier-2) are NOT
part of this project — they are the 2.0+ "opt-in intelligence / new frontends" roadmap row.
Framed 2026-07-12 immediately after PROJ-007 closed (per AGENTS §2: a project is framed
formally only once the prior one ships).*
