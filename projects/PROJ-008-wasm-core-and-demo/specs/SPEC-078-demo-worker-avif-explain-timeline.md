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
- [x] **build** (2026-07-13, worktree `crustyimg-wt-spec078`, PR #86) — worker first, as instructed:
  `demo/worker.js` (module Worker) `init()`s the wasm and every conversion goes through it; the
  existing smoke passed against it on day one, proving the `--target web` package inits inside a
  module worker. Then AVIF output enabled (+ Auto/JPEG), the `.avif` input via
  `createImageBitmap`→OffscreenCanvas→PNG, the explain readout, and a spinner (no fake %). `demo-smoke`
  extended: it now auto-attaches to the WORKER's CDP target (which is where the `.wasm` fetch moved),
  drives a real 800×600 PNG→AVIF encode, and proves the main thread stays alive through it — WITH a
  negative control (a deliberate 400 ms freeze reads 0 timers / 0 frames, so the counts are evidence).
  AVIF validity checked by three decoders the crate never met (an ISOBMFF `ispe` parse, Chrome's
  libavif, macOS `sips`). `src/` untouched (0-byte diff). Deviation: **no quality slider** — the
  shipped wasm surface takes no quality argument (DEC-064), so a slider would control nothing; the
  page shows how quality WAS decided instead, and the byte-budget/quality intent is a follow-up.
- [ ] **verify** — fresh adversarial session (worktree): main thread responsive during a slow AVIF
  encode, PNG→AVIF valid (independent decode), `.avif` input converts, readout correct, still
  client-side, deployed .wasm profiled, live gate passes.
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals (per-session usd + ship recorded_at),
  reflection, memory + brag. **Ship completes STAGE-027** → stage-ship reflection; then only SPEC-076
  (gated publish) remains in PROJ-008 → the launch.
