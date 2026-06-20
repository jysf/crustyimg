---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-039
  type: chore                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-007
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet (prescriptive prompt)
  created_at: 2026-06-19

references:
  decisions: []
  constraints: []
  related_specs: [SPEC-038]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-007's <capability>". Optional; null is acceptable.
value_link: >
  Second STAGE-007 step: a seeded CHANGELOG (v0.1.0 = the MVP) + a written
  semver/release-tag policy, so the (later) tag-triggered release pipeline has a
  changelog to publish and a documented versioning contract.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: "docs: CHANGELOG.md (Keep a Changelog; 0.1.0 = MVP narrated from moat/api-contract) + RELEASING.md (SemVer 0.x + vX.Y.Z annotated-tag convention + release-cut checklist, publish/tag steps maintainer-authorized) + README pointer; no code/dep/DEC; no tag/publish"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 1
---

# SPEC-039: changelog, semver, and release-tag conventions

## Context

**The second STAGE-007 step — pure documentation, safe (no outward-facing
action).** SPEC-038 made the crate publish-ready; before the (later)
tag-triggered release pipeline, the project needs two things written down:

1. A **CHANGELOG.md** (Keep a Changelog format) seeded with the **`0.1.0`** entry
   — the MVP summarized for a human reader (what `crustyimg` does), plus an
   `[Unreleased]` section for ongoing work.
