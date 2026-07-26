# SPEC-109 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-109-<cycle>.md`.

## Instructions

- [x] **design** — 2026-07-26. Re-measured all five committed classify fixtures with a release
      build and verified all six guard sites against source. Confirmed the calibration guard's
      window is **(3.03, 6.07]** — 3.04 wide against a documented gap of 1.15 — with our own
      numbers, not the review's. Corrected the review's site list: `tests/cli.rs:4381` and
      `:4392` are the same test (doc comment vs signature), so the work is four test functions
      plus two `iso_luma` fixtures. Surfaced one finding the review did not have: all three
      photo fixtures measure `flat_ratio` 0.76–0.83, **above** `FLAT_GRAPHIC_RATIO` 0.60, with
      `edge_ratio` 0.00 — so rule 4b would claim every one of them if rule 3.5 did not fire
      first, which makes rule 3.5 load-bearing and validates SPEC-108's choice to change the
      classifier's input rather than its cascade.
      11 acceptance criteria, 7 failing tests. **Un-metered main-loop cycle.**

- [x] **build** — 2026-07-26. Branch `spec-109-classifier-evidence-integrity`. All 11 ACs met.

      **Mutation, before → after.** Before: `PHOTO_ENTROPY_STRONG = 5.5` left the suite
      **52 passed, 0 failed** — the design's number, reproduced. Before, control at 7.0:
      **49 passed, 3 failed**, which is what makes the 52/52 a result rather than a build that
      never picked up the edit. After: **5.5 → RED (52 passed, 2 failed)** and
      **3.2 → RED (51 passed, 3 failed)**. Both edges, so the guard is whole. At 4.0 the suite
      is 54 passed, 0 failed (was 52).

      The calibration window narrows **(3.03, 6.07] → (3.6414, 4.5176]** — 3.04 bits to 0.88 —
      and the guard now states its achieved bounds and width and caps the width at 1.20, so a
      dropped specimen re-widening the window fails instead of passing.

      Two boundary specimens committed, seeded by `scripts/seed-classify-specimens.py` from
      documented recipes and measured by an independent entropy implementation that reproduces
      all four pre-existing fixtures to four decimals. **Deviation:** the dither specimen is
      `dither_32color.png`, not `dither_16color.png` — 16 levels of a 6.07–6.83-bit source
      lands at 2.46–2.88, below the 3.03 dither already committed, so it could not tighten the
      window at all. Reasoning in `tests/fixtures/classify/RECIPES.md`.

      Also found: the schema-fork gate's stated cause was false (it is a `has_alpha`
      disagreement between `optimize` and `web`/`apply`, not an unscored JPEG winner), and
      SPEC-084 makes no never-bigger-than-source promise on the branch AC-5/AC-6 target — both
      recorded in `## Build Completion`.

- [ ] **verify** — not started. Clean full matrix (default / `--no-default-features` /
      `--features webp-lossy`, clippy `-D warnings` each, `fmt --check`), `Compiling crustyimg`
      confirmed. AC-8 un-gates a test on the lean leg, so the lean leg is not optional here.
      Check the mutation in **both** directions: 5.5 and 3.2 must each fail something.

- [ ] **ship** — not started. Amends DEC-047 in place (two false claims + evidence roster);
      emits no new DEC of its own.
