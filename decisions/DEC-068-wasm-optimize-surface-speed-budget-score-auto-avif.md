---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-068
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-14
supersedes: null
superseded_by: null

affected_scope:
  - "src/wasm.rs"
  - "src/sink/mod.rs"
  - "src/quality/mod.rs"
  - "tests/wasm_roundtrip.rs"

tags:
  - wasm
  - avif
  - quality
  - performance
  - api-surface
---

# DEC-068: the wasm optimize surface takes a speed and a budget, returns a score, and Auto picks AVIF for photos — with the byte-parity contract extended to speed

## Decision

Four decisions, taken together because they are one surface (SPEC-079):

1. **A per-call AVIF encode speed exists — on the wasm surface only.** `sink::encode_to_bytes_with(img,
   fmt, quality, speed)` threads a rav1e speed; `speed = None` resolves to `AVIF_SPEED` (6).
   `encode_to_bytes` becomes a thin wrapper calling it with `None`, so **every native caller emits
   byte-identical output**. DEC-020's deferral of a *CLI* `--speed` flag **stands**: no flag is added,
   and the native default is still 6.
2. **The byte-parity contract (DEC-016/DEC-019) now covers speed.** The byte-budget search encodes its
   candidates at the caller's speed (`quality::auto_under_size_at_speed`), because a search that probes
   at speed 6 while the sink writes at speed 10 reports a budget for a file nobody writes.
3. **Auto picks AVIF for photographic input** on the wasm surface: bucket ∈ {`Lossy`, `MixedSafe`} and
   an AVIF encoder is built → AVIF at `AVIF_DEFAULT_QUALITY` (80) and the requested speed. Non-photo
   buckets fall through to today's shortlist-first behaviour, unchanged.
4. **`OptimizeResult` reports the score the engine actually measured, and `undefined` otherwise.** A
   perceptual search yields `score` + `scoredBy = "engine"`; **AVIF and lossless yield `None` +
   `scoredBy = "none"`** — never a fabricated number. A new `score(reference, candidate)` binding lets
   the *page* score an AVIF output that the engine cannot decode.

The legacy `optimize(input, out_format)` and `transform`/`info`/`version` are untouched.

## Context

STAGE-029's investigation (2026-07-13) drove the LIVE demo with real photos and found it mis-serves
exactly the input it exists to impress people with. Every branch of that failure traced back to the
wasm `optimize()` surface taking **no arguments** and returning **only bytes**:

- **Cost is megapixels, not megabytes**, and the only speed available was the native default. A 12 MP
  photo → AVIF took **~33 s** at speed 6. Measured: **speed 6 → 10 is ~3.6× faster for ~+4 % bytes.**
  The surface had no way to ask for that trade.
- **Auto avoided AVIF for photos.** `decide::format_shortlist` admits AVIF **only in
  `Mode::SizeBudget`** (DEC-048), because AVIF cannot be perceptually scored without a decoder
  (DEC-020/DEC-065). `optimize` runs `Mode::Perceptual`, so a photo's shortlist was `[JPEG]` — and the
  demo spent 4–11 s on an SSIMULACRA2 JPEG search to save **~13 %**, while declining to consider the
  AVIF that is both far smaller and, at speed 10, far faster to produce.
- **The score was computed and thrown away.** `QualityChoice.score` already carries the achieved
  SSIMULACRA2 of a searched encode; `optimize` returned bytes, so the demo could not show the quality
  it had just measured — SPEC-081's whole headline.

## Alternatives considered

- **Re-architect `format_shortlist` to admit AVIF in `Mode::Perceptual`.** Rejected: the mode gate is
  *correct*. AVIF is excluded from perceptual mode because the search **decodes each candidate to score
  it**, and this build has no AVIF decoder. Widening the gate would push the failure down into
  `auto_quality`, where it becomes a decode error mid-search instead of a clean shortlist rule. The
  narrow Auto-AVIF rule takes the SizeBudget-mode *admission criterion* (lossy-family content, no
  perceptual search) and applies it where the demo needs it, leaving the decision engine's invariants
  intact.
