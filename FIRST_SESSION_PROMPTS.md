# First-Session Prompts — Claude-Only Variant

Copy-paste-ready prompts for each cycle. Fill in bracketed parts before pasting.

**Prompts at a glance:**
- **1a:** Project Frame
- **1b:** Project Brief
- **1c:** Stage Frame
- **1d:** Stage Ship
- **1e:** Project Ship
- **2a:** Repo/Project Design
- **2b:** Spec Design
- **3:** Build (fresh session!)
- **4:** Verify (fresh session!)
- **5:** Ship
- **6:** Weekly Review

> **Session discipline:** Claude plays every role. Start a NEW Claude
> session for each cycle. The spec file is the handoff between sessions.
> Skipping this breaks the variant.

---

## Prompt 1a — PROJECT FRAME

> **Use when:** Starting a new project.
> **Time:** 5 min.

```
I want to frame a potential project before committing to it.

Prior projects in this repo:
[REPLACE: list prior projects or "none, this is the first"]

My raw idea for this project: [REPLACE: 1-3 sentences]

Produce a one-paragraph frame:
- What: what this wave of work delivers
- For: who the user is
- Why now: what makes this the right wave, now
- Success: one concrete outcome that would mean it worked

Your task:
1. Ask clarifying questions. Be skeptical.
2. Produce the one-paragraph frame.
3. Give a go/kill recommendation with reasoning.
4. Suggest a short-slug name for the project directory.

If not ready, say "kill" and explain.
```

---

## Prompt 1b — PROJECT BRIEF

> **Use when:** Frame approved.
> **Time:** 15 min.

```
Project frame approved. Produce the full brief.md.

Read first:
- /README.md
- /AGENTS.md (Work Hierarchy section)
- /.repo-context.yaml
- /projects/_templates/project-brief.md
- /projects/PROJ-001-example-mvp/brief.md (delete later)
- Any existing shipped projects

Project frame (from 1a):
[REPLACE: paste frame]

Your task:

1. Determine next PROJ-ID. First project: PROJ-001.

2. Ask clarifying questions:
   - Target ship date?
   - Anticipated stages (3-5 typical)?
   - Dependencies on prior projects?

3. Create the project directory:
   projects/PROJ-NNN-<slug>/
     ├── brief.md
     ├── stages/
     └── specs/
         └── done/

4. Populate brief.md with:
   - "What This Project Is" paragraph
   - "Why Now" justification
   - Success criteria (3-5 concrete)
   - Scope (in/out)
   - Stage Plan (2-5 stages)
   - Dependencies
   - `value:` block in front-matter: thesis (one sentence,
     testable claim), beneficiaries (2-4), success_signals (3-5
     observable outcomes), risks_to_thesis (2-4 honest things
     that could make the thesis wrong). Be specific. "Users
     will love it" is not a thesis; "reducing month-2 churn by
     making activation faster" is.

5. If the example project folder still exists
   (projects/PROJ-001-example-mvp/), propose deleting it.

Stop and let me review.
```

---

## Prompt 1c — STAGE FRAME

> **Use when:** Starting a new stage.
> **Time:** 15 min.

```
I want to frame a new stage in the active project.

Read first:
- /AGENTS.md
- /projects/<active-project>/brief.md
- /projects/_templates/stage.md
- Any already-shipped or in-progress stages
- Relevant /decisions/DEC-*.md

I ran `just new-stage "<title>"` which created:
[REPLACE: paste path]

The stage:
- Title: [REPLACE]
- Why this stage is next: [REPLACE]
- Dependencies: [REPLACE or "unknown"]

Your task:

1. Ask clarifying questions. Focus: what does "done" mean?
   What does it unblock?

2. Populate the stage file:
   - "What This Stage Is" paragraph
   - "Why Now"
   - Success criteria (3-5)
   - Scope (in/out)
   - Proposed Spec Backlog (3-8 specs, S/M/L)
   - Design Notes
   - Dependencies
   - `value_contribution:` block in front-matter: advances
     (which part of the project's value.thesis this stage
     advances), delivers (user-visible capabilities when done),
     explicitly_does_not (what this stage is NOT trying to do —
     those are other stages' jobs). If you can't articulate
     value_contribution, the stage may be infrastructure-only —
     acceptable but flag it.

3. Flag any complexity-L entries.

4. If thin (<3) or sprawling (>8), recommend rescoping.

Stop and let me review.
```

