---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-079
  type: story                      # epic | story | task | bug | chore
  cycle: build                     # frame | design | build | verify | ship
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
  sessions:
    - cycle: build
      interface: claude-code
      tokens_total: 120000
      duration_minutes: 35
      estimated_usd: 1.10
      recorded_at: 2026-07-14
      note: >
        ran in the build session's main loop, not a metered subagent — tokens_total is an
        order-of-magnitude ESTIMATE (~80/20 in/out at Opus 4.8 list rates, no cache discount),
        not a harness-reported number. Includes the native suite at both feature sets, the
        wasm VM run, and one `just wasm-build` of the real artifact.
  totals:
    tokens_total: 120000
    estimated_usd: 1.10
    session_count: 1
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

- **Branch:** `spec-079-wasm-optimize-surface`
- **PR (if applicable):** #87
- **All acceptance criteria met?** yes — every row below is proven by a test that RAN, not inferred:

| Acceptance criterion | Met | Where it is proven |
|---|---|---|
| `optimizeDetailed(photoPng, "auto", 10, null, null)` → avif / smaller / q80 / speed 10 / no score | ✅ | `wasm::optimize_detailed_auto_photo_picks_avif` |
| `optimizeDetailed(graphicPng, "auto", …)` → lossless, never avif | ✅ | `wasm::optimize_detailed_auto_graphic_stays_lossless` |
| `optimizeDetailed(photoPng, "jpeg", null, null, 90)` → jpeg + engine score in (0, 100] | ✅ | `wasm::optimize_detailed_jpeg_returns_engine_score` |
| `optimizeDetailed(photoPng, "avif", 10, 20000, null)` → valid AVIF ≤ 20 000 B, speed 10, speed-parity | ✅ | `wasm::optimize_detailed_budget_is_honoured_and_speed_parity` + `quality::tests::size_search_speed_parity` |
| legacy `optimize(input, out_format)` unchanged | ✅ | `wasm::legacy_optimize_unchanged` (+ the three pre-existing `optimize_*` wasm tests, still green) |
| **native unchanged** — `encode_to_bytes(img, Avif, q)` byte-identical; no CLI flag | ✅ | `sink::tests::encode_to_bytes_forwards_to_with_at_default_speed` asserts the three forms are byte-identical FILES; `sink::tests::avif_speed_changes_output` proves speed isn't merely accepted-and-dropped |
| `score(png, png)` ≈ max; degraded lower; bad input → typed `JsError` | ✅ | `wasm::score_identical_is_max`, `wasm::score_degraded_is_lower_and_bad_input_errs` |
| no panic on hostile input (over-cap PNG header) | ✅ | `wasm::optimize_detailed_rejects_oversize_without_panic` — asserts the `Err` **and** that a later call still succeeds (i.e. the module survived) |
| `just wasm-test`, `cargo test`, `cargo clippy`, `cargo fmt --check` | ✅ | wasm 20/20 in the VM; native 716 (default) / 726 (`--features avif`); clippy clean on both feature sets; fmt clean. Also `cargo build --no-default-features` (the lean build) green. |

Bundle size is unchanged at **1.33 MB brotli** (SPEC-074's baseline) — the new surface is glue over
code that was already linked.

- **New decisions emitted:**
  - `DEC-068` — the wasm optimize surface takes a speed and a budget, returns a score, and Auto picks
    AVIF for photos — with the byte-parity contract extended to speed.
- **Deviations from spec:**
  - **The Auto-AVIF rule is a predicate on the bucket, not shortlist membership.** Outputs say "reuse
    the SizeBudget AVIF admission"; the Notes state the rule directly (bucket ∈ {`Lossy`, `MixedSafe`}
    ∧ `WASM_CODECS.avif`). Built as the Notes state it, because reading it off
    `format_shortlist(.., Mode::SizeBudget, ..)` does **not** reproduce that rule: the shortlist appends
    AVIF **last** and then `truncate(MAX_SHORTLIST = 3)`, so a `MixedSafe` image *without alpha* — which
    already has three entries ahead of AVIF — would silently lose it. The admission *criterion* is
    reused; the truncation is not. Recorded in DEC-068's implementation notes.
  - **`max_bytes` on a lossless target is ignored**, and documented as such on the entry point. The spec
    did not say what a budget means for PNG / lossless-WebP; honouring it would mean *resizing*, which
    SPEC-080 makes an explicit user-facing OFFER — this call must not do it behind their back.
  - **`quality()` is `Some(80)` for a default-encoded AVIF**, not `undefined`. The getter list says
    "`undefined` for lossless", and acceptance criterion 1 requires `quality == 80` on the unsearched
    Auto-AVIF path — so `undefined` is reserved for genuinely lossless output, and a lossy format
    encoded at its default reports that default.
- **Follow-up work identified:**
  - The **wasm Auto path and the native Auto path now diverge** for photographic input (wasm prefers
    AVIF; native still runs the perceptual shortlist). Deliberate and bounded (DEC-068), but it is the
    seam the deferred SPEC-072 shared multi-candidate solve will have to reconcile — worth naming in
    that spec when it is written.
  - `quality::tests::size_search_speed_parity` is now the **only** thing holding the two AVIF encode
    arms together, and it has two arguments to keep in sync rather than one. If a third encoder knob
    lands, the comment-contract should give way to a shared encode seam.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Only one thing, and the spec had already resolved it: "reuse the SizeBudget AVIF admission"
   (Outputs) and the explicit bucket predicate (Notes) are **not** the same rule once `MAX_SHORTLIST`
   truncation is accounted for. Treating the Notes as authoritative was right. Nothing else needed a
   decision the spec hadn't already made.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The `quality` → `sink` **layering rule** (it lives in `src/quality/mod.rs`'s module doc, not in the
   spec's constraint list). It matters precisely here: the speed-parity test is the one place that must
   cross that boundary — legitimately, in test code — and a builder who read the layering rule as
   absolute would have written a weaker test that *re-implements* the sink's encode instead of comparing
   against it, which is exactly the drift the test exists to catch.
3. **If you did this task again, what would you do differently?**
   — Read the analysis layer's classification thresholds **before** writing the wasm fixtures. The
   existing 64×48 fixture is an **Icon** to the engine (`ICON_MAX_EDGE = 128`), so a "photographic"
   fixture built at that size buckets `LosslessFlat` and the Auto-AVIF test would have gone red for a
   reason having nothing to do with the code under test. Reading `classify` first avoided that; guessing
   would have cost a cycle.

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
