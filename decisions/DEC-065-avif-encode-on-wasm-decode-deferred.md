---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-065
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-12
supersedes: null
superseded_by: null

affected_scope:
  - src/wasm.rs
  - src/quality/mod.rs
  - justfile
  - tests/wasm_roundtrip.rs
  - docs/research/proj-008-wasm-build.md

tags:
  - wasm
  - avif
  - bundle-size
  - codecs
  - build-targets
---

# DEC-065: AVIF on wasm is ENCODE-only — and the encoder ships in the default artifact

## Decision

Three things, together:

1. **AVIF encode is IN the wasm build.** `rav1e` 0.8.1 / `ravif` 0.13.0 (via the
   `image/avif` feature) compile to `wasm32-unknown-unknown` and *run*: a PNG goes in,
   a valid `.avif` comes out, inside a wasm VM. `transform(bytes, recipe, "avif")` and
   `optimize(bytes, "avif")` are supported wasm surface calls.

2. **AVIF decode stays OUT, deferred.** `re_rav1d` still does not compile to bare
   wasm32 (DEC-064), and nothing here changes that. An AVIF *input* to the wasm surface
   still returns the typed `ImageError::CodecUnavailableOnTarget` — never a panic.
   Reading `.avif` files in the demo is the browser's job (`createImageBitmap`), a
   STAGE-027 concern, not an in-wasm capability. **No port, fork, or `wasm32-wasi`
   attempt is scheduled.**

3. **The shipped wasm artifact is built `--features avif`** — one artifact, encoder
   included, **1.52 MB brotli** (up from 1.19 MB). No separate AVIF bundle, no lazy
   chunk. The `just wasm-*` recipes carry the flag (`_wasm_features`), and a lean
   comparison build stays one override away
   (`just --set _wasm_features "" wasm-build`).

## Context

STAGE-025 was organized around one open question: **what is the AVIF story in the
browser?** SPEC-072 shipped the wasm seam with AVIF entirely absent and deferred the
question here, with a size baseline to argue from. A SPEC-073 design-time probe showed
the encode half *compiles*; this spec proved it *runs* and measured what it costs.

### The measurement (the decisive number)

Release `.wasm`, post `wasm-opt`, macOS aarch64, rustup stable — `just wasm-build`:

| | no AVIF (SPEC-072 baseline) | **with `--features avif`** | delta |
|---|---|---|---|
| raw | 4,496,577 B (4.29 MB) | 6,415,270 B (6.12 MB) | **+1.83 MB (+42.7%)** |
| gzip `-9` | 1,716,575 B (1.64 MB) | 2,272,806 B (2.17 MB) | **+0.53 MB (+32.4%)** |
| **brotli `-q 11`** | **1,248,818 B (1.19 MB)** | **1,594,482 B (1.52 MB)** | **+345,664 B (+27.7%)** |

**+345 KB over the wire** is the honest cost — brotli is what a real static host serves,
so it is the number a user waits for. Not the +1.83 MB the raw figure suggests: an AV1
encoder compresses well.

## Rationale

### Why encode is in, and decode is not

They are **different codecs**, not two halves of one. Encode is `rav1e` (pure Rust,
wasm-clean); decode is `re_rav1d`, which needs libc POSIX types bare wasm32 lacks
(`off_t`, `ptrdiff_t`, errno) and fails const-eval in its thread-task module. There is
no flag that reconciles them. So the asymmetry is not a compromise we chose — it is the
shape of the upstream ecosystem, and the decision is only about what to *do* with it.

What makes decode's absence survivable is that **the browser already has an AVIF
decoder**. Every current browser decodes AVIF natively via `createImageBitmap` / `<img>`.
A demo page that needs to *read* an `.avif` can hand it to the platform and pass us the
pixels. Porting a second AV1 decoder into our `.wasm` to duplicate a decoder already
sitting in the host is the wrong trade at any size — and it is why "restore AVIF decode
on wasm" is **deferred, not scheduled**.

