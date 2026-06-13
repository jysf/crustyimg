# Template Repo Hardening — Claude Code Onboarding

You are being asked to harden a spec-driven project template repo. This document brings you up to speed on what the repo is, how it was built, the known bugs, and how to approach fixing them. Read this fully before taking any action.

---

## 1. What this repo is

This is a **GitHub template repo** for running spec-driven development on software projects. The idea: someone clicks "Use this template," runs `just init`, picks a variant, and ends up with a scaffolded repo ready to run a disciplined workflow where Claude (and optionally a separate implementer agent) work through specs in a structured cycle.

### The conceptual hierarchy

```
Repo (the app — persists)
 └─ Project (a wave of work: "MVP", "v2 improvements")
     └─ Stage (a coherent chunk within a project)
         └─ Spec (an individual task)
              └─ Cycle (Frame → Design → Build → Verify → Ship)
```

### The two variants

- **`claude-only/`** — Claude plays every role (architect, implementer, reviewer) across separate sessions. No handoff documents; implementation context is folded into each spec.
- **`claude-plus-agents/`** — Claude architects and reviews; a separate agent (Kilo, Factory, AdaL, etc.) implements. Adds an explicit `/handoffs/` folder to carry context across agent boundaries.

Both variants live under `variants/` in the template repo. `just init` copies one of them up to the repo root and deletes the other.

### ContextCore alignment

