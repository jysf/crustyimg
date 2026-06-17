# SPEC-017 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. This prompt is deliberately prescriptive — follow it literally.
> Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-017 ("--max-size byte budget for shrink and
convert"). You are NOT the architect; the spec is your source of truth. This
extends the SHIPPED SPEC-016 `src/quality` search (generalizing it) and wires a
`--max-size <SIZE>` byte budget into `shrink` and `convert`. It is JPEG-only and
OPT-IN: `shrink`/`convert` with no `--max-size` behave exactly as today. NO new
dependency, NO new Operation, NO new DEC, NO change to the Sink public API, NO
dimension-reduction (that is a deferred follow-up). Use ABSOLUTE paths.

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST (before editing ANY file)
═══════════════════════════════════════════════════════════════════════════
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-017-max-size-byte-budget-for-shrink-and-convert
Confirm `git branch --show-current` shows that branch before committing anything.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════
1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — §5/§6 the EXACT gate commands (clippy `cargo clippy --all-targets -- -D
   warnings`; the gate set also includes `cargo deny check licenses` via `just
   deny`), §11 conventions, §12 testing, §13 git/PR, §15 build-cycle rules.
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-017-max-size-byte-budget-for-shrink-and-convert.md
   — THE SPEC. Implement its "## Outputs", "## Failing Tests", and "## Notes for
   the Implementer" EXACTLY (the `search_threshold` refactor, `search_jpeg_under_size`,
   `auto_jpeg_under_size`, `jpeg_size_at`, `SearchConfig::for_size_budget`, the
   `AutoQuality` enum, `parse_size`, the `run_shrink`/`run_convert` wiring, and the
   warning wording are all spelled out there).
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-019-perceptual-auto-quality-ssimulacra2.md
   — the governing decision; the byte-budget search is its dual (same capped
   binary search + best-effort + decode-once; metric is encoded size, goal is
   highest quality ≤ budget). Already on main — do NOT create a new DEC.
