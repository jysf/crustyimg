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

⚠ **Pulled forward from PROJ-013 on 2026-08-23 because PROJ-011 needs them, not because they share
its thesis.** Both are guards that cannot currently see the work this project is about to ship.

**PROJ-011 changes output-format resolution on shipped verbs and carries a lockfile migration.**
Two gates that should catch a mistake there are blind:

- **No CI leg runs `just wasm-test`.** `wasm::transform` is a real entry point sharing the same
  engine, and SPEC-118's own matrix names it as a dimension. Shipping byte-changing format work
  with no wasm leg is a gap **in this project specifically**, not a general nice-to-have.
  📌 Independently requested by **two external review batches** — corroboration, not just a wish.
- **`cost-audit` reports success having checked nothing.** It scopes to the active project;
  PROJ-011 has zero shipped specs, so it passes trivially with a message identical to the one it
  prints after checking 18. **This project will ship specs into that gate.**

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
- PROJ-011's own changes being caught by gates that can see them. ⚠ **Do this before STAGE-049
  merges**, or the first byte-changing spec ships past both blind spots.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
