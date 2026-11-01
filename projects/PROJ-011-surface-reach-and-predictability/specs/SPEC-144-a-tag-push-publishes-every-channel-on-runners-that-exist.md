---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-144
  type: chore                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-054
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: claude-opus-5-5     # usually same Claude, different session
  created_at: 2026-10-31

references:
  decisions:
    - DEC-040   # cargo-dist pinned to 0.32.0; owns dist-workspace.toml + release.yml
    - DEC-041   # crates.io as a separate tag workflow (point 2 to be replaced)
    - DEC-067   # npm published only by hand (Publishing section to be replaced)
    - DEC-066   # the size-profiled wasm build the npm publish must route through
    - DEC-052   # release artifacts build default features only (no heic)
    # DEC-111 is RESERVED for design (one tag push; registries publish after host)
  constraints:
    - no-secrets-in-code
    - one-spec-per-pr
    - cost-captured-per-cycle
  related_specs:
    - SPEC-042  # Homebrew + crates.io channels
    - SPEC-075  # the npm package
    - SPEC-076  # the gated hand publish

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-054's <capability>". Optional; null is acceptable.
value_link: "infrastructure: keeps the 0.8.x line STAGE-054 carries releasable; the next tag publishes every channel from CI, after the binaries exist"

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
    - cycle: frame
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 5399925
      tokens_breakdown:
        input: 90
        output: 28573
        cache_creation: 129976
        cache_read: 5241286
      estimated_usd: 4.15
      duration_minutes: null
      recorded_at: 2026-10-31
      rate_anchors: "claude-opus-5-5: input $5/MTok, output $25/MTok, cache_creation x1.25 ($6.25), cache_read x0.10 ($0.50)"
      note: >
        Interactive frame session in its own worktree (docs/spec-144-frame). Measured from the
        session's transcript (8805a15c-….jsonl, identified by content), deduped by `.message.id`
        (44 unique ids, all claude-opus-5-5), input/cache_creation/cache_read once per id, MAX
        output_tokens per id. Measured before the commit/push turns, so it slightly under-counts.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-144: a tag push publishes every channel, on runners that exist

## Context

0.8.2's release (tag pushed 2026-10-31 23:15 UTC) showed three problems in the release
pipeline. None of them is in crustyimg's code. Parent: STAGE-054, which carries the 0.8.x line.
PROJ-011.

1. **The Apple Silicon build was cancelled twice.** Release run `41251779509`, attempts 1 and 2:
   `build-local-artifacts (aarch64-apple-darwin)` ran on `macos-14` and was cancelled. Both
   annotations read "This job failed because of a scheduled macOS 14 runner image brownout. macOS
   14 runner images will be retired on November 2, 2026" (actions/runner-images#13518). Attempt 3,
   outside the brownout window, passed. After 2026-11-02 every tag fails this job.
2. **npm is published by hand.** No workflow publishes `crustyimg-wasm`. The maintainer runs
   `just wasm-npm-publish` from a tag worktree after `npm login`. HANDOFF records a 0.8.0 publish
   that hit a 404 on PUT because the saved token could read but not publish.
3. **The release went out partially, for about two hours.** crates.io was published at 23:18:29
   (`publish-crates.yml`, run `41251779599`). npm `crustyimg-wasm@0.8.2` was published by hand at
   23:51:13. The GitHub Release was published at 01:27:22 and the Homebrew formula at 01:28:24.
   If the build had been broken rather than browned out, crates.io and npm would hold a 0.8.2 that
   never got binaries, and a registry version cannot be published twice.

## Goal

One `v*` tag push publishes crates.io, npm, the GitHub Release and Homebrew from CI, on runners
GitHub still supports. No registry publishes unless all four binaries built and the GitHub
Release exists.

## Frame

**Verdict: GO.** Sized **low [M]**: the change is small, but proving it is harder than making it
(see Sizing). One spec, no split.

