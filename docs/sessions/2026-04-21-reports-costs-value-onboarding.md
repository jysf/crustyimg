# Template Reports + Costs + Business Value — Claude Code Onboarding

You are being asked to add three coherent features to a spec-driven development template repo:

1. **Business value structure** at the project and stage level
2. **AI cost tracking** via self-reporting cost blocks in spec files
3. **Daily and weekly reports** that read value and cost data to tell the project's story over time

These three features are bundled into one session because they're tightly coupled: reports without value structure produce thin stats; reports without cost data miss a key dimension; value and cost both live in spec/stage/project front-matter, so modifying those files once for both changes is cheaper than doing them in separate sessions.

Read this doc fully before taking any action. Then ask the user the questions in Section 14 before making changes.

---

## 1. Prerequisites and assumptions

This work assumes:

- **The hardening session has completed.** See `docs/sessions/2026-04-20-hardening-report.md` for what that session did and `CHANGELOG.md` for the condensed list. The scripts you're extending should be reliable. `just test` should print `PASS 30 checks`.
- **The user has a real project they may want to migrate to the new version.** Be careful about backwards compatibility — old specs that don't have value/cost blocks must not break reports.
- **`just` is installed and working.**
- **The user's priorities are clear:** they want the feature set described here, in this order: value first (informs reports), then costs (populate over time), then reports (aggregate both).

---

## 2. Orientation: what exists before you start

**Conventions established by the hardening session you must respect:**

- **Placeholder tokens use `__SCREAMING_SNAKE__`** — `__TODAY__` and `__REPO_ID__` are the canonical patterns. Any new placeholders you add should follow the same convention so scripts can substitute them without colliding with example text or format-documentation comments.
- **`find_spec` excludes archived specs** (`*/done/*`). Any new script that walks spec files should do the same, unless you explicitly want to include archived ones (reports sometimes do).
- **`get_repo_id` helper in `_lib.sh`** reads `metadata.repo.id` from `.repo-context.yaml` and falls back to `my-app`. Use the same pattern for any new helpers.
- **Shell scripts should handle both macOS and Linux.** Use `uname`-conditional branches for `stat` and `sed -i` as the existing scripts do.
- **30-assertion test harness in `scripts/test.sh`.** Extend it, don't replace it.
- **Templates live in two copies** (one per variant). Any change to shared structure gets applied in both. No dedup system exists; this is deliberate.

