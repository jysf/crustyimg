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
- [x] build — Sonnet, primary checkout. Wrote `decisions/DEC-078-*.md` from the spec draft (near-verbatim)
  + AGENTS.md §5 pointer + STAGE-007 backlog-#5 cross-ref. Good grounding catch: backlog #5 lives in
  STAGE-007 (per DEC-040/041), not `docs/backlog.md`. Docs-only, `git diff --stat` = 4 doc files.
- [x] verify — ✅ CLEAN (orchestrator inline review; docs-only, no runtime surface per the verify-skill
  guidance). Confirmed DEC-078 states all four required points + matches the audit's D4, the STAGE-007
  cross-ref clarifies the publish gate without contradicting "machinery ARMED", DEC-078 is the next free
  number, and zero `Cargo.toml`/`Cargo.lock`/`src/` bytes moved. `just validate` green.
- [x] ship — squash-merged PR #102 (**dd085d5**) 2026-07-19, CI green. DEC-078 records the pinning policy;
  the audit's D4 thread is closed. Bookkeeping: cycle→ship, 3 cost sessions (build Sonnet $1.5 / verify
  $0.3 / ship $0.3 ≈ **$2.1**), timeline, archive, STAGE-031 backlog. Optional micro-follow-up: a DEC-078
  pointer in `RELEASING.md`, fold into backlog #5.
