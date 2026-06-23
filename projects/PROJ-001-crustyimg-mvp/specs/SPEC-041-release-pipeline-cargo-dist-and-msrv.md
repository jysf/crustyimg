---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-041
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-007
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet (prescriptive prompt)
  created_at: 2026-06-23

references:
  decisions: [DEC-040]
  constraints:
    - no-new-top-level-deps-without-decision
    - clippy-fmt-clean
    - no-secrets-in-code
    - one-spec-per-pr
  related_specs: [SPEC-038, SPEC-039, SPEC-040]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-007's <capability>". Optional; null is acceptable.
value_link: >
  Backlog #3 — the tag-triggered cargo-dist release pipeline (cross-platform
  binaries + checksums + installers → GitHub Releases) plus a declared MSRV,
  delivering STAGE-007's "reproducible release from a tag" criterion and the
  plumbing #4/#5/#7 extend. Design + dry-run only — no release is cut here.

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
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-23
      notes: >
        Main-loop orchestrator work, not separately metered. Authored the spec +
        DEC-040 + the Sonnet build prompt for STAGE-007 #3 (cargo-dist release
        pipeline + MSRV). Ran a design-time probe (probe-load-bearing-crates-at-design,
        generalized to release tooling): installed `dist` 0.32.0, ran dist
        init/generate/plan against the real crate, and VERIFIED (a) the
        dist-workspace.toml config (4 brief-matching targets, shell+powershell
        installers, GitHub Releases, NO crates.io publish-job), (b) that `dist plan`
        emits per-target archives + sha256 checksums bundling the dual licenses +
        README + CHANGELOG, and (c) the SAFETY model of the generated release.yml
        (pull_request = non-publishing plan; push filtered to v* tags = publish;
        merging arms but does not cut a release) — then reverted all probe files
        (tree clean; `dist` left installed, harmless tooling). rustup/cargo-msrv not
        local → MSRV is a CI-verified declared floor, not a local bisect. Pinned: no
        tag/release/tap/publish; Homebrew installer + crates.io publish-job deferred
        to #4/#5.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-041: release pipeline cargo-dist and msrv

## Context

**STAGE-007 backlog #3 — the release CI pipeline.** SPEC-038 made the crate
publish-ready, SPEC-039 wrote the versioning/release policy, SPEC-040 gave it a
user-facing README + completions. What's missing is the **machinery that turns a
`vX.Y.Z` git tag into downloadable, checksummed, cross-platform binaries on GitHub
Releases** — the stage's success criterion "CI release pipeline … reproducible from a
tag (no manual artifact building)". The stage Design Notes nominate **`cargo-dist`**
(the Rust analog to `goreleaser`), and `RELEASING.md` step 7 already says "the release
pipeline (backlog #3) triggers on [the tag]".

This spec adds that pipeline (config + generated workflow) and **declares an MSRV**
(`rust-version`). It is the plumbing the remaining outward-facing items extend: #4
(Homebrew tap → a `homebrew` installer + tap repo), #5 (`cargo publish` → a crates.io
`publish-jobs` entry), #7 (dual lean/full artifacts → a second `--no-default-features`
build).

**This is design + dry-run ONLY — it cuts no release.** Per the probe baked into
DEC-040, the generated `release.yml` runs a **non-publishing plan** on pull requests
and only builds + creates a GitHub Release on a **pushed `v*` tag** (the
**[MAINTAINER-AUTHORIZED]** act in `RELEASING.md`). Merging this spec **arms** the
pipeline; it does not fire it. No tag, no release, no tap, no `cargo publish` here.

Parent: `STAGE-007` (#3). Decision: `DEC-040`. Related: `SPEC-038` (publish metadata +
dual licenses the archives bundle), `SPEC-039` (RELEASING/CHANGELOG the pipeline reads),
`SPEC-040` (README install section the installers back).

## Goal

Add a `cargo-dist` (`dist` `0.32.0`) release pipeline — a `dist-workspace.toml` +
`[profile.dist]` + a generated `.github/workflows/release.yml` that, on a pushed
`vX.Y.Z` tag, builds macOS arm64/x86_64 + Linux x86_64 + Windows binaries with
checksums and shell/powershell installers and creates a GitHub Release — and declare a
CI-enforced MSRV (`rust-version` + an `msrv` CI job). Validate by `dist plan` (dry-run);
**create no tag, release, tap, or crates.io publish.**

## Inputs

