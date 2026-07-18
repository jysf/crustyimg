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
- [ ] build — single session, primary checkout (demo files only; browser-driven smoke).
- [ ] verify — single session, primary checkout (drive the hero + funnel in headless Chrome).
- [ ] ship — orchestrator.
