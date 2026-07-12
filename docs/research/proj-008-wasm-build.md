# PROJ-008 — the WASM build: recipe, baseline, and what it cost

**Run date:** 2026-07-12 · **Spec:** SPEC-072 (STAGE-025) · **Decision:** DEC-064
**Machine:** macOS (Darwin 25.5.0), aarch64 · **Toolchain:** rustup stable (not the probe's nightly)

The record SPEC-073 (AVIF-on-wasm) and SPEC-074 (bundle size) argue from. Everything
here was produced by the commands in this document, on the real artifact.

---

## 1. The result, in one line

The pure engine runs a **real decode → transform → encode round-trip inside a wasm VM**:
a PNG plus a resize recipe comes back as PNG bytes that decode to the resized
dimensions, an SVG rasterizes, and an AVIF input fails cleanly instead of aborting the
module. 7/7 `#[wasm_bindgen_test]`s pass under Node. Native builds are unchanged.

**A green wasm compile was never the bar** — "it compiles to wasm32" is the unearned
verdict this project keeps catching. The numbers and claims below all come from driving
the artifact, not from a successful `cargo build`.

---

## 2. Size baseline (the number SPEC-074 tunes)

Release build, after `wasm-opt` (wasm-pack runs it automatically when binaryen is on PATH):

| | size | what it is |
|---|---|---|
| **raw** | **4.29 MB** (4,496,577 B) | the `.wasm` on disk |
| **gzip** (`-9`) | **1.64 MB** (1,716,575 B) | what a typical static host serves |
| **brotli** (`-q 11`) | **1.19 MB** (1,248,818 B) | what a good static host serves |

Reproduce with `just wasm-build` (which ends by printing all three) or `just wasm-size`.

**Read this honestly.** 1.19 MB over the wire is a real cost for a "watch it just work"
demo page — it is seconds on a slow connection before anything happens. It is *not*
absurd for what is linked in (a full codec set + an SVG text-shaping stack + a
perceptual metric), but it is the wave's main technical debt, and it is why SPEC-074
exists. Nothing in SPEC-072 tried to shrink it; the number is a starting line, not an
achievement.

Everything is linked **eagerly** — there is no lazy-loading, no code-splitting, and no
attempt to drop unused codecs. The obvious suspects for SPEC-074, in rough order of
likely payoff:

- **`ssimulacra2`** — pulled in by `optimize`'s perceptual quality search. Large, and
  only needed for one entry point.
- **The `resvg` text stack** (`usvg` → `fontdb`/`rustybuzz` → `ttf-parser`) + the
  bundled Go font, needed only for SVG `<text>`.
- **The `image` codec set** — GIF/BMP/TIFF/ICO are linked in whether or not a demo
  page ever decodes one.
- **`rayon`** was moved out of the wasm build entirely (it has no threads to use on
  bare wasm32), which is already a small win.
- **A separate `crustyimg-core` crate** would let the wasm build depend on the engine
  *only*, instead of `cfg`-ing the shell out of one crate. Cleaner, but a big refactor
  — DEC-064 defers it until a measurement argues for it. This is that measurement's
  home.

---

## 3. The build recipe

Baked into the justfile so it is not folklore: `just wasm-check` (compile),
`just wasm-build` (release `.wasm` + JS bindings → `pkg/`), `just wasm-test` (the
round-trip in Node), `just wasm-size` (the numbers above).

### One-time setup

```bash
rustup toolchain install stable
rustup target add --toolchain stable wasm32-unknown-unknown
brew install wasm-pack binaryen                    # binaryen = wasm-opt
cargo install wasm-bindgen-cli --version 0.2.126   # must MATCH the wasm-bindgen dep
```

### The toolchain gotcha (this is the one that costs an hour)

This machine has **both** a Homebrew rust (`/opt/homebrew/bin/rustc`, which ships **no
wasm std**) and a rustup rust. A bare `cargo build --target wasm32-unknown-unknown`
invokes `rustc` off `PATH`, finds Homebrew's first, and dies with:

```
error[E0463]: can't find crate for `core`
note: the `wasm32-unknown-unknown` target may not be installed
```

That reads like a broken dependency or a missing target. **It is neither — it is the
wrong compiler.** The design probe lost a debug cycle to it. Every `just wasm-*` recipe
therefore resolves the rustup stable toolchain explicitly and pins both `CARGO` and
`RUSTC` to it:

```make
_wasm_bin := `rustup which --toolchain stable rustc | xargs dirname`

wasm-check:
    PATH="{{_wasm_bin}}:$PATH" RUSTC="{{_wasm_bin}}/rustc" \
        "{{_wasm_bin}}/cargo" build --lib --target wasm32-unknown-unknown
```

The spec required **stable**, not the probe's nightly. It works on stable; nothing in
the wasm path needs a nightly feature.

### Why the tests do NOT run under `wasm-pack test`

`wasm-pack test` hardcodes `cargo build --tests`, which builds **every** integration
test for wasm32 — and ~20 of ours drive the CLI **binary** over real files
(`std::process::Command`, `tempfile`, argv). None of that exists in wasm, so the build
fails before a single test runs. Passing `--test wasm_roundtrip` does not help: cargo
unions it with the hardcoded `--tests`.

The alternative was to tattoo `#![cfg(not(target_arch = "wasm32"))]` onto every native
test file — and onto every future one, forever, or the wasm build breaks. Instead,
`.cargo/config.toml` registers `wasm-bindgen-test-runner` as the wasm32 runner, so the
round-trip runs through plain cargo with a real target filter:

```bash
cargo test --target wasm32-unknown-unknown --test wasm_roundtrip
```

which builds **only** that test target. (`wasm-pack` is still used for
`just wasm-build`, where it only builds the lib and is exactly the right tool.)

Two other things had to move for this to work at all:

- **`criterion`** (bench harness) hard-`compile_error!`s on wasm (`"Rayon cannot be
  used when targeting wasi32"`). Cargo builds **every** dev-dependency when it builds
  **any** test target, so a dev-dep that can't reach wasm breaks the wasm test build
  even though no wasm test touches it. Moved (with `tempfile`) to a native-only
  dev-dependency table.
- **`src/main.rs`** calls `crustyimg::cli::run()`, and `cli` is now native-only — so
  the bin target failed to compile for wasm32. It gets an empty `main` on wasm.

---

## 4. What compiles to wasm32, and the one thing that doesn't

Confirmed by the actual build (not the probe): the **entire** dependency tree compiles
to `wasm32-unknown-unknown` **except `re_rav1d`**.

| crate | wasm32 | notes |
|---|---|---|
| `image` (png/jpeg/gif/bmp/tiff/ico/webp) | ✅ | the whole default codec set |
| `resvg` / `usvg` / `tiny-skia` | ✅ | **SVG rasterizes in the browser** — the headline survivor |
| `fast_image_resize` | ✅ | SIMD resize backend |
| `ssimulacra2` | ✅ | the perceptual search really runs in wasm |
| `skrifa`, `zeno` | ✅ | text watermark glyph stack |
| `img-parts`, `kamadak-exif` | ✅ | container/metadata lane |
| `toml`, `serde`, `thiserror` | ✅ | recipes + typed errors |
| `rayon` | ✅ (compiles) | but moved out — bare wasm32 has no threads to use |
| **`re_rav1d`** (AVIF decode) | ❌ | libc POSIX types absent on bare wasm32 (`off_t`, `ptrdiff_t`/`intptr_t`/`uintptr_t`, errno `ENOENT`/`EIO`/`EINVAL`); thread-task const-eval failure (`E0080` in `src/thread_task.rs`) |

So the wasm build's input reach is: **PNG, JPEG, GIF, BMP, TIFF, ICO, WebP, SVG** —
everything the native default build reaches **except AVIF**. (HEIC was already opt-in
and off, DEC-052.) An AVIF input returns a typed
`ImageError::CodecUnavailableOnTarget`, never a panic — which matters more here than on
native, because **a panic in wasm aborts the module** and takes the page's instance with
it.

**For SPEC-073:** the blocker is `re_rav1d` specifically, not AV1 or AVIF as such.
`avif-parse` (the container parser) is only native-only because nothing else uses it —
it was not tested against wasm32 on its own and may well compile. The open questions
SPEC-073 owns: does `rav1e` (AVIF *encode*, already an optional native feature via
`image/avif`) compile to wasm32? Can `re_rav1d`'s libc/thread usage be `cfg`'d out
upstream or in a fork? Is there a third pure-Rust AV1 decoder that targets wasm?

> **Answered by SPEC-073 (§7 below): `rav1e` compiles AND runs — AVIF ENCODE is in the
> wasm build. Decode stays out and is now deliberately deferred (DEC-065), because the
> browser already has an AVIF decoder and we do not need to ship a second one.**

---

## 5. The surface (`src/wasm.rs`)

Thin glue, no new logic — every function is a few lines over the existing engine:

| export | does |
|---|---|
| `transform(bytes, recipe_toml, out_format)` | `Image::from_bytes` → `Recipe::from_toml` → `build_pipeline(OperationRegistry::with_builtins())` → `Pipeline::run` → `sink::encode_to_bytes` |
| `info(bytes)` | width / height / format / hasAlpha |
| `optimize(bytes, out_format)` | `Analysis::compute` → `decide::format_shortlist` → (lossy) `quality::auto_quality` SSIMULACRA2 search → encode |
| `version()` | the crate version, so a demo page can show what it loaded |

The recipe TOML is **the same schema the CLI reads off disk** (DEC-005), resolved
through the **same** operation registry. A recipe tuned in the terminal replays in the
browser because it is literally the same code path — that equivalence is the point of
the whole wave.

**`optimize` is deliberately partial.** It reuses the engine's real parts (analysis
bucket → format shortlist → perceptual quality search) but takes the shortlist's
*first* candidate instead of solving every candidate and running `decide::pick_winner`
over the measured outcomes, which is what native `optimize` does (SPEC-048). It will
pick the same *format* as native and a genuinely searched *quality* for it, but it does
not comparison-shop encodings. Wiring the full multi-candidate solve belongs in a shared
engine seam that both `cli` and `wasm` call — copy-pasting the solve loop into
`wasm.rs` would be exactly the duplication this architecture has avoided so far. Filed
as a follow-up; SPEC-072 explicitly permitted this.

