# SPEC-013 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-013-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-15. Authored by the ORCHESTRATOR (Opus) directly after two consecutive design-subagent sessions dropped on API socket errors. Emitted **DEC-016** (encode quality policy: `-q` → JPEG quality, ignored for lossless formats, `shrink` default 80). Spec: `shrink` = resize to default max 1600 + quality encode (default 80) + inherent metadata drop, reusing the shipped `Resize` op + the shared `run_pixel_op`; the new work is a quality-aware encode path in `src/sink` (`encode_to_bytes`/`Sink::write` gain a `quality` param) threaded through `run_pixel_op`. Complexity M. Build prompt: `prompts/SPEC-013-build.md`. api-contract shrink entry pinned (defaults + metadata scope).
- [ ] **build** — make the spec's Failing Tests pass; see `prompts/SPEC-013-build.md`. Gates incl. `cargo clippy --all-targets`.
