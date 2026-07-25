# SPEC-105 — VERIFY prompt

Cycle: verify. Verify the SPEC-105 build **adversarially** — an engine change to the shared
content classifier. The two real risks: (a) the calibrated threshold regresses a genuine
graphic, and (b) tests/fixtures were changed to pass rather than because the old behaviour
was wrong.

## Setup
- This prompt is on `main` — read it now, before switching. Build is on
  **`spec-105-entropy-photo-classify`** (PR #113). Check it out. Do NOT merge / push to `main`.
- DCO-sign any verify commit; `git rebase --signoff main` if one lands unsigned.
- rtk footgun: the hook silently zeroes `rg -c` and mangled `--nocapture`/globs for the
  build. Cross-check greps with raw `grep` + a positive control; drive conclusions off
  `optimize --explain=json`, not greps; use `rtk proxy <cmd>` if raw stdout matters.

## Read first
1. `projects/PROJ-008-wasm-core-and-demo/specs/SPEC-105-…-misclassification.md` (Build
   Completion + the calibration table).
2. `docs/research/proj-008-grayscale-photo-misclassification-probe.md`.
3. `decisions/DEC-047-*.md` (the amendment).
4. `git diff main...spec-105-entropy-photo-classify`.

## Verify — with evidence, not by re-reading asserts

1. **The grayscale symptom is fixed end-to-end.** Native `optimize --explain=json --max 2048`
   on `_incoming0/L1024678.DNG` → `class: photograph`, lossy AVIF winner (~62 KB), NOT
   823 KB lossless WebP. Re-run it; don't trust the build's number.

2. **★ THE COLOR CASE — the maintainer just reproduced the SAME bug on a COLOR Nikon RAW**
   (6000×4000, Auto → WebP, not AVIF, on the *currently deployed* demo which lacks this fix).
   This is the flat-detector mis-fire on an EXIF-stripped COLOR photo (probe finding #3), and
   the whole point of the strong-entropy rule is that it should fix this too (a color photo is
   high-entropy → Photograph before the flat gate). **CONFIRM IT:** find a real color RAW —
   a Nikon `.nef` (or any color RAW) in `_incoming0` / `~/Import/Photos` — with a shallow-DOF /
   soft look, strip/lack EXIF, run it through the fixed classifier, and confirm `class:
   photograph` → **AVIF**, native AND wasm. **Report its measured entropy** and confirm it's
   above the 4.0 threshold with margin — NOT marginally (the build flagged that a very
   low-contrast photo could dip below 4.0 and fall back to lossless; find out if a real soft
   color pet/portrait RAW is anywhere near the edge). If you can't find a color RAW, use a
   color photo with EXIF stripped and a large soft-blur region.

3. **No graphic regressed — re-drive the dithered case.** The committed 8-colour
   Floyd–Steinberg dither (build says entropy 3.03) must stay `GraphicLogo` → lossless.
   Independently re-measure its entropy and confirm it's below 4.0. Also confirm a plain
   logo, a document, and a UI screenshot stay lossless. This is the differentiator — the
   dangerous direction.

4. **The threshold sits in a real gap.** Re-measure entropy for the committed fixtures and
   confirm `PHOTO_ENTROPY_STRONG = 4.0` sits strictly between the highest-entropy graphic and
   the lowest-entropy photo. Mutation-check: nudging the constant up past the photo floor (or
   down past a graphic) should flip the relevant test.

5. **The 5 changed tests/fixtures are corrections, not weakenings.** The build redesigned two
   synthetic `classify` unit tests, two `cli.rs` never-bigger tests, and the demo-smoke
   "graphic" fixture, saying they relied on a gradient MISclassification it removed. For EACH,
   confirm the OLD test was asserting wrong behaviour (a photo/gradient wrongly called a
   graphic) and the NEW one asserts correct behaviour — not that a real assertion was diluted
   to go green.

6. **Native AND wasm both fixed**; full native suite + `just wasm-test` + `just demo-smoke` +
   `just validate` + lean build all green; no unrelated behaviour change.

## When done
- VERDICT CLEAN / NOT-CLEAN, each finding (real / severity / evidence). Fix small defects
  minimally + DCO-signed, or escalate. **If the color-RAW case is NOT fixed, or a real photo
  sits near the 4.0 edge, that is a finding — say so.** Do NOT merge.
- If CLEAN: `just advance-cycle SPEC-105 ship` (verify it edited the REAL spec's task.cycle —
  the find_spec glob bug), mark verify `[x]` in the timeline.
- Report to the orchestrator: verdict, the color-RAW result + its entropy, the dither
  re-measure, the threshold-gap confirmation, the 5-fixture-change audit, native+wasm proof,
  gate status, and real cost.
