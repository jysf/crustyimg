# SPEC-156 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-156-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-22, Opus) — GO at [S]. Premise driven on a release build: stripped `cmyk_16x16.jpg` scores
  SSIMULACRA2 41.9374, AE 56.47, djpeg `srgb(191,64,32)` → `srgb(93,134,30)`. A 3-component Adobe transform-0 file with
  component IDs 1,2,3 breaks too (−21.30); transform 1, RGB-letter IDs and 4-component transform 0 don't. Keep APP14
  (with the `Adobe` signature) only; set/clean/copy/pixel lane already keep the render. Orientation finding filed.
- [x] **design** (2026-11-22, Opus) — `prompts/SPEC-156-design.md`. [S] holds. Keep APP14 when its payload starts
  with `Adobe` and is ≥ 12 bytes (correction: the byte after `Adobe` is the version's high byte; `Adobe\x01…` still
  decodes orange). New fixture recipe (`cjpeg -rgb` + ids 1,2,3, sha256 recorded); 9 tests (6 new, 2 rewritten,
  1 binary); 12 mutation rows driven (10 on a full run of every target, M3b and M8 on the lib target); DEC-003 amendment + DEC-029 note written for ship.
- [x] **build** (2026-11-22, Sonnet, $2.53) — draft PR #269, head `b0b870d`; CI 18 green, 8 skipped
  (release-only). Fixture hashes equal the spec's; baseline red on FT-1–4 and FT-7–9 with the spec's
  messages; 6 ACs met; every mutation row matched, none wrong; no DEC edited (ship applies the text).
- [x] **verify** (2026-11-23, Opus, $2.76) — ⚠ PUNCH LIST at `b0b870d`, one **assert**: nothing pins
  the `marker == 0xEE` clause (an `Adobe_CM` APP13 would be kept without it; mutant MK leaves 1,357
  green). Fixture rebuilt byte for byte; nine extra JPEG shapes render the same under djpeg/ImageMagick;
  M1/M3/M3b/M5a/M5b/T14 match. Merges clean with #268. Out of spec: zune-jpeg refuses transform 5 →
  `guidance/ideas.yaml`.
- [x] **punch list** (2026-11-23, Sonnet, $0.76 measured before its doc commit) — FT-10
  `strip_all_removes_an_adobe_cm_app13`, test code only (`a691427`); MK reddens FT-10 alone (1,357/1),
  unmutated 1,358/0; AC-3 widened. Head `3aaa700`; every CI leg green there (orchestrator-read).
- [x] **merge** (2026-11-23, maintainer) — PR #269 as 58740ba8752f65080342bab3b16e408f8cd9d31c.
- [x] **ship** (2026-11-23, Sonnet, $1.56) — merged 58740ba (PR #269); DEC-003 amendment and DEC-029 note applied; shipped binary drives the fixture and `cmyk_16x16.jpg` to byte-identical strips; build, verify and punch list re-measured (total $22.80 over six sessions).
