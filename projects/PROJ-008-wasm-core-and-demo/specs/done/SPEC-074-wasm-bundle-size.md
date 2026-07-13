---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-074
  type: story
  cycle: ship  # frame | design | build | verify | ship
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
    - cycle: build
      interface: claude-code
      tokens_total: 210000
      estimated_usd: 1.90
      note: >
        ORDER-OF-MAGNITUDE ESTIMATE (build ran in the main loop, not a metered subagent —
        see the autonomous-run-cost-estimates lesson). Single build session, 2026-07-12.
        Dominated by 16 real wasm builds (each a full release link of rav1e + resvg) and
        the Node driver runs that timed the shipped artifact.
    - cycle: verify
      interface: claude-code
      tokens_total: 120000
      estimated_usd: 1.10
      note: >
        ORDER-OF-MAGNITUDE ESTIMATE (verify ran in the main loop, not a metered subagent —
        see the autonomous-run-cost-estimates lesson). Fresh adversarial session, 2026-07-12.
        Dominated by ~8 independent wasm builds reproducing the endpoints and ablation rows
        (baseline, shipped, wasm-opt on/off, opt-level=z, resvg-text drop, no-profile), plus
        the Node timing driver and the resvg-text mutation test.
  totals:
    tokens_total: 330000        # build 210k + verify 120k (design null, un-metered main loop)
    estimated_usd: 3.00         # ~330k @ ~$9/MTok — LABELLED ESTIMATE, not a meter read (§4)
    session_count: 3
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

- [x] An **ablation table** exists (`docs/research/proj-008-wasm-build.md`): each candidate lever
      (resvg `text`, each trimmable `image` codec, ssimulacra2/quality dead-code, `opt-level="z"`,
      `wasm-opt -Oz`) → its measured **brotli delta** on the shipped artifact. Measured, not guessed.
- [x] The shipped demo `.wasm` brotli is **reduced** vs the 1.52 MB (with-avif) / 1.19 MB (core)
      baseline by the no-capability-cost levers, with the new number recorded. (No hard numeric
      budget is invented; the win is whatever the honest levers yield — state it.)
- [x] Every **capability-losing** lever is an explicit DEC-066 decision with its measured saving
      and the tradeoff (e.g. "dropped SVG `<text>` → SVGs with text rasterize without glyphs, saves
      N KB" — keep or drop, with rationale). No silent capability loss.
- [x] **No silent functional regression:** `just wasm-test` still green — the round-trip, SVG
      rasterize, and AVIF-encode tests all pass (if SVG text is dropped, add/adjust a test that
      pins the new behavior honestly rather than letting a test quietly stop covering it).
- [x] **Native unaffected:** `cargo build`, `--no-default-features`, `cargo test`, `cargo clippy`,
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

- **Branch:** `feat/spec-074-wasm-bundle-size`
- **PR:** #83
- **All acceptance criteria met?** **Yes, all five.**
  - *Ablation table exists, measured:* 16 real builds, in `docs/research/proj-008-wasm-build.md` §8
    and DEC-066. Every lever the spec named plus three it didn't (`wasm-opt` on/off, `strip`,
    per-package `opt-level` overrides).
  - *Brotli reduced:* **1,595,028 → 1,394,313 B (−200,715, −12.6%). 1.52 MB → 1.33 MB.** No numeric
    budget was invented; this is what the honest levers yielded.
  - *Every capability-losing lever is an explicit DEC-066 call with its measured saving:* resvg
    `text` (−287,098, REFUSED), `opt-level=z`/`s` (−247,936/−169,552, REFUSED), `ssimulacra2`
    (−23,540, REFUSED), `image` tiff/bmp/ico (−84,327, **TAKEN** — the one capability sold).
  - *No silent functional regression:* `just wasm-test` **12/12** (was 10). The two new guardrails
    were **mutation-tested** — each made to fail by re-introducing what it guards, then restored.
    The shipped `pkg/` artifact was additionally driven from Node: all 10 demo conversions pass.
  - *Native unaffected:* build / `--no-default-features` / test / clippy / fmt / `just deny` all
    green, **and `Cargo.lock` is byte-identical** — the strongest available evidence the native dep
    graph never moved.
- **New decisions emitted:** **DEC-066** — the keep/drop calls, the full ablation table, and the
  three traps (wasm-opt's silent failure; `opt-level=z` selling encode speed; a lever's value
  depending on which other levers are pulled).
- **Deviations from spec:** **Two, both because the measurement contradicted the design.** The spec
  listed `opt-level = "z"` and `wasm-opt -Oz` as *"cheap, no-capability-cost levers (do these
  regardless)"*. Driven, neither is:
  1. `opt-level = "z"` costs **2.8× on AVIF encode** (350 → 956 ms) — a capability cost the spec
     didn't anticipate because it only weighed size. Refused; `opt-level` stays 3.
  2. `wasm-opt` **fails validation under `opt-level = "z"`** (thousands of validator errors), and
     once *made* to run it **costs 36 KB on the wire** for 340 KB of raw, at zero speed benefit.
     Turned **OFF** deliberately, with the working flag list recorded for whoever re-enables it.
     *(Corrected at verify: the build recorded that failure as silent — "swallowed at exit 0", and
     so cast doubt on SPEC-072/073's numbers. It is not silent; wasm-pack exits 1, and SPEC-072/073's
     numbers were genuinely post-wasm-opt. See the verify row in the timeline.)*
  The spec's genuinely free levers turned out to be the ones it didn't list: fat LTO **with**
  `codegen-units = 1` (−79,900; fat LTO *alone* is +1,450 — worse than useless) and `strip`
  (−58,533).
