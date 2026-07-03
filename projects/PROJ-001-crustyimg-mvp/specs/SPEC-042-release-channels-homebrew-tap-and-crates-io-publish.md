---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-042
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
  created_at: 2026-07-03

references:
  decisions: [DEC-041]
  constraints:
    - no-secrets-in-code
    - clippy-fmt-clean
    - one-spec-per-pr
  related_specs: [SPEC-038, SPEC-039, SPEC-040, SPEC-041]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-007's <capability>". Optional; null is acceptable.
value_link: >
  Backlog #4 + #5 — add the Homebrew tap (cargo-dist) and a tag-triggered
  crates.io `cargo publish` workflow to the SPEC-041 pipeline, so one `v0.1.0`
  tag lights up `brew install` and `cargo install` alongside GitHub Releases.
  Config only — arms the channels; fires nothing (no tag/secret/repo here).

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
      recorded_at: 2026-07-03
      notes: >
        Main-loop orchestrator work, not separately metered. Authored the spec +
        DEC-041 + the Sonnet build prompt for STAGE-007 #4+#5 (Homebrew tap +
        crates.io publish), after the maintainer chose "full launch, auto crates.io,
        defer #7". Ran a design-time `dist` probe (dist 0.32.0): confirmed (a)
        Homebrew IS a native cargo-dist publish-job (`installers += homebrew`, `tap`,
        `publish-jobs = ["homebrew"]` → a `publish-homebrew-formula` job pushing to
        `jysf/homebrew-tap` via `HOMEBREW_TAP_TOKEN`), and (b) crates.io is NOT — dist
        rejects `crates-io`/`cargo` as publish-jobs, so crates.io needs a SEPARATE
        tag-only `cargo publish` workflow (`CARGO_REGISTRY_TOKEN`). Reverted the probe
        (tree clean). Pinned: config-only (arms, fires nothing); both channels trigger
        ONLY on a `v*` tag; #7 deferred; the tap-repo + two secrets + the tag push are
        maintainer-authorized (not in this spec).
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-042: release channels homebrew tap and crates io publish

## Context

