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

- [ ] (not yet written) — [M] ⚡ **Every cost figure this repo has recorded is wrong, and most are
  ~2× overstated.** Found by SPEC-127's verify 2026-09-05, independently reproduced by the
  orchestrator on a third transcript.

  **The mechanism.** Claude Code writes **one JSONL line per content block**, not per API call.
  Lines sharing a `.message.id` carry **identical** `input` / `cache_creation` / `cache_read`;
  only `output_tokens` grows across them (early lines are in-flight snapshots, the last is final).
  So the naive "sum every line with `.message.usage`" method **double-counts the three static
  fields on every multi-block call**, and the inflation factor is that session's mean
  lines-per-call.

  **The correct method:** dedup by `.message.id`; take `input`/`cache_creation`/`cache_read` from
  any line of the group; take **max** (= last) `output_tokens`.

  | figure | recorded | correct | error |
  |---|---:|---:|---|
  | SPEC-123 verify | $14.16 | $5.29 | 2.68× over |
  | SPEC-126 build | $38.16 | $18.02 | 2.12× over |
  | SPEC-126 verify | $15.64 | $9.18 | 1.70× over |
  | SPEC-126 re-approve | $15.02 | $5.96 | 2.52× over |
  | **SPEC-126 `cost.totals`** | **$68.82** | **$33.16** | **2.08× over** |
  | SPEC-127 build | $38.49 | $39.63 | 0.97× **under** |

  SPEC-127's build is the only cycle that deduped at all — it kept the **first** line's output
  instead of the max, so it lands ~3 % low rather than ~2× high.

  ⚠ **Two hypotheses were tested and one was refuted, so do not re-litigate it.** `cache_read` is
  **per-call, not cumulative**: `cr[N] == cr[N−1] + cc[N−1]` holds exactly on 223/247 adjacent
  pairs of one orchestrator transcript (the rest differ by a small positive uncached tail), and
  the last call's `cache_read` is **148×** smaller than the session sum — cumulative would make
  those equal. Summing per-call `cache_read` **is** correct billing.

  📌 **`subagent_tokens` from the Agent tool result is a last-turn context snapshot, not a session
  bill** — 649,005 against a real 123.9M on SPEC-127's build, and within 0.8 % of that call's own
  context. SPEC-127's build was right to reject it. ⚠ There is a memory note claiming the reverse
  reading; this entry is the measured one.

  **Blast radius, which is why this is [M] and not [S]:** `cost.totals` on **18 shipped PROJ-010
  specs** plus SPEC-126, `just cost-audit`, the `cost-data` CI job, `just specs-by-stage`, and both
  reports. ⚠ **It also invalidates the two ratio-derived findings the parked cost-ledger design
  rests on** — "verify = 37 % of build" and "complexity barely predicts cost" — because those were
  computed across figures produced by *different* methods with *different* inflation factors, so
  they were never comparable to each other.

  **Do not silently restate the numbers.** Decide first whether shipped specs are corrected in
  place with a note, or left with an errata record — the figures are cited in brag entries and
  release notes that cannot be edited retroactively.

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
