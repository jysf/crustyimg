# SPEC-072 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] **design** (2026-07-12, orchestrator main loop) — framed build-ready. Grounded in the
  2026-07-12 design-time WASM compile probe (only `re_rav1d` blocks wasm32; the transform core
  is shell-free). Acceptance criteria + failing tests (`tests/wasm_roundtrip.rs`, `#[wasm_bindgen_test]`)
  specified; gating strategy = `cfg(target_arch = "wasm32")` + target-scoped dep tables.
- [x] **build** (2026-07-12, PR #74, ~400k tok est.) — **7/7 wasm round-trip tests green in Node** on the
  real `.wasm` (PNG+resize → bytes decoding to 32×24; SVG rasterizes; AVIF errors cleanly, no panic).
  Native unaffected: 714 tests + lean build + clippy + native AVIF decode all green; `just deny` needed
  **no new exception**. Emitted **DEC-064**. Size baseline **4.29 MB raw / 1.64 MB gzip / 1.19 MB brotli**
  → `docs/research/proj-008-wasm-build.md`. NOTE: the round-trip runs via
  `cargo test --target wasm32 --test wasm_roundtrip` + a `.cargo/config.toml` runner, **not**
  `wasm-pack test` — the latter hardcodes `--tests` and drags all ~20 CLI-driving native integration
  tests into the wasm build. Also carries one **out-of-scope** `chore(deny)` commit (RUSTSEC-2026-0206:
  `main` was already red before this branch — verified on a clean worktree).
- [ ] **verify** — fresh adversarial session: confirm the round-trip on the real `.wasm`, the AVIF-input
  typed error, and that the **native default + lean builds are unaffected** (the lean-build check is
  mandatory here) + native AVIF still decodes.
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals, reflection, memory + brag.
