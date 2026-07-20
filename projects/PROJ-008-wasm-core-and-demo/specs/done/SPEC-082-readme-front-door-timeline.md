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
- [x] build — Sonnet, primary checkout. README restructured: command-first hook, browser-demo link,
  "Why crustyimg" (measured quality / pure-Rust zero-dep / `web` / browser + the 98% headline), install
  de-staled, honest npm caveat, Usage kept. Self-built 47-command sweep (all pass + negative control).
  Applied the anti-AI-voice addendum (avoid-ai-writing skill). **Collision banked: the criterion I added
  mid-build landed on this branch (single-tree); relayed out-of-band.**
- [x] verify — ✅ CLEAN (Opus, independent). Re-derived the 48-command sweep vs a fresh binary (all
  parse+behave; negative control flags injected `shrink`/`strip`/`--frobnicate`); stale/AI-tell greps
  clean; links 200; second voice read clean. **Honesty fix: headline runtime 2–3s → 2–5s to match the
  measured corpus envelope** (207c1d1). `just validate` green.
- [x] ship — squash-merged PR #105 (**543f451**) 2026-07-20, CI CLEAN, after the maintainer's voice
  eyeball. The launch front-door is on main; **it renders on crates.io at the 0.5.0 cut** (next).
  Bookkeeping: cycle→ship, 3 cost sessions (build Sonnet ~$4 / verify Opus $3 / ship $0.5 ≈ **$7.5**),
  timeline, archive, STAGE-028 backlog. No DEC.
