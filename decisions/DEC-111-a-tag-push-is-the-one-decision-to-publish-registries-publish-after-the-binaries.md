---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-111
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-11-02
supersedes: null        # supersedes IN PART: DEC-067 "Publishing", DEC-041 point 2 (see below)
superseded_by: null

affected_scope:
  - dist-workspace.toml
  - .github/workflows/release.yml
  - .github/workflows/publish-npm.yml
  - .github/workflows/publish-crates.yml
  - scripts/release-publish-guard.sh
  - scripts/check-release-pipeline.sh
  - RELEASING.md
  - justfile

tags:
  - release
  - ci
  - distribution
  - npm
  - crates-io
  - cargo-dist
---

# DEC-111: A `v*` tag push is the one decision to publish; the registries publish only after every binary and the GitHub Release exist

## Decision

**Pushing a `vMAJOR.MINOR.PATCH` tag is the decision to publish, and the only one.** The
cargo-dist release workflow (`release.yml`) publishes **crates.io** and **npm**
(`crustyimg-wasm`) as two cargo-dist *custom publish jobs* (`./publish-crates`,
`./publish-npm`). They run only after `host`, so only after every target has built and the
GitHub Release exists. npm authenticates by **OIDC trusted publishing**. crates.io keeps
`CARGO_REGISTRY_TOKEN` for now.

No agent, and no recipe run outside that tag-triggered workflow, runs `npm publish` or
`cargo publish`. The maintainer's hand path (`just wasm-npm-publish`, `cargo publish --locked`) is
a documented **recovery step**, not a channel.

The rule has four parts:

1. **Order.** `plan` → four `build-local-artifacts` + `build-global-artifacts` → `host` (creates
   the GitHub Release) → in parallel `publish-homebrew-formula`, `custom-publish-crates`,
   `custom-publish-npm` → `announce`. A failed build means `host` never runs, so **nothing**
   publishes.
2. **Guards before any publish** (`scripts/release-publish-guard.sh`): the tag is
   `vMAJOR.MINOR.PATCH` with no prerelease suffix, it equals `Cargo.toml`'s version, and the
   tracked tree is clean. Then it asks the registry whether that exact version already exists.
   Present → the job **skips with exit 0**. Absent → publish. **Could not ask** (no connection, any
   non-200 for the package document, an unreadable body) → **exit 3, no publish.** A registry
   outage is never read as "not published".
3. **Prereleases publish to no registry.** cargo-dist skips custom publish jobs and Homebrew for a
   prerelease tag (`publish-prereleases` stays unset). The guard refuses a prerelease as a second
   layer. A `v0.9.0-rc.1` tag yields a GitHub *prerelease* with binaries, and nothing else.
4. **Runners.** cargo-dist 0.32.0 maps `aarch64-apple-darwin` to `macos-14`, which GitHub retired
   on 2026-11-02. `dist-workspace.toml` overrides it to `macos-15` through
   `[dist.github-custom-runners]` (this extends DEC-040's config). `scripts/check-release-pipeline.sh`
   asserts the full runner map.

## Context

0.8.2 (2026-10-31) showed three faults in the pipeline, none of them in crustyimg's code
(SPEC-144 Context):

- The Apple Silicon build was cancelled twice by the `macos-14` brownout.
- npm was published by hand, 1 h 36 min before any binary existed.
- crates.io went out from its own tag-triggered workflow 2 h 09 min before the GitHub Release.

If the build had been broken instead of browned out, both registries would hold a 0.8.2 that never
got binaries. A registry version cannot be published twice.

DEC-041 put crates.io in a separate workflow because cargo-dist has no built-in crates.io job. A
**custom** publish job needs no built-in: `publish-jobs = ["./publish-crates"]` makes
`dist generate` emit `custom-publish-crates` with `needs: [plan, host]` (probed with dist 0.32.0,
SPEC-144 design).

DEC-067 made `npm publish` a hand-only act because a one-keystroke path was a hazard. The hazard it
named still holds: npm publishes are effectively irreversible. But the tag push is already a
deliberate, maintainer-only act that cuts a crates.io release (also irreversible). A second, later
hand step is what produced the partial release. The guard chain DEC-066/DEC-067 built
(`wasm-npm-publish` → smoke → pkg → the size-profiled build) runs **unchanged** inside the CI job.

## Alternatives Considered

- **Option A: a separate `v*`-triggered `publish-npm.yml`** (like the old `publish-crates.yml`).
  Rejected: it keeps the partial-release hazard. npm publishes whether or not any binary built.
- **Option B (chosen): cargo-dist custom publish jobs after `host`,** for both registries.
- **A GitHub Environment with a required reviewer.** Rejected (maintainer ruling 3, 2026-11-01): a
  second click brings back the step this decision removes.
- **crates.io trusted publishing now.** Deferred (ruling 4). Each release changes one credential
  path. Filed in STAGE-054.
- **`dispatch-releases = true`** (a manual dry-run trigger). Rejected: it disables tag-push
  releases.
- **Upgrading cargo-dist.** Rejected: 0.33.1 still maps aarch64 to `macos-14`, and 0.33.0 moves
  the shell installer's `env` file.

## Consequences

- **Positive:** one act, four channels. No registry can hold a version without binaries. A
  re-run of failed jobs is safe, because each publish job skips a version that is already out.
  npm needs no long-lived token.
- **Negative:** "every channel or none" is **not** achievable, because four registries cannot
  commit atomically. If `host` passes and one publish job fails, the Release, Homebrew and the
  other registry are out, `announce` skips, and "Re-run failed jobs" closes the gap. The OIDC
  exchange and the real publishes are proven only by the first real tag.
- **Neutral:** `release.yml` stays generated; edit `dist-workspace.toml` and run
  `dist generate --mode=ci`. The npm trusted-publisher setting names **`release.yml`** (the
  *calling* workflow), not `publish-npm.yml`.

## Supersedes in part

- **DEC-067, "Publishing" section**, which chose "the tooling stops at `npm pack`" and hand-only
  publishing. Its name, `--target web` and lockstep-version rulings stand.
- **DEC-041, point 2**: "crates.io via a SEPARATE minimal workflow". Point 1 (Homebrew) stands.
- **Extends DEC-040**: the `[dist.github-custom-runners]` override and the
  `[dist.github-custom-job-permissions]` table join its `dist-workspace.toml`.

## Validation

Right if: the first tag after SPEC-144 publishes crates.io and npm from CI after the Release
exists, with no hand step, and a re-run of an already-published job exits green with its SKIP
line. `scripts/check-release-pipeline.sh` and `scripts/release-publish-guard.sh --self-test`
stay green.

Revisit if: cargo-dist gains native crates.io/npm jobs; crates.io trusted publishing lands
(STAGE-054); or `ubuntu-22.04` retirement (2027-04-17) moves the pipeline's runners.

## References

- Specs: SPEC-144 (this), SPEC-042 (Homebrew + crates.io), SPEC-075/SPEC-076 (the npm package and
  its hand publish)
- Decisions: DEC-040, DEC-041, DEC-052 (release artifacts build default features), DEC-066 (the
  wasm size profile the npm publish routes through), DEC-067
- External: npm trusted publishers, <https://docs.npmjs.com/trusted-publishers>; cargo-dist 0.32
  config reference (`publish-jobs`, `github-custom-runners`, `github-custom-job-permissions`,
  `pr-run-mode`, `publish-prereleases`), <https://axodotdev.github.io/cargo-dist/book/reference/config.html>
