---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-105
  type: bug                        # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
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
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 640000
      estimated_usd: 4.10
      note: >
        Order-of-magnitude estimate (main-loop build, not a separately-metered
        subagent — AGENTS §4). One session: reproduce the bug on the real Leica
        DNG, generate + measure a real photo/graphic calibration distribution
        (~70 images via optimize --explain=json), implement the rule, redesign two
        synthetic tests, commit fixtures, add native + wasm + unit tests, amend
        DEC-047. Opus 4.8 list rate, ~80/20 in/out, no cache discount.
    - cycle: verify
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 143805
      estimated_usd: 5.7
      recorded_at: 2026-07-25
      note: >
        Verify dispatched as a metered sub-agent (Agent tool, Opus); tokens_total is the
        REAL subagent usage count. VERDICT CLEAN: verified the classifier fix against 64
        real Nikon RAWs (the color case) + the Leica (grayscale), mutation-tested the 4.0
        threshold, confirmed no genuine graphic regressed. NOTE: verify passed the feature
        matrix only via stale incremental-build artifacts (a false green); CI caught real
        no-AVIF-leg test breakage, corrected in a follow-up fix pass (see the ship note).
    - cycle: ship
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: null
      estimated_usd: 12.0
      recorded_at: 2026-07-26
      note: >
        Orchestrator main-loop — merge + a LARGE CI-parity fix pass. SPEC-105's build
        and verify both false-greened the feature matrix on stale incremental-build
        artifacts, so CI (clean builds) caught real breakage the classifier change had
        exposed: json_shape_consistent_across_verbs (golden requires a scored AVIF
        winner) and 3 cli tests (grayscale/ICC/never-bigger fixtures reclassified
        graphic→photo → AVIF/JPEG/lossy-WebP by leg), plus a dead-code helper on the
        no-avif legs, plus a missing verify cost session. Diagnosed each against CI logs
        + reproduced with CLEAN local full-matrix builds, fixed codec-agnostically,
        merged (PR #113, 54ba05e). Order-of-magnitude estimate; the cost was diagnosis +
        repeated ~10-min CI cycles, not the fix size. Lessons banked (see Reflection).
  totals:
    tokens_total: null
    estimated_usd: 21.8
    session_count: 4
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

- **Branch:** `spec-105-entropy-photo-classify`
- **PR (if applicable):** #113
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - `DEC-047` amended (strong-entropy Photograph signal + calibration table + accepted crossings)
- **Chosen threshold:** `PHOTO_ENTROPY_STRONG = 4.0` bits (luma entropy), one new constant in
  `src/analysis/mod.rs`; the rule fires after EXIF (rule 2) + Document (rule 3), before the graphic
  gates (rule 4), returning `Photograph` at confidence 0.8.
- **Calibration table (entropy per validation image, all EXIF-stripped, measured via
  `optimize --explain=json`):**

  | Class | Image | Entropy | Class w/ fix |
  |---|---|---|---|
  | photo (gray, committed) | `grayscale_photo_leica.png` (the Leica B&W subject) | 6.07 | photograph ✓ |
  | photo (gray, committed) | `grayscale_photo_canon.png` | 6.83 | photograph ✓ |
  | photo (colour, committed) | `color_photo_fuji.png` | 6.37 | photograph ✓ |
  | photo distribution (48 real crops) | Canon/Fuji/Nikon, gray+colour | floor **4.58**, median ~6.8, max 7.6 | photograph ✓ |
  | graphic | solid fill | 0.00 | graphic-logo ✓ |
  | graphic | text on flat | 0.39 | document ✓ |
  | graphic | simple logo (shapes) | 0.96 | graphic-logo ✓ |
  | graphic | realistic UI dashboard screenshot | 1.56 | (lossless) ✓ |
  | graphic (dither) | 2-colour ordered dither | 1.00–1.50 | graphic/document ✓ |
  | graphic (dither, committed) | `dithered_graphic.png` (8-colour Floyd–Steinberg) | **3.03** | graphic-logo ✓ |
  | graphic (dither) | 16-colour Floyd–Steinberg | 3.43 | graphic-logo ✓ |
  | **crossing (accepted)** | 32-colour F-S dither of a photo | 5.14 | photograph (a dithered *photo*, lossy-safe) |
  | **crossing (accepted)** | smooth full-frame gradient | ~7.5 | photograph (no hard edges, lossy-safe) |

  **The threshold `4.0` sits in the gap `(3.43, 4.58]`** — above every realistic hard-edged graphic
  (≤1.6, or ≤3.43 counting dithers-of-photos) and below the real-photo floor (4.58). A
  `calibration_gap_holds_for_committed_fixtures` unit test locks `graphic_max < 4.0 ≤ photo_min`.
- **Symptom fixed end-to-end?** Yes. Native `optimize --explain=json --max 2048` on the real
  `_incoming0/L1024678.DNG`: `class: photograph`, winner **AVIF 63,894 B** (was 843,252 B lossless
  WebP — the 13× fix, matching the probe's ~62 KB). Wasm `optimize_detailed(auto)` on the committed
  grayscale fixture → **AVIF**, not lossless WebP. The classifier is shared, so both paths route
  identically.
- **Deviations from spec:**
  - Two *existing* synthetic tests used full-range gradient/noise as stand-ins for a "screenshot"
    (entropy 7.2) and an "ambiguous" image (entropy 7.6). The strong-entropy rule correctly reads
    those as photographic, so both fixtures were replaced with **realistic low-entropy**
    constructions (an iso-luma tint keeps luma entropy low while pushing the colour count past 256).
    Each now asserts `entropy < PHOTO_ENTROPY_STRONG` so it exercises its intended class, not the new
    rule. This is a fix, not a regression: the old synthetics were exactly the unrepresentative
    fixtures the memory lessons warn about.
  - The spec's suggested "~5" threshold would have *missed* real photos at the 4.58 floor; the
    measured distribution drove `4.0` instead. Recorded in DEC-047.
  - Three more existing fixtures relied on the same gradient misclassification and were updated to
    genuine content (the fix's correctness, surfaced by their green→red flip):
    (a) `tests/cli.rs` never-bigger passthrough now uses a committed fine-checkerboard JPEG
    (`checker_graphic.jpg`) — a genuine low-entropy graphic whose lossless candidates blow up, so it
    truly passes through; (b) the never-bigger ICC test now asserts the corrected photograph path
    (compact lossy AVIF/JPEG, ICC stripped, never a lossless blow-up) — the SPEC-084 blow-up scenario
    is only reachable via the misclassification this spec removes; (c) `just demo-smoke`'s
    "lossless shows no fabricated score" check used `makePng` (a gradient) as its "graphic"; it now
    uses a new `makeGraphicPng` (flat solid-colour blocks, entropy 2.34 → GraphicLogo → lossless).
- **Follow-up work identified:**
  - Scale-normalize the flat/edge detector (tuned on 64×64 synthetics; reads every megapixel photo as
    ~flat). This rule *masks* it for photos but does not fix it.
  - Carry EXIF through the RAW-preview decode (would restore the camera prior for RAW directly).
  - A broad, diverse labelled corpus to re-confirm the 4.0 anchor beyond this small validation set —
    in particular, whether an even lower-contrast real photo can dip below 4.0 (would fall back to
    lossless: a bigger file, the safe direction).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — The spec (and probe) framed the entropy gap as "wide and clean" (~7.45 photo vs ~1.0 graphic)
   and suggested a ~5.0 threshold. Measuring real material showed the gap is real but *narrower and
   two-sided*: the photo floor is ~4.58 (a low-contrast colour crop), a smooth gradient is a
   high-entropy *graphic* (~7.5), and a heavy error-diffusion dither of a photo climbs to ~5.1. A
   naive "~5" would have missed real photos. The spec's own note ("MEASURE it, don't assume") was the
   right instinct, and it paid off.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The interaction with the *UI-screenshot* rule (rule 5). The spec said fire "before the graphic
   gates (rule 4)", which also places the new rule before rule 5. The two existing high-entropy
   synthetic UI/ambiguous fixtures broke, and it wasn't obvious until running the suite that they were
   gradients/noise masquerading as their class. Worth a one-line heads-up that adding an
   ahead-of-rule-4 rule also preempts rules 5–6.

3. **If you did this task again, what would you do differently?**
   — Measure the two *existing* synthetic classify tests' entropy up front (they were the only
   surprises), rather than discovering the breakage after implementing. And I'd reach for `rtk proxy`
   immediately for any command whose real stdout/stderr I need — the rtk summarizer silently ate
   `--nocapture` eprintln output and a couple of glob qualifiers, costing a few iterations.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — The classifier feature was sound and verified against 64 real Nikon RAWs; the entire
   overrun was TEST-SCAFFOLDING fallout, made far worse by a **stale-incremental-build false
   green**. Build and verify both ran `cargo test` against incrementally-compiled artifacts
   and reported the feature matrix passing — but a clean CI build caught real breakage: a
   golden-shape test needing a scored AVIF winner, and three fixtures that SPEC-105 correctly
   reclassifies graphic→photo (so their winners move to AVIF/JPEG/lossy-WebP by feature leg).
   Two hard rules for next time: (a) **an engine change that touches the shared classifier
   requires a CLEAN full-matrix run** (`cargo test` on default / lean / webp-lossy + clippy on
   each + fmt) before it's trusted — incremental builds lie; (b) the **orchestrator
   independently re-runs that clean matrix** rather than relaying a sub-agent's "CLEAN." I did
   neither up front and paid for it in ~6 CI cycles.

2. **Does any template, constraint, or decision need updating?**
   — The **verify prompt** for any engine/shared-code change must mandate a clean full feature
   matrix (delete `target` or `--offline` fresh, not incremental) and forbid concluding from a
   possibly-stale local pass. Two process lessons also earned real cost today and are worth
   standing rules: **never push to `main` between a verify and its merge** (each push knocked
   the PR "behind" → an extra ~10-min CI cycle), and **a sub-agent can leave uncommitted work
   in the shared checkout** — check `git status` before trusting the branch state (a `git
   reset --hard` here discarded the verify agent's uncommitted codec-agnostic fixes, which I
   then had to reconstruct). Synthetic fixtures also bit again exactly as SPEC-105 warns:
   `detailed_png` (high-frequency noise) *inflates* under lossy re-encode, so it never behaves
   like a photo — real or downscale-driven sources only.

3. **Is there a follow-up spec I should write now before I forget?**
   — Two carried, both filed: **scale-normalize the flat/edge detector** (still tuned on 64×64
   synthetics — the strong-entropy rule masks it for photos but doesn't fix it) and **carry
   EXIF through the RAW-preview decode** (would restore the camera prior for RAW directly). Both
   post-launch. Also the maintainer's **batch-report flag** request (a `--report` summary of a
   multi-file run) → `docs/backlog.md`.
