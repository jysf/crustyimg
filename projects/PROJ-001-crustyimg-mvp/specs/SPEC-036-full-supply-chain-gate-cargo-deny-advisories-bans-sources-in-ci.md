---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-036
  type: chore                      # epic | story | task | bug | chore
  cycle: build                     # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-006
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet (prescriptive prompt)
  created_at: 2026-06-19

references:
  decisions: [DEC-037, DEC-018, DEC-009]
  constraints:
    - untrusted-input-hardening
    - no-agpl-default-deps
    - no-new-top-level-deps-without-decision
  related_specs: [SPEC-001]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: >
  Fourth STAGE-006 hardening item: turns the license-only cargo-deny CI job into
  the full supply-chain gate (advisories + bans + sources + licenses) so a
  vulnerable/unmaintained/yanked or non-crates.io dependency fails CI.

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

# SPEC-036: full supply-chain gate (cargo-deny advisories + bans + sources) in CI

## Context

**The fourth STAGE-006 hardening item — a CI/tooling chore.** SPEC-033/034/035
hardened the runtime (decode, paths, recipes); this closes the **supply-chain**
blind spot. CI already runs `cargo-deny`, but only `check licenses` (DEC-018) —
so a license violation fails CI, but a **known-vulnerable, unmaintained, or
yanked** dependency, a **duplicate/banned** crate, or a **non-crates.io source**
passes unnoticed. `cargo-deny` evaluates all four axes from one config; only the
CI `command` and a few `deny.toml` sections need adding.

Per **DEC-037**, the gate becomes `cargo deny check advisories bans sources
licenses` (and `just deny` mirrors it). `cargo audit` is **not** added: its
RUSTSEC advisory database is exactly what `cargo deny check advisories` reads, so
a separate job would be redundant. Parent: `STAGE-006` (backlog item #4).
Governing: **DEC-037** (the gate + cargo-audit consolidation), **DEC-018** (the
license policy this extends), **DEC-009** (CI). No new runtime dependency.

## Goal

Extend the CI `cargo-deny` job and `just deny` from `check licenses` to the full
`cargo deny check advisories bans sources licenses`, add schema-correct
`[advisories]`/`[bans]`/`[sources]` sections to `deny.toml`, and ensure the full
check is **green** — so a vulnerable/unmaintained/yanked/banned/non-crates.io
dependency fails CI, with any real finding handled by a dep bump or a
narrowly-scoped commented exception.

## Inputs

- **Files to read:**
  - `.github/workflows/ci.yml` — the `licenses` job (`EmbarkStudios/cargo-deny-action@v2`,
    `command: check licenses`) to extend.
  - `deny.toml` — currently `[graph]` + `[licenses]` only; add `[advisories]`,
    `[bans]`, `[sources]`.
  - `justfile` — the `deny` recipe (`cargo deny check licenses`) to update.
  - `decisions/DEC-037` (the policy), `DEC-018` (license policy), `DEC-009` (CI).
- **External APIs:** `cargo-deny` (the `EmbarkStudios/cargo-deny-action@v2` action,
  already used) + the RUSTSEC advisory DB (fetched at check time). No new crate.
  Docs: https://embarkstudios.github.io/cargo-deny/checks/index.html
- **Related code paths:** `.github/workflows/`, `deny.toml`, `justfile`.

## Outputs

- **Files modified:**
  - `.github/workflows/ci.yml` — the cargo-deny job `command:` →
    `check advisories bans sources licenses`; rename the job to reflect the
    broadened scope (e.g. "supply-chain policy (cargo-deny)").
  - `deny.toml` — add `[advisories]` (deny vulnerabilities/unmaintained, `yanked
    = "deny"`, empty `ignore`), `[bans]` (`multiple-versions = "warn"`, empty
    `deny`/`skip`), `[sources]` (`unknown-registry = "deny"`, `unknown-git =
    "deny"`, `allow-registry = ["https://github.com/rust-lang/crates.io-index"]`).
    Use the schema version current `cargo-deny` expects; validate with the tool.
  - `justfile` — `deny` recipe → `cargo deny check advisories bans sources licenses`.
  - `docs/api-contract.md` is NOT touched (no CLI surface change).
- **Files created:** none.
- **New exports / Database changes:** none.

## Policy (PINNED — DEC-037)

- **CI command:** `check advisories bans sources licenses` (explicit list, not the
  implicit `check`, for auditability).
- **advisories:** fail on RUSTSEC vulnerabilities + unmaintained; `yanked =
  "deny"`. `ignore = []` (any future advisory exception is a commented, dated entry
  with a revisit note — never a blanket disable).
- **bans:** `multiple-versions = "warn"` (visibility without breaking the build on
  unavoidable duplicate transitive versions); `deny = []`, `skip = []` for now.
- **sources:** only crates.io is allowed (`unknown-registry`/`unknown-git` =
  `"deny"`; `allow-registry` = the crates.io index). No git/path/alt-registry deps.
- **licenses:** UNCHANGED (DEC-018 `allow`/`exceptions`/`confidence-threshold`).
- **`cargo audit` is NOT added** (redundant with `cargo deny check advisories`,
  DEC-037).
- **Findings policy:** a real advisory/ban is fixed by a **dependency bump**; only
  if no fix exists, a **narrowly-scoped, commented** `ignore`/`skip` with a
  revisit trigger (mirrors the license `exceptions` discipline).

## Acceptance Criteria

- [ ] CI's cargo-deny job runs `check advisories bans sources licenses` (verified
  by reading `.github/workflows/ci.yml`) and is **green** on the PR.
