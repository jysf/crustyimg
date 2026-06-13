# Changelog

All notable changes to this template. One entry per fix; newest at top.

## 2026-06-03 — More blog drafts + SECURITY.md ships downstream (v5.10)

Documentation only — no script or behavior changes.

### Added

- **Two more blog drafts** (`docs/blog/`): "One agent or two" (the
  `claude-only` / `claude-plus-agents` split and session hygiene vs.
  handoffs) and "Two numbers traditional dev hides" (the v5.2 value
  thesis + per-spec AI cost tracking). The blog ideas list is now
  cleared — five drafts total.
- **`SECURITY.md` in both variants**, so a repo created with `just init`
  carries the trust model (local tooling + agents read files and run
  commands; treat externally-sourced content as untrusted; secret
  hygiene). User-facing, with a reporting section to adapt. The template
  root keeps its own maintainer-facing `SECURITY.md`. README's
  post-init file list now mentions it.

## 2026-06-02 — Project docs: usage, security, contributing, blog (v5.9)

Documentation only — no script or behavior changes.

### Added

- **`SECURITY.md`** — threat model (local tooling + the agentic trust
  model where agents read repo files and run `just`), what's been
  hardened, accepted low-severity items, and how to report a vuln.
- **`CONTRIBUTING.md`** — the non-negotiable design principles
  (zero-dependency, bash 3.2, portable shell, escape user input, variant
  parity) and the `just test` dev loop.
- **`PROJECTS.md`** — real projects built with the template (bragfile,
  rspeed) and a note that the template's own history lives in the
  CHANGELOG/git, not in self-specs.
- **`docs/USAGE.md`** — a deeper end-to-end walkthrough than the README:
  the full project → stage → spec → cycle loop with exact commands, the
  four read-only views, and decisions/guardrails.
- **`docs/blog/`** — an index plus three drafted posts (the repo-is-the-
  app philosophy, dogfooding bragfile, zero-dependency tooling) sourced
  from the project history; marked draft for editing into voice.
- README gains a Documentation section linking all of the above.

### Fixed