---

## Prompt 1d — STAGE SHIP

> **Use when:** All specs in a stage shipped.
> **Time:** 15-20 min.

```
STAGE-NNN is ready to ship. All specs in done/.

Read:
- /projects/<active-project>/stages/STAGE-NNN-<slug>.md
- Every shipped spec that belongs to this stage
- Their Reflection sections (both build-phase and ship)

Your task:

1. Check Success Criteria against what shipped. Did we deliver?

2. Review shipped specs against the stage's `value_contribution`.
   Flag any spec whose `value_link` didn't actually deliver what it
   claimed.

3. Summarize in 3 sentences: built vs planned, speed, emergent behavior.

4. Propose answers for ## Stage-Level Reflection.

5. Set the stage front-matter to reflect shipping:
   - `stage.status: shipped`
   - `shipped_at: <YYYY-MM-DD>` (today)
   This is how `just roadmap` will render this stage as completed.

6. Flag follow-up: new stage here? spec in next stage? defer to future
   project?

7. Propose updates to /AGENTS.md, /guidance/*, or templates.

I'll review and write proposals into the stage file.
```

---

## Prompt 1e — PROJECT SHIP

> **Use when:** All stages in a project shipped.
> **Time:** 20-30 min.

```
PROJ-NNN is ready to ship. All stages shipped.

Read:
- /projects/PROJ-NNN-<slug>/brief.md
- Every stage in the project
- Every shipped spec
- Their Reflection sections
- /decisions/DEC-* emitted during this project

Your task:

1. Check project Success Criteria. Did the wave of work deliver?

2. Cross-check shipped stages' `value_contribution` against the
   project's `value.thesis`. Did the thesis hold? Would you refine
   it based on what shipped?

3. Summarize in 3-5 sentences: scope evolution, smooth vs painful
   stages, major superseded decisions.

4. Propose answers for ## Project-Level Reflection in brief.md.

5. Identify deferred work for the next project's frame.

6. Recommend: mark project shipped and start PROJ-NN+1, or
   extend into "phase 2"?

7. Propose updates to /AGENTS.md, /guidance/*, or templates.

I'll review and write into the brief.
```

---

## Prompt 2a — REPO/PROJECT DESIGN

> **Use when:** Project brief + first stage approved.
> **Time:** 60-90 min.

```
Project brief and first stage approved. Repo-level design.

Read:
1. /README.md
2. /AGENTS.md
3. /.repo-context.yaml
4. /guidance/constraints.yaml
5. /decisions/DEC-001-example-structured-logging.md
6. /decisions/_template.md
7. /projects/_templates/spec.md
8. /projects/<active-project>/brief.md
9. /projects/<active-project>/stages/<first-stage>.md

Hard constraints:
- Tech choices forced: [REPLACE or "none"]
- Timeline: [REPLACE]
- Compliance: [REPLACE or "none"]
- Non-goals: [REPLACE]

Your task:

1. Ask clarifying questions.

2. Populate repo-level docs:
   - /docs/architecture.md (with Mermaid)
   - /docs/data-model.md (or delete if N/A)
   - /docs/api-contract.md (or delete if N/A)

3. For every meaningful decision, create
   /decisions/DEC-NNN-<slug>.md. Honest confidence.

4. Update /guidance/constraints.yaml.

5. Update /.repo-context.yaml.

6. Update /AGENTS.md: real tech stack, commands, conventions, glossary.

7. Delete examples:
   - /decisions/DEC-001-example-structured-logging.md
   - /projects/<active-project>/specs/SPEC-001-example-project-logger.md

8. Cross-check first stage's backlog against new architecture.

Stop after step 8.
```

---

## Prompt 2b — SPEC DESIGN

