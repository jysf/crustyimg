# Reports + Cost + Value Session Report — 2026-04-21

Narrative record of the Claude Code session that added the v5.2
feature set (business value structure, self-reported AI cost, daily
and weekly reports) on top of the v5.1 hardened template.

If you're the next session: start here, then read `CHANGELOG.md` for
the condensed list and `KNOWN_LIMITATIONS.md` for what's still
unfixed.

---

## 1. What this session was

**Trigger:** the user placed `docs/sessions/2026-04-21-reports-costs-value-onboarding.md`
in the repo and asked me to follow its process. That onboarding doc
specified three bundled features (value structure, cost tracking,
reports), the order to build them, and a Section 14 of questions to
ask upfront.

**User answers to the Section 14 questions:**

1. Hardening session complete, `just test` passes with PASS 30 checks. ✓
2. No migration needed — the user's current project has 10 specs but
   they're starting a new one once v5.2 lands. Migration doc kept
   short accordingly.
3. Yes, commit sample reports generated from the example project.
4. `just report-daily` can write-then-`cat` to stdout — fine.
5. Interface enum should be extensible — user mentioned adding
   Ollama-backed agents later. Kept `interface` as a documented free
   string with `ollama` in the examples.

**Contradictions I flagged before starting:**

- `feedback/` already existed (with the NOTES file inside), not at
  repo root as the onboarding implied — so the restructure was an
  in-directory rename plus scaffolding `_template.md` and `archive/`.
- `project-brief.md` and `stage.md` already carry front-matter YAML,
  so the value blocks landed in front-matter (the onboarding's
  explicit override rule) rather than as fenced markdown blocks.
- Adding `Business Value` and `Cost Tracking Discipline` between
  AGENTS.md §2 and §3 required renumbering §3–§15 and updating the
  `See Section 14` cross-reference. Done in full.
- Prompt names (1a–6) matched in both variants, so lockstep updates
  worked cleanly.

**Design question resolved before starting:** default for
`value_link` when unknown — `null` rather than `""`, since reports
already treat null as "no contribution stated."

---

## 2. Scope the user approved

Implement everything in the onboarding's Section 10 order, with two
simplifications:

- Migration doc short (user isn't migrating).
- Interface enum extensible (documented free string, not validated).

Everything else stuck to the onboarding as written.

---

## 3. Order of work and commits shipped

17 commits against the hardened baseline (`13cbf94` + this chain).
One commit per meaningful unit, both variants kept in lockstep.

### Feedback directory (1 commit)

- `1a4d00f docs(feedback): rename NOTES file, add _template.md and archive/`

Renamed `feedback/NOTES_FROM_BRAGFILE_EXPERIMENT_000.md` to the
dated-slug convention, added a front-matter-bearing `_template.md`
for future captures, and `archive/.gitkeep`.

### Value structure + cost block in templates (2 commits)

- `099f60f feat(templates): add business value structure to project/stage/spec`
- `e642c2c feat(templates): add self-reported cost block to spec front-matter`

Project-brief gains `value:` (thesis, beneficiaries,
success_signals, risks_to_thesis). Stage gains `value_contribution:`
(advances, delivers, explicitly_does_not). Spec gains `value_link:`
(optional one-line scalar) and `cost:` (empty sessions list + zeroed
totals).

### Prompt updates (2 commits)

- `55398e8 docs(prompts): add business-value instructions across 6 prompts`
- `1498297 docs(prompts): add self-reported AI cost instructions across 5 prompts`

Both variants' FIRST_SESSION_PROMPTS.md updated for 1b (Project
Brief), 1c (Stage Frame), 1d (Stage Ship), 1e (Project Ship), 2b
(Spec Design), 3 (Build), 4 (Verify), 5 (Ship), and 6 (Weekly
Review). Ship prompt now instructs the agent to append its own
cost session entry and compute `cost.totals` before archive.

### AGENTS.md and README (2 commits)

- `424853d docs(agents): add Business Value and Cost Tracking sections`
- `a6a0a07 docs(readme): mention report commands and the reports/ directory`

AGENTS.md §3 and §4 added; downstream sections renumbered (§3–§15
→ §5–§17); the `See Section 14` cross-reference in §15 updated to
§16. Directory Structure gains `feedback/` and `reports/`; Pointers
list same. READMEs gain a Reports section and the two new commands
in their common-commands block.

### Shell infrastructure (3 commits)

- `ab4804c feat(lib): add value/cost parsers and portable date helpers`
- `bcdb117 feat(reports): add report_daily.sh + reports/ skeleton`
- `97db4e8 feat(reports): add report_weekly.sh`

