# SPEC-001 — Design prompt (executed 2026-04-19)

> Example prompt file. Record of the prompt the architect ran during
> the design cycle for SPEC-001 in the claude-plus-agents variant.

```
Please write SPEC-001 for "Implement the project logger module"
from STAGE-001.

I ran `just new-spec "logger module" STAGE-001`. File at:
projects/PROJ-001-example-mvp/specs/SPEC-001-example-project-logger.md

Cycle starts "design". Set to "build" after you also create the
handoff.

Read first:
- /AGENTS.md
- /projects/PROJ-001-example-mvp/brief.md
- /projects/PROJ-001-example-mvp/stages/STAGE-001-foundational-infra.md
- /docs/architecture.md
- /guidance/constraints.yaml
- /decisions/DEC-001-example-structured-logging.md

When writing the spec:
- Target S. Testable acceptance criteria. Concrete failing tests.
- Populate value_link referencing STAGE-001's value_contribution.

Then create the handoff at
  projects/PROJ-001-example-mvp/handoffs/HANDOFF-001-example-project-logger.md

Then write prompts/SPEC-001-build.md for the implementer, populate
the timeline file with [x] design / [ ] build / [ ] verify /
[ ] ship, append a design cost session entry, and run
`just advance-cycle SPEC-001 build`.

Stop and let me review before handoff.
```

## Completion

- Spec populated.
- Handoff-001 written with full context the implementer needs.
- `prompts/SPEC-001-build.md` written.
- Timeline updated.
- Design cost session entry appended.
