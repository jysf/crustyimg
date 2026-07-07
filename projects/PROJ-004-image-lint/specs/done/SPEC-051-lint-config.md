---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-051
  type: story
  cycle: ship
  blocked: false
  priority: high
  complexity: M

project:
  id: PROJ-004
  stage: STAGE-013
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-06

references:
  decisions: [DEC-050, DEC-005]
  constraints: [no-new-top-level-deps-without-decision, ergonomic-defaults, untrusted-input-hardening, test-before-implementation]
  related_specs: [SPEC-050, SPEC-006]

value_link: >
  Makes `lint` tunable and quiet enough to leave on in CI — the config surface
  (select/ignore, per-rule severity, per-glob budgets, savings threshold) that
  turns a fixed rule set into a project-shaped one.

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
        Main-loop orchestrator (PROJ-004 framing session), not separately metered.
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 130000
      estimated_usd: 1.17
      duration_minutes: 38
      recorded_at: 2026-07-06
      notes: >
        ESTIMATE — autonomous merge-on-green run in the orchestrator main loop, NOT a metered
        subagent. Order-of-magnitude (~130k at Opus 4.8 ~80/20 ≈ $1.17). Built src/lint/config.rs
        (LintConfig + RawConfig serde split + discovery + effective_config/merge + validation),
        threaded config through run_lint (select/ignore/off filtering, per_file_ignores, severity
        override), plumbed budget/intended-width/savings-threshold onto LintTarget, added the
        LintFlags CLI surface (7 flags + savings-threshold parse). 8 config unit + 5 integration
        tests. PR #60.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 12000
      estimated_usd: 0.11
      duration_minutes: 3
      recorded_at: 2026-07-06
      notes: >
        ESTIMATE — same autonomous run; CI-driven verify, all matrix/feature/lean/msrv/deny jobs
        green on #60. Order-of-magnitude (~12k).
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-06
      notes: >
        Main-loop ship bookkeeping (reflection, cost totals, stage backlog, archive), not
        separately metered.
  totals:
    tokens_total: 142000
    estimated_usd: 1.28
    session_count: 4
---

# SPEC-051: `.crustyimg-lint.toml` config + severity/select CLI

## Context

SPEC-050 shipped `lint` with a fixed default rule set. Real trees need to tune it: turn rules off,
raise a warning to an error, set a byte budget per folder, and control what fails CI. This spec
adds the config surface — an auto-discovered `.crustyimg-lint.toml` plus the CLI flags — following
the conventions developers already know from ruff (`select`/`ignore`/`per-file-ignores`) and eslint
(per-rule severity). It also wires the `Warn`-under-`--max-warnings` exit behavior deferred from
SPEC-050. Config shape + discovery order + the savings-threshold defaults are fixed in **DEC-050**.

## Goal

Add `.crustyimg-lint.toml` (auto-discovered walking up to the repo/filesystem root) with
`select`/`ignore`, per-rule severity overrides, per-glob `[[budget]]`, `per-file-ignores`, and
savings-threshold defaults; plus `--config`/`--no-config`/`--select`/`--ignore`/`--max-warnings`/
`--max-intended-width`/`--savings-threshold` flags — resolving into an effective config the runner
applies, adding no dependency (reuse `toml`, DEC-005).

## Inputs

