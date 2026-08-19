# SPEC-122 — VERIFY prompt

Cycle: **verify**. New session, **read-only**, detached worktree. You did not build this.

**What it claims:** `Resize::apply` linearizes → resamples in `F32x4` → re-encodes, at the input's
own bit depth. All three SPEC-120 cases move to ~100 against the same regenerated independent
reference, and reverting returns them to **exactly** −63.85 / 70.45 / 84.45.

**PR #182**, branch `fix/spec-122-resize-resamples-in-linear-light`. DEC-095 **amended** (not a new
DEC). Cycle advanced to `verify`.

```
git worktree add --detach ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec122-verify <head>
```

**Make no commits.** Emit your `## Cost readout` and verdict in the return message — that is the
deliverable (AGENTS §13).

## Read in order

1. **The spec** — 10 ACs, 5 design calls, 4 failing tests, and `## Build Completion`.
2. **DEC-092** (the premise + the alpha refutation) and **DEC-095** as amended.
3. `src/operation/mod.rs` (`Resize::apply`), `scripts/spec120_linear_light.py`, the wasm size guard.

## Five specific things — the orchestrator has ruled on one, not the others

### 1 — ⚖ AC-6's upscale half: ALREADY RULED, confirm the reasoning holds

AC-6 said upscale and no-op resize are *"byte-identical to `main` where no resampling occurs."*
**The premise was wrong: an upscale IS a resample, and it was defective the same way**
(65.93 → 100.00, 89.16 → 98.44 against the same reference).

**Orchestrator ruling: the AC was imprecise; fixing upscale too is CORRECT and is not a deviation
to be undone.** Gating on direction would put a discontinuity at exactly 100% scale and has no
answer for `fill`/`cover`, where direction differs per axis. This is AGENTS §15's *"an acceptance
criterion may not transfer"*.

**Your job is not to re-rule it — it is to check the ruling's factual basis:** is the no-op half
genuinely met (byte-identical at the four colour types claimed), and is the upscale change an
**improvement** rather than an unmeasured behaviour change?

### 2 — ⚠ A CI guard was RELAXED because this change made it fire

The wasm bundle shrank **16.9%** (twelve unused pixel-type instantiations dropped by calling the
typed entry point), which turned CI red, and the build **moved the baseline**.

**A guard that gets relaxed whenever it fires stops being a guard.** The build says it checked the
floor still catches a missing AVIF encoder (lean build 20.4% below it) and fixed the guard's
message, which had asserted the wrong cause. **Drive both claims yourself.** Specifically: can the
floor still go red for the reason it exists? Force it.

### 3 — DEC-092 is now partly WRONG, and it is a shipped decision

AC-5 predicted a null and the build measured an **improvement**: translucent-edge error
**27/255 → 0**. DEC-092 read that residual as Lanczos ringing; the build says it was **8-bit
quantization inside `fir`'s premultiply round-trip**.

Confirm the new explanation, and check the correction actually **lands in DEC-092** (or an explicit
amendment) rather than only in this spec's prose. A shipped decision carrying a wrong mechanism is
the thing that misleads the next reader.

### 4 — The performance decomposition drives a follow-on decision

`resize` is **3.83× slower** (169 → 649 µs; 1.5–2.5× end to end). The build's decomposition —
**72% is the `F32x4` working type, not the transfer function** (the same pipeline with no transfer
function still costs 516 µs; swapping the `powf` encode recovers 7 µs of 479) — is what a
follow-on working-type decision will rest on. **Check that decomposition; it is load-bearing.**
AC-9 makes performance report-only, so this is not a gate.

### 5 — The reference must still be independent

**AC-4.** Regenerated with the same external tool (ImageMagick 7.1.2-29 Q16-HDRI, `-colorspace
RGB`), **not** substituted with crustyimg's own output — a fixture derived from the code under test
cannot fail [[fixtures-from-the-code-under-test-cannot-fail]]. Confirm the regeneration, not just
the numbers.

## Also check

- **AC-7's negative control** — reverting returns the three cases to **exactly** 70.45 / 84.45 /
  −63.85. The behavioural flip is the evidence, not a binary hash (AGENTS §15).
- **AC-3** — the positive control still fires on the synthetic worst case.
- **AC-1/AC-2** — both metrics reported per case. The build says they agree; confirm there is
  genuinely no disagreement being smoothed over.
- **AC-8** — the migration driven, and consistent with what SPEC-121's AC-8 found (the safety net
  fires on a version bump; its precondition is not met mid-wave).
- **SPEC-121's tests stay green** — `tests/colour_type_preservation.rs`. The bit-depth preservation
  it shipped is a regression guard on this change.
- **DEC-095 amended, not duplicated**; `affected_scope` still correct.
- **Decision drift:** `./scripts/decisions-audit.sh --changed main` — **pass the base ref.**
- **AC-10** — matrix clean; then **read the CI legs individually** at the true head.

## Guardrails

- **Read-only. No commits. Do not fix what you find.** Do not merge; do not bump the version.
- **⚡ NEVER POLL CI.** Background it and do not re-read its output while it runs:
  `gh pr checks 182 --watch --interval 30`. **Measured on this spec's build: ~$60 of $103.60 —
  well over half — went on CI polling**, because backgrounded watchers were repeatedly checked.
  Take your cost reading once, at the end.
- **Budget ~200 exchanges.** The build ran 608.
- macOS has no `timeout(1)`. A piped command reports the pipe's exit code — redirect and read `$?`.

## When you finish

1. No commits. 2. Emit `## Cost readout` (`cost-snippet.md`; price at the anchors
`.message.model` actually reports). 3. Verdict — ✅ / ⚠ PUNCH LIST / ❌, with item 2's guard ruling
stated explicitly.
