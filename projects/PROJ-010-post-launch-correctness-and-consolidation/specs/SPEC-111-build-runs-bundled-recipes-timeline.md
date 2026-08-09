# SPEC-111 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-111-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-07. **STAGE-039 framed this as needing no design cycle; it does.**
      Drove the failure on a release build at `3dd8fa7` with a real manifest: `build` exits **1**
      with `unknown operation 'optimize'` on a bundled recipe **both by file path and by name**,
      while two controls pass — `apply --recipe web` writes a real AVIF from the same source, and
      a plain pixel recipe through `build` exits 0. So the fault is precisely the terminal marker.
      **The framing's fix ("wire in the strip helper") is necessary but not sufficient, and
      shipping only that would be a worse bug than the current one.** `encode_one`
      (`src/cli/common.rs:52`) hardcodes `img.source_format()` — *"no --format override in batch
      path v1"* — so stripping alone would run the pixel pipeline and then write the result in the
      **source** format, silently discarding the modernization the recipe exists to perform. The
      current failure at least fails loudly.
      **Design question 1 — what picks the output format?** Answered by copying `apply`'s existing
      rule rather than inventing one: `run_apply` (`optimize.rs:80-102`) skips the decision when
      the format is **pinned**, so that `apply --recipe web hero.jpg -o hero.png` matches
      `web hero.jpg -o hero.png`. **Decision: `build` uses the name template as the pin** — a
      literal extension (`{stem}.png`) pins and skips the decision; `{ext}` (incl. the default)
      lets the decision choose and expands to it. `build.rs:575` already contemplates
      literal-extension templates. One rule across `apply` and `build`, which is the lesson
      SPEC-110 paid three cycles for.
      Encouraging: **`build` already anticipates a post-decode extension** — `EXT_SENTINEL`
      (`build.rs:115`) exists because *"the real output extension is only knowable after a
      decode"*, and `lock_output_path` already takes `ext` as a parameter.
      **Design question 2 — the recipe divergence SPEC-110 introduced.** **Decision: `edit
      --save-recipe` records `auto-orient` explicitly**, rather than giving `apply`/`build` an
      implicit prefix. A recipe is a record of what happened; an implicit prefix would make a
      recipe no longer a complete description of its own behaviour, which is the pattern SPEC-110
      just spent three cycles removing. And the precedent is already in the repo:
      **`recipes/web.toml` names `op = "auto-orient"` as its explicit first step.**
      Wrote 11 acceptance criteria and 10 failing tests plus a negative control. Complexity
      rated **M**, not the S–M the stage assumed.
      **Un-metered main-loop cycle** (AGENTS §4): one manifest fixture, four driven invocations
      with two controls, and a trace of the format path through `encode_one` →
      `lock_output_path` → `EXT_SENTINEL`.

