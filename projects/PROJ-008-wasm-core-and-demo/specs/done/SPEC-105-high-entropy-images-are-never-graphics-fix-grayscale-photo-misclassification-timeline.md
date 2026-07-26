# SPEC-105 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-105-<cycle>.md`.

## Instructions

- [x] **design** (2026-07-25, orchestrator main-loop) — Framed from the grayscale-misclassification probe
  (`docs/research/proj-008-grayscale-photo-misclassification-probe.md`) after a real B&W Leica DNG came out
  of the demo's Auto path as 823 KB lossless WebP (should be ~62 KB AVIF). Root cause: shared classifier
  (DEC-047) treats grayscale (≤256 colours) as a graphic, and RAW EXIF-stripping bypasses the camera prior
  that normally saves photos. Fix = a strong-entropy → Photograph signal ahead of the graphic gates.
  Engine change, native + wasm. Spec + Failing Tests + DEC-047-amendment plan written.
- [x] **build** (2026-07-25, Opus, PR #113, ~$4.1 est) — Added `PHOTO_ENTROPY_STRONG = 4.0` + the
  strong-entropy → Photograph rule (after EXIF/Document, before the graphic gates). Calibrated against
  48 real photo crops (floor 4.58) + real/generated graphics incl. an 8-colour Floyd–Steinberg dither
  (3.03); threshold sits in the (3.43, 4.58] gap. Committed 4 real fixtures under
  `tests/fixtures/classify/`. Native (Leica DNG → AVIF 63,894 B, was 843,252 B) + wasm both proven.
  Two unrepresentative high-entropy synthetic tests redesigned to realistic low-entropy fixtures.
  DEC-047 amended. All gates green.
- [x] **verify** — on Opus; re-drive the symptom end-to-end + the graphic no-regression guard (esp. the
  dithered case); confirm the threshold sits in the measured gap; native + wasm. CLEAN. Grayscale Leica
  DNG → photograph → AVIF 63,894 B (entropy 7.45). Color case (maintainer's repro) confirmed: 64 real
  EXIF-stripped D3300 color RAWs (6016×4016) all → photograph → AVIF, entropy floor 5.87 (>4.0 with wide
  margin); with the rule disabled the high-flat ones (flat 0.86–0.94) fall to graphic-logo → lossless WebP
  — exactly the reported bug. Dither re-measured 3.03 (<4.0, lossless). Threshold mutation-checked both
  directions. 5 fixture/test changes audited: all corrections for the removed gradient-misclassification,
  no diluted assertions. Native suite + wasm-test + demo-smoke + validate + lean + fmt/clippy all green.
- [x] **ship** (2026-07-26, orchestrator main-loop) — PR #113 squash-merged (54ba05e); demo redeploys from
  `main` so Auto picks AVIF for B&W **and** color EXIF-stripped photos. ⚠ Required a LARGE CI-parity fix
  pass: build + verify both false-greened the feature matrix on stale incremental-build artifacts, so CI
  (clean builds) caught real breakage the classifier change exposed — `json_shape_consistent_across_verbs`
  (avif-gated), 3 cli tests reclassified graphic→photo (codec-agnostic), a dead-code helper on no-avif
  legs, and a missing verify cost session. Fixed against clean local full-matrix builds. ~6 CI cycles.
  Lessons banked in the Ship reflection (clean-matrix verify for engine changes; orchestrator re-runs the
  matrix; don't push to main mid-merge; check git status for sub-agent uncommitted work). Prompts +
  readouts in `prompts/SPEC-105-{build,verify}.md` + `-readouts.md`.
