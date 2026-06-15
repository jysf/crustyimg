# SPEC-008 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-008-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-14
- [ ] **build** — make the `view` command real: thread `--width`/`--height` into
       `Sink::Display`, add `run_view` (mirrors `run_apply`), refuse on non-tty
       (exit 5). Prompt: `prompts/SPEC-008-build.md`. Gates incl.
       `cargo build/clippy --features display` (DEC-011).
