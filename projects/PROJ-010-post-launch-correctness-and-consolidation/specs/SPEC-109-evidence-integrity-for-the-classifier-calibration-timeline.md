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

- [ ] **build** — not started. **Land this before SPEC-108's build.** It is the instrument;
      building the fix first means measuring it with a guard never shown to move.
      AC-3 is the spec: with `PHOTO_ENTROPY_STRONG = 5.5` the analysis suite must go RED.
      Run the mutation before starting too, to establish the 52/52 baseline
      ([[a-control-you-never-verified-applied-is-not-a-control]]).

- [ ] **verify** — not started. Clean full matrix (default / `--no-default-features` /
      `--features webp-lossy`, clippy `-D warnings` each, `fmt --check`), `Compiling crustyimg`
      confirmed. AC-8 un-gates a test on the lean leg, so the lean leg is not optional here.
      Check the mutation in **both** directions: 5.5 and 3.2 must each fail something.

- [ ] **ship** — not started. Amends DEC-047 in place (two false claims + evidence roster);
      emits no new DEC of its own.
