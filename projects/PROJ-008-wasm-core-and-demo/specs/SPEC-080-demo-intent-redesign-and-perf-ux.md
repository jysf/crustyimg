---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-080
  type: story
  cycle: design
  blocked: false
  priority: high
  complexity: M

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
  decisions: [DEC-020, DEC-064, DEC-065]
  constraints: [untrusted-input-hardening, ergonomic-defaults]
  related_specs: [SPEC-077, SPEC-078, SPEC-079]

value_link: >
  Turns the demo from "technically works" into "good enough to send strangers to": a photo shrinks
  in seconds instead of tens of seconds, the default result is smaller (never bigger), and a slow
  encode never reads as a hang.

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-080: demo intent redesign + perf UX

## Context

STAGE-029's measured investigation found the live demo mis-serves its most common visitor. This spec
is the **demo half** of the fix; it consumes the surface **SPEC-079** adds (`optimizeDetailed` with a
speed knob + Auto-picks-AVIF + a returned score). The problems it fixes, all page-side:

1. **The default fights the use case.** The default output is lossless WebP, which makes an
   already-lossy photo *bigger* — the opposite of what someone dropping a photo wants.
2. **A slow encode reads as a hang.** AVIF at rav1e speed 6 is ~33 s on a 12 MP photo; the busy state
   is a bare spinner with no time and no expectation set (SPEC-078). Measured fix: **speed 10**
   (3.6× faster) + an **offered resize** + honest, megapixel-keyed warnings + a **live elapsed timer**.
3. **No "keep the original" path.** When nothing beats the source, the demo should hand back the
   original, not a bigger file (the never-bigger rule — the demo has both byte counts already).

It is **demo files only** (`demo/index.html`, `demo/demo.js`, `demo/demo.css`, `demo/worker.js`) — no
engine change (SPEC-079 owns the surface; SPEC-081 owns the score UI).

## Goal

