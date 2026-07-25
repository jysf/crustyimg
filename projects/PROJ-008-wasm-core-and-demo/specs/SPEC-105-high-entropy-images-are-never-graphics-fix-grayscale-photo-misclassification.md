---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-105
  type: bug                        # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
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
  implementer: claude-opus-4-8     # engine/classifier change + threshold calibration; verify on Opus
  created_at: 2026-07-25

references:
  decisions: [DEC-047]
  constraints: [ergonomic-defaults]
  related_specs: [SPEC-102]

value_link: >
  The demo's headline "drop a photo → watch it just work" mis-encodes a real
  grayscale photograph as a 13× oversized lossless file. Fix the shared content
  classifier so a high-entropy image is never treated as a graphic.

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-105: high-entropy images are never graphics — fix grayscale photo misclassification

## Context

A real Leica B&W portrait DNG, dropped into the demo's **Auto** flow, came back as
**lossless WebP, 823.5 KB** instead of a lossy AVIF (~62 KB) — 13× too large, labelled
"web-ready." A design-time probe reproduced and dissected it:
`docs/research/proj-008-grayscale-photo-misclassification-probe.md` (read it first).

**Root cause (in the SHARED classifier `src/analysis/mod.rs::classify`, DEC-047 — so the
native CLI mis-encodes identically, proven byte-for-byte, not just the demo):**

1. **A grayscale image has ≤256 distinct RGB colours** (r=g=b), so rule 4 clause 1
   (`few_colors = !saturated && n <= PALETTE_COLORS`, `mod.rs:584`) fires → `GraphicLogo`
   → lossless — no matter how detailed the photo is.
2. **EXIF-stripping removed the safety net.** The camera prior (rule 2,
   `if has_exif → Photograph`, `mod.rs:572`) normally classifies camera JPEGs *before* the
   palette gate. RAW preview-extraction drops EXIF, so that rule never fires — which is why
   the bug is invisible on JPEGs and surfaces on RAW.
3. **A latent third problem the probe surfaced:** the edge/flat detector's thresholds were
   tuned on 64×64 synthetics, so at megapixel resolution *every* photo reads ~96% "flat"
   and rule 4 clause 2 (`flat_ratio >= FLAT_GRAPHIC_RATIO && edge_ratio < GRAPHIC_EDGE_MAX`)
   also mis-fires on EXIF-stripped *colour* photos. **The EXIF prior has been masking a
   scale-broken flat detector.** (Fixing that detector is a separate, larger spec — see
   Out of scope. This spec makes it harmless for photos.)

The discriminator is **entropy**: a B&W photo measures ~7.45; a logo ~0.95, a document
~1.0 — a wide, clean gap.

## Goal

Make the shared classifier treat a **high-entropy image as a photograph, never a graphic**
— fixing the grayscale-photo symptom *and* the broader EXIF-stripped-photo exposure at
once, WITHOUT regressing genuine low-entropy graphics/logos/documents (crustyimg's
"doesn't blindly AVIF everything" differentiator). One calibrated threshold, validated
against a small set of REAL grayscale photos and real graphics.

## Inputs

- **Files to read:**
  - `docs/research/proj-008-grayscale-photo-misclassification-probe.md` — the probe (root
    cause, isolation matrix, measured entropy anchors, cost, recommended direction).
  - `src/analysis/mod.rs` — `classify()` (the 7-rule first-match cascade, ~544-605), the
    threshold constants near the top of the module (`PALETTE_COLORS`, `PHOTO_ENTROPY`,
    `DOC_ENTROPY_MAX`, `FLAT_GRAPHIC_RATIO`, `GRAPHIC_EDGE_MAX`, `PHOTO_FLAT_MAX`, …),
    `entropy` computation, and `ImageClass`→`OptBucket` mapping.
  - `src/analysis/decide.rs` — how `OptBucket` → `format_shortlist`/AVIF admission; and
    `src/wasm.rs` `auto_avif_quality` (AVIF only for `Lossy`/`MixedSafe`).
  - `decisions/DEC-047-*.md` — the deterministic classification cascade this amends.

## Outputs

- **`src/analysis/mod.rs`:** add a **strong-entropy → `Photograph`** signal that fires
  **ahead of the graphic gates** (rule 4), so a high-entropy image — grayscale or colour,
  EXIF or not — is classified `Photograph` before `few_colors`/flat-graphic can catch it.
  Equivalently expressed as an entropy floor on the graphic rules; pick whichever reads
  cleanest in the cascade, but the resulting behaviour is "high entropy ⇒ not a graphic."
  A new named constant, e.g. `PHOTO_ENTROPY_STRONG`, is the single calibrated knob.
  Do NOT touch the Document rule's own entropy bound or the EXIF rule.
- **`decisions/DEC-047` amendment:** a dated note recording the new rule + threshold, the
  entropy anchors it was calibrated against (real photos vs real graphics), and that the
  scale-broken flat detector is now masked-for-photos-but-not-fixed (its own follow-up).
- **Calibration record** (in the spec's Build Completion or a short `docs/research/`
  appendix): the measured entropy values for the validation set, showing the chosen
  threshold cleanly separates every real photo (above) from every real graphic (below).

## Acceptance Criteria

- [ ] **The symptom is fixed end-to-end.** The Leica B&W preview (native
      `optimize --explain --max 2048` on `_incoming0/L1024678.DNG`, or its extracted
      preview) classifies as **`Photograph`** and its winner is a **lossy** format (AVIF),
      not lossless WebP — reproducing the probe's ~62 KB / SSIMULACRA2 ~83 result, not 823 KB.
