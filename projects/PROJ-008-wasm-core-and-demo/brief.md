---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-008
  status: shipped                   # proposed | active | shipped | cancelled
  priority: high
  target_ship: null                 # optional: YYYY-MM-DD

repo:
  id: crustyimg

created_at: 2026-07-12
shipped_at: 2026-07-25

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
- [x] STAGE-026 (shipped on 2026-07-20) — **npm-packaged library.** Package the shipped
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
- [x] STAGE-028 (shipped, closed 2026-07-24) — **launch readiness.** The capstone: the README front
  door, honest `BENCHMARKS.md`, and AVIF in the distributed binary. Planned as 2 specs (082 README,
  083 BENCHMARKS); took **4** — SPEC-100 (README CI/Actions + RAW + recipes) and **SPEC-102**
  (AVIF into the default feature set, DEC-081) were both *discovered*, the latter because writing an
  honest benchmark exposed that a `brew install` user could not reproduce the flagship path. The
  Show HN / r/rust **go/no-go is a maintainer decision, not a deliverable** — it moved to the launch
  board (`docs/launch-readiness.md`) rather than holding the stage open.
- [x] STAGE-029 (shipped, closed 2026-07-24) — **demo launch quality.** The demo went from
  mis-serving its most common visitor (12 MP photo → ~33 s, a lossless default that made photos
  *bigger*) to an intent-led flow that picks the format, shows a measured SSIMULACRA2 score, and
  opens the maintainer's own RAW files. **9 specs against a plan of ~4** (079, 080, 081, 095, 096,
  101, 103, 104, 105) — every spec after 096 came from the maintainer *using* the live demo, not
  from a gate. SPEC-105 was an **engine** defect (the shared classifier mis-encoding photographs as
  13× oversized lossless) that the browser demo surfaced.
