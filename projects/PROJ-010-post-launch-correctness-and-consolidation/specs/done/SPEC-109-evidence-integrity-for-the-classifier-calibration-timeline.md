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

- [x] **verify** — 2026-07-26, @ `2006cc4`. **All 11 ACs verified**; readout in
      `prompts/SPEC-109-readouts.md`. Nothing blocking.

      **The gate, re-derived on both trees.** Pre-SPEC-109 at 5.5: **52 passed, 0 failed** —
      the premise reproduced, the guard could not see the bug value. Branch at 5.5: **RED
      (52/2)**; at 3.2: **RED (51/3)**; at 4.0: 54/0. The 7.0 control fires on both trees
      (3 failures before, 4 after), so the greens are results and not un-recompiled builds.
      Window `(3.6414278, 4.5176096]`, width `0.87618184`, matching an independent Python
      measurement to seven figures. Dropping either specimen from the roster re-widens past
      the 1.20 cap and the guard says so (1.4917 / 2.4329).

      **Driven, not read.** The SPEC-084 branch at `optimize.rs:1059` is genuinely reached —
      a `panic!()` there fires under `spec_084_metadata_forced_fallback_is_reached` and
      *not* under the ICC test — and mis-conditioning the call site turns the test red.
      AC-8 is not a tautology: on the lean leg the test goes red both when `ssim` is removed
      from the golden set and when a real per-verb fork is injected. AC-5's honesty
      assertion goes red on default *and* lean when `exceeds_source()` is inverted. AC-7's
      `photograph` verdict collapses to `graphic-logo` at threshold 7.0, so it is rule 3.5
      on the no-EXIF path.

      **Clean full matrix, fresh `CARGO_TARGET_DIR` in an isolated worktree**, all seven
      legs exit 0, `Compiling`/`Checking crustyimg` on each: lean **776**, default **796**,
      **`webp-lossy` 803 passed / 0 failed** — the loose end handed to this cycle is
      resolved, the earlier "0" was a log-capture artifact.

      **Scope: `one-spec-per-pr` is satisfied.** `cost-snippet.md` was moved out at
      `2006cc4` and is byte-identical to `main`; the change lives on
      `chore/cost-measurement-methodology`.

      **Four findings, none blocking.** (1) `rtk` dropped the newest commit from
      `git log main..HEAD` — three reported, four real — which would have produced a false
      scope violation. (2) The build's "27 `cfg(feature)` under `tests/`" reproduces under
      no scope I tried (19 / 20 / 26 / 33 / 41); the load-bearing half — `audit_bench` holds
      0 real gates — is exact. (3) DEC-047's revised "≤3.64 counting dithers-of-photos" is
      the chosen specimen's value, not a ceiling: the same recipe on the repo's Canon frame
      measures **3.8396**, cutting the margin to 4.0 from 0.36 to 0.16 bits. (4) DEC-047's
      "6.02" parenthetical re-measures at **6.03**; same verdict.

      **Not checked:** CI on any OS but this one, the build session's cost figures,
      SPEC-105's 48-crop 4.58 floor (sources absent from the repo), DEC-047 outside the two
      corrections, fixture provenance claims, `--profile docs`, wasm.

- [x] **ship** — 2026-07-27, merged as `408b0f9` (PR #114, squash). Amended DEC-047 in place
      (two false claims corrected + the two specimens added to its evidence roster) and emitted
      no new DEC of its own. A **third** DEC-047 correction landed post-verify as `ae01cb4`,
      docs-only: its revised "≤3.64 counting dithers-of-photos" was this spec's specimen value
      restated as a class ceiling, refuted by the Canon frame at 3.8396.
      Cost totals: **86,491,591 tokens / $60.97** across 3 sessions (design un-metered; build
      65,339,132 / $43.21; verify 21,152,459 / $17.76 — both component-priced, both measured
      from transcripts rather than estimated). Archived to `specs/done/` by hand with `git mv`,
      since `just archive-spec` mis-targets `specs/prompts/*.md`.
