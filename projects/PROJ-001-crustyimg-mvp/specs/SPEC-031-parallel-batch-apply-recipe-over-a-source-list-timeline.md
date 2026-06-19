# SPEC-031 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-031-<cycle>.md`.

## Instructions

- [x] design (2026-06-19) — Opus, main loop. Activated STAGE-005; authored the spec
  (`## Failing Tests` + `## Implementation Context`); first STAGE-005 spec = the thesis
  payoff (parallel batch `apply --recipe`). Emitted **DEC-033** (indicatif); rayon is
  pre-justified by DEC-006. Added `rayon =1.12.0` + `indicatif =0.18.4` — `just deny` +
  lean build green. Key design pinned: `Operation` is not `Send`, so each rayon task
  REBUILDS its pipeline from the (Sync) recipe + registry; mirror `run_pixel_op`'s
  fan-out + exit-6. Recipe load/validation reused from SPEC-006. Design + DEC-033
  pushed to `main` before build. **Build runs on Sonnet** (prescriptive prompt).
- [ ] build — see `prompts/SPEC-031-build.md`.
