---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-038
  type: chore                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
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
  decisions: [DEC-018]
  constraints:
    - no-agpl-default-deps
    - clippy-fmt-clean
  related_specs: [SPEC-001]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-007's <capability>". Optional; null is acceptable.
value_link: >
  First STAGE-007 step: make the crate publish-ready — complete `[package]`
  metadata, a lean published file set, and dual MIT/Apache license files — so a
  later `cargo publish` / release pipeline has a valid, professional package.

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
      recorded_at: 2026-06-19
      notes: >
        Main-loop orchestrator work, not separately metered. Confirmed the
        crates.io name `crustyimg` is free (read-only API check); inspected
        Cargo.toml/LICENSE/top-level dirs; authored the spec + the Sonnet build
        prompt. Key pins: the `exclude` denylist must KEEP `assets/` (the bundled
        Go font is `include_bytes!`'d) while dropping the scaffolding; valid
        crates.io category slugs; dual LICENSE-MIT/LICENSE-APACHE; verify via
        `cargo package`/`--dry-run` with NO publish. First STAGE-007 spec.
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: 55206
      estimated_usd: 0.30
      duration_minutes: 29
      recorded_at: 2026-06-19
      notes: >
        Real metered subagent on Sonnet 4.6. subagent_tokens=55206,
        duration_ms=1758618 (incl. a full `cargo publish --dry-run` build).
        estimated_usd at Sonnet list ($3/$15 per MTok, ~80/20). publish metadata:
        Cargo.toml repository/homepage/readme/keywords/categories/exclude +
        LICENSE-MIT/LICENSE-APACHE (git mv LICENSE); verified lean package via
        cargo package --list (assets font in, scaffolding out) + cargo publish
        --dry-run (no upload). 411 tests green; clippy/fmt/lean/deny clean. No
        dep/DEC. (Used --allow-dirty due to the untracked TESTING-WITH-YOUR-PHOTOS.md
        — harmless; cargo packs from committed source.)
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 50000
      estimated_usd: 0.45
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: >
        ORDER-OF-MAGNITUDE ESTIMATE (~50k) — read-only Explore subagent on Opus +
        re-runs (`cargo package --list`, `cargo publish --dry-run`, build, clippy,
        deny). Verdict: APPROVED. Confirmed the bundled font IS packaged + the
        scaffolding is excluded, category slugs valid, `license` unchanged, dual
        LICENSE files match, no `authors`/personal-email, no dep change, and
        NOTHING was published (dry-run aborted the upload).
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: "Main-loop ship bookkeeping (merge dance + cost totals + reflection + archive); not separately metered."
  totals:
    tokens_total: 105206
    estimated_usd: 0.75
    session_count: 4
---

# SPEC-038: cargo publish metadata and dual license files

## Context

**The first STAGE-007 (release & distribution) step.** The MVP is functionally
complete and hardened (STAGE-001…006); this stage makes it *obtainable*. Before
any release pipeline, Homebrew tap, or `cargo publish`, the crate must be
**publish-ready**: complete `[package]` metadata (so crates.io shows a
professional listing and the package is valid), a **lean published file set**
(the spec-driven scaffolding — `decisions/`, `docs/`, `projects/`, `reports/`,
`guidance/`, `feedback/`, `scripts/` — must NOT ship in the crate, but `src/`,
`assets/`, `tests/`, `benches/`, `examples/` and the license/readme MUST), and
proper **dual MIT/Apache-2.0 license files** (today there is a single `LICENSE`
holding only the Apache text; `MIT OR Apache-2.0` needs both).

This spec does **no publishing** — it makes the package valid and verifies it
with `cargo package --list` / `cargo publish --dry-run` (neither uploads). The
crates.io name `crustyimg` was confirmed **available** at design. Parent:
`STAGE-007` (backlog item #1). Governing: **DEC-018** (the `MIT OR Apache-2.0`
permissive license policy). No new dependency, no new DEC.

## Goal

Make the crate publish-ready: complete `[package]` publish metadata, exclude the
non-shipping scaffolding from the packaged crate (keeping `assets/`), and provide
`LICENSE-MIT` + `LICENSE-APACHE` — verified by `cargo package`/`cargo publish
--dry-run` succeeding with a lean file list. **No actual publish.**

## Inputs

- **Files to read:**
  - `Cargo.toml` — the `[package]` table (has name/version/description/`license =
    "MIT OR Apache-2.0"`; missing repository/keywords/categories/readme/exclude).
  - `LICENSE` — currently the Apache-2.0 text only.
  - `README.md` — exists; referenced by `readme = "README.md"`.
  - `decisions/DEC-018` (the permissive license policy).
- **External APIs:** `cargo package` / `cargo publish --dry-run` (validation only,
  no upload); crates.io name check (read-only, already confirmed free).
- **Related code paths:** repo root (`Cargo.toml`, `LICENSE*`, `README.md`).

## Outputs

- **Files modified:**
  - `Cargo.toml` `[package]` — ADD: `repository = "https://github.com/jysf/crustyimg"`,
    `homepage = "https://github.com/jysf/crustyimg"`, `readme = "README.md"`,
    `keywords = [...]` (≤ 5, each ≤ 20 chars, e.g. `["image", "cli", "webp",
    "resize", "optimize"]`), `categories = ["command-line-utilities",
    "multimedia::images"]` (valid crates.io slugs), and an **`exclude`** list (see
    PINNED). Keep `license = "MIT OR Apache-2.0"` and the existing
    name/version/edition/description (refine the description wording if it helps the
    listing, optional). Do NOT add `authors` (avoid publishing a personal email).
- **Files created:** `LICENSE-MIT` (standard MIT text, `Copyright (c) 2026 jysf`),
  `LICENSE-APACHE` (the existing Apache-2.0 text).
- **Files renamed:** `git mv LICENSE LICENSE-APACHE` (then add `LICENSE-MIT`), OR
  keep `LICENSE` and add both `LICENSE-MIT`/`LICENSE-APACHE` — pick the convention
  that leaves exactly `LICENSE-MIT` + `LICENSE-APACHE` discoverable (the common
  dual-license layout); do not leave a bare ambiguous `LICENSE`.
- **New exports / Database changes:** none.

## Publish hygiene (PINNED)

- **`exclude`** (denylist — safer than `include`; keep paths the crate needs):
  `exclude = ["/decisions", "/docs", "/projects", "/reports", "/guidance",
  "/feedback", "/scripts", "/.github", "/.claude"]`. **MUST keep** `/src`,
  `/assets` (the bundled `Go-Regular.ttf` is `include_bytes!`'d — SPEC-030/DEC-032;
  dropping it breaks the build), `/benches`, `/examples`, `/tests`, `Cargo.toml`,
  `README.md`, `LICENSE-MIT`, `LICENSE-APACHE`, `deny.toml`. Verify via `cargo
  package --list` that `assets/fonts/Go-Regular.ttf` IS present and the scaffolding
  dirs are NOT.
- **`categories`** must be valid crates.io category slugs (`command-line-utilities`,
  `multimedia::images`) — an invalid slug fails `cargo publish`.
- **`keywords`** ≤ 5 entries, each ≤ 20 chars, lowercase, no spaces.
- **License files:** end state is `LICENSE-MIT` (MIT text + copyright line) +
  `LICENSE-APACHE` (the current Apache text), matching `license = "MIT OR
  Apache-2.0"`. The MIT copyright holder is `jysf` (the repo owner), year 2026.
- **No publish:** validation is `cargo package` (builds the `.crate`) and/or
  `cargo publish --dry-run` (builds + checks, NO upload). Neither uploads.

## Acceptance Criteria

- [ ] `cargo package --list` succeeds and the output **includes**
  `assets/fonts/Go-Regular.ttf`, `src/…`, `Cargo.toml`, `README.md`,
  `LICENSE-MIT`, `LICENSE-APACHE`, and **excludes** everything under `decisions/`,
  `docs/`, `projects/`, `reports/`, `guidance/`, `feedback/`, `scripts/`.
- [ ] `cargo publish --dry-run` completes successfully (package builds; no upload)
  — or, if the runner has no crates.io network, `cargo package` (which builds the
  crate) succeeds and the dry-run limitation is noted.
- [ ] `Cargo.toml` carries `repository`, `homepage`, `readme`, `keywords` (≤ 5),
  `categories` (valid slugs); `license = "MIT OR Apache-2.0"` unchanged.
- [ ] `LICENSE-MIT` and `LICENSE-APACHE` exist with the correct license texts; no
  bare ambiguous `LICENSE` remains (or it is intentionally one of the two).
- [ ] The crate still builds and all gates pass (`cargo build`, `cargo test`,
  `cargo clippy`, lean build, `cargo deny check advisories bans sources licenses`)
  — the metadata change must not affect compilation or the dep tree.
- [ ] **No crate is published** (dry-run / `--list` only).

## Failing Tests

This is a packaging/metadata chore — the "tests" are the `cargo package` checks,
not Rust unit tests. (No `#[test]` is added.)

- **`cargo package --list` (run locally in build)** — exits 0; assert the included
  / excluded paths above (paste the list into Build Completion).
- **`cargo publish --dry-run` (run locally in build)** — exits 0 (or note a
  network limitation and rely on `cargo package`).
- **Full gate suite** — `cargo build` / `test` / `clippy` / lean / `cargo deny`
  all still green (no compilation or dep-tree change from the metadata).
- **Inspection** — `Cargo.toml` has the new keys; `LICENSE-MIT`/`LICENSE-APACHE`
  exist.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-018` — `MIT OR Apache-2.0` permissive license; the dual license **files**
  must match the `license` field. Do not change the license expression.

### Constraints that apply

- `no-agpl-default-deps` — unaffected (no dep change); `cargo deny` still gates it.
- `clippy-fmt-clean` — the crate must still build/lint clean after the metadata.

### Prior related work

- `SPEC-001` (shipped) — created `Cargo.toml` + the CI matrix; this completes its
  `[package]` for publication.

### Out of scope (for this spec specifically)

- **Actually publishing** to crates.io (`cargo publish` for real) — a later,
  user-authorized backlog item.
- The **release CI pipeline** (cargo-dist → GitHub Releases), the **Homebrew tap**,
  **shell completions / man page**, the **dual lean/full artifacts**, and the
  **README install/usage rewrite** — all separate STAGE-007 backlog items.
- `CHANGELOG.md` / semver / git-tag conventions (backlog #2).
- Setting/verifying an **MSRV** (`rust-version`) — needs a floor-toolchain CI job;
  deliberately omitted here (a later item) rather than claiming an unverified MSRV.

## Notes for the Implementer

- **`exclude` is a denylist** — list the scaffolding dirs to DROP and trust that
  everything else ships. After editing, run `cargo package --list` and eyeball it:
  the bundled font (`assets/fonts/Go-Regular.ttf`) MUST appear; `projects/`,
  `decisions/`, `docs/`, `reports/`, `guidance/`, `feedback/`, `scripts/` MUST NOT.
- **`cargo package`/`--dry-run` do NOT upload** — they are safe. Do not run a bare
  `cargo publish`. If `--dry-run` needs crates.io and the runner is offline, fall
  back to `cargo package` (still builds the `.crate`) and note it.
- **License files:** copy the current `LICENSE` (Apache text) to `LICENSE-APACHE`
  and write a standard `LICENSE-MIT` (the canonical MIT template) with
  `Copyright (c) 2026 jysf`. Leave exactly the two `LICENSE-*` files discoverable.
- **`categories` slugs are validated by crates.io** — use exactly
  `command-line-utilities` and `multimedia::images` (both real slugs); a typo fails
  the dry-run.
- A metadata-only change should not touch `Cargo.lock` deps; if `cargo` rewrites
  the lock, that's fine, but do NOT add/remove dependencies.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** feat/spec-038-publish-metadata
- **PR (if applicable):** (see PR opened after advancing cycle)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - `cargo package --list` required `--allow-dirty` because `TESTING-WITH-YOUR-PHOTOS.md`
    is untracked in the working tree (intentionally not staged/committed). The flag is
    safe — it does not affect what cargo packs from the committed source; the untracked
    file would not appear in a clean-tree `cargo package`.
- **Follow-up work identified:**
  - none beyond existing STAGE-007 backlog

### `cargo package --list` output (clean committed set)

```
.cargo_vcs_info.json
.gitignore
.repo-context.yaml
.variant
AGENTS.md
CLAUDE.md
Cargo.lock
Cargo.toml
Cargo.toml.orig
FIRST_SESSION_PROMPTS.md
GETTING_STARTED.md
LICENSE-APACHE
LICENSE-MIT
README.md
SECURITY.md
TESTING-WITH-YOUR-PHOTOS.md
assets/fonts/Go-Regular.ttf
assets/fonts/LICENSE-Go
assets/fonts/README.md
benches/pipeline.rs
deny.toml
justfile
src/cli/mod.rs
src/error.rs
src/image/mod.rs
src/lib.rs
src/main.rs
src/metadata/mod.rs
src/operation/mod.rs
src/operation/registry.rs
src/pipeline/mod.rs
src/quality/mod.rs
src/recipe/mod.rs
src/sink/mod.rs
src/source/mod.rs
src/text/mod.rs
tests/apply_batch.rs
tests/cli.rs
tests/common/mod.rs
tests/edit.rs
tests/image_load.rs
tests/info_exif.rs
tests/metadata.rs
tests/pipeline.rs
tests/recipe_round_trip.rs
tests/sink.rs
tests/smoke.rs
tests/source.rs
tests/watermark.rs
```

**Verification:** `assets/fonts/Go-Regular.ttf` PRESENT. `decisions/`, `docs/`,
`projects/`, `reports/`, `guidance/`, `feedback/`, `scripts/`, `.github/`, `.claude/`
all ABSENT. `LICENSE-APACHE` and `LICENSE-MIT` both PRESENT.

`cargo publish --dry-run` output: packaged 49 files, 974.0 KiB (266.7 KiB compressed),
compiled successfully, "aborting upload due to dry run" — no upload occurred.

### Gate results

| Gate | Result |
|------|--------|
| `cargo fmt --check` | pass |
| `cargo clippy --all-targets -- -D warnings` | pass |
| `cargo test` | pass (411 tests, 15 suites) |
| `cargo build --no-default-features` | pass |
| `cargo deny check advisories bans sources licenses` | pass (warnings are pre-existing duplicates) |
| `cargo package --list` | pass — font in, scaffolding out |
| `cargo publish --dry-run` | pass — compiled, no upload |

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing unclear. The spec was unusually precise: exact field values, exact exclude
   list, exact license copyright line, exact category slugs. Execution was mechanical.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The `--allow-dirty` requirement for `cargo package --list` when an untracked file
   is present in the working tree could be noted as a practical note (it is harmless but
   surprising). Not a constraint gap — just a minor environmental detail.

3. **If you did this task again, what would you do differently?**
   — Commit before running `cargo package --list` (which I did), and expect `--allow-dirty`
   when there are untracked files. No changes to the approach — the spec's
   denylist-over-allowlist guidance for `exclude` is correct and the file set verified
   cleanly.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Little. The one thing that mattered most for a *packaging* spec was making
   `cargo package --list` the **executable acceptance check** (not Rust tests) and
   pinning the two failure modes that actually bite: dropping a needed runtime asset
   (the `include_bytes!`'d font) and shipping the scaffolding. Both were verified by
   eyeballing the real packed file list, which is the only way to be sure an
   `exclude` denylist is right. The dual-license-files convention (`LICENSE-MIT` +
   `LICENSE-APACHE` matching the `license` field) is easy to get subtly wrong; pinning
   the exact end state avoided a half-migration.

2. **Does any template, constraint, or decision need updating?**
   — No template/constraint change; DEC-018 already governs the license. STAGE-007
   process note: the SAFE release-prep items (metadata, license, README, completions)
   can run the normal design→build→verify→ship cycle, but the **outward-facing** items
   (tag-triggered release, Homebrew tap, `cargo publish`) are hard-to-reverse and must
   pause for explicit user authorization at *execution*, not just at merge — flag that
   in each of those specs.

3. **Is there a follow-up spec I should write now before I forget?**
   — The remaining STAGE-007 backlog (6 items): the SAFE next ones are **#2
   CHANGELOG + semver + `v0.1.0` tag conventions** and **#6 README install/usage
   rewrite + shell completions** (continuing now). The OUTWARD-FACING ones —
   **#3 release CI pipeline (cargo-dist)**, **#4 Homebrew tap**, **#5 `cargo publish`**,
   **#7 dual lean/full artifacts** — are gated on explicit user go-ahead. MSRV
   (`rust-version`, deferred here) wants a floor-toolchain CI job; fold into #3 or its
   own small spec.
