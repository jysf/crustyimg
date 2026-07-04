# Developing crustyimg — the spec-driven workflow

crustyimg is built with a **spec-driven workflow** where Claude plays every role
(architect, implementer, reviewer) across separate sessions. This document is the
practical guide to that workflow. For the full conventions read [`AGENTS.md`](../AGENTS.md);
for how the template maps to the ContextCore framework, see
[`CONTEXTCORE_ALIGNMENT.md`](CONTEXTCORE_ALIGNMENT.md).

> **Note:** This is about *developing crustyimg*, not *using* it. If you just want to
> install and run the tool, see the [README](../README.md).

## Hierarchy

```
Repo (this app)
 └─ Project (a wave of work: "MVP", "v2 improvements")
     └─ Stage (a coherent chunk within a project)
         └─ Spec (an individual task)
              └─ Cycle (Frame → Design → Build → Verify → Ship)
```

## Getting started

**First time?** Read [`GETTING_STARTED.md`](../GETTING_STARTED.md) — it walks you
through the workflow end-to-end.

**Daily work?** Run `just --list` to see available commands, or `just status` to see
the active project, stage, and specs by cycle.

## Common `just` commands

```sh
just status                        # Active project, stage, specs by cycle
just backlog                       # Spec-grained: what's next in the active stage
just roadmap                       # Stage-grained: where this project is going
just new-spec "title" STAGE-001    # Scaffold a new spec
just advance-cycle SPEC-001 verify # Update a spec's cycle
just archive-spec SPEC-001         # Move a shipped spec to done/
just weekly-review                 # Print the weekly review prompt
just report-daily                  # Generate today's daily report
just report-weekly                 # Generate this week's weekly report
```

## Reports

`just report-daily` and `just report-weekly` generate quantitative snapshots under
`reports/daily/` and `reports/weekly/` from spec front-matter and the git log. Reports
are stand-alone artifacts — re-running overwrites, so they're always a current snapshot.

## Key discipline in this variant

Because Claude plays every role, context contamination is the biggest risk. Four habits
keep it at bay:

1. **New Claude session per cycle** (especially design → build and build → verify)
2. **The spec file is the source of truth** between sessions — no "as I said earlier"
3. **Weekly review is non-optional** (`just weekly-review`)
4. **Honest confidence values** on decisions

See [`AGENTS.md`](../AGENTS.md) section 15 for the full discipline. For the project
brief, see [`projects/PROJ-001-crustyimg-mvp/brief.md`](../projects/PROJ-001-crustyimg-mvp/brief.md).

## Where things live

| Path | Purpose |
|---|---|
| `AGENTS.md` | Conventions for Claude working in this repo |
| `.repo-context.yaml` | Structured metadata about the app |
| `docs/` | Architecture, data model, API contract, this guide |
| `guidance/` | Repo-level rules and open questions |
| `decisions/` | Decision log (accumulates across projects) |
| `projects/` | Each project (wave of work) lives here |
| `projects/*/brief.md` | What this project is and why |
| `projects/*/stages/` | Stages within a project |
| `projects/*/specs/` | Specs within a project (with folded-in Implementation Context) |
| `Cargo.toml` | Crate manifest and pinned dependencies |
| `src/` | The `crustyimg` crate (library modules + `main.rs`) — see [`docs/architecture.md`](architecture.md) |
| `tests/` | Integration tests and native-generated image fixtures |
