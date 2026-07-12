---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-072
  type: story
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-008
  stage: STAGE-025
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # separate build session
  created_at: 2026-07-12

references:
  decisions:
    - DEC-004    # pure-Rust default (wasm-bindgen is pure-Rust, permissive)
    - DEC-053    # AVIF decode via re_rav1d — the crate that blocks wasm32
    - DEC-054    # SVG via resvg — compiles to wasm32 (the probe confirmed)
    - DEC-027    # feature-gating precedent (display/watch optional deps)
  constraints:
    - pure-rust-codecs-default
    - single-image-library
    - untrusted-input-hardening
  related_specs:
    - SPEC-058   # AVIF decode built (the avif module being gated)
    - SPEC-060   # SVG rasterize (stays in the wasm build)
    - SPEC-067   # watch feature-gate + lean-build precedent

value_link: >
  The load-bearing baseline of STAGE-025: proves the pure engine runs a real
  decode → transform → encode round-trip in wasm with no backend, establishing the
  build seam every other PROJ-008 stage layers over.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null            # design done in orchestrator main loop (un-metered, §4)
      note: "framed build-ready in the orchestrator main loop; grounded in the 2026-07-12 design-time WASM compile probe"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-072: WASM build seam + baseline (AVIF-decode gated out)

## Context

First spec of STAGE-025 (WASM core build), the load-bearing leg of PROJ-008 (WASM core
+ demo page, roadmap Wave 3). A **design-time WASM compile probe (2026-07-12)** —
`cargo build --lib --target wasm32-unknown-unknown --no-default-features --keep-going` —
found that the *entire* dependency tree compiles to wasm32 **except `re_rav1d`** (the
AVIF decoder): it imports `libc` POSIX types absent on bare wasm32 (`off_t`,
`ptrdiff_t`/`intptr_t`/`uintptr_t`, errno `ENOENT`/`EIO`/`EINVAL`) and has threading
const-eval failures (`E0080` in `re_rav1d/src/thread_task.rs`). `resvg`/`usvg`/
`tiny-skia` (SVG), `image` (PNG/JPEG/GIF/WebP/BMP/TIFF), `fast_image_resize`, `rayon`,
`ssimulacra2`, `skrifa`, `zeno` all compiled.

Architecturally the cut is clean: the transform core (`operation`, `pipeline`, `recipe`,
`analysis`, `quality`, `text`, `metadata`) has **zero** imports of the filesystem/CLI
shell (`source`/`sink`/`cli`/`build`), and `sink::encode_to_bytes` is a pure
bytes-in/bytes-out function. So this spec: adds a wasm build target behind
`cfg(target_arch = "wasm32")`, gates the one compile blocker (`re_rav1d`/`avif-parse` +
the `avif` module) out of the wasm build, gates the fs/CLI shell out for a lean artifact,
adds a thin `wasm-bindgen` surface, and proves a real round-trip headlessly — while
leaving the native default + lean builds **byte-for-byte unaffected**.

The AVIF-*on-wasm* strategy (whether `rav1e` encode compiles; whether/how `re_rav1d`
decode is later restored) is the NEXT spec (SPEC-073) — this one deliberately gates AVIF
decode OUT so the rest can ship and be measured. That is a legitimate MVP: SVG + raster
conversion still land in-browser.

## Goal

Produce a `wasm32` build of crustyimg's pure engine — a `cdylib` exposing a `wasm-bindgen`
surface that runs a real decode → transform → encode round-trip in a browser/Node with no
backend — with AVIF decode gated out, the native builds unaffected, and a repeatable build
recipe.

## Inputs

- **Files to read:**
  - `src/lib.rs` — the module list to cfg-partition (core vs shell).
  - `src/image/mod.rs` — `mod avif;` (line 29) and the AVIF dispatch in `decode_with_limits`
    (lines 353–355: `if avif::is_avif(bytes) { … decode_avif … }`) — the gate points.
  - `src/image/avif.rs` — imports `re_rav1d` + `avif_parse` (the wasm compile blocker).
  - `src/sink/mod.rs` — `encode_to_bytes` (line 573, the pure encode entry to reuse); the rest
    of `sink` is fs (OpenOptions/canonicalize/create_dir_all) used only by native.
  - `src/recipe/mod.rs` — `build_pipeline(&registry)` (line 274): recipe TOML → `Pipeline`.
  - `Cargo.toml` — `[lib]`, `[dependencies]`, `[features]`.
- **External:** `wasm-bindgen` (docs.rs/wasm-bindgen); `wasm-pack` (rustwasm.github.io/wasm-pack);
  `wasm-opt`/binaryen. None are installed on this machine (verified) — the build recipe installs
  the target + tooling.
- **Related code paths:** `src/operation/`, `src/pipeline/`, `src/analysis/` (the transform core,
  all wasm-clean per the probe).

## Outputs