4. The SHIPPED code you change/reuse (read the real signatures):
   src/quality/mod.rs   — `search_jpeg_quality` (REFACTOR its loop into the shared
                          `search_threshold` core; make this a thin wrapper —
                          public signature + behavior UNCHANGED), `SearchConfig`,
                          `QualityChoice`, `auto_jpeg_quality`, `score_jpeg_at` (the
                          JPEG encode `jpeg_size_at` mirrors — keep the cross-ref
                          comment), `MIN_/MAX_SEARCH_QUALITY`, `MAX_SEARCH_ITERS`,
                          `QualityError`. ADD `search_threshold` (private),
                          `search_jpeg_under_size`, `auto_jpeg_under_size`,
                          `jpeg_size_at` (private), `SearchConfig::for_size_budget`.
   src/cli/mod.rs       — the `auto: Option<SearchConfig>` param on `run_pixel_op`
                          (GENERALIZE to `auto: Option<AutoQuality>`),
                          `resolve_effective_quality` (the per-output resolver +
                          unmet warning — add the SizeBudget arm + its 2 warnings),
                          `run_shrink` + `shrink_auto_config`, `run_convert`,
                          `Commands::Shrink`/`Commands::Convert`, the dispatch arms,
                          the four other `run_pixel_op` callers, `parse_wxh` (the
                          model for `parse_size`), `CliError` (no new variant).
   src/sink/mod.rs      — `encode_to_bytes` (the JPEG encode `jpeg_size_at` must
                          match byte-for-byte; the cross-ref comment is already
                          there). READ-ONLY.
   tests/common/mod.rs  — `detailed_jpeg`/`detailed_png`/`solid_png` fixtures.
   tests/cli.rs         — integration conventions (drive the real binary; decode
                          with image::load_from_memory; assert sizes via
                          std::fs::metadata(..).len()).

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact — follow the spec's ## Outputs)
═══════════════════════════════════════════════════════════════════════════
A. src/quality/mod.rs:
   - Refactor the binary search into private
     `fn search_threshold<F: FnMut(u8) -> Result<f64, QualityError>>(probe, cfg,
     accept: impl Fn(f64) -> bool, prefer_lower: bool) -> Result<QualityChoice, QualityError>`.
     `search_jpeg_quality` becomes `search_threshold(score_at, cfg, |m| m >=
     cfg.target, true)` — its signature + the SPEC-016 tests MUST stay unchanged.
   - `pub fn search_jpeg_under_size<F: FnMut(u8) -> Result<u64, QualityError>>(size_at,
     budget_bytes: u64, cfg) -> Result<QualityChoice, QualityError>` =
     `search_threshold(|q| Ok(size_at(q)? as f64), cfg, |m| m <= budget_bytes as f64, false)`.
   - `pub fn auto_jpeg_under_size(reference: &::image::DynamicImage, budget_bytes: u64)
     -> Result<QualityChoice, QualityError>` = `search_jpeg_under_size(|q|
     jpeg_size_at(reference, q), budget_bytes, &SearchConfig::for_size_budget())`.
   - private `fn jpeg_size_at(reference, quality) -> Result<u64, QualityError>`:
     encode JPEG at `quality.clamp(1,100)` via the SAME `JpegEncoder::new_with_quality`
     path as `score_jpeg_at`; return byte length. NO decode, NO scoring.
   - `pub fn SearchConfig::for_size_budget() -> Self` (bounds 1/100/8, target NaN).
   - Generalize the `QualityChoice.score` doc (score OR byte size). Module deps
     UNCHANGED (no new imports beyond what's there).
B. src/cli/mod.rs:
   - `#[derive(Debug, Clone)] pub enum AutoQuality { Perceptual(SearchConfig),
     SizeBudget(u64) }`. Change `run_pixel_op`'s `auto: Option<SearchConfig>` →
     `auto: Option<AutoQuality>`.
   - `resolve_effective_quality`: match AutoQuality — Perceptual+JPEG →
     `auto_jpeg_quality` (unchanged); SizeBudget+JPEG → `auto_jpeg_under_size`;
     non-JPEG → None + warning; None → fixed quality. Add the 2 SizeBudget warnings
     (infeasible; lossless format) per the spec's exact wording; add a unit-tested
     `fn fmt_bytes(u64) -> String`.
   - `fn parse_size(&str) -> Result<u64, CliError>` (units B/KB/K/MB/M/KiB/MiB;
     decimal KB/MB; reject empty/non-numeric/zero/negative/overflow → Usage exit 2).
   - `Commands::Shrink`: add `#[arg(long, value_name="SIZE", conflicts_with_all =
     ["target","ssim"])] max_size: Option<String>`. `Commands::Convert`: add
     `#[arg(long, value_name="SIZE")] max_size: Option<String>`.
   - `run_shrink(inputs, max, target, ssim, max_size, global)` and
     `run_convert(inputs, format, max_size, global)` resolve `Option<AutoQuality>`;
     reject `auto.is_some() && global.quality.is_some()` → Usage exit 2. Update
     dispatch arms + all `run_pixel_op` callers for the type change (resize/
     thumbnail/auto-orient pass None).
C. docs/api-contract.md: add `--max-size` to the `shrink` and `convert` entries.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (make them pass) — per the spec's ## Failing Tests
═══════════════════════════════════════════════════════════════════════════
UNIT (src/quality/mod.rs): search_under_size_finds_highest_fitting,
search_under_size_unfittable_is_best_effort, search_under_size_propagates_error,
auto_under_size_is_monotone_in_budget, search_config_for_size_budget_bounds. The
EXISTING SPEC-016 quality::tests MUST stay green through the refactor.
UNIT (src/cli/mod.rs): parse_size_units, parse_size_rejects_junk (+ a fmt_bytes test).
INTEGRATION (tests/cli.rs): shrink_max_size_fits_budget,
shrink_max_size_larger_budget_not_smaller, shrink_max_size_conflicts_with_target_exits_2,
shrink_max_size_conflicts_with_ssim_exits_2, shrink_max_size_conflicts_with_quality_exits_2,
shrink_max_size_infeasible_warns, shrink_max_size_non_jpeg_warns,
convert_max_size_to_jpeg_fits, convert_max_size_conflicts_with_quality_exits_2.
The existing shrink/convert/resize/etc. + all SPEC-016 tests MUST stay green.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════
- Dimension-reduction fallback (deferred follow-up). - --max-size on AVIF/WebP
  (SPEC-018/019). - --json / --strict. - Any new dependency, Operation, CliError
  variant, Sink public-API change, or DEC. - Changing shrink/convert behavior when
  --max-size is absent. If you think you need one of these, STOP and add a question
  to guidance/questions.yaml.

═══════════════════════════════════════════════════════════════════════════
THE GATES (run from repo root; ALL must pass before the PR)
═══════════════════════════════════════════════════════════════════════════
  cargo build
  cargo test
  cargo clippy --all-targets -- -D warnings
  cargo fmt --check                              # run `cargo fmt` first
  just deny                                      # no new dep → must stay green

RUN GATES AND COMMIT INCREMENTALLY (search refactor + size search green → CLI
wiring green → integration green). A green committed checkpoint must survive an
interruption.

BEFORE YOU FINISH: re-read the spec's ## Failing Tests and CONFIRM EACH NAMED TEST
EXISTS and runs — list them and check each off. In particular confirm the EXISTING
SPEC-016 quality::tests still pass after the `search_threshold` refactor, and that
`search_under_size_finds_highest_fitting` asserts BOTH quality==50 AND ≤ max_iters
probe calls. Derive `Debug` on `AutoQuality`.

═══════════════════════════════════════════════════════════════════════════
WHEN DONE
═══════════════════════════════════════════════════════════════════════════
1. Fill ONLY the spec's `## Build Completion` (branch, PR, criteria, deviations,
   follow-ups, 3-question reflection). Edit nothing else in the spec body.
