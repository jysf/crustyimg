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
- [ ] **build** — add the pixel/peak cap constant + a pure `check_pixel_budget(w,h)` (saturating,
  typed `LimitsExceeded`); wire it into the generic + RAW seams via a pre-decode header peek + align
  AVIF/SVG `check_caps`; one source of truth (thread through `decode_limits()`/a `DecodeCaps`); update
  the 2 hardcoded test mirrors; move F-RAW-1's reproducer into the corpus smoke + mark it closed in the
  run record; DEC-063. Make the Failing Tests pass. Gates: default + lean + clippy + fmt + `just deny`
  (unchanged) + `just validate`; repo-root Cargo/lock/deny diff empty.
- [ ] **verify** — fresh session. DRIVE THE REAL BINARY: `/usr/bin/time -l crustyimg info <F-RAW-1
  reproducer>` → rejected (limits exit code) with peak RSS bounded WELL under the budget (not ~1.9 GB);
  a legitimate ~24 MP photo still decodes (no false rejection); the cap fires uniformly on AVIF/SVG/RAW/
  generic; F-RAW-1's reproducer runs green in the corpus smoke; the record honestly still lists F-AVIF-3
  as open. Gate matrix green, no new dep.
- [ ] **ship** — merge PR; build/verify/ship cost sessions + totals + reflection; archive to done/.
  STAGE-024 backlog: SPEC-070 shipped → F-RAW-1 closed; the peak-memory gap in `untrusted-input-hardening`
  filled. Remaining backlog items follow. PROJ-007 continues until STAGE-024 completes.