The template is philosophically aligned with [ContextCore](https://github.com/neil-the-nowledgeable/contextcore) — same vocabulary (`task.*`, `insight.*`, `guidance.*`, `handoff.*`, `project.*`, `repo.*`), same artifact model, same forward-compatibility with OTel-based observability. But it requires **zero** ContextCore infrastructure. Everything is markdown files until someone decides to graduate. See `variants/<variant>/docs/CONTEXTCORE_ALIGNMENT.md` in the repo for details.

---

## 2. Read these files first, in this order

Before you touch anything, read these. They'll give you the full picture.

### At the root
1. `README.md` — top-level template README
2. `justfile` — commands users will run
3. `scripts/_lib.sh` — shared bash helpers (colors, next-id, slugify, frontmatter updater)
4. `scripts/status.sh`, `new-spec.sh`, `new-stage.sh`, `advance-cycle.sh`, `archive-spec.sh`, `weekly-review.sh`, `info.sh` — one at a time; they're small

### Inside one variant (pick `claude-only/` first)
5. `variants/claude-only/README.md`
6. `variants/claude-only/AGENTS.md` — the big conventions document
7. `variants/claude-only/GETTING_STARTED.md`
8. `variants/claude-only/FIRST_SESSION_PROMPTS.md` — the prompts users paste into Claude at each cycle
9. `variants/claude-only/.repo-context.yaml`
10. `variants/claude-only/docs/CONTEXTCORE_ALIGNMENT.md`

### Templates and examples
11. `variants/claude-only/projects/_templates/project-brief.md`
12. `variants/claude-only/projects/_templates/stage.md`
13. `variants/claude-only/projects/_templates/spec.md`
14. `variants/claude-only/decisions/_template.md`
15. `variants/claude-only/guidance/constraints.yaml`
16. The example `PROJ-001-example-mvp/` directory: `brief.md`, `stages/STAGE-001-*.md`, `specs/SPEC-001-*.md`

### Then the other variant
17. Compare `variants/claude-plus-agents/` against `variants/claude-only/` — most files are identical. The real differences are:
    - `projects/_templates/handoff.md` exists only in plus-agents
    - `projects/PROJ-001-example-mvp/handoffs/HANDOFF-001-*.md` exists only in plus-agents
    - `projects/_templates/spec.md` is shorter in plus-agents (no folded Implementation Context or Build Completion sections)
    - `AGENTS.md` differs on handoff-specific sections
    - `FIRST_SESSION_PROMPTS.md` differs on prompts 2b, 3, 4

---

## 3. Known bugs and weak spots

These are things the user reported finding while exercising the scripts, plus self-identified gaps I know about. Treat this as your starting punch list — don't assume it's complete. You'll likely find more.

### Known/reported

The user mentioned "Claude found some bugs running the scripts." I don't have their exact list. Your first task is to **ask the user for the specific bugs they hit**, or examine the repo state (git log, uncommitted changes, scratch notes) for any indication of what broke.

### Self-identified from my own testing

During the initial build I tested the scripts with `just` actually installed and caught two bugs, which I fixed. But the fixes were narrow and more issues likely exist:

1. **YAML parsing in `status.sh` was fragile.** I use awk to walk front-matter blocks and extract scalar values. It works for the example files but will almost certainly break on edge cases — nested keys at different indentation, multi-line values, quoted strings with colons, missing keys, etc. The current logic:
   ```bash
   awk '/^---$/{f=!f; next} f && /^[[:space:]]+cycle:/{print $2; exit}'
   ```
   Any YAML that doesn't match this exact shape will return blank or wrong values.

2. **`update_frontmatter_scalar` in `_lib.sh` is a hand-rolled awk state machine.** It handles the specific case of updating `task.cycle` and `handoff.status`. It will strip inline comments (e.g. `cycle: design    # comment` becomes `cycle: verify` with no comment). It will also probably break on deeply nested YAML or lists.

3. **`just` interpolation was unquoted** in the justfile, which broke multi-word titles. I fixed it (wrapped `{{TITLE}}` in double quotes) but there may be other places this pattern exists.

4. **`new-spec.sh` does NOT update the parent stage's backlog** when a spec is added. It just prints a reminder. Same for `archive-spec.sh` — it reminds the user to update the stage backlog manually. This is an "intentional" gap (markdown list formatting is judgment-laden) but it creates a drift risk where the stage's backlog count diverges from reality.

5. **There is no `new-project` command.** Creating PROJ-002 requires a user to manually create the directory structure or ask Claude to produce it. This is a real usability gap.

6. **`get_active_project` uses a naive heuristic:** it picks the lexically first non-example PROJ-* directory. If a user has multiple active projects, this will silently pick wrong. There's an `ACTIVE_PROJECT` env var override but it's undocumented outside `scripts/info.sh`.

7. **`next_id` uses sort -n on the numeric portion, which works, but the padding logic assumes exactly 3 digits.** If a user somehow creates SPEC-0001 or SPEC-10000 the behavior is undefined.

8. **No cross-platform testing.** I only tested on Ubuntu. The scripts have macOS branches for `stat` and `sed -i` but weren't actually run on macOS. Users on macOS may hit unexpected issues.

9. **The `just init` target prompts interactively** using `read`. This works in a normal shell but will break in any non-interactive context (CI, piping, etc.). Fine for the intended use case but worth knowing.

10. **`weekly-review.sh` outputs paths with `/tmp/template-test/...` prefixes** from where I tested it — actually, looking again, it uses `$REPO_ROOT` which is current directory, so this should be correct. Verify by running it and confirming output.

11. **No tests.** There is no test harness for the scripts themselves. Regressions will be found by users, not by CI.

12. **Two copies of templates/examples.** Since `claude-only` and `claude-plus-agents` share ~90% of their content, any change to shared files has to happen twice. There's no symlink or generation system. This is a maintainability hazard that will cause drift between variants over time.

### Suspected but unconfirmed

- `archive-spec.sh` has a block that tries to count remaining specs in a stage using `xargs -I{} awk ...`. The logic is complex and may have bugs around empty directories, specs without a `stage:` field, or edge cases in the awk pattern.
- The `required_initialized` check looks for `AGENTS.md` at root. If a user manually creates an `AGENTS.md` before running `just init`, the init will refuse to run. This might be correct behavior (don't clobber existing work) but could also be frustrating.
- The `.variant` file at root is the source of truth for `get_variant`, but if someone deletes or edits it, things may break without clear errors.

---

## 4. Your mission

Bring this template repo to a state where it can reliably ship to other humans. The bar is:

1. **All documented `just` commands work as described**, on both macOS and Linux, for both variants.
2. **Known bugs are fixed** and the fixes have tests or at least reproducible verification steps.
3. **Gaps that hurt first-use experience** (like no `new-project` command) are either closed or documented as known limitations.
4. **The template can be cloned, `just init`-ed, and taken through a full cycle** (Frame → Design → Build → Verify → Ship) by a new user without hitting a scripting wall.

You do NOT need to:
- Rewrite everything
- Add a test framework (though bats would be nice if you have time)
- Port to a different shell or language
- Add features beyond what's documented

Focus on making what exists work.

---

## 5. Recommended approach

**Phase 1 — Understand the current state (30-60 min):**
1. Read the files listed in Section 2.
2. Ask the user what specific bugs they hit.
3. Run `just --list` in a fresh checkout to see the full command surface.
4. Run `just init` with each variant choice in a `/tmp` copy of the repo.
5. For each scaffolded variant, run the full happy path: `just new-stage`, `just new-spec`, `just advance-cycle` through each cycle, `just archive-spec`, `just weekly-review`, `just status`. Capture every error.

**Phase 2 — Inventory all bugs (15-30 min):**
- Union of: Section 3 list + user-reported bugs + anything you hit in Phase 1.
- Classify each as blocking (command fails or does wrong thing) vs non-blocking (cosmetic, edge case, missing feature).
- Order blocking ones by severity.

**Phase 3 — Fix, one bug at a time (bulk of the session):**
- For each bug: reproduce it, fix it, verify the fix in the actual script via a test command, commit with a message that explains what and why.
- When changing shared logic in `_lib.sh`, test that ALL commands that use it still work.
- When changing files that exist in both variants (templates, example files, AGENTS.md sections), remember to apply the same change in both variants.

**Phase 4 — Fill genuine gaps (optional, only if you have time):**
- Add a `just new-project` command if the user wants it.
- Add a minimal bats test harness.
- Improve cross-variant consistency (consider whether a `variants/_shared/` symlink-based approach would be worth it — it might not be).

**Phase 5 — Document what you did (at the end):**
- Update `CHANGELOG.md` at repo root (create it if it doesn't exist) with every fix and feature.
- If any user-facing behavior changed, update the relevant README, GETTING_STARTED, or AGENTS.md.
- If known limitations remain, add a `KNOWN_LIMITATIONS.md` at the root listing them honestly.

---

## 6. Important discipline while working here

You are working on a template that teaches spec-driven discipline. It would be funny (and fair) if you held yourself to the same standards:

- **Commit often, with clear messages.** One bug per commit where reasonable. Messages should name what broke and how you fixed it.
- **Don't refactor for aesthetics while fixing bugs.** Make the bug go away with the minimum change. Refactor in a separate pass if needed.
- **Preserve the existing architecture.** Both variants should continue to exist. The ContextCore vocabulary should not be replaced. The Repo → Project → Stage → Spec → Cycle hierarchy should remain.
- **If you disagree with a design choice, say so in a note to the user — don't quietly change it.** The user iterated deliberately on naming, structure, and scope. Unilateral "improvements" are out of scope.
- **Ask before adding dependencies.** The current setup is bash + just. Anything more (python, node, a test framework) should be a conscious choice.
- **If you find something that looks intentional but confusing, ask.** Likely it's deliberate.

---

## 7. Questions to ask the user before starting

Copy these and ask them upfront:

1. **What specific bugs did you hit?** Can you paste the exact commands and errors?
2. **Which variant(s) were you using when you hit them — claude-only, claude-plus-agents, or both?**
3. **What platform are you on — macOS, Linux, or other?**
4. **Do you want me to add new features (like `just new-project`) or stay strictly in bug-fix mode?**
5. **Should I try to add tests, or is manual verification enough for now?**
6. **Do you want me to improve cross-variant consistency (deduplicating shared content), or leave the current "two copies" approach?**

Don't start writing code until you have answers to 1, 2, 3. Items 4-6 can be deferred until you've scoped the bug-fix work.

---

## 8. What success looks like

At the end of the session:

- A user can go from fresh clone → `just init` → full first spec cycle (Frame → Design → Build → Verify → Ship) on either variant, without hitting a scripting wall.
- Every known bug (from user report, from Section 3, from your Phase 1 testing) is either fixed or explicitly documented as a known limitation.
- The git log tells a coherent story of what was broken and what was fixed.
- If you added any new commands or behavior, the documentation (justfile comments, READMEs, AGENTS.md) reflects it.

---

## 9. Out of scope

Things NOT to do, even if tempted:

- Porting scripts to Python, Node, Go, etc.
- Adding a full testing framework with CI (unless the user explicitly asks)
- Changing the variants' philosophy or reorganizing the hierarchy
- Adding features that go beyond what's documented in READMEs and GETTING_STARTED.md (unless the user asks)
- Deleting either variant
- Renaming artifacts (SPEC, STAGE, PROJECT, DEC, HANDOFF)
- Switching away from ContextCore vocabulary
- "Improving" the prompts in FIRST_SESSION_PROMPTS.md without the user's direction

---

## 9a. Known upcoming work (don't build it, but don't block it)

After this hardening session ships clean, there will be a follow-on session that adds:

1. **Daily and weekly report commands** (`just report-daily`, `just report-weekly`) that generate markdown reports under `reports/daily/` and `reports/weekly/`, scraping file state and git log to tell the project's story over time.

2. **AI cost tracking** — a `cost:` block in spec front-matter that agents self-populate at the end of each cycle (Design, Build, Verify, Ship). Claude Code in particular will run `/cost` near the end of its work and append a session entry. Reports will aggregate cost data.

3. **Minor prompt updates** to all cycle prompts (in `FIRST_SESSION_PROMPTS.md`) so agents emit cost blocks as part of their completion discipline. Verify gets a check for cost-block presence.

**You do NOT need to build any of this.** But while fixing bugs, keep these principles in mind so the follow-on session is cheap:

- **If you're already modifying the spec template,** don't restructure the front-matter in ways that would make adding a `cost:` section awkward. Current shape (nested `task:`, `project:`, `repo:`, `agents:`, `references:` blocks) leaves a natural slot at the bottom of front-matter for a `cost:` block. Preserve that.
- **If you're modifying the status script,** keep its YAML-parsing helpers general enough to read `cost.totals.estimated_usd` without needing new helpers. The report scripts will reuse this code.
- **If you're fixing scripts that read spec files,** consider whether the fix generalizes — because report scripts will do similar reads at scale.
- **If the user asks you to add a `just` command during hardening,** check whether it's really a bug fix or a feature. Features like `report-daily` should be deferred to the follow-on session.

If you find yourself about to do something that would interact with cost tracking or reports, **stop and ask the user** whether to defer. The goal is a clean hardening session, not scope creep.

---

## 10. When you're ready

Start by asking the user the questions in Section 7. Then read through Section 2's file list. Then come back and propose a plan before you start making changes.

Good luck. You're not starting from scratch — you're polishing something that already works, mostly.
