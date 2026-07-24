---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-103
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-008
  stage: STAGE-029
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-5     # mechanical wiring; verify on Opus
  created_at: 2026-07-24

references:
  decisions: [DEC-055, DEC-063, DEC-064, DEC-065]
  constraints: [untrusted-input-hardening, ergonomic-defaults]
  related_specs: [SPEC-061, SPEC-069, SPEC-081, SPEC-101]

# One sentence on what this spec contributes to its stage's
# value_contribution.
value_link: >
  Closes the demo's RAW gap — the front-door pitch says "sharp and squoosh
  can't open these," so a photographer dropping a .dng must get a preview,
  not a leaked "Tiff is not supported" error.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-103: wire RAW decode into the demo behind a pixel gate

## Context

The browser demo **cannot open a RAW file today.** Drop a `.dng` (or any of the
15 `RAW_EXTENSIONS`) and it fails with a leaked internal error —
*"could not decode image: The image format Tiff is not supported."* This is the
carry logged at the end of SPEC-101, and it matters because the README front-door
pitch is explicitly *"sharp and squoosh can't open these at all"* — a photographer
testing the demo hits exactly the gap the pitch promises to fill.

**Mechanism (confirmed by the design-time probe, `docs/research/proj-008-raw-on-wasm-probe.md`):**
RAW routing on native is by file **extension** (`is_raw_extension(path: &Path)` →
`raw_preview` → `extract_preview`, `src/image/raw.rs`), because TIFF-based RAW is
byte-indistinguishable from a plain `.tif` (DEC-055). But every wasm entry point in
`src/wasm.rs` calls `Image::from_bytes(bytes)` with **no filename**, so a DNG sniffs
as TIFF, falls through to the generic decoder, and errors (the `tiff` decoder isn't
even linked into the wasm build, DEC-066). `raw.rs` has no `cfg` gate, and its
`extract_preview` only calls the JPEG decoder — already linked — so wiring it back
into the wasm reachable graph costs **+1,214 B brotli, measured** (the probe built it
through the real `just wasm-build` profile and diffed).

**The one real risk the probe could not close: mobile memory.** `extract_preview`
pulls the RAW's *largest embedded JPEG preview*, and that size is **camera-dependent,
not RAW-dependent**: a Leica DNG embeds a ~46.7 MP near-full-sensor preview
(~320 MB peak wasm memory to extract), while a Fujifilm RAF embeds only a 2.46 MP
screen-res preview (~110 MB peak). The probe measured these on Node's V8 on a desktop
Mac — **never on a real phone.** iOS Safari's per-tab memory ceiling is a moving
target no desktop estimate substitutes for, and this project has been burned before
by shipping unverified device-dependent claims
([[a-claimed-failure-mode-is-as-unproven-as-a-claimed-success]],
[[never-drive-the-maintainers-live-browser]]).