2. Append your build cost session to the spec front-matter `cost.sessions`:
     - cycle: build
       agent: claude-sonnet-4-6
       interface: claude-code
       tokens_total: null        # leave null — the ORCHESTRATOR fills the real
       estimated_usd: null       # number from your Agent result at ship
       duration_minutes: null
       recorded_at: 2026-06-16
       notes: "<one line>"
   Do NOT invent token numbers (the orchestrator records the real tokens_total /
   duration / usd from your Agent result during ship bookkeeping). If you run this
   cycle interactively rather than as a subagent, run `/cost` and write the real
   numbers yourself. (Per docs/cost-tracking.md / the cost-snippet template.)
3. Hand-edit the spec front-matter `task.cycle` from `design` to `verify`. Do NOT
   run `just advance-cycle` or `just archive-spec`.
4. Mark the build line `[x]` in the timeline (…/specs/SPEC-017-…-timeline.md) with
   ACCURATE wording — "PR #N opened" (real number). Never "merged"/"approved".
5. Commit on the branch (Conventional Commits, e.g.
   `feat(quality): --max-size byte budget for shrink and convert (SPEC-017)`),
   end EACH commit with: `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`.
6. Push and open a PR on `jysf/crustyimg` (§13 template): Summary; Spec metadata
   PROJ-001/STAGE-008/SPEC-017; Decisions referenced [DEC-019 (search policy this
   extends), DEC-016 (JPEG encode), DEC-015 (fan-out/format), DEC-004 (JPEG-only
   lossy core), DEC-018 (`just deny` green — no new dep), DEC-012/007 (clap/typed
   errors)]; Constraints checked (one-line evidence each); New decisions: "No new
   DEC — DEC-019 governs (byte-budget is its dual)". End with the Claude Code footer.
```