Reorient the demo around the intent **"make it smaller"**: default to the format that actually
shrinks the input (AVIF for photos, lossless for graphics, via SPEC-079's Auto), run AVIF at speed
10, **never present a bigger file** (keep the original and say so), and make a slow encode legible —
a megapixel-keyed expectation + warning, an **offered** (never forced) resize, a live elapsed timer,
and debounced re-conversion.

## Inputs

- **Files to read:** `demo/demo.js`, `demo/worker.js`, `demo/index.html`, `demo/demo.css` (the whole
  demo — small); `demo/README.md` (the candor to preserve).
- **The SPEC-079 surface (its dependency):** `optimizeDetailed(input, out_format, speed?, maxBytes?,
  target?) → OptimizeResult { bytes, format, quality, speed, score, scoredBy }`. Reconcile the exact
  exported names against SPEC-079 **as shipped** before building (it ships first).

## Outputs

- **Files modified:**
  - `demo/worker.js` — call `optimizeDetailed` (not `optimize`), pass **speed 10** for AVIF, thread
    an optional `maxBytes`; return the richer result (format/quality/speed/score) to the page.
  - `demo/demo.js` — the intent model: a primary **"Make it smaller"** action (Auto that shrinks);
    the **never-bigger** rule (if the result ≥ the input, offer the *original* for download and label
    it "already optimized — kept your file"); an **offered resize** for large inputs (a dismissible
    "this is N MP — most web uses need ~2 MP; resize to 2048px?" that sets `maxEdge`, never silently);
    a **megapixel-keyed** expectation/warning before a slow encode; a **live elapsed-seconds timer**
    in the busy state; **debounced** re-conversion so control fiddling doesn't queue jobs behind an
    in-flight encode.
  - `demo/index.html` — recast the controls around intent (a clear primary "smaller" path + the
    format/resize as secondary refinements); keep the format picker but no longer default to
    lossless-that-grows.
  - `demo/demo.css` — styling for the timer, the warning/expectation banner, the "kept original" state.

## Acceptance Criteria

- [ ] Dropping a **photo** (JPEG/PNG photographic) with defaults yields a **smaller** file (AVIF via
      SPEC-079 Auto) — not a bigger lossless one — and downloads with the right extension.
- [ ] When the best result is **≥ the input**, the demo shows a **"kept your original"** state and the
      download hands back the **original bytes**, with an honest one-line reason — it never downloads a
      bigger file by default.
- [ ] A **large** input (over a megapixel threshold, e.g. > ~6 MP) shows a **warning/expectation**
      ("N MP — AVIF runs a real codec in your browser; a few seconds") and **offers** a one-click
      resize to ~2048px; the full-resolution path stays available (warn, never silent cap).
- [ ] During a conversion the busy state shows a **counting-up elapsed timer** (honest — no fake %);
      the page stays responsive (the Web Worker guarantee from SPEC-078 holds).
- [ ] Rapidly changing a control does **not** stack conversions behind an in-flight encode (debounce /
      supersede); the newest request wins.
- [ ] AVIF conversions run at **speed 10** (visibly faster than 078's speed 6 on the same photo).
- [ ] The existing browser smoke (`just demo-smoke` / the headless-Chrome driver from SPEC-077/078)
      still passes end-to-end (drop → convert → download, zero network requests), updated for the new
      controls; hostile/edge inputs still surface a clean error, no hang.

## Failing Tests

Written at design; the demo's earned verdict is browser-driven (SPEC-077/078 precedent), not unit tests.

- **The headless-Chrome demo smoke (extend the SPEC-077/078 driver)**
  - `"photo_default_is_smaller_avif"` — drive a dropped photographic PNG on defaults; assert the
    result dataset shows `format == "avif"` and `outBytes < inBytes`.
  - `"never_bigger_keeps_original"` — drive an input the engine can't beat; assert the UI enters the
    "kept original" state and the download `blob` byte length == the input's (original handed back).
  - `"large_input_offers_resize"` — drive a > ~6 MP input; assert the resize offer / warning element
    is present and, when taken, the output dimensions are capped (~2048px long edge).
  - `"busy_state_shows_timer"` — assert an elapsed-time element updates during a conversion (a value
    that increases), and the page stays interactive (a control remains clickable — the 078 negative
    control).
- **Manual/verify (documented, driven at verify):** speed-10 is visibly faster than 078 on a real
  photo; the resize offer is dismissible; the never-bigger path is honest on an already-optimized JPEG.

## Implementation Context

### Decisions that apply
- `DEC-064`/`DEC-065` — the wasm surface + AVIF-encode-not-decode asymmetry; the worker already
  decodes `.avif` inputs via `createImageBitmap` (SPEC-078). Reuse that seam; add nothing to the wasm.
- `DEC-020` — rav1e speed; the demo passes **10** through SPEC-079's knob (the CLI stays 6).

### Constraints that apply
- `untrusted-input-hardening` — hostile/huge inputs surface a clean typed error already (SPEC-078);
  keep that — no hangs, no cryptic failures, with the new controls.
- `ergonomic-defaults` — the default must be the thing a photo-dropper wants (smaller), and choices
  must be honest (never silently bigger, never a silent cap).

### Prior related work
- `SPEC-077` (demo skeleton) / `SPEC-078` (Web Worker + AVIF + explain) — the page this reshapes; the
  worker, the `.avif`-input seam, the explain readout, and the browser smoke all come from here.
- `SPEC-079` (its hard dependency) — the `optimizeDetailed` surface. **Build SPEC-080 only after
  SPEC-079 ships**, and reconcile names against the shipped surface.
- STAGE-029 design note: **"never bigger = keep the original"** — the passthrough intent, mirrored
  from the CLI's `pick_winner`.

### Out of scope (for this spec specifically)
- Any `src/` / wasm change (SPEC-079 owns the surface).
- The SSIMULACRA2 **score readout UI** (SPEC-081) — this spec wires speed/format/never-bigger/timer;
  SPEC-081 adds the "here's the perceptual score" panel.
- A silent auto-cap of large images (decided: **warn + offer**, not cap).
- Mobile-specific layout beyond "it must not break" (the real-device test is a STAGE-028 human task).

## Notes for the Implementer
- **Keep the demo thin.** This is a marketing artifact, not a web app — recast the existing controls,
  don't add a framework or a pile of options.
- **Never-bigger is pure page logic:** the page already has `input.bytes` and `output.bytes`
  (`demo.js` render). If `output.bytes >= input.bytes`, switch the download to `source.file` (the
  original) and set the "kept original" copy. No engine round-trip.
- **Timer honesty:** count elapsed seconds; do **not** fake a percentage (one blocking rav1e call
  reports nothing — the SPEC-078 rationale stands).
- **Carry the candor** from `demo/README.md` into the new copy (WebP lossless-only, AVIF
  encode-not-decode). HN rewards it.

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
