---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-079
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-008
  stage: STAGE-029
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # usually same Claude, different session
  created_at: 2026-07-13

references:
  decisions: [DEC-016, DEC-019, DEC-020, DEC-048, DEC-064, DEC-065]
  constraints: [pure-rust-codecs-default, no-unwrap-on-recoverable-paths, untrusted-input-hardening, every-public-fn-tested, test-before-implementation, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-072, SPEC-073, SPEC-074, SPEC-078]

value_link: >
  Provides the engine surface STAGE-029's demo redesign (SPEC-080) and SSIMULACRA2 readout
  (SPEC-081) consume: a per-call encode speed, a size/quality target, a returned perceptual score,
  and an Auto path that actually shrinks a photo instead of picking a slow JPEG search.

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-079: wasm optimize surface — speed, budget, score, auto-avif

## Context

STAGE-029's measured investigation (2026-07-13) found the demo mis-serves a photo because the
**wasm `optimize()` surface is too thin** — it takes no arguments and returns only bytes
(`src/wasm.rs:187`). Three consequences, all rooted here:

1. **No speed control.** rav1e's speed is the hardcoded `AVIF_SPEED = 6` (`src/sink/mod.rs:48`); a
   12 MP photo → AVIF takes ~33 s. Speed 10 is a measured **3.6× faster for ~+4 % bytes**, but the
   surface has no way to ask for it.
