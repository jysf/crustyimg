# SPEC-098 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-098-<cycle>.md`.

## Instructions
- [x] design — framed 2026-07-19, closing the Rust audit's D4 (semver-in-toml) thread. Deliverable is a
  DECISION RECORD (`decisions/DEC-078-*.md`), NOT a dependency change: exact `=` pins stay policy for the
  binary today (reproducibility from committed `Cargo.lock`, zero downstream cost with no Cargo consumer);
  relaxing the library-public deps to caret is a MANDATORY, DEFERRED prerequisite of the crates.io publish
  (backlog #5); no migration now; refines (not contradicts) AGENTS.md §5 / DEC-011 / DEC-013. Full DEC
  draft is in the spec. **Framing landed on main 2026-07-19** under **STAGE-031** (maintainer blessed the
  stage + homing); the audit doc landed alongside so the DEC's references resolve. Build (write the DEC +
  cross-refs) gated on maintainer go. Complexity S.
