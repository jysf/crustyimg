# SPEC-105 — BUILD prompt

Cycle: build. You are NOT the architect. The spec is your primary context. This is an
ENGINE change to the shared content classifier — judgment + calibration, not mechanical.

Read in order:
1. `/AGENTS.md` — conventions (fixtures, DCO, `just` recipes).
2. `/projects/PROJ-008-wasm-core-and-demo/specs/SPEC-105-high-entropy-images-are-never-graphics-fix-grayscale-photo-misclassification.md`
   — the whole spec, esp. `## Notes for the Implementer`.
3. `/docs/research/proj-008-grayscale-photo-misclassification-probe.md` — the probe (root
   cause, isolation matrix, measured entropy anchors: photo ~7.45, logo ~0.95, doc ~1.0).
4. `src/analysis/mod.rs` (`classify()` cascade + threshold constants + `ImageClass`→
   `OptBucket`), `src/analysis/decide.rs`, `src/wasm.rs` (`auto_avif_quality`).
5. `decisions/DEC-047-*.md` (you amend it).

## What to build

- Branch `spec-105-entropy-photo-classify` off `main`.
- Add a **strong-entropy → `Photograph`** signal to `classify()` that fires **after** the
  EXIF (rule 2) and Document (rule 3) rules but **before** the graphic gates (rule 4), so a
  high-entropy image (grayscale or colour, EXIF or not) is never `GraphicLogo`. One new
  calibrated constant (e.g. `PHOTO_ENTROPY_STRONG`). Keep DEC-047's "fallback is
  Photograph; forcing a graphic lossy is the costly direction" philosophy.
- **Calibrate the threshold against REAL images, and RECORD every value.** Photo side:
  small self-owned grayscale photo crops (the maintainer's — `_incoming0` +
  `~/Import/Photos` have material; downscale to small crops, e.g. ≤512px, and COMMIT them as
  fixtures). Graphic side: generated is fine (graphics are inherently low-entropy flat) —
  solid fills, simple shapes, text-on-flat — PLUS at least one **dithered** graphic (the
  one plausible high-entropy graphic — the adversarial case). Show `PHOTO_ENTROPY_STRONG`
  lands in the gap between the highest-entropy real graphic and the lowest-entropy real photo.
- Amend `DEC-047` (dated note: the new rule + threshold + calibration anchors; note the
  scale-broken flat detector is now masked-for-photos but NOT fixed → its own follow-up).

## Make the Failing Tests pass (in the spec)
Analysis unit tests (high-entropy grayscale → Photograph; low-entropy graphic stays
GraphicLogo; dithered-graphic case; document/UI unchanged), a native `optimize
--explain=json` integration test on the real grayscale fixture (→ `class: photograph`,
lossy winner), and a wasm `optimize_detailed` Auto test (→ AVIF, not lossless WebP).

## Non-negotiables
- **Do not regress genuine graphics** — logos/documents/screenshots must stay lossless.
  This is the differentiator; the dithered-graphic test is the guard.
- **Prove native AND wasm** — the classifier is shared; assert both paths route the
  grayscale photo to AVIF.
- Do NOT touch the flat/edge detector calibration or the RAW-EXIF path (separate specs).
- Real images on the photo side of the calibration set — no synthetic-only photo evidence
  (synthetics caused this bug).
- rtk footgun: cross-check greps with raw `grep` + a positive control; drive conclusions
  off `optimize --explain=json`.
- DCO-sign every commit.

## When done
1. Fill `## Build Completion` incl. the **calibration table** (entropy per validation image
   + chosen threshold) and the 3 reflection questions.
2. Append a `build` cost session to `cost.sessions` (best-available numbers).
3. `just validate` green.
4. `just advance-cycle SPEC-105 verify` — NOTE the `find_spec` glob bug (it can target a
   `prompts/*.md` file); VERIFY it edited the real spec's `task.cycle`, fix by hand if not.
5. Open a PR from `spec-105-entropy-photo-classify` (name PROJ-008, STAGE-029, SPEC-105,
   DEC-047 amendment). Mark build `[x]` in the timeline with PR #, cost, date.

Do NOT merge. Hold for verify (Opus). Report to the orchestrator: branch, PR #, the
calibration table (threshold + the gap it sits in), criteria met, native+wasm proof, gate
status, deviations, follow-ups.