- **sed-injection hardening in `new-spec` / `new-stage`.** User-supplied
  titles (and the repo id) were substituted into templates via
  `sed "s|<Short Title>|${TITLE}|"` with no escaping. A title containing
  the `|` delimiter could close the s-command early, and a trailing `e`
  would reach GNU sed's execute flag — i.e. command injection. Not
  reachable as shipped (the placeholder sits on a `#`-prefixed markdown
  line, and BSD/macOS sed lacks the `s///e` flag), but it's one template
  edit away on GNU sed and already corrupts files on titles containing
  `|`, `&`, or `\`. Added `sed_escape_replacement` to `_lib.sh` (pure
  bash, escapes `\`, `|`, `&`) and routed the user-controlled
  substitutions through it. Hostile titles now render verbatim instead
  of executing or breaking. (+3 regression checks; 91 total.)

  This was found in a security audit of the template's bash scripts,
  justfile, and CI. No other code findings: `advance-cycle` allowlists
  cycle values, `archive-spec` is awk-only with validated IDs, there are
  no GitHub Actions workflows, and `.gitignore` excludes secrets.

## 2026-06-02 — Recommended-tools catalog + Mermaid convention (v5.7)

Consolidates optional, project-level tool guidance into one catalog and
makes Mermaid the blessed default for diagrams. No new dependencies.

### Added

- **`guidance/recommended-tools.md`** (both variants) — a single catalog
  of optional, project-level tool escalations, organized by concern
  (Diagrams, Testing/Verify, Decisions). States the template's stance
  once — zero-dependency defaults, with "reach for it when / skip it
  when" framing — so adopting any of them is a deliberate `DEC-*`, not a
  default. Covers Mermaid (default), Structurizr (optional C4),
  LineSpec (optional protocol tests), and native `decisions-audit` vs.
  LineSpec provenance.
- **Mermaid as the blessed diagram default.** AGENTS.md (both variants)
  Coding Conventions now states diagrams are Mermaid fenced blocks in
  markdown, updated in the same change as the work. Added a starter
  Mermaid ER diagram to `docs/data-model.md` to match the existing one
  in `docs/architecture.md`.

### Changed

- **Folded `guidance/verify-tooling.md` into `recommended-tools.md`**
  (single source of truth) and repointed the AGENTS.md verify-check and
  Pointers references in both variants. README now mentions the catalog.

## 2026-06-02 — Specs-by-stage ledger (v5.6)

Back-ported and modernized from a downstream project (bragfile000)
built on an earlier template. Fills the one gap the `status` /
`backlog` / `roadmap` trio left: a flat, every-spec ledger.

### Added

- **`just specs-by-stage`** — lists every spec grouped by stage, with
  status, ship date, and complexity. Defaults to **all projects** (a
  historical ledger — unlike `roadmap`/`backlog`, which scope to the
  active project). `--active` (or `--current`) scopes to the active
  project; a `PROJ-NNN` id (or full dir name) scopes to one project.
  Reads authoritative front-matter — `project.stage`, `task.cycle`,
  `task.complexity`, and the `ship` cost session's `recorded_at` for
  archived specs — rather than scraping backlog prose, so it stays
  accurate on the current data model. Also tallies un-promoted
  "(not yet written)" backlog bullets per stage. Read-only.
  (`scripts/specs-by-stage.sh`, `_lib.sh`-based; +5 test.sh checks)

  The original downstream version scraped inline `(shipped on …)` /
  `(S)` annotations out of stage backlogs; this rewrite derives the
  same facts from the fields the template already maintains. Other
  downstream scripts (`claude-code-post-session.sh`,
  `build`/`install`/`uninstall`/`test-docs`) were reviewed and left
  out as project-specific.

## 2026-06-02 — Decision auditing (v5.5)

A native, zero-dependency take on LineSpec-style provenance auditing,
plus a note on where heavier external verify tooling fits. The
template stays dependency-free; the new command is pure bash + awk +
git and works on bash 3.2 (macOS default).

### Added

- **`just decisions-audit`** — audits `decisions/*.md`. Default mode
  lints structure (filename ↔ `insight.id` match, no duplicate IDs,
  required `created_at`/`insight.type`, and bidirectional
  `supersedes`/`superseded_by` integrity — no dangling or one-sided
  links) and warns on scope overlap between active decisions. Exits 1
  on structural errors, so it drops into CI or a pre-commit hook.
  (`scripts/decisions-audit.sh`)
- **`just decisions-audit --changed [BASE]`** — flags which active
  decisions govern the files you're about to commit (working tree +
  staged + untracked, or `BASE...HEAD` with a ref). Advisory (exit 0):
  "re-read DEC-007 before changing this." This is the LineSpec
  `provenance audit` idea done natively.
- **Optional `affected_scope:` field** in the decision front-matter
  (both variants' `decisions/_template.md` and example `DEC-001`) — a
  glob list (`**` spans dirs, `*` within a segment) that powers the
  scope checks. Decisions without it are still linted; they're just
  skipped by the scope passes.
- **`guidance/verify-tooling.md`** (both variants) — documents
  LineSpec for protocol-level integration tests in the Verify phase as
  an *optional, project-level* choice, and explains why the decision
  side is native rather than a dependency.
- AGENTS.md (both variants): the Verify checklist's "decision drift"
  check now points at `just decisions-audit --changed`; Pointers gains
  the verify-tooling note. The build-cycle "Create `DEC-*`" step now
  instructs the agent to fill `affected_scope` for file-bound
  decisions, so the field gets populated at creation time rather than
  rotting empty.

## 2026-04-25 — Backlog and roadmap views (v5.4)

Two read-only views over existing data, answering different
questions at different grains. Together with `just status`, they
form a small "what's the state of work?" trio.

### Added

- **`just backlog`** — spec-grained "what's next" view. Surfaces
  three things `just status` deliberately doesn't: in-flight specs
  (cycle ≠ archived) in the active stage, un-promoted "(not yet
  written)" bullets in the active stage's `## Spec Backlog`, and
  counts of un-promoted bullets in upcoming stages. `--all`
  widens scope across stages. Read-only — no front-matter writes.
  Optional complexity tag (`[S]/[M]/[L]`) parsed if present in a
  backlog line; omitted otherwise. (`scripts/backlog.sh`)

- **`just roadmap`** — stage-grained "where is this project going"
  view. One row per stage with status (shipped / cancelled /
  active / upcoming), date range from existing front-matter
  (`created_at` → `shipped_at` for shipped/active, `target:
  target_complete` for upcoming), and spec counts for active and
  upcoming stages. Active stage row is bolded. (`scripts/roadmap.sh`)

- **`_lib.sh` helpers** for stage front-matter parsing:
  `get_active_stage_file` (lifted from inline use in
  `report_daily.sh`), `get_stage_status`, `get_stage_target`,
  `get_stage_created_at`, `get_stage_shipped_at`. Pure bash + awk;
  null-safe.

- **Both READMEs** mention the two new commands in the
  common-commands block.

- **7 new test assertions** (73 → 80 total): backlog header
  prints, surfaces un-promoted bullets, lists in-flight specs,
  `--all` exits cleanly; roadmap header prints, renders active
  stage with bucket, shows correct spec counts.

### Changed

- **`scripts/report_daily.sh`** uses the shared
  `get_active_stage_file` helper instead of an inline copy. No
  behavior change.

- **Both variants' Prompt 1d (Stage Ship)** gain one numbered step
  instructing the architect to flip `stage.status` to `shipped`
  and set `shipped_at` when wrapping up a stage. This keeps the
  new roadmap accurate without auto-modifying frontmatter from
  `archive-spec.sh` (which `KNOWN_LIMITATIONS.md` explicitly
  documents as deliberate).

### Design notes preserved

- No "accepted" state between bullet and spec. Running `just
  new-spec` is the acceptance.
- Backlog and roadmap stay separate views — one is spec-grained,
  the other stage-grained. Don't merge them.
- No existing front-matter renamed. Roadmap reads what's already
  there (`created_at`, `shipped_at`, `target_complete`).

## 2026-04-25 — Daily status snapshot command (v5.3.1)

Small follow-up. Mirrors a `just daily-status-report` command from a
downstream project (bragfile000) — a thin wrapper that captures
`just status` output to a dated markdown file, distinct from v5.2's
heavier `report-daily`.

### Added

- **`just daily-status-report`** — writes
  `reports/daily/YYYY-MM-DD-status.md` with the current `status.sh`
  output. Lighter than `report-daily`: no curation, no front-matter
  scraping, no git log. Co-located with `report-daily` under
  `reports/daily/`; the `-status.md` suffix distinguishes the two
  artifacts when both run on the same day.
- README mention in both variants' command list.
- Two new test assertions (71 → 73 total): file written at expected
  path, header carries today's date.

## 2026-04-22 — Instruction timeline convention (v5.3)

A small convention, not a mechanism. Every spec gets a peer
markdown timeline file tracking cycle instructions with status
markers. The architect writes cycle prompts to files instead of
leaving them in chat. Executors (build agent, verify reviewer,
shipper) read the prompt file and update the timeline as they go.
No dispatch commands, no MCP servers, no file watchers — just
markdown and discipline.

### Added

- **Timeline file per spec.** Lives at
  `projects/*/specs/SPEC-NNN-<slug>-timeline.md`. Four status
  markers: `[ ]` not started, `[~]` in progress, `[x]` complete,
  `[?]` blocked (with a one-line reason — needs human or external
  unblock; NOT a "I don't know what to do" dumping ground).
  Scaffolded alongside the spec by `just new-spec`, from
  `projects/_templates/timeline.md` (new, in both variants).

- **Per-project prompts directory** at
  `projects/*/specs/prompts/`. Architect writes the next cycle's
  prompt here (`SPEC-NNN-build.md`, `SPEC-NNN-ship.md`); executors
  read from here. Created lazily by `new-spec`.

- **AGENTS.md §9 Instruction Timeline** in both variants.
  Documents all four markers with the onboarding's discipline
  wording. Downstream sections renumbered; cross-references in
  both variants updated.

- **Example artifacts for SPEC-001** in both variants:
  `SPEC-001-example-project-logger-timeline.md` with `[x] design`
  completed and `[ ]` placeholders for build/verify/ship;
  `prompts/SPEC-001-design.md` (retrospective of the design
  prompt) and `prompts/SPEC-001-build.md` (forward-looking build
  prompt). Makes the convention concrete for anyone cloning fresh.

- **14 new test assertions** (57 → 71 total). Covers: timeline
  scaffold at the expected path, legend documents all four
  markers, `prompts/` directory exists, AGENTS.md section present,
  AGENTS.md documents all four markers, archive-spec co-moves the
  timeline into done/.

### Changed

- **`scripts/new-spec.sh`** scaffolds the timeline file + an empty
  `prompts/` directory in addition to the spec.

- **`scripts/archive-spec.sh`** co-archives the spec's timeline
  file into `done/`, keeping history paired.

- **`scripts/_lib.sh`:** `find_spec` now excludes `*-timeline.md`
  (the timeline filename shares the `SPEC-NNN-*` prefix with the
  spec, so the naive glob matched both). New helper
  `find_spec_timeline` locates the paired timeline by ID.

- **Both variants' `FIRST_SESSION_PROMPTS.md`** gain timeline
  instructions across four prompts:
  - 2b (Design): write `prompts/SPEC-NNN-build.md`; replace the
    timeline placeholder with `[x] design` + `[ ]` for later cycles.
  - 3 (Build): mark `[~]` before coding; mark `[x]` with PR/cost/
    date when done; `[?]` only for real blockers needing judgment.
  - 4 (Verify): mark `[~]` before reading; on APPROVED, write
    `prompts/SPEC-NNN-ship.md` and mark verify `[x]` with the SHA.
  - 5 (Ship): mark `[~]` at start; `[x]` with merge date and cost
    before archive.

- **Both variants' `GETTING_STARTED.md`** gain a short paragraph
  + example timeline block in Step 6 (First Spec) explaining the
  convention and reinforcing that the timeline is a dumb markdown
  file with no enforcement.

## 2026-04-21 — Reports, cost tracking, business value (v5.2)

Three bundled features that are tightly coupled: reports need value
structure to tell a project's story; reports need cost data to cover
AI spend; value and cost both live in spec/stage/project front-matter,
so touching those files once for both is cheaper than separate sessions.

Nothing breaks for existing projects — old specs without `cost:` or
`value_link:` advance through cycles and archive as before. Reports
degrade gracefully on pre-v5.2 data. See
`MIGRATION_TO_REPORTS_AND_COSTS.md` for details and optional
backfill blocks.

### Added

- **Business value structure** at project and stage level.
  - `value:` block in project-brief front-matter: `thesis`,
    `beneficiaries`, `success_signals`, `risks_to_thesis`. Testable
    claim, not marketing.
  - `value_contribution:` block in stage front-matter: `advances`,
    `delivers`, `explicitly_does_not`. What this stage advances and
    what it's explicitly not trying to do.
  - `value_link:` scalar on specs. Optional one-sentence reference
    back to the parent stage's value. `null` is acceptable.
  - Applied to both variants' `_templates/` directories in lockstep.
  - `variants/*/projects/_templates/project-brief.md`,
    `variants/*/projects/_templates/stage.md`,
    `variants/*/projects/_templates/spec.md`.

- **Self-reported AI cost** on every spec.
  - `cost.sessions[]` accumulates one entry per cycle (design, build,
    verify, ship). Each entry: `cycle`, `agent`, `interface`,
    `tokens_input`, `tokens_output`, `estimated_usd`,
    `duration_minutes`, `recorded_at`, `notes`.
  - `cost.totals` (`tokens_total`, `estimated_usd`, `session_count`)
    computed at ship.
  - `interface` is a free string. Known values: `claude-code`,
    `claude-ai`, `api`, `ollama`, `other`. Open for future agents.
  - Null numeric fields are fine — reports skip nulls in sums, count
    them in `session_count`.

- **Daily and weekly reports.** Two new commands:
  - `just report-daily` → `reports/daily/YYYY-MM-DD.md`. Sections:
    snapshot (specs by cycle with IDs, project progress), value
    (project thesis, stage advances, value_link population), activity
    today (files touched), cost activity (sessions, WIP cost, specs
    missing cost data), flags (stalled specs, stale decisions), 24h
    git activity.
  - `just report-weekly [YYYY-MM-DD]` →
    `reports/weekly/YYYY-WNN.md`. ISO week; optional date arg for
    back-dated weeks. Sections: summary, value advancement,
    shipped-this-week table, cost breakdown by cycle/interface/top-3,
    decision activity, reflection notes from shipped specs, flags.
  - Both: idempotent (re-run overwrites), graceful on pre-v5.2
    content, deterministic, no daemons.
  - Scripts: `scripts/report_daily.sh`, `scripts/report_weekly.sh`.

- **`scripts/_lib.sh` helpers** for value/cost parsing and portable
  dates. Pure bash + awk + `date`; no `yq` dependency.
  `find_all_specs`, `get_spec_cycle`, `sum_cost_tokens_for_spec`,
  `sum_cost_usd_for_spec`, `sessions_recorded_on`,
  `count_cost_sessions`, `extract_value_link`, `get_project_thesis`,
  `get_stage_value_contribution`, `days_ago`, `iso_week_number`,
  `iso_week_bounds`, `spec_mtime_date`.
  Cost parsers disambiguate `estimated_usd` in sessions vs totals
  by 6-space vs 4-space indent.

- **`reports/`** directory with `daily/` and `weekly/` subdirs;
  sample outputs from the example project committed so users see
  what reports look like before running them.

- **`feedback/`** as a known home for downstream user feedback.
  Rename of the bragfile NOTES file into the dated-slug convention;
  new `_template.md` with front-matter (source, captured_at,
  captured_by, status); `archive/` subdir for addressed/deferred
  items.

- **Prompt updates** across 6 prompts in both variants.
  - 1b Project Brief, 1c Stage Frame, 2b Spec Design: populate the
    new value/cost fields during design.
  - 1d Stage Ship, 1e Project Ship: cross-check shipped
    `value_link`/`value_contribution` against the parent thesis.
  - Prompt 3 Build, 4 Verify, 5 Ship: append cost sessions; compute
    totals at ship; Verify flags specs missing cost data without
    blocking.
  - Prompt 6 Weekly Review: report `value_link` population rate and
    aggregate costs.

- **AGENTS.md sections.** Both variants gained `## 3. Business Value`
  and `## 4. Cost Tracking Discipline` between the Work Hierarchy
  and Tech Stack sections. Cycle-Specific Rules updated to include
  cost-session appends. Downstream sections renumbered;
  cross-references to section 14 updated. `feedback/` and
  `reports/` added to the Directory Structure diagram and Pointers
  list.