**STAGE-007 backlog #4 (Homebrew tap) + #5 (crates.io publish).** SPEC-041 armed the
cargo-dist pipeline that turns a `v*` tag into GitHub-Releases binaries. This spec adds
the two remaining install channels so one `v0.1.0` tag also lights up
`brew install jysf/tap/crustyimg` and `cargo install crustyimg`. The maintainer chose a
**"full launch"** with crates.io publishing **automated on the tag**; the dual lean/full
artifact (#7) is **deferred** to a fast-follow (see DEC-041 / `docs/backlog.md`).

Per the DEC-041 probe, the two channels differ in mechanism:
- **Homebrew is native to cargo-dist** — a config change regenerates `release.yml` with
  a `publish-homebrew-formula` job.
- **crates.io is NOT a cargo-dist job** (dist 0.32 rejects it) — so it needs a **separate
  minimal tag-triggered `cargo publish` workflow**.

**This is config only — it arms the channels but fires nothing.** Both trigger **only on
a pushed `v*` tag**. Merging this spec creates no tag, no release, no publish, no tap
repo, and adds no secret. The **[MAINTAINER-AUTHORIZED]** acts — creating the
`jysf/homebrew-tap` repo, adding the `HOMEBREW_TAP_TOKEN` + `CARGO_REGISTRY_TOKEN` repo
secrets, and pushing the tag — happen **after** this ships, not here.

Parent: `STAGE-007` (#4, #5). Decision: `DEC-041`. Related: `SPEC-041` (the pipeline
this extends), `SPEC-038` (publish-ready crate), `SPEC-040` (README install section /
the `brew install jysf/tap/crustyimg` line, already correct).

## Goal

Add the Homebrew tap to the cargo-dist config (`installers += homebrew`,
`tap = "jysf/homebrew-tap"`, `publish-jobs = ["homebrew"]`) and regenerate
`release.yml`, and add a separate `.github/workflows/publish-crates.yml` that runs
`cargo publish --locked` on a `v*` tag — so the next tag publishes to Homebrew and
crates.io alongside GitHub Releases. Config only; validated by `dist plan` /
`dist generate --check` / workflow inspection. **No tag, release, publish, tap repo, or
secret is created here.**

## Inputs

- **Files to read:**
  - `decisions/DEC-041-release-channels-homebrew-and-crates-io.md` — the exact config +
    the probe-verified mechanism split + the safety model. Authoritative.
  - `decisions/DEC-040-...md` + `dist-workspace.toml` + `.github/workflows/release.yml`
    (SPEC-041) — the pipeline being extended.
  - `RELEASING.md` — the release-cut checklist; steps 6–8 [MAINTAINER-AUTHORIZED]. A
    small update lands here documenting the new secrets + tap-repo prerequisite.
  - `README.md` — the install section; `brew install jysf/tap/crustyimg` (line ~33) is
    already correct for a repo named `homebrew-tap` (no change needed).
  - `Cargo.toml` — `version = "0.1.0"`, `rust-version = "1.89.0"`; publish metadata
    (SPEC-038). No change here.
- **External tooling:** `cargo-dist` / `dist` `0.32.0` (installed). CI tooling, not a
  runtime dep.
- **Related code paths:** `dist-workspace.toml`, `.github/workflows/`
  (`release.yml` regenerated, `publish-crates.yml` new), `RELEASING.md`.

## Outputs

- **Files created:**
  - `.github/workflows/publish-crates.yml` — a standalone workflow triggered **only** on
    `push: tags: ['v*']` (or the version-tag glob `release.yml` uses), that checks out,
    installs stable Rust, and runs `cargo publish --locked` with
    `CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}`. **No `pull_request` /
    branch trigger** — it can only ever run on a tag. Minimal; single job.
- **Files modified:**
  - `dist-workspace.toml` — add `homebrew` to `installers`, add `tap = "jysf/homebrew-tap"`
    and `publish-jobs = ["homebrew"]`.
  - `.github/workflows/release.yml` — **regenerated by `dist generate`** (adds the
    `publish-homebrew-formula` job referencing `secrets.HOMEBREW_TAP_TOKEN` and
    `repository: "jysf/homebrew-tap"`). Do NOT hand-edit.
  - `RELEASING.md` — document, in the checklist, the new one-time prerequisites (create
    `jysf/homebrew-tap`; add `HOMEBREW_TAP_TOKEN` + `CARGO_REGISTRY_TOKEN` secrets) and
    that pushing the tag now also publishes to Homebrew + crates.io. Keep the
    [MAINTAINER-AUTHORIZED] markers.
- **New exports / Database changes:** none. No `src/` change.

## Acceptance Criteria

- [ ] `dist-workspace.toml` has `installers = ["shell", "powershell", "homebrew"]`,
  `tap = "jysf/homebrew-tap"`, and `publish-jobs = ["homebrew"]` (targets/version
  unchanged from SPEC-041).
- [ ] `.github/workflows/release.yml` was regenerated by `dist generate` (pinned 0.32.0),
  `dist generate --check` reports **in sync**, and it now contains a
  `publish-homebrew-formula` job referencing `repository: "jysf/homebrew-tap"` and
  `secrets.HOMEBREW_TAP_TOKEN`.
- [ ] `.github/workflows/publish-crates.yml` exists, triggers **only** on a version
  `tags:` push (NO `pull_request`, NO branch push), runs `cargo publish --locked`, and
  reads the token from `secrets.CARGO_REGISTRY_TOKEN` (the token is **not** hard-coded —
  `no-secrets-in-code`).
- [ ] **Safety:** neither channel can fire without a `v*` tag. `release.yml` still runs a
  non-publishing plan on PRs (`publishing: ${{ !github.event.pull_request }}`);
  `publish-crates.yml` has no PR/branch trigger at all. Confirm by inspection.
- [ ] `dist plan` succeeds and now lists the Homebrew formula artifact alongside the
  shell/powershell installers + the 4 target archives.
- [ ] `RELEASING.md` documents the tap-repo + two-secret prerequisites and keeps the
  [MAINTAINER-AUTHORIZED] markers.
- [ ] Existing gate suite stays green: `cargo fmt --check`, `cargo clippy --all-targets
  -- -D warnings`, `cargo test`, `cargo build --no-default-features`, `cargo deny check
  advisories bans sources licenses`. (No `src/` change.)
- [ ] **No outward-facing action:** `git tag` shows no new tag; no GitHub Release; no
  `cargo publish` was run; no `jysf/homebrew-tap` repo created; no secret added. The diff
  is config + workflows + a RELEASING wording update + the spec docs.

## Failing Tests

Build-and-release **tooling / CI config — no Rust tests** (no `src/` change; no new
function). Verification is by dry-run + inspection, run in build and re-run in verify:

- `dist generate --check` reports `release.yml` in sync with `dist-workspace.toml`.
- `dist plan` exits 0 and lists the Homebrew formula + the shell/powershell installers +
  the 4 target archives.
- `grep` confirms `release.yml` has the `publish-homebrew-formula` job with
  `jysf/homebrew-tap` + `HOMEBREW_TAP_TOKEN`.
- `grep` confirms `publish-crates.yml` triggers only on `tags:`, runs `cargo publish
  --locked`, uses `secrets.CARGO_REGISTRY_TOKEN`, and hard-codes no token.
- `git tag` shows no new tag; `gh release list` shows none; the existing gate suite is
  green.

## Implementation Context

*Read this section (and DEC-041) before starting the build cycle.*

### Decisions that apply

- `DEC-041` — **authoritative**: the exact Homebrew config, the separate crates.io
  workflow (because dist can't publish to crates.io — probe-verified), the deferred #7,
  and the safety model. Follow it.
- `DEC-040` — the SPEC-041 pipeline this extends; `release.yml` is machine-generated
  (config + `dist generate`, never hand-edit); tag-only publish, PR = plan.
- `DEC-037` — precedent that release/supply-chain tooling gets a DEC without being a
  runtime dep; cargo-dist + the crates.io workflow are the same shape.

### Constraints that apply

- `no-secrets-in-code` — tokens come from `secrets.*` only; never hard-code a crates.io
  token or PAT. The workflows reference `secrets.CARGO_REGISTRY_TOKEN` /
  `secrets.HOMEBREW_TAP_TOKEN` (added later by the maintainer).
- `clippy-fmt-clean` — unaffected (no `src/`), but the gate must stay green.
- `one-spec-per-pr` — one PR: both release channels + the RELEASING update.

### Prior related work

- `SPEC-041` (shipped, PR #45) — the cargo-dist pipeline + MSRV this extends.
- `SPEC-038` (PR #42) — publish-ready `Cargo.toml` metadata (crates.io needs it);
  `SPEC-040` (PR #44) — the README install section.

### Out of scope (for this spec specifically)

- **Any outward-facing / [MAINTAINER-AUTHORIZED] action**: creating the
  `jysf/homebrew-tap` repo, adding the `HOMEBREW_TAP_TOKEN` / `CARGO_REGISTRY_TOKEN`
  secrets, cutting/pushing the `v0.1.0` tag, running `cargo publish`. This spec only
  writes the config/workflows.
- **#7 dual lean/full artifact** — deferred (DEC-041); a fast-follow, not here.
- **`release-plz` or version-management automation** — not adopted (DEC-041); the plain
  `cargo publish --locked` on tag is the chosen mechanism.
- **Any `src/` or dependency change.**

## Notes for the Implementer

- **Read DEC-041 first.** `dist` 0.32.0 is installed (`~/.cargo/bin/dist`); confirm
  `dist --version`. Flow: edit `dist-workspace.toml` (add homebrew installer + tap +
  publish-jobs) → `dist generate` → `dist generate --check` (in sync) → `dist plan`
  (paste it). Then hand-write `publish-crates.yml`.
- **`publish-crates.yml` shape** (keep it minimal and tag-only):
  ```yaml
  name: Publish to crates.io
  on:
    push:
      tags: ['**[0-9]+.[0-9]+.[0-9]+*']   # match release.yml's version-tag glob
  jobs:
    publish:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - run: cargo publish --locked
          env:
            CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
  ```
  Use the **same tag glob** `release.yml` uses (check it — it's a `**[0-9]+...` pattern),
  so the two fire on the same tags. NO `pull_request:` and NO `branches:` — tag-only.
- **Safety check yourself** after `dist generate`: `release.yml` still has `publishing:
  ${{ !github.event.pull_request }}` and a tag-filtered `push:`; the new
  `publish-homebrew-formula` job uses `HOMEBREW_TAP_TOKEN` + `jysf/homebrew-tap`; and
  `publish-crates.yml` has no non-tag trigger.
- **Do NOT** run `git tag`, `git push --tags`, `gh release`, `cargo publish`, create the
  tap repo, or add any secret. `dist plan` is the only dist command that "runs".
- `RELEASING.md`: add the one-time prerequisites (create `jysf/homebrew-tap`; add the two
  secrets) near the top of the checklist, and note the tag push now also publishes to
  Homebrew + crates.io. Keep every [MAINTAINER-AUTHORIZED] marker.
- The default binary keeps `view` (DEC-027); the lean artifact (#7) is deferred — do not
  add a second artifact here.

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
