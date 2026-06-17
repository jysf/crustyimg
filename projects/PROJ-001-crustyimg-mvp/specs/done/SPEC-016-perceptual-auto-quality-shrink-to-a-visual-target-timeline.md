# SPEC-016 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-016-<cycle>.md`.

## Instructions

- [x] **design** — spec + `## Failing Tests` + Implementation Context authored by the ORCHESTRATOR (Opus) directly (SPEC-013/014/015 pattern). Emitted **DEC-019** (ssimulacra2 + perceptual metric/threshold/search policy). Verified the exact `ssimulacra2` 0.5.1 API against docs.rs. Build prompt written to `prompts/SPEC-016-build.md`. Completed 2026-06-16.
- [x] **build** — PR #18 opened. New `src/quality/` module (SSIMULACRA2 metric + generic quality search) + `ssimulacra2` 0.5.1 default dep (`just deny` green, no deny.toml change) + `shrink --target`/`--ssim` wiring; 14 tests; full suite 220 green, all 5 gates pass. Executed by the orchestrator (Opus) as the fallback after the Sonnet background subagent couldn't get Bash permission. Branch `feat/spec-016-perceptual-auto-quality-shrink-to-a-visual-target`. 2026-06-16.
- [x] **verify** — orchestrator-run independent review (7-agent `/code-review` high, ~511k metered tokens) + gate re-run. ✅ No correctness bugs; surfaced + applied 3 quality fixes (dead memoization removed, `score_jpeg_at`↔`encode_to_bytes` cross-ref comment, unmet-target stderr warning + test). 221 tests; all 5 gates + 3-OS CI + cost-capture audit green. Caveat: same session as design+build (subagent Bash blocked), not a fresh clean-room verify. 2026-06-16.
- [x] **ship** — PR #18 squash-merged to `main` (585b6f9, CLEAN after branch-merge of the concurrent cost-capture work). Orchestrator bookkeeping on `main`: cost sessions (real verify figure + estimated build), ship reflection, archived to `specs/done/`, STAGE-008 backlog updated, brag added. 2026-06-16.
