---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-082
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent
    - operator

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-24
supersedes: null
superseded_by: null

affected_scope:
  - src/wasm.rs
  - src/image/raw.rs
  - src/image/mod.rs
  - demo/demo.js
  - demo/worker.js
  - demo/index.html
  - tests/wasm_roundtrip.rs
  - tests/demo_smoke.mjs
  - examples/gen_raw_gate_fixtures.rs

tags:
  - wasm
  - raw
  - untrusted-input
  - memory
  - demo
---

# DEC-082: RAW preview extraction on wasm, behind a demo-specific pixel gate

## Decision

The browser demo can now open RAW files (`.nef`/`.cr2`/`.cr3`/`.arw`/`.dng`/…):
two additive `#[wasm_bindgen]` exports, `rawPreview(bytes) -> Result<Vec<u8>,
JsError>` (extract the largest embedded JPEG preview, PNG-encoded) and
`isRawExtension(name) -> bool` (a thin wrapper over `raw::is_raw_extension`),
routed by file extension exactly as native `Image::load` routes RAW (DEC-055).

**`rawPreview` is gated by ONE tunable constant, `MAX_RAW_PREVIEW_MEGAPIXELS =
40` (`src/wasm.rs`).** Before the full extraction, it peeks the largest
candidate preview's *declared* dimensions (a JPEG header parse, not a pixel
decode — `raw::largest_declared_preview_pixels`, reusing the exact SOI scan and
candidate prune `raw::scan_for_preview` uses) and returns a typed "too large"
error if the declared pixel count exceeds the gate, **before** any
full-resolution allocation. This is the DEC-063 SOF-peek pattern, reused at a
demo-specific ceiling BELOW the native decode budget — native RAW handling
(`Image::load`, `raw_preview`, `extract_preview`) is completely unchanged.

**The threshold is a maintainer-set framing default, not a measured optimum.**
The design-time probe (`docs/research/proj-008-raw-on-wasm-probe.md`) measured
two real camera files through the actual wasm-reachable extraction path:

| | declared/measured preview | wasm peak memory |
|---|---|---|
| Fujifilm RAF (screen-res preview) | 2.46 Mpix | ~110 MB |
| Leica DNG (near-full-sensor preview) | 46.7 Mpix | ~320 MB |

40 Mpix sits between these: it passes the measured-safe RAF preview and a
typical full-frame preview (~24 Mpix), and rejects the measured-risky Leica
DNG before its ~320 MB decode ever allocates. **No real phone or mobile
browser was used to derive 40** — the probe's numbers are Node's V8 on a
desktop Mac, not iOS Safari's JavaScriptCore, and this project has been burned
before by an unverified device-dependent claim shipping wrong
([[a-claimed-failure-mode-is-as-unproven-as-a-claimed-success]]). The
maintainer's explicit instruction (2026-07-24): ship a permissive default now
— don't block a working, cheap, correct capability on a phone test that
cannot happen before the feature exists — and tune the threshold on-device as
a post-ship launch-readiness step, not a build decision.

**User-facing copy is maintainer-approved, shipped verbatim:**
- Too large: *"This RAW's built-in preview is very high-resolution — convert
  it with the crustyimg CLI instead."*
- No decodable preview: *"Couldn't find a preview image inside this RAW
  file."*

