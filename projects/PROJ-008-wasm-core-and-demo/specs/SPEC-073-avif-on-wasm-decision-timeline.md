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
- [ ] **build** — through a PR. Wire `out_format="avif"` into `src/wasm.rs`; enable `avif` for the
  wasm build per the size-measured strategy; PNG→AVIF `#[wasm_bindgen_test]`; measure the size
  delta; emit DEC-065. Drive the real encode in the wasm VM (not just compile); keep native + lean
  unaffected. Commit with `-s` (DCO — the SPEC-072 verify miss).
- [ ] **verify** — fresh adversarial session: re-drive PNG→AVIF in the wasm VM (valid AVIF bytes),
  confirm AVIF-input still errors, native+lean unaffected, native AVIF encode intact, the size
  delta reproduced.
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals, reflection, memory + brag.
