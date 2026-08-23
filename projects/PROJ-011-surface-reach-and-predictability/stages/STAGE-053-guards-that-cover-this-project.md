---
stage:
  id: STAGE-053
  status: proposed
  priority: high
  target_complete: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-08-23
shipped_at: null

value_contribution:
  advances: >
    Not the thesis — the guards that have to work WHILE the thesis is delivered.
    PROJ-011 ships byte-changing code to shipped verbs; two of this repo's gates
    cannot currently see it.
  delivers:
    - "A wasm32 CI leg, so byte-changing format work is tested on the target that shares the engine"
    - "A cost gate that reports what it checked, so it cannot pass having checked nothing"
  explicitly_does_not:
    - "Take on the rest of PROJ-013's instrument backlog — only what this project's own changes need"
---

# STAGE-053: Guards That Cover This Project

## What This Stage Is

⚠ **Pulled forward from PROJ-013 on 2026-08-23. Its original framing overstated the urgency, and
this is the corrected version** — the two items are real but they are **not equally urgent, and
neither blocks STAGE-049.**

### 1 — `just wasm-test` and `wasm-check` never run in CI

**What is true:** no CI job runs either recipe. **What is NOT true — and the first draft of this
stage implied it — is that wasm is untested.** `pages.yml`'s *"build + browser smoke"* job builds
for `wasm32` via `just demo-build` **and** drives the real engine in headless Chrome via
`just demo-smoke` (SPEC-077/078: init over HTTP, file picker, convert, decode the output with an
independent parser). **So there is real coverage — through the demo, of one path.** What is missing
is the assertions in `just wasm-test`.

⚠ **The connection to PROJ-011 is narrower than first claimed, and it is to STAGE-050, not
STAGE-049.** `wasm::transform(input, recipe_toml, out_format)` (`src/wasm.rs:171`) takes the output
format as its **own argument** and parses it itself — it does not go through the CLI's format
resolution, so **SPEC-126 probably does not touch it at all.**
⚡ **The real interaction is a design question STAGE-050 must answer:** if `Recipe` gains a `format`
field, `wasm::transform` has **two sources of truth for format** — the recipe's and its own
parameter. Which wins? That needs deciding in STAGE-050 regardless of whether this CI leg exists.
📌 Independently requested by two external review batches, which is why it is worth doing —
corroboration, not urgency.

### 2 — `cost-audit` reports success having checked nothing

`scripts/cost-audit.sh:23` scopes to the active project. PROJ-011 has zero shipped specs, so it
passes trivially, with a message **identical** to the one it prints after checking 18.

⚠ **Honest severity: this is hygiene, not a live hazard.** It becomes non-vacuous the moment
PROJ-011 ships its first spec, and PROJ-010's 18 all carry correct cost today. The real defect is
that **the gate cannot say what it covered**, so it went blind silently and could again. **Fix the
reporting, not just the scope** — a gate that names its coverage cannot go vacuous unnoticed.

### So why is this a stage at all?

Because both are guards, both are small, and **the wasm one has a design question inside it that
STAGE-050 needs answered anyway.** ⚠ **Do not treat this as blocking STAGE-049** — the earlier
"do this before STAGE-049 merges" was wrong and is withdrawn.

## Spec Backlog

- [ ] (chore) — **A wasm32 CI leg** running `just wasm-check` + `just wasm-test`. Currently filed
  as STAGE-038 item #8 and **moved here**, because it is the same thesis as this stage rather
  than housekeeping: a guard that does not run. The runner needs the `wasm32-unknown-unknown`
  target and `wasm-bindgen-test-runner` — [[probe-load-bearing-crates-at-design]] applies to the
  test *runner* for a new target. Complexity **S–M**.

- [ ] (not yet written) — [S] ⚡ **`cost-audit` scopes to the ACTIVE project, so activating a new
  one silently takes the gate out of service — and its message cannot tell you.** Found 2026-08-23,
  the moment PROJ-011 was activated. `scripts/cost-audit.sh:23` does `project=$(get_active_project)`.

  **Driven, with a positive control:**

  | project | shipped specs | result | message |
  |---|---:|---|---|
  | PROJ-010 (via `ACTIVE_PROJECT=`) | **18** | exit 0 | *"✓ all shipped specs have build/verify cost recorded"* |
  | PROJ-011 (now active) | **0** | exit 0 | **identical message** |

  **The output cannot distinguish "checked 18 and all passed" from "checked nothing."** This is the
  repo's own [[a-harness-that-exercises-nothing-reports-green]] lesson, in the gate that enforces
  constraint `cost-captured-per-cycle` and backs the CI job `cost-data`.
  ⚠ **Not currently wrong** — PROJ-010's 18 specs do all carry cost — but the gate no longer looks
  at them, and nothing announces that.
  **Two fixes, and the second matters more than the first:** audit **every** project rather than the
  active one; and **report the count checked**, so a zero is visible. A gate that says how much it
  checked cannot go vacuous unnoticed.
  📌 ⚠ **Eleven other scripts call `get_active_project`** (`weekly-review`, `report_daily`,
  `report_weekly`, `roadmap`, `backlog`, `status`, `info`, `lifetime-report`, `new-spec`,
  `new-stage`, `_lib`). Scoping is *correct* for most of them — `new-spec` should target the active
  project. **Audit which ones are reporting/gating (should span projects) versus authoring (should
  not).** That distinction has never been drawn.

**Count:** 0 shipped / 0 active / **2 pending** — re-derive with a grep you just ran.

## Dependencies

### Depends on
- Nothing. Both are small and independent of STAGE-049/050.

### Enables
- STAGE-050's `Recipe`-gains-a-format decision, which must resolve `wasm::transform`'s two sources
  of truth whether or not this stage ships first.
- A cost gate that reports its own coverage.
⚠ **Blocks nothing.** An earlier version of this file said to do it before STAGE-049 merges; that
was overstated and is withdrawn.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
