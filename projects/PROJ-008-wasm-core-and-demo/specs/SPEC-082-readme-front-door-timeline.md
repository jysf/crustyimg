# SPEC-082 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-082-<cycle>.md`.

## Instructions
- [x] design — framed build-ready 2026-07-19 (activates STAGE-028). Turn the README into the launch
  front-door + de-stale it, shipping in 0.5.0 (it renders on crates.io). Gaps: (1) stale "Once v0.1.0 is
  published" install split — we've been published since v0.1.0 (same false premise SPEC-099 swept, README
  missed); (2) NO browser-demo link (jysf.github.io/crustyimg — the best "watch it just work" hook); (3)
  no wasm/library story; (4) no why-vs-sharp/squoosh/imagemagick positioning. Keep the current Usage
  section (already frozen-CLI-correct). **Verify = a commands-and-claims sweep: run every fenced
  `crustyimg …` command against the 0.5.0 binary; grep out stale-publish claims.** Honesty: link the LIVE
  demo, do NOT write `npm install crustyimg-wasm` (unpublished), attribute the benchmark number. Sonnet
  build / Opus verify; **maintainer eyeballs the final voice** before it ships. Complexity M.
