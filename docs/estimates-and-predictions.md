# Estimates & Predictions

> A living bet on how long crustyimg's MVP takes — made 2026-06-14, right
> after STAGE-001 shipped. Half forecasting exercise, half fun. We log
> predictions here and grade them against reality as stages ship.

## How to read this — two clocks

1. **Engine speed** — how fast the multi-agent loop ships a spec when you're
   *actively* working. This is fast and fairly predictable.
2. **Calendar** — when *you* actually sit down. This is the real variable.
   "Maybe I stop tomorrow, who knows." Totally fine — the engine pauses with you.

So estimates are in **specs / active-hours** (engine), then mapped to calendar
dates under a few **session-cadence scenarios**.

## Baseline — what STAGE-001 actually cost

- **7 specs** (SPEC-001..007), scaffold → shipped across **2 calendar days**
  (2026-06-13 → 2026-06-14), in a handful of focused orchestration sessions.
- Per spec ≈ **~1 active hour** wall-clock (design on Opus + build on Sonnet +
  read-only verify on Opus + manual ship bookkeeping). Range ~0.5–2h.
- Engine throughput ≈ **~3 specs per focused (~2–3h) session**.

## Assumptions

- Pace holds (no major architecture surprises). The two low-confidence
  decisions — DEC-008 resize backend (0.75) and DEC-003 metadata lane (0.80)
  — are the most likely to add friction.
- "Active session" ≈ 2–3 hours ≈ ~3 specs.
- Build stays on Sonnet, design/verify on Opus, pause-at-merge cadence.

## Per-stage estimate (remaining work = 28 specs)

| Stage | Specs | Est. active hours | Risk | Why |
|---|---:|---:|---|---|
| 002 view & info | 2 | 2–3 | 🟢 low | first real commands; viuer already wired |
| 003 transform & output | 5 | 5–7 | 🟡 med | resize on `fast_image_resize` (DEC-008 @0.75) |
| 004 compose & metadata | 6 | 8–11 | 🔴 high | container metadata lane (DEC-003 @0.80) — hardest part of the whole project |
| 005 batch & recipes | 4 | 4–6 | 🟡 med | rayon + progress, name-templating |
| 006 hardening & security | 5 | 4–6 | 🟡 med | mostly tests + cargo-audit/CI |
| 007 release & distribution | 6 | 5–8 | 🟠 med-high | cross-platform release CI + Homebrew tap (external friction) |
| **Total remaining** | **28** | **~28–41 (≈33 central)** | | ≈ **9–10 focused sessions** |

## Calendar scenarios (from 2026-06-14)

| Cadence | Sessions/wk | MVP release ≈ |
|---|---|---|
| 🔥 Aggressive (≈daily) | 5 | **~2026-06-28** (2 wks) |
| 🙂 Realistic (≈2×/wk) | 2 | **~2026-07-19** (5 wks) |
| 🐢 Relaxed (≈1×/wk) | 1 | **~2026-08-23** (10 wks) |
| 👻 Sporadic / stalls | ? | open-ended — resume anytime |

## 🎯 The prediction (central guess)

**MVP (`v0.1.0`, brew-installable) ships ~2026-07-25** — assuming an
intermittent ~2×/week pace with a couple of "life happens" gaps.

- **Optimistic:** 2026-06-28 · **Pessimistic:** 2026-09-15
- **Confidence:** medium on engine-hours (~33h), low on the calendar date
  (entirely cadence-dependent).
- **Biggest things that could blow the estimate:** STAGE-004's metadata lane
  (img-parts/little_exif container surgery), STAGE-007's release CI + tap
  setup, and simply not sitting down.

## Tracking table — predicted vs actual (fill in as stages ship)

Predicted ship dates below use the 🙂 Realistic (2×/wk) scenario.

| Stage | Predicted ship | Actual ship | Pred. specs | Actual specs | Pred. active-h | Actual | Notes |
|---|---|---|---:|---:|---:|---:|---|
| 001 foundation | (baseline) | 2026-06-14 | 7 | 7 | ~7 | ~7 | shipped; the baseline |
| 002 view & info | ~2026-06-21 | — | 2 | — | 2–3 | — | |
| 003 transform & output | ~2026-06-30 | — | 5 | — | 5–7 | — | |
| 004 compose & metadata | ~2026-07-12 | — | 6 | — | 8–11 | — | metadata lane risk |
| 005 batch & recipes | ~2026-07-19 | — | 4 | — | 4–6 | — | |
| 006 hardening & security | ~2026-07-25 | — | 5 | — | 4–6 | — | |
| 007 release & distribution | ~2026-07-25 → ~08-01 | — | 6 | — | 5–8 | — | 🎯 release |

## Prediction log

- **2026-06-14** — Initial forecast after STAGE-001. Central guess: MVP
  ~2026-07-25 (range 06-28 → 09-15), 28 specs / ~33 active hours remaining.
  Revisit at each stage ship: record actual ship date + spec count, note
  whether the metadata/release risks materialized, and re-forecast.
