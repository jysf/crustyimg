# SPEC-108 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-108-<cycle>.md`.

## Instructions

- [x] **design** — 2026-07-26. Chose **placement** (classify the source image) over the narrow
      rule-4 gating alternative, on measured evidence rather than a complexity estimate.
      Instrumented the committed fixture at four `--max` values with a release build and
      `web --json`; the feature table is in the spec's Context. The refutation turns on
      `unique_colors = 217 ≤ PALETTE_COLORS 256` at `--max 256`, which makes `many_colors`
      false, leaves rules 5 and 6 unreachable, and drops the input on rule 7's `Photograph`
      fallback. Wrote 9 acceptance criteria and 6 failing tests plus the
      `PHOTO_ENTROPY_STRONG = 5.5` mutation control.
      **Un-metered main-loop cycle** (one release build, four instrumented runs).

- [ ] **build** — not started. Sequencing note: AC-3 needs SPEC-109's boundary specimens.
      Either land SPEC-109 first, or commit the two specimens in this build and leave the
      guard rework to SPEC-109. Decide before starting.

- [ ] **verify** — not started. Engine change ⇒ **clean full matrix**, re-run by the
      orchestrator rather than relayed: default / `--no-default-features` /
      `--features webp-lossy`, `clippy -D warnings` each, plus `fmt --check`. Confirm the log
      says `Compiling crustyimg`. The gate that matters is the mutation control: with
      `PHOTO_ENTROPY_STRONG = 5.5`, at least one test must go RED. It currently leaves
      `cargo test --release --lib analysis` green at 52/52.

- [ ] **ship** — not started. Emits a new DEC for "classify the source, not the pipeline
      output", which must record the measured refutation of the narrow alternative so it is
      not re-proposed.
