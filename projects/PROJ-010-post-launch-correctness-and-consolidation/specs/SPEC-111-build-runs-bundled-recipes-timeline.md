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
      **Second build session, 2026-08-08 — PUNCH LIST pass (record accuracy only, no
      behaviour change).** Verify returned ⚠ PUNCH LIST on PR #138. Fixed all three
      record-accuracy items: narrowed DEC-087's "complete" recipe claim to the pixel steps
      (quality is not a recipe field); re-justified `cache.rs`'s module doc, which had
      claimed the format-implies-hit invariant follows from "a pure function of the input
      bytes and extension" — SPEC-111 itself falsifies that, the real basis is
      `target_recipe_hash` folding the Pin/Decide plan into the key; and corrected
      `SPEC-111-verify.md:63` (merged to `main` in #139), which called the cache-collision
      risk "a real pre-existing defect" — it is a regression this spec's own new capability
      would have introduced, caught inside the same change, and the mischaracterization was
      the orchestrating architect's transcription error, not this build's. Decided and
      recorded three further non-blocking items verify raised: strengthened the weak AC-7
      test (it used `--check`, which never writes, so it never drove a real cache hit —
      now deletes the output and re-runs for real, asserting the "(1 cached, 0 rebuilt)"
      summary and byte identity); named the orphaned-artifacts gap (an extension flip on
      `{ext}`/Decide leaves the old file in `out`; pre-existing class, newly reachable, not
      fixed) in DEC-087; and broadened `docs/api-contract.md`'s exit-4 wording to cover a
      no-extension `name` template (`{stem}`), not just an unrecognized literal one — same
      exit code, previously undocumented case. Re-ran the default leg through `rtk proxy`:
      838/838, 0 failed, exact match to the first build session's reference count; `just
      wasm-test` 30/30; clippy/fmt clean. Lean/webp-lossy relied on CI rather than a local
      re-run, given the change surface (doc comments, markdown, one test body, no
      production code). **Read the CI legs after pushing:** polled PR #138 to completion —
      12/12 green (`build/test/clippy/fmt` × macOS/Ubuntu/Windows, `avif`/`webp-lossy`/
      `heic` feature legs, `heic` × macOS/Ubuntu, `lean build`, `msrv`, `cargo-deny`,
      `cost-capture audit`, `front-matter validation`), cross-checked against `rtk proxy
      gh pr checks 138`'s raw per-job table since the plain command is itself
      `rtk`-rewritten to a summary. PR #138, still not merged — `mergeable: CONFLICTING`
      against `main`, an expected add/add conflict on `SPEC-111-verify.md` (this branch
      materializes it with the Item-3 fix; `main` already has the unfixed version via
      #139), for whoever merges to resolve.

- [ ] **verify** — fresh session, **Opus**. Re-derive the driven table yourself on your own
      builds of branch and `main`. Enumerate every path that builds a pipeline from a `Recipe`
      rather than trusting this spec's list — SPEC-110's roster omitted a verb and it cost a full
      extra build cycle.

- [ ] **ship** — bookkeeping on `main` after the PR merges: cost totals, reflection,
      `just archive-spec SPEC-111`, stage backlog. STAGE-039 closes only once the
      `docs/data-model.md` chore also lands.
