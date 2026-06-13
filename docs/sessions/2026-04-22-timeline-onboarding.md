# Template v5.3 — Instruction Timeline Convention

You are being asked to add one convention to a spec-driven development template: an instruction timeline file per spec, plus prompts-to-files so the architect's instructions are versioned artifacts instead of ephemeral chat output.

Read this doc fully before acting. Ask the user the questions in Section 8 before making changes.

## 1. The idea, in one paragraph

Today, when the architect (Claude) produces a prompt for the next cycle (Build, Verify, Ship), the prompt lives in the architect's chat window. The user copy-pastes it into whichever agent executes that cycle. This doesn't scale and throws away valuable artifacts.

The convention is simple: **the architect writes the prompt to a file, and appends a line to a timeline file that tracks cycles in order.** The executor (builder, verifier, shipper) reads the prompt file and executes. Status markers show what's done, what's in progress, what's pending, what's blocked.

Everything is markdown. No daemons, no MCP servers, no dispatch mechanisms. The filesystem is the queue; the human is the orchestrator.

## 2. What gets added

### The timeline file

Located at: `projects/PROJ-NNN-<slug>/specs/SPEC-NNN-<slug>-timeline.md`

Status markers (four states):

- `[ ]` **not started** — pending; no one has picked this up yet
- `[~]` **in progress** — an executor is currently running this
- `[x]` **complete** — cycle finished; see the prompt file for what was run
- `[?]` **blocked** — needs human decision or external unblock before proceeding

Shape:

```markdown
# SPEC-NNN timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: [ ] not started · [~] in progress · [x] complete · [?] blocked.

## Instructions

- [x] **design** — prompt: `prompts/SPEC-NNN-design.md`
       agent: claude-code (desktop), completed 2026-04-23

- [~] **build** — prompt: `prompts/SPEC-NNN-build.md`
       agent: kilo (CLI), in progress

- [ ] **verify** — prompt: pending (waiting on build)

- [ ] **ship** — prompt: pending (waiting on verify)
```

A blocked example:

```markdown
- [?] **build** — prompt: `prompts/SPEC-NNN-build.md`
       agent: kilo (CLI), blocked: API contract unclear — needs architect input
```

### The prompts directory

Located at: `projects/PROJ-NNN-<slug>/specs/prompts/`

One markdown file per dispatched cycle, named `SPEC-NNN-<cycle>.md`. The file contains the actual prompt the executor reads — everything needed to run that cycle, with IDs substituted in, context referenced, acceptance criteria clear.

When the architect finishes a cycle (Design, Verify), part of its output is writing the next cycle's prompt to this directory.

### No new commands

The timeline is just a file. You read it to see what's next. You open the prompt file it references. You hand that prompt to whatever agent runs that cycle.

If a small convenience is useful, add one shell helper like `just next SPEC-NNN` that prints the first non-`[x]` timeline line. But even this is optional — `cat` works.

## 3. What changes in the template

### Architect prompts in FIRST_SESSION_PROMPTS.md

For both variants (`claude-only` and `claude-plus-agents`), update these prompts to include timeline/prompts output as part of their completion:

- **Prompt 2b (Spec Design)** — at end, after spec is written, instruct architect to write `prompts/SPEC-NNN-build.md` and append the build line to the timeline
- **Prompt 4 (Verify)** — at end, if verify approves, instruct architect to write `prompts/SPEC-NNN-ship.md` and append the ship line; mark verify as `[x]`
- **Prompt 5 (Ship)** — at end, mark ship as `[x]` in the timeline before archiving

The Design prompt additionally scaffolds the initial timeline file when the spec is first written — with `[x] design` (since design is completing) and `[ ] build` populated, and `[ ]` placeholders for verify and ship.

Prompts should also instruct executors:
- Mark the cycle `[~]` when starting work
- Mark `[?]` with a one-line reason if they hit something requiring architect judgment or external unblock
- Mark `[x]` with completion details (PR number, cost, date) when done

### The `new-spec` script

When `just new-spec` scaffolds a new spec, it should also scaffold an empty timeline file alongside it. Contents:

```markdown
# SPEC-NNN timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: [ ] not started · [~] in progress · [x] complete · [?] blocked.

## Instructions

_(Timeline will be populated as the architect writes each cycle's prompt.)_
```

The prompts directory can be created lazily — when the first prompt file is written, the directory appears. Or scaffolded empty with a `.gitkeep`. Either works.

### AGENTS.md — both variants

Add a new section, "Instruction Timeline," between the existing Cycle Model and Cross-Reference Rules sections (probably around section 9). Brief — roughly this content:

