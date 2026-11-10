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
- [x] **build** (2026-11-07, `prompts/SPEC-152-build.md`, Sonnet, worktree `feat/spec-152-trim`): draft PR #258
  at `3d02d45`, CI green (18 passed, 8 skipped). 65/65 ACs; 28 files (29 with the spec). Rebased onto SPEC-150 after
  it merged; every `UInt` literal has `max`. wasm 55/53, bundle +13,011 B raw / +2,910 B brotli. Mutations red as
  tabled except: M3 driven with a different mutant (`ceil`), AC-13's binary no-op row green under M2, M12 finished
  by hand. One hand-written CI wait loop, stopped at once. Cost $9.58.
- [x] **verify** (2026-11-09, `prompts/SPEC-152-verify.md`, Opus, worktree `verify/spec-152`): APPROVED at
  `3d02d45`. Messages diffed by script with a negative control; Python oracle agrees with all 36 match rows and
  re-derives 2,357,504; ImageMagick `-trim` AE 0 on 8-bit RGB, 16-bit RGBA and grey; a float TIFF drives the
  `f32(1/255)` edge on the binary. M3 driven as the spec wrote it: the matrix stays green, so the table is the float
  guard (confirmed). M12 re-driven: 24 red. Two wording items for ship (M2's AC-13 cell; M3's alpha cell and row
  names) and four optional cleanups. Out-of-spec: `parse_color` accepts a sign (`"+f+f+f"` → `#0f0f0f`) on main
  too, a claim for STAGE-058. Cost $6.23.
- [x] **merge** — PR #258 updated to `main` and marked ready by the orchestrator after verify approved
  (CI green at `6e97832`, 18 passed, 8 skipped); merged by the maintainer as `6fed684` on 2026-11-10.
- [ ] **ship** (`prompts/SPEC-152-ship.md`, Sonnet, worktree `docs/spec-152-ship`)
