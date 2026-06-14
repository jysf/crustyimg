# SPEC-001 — SHIP prompt

> **Use when:** verify APPROVED. Gated by the orchestrator.
> **Approved at commit:** `8d74a78` (PR #1, head `8d74a7879d20a4c659a2e39f34d600ce6bb26918`)
> **Time:** 5-10 min.

```
Cycle: ship. PR #1 for SPEC-001 is APPROVED (verify ✅ at commit 8d74a78).
Branch feat/spec-001-cargo-project-and-multi-os-ci → main.

Before starting, mark ship `[~]` in
  projects/PROJ-001-crustyimg-mvp/specs/SPEC-001-cargo-project-and-multi-os-ci-timeline.md

Pre-ship checklist (confirm each before merging):
[ ] CI passing? — verify reported 6/6 green (ubuntu/macos/windows ×
    build·test·clippy·fmt). Re-confirm with `gh pr checks 1` — must
    still be green at the head SHA you merge.
[ ] Deployment steps? — none. This is a library+binary scaffold; no
    deploy target, no release/publish in STAGE-001 (crates.io/brew are
    a later project concern, out of scope here).
[ ] Rollback plan? — none required; revert the merge commit if needed.
    Greenfield, no consumers yet.
[ ] CHANGELOG? — no CHANGELOG exists in the repo yet and the spec did
    not introduce one; nothing to update. (If you decide a CHANGELOG
    should start, that is a follow-up, not a ship blocker.)

Merge the PR (squash or merge per AGENTS.md §13 git/PR conventions):
  gh pr merge 1 --squash --delete-branch   # or per repo convention

After merge, answer three reflection questions and append them to the
spec as a "## Reflection (Ship)" block (the spec already has the
heading stub at the bottom — fill in the three answers). These are
OUTCOME-focused, distinct from the process-focused build reflection:

1. What would I do differently next time?
   [REPLACE: answer]

2. Does any template, constraint, or decision need updating?
   [REPLACE: answer]

3. Is there a follow-up spec to write before I forget?
   [REPLACE: answer — note the known follow-up: the `--features mozjpeg`
   CI job (DEC-009) is deferred until a codec feature exists; it lands
   with the spec that introduces the native-codec dependency. SPEC-002..007
   already cover the planned stage work, so likely no NEW spec is needed.]

Then:
- Append a ship cost session entry to `cost.sessions` (cycle: ship,
  agent: claude-opus-4-8, interface: claude-code, null numerics,
  duration_minutes estimate, recorded_at 2026-06-13, notes:
  "subagent; cost not separately reported").
- Compute `cost.totals` from the session entries:
  * tokens_total = sum(tokens_input + tokens_output) skipping nulls → 0
    (all four cycles report null token fields)
  * estimated_usd = sum(estimated_usd) skipping nulls → 0
  * session_count = len(sessions) → 4 (design, build, verify, ship)
  Reports will show "partial cost data available" given the nulls — that
  is expected for subagent runs.
- Mark ship `[x]` in the timeline with the merge date and the cost
  summary, referencing this prompt (prompts/SPEC-001-ship.md).
- Run: just advance-cycle SPEC-001 ship
- Run: just archive-spec SPEC-001   (also moves the timeline into done/)
- Update STAGE-001 backlog: flip
    `- [ ] SPEC-001 (build) — Cargo project + multi-OS CI ...`
  to
    `- [x] SPEC-001 (shipped on 2026-06-13) — Cargo project + multi-OS CI ...`
  and update the **Count** line (1 shipped / 0 active / 6 pending).
- If you concluded any template/constraint/decision needs updating in
  the reflection, propose the edits.
- If a follow-up spec is warranted, add it to STAGE-001's backlog.

SPEC-001 is the FIRST spec in STAGE-001's backlog, not the last — do
NOT run Prompt 1d (Stage Ship) yet; six specs remain.
```
