# SPEC-026 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-026-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Activated STAGE-004; authored the spec
  (`## Failing Tests` + `## Implementation Context`), emitted **DEC-029** (pins
  `img-parts` `=0.4.0` + `little_exif` `=0.6.23`, pure-Rust + permissive), added both
  deps to `Cargo.toml`, ran a **design-time probe** (throwaway) confirming strip +
  clean --gps on real JPEG + PNG with byte-identical decoded pixels, and fleshed out
  the `strip`/`clean` entries in `docs/api-contract.md`. v1 = JPEG + PNG; container
  lane only (no pixel re-encode). `just deny` green. Design pushed to `main` before
  build.
- [ ] build — see `prompts/SPEC-026-build.md`.
