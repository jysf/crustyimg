# SPEC-143 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-143-<cycle>.md`.

## Instructions

- [x] **frame** (2026-10-30, `prompts/SPEC-143-frame.md`): GO, [S]. Premise confirmed on the CLI,
  `apply` and `build`: blank text, size 0.01, opacity 0, colour alpha 00 and a transparent image
  all exit 0 with 0 px changed. `visible_pixels` is counted before opacity, so design adds a
  separate post-opacity max-alpha predicate, decided from the mark (as DEC-104 is), at two call
  sites: `run_watermark` and `from_params`.
- [x] **design** (2026-10-30, `prompts/SPEC-143-design.md`): 14 ACs, 16 failing tests, DEC-109.
  `Watermark::invisible_mark()` (additive) refuses at 16-bit precision via the composite's own
  `faded_alpha16`, at the end of `from_params` and in `run_watermark`. Only opacity 0 and colour
  alpha 00 are flag checks: tab and U+3000 draw the font's fallback box, so blank text is not.
- [x] **build** (2026-10-30, `prompts/SPEC-143-build.md`, Sonnet, $3.88): draft PR #235 at
  `c467490`, CI green. Golden blessed on unmodified `src/` (`2ccbdaf`); refactor `8a91c83` kept FT-6
  green. 14/14 ACs; three mutation-table deviations reported (AC-3 two lines, M5/FT-15, M4a/FT-13).
- [x] **verify** (2026-10-30, `prompts/SPEC-143-verify.md`, Opus, $9.85): ⚠ PUNCH LIST at
  `c467490`. Code right on every path (36 cause rows, boundaries, 18/18 out-of-repo `cmp`). Docs and
  tests only: two stale doc promises, a surviving recipe-side `max_alpha` mutation, a 16-bit
  mid-opacity golden gap, and FT-11/FT-12 failing on `main` for the 5 % cap, not the mark.
- [x] **punch list** (2026-10-30, `prompts/SPEC-143-punchlist.md`, Sonnet, $1.86): `36c3b66`, CI
  green, no `src/` change. Two low-alpha FT-8 rows, a 16-bit 0.6 golden (matches `main`), FT-11/FT-12
  enlarged past the 5 % cap; each proven red by its mutation. Confirmed by the orchestrator (docs-and-
  tests only, no re-verify); PR #235 marked ready.
- [x] **ship** (2026-10-31, `prompts/SPEC-143-ship.md`, Sonnet, $0.66): PR #235 merged as `06eb552`, 2026-10-31. Reflection written; one backlog entry filed (verify leftovers); cost re-measured (build $4.70, punch list $2.39), verify added ($10.25). Total $28.18 across 6 sessions.