> **Use when:** Writing one spec.
> **Time:** 15-30 min.

```
Please write SPEC-NNN for "[REPLACE: title]" from STAGE-MMM.

I ran `just new-spec "<title>" STAGE-MMM`. File at:
[REPLACE: paste path]

Cycle starts "design". Set to "build" when complete.

Read first:
- /AGENTS.md
- /projects/<active-project>/brief.md
- /projects/<active-project>/stages/STAGE-MMM-<slug>.md
- /docs/architecture.md
- /docs/data-model.md, /docs/api-contract.md (if applicable)
- /guidance/constraints.yaml
- All /decisions/DEC-*.md relevant
- Related shipped specs

When writing:
- Target S or M. Split L.
- Testable acceptance criteria.
- Concrete failing tests: paths + assertions.
- Set project.stage to STAGE-MMM in front-matter.
- Fill the "## Implementation Context" section carefully. The build
  session won't have design-session context; this section must be
  self-contained. Include:
  * Decisions that apply (DEC-NNN + why here)
  * Constraints that apply (IDs + one-line each)
  * Prior related work (shipped specs + PRs)
  * Out of scope for this spec
- Emit new /decisions/DEC-*.md files for new decisions made here.
- Populate `value_link:` at the bottom of front-matter. One
  sentence on what this spec contributes to its stage's
  value_contribution. If the spec is infrastructure with no
  direct user-visible contribution, write "infrastructure
  enabling STAGE-XXX's <capability>". Leave null only if
  genuinely unknown.

Then: `just advance-cycle SPEC-NNN build` (or update front-matter
manually) and update stage's Spec Backlog.

Before you stop, append a design cost session entry to
`cost.sessions`. If you're in Claude Code, run `/cost` first and
use its numbers. On Claude.ai web, set `interface: claude-ai` and
null the numeric fields (or best-guess). If your agent doesn't
report cost, null fields + a notes line.

  - cycle: design
    agent: <your model, e.g. claude-opus-4-7>
    interface: <claude-code | claude-ai | api | ollama | other>
    tokens_input: <from usage if known, else null>
    tokens_output: <same>
    estimated_usd: <best available, or null>
    duration_minutes: <estimate>
    recorded_at: <YYYY-MM-DD>
    notes: <one line if unusual, else null>

Scaffold the cycle timeline. Write the build prompt to
  projects/<active-project>/specs/prompts/SPEC-NNN-build.md
Base it on Prompt 3 below with SPEC-NNN, stage ID, and paths
substituted. It is the only thing the build executor will read.

Then replace the placeholder block in
  projects/<active-project>/specs/SPEC-NNN-<slug>-timeline.md
with:

  ## Instructions

  - [x] **design** — completed <YYYY-MM-DD>
  - [ ] **build** — prompt: `prompts/SPEC-NNN-build.md`
  - [ ] **verify** — prompt: pending (waiting on build)
  - [ ] **ship** — prompt: pending (waiting on verify)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete
· `[?]` blocked (one-line reason, needs human or external unblock).

Stop and let me review before a fresh build session.
```

---

## Prompt 3 — BUILD (fresh session!)

> **Use when:** Spec ready. START A NEW CLAUDE SESSION.
> **Time:** Task-dependent.

