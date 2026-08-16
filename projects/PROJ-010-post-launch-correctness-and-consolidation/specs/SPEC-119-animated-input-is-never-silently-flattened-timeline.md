# SPEC-119 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-119-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. 10 ACs, 4 settled design calls, 9 pre-written tests.
      The multi-frame sweep ran at design and changed the scope: the defect is
      **three formats (GIF, APNG, animated WebP), not one**, so complexity moved
      S → M against the stage's estimate.
- [x] **build** — 2026-08-16, Sonnet. PR #176, 16/16 CI green, 12 files, +1204/−82.
      **$51.24 / 143,470,855 tokens / 61 min** — the most expensive build of the wave, 4.3× the
      cheapest, driven by message count rather than duration (see STAGE-042's budget item).
      Went beyond the spec in one good way: `lint` now reads `Image::is_animated_input` instead
      of re-decoding, so the linter and the pixel path **cannot** disagree — that divergence was
      the root cause. AVIF settled by reading `avif-parse` (hard-rejected pre-decode, no code
      needed). Chose a separate `format/animated-input` rule over broadening the GIF rule.
      ⚠ Its DEC shipped as DEC-092, colliding with SPEC-120's; **renumbered to DEC-093** by the
      orchestrator in `d66a357`.
- [ ] **verify** — prompt: `prompts/SPEC-119-verify.md`. Opus, new session, own worktree.
      Five items flagged, AC-7b load-bearing: `lint --max-warnings 0` is the answer to the
      maintainer's reservation about warn-and-proceed, so it must be driven on all three
      families, not asserted.
- [ ] **ship**
