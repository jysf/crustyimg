# SPEC-012 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-012-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-15 (Opus). Spec authored: thumbnail maps onto the shipped `Resize` op (`--square` ≡ resize `fill` NxN; plain ≡ resize `max` N), default `--size` 256; shared `run_pixel_op` fan-out helper extracted from `run_resize` (both call it). No new Operation, no new DEC. Complexity S. Build prompt: `prompts/SPEC-012-build.md`.
- [ ] **build** — make the spec's `## Failing Tests` pass on a feature branch; run all four gates (`cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`); fill `## Build Completion`; open PR. Prompt: `prompts/SPEC-012-build.md`.
