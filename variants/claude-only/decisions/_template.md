---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-XXX                        # stable, never reused
  type: decision                     # decision | analysis | recommendation | observation
  confidence: 0.00                   # 0.0 - 1.0, honest assessment
  audience:                          # who needs to know?
    - developer                      # executive | developer | agent | operator
    - agent

agent:
  id: claude-opus-4-7
  session_id: null

# Decisions are repo-level, but it's useful to track which project
# caused them to be emitted.
project:
  id: PROJ-XXX                       # the project during which this was decided
repo:
  id: __REPO_ID__

created_at: YYYY-MM-DD
supersedes: null                     # DEC-YYY if this replaces a prior decision
superseded_by: null                  # filled in when this decision is replaced

# OPTIONAL. Path globs this decision governs. When present, `just
# decisions-audit` warns if another active decision overlaps the same
# scope, and `just decisions-audit --changed` flags this decision when
# your pending changes touch these paths. `**` spans directories,
# `*` stays within a path segment. Omit if the decision isn't tied to
# specific files.
affected_scope: []                   # e.g. ["src/lib/log.ts", "src/**/*.ts"]

tags:
  - tag-1
  - tag-2
---

# DEC-XXX: <Short Title — what was decided>

## Decision

One sentence stating what was chosen. The most important line in
the file. Make it precise.

## Context

What problem were we solving? What constraints applied? What
triggered this decision (a spec, a bug, a question)?

## Alternatives Considered

- **Option A: <name>**
  - What it is: ...
  - Why rejected: ...

- **Option B: <name>**
  - What it is: ...
  - Why rejected: ...

- **Option C (chosen): <name>**
  - What it is: ...
  - Why selected: ...

## Consequences

- **Positive:** What this unlocks or makes easier.
- **Negative:** What this costs or forecloses.
- **Neutral:** Side effects worth noting.

## Validation

How will we know this decision was right? What would cause us to
revisit it?

## References

- Related specs: SPEC-XXX
- Related decisions: DEC-YYY
- External docs: [links]
- Discussions: [links to GitHub issues, PRs, etc.]
