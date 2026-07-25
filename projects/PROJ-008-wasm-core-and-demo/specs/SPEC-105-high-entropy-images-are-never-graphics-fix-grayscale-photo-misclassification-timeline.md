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
- [~] **build** — add the entropy rule + calibrate the threshold against REAL grayscale photos + graphics
  (incl. a dithered one); prove native + wasm. Dispatched to Opus 2026-07-25. Prompt: `prompts/SPEC-105-build.md`.
- [ ] **verify** — on Opus; re-drive the symptom end-to-end + the graphic no-regression guard (esp. the
  dithered case); confirm the threshold sits in the measured gap; native + wasm.
- [ ] **ship** — squash-merge on maintainer go-ahead; demo redeploys from `main` so Auto picks AVIF for
  B&W photos.
