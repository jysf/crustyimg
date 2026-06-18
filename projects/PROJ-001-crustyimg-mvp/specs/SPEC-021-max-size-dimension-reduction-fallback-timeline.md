# SPEC-021 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-021-<cycle>.md`.

## Instructions

- [x] **design** — spec + `## Failing Tests` + Implementation Context authored by the ORCHESTRATOR (Opus) directly. Studied the seams: `resolve_effective_quality` returns `Option<u8>` then `sink.write(&out_img, quality)`, so output pixels are fixed before budget resolution — the model change is threading a replacement (resized) image back. The generic `search_under_size` reuses for a scale-percent 1..=100 axis; `quality` is self-contained so resize via `DynamicImage::resize_exact(Lanczos3)` (NOT the fast_image_resize op — layering); `Image::from_parts` rebuilds the replacement. Emitted **DEC-023** (quality-first then downscale; reuse the search core; floor + best-effort; lossless gains `--max-size`). Build prompt at `prompts/SPEC-021-build.md`. Completed 2026-06-17.
- [ ] **build** — Sonnet 4.6 (or orchestrator-direct if subagent Bash blocked), per `prompts/SPEC-021-build.md`: `SizeFit` + `fit_under_size` in `src/quality` (quality-first, then scale search reusing `search_under_size`); `resolve_effective_quality` → `EncodePlan { quality, image }` + the two `run_pixel_op` call sites; CLI scaled/unmet warnings; docs. NO new dep, NO Operation, NO sink-encode change. Branch `feat/spec-021-max-size-dimension-reduction-fallback`.
- [ ] **verify** — confirm lossless `--max-size` now downscales to fit (was a no-op); lossy scales only when quality alone can't; budget met at full size never resizes; written bytes match the probe (cross-sync) and `len ≤ budget` when met; a downscale warns; perceptual / fixed-`-q` paths unchanged; all gates.
- [ ] **ship** — orchestrator bookkeeping on `main` after merge (real cost numbers; PAUSE before merge/ship). After this, STAGE-008 can move to its stage-ship.
