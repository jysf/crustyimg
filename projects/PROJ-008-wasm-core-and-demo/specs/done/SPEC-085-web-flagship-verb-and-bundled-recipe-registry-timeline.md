# SPEC-085 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-085-<cycle>.md`.

## Cycles

- [x] **design** (2026-07-14) — Framed build-ready; probed the CLI verb wiring + recipe system: `web` =
  `optimize` + a default downscale + always-score (reuse `optimize_pipeline`/`optimize_decide_one`) +
  a bundled-recipe registry. Flagged `web == apply --recipe web` as the load-bearing design decision.
- [x] **build** (2026-07-14, ~30 min) — `web` verb (2048 default, never upscale) reusing the SPEC-084
  engine; `src/recipe/bundled.rs` (`include_str!` web/gallery/product, file-path-wins precedence).
  **Delivered the equivalence** (not descoped) via a terminal-optimize recipe step routing `apply`
  through the fast-decision fan-out — `web` and `apply --recipe web` byte-identical. RAW highlight works.
- [x] **verify** (2026-07-14, ~14 min) — **near-CLEAN.** Reproduced the equivalence byte-identically +
  the corpus flow (8 photos → AVIF@2048, size-insensitive, scored; graphics lossless; RAW; hostile).
  DEFECT: the terminal-optimize apply path ignored `-o <ext>`/`--format` → AVIF-bytes-in-a-`.png`
  (equivalence broken in the pinned corner); + a missing DEC for the recipe-model change.
- [x] **build (fix)** (2026-07-14, ~9 min) — Honored the `-o`/`--format` pin in the terminal-optimize
  apply branch (mirror the verb's `run_pixel_op` diversion; new pinned-equivalence test). Emitted
  **DEC-070** (terminal-op semantics, precedence, build-manifest limitation).
- [x] **verify (re-verify)** (2026-07-15, ~10 min) — **CLEAN.** Pinned + unpinned equivalence
  byte-identical; DEC-070 present; the build-manifest limitation is a clean error (no panic); graphic →
  lossless, hostile input, `optimize`/`preserve`/plain-`apply` regressions + gates all green.
- [x] **ship** (2026-07-15) — **SHIPPED.** PR #89 was BEHIND main → `gh pr update-branch` (re-ran CI) →
  clean squash-merge (`f1e8ba7`). DEC-070 recorded. Cost 475k tok / **$5.25** (build $1.80 + verify $1.40
  + fix $1.00 + re-verify $1.05; design/ship null; session_count 6). Ship reflection (enumerate the
  pinned corner when claiming equivalence; don't drop the cost block). STAGE-030 backlog: SPEC-085
  shipped (2/6). Follow-ups (DEC-070): `build` + terminal-optimize recipes; clearer unknown-name error.
  `just validate` + `just cost-audit` green.
