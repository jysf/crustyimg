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
- [x] **design** (2026-11-06, `prompts/SPEC-152-design.md`, Opus, worktree `docs/spec-152-design`): 65 ACs
  (23 refusal rows), [S] in code, no DEC. Float rule re-driven in Rust (2,357,504 cases, 0 wrong); NaN never
  matches, out-of-range floats clamp; `UInt` gains `max`; folds a `parse_color` panic fix (non-ASCII, exit 101
  today). Inverse corrected for grey + chromatic fill. Corpus 22/72/75/66, wasm 55/53. 21 mutation rows, all driven.
- [ ] **build** (`prompts/SPEC-152-build.md`, Sonnet, worktree `feat/spec-152-trim`). Keeps the `parse_color` fold-in. Adds a `UInt {` grep after each rebase: PR #257 also edits `src/cli/ops.rs`, which the design missed.
- [ ] **verify**
- [ ] **ship**
