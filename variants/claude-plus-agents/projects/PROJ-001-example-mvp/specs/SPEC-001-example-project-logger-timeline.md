# SPEC-001 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-001-<cycle>.md`.

## Instructions

- [x] **design** — prompt: `prompts/SPEC-001-design.md`
       agent: claude-opus-4-7 (claude-code), completed 2026-04-19

- [ ] **build** — prompt: `prompts/SPEC-001-build.md`
       (will also use `../handoffs/HANDOFF-001-example-project-logger.md`)

- [ ] **verify** — prompt: pending (waiting on build)

- [ ] **ship** — prompt: pending (waiting on verify)