2. A **versioning + release policy** (in `RELEASING.md` and a short README/`docs`
   pointer): semver rules for a `0.x` CLI, the **`vMAJOR.MINOR.PATCH` git-tag**
   convention (annotated tags; tag == the `Cargo.toml` version), and the
   release-cut checklist the future cargo-dist pipeline (backlog #3) will trigger
   from.

No code, no dependency, no publish, no DEC — markdown only. Parent: `STAGE-007`
(backlog item #2). Source material for the changelog: the shipped stages
(STAGE-001…006, 008, 009) and `specs/done/` + `docs/moat.md`.

## Goal

Add a `CHANGELOG.md` seeded with the `0.1.0` MVP entry (Keep a Changelog format)
and a `RELEASING.md` documenting the semver policy + `vX.Y.Z` git-tag convention +
the release-cut checklist — so the release pipeline has a changelog to ship and a
written versioning contract. Docs only; nothing is published or tagged.

## Inputs

- **Files to read:**
  - `docs/moat.md` — the feature arc (engine / surface / privacy / reproducibility
    / verification / trust) → the `0.1.0` changelog narrative.
  - `projects/PROJ-001-crustyimg-mvp/stages/STAGE-00*.md` (shipped stages) +
    `specs/done/` — the per-area feature list.
  - `docs/api-contract.md` — the command surface to list under `0.1.0` Added.
  - `Cargo.toml` — current `version = "0.1.0"` (the tag/changelog must match).
- **External APIs:** none. (Keep a Changelog https://keepachangelog.com ; SemVer
  https://semver.org — conventions, not deps.)
- **Related code paths:** repo root (`CHANGELOG.md`, `RELEASING.md`), `README.md`.

## Outputs

- **Files created:**
  - `CHANGELOG.md` — Keep a Changelog format. An `[Unreleased]` section (empty
    headings) + a `## [0.1.0] - 2026-06-19` entry with **`### Added`** summarizing
    the MVP by capability (view/info; resize/thumbnail/shrink/convert/auto-orient;
    optimize/diff/responsive; watermark + metadata lane; edit/--save-recipe +
    parallel apply; modern formats WebP/AVIF; perceptual auto-quality + byte
    budgets; the STAGE-006 hardening) — human-readable, not a spec dump. Include
    link-reference definitions for the version/compare URLs
    (`https://github.com/jysf/crustyimg/...`).
  - `RELEASING.md` — the release policy + checklist (see PINNED).
- **Files modified:**
  - `README.md` — a short "Changelog & releases" pointer to `CHANGELOG.md` /
    `RELEASING.md` (one small section; the full install/usage rewrite is backlog #6).
- **New exports / Database changes:** none.

## Policy (PINNED)

- **Versioning:** SemVer. While `0.x`, the CLI surface may change between minor
  versions; document that `0.x` minor bumps can carry breaking CLI changes and
  `0.x` patch bumps are fixes only. `1.0.0` is the first stability commitment.
- **Git tags:** annotated tags named **`vMAJOR.MINOR.PATCH`** (e.g. `v0.1.0`); the
  tag's version **must equal** `Cargo.toml`'s `version`. The tag is what the
  release pipeline (backlog #3) triggers on. Do NOT create any tag in this spec —
  document the convention only.
- **CHANGELOG:** Keep a Changelog format; `[Unreleased]` at top; newest version
  first; categories `Added/Changed/Deprecated/Removed/Fixed/Security`; each
  released version has a date and a compare link.
- **Release-cut checklist (`RELEASING.md`):** bump `Cargo.toml` version → move
  `[Unreleased]` → a dated version section in `CHANGELOG.md` → `cargo publish
  --dry-run` + full gate suite green → commit → annotated `vX.Y.Z` tag → push tag
  (which the future pipeline turns into artifacts). Mark the publish/tag steps as
  **maintainer-authorized** (outward-facing).
- **No outward-facing action in this spec:** no tag, no release, no publish — just
  the documents.

## Acceptance Criteria

- [ ] `CHANGELOG.md` exists, parses as Keep a Changelog (an `[Unreleased]` section
  + a `## [0.1.0] - 2026-06-19` section with a non-trivial `### Added` summarizing
  the MVP), with working version/compare link references to the GitHub repo.
- [ ] `RELEASING.md` exists and documents: SemVer (`0.x` caveat), the `vX.Y.Z`
  annotated-tag convention (tag == `Cargo.toml` version), and the release-cut
  checklist with the publish/tag steps marked maintainer-authorized.
- [ ] `README.md` has a short "Changelog & releases" pointer to both files.
- [ ] The `0.1.0` version in `CHANGELOG.md` matches `Cargo.toml`'s `version`.
- [ ] No git tag is created, nothing is published, no code/deps change (the gate
  suite is unaffected; a quick `cargo build` still succeeds).

## Failing Tests

A docs-only chore — no Rust tests. Verification is by inspection:

- `CHANGELOG.md` and `RELEASING.md` exist with the required sections; the
  `0.1.0` date is `2026-06-19` and the version matches `Cargo.toml`.
- `README.md` links to both.
- `git tag` shows **no** new tag; no `cargo publish` was run; `git diff` touches
  only `CHANGELOG.md`, `RELEASING.md`, `README.md` (+ the spec docs).
- `cargo build` still succeeds (sanity — no code touched).

## Implementation Context

*Read this section before starting the build cycle.*

### Decisions that apply

- (none new) — this codifies conventions; it introduces no DEC. The `0.1.0`
  version is already set in `Cargo.toml` (SPEC-001/038).

### Constraints that apply

- Docs only; the `clippy-fmt-clean` / test gates are unaffected (no code).

### Prior related work

- `SPEC-038` (shipped) — made the crate publish-ready; this adds the changelog +
  release policy the pipeline will use.
- `docs/moat.md` — the best single source for the `0.1.0` capability narrative.

### Out of scope (for this spec specifically)

- The **release CI pipeline** (cargo-dist, backlog #3), **Homebrew tap** (#4),
  **`cargo publish`** (#5), **README install/usage rewrite + completions** (#6),
  **dual artifacts** (#7) — separate items; the outward-facing ones need explicit
  user authorization.
- Creating an actual `v0.1.0` tag or GitHub Release — that happens at release-cut
  time under maintainer authorization, not here.

## Notes for the Implementer

- **Write the `0.1.0` changelog for a human**, grouped by capability, not a
  spec-by-spec dump. Pull the narrative from `docs/moat.md` (the five built axes +
  trust) and the command list from `docs/api-contract.md`. Keep it tight.
- Use real compare/version link references to `https://github.com/jysf/crustyimg`
  (e.g. `[0.1.0]: https://github.com/jysf/crustyimg/releases/tag/v0.1.0`,
  `[Unreleased]: https://github.com/jysf/crustyimg/compare/v0.1.0...HEAD`).
- **Do NOT** run `git tag`, `cargo publish`, or any release command — this spec
  produces documents only. Mark the publish/tag steps in `RELEASING.md` as
  maintainer-authorized (outward-facing).
- Keep `RELEASING.md` practical and short — a checklist a human follows to cut a
  release, that the cargo-dist pipeline (backlog #3) will later automate from the
  tag.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-039-changelog-releasing`
- **PR (if applicable):** opened — see PR URL in session notes
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - none
- **Follow-up work identified:**
  - none beyond the existing STAGE-007 backlog items

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing was genuinely unclear. The build prompt and spec were fully aligned; the
   source material (moat.md + api-contract.md) gave everything needed for a
   capability-grouped changelog without extra digging.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The "docs only, no tag, no publish" constraint was stated clearly and the
   gate commands made it easy to verify compliance.

3. **If you did this task again, what would you do differently?**
   — Nothing significant. Reading moat.md first gave a clean capability grouping that
   translated directly into the `### Added` structure; starting there rather than
   scanning specs/done/ saved time.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
