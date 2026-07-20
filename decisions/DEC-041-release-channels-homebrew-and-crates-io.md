---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-041
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-23
supersedes: null
superseded_by: null

affected_scope:
  - dist-workspace.toml
  - .github/workflows/release.yml
  - .github/workflows/publish-crates.yml

tags:
  - release
  - distribution
  - homebrew
  - crates-io
  - cargo-dist
---

# DEC-041: release channels — Homebrew tap (cargo-dist) + crates.io (separate tag workflow)

## Decision

Extend the SPEC-041 tag-triggered release pipeline (DEC-040) with two publish
channels, both firing **only on a pushed `v*` tag**:

1. **Homebrew — via cargo-dist (native).** Add `homebrew` to `installers`, set
   `tap = "jysf/homebrew-tap"` and `publish-jobs = ["homebrew"]` in
   `dist-workspace.toml`; `dist generate` emits a `publish-homebrew-formula` job in
   `release.yml` that pushes a generated formula to the tap repo using a
   **`HOMEBREW_TAP_TOKEN`** secret (a PAT with write access to the tap). The tap repo
   `jysf/homebrew-tap` must exist. Install UX: `brew install jysf/tap/crustyimg`
   (Homebrew maps repo `homebrew-tap` → tap `jysf/tap`), matching the README already.

2. **crates.io — via a SEPARATE minimal workflow.** cargo-dist **does not** publish to
   crates.io (probe-verified: `publish-jobs = [..., "crates-io"|"cargo"]` errors —
   "not a recognized job value"). So add a standalone
   `.github/workflows/publish-crates.yml` triggered **only** on `push: tags: ['v*']`
   that runs `cargo publish --locked` with a **`CARGO_REGISTRY_TOKEN`** secret. It has
   **no `pull_request` / branch trigger** — it can only ever run on a tag.

3. **#7 dual lean/full artifact — DEFERRED.** cargo-dist builds one variant per target
   from one config; a second `--no-default-features` (headless) artifact is not a native
   artifact and would need custom build steps or a second bin target. It is a fast-follow,
   not part of the first release (recorded in `docs/backlog.md`).

## Context

STAGE-007 backlog #4 (Homebrew tap) and #5 (`cargo publish`) are the remaining
distribution channels on top of the GitHub-Releases pipeline (SPEC-041/DEC-040). The
maintainer chose to wire both into the pipeline ahead of the `v0.1.0` cut ("full
launch"), with crates.io publishing **automated on the tag**.

A **design-time `dist` probe** (dist 0.32.0, config edits + `dist generate`, then
reverted) established the split:

- Homebrew **is** a cargo-dist `publish-jobs` value; the generated
  `publish-homebrew-formula` job targets `repository: "jysf/homebrew-tap"` and uses
  `secrets.HOMEBREW_TAP_TOKEN`.
- crates.io **is not** — `crates-io`/`cargo` are rejected as `publish-jobs` values in
  0.32.0. Hence the separate `cargo publish` workflow.

**Safety (unchanged from DEC-040's model):** merging the spec that added this config
**armed** the channels but fired nothing — both the cargo-dist homebrew job and the
crates.io workflow trigger only on a real `v*` tag push. The **[MAINTAINER-AUTHORIZED]**
one-time setup (creating the `jysf/homebrew-tap` repo, adding the two repo **secrets**
`HOMEBREW_TAP_TOKEN`/`CARGO_REGISTRY_TOKEN`) was completed before `v0.1.0`
(2026-07-04), and both channels have fired on every tag since — crustyimg is on
crates.io now (latest 0.4.0) and `brew install jysf/tap/crustyimg` installs it. A
crates.io publish is **irreversible** (a version can never be re-published);
`RELEASING.md`'s `cargo publish --dry-run` + gate suite remain the guards for each
new tag.

## Alternatives Considered

- **`release-plz` for crates.io** — manages versions + release PRs + publishes. Heavier
  than a `0.x` manual-cadence CLI needs; the one-line `cargo publish --locked` on tag is
  simpler and transparent. Revisit if version management becomes a burden.
- **Manual `cargo publish`** (maintainer runs it) — maximum control over the single most
  irreversible step, but not tag-reproducible. The maintainer chose the automated
  workflow; the dry-run + gates provide the safety a manual step would.
- **Hand-written Homebrew formula** — cargo-dist generates and updates it from the
  release artifacts (checksums, URLs) automatically; hand-maintaining it would rot.
- **Include #7 (lean artifact) now** — deferred: not a native cargo-dist artifact, adds
  custom build complexity to a first-ever release for low value.

## Consequences

- **Positive:** One `v0.1.0` tag → GitHub Release binaries (DEC-040) + a pushed Homebrew
  formula + a crates.io publish, all reproducible from the tag. `brew install
  jysf/tap/crustyimg` and `cargo install crustyimg` both light up.
- **Negative:** Two new secrets to manage and a tap repo to create (maintainer, one-time).
  crates.io publishing is irreversible per version. The crates.io step lives outside
  cargo-dist (a second small workflow) — a minor split of release logic, documented here.
- **Neutral:** No runtime code/dependency change; the shipped binary is unaffected.
  `#7` remains open in the backlog. cargo-dist and the crates.io workflow are release
  tooling, not runtime deps (cf. DEC-037/DEC-040) — `cargo deny` unaffected.
