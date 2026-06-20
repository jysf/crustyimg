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

# DEC-038: resize output-size limit (upscale-bomb defense)

## Decision

Bound the **output** of the `resize` operation so a hostile request — via a
recipe (`apply --recipe`) or the CLI (`resize`/`edit`/`shrink`/`thumbnail`) —
cannot drive an enormous allocation. In `Resize::apply`, after the target
dimensions `(tw, th)` are computed (the single point all six modes converge to)
and **before** the output buffer is allocated, reject if the output would exceed
an allocation cap:

- **`tw as u64 * th as u64 * 4 > MAX_RESIZE_OUTPUT_BYTES`** → typed
  `OperationError::Apply { op: "resize", reason: … }` (CLI exit 1). `4` =
  bytes-per-RGBA8-pixel (the working buffer `Resize::apply` allocates).
- **`MAX_RESIZE_OUTPUT_BYTES = 512 MiB`** — numerically the **same** cap as the
  decode allocation limit (DEC-034), so an image cannot be *resized* past the
  size at which it could be *decoded*. The two limits are deliberately symmetric.

This covers the modes that can upscale — `exact`, `percent`, `cover`, `fill` —
and is a no-op for `max`/`fit` (which never upscale). It is enforced at
**`apply` time**, not in `from_params`, because the dangerous case (`percent`,
`cover`, `fill`) depends on the *input* dimensions, which only `apply` knows. The
input itself is already bounded by the decode limit (DEC-034); this closes the
*output* side.

**Reject, do not clamp.** An over-cap resize is refused with a typed error, not
silently shrunk — the request is unsatisfiable as written.

## Context

STAGE-006's recipe-validation work (SPEC-035, DEC-036) bounded recipe *size* and
*step count*, but explicitly deferred **op-parameter bounds**. The most impactful
recipe-borne (and CLI-borne) resource vector is `resize`: `Resize::from_params`
validates `width/height > 0` and `percent > 0` but has **no upper bound**, so
`resize exact 100000x100000` (≈40 GB) or `resize percent 1000000` is an
upscale-bomb. The decode limit (DEC-034) bounds inputs, not resize outputs. This
DEC closes that, the last concrete runtime-hardening gap before the STAGE-006
threat-model verification pass (SPEC-037).

## Alternatives Considered

- **Clamp the output to the cap instead of rejecting** — rejected: silently
  resizing to a different size than asked is surprising; the safe, predictable
  behavior is to refuse an unsatisfiable request (consistent with DEC-034).
- **Per-dimension cap (e.g. ≤ 65 535/side) instead of an allocation cap** —
  rejected as the primary guard: 65 535×65 535×4 ≈ 17 GB would still pass a
  per-dimension check, so the *allocation* cap is the meaningful bomb defense.
  (A per-side cap could be added too, but the byte cap subsumes the OOM risk.)
- **Bound it in `from_params` (parse time)** — insufficient: `percent`/`cover`/
  `fill` outputs depend on the input size, unknown until `apply`. The apply-time
  check is the one place that covers every mode.

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
