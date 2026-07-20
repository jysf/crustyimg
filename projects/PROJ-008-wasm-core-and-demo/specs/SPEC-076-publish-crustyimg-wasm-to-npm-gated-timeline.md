# SPEC-076 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-076-<cycle>.md`.

## Instructions
- [x] design — framed 2026-07-20 (maintainer decided to publish for the launch). Publish `crustyimg-wasm`
  as a dual-surface JS/TS library: nail identity (`crustyimg-wasm`, 0.5.0 lockstep — the raw `pkg/` emits
  `crustyimg` v0.4.0), a usage README for the npm page (honest caveats: `init()`, single-threaded, AVIF
  decode via browser), `wasm-npm-smoke` green, `npm publish --dry-run` clean. **The actual `npm publish`
  is [MAINTAINER-AUTHORIZED] + effectively permanent — build stops at the dry-run and hands the maintainer
  the command.** The crustyimg README wasm line flips to a real `npm install` only once published.
  **Build-ready but sequenced with the launch (publishes at/after the 0.5.0 crate cut).** Sonnet build /
  Opus verify. Complexity M.
