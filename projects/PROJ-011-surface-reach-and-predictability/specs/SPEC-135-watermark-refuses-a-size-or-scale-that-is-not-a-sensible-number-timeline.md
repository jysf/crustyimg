# SPEC-135 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-135-<cycle>.md`.

## Instructions

- [x] **frame** — `prompts/SPEC-135-frame.md` (2026-10-27, Opus). GO, [S] upper. Premise
  confirmed on a release build of `4d6808b`; `--size 100000` is worse than filed (a `zeno` panic,
  exit 101). Only `--size` and `--scale` disagree between CLI and recipe; recommended cap
  `size <= 1024` as a schema row (peak RSS 106 MiB at 1024, 1 GiB at 4096, 11-char text).
  3 out-of-spec findings filed on STAGE-054.
- [x] **design** — `prompts/SPEC-135-design.md` (2026-10-27, Opus). 14 ACs, 16 failing tests.
  CLI checks `--size`/`--scale`/`--opacity` through a public per-key table check, naming the flag,
  before any file IO; `size <= 1024` schema row; an unrepresentable scaled mark is refused as
  `the scaled mark does not fit inside the W×H px image`. Released 0.7.1 panics at size 100000.
- [x] **build** — `prompts/SPEC-135-build.md` (2026-10-27, Sonnet). Draft PR #207 at `8922f31`,
  CI green (16 pass, 6 skipped). Commit 1 `d1484cf` holds the goldens and touches no `src/`. 14/14 ACs;
  every negative control flipped except AC-3 under "cap removed" (it compares the table with
  itself, so the design's prediction was wrong). Measured $4.52 before the final turns.
- [x] **verify** — `prompts/SPEC-135-verify.md` (2026-10-27, Opus). ⚠ PUNCH LIST at `8922f31`:
  one item, M-c (a message printing the value widened to f64) survives every test because every
  row's value is exact in f32. Everything else held: 14/14 ACs driven on both binaries, all six
  controls flipped, byte identity vs `main` by ImageMagick AE = 0. Three wording cleanups. $4.31.
- [x] **fix** — `prompts/SPEC-135-fix.md` (2026-10-27, Sonnet). PR #207 at `161b637`
  (`2aa78a4` test, `b11dd40` docs, `161b637` cost). Two f32-inexact rows in AC-4 (12 rows);
  M-c went RED on both; no `src/` change since `8922f31`; three sentences corrected. Full CI
  green at `161b637`, 13 jobs (the `b11dd40` run was cancelled when the next push replaced it). ~$0.82.
- [x] **re-verify** — `prompts/SPEC-135-reverify.md` (2026-10-27, Opus). ✅ APPROVED at `161b637`.
  M-c caught by each new row alone; no `src/` change since `8922f31`; three sentences true; CI
  green on every required leg. Two optional cleanups. $1.45. Marked ready; the maintainer
  updated the branch (it brought in only orchestrator docs commits; `src/`, tests and docs
  identical to `161b637`) and squash-merged it as `df91f7e` (PR #207, 2026-10-27).
- [~] **ship** — `prompts/SPEC-135-ship.md` (handed out 2026-10-27, Sonnet)

