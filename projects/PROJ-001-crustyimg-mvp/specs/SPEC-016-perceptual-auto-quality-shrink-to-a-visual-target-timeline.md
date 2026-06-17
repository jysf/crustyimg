# SPEC-016 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-016-<cycle>.md`.

## Instructions

- [x] **design** — spec + `## Failing Tests` + Implementation Context authored by the ORCHESTRATOR (Opus) directly (SPEC-013/014/015 pattern). Emitted **DEC-019** (ssimulacra2 + perceptual metric/threshold/search policy). Verified the exact `ssimulacra2` 0.5.1 API against docs.rs. Build prompt written to `prompts/SPEC-016-build.md`. Completed 2026-06-16.
- [x] **build** — PR #18 opened. New `src/quality/` module (SSIMULACRA2 metric + generic quality search) + `ssimulacra2` 0.5.1 default dep (`just deny` green, no deny.toml change) + `shrink --target`/`--ssim` wiring; 14 tests; full suite 220 green, all 5 gates pass. Executed by the orchestrator (Opus) as the fallback after the Sonnet background subagent couldn't get Bash permission. Branch `feat/spec-016-perceptual-auto-quality-shrink-to-a-visual-target`. 2026-06-16.
- [ ] **verify** — Opus, read-only: confirm every named test exists + runs; `just deny` green; DEC-019 conformance; auto-quality opt-in (existing `shrink` unchanged); all 5 gates + 3-OS CI.
- [ ] **ship** — orchestrator bookkeeping on `main` after merge (PAUSE for the user before merge/ship).
