# SPEC-074 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] **design** (2026-07-12, orchestrator main loop) — framed build-ready. Grounded in a
  design-time twiggy size-attribution probe on the raw release cdylib: **no single whale** — mass
  clusters in the SVG text/font stack (usvg text + ttf_parser CFF/COLR + rustybuzz + unicode_bidi)
  and the raster-codec spread (zune_jpeg/png/tiff/image_webp/fdeflate); ssimulacra2 is NOT a top
  contributor. Method = feature-ablation brotli-diffing on the wasm-opt'd artifact + a size-tuned
  wasm profile. Capability-losing levers (drop SVG text, trim a codec) = explicit DEC-066 calls.
- [x] **build** (2026-07-12, branch `feat/spec-074-wasm-bundle-size`, PR #83) — ablation table built
  (16 real builds), **1,595,028 → 1,394,313 B brotli (−200,715, −12.6%): 1.52 → 1.33 MB**. DEC-066
  emitted. **Both of the design's two suggested "no-cost" levers turned out to COST, and were
  refused:** `opt-level="z"` makes the AVIF encoder **2.8× slower** (350→956 ms — rav1e is generic,
  so it monomorphizes into `ravif`; protecting it hands back 161 of the 165 KB), and `wasm-opt` was
  **silently failing validation** while wasm-pack shipped the unoptimized module at exit 0 — and
  measured, it *costs* 36 KB on the wire for 340 KB of raw it buys no speed with, so it is now OFF.
  Taken instead: fat LTO + codegen-units=1 + strip (−138 KB, free) and a wasm-only `image` trim of
  tiff/bmp/ico (−84 KB). **resvg `text` REFUSED at −287 KB** — dropping it doesn't degrade SVG text,
  it silently *deletes* it (built that artifact; `transform()` still returned Ok and the old test
  stayed green through the corruption). 2 new mutation-tested guardrails; wasm-test 12/12; native +
  lean + deny green; Cargo.lock untouched.
- [ ] **verify** — fresh adversarial session: reproduce the brotli number, confirm no capability
  silently dropped (drive the demo conversions), native + lean unaffected.
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals, reflection, memory + brag.
  **Ship completes STAGE-025** → run the stage-ship reflection; next = STAGE-026 (npm) / STAGE-027 (demo).
