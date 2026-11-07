# SPEC-152 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-152-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-06, Opus 5.5) — GO at [S], no DEC. Premise confirmed (`unknown operation 'trim'`,
  exit 1). Drove ImageMagick (`-fuzz` is a scaled colour distance, not per channel; matches alpha-0 on
  alpha alone) and a Python model: tolerance in 8-bit steps, ×257 at 16 bits, float read as the
  nearest 16-bit value (exact; an `f64` `/255` rule misses half the edges). Folds the `resize` cap-text entry.
  Rulings (2026-11-06, Q1–Q7 as recommended): top-left default, all sides against it; tolerance in
  8-bit steps, ×257 at 16 bits, float read as nearest 16-bit, in the doc entry with no DEC; alpha 0
  matches alpha 0 whatever is hidden; one colour, each side on its own; whole match fails at apply;
  `tolerance` an integer (`8.0` refused); `color` optional and may be translucent. Frame cost $4.38.
- [ ] **design** (`prompts/SPEC-152-design.md`, Opus, worktree `docs/spec-152-design`)
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
