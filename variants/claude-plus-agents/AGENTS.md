# AGENTS.md — Claude + Implementer Variant

Instructions for any AI agent working in this repository. Read this file first, every session.

> This file contains conventions only. For rules/constraints, see `/guidance/constraints.yaml`. For architectural rationale, see `/decisions/`. For waves of work against this app, see `/projects/`.

---

## 1. Repo Overview

- **Repo (the app):** [REPLACE: My App]
- **Purpose:** [REPLACE: one sentence]
- **Primary stakeholders:** [REPLACE]
- **Active project:** [REPLACE: PROJ-001 — MVP]

See `.repo-context.yaml` for structured metadata.

---

## 2. Work Hierarchy

```
REPO (the app — persists across all projects)
 └─ PROJECT (a wave of work: "MVP", "improvements", "v2 redesign")
     └─ STAGE (a coherent chunk within a project)
         └─ SPEC (an individual task)
              └─ HANDOFF (architect → implementer delegation record)
```

Key distinctions:

- The **repo** is the app. It persists. `AGENTS.md`, `/docs/`, `/guidance/`,
  `/decisions/` live at repo level because they accumulate across all
  projects.
- A **project** (`/projects/PROJ-*/`) is a bounded wave of work. Project
  artifacts (brief, stages, specs, handoffs) live inside the project
  folder.
- A **stage** is an epic-sized chunk within a project. A project typically
  has 2–5 stages.
- A **spec** is a single implementable task. It belongs to exactly one
  stage within one project.
- A **handoff** is an architect-to-implementer delegation document.

**Decisions persist at repo level**, even though they're often made
during a specific project. A decision like "we use pino for logging"
was made during PROJ-001 but binds PROJ-002 and PROJ-003 too. This is
intentional.

**Specs do not cross project boundaries.** If a task isn't finished
when a project ships, either finish it first or defer it explicitly into
the next project's brief.

---

## 3. Business Value

Value structure exists at project and stage levels; specs link lightly.

**Project `value:` block** states the thesis — a testable claim about
what this wave of work delivers. Beneficiaries, success signals, and
risks to the thesis make it falsifiable, not marketing copy.

**Stage `value_contribution:` block** states what this coherent chunk
of work advances, what capabilities it delivers, and what it
explicitly doesn't try to do. Helps avoid stages that seem valuable
but don't contribute to the project thesis.

**Spec `value_link:`** is a one-sentence reference back to the
stage's value. Infrastructure specs may have
`value_link: "infrastructure enabling X"`. Optional but encouraged —
it surfaces specs that don't trace back to the thesis.

Reports (`just report-daily`, `just report-weekly`) aggregate these
signals: which stages advanced the thesis, which specs most directly
delivered it, and where value traceability broke down.

---

## 4. Cost Tracking Discipline

Every cycle on a spec appends a session entry to the spec's
`cost.sessions` list. Agents self-report so reports can aggregate AI
spend over time.

- **Claude Code:** run `/cost` at the end of your session.
- **API calls:** use the `usage` object in the API response.
- **Claude.ai web:** estimate based on session length. Set
  `interface: claude-ai` so reports can distinguish estimates.
- **Third-party agents** (Ollama, Kilo, Factory, etc.): use whatever
  cost mechanism the agent provides. If none, enter null numeric
  values with a note.

Verify cycle flags specs missing cost entries for prior cycles (does
not block the PR — visibility only). Ship cycle computes `cost.totals`
from the session entries.

Reports aggregate cost by cycle, by interface, by spec, and by stage.

---

## 5. Tech Stack

Replace this section with your actual stack. Be specific about versions.

- **Language:** [REPLACE]
- **Runtime:** [REPLACE]
- **Framework:** [REPLACE]
- **Database:** [REPLACE]
- **Testing:** [REPLACE]
- **Linter / Formatter:** [REPLACE]
- **Hosting:** [REPLACE]
- **CI:** [REPLACE]

---

## 6. Commands (exact)

These are the APP's commands. For template/workflow commands, see `justfile`.

```bash
[REPLACE: install command]
[REPLACE: dev command]
[REPLACE: test command]
[REPLACE: test single file command]
[REPLACE: lint command]
[REPLACE: typecheck command]
[REPLACE: build command]
```

---

## 7. Directory Structure

