---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-081
  type: story
  cycle: ship
  blocked: false
  priority: high
  complexity: S

project:
  id: PROJ-008
  stage: STAGE-029
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-13

references:
  decisions: [DEC-019, DEC-065, DEC-068, DEC-069]
  constraints: [ergonomic-defaults]
  related_specs: [SPEC-078, SPEC-079, SPEC-080, SPEC-095]

value_link: >
  The demo's differentiator vs squoosh: we don't guess the quality, we measure it. Shows the
  input↔output SSIMULACRA2 score so "smaller AND still looks right" is proven on the page, not asserted.

cost:
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 100000
      duration_minutes: null
      estimated_usd: 3.0
      note: >
        Estimated order-of-magnitude (main-loop build run directly in the primary checkout, not a
        separately-metered subagent, per AGENTS.md worktree-per-session + the autonomous-run cost
        convention) — demo-files-only edits + one headless-Chrome smoke (reused committed vendor/, no
        wasm rebuild). ~80/20 input/output at Opus 4.8 list rate ($5/$25 per MTok), no cache discount.
    - cycle: verify
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 110000
      duration_minutes: null
      estimated_usd: 3.5
      note: >
        Estimated order-of-magnitude (main-loop verify in the primary checkout) — dominated by ~5
        headless-Chrome smoke runs (one from-source wasm rebuild + 4 negative-control runs: masquerade,
        negative, >100, break-the-gate). ~80/20 input/output at Opus 4.8 list rate, no cache discount.
    - cycle: ship
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: null
      estimated_usd: 0.45
      recorded_at: 2026-07-18
      note: >
        orchestrator main loop (un-metered) — ESTIMATE. Handed off build (Opus) → verify (Opus, CLEAN,
        4 negative controls) as prompts and stayed out of the repo while each ran; confirmed the merge
        with the maintainer, opened PR #100, polled CI (27/0 matrix green), squash-merged (60511f5),
        bookkeeping. No new DEC.
  totals:
    tokens_total: 210000
    estimated_usd: 6.95
    session_count: 3
---

# SPEC-081: demo SSIMULACRA2 diff UI

## Context

crustyimg's whole thesis is an *optimization engine* that uses **SSIMULACRA2** to decide quality
rather than guessing. The demo can now surface that: SPEC-079 returns the achieved score for a
searched lossy encode and adds a `score(a, b)` binding for the one case it can't self-score (AVIF, no
decoder — DEC-065). This spec is the **UI** that turns those into the demo's headline: *"we didn't
pick a quality and hope — here's the measured perceptual score."* That is the crisp wedge vs squoosh
(and the honest counter to "AVIF made it tiny — but does it still look right?").

It is **demo files only**, and it **extends the shipped SPEC-080/095 demo** — no engine change (SPEC-079
owns every binding).

**★ Reconciled against the shipped demo (2026-07-18) — build on what exists, don't rebuild:**
- `demo/demo.js` already has **`renderScore(...)`** (SPEC-080): it shows a real `SSIMULACRA2 X.X` for a
  JPEG (`scoredBy: "engine"`), an honest *"AVIF isn't scored here — this build can't decode it to
  measure"* for AVIF, and *"lossless — every pixel preserved"* for lossless. **This spec's job is to
  (a) make the AVIF case actually score, and (b) add the interpretable visual band.** Extend
  `renderScore`; don't fork a second path.
- `demo/worker.js` already has **`decodeInBrowser(bytes, label)`** (createImageBitmap → OffscreenCanvas)
  — the exact AVIF read-back seam. It currently reads output dimensions; here it also yields the PNG
  pixels to score. **`score` is NOT yet imported** in the worker (only `optimizeDetailed`/`transform`/
  `info`/`version`) — add `score` to that import.
- **SPEC-095 shipped:** the demo AVIF is now q85 (matches the CLI), so a visually-lossless AVIF should
  score high — a good sanity anchor for the browser-decode score.

## Goal

