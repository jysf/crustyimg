# SPEC-160 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-160-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-11, Opus) — GO at [M], one spec, no DEC. Premise confirmed (`edit --pad`/`--trim`
  exit 2, unknown flag). Driven: `round((N/long)·long) == N` for all 2.5e9 pairs up to 50,000 (worst error
  7.3e-12 px); `+f+f+f` loads on all four routes, and `+f+f+f+f` adds alpha 0f. Recommends one `--pad N|W:H`,
  a two-place alpha check in `run_pixel_op` gated on `introduces_alpha()`, and `parse_color` refusing any non-hex byte.
- [x] **rulings** (2026-11-11, maintainer) — all three recommendations: entries 1–4 moved (ideas.yaml ×3,
  STAGE-060 ×1), nine-gravities entry folded in, so this spec closes STAGE-058; [M] uncut; one `--pad N|W:H`.
- [x] **design** (2026-11-11, Opus, $14.47) — `prompts/SPEC-160-design.md`. [M] holds, no DEC. Prototype-driven: 18 FTs,
  22 mutation rows (all driven). Corrections: `--pad N` past the long edge fails at apply (`border` <= 1);
  `edit`'s alpha line names `--pad-color` via `IntroducedAlpha.flag`; the batch-wide format is `--format` or the
  template only; one `FlagWorded` wrapper for the aspect pad and `trim`. No wasm row.
- [x] **build** (2026-11-13, Sonnet, $7.27) — draft PR #264 at `23a5c0e`, CI green (18 pass, 8 release
  jobs skipped; the first run's Windows leg failed on the build's own `/in/a.jpg` separator asserts, fixed).
  RED on `main`: `edit_pad_trim` 13/13, FT-18 ×2, FT-16; FT-17 green. 16/16 ACs claimed; every behaviour
  sentence driven per exit path. 22 mutation rows driven; table gaps: M6a also reds two `tests/trim.rs`
  tests, M9 also FT-4 and FT-9, M10a two `tests/pad.rs` tests, M10b the recipe-corpus golden; trap 14 stops
  at the lib's E0063, so its integration rows did not run. Files: the 13 in Outputs plus the spec. Cost
  measured before the CI-fix turns.
- [ ] **verify** — `prompts/SPEC-160-verify.md`
- [ ] **ship**