```
/
├── AGENTS.md                          # This file
├── CLAUDE.md                          # Pointer to AGENTS.md
├── README.md                          # Human-facing readme
├── GETTING_STARTED.md                 # First-project walkthrough
├── FIRST_SESSION_PROMPTS.md           # Phase prompts
├── .repo-context.yaml                 # Repo (app) metadata
├── .variant                           # "claude-plus-agents"
├── justfile                           # Commands: just status, just new-spec, etc.
├── scripts/                           # Shell scripts powering justfile
├── docs/                              # Architecture, data model, API contract
├── guidance/                          # Repo-level rules (across all projects)
│   ├── constraints.yaml
│   └── questions.yaml
├── decisions/                         # Repo-level DEC-* (across all projects)
├── feedback/                          # Downstream user feedback captures
├── reports/                           # Daily + weekly report outputs
├── projects/                          # Waves of work
│   ├── _templates/                    # Shared templates
│   │   ├── spec.md
│   │   ├── stage.md
│   │   ├── handoff.md
│   │   └── project-brief.md
│   ├── PROJ-001-<slug>/
│   │   ├── brief.md
│   │   ├── stages/
│   │   ├── specs/
│   │   │   └── done/
│   │   └── handoffs/
│   └── PROJ-002-<slug>/
│       └── (same structure)
└── src/                               # [REPLACE]
```

---

## 8. Cycle Model

Every spec moves through five cycles. **Cycles are tags, not gates** — edit any artifact anytime. The word "cycle" names what a spec goes through on its way to shipping.

| Cycle | Purpose | Who |
|---|---|---|
| **frame** | Go/no-go on the spec | Human + Claude (1 min) |
| **design** | Spec + failing tests + handoff | Claude (architect) |
| **build** | Make failing tests pass | Implementer agent |
| **verify** | Review + validation | Claude (reviewer) |
| **ship** | Merge, deploy, reflect, archive | Human + light agent |

Valid transitions:
```
frame → design → build → verify → ship
                   ↑       │
                   └───────┘ (verify sends back on punch list)
```

Projects and stages have lighter lifecycles (not full cycles):

- **Project status:** `proposed | active | shipped | cancelled`
- **Stage status:** `proposed | active | shipped | cancelled | on_hold`

A stage is `active` when its first spec enters design. `shipped` when
its spec backlog is complete AND the stage-level reflection is written.

---

## 9. Instruction Timeline

Every spec has a timeline file at
`projects/*/specs/SPEC-NNN-<slug>-timeline.md` listing cycle
instructions in order with status markers.

Status markers:

- `[ ]` not started — no one has picked this up yet
- `[~]` in progress — an executor is currently running this
- `[x]` complete — cycle finished; see the prompt file for what was run
- `[?]` blocked — needs a human decision or external unblock before
  proceeding. Include a one-line reason after the marker.

Cycle prompts live at `projects/*/specs/prompts/SPEC-NNN-<cycle>.md`.
The architect writes them; executors (the implementer agent for
build, Claude again for verify) read and run them.

**Discipline for executors:**

- When you start a cycle, mark it `[~]`.
- When you finish, mark it `[x]` with a one-line result (PR number,
  cost, completion date).
- If you hit a real blocker — constraint ambiguous, dependency
  missing, verify surfaced something needing architect judgment —
  mark `[?]` with a one-line reason. Do NOT use `[?]` as a "I don't
  know what to do" dumping ground. Blocked means the next move
  requires someone else; everything else is in-progress or a
  question to resolve in the current session.

This is a convention, not a mechanism. No tooling enforces it; the
discipline lives in the prompt set. Skip it and nothing breaks, but
you lose the history artifact and the next executor has to hunt for
the right prompt.

---

## 10. Cross-Reference Rules

Every spec has these relationships, encoded in front-matter:

- `project.id` → the project it belongs to (e.g., `PROJ-001`)
- `project.stage` → the stage within that project (e.g., `STAGE-002`)
- `references.decisions` → DEC-* it was designed against
- `references.constraints` → constraints that apply
- `handoff.from_agent` / `handoff.to_agent` → roles in the delegation

When a spec references a DEC, the DEC does not reciprocally list the
spec. DECs are stable repo-level records; specs come and go.

---

## 11. Coding Conventions

- **Naming:** [REPLACE]
- **File organization:** [REPLACE]
- **Imports:** [REPLACE]
- **Error handling:** [REPLACE]
- **Logging:** [REPLACE]
- **Comments:** Explain *why*, not *what*.
- **No dead code.** Delete, don't comment out.
- **Diagrams:** author them as Mermaid fenced blocks in markdown
  (`/docs/`, `/decisions/`, specs) so they render on GitHub and you can
  keep them current as part of the work. Update the relevant diagram in
  the same change, not afterward. See `/guidance/recommended-tools.md`.

---

## 12. Testing Conventions

- Every new function gets at least one test.
- Test file naming: [REPLACE]
- Coverage expectations: [REPLACE]
- Must test: happy path, error cases, edge cases from acceptance criteria.
- Need not test: third-party internals, framework behavior.
- **TDD:** Tests live in the spec's `## Failing Tests` section, written
  during **design**, made to pass during **build**.

---

## 13. Git and PR Conventions

