# SPEC-104 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-104-<cycle>.md`.

## Instructions

- [x] **design** (2026-07-24, orchestrator main-loop) — On-device tuning of SPEC-103's gate: the
  maintainer hit the 40 MP fallback on a real desktop Leica `.DNG` (85 MB, ~47 MP preview). Root cause:
  40 was a mobile bound wrongly applied globally. Maintainer chose to raise the single global gate.
  Framed raise 40→60 (clears real Leica bodies, stays below the 64 Mpix native cap so the CLI fallback
  stays honest). Spec + Failing Tests + DEC-082-amendment plan written.
- [~] **build** — one constant + boundary tests + DEC-082 amendment. Dispatched to Sonnet 2026-07-24.
  Prompt: `prompts/SPEC-104-build.md`.
- [ ] **verify** — on Opus; re-drive the [60,64] Mpix straddle + mutation check; confirm native cap +
  fallback copy untouched.
- [ ] **ship** — squash-merge on maintainer go-ahead; demo redeploys from `main` (no tag) so the
  maintainer can re-drop `L1024678.DNG`.
