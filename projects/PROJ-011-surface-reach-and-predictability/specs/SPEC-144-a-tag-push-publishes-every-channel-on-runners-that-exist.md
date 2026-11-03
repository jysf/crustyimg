---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-144
  type: chore                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
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
    - DEC-111   # one tag push; registries publish after host (written at design)
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
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 4369768
      tokens_breakdown:
        input: 68
        output: 67372
        cache_creation: 163683
        cache_read: 4138645
      estimated_usd: 4.78
      duration_minutes: null
      recorded_at: 2026-11-02
      rate_anchors: "claude-opus-5-5: input $5/MTok, output $25/MTok, cache_creation x1.25 ($6.25), cache_read x0.10 ($0.50)"
      note: >
        Interactive design session in its own worktree (docs/spec-144-design). Measured from the
        session's transcript (90249c55-….jsonl, identified by content), deduped by `.message.id`
        (34 unique ids, all claude-opus-5-5), input/cache_creation/cache_read once per id, MAX
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


### Maintainer rulings (2026-11-01, settled)
1. **crates.io moves behind the binaries too.** It becomes a cargo-dist custom publish job after
   `host`, and `publish-crates.yml` as a tag-triggered workflow goes away.
2. **npm "disallow tokens" is turned on after the first green CI publish,** not before. Until
   then the hand fallback (`just wasm-npm-publish`) stays usable.
3. **No GitHub Environment or required reviewer.** Pushing the tag is the one deliberate act.
4. **crates.io keeps `CARGO_REGISTRY_TOKEN` in this spec.** Its trusted publishing is a later
   spec (filed in STAGE-054), so each release changes one credential path.

## Design

*Design cycle, 2026-11-02, claude-opus-5-5. The Frame above is the binding scope. This section
settles the calls the design prompt left open. Every probe below ran in a scratch worktree with
the repo's own `dist 0.32.0`, and the scratch copy was then discarded.*

### What the probes showed (read before the calls)

| # | Probe | Result | Consequence |
|---|---|---|---|
| P1 | `publish-jobs = ["homebrew", "./publish-crates", "./publish-npm"]` + `dist generate --mode=ci` | Adds `custom-publish-crates` and `custom-publish-npm` (`needs: [plan, host]`, prerelease `if:`, `uses: ./.github/workflows/publish-*.yml`, `with: plan`, `secrets: inherit`). `announce` gains both in `needs` and `if`. No other job changes. | Option B is what the frame said. |
| P2 | The same, default permissions | Each custom job gets `{ id-token: write, packages: write }` and **no `contents`**. A job-level `permissions` block sets every unlisted scope to `none`, so the called workflow could not request `contents: read` (a startup failure that would surface only on the tag). | **Add `[dist.github-custom-job-permissions]`** (Call 1). With it, both jobs get exactly `{ contents: read, id-token: write }`. The default `packages: write` is dropped, since nothing here pushes a package to GHCR. |
| P3 | `[dist.github-custom-runners] aarch64-apple-darwin = "macos-15"`, then `dist plan --output-format=json` | `aarch64-apple-darwin → macos-15`, `x86_64-apple-darwin → macos-15-intel`, `x86_64-pc-windows-msvc → windows-2022`, `x86_64-unknown-linux-gnu → ubuntu-22.04`. `release.yml` is byte-identical. | The runner lives in `plan`'s JSON, **not** in `release.yml`. A check of the runner must run `dist plan`. Grepping the YAML cannot see it. |
| P4 | Hand-edit `release.yml` (append a line), then `dist plan` and `dist generate --mode=ci --check` | Both exit **255**: "out of date … run 'dist init' to update the file". In sync, `--check` exits 0. | **CI already has a drift check.** `release.yml` runs on every `pull_request`, and its `plan` job runs `dist plan`, which fails on a stale workflow. `--check` is the non-mutating local form. |
| P5 | `npm view crustyimg-wasm@9.9.9 version` (absent) vs the registry at `127.0.0.1:9` (unreachable) | **Both exit 1.** | `npm view` **cannot** tell "absent" from "could not ask". The guard reads the package document instead (Call 4). |
| P6 | `curl https://registry.npmjs.org/crustyimg-wasm` | 200; `.versions` keys `0.5.0 0.7.0 0.8.0 0.8.1 0.8.2`. Per-version URL: 200 / 404. Missing package: 404. | Absent = **200 on the package document and the version key missing**. |
| P7 | crates.io: API `/api/v1/crates/crustyimg/0.8.2` with a User-Agent → 200; `9.9.9` → 404; **no User-Agent → 403**. Sparse index `https://index.crates.io/cr/us/crustyimg` → 200, one JSON line per version (`0.1.0` … `0.8.2`). | The per-version API has the same 404 ambiguity as npm. | The guard reads the **sparse index** (the file `cargo` itself resolves against; it lists yanked versions too, and a yanked version cannot be republished either). It still sends a User-Agent. |
| P8 | `npm_config_dry_run=true npm publish` in a scratch package, registry `127.0.0.1:9` | exit 0, "Publishing to … (dry-run)". Without it: exit 1 `ENEEDAUTH`. | The PR can dry-run **`just wasm-npm-publish` unchanged** by setting one env var. |
| P9 | npm 11 `publish --dry-run` of `0.0.1-rc.1` | exit 1: "You must specify a tag using --tag when publishing a prerelease version." | A prerelease could not reach npm `latest` anyway (Call 5). |
| P10 | `cargo publish --dry-run --locked` on this tree (clean checkout, scratch `CARGO_TARGET_DIR`) | exit 0. "Packaged 278 files, 4.7MiB (2.1MiB compressed)". "aborting upload due to dry run". 26 s on the Mac. | The crates dry run is cheap enough to run on PRs. |
| P11 | Ruby `YAML.load_file` on `release.yml` | The `on:` key loads as Ruby `true` (YAML 1.1 boolean). | The structural check reads `on` as `d["on"] \|\| d[true]`. |
| P12 | Action versions (`gh api …/releases/latest`, 2026-11-02) | `actions/checkout` v7.0.1; `actions/setup-node` v7.1.0; `taiki-e/install-action` v2.89.1. npm's trusted-publishing example uses `actions/setup-node@v6`, Node `'24'`, `registry-url`, and `package-manager-cache: false`. | Pins in Call 2. |

### Call 1 — `dist-workspace.toml`