- **Branch:** `feat/spec-NNN-<slug>`, `fix/spec-NNN-<slug>`, `chore/<slug>`
- **One spec per branch, one PR per branch.**
- **Commits:** [REPLACE: e.g., Conventional Commits]
- **PR description must include:**
  - Project: `PROJ-NNN`
  - Stage: `STAGE-NNN`
  - Spec: `SPEC-NNN`
  - Handoff: `HANDOFF-NNN`
  - Decisions referenced: `DEC-NNN, DEC-MMM`
  - Constraints checked: `[list]`
  - New `DEC-*` files created during build

---

## 14. Domain Glossary

- **[REPLACE: Term]** — [REPLACE: Definition]

---

## 15. Cycle-Specific Agent Rules

### During **build** (implementer reads this)

Before writing code:
1. Read the `/projects/PROJ-*/handoffs/HANDOFF-*.md` for your spec.
2. Read the linked `SPEC-*.md`, `STAGE-*.md`, and the project's `brief.md`.
3. Read every `DEC-*` listed in the handoff's references.
4. Read `/guidance/constraints.yaml`; check rules for paths you'll touch.
5. If anything is ambiguous, add to `/guidance/questions.yaml` and stop.

When done:
1. Fill in the handoff's `## Completion` section (including reflection).
2. Update `handoff.status` → `completed`; update spec's `task.cycle` → `verify`.
3. Append a build cost session entry to the spec's `cost.sessions`.
4. Create `DEC-*` files for non-trivial implementer decisions. When a
   decision is tied to specific code, fill in its `affected_scope`
   with the path globs it governs (e.g. `src/lib/log.ts`,
   `src/api/**`). This is required for file-bound decisions — it's
   what lets `just decisions-audit --changed` surface the decision
   when those paths change later. Leave `affected_scope: []` only for
   decisions not tied to particular files (e.g. a process choice).
5. Open PR following Section 13.

Shortcut: `just advance-cycle SPEC-NNN verify`.

### During **verify** (reviewer reads this)

Check:
1. Acceptance criteria all met and tested?
2. Failing tests from spec now pass?
3. No drift from referenced decisions?
4. No constraint violations?
5. Non-trivial implementer choices have accompanying `DEC-*`?
6. Implementer reflection answered (not mailed in)?
7. `cost.sessions` has entries for prior cycles? Flag if missing
   (don't block).

For check 3, run `just decisions-audit --changed` — it flags which
`DEC-*` records govern the files the implementer touched, so you can
confirm the work stayed consistent with them. `just decisions-audit`
(no flag) lints the records themselves. See `/guidance/recommended-tools.md`
for optional, heavier verify tooling (e.g. LineSpec for protocol-level
integration tests).

Append a verify cost session entry before returning the verdict.

Output: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### During **ship**

Append a `## Reflection` block to the spec with three answers:
1. What would I do differently next time?
2. Does any template, constraint, or decision need updating?
3. Is there a follow-up spec to write now?

Then:
- Update the spec's `task.cycle` → `ship`.
- Append a ship cost session entry, then compute `cost.totals`.
- Run `just archive-spec SPEC-NNN` (moves to `done/`, updates stage).
- If stage backlog is complete, run the Stage Ship prompt.
- Commit.

---

## 16. Confidence Discipline

Decisions in `/decisions/` have an `insight.confidence` field (0.0–1.0).
Honest values matter — they drive these behaviors:

- **Design phase:** if Claude emits a decision at confidence < 0.7, it
  also adds an entry to `/guidance/questions.yaml` flagging it for
  further investigation.
- **Verify phase:** if a spec references any decision at confidence < 0.6,
  that's a yellow flag worth surfacing in the review.
- **Weekly review:** all decisions at confidence < 0.8 are listed with
  a note on whether recent work has strengthened or weakened them.

Use 1.0 only for decisions that are truly locked (tech stack choice
after it's been installed and working, for example). Most decisions
should land between 0.7 and 0.95.

---

## 17. Pointers

- Constraints: `/guidance/constraints.yaml`
- Open questions: `/guidance/questions.yaml`
- Decisions: `/decisions/` (audit with `just decisions-audit`)
- Recommended (optional) tools: `/guidance/recommended-tools.md`
- Projects: `/projects/`
- Templates: `/projects/_templates/`
- What we're building (architecture): `/docs/architecture.md`
- Feedback: `/feedback/`
- Reports: `/reports/` (daily, weekly)
- Timelines: `/projects/*/specs/SPEC-NNN-*-timeline.md` (per-spec)
- Cycle prompts: `/projects/*/specs/prompts/`
- Phase prompts: `/FIRST_SESSION_PROMPTS.md`
- First walkthrough: `/GETTING_STARTED.md`
- Daily commands: run `just --list`
