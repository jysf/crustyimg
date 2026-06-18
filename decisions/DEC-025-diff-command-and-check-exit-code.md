---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-025
  type: decision                     # decision | analysis | recommendation | observation
  confidence: 0.80
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-17
supersedes: null
superseded_by: null

affected_scope:
  - src/cli/mod.rs
  - docs/api-contract.md

tags:
  - cli
  - diff
  - perceptual-quality
  - ci-gate
  - exit-codes
---

# DEC-025: diff command shape + a dedicated "check not satisfied" exit code (7)

## Decision

`crustyimg diff <a> <b>` prints the SSIMULACRA2 score of `b` relative to `a`
(reusing `crate::quality::score`); `--fail-under <N>` turns it into a CI
visual-regression gate that exits with a **new dedicated code 7 ("a check/gate was
not satisfied")** when the score is below `N`. Comparing images of different
dimensions is a **usage error (exit 2)** — not an implicit resize. `--json` emits a
hand-rolled machine-readable result. The **visual-diff heatmap image is deferred**
to a follow-up spec; v1 `diff` is score + gate + json only.

## Context

STAGE-009 needs the verification half of the differentiator: a way to prove a
processed image is still good. The SSIMULACRA2 metric already exists
(`crate::quality::score`, DEC-019); exposing it as `diff` is almost pure reuse. The
design questions this DEC settles:

1. What exit code signals "the gate failed" vs "the tool errored"? The api-contract
   (DEC-007) defined codes 0–6; none means "computed fine, but the checked
   condition was not met." A CI gate MUST be distinguishable from a crash/decode
   error, or scripts can't tell "regression detected" from "couldn't run."
2. What happens when the two images differ in dimensions (SSIMULACRA2 requires equal
   dimensions)?
3. Is the visual-diff heatmap in v1?

Constraints: `ergonomic-defaults`; reuse the metric, don't reinvent; keep the
exit-code mapping total and single-sourced (DEC-007).

## Alternatives Considered

- **Option A: gate failure reuses exit 1 (generic runtime error).**
  - What it is: score < `N` returns the existing generic-error code.
  - Why rejected: collides with real failures (a decode error also exits 1). A CI
    job can't distinguish "image regressed below threshold" from "image failed to
    load." For a gate whose entire job is to be machine-interpreted, that ambiguity
    defeats the purpose.

- **Option B: include the visual-diff heatmap in v1.**
  - What it is: also write a highlighted pixel-diff image.
  - Why rejected (for v1, not forever): it is the only genuinely new pixel code and
    carries open design questions (colormap, amplification factor, output format)
    that would expand and slow the spec. The score + CI gate is self-contained,
    higher-value, and ships clean now; the heatmap is an additive follow-up.

- **Option C (chosen): score + `--fail-under` gate with a new exit code 7;
  dimension-mismatch = exit 2; heatmap deferred.**
  - What it is: extend the exit-code contract with **7 = "a check/gate was not
    satisfied"**, used by `diff --fail-under` now and reusable by the future EXIF
    audit-linter; treat mismatched dimensions as a usage error; ship the number +
    gate + `--json`, defer the heatmap.
  - Why selected: gives CI a distinguishable, reusable signal; keeps the comparison
    well-defined (equal dimensions only); reuses the metric with no new dependency;
    and lands a complete, useful tool now while leaving the visualization as a clean
    follow-up.

## Consequences

- **Positive:** a one-line CI visual-regression gate (`crustyimg diff orig.png
  new.png --fail-under 90`); a distinguishable exit code that the EXIF audit-linter
  will reuse; no new dependency; pure reuse of `crate::quality::score`.
- **Negative:** extends the api-contract exit-code set from 0–6 to 0–7 (every place
  that documents/maps exit codes must include 7, and `exit_code_mapping_is_total`
  must assert it). A small, deliberate contract growth.
- **Negative:** no visual heatmap yet — users who want to *see* where images differ
  must wait for the follow-up.
- **Neutral:** different-dimension inputs are rejected rather than auto-resized; a
  future enhancement could offer an opt-in resize-to-compare mode.

## Validation

- Right if: `diff --fail-under` is dropped into CI and reliably distinguishes a
  perceptual regression (exit 7) from a broken run (exit 1/3/4), and the audit-linter
  later reuses exit 7 without redefining it.
- Revisit when: the visual-diff heatmap spec is written (it will reference this DEC);
  or if a second "check" tool wants richer gate semantics (then generalize the exit-7
  meaning rather than adding more codes).

## References

- Related specs: SPEC-023 (this command), SPEC-016 (introduced `quality::score`),
  SPEC-022 (`optimize`, the prior STAGE-009 command)
- Related decisions: DEC-019 (SSIMULACRA2 metric), DEC-007 (typed errors +
  single-sourced exit-code mapping)
- External docs: `docs/api-contract.md` (Exit Codes table — code 7 added here)