Final state (the build PR's last commit). Keep the existing comment and keys. Change `publish-jobs`
and append two tables:

```toml
publish-jobs = ["homebrew", "./publish-crates", "./publish-npm"]
# … targets, install-path unchanged …

# cargo-dist 0.32.0 puts aarch64-apple-darwin on macos-14, which GitHub retired on
# 2026-11-02. The other three targets keep dist's defaults (DEC-111, extends DEC-040).
[dist.github-custom-runners]
aarch64-apple-darwin = "macos-15"

# The registry publish jobs read the repo (contents) and mint an OIDC token for npm
# trusted publishing (id-token). dist's default grants id-token + packages and no
# contents, which a called workflow cannot raise.
[dist.github-custom-job-permissions]
"publish-crates" = { contents = "read", id-token = "write" }
"publish-npm" = { contents = "read", id-token = "write" }
```

During the build PR only, a middle commit also sets `pr-run-mode = "upload"` under `[dist]`
(Call 7). The last commit removes that line. Every change ends with `dist generate --mode=ci`;
nobody hand-edits `release.yml`.

### Call 2 — the two publish workflows, line by line

**Name: keep `publish-crates.yml`**, reworked in place, and add `publish-npm.yml`. cargo-dist maps
`./publish-crates` to `.github/workflows/publish-crates.yml`. DEC-041's `affected_scope` already
names that path, and history stays on one file. Its `push: tags` trigger is **deleted**; that
deletion is what moves crates.io behind `host`.

**Shape: each file has two triggers and three jobs.**

```
on:
  workflow_call:            # from release.yml's custom-publish-* job (a tag push)
    inputs:
      plan: { required: true, type: string }
  pull_request:             # the PR dry run; paths filter below
    paths: [...]
permissions:
  contents: read
jobs:
  guard:    if: github.event_name == 'push' && github.ref_type == 'tag'
  publish:  needs: guard
            if: github.event_name == 'push' && github.ref_type == 'tag' && needs.guard.outputs.action == 'publish'
            permissions: { contents: read, id-token: write }
  dry-run:  if: github.event_name == 'pull_request'
            permissions: { contents: read }
```

- **What a reusable workflow sees.** Inside a `workflow_call`, the `github` context and the
  `GITHUB_*` variables are the **caller's**. On a tag push, `github.event_name == 'push'`,
  `github.ref_type == 'tag'` and `GITHUB_REF_NAME` is the tag (`v0.8.3`), so the guard reads the
  tag from `GITHUB_REF_NAME`. `inputs.plan` (dist's manifest JSON) is declared because dist passes
  it. The jobs do not need it; the guard decides from the tag and the tree.
- **Permissions.** The caller grants `{contents: read, id-token: write}` (Call 1). Only `publish`
  asks for `id-token`. `guard` and `dry-run` hold `contents: read` only, so **a PR run never holds
  an OIDC token.**
- **`secrets: inherit`** (generated) passes `CARGO_REGISTRY_TOKEN` into the called workflow. Only
  the `publish` job's publish *step* names it, in that step's `env`. The `dry-run` job references
  no secret.
- **What the maintainer sees** on the tag's Release run: `custom-publish-npm / guard` ✓, then
  `custom-publish-npm / publish` ✓ (or *skipped* when the guard printed SKIP), and the same for
  crates. `dry-run` does not appear (skipped). A called workflow whose jobs are success or skipped
  reports success, so `announce` runs.

**Pins** (exact): `actions/checkout@v6` (as `release.yml`; `persist-credentials: false`),
`dtolnay/rust-toolchain@stable` and `taiki-e/install-action@just` / `@wasm-pack` (as `pages.yml`),
and `actions/setup-node@v6` (npm's documented example; v7 exists, but v6 is the version the npm page
shows working). No `Swatinem/rust-cache` in these workflows: release jobs build clean, and a tag
run can restore `main`'s caches. That is a cache-poisoning surface worth nothing here.

#### `publish-crates.yml`

```yaml
name: publish-crates
# Called by release.yml (custom-publish-crates) after `host`, on a v* tag push: crates.io
# publishes only once every binary and the GitHub Release exist (DEC-111). On a PR it
# runs the release-pipeline checks and a dry run instead, and never publishes.
on:
  workflow_call:
    inputs:
      plan:
        required: true
        type: string
  pull_request:
    paths:
      - ".github/workflows/release.yml"
      - ".github/workflows/publish-crates.yml"
      - ".github/workflows/publish-npm.yml"
      - "dist-workspace.toml"
      - "Cargo.toml"
      - "Cargo.lock"
      - "scripts/release-publish-guard.sh"
      - "scripts/check-release-pipeline.sh"
permissions:
  contents: read
jobs:
  guard:
    if: github.event_name == 'push' && github.ref_type == 'tag'
    runs-on: ubuntu-latest
    outputs:
      action: ${{ steps.guard.outputs.action }}
    steps:
      - uses: actions/checkout@v6
        with: { persist-credentials: false }
      - id: guard
        run: scripts/release-publish-guard.sh crates "$GITHUB_REF_NAME"
      - name: The guard's verdict is publish or skip
        env: { ACTION: "${{ steps.guard.outputs.action }}" }
        run: |
          case "$ACTION" in
            publish|skip) echo "guard verdict: $ACTION" ;;
            *) echo "::error::release-publish-guard.sh wrote action='$ACTION', expected publish or skip"; exit 1 ;;
          esac
  publish:
    needs: guard
    if: github.event_name == 'push' && github.ref_type == 'tag' && needs.guard.outputs.action == 'publish'
    runs-on: ubuntu-latest
    permissions: { contents: read, id-token: write }
    steps:
      - uses: actions/checkout@v6
        with: { persist-credentials: false }
      - uses: dtolnay/rust-toolchain@stable
      - name: cargo publish
        run: cargo publish --locked
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
  dry-run:
    if: github.event_name == 'pull_request'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
        with: { persist-credentials: false }
      - uses: dtolnay/rust-toolchain@stable
      - name: Install dist 0.32.0
        shell: bash
        run: "curl --proto '=https' --tlsv1.2 -LsSf https://github.com/axodotdev/cargo-dist/releases/download/v0.32.0/cargo-dist-installer.sh | sh"
      - name: Release pipeline structure
        run: scripts/check-release-pipeline.sh
      - name: Publish guard self-test
        run: scripts/release-publish-guard.sh --self-test
      - name: cargo publish --dry-run
        run: cargo publish --dry-run --locked
```

`publish` holds `id-token` although crates.io still uses a token (ruling 4). That is deliberate:
crates.io trusted publishing (the STAGE-054 entry) then changes only that job's step. If the
builder prefers least privilege now, `contents: read` alone is acceptable, but the
`github-custom-job-permissions` entry for `publish-crates` must match it, and C4 checks them.

#### `publish-npm.yml`

```yaml
name: publish-npm
# Called by release.yml (custom-publish-npm) after `host`, on a v* tag push. Publishes
# crustyimg-wasm by npm trusted publishing (OIDC; the npmjs.com setting names release.yml,
# the CALLING workflow). Runs `just wasm-npm-publish` unchanged, so the size-profiled build,
# the smoke test and the pkg/-vs-Cargo.toml refusal all run first (DEC-066, DEC-111).
on:
  workflow_call:
    inputs:
      plan:
        required: true
        type: string
  pull_request:
    paths:
      - ".github/workflows/publish-npm.yml"
      - "Cargo.toml"
      - "Cargo.lock"
      - "justfile"
      - "npm/**"
      - "scripts/wasm-npm-finalize.mjs"
      - "scripts/release-publish-guard.sh"
      - "tests/npm_smoke.mjs"
permissions:
  contents: read
jobs:
  guard:      # identical to publish-crates.yml's guard job, with `npm` for `crates`
  publish:
    needs: guard
    if: github.event_name == 'push' && github.ref_type == 'tag' && needs.guard.outputs.action == 'publish'
    runs-on: ubuntu-latest
    permissions: { contents: read, id-token: write }
    steps:
      - uses: actions/checkout@v6
        with: { persist-credentials: false }
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: wasm32-unknown-unknown }
      - uses: taiki-e/install-action@just
      - uses: taiki-e/install-action@wasm-pack
      - uses: actions/setup-node@v6
        with:
          node-version: '24'
          registry-url: 'https://registry.npmjs.org'
          package-manager-cache: false
      - name: npm is new enough for trusted publishing (>= 11.5.1)
        run: |
          v=$(npm --version)
          if ! printf '%s\n' 11.5.1 "$v" | sort -V -C; then
            echo "::error::npm $v is older than 11.5.1; trusted publishing needs 11.5.1 or later"; exit 1
          fi
          echo "npm $v"
      - name: just wasm-npm-publish
        run: just wasm-npm-publish
  dry-run:
    if: github.event_name == 'pull_request'
    runs-on: ubuntu-latest
    steps:
      # checkout, rust-toolchain (wasm32), just, wasm-pack, setup-node: the same five steps as `publish`
      - name: npm publish --dry-run, through the unchanged recipe
        env: { npm_config_dry_run: "true" }
        run: |
          set +e
          just wasm-npm-publish > "$RUNNER_TEMP/npm.log" 2>&1
          rc=$?
          set -e
          cat "$RUNNER_TEMP/npm.log"
          [ "$rc" -eq 0 ] || exit "$rc"
          grep -q '(dry-run)' "$RUNNER_TEMP/npm.log" || { echo "::error::npm publish did not run as a dry run"; exit 1; }
```

**Can the PR dry run reach a real publish? No, behind three independent locks.** (1) `publish` has
`if: github.event_name == 'push' && …`, and a `pull_request` run has `event_name ==
'pull_request'`. (2) `dry-run` holds no `id-token` and no `NODE_AUTH_TOKEN`, so a non-dry
`npm publish` there fails `ENEEDAUTH` (P8). (3) npm's trusted publisher names `release.yml`, and a
PR-triggered run's workflow is `publish-npm.yml`. The crates `dry-run` job names no secret, so
`cargo publish` without `--dry-run` would fail for want of a token.

The `release.yml` side has no path to a publish either. On a PR, `host` is skipped
(`publishing == false`), so both `custom-publish-*` jobs are skipped. That holds in
`pr-run-mode = "upload"` too: upload builds artifacts, and `host` still requires `publishing`.

### Call 3 — the guards, and where each one lives

| Guard | Lives in | Refusal (stderr) | Exit |
|---|---|---|---|
| Tag shape `^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$` | `release-publish-guard.sh` | `REFUSING: tag '<t>' is not vMAJOR.MINOR.PATCH.` | 1 |
| Prerelease (tag has `-…`) | `release-publish-guard.sh` | `REFUSING: <t> is a prerelease; DEC-111 publishes stable versions only to <registry>.` | 1 |
| Tag version == `Cargo.toml` version | `release-publish-guard.sh` | `REFUSING: tag <t> says <x>, Cargo.toml says <y>.` | 1 |
| Tracked tree clean (`git diff --quiet && git diff --cached --quiet`, the recipe's own test) | `release-publish-guard.sh` | `REFUSING: the working tree has uncommitted changes; publish from a clean checkout of the tag.` | 1 |
| Already on the registry | `release-publish-guard.sh` | stdout `SKIP: <name>@<v> is already on <registry>; nothing to publish.` | **0**, `action=skip` |
| Could not ask the registry | `release-publish-guard.sh` | `COULD NOT ASK <registry>: <reason>. Not publishing; re-run this job when the registry answers.` | 3 |
| `pkg/` version == `Cargo.toml` version; tree clean after the build | `just wasm-npm-publish` (unchanged) | its existing `REFUSING:` lines | 1 |
| npm ≥ 11.5.1 | `publish-npm.yml` step | `::error::npm <v> is older than 11.5.1 …` | 1 |
| Bad usage (no args, unknown registry) | `release-publish-guard.sh` | `usage: release-publish-guard.sh <npm\|crates> <tag> \| --self-test` | 2 |

Exit codes follow the repo's script convention (`scripts/release.sh`: 1 = refused, 2 = usage),
plus **3 = could not ask the registry** (an I/O-class failure, as in DEC-015's CLI table). The order
is usage → shape → prerelease → version → tree → registry. A refusal costs no network call, and
the registry is asked last, so a SKIP is printed only for a tag that would otherwise have
published. On success the script prints `PUBLISH: <name>@<v> is not on <registry> yet.` and writes
`action=publish` (or `action=skip`) to `$GITHUB_OUTPUT` when that variable is set. It always prints
the same `action=` line to stdout too, so a local run shows the verdict.

**The tag-version guard is the second layer, not the first.** `dist plan` already refuses a tag
whose version matches no package (the v0.1.1 incident in `scripts/release.sh`'s header). On the
pipeline path the guard's tag check therefore fires only if dist's check regresses. It earns its
place on the **hand-fallback** path, where RELEASING.md runs the same script before
`just wasm-npm-publish` / `cargo publish --locked`.

### Call 4 — idempotency: "absent" vs "could not ask"

```
curl -sS --max-time 20 --retry 2 --retry-delay 2 -A "crustyimg-release-guard (+https://github.com/jysf/crustyimg)" \
     -o "$body" -w '%{http_code}' "$url"
```

- **npm:** `url = ${RELEASE_GUARD_NPM_URL:-https://registry.npmjs.org/crustyimg-wasm}`, the whole
  package document. **Present** = HTTP 200 and `jq -e --arg v "$V" '.versions | has($v)'` is true.
  **Absent** = HTTP 200 and that is false.
- **crates:** `url = ${RELEASE_GUARD_CRATES_URL:-https://index.crates.io/cr/us/crustyimg}`, the sparse
  index file. **Present** = HTTP 200 and `jq -s -e --arg v "$V" 'any(.[]; .vers == $v)'` is true.
  **Absent** = HTTP 200 and that is false.
- **Could not ask** is everything else: curl exits non-zero (no DNS, refused, timeout), any HTTP
  code other than 200 (**including 404**: both packages exist, so a 404 for the package document
  means a wrong URL or a registry fault, not "unpublished"), or a 200 body that `jq` cannot read
  (jq exit ≥ 2, or `.versions` missing). Each prints the reason (`curl exit 7: …`, `HTTP 503 from
  <url>`, `unreadable body from <url>`) and **exits 3**. Nothing publishes, and the job goes red.
  The fix is **Re-run failed jobs** once the registry answers.
- **Absent → publish**, exit 0. **Present → skip**, exit 0. A re-run after a late failure (for
  example npm published but the job died after) therefore goes **green with a SKIP line** instead
  of failing "cannot publish over".
- **Staleness is fail-safe.** If a CDN still serves a document from before a publish, the guard
  says "absent", the publish runs, and the registry itself refuses the duplicate. The job is red,
  never silently wrong. One more re-run then skips.
- `--retry 2` makes curl retry transient failures (timeouts, 5xx) before the verdict.
  `RELEASE_GUARD_RETRIES` (default 2) lets the self-test set 0.

### Call 5 — prereleases

**A `v0.9.0-rc.1` tag publishes to no registry.** `publish-prereleases` stays unset. cargo-dist then
skips `custom-publish-*` and `publish-homebrew-formula` for a prerelease (their generated `if:`).
`host` still builds everything and creates a GitHub **prerelease**. If someone later sets
`publish-prereleases`, the guard refuses (exit 1) before any registry call. npm would refuse anyway
without `--tag` (P9), and putting an rc on crates.io is a decision for a future DEC.

**Today**, with `publish-crates.yml` still tag-triggered, an rc tag *would* publish to crates.io:
its glob `**[0-9]+.[0-9]+.[0-9]+*` matches it. After this spec it does not.

### Call 6 — the generated file, and the check that proves it

- The build changes `release.yml` **only** by `dist generate --mode=ci` at 0.32.0.
- **Local proof:** `dist generate --mode=ci --check` exits 0. Equivalently, after
  `dist generate --mode=ci`, `git diff --exit-code .github/workflows/release.yml` is empty.
- **CI already enforces it (P4).** The Release workflow's `plan` job runs `dist plan` on every
  PR, and that exits 255 on a stale `release.yml`. `check-release-pipeline.sh` C1 runs `--check`
  too, so the failure carries the script's words and runs in the `publish-crates` dry-run job.
- **Not provable from `release.yml`: the runner (P3).** C2 reads it from `dist plan`.

### Call 7 — proving it before the next real tag

1. **The build PR's commits, in order:** (a) the config, workflows, scripts and docs, plus
   `pr-run-mode = "upload"`, regenerated; (b) **after the live run is recorded**, remove
   `pr-run-mode` and regenerate. Push each commit, and never squash (a) and (b) locally before the
   run exists. C2 fails on commit (a) by design (it asserts `pr_run_mode == "plan"`), so the
   `publish-crates / dry-run` check is **expected red on (a) for exactly that reason** and green
   on (b). The builder quotes the C2 line from (a)'s log.
2. **What the build records from (a)'s Release run** (`gh run list --workflow release.yml
   --branch <branch>`):
   - `gh api repos/jysf/crustyimg/actions/runs/<id>/jobs --jq '.jobs[] | [.name, .conclusion, (.labels|join(","))] | @tsv'`,
     quoted in full: the four `build-local-artifacts (<target>)` jobs **success** on
     `macos-15`, `macos-15-intel`, `ubuntu-22.04` and `windows-2022`; `build-global-artifacts`
     success; `host`, `publish-homebrew-formula`, both `custom-publish-*` and `announce`
     **skipped**.
   - `gh api repos/jysf/crustyimg/actions/runs/<id>/artifacts --jq '.artifacts[].name'`: the four
     `artifacts-build-local-<target>` entries, plus `artifacts-build-global`,
     `artifacts-plan-dist-manifest` and `cargo-dist-cache`.
   - `gh run download <id> -n artifacts-build-local-aarch64-apple-darwin`, unpack, and `file` the
     binary: **`Mach-O 64-bit executable arm64`**. A label alone says where it ran, not what it
     built.
3. **Wiring, without running a publish:** C3–C7 (the needs graph, `if:`s, permissions, triggers,
   guard order), plus `actionlint .github/workflows/release.yml .github/workflows/publish-*.yml`
   with **zero** findings. actionlint checks a local `uses: ./…` call's inputs and secrets against
   the callee's `workflow_call` block. The build records its version.
4. **Dry runs on the PR:** `publish-npm / dry-run` (log contains `(dry-run)` and `+ crustyimg-wasm@<v>`)
   and `publish-crates / dry-run` (log contains `aborting upload due to dry run`). The build quotes
   both lines.
5. **Diff the release.yml that ran against the final one:** `git diff <commit a> <commit b> --
   .github/workflows/release.yml` shows only the `pr-run-mode`-dependent lines (the frame's probe:
   one `Swatinem/rust-cache` step). The build quotes the diff.
6. **What only the first real tag proves:** the OIDC exchange, the real `npm publish`, and
   `cargo publish` from the new job. The maintainer checklist is in RELEASING.md (Call 9). It
   includes the hand fallback (guard script, then `just wasm-npm-publish` / `cargo publish --locked`)
   and keeps "disallow tokens" **off** until that tag publishes green (ruling 2).

### Call 8 — the DEC

**New DEC-111**, committed by this design cycle
(`decisions/DEC-111-a-tag-push-is-the-one-decision-to-publish-registries-publish-after-the-binaries.md`).
Amending DEC-067 alone would leave crates.io's order in DEC-041 and the runner in DEC-040. The
**build** adds these notes, because each one is true only once the pipeline exists. Each goes
after the target DEC's last section. `superseded_by` stays `null`, since most of each DEC stands;
the repo's partial-change precedent is an `## Amendment` section (DEC-047).

- **DEC-067:** `## Amendment — <date> (SPEC-144): publishing moves to the tag (DEC-111)`.
  "The *Publishing* section above is superseded by DEC-111. `npm publish` now runs in CI, from
  `release.yml`'s `custom-publish-npm` job, after every binary and the GitHub Release exist,
  through `just wasm-npm-publish` unchanged. The hand run is a recovery step. The name, the `web`
  target and lockstep versioning stand." Add `.github/workflows/publish-npm.yml` to its
  `affected_scope` (block list).
- **DEC-041:** `## Amendment — <date> (SPEC-144): crates.io moves behind host (DEC-111)`. "Point 2
  is superseded. crates.io publishes from `release.yml`'s `custom-publish-crates` job, a cargo-dist
  *custom* publish job (`./publish-crates`), after `host`. `publish-crates.yml` is now a
  `workflow_call` workflow with no tag trigger. Point 1 (Homebrew) and point 3 stand."
- **DEC-040:** `## Amendment — <date> (SPEC-144): runner override (DEC-111)`. "`aarch64-apple-darwin`
  builds on `macos-15` through `[dist.github-custom-runners]`; dist 0.32.0's default `macos-14` was
  retired on 2026-11-02. The custom publish jobs' permissions are set in
  `[dist.github-custom-job-permissions]`. `scripts/check-release-pipeline.sh` asserts both. The
  0.32.0 pin stands."

### Call 9 — docs, sentence by sentence

**`RELEASING.md`**

- **"Git tags"**: replace "Pushing a version tag ALSO (SPEC-042):" and its two bullets with:
  > Pushing a version tag is the one decision to publish (DEC-111). After every target has built
  > and the GitHub Release exists, the same workflow run publishes, in parallel:
  > - the Homebrew formula to **`jysf/homebrew-tap`** (`publish-homebrew-formula`);
  > - **crates.io** (`custom-publish-crates`, `cargo publish --locked`);
  > - **npm** `crustyimg-wasm` (`custom-publish-npm`, `just wasm-npm-publish` by trusted publishing).
  >
  > If any build fails, nothing publishes. A prerelease tag (`v0.9.0-rc.1`) creates a GitHub
  > prerelease and publishes nowhere else.
- **"One-time setup"**: in item 2, change "used by `publish-crates.yml`" to "used by the
  `custom-publish-crates` job". Add:
  > 3. **npm trusted publisher**: on npmjs.com, open `crustyimg-wasm` → **Settings** → **Trusted
  >    publisher** → **GitHub Actions**, then enter user **`jysf`**, repository **`crustyimg`** and
  >    workflow filename **`release.yml`**. Name `release.yml`, the workflow the tag starts, **not**
  >    `publish-npm.yml`: npm checks the *calling* workflow. Leave the environment empty.
  > 4. **After the first tag publishes npm from CI**, and only then: `crustyimg-wasm` →
  >    **Settings** → **Publishing access** → "Require two-factor authentication and disallow
  >    tokens". Until that first green publish, the hand fallback below must keep working.
- **Step 7**: replace "This single push fires all three channels (SPEC-042): …" with:
  > This single push fires all four channels, in this order: the builds (four targets) → `host`
  > creates the **GitHub Release** → **Homebrew**, **crates.io** and **npm** in parallel →
  > `announce`. Watch it on the Actions page (workflow **Release**). Confirm the one-time setup
  > above first.
- **Step 8**: replace the paragraph with:
  > **Verify the channels.** When the Release run is green: the GitHub Release has its artifacts
  > and checksums; `cargo search crustyimg` and `npm view crustyimg-wasm version` both show the
  > new version; `brew install jysf/tap/crustyimg` installs it.
  >
  > **If one publish job fails:** open the run and click **Re-run failed jobs**. Each publish job
  > first asks its registry whether the version is already there. If it is, the job prints
  > `SKIP: … already on …` and passes. If the registry could not be asked, the job fails with
  > `COULD NOT ASK …` and publishes nothing; re-run it later. A failed *build* publishes nothing at
  > all. Re-run it the same way.
  >
  > **Hand fallback (recovery only, [MAINTAINER-AUTHORIZED], irreversible).** Use it only when a
  > re-run cannot succeed, for example a revoked token or a broken npm trusted-publisher setting.
  > From a clean checkout of the tag (`git worktree add ../crustyimg-vX.Y.Z vX.Y.Z`), run
  > `scripts/release-publish-guard.sh npm vX.Y.Z`. If it prints `action=publish`, run `npm login`
  > and then `just wasm-npm-publish`. For crates.io, run `scripts/release-publish-guard.sh crates
  > vX.Y.Z`, then `cargo publish --locked`. Once "disallow tokens" is on, the npm hand path needs a
  > token-free login; fix the trusted-publisher setting and re-run the job instead.
- The "One-time setup" header sentence "(currently 0.4.0)" is stale but **out of scope**. Leave
  it.

**Other files** (comments only, no behaviour):

- `justfile`, the comment above `wasm-npm-publish`: replace "and it stays SPEC-076's
  maintainer-gated decision. That gate is "a human decides to publish" -- NOT …" with "The decision
  to publish is the `v*` tag push (DEC-111): CI's `publish-npm.yml` runs this recipe after the
  GitHub Release exists. Run by hand it is the recovery path, not a channel. Either way, the
  publish must never be shorter than the safe route -- NOT …". Keep the rest of the paragraph.
- `scripts/wasm-npm-finalize.mjs` header: "`npm publish` is gated on maintainer approval
  (SPEC-076)" → "`npm publish` happens only through `just wasm-npm-publish`, which CI runs on a tag
  (DEC-111)".
- `tests/npm_smoke.mjs:17`: "No npm publish — that is SPEC-076, and gated" → "No npm publish —
  that is `just wasm-npm-publish`, run by CI on a tag (DEC-111)".
- `projects/PROJ-011-…/HANDOFF.md`, "How the release went": replace the npm bullet's last two
  sentences with "npm is now published by CI on the tag (SPEC-144, DEC-111). The hand route
  (`npm login`, then `just wasm-npm-publish` from a detached worktree at the tag) is recovery only;
  see RELEASING.md step 8."
- Left alone: `docs/roadmap.md:239` and `docs/research/proj-008-wasm-build.md:471` (historical).

### Size: **[M], upper end.** Not cut.

The two scripts and the self-test are about half the build. The live PR run, the dry runs and the
mutation table are the other half. Nothing grew past M, so nothing was cut. The only addition
beyond the frame's file list is the two `scripts/` files, which the prompt invited.

## Inputs

- **Files to read:** `dist-workspace.toml`; `.github/workflows/release.yml`, `publish-crates.yml`,
  `pages.yml` (toolchain steps to copy); `justfile` (`wasm-build` … `wasm-npm-publish`);
  `scripts/release.sh` (exit-code convention); `scripts/ci-code-paths.sh` (the `--self-test`
  precedent); `RELEASING.md`; `decisions/DEC-040`, `DEC-041`, `DEC-066`, `DEC-067`, `DEC-111`.
- **External:** npm trusted publishers <https://docs.npmjs.com/trusted-publishers>; cargo-dist 0.32
  config reference <https://axodotdev.github.io/cargo-dist/book/reference/config.html>; npm registry
  `https://registry.npmjs.org/crustyimg-wasm`; crates.io sparse index
  `https://index.crates.io/cr/us/crustyimg`.
- **Tools:** `dist` **0.32.0** exactly; `actionlint` (1.7.12 at design); `jq`; `ruby` (YAML → JSON,
  preinstalled on `ubuntu-latest` and on macOS); `python3` (the self-test's fake registry).

## Outputs

- **Files created:**
  - `.github/workflows/publish-npm.yml`: Call 2.
  - `scripts/release-publish-guard.sh`: Calls 3 and 4, plus `--self-test`.
  - `scripts/check-release-pipeline.sh`: checks C1–C7.
- **Files modified:**
  - `dist-workspace.toml`: Call 1.
  - `.github/workflows/release.yml`: **regenerated only**.
  - `.github/workflows/publish-crates.yml`: reworked (Call 2); the tag trigger is removed.
  - `justfile`, `scripts/wasm-npm-finalize.mjs`, `tests/npm_smoke.mjs`: comments only (Call 9).
  - `RELEASING.md`, `projects/PROJ-011-…/HANDOFF.md`: Call 9.
  - `decisions/DEC-067-…`, `DEC-041-…`, `DEC-040-…`: amendment notes (Call 8).
- **New exports:** none. No `src/` change.

## Acceptance Criteria

Workflows, not `src/`: each AC names its check(s) (§ Checks) and what turns that check red.

- [ ] **AC-1** `dist plan` maps `aarch64-apple-darwin` to `macos-15`. *(C2. Red: runner ≠ macos-15. Live: L1's aarch64 job label.)*
- [ ] **AC-2** The other three targets keep `macos-15-intel`, `ubuntu-22.04` and `windows-2022`. *(C2's exact four-entry map. L1's three other labels.)*
- [ ] **AC-3** `release.yml` is exactly what `dist generate --mode=ci` at 0.32.0 produces. *(C1; V1 `git diff` empty. Red: any hand edit.)*
- [ ] **AC-4** `custom-publish-crates` **and** `custom-publish-npm` each: `needs == [plan, host]`; `uses` its own `publish-*.yml`; `permissions == {contents: read, id-token: write}`; `secrets: inherit`; `with.plan`; and an `if:` that contains `announcement_is_prerelease`. *(C3 and C4, one assertion per job per property.)*
- [ ] **AC-5** `announce.needs` includes both custom jobs, and its `if:` names both custom jobs' `.result`. Every other job's `needs` equals today's. *(C3 exact needs table, C5.)*
- [ ] **AC-6** No workflow has a tag-push trigger that publishes: `publish-crates.yml` and `publish-npm.yml` have **no** `push` trigger, and `cargo publish` / `npm publish` / `wasm-npm-publish` appear in no workflow file except those two. *(C6.)*
- [ ] **AC-7** In **each** publish workflow: `publish` runs only when `event_name == 'push' && ref_type == 'tag'` and the guard said `publish`, and it is the only job holding `id-token: write`; `dry-run` runs only on `pull_request` and names no secret; and `guard`'s script step is followed by the verdict-validation step. *(C7, per file.)*
- [ ] **AC-8** The guard refuses, with exit 1 and its line: a malformed tag, a prerelease tag, tag ≠ `Cargo.toml`, and a dirty tracked tree. *(Self-test G1–G4, plus local runs R1–R4 against a scratch checkout.)*
- [ ] **AC-9** Idempotent: a version that **is** on the registry gives exit 0, `SKIP:` and `action=skip`; one that is **not** gives exit 0, `PUBLISH:` and `action=publish`. Proved for **npm and crates.io**. *(G5, G6, G11, G12. Local R5 against the real registries with `v0.8.2` / Cargo.toml 0.8.2: SKIP on both.)*
- [ ] **AC-10** Could not ask means exit 3, `COULD NOT ASK`, and no `action=` written. Covered: unreachable, a 404 for the package, a 503, an unreadable 200 (npm); unreachable and a 503 (crates). *(G7–G10, G13, G14. Local R6: `RELEASE_GUARD_NPM_URL=http://127.0.0.1:9/x`.)*
- [ ] **AC-11** The self-test's summary counts every failure kind and exits 1 if any row fails. *(G-summary; mutation M12.)*
- [ ] **AC-12** The PR dry runs pass: `publish-npm / dry-run` (`(dry-run)` in its log) and `publish-crates / dry-run` (`aborting upload due to dry run`). *(D1, D2, quoted.)*
- [ ] **AC-13** The build PR's Release run (with `pr-run-mode = "upload"`) builds all four targets green, with the runner labels and the artifact list quoted from the API, the aarch64 binary is arm64, and `host` plus every publish job is skipped. *(L1, L2, L3.)*
- [ ] **AC-14** The merged state has no `pr-run-mode` key. `release.yml` is regenerated, and its diff from the run's version is only the upload-mode lines. *(C2 `pr_run_mode == "plan"`, L4.)*
- [ ] **AC-15** `actionlint` reports zero findings on `release.yml`, `publish-npm.yml` and `publish-crates.yml`. *(A1.)*
- [ ] **AC-16** DEC-111 exists (design). DEC-067, DEC-041 and DEC-040 carry the Call 8 amendments. `./scripts/decisions-audit.sh --changed` lists DEC-111 for the changed workflow and script paths. *(V2.)*
- [ ] **AC-17** Docs: RELEASING.md has each Call 9 item. Each one is checked by a `grep -n` the build quotes: `release.yml` in the trusted-publisher step; the order line; "Re-run failed jobs"; "disallow tokens" with "after"; "recovery only". Plus the three comment edits and the HANDOFF bullet. *(V3.)*
- [ ] **AC-18 (post-merge, maintainer; not a build gate)** The first real tag publishes both registries from CI after `host`, per RELEASING.md step 8. The ship cycle records the run id and both publish jobs' logs.

## Checks (in place of Failing Tests)

"Checks" reads truer than "Failing Tests" here: these are scripts and live runs, not `cargo test`.
**RED on `main` today** is stated for each one, and the build quotes the RED line before it writes
the change (trap 13).

### `scripts/check-release-pipeline.sh` (C1–C7)

Bash, `set -euo pipefail`. It needs `dist` (refuses unless `dist --version` is exactly
`cargo-dist 0.32.0`; exit 2), `ruby` and `jq`. It parses YAML with `ruby -ryaml -rjson -e 'puts
JSON.dump(YAML.load_file(ARGV[0]))'`, reading `on` as `d["on"] || d[true]` (P11). Each check
prints `ok Cn: …` or `FAIL Cn: <what>, expected <x>, got <y>`. All checks run even after a
failure. The last line is `<n> checks, <f> failed`, and the exit is 1 if `f > 0`. Assign every
command's output to a variable **before** testing it (trap 15). No `[ "$(cmd)" … ]`.

| Check | Asserts | RED on `main` today (expected line) |
|---|---|---|
| C1 | `dist generate --mode=ci --check` exits 0 | green (in sync today) |
| C2 | `dist plan --output-format=json` `.ci.github.artifacts_matrix.include[]` maps to **exactly** `{aarch64-apple-darwin: macos-15, x86_64-apple-darwin: macos-15-intel, x86_64-unknown-linux-gnu: ubuntu-22.04, x86_64-pc-windows-msvc: windows-2022}`, and `.ci.github.pr_run_mode == "plan"` | `FAIL C2: aarch64-apple-darwin runs on macos-14, expected macos-15` |
| C3 | `release.yml` job ids are exactly the 8 below, and each `needs` equals: `plan` none; `build-local-artifacts` [plan]; `build-global-artifacts` [plan, build-local-artifacts]; `host` [plan, build-local-artifacts, build-global-artifacts]; `publish-homebrew-formula` [plan, host]; `custom-publish-crates` [plan, host]; `custom-publish-npm` [plan, host]; `announce` [plan, host, publish-homebrew-formula, custom-publish-crates, custom-publish-npm] | `FAIL C3: job custom-publish-crates missing` (and npm), `FAIL C3: announce needs [plan,host,publish-homebrew-formula] …` |
| C4 | For **each** custom job: `uses == "./.github/workflows/publish-<x>.yml"`, `permissions == {contents: read, id-token: write}`, `secrets == "inherit"`, `with.plan` present, and `if` contains `announcement_is_prerelease` | `FAIL C4: custom-publish-crates missing` ×2 |
| C5 | `announce.if` contains `needs.custom-publish-crates.result` and `needs.custom-publish-npm.result` | FAIL ×2 |
| C6 | `publish-crates.yml` and `publish-npm.yml` both exist, and neither `on` has a `push` key. `grep -lE 'cargo publish\|npm publish\|wasm-npm-publish' .github/workflows/*.yml` returns exactly those two files | `FAIL C6: publish-crates.yml has a push trigger`; `FAIL C6: publish-npm.yml missing` |
| C7 | Per file: `on.workflow_call.inputs.plan` and `on.pull_request` exist; job `publish`'s `if` contains `github.event_name == 'push'`, `github.ref_type == 'tag'` and `needs.guard.outputs.action == 'publish'`, and its permissions include `id-token: write`; jobs `guard` and `dry-run` have **no** `id-token`; `dry-run.if` contains `pull_request`; no step in `dry-run` contains `secrets.`; `guard` has a step with `release-publish-guard.sh <npm\|crates>` whose **next** step's `run` contains `expected publish or skip`; npm only: `dry-run` has `npm_config_dry_run: "true"` and both npm jobs use `node-version: '24'` | FAIL (files missing) |

### `scripts/release-publish-guard.sh --self-test` (G1–G14)

It builds a scratch git repo under `mktemp -d` (its `Cargo.toml` is `version = "1.2.3"`, committed)
and starts a fake registry: `python3 -m http.server`-style, a ~25-line inline handler on a free
port, serving fixed routes:

- `/npm/absent`: 200, `{"versions":{"1.2.2":{}}}`
- `/npm/present`: 200, versions include `1.2.3`
- `/npm/404`, `/npm/503`
- `/npm/junk`: 200, `not json`
- `/crates/absent`, `/crates/present`: NDJSON lines with `"vers"`
- `/crates/503`

Unreachable is `http://127.0.0.1:9/`. Each row runs the guard **as a subprocess** with
`RELEASE_GUARD_RETRIES=0`, `GITHUB_OUTPUT` set to a fresh temp file, and cwd the scratch repo. It
asserts three things separately: the **exit code**, a **message substring**, and the **`action=`
content of `GITHUB_OUTPUT`** (exact line, or empty). It kills the server on exit (trap). The
summary line is `14 rows, F failed (exit: a, message: b, action: c)`, and the exit is 1 if `F > 0`.

| Row | Setup | Exit | Message contains | `GITHUB_OUTPUT` |
|---|---|---|---|---|
| G1 | tag `main` | 1 | `is not vMAJOR.MINOR.PATCH` | empty |
| G2 | tag `v1.2.3-rc.1` | 1 | `is a prerelease` | empty |
| G3 | tag `v1.2.4`, npm → `/npm/absent` | 1 | `tag v1.2.4 says 1.2.4, Cargo.toml says 1.2.3` | empty |
| G4 | tracked file modified, tag `v1.2.3`, npm → absent | 1 | `uncommitted changes` | empty |
| G5 | npm → `/npm/absent` | 0 | `PUBLISH: crustyimg-wasm@1.2.3` | `action=publish` |
| G6 | npm → `/npm/present` | 0 | `SKIP: crustyimg-wasm@1.2.3 is already on npm` | `action=skip` |
| G7 | npm → `/npm/404` | 3 | `COULD NOT ASK npm: HTTP 404` | empty |
| G8 | npm → `/npm/503` | 3 | `COULD NOT ASK npm: HTTP 503` | empty |
| G9 | npm → `http://127.0.0.1:9/` | 3 | `COULD NOT ASK npm: curl exit` | empty |
| G10 | npm → `/npm/junk` | 3 | `COULD NOT ASK npm: unreadable body` | empty |
| G11 | crates → `/crates/absent` | 0 | `PUBLISH: crustyimg@1.2.3` | `action=publish` |
| G12 | crates → `/crates/present` | 0 | `SKIP: crustyimg@1.2.3 is already on crates.io` | `action=skip` |
| G13 | crates → `/crates/503` | 3 | `COULD NOT ASK crates.io: HTTP 503` | empty |
| G14 | crates → `http://127.0.0.1:9/` | 3 | `COULD NOT ASK crates.io: curl exit` | empty |

G3 and G4 point the registry at **absent** on purpose. With their refusal removed, they reach
`PUBLISH` and exit 0, so they go red **for the missing guard**, never because a network call
failed (trap 13). On `main` the script does not exist, so every row is RED with "No such file".
The build quotes that once.

### Local refusal runs (R1–R6, AC-8 to AC-10; the build quotes each output and `$?`)

From a scratch `git worktree add --detach` at the build branch, with the outputs redirected and
`$?` read directly (no pipe):

- R1 `scripts/release-publish-guard.sh npm v9.9.9` → 1, the tag line.
- R2 Edit `README.md` (no commit), then run with `v<Cargo version>` → 1, the dirty-tree line.
- R3 `… crates v0.9.0-rc.1` → 1, the prerelease line.
- R4 `… npm main` → 1, the shape line.
- R5 **Already published, real registries.** At a worktree of tag `v0.8.2`, with the script copied
  in: `… npm v0.8.2` and `… crates v0.8.2` → 0, `SKIP:` and `action=skip` for both.
- R6 **Registry unreachable.** `RELEASE_GUARD_NPM_URL=http://127.0.0.1:9/x … npm v<current>` and
  `RELEASE_GUARD_CRATES_URL=http://127.0.0.1:9/x … crates v<current>` → 3, `COULD NOT ASK`.

### Live and other checks

- **L1** Commit (a)'s Release run: the jobs table (Call 7.2), quoted from `gh api`.
- **L2** The artifact list, quoted.
- **L3** `file` on the downloaded aarch64 binary: `arm64`.
- **L4** `git diff <a> <b> -- .github/workflows/release.yml`, quoted; only the upload-mode lines.
- **D1 / D2** The two dry-run jobs' quoted lines (Call 7.4).
- **A1** `actionlint` on the three workflow files: no output, exit 0. Record the version.
- **V1** `dist generate --mode=ci && git diff --exit-code .github/workflows/release.yml` → 0.
- **V2** `./scripts/decisions-audit.sh --changed` output, quoted.
- **V3** The `grep -n` lines for AC-17.

### Mutation table (for build and verify; traps 12 and 13)

Apply each mutation alone in a scratch worktree, run **C1–C7 and the self-test** (and A1 where
noted), record every red check with its quoted line, then revert and confirm green.
"Path" says where the mutation is applied. Most rows have one path. Where there are two
(hand-edit vs config), both are listed.

| # | Mutation | Exact location | Expected RED (and the line that proves the reason) | Expected GREEN |
|---|---|---|---|---|
| M1a | Drop `host` from a custom job's `needs`, by hand | `release.yml` → `custom-publish-npm.needs`: delete `- host` | C1 (`--check` exit 255, "out of date"); C3 (`FAIL C3: custom-publish-npm needs [plan], expected [plan, host]`); the PR's Release `plan` job (exit 255) | C2, C4–C7, self-test |
| M1b | The same, via config | n/a: dist always emits `needs: [plan, host]` for a custom job. **No config path exists.** Verify confirms by searching the 0.32 config reference for a `needs` override and records the result. | — | — |
| M2 | Hand-edit `release.yml` without regenerating | append `# hand edit` to `release.yml` | C1 only (`FAIL C1: … out of date`); Release `plan` job | C2–C7 (structure unchanged), self-test |
| M3 | Remove the runner override | delete the `[dist.github-custom-runners]` table from `dist-workspace.toml` (no regenerate: `release.yml` does not change, P3) | C2 (`FAIL C2: aarch64-apple-darwin runs on macos-14, expected macos-15`) | **C1** (byte-identical; this row proves C1 cannot see the runner), C3–C7, self-test |
| M4a | Remove the tag-version guard | delete the `Cargo.toml` comparison block in `release-publish-guard.sh` | G3 (`exit 0, expected 1` / `action=publish, expected empty`) | G1, G2, G4–G14, C1–C7 |
| M4b | Remove the guard **step** from a workflow | `publish-npm.yml` → `guard` job: delete the `release-publish-guard.sh` step | C7 (`FAIL C7: publish-npm.yml guard has no release-publish-guard.sh step`); A1 (an undefined `steps.guard`) | C1–C6, self-test |
| M5 | Treat a 5xx as "absent" | in the guard's HTTP-code branch: `case $code in 200) parse;; 5??) absent;; *) could-not-ask;; esac` | G8 and G13 (`exit 0, expected 3`; `action=publish, expected empty`) | G7 (404 still exits 3), G9, G14, the rest |
| M6 | Treat a 404 as "absent" | the same branch: `404) absent;;` | G7 | G8, G13, the rest |
| M7 | Read `npm view`'s exit code as the verdict (P5) | replace the npm curl+jq block with `npm view crustyimg-wasm@$V version` (exit 0 = present, else absent) | G7, G8, G9, G10 (each exits 0 with `action=publish`, because the fake registry is not npm's) | crates rows, G1–G4 |
| M8 | The verdict is junk | the guard writes `action=yes` | G5, G6, G11, G12 (`action=yes, expected action=publish`) | exit-code and message checks on those rows; G1–G4, G7–G10 |
| M9 | Drop the verdict-validation step | `publish-crates.yml` → `guard`: delete the "expected publish or skip" step | C7 (`FAIL C7: publish-crates.yml guard step is not followed by the verdict check`) | the rest |
| M10 | `publish` loses its event gate | `publish-npm.yml` → `publish.if`: delete `github.event_name == 'push' && github.ref_type == 'tag' &&` | C7 (`… publish.if lacks github.event_name == 'push'`) | the rest. (Live, `guard` is still event-gated, so `publish` would skip on its `needs`. This is why C7 checks both gates.) |
| M11 | Default custom-job permissions | delete `[dist.github-custom-job-permissions]`, then `dist generate --mode=ci` | C4 ×2 (`permissions {id-token: write, packages: write}, expected {contents: read, id-token: write}`) | **C1** (regenerated), C2, C3, C5–C7 |
| M12 | The self-test summary undercounts | in `--self-test`, increment the failure count only on exit-code mismatches (message and action mismatches still print `FAIL`), then also apply M8 | under M8 alone: self-test exit 1, summary `(exit: 0, message: 0, action: 4)`. Under M12+M8: it prints 4 `FAIL` lines and **exits 0**, so verify's cross-check **V-M12** goes red. V-M12 is run for every row of this table: the self-test exits 1 **iff** its output has a line starting `FAIL` | — |
| M13 | `pr-run-mode` left at upload | add `pr-run-mode = "upload"` under `[dist]`, then regenerate | C2 (`FAIL C2: pr_run_mode is upload, expected plan`) | C1, C3–C7 |
| M14 | A tag trigger comes back | `publish-crates.yml`: add `push: { tags: ['v*'] }` under `on` | C6 (`FAIL C6: publish-crates.yml has a push trigger`) | the rest |
| M15 | The PR dry run is not dry | `publish-npm.yml` → `dry-run`: delete the `npm_config_dry_run` env | C7 (`… dry-run lacks npm_config_dry_run`). Live (verify may push it to a scratch PR, optional): `ENEEDAUTH`, red | the rest |
| M16 | The guard crashes before writing a verdict (trap 15) | insert `exit 7` as the guard's first line after `set -euo pipefail` | every G row (`exit 7, expected …`); live, the `guard` step fails red, never a silent skip | C1–C7 |

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-111` (new, this design): the rule this spec implements.
- `DEC-040`: dist pinned at 0.32.0; `release.yml` is generated. Gets the runner amendment.
- `DEC-041`: point 2 is superseded (crates.io's separate workflow).
- `DEC-067`: Publishing is superseded. The name, target and lockstep version stand. `pkg/` must
  equal `Cargo.toml`, enforced by the unchanged recipe.
- `DEC-066`: the npm publish must route through `just wasm-build`'s size profile. `wasm-npm-publish`
  does, unchanged.
- `DEC-052`: release artifacts build default features. Do not add `features` to
  `dist-workspace.toml`.
- `DEC-015`: exit-code style. The guard uses 1 = refused, 2 = usage, 3 = could not ask.

### Constraints that apply

- `no-secrets-in-code`: only `${{ secrets.CARGO_REGISTRY_TOKEN }}` in the crates `publish` step's
  env. npm uses OIDC, with no token.
- `one-spec-per-pr`.
- `cost-captured-per-cycle`.

### Prior related work

- SPEC-042 (Homebrew + crates.io channels), SPEC-075 (the npm package), SPEC-076 (the hand
  publish), SPEC-142 (trap 15: `ci-code-paths.sh --self-test`, the self-test precedent).
- 0.8.2 Release run `41251779509` (the macos-14 brownout) and `publish-crates.yml` run
  `41251779599`.

### Out of scope (for this spec specifically)

- **Moving off `ubuntu-22.04`** (retires 2027-04-17): it changes the Linux binary's glibc floor.
  Filed as its own STAGE-054 backlog item.
- **Upgrading cargo-dist** past 0.32.0: it does not fix the runner, and 0.33.0 changes the shell
  installer's layout.
- **crates.io trusted publishing** (`rust-lang/crates-io-auth-action`): `CARGO_REGISTRY_TOKEN`
  stays. Filed in STAGE-054 (ruling 4).
- **The wasm round-trip tests in CI**: stays its own STAGE-054 entry (Frame Q8).
- **A GitHub Environment or a required reviewer** on the publish jobs (ruling 3).
- **Changes to the npm package's name, target, version line or contents** (DEC-067 stands).
- **`pr-run-mode = "upload"` as a permanent setting**: it is used once, on the build PR.
- **The recipe itself**: `just wasm-npm-publish` is unchanged except for its comment.
- **RELEASING.md's stale "(currently 0.4.0)"**.

## Notes for the Implementer

- **Never hand-edit `release.yml`.** Edit `dist-workspace.toml`, then `dist generate --mode=ci`.
  Use the repo's `dist` (`dist --version` = `cargo-dist 0.32.0`).
- **Order of work:** write both scripts and their self-test first. Quote C1–C7 and G1–G14 RED on
  `main`. Then the config, regenerate, the workflows, then GREEN. Then commit (a) with `upload`,
  push, wait for the Release run (background `gh run watch`; do not poll), and record L1–L3, D1
  and D2. Then commit (b), and record L4. **Push after each commit** (long cycles checkpoint).
- **Do not run** `git tag`, `npm publish` without `--dry-run`, `cargo publish` without
  `--dry-run`, or `gh workflow run` on any release or publish workflow. Do not touch npm or
  crates.io settings.
- **zsh:** quote `"===="`-style separators (a leading `=` expands as a command path) and do not
  rely on word-splitting. **macOS** has no `timeout(1)`, so give curl `--max-time`. GNU `sort -V
  -C` runs on the Ubuntu runner only; locally the npm version step is not run.
- **The self-test's fake registry** must bind `127.0.0.1` on a port it reports (port 0, then read
  the bound port), and the shell must `trap` to kill it.
- **Comments are plain** (no spec ids in user-facing text; DEC ids in workflow comments are fine,
  as `pages.yml` does with DEC-066).
- **Build on Sonnet or Opus?** Opus. The code is small, but the live-run choreography (commit a/b,
  red-by-design C2 on (a)), the 16-row mutation table and the "absent vs could not ask" reasoning
  are where a mechanical builder slips.

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
