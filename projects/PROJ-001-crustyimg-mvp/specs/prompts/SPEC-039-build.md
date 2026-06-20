# SPEC-039 build prompt — CHANGELOG + semver/release-tag conventions

Start a **fresh session**. You are the IMPLEMENTER for SPEC-039 in the `crustyimg`
repo (cwd is the repo root). This is a **pure-documentation chore** — write
`CHANGELOG.md` + `RELEASING.md` + a README pointer. **No code, no dependency, no
git tag, no `cargo publish` — documents only.** Open a PR and STOP. Follow this
prompt exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-039-changelog-semver-and-release-tag-conventions.md`
   — especially `## Policy (PINNED)`, `## Acceptance Criteria`, `## Notes`.
2. `docs/moat.md` — the feature arc (five built axes + trust) = the `0.1.0` narrative.
3. `docs/api-contract.md` — the command surface to list under `0.1.0 Added`.
4. `Cargo.toml` — confirm `version = "0.1.0"` (the changelog/tag must match).

## What to build
- **`CHANGELOG.md`** — Keep a Changelog format (https://keepachangelog.com):
  - A leading `# Changelog` + the standard preamble line.
  - `## [Unreleased]` (empty category headings or just a placeholder).
  - `## [0.1.0] - 2026-06-19` with a substantial **`### Added`** that summarizes the
    MVP **by capability for a human reader** (NOT a spec-by-spec dump): view/info;
    resize/thumbnail/shrink/convert/auto-orient; optimize/diff/responsive;
    watermark + the metadata lane (strip/clean --gps/set/copy-metadata, default
    drop-GPS); edit/--save-recipe + parallel apply --recipe; modern formats
    (WebP default, AVIF feature-gated); perceptual auto-quality (SSIMULACRA2) +
    byte budgets; and the STAGE-006 hardening (decode/recipe/resize limits,
    path/symlink guards, supply-chain CI). Optionally a `### Security` note for the
    hardening. Keep it tight and readable.
  - Link-reference definitions at the bottom:
    `[Unreleased]: https://github.com/jysf/crustyimg/compare/v0.1.0...HEAD` and
    `[0.1.0]: https://github.com/jysf/crustyimg/releases/tag/v0.1.0`.
- **`RELEASING.md`** — short + practical:
  - **Versioning:** SemVer; note that while `0.x`, minor bumps may carry breaking
    CLI changes and patch bumps are fixes only; `1.0.0` is the first stability
    commitment.
  - **Git tags:** annotated `vMAJOR.MINOR.PATCH` (e.g. `v0.1.0`); the tag version
    MUST equal `Cargo.toml`'s `version`; the release pipeline (a later item)
    triggers on the tag.
  - **Release-cut checklist:** bump `Cargo.toml` → move `[Unreleased]` → a dated
    version section in `CHANGELOG.md` → `cargo publish --dry-run` + full gates
    green → commit → annotated `vX.Y.Z` tag → push tag. **Mark the publish + tag +
    push-tag steps as MAINTAINER-AUTHORIZED (outward-facing).**
- **`README.md`** — add ONE short "Changelog & releases" section pointing to
  `CHANGELOG.md` and `RELEASING.md`. (Do NOT do the full install/usage rewrite —
  that is backlog #6.)

## Hard rules
- **Docs only.** Do NOT run `git tag`, `cargo publish`, or any release command. Do
  NOT touch `src/`, `Cargo.toml` deps, or any code. The `git diff` should touch only
  `CHANGELOG.md`, `RELEASING.md`, `README.md` (+ the spec docs you fill).
- The `0.1.0` version in `CHANGELOG.md` MUST match `Cargo.toml` (`0.1.0`), date
  `2026-06-19`.
- Sanity-check only: `cargo build` still succeeds (no code changed). No test/clippy
  changes expected.

## Gates (lightweight — docs only)
```
cargo build --quiet            # sanity: nothing broke (no code touched)
git tag                        # MUST show no new v0.1.0 tag (you create none)
git diff --stat                # only CHANGELOG.md / RELEASING.md / README.md (+ spec docs)
```

## Git / PR
- Branch `feat/spec-039-changelog-releasing` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked `reports/*.md`
  and `TESTING-WITH-YOUR-PHOTOS.md` (do NOT stage them).
- PR title: `docs(release): CHANGELOG + semver/tag conventions (SPEC-039)`.
- PR body per AGENTS.md §13 (Decisions referenced — none / Constraints — docs only /
  New decisions — none).
- Fill the spec's `## Build Completion` + 3 reflection answers; append the build cost
  session (numerics null; agent `claude-sonnet-4-6`).

## Cost
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-19
  notes: "docs: CHANGELOG.md (Keep a Changelog; 0.1.0 = MVP narrated from moat/api-contract) + RELEASING.md (SemVer 0.x + vX.Y.Z annotated-tag convention + release-cut checklist, publish/tag steps maintainer-authorized) + README pointer; no code/dep/DEC; no tag/publish"
```

## When done
`just advance-cycle SPEC-039 verify` (if it mis-globs or doesn't update the spec's
`cycle:` field, set `cycle: verify` in the spec frontmatter by hand), open the PR with
`gh`, and **stop** — the orchestrator pauses for the user before any merge.
