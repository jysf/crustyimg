# SPEC-029 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-029-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`); the compositing half of STAGE-004 and the first
  multi-image `Operation`. Emitted **DEC-031** (overlay loaded at the IO boundary /
  CLI, op holds in-memory pixels, `apply()` file-free, not in `with_builtins()` —
  recipe round-trip deferred to STAGE-005). A design-time probe confirmed
  `image::imageops` overlay/alpha-opacity/resize/clip primitives — **no new dep**.
  `Gravity` enum + `Watermark` op + `run_watermark` (IO boundary) over `run_pixel_op`.
  Fleshed out the api-contract entry. Design + DEC-031 pushed to `main` before build.
- [x] build (2026-06-18, PR #33) — foreground metered subagent (Opus, 149k tok,
  ~12 min). Added `Gravity` enum (FromStr/Display + placement math) + `Watermark`
  Operation (image::imageops overlay/opacity/scale/tile in RGBA8) + `run_watermark`
  (IO boundary: loads overlay, validates, runs over `run_pixel_op`). 13 new tests
  (8 unit + 5 integration). No file IO in operation/, not in `with_builtins` (DEC-031).
  Repointed a stale NotImplemented stub test watermark→edit. No new dep, no new DEC.
  All gates green incl. the lean build.
- [x] verify (2026-06-18) — independent read-only Explore subagent: ✅ APPROVED,
  no concerns. Confirmed the DEC-031 boundary (no file IO in `src/operation/**`,
  `Watermark` not registered, overlay loaded only in `run_watermark`), compositing
  math (gravity/opacity/scale/tile/clip), exit codes (3/2), all 13 tests substantive,
  no new dep. Orchestrator re-ran gates: `cargo test` 343 ok (0 failed), clippy/fmt/
  deny clean, `cargo build --no-default-features` (lean) clean.
- [x] ship (2026-06-18, PR #33 squash-merged → `f952e1b`) — reflections + cost
  totals filled (build 149046 real / verify ~50k est; totals 199046 / $1.79 / 4),
  STAGE-004 backlog flipped, archived to `specs/done/`, `just cost-audit` green +
  cost-capture + lean build confirmed on main CI.
