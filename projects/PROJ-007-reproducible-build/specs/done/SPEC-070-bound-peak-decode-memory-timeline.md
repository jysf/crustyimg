# SPEC-070 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — bound PEAK decode memory (user-prioritized after SPEC-069's F-RAW-1). Root:
  `image::Limits.max_alloc` (512 MB) bounds a SINGLE allocation, not the cumulative/peak working set,
  and `image` 0.25 has no total-pixel field — so a near-max-dimension image (16384×9776) passes every
  DEC-034 cap yet peaks ~1.9 GB (confirmed: 782 B `.nef` → `info` ~1.93 GB). Spec = a crustyimg-side
  **total-pixel / peak-memory cap enforced BEFORE the full decode**, via a header dimension peek at the
  two seams that lack one (generic `ImageReader` `mod.rs:350-355`; RAW `decode_jpeg_with_limits`
  `raw.rs:190-194`), aligning AVIF/SVG `check_caps` (already have dims) to the same cap. **DEC-063** =
  the budget + amplification factor (~4× RGBA, SPEC-069-measured) + pixel cap (rec. ~1 GiB ⇒ ~64 Mpix,
  rejects the bomb, keeps 24–50 MP photos) + tradeoff. **Scope precision: closes F-RAW-1 + general
  decode peak; does NOT close F-AVIF-3 (upstream avif-parse PARSE-stage, needs vendoring) — say so, no
  overclaim.** Bonus: F-RAW-1's reproducer graduates into the always-on `fuzz_corpus_never_panics`
  smoke once rejected cheaply at the header. Gotchas: `into_dimensions()` consumes the reader; 2
  hardcoded test mirrors (`raw.rs:234`, `avif.rs:609`) bypass `decode_limits()`. No new default dep.
  Framing, 2026-07-10.
- [x] **build** — added `MAX_IMAGE_PIXELS` (64 Mpix) + a pure saturating `check_pixel_budget(w,h)`
  (typed `LimitsExceeded`); wired a pre-decode header/SOF peek into the generic + RAW seams + aligned
  AVIF/SVG/HEIC `check_caps` + dav1d `frame_size_limit` to one source of truth; updated the hardcoded
  test mirrors (avif's now asserts equality against the const); committed F-RAW-1's reproducer + moved
  it into the corpus smoke; DEC-063. Measured: bomb 1.93 GB → 8.7 MB, exit 1; 24 MP photo still
  converts (280 MB, re-validating ~4×). → **PR #78**, 26/26 3-OS CI green, dep diff empty. Sound
  deviation: the cap is a module const (can't live in `image::Limits` — that's the bug). Scope
  addition: RAW rejections now name the cap (the human-facing-string lesson again). Est. ~120k tok. 2026-07-11.
- [x] **verify** — fresh adversarial session. **CLEAN on the core memory goal.** Bypass hunt on the
  real binary: bomb bounded on every route (RAW/baseline+progressive JPEG/PNG/GIF/SVG ~8 MB); the
  progressive-JPEG worry HELD (64 Mpix at-cap → 418 MB, 1.6× baseline; 4× is conservative); every
  decode command routes through the cap; no false rejection (24/50 MP + 8192×8192 RGBA16 decode,
  8192×8193 rejected, 0 corpus regressions); overflow-safe; F-RAW-1 in the smoke; F-AVIF-3 honestly
  open. Observation filed: full-pipeline peak (decode + encode/rule) can approach 1 GiB for an at-cap
  image (49 Mpix convert 934 MB). Est. ~180k tok. 2026-07-11.
- [x] **ship** — orchestrator due-diligence (per-command routing on the real binary, all bounded
  ~2.7 MB) surfaced the **lint `LimitsExceeded` false-diagnosis** (pre-existing `Err(_)` catch-all,
  widened by the cap); dispositioned ship-now-clean-core + file-follow-up. Squash-merged **PR #78** →
  main (**5ecc717**); filled verify/ship cost sessions + `cost.totals` (380k tok / ~$3.42, 4 sessions,
  labelled estimates §4) + ship reflection; timeline; **STAGE-024 marks SPEC-070 shipped** + files 3
  follow-ups (lint false-diagnosis [next]; full-pipeline peak envelope; `--max-pixels`/cap-raise for
  medium-format); archived to `done/`; cost-audit + validate green; brag + memory. **SPEC-070 SHIPPED —
  F-RAW-1 closed; the peak-decode-memory leg of untrusted-input-hardening filled.** PROJ-007 continues.
  2026-07-11.
