---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-057
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L

project:
  id: PROJ-004
  stage: STAGE-015
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-06

references:
  decisions: [DEC-051, DEC-050, DEC-040, DEC-025]
  constraints: [no-new-top-level-deps-without-decision, ergonomic-defaults]
  related_specs: [SPEC-052, SPEC-053, SPEC-056]

value_link: >
  The single highest-leverage distribution move: turns `crustyimg lint` into a
  one-line CI win — a `setup-crustyimg` installer Action + a `crustyimg-action`
  lint wrapper + in-repo pre-commit/recipe/docs glue — so 0.4.0 can announce
  "drop image linting into any CI in three lines".

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-057: the GitHub Actions adoption on-ramp

## Context

STAGE-013 shipped `crustyimg lint` (format-aware, exit-coded, stable `crustyimg.lint/v1` JSON). The
payoff is making it trivially adoptable. Per **DEC-051**, the on-ramp is two small composite Actions
in their **own public repos** (an Action must live at a repo root to be `uses:`-able) plus in-repo
glue — wrapping the **existing cargo-dist installer** (checksum-verifying, os/arch-aware) rather than
hand-rolling a download matrix, and adding **no new crate dependency**. This lands before the 0.4.0
release-cut (SPEC-056) so the notes can point at it.

## Goal

Produce (1) `jysf/setup-crustyimg` — a generic installer Action; (2) `jysf/crustyimg-action` — a
lint/optimize wrapper that annotates findings; and (3) in-repo glue — `.pre-commit-hooks.yaml`, a
`just lint-images` recipe, and a **Continuous integration** docs section. Each Action repo carries a
3-OS self-test workflow (its CI, since it can't run this repo's). No new crate dependency.

## Inputs

- **Files to read:**
  - `DEC-051` — the on-ramp design (wrap the installer, two repos, no crate dep, deferred autofix).
  - The real release assets — `gh release view v0.3.1 --json assets`: the `crustyimg-installer.sh`/
    `.ps1` (version-pinned by release-tag URL; honors `CRUSTYIMG_INSTALL_DIR` +
    `CRUSTYIMG_NO_MODIFY_PATH`; verifies sha256 via `verify_checksum`) + per-platform tarballs.
  - `src/lint/report.rs` / SPEC-052 — the `crustyimg.lint/v1` schema the wrapper parses (`findings[]`
    `{file, rule, severity, message, fix, bytes_saved?}` + `summary{…, passed}`).
  - `justfile` — the recipe conventions to mirror for `just lint-images`.
  - `README.md` — §Usage / §Changelog; add a **Continuous integration** section.

## Outputs

- **Produced artifacts (separate repos — the deliverable this spec creates):**
  - `jysf/setup-crustyimg`: `action.yml` (composite; `version` input, cargo-dist installer branch on
    `runner.os`, `$GITHUB_PATH`, `actions/cache`), `README.md` (copy-paste example + the remaining
    tag/Marketplace step), `.github/workflows/self-test.yml` (3-OS: assert `crustyimg --version` + a
    trivial `optimize` and `lint`).
  - `jysf/crustyimg-action`: `action.yml` (composite; `uses: jysf/setup-crustyimg`, inputs `mode`
    (lint|optimize) / `paths` / `args` / `version` / `fail-level`; lint mode → `--format json` →
    `::error/::warning/::notice` annotations + job-summary table + exit propagation), `README.md`
    (example + deferred-autofix note + tag step), `.github/workflows/self-test.yml` (3-OS, pointed at
    a tiny fixture tree with a known GPS leak so annotations + non-zero exit are exercised).
- **Files created/modified (crustyimg repo — testable here):**
  - `.pre-commit-hooks.yaml` (new) — a `crustyimg-lint` hook (`language: rust`, `types: [image]`).
  - `justfile` — a `lint-images` recipe.
  - `README.md` — a **Continuous integration** section (the Actions + the plain-binary + pre-commit).
- **Database changes:** none.

## Acceptance Criteria

- [ ] **`setup-crustyimg`**: a composite `action.yml` installs a checksum-verified crustyimg via the
  cargo-dist installer, pinned to the `version` input (`latest` default → resolved release tag), adds
  it to `$GITHUB_PATH`, and caches it. Its 3-OS self-test (Linux/macOS/Windows) passes:
  `crustyimg --version` + a trivial `optimize` and `lint` succeed. Generic — installs the binary, not
  only lint. No hardcoded asset names/URLs (derived from the release tag).