Both `rawPreview`'s error paths are remapped to exactly these two strings —
the engine's internal `raw: …`-prefixed messages (and a bare `Tiff is not
supported`, the original leaked symptom) never reach the browser banner.

## Context

The demo's front-door pitch is *"sharp and squoosh can't open these at
all"* (RAW). Before this spec, dropping a RAW produced exactly the failure
mode the pitch promises to fix: every wasm entry point calls
`Image::from_bytes` with no filename, a DNG sniffs as plain TIFF
(byte-ambiguous, DEC-055), and the wasm build has no TIFF decoder (DEC-066,
−84 KB brotli) — so the browser showed *"could not decode image: The image
format Tiff is not supported."*

`raw::extract_preview` was already compiled into every wasm build (no `cfg`
gate on `raw.rs`) but dead-code-eliminated because nothing in the
wasm-reachable call graph called it. The probe measured wiring it in at
**+1,214 B brotli** (no new codec — JPEG is already linked for every other
format), and independently verified the extraction itself runs correctly
inside a real wasm VM on two real camera files (in-VM dimensions cross-checked
against macOS `sips` and `file`, decoders the project didn't write). The one
open question the probe could not close was mobile memory, above.

## Alternatives Considered

- **Wire it in unconditionally (no gate).** Rejected: the 46.7 Mpix Leica
  sample's ~320 MB peak is a plausible OOM/tab-kill on a memory-constrained
  mobile Safari tab, and shipping that risk as a blanket "RAW now supported"
  claim repeats a pattern this project has already been burned by.
- **Wait for a real iOS device test before shipping anything.** Rejected —
  the maintainer's explicit call: there is nothing to phone-test until the
  feature exists, and the mechanism is proven correct and nearly free in
  bundle size. Gating converts an open-ended, unverified risk into a bounded,
  engineered, testable one that can ship now.
- **A conservative/small threshold (e.g. 8–12 Mpix) to be safe by
  construction.** Rejected per the maintainer's explicit steer: the gate
  should be *permissive by default*, catching only genuinely huge previews,
  tightened later only if a real device shows it needs to be. A
  low-resolution scanner RAW today would fail the gate for no measured
  reason.
- **Hand-copy the RAW extension list into `demo.js`.** Rejected — a second
  copy can silently drift from `raw::RAW_EXTENSIONS`. `isRawExtension` exists
  specifically so the JS routing list is DERIVED from the engine's own list.

## Consequences

**Good**

- The demo closes its stated RAW gap: a dropped `.dng`/`.cr2`/… under the
  gate previews and converts through the exact same `web` flow as any other
  input, with zero network requests.
- The gate is a single named constant (`MAX_RAW_PREVIEW_MEGAPIXELS`), so
  tuning it later (the launch-readiness step below) touches one line.
- Bundle cost stayed negligible: **+1,262 B brotli** measured against this
  spec's own before/after build (1,395,239 B baseline
  from the probe's session → 1,396,501 B with `rawPreview` + `isRawExtension`
  + the gate) — within a small margin of the probe's own +1,214 B estimate
  for the bare export alone.
- Native RAW handling is provably unchanged: the gate lives entirely in
  `src/wasm.rs` (`cfg(target_arch = "wasm32")`), and the new `raw.rs` helper
  (`largest_declared_preview_pixels`) is additive, `pub(crate)`, called by no
  native code path.

**Bad / risky**

- **The 40 Mpix threshold is still unverified on real mobile hardware.** It
  is an engineered, bounded, testable default — not a measured safe ceiling
  for iOS Safari specifically. A phone with less available tab memory than
  this project's desktop-Node probe assumed could still OOM on a preview
  just under the gate. This is a known, accepted, and explicitly deferred
  risk, not an oversight.
- **A legitimate, safely-sized preview from an unusually preview-heavy
  camera could still be rejected** if its declared dimensions exceed 40 Mpix
  even though the actual file would have decoded fine on the user's device —
  the gate trades a small false-rejection rate for bounded worst-case memory.
- The gate's own peek (`largest_declared_preview_pixels`) duplicates
  `raw::scan_for_preview`'s SOI-scan structure rather than sharing it,
  because the two need different per-candidate work (a header parse vs. a
  full decode). A future refactor could unify them behind one
  candidate-iterator if a third caller ever needs the same scan.

**Neutral**

- `rawPreview` reports its output through the same "browser bridges a
  capability via re-encoded bytes" shape `score()` already documents
  (DEC-065) — PNG bytes handed back, decoded through the engine's existing
  `info`/`transform`/`optimize_detailed` path. No new bridge pattern was
  invented.

## Validation

Right if: a synthetic RAW fixture (TIFF header + embedded JPEGs, generated
in-process — `raw_blob`/`jpeg_declaring` in `raw.rs`'s own tests, and
`tests/wasm_roundtrip.rs`'s duplicated equivalents) round-trips through
`rawPreview` inside a real wasm VM to the expected dimensions
(`raw_preview_extracts_largest_embedded_preview_as_png`); a preview declaring
more than 40 Mpix — but carrying only tiny real entropy, so a pass can only
come from the header peek — is rejected before decode
(`raw_preview_rejects_over_threshold_before_decode_and_extracts_under_it`,
which also proves a real preview under the threshold still extracts); a RAW
with no decodable preview at all gets the OTHER honest message
(`raw_preview_no_preview_is_distinct_from_too_large`); `isRawExtension`
matches `RAW_EXTENSIONS` exactly
(`is_raw_extension_mirrors_the_engine_list`); the browser smoke
(`tests/demo_smoke.mjs`) drives a real headless-Chrome drop of a synthetic
`.nef` through to a download with zero network requests, and separately
drives the over-threshold and no-preview fixtures to distinct, leak-free
error banners; `just wasm-npm-smoke` and the full native suite stay green;
`cargo build --no-default-features` is unaffected (the gate is wasm32-only).

Revisit if: a real low/mid-range mobile device test (the launch-readiness
step this decision explicitly defers) shows 40 Mpix is unsafe — lower the one
constant, no other code changes needed; or if a camera family's typical
preview size turns out to sit uncomfortably close to the gate, prompting a
per-format or configurable threshold instead of one flat constant.

## Amendment (2026-07-24, SPEC-104): retuned 40 → 60

The maintainer hit the 40 Mpix gate on a real desktop Leica `.DNG` (an 85 MB
file, ~47 MP embedded preview) — the very launch-readiness scenario this
decision's "Revisit if" clause anticipated, arriving faster than expected and
on desktop rather than mobile. Investigating showed 40 Mpix had conflated two
different bounds into one constant: it was sized as a **mobile** memory
ceiling (iOS Safari's per-tab budget), but applied as a **single global**
value to every visitor — so on desktop, where a ~320 MB preview decode is
trivial, the gate blocked for no measured reason.

`MAX_RAW_PREVIEW_MEGAPIXELS` is now **60**. 60 clears any realistic
Leica-class preview (including a 60 MP M11) in one shot while staying below
the native DEC-063 64 Mpix decode budget (`MAX_IMAGE_PIXELS`), so the "convert
with the CLI instead" fallback message stays honest — the CLI cannot decode
above 64 Mpix either, so the gate must never reach or exceed it. The
mechanism, the two fallback messages, and every native RAW/decode path are
unchanged; only the one constant moved.

This raise is **global**, not mobile-aware: the maintainer chose the simpler
across-the-board change over building platform detection now, explicitly
accepting that phone visitors can now attempt a larger preview decode
(~400 MB peak, extrapolating the probe's Leica measurement) than the original
40 Mpix value would have let through. **On-device mobile verification is
still the open item this decision deferred in 2026-07-24's original text** —
raising the number does not resolve it, and remains a launch-readiness
checklist item, not a build decision. Platform-aware (desktop-high /
mobile-conservative) gating stays a possible future refinement, out of scope
for this retune.

Tests moved to match: the over-threshold wasm fixture now declares ~62.4 Mpix
(between 60 and 64 Mpix, so it exercises this demo gate and not the native
cap — the SPEC-103 straddle lesson, reapplied at the new boundary), and
`raw.rs`'s native boundary test now straddles 60 Mpix instead of 40.

## References

- Supersedes: none
- Related specs: SPEC-103 (this), SPEC-061 (native RAW preview extraction),
  SPEC-069 (the DEC-063 SOF-peek hardening this reuses), SPEC-081/SPEC-101
  (the demo's score UI and smoke harness extended here)
- Related decisions: DEC-055 (RAW routes by extension — the reason
  `isRawExtension` takes a filename, not bytes), DEC-063 (the peak-decode-
  memory pixel budget and SOF-peek pattern this demo gate reuses at a lower,
  wasm-specific ceiling — native's 64 Mpix cap is unchanged), DEC-064 (the
  wasm boundary stays a thin additive shim — `rawPreview`/`isRawExtension`
  are glue over `crate::image::raw`, not new logic), DEC-065 (the "browser
  bridges a capability via re-encoded bytes" precedent `rawPreview`→PNG
  follows)
- Related constraints: `untrusted-input-hardening` (every path returns a
  typed `JsError`; no `unwrap`/`expect` on input-derived values; the scan
  stays bounded by `MAX_PREVIEW_CANDIDATES`), `ergonomic-defaults` (the
  fallback copy is plain, honest, and actionable for a non-expert)
