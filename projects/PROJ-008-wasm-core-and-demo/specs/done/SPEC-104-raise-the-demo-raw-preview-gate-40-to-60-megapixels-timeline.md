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
- [x] **verify** (2026-07-25, Opus sub-agent) — VERDICT CLEAN. Mutation-tested the gate (raising it to 63
  flips the 62.4 Mpix bomb test → proves the DEMO gate, not the native cap, rejects); fixture reproducible;
  native paths byte-unchanged; −147 B. `just advance-cycle` mis-targeted a prompt file (the find_spec glob
  bug) → fixed task.cycle by hand. Cost: 95,240 tok (real).
- [x] **ship** (2026-07-25, orchestrator main-loop) — PR #112 squash-merged (after two update-branch
  cycles as main advanced under it); branch deleted; demo redeploys from `main` (no tag) so the 60 MP gate
  goes live and the maintainer can re-drop `L1024678.DNG`. Bookkeeping: real sub-agent token counts (build
  151,255 / verify 95,240), Ship reflection, stage backlog, spec archived, DEC-082 amended, find_spec bug
  filed to tooling-backlog + chip dismissed, memory + brag. Prompts + readouts in
  `prompts/SPEC-104-{build,verify}.md` + `-readouts.md`.
