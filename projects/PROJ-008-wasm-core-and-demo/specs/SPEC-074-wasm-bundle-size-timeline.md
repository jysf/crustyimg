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
- [ ] **build** — through a PR. Build the ablation table (each lever → brotli delta), pull the
  no-cost levers (opt-level="z", wasm-opt -Oz, trim unused image codecs), decide the capability
  levers with data → DEC-066. Keep just wasm-test green (no silent capability loss). Native
  unaffected. Commit with `-s`.
- [ ] **verify** — fresh adversarial session: reproduce the brotli number, confirm no capability
  silently dropped (drive the demo conversions), native + lean unaffected.
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals, reflection, memory + brag.
  **Ship completes STAGE-025** → run the stage-ship reflection; next = STAGE-026 (npm) / STAGE-027 (demo).
