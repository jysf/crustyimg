---
source: external engineering review (batch 2)
received: 2026-08-21
scope: audit charter — 21 sections, correctness through documentation honesty
triaged_by: orchestrator session f20dabb9, 2026-08-21
status: triaged, not executed — this is stage-sized work
---

# External review batch 2 — an audit charter, and the §1 investigation

Unlike batch 1 this is **not a findings list**; it is a methodology prompt asking for
investigation, prioritisation and a report. It is well-shaped — it explicitly says *"It is
completely acceptable to say: I investigated this and found no actionable problem"* and *"Do not
manufacture P0/P1 issues"*. Treat it as a **stage charter**, not a backlog item.

**Nothing in it gates the 0.7.1 tag.** §1 is the only section that could have, and it was
investigated at intake — see below.

---

## §1 — optimizer monotonicity: NOT REPRODUCED in its serious form

**The claim to test, in the review's own words:** *"If the optimizer claims to satisfy a quality or
byte constraint, the returned artifact must actually satisfy that constraint"* — and it asks
whether binary search can skip a valid candidate or return a violating result.

**Status: partially confirmed. The correctness invariant holds; the optimality one does not.**

**Evidence — the invariant is enforced three independent ways, none of which depend on
monotonicity:**

1. **Structural, in `search_threshold` (`src/quality/mod.rs:192-260`).** `best` is assigned **only**
   inside `if accept(m)`. So `met_target: true` means that exact `(quality, score)` pair was probed
   and *did* satisfy the predicate. A non-monotonic curve cannot produce a `met_target: true` result
   that fails `accept` — the value is not inferred from the search's shape, it is recorded from a
   real probe.
2. **Winner selection filters on it** — `src/analysis/decide.rs:228`:
   `.filter(|&i| cands[i].met_target && cands[i].bytes < source_bytes)`, with a dedicated test
   `winner_unmet_target_excluded()` (`:704`). An unmet candidate cannot win.
3. **Probe/artifact byte parity** — the thing that would break the invariant *after* a correct
   search is the sink writing different bytes than the probe measured. That is the DEC-019 /
   DEC-068 / DEC-096 lockstep, now driven by
   `tests/avif_tile_pin.rs::both_encode_paths_set_the_thread_count`.

When nothing satisfies the constraint the search returns a fallback quality with
`met_target: false` — it reports the miss rather than hiding it.

**What IS real:** non-monotonicity costs **optimality**, not correctness. On a plateau, binary
search may return a satisfying-but-not-best quality. The repo has already observed this and written
it down — SPEC-125's `both_encode_paths_set_the_thread_count` comment notes that a flat fixture
*"is flat enough that many qualities tie on byte count (AVIF saturates well below 100 on near-solid
content), so the search can legitimately land on any member of that tie set."* That is precisely
the plateau §1 warns about, already characterised.

**Recommended action:** no algorithm change. **P2**: a test that pins the *contract* rather than the
outcome — for a mocked non-monotonic/plateau probe, assert `met_target == true ⟹ accept(score)`.
That protects the invariant against a future refactor of `search_threshold`, which is where the risk
actually lives. Do **not** replace the optimizer.

---

## Triage of the remaining sections

### Already done, or already filed — the corroboration is the value

Three sections independently land on things this repo found on its own. Two sources with no contact
agreeing is worth more than either alone:

- **§2/§3 resource limits.** *"Does every externally controllable expensive operation have an
  appropriate bound?"* — batch 1 asked for per-operation memory caps, and checking the code found
  a concrete gap: `check_caps` budgets `w * h * 4` (RGBA8) at decode
  (`src/image/avif.rs:216`), while **SPEC-122 moved resize to `F32x4` working buffers — 16
  bytes/pixel**. The decode-time cap under-estimates a resize's peak by **4×**, and it became true
  *this wave*. **Now two independent reviews point at this. Strongest item across both batches.**
- **§15 WASM/native parity.** No CI leg runs `just wasm-test` — already filed on STAGE-042 as a
  chore, and now asked for by both batches. **Third independent hit.**
- **§10 property-based testing.** Same as batch 1. The specific justification is local: SPEC-125's
  AC-6 passed green on an 8-bit-only corpus, so the criterion never exercised the path it guarded.
  Random valid configs would have generated >8-bit inputs.
- **§4/§17 determinism precision.** Largely **already done, and recently**: SPEC-123 and SPEC-124
  drew exactly the guaranteed / conditional / observed distinction for AVIF (DEC-094, DEC-096), and
  the one remaining overclaim — `lock.rs:124-129`'s *"`[env]` … same machine"* against an
  `env.target` of `{ARCH}-{OS}` — **is already filed on STAGE-042** as §17's ask nearly verbatim.

### Real gaps, not yet filed

- **§11 fuzzing reach.** Four decode targets exist (`avif`, `svg`, `heic`, `raw_preview`). **Recipes
  and pipelines are not fuzzed at all** — and a recipe is TOML from an untrusted file that drives an
  operation sequence, which is the highest-value unfuzzed surface in the tree.
- **§6 pipeline invariants** — *"are invariants encoded in the architecture, or merely assumed by
  individual operations?"* This wave is evidence the question is fair: SPEC-121 (colour type),
  SPEC-122 (linear light) and SPEC-125 (depth reporting) were each an invariant that no single place
  owned.
- **§14 concurrency/cache correctness** — TOCTOU and cache-corruption review. Not audited before.

### Weak or already answered

- **§7 abstraction complexity / §9 AI-code smells.** Legitimate questions, but stated as a sweep
  with no named instance. A sweep without a target is how a refactoring spree starts — which §21
  itself forbids. Ask for specific call sites before acting.
- **§18 benchmark methodology.** Explicitly says to preserve current methodology absent an obvious
  problem. No problem named. **No action.**
- **§19 "already good, preserve."** Accurate; nothing to do.

### Process note

§20/§21 ask for a Finding / Status / Severity / Evidence / Recommended-action report, then a
Changed / Tests / Not-changed / Remaining / Regression-risk summary. **That is this repo's verify
cycle**, which already emits a verdict plus a punch list and already distinguishes confirmed from
not-reproduced. The charter maps onto existing machinery rather than needing new machinery.
