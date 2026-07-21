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
- [x] build — Opus, 2026-07-20 on `spec-083-honest-benchmarks`. Chose **(a) iso-quality** at an
  SSIMULACRA2 ~82 band (probe confirmed the band is hittable for every tool). Deliverables:
  `scripts/bench-compare.py` (cross-tool harness — one scorer = `crustyimg diff`, own-reference
  encode-fidelity scoring, fixed per-tool grids picked-nearest-band, competitors labelled not dropped),
  `BENCHMARKS.md` (methodology + machine + pinned versions + exact commands + per-bucket AND per-photo
  tables + honest caveats + reproduce section), `DEC-080` (methodology), `just bench-compare`, README
  link. **Honest headline: crustyimg is NEITHER smallest (sharp wins size 5/8) NOR fastest (sharp/IM
  3–8× faster on wall-clock, being multi-threaded); per-core it's a wash vs single-thread libvips.**
  Value framing = zero-dep pure-Rust binary + measured quality + RAW + wasm. Reproducibility proven
  (run1≡run2 on all deterministic fields, wall-times ≤~2%). Key finds: the distributed 0.5.0 binary has
  **no AVIF** (needs `--features avif`) — the central honesty pivot; brief's "5 cameras/Sony" corrected
  from EXIF (6 models/4 brands, no Sony); IM errored on the 47MP Leica (bad iCCP) — reported honestly.
  `just validate` green, no `src/` change. **Handed to verify (Opus) — NOT merged.** See Build Completion.
- [x] verify — Opus, 2026-07-20 on `spec-083-honest-benchmarks` @ 0ded21e. **⚠ PUNCH LIST — back to build.**
  What held: every published cell re-derived from `run1.json` matches exactly (per-photo, per-bucket
  medians, per-core table, smallest-AVIF tally 5/2/1); determinism confirmed 47/47 deterministic fields
  identical run1≡run2; `just validate` green; the `--features avif` pivot is real (`dist-workspace.toml`
  builds default features); RAW extension list correct; harness runs end-to-end and does label "NOT RUN".
  **Blocking (1):** `@squoosh/cli` is invoked with BOTH `width` and `height` set, which squashes aspect —
  its outputs are distorted 2048×2048 on **6 of the 8 photos** (verified: 6016×4016 → 2048×2048 while every
  other tool gives 2048×1367). Own-reference scoring masks it (still scores ~82), so the quality column
  provides no protection, and the doc's "same pipeline for every tool" is false for squoosh. Corrected
  aspect-preserving re-run of DSC_9952 gives 21 KB @ cq18, not the published 26 KB @ cq14.
  **Also:** the per-core table's DSCN3478 row is not iso-quality (sharp picked q78 multi-thread vs q70 at
  `VIPS_CONCURRENCY=1` — libaom output shifts with thread count), and it is the closest row the "faster on
  4 of 8" tally rests on; `sharp`/`cwebp` mis-size PORTRAIT sources (long edge 3068, not 2048) which breaks
  the "run it on your own corpus" promise; DEC-080's calibration is wrong — `crustyimg web` is byte-identical
  to `convert -q 80` (md5-verified), not `-q 85`, and lands 73.5–79.0 not 79–82, so the stated rationale for
  the 82 band centre is unfounded; "3–8× faster" understates ImageMagick (up to 14.1×); "none of the
  competitors ship a perceptual quality readout" is false (`magick compare -metric DSSIM/SSIM`,
  `cwebp -print_ssim`). Minor: prose "79–83.5" vs the table's own 83.6; the documented squoosh command omits
  `"method":"lanczos3"`; front-matter `cycle:` was never advanced past `design`. See the verify report.
