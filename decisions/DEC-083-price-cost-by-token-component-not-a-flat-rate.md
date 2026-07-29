---
insight:
  id: DEC-083
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent
    - operator

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-07-26
supersedes: null
superseded_by: null

affected_scope:
  - AGENTS.md
  - docs/cost-tracking.md
  - projects/_templates/prompts/cost-snippet.md
  - projects/**/specs/**

tags:
  - cost
  - methodology
  - measurement
---

# DEC-083: price cycle cost by token component, not by a flat rate

## Decision

`estimated_usd` prices each token component separately at the model's list anchors
(Opus $5/$25, Sonnet $3/$15 per MTok), with the standard cache multipliers —
`cache_creation` ×1.25 input, `cache_read` ×0.10 input — and records which anchors were used.

The previous rule — `tokens_total` × list rate, ~80/20 input/output, no cache discount — is
**withdrawn**.

Separately and relatedly: a metered cycle **measures its own cost from its session
transcript** and closes its return with a `## Cost readout` block. The orchestrator's job at
ship changes from *sourcing* the number to *checking* it.

## Context

The flat rule was reasonable when written: the harness reported one combined token metric
with no input/output split, so a stated-assumption estimate was the honest available option,
and it called itself order-of-magnitude.

Two things broke it.

**Cache reads came to dominate volume.** SPEC-109's build cycle measured **65,339,132 tokens,
of which 64,473,663 — 98.7% — were cache reads**, priced at 0.10× input. The flat rule
returns **$588**; pricing the components returns **$43.21**. That is a **14× overstatement** —
worse than the one order of magnitude the rule advertised, so it was not merely imprecise, it
was outside its own stated tolerance.

**The measurement instruction made a fabricated number the only compliant answer.** The
template said to leave `tokens_total` null because "the orchestrator fills the real number
from the Agent result's `subagent_tokens`". A cycle run interactively has no
`subagent_tokens`, and `just cost-audit` rejects a null on a metered cycle at ship. The two
rules together left inventing a plausible figure as the only way to satisfy both — and that
is what happened on SPEC-109's build before it was caught. A rule that makes fabrication the
compliant path is a defect in the rule.

Per-message `usage` is present in the session transcript regardless of dispatch mode, so the
cycle can always measure itself. Where a cycle *was* dispatched as a subagent, the
orchestrator now has two independently derived numbers to compare, which is strictly better
than one.

## Alternatives Considered

- **Keep the flat rule, widen its stated tolerance.** Rejected: a 14× band makes the figure
  useless for the thing it exists for (comparing cycles and specs).
- **Record tokens only, drop `estimated_usd`.** Rejected: the dollar figure is what makes the
  cost record legible to a human, and the repo publishes spend-per-spec.
- **Have the orchestrator price it at ship.** Rejected: the orchestrator does not have the
  component breakdown for a cycle it did not run. The cycle does.

## Consequences

- **Positive:** numbers are measured rather than estimated, carry a component breakdown, and
  are cross-checkable when the cycle ran as a subagent. Fabrication is no longer the
  compliant path.
- **Negative:** entries before and after 2026-07-26 are **in different units** and must not be
  summed naively. See below.
- **Neutral:** the per-cycle work is a transcript sum — cheap, but it is real work the cycle
  must not skip.

## What this means for the existing record

**317 non-zero `estimated_usd` entries exist, summing $897.98.** They were computed under the
withdrawn rule.

**How much they overstate is UNKNOWN and deliberately not asserted here.** It depends on
whether the `subagent_tokens` those entries were derived from counted cache reads at all. If
it did, they are inflated on the same order as the SPEC-109 measurement; if it counted only
non-cached tokens, they may be roughly right. **That question has not been investigated** —
it is filed in `docs/backlog.md`, not answered.

Until it is answered:

- Do **not** restate historical entries. A recomputation needs a per-component breakdown that
  old cycles most likely did not preserve, so it may be impossible in principle rather than
  merely unfinished.
- Do **not** quote the aggregate — including in a launch post, a README, or a benchmark note —
  without stating the methodology boundary. This is the concrete exposure: a spend-per-spec
  figure is exactly the kind of number that reads as precise and gets repeated.
- `just cost-audit` still passes, because it checks for *presence*, not method. That is not
  evidence the numbers agree.

## Validation

Right if: new entries carry a component breakdown, a stated `source`, and a `## Cost readout`
in the cycle's return; a subagent cycle's self-measured total agrees with `subagent_tokens`.
Revisit if: published rates change, cache multipliers change, or the harness starts reporting
a priced figure directly — at which point prefer the harness's number and record the delta.

## References

- Related specs: SPEC-109 (where both defects surfaced; its build entry is the first recorded
  under this rule and carries the departure note)
- Related decisions: none superseded — this replaces prose rules in `AGENTS.md` and
  `docs/cost-tracking.md` that were not previously carried by a decision record
- Related: `docs/backlog.md` — the open question about the 317 historical entries