- [ ] **`crustyimg-action`**: `uses: jysf/setup-crustyimg`, runs `crustyimg lint <paths> --format
  json`, emits native `::error`/`::warning`/`::notice` annotations + a job-summary table, and
  propagates pass/fail per `fail-level` (error|warn|never). An `optimize` mode runs `optimize`.
  Autofix/commit-back is **not** built (README notes it as v2). Its 3-OS self-test passes: a
  GPS-leaking fixture yields an annotation + a non-zero job (and `fail-level: never` stays green).
- [ ] **In-repo glue**: `.pre-commit-hooks.yaml` defines a valid `crustyimg-lint` hook; `just
  lint-images <paths>` runs the linter with the right exit semantics; the README gains a **Continuous
  integration** section with a copy-paste Actions snippet (lint + optimize) and the pre-commit block.
- [ ] **No new crate dependency**; `just deny` green; every existing test stays green. The Actions add
  nothing to `Cargo.toml`.
- [ ] Cross-OS: each Action's self-test runs the 3-OS matrix and is green on a real Actions run.
- [ ] The outward boundary is respected: repos created + pushed + self-test green is done; **tagging
  `v1` + Marketplace listing is left to the maintainer** and documented in each README.

## Failing Tests

The Action repos are validated by their **own self-test workflows** (real 3-OS Actions runs — the
build cycle waits for them to go green). In the crustyimg repo, the glue is verified by:

- **`tests/adoption_glue.rs` (integration)**
  - `".pre-commit-hooks.yaml exists, is valid YAML, and defines the crustyimg-lint hook (entry
    'crustyimg lint', language rust)"`.
  - `"just lint-images over a clean fixture dir exits 0; over a GPS-leaking fixture dir exits 7"`
    (drives the recipe / the binary end-to-end).
- **README check:** the **Continuous integration** section contains `uses: jysf/setup-crustyimg` and
  a `crustyimg lint` invocation (asserted by the pre-commit-hooks/README test, or a doc-presence
  check).

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply
- `DEC-051` (this on-ramp) — wrap the cargo-dist installer; two composite-action repos; no crate dep;
  defer autofix.
- `DEC-050` — the wrapper reads the stable `crustyimg.lint/v1` schema + stable rule ids.
- `DEC-040` — the cargo-dist pipeline that produces the installer the setup Action wraps.
- `DEC-025` — the exit-7 gate the wrapper propagates.

### Constraints that apply
- `no-new-top-level-deps-without-decision` — the Actions are packaging; zero crate deps.
- `ergonomic-defaults` — `setup-crustyimg` defaults to `latest`; `crustyimg-action` defaults to
  `mode: lint`, `paths: .`, `fail-level: error`; zero-config works.

### Prior related work
- `SPEC-052` (shipped) — `--format json` (`crustyimg.lint/v1`), the wrapper's input.
- `SPEC-053` (shipped) — the rule catalog whose findings become annotations.
- `SPEC-056` (STAGE-015, later) — `--format sarif` + the 0.4.0 release-cut that announces these.

### Out of scope (for this spec specifically)
- `lint --format sarif` — SPEC-056. The 0.4.0 release-cut/CHANGELOG — SPEC-056.
- Autofix / commit-back mode — a v2 (fork-safety + write permissions); README-noted only.
- Tagging the Actions `v1` / GitHub Marketplace listing — the maintainer's outward step.

## Notes for the Implementer

- Pin the installer by release-tag URL (`…/releases/download/<tag>/crustyimg-installer.sh` or
  `…/releases/latest/download/…`). Set `CRUSTYIMG_INSTALL_DIR` to a known dir + `CRUSTYIMG_NO_MODIFY_
  PATH=1`, then own the `echo "<dir>/bin" >> $GITHUB_PATH` step (uniform across install + cache-hit).
- Resolve `latest` → a concrete tag (GitHub API) so the cache key is stable and both installers pin
  the same version. Use `bash` for the resolve/PATH steps (present on all 3 runners); only the
  installer invocation branches sh vs pwsh.
- The wrapper's annotation parse uses `jq` (preinstalled on GitHub runners) over the JSON `findings[]`;
  lint findings are file-level (no line) → emit `::<level> file=…::` (line defaults). Map
  `error→error`, `warn→warning`, `info→notice`.
- Keep each Action's self-test cheap: generate the fixture in the workflow (a committed tiny asset or
  an inline byte write), run on `ubuntu/macos/windows-latest`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **Produced Action repos (+ self-test run URLs):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?** — <answer>
3. **If you did this task again, what would you do differently?** — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?** — <answer>
2. **Does any template, constraint, or decision need updating?** — <answer>
3. **Is there a follow-up spec I should write now before I forget?** — <answer>
