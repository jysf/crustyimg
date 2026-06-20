---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-038
  type: decision
  confidence: 0.85
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

created_at: 2026-06-19
supersedes: null
superseded_by: null

affected_scope:
  - src/operation/mod.rs

tags:
  - security
  - hardening
  - resize
  - untrusted-input
  - resource-limits
---

# DEC-038: resize output-size limit — tighten for decode symmetry

## Decision

**Correction note:** the STAGE-006 threat-model verification pass found that the
resize upscale-bomb was **already defended** — `Resize::apply` has carried an
oversize cap since SPEC-010 (`MAX_EDGE = 50_000` per dimension + `MAX_AREA`
on total pixels, both "untrusted-input-hardening", rejecting before allocation).
So this DEC does **not** add a new guard; it **tightens the existing one** for
symmetry with the decode allocation limit:

- **Lower `MAX_AREA` from 256 Mpx → 128 Mpx** (`134_217_728` px). At RGBA8 that is
  a **512 MiB** output buffer — numerically the **same** cap as the decode
  allocation limit (DEC-034), so an image cannot be *resized* to a buffer larger
  than one that could be *decoded*. The two limits are now deliberately symmetric.
- `MAX_EDGE` (per-dimension sanity, 50 000 px) is kept unchanged.
- The single area check sits in `Resize::apply` after the `(tw, th)` match (the
  point all six modes converge to) and **before** any allocation, returning
  `OperationError::Apply { op: "resize", … }` (CLI exit 1). It covers the
  upscaling modes (`exact`/`percent`/`cover`/`fill`); `max`/`fit` never upscale.
  Enforcing at `apply` time (not `from_params`) is required because `percent`/
  `cover`/`fill` outputs depend on the *input* dimensions, known only at apply.

**Reject, do not clamp** — an over-cap resize is refused, not silently shrunk.
**No new constant is introduced** (an earlier draft of this spec added a redundant
`MAX_RESIZE_OUTPUT_BYTES`; it was removed in favor of tightening `MAX_AREA`, so
there is one cap, not three).

## Context

STAGE-006's recipe-validation work (SPEC-035, DEC-036) deferred **op-parameter
bounds** to the threat-model pass (SPEC-037), flagging `resize` as the most
impactful recipe-/CLI-borne resource vector. On inspection, the verification pass
found `resize` was **not** an open hole: SPEC-010 introduced `MAX_EDGE`/`MAX_AREA`
when the op was first built, so `resize exact 100000x100000` or `resize percent
1000000` was already rejected (`OperationError::Apply`) before allocation. The
only refinement worth making is **consistency**: the pre-existing area cap allowed
a ~1 GB output buffer, looser than the decode allocation cap (512 MiB, DEC-034).
Lowering `MAX_AREA` to the 512 MiB-equivalent makes "what can be resized to" and
"what can be decoded" share one ceiling — a clean invariant, and an honest outcome
of the verification (confirm the as-built defense, tighten only where it helps).

## Alternatives Considered

- **Clamp the output to the cap instead of rejecting** — rejected: silently
  resizing to a different size than asked is surprising; the safe, predictable
  behavior is to refuse an unsatisfiable request (consistent with DEC-034).
- **Add a new, separate byte cap on top of the existing `MAX_EDGE`/`MAX_AREA`**
  (the SPEC-037 build's first attempt) — rejected: three overlapping caps with
  different rationales is confusing. The existing `MAX_AREA` already bounds the
  allocation; tightening its value to the 512 MiB-equivalent achieves the
  decode-symmetry with ONE cap and no redundant code.
- **Leave `MAX_AREA` at 256 Mpx (~1 GB) and call it done** — defensible (resize is
  already bounded, not OOM-exploitable), but the 512 MiB symmetry with the decode
  cap is a cheap, clean invariant worth the one-line tightening.
- **Bound it in `from_params` (parse time)** — insufficient: `percent`/`cover`/
  `fill` outputs depend on the input size, unknown until `apply`. The apply-time
  check is the one place that covers every mode (which SPEC-010 already chose).

## Consequences

- **Positive:** A recipe or CLI `resize` cannot allocate an unbounded buffer; an
  over-cap request fails with a typed error (exit 1), never an OOM. Symmetric with
  the decode limit, so "what can be decoded" and "what can be resized to" share
  one ceiling. One guard covers all six modes.
- **Negative:** A *legitimately* huge resize (output buffer > 512 MiB, ≈ 11 600 ×
  11 600 RGBA) is refused — far above realistic web/thumbnail use; the intended
  hardening trade-off. A future `--max-pixels`-style override (DEC-034's planned
  follow-up) would cover both decode and resize.
- **Neutral:** `max`/`fit` are unaffected (never upscale); existing tests use tiny
  dimensions, well under the cap. No new dependency.

## Validation

Right if: `resize exact`/`percent`/`cover`/`fill` whose output buffer would exceed
512 MiB is rejected with `OperationError::Apply` (exit 1) before allocation —
through both a recipe and the CLI — while normal resizes and `max`/`fit` are
unchanged (SPEC-037 tests). Revisit if: a configurable `--max-pixels` override is
added (apply it to both decode and resize), or the cap proves too low for a real
use case.

## References

- Related specs: SPEC-037 (threat-model pass — bundles this fix); SPEC-035 /
  DEC-036 (recipe limits, which deferred op-param bounds to here); SPEC-010
  (the resize op)
- Related decisions: DEC-034 (decode allocation limit — the symmetric input
  bound, same 512 MiB cap), DEC-007 (typed errors)
- Constraints: `untrusted-input-hardening` (blocking), `no-unwrap-on-recoverable-paths`