**Spec front-matter shape after hardening** (this is the shape you'll extend — do not restructure):

*claude-only variant:*
```yaml
---
task:
  id: SPEC-XXX
  type: story
  cycle: design
  blocked: false
  priority: medium
  complexity: S

project:
  id: PROJ-XXX
  stage: STAGE-XXX
repo:
  id: __REPO_ID__

agents:
  architect: claude-opus-4-7
  implementer: claude-opus-4-7
  created_at: __TODAY__

references:
  decisions: []
  constraints: []
  related_specs: []
---
```

*claude-plus-agents variant is similar but has a `handoff:` block instead of `agents:`.*

**You will add** a `cost:` block and a `value_link:` field to the bottom of the front-matter. Don't touch what's above it.

---

## 3. Read these first, in this order

1. **`docs/sessions/2026-04-20-hardening-report.md`** — understand what the hardening session did and what it left open
2. **`CHANGELOG.md`** — the condensed fix list
3. **`KNOWN_LIMITATIONS.md`** — what's explicitly unfixed and why
4. **`scripts/_lib.sh`** — shared helpers (especially `get_repo_id`, `find_spec`, `update_frontmatter_scalar`, placeholder substitution patterns)
5. **`scripts/test.sh`** — the test harness; you'll extend it
6. **`justfile`** — commands; you'll add four new ones
7. **`variants/claude-only/projects/_templates/spec.md`** — you'll extend the front-matter
8. **`variants/claude-only/projects/_templates/stage.md`** — you'll add a value-contribution block
9. **`variants/claude-only/projects/_templates/project-brief.md`** — you'll add a value thesis block
10. **`variants/claude-only/FIRST_SESSION_PROMPTS.md`** — you'll update 6 prompts
11. **`variants/claude-only/AGENTS.md`** — you'll add two new sections
12. **All of the above again for `claude-plus-agents`** — same edits, different variant directory

---

## 4. Feature 1: Business value structure

### Why this goes in first

Reports tell a project's story. Without value structure, reports can only say "4 specs shipped" — they can't say "this week's work advanced the retention thesis by delivering the onboarding flow." The storytelling needs a value spine.

### The design (stick to this — do not redesign without asking the user)

Value lives at project and stage level. Specs get only an optional lightweight reference (`value_link:`), not a structured value block of their own. This mirrors how people actually think about value: projects have theses, stages contribute toward them, individual specs are implementation mechanisms.

#### Project brief gets a `value:` block

Added after the existing `## Why Now` section or wherever feels natural in the brief structure. Place it prominently so it's the first thing you see after "What This Project Is."

```yaml
value:
  thesis: "One-sentence claim about what this project delivers for the business/user."
  beneficiaries:
    - "who benefits (users, team, business function)"
    - "..."
  success_signals:
    - "observable outcome 1"
    - "observable outcome 2"
    - "observable outcome 3"
  risks_to_thesis:
    - "what could make this thesis wrong"
    - "..."
```

This is YAML-in-markdown, placed inside a fenced code block with the label `yaml` so it renders readably. Don't use front-matter YAML for this — project briefs don't have front-matter in the current structure (they have inline YAML only for `project:`/`repo:` metadata at top).

Actually check the current `project-brief.md` template — if it has front-matter YAML, put the `value:` block there. If it has inline YAML elsewhere, match the existing pattern.

#### Stage gets a `value_contribution:` block

Similarly inline YAML, placed early in the stage file.

```yaml
value_contribution:
  advances: "What part of the parent project's thesis this stage advances."
  delivers:
    - "user-visible capability 1"
    - "user-visible capability 2"
  explicitly_does_not:
    - "what this stage is NOT trying to do"
    - "..."
```

#### Spec gets a lightweight `value_link:` field

This is the one structured addition to spec front-matter. Place it at the bottom of the existing front-matter, after `references:`, as a single string field (not a structured block):

```yaml
references:
  decisions: []
  constraints: []
  related_specs: []

value_link: "One sentence about what this spec contributes to its stage's value. May be 'infrastructure; no direct user value' for plumbing specs. Optional but encouraged."
```

`value_link` is optional. If not populated, reports should not flag it as missing — value-linking specs is aspirational, not required. Reports *should* surface the count of specs with populated `value_link` as a signal, but not as a red flag.

### Prompt updates for value

In `FIRST_SESSION_PROMPTS.md`:

- **Prompt 1b (Project Brief)** — add: "After the 'Why Now' section, add a `value:` block with thesis, beneficiaries (2-4), success_signals (3-5 observable outcomes), and risks_to_thesis (2-4 honest things that could make the thesis wrong). Be specific. 'Users will love it' is not a thesis; 'reducing month-2 churn by making activation faster' is."
- **Prompt 1c (Stage Frame)** — add: "Add a `value_contribution:` block. What part of the project's thesis does this stage advance? What user-visible capabilities does it deliver when done? What is it explicitly NOT trying to do (those are other stages' jobs)? If you can't articulate value_contribution for this stage, the stage may be infrastructure-only — that's acceptable but flag it."
- **Prompt 2b (Spec Design)** — add: "Populate `value_link:` at the bottom of front-matter. One sentence on what this spec contributes to its stage's value_contribution. If the spec is infrastructure with no direct user-visible contribution, write 'infrastructure enabling STAGE-XXX's [capability]'. Leave null only if genuinely unknown."
- **Prompt 1d (Stage Ship)** — add: "When reviewing shipped specs against the stage's `value_contribution`, flag any spec whose `value_link` didn't actually deliver what it claimed."
- **Prompt 1e (Project Ship)** — add: "Cross-check shipped stages' `value_contribution` against the project's `value.thesis`. Did the thesis hold? Would you refine it based on what shipped?"
- **Prompt 6 (Weekly Review)** — add: "Report on `value_link` population rate. Specs without `value_link` aren't a problem per se, but a trend toward never populating them means the thesis isn't driving spec selection."

### AGENTS.md updates for value

Add a new section "Business Value" between the existing hierarchy and tech-stack sections:

```markdown
## N. Business Value

Value structure exists at project and stage levels. Specs link lightly.

**Project `value:` block** states the thesis — a testable claim about
what this wave of work delivers. Beneficiaries, success signals, and
risks to the thesis make it falsifiable, not marketing copy.

**Stage `value_contribution:` block** states what this coherent chunk
of work advances, what capabilities it delivers, and what it explicitly
doesn't try to do. Helps avoid stages that seem valuable but don't
actually contribute to the project thesis.

**Spec `value_link:`** is a one-sentence reference back to the stage's
value. Infrastructure specs may have `value_link: "infrastructure
enabling X"`. This is optional but encouraged — it surfaces specs that
don't trace back to the thesis.

Reports aggregate value signals: which stages advanced the thesis,
which specs most directly delivered it, and where value traceability
broke down.
```

---

## 5. Feature 2: AI cost tracking

### The core insight (respect this, don't redesign)

Cost data lives in the spec file. Each cycle on the spec (design, build, verify, ship) is a session. Each session appends an entry to `cost.sessions[]` with its token usage. The agent running the session writes its own cost entry — self-reporting — as part of its completion discipline. At ship time, `cost.totals` are computed.

This design was settled after the user asked whether Claude Code could report its own cost back into the file (answer: yes, via `/cost`). It beats JSONL scraping because:
- Self-attribution by construction
- Works across agents (Claude Code, Claude.ai, API, third-party)
- Platform-agnostic
- No infrastructure needed
- Maps cleanly to future ContextCore spans

### The cost block design (stick to this)

Added to spec front-matter, placed at the bottom after `value_link:`:

```yaml
value_link: "..."

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
```

The `sessions` list is empty by default; agents append entries. A populated entry looks like:

```yaml
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-7
      interface: claude-ai
      tokens_input: 12000
      tokens_output: 4200
      estimated_usd: 0.12
      duration_minutes: 18
      recorded_at: __TODAY__
      notes: "one-line if unusual, else null"
    - cycle: build
      agent: claude-sonnet-4-7
      interface: claude-code
      tokens_input: 85000
      tokens_output: 22000
      estimated_usd: 0.45
      duration_minutes: 42
      recorded_at: __TODAY__
      notes: null
  totals:
    tokens_total: 123200
    estimated_usd: 0.57
    session_count: 2
```

### Design principles to preserve

- **Nullable fields are OK.** Claude.ai web sessions won't have exact tokens. Record what's known; leave the rest null. Reports handle nulls by excluding from sums, counting in session_count.
- **`interface` field matters.** Distinguishes Claude.ai web (manual estimate), Claude Code (has `/cost`), API (exact), third-party agents. Reports filter by interface to see where cost is incurred.
- **`recorded_at`** is the date the session ran, not when the block was written. Use `__TODAY__` in templates; agents substitute when they write entries.
- **No budget fields, no caps, no alerting.** Cost tracking is about visibility, not control. Keep it that way.

### Prompt updates for cost

**Prompt 2b (Spec Design)** — append after the existing "Stop and let me review" section:

> **Before you stop**, append a cost session entry to the spec's
> `cost.sessions` list:
>
> ```yaml
> - cycle: design
>   agent: <your model name, e.g. claude-opus-4-7>
>   interface: <claude-ai | claude-code | api | other>
>   tokens_input: <from your session's usage if known, else null>
>   tokens_output: <from your session's usage if known, else null>
>   estimated_usd: <your estimate, or null>
>   duration_minutes: <your best guess>
>   recorded_at: <today in YYYY-MM-DD>
>   notes: <one line if unusual, else null>
> ```
>
> If you're in Claude Code, run `/cost` first and use its numbers.

**Prompt 3 (Build)** — both variants. Append before "Open PR":

> **Before opening the PR**, append a cost session entry:
>
> - If in Claude Code: run `/cost`, use its numbers
> - If via API: use the `usage` object from the API response
> - If via Claude.ai web: best-guess the numbers
> - If your agent doesn't report cost: set numeric fields to null, add
>   `notes: "agent does not report cost"`
>
> ```yaml
> - cycle: build
>   agent: <model>
>   interface: <claude-code | api | kilo-code | factory | etc.>
>   tokens_input: <best available>
>   tokens_output: <best available>
>   estimated_usd: <best available>
>   duration_minutes: <estimate>
>   recorded_at: <today>
>   notes: <one line if rework or unusual, else null>
> ```

**Prompt 4 (Verify)** — both variants. Add to the verifier's own completion:

> Before returning your verdict, append your own cost session entry
> (same format as Design, but `cycle: verify`).

And add to the flag list:

> **Cost block check:** does the spec have `cost.sessions` entries for
> all prior cycles (design, build)? If not, flag it so missing agents
> can be prompted to fill in retroactively. Don't block the PR for this
> — just note it.

**Prompt 5 (Ship)** — both variants. Add before `archive-spec`:

> **Compute cost totals.** Sum entries in `cost.sessions[]`:
>
> - `totals.tokens_total = sum(tokens_input + tokens_output)` across
>   sessions, skipping nulls
> - `totals.estimated_usd = sum(estimated_usd)` skipping nulls
> - `totals.session_count = len(sessions)` (include sessions with null
>   numeric fields)
>
> If any session had nulls, that's fine — reports will show "partial
> cost data available" rather than missing. Append your own ship-cycle
> cost session before computing totals.

**Prompt 6 (Weekly Review)** — add a new section:

> **Cost review.** Aggregate `cost.totals.estimated_usd` across shipped
> specs this week. Outliers? Patterns (design-heavy vs build-heavy)?
> Specs missing cost data entirely? The last one is an agent-discipline
> signal worth flagging to the human.

### AGENTS.md updates for cost

Add a new section after the value section:

```markdown
## N+1. Cost Tracking Discipline

Every cycle on a spec appends a session entry to the spec's
`cost.sessions` list. Agents self-report cost data so reports can
aggregate AI spend over time.

- **Claude Code:** run `/cost` at the end of your session.
- **API calls:** use the `usage` object in the API response.
- **Claude.ai web:** estimate based on session length. Note
  `interface: claude-ai` so reports can distinguish estimates.
- **Third-party agents:** use whatever cost mechanism the agent
  provides. If none, enter null values with a note.

Verify cycle flags specs missing cost data (doesn't block the PR).
Ship cycle computes `cost.totals` from the session entries.

Reports aggregate cost by cycle, by interface, by spec, and by stage.
```

---

## 6. Feature 3: Daily and weekly reports

### The commands

Two new commands:

```bash
just report-daily     # generates reports/daily/YYYY-MM-DD.md
just report-weekly    # generates reports/weekly/YYYY-WNN.md
```

Reports overwrite on re-run — they're a snapshot, not an append-only log.

### The directory structure

Create at repo root:

```
reports/
  daily/
    .gitkeep
  weekly/
    .gitkeep
```

### Design principles

- **Quantitative only, no narrative.** The user explicitly deferred narrative generation. Claude can narrate on-demand from reports later. Keep reports deterministic and cheap.
- **File-scrape + git log.** No daemons, no external data sources. Read files, grep git log for SPEC/STAGE/PROJ IDs.
- **Graceful degradation.** Old specs without cost blocks or value_link should not crash reports. They appear in "missing data" flags.
- **Stand-alone artifacts.** Reports don't feed into the weekly review prompt (that was explicit user choice). They're readable on their own.

### Daily report structure

Output file: `reports/daily/YYYY-MM-DD.md`. Target length: under 200 lines.

**Sections in order:**

1. **Header** — date, active project name, active stage name, time of generation

2. **Snapshot**
   - Specs by cycle with IDs:
     ```
     - frame:  0
     - design: 1  (SPEC-014)
     - build:  2  (SPEC-012, SPEC-013)
     - verify: 1  (SPEC-011)
     - ship:   0
     ```
   - Project progress: shipped / planned ratio, percentage. "Planned" = shipped + active + specs referenced in stage backlogs that haven't been scaffolded. Use "unknown" if stage backlogs don't have clear counts.
   - Stage progress: shipped / active / remaining, percentage, for the active stage

3. **Value section** (new in this session)
   - Active project's `value.thesis` quoted
   - Active stage's `value_contribution.advances` quoted
   - Count of specs with populated `value_link:`, count without. Not a problem, just a visibility signal.

4. **Changes since yesterday**
   - Diff against the most recent prior daily report (if any)
   - Cycle advances, ships, new decisions, new ideas, new feedback captures

5. **Cost activity today**
   - Sessions recorded today (from `cost.sessions[].recorded_at == today`)
   - Today's total cost
   - Current WIP accumulated cost (sum of cost.totals for all non-shipped specs, or sum of sessions if totals aren't computed yet)
   - Specs missing cost data (flagged)

6. **Flags**
   - Stalled specs (in build/verify >7 days by file mtime)
   - Low-confidence decisions (<0.7) unchanged for >14 days
   - Open questions >14 days
   - Specs missing cost data for current or past cycles

7. **Git activity** — commit count, PR merges, reverts in last 24h

### Weekly report structure

Output file: `reports/weekly/YYYY-WNN.md` where WNN is ISO week number. Target: 300-500 lines.

**Sections in order:**

1. **Header** — week range, active project, generation time

2. **Summary table**
   ```
   - Specs shipped:                4
   - Specs advanced:               6 cycle changes
   - Decisions emitted:            3
   - Decisions superseded:         1
   - Ideas captured:               2
   - Questions raised / resolved:  1 / 1
   - Total AI cost:                $14.20
   - Avg cost per shipped spec:    $3.55
   ```

3. **Value advancement**
   - Project `value.thesis` quoted at top
   - Shipped stages this week and their `value_contribution.advances`
   - Shipped specs this week, each with `value_link:` if populated
   - Narrative-free summary: "STAGE-002 completed, advancing the onboarding thesis. 4 specs shipped, 3 directly linked to value, 1 infrastructure."
   - Flag if any shipped spec's `value_link` was null

4. **Shipped this week table**
   ```
   | Spec ID | Title | Cycle time | Stage | Day | Cost | Value link |
   ```
   Cycle time = date shipped minus date of first cycle advance (from git log or mtime)

5. **Cycle time trends**
   - This week avg, last week avg (if prior report exists), trend arrow
   - By stage breakdown

6. **Cost breakdown**
   - Total this week
   - By cycle (which phases are expensive?)
   - By interface (where is cost incurred?)
   - Top 3 most expensive specs, with reasons if identifiable from cost.sessions.notes
   - Cost-per-spec trend vs. last week

7. **Decision activity**
   - DECs emitted (with confidence)
   - DECs superseded (with reason if captured)
   - Long-lived low-confidence DECs that moved this week

8. **Reflection themes**
   - Scan shipped specs' Reflection sections (both build and ship)
   - List common friction points, template-improvement suggestions
   - Heuristic only: group by keyword overlap or just list all reflections

9. **Flags** (same categories as daily, aggregated)

10. **Comparison to last week** (if prior weekly report exists)

### Implementation structure

**New scripts:**
- `scripts/report_daily.sh` (~200-300 lines)
- `scripts/report_weekly.sh` (~300-400 lines)

Both scripts:
- Source `scripts/_lib.sh` for shared helpers
- Use `get_repo_id`, `get_active_project` as needed
- Parse spec YAML front-matter (see existing patterns in `status.sh`)
- Parse cost blocks: extract `cost.sessions[].recorded_at`, `.estimated_usd`, etc.
- Format markdown deterministically
- Exit 0 on success, nonzero on hard errors (missing active project, etc.)
- Idempotent: running twice on the same day produces the same output

**New `_lib.sh` helpers needed:**
- `sum_cost_tokens_for_spec <spec-file>` — sums tokens across sessions
- `sum_cost_usd_for_spec <spec-file>` — sums estimated_usd, skipping nulls
- `sessions_recorded_on <spec-file> <date>` — extracts sessions matching date
- `days_ago <N>` — portable date math (macOS vs Linux `date -d` vs `date -v`)
- `iso_week_number <date>` — ISO week calculation, portable
- `extract_value_link <spec-file>` — returns value_link string or empty
- `get_project_thesis <project-dir>` — extracts value.thesis from brief
- `get_stage_value_contribution <stage-file>` — extracts value_contribution.advances

**New justfile commands:**

```just
# Generate today's daily report
report-daily:
    @./scripts/report_daily.sh

# Generate this week's weekly report
report-weekly:
    @./scripts/report_weekly.sh
```

**New section in both AGENTS.md files** explaining the reports commands, where output lives, and the weekly cadence.

**Update both variant README.md files** with one-paragraph mention of the reports commands.

---

## 7. Testing — extend `scripts/test.sh`

Add to the existing 30-assertion test harness. Do NOT replace or refactor what's there.

New test cases to add (aim for 15-20 new assertions):

**Value tests:**
- Project brief template has a `value:` block with expected fields
- Stage template has a `value_contribution:` block
- Spec template has a `value_link:` field
- `new-spec` scaffolds a spec with `value_link: null` populated correctly
- AGENTS.md in both variants has the Business Value section

**Cost tests:**
- Spec template has a `cost:` block with empty sessions and zeroed totals
- `new-spec` scaffolds a spec whose cost block is valid YAML
- Adding a session entry by hand (via `echo` in test) and re-reading the file produces correct totals if computed

**Report tests:**
- `just report-daily` generates a file at `reports/daily/YYYY-MM-DD.md`
- Output file is valid markdown with expected sections
- Re-running on same day overwrites, not appends
- `just report-weekly` generates a file at `reports/weekly/YYYY-WNN.md`
- Reports don't crash on specs without cost blocks (backwards compat)
- Reports don't crash on specs without `value_link` (optional field)
- Reports include value thesis from project brief
- Reports include cost totals when populated
- Reports gracefully show "missing cost data" when specs lack cost blocks

**Edge cases:**
- Spec with `cost.sessions: []` (empty) — report shows "no cost data" not "$0"
- Spec with a session where `estimated_usd: null` — skipped in sums, counted in session_count
- Project with no specs yet — reports work
- Week with no prior weekly report — comparison section skipped, not errored

---

## 8. Feedback directory restructure (also in this session)

The user wants `NOTES_FROM_BRAGFILE_EXPERIMENT_000.md` moved into a `feedback/` directory as part of this session. Structure:

```
feedback/
  _template.md
  2026-04-20-bragfile-project.md
  archive/
    .gitkeep
```

- Rename `NOTES_FROM_BRAGFILE_EXPERIMENT_000.md` → `feedback/2026-04-20-bragfile-project.md`
- Create `feedback/_template.md` explaining how to capture feedback:
  ```markdown
  # Feedback entry template

  Capture downstream user feedback with structured front-matter so
  it's searchable and status can be tracked over time.

  ---
  source: "name of downstream project / user"
  captured_at: YYYY-MM-DD
  captured_by: human | claude | <agent>
  status: open          # open | addressed | deferred
  ---

  # One-line title

  ## The issue

  What the user reported, in their words where possible.

  ## Context

  Where they were in their workflow when this came up.

  ## Priority (their assessment)

  <priority and reasoning>

  ## Resolution

  Filled in when status changes from open.
  ```
- Create `feedback/archive/.gitkeep`

No need to add feedback-related commands or automation. The directory is just a known home for this kind of artifact.

---

## 9. Backwards compatibility requirements

**This is critical.** The user has a real project on the v5.1 template and may want to migrate to v5.2 (what this session produces).

Design rules:

1. **Reports must not crash on specs without cost blocks, without value_link, or without value structure.** Show as "missing data" flags where appropriate; don't hard-error.

2. **Don't auto-modify existing specs.** Adding empty cost blocks or empty value_link to specs that don't have them pollutes git history. New specs get them; old specs are left alone.

3. **Old project briefs without `value:` block are fine.** Reports simply say "project has no value thesis defined" rather than erroring. Same for stages without `value_contribution`.

4. **Ship the `MIGRATION_TO_REPORTS_AND_COSTS.md` doc** (create it) that tells existing-project users:
   - Your existing specs won't have cost blocks or value_link. That's fine.
   - Your existing project brief and stages won't have value structure. That's fine.
   - New specs/stages/briefs will have the new structure.
   - Reports will show "missing" flags for pre-migration items. You can ignore them.
   - If you want to add value structure to an existing project brief, here's the block to copy in. Same for stages.

### Compatibility test cases

Add to `scripts/test.sh`:

- Simulate a pre-migration spec file (no cost block, no value_link). Ensure `just report-daily` and `just report-weekly` handle it gracefully.
- Simulate a pre-migration project brief (no value block). Reports should acknowledge its absence without crashing.

---

## 10. Implementation order (recommended)

1. **Read everything in Section 3** — understand the shape you're extending
2. **Run `just test`** — confirm the hardened baseline is intact (PASS 30 checks)
3. **Feedback restructure first** (cheap, unblocks nothing) — move NOTES file, create _template, commit
4. **Value structure in templates** — update project-brief, stage, spec templates with new fields in both variants. Commit.
5. **Cost block in spec templates** — add the `cost:` block to both variant spec templates. Commit.
6. **Prompt updates** — update Design, Build, Verify, Ship, Stage Ship, Project Ship, and Weekly Review prompts in both variants. One commit per variant (or one commit each for value, then cost additions).
7. **AGENTS.md updates** — new Business Value and Cost Tracking sections in both variants. One commit.
8. **README updates** — one-paragraph mention of reports in both variants. Small commit.
9. **`_lib.sh` helpers** — add new helpers for value/cost extraction. Commit.
10. **`report_daily.sh`** — build, test manually, commit.
11. **`report_weekly.sh`** — build, test manually, commit.
12. **Justfile additions** — add `report-daily` and `report-weekly` commands. Commit.
13. **Test harness extensions** — add the new assertions to `scripts/test.sh`. Commit.
14. **Migration doc** — create `MIGRATION_TO_REPORTS_AND_COSTS.md`. Commit.
15. **CHANGELOG update** — add v5.2 entries to the existing CHANGELOG.md. Commit.
16. **Run full `just test`** — confirm all tests pass (original 30 + your new ones).

Commit count: likely 14-18 commits, one per meaningful unit of work.

---

## 11. What success looks like

At the end of the session:

- Both variants have value structure in project/stage/spec templates
- Both variants have cost blocks in spec templates
- Both variants have updated prompts that instruct agents to emit value and cost blocks
- Both variants have updated AGENTS.md with value and cost sections
- `just report-daily` and `just report-weekly` work on both variants and degrade gracefully on pre-migration specs
- `reports/daily/` and `reports/weekly/` directories exist with sample outputs committed (run the commands once on the example project)
- `scripts/test.sh` has 45-50 total assertions, all passing
- `MIGRATION_TO_REPORTS_AND_COSTS.md` exists
- `feedback/` directory exists with the moved note and a template
- `CHANGELOG.md` has a v5.2 section
- Git history tells a coherent story of what was added, one feature at a time
- All existing functionality still works

---

## 12. Discipline while working

Same principles as the hardening session:

- **Commit per meaningful unit.** One bug fix or one feature addition per commit.
- **Keep both variants in lockstep.** Use a checklist. When you add a field to claude-only's spec template, add it to claude-plus-agents' spec template in the same commit or the immediate next one. Don't let them drift.
- **Don't refactor while adding features.** If you see shared patterns that want deduplication, note them in a scratch file and surface as a deferrable item. Dedup is out of scope.
- **Ask before adding dependencies.** Bash + just + coreutils. If you need something more, check first.
- **Stay in scope.** Monthly reports are deferred. Narrative generation is deferred. Budget tracking is deferred. Admin API integration is deferred. Don't build them. If you find yourself wanting to, note it for a future session.
- **Apply fixes consistently across variants.** Any change to a shared concept (value block structure, cost block shape) applies in both variants simultaneously.

---

## 13. Out of scope

- **Monthly reports** — deferred; daily+weekly first
- **Narrative generation in reports** — explicit user decision to defer
- **Budget tracking, cost alerts, spending caps**
- **Visualization** (charts, graphs)
- **Real-time dashboards**
- **Admin API integration** for authoritative cost data
- **JSONL parsing of `~/.claude/projects/`** (cost block approach replaces this)
- **Cost pricing tables** (agents self-report estimates)
- **Per-user cost breakdown** (no user concept in template)
- **Variant dedup / shared template system** (ongoing known limitation)
- **Addressing the "fresh session is weaker than it looks" feedback** — this is a methodology question deferred to a later conversation
- **Adding a third variant (e.g., `claude-multi-model/`)** — deferred
- **Renaming cycles or other structural concepts**

---

## 14. Questions to ask the user before starting

Before writing code:

1. **Is the hardening session complete and `just test` passing?** If no, stop and investigate.
2. **How many specs are in the real project the user may migrate?** This affects migration doc detail.
3. **Do they want sample reports generated from the example project committed to the repo?** (My recommendation: yes — lets users see what output looks like before running it themselves.)
4. **Should `just report-daily` also print to stdout, or just write the file?** (My recommendation: write file, then `cat` the result for immediate visibility.)
5. **Any agents in active use beyond Claude?** Affects the `interface` enum values.

Don't start writing code until you have answers to 1 and 2. Items 3-5 can be answered as you go.

---

## 15. When you're ready

Ask Section 14 questions 1 and 2. Then read the files listed in Section 3. Then propose a plan covering:

1. Order of implementation (my suggested order is in Section 10)
2. Estimated session length (3-6 hours reasonable)
3. Questions or design concerns that aren't covered here

Then proceed, committing after each meaningful unit.

If anything in this doc feels wrong, contradicts what's actually in the hardened repo, or lacks detail you need — stop and ask the user. Don't guess on methodology or structure choices; those were all deliberate decisions.

Good luck. This is ambitious work but the foundation is strong. Your job is to extend carefully, not invent.
