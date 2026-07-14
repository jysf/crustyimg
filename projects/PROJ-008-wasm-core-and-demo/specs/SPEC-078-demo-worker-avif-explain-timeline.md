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
- [x] **verify** (2026-07-13, worktree `crustyimg-wt-verify078`) — **CLEAN on the engineering; the
  cross-browser criterion was NOT met by the build and is now met in part.** Drove the real page with
  a CDP/BiDi/WebDriver client written for this pass, not the build's smoke. **Chrome 150** 17/17,
  **Firefox 150** (real Gecko, BiDi) 9/9, **Safari 26.5** (real WebKit, safaridriver) 8/8 — all three
  do module Worker + `instantiateStreaming` + `createImageBitmap`-decodes-AVIF, and all three stayed
  responsive through a real ~3.1 s 1600×1200 PNG→AVIF encode (Chrome 311 timers/295 frames, FF
  274/392, Safari 260/187) against a **0/0 negative control** — the build's control reproduces, so the
  counts are evidence. AVIF validity re-judged by three outside decoders (`sips`, Chrome libavif, a
  from-the-spec ISOBMFF `ispe` parse); confirmed the engine itself *refuses* its own AVIF, which is
  what makes the outside opinion necessary. Zero network from page *or* worker. No dead quality slider
  (0 range inputs); Auto is real — it chose **jpeg** for a noisy photo, i.e. the engine's analysis, not
  a stub. `src/` diff vs main = 0 bytes; `just check`/`deny`/`validate` green; the size-profile guard
  **bites** (a forged fat-`name`-section artifact was refused, exit 1). **Findings:** (1) the build's
  completion table silently omitted the cross-browser criterion while claiming all met, and never
  drove a non-Chrome engine — now driven, and the matrix the spec asked for is recorded in
  `demo/README.md`; (2) **mobile (iOS Safari / Android Chrome) is still UNVERIFIED** — undrivable here
  (no simulator/SDK), stays a launch-checklist item; (3) the branch is 2 commits behind main → needs
  `gh pr update-branch` before merge.
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals (per-session usd + ship recorded_at),
  reflection, memory + brag. **Ship completes STAGE-027** → stage-ship reflection; then only SPEC-076
  (gated publish) remains in PROJ-008 → the launch.
