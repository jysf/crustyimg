# crustyimg

A fast, friendly command-line tool for viewing and transforming images —
a clean rebuild of an earlier prototype, built spec-driven from the ground up.

This repo uses a spec-driven workflow where Claude plays every role (architect, implementer, reviewer) across different sessions.

## Hierarchy

```
Repo (this app)
 └─ Project (a wave of work: "MVP", "v2 improvements")
     └─ Stage (a coherent chunk within a project)
         └─ Spec (an individual task)
              └─ Cycle (Frame → Design → Build → Verify → Ship)
```

## Getting started

**First time?** Read `GETTING_STARTED.md` — it walks you through your first project end-to-end.

**Daily work?** Run `just --list` to see available commands.

**Common commands:**
```bash
just status                        # See active project, stage, specs by cycle
just backlog                       # Spec-grained: what's next in the active stage
just roadmap                       # Stage-grained: where this project is going
just new-spec "title" STAGE-001    # Scaffold a new spec
just advance-cycle SPEC-001 verify # Update a spec's cycle
just archive-spec SPEC-001         # Move a shipped spec to done/
just weekly-review                 # Print the weekly review prompt
just report-daily                  # Generate today's daily report
just report-weekly                 # Generate this week's weekly report
just daily-status-report           # Snapshot `just status` to reports/daily/<date>-status.md
```

## Reports

`just report-daily` and `just report-weekly` generate quantitative
snapshots under `reports/daily/` and `reports/weekly/` from spec
front-matter and git log. Daily reports show specs by cycle, value
thesis, cost activity today, and flags. Weekly reports aggregate
ships, cycle times, cost by cycle and interface, and value
advancement. Reports are stand-alone artifacts — re-running
overwrites, so they're always a current snapshot.

## Key discipline in this variant

Because Claude plays every role, context contamination is the biggest risk. Four habits keep it at bay:

1. **New Claude session per cycle** (especially design → build and build → verify)
2. **The spec file is the source of truth** between sessions — no "as I said earlier"
3. **Weekly review is non-optional** (`just weekly-review`)
4. **Honest confidence values** on decisions

See `AGENTS.md` section 15 for the full discipline.

## The app itself

> **Project frame (PROJ-001 — crustyimg MVP):**
>
> - **What:** A Rust CLI that views images directly in the terminal and
>   performs the everyday transformations people actually reach for —
>   resize, shrink/optimize-for-web, thumbnail, strip metadata, inspect
>   info/EXIF — through a clean, composable pipeline with a real
>   subcommand interface.
> - **For:** Developers and power users who want a fast, scriptable
>   alternative to clicking through a GUI (or memorizing ImageMagick
>   incantations) for routine image work — especially preparing images
>   for the web.
> - **Why now:** An earlier prototype proved the feature set is useful
>   but accreted into ~1,000 lines of flag-soup with two competing image
>   models, hardcoded output paths, dead modules, and zero tests. A clean
>   rebuild on a single image model + pipeline architecture, with tests
>   and CI from spec one, turns a throwaway prototype into something
>   shippable (brew / crates.io).
> - **Success:** A user can `crustyimg view`, `info`, `resize`, `shrink`,
>   `thumbnail`, and `strip` real images; each command is tested and
>   green on Linux/macOS/Windows CI; the binary installs cleanly from a
>   release artifact.
>
> Effects/filters (sepia, grayscale, solarize, pixelize, edge-detect) and
> integrations (`open` in Preview/browser, batch over a directory) are a
> deliberate fast-follow — see the stage plan in
> `projects/PROJ-001-crustyimg-mvp/brief.md`.

- **Run it:** see `AGENTS.md` Section 6 (Commands).
- **Tests:** `cargo test` (and `cargo clippy` / `cargo fmt --check`).

## Where things live

| Path | Purpose |
|---|---|
| `AGENTS.md` | Conventions for Claude working in this repo |
| `.repo-context.yaml` | Structured metadata about the app |
| `docs/` | Architecture, data model, API contract |
| `guidance/` | Repo-level rules and open questions |
| `decisions/` | Decision log (accumulates across projects) |
| `projects/` | Each project (wave of work) lives here |
| `projects/*/brief.md` | What this project is and why |
| `projects/*/stages/` | Stages within a project |
| `projects/*/specs/` | Specs within a project (with folded-in Implementation Context) |
| `Cargo.toml` | Crate manifest and pinned dependencies |
| `src/` | The `crustyimg` crate (library modules + `main.rs`) — see `docs/architecture.md` |
| `tests/` | Integration tests and native-generated image fixtures |

## License

Licensed under the Apache License, Version 2.0. See `LICENSE`.
