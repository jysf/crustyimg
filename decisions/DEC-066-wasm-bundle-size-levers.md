---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-066
  type: decision
  confidence: 0.90
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
  - "Cargo.toml"
  - "justfile"
  - "tests/wasm_roundtrip.rs"

tags:
  - wasm
  - bundle-size
  - build-profile
  - codecs
---

# DEC-066: WASM bundle size — what we cut, what we paid for, and what we refused to sell

## Decision

The shipped wasm artifact goes from **1,595,028 → 1,394,313 B brotli (−200,715 B, −12.6%)** by
taking three levers that cost nothing (**fat LTO + codegen-units=1 + strip**, a **wasm-target-only
`image` codec trim** dropping TIFF/BMP/ICO decode, and **turning `wasm-opt` OFF**), and by
**refusing three levers that would have sold a capability**: SVG `<text>` (−287,098 B but silently
corrupts every SVG with a label), `opt-level = "z"` (−165 KB but makes the AVIF encoder 2.8×
slower), and the SSIMULACRA2 perceptual search (−23,540 B, the engine's whole differentiator).
Every number here is a real build measured by ablation, not an estimate.

## Context

SPEC-074 closes STAGE-025. The demo bundle was **1.52 MB brotli**, of which ~0.35 MB is `rav1e` —
a KEEP, it is the headline (DEC-065). Wave 3's promise is "zero-install, instant try it", and
1.52 MB is the debt against it.

The design-time twiggy probe found **no whale**: the mass is spread over 5,300+ items. That framing
turned out to be the important one — there was no single fix, so the spec's real deliverable is an
**ablation table** (toggle one lever → `just wasm-build` → diff the brotli) and an honest
keep/drop call on each row. Guessing was explicitly off the table: the probe's own prime suspect
(`ssimulacra2`) turned out to be worth 23 KB, and the biggest *free* win turned out to be a
compiler flag nobody had listed.

**brotli is the metric.** It is what a real static host serves and what the user actually waits
for; raw size is a second-order cost (parse/compile, and browsers stream-compile that during the
download anyway).

## The ablation table

Every row is a real `just wasm-build`, `--features avif`, measured with `just wasm-size`.
Deltas are against the row's stated base, not against each other.

### Levers TAKEN (no capability cost)

| lever | brotli | Δ | what it costs |
|---|---|---|---|
| baseline (SPEC-073 shipped) | 1,595,028 | — | — |
| `lto = "fat"` + `codegen-units = 1` | 1,515,128 | **−79,900** | nothing. Fat LTO alone is worth ~0 (it needs cu=1 to bite). |
| `image` −tiff −bmp −ico (wasm only) | 1,430,801 | **−84,327** | TIFF/BMP/ICO **decode** in the browser (see below) |
| `wasm-opt` OFF | 1,394,313 | **−36,488** | nothing. It buys no speed. (+340 KB raw) |
| `strip = true` | (included above) | **−58,533** | nothing — but only load-bearing *because* wasm-opt is off |
| **shipped total** | **1,394,313** | **−200,715 (−12.6%)** | **1.52 MB → 1.33 MB** |

### Levers REFUSED (each would sell a capability)

| lever | brotli saving | base it was measured against | why we did NOT take it |
|---|---|---|---|
| resvg `text` | **−287,098** (20% of the bundle!) | 1,430,801 → 1,143,703 | SVG `<text>` **silently vanishes** — see below |
| `opt-level = "z"` | −247,936 | 1,515,128 → 1,267,192 | AVIF encode 350 ms → **956 ms** (2.8×) |
| `opt-level = "s"` | −169,552 | 1,515,128 → 1,345,576 | AVIF encode 350 ms → 537 ms (1.5×) |
| drop `ssimulacra2` | −23,540 | 1,430,801 → 1,407,261 | kills the perceptual auto-quality search — the engine's differentiator, for 1.7% of the bundle |

### Levers that did nothing

| lever | result |
|---|---|
| `panic = "abort"` | **0 B** — `wasm32-unknown-unknown` already defaults to it (`rustc --print cfg`) |
| `lto = "fat"` alone (cu=16) | **+1,450 B** — LTO without `codegen-units = 1` is worse than useless |
| `wasm-opt -Oz` vs `-O` | +2,385 B brotli (−34 KB raw) — and both are worse than off |

## Alternatives Considered

- **Option A: drop resvg's `text` feature (−287,098 B, the single biggest lever).**
  - Why rejected: **it does not degrade text, it deletes it.** `usvg` without `text` drops `<text>`
    nodes from the tree entirely. The SVG still rasterizes, still reports the right dimensions, and
    `transform()` still returns `Ok` — with a **hole where the label was**. We built that artifact
    and confirmed the failure is invisible: `info()` cheerfully answered `40x30` on an SVG whose
    text was gone, and the existing `svg_rasterizes_in_wasm` test (which asserts dimensions) stayed
    **green straight through the corruption**. DEC-054 already named text-to-nothing a
    "silent-wrong-output footgun"; 287 KB does not buy the right to ship one. And SVGs *with* text
    are most real SVGs (logos, charts, diagrams) — a demo that eats your labels is worse than a
    demo that loads a beat slower. `svg_text_renders_glyphs_in_wasm` now makes an accidental drop
    fail LOUD. **Revisit at STAGE-026**, where a lazy chunk could offer the text stack on demand;
    the price tag is now known.

- **Option B: `opt-level = "z"` (−247,936 B) or `"s"` (−169,552 B) for the wasm profile.**
  - Why rejected: it is not free — it is **paid for in encode speed**, on the exact path that is
    the demo's headline (DEC-065). Measured through the real artifact on a 512×384 photo: AVIF
    encode 350 ms → **956 ms** at `z`, 537 ms at `s`; the SSIMULACRA2 search 719 ms → 1132 ms.
    Scale that to a real photo a user drags in and `z` turns a slow conversion into a broken one —
    and rav1e is *already* serial on wasm (no threads), the known STAGE-027 risk.
  - The autopsy is the part worth keeping: rav1e is **generic over its pixel type**, so its encoder
    monomorphizes into **`ravif`**, not `rav1e` — pinning `[profile.release.package.rav1e]` does
    almost nothing, which is exactly the wrong-crate rabbit hole this note exists to save you from.
    At the shipped feature set, a global `z` with the hot crates (rav1e, ssimulacra2, image,
    fast_image_resize) pinned back to `opt-level = 3` is worth **−164,933 B** and still encodes at
    700 ms. Add **`ravif`** to that pin list and the speed returns *exactly* (348 ms ≈ the 350 ms
    baseline) — while **161,056 of those 164,933 B come straight back**. The size win *was* the
    slowdown, the same bytes viewed twice. With the encoder protected, `z` nets **3,877 B**. There
    is no clever middle here: you either sell the encoder's speed or you don't. We don't — a
    one-time 165 KB download does not justify seconds on every conversion — and the per-package
    override table earns nothing, so it is not in the tree.

- **Option C: keep `wasm-opt` on (the conventional choice).**
  - Why rejected: measured, it is a **raw-size tool, not a wire-size tool**. It restructures LLVM's
    very regular output (merging, folding), which removes 340 KB of raw bytes *and the redundancy
    the compressor was eating*: **−340 KB raw, +36 KB on the wire.** The penalty holds at every
    setting we tried (brotli q4/q5/q9/q11 **and** gzip -9: +17 KB…+64 KB), so it is not an artifact
    of one compression level. And it buys **no speed** — through the real artifact, the module is a
    hair *faster* without it (AVIF 343 ms vs 350 ms; search 633 ms vs 719 ms). It costs download
    and returns nothing the user can feel.
  - **The trap it hides.** wasm-pack invokes `wasm-opt` allowing an older feature set than rustc
    now emits. Under `opt-level = "z"` (where LLVM starts emitting `memory.copy` and
    `i32.trunc_sat_*`) it dies in validation with **thousands of `[wasm-validator error]` lines**.
    The working invocation is recorded in `Cargo.toml` for whoever turns it back on: Rust's wasm32
    baseline (`--enable-bulk-memory`, `-bulk-memory-opt`, `-nontrapping-float-to-int`, `-sign-ext`,
    `-mutable-globals`, `-reference-types`, `-multivalue`) **plus `--enable-simd`**, which
    `fast_image_resize` emits. Not `-all`, which would also permit GC/threads/exceptions.
  - **Corrected at verify (2026-07-12).** The build recorded this failure as *silent* — "wasm-pack
    swallows it and exits 0", casting doubt on SPEC-072/073's numbers. **That is wrong, and the
    correction matters more than the original claim.** Re-driven on wasm-pack 0.15.0 / binaryen 130
    across four invocation shapes (wasm-pack's default, `wasm-opt = true`, a flag list without
    `--enable-simd`, and the full list), the failure is **LOUD every time**: `Error: failed to
    execute 'wasm-opt': exited with exit status: 1`, and the recipe aborts **exit 1**. And under the
    config SPEC-072/073 actually shipped (`opt-level = 3`, thin LTO), `wasm-opt` **validates clean
    and really runs** — it strips 1.6 MB of raw (8,015,811 → 6,414,690 B), so **their "post
    wasm-opt" numbers were genuinely post-wasm-opt.** The baseline this DEC diffs against is sound.
    What *is* true, and is the durable hazard: on failure wasm-pack leaves the **un-optimized module
    in `pkg/`**, so a caller that reads an exit code through a pipe (`just wasm-build | tail` — the
    likely origin of the "exit 0") finds a plausible `pkg/` and a size number no optimizer touched.
    **Check the raw size moved, not the exit code.**

- **Option D (chosen): the three free levers + the codec trim; refuse the three that cost.**
  - Why selected: it is the honest floor. Every byte we removed is a byte no user misses, and every
    byte we kept, we can now name a price for.

## Consequences

- **Positive:** 1.52 MB → **1.33 MB brotli** (−12.6%) with **zero** loss of the demo's
  capabilities — SVG text, the AVIF encoder's speed, and the perceptual search all survive intact.
  The `wasm-opt` silent-failure trap is documented rather than lurking. The two capability levers we
  declined are now *priced*, so STAGE-026 can revisit them against a real packaging seam instead of
  a hunch.
- **Negative:** **TIFF, BMP and ICO no longer decode in the browser build.** They return the same
  typed `ImageError` an unsupported format always did — no panic, no silent garbage — and the
  native CLI still reads all three. This is the one capability sold, for 84 KB, on the judgment that
  a file picker is fed PNG/JPEG/GIF/WebP/SVG (+AVIF via the browser's own decoder), not scanner
  TIFFs and favicons. If a demo user ever needs it, the lever reverses in one line.
- **Negative:** the wasm build now depends on **being built through `just wasm-build`**. The size
  profile lives in `CARGO_PROFILE_RELEASE_*` env vars in the recipe, because `[profile.release]` is
  shared with the native release build (and `[profile.dist]` inherits it) and DEC-064 requires the
  native binary to stay byte-identical — cargo has no per-target profiles. A bare `cargo build
  --target wasm32-…` silently produces a heavier artifact. This compounds the same hazard DEC-065
  already noted for `--features avif`. **A CI job that builds the wasm artifact must call the
  recipe**, and that job is still unwritten (a live STAGE-026 follow-up).
- **Neutral:** `image` and `resvg` are now declared **once per target table** rather than once in
  `[dependencies]`, because cargo features are additive — you cannot subtract a feature for one
  target. `Cargo.lock` is unchanged, and a paired test asserts opposite verdicts on the same three
  fixtures (native decodes them, wasm refuses them), so a "cleanup" of the duplicated lines cannot
  quietly take the native codecs down with it.

## Revisit if

- **STAGE-026 packaging lands a lazy-chunk or multi-artifact seam.** Then the 287 KB SVG-text stack
  and the tiff/bmp/ico decoders become opt-in downloads rather than a keep/drop, and both refusals
  above should be re-decided with that option on the table.
- **A future binaryen/wasm-pack makes `wasm-opt` compression-neutral.** It would then be a free
  −340 KB raw. Re-measure with the flags recorded in `Cargo.toml`; do not trust an exit code.
- **rav1e gains wasm threads** (`wasm-bindgen-rayon` + COOP/COEP). If the AVIF encode stops being
  the critical path, `opt-level = "z"`'s 165 KB comes back into play.
- **The crustyimg-core crate split** (deferred by DEC-064) — still not a size lever on this
  evidence: nothing in the table was CLI code the linker failed to drop.
