# SPEC-073 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] **design** (2026-07-12, orchestrator main loop) — framed build-ready. Grounded in a
  design-time probe: `cargo build --lib --target wasm32-unknown-unknown --features avif` compiled
  clean (exit 0) — **rav1e 0.8.1 + ravif 0.13.0 build to wasm32**, so AVIF *encode* is achievable
  (the "convert to AVIF in-browser" headline); AVIF *decode* (re_rav1d) stays gated (SPEC-072).
  Spec = wire encode into the wasm surface + measure the size delta + DEC-065 (encode in, decode
  deferred). Failing tests specified.
- [x] **build** (2026-07-12, branch `feat/spec-073-avif-on-wasm`, PR #82) — AVIF encode RUNS in the
  wasm VM: `transform(png, recipe, "avif")` returns ftyp/`avif`-branded bytes, 10/10
  `#[wasm_bindgen_test]`s green under `just wasm-test`. Size measured on the release artifact:
  **1.19 → 1.52 MB brotli (+345 KB, +27.7%)** → strategy = **one artifact, `avif` ON** (a lazy
  chunk would re-link the whole engine: 2.71 MB for the AVIF user). **DEC-065** emitted (encode in
  / decode deferred — the browser's own `createImageBitmap` reads `.avif`). `optimize(_, "avif")`
  skips the perceptual search (it needs a decoder). NO `Cargo.toml` dep change; native + lean +
  clippy + deny green. Ready for verify.
- [ ] **verify** — fresh adversarial session: re-drive PNG→AVIF in the wasm VM (valid AVIF bytes),
  confirm AVIF-input still errors, native+lean unaffected, native AVIF encode intact, the size
  delta reproduced.
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals, reflection, memory + brag.
