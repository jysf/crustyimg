---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-081
  type: story
  cycle: design
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
  decisions: [DEC-019, DEC-065]
  constraints: [ergonomic-defaults]
  related_specs: [SPEC-078, SPEC-079, SPEC-080]

value_link: >
  The demo's differentiator vs squoosh: we don't guess the quality, we measure it. Shows the
  input↔output SSIMULACRA2 score so "smaller AND still looks right" is proven on the page, not asserted.

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-081: demo SSIMULACRA2 diff UI

## Context

crustyimg's whole thesis is an *optimization engine* that uses **SSIMULACRA2** to decide quality
rather than guessing. The demo can now surface that: SPEC-079 returns the achieved score for a
searched lossy encode and adds a `score(a, b)` binding for the one case it can't self-score (AVIF, no
decoder — DEC-065). This spec is the **UI** that turns those into the demo's headline: *"we didn't
pick a quality and hope — here's the measured perceptual score."* That is the crisp wedge vs squoosh
(and the honest counter to "AVIF made it tiny — but does it still look right?").

It is **demo files only**, and it builds on **SPEC-080**'s reshaped page (both touch `demo/`, so it
is built after SPEC-080). No engine change — SPEC-079 owns every wasm binding it needs.

## Goal

Show the input↔output **SSIMULACRA2 score** for each conversion, on a scale a lay visitor
understands (100 = indistinguishable, ~90 = visually lossless, ~70 high, ~50 medium), sourced
honestly: from the engine for a searched lossy encode, from a browser-decode + `score(a, b)` for
AVIF, and shown as "lossless — every pixel preserved" for a lossless output. Where it genuinely
can't be computed, say so — never show a fabricated number.

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

- [ ] A **JPEG** conversion shows a numeric SSIMULACRA2 score in `(0, 100]` labelled "measured by the
      engine" (from `OptimizeResult.score`).
- [ ] An **AVIF** conversion shows a numeric score obtained by decoding the AVIF back in the browser
      and calling `score(...)`, labelled as a browser-side measurement — the headline case works.
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

- `"jpeg_shows_engine_score"` — drive a photo → JPEG; assert a score element with a number in
  `(0,100]` and a "measured by the engine"-class label.
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
- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection (3 questions, short answers)
1. **What was unclear in the spec that slowed you down?** — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?** — <answer>
3. **If you did this task again, what would you do differently?** — <answer>

---

## Reflection (Ship)
1. **What would I do differently next time?** — <answer>
2. **Does any template, constraint, or decision need updating?** — <answer>
3. **Is there a follow-up spec I should write now before I forget?** — <answer>