```markdown
## N. Instruction Timeline

Every spec has a timeline file at
`projects/*/specs/SPEC-NNN-<slug>-timeline.md` that lists cycle
instructions in order with status markers.

Status markers:
- `[ ]` not started — no one has picked this up yet
- `[~]` in progress — an executor is currently running this
- `[x]` complete — cycle finished; see the prompt file for what was run
- `[?]` blocked — needs a human decision or external unblock before
  proceeding (include a one-line reason after the marker)

Cycle prompts live at `projects/*/specs/prompts/SPEC-NNN-<cycle>.md`.
The architect writes them; executors read and run them.

**Discipline for executors:**
- When you start a cycle, mark it `[~]`
- When you finish, mark it `[x]` with a one-line result (PR number,
  cost, completion date)
- If you hit a real blocker — constraint is ambiguous, dependency
  isn't ready, verify surfaced something that needs architect
  judgment — mark `[?]` with a one-line reason. Do NOT use `[?]` as
  a "I don't know what to do" dumping ground. Blocked means the
  next move requires someone else; everything else is in-progress
  or a question to resolve in the current session.

This is a convention, not a mechanism. No tooling enforces it; the
discipline is in the prompt set. If you skip it, nothing breaks, but
you lose the history artifact and the next executor has to hunt for
the right prompt.
```

### Small addition to spec front-matter (optional, not required)

The spec's front-matter could reference its timeline for completeness:

```yaml
artifacts:
  timeline: SPEC-NNN-<slug>-timeline.md
  prompts_dir: prompts/
```

This is a convenience — tools that read the spec know where to find its timeline. Skip it if it feels like ceremony. The convention is the directory structure, not a front-matter field.

### README / GETTING_STARTED updates

One paragraph in GETTING_STARTED.md (both variants) explaining the timeline convention and showing an example timeline so users understand the workflow.

README can get a one-sentence mention in the "What this template is" section.

## 4. Test coverage

Add to `scripts/test.sh`:

- `new-spec` scaffolds an empty timeline file at the expected path
- Timeline file has the expected header and the four-state status-marker legend
- `prompts/` directory exists after a spec scaffolded
- AGENTS.md in both variants has the Instruction Timeline section
- AGENTS.md documents all four status markers

Roughly 4-6 new assertions. Keep the harness simple.

## 5. Example, for the example project

Update `projects/PROJ-001-example-mvp/specs/SPEC-001-example-project-logger.md`
and scaffold an example timeline file + example `prompts/SPEC-001-design.md`
showing what a completed design cycle's prompt looks like. This makes the
convention concrete for anyone who clones the template fresh.

The example timeline should show `[x] design` completed with a real-looking
prompt file, and `[ ]` placeholders for build/verify/ship. Don't fake completion
of later cycles — the example project ships with only the design cycle done.

## 6. What's deliberately out of scope

Don't build:

- A `just dispatch` command that runs the agent automatically
- Any MCP server or daemon or file-watching mechanism
- A prompt-templating system (substitutes variables automatically)
- A status-change validation script
- Cross-spec timeline aggregation (single-spec for now)
- "Send from Claude desktop to terminal" mechanisms

Any of these might be worth building later. None of them belong in this session.

The timeline is a convention for humans to read and agents to follow. If it
starts breaking down at scale, that's the signal for v6 to add orchestration —
but the convention needs to prove useful first.

## 7. Implementation order

1. Read SESSION_REPORT files in `docs/sessions/` to confirm `just test` passes at 57
2. Read relevant template files: spec.md template, new-spec.sh, AGENTS.md, FIRST_SESSION_PROMPTS.md
3. Scaffold timeline file in `new-spec.sh` (one commit)
4. Update architect prompts in FIRST_SESSION_PROMPTS.md for both variants (one commit per variant or combined)
5. Add AGENTS.md section in both variants (one commit)
6. Update GETTING_STARTED.md in both variants (one commit)
7. Scaffold example timeline and example prompt for SPEC-001 (one commit)
8. Extend test harness (one commit)
9. CHANGELOG entry (one commit)

Roughly 6-8 commits. Session length: 90 minutes to 2 hours.

## 8. Questions for the user before starting

1. Does `just test` currently pass with 57 checks? If not, stop and investigate.
2. Should timeline files get committed to the repo, or be gitignored?
   Recommendation: committed. They're part of the work record.
3. Should the timeline be one-per-spec (default), or also have a stage-level
   or project-level timeline? Recommendation: start with spec-only. Add more
   granularity only if usage shows it's needed.
4. Should the prompts directory be per-spec (recommended), or per-project
   with all prompts mixed together? Recommendation: per-spec. Co-location
   with the spec makes navigation obvious.

Answer 1 first. Items 2-4 can be confirmed as you go.

## 9. Success criteria

At end of session:

- Both variants have the timeline convention documented and scaffolded
- `just new-spec` creates both a spec file and a timeline file
- All four status markers (`[ ]`, `[~]`, `[x]`, `[?]`) are documented in AGENTS.md
- Architect prompts include timeline/prompt-file output instructions
- Executor prompts include guidance on marking `[~]`, `[x]`, and `[?]`
- Example project shows a working timeline + at least one prompt file
- `just test` passes at 61-63 checks (57 + 4-6 new)
- CHANGELOG has a v5.3 entry
- Commit history tells a coherent "added convention X" story

If something in this doc contradicts the actual repo, or if you encounter a
decision point not covered here, stop and ask. Don't guess on conventions.
