---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-056
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L

project:
  id: PROJ-004
  stage: STAGE-015
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-06

references:
  decisions: [DEC-050, DEC-025, DEC-049]
  constraints: [no-new-top-level-deps-without-decision, ergonomic-defaults, test-before-implementation]
  related_specs: [SPEC-052, SPEC-057]

value_link: >
  The GitHub-native code-scanning tier (`lint --format sarif`) + the 0.4.0
  release-cut that ships the whole lint wave — the notes announce "crustyimg lint
  + a setup-crustyimg Action + a crustyimg-action lint mode".

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-06
      notes: >
        Main-loop orchestrator (STAGE-015 continuation — SARIF + release-cut framing), not
        separately metered.
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 92000
      estimated_usd: 0.83
      duration_minutes: 24
      recorded_at: 2026-07-06
      notes: >
        ESTIMATE — autonomous merge-on-green run in the orchestrator main loop, NOT a metered
        subagent. Order-of-magnitude (~92k at Opus 4.8 ~80/20 ≈ $0.83). Added write_sarif (mirror
        write_json; version-param golden; cwd-relativized/forward-slashed uris) + --format sarif
        wiring + README code-scanning snippet; staged 0.4.0 (Cargo.toml + CHANGELOG + compare-links
        + Cargo.lock). 4 SARIF unit + 2 integration tests. PR #64.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 10000
      estimated_usd: 0.09
      duration_minutes: 3
      recorded_at: 2026-07-06
      notes: >
        ESTIMATE — same autonomous run; CI-driven verify, all matrix/feature/lean/msrv/deny jobs
        green on #64 (cross-OS SARIF golden stable). Order-of-magnitude (~10k).
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-06
      notes: >
        Main-loop ship bookkeeping (reflection, cost totals, stage backlog, archive) + STAGE-015
        stage-ship, not separately metered.
  totals:
    tokens_total: 102000
    estimated_usd: 0.92
    session_count: 4
---

# SPEC-056: `lint --format sarif` + the 0.4.0 release-cut

## Context