- **Files created:**
  - `src/wasm.rs` — `#[cfg(target_arch = "wasm32")]` module with the `#[wasm_bindgen]` surface.
  - A wasm round-trip test (see Failing Tests) — `#[wasm_bindgen_test]`, run via `wasm-pack test`.
  - `docs/research/proj-008-wasm-build.md` (or fold into SPEC-073's run record) — the build
    recipe, the measured `.wasm` size, and the toolchain gotcha, so it's repeatable.
- **Files modified:**
  - `Cargo.toml` — `[lib] crate-type = ["cdylib", "rlib"]`; move `re_rav1d` + `avif-parse`
    (REQUIRED — the compile blockers) and the shell-only deps (`clap`, `clap_complete`, `glob`,
    `sha2`, `rayon`, `indicatif`; `notify`/`viuer` are already optional) to
    `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`; add `wasm-bindgen` (+ dev
    `wasm-bindgen-test`) under `[target.'cfg(target_arch = "wasm32")'.dependencies]`. Confirm
    each move against the compiler — keep any dep a wasm-retained module actually needs.
  - `src/lib.rs` — cfg-partition: keep `error`/`image`/`operation`/`pipeline`/`recipe`/
    `analysis`/`quality`/`text`/`metadata`/`sink` on wasm; gate `cli`/`source`/`build` (and
    optionally `lint`) with `#[cfg(not(target_arch = "wasm32"))]`; add
    `#[cfg(target_arch = "wasm32")] pub mod wasm;`.
  - `src/image/mod.rs` — gate `mod avif;` and the AVIF dispatch branch (353–355) with
    `#[cfg(not(target_arch = "wasm32"))]`; on wasm, an `.avif`/AVIF-sniffed input returns a
    **typed** error (a new `ImageError` variant or reuse of a "codec unavailable" shape — mirror
    the `CodecNotBuilt` pattern, DEC-056/SPEC-062), never a panic.
  - `justfile` — a `wasm-build` recipe (and optionally `wasm-test`) that installs/uses a stable
    toolchain + wasm32 target and works past the RUSTC gotcha (see Notes).
- **New exports (`src/wasm.rs`, exact signatures the builder's call — keep thin):**
  - `transform(input: &[u8], recipe_toml: &str, out_format: &str) -> Result<Vec<u8>, JsError>`
    — `Image::from_bytes` → `Recipe::from_toml` → `build_pipeline(&OperationRegistry::with_builtins())`
    → run → `sink::encode_to_bytes`. (Match `encode_to_bytes`'s real signature; read it.)
  - `info(input: &[u8]) -> Result<JsValue, JsError>` — width/height/source format.
  - `optimize(input: &[u8], out_format: &str) -> Result<Vec<u8>, JsError>` — a minimal
    auto-format/quality entry over `analysis`/`decide` if it wires cleanly; otherwise stub to a
    quality-preserving encode and file the full engine wiring as a follow-up (don't force it here).

## Acceptance Criteria

- [ ] `cargo build --lib --target wasm32-unknown-unknown` succeeds (release too) and produces a
      `.wasm` (with `wasm-pack build` / `wasm-bindgen`), with AVIF decode gated out.
- [ ] A `#[wasm_bindgen_test]` round-trip passes under `wasm-pack test --node` (or headless
      browser): a PNG fixture + a resize recipe TOML → output **decodes to the resized dimensions**;
      an SVG fixture → rasterized raster output; `info` returns correct width/height/format.
- [ ] An AVIF byte input to the wasm surface returns a **typed error string**, not a panic/abort.
- [ ] `cargo build` (native default) and `cargo build --no-default-features` (lean) still succeed;
      `cargo test` green; **native AVIF decode still works** (existing avif tests unaffected).
- [ ] `just deny` unchanged, OR any wasm-only dep (`wasm-bindgen` tail) is covered by a scoped,
      justified exception (record it); no change to the native dependency set.
- [ ] The `.wasm` size (release + `wasm-opt` if available) is **measured and recorded** with the
      build recipe, so SPEC-074 has a baseline number.
- [ ] `just wasm-build` reproduces the build on a **stable** toolchain (not just the probe's nightly).

## Failing Tests

Written now (design), before build. The build cycle makes them pass.

- **`tests/wasm_roundtrip.rs`** (or `src/wasm.rs` `#[cfg(test)]`), `#[wasm_bindgen_test]` —
  runs only under `wasm-pack test`, not the native `cargo test`:
  - `"transform_png_resize_roundtrip"` — a small PNG (reuse an existing fixture) + a resize
    recipe TOML → `transform(...)` → the returned bytes decode (native-side assert in the test,
    or a re-`info` call) to the resized dimensions. Asserts: decode→transform→encode works in wasm.
  - `"info_reports_png_dimensions"` — PNG in → `info` returns the correct w/h and format.
  - `"svg_rasterizes_in_wasm"` — a tiny SVG fixture → `transform`/decode yields a raster of the
    expected dims. Asserts: the resvg path lives in the wasm build.
  - `"avif_input_errors_not_panics"` — the AVIF fixture bytes → `transform` returns `Err(JsError)`
    (a clean "AVIF decode unavailable in wasm" message), no panic/abort.
- **Native guard** (ordinary `cargo test`, `#[cfg(not(target_arch = "wasm32"))]`):
  - `"native_avif_still_decodes"` — assert the existing AVIF fixture still decodes on native after
    the cfg gating (guards against the gate breaking the native path). May already be covered by an
    existing `image/mod.rs` avif test — if so, reference it instead of duplicating.

## Implementation Context

### Decisions that apply
- `DEC-004` (pure-Rust default) — `wasm-bindgen` is pure-Rust + permissive (MIT/Apache); it's a
  wasm-target-only dep, native set unchanged.
- `DEC-053` (AVIF decode = re_rav1d) — the crate being gated; its "serves the WASM demo" note was
  optimistic (the probe found it wasm-hostile). Don't remove it from native; only gate it off wasm.
- `DEC-054` (SVG = resvg) — confirmed wasm-clean; stays in the wasm build (a headline capability).
- `DEC-027` (display/watch feature-gating) — the precedent for keeping optional/native-only code
  out of a build without breaking it; the lean-build discipline mirror.

### Constraints that apply
- `pure-rust-codecs-default` — the wasm build is pure-Rust by construction (the C codecs
  libheif/lossy-webp are already opt-in and stay out).
- `single-image-library` — no second pixel library; `wasm-bindgen` is glue, not a codec.
- `untrusted-input-hardening` — the decode caps (DEC-034/063) live in the core and carry into wasm;
  the browser is an untrusted-input surface too. Don't drop the caps on the wasm path.

### Prior related work
- `SPEC-058` (AVIF decode) / `SPEC-060` (SVG) — the modules this spec partitions.
- `SPEC-067` (watch) — the "optional dep + lean build stays green + clear error when absent"
  precedent; verify includes the lean build ([[verify-includes-lean-no-default-features-build]]).

### Out of scope (for this spec)
- The AVIF-on-wasm DECISION (rav1e-encode feasibility; re_rav1d restore path A/B/C/D) — **SPEC-073**.
- Bundle-size optimization beyond a baseline measurement (lazy-loading, `wasm-opt` tuning) — **SPEC-074**.
- npm packaging (STAGE-026), the demo page (STAGE-027), a wasm CI job (stage decision; local
  `wasm-pack test` is the SPEC-072 floor, mirroring the fuzz-gate CI decision in DEC-062).
- Wiring the FULL optimization engine into `optimize` if it doesn't wire cleanly — stub + file it.

## Notes for the Implementer

- **Toolchain gotcha (cost the probe a debug cycle):** this machine has Homebrew rust
  (`/opt/homebrew/bin`, no wasm std) *and* rustup. `cargo build --target wasm32` invokes bare
  `rustc`, which PATH-resolves to Homebrew's → a misleading `error[E0463]: can't find crate for
  core/std`. Fix in `just wasm-build`: install a rustup **stable** toolchain + its `wasm32-unknown-unknown`
  target and invoke through it (e.g. `rustup run stable cargo …` with `RUSTC` forced to that
  toolchain's rustc), or use `wasm-pack` which manages this. The probe used nightly; the spec must
  work on stable/MSRV.
- **crate-type:** adding `cdylib` makes native builds also emit a dylib — harmless; `rlib` stays for
  the bin + tests.
- **Prefer `cfg(target_arch = "wasm32")` over a new feature flag** — the wasm build is target-selected,
  so no `--features wasm` needed; put wasm-only deps under the `[target.'cfg(target_arch = "wasm32")']`
  table and native-only deps under `[target.'cfg(not(target_arch = "wasm32"))']`. This keeps the
  native default/lean feature matrix exactly as-is.
- **Move deps conservatively:** `re_rav1d` + `avif-parse` MUST move (compile blockers). For the others
  (`clap`/`clap_complete`/`glob`/`sha2`/`rayon`/`indicatif`), let the compiler tell you — gate the
  module first, then move the dep only if nothing wasm-retained still needs it. `image`,
  `fast_image_resize`, `resvg`, `ssimulacra2`, `skrifa`, `zeno`, `img-parts`, `kamadak-exif`, `toml`,
  `serde`, `thiserror` all STAY (wasm-clean, core).
- **Likely a DEC** for adding `wasm-bindgen` + the target-cfg boundary (next id **DEC-064**) — emit it
  at build if the boundary/dep warrants recording (a new dep normally does).
- **Reuse, don't duplicate:** the transform path already exists end-to-end — `Image::from_bytes`,
  `Recipe`/`build_pipeline`/`OperationRegistry::with_builtins`, `Pipeline::run`, `sink::encode_to_bytes`.
  `src/wasm.rs` is thin glue over them, not new logic.
- **Drive it for real:** the wave's load-bearing lesson — a green compile is NOT the bar; run the
  `wasm-pack test` round-trip and confirm real output bytes. A "it compiles to wasm" claim without a
  driven round-trip is the unearned-verdict trap this project keeps catching.

---

## Build Completion

*Filled in at the end of the build cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** —
2. **Was there a constraint or decision that should have been listed but wasn't?** —
3. **If you did this task again, what would you do differently?** —

---

## Reflection (Ship)

*Appended during the ship cycle.*

1. **What would I do differently next time?** —
2. **Does any template, constraint, or decision need updating?** —
3. **Is there a follow-up spec I should write now before I forget?** —