- [x] **build** — 2026-08-08. `feat/spec-111-build-recipes`, own worktree, Sonnet. Both design
      decisions implemented as specified: `split_terminal_optimize` (`optimize.rs`) made
      `pub(super)` and reused (not copied) by `build.rs`'s `prepare_target`; `encode_one`
      (`common.rs`) gained a `format_override: Option<ImageFormat>` param (`None` on `apply_one`'s
      call site — byte-identical, AC-6) so `build` can thread a PIN through without touching the
      preserve-format path; a new `encode_one_optimize_decided` (`optimize.rs`) reuses
      `optimize_decide_one` for the auto-decide case, hardcoding `Profile::Web` to match
      `run_apply`'s own hardcode (decision 1's "one rule"). `edit --save-recipe` now prepends an
      explicit `auto-orient` `RecipeStep` unless `--auto-orient` already put one there (decision 2).
      **Sweep** (`.build_pipeline(` grep, 10 call sites, each classified): the only unfixed
      production site sharing this defect class is `src/wasm.rs::transform` — explicitly out of
      scope, named in DEC-087, not fixed.
      **Found beyond the spec's Outputs list, driven by implementing decision 1:** two `build`
      targets sharing one recipe file but different name templates (Pinned vs Decide) would
      collide in the content-addressed cache under the pre-existing `recipe_hash` formula — fixed
      via `target_recipe_hash`, scoped so a plain pixel recipe hashes exactly as before (no stale
      cache/lockfile). Also named, not fixed: `build`'s new Decide path reaches
      `optimize_decide_one` but doesn't thread SPEC-107's truncated-JPEG warning through.
      AC-4 negative control driven for real: mutated `split_terminal_optimize` to drop the last
      step unconditionally, confirmed `build_still_rejects_an_unknown_terminal_op` went RED, and
      confirmed the mutation reached the compiled test binary (SHA-256 changed, a new dead-code
      warning appeared), not just the source — then reverted and reconfirmed green.
      **One pre-written failing test needed a fix, not just a pass:** AC-2's hardcoded ".avif"
      assumption doesn't hold on the `webp-lossy` leg (a second competing lossy candidate makes
      the byte-race winner measured, not assumed) or, for a related new test, on `lean` (falls back
      to a baseline JPEG, no avif/webp-lossy built) — both correct decision-engine behavior, not
      bugs. Re-gated the AVIF-specific assertion to `avif && !webp-lossy`; added an unconditional
      `build_decided_format_matches_apply_on_every_feature_leg` proving the real "one rule"
      requirement (byte-parity with `apply --recipe web`) on every leg.
      **Full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential, through `rtk proxy`, "Compiling
      crustyimg" confirmed each leg:** lean 818/818 (805 reference + 13 new), default 838/838 (824
      + 14 — one extra test is `avif`-gated so only default/webp-lossy carry it), webp-lossy
      844/844 (831 + 13), `clippy --all-targets -D warnings` clean on all three, `fmt --check`
      clean, `just wasm-test` 30/30 (untouched by construction — `cli`/`build`/`source`/`lint` are
      `#[cfg(not(target_arch = "wasm32"))]`, so nothing this spec touched compiles into wasm).
      **Read the CI legs (not just the local matrix):** PR #138's first CI run showed 26/27
      green and one red — "build + browser smoke" (Pages workflow's headless-Chrome demo
      smoke), `no DevToolsActivePort`. Nothing in this PR touches the wasm-compiled surface
      (`cli`/`build`/`source`/`lint` are all `#[cfg(not(target_arch = "wasm32"))]`) or
      `demo/`, and the failure signature is a Chrome-launch infra hiccup (the wasm build,
      size report, and demo assembly all succeeded before it; only spinning up the browser
      itself failed) — but per SPEC-107's own lesson, that reasoning alone isn't a green.
      Re-ran just the failed job with no code change: it passed. 27/27 green.
      PR #138, not merged.

