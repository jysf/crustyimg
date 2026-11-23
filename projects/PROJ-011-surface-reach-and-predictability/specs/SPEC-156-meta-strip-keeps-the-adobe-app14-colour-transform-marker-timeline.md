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
- [ ] **verify** — `prompts/SPEC-156-verify.md` (Opus, read-only)
- [ ] **ship**
