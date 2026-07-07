---
# Maps to ContextCore epic-level conventions.

stage:
  id: STAGE-015
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium
  target_complete: null

project:
  id: PROJ-004
repo:
  id: crustyimg

created_at: 2026-07-06
shipped_at: null

value_contribution:
  advances: >
    Turns the linter into a one-line CI win and cuts the 0.4.0 release. Delivers the in-repo
    adoption surface — SARIF for GitHub code-scanning, a pre-commit hook, a `just lint-images`
    recipe, and CI docs. (The GitHub Actions that wrap it live in SEPARATE repos, Track B.)
  delivers:
    - "`lint --format sarif`: GitHub code-scanning output (opt-in second tier), hand-rolled (no dep)"
    - "a `pre-commit` hook config + a `just lint-images` recipe — the format-aware upgrade from
      `check-added-large-files`"
    - "CI docs/examples (a GitHub-Actions snippet using the plain binary + exit code) and the 0.4.0
      release bookkeeping (CHANGELOG + version)"
  explicitly_does_not:
    - Build the `setup-crustyimg` / `crustyimg-action` composite Actions — those are SEPARATE repos
      (packaging, PR annotations, commit-back autofix); this stage ships what they wrap
    - Add a new default dependency
---

# STAGE-015: CI integration & adoption

## What This Stage Is

The stage that makes `lint` trivially adoptable and ships the wave: SARIF output for GitHub
code-scanning, a `pre-commit` hook and a `just lint-images` recipe, CI docs, and the 0.4.0 release
bookkeeping. The single highest-leverage adoption move — the GitHub Action — lives in a **separate
repo**; this stage delivers the binary + exit code + SARIF + hook it wraps, so the Action is thin.

## Why Now

- **It's the payoff:** a linter no one runs has no value. One binary + an exit code + a
  copy-paste CI snippet is the on-ramp.
- **SARIF is the GitHub-native surface** (code-scanning annotations) and is hand-rollable (no dep),
  matching the JSON/`write_diff_json` precedent.

## Success Criteria

- `lint --format sarif` emits valid SARIF for GitHub code-scanning; the pre-commit hook + `just
  lint-images` run the linter over an asset glob with the right exit semantics.
- CI docs show a working GitHub-Actions snippet using the plain binary; 0.4.0 CHANGELOG + version
  bump staged (tag/publish is the maintainer's outward step, per RELEASING.md).
- No new default dependency; `just deny` green.

## Scope

### In scope
- `lint --format sarif` (hand-rolled) + the pre-commit hook + `just lint-images` + CI docs.
  **(SPEC-056)**
- 0.4.0 release bookkeeping (CHANGELOG + version). **(part of SPEC-056 ship, or a chore.)**

### Explicitly out of scope
- The `setup-crustyimg` / `crustyimg-action` repos (Track B, separate). Commit-back autofix mode.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-056 (not yet written) — `lint --format sarif` + pre-commit hook + `just lint-images` +
  CI docs/examples; then cut 0.4.0 (CHANGELOG + version, untagged).

**Count:** 0 shipped / 0 active / 1 pending

## Dependencies

### Depends on
- STAGE-013 + STAGE-014 — the full rule set + the report framework SARIF serializes.

### Enables
- The separate `crustyimg-action` / `setup-crustyimg` repos (Track B) — wrap this binary.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
