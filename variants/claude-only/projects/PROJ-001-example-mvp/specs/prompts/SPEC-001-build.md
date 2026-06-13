# SPEC-001 — Build prompt

> Example prompt file. This is what a fresh build session would run.
> The architect wrote this during Prompt 2b (Spec Design). The
> builder reads this as its only context (plus the files it points at).

```
Cycle: build. You are NOT the architect who wrote this spec. The
spec file is your only context.

Before coding, mark the build cycle [~] in
  projects/PROJ-001-example-mvp/specs/SPEC-001-example-project-logger-timeline.md
If you hit a real blocker (constraint unclear, dependency missing,
scope drift), change it to [?] with a one-line reason and stop.

Read files in order:

1. /AGENTS.md — conventions.
2. /projects/PROJ-001-example-mvp/specs/SPEC-001-example-project-logger.md
   — the spec. Read the ENTIRE "## Implementation Context" section.
3. /projects/PROJ-001-example-mvp/stages/STAGE-001-foundational-infra.md
4. /projects/PROJ-001-example-mvp/brief.md
5. /decisions/DEC-001-example-structured-logging.md
6. /guidance/constraints.yaml — especially no-secrets-in-code,
   test-before-implementation, no-new-top-level-deps-without-decision.

Implement:
- Make the failing tests in `src/lib/log.test.ts` pass.
- Don't violate constraints. If you need to break one, STOP and ask.
- For non-trivial decisions, create decisions/DEC-NNN-<slug>.md.
- If ambiguous, STOP and ask.

When done:
1. Fill the spec's "## Build Completion" section including the three
   reflection questions.
2. Append a build cost session entry to the spec's cost.sessions
   (use /cost if in Claude Code).
3. Run: just advance-cycle SPEC-001 verify
4. Open PR from feat/spec-001-logger-module.
5. PR description: project ID, stage ID, spec ID, decisions used,
   constraints checked, new DEC-* files.
6. Mark build [x] in the timeline with PR number, cost, date.
```
