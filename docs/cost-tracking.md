# Cost tracking — how it works and how it's enforced

Every spec records its AI cost per cycle so reports can aggregate spend over time
(AGENTS.md §4). This doc is the operational reference, and the **fold-back plan**
for upstreaming the enforcement into the spec-driven template.

## The schema (per spec front-matter)

```yaml
cost:
  sessions:
    - cycle: build            # frame | design | build | verify | ship
      agent: claude-sonnet-4-6
      interface: claude-code  # claude-code | claude-ai | api | ollama | other
      tokens_total: 130653    # ONE combined number (the harness reports one)
      estimated_usd: 0.71     # order-of-magnitude (see rate note)
      duration_minutes: 22
      recorded_at: 2026-06-15
      notes: "…"
  totals:
    tokens_total: 201141      # sum of non-null sessions (use 0, never null)
    estimated_usd: 1.34
    session_count: 4
```

**`tokens_total` is the schema** — a single combined token count, because that is
what the harness surfaces (`subagent_tokens` in an `Agent` result, or `/cost` in
an interactive session). There is no reliable input/output split, so don't try to
record one. (The reporting lib also still sums legacy `tokens_input`/`tokens_output`
if present, for forward-compatibility.)

## Where the numbers come from

- **build / verify cycles** run as metered subagents — the orchestrator reads
  `subagent_tokens` + `duration_ms` straight from the `Agent` result and writes
  them at **ship**. (Interactive instead of subagent? run `/cost`.)
- **design / ship cycles** are orchestrator main-loop work with no clean per-cycle
  metering — leave their numerics `null` with a "main-loop, not separately
  metered" note.
- **`estimated_usd`** = `tokens_total × list rate` (Opus 4.8 $5/$25, Sonnet 4.6
  $3/$15 per MTok), applying a stated ~80/20 input/output mix at list rates with
  no cache discount. It is explicitly order-of-magnitude — note that in the entry.

## How it's enforced (so it can't silently go empty again)

It already went empty once: SPEC-001–013 shipped with all-null numerics because
documentation alone was skippable (and the build prompts even said "null
numerics"). Now it's mechanical — three layers:

1. **Rule** — AGENTS.md §4 (capture real numbers; null only for main-loop cycles)
   + constraint `cost-captured-per-cycle` in `guidance/constraints.yaml`.
2. **Check (the teeth)** — `just cost-audit` (`scripts/cost-audit.sh`) fails if any
   *shipped* spec lacks a positive `tokens_total` on its build/verify cycles. It
   runs in CI (the `cost-data` job) and is surfaced in `just status` ("Specs
   missing cost data"). `report-weekly` flags the same.
3. **No-regression** — the cost wording lives in
   `projects/_templates/prompts/cost-snippet.md`, so new cycle prompts don't
   re-introduce the "null numerics" line.

Grandfathered pre-process specs (SPEC-001–013, real numbers unrecoverable) are
skipped via `COST_AUDIT_GRANDFATHERED` in `scripts/_lib.sh`.

## Reports

`just report-weekly` aggregates cost by spec / cycle / interface, plus total,
avg-per-shipped-spec, top cost drivers, and the "shipped without cost data" flag.
`just status` shows the missing-cost list. The data is only as good as what's
recorded — it gets richer as specs ship under the enforced process.

---

## Fold-back into the spec-driven template

This enforcement is **generic** — every project on this template wants it. When
upstreaming to the template repo, port these files **verbatim**:

| File | What to upstream |
|---|---|
| `scripts/cost-audit.sh` | the whole script (generic) |
| `scripts/_lib.sh` | `sum_cost_tokens_for_spec` (now reads `tokens_total`), `is_grandfathered_cost`, `cycle_tokens_total`, `spec_missing_cost_cycles` |
| `scripts/status.sh` | the "Specs missing cost data" section |
| `scripts/report_weekly.sh` | the missing-cost predicate (null-numeric check, not entry-count) |
| `justfile` | the `cost-audit` recipe |
| `.github/workflows/ci.yml` | the `cost-data` job |
| `guidance/constraints.yaml` | the `cost-captured-per-cycle` constraint |
| `AGENTS.md` | §4 rewrite (no null loophole; `tokens_total` schema; capture mechanism) |
| `projects/_templates/spec.md` | the corrected `cost:` block comment |
| `projects/_templates/prompts/cost-snippet.md` | the whole file |
| `docs/cost-tracking.md` | this doc (drop the crustyimg-specific notes) |

**One project-specific value, not the template's:** the
`COST_AUDIT_GRANDFATHERED` list in `scripts/_lib.sh` (here: SPEC-001–013). In a
fresh template instance it must be **empty** — there is no pre-process history to
grandfather. Everything else is project-agnostic.

> Suggested upstream framing: this is the same pattern as the license gate
> (DEC-018) — a discipline that documentation couldn't keep, made mechanical with
> a `just` check + a CI job. The template should ship both gates on by default.