`lint` emits `human` and `json` (SPEC-052). The GitHub-native adoption tier is **SARIF** — upload it
with `github/codeql-action/upload-sarif` and findings land in the repo's **Security tab**, persist
across runs, and annotate PRs even without the wrapper Action. It's hand-rollable (a third renderer
alongside `write_json`, **no new dependency**), exactly the JSON/`write_diff_json`/`ExplainTrace`
precedent. This is the **last piece of STAGE-015 (and PROJ-004's lint wave)**: after it, cut **0.4.0**
— the release that actually publishes `lint` and makes the two shipped Actions usable by real
consumers (today no published release has `lint`). Tagging/publishing stays the maintainer's step.

## Goal

Add `lint --format sarif` — a hand-rolled, deterministic SARIF 2.1.0 report (a single `run` with the
`crustyimg` tool component + the rule catalog + one result per finding), golden-testable and
cross-OS stable; document the code-scanning upload; then stage the **0.4.0** version bump + CHANGELOG
(untagged). Adds no dependency; the output format never changes the exit code (DEC-025).

## Inputs

- **Files to read:**
  - `src/lint/report.rs` (SPEC-052) — `write_json` + `escape_json` + the synthetic-golden discipline
    to mirror; `Severity`/`Finding`/`LintOutcome`.
  - `src/analysis/decide.rs` (SPEC-049) — the hand-rolled-JSON + synthetic-golden precedent.
  - `src/cli/mod.rs` — `lint_report_format` / `LintReportFormat` (SPEC-052) to extend with `sarif`;
    `run_lint`.
  - `docs/research/proj-002-design-lint.md` §Output — SARIF positioned as the second tier.
  - `CHANGELOG.md` (Keep a Changelog; the `[Unreleased]` section) + `RELEASING.md` (the tag/publish
    checklist — the maintainer's outward step) for the 0.4.0 cut.

## Outputs

- **Files modified:** `src/lint/report.rs` — `write_sarif`; `src/lint/mod.rs` — re-export it;
  `src/cli/mod.rs` — add `sarif` to `--format`. `README.md` — a SARIF/code-scanning note in the CI
  section. `CHANGELOG.md` + `Cargo.toml` — the 0.4.0 cut (staged, untagged).
- **New exports:** `pub fn write_sarif(&LintOutcome, version: &str, base: Option<&Path>, &mut impl
  Write) -> io::Result<()>`.
- **Schema:** SARIF 2.1.0 — `{version:"2.1.0", $schema, runs:[{tool:{driver:{name:"crustyimg",
  informationUri, version, rules:[{id, defaultConfiguration:{level}, helpUri}]}}, results:[{ruleId,
  level, message:{text}, locations:[{physicalLocation:{artifactLocation:{uri}}}]}]}]}`. Severity →
  SARIF level: `Error→error`, `Warn→warning`, `Info→note`.
- **Database changes:** none.

## Acceptance Criteria

- [ ] `lint --format sarif` emits one valid SARIF 2.1.0 object — a single `run`, the `crustyimg` tool
  driver (name + informationUri + version), a `rules[]` catalog entry per rule referenced by the
  findings (id + default level), and one `result` per finding (ruleId + level + message + a file
  location). Hand-rolled (**no `serde_json`**; `just deny` unchanged).
- [ ] **Paths are code-scanning-usable:** result `artifactLocation.uri` is relativized to a `base`
  (the cwd) when given and forward-slashed, so GitHub anchors findings to repo-relative files (not
  the absolute canonicalized path). `base = None` leaves paths as-is (for the golden).
- [ ] **Deterministic + golden-testable:** findings pre-sorted `(path, severity, rule)`, rules sorted
  by id, strings `escape_json`-escaped, `version` is a passed parameter (not the live crate version,
  so the golden is version-independent); a synthetic-`LintOutcome` golden asserts the exact string.
- [ ] The output format never changes the exit code (`sarif` still exits `0`/`7` per the gate,
  DEC-025); `human` stays the default.
- [ ] **0.4.0 staged:** `Cargo.toml` version → `0.4.0`; `CHANGELOG.md` `[Unreleased]` → a `[0.4.0]`
  section announcing `lint` (command, config, JSON, rules, SARIF) + the on-ramp (setup-crustyimg /
  crustyimg-action / pre-commit / just lint-images). **Untagged** — the maintainer tags `v0.4.0`.
- [ ] No new dependency; every existing test stays green; the 3-OS matrix + lean + msrv + deny green.

## Failing Tests

- **`src/lint/report.rs` (unit tests — pure over a synthetic `LintOutcome`)**
  - `"sarif golden: a fixed finding set renders an exact, byte-stable SARIF 2.1.0 string"` (version
    pinned as a literal; forward-slash synthetic paths).
  - `"sarif determinism: two renders are byte-identical"`.
  - `"sarif severity → level mapping (error/warning/note); clean outcome → empty results"`.
  - `"sarif relativizes a location uri against a base dir (and forward-slashes it)"`.
- **`tests/lint.rs` (integration)**
  - `"lint --format sarif on a GPS-leaking tree emits a SARIF result with ruleId
    privacy/gps-metadata-leak and level error"`.
  - `"--format sarif and --format human produce the same exit code"`.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply
- `DEC-050` — the report is part of the pinned stability surface; DEC-050's Revisit-when already
  anticipated a SARIF tier. Rule ids are the SARIF `ruleId`s.
- `DEC-049` — the hand-rolled-JSON + synthetic-golden technique to copy.
- `DEC-025` — the exit code is unchanged by output format.

### Constraints that apply
- `no-new-top-level-deps-without-decision` — hand-rolled SARIF, matching `write_json`; no serde_json.
- `ergonomic-defaults` — `human` is the default; `sarif` is opt-in for code-scanning.
- `test-before-implementation` — the golden + determinism + mapping tests are the contract.

### Prior related work
- `SPEC-052` (shipped) — `write_json` + `escape_json` + the synthetic-golden pattern.
- `SPEC-057` (shipped) — the two Actions; the 0.4.0 notes announce them.

### Out of scope (for this spec specifically)
- Tagging `v0.4.0` / publishing (cargo-dist, crates.io, Homebrew) — the maintainer's outward step
  (RELEASING.md); this spec stages version + CHANGELOG only.
- Tagging the Action repos `v1` — the maintainer's step (SPEC-057 noted it).

## Notes for the Implementer

- Copy SPEC-052's discipline verbatim: sort first, `escape_json`, a fixed `version` param, no
  wall-clock — so the golden is stable across the 3-OS CI. Build the `rules[]` from the distinct
  ruleIds in the findings, each with its default level looked up from `default_rules()`.
- Relativize the location uri: `path.strip_prefix(base)` when `base` is `Some`, then replace `\` with
  `/`. The CLI passes `base = Some(cwd)`; the golden passes `None` with synthetic `img/…` paths.
- For the cut, `CHANGELOG.md` `[Unreleased]` accumulates nothing lint-related yet — move the whole
  lint + Actions story into `[0.4.0] - <date>` and leave a fresh empty `[Unreleased]`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-056-lint-sarif`
- **PR (if applicable):** (opened after green local gates)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - None — SARIF was anticipated by DEC-050; it follows the SPEC-052/DEC-049 hand-rolled-JSON
    precedent. No new DEC.
- **Deviations from spec:**
  - None of substance. `write_sarif` takes `(outcome, version, base, out)` exactly as specced; the
    golden pins a literal `version` so it's independent of the 0.4.0 bump. The 0.4.0 cut (Cargo.toml
    `0.3.1 → 0.4.0` + CHANGELOG `[0.4.0]` + backfilled compare-links; Cargo.lock refreshed) is
    **staged in this same PR** (untagged) rather than a separate chore — it's all SPEC-056.
- **Follow-up work identified:**
  - **Maintainer's outward step:** tag `v0.4.0` (cargo-dist publishes → crates.io / Homebrew /
    Release binaries; this is what makes `lint` + the two Actions usable by real consumers). Then tag
    the Action repos `v1` (per SPEC-057). Nothing else in STAGE-015.
  - Minor (already noted in SPEC-057): lint finding paths are absolute unless relativized — the SARIF
    writer relativizes to cwd (`base`), so code-scanning anchors are repo-relative in CI; the human/
    JSON reports still show absolute paths.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** — Nothing. SPEC-052's `write_json` was a
   near-exact template; the only new wrinkle (relativizing the location uri for code-scanning) was
   called out in the spec's Notes and unit-tested directly.