Show the input↔output **SSIMULACRA2 score** for each conversion, sourced honestly: from the engine for a
searched lossy encode, from a **browser-decode + `score(a, b)`** for AVIF (the headline case), and shown
as "lossless — every pixel preserved" for a lossless output. Present it on an **interpretable band** a lay
visitor reads ("visually lossless" / "high" / "medium" / "low"), **mapped honestly from the RAW
SSIMULACRA2 value** — which is *not* a 0–100 percentage: ~100 ≈ visually identical, it can exceed 100 and
**can go negative** on a bad encode (SPEC-079). Never clamp to (0,100]; never show a fabricated number.

## Inputs

- **Files to read:** the SPEC-080 demo (`demo/demo.js`, `demo/worker.js`, `demo/index.html`,
  `demo/demo.css`) as shipped; `src/quality/mod.rs` `score()` (the metric's meaning/scale).
- **The SPEC-079 surface:** `OptimizeResult { score, scoredBy, format, ... }` and
  `score(referenceBytes, candidateBytes) → f64`. Reconcile names against SPEC-079 as shipped.

## Outputs

- **Files modified:**
  - `demo/worker.js` — attach a score to every result:
    - **searched lossy** (JPEG): use `OptimizeResult.score` (`scoredBy: "engine"`).
    - **AVIF**: decode the output back to pixels (the worker already does this for `readBack` via
      `createImageBitmap` → OffscreenCanvas → PNG) and call `score(inputPngBytes, decodedOutputPngBytes)`
      (`scoredBy: "browser"`). Wrap in try/catch — an old browser that can't decode AVIF yields
      `scoredBy: "unavailable"`, not a thrown error.
    - **lossless** (PNG/WebP-lossless): no score needed — mark `scoredBy: "lossless"` (pixels are
      preserved by definition).
  - `demo/demo.js` — render a **score panel**: the number (1 decimal), a labelled band
    (indistinguishable / visually-lossless / high / medium / low), and a one-line honest source
    ("measured by the engine" / "measured by decoding the AVIF back in your browser" / "lossless —
    every pixel preserved" / "couldn't score this output"). Fold it into the existing explain readout.
  - `demo/index.html` — the score panel markup (a compact meter/scale + the number + the label).
  - `demo/demo.css` — the meter/scale styling (a simple band, not a heavy gauge); theme-aware.

## Acceptance Criteria

- [ ] A **JPEG** conversion shows the numeric raw SSIMULACRA2 score labelled "measured by the engine"
      (from `OptimizeResult.score`) with an interpretable band — **not clamped to (0,100]** (the value is
      raw; ~100 ≈ identical, can exceed 100 or go negative).
- [ ] An **AVIF** conversion (the demo's hero output) shows a numeric score obtained by **decoding the
      AVIF back in the browser** (`decodeInBrowser`) and calling `score(inputPixels, decodedOutputPixels)`,
      labelled as a browser-side measurement — **this is the case SPEC-080 could not score; making it work
      is the whole point.** At q85 (SPEC-095) a photo should land in the "visually lossless / high" band.
- [ ] A **lossless** (PNG / lossless-WebP) conversion shows "lossless — every pixel preserved" (no
      misleading number).
- [ ] When scoring is genuinely impossible (e.g. a browser too old to decode AVIF), the panel says so
      plainly and the rest of the result still renders — no thrown error, no fabricated score.
- [ ] The score is presented on an **interpretable scale** (band + one-line meaning), not a bare
      float — a non-expert can read "smaller AND still looks right".
- [ ] The browser smoke passes: the score element is present and numeric for a lossy output, and the
      page still makes zero network requests during a conversion.

## Failing Tests

Browser-driven (the SPEC-077/078/080 headless-Chrome smoke, extended).

- `"jpeg_shows_engine_score"` — drive a photo → JPEG; assert a score element with the raw numeric
  SSIMULACRA2 value (NOT clamped to `(0,100]`) + a band and a "measured by the engine"-class label.
- `"avif_shows_browser_score"` — drive a photo → AVIF (default); assert a numeric score element and a
  `scoredBy=="browser"` label (the AVIF was decoded back and scored).
- `"lossless_shows_lossless_not_a_number"` — drive a graphic → lossless; assert the "lossless"
  state, and that no misleading numeric score is shown.
- **Verify (documented):** the AVIF browser-decode score is *sane* (near the engine's target for a
  visually-lossless encode, well below 100 for an aggressive one) — sanity, not an exact golden.

## Implementation Context

### Decisions that apply
- `DEC-019` — SSIMULACRA2 is the perceptual target the engine searches to; the demo now *shows* it.
- `DEC-065` — AVIF encodes but does not decode on wasm; that is exactly why AVIF's score comes from a
  **browser** decode + the `score()` binding, not the engine. Be honest about which did the measuring.

### Constraints that apply
- `ergonomic-defaults` — the score must be *interpretable* by a non-expert (a band + a plain-language
  meaning), not a raw metric only a codec nerd parses.

### Prior related work
- `SPEC-078` — the explain readout this extends; the worker's `readBack` already browser-decodes an
  AVIF output for its dimensions — reuse that exact decode to get pixels for scoring.
- `SPEC-079` — the `score()` binding and `OptimizeResult.score` this consumes.
- `SPEC-080` — the reshaped page this builds on (build SPEC-081 after SPEC-080).

### Out of scope (for this spec specifically)
- Any `src/` / wasm change (SPEC-079 owns the bindings).
- A side-by-side pixel-diff or zoom/loupe comparison — this spec shows the *score*; a visual
  before/after diff would be its own follow-up if wanted.
- Re-encoding at multiple qualities to plot a size/quality curve (a nice future idea, not this).

## Notes for the Implementer
- **Don't double-decode.** The worker's `readBack` already decodes the AVIF output once (for dims);
  reuse those pixels for the score rather than decoding a second time.
- **Never fabricate.** No score for AVIF-that-won't-decode and no "100" dressed up for lossless as if
  it were measured — say what actually happened. The candor is the point.
- **Keep the meter simple** and theme-aware; a labelled band beats a skeuomorphic gauge.

---

## Build Completion
- **Branch:** `spec-081-score-ui`
- **PR (if applicable):** none yet (handed back for review, not opened)
- **All acceptance criteria met?** yes
  - JPEG → numeric raw SSIMULACRA2, "measured by the engine", banded, not clamped — ✓ (smoke: `scoredBy="engine"`, 91.0)
  - AVIF → numeric score via browser decode + `score(...)`, labelled browser-side; q85 photo lands "high" — ✓ (smoke: `scoredBy="browser"`, 80.2, band "high")
  - Lossless → "every pixel preserved", no number — ✓ (smoke: `scoredBy="lossless"`, no value, meter hidden)
  - Scoring-impossible → candid, no throw, rest still renders — ✓ (`scoredBy="unavailable"` branch, worded panel)
  - Interpretable scale (band + one-line meaning) — ✓ (band pill + meter + honest source line)
  - Smoke passes, score element numeric for a lossy output, zero network requests — ✓ (exit 0; 0 requests during conversion)
- **New decisions emitted:** none (SPEC-079 owns the bindings; no engine change).
- **Files changed:** `demo/worker.js` (import `score`; `readBack` yields the decoded-AVIF PNG from its single decode; attach honest `{score, scoredBy}` — engine/browser/lossless/unavailable), `demo/demo.js` (ui refs + `scoreBand()` + rewritten `renderScore` into a banded panel), `demo/index.html` (score-panel markup), `demo/demo.css` (banded meter, theme-token styling), `tests/demo_smoke.mjs` (3 named tests + AVIF sanity; `drop()` exposes score sub-elements).
- **Deviations from spec:**
  1. **Reference for the browser AVIF score is the *downscaled* PNG (`pixels` after `transform`), not the original input.** The spec wrote `score(inputPngBytes, decodedOutputPngBytes)`; `score()` requires matching dimensions and the `web` flow downscales before encoding, so the original (2200×1650) vs the AVIF (2048×1536) would throw a dimension mismatch. The downscaled PNG is also the exact basis the engine's own JPEG search scores on, so the two provenances are consistent. This is the intended behaviour, just more precise than the spec's shorthand.
  2. **Added a sixth band, "very low", below "low"** (for negative / badly-degraded scores). The spec enumerated five; the honesty rule ("band must handle negatives") makes "low" for a −4.70 misleading, so the bottom band is named honestly.
  3. **"Theme-aware" = built from the page's shared CSS theme tokens**, not a separate light/dark scheme. The demo commits to one colour scheme (`color-scheme: dark`, SPEC-080); a lone light-mode meter on a dark page would look broken. The panel tracks the theme via the existing vars.
  4. **The `"unavailable"` branch mainly guards the `score()` call itself, not the AVIF decode.** The demo already browser-decodes an AVIF output for its dimensions (pre-existing `readBack`), so a browser that genuinely can't decode AVIF already fails the whole conversion upstream. The score try/catch keeps a *scoring* failure (mismatch/cap/missing pixels) from crashing an otherwise-good result — honest, but the "old browser" scenario is largely theoretical given the existing dims dependency.
- **Follow-up work identified:** the side-by-side pixel/loupe diff is explicitly out of scope here (spec's Out-of-Scope) — a candidate follow-up if wanted. The observed q85 AVIF score (~80, "high") is a touch below the "visually lossless" band; not a bug (it's candid), but worth a glance during launch BENCHMARKS.

### Build-phase reflection (3 questions, short answers)
1. **What was unclear in the spec that slowed you down?** — The reference argument for the AVIF `score()`: the spec's `score(inputPngBytes, ...)` reads as "the original input", but that mismatches dimensions after the downscale. Reading `score()`'s "both must match dimensions" contract resolved it — the reference must be the downscaled PNG the encoder saw.
2. **Was there a constraint or decision that should have been listed but wasn't?** — The `score()` dimension-match requirement deserved a one-line note in the spec's Notes for the Implementer; it's the one thing that turns "call score(input, output)" from wrong to right.
3. **If you did this task again, what would you do differently?** — Verify the `drop()` helper's same-file `change`-event quirk before writing tests; I lost a run to re-dropping `heroPath` for the JPEG case (no `change` fires when the file input already holds that path). Distinct fixtures per drop is the established pattern.

---

## Reflection (Ship)
1. **What would I do differently next time?** — Not offload the build to the maintainer as a copy-paste
   chore after mis-starting a subagent. This ship's process wobble was mine as orchestrator: I first
   spawned a background build subagent (wrong), then over-corrected into handing the maintainer a prompt
   to run manually. The maintainer's steer landed the pattern: the orchestrator hands off a build/verify
   prompt to a *session* and stays out of the repo; it does not do the build itself nor make the human
   the runner. Held that cleanly through verify → merge.
2. **Does any template, constraint, or decision need updating?** — No template/DEC change. Reinforces two
   standing lessons: the demo-honesty discipline held (every honesty claim gated by a negative control —
   NC-1 masquerade `score(px,px)=100` vs real `80.17`, NC-2 negative renders unclamped, NC-4 breaking
   `score()` fails the gate), continuing the stage's "a plausible test result is not a checked one" theme.
   One spec-clarity note for future demo specs: when a spec says `score(input, output)`, spell out the
   `score()` dimension-match contract so "reference = the downscaled PNG the encoder saw" isn't a
   build-time discovery (the build's own reflection flagged this).
3. **Is there a follow-up spec I should write now before I forget?** — No new spec required. Two carries,
   both belong to STAGE-028 launch-readiness, not their own spec: (a) the score panel is Chrome-verified
   only — new `color-mix()` CSS degrades gracefully but glance on Firefox/Safari before Show HN; (b) the
   observed q85 AVIF score (~80, "high", a touch below "visually lossless") is candid and correct — worth
   noting when BENCHMARKS frames the quality story so the number isn't read as a defect. The out-of-scope
   side-by-side pixel/loupe diff remains a genuine future idea, not launch-blocking.