Pure bash + awk + `date`; no `yq`. Helpers disambiguate
`estimated_usd` in session entries (6-space indent) from
`estimated_usd` in totals (4-space indent). Hand-verified each
helper against a fixture spec before shipping.

Gotcha fix during weekly-report build: `read -r A B < <(cmd | tr '\n' ' ')`
trips `set -e` because `read` returns 1 on EOF-without-newline.
Switched to splitting bounds via `sed -n` onto named vars.

### Justfile wiring + sample reports + test harness (3 commits)

- `17ac3bf feat(justfile): add report-daily and report-weekly commands`
- `97a3aac docs(reports): commit sample daily + weekly reports from example project`
- `721dd82 test: extend harness with 27 v5.2 assertions (30 → 57 total)`

Sample reports committed so users see what output looks like before
running. They also demonstrate graceful degradation: the example
project ships pre-v5.2, so `Project thesis: *(not set)*`, `Specs
with no cost data yet`, etc. appear in the samples.

Test harness added 27 assertions covering the v5.2 shape of
scaffolded specs and stages, AGENTS.md new sections, report file
writing and content, idempotency (re-run overwrites), and
graceful handling of pre-v5.2 data.

### Migration doc + CHANGELOG (2 commits)

- `8c9b7d8 docs: add MIGRATION_TO_REPORTS_AND_COSTS.md`
- `bf33142 docs(changelog): add v5.2 section for reports + cost + value`

Migration doc leads with "nothing breaks," includes optional
backfill blocks for users who want to add value structure to an
existing project.

### License (1 commit, user-requested mid-session)

- `f57a620 chore(license): switch from MIT to Apache 2.0`

Nameless copyright line.

### Session hygiene — at session close

This report + moving `SESSION_REPORT.md` into `docs/sessions/` to
match where this session's onboarding doc lives.

---

## 4. Test harness — what's covered now

57 assertions, all passing at session close. The 27 new ones:

| # | Area | Assertion |
|---|---|---|
| §8 | v5.2 spec shape | scaffold has value_link: null, cost: block, sessions:[], totals zeroed |
| §9 | v5.2 stage shape | scaffold has value_contribution block with starting values |
| §10 | AGENTS.md | Business Value (§3) and Cost Tracking (§4) sections present post-init |
| §11 | report-daily | writes file, prints header, has expected sections, handles missing thesis, idempotent |
| §12 | report-weekly | writes file, correct sections, picks up freshly-archived spec |
| §13 | backwards compat | daily report flags pre-v5.2 specs as missing cost data |

Run: `just test`. Pre-init root; `variants/` must still exist.

---

## 5. What's deliberately deferred (out of scope for v5.2)

- Monthly reports — daily + weekly first.
- Narrative generation in reports — on-demand via Claude.
- Budget tracking, cost alerts, spending caps.
- Visualization (charts, graphs).
- Admin API integration for authoritative cost data.
- JSONL parsing of `~/.claude/projects/` (self-report supersedes).
- Cost pricing tables (agents self-estimate).
- Per-user cost breakdown.
- Variant dedup / shared template system.
- "Fresh session is weaker than it looks" feedback from bragfile —
  still open, now explicitly a design question for a later session
  about multi-agent discipline.
- A third variant (`claude-multi-model/`).
- Renaming cycles or other structural concepts.

---

## 6. Known limitations introduced in v5.2

- **`cost:` block parsing assumes exact 6-space indent** for session
  scalar fields and 4-space for totals. Hand-written blocks with
  off-indent YAML will silently drop out of sums. Documented in
  `MIGRATION_TO_REPORTS_AND_COSTS.md`.
- **Cycle time trends in weekly report are not computed.** The
  onboarding doc lists them; implementing required cycle-advance
  timestamps that don't exist in the template. Deferred — would
  need either git log-based heuristics or a new cycle-history
  field.
- **Weekly report comparison-to-last-week is not implemented.** The
  onboarding lists it; skipped to keep scope tight. Trivial to add
  later when there are real prior reports to diff against.
- **`spec_mtime_date` is used as a ship-date proxy.** Works because
  `archive-spec` touches the file on move, but a subsequent edit to
  an archived spec (rare but possible) would bump the date. Good
  enough for v5.2; worth revisiting if someone reports a
  miscategorized spec.

---

## 7. Files touched — quick map

- Templates (both variants):
  - `projects/_templates/project-brief.md` — value: block
  - `projects/_templates/stage.md` — value_contribution: block
  - `projects/_templates/spec.md` — value_link: + cost: blocks