```
Cycle: build. You are NOT the architect who wrote this spec. The spec
file is your only context.

Read files in order:

1. /AGENTS.md — conventions.
2. /projects/<active-project>/specs/SPEC-NNN-<slug>.md — the spec.
   Read the ENTIRE "## Implementation Context" section carefully;
   it contains the decisions, constraints, and prior work you need.
3. /projects/<active-project>/stages/STAGE-MMM-<slug>.md — the stage.
4. /projects/<active-project>/brief.md — the project.
5. Every decision in the spec's references.
6. /guidance/constraints.yaml — constraints for paths you'll touch.

Before coding, mark the build cycle `[~]` in
  projects/<active-project>/specs/SPEC-NNN-<slug>-timeline.md
so anyone checking can see this cycle is live. If you hit something
that needs architect judgment or an external unblock (constraint
unclear, dependency missing, scope drift), change it to `[?]` with
a one-line reason and stop. `[?]` is NOT a dumping ground for "I
don't know what to do" — ask first if unsure.

Implement:
- Make the failing tests pass.
- Don't violate constraints. If one needs breaking, STOP and ask.
- For non-trivial decisions, create /decisions/DEC-NNN-<slug>.md
  with honest confidence.
- If ambiguous, STOP and ask. Don't guess.

When done:
1. Fill in the spec's "## Build Completion" section INCLUDING the
   three build-phase reflection questions. Not optional.
2. Append a build cost session entry to the spec's `cost.sessions`:

     - cycle: build
       agent: <your model>
       interface: <claude-code | claude-ai | api | ollama | other>
       tokens_input: <best available>
       tokens_output: <best available>
       estimated_usd: <best available>
       duration_minutes: <estimate>
       recorded_at: <YYYY-MM-DD>
       notes: <one line if rework or unusual, else null>

   In Claude Code: run `/cost`, use its numbers. API: use the
   `usage` object. Claude.ai web: best-guess. Agent that doesn't
   report: null fields + `notes: "agent does not report cost"`.
3. Run: just advance-cycle SPEC-NNN verify
4. Open PR from feat/spec-NNN-<slug>.
5. PR description: project ID, stage ID, spec ID, decisions used,
   constraints checked, new DEC-* files.
6. Mark build `[x]` in the timeline with PR number, cost, and date:
     - [x] **build** — prompt: `prompts/SPEC-NNN-build.md`
            PR #NNN, $X.XX, completed <YYYY-MM-DD>
```

---

## Prompt 4 — VERIFY (another fresh session!)

> **Use when:** Build complete. ANOTHER NEW SESSION.
> **Time:** 10-30 min.

```
Cycle: verify. You are NOT the architect or implementer. Reviewing
SPEC-NNN's PR cold.

Before reading, mark verify `[~]` in
  projects/<active-project>/specs/SPEC-NNN-<slug>-timeline.md

Review: [REPLACE: paste PR link or diff]

Read against:
- /projects/<active-project>/specs/SPEC-NNN-<slug>.md (acceptance
  criteria? tests pass? build completion filled in honestly?)
- /projects/<active-project>/stages/STAGE-MMM-<slug>.md (advances
  stage as intended?)
- Decisions in spec references (drift? superseding DEC-*?)
- /guidance/constraints.yaml (violations?)

Flag:
- Untested acceptance criteria
- Decision drift without supersession
- Constraint violations
- Non-trivial choices missing DEC-*
- Mailed-in build reflection ("nothing was unclear" is suspicious —
  that often means same-session build+design contamination)
- Decisions referenced at confidence < 0.6 (yellow flag)
- Follow-up specs implied
- **Missing cost data:** `cost.sessions` lacks entries for prior
  cycles (design, build). Flag so missing agents can fill in
  retroactively. Do not block the PR for this.

Before returning your verdict, append your own verify cost session
entry to the spec's `cost.sessions` (same format as Design, but
`cycle: verify`).

Output exactly ONE of:

✅ APPROVED — merge at commit <SHA>.

⚠ PUNCH LIST:
   1. [specific]
   2. [specific]

❌ REJECTED because [reason].
   Recommended: [revise spec | split | revisit design].

If ✅ APPROVED: write the ship prompt to
  projects/<active-project>/specs/prompts/SPEC-NNN-ship.md
(base on Prompt 5 below with IDs/paths substituted), then mark
verify `[x]` in the timeline with the approved SHA, and add the
ship line referencing `prompts/SPEC-NNN-ship.md`.

If ⚠ or ❌: leave verify `[~]` if the builder needs to rework in
the same cycle, or mark `[?]` with a one-line reason if architect
judgment is required (e.g. "acceptance criteria ambiguous, re-design
needed").
```

---

## Prompt 5 — SHIP

> **Use when:** Approved.
> **Time:** 5-10 min.

