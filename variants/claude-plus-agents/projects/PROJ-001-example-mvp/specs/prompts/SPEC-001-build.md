# SPEC-001 — Build prompt

> Example prompt file. What a fresh build session (implementer
> agent — Kilo, Factory, Cursor, an Ollama-backed local model, etc.)
> would run. Points the implementer at the HANDOFF and the spec.

```
You're the implementer for a spec-driven workflow. Cycle: build.

Before coding, mark the build cycle [~] in
  projects/PROJ-001-example-mvp/specs/SPEC-001-example-project-logger-timeline.md
If you hit a real blocker (constraint unclear, dependency missing,
scope drift), change it to [?] with a one-line reason and stop.

Read files in order:

1. /AGENTS.md — conventions.
2. /projects/PROJ-001-example-mvp/handoffs/HANDOFF-001-example-project-logger.md
   — your handoff. Follow its "Context the Receiving Agent Needs"
   section exactly.
3. /projects/PROJ-001-example-mvp/brief.md
4. /projects/PROJ-001-example-mvp/stages/STAGE-001-foundational-infra.md
5. /decisions/DEC-001-example-structured-logging.md
6. /guidance/constraints.yaml
7. /projects/PROJ-001-example-mvp/specs/SPEC-001-example-project-logger.md

Implement:
- Make the failing tests in `src/lib/log.test.ts` pass.
- Don't violate constraints. If you need to break one, STOP and ask.
- For non-trivial decisions (library, pattern, API shape), create
  decisions/DEC-NNN-<slug>.md.
- If ambiguous, STOP and ask.

When done:
1. Fill in the handoff's "Completion" section with the three
   reflection questions.
2. Update handoff.status → "completed".
3. Append a build cost session entry to the spec's cost.sessions.
4. Run: just advance-cycle SPEC-001 verify
5. Open PR from feat/spec-001-logger-module.
6. PR description: project, stage, spec, handoff, decisions,
   constraints, new DEC-* files.
7. Mark build [x] in the timeline with PR number, cost, date.
```
