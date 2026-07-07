---
# Maps to ContextCore epic-level conventions.

stage:
  id: STAGE-015
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-004
repo:
  id: crustyimg

created_at: 2026-07-06
shipped_at: null

value_contribution:
  advances: >
    Turns the linter into a one-line CI win and cuts the 0.4.0 release. Delivers the full adoption
    surface — the two composite GitHub Actions (in their own repos, per DEC-051), the in-repo
    pre-commit hook + `just lint-images` recipe + CI docs, SARIF for GitHub code-scanning, and the
    0.4.0 release bookkeeping. (The maintainer pulled the Actions INTO this wave 2026-07-06 so 0.4.0
    can announce them.)
  delivers:
    - "`setup-crustyimg` (installer Action) + `crustyimg-action` (lint/optimize wrapper) — two
      composite Actions in their own public repos, wrapping the cargo-dist installer + the
      `crustyimg.lint/v1` JSON report; no new crate dep (SPEC-057, DEC-051)"
    - "in-repo glue: a `pre-commit` hook config + a `just lint-images` recipe + a Continuous
      integration docs section — the format-aware upgrade from `check-added-large-files` (SPEC-057)"
    - "`lint --format sarif`: GitHub code-scanning output (opt-in second tier), hand-rolled (no dep)
      (SPEC-056)"
    - "the 0.4.0 release bookkeeping (CHANGELOG + version), announcing lint + the Actions (SPEC-056)"
  explicitly_does_not:
    - Build the autofix / commit-back Action mode — a v2 (fork-safety + write permissions);
      README-noted only
    - Add a new default dependency (the Actions are packaging, not crate code)
---

# STAGE-015: CI integration & adoption

## What This Stage Is

The stage that makes `lint` trivially adoptable and ships the wave: the **two composite GitHub
Actions** (`setup-crustyimg` + `crustyimg-action`, in their own repos per DEC-051), the in-repo glue
(a `pre-commit` hook, a `just lint-images` recipe, a Continuous-integration docs section), SARIF
output for GitHub code-scanning, and the 0.4.0 release bookkeeping. The Actions wrap the shipped
binary + exit code + `crustyimg.lint/v1` JSON report, so they add **no new crate dependency**.

> Framing change (2026-07-06): the maintainer pulled the two Actions **into** this wave (they were
> originally deferred as "separate repos, out of scope") so 0.4.0 can announce them. SPEC-057 owns
> the Actions + the in-repo glue; SPEC-056 keeps SARIF + the release-cut.

## Why Now

- **It's the payoff:** a linter no one runs has no value. "Drop image linting into any CI in three
  lines" is the on-ramp — `setup-crustyimg` + `crustyimg lint` (or the `crustyimg-action`).
- **The binaries already ship** via cargo-dist (v0.3.1 live), with a checksum-verifying installer to
  wrap — so the Actions are buildable now, before the 0.4.0 cut.
- **SARIF is the GitHub-native surface** (code-scanning) and is hand-rollable (no dep), matching the
  JSON/`write_diff_json` precedent.

## Success Criteria

- `setup-crustyimg` + `crustyimg-action` install + lint on the 3-OS matrix (their own self-tests
  green); a GPS-leaking fixture yields PR annotations + a failing job. No new crate dependency.
- The in-repo pre-commit hook + `just lint-images` run the linter over an asset glob with the right
  exit semantics; the README has a copy-paste Actions + pre-commit snippet.
- `lint --format sarif` emits valid SARIF for GitHub code-scanning; 0.4.0 CHANGELOG + version bump
  staged (tag/publish is the maintainer's outward step, per RELEASING.md). `just deny` green.

## Scope

### In scope
- The two composite Actions (`setup-crustyimg` + `crustyimg-action`, own repos) + the in-repo glue
  (`.pre-commit-hooks.yaml`, `just lint-images`, Continuous-integration docs). **(SPEC-057)**
- `lint --format sarif` (hand-rolled). **(SPEC-056)**
- 0.4.0 release bookkeeping (CHANGELOG + version), announcing lint + the Actions. **(SPEC-056)**

### Explicitly out of scope
- The autofix / commit-back Action mode — a v2 (fork-safety + write permissions); README-noted only.
- Tagging the Actions `v1` / GitHub Marketplace listing — the maintainer's outward step (SPEC-057
  ships repos created + pushed + self-test green; the tag/listing is deferred, like the crate tag).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-057 (design → build NEXT) — the GitHub Actions on-ramp: `setup-crustyimg` (installer,
  wraps the cargo-dist installer) + `crustyimg-action` (lint/optimize wrapper, JSON → annotations) in
  their own repos, + the in-repo glue (`.pre-commit-hooks.yaml`, `just lint-images`, CI docs).
  DEC-051 pins the design. **The last piece before the 0.4.0 cut.**
- [ ] SPEC-056 (not yet written) — `lint --format sarif` (hand-rolled, no dep) + the SARIF
  code-scanning docs; then cut 0.4.0 (CHANGELOG + version, untagged) announcing lint + the Actions.

**Count:** 0 shipped / 0 active / 2 pending

## Dependencies

### Depends on
- STAGE-013 — the shipped `lint` command + `crustyimg.lint/v1` JSON report the Actions wrap.
- DEC-040 (cargo-dist) — the release installer `setup-crustyimg` wraps (v0.3.1 live).
- STAGE-014 (for SARIF's full rule set) — SPEC-056 only.

### Enables
- 0.4.0's release notes: "crustyimg lint + a `setup-crustyimg` Action + a `crustyimg-action` lint
  mode — drop image linting into any CI in three lines."

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
