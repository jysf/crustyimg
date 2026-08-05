# SPEC-110 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-110-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-03. Drove a purpose-built `Orientation=6` fixture (stored 1200×800,
      correct display 800×1200) through every pixel-lane verb on a release build at `d854038`
      before writing anything. **The sweep STAGE-039 framed as a check is where most of the
      defect lives:** `convert` (both formats), `resize`, `thumbnail`, `responsive`, and `edit`
      without its flag — **seven invocations return a sideways image, and every one also drops
      the EXIF**, so the information needed to correct the output is destroyed by the same
      operation that made it wrong. `web`, `optimize`, `auto-orient` and `edit --auto-orient`
      bake correctly. Full table in the spec's Context.
      **The current split is not a design** — nothing distinguishes the two groups except which
      pipeline builder each verb happened to call (`optimize_pipeline()` pins `auto-orient`
      first; every other handler builds its own without it).
      **DEC-003's own falsifiability condition is currently false.** It wrote its success test
      as *"Right if: a resize preserves orientation…"* and asserts *"Orientation/ICC survive
      transforms"*; `AGENTS.md:448` repeats it. A `resize` today neither preserves the tag nor
      bakes it — a decision record that has stopped describing the code, the same decay that
      made the launch board read red for two weeks.
      **Why nothing caught it:** all five callers of the orientation fixture builders outside
      `tests/common` are on verbs that already bake (`auto-orient`, `optimize`) or on lint. **No
      test asserts orientation behaviour on any of the five broken verbs.**
      **Maintainer decision: bake everywhere** — pin the existing `auto-orient` operation first
      on every pixel-lane verb. Rejected: preserve-the-tag (more faithful to DEC-003 on paper,
      but needs per-format container-lane writes and still renders sideways in EXIF-ignoring
      viewers) and split-by-verb-intent (two rules, and the seam is where the next bug hides).
      The "convert must stay byte-faithful" objection is weaker than it looks: `convert` already
      discards all metadata, so it is not faithful in any archival sense today.
      **Sub-decision:** `edit --auto-orient` becomes an accepted, documented no-op — it cannot be
      removed (CLI frozen, STAGE-030) — and **no opt-out flag is added** (filed, not built, on
      DEC-063's `--max-pixels` precedent).
      Wrote 11 acceptance criteria and 9 failing tests plus a negative control. Two traps
      called out explicitly: **a double rotation** is the obvious failure mode (AC-2), and **a
      square fixture would make the whole spec vacuous** (AC-4).
      **Un-metered main-loop cycle** (AGENTS §4): one fixture build, ~15 driven invocations on a
      release binary, plus an audit of the five existing orientation-fixture callers.

- [x] **build** — 2026-08-04, Sonnet, own worktree (`feat/spec-110-orientation`). Factored
      `optimize_pipeline()`'s existing `auto-orient` push into a shared `auto_orient_prefix()`
      (`src/cli/optimize.rs`) and wired it into `run_convert`/`run_responsive`
      (`optimize.rs`) and `run_resize`/`run_thumbnail`/`run_edit` (`ops.rs`) — the six sites
      the mechanical grep of every `Pipeline::new()` construction in `src/cli/` identified as
      in-scope (`run_auto_orient` already IS the op; `apply`/`build`'s recipe-driven pipeline
      and `run_watermark` are out of scope, not in the measured table). `edit`'s
      `build_edit_ops` is unchanged — `--auto-orient` still adds its own explicit op, now
      redundant-but-safe (idempotent) alongside the prefix.
      9 tests in a new `tests/orientation.rs` (AC-1 through AC-6 directly; AC-7/AC-8/AC-9
      are structural/doc, not test-shaped). **AC-10 negative control:** reverted the prefix on
      `convert`, confirmed 2 tests go RED (`convert_bakes_orientation_into_pixels` AND
      `all_eight_orientation_values_are_applied`, since AC-5's representative verb is
      `convert`), restored, confirmed GREEN — each rebuild verified via a changed binary MD5
      (reverting source does not rebuild the binary by itself; proved the artifact actually
      changed each time). **AC-11 full matrix**, fresh per-leg `CARGO_TARGET_DIR`, sequential,
      every leg through `rtk proxy`, every log confirmed `Compiling crustyimg`: lean 804 /
      default 823 / webp-lossy 830 passed, 0 failed — reconciles exactly against a freshly
      measured `origin/main` baseline (795/814/821, i.e. the build prompt's stated
      797/816/823 reference was stale by 2 in every leg) plus the 9 tests added.
      `just wasm-test` 30/30. `cargo fmt --check` clean. New DEC-086 (bake on every
      pixel-lane verb) + a dated amendment to DEC-003 + `AGENTS.md:448`'s glossary line
      corrected. Full readout in the spec's `## Build Completion`.

- [x] **verify** — 2026-08-05, Opus, own worktree. **⚠ PUNCH LIST** — PR #133 NOT merged.
      Re-derived everything on own release builds of the branch and `origin/main`, with
      ImageMagick + exiftool as oracles independent of the code under test. The design's
      measured table reproduces exactly on `main`; the branch fixes all seven wrong cells.
      **AC-1/AC-2/AC-3/AC-4/AC-5/AC-6/AC-10/AC-11 confirmed**, several beyond what they
      asked: AC-2 was driven at pixel-CONTENT level across all eight orientation values (a
      dimensions-only check cannot see a 180° double-bake or a mirror flip) and shows no
      double rotation anywhere, including `edit --auto-orient`, which genuinely does run
      `AutoOrient` twice; AC-3's byte claim holds on 24/24 outputs main-vs-branch with an
      orientation-6 negative control that separates 7 changed / 5 unchanged. Test counts
      re-measured both sides: main 795/814/821 → branch 804/823/830, **+9 in every leg**,
      matching the 9 tests in `tests/orientation.rs`; `ignored=2` confirms the build
      prompt's reference numbers were passed+ignored (the architect's stale-by-2, real).
      AC-10 re-run, not read — and proved **behaviorally** (the rebuilt binary drove
      1200×800) rather than only by changed MD5, which shows a rebuild happened but not
      that the revert took effect. **AC-7 REFUTED:** `watermark` still returns 1200×800
      where 800×1200 is correct, so the spec's Goal — no shipped verb can hand back a
      sideways image — is false on this branch, and **DEC-086 is false on the day it is
      written** (its title, Decision and Consequences all assert every pixel-lane verb
      bakes). Own grep, scope stated as a claim, confirms watermark is the ONLY missed
      site. Item 6 re-characterized: the `edit --save-recipe` divergence is **introduced
      by this PR**, not pre-existing — on `main` an `edit --invert` recipe replayed to the
      same geometry. Full punch list in the verify readout.

- [ ] **punch list** — see the verify readout: fix `run_watermark`'s prefix + test
      (blocking), correct DEC-086 / the DEC-003 amendment / `optimize.rs:782`'s doc
      comment (blocking), re-characterize the save-recipe gap, and drop
      `cli-reference.md`'s now-false "byte-pinned to what `apply` produces".

- [ ] **ship** — bookkeeping on `main` after the PR merges: cost totals, reflection,
      `just archive-spec SPEC-110`, stage backlog. STAGE-039 also holds SPEC-111 and a doc
      chore, so shipping this does **not** close the stage.
