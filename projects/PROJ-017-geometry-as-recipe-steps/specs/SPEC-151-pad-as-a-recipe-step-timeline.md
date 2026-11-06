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
- [ ] **design** (`prompts/SPEC-151-design.md`): after the maintainer rules Q1–Q5.
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
