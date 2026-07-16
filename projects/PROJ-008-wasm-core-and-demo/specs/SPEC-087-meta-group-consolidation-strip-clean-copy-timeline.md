# SPEC-087 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-087-<cycle>.md`.

## Instructions

- [x] design — framed build-ready 2026-07-15 (surface move within the STAGE-030 freeze).
- [x] build — `meta` group (strip/clean/copy), 3 top-level verbs removed, live-surface grep-clean;
  gates green. Branch `spec-087-meta-group`, PR #PENDING. Flagged: a top-level `set` verb exists
  (spec grounding said otherwise) — left top-level per scope.
- [ ] verify — orchestrator (independent session).
- [ ] ship — orchestrator.
