---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-052
  type: story
  cycle: design
  blocked: false
  priority: high
  complexity: S

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
  decisions: [DEC-050, DEC-025]
  constraints: [no-new-top-level-deps-without-decision, ergonomic-defaults, test-before-implementation]
  related_specs: [SPEC-050, SPEC-049]

value_link: >
  Makes lint findings machine-readable for CI tooling — a stable, hand-rolled JSON
  report (no new dependency), plus the human report polish (runnable-fix line + a
  potential-savings summary).

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-052: `lint --format json` report + human polish

## Context

SPEC-050 prints findings for humans. CI tooling needs them structured. This spec adds
`lint --format human|json` — a **hand-rolled** JSON report (matching the existing
`write_json`/`write_diff_json` pattern, so **no `serde_json` runtime dependency**), and polishes
the human report (the fix line is always a runnable `crustyimg` command; a summary with total
potential savings). The report schema is part of DEC-050's stability surface. This mirrors exactly
how SPEC-049 shipped `--explain=json` — same technique, same determinism discipline.

## Goal

Add `lint --format json` emitting a stable, single-object hand-rolled JSON report
(`{schema, findings[], summary}`) and refine `--format human` (runnable-fix line + savings
summary), deterministic and golden-testable, adding no dependency.

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — `escape_json` (`:1130`) + `write_json`/`write_diff_json`, the hand-rolled
    JSON writers to match (no `serde_json`).
  - `src/analysis/decide.rs` (SPEC-049) — `ExplainTrace::write_json`, the exact precedent for a
    deterministic hand-rolled report + a synthetic-input golden test.
  - `src/lint/mod.rs` (SPEC-050) — `Finding`/`Severity`/`LintOutcome` this serializes.
  - `docs/research/proj-002-design-lint.md` §"Output" — the report shape.

## Outputs

- **Files modified:** `src/lint/` — a `report.rs` (or in `mod.rs`) with `write_json(outcome, w)` +
  the human renderer; `src/cli/mod.rs` — the `--format human|json` flag.
- **New exports:** `pub fn write_json(&LintOutcome, &mut impl Write) -> io::Result<()>`;
  `pub fn render_human(&LintOutcome, &mut impl Write) -> io::Result<()>`.
- **Schema:** `{"schema":"crustyimg.lint/v1","findings":[{"file","rule","severity","message",
  "fix","bytes_saved"?}],"summary":{"files_scanned","errors","warnings","infos",
  "potential_bytes_saved","passed"}}`.
- **Database changes:** none.

## Acceptance Criteria

- [ ] `lint --format json` emits one JSON object with the schema above, hand-rolled (no
  `serde_json`; `just deny` unchanged); `--format human` (default) is unchanged behaviorally from
  SPEC-050 plus the polish below.
- [ ] The human report groups findings by file (eslint/ruff style), sorts by severity, prints a
  **runnable `crustyimg` fix** per finding, and ends with a summary (counts + total potential
  savings).
- [ ] The JSON is **deterministic and golden-testable**: findings sorted (path, severity, rule),
  no absolute paths beyond the input paths given, no timestamps; a synthetic-`LintOutcome` golden
  test asserts the exact JSON string (SPEC-049's technique).
- [ ] `summary.passed` reflects the exit gate (SPEC-050/051): `true` iff no `Error` and warnings
  within `--max-warnings`; the exit code is unchanged by the output format.
- [ ] No panic on any outcome (including zero findings → `passed:true`, empty `findings`); all
  existing tests green.

## Failing Tests

- **`src/lint/report.rs` (unit tests — pure over a synthetic `LintOutcome`)**
  - `"json golden: a fixed finding set renders an exact, byte-stable JSON string"`.
  - `"json determinism: two renders are byte-identical"`.
  - `"clean outcome → passed:true, empty findings array"`.
  - `"human render groups by file, shows the runnable fix + savings summary"`.
- **`tests/lint.rs` (integration)**
  - `"lint --format json on a GPS-leaking tree emits the finding with fix 'clean --gps' and
    passed:false"`.
  - `"--format human and --format json produce the same exit code"`.

## Implementation Context

### Decisions that apply
- `DEC-050` — the report is part of the pinned stability surface (schema id `crustyimg.lint/v1`).
- `DEC-025` — `passed`/exit alignment.

### Constraints that apply
- `no-new-top-level-deps-without-decision` — hand-rolled JSON, matching `write_diff_json`; no
  `serde_json`.
- `ergonomic-defaults` — `human` is the default; `json` is opt-in for tooling.
- `test-before-implementation` — the golden + determinism tests are the contract.

### Prior related work
- `SPEC-049` (shipped) — `ExplainTrace::write_json`: the exact hand-rolled-JSON + synthetic-golden
  pattern to copy. `SPEC-050` — the `Finding`/`LintOutcome` types.

### Out of scope (for this spec specifically)
- SARIF output — STAGE-015 (SPEC-056). `bytes_saved` is populated only for rules that compute it
  (the engine-backed rules, STAGE-014); here it's `null`/absent for the shipped-capability rules.

## Notes for the Implementer

- Copy SPEC-049's discipline verbatim: sort first, escape via `escape_json`, keep floats/ints exact,
  no wall-clock — so the golden is stable across the 3-OS CI (make the golden a *synthetic*
  `LintOutcome`, not real-tree output whose paths vary).

---

## Build Completion

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:** <DEC-NNN or none>
- **Deviations from spec:** <list>
- **Follow-up work identified:** <list>

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?** — <answer>
3. **If you did this task again, what would you do differently?** — <answer>

---

## Reflection (Ship)

1. **What would I do differently next time?** — <answer>
2. **Does any template, constraint, or decision need updating?** — <answer>
3. **Is there a follow-up spec I should write now before I forget?** — <answer>