- Prompts (both variants): `FIRST_SESSION_PROMPTS.md`
- AGENTS.md (both variants) — two new sections + renumbering
- README (both variants) — Reports section + commands
- Scripts: `_lib.sh` (helpers), `report_daily.sh` (new),
  `report_weekly.sh` (new), `test.sh` (extended)
- Root: `justfile` (two new commands), `CHANGELOG.md`,
  `MIGRATION_TO_REPORTS_AND_COSTS.md` (new), `LICENSE` (MIT →
  Apache 2.0), `SESSION_REPORT.md` → `docs/sessions/2026-04-20-hardening-report.md`
- Reports: `reports/daily/2026-04-21.md`, `reports/weekly/2026-W17.md`
  (committed samples)
- Feedback: `feedback/_template.md`, `feedback/archive/.gitkeep`,
  `feedback/2026-04-20-bragfile-project.md` (rename of the NOTES
  file)

---

## 8. How to verify (for the next session)

```bash
# 1. Full happy-path test — should print "PASS 57 checks".
just test

# 2. Exercise reports manually in a scratch dir:
tmp=$(mktemp -d); cp -R . "$tmp/repo"; rm -rf "$tmp/repo/.git"; cd "$tmp/repo"
printf "1\n" | just init
just report-daily   # writes reports/daily/YYYY-MM-DD.md, cats output
just report-weekly  # writes reports/weekly/YYYY-WNN.md, cats output

# 3. Scaffold a v5.2 spec and inspect:
just new-spec "Demo" STAGE-001
head -40 projects/PROJ-001-example-mvp/specs/SPEC-*-demo.md
# Expect: value_link: null and cost: block at bottom of front-matter.

# 4. Grep sanity:
grep -r "value_link" variants/    # should match both variants' spec templates
grep -rn "Cost Tracking" variants/  # should match both variants' AGENTS.md
```

All of #1 is automated.

---

## 9. Open items for the next session

### From user mid-session

- **License switch to Apache 2.0** landed this session with a
  nameless copyright line. If the user wants to add a name/org
  later, it's a one-line edit in `LICENSE`.

### Noted during the pass

- **Cycle time trends in weekly report** — see limitations §6.
- **Comparison to last week in weekly report** — see limitations §6.
- **`spec_mtime_date` as ship-date proxy** — see limitations §6.
- **Bragfile feedback item 3 ("Fresh session is weaker than it
  looks")** — still open from the hardening session, deliberately
  deferred again. Was scoped out of v5.2 because it's a
  methodology question, not a script bug.
- **Variant dedup / shared template system** — still a known
  limitation. V5.2 touched 6 files across both variants for value
  alone, plus 2 for cost, plus prompts, plus AGENTS.md. The
  duplication tax grew this session; worth revisiting.

### Things I considered and decided not to do

- Auto-backfill `cost:`/`value_link:` into the example project's
  SPEC-001 — the onboarding explicitly said don't auto-modify
  existing specs, and having the example stay pre-v5.2 turned out
  to be useful for backwards-compat testing.
- Rename `weekly-review` to something like `weekly-review-prompt`
  to distinguish from the new `report-weekly` — confusing names are
  annoying but renaming a shipped command is worse.
- Add monthly reports — explicit out-of-scope.

---

## 10. Commit log (reference)

```
f57a620 chore(license): switch from MIT to Apache 2.0
bf33142 docs(changelog): add v5.2 section for reports + cost + value
8c9b7d8 docs: add MIGRATION_TO_REPORTS_AND_COSTS.md
97a3aac docs(reports): commit sample daily + weekly reports from example project
721dd82 test: extend harness with 27 v5.2 assertions (30 → 57 total)
17ac3bf feat(justfile): add report-daily and report-weekly commands
97db4e8 feat(reports): add report_weekly.sh
bcdb117 feat(reports): add report_daily.sh + reports/ skeleton
ab4804c feat(lib): add value/cost parsers and portable date helpers
a6a0a07 docs(readme): mention report commands and the reports/ directory
424853d docs(agents): add Business Value and Cost Tracking sections
1498297 docs(prompts): add self-reported AI cost instructions across 5 prompts
55398e8 docs(prompts): add business-value instructions across 6 prompts
e642c2c feat(templates): add self-reported cost block to spec front-matter
099f60f feat(templates): add business value structure to project/stage/spec
1a4d00f docs(feedback): rename NOTES file, add _template.md and archive/
13cbf94 add feedback                               # pre-session
```

End of session, 2026-04-21.