- [x] **verify** — 2026-08-08. Opus, own detached worktree (`crustyimg-spec111-verify`), plus two
      throwaway worktrees for mutation controls and a `main` baseline. **⚠ PUNCH LIST** — every
      acceptance criterion holds under driving; two documentation claims are over-broad.
      **Re-derived, not inherited.** Built four release binaries (branch, `main`, `webp-lossy`,
      mutant) and drove every AC on bytes: AC-1 all **six** routes (web/gallery/product × bundled
      name and file path) exit 0 and write real `ftypavif`, with a 3000×2000 source proving the
      three recipes are distinct (2048/2560/1600 wide, three distinct hashes) — all six fail
      identically on `main` with `unknown operation 'optimize'`. AC-2 build == `apply --recipe X`
      **byte-for-byte** on all three recipes. AC-3 literal-extension template writes real PNG
      magic. AC-4/AC-5 exit 1, nothing written. AC-6 plain pixel recipe is byte-identical AND
      **lockfile-identical** across `main` and branch (same cache key `6087044f…`), and `apply`
      is unchanged for both a plain and a bundled recipe. AC-7 driven as a real hit: delete the
      output, re-run, `1 cached, 0 rebuilt` re-materializes `dist/photo.avif` at the same 6755
      bytes the lock records. AC-8/AC-9 `edit --invert --save-recipe` and its replay are
      **byte-identical** (not merely same-dimension), recipe TOML = `[auto-orient, invert]`,
      independent SOF parse confirms 800×1200; `main` gives `[invert]` and 1200×800.
      **AC-10 `--watch` driven** (DEC-087 had judged it not worth a harness): initial build +
      a debounced rebuild on a new source both write real AVIF, no lockfile under watch,
      `--watch --check` is exit 2; `--check` also correctly reports **drift with exit 7** when a
      content change flips the decided extension.
      **Sweep re-run and tightened.** `grep -rn build_pipeline src/` → 17 hits, 11 real call
      sites (7 production, 4 `#[cfg(test)]`), positive control on a misspelled token = 0. Scope
      closed one level deeper than the build's: `registry.build(&step.op, …)` — the only route
      from a recipe step's op *name* to an `Operation` — has exactly **one** production caller,
      `Recipe::build_pipeline` (`src/recipe/mod.rs:280`), so no path can reach a pipeline from a
      `Recipe` without appearing in that grep. `benches/`/`fuzz/`/`examples/` reference `Recipe`
      zero times. The build's roster is complete; `src/wasm.rs::transform` is the only unfixed
      production site.
      **Mutation controls, all driven against rebuilt binaries.** (1) AC-4: drop-last-
      unconditionally turns `build_still_rejects_an_unknown_terminal_op` RED (exit 0 vs 1),
      test-binary SHA `3130c78c…` → `6ff1afa8…`, revert restores green. (2) The cache-key fix is
      load-bearing: reverting `target_recipe_hash` to plain `recipe_hash` makes a Pinned target
      serve the Decide target's cached bytes — `dist/pinned/photo.png` containing `ftypavif`,
      both lock entries sharing key `5893c36c…`, exit 0, silent. That is the exact
      AVIF-in-a-`.png` bug the spec exists to prevent. (3) AC-2's coverage question resolved:
      forcing `Decide → Preserve` turns the unconditional
      `build_decided_format_matches_apply_on_every_feature_leg` RED on **webp-lossy**
      (`photo.webp` vs `photo.png`) *and* on **lean** (`photo.jpg` vs `photo.png`), so the
      criterion is pinned on every leg despite the AVIF-specific assertion being single-leg.
      (4) The wasm exception driven on the **real wasm target**, not read:
      `transform(png, recipes/web.toml, "png")` → `unknown operation 'optimize'`.
      **The cache collision was NOT pre-existing** — refuted. It is unreachable on `main` (the
      terminal-`optimize` target dies at prepare), and the closest `main`-reachable shape (plain
      recipe, two templates) shares a key and correctly serves identical bytes. DEC-087's own
      wording ("that invariant … held before this spec") is accurate; it is a regression this PR
      would have introduced and caught in the same change. SPEC-065 is intact and was already
      designed for the literal-vs-`{ext}` case: same-`out` Pinned+Decide is caught post-write by
      the lockfile collision check (exit 2, no lockfile) — untouched by this PR.
      **Matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential, `rtk proxy` from the first leg,
      `Compiling crustyimg` present in each:** default 838/838, lean 818/818, webp-lossy 844/844
      (34 suites, 0 failed, cross-checked with `python3`), `clippy --all-targets -D warnings`
      0 errors / 0 warnings on all three, `fmt --check` clean, `just wasm-test` 30/30 — exactly
      the build's numbers. `just validate` 248 blocks ✓, `just cost-audit` ✓,
      `just decisions-audit` 0 structural errors.
      **Punch list (both documentation-only, no code change):** (1) DEC-087 Consequences calls a
      saved recipe "a **complete**, replayable description of what `edit` did" — driven false by
      `edit --invert -q 40`, whose quality is not a recipe field, so the replay differs in bytes
      unless `-q 40` is passed again. Narrow to the pixel steps. (2) `src/build/cache.rs`'s module
      doc still asserts the output format "is a pure function of the input bytes and extension —
      both already keyed — so a hit implies the same format." SPEC-111 falsifies the premise; the
      conclusion now holds only because `target_recipe_hash` folds the plan into `recipe_hash`.
      DEC-087 explicitly declines to amend it, which leaves the cache's correctness core carrying
      a false justification for the invariant this PR had to work to preserve.
      PR #138 not merged; verify branch `verify/spec-111-build-recipes`, unpushed.

- [ ] **ship** — bookkeeping on `main` after the PR merges: cost totals, reflection,
      `just archive-spec SPEC-111`, stage backlog. STAGE-039 closes only once the
      `docs/data-model.md` chore also lands.
