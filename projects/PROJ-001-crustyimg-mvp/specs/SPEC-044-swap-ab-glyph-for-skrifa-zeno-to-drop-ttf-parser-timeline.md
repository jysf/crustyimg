# SPEC-044 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-044-<cycle>.md`.

## Instructions

- [x] design (2026-07-04) — Opus, main loop. First spec of the new **STAGE-010** (advisory
  elimination). Authored the spec (`## Failing Tests` + `## Implementation Context`) and
  **DEC-045** (`skrifa`+`zeno` rasterizer, drop `ttf-parser`, remove the RUSTSEC-2026-0192
  ignore; supersedes DEC-032's rasterizer choice; drop pairwise kerning). **Two design-time
  probes:** (1) the backlog's `fontdue` plan was disproven — fontdue 0.9.3 still pulls
  `ttf-parser 0.21.1`, and the advisory is crate-wide (`patched=[]`), so it wouldn't remove
  the ignore; (2) `skrifa 0.44` + `zeno 0.3.3` on the real Go-Regular reproduced ascent /
  advance / glyph bounds exactly and gave a `(coverage, Placement)` analog of ab_glyph's
  `px_bounds()`+`draw()`. Retargeted to `skrifa`+`zeno` (the advisory's own recommended
  alt); all permissive, `ttf-parser`-free. Maintainer approved the re-scope from PATCH→SPEC.
  Design + DEC-045 + STAGE-010 to be pushed to `main` before the build branch.
- [x] build (2026-07-04) — Sonnet, prescriptive prompt. Rewrote `src/text/mod.rs`'s
  rasterizer on `skrifa 0.44.0` + `zeno 0.3.3` per the probed API (layout→composite
  structure preserved, y-negating `ZenoPen`, no kerning); swapped `Cargo.toml` deps
  (`ab_glyph` → `skrifa`+`zeno`); deleted the `-0192` `deny.toml` entry; all 10
  `src/text` tests (6 existing + 4 new) pass. All gates green: `cargo test` (423
  passed), `cargo clippy --all-targets -D warnings` clean, `cargo fmt --check` clean,
  `cargo build --no-default-features` compiles, `cargo deny check advisories bans
  sources licenses` passes, `cargo tree` shows 0 `ttf-parser` / 0 `ab_glyph`. Manual
  `watermark --text "© me 2026"` check confirmed legible rendering at the gravity
  anchor.
- [ ] verify — independent Explore subagent (Opus).
- [ ] ship.