- [ ] `cargo deny check advisories bans sources licenses` passes locally (the
  build agent runs it; if an advisory/ban surfaces, it is resolved per the
  findings policy, NOT by disabling a whole check).
- [ ] `deny.toml` has schema-correct `[advisories]`, `[bans]`, `[sources]`
  sections and the existing `[licenses]` policy is unchanged (same `allow` +
  `exceptions`).
- [ ] `just deny` runs the same full check (local == CI).
- [ ] The existing license-only behavior is preserved (a copyleft dep would still
  fail, now alongside advisory/ban/source failures); the rest of CI is unchanged.
- [ ] No new top-level runtime dependency is added (cargo-deny is CI tooling).

## Failing Tests

This is a CI/config chore — the "tests" are the gate itself, not Rust unit tests.

- **`cargo deny check advisories bans sources licenses` (run locally in build)** —
  must exit 0. This is the executable acceptance check; the build agent runs it
  and pastes the result into Build Completion.
- **`.github/workflows/ci.yml` (assert by inspection)** — the cargo-deny step's
  `command:` is `check advisories bans sources licenses` (not `check licenses`).
- **`just deny` (run locally)** — runs the full check and exits 0.
- **CI on the PR** — the (renamed) supply-chain job is green across the workflow.
- *No Rust `#[test]` is added* (there is no library/CLI behavior change to unit-test).

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-037` — the full-gate policy + the `cargo audit` consolidation. **Implement
  exactly the four checks; do NOT add a separate cargo-audit job.**
- `DEC-018` — the permissive license policy; **do not change** `[licenses]`
  (`allow`/`exceptions`/`confidence-threshold`) — only ADD the other sections.
- `DEC-009` — CI conventions (no secrets; minimal permissions).

### Constraints that apply

- `untrusted-input-hardening` — the supply chain is part of the threat surface.
- `no-agpl-default-deps` — already enforced by `[licenses]`; unchanged.
- `no-new-top-level-deps-without-decision` — no new runtime dep is added here.

### Prior related work

- `SPEC-001` (shipped) — the CI workflow + matrix (DEC-009); the `licenses` job
  (DEC-018) is the one extended here.

### Out of scope (for this spec specifically)

- A separate `cargo audit` job (redundant with `cargo deny check advisories`,
  DEC-037).
- A scheduled/cron advisory job (the blocking PR gate is enough for the MVP; a
  non-blocking nightly advisory scan is a possible later addition).
- Bumping dependencies for reasons OTHER than a surfaced advisory/ban (out of
  scope; this is a gate-wiring chore, not a dependency-upgrade pass).
- The threat-model verification pass + `/security-review` (backlog item #5).

## Notes for the Implementer

- **Validate against the real tool.** `cargo-deny`'s `deny.toml` schema has
  changed across versions (e.g. `[advisories]` once used
  `vulnerability`/`unmaintained` keys that are now defaults; modern schema may want
  `version = 2` under a section). Write the sections, then run `cargo deny check
  advisories bans sources licenses` and fix any schema deprecation warnings/errors
  it reports until it is clean. The tool is the source of truth, not this spec's
  example keys.
- **If an advisory surfaces** on a transitive dep with a fix available, bump it
  (`cargo update -p <crate>`); if NO fix exists, add a commented, dated
  `ignore = ["RUSTSEC-XXXX-YYYY"]` entry with a one-line reason + revisit trigger.
  Do the same for an unavoidable duplicate under `[bans] skip`. **Never** switch a
  whole check to `"allow"`/remove it to make CI pass — that defeats the spec.
- **Keep `[licenses]` byte-for-byte** as it is (DEC-018). The diff should ADD the
  three sections + change the CI `command` + the `just deny` recipe, nothing else.
- The cargo-deny CI job needs no toolchain step (the action is self-contained); do
  not add a `dtolnay/rust-toolchain` step to it.
- Record the exact `cargo deny check …` output (clean) in `## Build Completion` so
  verify can confirm the gate actually passed, not just that the YAML changed.

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
