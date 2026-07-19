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
- [x] build — Sonnet, primary checkout. All 5 criteria met; guards landed in `src/recipe/bundled.rs`
  (`#[cfg(test)]`-only) + `tests/demo_smoke.mjs`. Builder ran its own negative controls on all three
  guards before handback. `WEB_RECIPE` synced byte-for-byte; static `🦀` with `animation: none`.
- [x] verify — ✅ CLEAN (Opus, primary checkout, adversarial). Scope: only `src/recipe/bundled.rs`
  touched in `src/`, test-only — zero engine/parse/registry change. **Recipe behavior byte-identical
  parent↔branch** across web/gallery/product at 3000×2000 (distinct hashes → resize actually fired, not
  a no-op). Fresh negative controls: NC-1 (`SPEC-999`) + NC-1b (`Mode::Fast`, different needle + file),
  NC-2 (step mutation), NC-3 (re-add spin — crab check still passed, so it catches motion specifically).
  Byte-sync reconciled (692 UTF-8 B vs the smoke's 690 char-count = the em-dash). Crab/no-motion/zero-
  network confirmed in real headless Chrome. 779/0 tests, clippy `-D warnings`, fmt, smoke all green.
- [x] ship — squash-merged PR #101 (**ad94997**) 2026-07-18, 27/0 matrix green. The recipes a visitor
  copies now read plainly and the busy state is a static 🦀. No DEC. Bookkeeping: cycle→ship, 3 cost
  sessions with `model:` (build Sonnet $3.0 / verify Opus $4.5 / ship $0.45 ≈ **$7.95**), timeline,
  archive, memory + brag. New feedback memory [[comments-plain-no-spec-refs]].