- **Follow-up work identified:**
  - **A CI job must build the wasm artifact through `just wasm-build`.** The size profile now lives
    in the recipe's `CARGO_PROFILE_RELEASE_*` env vars (it cannot live in `[profile.release]`, which
    native shares — DEC-064), so a bare `cargo build --target wasm32-…` silently ships a heavier
    artifact. This compounds the identical hazard DEC-065 left for `--features avif`. Still no wasm
    CI job at all (SPEC-072 follow-up) — this raises its priority.
  - **STAGE-026 should re-decide the two refused capability levers** against a real packaging seam:
    a lazy chunk could make the 287 KB SVG-text stack and the 84 KB tiff/bmp/ico decoders opt-in
    downloads rather than a keep/drop. The price tags are now known.
  - `optimize`'s single-candidate shortcut (a SPEC-072 follow-up) is untouched and still open.

### Build-phase reflection

1. **What was unclear in the spec that slowed you down?** — Nothing was unclear; the spec was
   unusually well-framed, and its "measure by ablation, don't trust twiggy" instruction was exactly
   right. What cost time was the spec being *confidently wrong* in its Notes: it pre-classified
   `opt-level="z"` and `wasm-opt -Oz` as free wins to "do regardless", and I nearly did. Both are
   capability trades. The cheap insurance was driving the artifact for **runtime**, not just size —
   a size spec that never times the thing it shrinks will happily ship a 2.8× slower encoder and
   call it a win.
2. **Was there a constraint or decision that should have been listed but wasn't?** — Yes: **encode
   speed is a capability.** The spec's boundary was "no capability loss", but it defined capability
   as *what the build can do*, never *how fast* — so `opt-level` looked free. On a demo whose
   headline is a codec that already runs serial (DEC-065/STAGE-027), latency **is** the product. A
   constraint like `wasm-encode-latency-is-load-bearing` would have made the biggest decision here
   fall out immediately instead of after four builds.
3. **If you did this task again, what would you do differently?** — Build the Node driver that
   exercises the shipped `pkg/` **first**, before touching a single lever, and make it print size
   *and* timings together. I built it midway to prove the demo still worked, and only then
   discovered the speed regression that reversed two decisions. Size and speed are one number here,
   and I measured them serially. Related: I twice trusted a lever measured in a config I wasn't
   shipping — `strip` looked like 250 B of noise (because wasm-opt was doing the stripping) and I
   nearly deleted it; it is worth 58 KB in the config we ship. **Measure in the config you ship.**

---

## Reflection (Ship)

*Appended during ship (2026-07-12). Shipped via PR #83 (squash `506df80`, DEC-066); clean path,
both commits signed off. Its ship COMPLETES STAGE-025.*

1. **What would I do differently next time?** — The build's headline finding was *wrong* and
   propagated into four repo files + the auto-memory before verify caught it: it claimed `wasm-opt`
   "silently fails at exit 0, so wasm-pack shipped an unoptimized module — SPEC-072/073's numbers
   were never optimized." Verify reproduced it and found the opposite: `wasm-opt` fails **loud**
   (exit 1) and main's baseline was genuinely optimized (it strips 1.6 MB raw). The −12.6% headline
   was never in doubt — but a *striking, quotable* claim, written as durable guidance, is exactly
   what spreads fastest and is most worth an adversarial re-drive. **Lesson: an adversarial verify
   must re-drive the build's incidental "aha" findings, not just its acceptance criteria** — the
   more memorable the claim, the higher the cost if it's wrong (it "defamed a sound baseline" and
   would have misled whoever re-enables wasm-opt).
2. **Does any template, constraint, or decision need updating?** — DEC-066 records the levers
   (taken: fat-LTO+cgu=1, strip, wasm-opt-off, wasm-only tiff/bmp/ico trim; refused: resvg `text`
   −287 KB because dropping it *silently deletes* SVG text, opt-level z/s because they cost 3.4×
   AVIF encode speed, ssimulacra2 because the search is live on wasm). Two durable lessons banked
   to memory: **a size spec must TIME the artifact, not just weigh it** (encode speed is a
   capability, and rav1e is generic so it monomorphizes into `ravif` — pin *that*, not rav1e); and
   **the wasm size profile can't live in `[profile.release]`** (native shares it) so it lives in
   the `just wasm-build` env vars — which means a bare `cargo build --target wasm32` silently ships
   +109 KB heavier. The corrected wasm-opt guidance ("check the raw size moved; don't read the exit
   code through a pipe") is in DEC-066 + the research doc.
3. **Is there a follow-up spec I should write now before I forget?** — Filed to `docs/roadmap.md`:
   (a) **a wasm CI job must build through `just wasm-build`** (the +109 KB footgun; already a
   carried SPEC-072/073 follow-up, now measured and reinforced); (b) **`optimize(img, "webp")` on
   wasm returns *lossless* WebP** (320 KB vs a 44 KB JPEG) — there's no lossy-WebP encoder in the
   wasm feature set, so the perceptual path silently isn't available for WebP; worth resolving
   before STAGE-027's demo offers WebP as an output. Neither is a SPEC-074 defect. STAGE-025 is now
   complete; PROJ-008 continues with STAGE-026 (npm) and STAGE-027 (demo).