**Maintainer decision (2026-07-24):** wire it in, but **do not** block on a phone test
we cannot run before the feature exists (the demo rejects RAW today, so there is
nothing to phone-test until it's built). Gate it behind a cheap pre-decode pixel
check with an honest CLI fallback, ship via `main` (the demo redeploys, no tag), then
tune the threshold on-device as a launch-readiness step. The maintainer also said the
gate should **not be super-conservative** — permissive by default, catching only
genuinely huge previews, tightened only if a real phone shows it needs to be.

## Goal

Wire RAW embedded-preview extraction into the browser demo as an additive wasm
export, routed by file extension, and gated by a **single tunable pixel threshold**
that falls back to an honest "convert with the CLI" message for previews too large to
decode safely in a mobile tab. No change to the existing published `crustyimg-wasm`
API surface; no `src/` behavior change for native.

## Inputs

- **Files to read:**
  - `docs/research/proj-008-raw-on-wasm-probe.md` — the probe. Read it first; it has
    the measured numbers, the verified wiring shape, and the ruled-out alternatives.
  - `src/wasm.rs` — the wasm surface. The new export is added here; `score()`'s doc
    (lines 499–514) is the sanctioned precedent for "browser bridges a capability via
    re-encoded bytes."
  - `src/image/raw.rs` — `extract_preview` (line 106), `scan_for_preview` (130),
    `decode_jpeg_with_limits` + the **SOF dimension peek** (206–217, DEC-063),
    `is_raw_extension` (88), `RAW_EXTENSIONS` (61), and the existing test
    `raw_preview_rejects_oversize_embedded_jpeg_before_decode` (374) — the exact
    "reject an oversize preview *before* the full decode" pattern this spec reuses at
    a lower threshold.
  - `src/image/mod.rs` — `raw_preview(bytes) -> Result<Image>` (457), the public byte
    entry the new export calls; `decode_limits()` (the production DEC-034 caps).
  - `demo/demo.js` — the load/decode/convert path; `source.file.name` is already in
    hand at load (probe: lines ~320, 327, 471, 542), `showError` surfaces `e.message`
    verbatim (~278), `scheduleConvert`→`convert()` is the existing rerun path
    (~520–536).
  - `demo/index.html` — the file-input `accept` attribute (line 40).
  - `tests/wasm_roundtrip.rs` and `tests/demo_smoke.mjs` — the two harnesses to extend.
- **Related code paths:** `src/image/raw.rs`, `src/wasm.rs`, `demo/`.

## Outputs

- **New wasm exports (`src/wasm.rs`), additive only:**
  - `rawPreview(input: &[u8]) -> Result<Vec<u8>, JsError>` — extract the RAW's
    largest embedded JPEG preview and return it **PNG-encoded** (so the demo feeds it
    straight into the existing `optimizeDetailed`/`transform`/`info` pipeline, the
    same bridge shape `score()` documents). **Before** the full decode, peek the
    largest candidate's declared SOF dimensions and, if the pixel count exceeds the
    demo threshold (below), return a typed error the demo recognizes as the
    "too large" case — distinct from a genuinely-undecodable RAW.
  - `isRawExtension(name: &str) -> bool` — a thin wrapper over `raw::is_raw_extension`
    so the JS routing list **cannot drift** from `RAW_EXTENSIONS`. (Expose the
    existing `pub(crate)` fn via a `&str`/filename-taking wrapper; do not duplicate
    the list in JS.)
- **New threshold constant + gate (`src/wasm.rs`, or a wasm-cfg'd helper):**
  - A single named constant — proposed `MAX_RAW_PREVIEW_MEGAPIXELS` (or a `_PIXELS`
    count) — that is the **one place** the gate is tuned. Framing default: **40 MP**
    (see the DEC below and Notes). The gate peeks declared dimensions and rejects
    **before** allocating the full-resolution decode.
- **`src/image/raw.rs` (small, additive):** whatever minimal surface the gate needs
  to peek the largest embedded preview's declared dimensions *without* a full decode
  (e.g. a `pub(crate)` dimension-scan that reuses the existing per-candidate SOF peek
  in `decode_jpeg_with_limits`, taking the max across plausible candidates). Keep the
  scan bounded by `MAX_PREVIEW_CANDIDATES` exactly as `scan_for_preview` is.
- **Files modified (demo):**
  - `demo/demo.js` — on file load, if `isRawExtension(source.file.name)`, route
    through `rawPreview` instead of the normal decode; on the typed "too large" error,
    show the honest CLI-fallback copy (draft below) rather than the raw engine string.
  - `demo/index.html` — add the RAW extensions to `accept` (extension tokens; RAW has
    no broadly-standard `image/*` MIME) for picker parity with drag-and-drop. Note:
    drag-drop bypasses `accept` entirely (that's how the original symptom happened),
    so `accept` is cosmetic parity, not the gate.
- **New DEC (emitted at build):** `DEC-082` — RAW preview extraction on the
  wasm/demo surface, gated by a demo-specific embedded-preview pixel budget. Records
  the threshold value + rationale, notes it is tuned on-device post-ship, and that it
  is a demo/wasm ceiling layered *below* the native DEC-063 decode cap (native is
  unchanged). Relates to DEC-055 (extension routing required), DEC-063 (the SOF peek
  reused), DEC-064 (additive wasm boundary), DEC-065 (encode-in/decode-out bridge
  precedent).

## Acceptance Criteria

- [ ] **RAW opens in the demo.** Dropping/picking a RAW file whose embedded preview
      is within the threshold produces a real preview and flows through the normal
      convert path — no "Tiff is not supported" leak. Proven by driving the demo smoke
      with a synthetic RAW fixture (TIFF header + embedded JPEG, the `raw_blob` shape).
- [ ] **The gate rejects an oversize preview BEFORE the full-resolution decode** —
      not after allocating it. Proven by a `wasm_roundtrip` test using a preview whose
      SOF header *declares* dimensions over the threshold while carrying tiny entropy
      (the `jpeg_declaring` bomb shape already in `raw.rs`), asserting the typed
      "too large" error comes back without the multi-hundred-MB allocation. This is
      the DEC-063 pattern at the demo threshold.
- [ ] **The "too large" case is distinguishable from "no decodable preview."** The
      demo shows the honest CLI-fallback message for an over-threshold preview, and a
      *different* honest message for a RAW with no decodable embedded JPEG at all
      (`ImageError::Decode`). Neither leaks a CLI-flavored `raw: …` internal string
      into the browser banner ([[comments-plain-no-spec-refs]]).
- [ ] **`isRawExtension` matches `RAW_EXTENSIONS` exactly** — `isRawExtension("x.dng")`,
      `("photo.CR2")` (case-insensitive) are `true`; `("x.png")`, `("x.tif")`,
      `("x")` are `false`. The JS routing list is derived from this export, not a
      hand-copied array (assert no second copy of the list exists in `demo/`).
- [ ] **The published API is untouched.** `info`, `transform`, `optimize`,
      `optimizeDetailed`, `score`, `version` keep their exact signatures; the new
      exports are purely additive. `just wasm-npm-smoke` still passes.
- [ ] **No native `src/` behavior change.** Native decode/routing is unchanged; the
      threshold gate is wasm-only. Full native gate suite green.
- [ ] **Bundle cost stays negligible** — within a small margin of the probe's measured
      +1,214 B brotli (JPEG decoder already linked; the gate adds no codec). Report the
      real `just wasm-build` brotli delta.
- [ ] **Zero network requests during conversion still holds** (the RAW path is
      all-local, same as every other input).
- [ ] Browser smoke + `just validate` green; DCO-signed commits; wasm built through
      `just wasm-build` (the size profile — bare `cargo build --target wasm32` ships
      +109 KB, DEC-066/SPEC-074).

## Failing Tests

Written during **design**, before build.

- **`tests/wasm_roundtrip.rs`**
  - `"rawPreview extracts the largest embedded preview as a valid PNG"` — a synthetic
    `raw_blob((160,120) thumb, (1024,768) preview)` in → `rawPreview` returns bytes
    that decode as a 1024×768 PNG (assert dims via an independent decode, per
    [[verify-wasm-output-with-an-independent-decoder]]).
  - `"rawPreview rejects an over-threshold preview before decoding it"` — a `raw_blob`
    whose only preview *declares* dims over `MAX_RAW_PREVIEW_MEGAPIXELS` (via the
    `jpeg_declaring` patch) but carries 16×12 entropy → returns the typed "too large"
    error, and does so via the header peek (no full decode / no large allocation).
    Mutation check: a preview *just under* the threshold extracts normally — proving
    the gate isn't vacuous.
  - `"rawPreview surfaces no-preview distinctly from too-large"` — a `raw_blob` with a
    TIFF header and no decodable JPEG → the "no decodable preview" error, a *different*
    typed value than the over-threshold one.
  - `"isRawExtension mirrors RAW_EXTENSIONS"` — `dng`/`CR2`/`raf`/`nef`… → `true`;
    `png`/`tif`/empty → `false`.
- **`tests/demo_smoke.mjs`** (headless, the SPEC-078/101 harness)
  - `"a synthetic RAW file drops, previews, and converts"` — a generated RAW-shaped
    fixture routes through `rawPreview` and yields a downloadable output; **0 network
    requests** during the flow.
  - `"an over-threshold RAW shows the honest CLI-fallback copy, not a decoder leak"` —
    the banner shows the fallback message; the string `Tiff is not supported` and the
    prefix `raw:` never appear in the UI.
  - `"index.html accept lists the RAW extensions"` — structural assert the `accept`
    attribute contains `.dng`/`.cr2`/… .

## Implementation Context

### Decisions that apply
- `DEC-055` — RAW routes by **extension** (byte-ambiguous with TIFF); content-sniffing
  is ruled out for the format family, so `isRawExtension` on the filename is the
  correct discriminator. Do not try to sniff RAW from bytes in `from_bytes`.
- `DEC-063` — the SOF-dimension peek that rejects an oversize embedded preview before
  the ~GB decode. The demo gate is the same mechanism at a lower, wasm-specific ceiling.
- `DEC-064` — the wasm boundary is a thin **additive** `wasm-bindgen` shim; glue, not
  logic. The new exports must not re-implement engine behavior.
- `DEC-065` — the "browser bridges a capability via re-encoded bytes" precedent
  (`score()` decodes AVIF in-browser, hands PNG back). `rawPreview`→PNG is the same
  shape, Rust doing the extraction.

### Constraints that apply
- `untrusted-input-hardening` — **a panic aborts the wasm module and kills the page's
  engine instance.** Every path returns a typed `JsError`; never `unwrap`/`expect` on
  input-derived values. The scan stays bounded (`MAX_PREVIEW_CANDIDATES`).
- `ergonomic-defaults` — the fallback message helps a non-expert; plain, honest,
  behavior-first, no internal symbols.

### Prior related work
- `SPEC-061` / `SPEC-069` — the native RAW preview path + the F-RAW-1 SOF-peek
  hardening this reuses.
- `SPEC-081` / `SPEC-101` — the demo's convert/score UI and smoke harness to extend.
- The probe: `docs/research/proj-008-raw-on-wasm-probe.md`.

### Out of scope (for this spec specifically)
- **Any on-device phone test.** The threshold ships at its framing default and is
  tuned later on real hardware as a launch-readiness step — a maintainer checklist
  item, not a build acceptance criterion ([[never-drive-the-maintainers-live-browser]]).
- **RAW metadata / EXIF** through the preview (native already defers this, `raw.rs`
  bundle is `None`).
- **CR3 verification on a real file** (no sample; DEC-055 covers it by the same scan,
  and the synthetic tests exercise the mechanism).
- **`.x3f`** (Sigma Foveon, no baseline-JPEG preview — deliberately not in
  `RAW_EXTENSIONS`; falls through to the normal unsupported-format error).
- Any native routing change, new format, or `optimizeDetailed` change.

## Notes for the Implementer

- **The threshold is a product decision — surface it, don't bury it.** Framing default
  is **40 MP**: it passes the measured-safe 2.46 MP RAF and typical full-frame
  previews (24 MP), and catches the measured-risky 46.7 MP Leica. The maintainer
  asked for permissive-not-conservative, so err high, keep it ONE named constant, and
  record the safe/risky bracket (2.46 MP OK ↔ 46.7 MP ≈ 320 MB peak) in DEC-082 so
  whoever tunes it on a phone has the data. **Confirm the exact default number with
  the maintainer before shipping** — it's the one value the whole spec pivots on.
- **User-facing copy needs a maintainer draft-review** ([[comments-plain-no-spec-refs]]).
  Draft for the "too large" fallback: *"This RAW's built-in preview is very
  high-resolution — convert it with the crustyimg CLI instead."* Draft for
  "no preview": *"Couldn't find a preview image inside this RAW file."* Show these
  before finalizing; keep them on the demo's plain voice.
- **Gate before allocating.** The whole point of the pre-check is to *not* allocate
  ~232 MB for a preview you're about to reject. Peek declared SOF dims first; reject
  on the header, exactly like `raw_preview_rejects_oversize_embedded_jpeg_before_decode`.
- **Single-source the extension list.** `isRawExtension` exists so `demo.js` never
  hand-copies `RAW_EXTENSIONS`. If you find yourself writing `['dng','cr2',…]` in JS,
  stop and call the export.
- **Build through `just wasm-build`** (size profile). Verify the brotli delta against
  the probe's +1,214 B; a large jump means a codec got pulled in that shouldn't have.
- **rtk footgun:** cross-check any grep sweep with raw `grep` + a positive control —
  the rtk hook silently zeroes `rg -c` counts here
  ([[rtk-can-silently-corrupt-grep-counts]]).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-082` — RAW preview on wasm behind a demo pixel gate (if built)
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

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>
2. **Does any template, constraint, or decision need updating?**
   — <answer>
3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
