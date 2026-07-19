# SPEC-096 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-096-<cycle>.md`.

## Instructions
- [x] design — framed build-ready 2026-07-18. Two user-facing warts, batched: (1) rewrite the AI-ish,
  spec/DEC-referencing headers of the bundled recipes (`web`/`gallery`/`product`) to plain behavior-first
  copy, keeping `recipes/web.toml` ↔ demo `WEB_RECIPE` byte-identical (`tests/demo_smoke.mjs:718` pin);
  (2) replace the spinning busy glyph with a static 🦀 placeholder. No engine/recipe-behavior change.
  Mechanical guard test asserts no `SPEC-`/`DEC-` in shipped recipe headers. Recommended build on Sonnet
  (mechanical sweep — extends the model experiment), verify on Opus. Complexity S.