- [x] STAGE-030 (shipped, closed 2026-07-20) — **command taxonomy & CLI-quality freeze.** A hard
  pre-launch cutover from ~20 verbs to ~14 one-intent verbs, no aliases, no deprecation; `web`
  shipped as the measured flagship (98% / 2.7 s vs the old `optimize` default's 24% / 16.5 s);
  `optimize` demoted to an honest byte-primitive; the `meta` group made whole. 9 planned specs
  (084–091, 093) + SPEC-094 as a follow-up; SPEC-092 was optional from the start and deferred to
  STAGE-032. Two of the nine were LIVE bugs the freeze flushed out (093 metadata byte-order
  corruption, 094 empty-OBU abort).
- [x] STAGE-031 (shipped 2026-07-26; 3 specs shipped 2026-07-19) — **engineering quality and code
  health.** **NOT PROJ-008 thesis work** — it is post-audit code hygiene that happened to run during
  this wave (SPEC-097 `src/cli/mod.rs` 6,483 → 1,426 lines byte-identically; SPEC-098/099 the
  dependency pinning decision record and its correction, DEC-078/079). Closed in place on 2026-07-26:
  all three adopted audit items had shipped, and its one unframed follow-up (strict-JSON
  `escape_json`) moved to **PROJ-010 STAGE-036**, the continuation.
- [→] STAGE-032 → **PROJ-010 STAGE-037** (re-homed 2026-07-26) — **post-launch CLI surface.**
  Additive conveniences STAGE-030 deliberately deferred (SPEC-092 `convert --to` + social/archive
  recipes). **NOT PROJ-008 thesis work.**
- [→] STAGE-033 → **PROJ-010 STAGE-038** (re-homed 2026-07-26) — **post-launch polish and repo
  housekeeping.** Shell completions (SPEC-106) plus six repo-tooling chores. **NOT PROJ-008 thesis
  work.** The hostile/edge input confirmation pass (**SPEC-107 — launch-gating**) went to **PROJ-010
  STAGE-035** instead of travelling with the rest of the stage.

**Count:** **7 shipped / 0 active / 0 open; 2 re-homed to PROJ-010.** The project thesis —
wasm core (025) → npm library (026) → demo page (027) → launch readiness (028) → demo launch
quality (029) → CLI surface freeze (030) — is **complete and live**. STAGE-031/032/033 were code
health, additive CLI surface, and housekeeping: real work, but not this wave's thesis, and they never
held it open. The Show HN / r/rust go/no-go likewise does not hold it open — same call already
made for STAGE-028 — it is a maintainer decision on `docs/launch-readiness.md`.

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

*Shipped 2026-07-25.*

### Did we deliver the outcome in "What This Project Is"?

**Yes — all three artifacts are live, and the engine behind them is the same one the CLI ships.**

- The pure core compiles to `wasm32-unknown-unknown` and runs a full decode → transform → encode
  round-trip in a browser with **no backend** (STAGE-025, DEC-064). The headline turned out to be
  the *asymmetry*: AVIF **encode** runs on wasm (DEC-065) while decode is deferred to the browser's
  own `createImageBitmap` — proven by decoding wasm-produced bytes with two independent decoders,
  not by sniffing magic bytes.
- **`crustyimg-wasm` is published on npm** — installable, typed, zero dependencies, no native
  addon, no postinstall script, running client-side in the browser and in Node (STAGE-026,
  DEC-067). The "sharp without the native addon" pitch is real and stated with its edges
  (`init()` on `--target web`, single-threaded/blocking, AVIF encode-only).
- **The demo page is live** at https://jysf.github.io/crustyimg/ — drop → convert client-side →
  download, AVIF both directions off the main thread, `.avif` and camera **RAW** inputs opened
  page-side, an honest explain readout and a **measured SSIMULACRA2 score** (STAGE-027 + 029).
  Driven clean in Chrome, Firefox and Safari with separate per-engine drivers and a frozen-thread
  negative control.
- First-load bundle landed at **1.33 MB brotli** (1.52 → 1.33 by ablation, DEC-066), with every
  capability-losing lever refused *with data*.
- The no-regression clause held: `just deny` unchanged, no service, no CDN, no backend. The one
  deliberate native-path change was the opposite direction — SPEC-102 moved **AVIF encode into the
  default feature set** (DEC-081) so a `brew install` user gets the flagship path.

**0.6.0 is live** (tag `v0.6.0`, crates.io + Homebrew + GitHub Releases) with `crustyimg-wasm` on
npm alongside it.

### ⚠ This wave did NOT close clean: a live, launch-gating classifier regression is open

**Classification runs *after* the resize pipeline** (`src/cli/optimize.rs:989` → `:1013`), so
`--max` chooses the content class — and `web` downscales to 2048 by default. DEC-047's calibration
was measured at **native** size, so it does not describe the path most users take.

Re-derived and measured against a from-scratch release build of `main` @ `b71c96b` (see
`docs/research/pr113-classifier-review-findings.md`, "Re-derivation (2026-07-25)"):

> A 3000×2250 1-bit halftone passes through **untouched at native size** (entropy 0.62 →
> `document`, 45,527 B). Through a bare `crustyimg web file.png` it becomes a **lossy AVIF of
> 844,492 B — 18.5× larger than its input — at SSIMULACRA2 69.2** (entropy 5.29 → `photograph`).
> At `--max 2560` it reaches 1,590,638 B (35×). The class flips purely because `web` resized it
> first.

Blast radius is **dithered / halftoned graphics**, not screenshots: four substituted 4K–6K
screenshots showed the same monotonic entropy rise under downscale but topped out at 1.14–3.35,
well under the 4.0 threshold. The committed fixture `tests/fixtures/classify/dithered_graphic.png`
reproduces the promotion exactly at `--max 256`. Two related findings reproduce cleanly: the
headline calibration guard is a **tautology** (it stays green with `PHOTO_ENTROPY_STRONG` moved to
5.5, which reinstates the original bug), and the `Icon` rule silently masks the entropy rule below
128 px.

**This is ENGINE work, not wasm work. It belongs to the NEXT project, not to PROJ-008.** It is
recorded here so the closeout is not read as a clean close: the wave shipped a live defect on the
flagship verb, and the browser demo — built as a marketing artifact — is what surfaced the
classifier class of bug in the first place.

### How many stages did it actually take?

**6 shipped against a plan of 3** (025 wasm core, 026 npm library, 027 demo page — plus 028 launch
readiness, 029 demo launch quality, 030 CLI surface freeze), across **33 specs** (SPEC-072..105,
less the deferred SPEC-092) and **17 decisions** (DEC-064..DEC-082, less 072/073).

The three unplanned stages are the finding. 028 and 029 were appended because *shipping to real
users is not the same as building the artifact*: 028 because a launch needs a front door and honest
numbers, 029 because the maintainer using the live demo generated four more specs after the stage
was content-complete. 030 (the ~20 → ~14 verb freeze) is arguably not this wave's thesis at all —
it was pulled in because the demo and the CLI had to present the same Auto path, and a surface you
launch on cannot be renamed afterwards.

Three further stages (031 code health, 032 post-launch CLI surface, 033 polish + housekeeping) were
framed during this wave and are explicitly **not** thesis work — see "carried forward" below.

### What it cost

Read from the specs' `cost.sessions` bookkeeping (`projects/PROJ-008-wasm-core-and-demo/specs/done/`):

| Figure | Value |
|---|---|
| Specs | 33 |
| Cost sessions recorded | 129 |
| Tokens (metered sessions only) | **≈ 27.6 M** |
| Estimated spend | **≈ \$283** |
| Sessions with `tokens_total: null` | 41 (orchestrator main-loop design/ship, per AGENTS §4) |
| Elapsed | 2026-07-12 → 2026-07-25 (13 days) |

Caveats, stated rather than smoothed over: `estimated_usd` is an order-of-magnitude list-rate
estimate (no cache discount), several build/verify token counts are labelled ESTIMATE rather than a
finalized `subagent_tokens`, SPEC-105 carries `tokens_total: null` with a recorded \$21.8, and
SPEC-087 never got a `cost.totals` block (its sessions are present and are included above).
`duration_minutes` is recorded almost nowhere in this project, so **there is no wall-clock figure to
report** — I am not going to invent one.

Cost concentrated hard in a few places, and not where the thesis was: **SPEC-083 (honest
benchmarks) alone was \$40.5 across 12 sessions** — the most expensive spec in repo history and a
*documentation* spec. SPEC-102 (\$19.4), SPEC-088 (\$19.8), SPEC-097 (\$17.5), SPEC-090 (\$14.7)
follow. The wasm core itself — the load-bearing technical unknown, SPEC-072/073/074 — cost about
\$12.2 combined. **The engine port was the cheap part; being honest in public was the expensive
part.**

### What changed between framing and shipping

1. **The risk register was wrong about which risk would bite.** The brief's top risk was binary
   size; it resolved early and cheaply (1.33 MB brotli, one ablation spec). The threads/SIMD risk
   resolved into "run it in a Web Worker." What actually consumed the wave was **quality and
   honesty** — a demo that mis-served a 12 MP photo, benchmarks that could not honestly describe
   the flagship path, and a shared classifier that mis-encoded real photographs.
2. **The demo became a test harness.** It was framed as a thin marketing artifact. It turned out to
   be the repo's best instrument for the *shared* classifier — SPEC-105 was a native-CLI engine
   defect that a browser page surfaced, and the regression above came out of reviewing that fix.
   That is not what it was built for and is the most valuable unplanned outcome of the wave.
3. **AVIF moved onto the native default path.** Framed as "wasm-only concessions never touch
   native", the wave ended up making a deliberate native change in the *other* direction (DEC-081),
   because a benchmark you cannot reproduce with `brew install` is not an honest benchmark.
4. **The launch itself was decoupled from the work.** The Show HN / r/rust go/no-go was originally
   inside STAGE-028's success criteria. It is a maintainer decision on human-hardware and timing
   grounds, not a deliverable a stage can produce, so it lives on `docs/launch-readiness.md`. The
   same call is made here at project level, consistently.

### Lessons that should update AGENTS.md, templates, or constraints

- **An engine or shared-classifier change requires a CLEAN full-matrix verify, re-run by the
  orchestrator.** SPEC-105 reported the feature matrix green on incrementally-compiled artifacts; a
  clean CI build caught real no-AVIF-leg breakage and cost about a day
  ([[a-stale-incremental-build-is-a-false-green]]). This belongs in AGENTS §15 verify guidance as a
  hard rule, together with: never relay a sub-agent's "CLEAN", never push to `main` between verify
  and merge, and check `git status` for a sub-agent's uncommitted work.
- **A calibration constant needs a guard that FAILS when the constant moves.** The classifier's
  headline calibration test stays green with `PHOTO_ENTROPY_STRONG` at 5.5 — the value that
  reinstates the bug it was written to prevent. Any spec that ships a tuned threshold should be
  required to demonstrate the mutation failing, i.e. a negative control on the constant itself.
  This is the strongest concrete case yet for [[a-plausible-test-result-is-not-a-checked-one]] and
  the one worth mechanizing.
- **A calibration measured on one path does not describe another path.** DEC-047 was measured at
  native size; the default verb resizes first. Constraint-worthy: a decision that states a
  numeric threshold must state **the pipeline position and input scale it was measured at**, and a
  spec that changes pipeline order must re-check every threshold downstream of it.
- **Documentation work that keeps generating code work is a product signal, not a scoping failure**
  (STAGE-028: BENCHMARKS.md forced SPEC-102). Worth saying in the templates so a docs spec that
  overruns isn't treated as mis-scoped.
- **A stage held open "deliberately" drifts.** STAGE-029 was content-complete on 2026-07-18 and
  stayed active eight more days, accumulating four specs and a duplicate backlog entry. Prefer
  closing and opening a successor over holding a stage open as a catch-all — and note that this
  project repeated the same shape at project level, which is exactly why it is being closed now
  with three stages carried forward rather than absorbed.
- **A doc that gates a decision rots faster than the work it tracks.** `docs/launch-readiness.md`
  sat at its 2026-07-13 snapshot while five of its blockers shipped. Closing a spec that clears a
  checklist item must include ticking it.
- **On wasm a panic aborts the module and crashes the page** — "typed error, never panic" is a hard
  rule there. Worth a wasm framing in the `untrusted-input-hardening` constraint.
- Already banked as standing memories from this wave, no further action:
  [[verify-wasm-output-with-an-independent-decoder]],
  [[assert-the-build-profile-structurally-not-by-size]],
  [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]],
  [[never-drive-the-maintainers-live-browser]], [[documentation-has-no-green]],
  [[a-number-from-an-unproven-path-is-not-a-measurement]],
  [[a-self-referential-control-cannot-detect-a-broken-pipeline]],
  [[a-guards-advertised-reach-is-a-claim]], [[mechanical-sweeps-need-a-mechanical-check]].

