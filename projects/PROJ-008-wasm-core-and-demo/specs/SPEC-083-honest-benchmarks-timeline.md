# SPEC-083 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-083-<cycle>.md`.

## Instructions
- [x] design — framed build-ready 2026-07-20; **reframed + sharpened 2026-07-20 for the 0.5.0-live
  reality** (build now, crustyimg side = shipped 0.5.0). BENCHMARKS.md = honest, EQUAL-QUALITY,
  reproducible cross-tool comparison vs sharp/`@squoosh/cli`/ImageMagick on size+speed, off a real
  `--corpus`. **Pinned the concrete reference corpus** (`~/PSeven/experiments/crustimg_redo_plus/_incoming0`,
  8 photos 0.7–47 MP / 5 cameras — the STAGE-030 set; committed CC0 corpus is all <2048px, SPEC-088 carry).
  **Pinned the matched-quality method: score EVERY tool's output with ONE scorer — `crustyimg diff`
  (SSIMULACRA2) vs the same original — show the quality column; iso-quality-band OR honest size-vs-quality
  scatter, methodology fixed BEFORE numbers are read.** Expects a **DEC** (methodology/scorer/tool-set/
  corpus provenance). Judgment-bound, not mechanical → **recommend OPUS build** (credibility stakes), Opus
  verify. Extends the SPEC-088 `just bench` discipline to competitors; report losses honestly; tell the
  q85-AVIF "high" (~80) story straight. Complexity M (leans L — installs 3 competitors + a cross-tool
  harness). **Build-ready; awaiting dispatch decision (Opus build).**
