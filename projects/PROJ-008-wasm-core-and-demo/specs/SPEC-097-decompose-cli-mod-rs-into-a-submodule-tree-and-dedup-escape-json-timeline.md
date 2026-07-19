# SPEC-097 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-097-<cycle>.md`.

## Instructions
- [x] design — framed 2026-07-19 from the pre-launch Rust audit's one unambiguous structural item.
  Split `src/cli/mod.rs` (6,483 lines) into a `cli/` submodule tree (candidate seams: build / report /
  optimize / ops / common, thin `mod.rs` front door + dispatch + re-exports) and dedup the two
  hand-rolled `escape_json` (`cli/mod.rs:2323` + `lint/report.rs:124`). PURE mechanical, byte-identical
  gate; external contract is just `crustyimg::cli::run()`. Framing on a **design branch off main** —
  **build GATED on maintainer review of the approach + the verification gate** (a 6k-line move). Sonnet
  or Opus build (maintainer's call), Opus verify. `run_optimize` arg-bundling explicitly deferred.
