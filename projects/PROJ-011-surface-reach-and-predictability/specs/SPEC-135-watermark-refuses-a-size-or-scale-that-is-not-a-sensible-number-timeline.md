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
- [~] **design** — `prompts/SPEC-135-design.md` (handed out 2026-10-27, Opus). Rulings recorded
  in `## Frame`: cap 1024 in the schema; draw-nothing-on-both-paths out; keep `f32` CLI parsing.
- [ ] **build** — draft PR
- [ ] **verify**
- [ ] **ship**