- **Files to read:**
  - `docs/research/proj-002-design-lint.md` §"Config + CLI + exit" — the discovery order,
    select/ignore semantics, per-glob budget, the 4 KiB / 10% savings default.
  - `src/lint/mod.rs` (SPEC-050) — the `Rule` registry + `Severity` the config overrides.
  - `src/recipe/mod.rs` — the shipped `toml` (de)serialization pattern (DEC-005) to mirror; no new
    dep.
  - `src/source/mod.rs` — glob matching for `per-file-ignores` / per-glob budgets (reuse the
    shipped glob, don't add one).

## Outputs

- **Files modified:** `src/lint/` — add `config.rs` (the `LintConfig` model + discovery + merge);
  `src/cli/mod.rs` — the new `lint` flags; the runner consumes the effective config.
- **New exports:** `pub struct LintConfig { select, ignore, per_rule_severity, budgets,
  per_file_ignores, savings_threshold, max_intended_width }`; `pub fn discover_config(start) ->
  Option<PathBuf>`; `pub fn effective_config(cli_flags, file) -> Result<LintConfig, CliError>`.
- **Database changes:** none.

## Acceptance Criteria

- [ ] `.crustyimg-lint.toml` is **auto-discovered** by walking up from the first input (or cwd) to
  the repo/filesystem root; the nearest one wins; `--config PATH` forces one; `--no-config` ignores
  discovery and uses defaults.
- [ ] `select`/`ignore` (rule-id prefixes, ruff-style) filter the active rule set; `--select`/
  `--ignore` CLI flags override/extend the file; an unknown rule id is a usage error (exit 2).
- [ ] Per-rule **severity override** (eslint-style: `error`/`warn`/`info`/`off`) changes a finding's
  severity — and thus the exit code; `off` disables the rule.
- [ ] Per-glob `[[budget]]` (`max_bytes`, `max_intended_width`) and `per_file_ignores` (glob → rule
  ids) apply by matching the shipped glob; these feed SPEC-053's size/dimension rules.
- [ ] `--max-warnings N`: `Warn` findings fail the gate (exit 7) only when their count exceeds `N`
  (default: warnings do **not** fail — matching SPEC-050); `Error` always fails; `Info` never.
- [ ] Savings-threshold defaults (`min_bytes = 4096`, `min_percent = 10`) are configurable
  (`--savings-threshold` / config) and exposed to the engine-backed rules (STAGE-014).
- [ ] Zero-config still works (no file ⇒ defaults); a malformed config is a clear usage error
  (exit 2), never a panic; `just deny` green (no new dep); all existing tests green.

## Failing Tests

- **`src/lint/config.rs` (unit tests)**
  - `"discovery walks up and picks the nearest config"` — nested dirs, config at an ancestor.
  - `"select/ignore filter rules; unknown id → usage error"`.
  - `"per-rule severity override changes severity (warn→error) and off disables"`.
  - `"per-file-ignores suppress a rule for a matching glob"`.
  - `"--max-warnings: warns over N ⇒ fail; at/under N ⇒ pass; errors always fail"`.
  - `"malformed toml → CliError::Usage (exit 2), no panic"`.
- **`tests/lint.rs` (integration)**
  - `"a per-glob byte budget from config drives a size finding"` (with SPEC-053's rule, or a stub).
  - `"--no-config ignores a present .crustyimg-lint.toml"`.

## Implementation Context

### Decisions that apply
- `DEC-050` — the config schema, discovery order, severity model, savings-threshold defaults.
- `DEC-005` — the shipped `toml` serialization pattern; reuse it, add no crate.

### Constraints that apply
- `no-new-top-level-deps-without-decision` — reuse `toml` + the shipped glob.
- `ergonomic-defaults` — zero-config is the common path; the file only tunes.
- `untrusted-input-hardening` — a malformed config is a typed usage error, never a panic.
- `test-before-implementation` — the tests above are the contract.

### Prior related work
- `SPEC-050` (this stage) — the rule registry + severity this configures.
- `SPEC-006` (shipped) — recipe TOML (de)serialization, the pattern to mirror.

### Out of scope (for this spec specifically)
- The rules that consume budgets/intended-width — SPEC-053. The JSON report — SPEC-052.

## Notes for the Implementer

- Precedence: CLI flags > discovered/`--config` file > built-in defaults. Merge into one
  `LintConfig` the runner reads; don't thread raw flags into rules.
- Reuse the shipped glob for `per_file_ignores` + budgets — do not add a globbing crate.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-051-lint-config`
- **PR (if applicable):** (opened after green local gates)
- **All acceptance criteria met?** yes (with one test reallocated — see deviations)
- **New decisions emitted:** None — DEC-050 fixed the config schema/discovery/severity/
  savings-threshold; the build followed it.
- **Deviations from spec:**
  - The integration test *"a per-glob byte budget from config drives a size finding"* is
    **deferred to SPEC-053**, where its consuming rule (`size/oversized-bytes`) lands. The spec
    hedged it ("with SPEC-053's rule, or a stub"), and injecting a throwaway stub rule into the
    real binary would pollute the public rule catalog. Instead, the **budget config is fully
    plumbed here** — `LintConfig::byte_budget_for`/`intended_width_for` (unit-tested:
    `per_glob_byte_budget_resolves_by_path`) and surfaced on `LintTarget`
    (`byte_budget()`/`intended_width()`/`savings_threshold()`) — so SPEC-053's rule is a pure
    consumer. The end-to-end budget→finding assertion moves to SPEC-053's test list.
  - Config integration is instead proven end-to-end through the binary with the *existing* rules:
    `off`/`--no-config`, `--ignore`, per-rule `warn` downgrade + `--max-warnings`, unknown-id and
    malformed-config usage errors (all in `tests/lint.rs`).
- **Follow-up work identified:**
  - SPEC-053 gains one integration test: a per-glob `[[budget]]` drives a `size/oversized-bytes`
    finding (the budget plumbing built here, now consumed).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** — Only the budget integration test's
   ordering (a size rule doesn't exist until SPEC-053). Resolved by plumbing + unit-testing budget
   resolution here and moving the end-to-end assertion to SPEC-053 (the spec's own "or a stub"
   hedge licensed this).
2. **Was there a constraint or decision that should have been listed but wasn't?** — No. The
   shipped `glob` crate's `Pattern::matches_path` covered `per_file_ignores`/budget globbing with
   no new dependency, exactly as the spec anticipated.
3. **If you did this task again, what would you do differently?** — Split the CLI `LintFlags` out
   from the start (I introduced it once the flag count grew); otherwise the RawConfig→LintConfig
   two-type split (serde-facing vs runtime) kept validation and enum parsing clean.

---

## Reflection (Ship)

1. **What would I do differently next time?** — Nothing structural. Splitting the on-disk
   `RawConfig` (serde, `deny_unknown_fields`) from the runtime `LintConfig` kept string→enum parsing
   and rule-id validation explicit, and giving `LintConfig` the query methods (`is_rule_active`,
   `severity_for`, `is_ignored_for_path`, `byte_budget_for`) let the runner stay a thin loop.
2. **Does any template, constraint, or decision need updating?** — No. DEC-050's config contract is
   now concrete. Worth noting for STAGE-014: the savings-threshold + intended-width are already
   resolved per-file on `LintTarget`, so the engine-backed "could be smaller" rules read them
   directly — no further config plumbing needed.
3. **Is there a follow-up spec I should write now before I forget?** — No new spec. SPEC-053 inherits
   one extra test (per-glob budget → `size/oversized-bytes` finding) since the budget config plumbing
   landed here; noted in this spec's Build Completion. SPEC-052 (JSON report) is next.