- **`MIGRATION_TO_REPORTS_AND_COSTS.md`** at repo root — short,
  leads with "nothing breaks," includes optional backfill blocks.

- **27 new test assertions** in `scripts/test.sh` (30 → 57 total).
  Covers: v5.2 shape in scaffolded specs/stages, AGENTS.md new
  sections, `just report-daily` + `report-weekly` file writing and
  content, idempotency (re-run overwrites), and graceful handling
  of pre-v5.2 data (the critical backwards-compat guarantee).

### Changed

- **`justfile`** gains `report-daily` and `report-weekly` commands.
- **Both variants' README.md** gain a Reports section and the two
  new commands in their common-commands block. Claude-only's
  section-13 cross-reference updated to section 15 to track the
  AGENTS.md renumbering.

## 2026-04-20 — Hardening pass

First polish of the scripts after v5 delivery. Focus was bug-fix only,
exercised on macOS (the original build was tested on Ubuntu). No new
features, no variant dedup, no prompt changes.

### Fixed (follow-on, same-day — reported by downstream user building bragfile)

- **`archive-spec` stage-shipped message no longer falsely claims
  completion.** Archiving the last active spec under a stage used to
  print "All specs for STAGE-X are shipped", which was a false
  positive whenever the stage's Spec Backlog still listed unwritten
  specs. Reworded to "No active specs remain for STAGE-X" — an
  observation, not a completion claim. Stage completion judgment
  stays with the user (and the Stage Ship prompt).
  (`scripts/archive-spec.sh`)