- **Files to read:**
  - `decisions/DEC-040-cargo-dist-release-pipeline.md` — the **probe-verified config,
    target/installer choices, and the safety model**. The exact `dist-workspace.toml`
    is in this DEC — use it verbatim.
  - `projects/PROJ-001-crustyimg-mvp/stages/STAGE-007-release-and-distribution.md` —
    Success Criteria + Design Notes (#3, cargo-dist).
  - `RELEASING.md` — the release-cut checklist; steps 6–8 are **[MAINTAINER-AUTHORIZED]**
    and reference "the pipeline (backlog #3)". A small wording update lands here so the
    docs match the now-existing pipeline.
  - `.github/workflows/ci.yml` — the existing PR-CI matrix (DEC-009); the new `msrv`
    job is added here, in that style (`dtolnay/rust-toolchain@<version>`).
  - `Cargo.toml` — `version = "0.1.0"`, `edition = "2021"`, the pinned deps; gains
    `[profile.dist]` and `rust-version`.
  - `README.md` (SPEC-040) — its install section references the installers/Releases;
    keep it consistent (no rewrite — a pointer at most).
- **External tooling:** `cargo-dist` / `dist` `0.32.0`
  (https://opensource.axo.dev/cargo-dist/) — release tool; **CI tooling, not a runtime
  dependency** (cf. cargo-deny / DEC-037). Install via the prebuilt installer or
  `cargo install cargo-dist --version 0.32.0 --locked`.
- **Related code paths:** repo root (`dist-workspace.toml`, `Cargo.toml`),
  `.github/workflows/` (`release.yml` generated, `ci.yml` edited), `RELEASING.md`.

## Outputs

- **Files created:**
  - `dist-workspace.toml` — the `[dist]` config (exact contents in DEC-040: 4 targets,
    `installers = ["shell", "powershell"]`, `ci = "github"`,
    `cargo-dist-version = "0.32.0"`, `install-path = "CARGO_HOME"`).
  - `.github/workflows/release.yml` — **generated by `dist generate`** (do not
    hand-author); pinned to dist 0.32.0; tag-triggered, with a non-publishing PR plan.
- **Files modified:**
  - `Cargo.toml` — add `[profile.dist]` (`inherits = "release"`, `lto = "thin"`) and
    `rust-version = "<MSRV>"` in `[package]`.
  - `.github/workflows/ci.yml` — add an `msrv` job that builds on exactly the pinned
    `rust-version` toolchain (default + `--no-default-features`), so the floor is
    enforced.
  - `RELEASING.md` — update the wording so the checklist reflects that the pipeline now
    exists (tag push → automated artifacts + GitHub Release); **keep the
    [MAINTAINER-AUTHORIZED] markers** on tag/push/publish.
- **New exports / Database changes:** none. No `src/` change; the shipped binary's
  dependency tree is unchanged.

## Acceptance Criteria

Testable outcomes.

- [ ] `dist-workspace.toml` exists with the DEC-040 config: `cargo-dist-version =
  "0.32.0"`, `ci = "github"`, `installers = ["shell", "powershell"]`, and exactly the
  4 targets `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`,
  `x86_64-pc-windows-msvc`.
- [ ] `Cargo.toml` has `[profile.dist]` (inherits release, `lto = "thin"`) and
  `rust-version = "<MSRV>"` in `[package]`.
- [ ] `.github/workflows/release.yml` exists, was produced by `dist generate` (pinned
  to 0.32.0), and `dist generate --check` reports it **in sync** with the config.
- [ ] **Safety (verifiable by reading `release.yml`):** it triggers a **non-publishing
  plan** on `pull_request` (`publishing: false`) and only builds + `gh release create`
  on a pushed `v*` **tag**; it does **not** run on an ordinary `main` push; it contains
  **no `cargo publish` / crates.io step** and **no Homebrew installer** (those are
  #5/#4).
- [ ] `dist plan` (the dry-run) succeeds and lists, for `v0.1.0`: the 4 per-target
  archives (`.tar.xz` for unix, `.zip` for windows), each with a `.sha256`, plus a
  combined `sha256.sum` and `crustyimg-installer.sh` / `.ps1`. Paste the output in
  Build Completion.
- [ ] An `msrv` CI job exists in `ci.yml` pinning the declared toolchain and building
  the crate (default + lean); the value declared in `rust-version` is the one the job
  pins.
- [ ] The existing gate suite stays green: `cargo fmt --check`, `cargo clippy
  --all-targets -- -D warnings`, `cargo test`, `cargo build --no-default-features`,
  `cargo deny check advisories bans sources licenses`. (No `src/` change; the new files
  don't affect a normal build.)
- [ ] **No outward-facing action:** `git tag` shows no new tag; no GitHub Release was
  created; no tap repo; no `cargo publish`. The diff adds only the release config /
  workflow / CI job / manifest fields / a RELEASING wording tweak.

## Failing Tests

This is **build-and-release tooling + CI config — no Rust unit tests** (no `src/`
change; `every-public-fn-tested` / `test-before-implementation` don't apply — there is
no new function). Verification is by the **dry-run + inspection** gate below, run in
build and re-run in verify:

- `dist plan` exits 0 and lists the 4 targets + checksums + installers for `v0.1.0`.
- `dist generate --check` reports `release.yml` is in sync with `dist-workspace.toml`.
- `release.yml` shows `publishing: ${{ !github.event.pull_request }}` (PR = plan only)
  and a `push:` trigger filtered to `tags:`; `grep` finds **no** `cargo publish` and
  **no** homebrew installer job.
- The existing gate suite (fmt/clippy/test/lean/deny) is green.
- `git tag` shows no new tag; `gh release list` shows no new release.

## Implementation Context

*Read this section (and DEC-040) before starting the build cycle.*

### Decisions that apply

- `DEC-040` — **the authoritative source**: the exact `dist-workspace.toml`, the pinned
  `dist 0.32.0`, the 4 targets + shell/powershell installers, the GitHub-Releases-only
  scope (no crates.io publish-job), the deferred Homebrew installer, the MSRV
  declaration, and the verified safety model. Follow it.
- `DEC-009` — the PR-CI matrix lives in `ci.yml`; the `msrv` job is added there in the
  same `dtolnay/rust-toolchain` style. The release pipeline is a **separate** workflow
  (`release.yml`), not part of the PR matrix.
- `DEC-027` — `display` default-on; the lean `--no-default-features` build is the
  headless artifact. The default release binary built here ships with `view`; the lean
  *artifact* is backlog #7 (out of scope) — but the `msrv` job should still cover the
  lean build so the floor holds for both.
- `DEC-037` — precedent that release/supply-chain **tooling** (cargo-deny) gets a DEC
  without being a runtime dep; cargo-dist is the same shape (DEC-040).
- `SPEC-038`/`DEC-038` — the dual `LICENSE-MIT`/`LICENSE-APACHE` the archives bundle.

### Constraints that apply

- `no-new-top-level-deps-without-decision` — cargo-dist is CI tooling (no `[dependencies]`
  entry); DEC-040 records the choice regardless.
- `no-secrets-in-code` — the release workflow uses the default `GITHUB_TOKEN` only; **do
  not add any crates.io token / secret** (that is #5, maintainer-authorized).
- `clippy-fmt-clean` — unaffected (no `src/`), but the gate must stay green.
- `one-spec-per-pr` — one PR: the release pipeline + MSRV.

### Prior related work

- `SPEC-038` (shipped, PR #42), `SPEC-039` (PR #43), `SPEC-040` (PR #44) — the three
  prior STAGE-007 steps this builds on.

### Out of scope (for this spec specifically)

- **Cutting an actual release** — no `git tag`, no tag push, no GitHub Release, no
  `dist build` upload. The pipeline is armed, not fired. (The real `v0.1.0` cut is a
  later **[MAINTAINER-AUTHORIZED]** action per `RELEASING.md`.)
- **#4 Homebrew tap** — no `homebrew` installer, no `jysf/homebrew-tap` repo.
- **#5 `cargo publish`** — no crates.io `publish-jobs`, no token/secret.
- **#7 dual lean/full artifacts** — a second `--no-default-features` release artifact is
  a separate item; this ships the default full binary only (the `msrv` job covering the
  lean build is *not* the same as publishing a lean artifact).
- A 5th `aarch64-unknown-linux-gnu` target (dist's default) — dropped to match the
  brief; a trivial future add.
- Hand-editing `release.yml` — it is machine-generated; change config + `dist generate`.

## Notes for the Implementer

- **Read DEC-040 first** — the `dist-workspace.toml` contents are there verbatim. The
  flow: install `dist 0.32.0` → write `dist-workspace.toml` (or `dist init --yes` then
  edit to match) → `dist generate` (writes `release.yml` + adds `[profile.dist]`) →
  `dist generate --check` (must report in-sync) → `dist plan` (the dry-run; paste it).
- **`dist generate` adds `[profile.dist]` to `Cargo.toml`.** After it runs, re-check
  `cargo fmt`/`cargo build` still pass (the profile is inert for normal builds).
- **MSRV:** rustup/cargo-msrv are **not** in the local toolchain, so determine the floor
  via **CI**, not a local bisect. Declare `rust-version = "1.85.0"` (a conservative
  `0.x` floor for the modern pinned deps) and add an `msrv` job to `ci.yml` that does
  `dtolnay/rust-toolchain@1.85.0` + `cargo build` + `cargo build --no-default-features`.
  **The PR's `msrv` job is the verification** — if it goes red (a dep needs newer),
  raise `rust-version` until green and keep the job's pin equal to it. (Optional: if you
  can run `cargo msrv find`, use the true minimum instead — but don't block on it.)
- **Safety is the headline.** After `dist generate`, **verify the generated
  `release.yml` yourself**: confirm `publishing: ${{ !github.event.pull_request }}`,
  that `push:` is filtered to `tags:`, and that there is no `cargo publish` step and no
  homebrew installer. If `dist` ever emits a crates.io publish step, remove the
  `publish-jobs` config — this spec is GitHub-Releases-only.
- **Do NOT** run `git tag`, `git push --tags`, `dist build` with upload, `gh release
  create`, or `cargo publish`; do NOT create the tap. `dist plan` is the only dist
  command that "runs" — it just prints the plan.
- **RELEASING.md:** a light wording pass — e.g. step 7's note can change from "the
  future release pipeline (backlog #3) will trigger" to "the release pipeline triggers";
  **keep every [MAINTAINER-AUTHORIZED] marker**. Don't rewrite the checklist.
- The generated `release.yml` is large — that's expected; it's owned by `dist`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
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