2. **Was there a constraint or decision that should have been listed but wasn't?** — No. Parameterizing
   `version` (rather than reading `crate::version()` inside the writer) was the key move that kept the
   golden stable across the 0.4.0 bump — worth repeating for any versioned machine-readable output.
3. **If you did this task again, what would you do differently?** — Nothing. Deriving `rules[]` from
   the distinct referenced rule ids (with default levels looked up from the catalog) kept the golden
   small and the SARIF valid without expanding the `Rule` trait.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?** — Nothing. Parameterizing `version` in `write_sarif`
   (instead of reading `crate::version()` inside it) let the golden survive the same-PR 0.4.0 bump —
   the one non-obvious bit, and the right call.
2. **Does any template, constraint, or decision need updating?** — No. `crustyimg.lint/v1` (JSON) and
   SARIF 2.1.0 are both concrete now; DEC-050's stability surface holds. Bundling the release-cut into
   the feature PR worked cleanly (Cargo.lock refresh + backfilled compare-links were the only fiddly
   parts) — fine to repeat for a small wave-closing spec.
3. **Is there a follow-up spec I should write now before I forget?** — No new spec. **STAGE-015 is
   complete (2/2).** The only remaining PROJ-004 stage is **STAGE-014 (engine-backed rules)**, still
   `proposed` and needing a framing pass. The maintainer's outward steps: tag `v0.4.0` (publishes the
   whole lint wave + unblocks the two Actions for real consumers), then tag the Action repos `v1`.
