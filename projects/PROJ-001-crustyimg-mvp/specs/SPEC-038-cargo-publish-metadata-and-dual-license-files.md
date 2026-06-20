---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-038
  type: chore                      # epic | story | task | bug | chore
  cycle: build                     # frame | design | build | verify | ship
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
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
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
