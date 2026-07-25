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
- [x] **build** (2026-07-24, Sonnet) — `rawPreview`/`isRawExtension` wasm exports + the
  `MAX_RAW_PREVIEW_MEGAPIXELS = 40` pre-decode gate + demo wiring, all acceptance criteria met.
  PR [#111](https://github.com/jysf/crustyimg/pull/111) (`spec-103-raw-on-wasm`). DEC-082 emitted.
  Real brotli delta: +1,262 B vs the probe's own baseline (1,395,239 B → 1,396,501 B). `just
  wasm-test` (25/25), `just demo-smoke`, `just wasm-npm-smoke`, full native `cargo test` (32/32),
  clippy/fmt/`--no-default-features`/`just validate` all green. Cost: see spec `cost.sessions`
  (build entry recorded null-with-note; orchestrator to fill from the Agent result per AGENTS §4).
  Not merged — held for verify.
- [ ] **verify** — on Opus; adversarial (independent decoder on RAW output, gate-fires-before-decode
  negative control, published-API-untouched, brotli delta vs the probe's +1,214 B).
- [x] **ship** (2026-07-24, orchestrator main-loop) — PR #111 squash-merged (fe66a89) after
  `gh pr update-branch` re-ran the matrix green; branch deleted; demo redeploys from `main` (no tag) so
  RAW support goes live. Bookkeeping done: cost.sessions filled with REAL sub-agent token counts (build
  301,224 / verify 152,014 / ship est), Ship reflection, stage backlog, spec archived, memory + brag.
  Sub-agent prompts + readouts captured in `prompts/SPEC-103-{build,verify}.md` + `-readouts.md`.
