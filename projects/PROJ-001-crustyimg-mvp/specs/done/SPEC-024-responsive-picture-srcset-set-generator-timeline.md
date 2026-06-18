# SPEC-024 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-024-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`), emitted DEC-026 (responsive command + HTML
  emission in-scope + scope/deferrals), and added a `responsive` entry to
  `docs/api-contract.md`. Composition over the resize `fit`-by-width primitive + the
  per-format sink; HTML is dependency-free string building. v1 = width×format
  variants + `<picture>`/srcset to stdout; blurhash/perceptual-per-variant/batch
  deferred.
- [x] build (2026-06-18, PR #27) — Opus, main loop. Added `Commands::Responsive`,
  `run_responsive`, `parse_widths`/`parse_formats`/`mime_for_format`/
  `responsive_quality`/`fit_width_params`/`build_picture_html` + 6 unit + 8
  integration tests. Composition over the resize `fit` op + per-format sink; no new
  dep. `--out-dir` reuses the global flag (deviation, documented). All gates + 3-OS +
  feature CI green.
- [x] verify (2026-06-18) — independent read-only Explore subagent: ✅ APPROVED.
  Confirmed the fit-by-width math (width binds, no upscale), decode-once,
  no-upscale+dedup, feature-gate-up-front (no files on unbuilt codec), srcset uses
  actual widths; all 14 named tests present; no `unwrap` outside tests.
- [x] ship (2026-06-18, PR #27 squash-merged) — reflections + cost totals filled
  (index-verified), STAGE-009 backlog flipped, archived to `specs/done/`,
  `just cost-audit` green + cost-capture confirmed on main CI.
