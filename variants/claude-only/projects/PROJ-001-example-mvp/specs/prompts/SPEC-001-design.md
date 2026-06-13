# SPEC-001 — Design prompt (executed 2026-04-19)

> Example prompt file. This is a record of the prompt the architect
> ran during the design cycle, with SPEC-001's IDs substituted in. A
> real project's design prompts would look similar.

```
Please write SPEC-001 for "Implement the project logger module"
from STAGE-001.

I ran `just new-spec "logger module" STAGE-001`. File at:
projects/PROJ-001-example-mvp/specs/SPEC-001-example-project-logger.md

Cycle starts "design". Set to "build" when complete.

Read first:
- /AGENTS.md
- /projects/PROJ-001-example-mvp/brief.md
- /projects/PROJ-001-example-mvp/stages/STAGE-001-foundational-infra.md
- /docs/architecture.md
- /guidance/constraints.yaml
- /decisions/DEC-001-example-structured-logging.md

When writing:
- Target S. Concrete failing tests: paths + assertions.
- Fill the Implementation Context section with DEC-001, the three
  relevant constraints, and explicit out-of-scope notes.
- Populate value_link referencing STAGE-001's value_contribution.

Then: write prompts/SPEC-001-build.md (prompt for the builder to
read cold), populate the timeline file with [x] design / [ ] build /
[ ] verify / [ ] ship, append a design cost session entry, and run
`just advance-cycle SPEC-001 build`.

Stop and let me review before a fresh build session.
```

## Completion

- Spec populated with acceptance criteria, failing tests, and
  implementation context (DEC-001 + 3 constraints + "out of scope"
  section).
- `prompts/SPEC-001-build.md` written.
- Timeline updated to `[x] design`, `[ ] build`, etc.
- Design cost session entry appended.
