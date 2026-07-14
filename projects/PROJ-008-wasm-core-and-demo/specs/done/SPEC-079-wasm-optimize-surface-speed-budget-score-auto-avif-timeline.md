# SPEC-079 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-079-<cycle>.md`.

## Cycles

- [x] **design** (2026-07-13) — Framed build-ready in the orchestrator main loop, grounded in a read
  of `decide.rs`/`quality`/`sink`. Surface: `optimizeDetailed(input, out_format, speed?, maxBytes?,
  target?) → OptimizeResult`, a `score(a,b)` binding (added so SPEC-081 stays pure demo), Auto-picks
  AVIF for photos (reuse the SizeBudget admission), a per-call speed via a non-invasive
  `encode_to_bytes_with`. Failing tests written. Emits DEC-068. `optimize`/native/CLI unchanged.
- [x] **build** (2026-07-14) — One worktree session. `sink::encode_to_bytes_with(.., speed)` (AVIF
  only, defaults to `AVIF_SPEED`) with `encode_to_bytes` re-expressed as a thin wrapper;
  `quality::auto_under_size_at_speed` so the byte-budget search probes at the speed the sink emits.
  Proved native-unchanged on **bytes** before touching wasm. 3 documented deviations (Auto-AVIF as a
  bucket predicate not shortlist membership — avoids the `MAX_SHORTLIST` truncation that would drop
  AVIF; `maxBytes` ignored on lossless; `quality()`=Some(80) for default AVIF). Gates green:
  716/726 native, wasm 20/20, clippy ×2, fmt, lean build. Bundle unchanged at 1.33 MB brotli.
- [x] **verify** (2026-07-14) — **CLEAN.** Adversarial, own worktree. All 9 acceptance criteria driven
  against a fresh `just wasm-build` pkg (34/34 Node asserts) + 3 self-written native tests — nothing
  on the build's word. Speed knob proven real (speed 1 = 8862 ms vs 10 = 161 ms, **55×**, different
  bytes); speed-parity re-driven at 1/6/10; AVIF decoded by an outside decoder (`sips`) at the right
  dims; native byte-identical (main vs PR binary); hostile 100000² PNG → typed error, module survived.
  Acceptance table diffed row-by-row (no silent omissions). **Two forward-notes** for SPEC-080/081:
  `score()` can be **negative** (SSIMULACRA2 isn't 0–100); an unsatisfiable byte budget returns
  over-budget bytes **silently** (native CLI warns; the wasm result has no signal).
- [x] **ship** (2026-07-14) — **SHIPPED.** Clean squash-merge **PR #87** (`de6f901`) — no conflict
  (the demo specs are separate files). DEC-068 recorded at build. Cost 250k tok / **$2.65**
  (per-session usd; build $1.10 + verify $1.55; design/ship main-loop null). Ship reflection (the
  "name the exact criterion you reuse" lesson; the quality→sink layering constraint candidate; the
  negative-score surface property). Forward-notes folded into STAGE-029 design notes. Backlog: SPEC-079
  shipped (1/3); SPEC-080 **on hold** pending the strategy reconciliation. `just validate` +
  `just cost-audit` green.
