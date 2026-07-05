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
- [x] verify (2026-07-04) — independent Explore subagent (Opus, 59862 tok, ~4 min).
  Adversarial review of the rasterization port (y-negation across all path commands,
  `alpha = round(cov/255 * base_alpha)`, row-major buffer indexing, whitespace/bounds
  union, source-over keep-larger-alpha) + API preservation + hardening (no new panics on
  the font-bytes path) + no scope creep; re-ran all gates. VERDICT **PASS**, no defects.
  Orchestrator also ran a visual old-vs-new pixel A/B (mean channel diff 3.07; legible,
  same placement — the few-px width delta is the documented kerning drop).
- [x] ship (2026-07-04) — Opus, main loop. Squash-merged PR #49 → `main` (6d79f1b); all
  19 PR checks + main CI green. Recorded real cycle tokens; archived the spec; STAGE-010
  backlog updated (1/2 → SPEC-044 shipped). **First `deny.toml` ignore eliminated**
  (RUSTSEC-2026-0192 gone; `ttf-parser`/`ab_glyph` out of the tree) toward the clean 0.2.0.