Encode has no such escape hatch: no browser API *writes* AVIF. If we do not ship
`rav1e`, "drop a PNG, get a tiny AVIF, no server" — the demo's headline, and the thing
that makes the wave worth doing — is simply not possible. That asymmetry in *who else
can do it* is the whole argument.

### Why one artifact and not a size-managed split

The obvious frugal move is to keep the 1.19 MB core and lazy-load an AVIF chunk. **It
does not work here, and it is worth being precise about why:** a `.wasm` module is not a
JS bundle. Two artifacts do not *share* the engine — each links its own copy of `image`,
`resvg`, `fast_image_resize`, `ssimulacra2`. So an "AVIF chunk" is not 345 KB of rav1e;
it is a second **1.52 MB** module. The user who actually converts to AVIF — the user the
demo exists for — would download **1.19 + 1.52 = 2.71 MB** instead of 1.52 MB. The split
optimizes for the visitor who never uses the headline feature, at the expense of the one
who does.

Real code-splitting across wasm modules (shared-memory dynamic linking, or a
`crustyimg-core` module both link against) is a genuine engineering project. It is also
exactly the kind of restructuring **SPEC-074** owns. Doing a half-version of it here, to
save 345 KB on a bundle whose *real* problem is the 1.19 MB core, would be optimizing
the wrong end.

**The core is the problem, not rav1e.** 1.19 MB of the 1.52 MB is engine — `ssimulacra2`,
the `resvg` text stack, the full `image` codec set, all linked eagerly. Gating the
headline feature to protect a bundle that is *already* over budget is a rounding-error
saving bought with a capability. SPEC-074's lever is the core; this spec's job was to
hand it the number, and the number says rav1e is 23% of the artifact and the least of
its worries.

This is a decision to **accept a cost, not to ignore one**: 1.52 MB brotli is heavy for
a demo page, it is written down here, and SPEC-074 exists to attack it.

### Why the feature flag stayed, rather than making AVIF unconditional on wasm

DEC-064's principle is "the target is unforgeable — no flag to remember". The purist
reading says: add `image = { features = ["avif"] }` to the wasm dep table and change
every `#[cfg(feature = "avif")]` to `#[cfg(any(feature = "avif", target_arch = "wasm32"))]`,
so a bare `cargo build --target wasm32-…` gets the encoder automatically.

Rejected, for two reasons. First, it smears the meaning of the `avif` cfg across ~8 call
sites in `sink` and `quality` — and a gate whose meaning differs by target is exactly the
sharp edge DEC-064 already regrets having one of (`display`). Second, and decisively, it
**welds rav1e to the wasm target permanently**: a lean, no-AVIF artifact would no longer
be buildable, and that artifact is precisely the comparison SPEC-074 needs. Keeping the
cargo feature as the one gate keeps AVIF a **knob** — which is what a size-managed path
would need if SPEC-074's numbers ever argue for one. The flag lives in the justfile
(`_wasm_features`), so no human has to remember it; the artifact is an AVIF artifact by
recipe, not by luck.

### Why `optimize(_, "avif")` does not run the perceptual search

The perceptual quality search encodes a candidate and **decodes it back** to score the
round-trip with SSIMULACRA2 (DEC-019) — it needs a decoder. AVIF on wasm has an encoder
and no decoder, so the search cannot run; asking for it would fail on the first
candidate's decode. `src/wasm.rs::optimize` therefore guards on
`LossyFormat::supports_perceptual_quality` (the same seam the CLI guards on) and, for
AVIF, encodes once at the encoder's default quality (80). Native behaves the same way for
the same reason (`supports_perceptual_quality` already excludes AVIF), so the surfaces
agree. AVIF is also never *auto*-picked by `optimize(_, "auto")`: `decide::format_shortlist`
only shortlists AVIF in `Mode::SizeBudget`, and the wasm surface runs `Mode::Perceptual`.
**AVIF on wasm is an explicit choice by the caller, at a fixed default quality.**

## Consequences

**Good**

- **The demo's headline is real.** PNG/JPEG/SVG → AVIF, client-side, no backend —
  driven in a real wasm VM by `transform_png_to_avif_is_valid_avif` (the output bytes
  are asserted to be a real `ftyp`/`avif`-branded file, not merely `Ok`).
