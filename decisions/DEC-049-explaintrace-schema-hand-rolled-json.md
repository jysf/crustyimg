---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-049                        # stable, never reused
  type: decision                     # decision | analysis | recommendation | observation
  confidence: 0.75                   # 0.0 - 1.0, honest assessment
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002                       # the project during which this was decided
repo:
  id: crustyimg

created_at: 2026-07-05
supersedes: null
superseded_by: null

# Path globs this decision governs.
affected_scope:
  - src/quality/decide.rs
  - src/quality/explain.rs
  - src/cli/mod.rs

tags:
  - explain
  - trace-schema
  - json
  - no-serde-json
  - manifest-forward-compat
---

# DEC-049: `ExplainTrace` schema — a hand-rolled-JSON decision record, a forward-compatible subset of the planner/manifest schema

## Decision

The auto-decision is surfaced through a typed **`ExplainTrace`** record (input facts, feature
vector, `class`, profile/mode/target, the per-candidate array `{format, quality, score, bytes,
met}`, `winner_format`, `win_reason`, and `savings{in,out,percent}`). It has two renderers: a
concise **human view to stderr** (`--explain`, stdout stays pipe-clean) and a
**hand-rolled JSON** view (`--explain=json`) that matches the existing `write_json` /
`write_diff_json` writers and therefore adds **no `serde_json` runtime dependency**. The trace is
**deterministic** (no timestamps, no absolute paths in the metric portion) so it is
golden-testable. Its field set is intentionally a **forward-compatible subset** of the fuller
planner schema in `docs/research/proj-002-design-planner.md`: PROJ-003 (planner `objective`,
`warnings[]`, `met_goal`, per-candidate `scale_percent`/`excluded_reason`) and PROJ-005 (the
per-image manifest `optimization` field) **extend it additively**, with no breaking rename.

## Context

SPEC-048 prints a one-line summary by default (chosen format + savings %) but not *why* — and an
auto-decider that can't justify itself reads as a black box (the explicit PROJ-002 thesis risk:
"may read as a black box vs. Cloudinary `f_auto` if the `explain` trace isn't clear"). SPEC-049
makes the full decision auditable. The design questions this DEC settles:

1. Is the trace ad-hoc prints or a typed record?
2. How is the machine-readable form serialized — do we take a `serde_json` runtime dep?
3. How do we shape the schema so the planner and the manifest reuse it rather than fork it?

Constraints in play: `no-new-top-level-deps-without-decision` (the `diff` command already emits
machine JSON hand-rolled, DEC-025 — no `serde_json` in the runtime); `ergonomic-defaults` +
AGENTS §11 (diagnostics to stderr, explain off by default); `untrusted-input-hardening` (rendering
never panics). The trace is also the golden-test fixture for the decision engine and the seed of
the PROJ-005 manifest, so its stability matters beyond `optimize`.

## Alternatives Considered

- **Option A: ad-hoc `eprintln!` lines, no typed record.**
  - What it is: print the decision inline as the engine runs.
  - Why rejected: not machine-readable, not golden-testable, and impossible to reuse as the
    manifest field. The trace needs to be a value, not a side effect.

- **Option B: derive `serde::Serialize` and emit via `serde_json`.**
  - What it is: take a `serde_json` runtime dependency for the JSON view.
  - Why rejected (for v1): adds a runtime dependency the repo has so far avoided — `diff`/`lint`
    already hand-roll JSON (DEC-025) to keep the tree lean. Not worth a new default dep for one
    render path. Left as a **future, non-blocking** switch: if serde is later promoted for the
    manifest, moving `ExplainTrace` to `#[derive(Serialize)]` is additive.

- **Option C (chosen): typed `ExplainTrace` + hand-rolled JSON + planner/manifest-subset schema.**
  - What it is: a plain struct with `render_human` + `write_json` (hand-rolled, matching
    `write_diff_json`), field-named as a subset of the planner schema.
  - Why selected: zero new dependency, deterministic + golden-testable, reusable as the manifest
    field, and forward-compatible with the planner without a rename — the cheapest way to make the
    decision auditable and keep it that way across three projects.

## Consequences

- **Positive:** `optimize --explain` makes the moat legible ("smallest file, with the reasons
  shown") at zero dependency cost; the JSON is a stable regression fixture for the decision engine;
  PROJ-003 and PROJ-005 inherit the schema instead of inventing their own.
- **Negative:** hand-rolled JSON must match the existing writers' escaping / key-order / number
  formatting by hand (a golden fixture guards this); if the planner's schema grows a field we
  didn't anticipate, we extend the struct (additive, but a touch of churn).
- **Neutral:** the human trace is stderr-only by design; scripts consume `--explain=json`. The
  `class` label is the only user-facing leak of the otherwise-internal classification.

## Validation

- **Right if:** `--explain` clearly answers "why this format?", `--explain=json` byte-matches its
  golden fixture across platforms, `just deny` shows no new dependency, and PROJ-003/PROJ-005 reuse
  `ExplainTrace` without a breaking change.
- **Revisit when:** serde is promoted for the manifest (then switch to `#[derive(Serialize)]`); or
  the planner needs a field the subset can't hold additively (re-confirm the subset contract).

## References

- Related specs: SPEC-049 (this trace), SPEC-048 (the decision record it renders), PROJ-005
  manifest (future consumer)
- Related decisions: DEC-048 (the decision engine), DEC-025 (`diff` hand-rolled JSON precedent /
  exit-code contract), DEC-016 (byte-sync → truthful savings %), DEC-024 (optimize shape), DEC-047
  (the `class` label surfaced)
- External docs: `docs/research/proj-002-design-format-engine.md` §Explain trace,
  `docs/research/proj-002-design-planner.md` §Explain trace (the superset schema)
- Discussions: PROJ-002 framing session 2026-07-05