- **Scaffolded specs and stages now pick up the real repo ID.**
  Every template hardcoded `id: my-app` in its `repo:` block;
  `.repo-context.yaml` had a "REPLACE" comment but nothing read
  that file. Even after the user updated `.repo-context.yaml`,
  every new spec/stage still stamped `my-app`. Same fix pattern as
  `__TODAY__`: templates use `__REPO_ID__`, `new-spec`/`new-stage`
  substitute the value parsed from `.repo-context.yaml`
  (`metadata.repo.id`), with `my-app` as the fallback so behavior
  never regresses on a pristine clone.
  (`scripts/_lib.sh`, `scripts/new-spec.sh`, `scripts/new-stage.sh`,
   `variants/*/projects/_templates/*.md`,
   `variants/*/decisions/_template.md`)

### Fixed

- **`just init` no longer silently half-initializes on re-run.** The
  recipe chained steps with `\ ;`, so a failed `cp` would still print
  `✓ Done`, write `.variant`, and leave a broken repo. Now chained with
  `&&` and guarded by a second check that aborts if `variants/` is
  missing. The "already initialized" hint now tells the truth: init is
  one-shot, restore from git or re-clone to start over.
  (`justfile`)

- **`advance-cycle` preserves the cycle-legend inline comment.**
  `update_frontmatter_scalar` used to wipe everything after `:`,
  stripping `# frame | design | build | verify | ship` on first use.
  The updater now preserves any trailing `# …` comment.
  (`scripts/_lib.sh`)