- [ ] **A real high-entropy grayscale photo → `Photograph` → AVIF-admitting bucket**, via a
      committed **real** small grayscale-photo fixture (a downscaled crop the maintainer
      owns — NOT a synthetic, because synthetics are what mis-tuned this in the first place).
- [ ] **Genuine graphics still classify lossless (no regression).** A real/low-entropy logo,
      a document-like image, and a UI screenshot each keep their existing class
      (`GraphicLogo`/`Document`/`UiScreenshot` → lossless). Proven by fixtures, including at
      least one **dithered / noisy** graphic (the known risk: dithering raises entropy — if
      it crosses the threshold and mis-routes to lossy, the threshold or the rule needs
      adjusting, documented).
- [ ] **The threshold is calibrated, not guessed.** The Build records the entropy of every
      validation image and shows `PHOTO_ENTROPY_STRONG` sits in the gap between the
      highest-entropy real graphic and the lowest-entropy real photo.
- [ ] **Native and wasm both fixed** (shared layer): the native `optimize`/`web` path and
      the wasm `optimize_detailed` Auto path both route the grayscale photo to AVIF.
- [ ] Full native gate suite green; `just wasm-test`, `just demo-smoke`, `just validate`,
      `cargo build --no-default-features` all green. No unrelated behaviour change.

## Failing Tests

- **`src/analysis/mod.rs` (`mod tests`)**
  - `"a high-entropy grayscale image is a Photograph, not a GraphicLogo"` — an image with
    entropy above the new threshold and ≤256 colours classifies `Photograph`. Prefer a
    committed real grayscale-photo crop; a high-entropy grayscale texture is an acceptable
    *supplement*, not the sole evidence (the bug was a synthetic-vs-real gap).
  - `"a low-entropy few-colour graphic stays GraphicLogo"` — a solid/flat logo-like image
    stays `GraphicLogo` (regression guard for the differentiator).
  - `"a dithered graphic does not mis-route to Photograph"` — or, if it does at the chosen
    threshold, the test documents the accepted tradeoff explicitly.
  - `"a document-like bimodal image stays Document"` and a UI screenshot stays
    `UiScreenshot` — the other lossless classes are untouched.
- **`tests/` (integration, native)** — `optimize --explain=json` on the real grayscale
  photo fixture reports a lossy winner (AVIF) and `class: photograph`, not
  `graphic-logo`/`webp lossless`.
- **wasm (`tests/wasm_roundtrip.rs`)** — the grayscale photo through `optimize_detailed`
  Auto yields AVIF (a scored/lossy result), not lossless WebP.

## Implementation Context

### Decisions that apply
- `DEC-047` — the deterministic, no-ML classification cascade (this spec amends it: adds
  the strong-entropy Photograph signal). Keep the "safe fallback is Photograph; a graphic
  forced lossy is the costly direction" philosophy — the entropy rule protects exactly that.

### Constraints that apply
- `ergonomic-defaults` — Auto must make the correct format call for the common case (a
  photo), without the user knowing to override.

### Prior related work
- `SPEC-102` — established "AVIF wins photographs; lossless wins small/simple graphics —
  that IS the content-aware branch working." This spec fixes a case where that branch
  mis-fired; it must not break the genuinely-graphic side.
- The probe: `docs/research/proj-008-grayscale-photo-misclassification-probe.md`.

### Out of scope (separate follow-ups)
- **Scale-normalizing the edge/flat detector** (the thresholds tuned on 64×64 synthetics
  that read every megapixel photo as ~flat). Real, but the strong-entropy rule makes it
  harmless for photos; it's a larger correctness spec of its own, post-launch.
- **Carrying EXIF through the RAW-preview decode** (would restore the camera prior for RAW,
  a narrower band-aid) — separate.
- **A full diverse benchmark corpus** — this spec needs only a *small* validation set to
  calibrate the threshold; the broad corpus is the existing content-diversity backlog item.

## Notes for the Implementer

- **The calibration set must include REAL images on the photo side.** Synthetic 64×64
  fixtures are what mis-tuned the flat detector; do not repeat that. Use small,
  self-owned, downscaled real grayscale photo crops (the maintainer's — ask if you need
  them; `_incoming0` + `~/Import/Photos` have material) for the photo side. The graphic
  side may be generated (real graphics ARE low-entropy flat by nature: solid fills, simple
  shapes, text on flat backgrounds) — but include at least one **dithered** graphic, the
  one plausible high-entropy graphic, as the adversarial case.
- **The gap is wide** (photo ≥ ~6.5, graphic ~1.0), so a threshold around ~5 is likely
  safe — but MEASURE it, don't assume; report every value.
- **Place the rule so it can't mis-order.** It must fire after the EXIF and Document rules
  (a low-entropy document must not be pulled into Photograph) and before the graphic gates.
  Confirm the Document rule (`entropy < DOC_ENTROPY_MAX`) and the new floor don't overlap.
- **Prove native AND wasm.** The classifier is shared; assert the fix on both paths.
- rtk footgun: cross-check greps with raw `grep` + a positive control; drive conclusions
  off `optimize --explain=json`, not greps, as the probe did.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-047` amended (strong-entropy Photograph signal)
- **Calibration table (entropy per validation image, chosen threshold):**
  - [fill in]
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [scale-normalize the flat/edge detector; EXIF-through-RAW; diverse corpus]

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
