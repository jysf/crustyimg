---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-051                        # stable, never reused
  type: decision                     # decision | analysis | recommendation | observation
  confidence: 0.82                   # 0.0 - 1.0, honest assessment
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-004                       # the project during which this was decided
repo:
  id: crustyimg

created_at: 2026-07-06
supersedes: null
superseded_by: null

# Path globs this decision governs.
affected_scope:
  - .pre-commit-hooks.yaml
  - justfile
  - README.md

tags:
  - github-actions
  - adoption
  - packaging
  - ci
  - cargo-dist
  - pre-commit
---

# DEC-051: GitHub Actions adoption on-ramp — wrap the cargo-dist installer, two composite-action repos

## Decision

crustyimg's CI on-ramp ships as **two small composite GitHub Actions in their own public repos**
plus **in-repo adoption glue**, not as code inside the crustyimg crate:

1. **`jysf/setup-crustyimg`** — the generic installer Action. A composite `action.yml` that, given a
   `version` input (default `latest`), invokes crustyimg's **existing cargo-dist installer**
   (`crustyimg-installer.sh` on Unix, `.ps1` on Windows) pinned to the release tag, into a known
   install dir (`CRUSTYIMG_INSTALL_DIR` + `CRUSTYIMG_NO_MODIFY_PATH=1`), adds it to `$GITHUB_PATH`,
   and caches it (`actions/cache`, keyed on os/arch/resolved-tag). It installs the *binary*, so it
   enables `optimize`/`convert`/`lint` — every crustyimg command — in anyone's CI, not only lint.
2. **`jysf/crustyimg-action`** — the lint/optimize wrapper. `uses: jysf/setup-crustyimg`, then in
   `lint` mode runs `crustyimg lint <paths> --format json`, parses the `crustyimg.lint/v1` report,
   and emits native `::error`/`::warning`/`::notice` annotations + a job-summary table, propagating
   the pass/fail exit per a `fail-level` input. An `optimize` mode runs `optimize` over a tree.
3. **In-repo glue (crustyimg repo, STAGE-015):** a `.pre-commit-hooks.yaml` (the format-aware
   upgrade from `check-added-large-files`), a `just lint-images` recipe, and a **Continuous
   integration** docs section showing the Actions + the plain binary.

Two load-bearing choices:

- **Wrap the cargo-dist installer; never hand-roll a download matrix.** cargo-dist already publishes
  a per-release `.sh`/`.ps1` installer that does OS/arch detection **and verifies sha256 checksums**
  (`verify_checksum`), and honors `CRUSTYIMG_INSTALL_DIR`/`CRUSTYIMG_NO_MODIFY_PATH`. Version pinning
  is just the release-tag URL (`…/releases/download/<tag>/…` or `…/releases/latest/download/…`), so
  asset names/URLs are never hardcoded — they keep working across versions.
- **Separate repos, because a GitHub Action must live at a repo root to be `uses:`-able.** The
  Actions are packaging that *wraps* the shipped binary + exit code + JSON report; they add **no new
  dependency to the crustyimg crate**. The crate stays the single source of the behavior; the Actions
  are thin.

The commit-back / autofix mode is **deferred to a v2** (fork-safety + write permissions), noted as
future work in the wrapper's README.

## Context

STAGE-013 shipped `crustyimg lint` (a source-file, format-aware, exit-coded CI linter with a stable
hand-rolled JSON report). The highest-leverage distribution move is making it a *one-line CI win*.
The framing originally deferred the Actions to "separate repos, out of scope"; the maintainer is
pulling them into the 0.4.0 wave so the release notes can point at them. The enabler already exists:
crustyimg publishes cross-platform release binaries via cargo-dist (v0.3.1 live, 0.4.0 to come),
including the self-contained installer scripts.

Constraints in play: `no-new-top-level-deps-without-decision` (the Actions are packaging, not crate
code — zero crate deps added); the outward-publish boundary (creating repos + a green self-test is
in scope; **tagging the Actions `v1` and Marketplace listing is the maintainer's step**, mirroring
the crate release-tag boundary in RELEASING.md).

## Alternatives Considered

- **Option A: a single Action that both installs and lints.**
  - Why rejected: conflates the generic installer (useful for *any* crustyimg command in CI) with
    the lint-specific PR-annotation wrapper. Splitting them lets `setup-crustyimg` stand alone
    (optimize/convert/lint) and keeps the wrapper thin (`uses:` the setup).

- **Option B: hand-roll a download matrix in the Action** (map runner os/arch → asset name, curl the
  tarball, checksum, extract).
  - Why rejected: duplicates exactly what cargo-dist's installer already does (os/arch detection +
    checksum verification), hardcodes asset names that change across versions, and drifts from the
    installer the docs already recommend. Wrapping the installer is less code and self-maintaining.

- **Option C (chosen): wrap the cargo-dist installer in `setup-crustyimg`; a thin
  `crustyimg-action` wrapper on top; in-repo pre-commit/recipe/docs glue.**
  - Why selected: minimal, version-agnostic, reuses the shipped checksum-verifying installer, adds no
    crate dependency, and gives both a generic on-ramp and a lint-native PR experience.

## Consequences

- **Positive:** "drop image linting into any CI in three lines"; the installer's checksum + os/arch
  logic is reused, so the Action is small and survives version bumps; `setup-crustyimg` is reusable
  for the whole CLI; the crate gains no dependency and no CI coupling.
- **Negative:** three repos to keep in loose sync (a rule-id or JSON-schema change in crustyimg could
  need a wrapper bump — mitigated: the wrapper only reads the stable `crustyimg.lint/v1` schema, a
  DEC-050 stability surface). The Actions can't run this repo's CI, so they carry their own 3-OS
  self-test workflows.
- **Neutral:** until the maintainer tags `v1`, the wrapper references `setup-crustyimg@main` and docs
  use `@main`; each README documents the exact tag/Marketplace step that remains.

## Validation

- **Right if:** a consumer adds `uses: jysf/setup-crustyimg` + `crustyimg lint` (or the
  `crustyimg-action`) and it installs a checksum-verified binary on Linux/macOS/Windows, annotates a
  real GPS-leaking asset in a PR, and fails the job on error — with no per-version maintenance.
- **Revisit when:** the autofix/commit-back v2 is scoped; or if a future `lint` JSON-schema change
  forces a non-additive wrapper update; or if cargo-dist changes its installer env-var contract.

## References

- Related specs: SPEC-057 (this on-ramp), SPEC-056 (`lint --format sarif` + release-cut — sibling in
  STAGE-015), SPEC-050/051/052/053 (the `lint` command + `crustyimg.lint/v1` report the wrapper reads)
- Related decisions: DEC-050 (the lint contract / stable rule-id + JSON schema the wrapper depends
  on), DEC-040 (the cargo-dist release pipeline that produces the installer), DEC-041 (release
  channels — Homebrew/crates.io; the Actions are a third channel), DEC-025 (the exit-7 gate the
  wrapper propagates)
- External docs: cargo-dist installer (`crustyimg-installer.sh`/`.ps1` in each release);
  `docs/roadmap.md`; the crustyimg RELEASING notes (the maintainer's tag/publish boundary)
- Discussions: PROJ-004 STAGE-015 — maintainer pulled the Actions into the 0.4.0 wave 2026-07-06
