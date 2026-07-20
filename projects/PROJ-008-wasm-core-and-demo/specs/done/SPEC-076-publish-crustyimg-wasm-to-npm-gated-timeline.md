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
- [x] build — primary checkout @ c366e8c. **The identity mismatch was already fixed** — SPEC-075's
  finalize script + the cut 0.5.0 crate produce `crustyimg-wasm@0.5.0` on the first build (raw wasm-pack
  now emits v0.5.0, not v0.4.0). One real gap found + fixed: the non-canonical `repository.url` npm
  silently normalizes → explicit override in `npm/package.overrides.json` so the dry-run is warning-clean.
  npm README brought current with the wasm surface (added `optimizeDetailed`/`score`, a Caveats section).
  `wasm-npm-smoke` green, `npm publish --dry-run` clean (8 files / 2.0 MB, zero deps, no lifecycle, no
  native addon); native gate green; no `src/` touch. ~$0.40.
- [x] verify — ✅ CLEAN (Opus 4.8, 1M ctx, primary checkout). 7/7 criteria; **both required negative
  controls fired**: override removed → the `repository.url` normalize warning returns; smoke version-assert
  mutated to 0.4.0 → RED. Identity from a fresh build (version derived from Cargo.toml, lockstep-guarded —
  poisoning pkg/ to 0.4.0 → finalize exits 1); README caveats graded live against `src/wasm.rs` (AVIF
  encode-only + can't self-score AVIF, all exports real); nothing published (`crustyimg-wasm` still 404).
  ~$2.0.
- [x] ship — squash-merged PR #107 (**0d3f936**) 2026-07-20, CI CLEAN (full 3-OS matrix). Ships the npm
  README + `repository.url` override to main = `crustyimg-wasm@0.5.0` prepped to the dry-run gate. **No
  publish on merge** — `npm publish` stays maintainer-fired + permanent. No DEC (DEC-067 governs; identity
  a non-event thanks to its lockstep guard). ~$2.7 (build $0.4 / verify $2.0 / ship $0.3). Two gated
  follow-throughs owed once publish is live: fix `demo/index.html:168` npm link + flip the README
  "isn't on npm yet" line.
