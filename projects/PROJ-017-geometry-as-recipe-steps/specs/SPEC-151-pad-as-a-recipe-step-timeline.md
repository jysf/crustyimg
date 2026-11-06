# SPEC-151 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-151-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-06, `prompts/SPEC-151-frame.md`, Opus): SPLIT. SPEC-151 is `pad` with
  opaque fills, [S], no DEC; `transparent` and its JPEG rule go to a new STAGE-058 entry. No `pad`
  today (`unknown operation 'pad'`, exit 1, on 0.8.2). Driven: JPEG drops alpha and keeps the hidden
  colour (transparent red came out `srgb(254,0,0)`, exit 0, no warning); crop on a tag-6 input writes
  no tag (DEC-113 covers pad); `resize fit` + pad aspect lands exactly on 1080×1080 / 1200×630 (36/36).
  Keys: `aspect` + `gravity`, or `border` (long edge), with a required opaque `color`. No condition
  uses the refusal hook. Folds in the SPEC-147 cleanups and the refusal-hook doc entry. Five maintainer
  questions, each with a recommendation.
  Rulings (2026-11-06, Q1–Q5 as recommended): split `transparent` into its own STAGE-058 entry,
  refused into a format with no alpha (exit 4); `color` required, no default; `border` is a fraction
  of the long edge, the same width on every side; a grey image with a chromatic fill widens to RGB.
- [x] **design** (2026-11-06, `prompts/SPEC-151-design.md`, Opus): 83 ACs, [S] in code, no DEC. The
  canvas rule restated per axis (the frame's "long side kept" fails a square); no-op = canvas equals
  image; float images keep their variant. 37 exact load messages and 3 apply failures, a 20-cell
  colour-type table, corpus 20/66/69/60, wasm 53/51. 14 mutation rows, 12 driven on a prototype.
- [x] **build** (2026-11-06, `prompts/SPEC-151-build.md`, Sonnet, worktree `feat/spec-151-pad`): draft
  PR #255 at `ef6befa`, CI green on every leg at `746733c`. 83/83 ACs claimed. All 15 mutation rows
  driven natively on the pushed head, no cell wrong (extra reds: M1 `apply_pads_a_png_end_to_end`, M4
  `float_images_keep_their_variant`, M8b `colour_type_table_row_by_row`; M14 all green as predicted;
  wasm rows not run under mutation). Corpus `cmp` identical across targets, no existing output changed.
  wasm 53/51; main's constants were still 51/49. Deviations: canvas formula in its own `aspect_canvas`
  (AC-3's sweep can't run through `layout` at the caps); ids stripped from the hoisted cap comments;
  `orientation_of` has 3 callers. Out-of-spec: the area cap counts pixels, so an `Rgba32F` canvas at
  the cap is 2 GiB. About $10.85 (Sonnet, 28.4M tokens).
- [ ] **verify** (`prompts/SPEC-151-verify.md`, Opus, worktree `verify/spec-151`, read-only)
- [ ] **ship**