- **`archive-spec` refuses to re-archive a shipped spec.** Running
  `archive-spec SPEC-NNN` twice used to produce `specs/done/done/…`
  because `find_spec` happily returned already-archived files. Now
  `find_spec` excludes `*/done/*`, so both `archive-spec` and
  `advance-cycle` fail loudly with `Spec not found` on archived specs.
  (`scripts/_lib.sh`)

- **`weekly-review` emits repo-relative paths consistently.** Files
  discovered via `find` printed with absolute paths while hand-listed
  files were relative. The prompt promises "paths relative to repo
  root"; this change makes the output match.
  (`scripts/weekly-review.sh`)

- **`YYYY-MM-DD` → today's-date substitution no longer touches comment
  lines.** Templates used the same token for real placeholder values
  and format-documentation comments like `# optional: YYYY-MM-DD`.
  After substitution the comment read like a real target date. Real
  placeholders are now `__TODAY__`; format comments stay as
  `YYYY-MM-DD`.
  (`variants/*/projects/_templates/spec.md`,
   `variants/*/projects/_templates/stage.md`,
   `scripts/new-spec.sh`, `scripts/new-stage.sh`)

- **Removed dangling `just new-project` references.** The die() message
  in `_lib.sh` and the example `brief.md` in both variants pointed
  users at a command that doesn't exist. Replaced with accurate
  instructions (copy `projects/_templates/project-brief.md` into
  `projects/PROJ-NNN-<slug>/brief.md`).
  (`scripts/_lib.sh`, `variants/*/projects/PROJ-001-example-mvp/brief.md`)

### Added

- **`just test` / `scripts/test.sh`** — end-to-end happy-path test that
  spins up a temp copy, runs init + full cycle + archive + weekly-review,
  and asserts the invariants the fixes above depend on. No new deps.
  Intended for template maintainers.
