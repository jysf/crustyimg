# SPEC-119 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-119-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. **11 ACs** (10 at framing; AC-7b added 2026-08-16 when Call 4
      was revised), 4 design calls, 9 pre-written tests.
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
- [x] **verify** — 2026-08-16, Opus. **⚠ PUNCH LIST** (3 items, all documentation).
      $30.99 / 47,880,996 tokens / 28 min. Headline: *"the implementation is correct and I could
      not break it"* — all 11 ACs re-driven against fixtures built OUTSIDE the repo and validated
      with decoders it doesn't ship, plus 96 byte-identity pairs vs `main`.
      P1 `responsive`/`apply`-plain/`build`-plain still flatten silently, so the Goal is not met;
      P2 the new api-contract paragraph is the exhaustive-sounding short list its own neighbour
      warns against; P3 `lint --max-warnings 0` returns a FALSE GREEN in directory mode on
      animated WebP — the shape CI uses, and the shape Call 1 was accepted on.
      Also: measured the detector's cost (+3.4 ms on animated GIF, controls flat), and caught
      itself in the reverting-source-does-not-rebuild trap mid-run, then re-drove every headline
      claim against a correctly-rebuilt binary.
      ⚠ Its cycle-advance + cost commit `c920cb9` is **local on a detached HEAD, unpushed** — the
      cost block is transcribed below and lands on `main` at ship per §13.
- [x] **punch list** — 2026-08-16, `d9f2d8c`. All five items, documentation only; no
      `src/` or `tests/` change, `cycle:` left at verify. Went beyond the brief: found that
      `info` warns for truncated-JPEG but **not** animated-input, so the two warnings have
      genuinely different verb sets.

      change, no re-verification, and `cycle:` stays put.
- [~] **verify (re-read)** — prompt: `prompts/SPEC-119-verify-reread.md`. Opus, own worktree.
      Deliberately narrow: close the three punch-list items, and settle the NEW unverified claim
      about `info` — which may itself be a defect worth filing.
      Five items flagged, AC-7b load-bearing: `lint --max-warnings 0` is the answer to the
      maintainer's reservation about warn-and-proceed, so it must be driven on all three
      families, not asserted.
- [ ] **ship** — **at ship:** append the verify cost entry
      (`agent: claude-opus-5`, `tokens_total: 47880996`, `duration_minutes: 28.3`,
      `estimated_usd: 30.99`, breakdown `{input: 538, output: 179350, cache_creation: 461199,
      cache_read: 47239909}`), compute `cost.totals` (**191,351,851 / $82.23**), run
      `just cost-audit`.
