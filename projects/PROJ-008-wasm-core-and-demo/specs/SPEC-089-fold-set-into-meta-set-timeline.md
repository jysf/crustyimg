# SPEC-089 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-089-<cycle>.md`.

## Instructions

- [x] design — framed build-ready 2026-07-16. Fold the top-level `set` verb into `meta set`, completing
  the `meta` group SPEC-087 created. Pure hard-cutover surface move (byte-identity vs the pre-move
  binary), mirrors SPEC-087 exactly; the one deliberate divergence is updating the usage-error string
  `set requires …` → `meta set requires …`. No DEC.
- [ ] build — worktree session.
- [ ] verify — independent worktree session.
- [ ] ship — orchestrator.