```
Cycle: ship. PR for SPEC-NNN approved.

Pre-ship checklist:
[ ] CI passing?
[ ] Deployment steps?
[ ] Rollback plan?
[ ] CHANGELOG?

After merge + deploy, answer three reflection questions.
Append as "## Reflection (Ship)" block (separate from the
build-phase reflection, which is process-focused; these are
outcome-focused):

1. What would I do differently next time?
   [REPLACE: answer]

2. Does any template, constraint, or decision need updating?
   [REPLACE: answer]

3. Is there a follow-up spec to write before I forget?
   [REPLACE: answer]

Before starting, mark ship `[~]` in
  projects/<active-project>/specs/SPEC-NNN-<slug>-timeline.md

After I paste:
- Format as ## Reflection (Ship) block
- Append a ship cost session entry to `cost.sessions` (same format
  as Design, `cycle: ship`).
- Compute `cost.totals` from the session entries:
  * `tokens_total` = sum(tokens_input + tokens_output) across
    sessions, skipping nulls
  * `estimated_usd` = sum(estimated_usd) skipping nulls
  * `session_count` = len(sessions) (include sessions with null
    numeric fields)
  If any session had nulls, that's fine — reports will show
  "partial cost data available" rather than missing.
- Mark ship `[x]` in the timeline with merge date + total cost.
- Run: just advance-cycle SPEC-NNN ship
- Run: just archive-spec SPEC-NNN  (also moves timeline into done/)
- If template/constraint/decision updates mentioned, propose edits
- If follow-up spec mentioned, add to stage's backlog
- Update parent STAGE's backlog

If LAST spec in STAGE-MMM's backlog, remind me to run Prompt 1d.
```

---

## Prompt 6 — WEEKLY REVIEW (non-optional here)

> **Use when:** Once a week, always.
> **Time:** 20-30 min.
>
> **Shortcut:** `just weekly-review` pre-loads context — paste its
> output as the prompt body.

```
Weekly review. Non-optional in claude-only variant because without
a second agent, drift compounds silently.

Read:
- All /decisions/DEC-*.md
- /guidance/constraints.yaml, /guidance/questions.yaml
- /AGENTS.md
- /projects/<active-project>/brief.md
- All stages in the active project
- Reflection sections (build + ship) of recent shipped specs

Produce a short report:

1. Stale decisions — DEC-* to supersede. Flag, don't supersede yet.

2. Low-confidence decisions — DEC < 0.8 strengthened or weakened
   by recent work.

3. Missing constraints — patterns to formalize. Propose YAML.

4. Resolved questions — items answered. Flag formalizing DEC.

5. AGENTS.md drift — propose specific edits.

6. Template improvements — specific: "add X to spec.md because Y."

7. Stage health — progressing? stalled? rescope?

8. Cycle health — skipped cycles? mailed-in reflections?

9. Project health — scoped well? creep? time to declare done?

10. Session hygiene — any signs build and verify happen in same
    session? (Build reflections saying "nothing was unclear" is
    the telltale.) Flag and recommend strict discipline.

11. Value linkage — report on `value_link` population rate across
    active specs. Specs without `value_link` aren't a problem per
    se, but a trend toward never populating them means the thesis
    isn't driving spec selection.

12. Cost review — aggregate `cost.totals.estimated_usd` across
    specs shipped this week. Outliers? Patterns (design-heavy vs
    build-heavy)? Specs missing cost data entirely? The last one
    is an agent-discipline signal worth flagging.

Tight report. Actionable in 10 min.
```

---

## Quick reference

| You just... | Use this prompt |
|---|---|
| New project idea | 1a (Project Frame) |
| Frame approved | 1b (Project Brief) |
| Ready to frame a stage | 1c (Stage Frame) |
| Brief + first stage approved | 2a (Repo/Project Design) |
| Ready to write a spec | 2b (Spec Design) |
| Spec ready to build | 3 (Build) — NEW SESSION |
| Build complete, PR open | 4 (Verify) — NEW SESSION |
| Approved | 5 (Ship) |
| All stage specs shipped | 1d (Stage Ship) |
| All project stages shipped | 1e (Project Ship) |
| Friday | 6 (Weekly Review) |
