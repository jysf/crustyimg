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
- [ ] build — Sonnet, prescriptive prompt. Rewrite `src/text/mod.rs` rasterizer on the
  probed API; swap `Cargo.toml` deps; delete the `-0192` `deny.toml` entry; make the 4 new
  + 6 existing tests pass; lean + full `deny` green.
- [ ] verify — independent Explore subagent (Opus).
- [ ] ship.
