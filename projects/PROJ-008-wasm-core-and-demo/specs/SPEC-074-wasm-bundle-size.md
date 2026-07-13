---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-074
  type: story
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L

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
    - DEC-064    # the wasm target-cfg boundary + the deferred crustyimg-core split idea
    - DEC-065    # AVIF encode is IN (rav1e ~0.35 MB is a KEEP, not a lever)
    - DEC-054    # SVG via resvg with the `text` feature — the biggest addressable cluster
    - DEC-019    # SSIMULACRA2 perceptual search — skipped on wasm (SPEC-073), so its code may be dead weight
  constraints:
    - pure-rust-codecs-default
    - single-image-library
  related_specs:
    - SPEC-072   # the wasm build seam + size baseline (1.19 MB brotli core)
    - SPEC-073   # AVIF encode (1.52 MB brotli with avif); the shipped demo artifact

value_link: >
  Shrinks the demo bundle so "zero-install, instant try it" is actually instant — the wave's
  main technical debt (1.52 MB brotli). Completes STAGE-025.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      note: >
        framed build-ready in the orchestrator main loop; grounded in a design-time twiggy
        size-attribution probe (2026-07-12) on the raw release cdylib — no single whale; mass
        clusters in the SVG text/font stack + the raster-codec spread; ssimulacra2 is NOT a
        top contributor (weakening a prior hypothesis).
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-074: WASM bundle size

## Context

Last spec of STAGE-025 (its ship completes the stage). The shipped demo `.wasm` is **1.52 MB
brotli** (with AVIF encode, SPEC-073), of which ~0.35 MB is `rav1e` — a **keep**, it's the
headline (DEC-065) — and ~1.19 MB is the core engine (SPEC-072). That core is the addressable
debt: a squoosh-style "zero-install, instant try it" demo (Wave 3's whole point) wants a small
first load.

A **design-time twiggy probe (2026-07-12)** on the raw release cdylib found **no single whale** —
the code is spread across 5,300+ items. The addressable mass clusters in:
1. **The SVG text/font stack** — `usvg::text::layout`, `ttf_parser` (CFF/COLR glyph parsing),
   `rustybuzz`, `unicode_bidi`: resvg's `text` feature (DEC-054, `features = ["text"]` + the
   bundled Go font) pulls a full text-shaping/font-parsing subtree.
2. **The raster-codec spread** — `zune_jpeg`, `png`, `tiff`, `image_webp`, `fdeflate`: the `image`
   crate's decoder set (png/jpeg/gif/bmp/tiff/ico/webp), some of which the demo never decodes.
3. **`.rodata` (~15%)** — static tables tied to the above (font/unicode/codec tables).
4. **`ssimulacra2` is NOT a top contributor** — which weakens the build's earlier hypothesis;
   note that on wasm `optimize` skips the perceptual search (SPEC-073), so `quality`/ssimulacra2
   code may be partly dead weight the linker isn't eliminating.

**Probe caveat (carried, not hidden):** that attribution is from the debug-heavy, non-`wasm-opt`
raw build — directional, not exact. The honest per-lever measure is **feature-ablation
brotli-diffing on the shipped `wasm-opt`'d artifact**: toggle a lever, rebuild via `just
wasm-build`, diff the brotli size. That is this spec's method, not twiggy guesses.

## Goal

Measurably shrink the shipped demo `.wasm` (brotli) by pulling the size levers that cost no
capability, and turning each capability-losing lever (dropping SVG `<text>`, trimming a codec)
into an explicit, data-backed keep/drop decision — recorded in a DEC. rav1e/AVIF encode stays.

## Inputs

- **Files to read:**
  - `docs/research/proj-008-wasm-build.md` — the SPEC-072/073 size baselines to diff against.
  - `Cargo.toml` — `resvg = { features = ["text"] }` (:~), the `image` feature list
    (`png,jpeg,gif,bmp,tiff,ico,webp`), the wasm target dep tables (DEC-064), `[profile.*]`.
  - `justfile` — `wasm-build`/`wasm-size` (the brotli measurement) + the `_wasm_features` override.
  - `src/wasm.rs` / `src/image/` — what the wasm surface actually decodes/encodes (to know which
    `image` codecs + whether SVG text are load-bearing for the demo).
- **External tooling:** `twiggy` + `cargo-bloat` (installed during the probe); `wasm-opt` (binaryen).

## Outputs

- **Files modified:**
  - `Cargo.toml` — trim wasm-side features per the measured decisions (e.g. `image` codec set;
    `resvg` `text`); possibly a size-tuned wasm build profile (`opt-level = "z"`, `lto = true`,
    `codegen-units = 1`, `panic = "abort"`, `strip`) if it wins brotli without breaking the tests.
  - `justfile` — `wasm-opt -Oz` vs `-O` if `-Oz` wins; keep `wasm-size` reporting brotli.
  - `docs/research/proj-008-wasm-build.md` — the ablation table (each lever → brotli delta) + the
    new baseline.
  - `decisions/DEC-066-*.md` — the keep/drop calls (what was trimmed, what was kept and why,
    esp. any capability tradeoff like SVG text or a dropped input codec).
- **No change to:** rav1e/AVIF encode (kept); the native build feature set (wasm-only trims stay
  behind the target-cfg boundary, DEC-064); the pure-Rust posture.

## Acceptance Criteria

- [ ] An **ablation table** exists (`docs/research/proj-008-wasm-build.md`): each candidate lever
      (resvg `text`, each trimmable `image` codec, ssimulacra2/quality dead-code, `opt-level="z"`,
      `wasm-opt -Oz`) → its measured **brotli delta** on the shipped artifact. Measured, not guessed.
- [ ] The shipped demo `.wasm` brotli is **reduced** vs the 1.52 MB (with-avif) / 1.19 MB (core)
      baseline by the no-capability-cost levers, with the new number recorded. (No hard numeric
      budget is invented; the win is whatever the honest levers yield — state it.)
- [ ] Every **capability-losing** lever is an explicit DEC-066 decision with its measured saving
      and the tradeoff (e.g. "dropped SVG `<text>` → SVGs with text rasterize without glyphs, saves
      N KB" — keep or drop, with rationale). No silent capability loss.
- [ ] **No silent functional regression:** `just wasm-test` still green — the round-trip, SVG
      rasterize, and AVIF-encode tests all pass (if SVG text is dropped, add/adjust a test that
      pins the new behavior honestly rather than letting a test quietly stop covering it).
- [ ] **Native unaffected:** `cargo build`, `--no-default-features`, `cargo test`, `cargo clippy`,
      `just deny` green; the native codec/feature set unchanged (trims are wasm-target-scoped).

## Failing Tests

Written now (design). Size work is measurement-driven, so the "tests" are the guardrails that a
size cut didn't break a capability:

- **`tests/wasm_roundtrip.rs`** (existing, must stay green): the PNG-resize round-trip, SVG
  rasterize, and PNG→AVIF tests. If a codec the demo relies on is trimmed, its test must fail —
  that's the guardrail. If SVG `<text>` is dropped, add `svg_without_text_still_rasterizes` (a
  text-free SVG works) and make the text case's changed behavior explicit, not silently uncovered.
- **Native guard** (`#[cfg(not(target_arch = "wasm32"))]`): assert the native `image` codec set +
  SVG text are untouched (a native SVG-with-text still renders glyphs), so the wasm trim didn't
  leak into native.

