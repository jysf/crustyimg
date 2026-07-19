# SPEC-080 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-080-<cycle>.md`.

## Instructions

- [~] design — **REFRAMED build-ready 2026-07-18.** Originally framed (2026-07-13) as a generic
  "make it smaller / Auto" perf-UX fix. Reframed after the STAGE-030 taxonomy reconciliation +
  maintainer product decisions (2026-07-18): the demo hero **IS the shipped `web` flow** (downscale-2048
  + Auto-modernize-to-AVIF + never-bigger + score, at speed 10), **one-click with Advanced hidden**, and
  every conversion becomes a **first-class CLI adoption funnel** (`crustyimg web <file>` + the web.toml
  recipe + copy). Key insight: making downscale-2048 the default dissolves the perf problem (2 MP AVIF at
  speed 10 ≈ 1–2 s, not 33 s) — the timer/warning machinery demotes to the Advanced full-resolution
  fallback. Demo files only; consumes the shipped SPEC-079 surface + SPEC-085 `web`. Complexity M.
- [x] build — demo reframe (hero `web` flow + CLI funnel) + rewritten headless-Chrome smoke. PR #98.
- [x] verify — **✅ APPROVED (2026-07-18).** Hero + funnel + never-bigger + timer driven end-to-end in
  headless Chrome (baseline green); ran the red→green substitute the build owed — mutated the demo to
  break each of the four new smoke checks and confirmed every one FAILS. Spot-checked engine-transform
  geometry vs a real `crustyimg web` (both 2048×1536) + raw-SSIMULACRA2 score honesty. PR #98 CI green
  on the full matrix. No punch list. (~240k tok est / ~$2.16, main-loop.)
- [x] ship — squash-merged PR #98 (**94d4665**) 2026-07-18, full matrix green. The demo is now the
  `web`-flow hero + CLI adoption funnel. Bookkeeping: cycle→ship, 3 cost sessions with `model:` (build
  Opus $4.05 / verify Opus $2.16 / ship $0.70 ≈ **$6.91**), timeline, archive, memory + brag. Process
  incident banked: framed SPEC-095 during 080's verify (single-tree violation) — recovered cleanly, PR
  unaffected. NEXT = build SPEC-095 (wasm q85).