### Premise check (evidence, not the prompt)
- **Runner:** `gh api …/runs/41251779509/attempts/{1,2,3}/jobs`. The labels came from `plan`'s
  matrix: aarch64 → `macos-14` (cancelled ×2, passed on attempt 3), x86_64-apple → `macos-15-intel`,
  linux → `ubuntu-22.04`, windows → `windows-2022`. `plan`, `host`, `build-global-artifacts`,
  `publish-homebrew-formula` and `announce` all run on `ubuntu-22.04`. The annotation text is quoted
  in Context. **Confirmed.**
- **Hand publish:** no workflow in `.github/workflows/` runs `npm publish`. `npm view crustyimg-wasm
  time` shows 0.8.2 at 23:51:13Z, 1 h 36 min before the Release existed. **Confirmed.**
- **Partial release:** the timestamps in Context come from `gh run list --workflow publish-crates.yml`,
  `gh release view v0.8.2`, `npm view` and the `host`/`publish-homebrew-formula` job completion
  times. The gap was 2 h 09 min. **Confirmed.**

### Q1 — the runner fix
- **Smallest change:** three lines in `dist-workspace.toml`:
  ```toml
  [dist.github-custom-runners]
  aarch64-apple-darwin = "macos-15"
  ```
  Probed in a scratch copy with the same `dist 0.32.0`. `dist generate --mode=ci` leaves
  `release.yml` **byte-identical**: the build matrix is computed at run time by the `plan` job,
  not written into the workflow. `dist plan` then gives aarch64 → `macos-15`, and the other three
  are unchanged. A no-op regenerate on current `main` is also clean, so `release.yml` is in sync
  today.
- **Other retiring images:** **`ubuntu-22.04` retires 2027-04-17** (actions/runner-images#14254,
  with brownouts in March 2027). It runs `plan`, the Linux build, `host`, `build-global-artifacts`,
  Homebrew and `announce`, so the same failure would come back next spring. `macos-15-intel` and
  `windows-2022` have no announced retirement (no open deprecation issue; the runner-images README
  marks only macOS 14 deprecated). Moving the **Linux build** is not just a label change: it raises
  the glibc floor of the released Linux binary (22.04 → 24.04). It is **filed as its own backlog
  item**, out of this spec.