### What is carried forward to the next project

**Resolved 2026-07-26 — the maintainer framed PROJ-010 and re-homed these.** They were **not PROJ-008
thesis work** and never held this project open. What happened to each, and why the three were not
treated alike:

- **STAGE-031 — engineering quality and code health.** **Stayed here, and is now `shipped`.** All
  three adopted audit items shipped during this wave (097/098/099, PRs #103/#102/#104, DEC-078/079)
  and those spec files live in this project's `specs/done/`. Moving the stage would have relocated
  PROJ-008's shipped work and PR provenance into a project that has not started. Its one unframed
  follow-up (strict-JSON `escape_json`) went to **PROJ-010 STAGE-036**, the continuation, which also
  inherits the shelved-directive record (D1/D2/D3/D5/D6, do not re-raise) and the byte-identical
  pre-change-oracle gate.
- **STAGE-032 → PROJ-010 STAGE-037.** Re-homed by `git mv`; content unchanged. SPEC-092
  (`convert --to` + social/archive recipes), additive only. No spec had shipped under the old number.
- **STAGE-033 → PROJ-010 STAGE-038.** Re-homed by `git mv`. SPEC-106 (shell completions: nothing
  installs them, zero `ValueHint` in `src/`, a pre-freeze script silently stops completing `web`)
  plus six repo-tooling chores (duplicate CI matrix, DCO sign-off hook, release-size baseline,
  `wasm-size` banner, `lifetime-report` port, `activity:` front-matter field). **One change on the
  move:** SPEC-107 (hostile / edge input confirmation pass — LAUNCH-GATING) left for **PROJ-010
  STAGE-035**, a launch-gating stage of its own, which resolves structurally the caveat STAGE-033
  carried about holding a launch gate inside a post-launch stage.

**Also carried forward, engine work with no stage yet — resolved 2026-07-26:**

- **The classifier regression above** — classification placement / scale-aware entropy, plus an
  evidence-integrity pass (commit the boundary specimens DEC-047 cites but the repo does not
  contain; re-establish each diluted guard with a negative control; correct DEC-047's two false
  claims). Launch-gating. It likely **subsumes** the queued "scale-normalize the flat/edge
  detector" item rather than layering on it. **Now PROJ-010 STAGE-034**, as SPEC-108 (the fix)
  and SPEC-109 (evidence integrity).
- **`web` returns larger-than-source output on the *lossless* path too**, for ordinary screenshots,
  with no misclassification involved (a 4K spreadsheet screenshot: 420,717 B → 567,140 B at the
  default `--max 2048`). It is disclosed (`larger_than_source: true` + help text), not hidden — but
  it is a poor default to lead a launch post with, and it is independent of the classifier.
- Non-blocking SPEC-091 follow-ups: report the `re_rav1d` `DisjointMut` race upstream (maintainer
  files); `par_iter run_pixel_op` to reclaim serial decode throughput. Encoder threading (probe
  first) and an LLM-free benchmark refresh are the queued post-launch build items.
- EXIF-through-RAW-preview; the mobile RAW on-device test (maintainer hardware, launch board).

**Maintainer-owned, deliberately not a project deliverable:** the Show HN / r/rust go/no-go and the
on-device mobile test. Both live on `docs/launch-readiness.md`.

---

*Lineage: this instantiates the roadmap's provisional "PROJ-008 (WASM seam)" as the
concrete WASM-core + npm-library + demo-page wave (Wave 3). The "stretch" frontier items
once parked under PROJ-008 (AI super-res, face-aware crop, JPEG XL, RAW Tier-2) are NOT
part of this project — they are the 2.0+ "opt-in intelligence / new frontends" roadmap row.
Framed 2026-07-12 immediately after PROJ-007 closed (per AGENTS §2: a project is framed
formally only once the prior one ships).*