- The AVIF question STAGE-025 was organized around is **closed with a measurement**, not
  a guess. SPEC-074 inherits a concrete target: 1.52 MB brotli, of which ~1.19 MB is
  engine and ~0.35 MB is rav1e.
- **`just deny` stays green with no new exception** — `rav1e`/`ravif`/`av1-grain`/
  `av-scenechange` are permissive pure Rust, so `pure-rust-codecs-default` (DEC-004)
  holds on the wasm path too. No nasm, no C.
- The native feature matrix is **untouched** (no `Cargo.toml` dependency change at all):
  `avif` remains an off-by-default native feature, and the released native binary is
  byte-identical.

**Costs / risks**

- **The demo bundle is 1.52 MB brotli.** On a slow connection that is seconds before
  anything happens. Accepted, recorded, and owned by SPEC-074 — not waved away.
- **rav1e is SLOW on wasm.** It uses `maybe-rayon`, and bare wasm32 has no threads, so
  the encoder runs **serial** — noticeably slower than the native CLI. The demo page
  (STAGE-027) must therefore treat AVIF encode as a *progress-bar* operation, and should
  keep it off the main thread (a Web Worker). Nothing in this decision makes that easy;
  it just makes it necessary.
- **The `.avif`-input gap is now the demo's job.** A user who drops an `.avif` on the
  page gets a typed error from us. STAGE-027 must decode it with `createImageBitmap` and
  hand us pixels, or show a clear message. If it does neither, the page looks broken.
- **`just wasm-*` now carries a feature flag.** Anyone building the artifact outside the
  recipes (a CI job, an npm packaging script in STAGE-026) must pass `--features avif`
  or they will silently ship an artifact whose headline call returns "codec not built".
  A wasm CI job (already a SPEC-072 follow-up) should build the *recipe*, not a bare
  cargo line.

## Alternatives considered

| Option | Why not |
|---|---|
| Ship the wasm build without AVIF encode (opt-in flag, off) | Kills the demo's headline to save 345 KB on a bundle that is 1.19 MB regardless. No browser API writes AVIF, so if we don't ship rav1e, nobody can. |
| Two artifacts: lean core + lazy AVIF module | Wasm modules don't share code — the "chunk" is a second full 1.52 MB engine. The AVIF user pays 2.71 MB instead of 1.52 MB. Optimizes for the visitor who doesn't use the feature. |
| Make AVIF unconditional on wasm (`any(feature, target_arch)` gates + wasm dep table) | Smears the `avif` cfg's meaning across ~8 sites, and welds rav1e to the target — deleting the lean comparison build SPEC-074 needs. The justfile remembers the flag so humans don't have to. |
| Port / fork `re_rav1d` for wasm32 (restore decode) | A codec port is a project, not a spec — and the browser **already has** an AVIF decoder. Duplicating it inside our module is the wrong trade at any size. |
| `wasm32-wasi` to get the POSIX types `re_rav1d` wants | Not a browser target. It would buy decode by giving up the demo. |
| Run the perceptual search for AVIF on wasm | It decodes each candidate to score it; there is no AVIF decoder here. It would fail on candidate #1. |

## Revisit when

- **SPEC-074 lands its bundle cuts.** If the core drops sharply (say below ~500 KB
  brotli), rav1e's 345 KB becomes a *large* share of a small artifact, and a real
  two-module split (shared core, AVIF as a linked module) may finally pay. The lean
  build (`just --set _wasm_features "" wasm-build`) is kept alive for exactly that
  measurement.
- **A pure-Rust AV1 decoder targets wasm32** (`re_rav1d` `cfg`s out its libc/thread
  usage, or `dav1d-rs` gains a wasm path). Then reconsider decode — but weigh it against
  `createImageBitmap`, which will still be free.
- **wasm threads become viable for us** (`wasm-bindgen-rayon` + COOP/COEP headers on the
  demo host). rav1e would get its parallelism back, which changes the *runtime* story,
  not the size one. Note COOP/COEP is a hosting constraint STAGE-027 would inherit.
</content>
</invoke>
