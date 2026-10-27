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
- [ ] **build** — `prompts/SPEC-135-build.md` — draft PR
- [ ] **verify**
- [ ] **ship**
