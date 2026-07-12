---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-072
  type: story
  cycle: verify  # frame | design | build | verify | ship
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
    - cycle: build
      interface: claude-code
      tokens_total: 400000
      estimated_usd: 3.60
      note: >
        ORDER-OF-MAGNITUDE ESTIMATE, not a meter reading: the build ran in an
        interactive main-loop session, not a metered subagent, so no
        `subagent_tokens` exists to copy (§4 / the labelled-estimates practice).
        Costed at the Opus 4.8 list rate ($5/$25 per MTok, ~80/20 in/out => $9/MTok)
        with no cache discount. Ship should keep the label; the number is a floor-ish
        guess, not a measurement.
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

- **Branch:** `feat/spec-072-wasm-build-seam`
- **PR:** #74
- **All acceptance criteria met?** **yes** — all 7.

| criterion | evidence |
|---|---|
| wasm32 lib builds (release too), `.wasm` produced, AVIF gated out | `just wasm-check` + `just wasm-build` → `pkg/crustyimg_bg.wasm` |
| `#[wasm_bindgen_test]` round-trip passes | **7/7 green** in Node: PNG+resize → bytes that **decode to 32×24**; SVG → raster; `info` correct |
| AVIF input → typed error, not a panic | `avif_input_errors_not_panics` asserts `Err`, message names AVIF, and does **not** advise `--features` |
| native default + lean build + `cargo test` green; native AVIF still decodes | **714 tests green**; `--no-default-features` builds; `native_avif_still_decodes` asserts it directly |
| `just deny` unchanged / no new exception | **licenses, bans, sources all ok — NO new exception** (wasm-bindgen tail is MIT/Apache) |
| `.wasm` size measured + recorded | **4.29 MB raw / 1.64 MB gzip / 1.19 MB brotli** (post `wasm-opt`) → `docs/research/proj-008-wasm-build.md` |
| `just wasm-build` reproduces on **stable** | yes — recipes resolve the rustup stable toolchain explicitly; no nightly needed |

- **New decisions emitted:** **DEC-064** — the WASM boundary is `cfg(target_arch)`, not a
  cargo feature; `wasm-bindgen` as the JS glue; AVIF decode out of the wasm build.

- **Deviations from spec:**
  1. **`info` returns a `#[wasm_bindgen] struct ImageInfo`, not `JsValue`.** A real JS
     object with getters (`width`/`height`/`format`/`hasAlpha`) — better DX and no
     `serde-wasm-bindgen` dep, which `JsValue` would have needed.
  2. **The AVIF sniff moved to a new `src/image/sniff.rs`.** Not in the spec's file list,
     but forced: the spec says the wasm build must *detect* AVIF and return a typed
     error, and `is_avif` lived inside the module that had to leave the wasm build.
     Detection now lives apart from the decoder it dispatches to.
  3. **A new `ImageError::CodecUnavailableOnTarget`,** rather than reusing
     `CodecNotBuilt` — whose message ("rebuild with `--features X`") would be a lie in a
     browser. Gated `cfg(target_arch = "wasm32")` so the native exit-code map stays
     total (the SPEC-061/062 lesson).
  4. **Tests run via `cargo test --target wasm32 --test wasm_roundtrip` (with a
     `.cargo/config.toml` runner), not `wasm-pack test`.** `wasm-pack test` hardcodes
     `cargo build --tests`, which drags all ~20 CLI-driving native integration tests into
     the wasm build. The alternative was `#![cfg(not(target_arch = "wasm32"))]` on every
     native test file — forever, on every new one. `wasm-pack` is still used for
     `wasm-build`.
  5. **`criterion`/`tempfile` moved to native-only dev-deps** — cargo builds *every*
     dev-dep for *any* test target, and `criterion` hard-`compile_error!`s on wasm.
     Unforeseen but unavoidable.
  6. **`optimize` is partial (as the spec permitted).** It uses the real engine
     (`Analysis` → `decide::format_shortlist` → `quality::auto_quality` SSIMULACRA2
     search) but takes the shortlist's first candidate instead of solving all candidates
     via `pick_winner`. Follow-up below.
  7. **One out-of-scope commit:** `chore(deny)` ignoring RUSTSEC-2026-0206. **`main` was
     already red** — verified on a clean `main` worktree — because `rustybuzz` was newly
     declared unmaintained upstream. Same resvg/usvg text stack, same upstream remedy, as
     the already-ignored RUSTSEC-2026-0192. Kept as a **separate commit** so it is not
     mistaken for SPEC-072 work.

- **Follow-up work identified:**
  1. **A shared engine seam for the full `optimize` solve**, called by both `cli` and
     `wasm`, so the multi-candidate `pick_winner` path isn't CLI-only (and isn't
     copy-pasted into `wasm.rs`).
  2. **A wasm CI job.** Today nothing stops a commit from silently breaking the wasm
     build; `just wasm-test` is a local floor (mirrors the fuzz-gate precedent, DEC-062).
     Stage-level decision.
  3. **Bundle size — SPEC-074's brief.** 1.19 MB brotli is the wave's main debt.
     Prime suspects: `ssimulacra2`, the resvg text stack, unused `image` codecs. The
     bigger lever is a separate `crustyimg-core` crate (DEC-064 defers it until a
     measurement argues for it — this is that measurement).
  4. **A default-ON feature with a native-only dep needs a `not(wasm32)` conjunct.**
     Today only `display`. If a second appears, introduce a cfg alias.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** — Nothing about the *design*;
   the spec was unusually well-grounded (the probe's findings held exactly, and the
   "clean cut" claim about the core was true). What it could not have known is that the
   wasm **test harness** — not the wasm build — was the hard part. The spec said "run it
   via `wasm-pack test`", and `wasm-pack test` cannot work in a repo whose integration
   tests drive a CLI binary. Three of the seven deviations above are consequences of
   that one discovery. A design-time probe of `wasm-pack test` (not just `cargo build
   --target wasm32`) would have surfaced it.

2. **Was there a constraint or decision that should have been listed but wasn't?** — The
   spec listed `untrusted-input-hardening` but framed it as "keep the decode caps". The
   sharper point, which I had to derive: **in wasm a panic aborts the module**, so
   "typed error, never panic" stops being a code-quality rule and becomes a
   crash-the-user's-page rule. That's the actual reason `CodecUnavailableOnTarget` had
   to be an error rather than an `unimplemented!()`, and it should be stated in the
   constraint's wasm framing.

3. **If you did this task again, what would you do differently?** — Build the test
   harness *first*, before touching `Cargo.toml`. I partitioned the deps, gated the
   modules, wrote the surface, got a clean wasm compile — and only then discovered the
   test runner couldn't run in this repo, which forced changes back into the manifest
   (`criterion`) and `main.rs`. The compile was the easy half and I did it first because
   it was the easy half. Driving one trivial `#[wasm_bindgen_test]` end-to-end on day
   one would have exposed the `wasm-pack`/`--tests`/`criterion`/`main.rs` chain in
   minutes instead of at the end.

---

## Reflection (Ship)

*Appended during the ship cycle.*

1. **What would I do differently next time?** —
2. **Does any template, constraint, or decision need updating?** —
3. **Is there a follow-up spec I should write now before I forget?** —
