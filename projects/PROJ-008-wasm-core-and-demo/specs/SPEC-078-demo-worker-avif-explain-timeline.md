# SPEC-078 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] **design** (2026-07-13, orchestrator main loop) — framed build-ready. Grounded in the shipped
  SPEC-077 demo + the wave's facts: AVIF encode is ALREADY compiled into the deployed `.wasm`
  (DEC-065) so this enables + off-loads it; rav1e is serial → must go in a Web Worker; the wasm can't
  decode AVIF → `.avif` inputs via `createImageBitmap`→canvas→PNG. Spec = move all conversions into a
  module Worker (main thread stays responsive), enable AVIF output, `.avif` input, the bytes/format/
  saving readout + intent controls. Ship completes STAGE-027. Named assumptions to drive: module
  worker inits the wasm; createImageBitmap decodes AVIF; "progress" = a busy state, not a %.
- [ ] **build** — through a PR, in a WORKTREE. Worker first (module worker + init + one round-trip),
  then AVIF/`.avif`-input/UX. Drive the real browser (responsiveness + valid AVIF). Commit `-s`,
  per-session usd.
- [ ] **verify** — fresh adversarial session (worktree): main thread responsive during a slow AVIF
  encode, PNG→AVIF valid (independent decode), `.avif` input converts, readout correct, still
  client-side, deployed .wasm profiled, live gate passes. **+ CROSS-BROWSER/MOBILE (launch-readiness,
  biggest risk): drive Safari + Firefox + mobile — works or degrades gracefully; confirm module
  Worker / instantiateStreaming / createImageBitmap-AVIF per engine** (`docs/launch-readiness.md`).
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals (per-session usd + ship recorded_at),
  reflection, memory + brag. **Ship completes STAGE-027** → stage-ship reflection; then only SPEC-076
  (gated publish) remains in PROJ-008 → the launch.