---

## 6. Things that will bite the next person

- **`display` is a default-ON feature whose dep (`viuer`) is native-only.** On wasm the
  feature is *enabled* while the crate is *absent*, so the code is gated on
  `all(feature = "display", not(target_arch = "wasm32"))`. Any future default-ON feature
  with a native-only dep needs the same conjunct or the wasm build breaks. This is the
  sharpest edge DEC-064 leaves behind.
- **`wasm-bindgen` and `wasm-bindgen-cli` must be the same version.** The runner is a
  separate binary; a mismatch is a confusing runtime error, not a build error. Pinned to
  `=0.2.126` on both sides.
- **The AVIF sniff had to move.** `is_avif` used to live in `src/image/avif.rs`, which is
  now native-only — so the sniff moved to `src/image/sniff.rs`, which compiles
  everywhere. Detection in every build, decode only where the codec exists. (HEIC solved
  the same problem the easy way, because `libheif-rs` is behind an off-by-default
  feature; `re_rav1d` is a default dep, so its whole module has to leave the wasm build.)
- **No wasm CI job yet.** `just wasm-test` is the local floor, matching the fuzz-gate
  precedent (DEC-062). Nothing stops a future commit from breaking the wasm build
  silently. A CI job is a stage-level decision, filed as a follow-up.

