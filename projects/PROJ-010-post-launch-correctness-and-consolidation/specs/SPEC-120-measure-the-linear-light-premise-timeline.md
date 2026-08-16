# SPEC-120 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-120-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. 8 ACs, 5 settled design calls, no failing tests (a
      measurement spike). Design found that SSIMULACRA2 cannot score a downscale
      against its source (`report.rs:329`), which reshaped the experiment: it
      needs an independently-generated reference at the target dimensions.
- [~] **build** — prompt: `prompts/SPEC-120-build.md`. **Opus, not Sonnet** — the output is a
      judgment about instrument validity; AC-2's positive control is load-bearing.
      Worktree `../crustyimg-spec120` on `chore/spec-120-measure-linear-light-premise`,
      created 2026-08-16 off `main` at `4b1192c`. **Forked to run in isolation** — no `src/`
      changes, no overlap with SPEC-119, and nothing is blocked on it.
- [ ] **verify** — Opus, new session.
- [ ] **ship**
