# SPEC-146 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-146-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-05, `prompts/SPEC-146-frame.md`, Opus): GO, [S]. All tests pass today
  (47 full, 45 slim, 0 ignored). A `src/wasm.rs` mutation turns `info_reports_png_dimensions` red
  on both sets. Recommends a new `wasm-roundtrip` job with the runner from `taiki-e/install-action`,
  its version read from `Cargo.lock` and asserted, because the runner's own check accepts
  0.2.122-0.2.127. Also adds `.cargo/` to the CI code-paths filter. Folds STAGE-053's wasm32 CI leg.
- [x] **design** (2026-11-05, `prompts/SPEC-146-design.md`, Opus): 11 ACs, 12 checks, 17-row
  mutation table. One non-matrix `wasm-roundtrip` job; a new `scripts/wasm-ci-check.sh` reads the
  lock version, asserts the runner, and holds an exact test-count floor (47/45), since compiling the
  wasm module out was driven to print `1 passed`, exit 0. Shape check runs in `metadata`.
- [x] **build** (2026-11-05, `prompts/SPEC-146-build.md`, Sonnet): draft PR #245 at `5098e4f`, CI green (19 pass). Cold job 4m04s, warm 1m31s; 47/45 passed. One design call changed (shape-check assertions 10/11). Cost about $2.15.
- [~] **verify** (`prompts/SPEC-146-verify.md`)
- [ ] **ship** (`prompts/SPEC-146-ship.md`)
