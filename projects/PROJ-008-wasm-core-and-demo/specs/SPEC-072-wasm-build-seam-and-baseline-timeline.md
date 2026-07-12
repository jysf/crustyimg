# SPEC-072 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] **design** (2026-07-12, orchestrator main loop) — framed build-ready. Grounded in the
  2026-07-12 design-time WASM compile probe (only `re_rav1d` blocks wasm32; the transform core
  is shell-free). Acceptance criteria + failing tests (`tests/wasm_roundtrip.rs`, `#[wasm_bindgen_test]`)
  specified; gating strategy = `cfg(target_arch = "wasm32")` + target-scoped dep tables.
- [ ] **build** — make the failing tests pass on a fresh build session (via a PR). Likely emits DEC-064
  (wasm-bindgen dep + target-cfg boundary). Drive the real `wasm-pack test` round-trip, not just a compile.
- [ ] **verify** — fresh adversarial session: confirm the round-trip on the real `.wasm`, the AVIF-input
  typed error, and that the **native default + lean builds are unaffected** (the lean-build check is
  mandatory here) + native AVIF still decodes.
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals, reflection, memory + brag.
