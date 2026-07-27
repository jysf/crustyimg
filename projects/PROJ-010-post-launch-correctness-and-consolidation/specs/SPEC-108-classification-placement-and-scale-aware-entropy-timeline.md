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

- [x] **build** — 2026-07-27. SPEC-109 (#114) had already landed, so no specimen-sequencing
      decision was needed. Moved `Analysis::compute` to run on the source image before
      `pipeline.run` in `optimize_decide_one` (`src/cli/optimize.rs`), threading the verdict
      + a source-derived `has_alpha` to the decision site (AC-1–4); confirmed the second
      `pipeline.run` site (`:160`, a plain-pixel-recipe path with no classify step) and a
      third found by grep (`:1600`, `responsive`, fixed formats, no classify step) don't need
      the same treatment. Split rule 3.5 into an unconditional zone
      (`entropy >= DOC_ENTROPY_MAX`) and a contested zone that defers to the graphic gates
      (AC-5); deleted rule 6 and its two constants as confirmed-unreachable dead code (AC-6).
      Found, en route, that `has_alpha` had been read from the post-pipeline buffer (always
      RGBA internally) rather than the source, so a JPEG — which never has real alpha —
      reported `true`; fixing it (AC-2's own requirement) also fixed the has_alpha-driven
      cross-verb schema fork SPEC-109 flagged as this spec's to fix, but required updating
      three pre-existing tests whose fixtures had unknowingly depended on that bug to
      reproduce a "nothing beats source" case. `format_shortlist`'s `Lossy`+alpha arm now
      also offers lossless WebP (AC-7); `--profile docs` now downgrades a promoted photograph
      to lossless too, not just the ambiguous bucket (AC-8, a genuine behavior decision, not
      just a doc gap). New DEC-084 records the placement decision, the refutation of the
      narrow alternative, and all four AC-5–8 sub-decisions. Full matrix clean in three
      isolated fresh `CARGO_TARGET_DIR`s (a concurrent shared-target-dir run corrupted the
      first lean-leg attempt — re-run isolated, see build notes): lean 783 / default 802 /
      webp-lossy 809, all zero failures (exceeds the 776/796/803 reference). `clippy -D
      warnings` clean on all three legs; `fmt --check` clean. Mutation control confirmed:
      `PHOTO_ENTROPY_STRONG = 5.5` still goes RED (`calibration_gap_matches_the_documented_gap`).

- [ ] **verify** — not started. Engine change ⇒ **clean full matrix**, re-run by the
      orchestrator rather than relayed: default / `--no-default-features` /
      `--features webp-lossy`, `clippy -D warnings` each, plus `fmt --check`. Confirm the log
      says `Compiling crustyimg`. The gate that matters is the mutation control: with
      `PHOTO_ENTROPY_STRONG = 5.5`, at least one test must go RED. It currently leaves
      `cargo test --release --lib analysis` green at 52/52.

- [ ] **ship** — not started. Emits a new DEC for "classify the source, not the pipeline
      output", which must record the measured refutation of the narrow alternative so it is
      not re-proposed.