2. **No returned score.** The engine already *computes* the achieved SSIMULACRA2 for a searched
   lossy encode (`QualityChoice.score`, `src/quality/mod.rs:155`) and even exposes a public
   `quality::score(ref, cand)` — but `optimize` throws it away, so the demo can't show the quality it
   chose (SPEC-081's headline).
3. **Auto avoids AVIF for photos.** `decide::format_shortlist` only admits AVIF in
   `Mode::SizeBudget` (it has no decoder, so it can't be perceptually scored — DEC-020/DEC-048). The
   wasm `optimize` runs `Mode::Perceptual`, so a photo's shortlist is `[JPEG]` and it runs the slow
   SSIMULACRA2 JPEG search (~4–11 s for only ~13 % savings) instead of a much smaller AVIF.

This spec is **SPEC-079**, the first (foundation) spec of STAGE-029; SPEC-080 (demo redesign) and
SPEC-081 (SSIMULACRA2 diff UI) consume its surface. It is engine + `wasm-bindgen` surface only —
**no demo UI, no CLI change.**

## Goal

Give the wasm build a richer optimize entry that (a) takes a per-call **encode speed**, a
**size/quality target**, (b) returns the achieved **SSIMULACRA2 score** alongside the bytes, and
(c) in **Auto** mode picks **AVIF at a fixed good quality for photographic input** (reusing the
engine's existing `SizeBudget` AVIF semantics), so a photo shrinks fast instead of running a slow
JPEG search. The existing `optimize(input, out_format)` and all native/CLI behaviour stay unchanged.

## Inputs

- **Files to read:**
  - `src/wasm.rs` — the surface being extended; today's `optimize` (lines 171–243) is the model.
  - `src/analysis/decide.rs` — `format_shortlist` (AVIF-in-SizeBudget rule, line 152), `pick_winner`,
    `Mode`, `Disposition`, `BuiltCodecs`.
  - `src/quality/mod.rs` — `auto_quality` → `QualityChoice { quality, score }` (line 148),
    `search_size` (byte-budget, no decode, line ~455), public `score()` (line 99), `SearchConfig`.
  - `src/sink/mod.rs` — `encode_to_bytes` (line 582), the AVIF arm (`new_with_speed_quality`, line
    614), `AVIF_SPEED = 6` (48), `AVIF_DEFAULT_QUALITY = 80` (54).
- **Related code paths:** `tests/wasm_roundtrip.rs` (the `#[wasm_bindgen_test]` harness run by
  `just wasm-test`), `src/sink/mod.rs` unit tests (native AVIF parity).

## Outputs

- **Files modified:**
  - `src/sink/mod.rs` — add a **non-invasive** speed-aware encode entry:
    `pub fn encode_to_bytes_with(img, fmt, quality: Option<u8>, speed: Option<u8>) -> Result<Vec<u8>, _>`,
    where `speed` affects **only** AVIF and defaults to `AVIF_SPEED` when `None`. Re-express the
    existing `encode_to_bytes(img, fmt, quality)` as a thin wrapper calling `..._with(.., None)` so
    **every current caller is untouched** and native behaviour is byte-identical.
  - `src/quality/mod.rs` — allow the byte-budget AVIF search to encode candidates at a **given
    speed** (so a size search at speed S emits bytes equal to the sink at speed S — the byte-parity
    cross-sync contract, DEC-016/DEC-019, **extended to speed**). Perceptual search is unchanged
    (AVIF is never perceptually searched).
  - `src/wasm.rs` — add the new entry + result type (below); leave `optimize`, `transform`, `info`,
    `version` unchanged.
- **New exports (wasm surface):**
  - `optimize_detailed(input: &[u8], out_format: &str, speed: Option<u8>, max_bytes: Option<u32>, target: Option<f64>) -> Result<OptimizeResult, JsError>`
    — JS name `optimizeDetailed`. Positional optional args (no `serde-wasm-bindgen`; consistent with
    SPEC-072's `ImageInfo`-struct choice, DEC-064). `speed` = rav1e speed (AVIF only); `max_bytes` =
    a byte budget (runs the size search); `target` = perceptual SSIMULACRA2 target for searchable
    lossy formats (defaults to `DEFAULT_TARGET = 90`).
  - `#[wasm_bindgen] struct OptimizeResult` with getters: `bytes(): Uint8Array`, `format(): string`,
    `quality(): number | undefined` (encoder quality; `undefined` for lossless), `speed(): number |
    undefined` (AVIF only), `score(): number | undefined` (achieved SSIMULACRA2, `undefined` where
    the engine cannot score it — i.e. AVIF and lossless), `scoredBy(): string` (`"engine"` |
    `"none"`).
  - `score(reference: &[u8], candidate: &[u8]) -> Result<f64, JsError>` — JS name `score`. Decodes
    both images (`Image::from_bytes`) and returns their SSIMULACRA2 score via the existing public
    `quality::score`. This is what lets **SPEC-081** put a number on an **AVIF output** (which the
    engine cannot self-score, DEC-065): the page decodes the AVIF back to pixels in the browser
    (the worker already does this) and calls `score(inputBytes, decodedOutputBytes)`. Keeping this
    binding in SPEC-079 keeps SPEC-081 pure demo/JS (no `src/wasm.rs` change in a UI spec).
- **New native export:** `sink::encode_to_bytes_with` (+ the internal speed-threaded size search).
- **New decision:** `DEC-068` (surface shape + Auto-AVIF rule + speed-parity; see below).

## Acceptance Criteria

- [ ] `optimizeDetailed(photoPng, "auto", 10, null, null)` returns `format == "avif"`, `bytes` < the
      input, `quality == 80`, `speed == 10`, `score == undefined`, `scoredBy == "none"`.
- [ ] `optimizeDetailed(graphicPng, "auto", null, null, null)` returns a **lossless** format
      (`png`/`webp`), never `avif` — the Auto-AVIF rule fires only for photographic (lossy-bucket) input.
- [ ] `optimizeDetailed(photoPng, "jpeg", null, null, 90)` returns `format == "jpeg"`, `quality`
      set, `score` a number in `(0, 100]`, `scoredBy == "engine"` (the perceptual search still runs
      for JPEG).
- [ ] `optimizeDetailed(photoPng, "avif", 10, 20000, null)` returns valid AVIF with `bytes <= 20000`
      (byte budget honoured by the size search) and `speed == 10`; the emitted byte length equals the
      size search's chosen-candidate length (speed-parity holds).
- [ ] The legacy `optimize(input, out_format)` is **unchanged** — its existing round-trip test still
      passes byte-for-byte.
- [ ] **Native unchanged:** `encode_to_bytes(img, Avif, q)` produces bytes identical to before this
      spec (it forwards to `encode_to_bytes_with(.., None)` at `AVIF_SPEED`); no CLI flag added.
- [ ] `score(png, png)` (identical bytes) returns a value at/near the SSIMULACRA2 max (~100);
      `score(png, degradedJpegOfSameImage)` returns a value below it; `score` on undecodable/over-cap
      input returns a typed `JsError`, never a panic.
- [ ] **No panic on hostile input:** `optimizeDetailed` on an over-cap image (e.g. a 100000² PNG
      header) returns a typed `JsError`, never a panic (the DEC-034/DEC-063 caps carry;
      `untrusted-input-hardening`).
- [ ] `just wasm-test`, `cargo test` (native), `cargo clippy`, and `cargo fmt --check` all pass.

## Failing Tests

Written now, at **design**; the build makes them pass.

- **`tests/wasm_roundtrip.rs`** (`#[wasm_bindgen_test]`, run by `just wasm-test`)
  - `"optimize_detailed_auto_photo_picks_avif"` — a synthetic photographic PNG (high-entropy, no
    alpha) through `optimize_detailed(_, "auto", Some(10), None, None)` → `format() == "avif"`,
    `bytes().len() < input.len()`, `quality() == Some(80)`, `speed() == Some(10)`, `score() == None`.
  - `"optimize_detailed_auto_graphic_stays_lossless"` — a flat few-colour PNG through
    `optimize_detailed(_, "auto", None, None, None)` → `format()` ∈ {`png`, `webp`} and **not** `avif`.
  - `"optimize_detailed_jpeg_returns_engine_score"` — a photographic PNG → `optimize_detailed(_,
    "jpeg", None, None, Some(90.0))` → `format()=="jpeg"`, `score()` is `Some(s)` with `0.0 < s <=
    100.0`, `scoredBy()=="engine"`.
  - `"optimize_detailed_budget_is_honoured_and_speed_parity"` — photographic PNG →
    `optimize_detailed(_, "avif", Some(10), Some(20_000), None)` → valid AVIF (ftyp sniff),
    `bytes().len() <= 20_000`, `speed()==Some(10)`.
  - `"optimize_detailed_rejects_oversize_without_panic"` — a 100000×100000 PNG header →
    `optimize_detailed` returns `Err`, module still alive (assert a later call succeeds).
  - `"score_identical_is_max"` — `score(png, png)` (same bytes) ≈ SSIMULACRA2 max (assert `> 99.0`).
  - `"score_degraded_is_lower_and_bad_input_errs"` — `score(png, lowQualityJpegOfIt)` `< 99.0`;
    `score(png, b"not an image")` returns `Err` (no panic).
  - `"legacy_optimize_unchanged"` — keep/keep-green the existing `optimize(_, "auto")` assertion.
- **`src/sink/mod.rs`** (native `#[test]`)
  - `"encode_to_bytes_forwards_to_with_at_default_speed"` — `encode_to_bytes(img, Avif, Some(q))`
    bytes `==` `encode_to_bytes_with(img, Avif, Some(q), None)` bytes `==` `..._with(.., Some(AVIF_SPEED))`.
  - `"avif_speed_changes_output"` — `encode_to_bytes_with(img, Avif, Some(q), Some(1))` ≠
    `..._with(img, Avif, Some(q), Some(10))` (speed is actually threaded, not ignored).
- **`src/quality/mod.rs`** (native `#[test]`)
  - `"size_search_speed_parity"` — the byte-budget AVIF search at speed S returns a `QualityChoice`
    whose winning candidate length equals `encode_to_bytes_with(img, Avif, Some(choice.quality),
    Some(S))` length (the parity the sink relies on).

## Implementation Context

### Decisions that apply

- `DEC-020` — AVIF speed/quality defaults (`AVIF_SPEED=6`, `AVIF_DEFAULT_QUALITY=80`) and the
  **deferral of a per-call speed knob**. This spec reopens it *for the wasm surface only* (no CLI
  flag); the native default stays 6.
- `DEC-048` — the format decision engine: `format_shortlist` admits AVIF **only in `SizeBudget`
  mode**, `pick_winner` picks smallest-that-beats-source. The Auto-AVIF rule here is deliberately a
  *narrow* reuse of the SizeBudget AVIF admission, **not** a re-architecture of the decision tree.
- `DEC-019` / `DEC-016` — the byte-parity cross-sync contract (a searched candidate's bytes must
  equal the sink's bytes). Extending speed into the size search means the search MUST encode at the
  same speed the sink will emit, or parity breaks.
- `DEC-065` — AVIF encodes but does not decode on wasm. Hence AVIF is never perceptually searched
  and its `score()` is `None` from the engine (SPEC-081 does the browser-decode scoring page-side).
- `DEC-064` — the wasm surface avoids `serde-wasm-bindgen` (uses `#[wasm_bindgen]` structs with
  getters); `OptimizeResult` follows `ImageInfo`'s pattern, and options are positional args.

### Constraints that apply

- `pure-rust-codecs-default` — no new codec/dep; AVIF encode is already built (DEC-065).
- `no-unwrap-on-recoverable-paths` / `untrusted-input-hardening` — a panic aborts the wasm module and
  kills the page instance; every path returns a typed `JsError`. The decode caps (DEC-034/DEC-063)
  already live in the core and carry unchanged.
- `every-public-fn-tested` / `test-before-implementation` — the Failing Tests above exist before build.
- `no-new-top-level-deps-without-decision` — none needed.

### Prior related work

- `SPEC-072` (shipped) — the wasm surface + the "wasm `optimize` takes the shortlist's first
  candidate, full multi-candidate solve deferred to a shared seam" follow-up. This spec adds a
  *targeted* Auto-AVIF rule, **not** that full solve (still deferred).
- `SPEC-073` (shipped, DEC-065) — AVIF encode on wasm; the browser reads `.avif` inputs.
- `SPEC-074` (shipped) — the size profile; a size regression check applies to the built artifact.

### Out of scope (for this spec specifically)

- The demo UI, the speed-10 default, warnings/timer/resize (**SPEC-080**).
- The SSIMULACRA2 diff UI and browser-side decode-scoring of AVIF output (**SPEC-081**).
- A native CLI `--speed` flag (DEC-020's CLI deferral stands).
- The **full comparison-shop** multi-candidate `pick_winner` solve on wasm (encode every shortlist
  entry, measure, compare) — SPEC-079 does the narrow "photo → AVIF@default" rule; the general solve
  remains the SPEC-072 shared-seam follow-up.
- An options *struct* (`OptimizeOptions`) — positional optional args are enough for now; revisit if
  the arg list grows.

## Notes for the Implementer

- **Keep the seam thin.** `optimize_detailed` is glue over `Analysis::compute` →
  `decide::format_shortlist` → (`auto_quality` | `search_size`) → `encode_to_bytes_with`. Do not
  re-implement any decision or encode.
- **The Auto-AVIF rule, precisely:** when `out_format` is empty/`"auto"`, compute the bucket; if it is
  a lossy-family bucket (`OptBucket::Lossy`, or `MixedSafe` without the docs bias) **and**
  `WASM_CODECS.avif` — choose AVIF. Then: if `max_bytes` is set, run the **size** search
  (`search_size`, no decode) at the requested speed to hit the budget; else encode once at
  `AVIF_DEFAULT_QUALITY` and the requested speed. For any non-photo bucket, fall through to today's
  behaviour (shortlist-first + perceptual search where scoreable). This is the one behavioural change
  and it is confined to the wasm Auto path.
- **`score()` truth table:** searched lossy (JPEG / lossy-WebP, though wasm has no lossy-WebP) →
  `Some(QualityChoice.score)`, `scoredBy="engine"`; AVIF and lossless → `None`, `scoredBy="none"`.
  Do not fabricate a score for AVIF — SPEC-081 will decode-and-score it in the browser via the public
  `quality::score`.
- **Speed default must be invisible.** `encode_to_bytes(_, _, q)` and every native caller must emit
  the exact bytes they do today; prove it with the parity test before touching the wasm path.
- **Verify will drive the real artifact.** Expect the verifier to time speed 6 vs 10 on a real photo
  (the scratchpad `genpng.py`/`timedir.mjs` from framing are reusable) and to confirm the Auto path
  picks AVIF for a photo and lossless for a graphic — so the acceptance criteria are driven, not just
  unit-tested.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-068` — wasm optimize surface (speed/target/maxBytes + OptimizeResult score; Auto prefers
    AVIF@default for photographic input; per-call speed threads through `encode_to_bytes_with` and the
    byte-parity contract extends to speed; native/CLI unchanged) — draft title, confirm at build.
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>
3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct from the process-focused
build reflection above.*

1. **What would I do differently next time?**
   — <answer>
2. **Does any template, constraint, or decision need updating?**
   — <answer>
3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