## Implementation Context

### Decisions that apply
- `DEC-065` — AVIF encode (rav1e, ~0.35 MB) is a KEEP; do not treat it as a size lever.
- `DEC-064` — every wasm-only trim stays behind `cfg(target_arch = "wasm32")` / a wasm dep-table
  entry; the native feature matrix and released binary stay byte-identical. The deferred
  `crustyimg-core` crate split is **not** a pure-size lever (it enables multi-artifact packaging,
  STAGE-026) — don't reach for it here unless a measured reason appears.
- `DEC-054` — resvg's `text` feature is the biggest single addressable cluster; dropping it is a
  real capability tradeoff (SVG `<text>` stops rendering as glyphs), so it's a DEC-066 decision,
  not a silent trim.
- `DEC-019` — the SSIMULACRA2 perceptual search is skipped on wasm (SPEC-073); check whether its
  code is actually eliminated or is dead weight a `cfg` could drop.

### Constraints that apply
- `pure-rust-codecs-default` / `single-image-library` — trims are feature/codec removals, not codec
  swaps; no new pixel library, no C.

### Prior related work
- `SPEC-072` (wasm seam, 1.19 MB core) / `SPEC-073` (AVIF encode, 1.52 MB). The `just wasm-size`
  brotli number is the metric; the `_wasm_features` override toggles avif for A/B measurement.

### Out of scope (for this spec)
- The `crustyimg-core` crate split (a STAGE-026/packaging concern unless size forces it).
- npm packaging (STAGE-026), the demo page (STAGE-027).
- Shrinking rav1e / dropping AVIF encode (DEC-065 keep).
- Any native-build size change.

## Notes for the Implementer

- **Measure on the shipped artifact, ablation-style.** twiggy on the raw cdylib (what the probe
  ran) is directional but debug-inflated; the number that matters is brotli after `wasm-opt`.
  Toggle one lever → `just wasm-build` → `just wasm-size` → record the brotli delta. Build the
  table before deciding.
- **Cheap, no-capability-cost levers first** (do these regardless): a size-tuned wasm profile
  (`opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`) and
  `wasm-opt -Oz`; confirm each is a brotli win and keeps `just wasm-test` green.
- **Then the capability levers, with data:** which `image` codecs does the demo actually decode?
  (It converts *from* PNG/JPEG/GIF/WebP/SVG — tiff/bmp/ico decode are likely unused and trimmable
  for the wasm target.) Does the demo need SVG `<text>`? That's the big one and a genuine tradeoff
  — bring the measured saving + the UX cost to DEC-066; if it's large and the demo can live without
  perfect text SVGs (or fall back), it may be worth it. If uncertain, keep text and record why.
- Sync any user-facing note if the wasm feature story changes (the SPEC-071 doc-drift lesson).
- Next DEC id is **DEC-066**. Commit with `-s` (DCO is real; it bit SPEC-072). Drive the real
  `wasm-build` + `wasm-test` — a smaller binary that lost a capability silently is a defect.

---

## Build Completion

*Filled in at the end of the build cycle.*

- **Branch:**
- **PR:**
- **All acceptance criteria met?**
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection

1. **What was unclear in the spec that slowed you down?** —
2. **Was there a constraint or decision that should have been listed but wasn't?** —
3. **If you did this task again, what would you do differently?** —

---

## Reflection (Ship)

1. **What would I do differently next time?** —
2. **Does any template, constraint, or decision need updating?** —
3. **Is there a follow-up spec I should write now before I forget?** —