- **Upgrading `dist` instead does not fix it.** `cargo-dist/src/backend/ci/github.rs` still maps
  aarch64-apple-darwin to `"macos-14"` at `v0.33.1` (latest, 2026-10-21) and on `main`. The 0.33.0
  notes add Azure signing and move the shell installer's `env` file, which changes users' install
  layout. 0.33.1 updates dependencies only. Upgrading buys nothing here and changes installer
  behaviour, so stay on 0.32.0 (DEC-040's pin).

### Q2 — proving it without publishing
- `release.yml` already runs on `pull_request`. With the default `pr-run-mode = "plan"` its build
  jobs skip on PRs. With `"upload"` (cargo-dist 0.32 config reference) they build every target and
  upload the artifacts to the PR run. They never host or publish: `publishing` is
  `!github.event.pull_request`. `publish-crates.yml` cannot fire because it triggers on tags only.
- **Recommendation:** the build PR sets `pr-run-mode = "upload"` **temporarily**. The probe shows
  `release.yml` then gains one `swatinem/rust-cache` step. That PR's own Release run must build all
  four targets green, with aarch64 on `macos-15`. A final commit reverts to `"plan"` before merge,
  so PRs do not pay four builds each, macOS included. Verify reads that run's job labels and
  conclusions, and diffs the final `release.yml` against the one that ran.
- `dispatch-releases = true` would add `workflow_dispatch` with a "dry-run" tag, but it **disables
  tag-push releases**, so it is rejected. A pre-release tag is rejected too: today
  `publish-crates.yml`'s glob `**[0-9]+.[0-9]+.[0-9]+*` matches `v0.9.0-rc.1` and would publish
  to crates.io.
- **What no PR can prove:** the OIDC exchange and the real `npm publish`/`cargo publish`. The
  publish jobs' steps up to the publish itself (build, smoke, version and tag guards, `npm publish
  --dry-run`, `cargo publish --dry-run`) can run on the PR. The first real tag proves the rest,
  using the maintainer checklist in `RELEASING.md` with the hand fallback kept.

### Q3 — npm trusted publishing: the facts
All from <https://docs.npmjs.com/trusted-publishers>, checked against the raw page:
- **Versions:** "npm CLI version 11.5.1 or later and Node version 22.14.0 or higher". The
  `ubuntu-22.04` and `ubuntu-24.04` images ship **Node 22.23.4 / npm 10.9.9** (runner-images
  `Ubuntu2204-Readme.md`), **too old**. Node 24's bundled npm is 11.19.1 (`nodejs/node` `v24.x`
  `deps/npm/package.json`), so the job uses `actions/setup-node` with Node 24, or upgrades npm.
- **Permission:** "the `id-token: write` permission". Through `workflow_call` it must be given "to
  both parent and child workflows". cargo-dist already gives custom publish jobs `{ id-token =
  "write", packages = "write" }` by default (config reference, `github-custom-job-permissions`).
- **`package.json`:** `repository.url` "must exactly match your GitHub repository". **It is written
  today:** `npm/package.overrides.json` sets `git+https://github.com/jysf/crustyimg.git`, and
  `scripts/wasm-npm-finalize.mjs` merges it in. Provenance is automatic ("npm automatically
  generates and publishes provenance attestations").
- **One-time setting on npmjs.com:** package → Settings → Trusted publisher → GitHub Actions, with
  organization or user `jysf`, repository `crustyimg`, and a workflow filename ("Enter only the
  filename, not the full path"). The environment is optional. Runners must be GitHub-hosted:
  "Self-hosted runners are not currently supported".
- **Turning tokens off:** Settings → Publishing access → "Require two-factor authentication and
  disallow tokens". npm "strongly recommend[s]" it. **Recommend: yes, but only after the first CI
  publish succeeds**, so the hand fallback still works if the first OIDC run fails. (A package may
  have "up to 10 trusted publishers".)

### Q4 — where the publish runs, and when
- **Option A (own `v*` workflow, like `publish-crates.yml`):** it keeps today's partial-release
  hazard. npm publishes whether or not any binary built. A broken build leaves npm (and crates.io)
  with a version that can never be republished.
- **Option B (cargo-dist custom publish job), recommended, and the same for crates.io.** Probed:
  `publish-jobs = ["homebrew", "./publish-npm"]` makes `dist generate` add `custom-publish-npm`
  with `needs: [plan, host]`. It is skipped for prereleases unless `publish-prereleases`, has
  `secrets: inherit` and `id-token: write`, and gates `announce`. Move crates.io the same way
  (`./publish-crates`) and delete `publish-crates.yml`. DEC-041 recorded that `dist` has no
  built-in crates.io job, and a custom job needs no built-in. This also stops a pre-release tag
  from publishing to crates.io.
- **Failure halfway, Option B:** a build fails → `host` never runs → **nothing** publishes. Re-run
  the failed jobs and the rest follows. `host` passes but one publish fails → the Release, Homebrew
  and the other registry are out, one registry is missing, and `announce` skips. "Re-run failed
  jobs" re-runs just that publish. The remaining gap is minutes, and a re-run fixes it.
  **Strictly "every channel or none" is not achievable**: four registries cannot commit
  atomically. The guarantee is "no registry before every binary and the Release". Design must make
  each publish job idempotent: it skips cleanly when the version is already on the registry
  (`npm view crustyimg-wasm@X version`, the crates.io API). Otherwise a re-run after a late-step
  failure fails with "cannot publish over".
- **Failure halfway, Option A:** a build fails → crates.io and npm are already out, indefinitely,
  and the fix is a new patch version.
- **Does B satisfy the trusted-publisher setting?** Yes, if the setting names **`release.yml`**,
  not `publish-npm.yml`. npm: "validation checks the calling workflow's name instead of the
  workflow that actually contains the publish command". Getting this wrong is the most likely
  first-publish failure, so `RELEASING.md` must say it plainly.

### Q5 — the publish path stays guarded
- `just wasm-npm-publish` can run **unchanged** in CI. It chains `wasm-npm-smoke` → `wasm-npm-pkg`
  → `wasm-build` (the size profile, DEC-066). It refuses `pkg/` ≠ `Cargo.toml`, and refuses a
  dirty tree with `git diff --quiet`, which looks only at tracked files. `/pkg/` is gitignored, and
  a tagged commit's `Cargo.lock` is in sync, so a CI checkout stays clean. `pages.yml` already
  installs the same toolchain (`dtolnay/rust-toolchain` + wasm32, `taiki-e/install-action` for
  `just` and `wasm-pack`) and runs `just demo-build`. The `npm` on PATH must be ≥ 11.5.1 (Q3).
- **Add a tag check:** yes. A workflow step before the recipe asserts `${GITHUB_REF_NAME#v}` ==
  `Cargo.toml` version. (The recipe already ties `pkg/` to `Cargo.toml`, which closes the chain.)
  The crates job gets the same step. Design decides whether the step lives in the workflow or in
  an opt-in env check in the recipe. The workflow is the smaller change.

### Q6 — DEC-067, or DEC-111
**Recommend new DEC-111.** It replaces DEC-067's *Publishing* section and DEC-041's point 2
(crates.io's separate workflow), and extends DEC-040 (the runner override). DEC-067 keeps its
name, target and lockstep-version rulings. Amending DEC-067 alone would leave crates.io's ordering
in DEC-041 and the runner in DEC-040, three DECs edited for one rule. DEC-067 is also already out
of date: it rejected "a `just wasm-npm-publish` recipe", and that recipe exists. Draft rule:

> **Pushing a `v*` tag is the decision to publish, and the only one.** The release workflow
> publishes crates.io and npm (by OIDC trusted publishing where the registry supports it) only
> after every target has built and the GitHub Release exists. No agent, and no recipe run outside
> that tag-triggered workflow, runs `npm publish` or `cargo publish`; the maintainer's hand
> fallback (`just wasm-npm-publish`, `cargo publish --locked`) is a documented recovery step, not
> a channel.

### Q7 — docs: each sentence that changes
- `RELEASING.md` "Git tags" ("Pushing a version tag ALSO (SPEC-042):" and its two bullets): add
  npm, and say the publishes wait for the Release.
- `RELEASING.md` "One-time setup": add the npm trusted-publisher step (workflow filename
  `release.yml`), and later "disallow tokens". `CARGO_REGISTRY_TOKEN` stays.
- `RELEASING.md` step 7, "This single push fires all three channels (SPEC-042): …": becomes four
  channels, in order.
- `RELEASING.md` step 8, "Verify the channels …" and the crates.io manual-fallback sentence: add
  `npm view crustyimg-wasm version`, and the npm hand fallback.
- `HANDOFF.md` (PROJ-011) "How the release went": "`npm login`, then re-run. Publish npm from a
  detached worktree at the tag …" becomes recovery-only.
- `justfile` comment above `wasm-npm-publish`: "it stays SPEC-076's maintainer-gated decision.
  That gate is 'a human decides to publish'" becomes "the tag push decides; this is the recovery
  path".
- `scripts/wasm-npm-finalize.mjs` header: "`npm publish` is gated on maintainer approval
  (SPEC-076)", and `tests/npm_smoke.mjs:17`: "No npm publish — that is SPEC-076, and gated". Point
  both at DEC-111.
- `decisions/DEC-067…` and `DEC-041…`: `superseded_by`/a note pointing at DEC-111 for the parts it
  replaces.
- Left alone: `docs/roadmap.md:239` and `docs/research/proj-008-wasm-build.md:471` are historical
  records.

### Q8 — size and scope
- **Size: low [M].** The diff is about 6 files plus docs. But verification spans a live PR run
  (four builds), dry-run publish steps, a structural check of the generated `release.yml`, and a
  first real publish that only the next tag proves. The two publish jobs also need idempotent
  re-runs. That is more than [S].
- **The wasm-tests-in-CI entry: leave it separate.** It is a PR-time gate on code paths, needs
  `wasm-bindgen-cli` pinned, and its own backlog text says "Measure its runtime before making it
  required". This spec runs on tags and never touches `src/` tests. They share only the toolchain
  setup, which design can copy from `pages.yml`. Folding it in would add a required-check decision
  to release infrastructure that has to land before the next tag.

### Scope
**In:** the `macos-15` runner override; npm publish as a cargo-dist custom publish job by OIDC;
crates.io moved into a custom publish job and `publish-crates.yml` deleted; tag == crate version
guards; idempotent re-runs; a one-time PR proof of all four builds; DEC-111; the doc sentences in
Q7.
**Out:** see "Out of scope" below.

### Rough build file list
- `dist-workspace.toml`: the custom runner; `publish-jobs` += `./publish-crates`, `./publish-npm`;
  `pr-run-mode` toggled during the PR only.
- `.github/workflows/release.yml`: regenerated by `dist generate` (two `custom-publish-*` jobs,
  `announce` needs).
- `.github/workflows/publish-npm.yml` (new, `workflow_call`).
- `.github/workflows/publish-crates.yml`: rewritten as a `workflow_call` job (or deleted, with a
  new file name; design picks).
- `justfile` (comments; perhaps an env-gated tag check), `scripts/wasm-npm-finalize.mjs`,
  `tests/npm_smoke.mjs` (comments only).
- `RELEASING.md`, `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`.
- `decisions/DEC-111-*.md` (new), `decisions/DEC-067-*.md`, `decisions/DEC-041-*.md` (supersede
  notes).

### Questions only the maintainer can answer
1. Move crates.io into the dist publish stage too, so nothing publishes before the binaries?
   **Recommend yes.**
2. After the first green CI publish, set npm "Require two-factor authentication and disallow
   tokens"? **Recommend yes, after, not before.**
3. A GitHub Environment (for example `release`) with a required reviewer on the publish jobs?
   **Recommend no.** The tag push is the deliberate act, and a second click brings back the step
   this spec removes.
4. crates.io also offers trusted publishing (OIDC), which would drop `CARGO_REGISTRY_TOKEN`. Do it
   here? **Recommend no.** Keep the token this time, and change one credential path per release.

## Inputs

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth>
- **Related code paths:** `src/some/module/`

## Outputs

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New exports:** <names and signatures>
- **Database changes:** <migrations>

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

### Out of scope (for this spec specifically)

- **Moving off `ubuntu-22.04`** (retires 2027-04-17): it changes the Linux binary's glibc floor.
  Filed as its own STAGE-054 backlog item.
- **Upgrading cargo-dist** past 0.32.0: it does not fix the runner, and 0.33.0 changes the shell
  installer's layout.
- **crates.io trusted publishing** (`rust-lang/crates-io-auth-action`): `CARGO_REGISTRY_TOKEN`
  stays. A maintainer question; no backlog item unless the maintainer wants one.
- **The wasm round-trip tests in CI**: stays its own STAGE-054 entry (Frame Q8).
- **A GitHub Environment or a required reviewer** on the publish jobs (maintainer question 3).
- **Changes to the npm package's name, target, version line or contents** (DEC-067 stands).
- **`pr-run-mode = "upload"` as a permanent setting**: it is used once, on the build PR.

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Not "src/foo and tests": the enumeration is what a verify cycle diffs
  the real change against, and what tells you whether every affected `DEC-*` has
  the path in its `affected_scope`. SPEC-122's Deviations claimed "`src/operation`
  and `tests/` only" and was wrong by two `scripts/` files, which left
  `affected_scope` blind; SPEC-124's build added this section by hand after that
  lesson and still listed 8 of its 9 files. It is a checklist because it is easy
  to get wrong from memory — build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

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
