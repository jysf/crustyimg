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
- [x] **verify** (2026-11-06, `prompts/SPEC-151-verify.md`, Opus, read-only): ⚠ PUNCH LIST at
  `ef6befa`, two items, neither a claim, so no re-verify (rule 7). (1) **assert:** nothing pins
  DEC-112's `f64` border read: M14 is all green, but `0.01` on a 50 px edge is exactly ½ px, so `f64`
  gives 1 px and `f32` 0 px (binary 52×12 vs the mutant's 0 px refusal). The design's "no `f32` split
  in the load range" was reasoned, not driven, and is refuted. (2) **wording:** the PR is 31 files
  including the spec, not 32. Driven clean: the canvas against a rational oracle (332,800 exhaustive +
  200,000 random, 0 differ), pixels vs ImageMagick AE 0 in 12 cases, 37 + 5 messages by script, DEC-113
  with `exiftool`, M1/M3/M6/M12 matching the table, corpus dims vs the oracle, wasm 53/51 (main still
  51/49). Slim bundle +17,354 B raw, +3,410 B brotli. The `Rgba32F` 2 GiB finding is pre-existing
  (`resize` reaches it from any input on `main`). Optional, for ship to file: no test pins resize's cap
  message text. About $8.66 (Opus, 12.3M tokens; the input and cache_creation lines were cut off in
  the paste).
- [ ] **punch list** (`prompts/SPEC-151-punchlist.md`, Sonnet, worktree `fix/spec-151-punch`)
- [ ] **ship**