- **Run the full multi-candidate `pick_winner` solve on wasm** (encode every shortlist entry, measure,
  compare). Still deferred to the SPEC-072 shared-seam follow-up: it belongs in an engine seam both
  `cli` and `wasm` call, not copy-pasted into the surface — and it would encode a photo several times
  in a browser tab, which is the *opposite* of the problem being solved.
- **A CLI `--speed` flag.** Out of scope, and DEC-020's reasoning is unchanged for the CLI: a
  filesystem tool with no wall-clock crisis does not need the knob. The browser has the crisis.
- **Fabricate an AVIF score** (e.g. from the encoder's quality number). Refused. The engine cannot
  decode its own AVIF output, so any number it printed would be a *guess presented as a measurement*.
  `scoredBy` names the provenance instead, and the `score()` binding gives the page the honest route:
  the browser decodes AVIF natively, so it hands the pixels back and the engine scores them for real.
- **An `OptimizeOptions` struct** instead of positional optional args. Deferred (SPEC-079 out-of-scope):
  three optional args is not yet a struct's worth of surface, and DEC-064 keeps `serde-wasm-bindgen`
  out. Revisit if the arg list grows.

## Consequences

**Good**

- A 12 MP photo can now be optimized in the browser in seconds rather than half a minute, and Auto
  actually shrinks it (AVIF) instead of running a slow search for a marginal JPEG.
- SPEC-080 (demo redesign) and SPEC-081 (SSIMULACRA2 readout) are both **pure demo/JS specs** — neither
  needs to touch `src/wasm.rs`, because the score binding landed here.
- The native CLI is provably unchanged: `encode_to_bytes` forwards to `..._with(.., None)` and a unit
  test asserts the three encodes are **byte-identical files**, not merely equal-sized.

**Bad / risky**

- **The wasm Auto path and the native Auto path now differ** for photographic input (wasm prefers AVIF;
  native still runs the perceptual shortlist). That divergence is deliberate and bounded — different
  targets have different codec sets and *radically* different cost curves — but it is a seam that will
  need reconciling when the shared multi-candidate solve lands.
- **Speed is a second axis the parity contract must hold across.** `quality::encode_candidate_bytes_with`
  and `sink::encode_to_bytes_with` are kept identical by a *comment contract* (layering forbids
  `quality` depending on `sink`), and there are now two arguments to keep in sync instead of one. The
  guard is `quality::tests::size_search_speed_parity`, which crosses the layer **in test code only** and
  asserts probe bytes `==` sink bytes at speed 10 — the only thing that can catch the two arms drifting.
- `max_bytes` on a **lossless** target is ignored (there is no quality knob to search). Fitting a
  lossless image into a budget means *resizing* it, which is a choice the user makes — SPEC-080 offers
  it explicitly rather than having this call silently shrink their image.

## Implementation notes

- The Auto-AVIF rule is written as a **predicate on the bucket** (`wasm::auto_avif_quality`), not read
  off `format_shortlist(.., Mode::SizeBudget, ..)`. Reason: the shortlist **appends AVIF last and then
  truncates to `MAX_SHORTLIST` (3)**, so a `MixedSafe` image without alpha would lose AVIF to the
  truncation — making the rule depend on how many *other* candidates happened to precede it. The rule
  the demo needs is about content, so it keys on content. It mirrors DEC-048's admission criterion
  ("lossy-family bucket, and only where no perceptual search will run"), which is the part that carries
  the reasoning.
- Speed is clamped to `1..=10` and quality to `1..=100` in **both** encode arms, identically.
- Every new wasm entry returns `Result<_, JsError>`; nothing panics. A panic in wasm aborts the module
  and takes the page's engine instance down with it, so
  `optimize_detailed_rejects_oversize_without_panic` asserts both halves: the over-cap input errors,
  **and a later ordinary call still succeeds**.
- Bundle size is unchanged at **1.33 MB brotli** (SPEC-074's baseline) — the new surface is glue over
  code that was already linked.