---

## 7. SPEC-073 addendum — AVIF encode, and what it costs

**Run date:** 2026-07-12 · **Spec:** SPEC-073 · **Decision:** DEC-065 · same machine and
toolchain as above.

### It runs, not just compiles

`rav1e` 0.8.1 + `ravif` 0.13.0 (via `image/avif`) compile to `wasm32-unknown-unknown`
**and execute in a wasm VM**: `transform(png, recipe, "avif")` returns bytes whose
ISOBMFF header carries a `ftyp` box with an `avif` major brand. Asserted by
`transform_png_to_avif_is_valid_avif` under `just wasm-test` (10/10 green) — the output
bytes are sniffed, not merely `Ok`-checked. They cannot be fed back through `info()`,
because **there is still no AVIF decoder in the wasm build** — which is the asymmetry, in
one sentence.

### The size delta (the number SPEC-074 argues from)

Release `.wasm`, post `wasm-opt`. The lean column reproduces §2 exactly.

| | lean (no AVIF) | **shipped (`--features avif`)** | delta |
|---|---|---|---|
| raw | 4,496,577 B (4.29 MB) | 6,415,270 B (6.12 MB) | +1.83 MB (+42.7%) |
| gzip `-9` | 1,716,575 B (1.64 MB) | 2,272,806 B (2.17 MB) | +0.53 MB (+32.4%) |
| **brotli `-q 11`** | **1,248,818 B (1.19 MB)** | **1,594,482 B (1.52 MB)** | **+345,664 B (+27.7%)** |

Reproduce: `just wasm-build` (shipped) and `just --set _wasm_features "" wasm-build`
(lean). **+345 KB over the wire** is the honest figure — brotli is what a real host
serves; the raw +1.83 MB overstates it, because an AV1 encoder compresses well.

**DEC-065 accepts it and ships one artifact.** A lazy "AVIF chunk" is not 345 KB: wasm
modules don't share code, so a second module re-links the whole engine, and the user who
actually converts to AVIF would pay 1.19 + 1.52 = **2.71 MB** instead of 1.52. The
bundle's real problem is the **1.19 MB core** (`ssimulacra2`, the resvg text stack, the
full `image` codec set) — that is SPEC-074's lever, not the headline codec.

### What bites next

- **rav1e runs SERIAL on wasm** — `maybe-rayon` with no threads on bare wasm32. Encoding
  is noticeably slower than the native CLI. STAGE-027 must run it in a Web Worker with a
  progress indication, or the page will feel hung.
- **`optimize(_, "avif")` does not search.** The perceptual search decodes each candidate
  to score it (DEC-019) and there is no decoder — so `src/wasm.rs` guards on
  `supports_perceptual_quality` and encodes once at the default quality (80). AVIF is
  also never auto-picked (`format_shortlist` only offers it in `Mode::SizeBudget`; the
  wasm surface runs `Mode::Perceptual`).
- **The wasm recipes now carry a feature flag** (`_wasm_features := "--features avif"`).
  A CI job or npm packaging script that shells a bare `cargo build --target wasm32-…`
  will silently produce an artifact whose headline call answers "codec not built". Build
  through the recipe.
- **`.avif` INPUTS are the demo's problem now.** The page must read them with
  `createImageBitmap` and hand us pixels; we return a typed error.
