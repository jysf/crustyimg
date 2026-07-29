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

- [x] **verify** — 2026-07-27/28. All AC-1–9 independently re-derived (not inferred from green
      tests) via direct CLI driving on my own release builds of branch and `main` (`14eab28`).
      Mutation control confirmed live: `PHOTO_ENTROPY_STRONG = 5.5` goes RED
      (`calibration_gap_matches_the_documented_gap`, 54/55), a `= 7.0` must-fail control also RED
      (52/55, two more tests fail), cleanly reverted (55/55, empty diff). Full matrix, isolated
      sequential `CARGO_TARGET_DIR`s: lean 783/0 (main re-measured 776/0, exact match), default
      802/0, webp-lossy 809/0 — deltas (+7/+6/+6) reconcile exactly against the 7 new tests (6 on
      every leg, 1 lean-only via `cfg`). `clippy -D warnings` and `fmt --check` clean on all three
      legs. All three modified pre-existing tests judged and independently confirmed as corrected
      truth (fixture-mechanism swaps forced by the has_alpha fix, not weakened assertions) — see
      readout for the empirical proof on each.
      **New finding (not in any AC or DEC-084):** EXIF fallout is real but only for EXIF carrying
      an actual orientation tag — `AutoOrient` no-ops (preserves metadata either way) on a
      zero-entry IFD, so the existing `jpeg_with_exif` fixture never exercised the risk DEC-084
      names. Quantified one genuine flip: a scanned-document-shaped fixture with real orientation
      EXIF goes `document`→lossless on `main` but `photograph`→lossy AVIF (SSIM 92.2 vs 100.0) on
      this branch. DEC-084 already names this as an accepted, revisit-later tradeoff, so not
      blocking, but should become its own fixture rather than staying open.
      **Second new finding:** a real, repeatable ~40% total-time regression on large
      (24 MP) few-colour/graphic inputs (484–507 ms vs 344–352 ms, isolated from encode/decode
      cost) — `Analysis::compute` is an O(pixels) full-buffer scan now running on the source
      instead of the post-resize buffer, contradicting `web`'s own "size-insensitive" help text
      for the lossless-class path specifically (the lossy/AVIF path is unaffected — encode
      dominates there). Full readout: `specs/prompts/SPEC-108-readouts.md`.

- [x] **ship** — 2026-07-28, merged as `a8694fd` (PR #121, squash). Emitted **DEC-084**
      recording the placement decision and the measured refutation of the narrow alternative,
      so it is not re-proposed. Two post-verify commits landed on the branch before merge:
      `6bde4b8` pinned the accepted EXIF trade with a fixture and corrected `web`'s
      size-insensitivity claim; `e0579eb` corrected two claims a focused re-verify falsified
      in that commit — a mislabelled assertion comment, and a "no benefit" framing drawn from
      an uncommitted fixture. Confirmed on merged `main`: the committed fixture measures
      `graphic-logo` / entropy 3.03 / 9 colours at every `--max` in {4096, 512, 256, 128}.
      Cost totals: **149,070,662 tokens / $103.69** across 3 sessions (design un-metered;
      build 103,470,877; verify 45,599,785) — see the spec's cost note for the Opus-vs-Sonnet
      anchor mismatch, which likely overstates the dollar figure by ~67%. Archived to
      `specs/done/` by hand with `git mv`, since `just archive-spec` mis-targets
      `specs/prompts/*.md`.
