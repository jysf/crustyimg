# SPEC-103 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-103-<cycle>.md`.

## Instructions

- [x] **design** (2026-07-24, orchestrator main-loop) — Framed from the completed RAW-on-wasm probe
  (`docs/research/proj-008-raw-on-wasm-probe.md`) after the maintainer chose "wire it in behind a
  permissive pixel gate, tune the threshold on-device post-ship." Full spec + Failing Tests + DEC-082
  plan written. Two open items carried into build: (1) confirm the exact default threshold (framing:
  40 MP) with the maintainer; (2) maintainer draft-review of the two user-facing fallback strings.
  Both RESOLVED 2026-07-24: threshold = **40 MP**, both fallback strings approved verbatim.
- [~] **build** — make the Failing Tests pass; emit DEC-082. Dispatched to Sonnet 2026-07-24.
- [ ] **verify** — on Opus; adversarial (independent decoder on RAW output, gate-fires-before-decode
  negative control, published-API-untouched, brotli delta vs the probe's +1,214 B).
- [ ] **ship** — squash-merge on maintainer go-ahead; demo redeploys from `main` (no tag).
