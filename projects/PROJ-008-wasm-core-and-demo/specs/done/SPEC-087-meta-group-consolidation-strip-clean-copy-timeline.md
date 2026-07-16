# SPEC-087 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-087-<cycle>.md`.

## Instructions

- [x] design — framed build-ready 2026-07-15 (surface move within the STAGE-030 freeze).
- [x] build — `meta` group (strip/clean/copy), 3 top-level verbs removed, live-surface grep-clean;
  gates green. Branch `spec-087-meta-group`, PR #91. Flagged: a top-level `set` verb exists
  (spec grounding said otherwise) — left top-level per scope.
- [x] verify — ✅ CLEAN (independent session, own worktree). Byte-identity proven against the OLD
  parent-commit binary (not just the lib op); top-level verbs exit 2; bare `meta` help; lint fix
  fragments run post-cutover; all gates re-run green (test 723/736, clippy, fmt, no-default, validate).
  `set` correctly left top-level per scope.
- [x] ship — squash-merged PR #91 (f7ce015) 2026-07-15 after the CI matrix (27 checks) went green;
  main synced, both worktrees + branch removed. Bookkeeping: cycle→ship, cost sessions (build $2.30 /
  verify $2.10 / ship $0.55 est), STAGE-030 → 4/6, `just archive-spec`, memory + brag. Corrected the
  "no `set` verb" grounding error in the spec Context + STAGE-030 backlog. `meta set` follow-up framed
  per the maintainer decision (fold `set` in → the metadata group becomes whole).
