# How to use this template

A practical, end-to-end walkthrough. The README covers setup and the two
variants; this goes deeper into the daily loop. If you're brand new,
`just init` then open `GETTING_STARTED.md` first — this doc is the
reference you come back to.

## The mental model

```
Repo (the app — persists forever)
 └─ Project (a wave of work: "MVP", "v2", "redesign")
     └─ Stage (a coherent chunk: 2–5 per project)
         └─ Spec (one implementable task)
              └─ Cycle (Frame → Design → Build → Verify → Ship)
```

The repo *is* the app and outlives every project. Architecture,
conventions, constraints, and decisions accumulate at repo level
(`docs/`, `guidance/`, `decisions/`) so they survive across waves of
work. Projects are bounded; specs are individual tasks; the cycle is the
five phases each spec moves through.

## Day 0 — initialize

```bash
just init          # pick claude-only or claude-plus-agents; scaffolds the root
```

Then describe the app once in `.repo-context.yaml`, and skim
`AGENTS.md` — it's the single source of truth every agent reads.

## Start a project, a stage, a spec

A **project** is a brief. Copy `projects/_templates/project-brief.md`
into `projects/PROJ-NNN-<slug>/brief.md` and fill in the value thesis
(see `GETTING_STARTED.md` and Prompt 1a in `FIRST_SESSION_PROMPTS.md`).

A **stage** and a **spec** are scaffolded for you, with IDs and
front-matter filled in:

```bash
just new-stage "Foundational infra"            # → STAGE-NNN in the active project
just new-spec  "Logger module" STAGE-001       # → SPEC-NNN under that stage
```

## The five-phase cycle

Each spec moves through the cycle; `task.cycle` in its front-matter is
the source of truth, advanced with `just advance-cycle`:

```bash
just advance-cycle SPEC-001 design
just advance-cycle SPEC-001 build     # (allowlisted: frame|design|build|verify|ship)
```

| Phase | What happens | Commands / artifacts |
|---|---|---|
| **Frame** | Why this spec exists, acceptance criteria. | Fill the spec; add open questions to `guidance/questions.yaml`. |
| **Design** | The approach, interfaces, trade-offs. | Record non-trivial choices as `DEC-*` in `decisions/`. |
| **Build** | Implement against the spec. | Create `DEC-*` for build decisions; fill `affected_scope`. |
| **Verify** | Acceptance met? tests pass? no decision drift? | `just decisions-audit --changed` (see below). |
| **Ship** | Reflection + cost totals, then archive. | `just archive-spec SPEC-001`. |

Honest confidence matters: decisions carry an `insight.confidence`
(0.0–1.0) that drives questions at design and flags at verify. See
`AGENTS.md` → Confidence Discipline.

## Decisions and guardrails

Architectural decisions live in `decisions/` as `DEC-*` records. Audit
them anytime:

```bash
just decisions-audit             # lint structure + warn on scope conflicts
just decisions-audit --changed   # which decisions govern your pending edits
```

Fill a decision's optional `affected_scope:` (path globs it governs) so
`--changed` can flag it when those paths change — this is the "decision
drift" check in the Verify phase. Repo-wide rules live in
`guidance/constraints.yaml` (each rule has a severity and paths).

## Staying oriented

Four read-only views, each answering a different question:

```bash
just status            # current state: active project/stage, specs by cycle, stale items
just backlog           # spec-grained "what's next" (active stage; --all to widen)
just roadmap           # stage-grained "where is this project going" (counts per stage)
just specs-by-stage    # flat ledger: every spec by stage, ship date + complexity
                       #   defaults to ALL projects; --active or PROJ-NNN to scope
```

And periodic reflection / reporting:

```bash
just weekly-review     # prints the weekly-review prompt with recent activity loaded
just report-daily      # writes reports/daily/YYYY-MM-DD.md
just report-weekly     # writes reports/weekly/YYYY-WNN.md
```

## When you outgrow the defaults

Everything above is zero-dependency. For diagrams, use Mermaid fenced
blocks in markdown (the default — `docs/architecture.md` and
`docs/data-model.md` ship with examples). When a project genuinely needs
more — C4 modeling, protocol-level integration tests — see the optional,
project-level escalations in `guidance/recommended-tools.md`.

## Pointers

- First-time walkthrough: `GETTING_STARTED.md`
- Copy-paste prompts per phase: `FIRST_SESSION_PROMPTS.md`
- Conventions every agent follows: `AGENTS.md`
- All commands: `just --list`
