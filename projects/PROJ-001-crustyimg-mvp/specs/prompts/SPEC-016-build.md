# SPEC-016 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. This prompt is deliberately prescriptive — follow it literally.
> Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-016 ("perceptual auto-quality — shrink to a
visual target"). You are NOT the architect; the spec is your source of truth.
This adds a NEW `src/quality/` module (SSIMULACRA2 perceptual metric + a generic
quality binary-search) backed by a NEW default dependency `ssimulacra2` (v0.5.1,
BSD-2-Clause, pure-Rust), and wires `--target`/`--ssim` perceptual auto-quality
into the `shrink` command. It is JPEG-only and OPT-IN: `shrink` with no new flag
behaves exactly as today. NO new Operation, NO second image library, NO change to
the Sink public API, NO change to any command other than `shrink`. Use ABSOLUTE
paths for every file.

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST (before editing ANY file)
═══════════════════════════════════════════════════════════════════════════
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-016-perceptual-auto-quality-shrink-to-a-visual-target
Confirm `git branch --show-current` shows that branch — NOT `main`, NOT any
`chore/*` branch — before you commit anything. ALL edits happen on this branch.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════
1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — §5/§6 the EXACT gate commands (clippy is `cargo clippy --all-targets -- -D
   warnings`; the gate set now ALSO includes `cargo deny check licenses` via
   `just deny`), §11 conventions (typed errors; NO unwrap/expect/panic on
   recoverable paths; ONE image library; a new module must NOT depend on
   clap/cli/sink/fs/terminals), §12 testing (native in-memory fixtures), §13
   git/PR, §15 build-cycle rules (spec edits limited to ## Build Completion;
   append a build cost session; DEC-019 already exists — do NOT create a new DEC).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-016-perceptual-auto-quality-shrink-to-a-visual-target.md
   — THE SPEC. Implement its "## Outputs", "## Failing Tests", and "## Notes for
   the Implementer" EXACTLY. The module API (`score`, `search_jpeg_quality`,
   `auto_jpeg_quality`, `SearchConfig`, `QualityChoice`, `QualityError`, the
   constants), the exact `ssimulacra2` calls, the binary-search sketch, the
   `run_pixel_op` `auto` param, the `run_shrink` flag handling, and the fixtures
   are all spelled out there.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-019-perceptual-auto-quality-ssimulacra2.md
   — the governing decision: adopt `ssimulacra2` (BSD-2, default, permissive);
   SSIMULACRA2 metric (higher=better, ~100 identical); presets visually-lossless
   90 / high 70 / medium 50; binary-search lowest JPEG quality with score ≥
   target, cap 8 iters, memoize, best-effort max-quality on unreachable target,
   JPEG-only v1, target ignored for non-JPEG. Already authored — do NOT create a
   new DEC.
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-016-encode-quality-policy.md
   — the JPEG `JpegEncoder::new_with_quality` path your candidate-encode mirrors.
5. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-018-permissive-license-policy-cargo-deny.md
   — the license gate; the dep MUST stay permissive; run `just deny` after adding.
6. The SHIPPED code you change/reuse (read the real signatures):
   src/sink/mod.rs       — `encode_to_bytes(img, format, quality: Option<u8>)` and
                           its JPEG `new_with_quality` branch (the candidate encode
                           mirrors it); `Sink::write` (final write — UNCHANGED).
   src/cli/mod.rs        — `run_pixel_op` (you append a trailing `auto:
                           Option<quality::SearchConfig>` param exactly like
                           SPEC-014 appended `forced_format`; resolve effective
                           quality per-image in BOTH the single AND the multi
                           `sink.write` sites), the four OTHER callers
                           (`run_resize`/`run_thumbnail`/`run_convert`/`run_auto_orient`
                           — each gets one new `None` arg), `run_shrink` +
                           `shrink_params` + `DEFAULT_SHRINK_MAX`/`DEFAULT_SHRINK_QUALITY`,
                           `Commands::Shrink`, the dispatch arm, `CliError` +
                           `code()` + `exit_code_mapping_is_total`. `run_apply`
                           calls `sink.write` directly (NOT run_pixel_op) — leave it.
   src/image/mod.rs      — `Image::pixels() -> &DynamicImage` (the resized pixels
                           you score). READ-ONLY — do NOT edit this module.
   src/lib.rs            — the `pub mod ...;` list (add `pub mod quality;`).
   Cargo.toml            — `[dependencies]` + `[features]` (add the dep, no gate).
   deny.toml             — the cargo-deny allow-list (DEC-018).
   tests/common/mod.rs   — fixture conventions (`gradient_jpeg`, `solid_png`); you
                           ADD `detailed_jpeg`/`detailed_png` (high-frequency).
   tests/cli.rs          — integration conventions (drive the real binary; decode
                           with image::load_from_memory; tempfile; exit codes).

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact — follow the spec's ## Outputs)
═══════════════════════════════════════════════════════════════════════════
A. Cargo.toml — add `ssimulacra2 = "=0.5.1"` to `[dependencies]` (DEFAULT, NOT
   behind a feature; pure-Rust, BSD-2). Then `cargo build` to fetch+compile, then
   `just deny` (see the LICENSE GATE box below — this is mandatory).
B. src/lib.rs — add `pub mod quality;`.
C. src/quality/mod.rs — NEW module. Public API per the spec's ## Outputs:
   - `QualityError` (thiserror enum: `Score(String)`, `Convert(String)`,
     `Encode(String)`), `#[derive(Debug)]`.
   - `score(reference: &::image::DynamicImage, candidate: &::image::DynamicImage)
     -> Result<f64, QualityError>` — private `to_ss_rgb` helper (`.to_rgb8()` →
     `Vec<[f32;3]>` each channel `/255.0` → `ssimulacra2::Rgb::new(data, w, h,
     ssimulacra2::TransferCharacteristic::SRGB, ssimulacra2::ColorPrimaries::BT709)`),
     then `ssimulacra2::compute_frame_ssimulacra2(ref_rgb, cand_rgb)`; map the two
     error types into `QualityError` via `.to_string()`.
   - `SearchConfig { target: f64, min_quality: u8, max_quality: u8, max_iters: u8 }`
     + `SearchConfig::for_target(target) -> Self` using the consts.
   - `QualityChoice { quality: u8, score: f64, iterations: u8, met_target: bool }`
     (`#[derive(Debug, Clone, Copy)]`).
   - `search_jpeg_quality<F: FnMut(u8) -> Result<f64, QualityError>>(score_at: F,
     cfg: &SearchConfig) -> Result<QualityChoice, QualityError>` — the GENERIC
     binary search (lowest quality with score ≥ target; memoize; ≤ max_iters
     distinct scorer calls; best-effort max_quality + met_target=false if none
     meets; propagate a scorer Err). Use the spec's sketch; mind the u8 underflow
     guards at min_quality/max_quality.
   - `auto_jpeg_quality(reference, cfg) -> Result<QualityChoice, QualityError>` —
     `search_jpeg_quality(|q| { encode reference→JPEG bytes via
     ::image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, q.clamp(1,100))
     + reference.write_with_encoder(enc); ::image::load_from_memory(bytes); then
     score(reference, &decoded) }, cfg)`. Encode/decode failures →
     QualityError::Encode.
   - `pub const MIN_SEARCH_QUALITY: u8 = 1; MAX_SEARCH_QUALITY: u8 = 100;
     MAX_SEARCH_ITERS: u8 = 8;`.
   - Module depends ONLY on `::image`, `ssimulacra2`, `thiserror`, `std`. NO clap,
     cli, sink, files, terminals.
D. src/cli/mod.rs:
   - `#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)] pub enum
     QualityTarget { VisuallyLossless, High, Medium }` (clap → `visually-lossless`,
     `high`, `medium`) + `fn target_score(self) -> f64` (90/70/50).
   - `Commands::Shrink` gains `#[arg(long, value_enum)] target: Option<QualityTarget>`
     and `#[arg(long, conflicts_with = "target")] ssim: Option<f64>`.
   - Dispatch: `Commands::Shrink { inputs, max, target, ssim } => run_shrink(inputs,
     *max, *target, *ssim, &cli.global)`.
   - Rewrite `run_shrink(inputs, max, target, ssim, global)` per the spec: resolve
     `auto: Option<quality::SearchConfig>`; validate `--ssim` in `0.0..=100.0` (else
     `CliError::Usage` exit 2); reject `auto.is_some() && global.quality.is_some()`
     (`CliError::Usage` exit 2); build the resize op as today; `fixed_quality = if
     auto.is_some() { None } else { Some(global.quality.unwrap_or(DEFAULT_SHRINK_QUALITY)) }`;
     call `run_pixel_op(pipeline, inputs, global, fixed_quality, None, auto)`.
   - `run_pixel_op`: append `auto: Option<quality::SearchConfig>`; in BOTH the
     single and multi arms compute `effective_quality` (Some(cfg)&&fmt==Jpeg →
     `Some(quality::auto_jpeg_quality(out_img.pixels(), cfg)?.quality)`; Some(_) →
     None; None → `quality`) and pass it to `sink.write`. The four other callers
     pass `None`.
   - `CliError`: add `#[error(transparent)] Quality(#[from] quality::QualityError)`
     → exit code 1 in `code()`; extend `exit_code_mapping_is_total`.
E. tests/common/mod.rs — add `detailed_jpeg(w,h)` and `detailed_png(w,h)` with the
   EXACT deterministic STRUCTURED pattern from the spec (smooth gradient + 8px
   checker texture). NOT a flat gradient/solid (compresses near-losslessly → search
   degenerates) and NOT pure noise (too adversarial → monotonicity test collapses).
   Copy the per-channel formula from the spec's ## Outputs verbatim.
F. docs/api-contract.md — extend the `shrink` entry with the `--target`/`--ssim`
   wording quoted in the spec's Notes.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (make them pass) — per the spec's ## Failing Tests
═══════════════════════════════════════════════════════════════════════════
UNIT (src/quality/mod.rs `#[cfg(test)] mod tests`; local `detailed_rgb(w,h) ->
DynamicImage` helper; fixtures ≥ 64×64):
  - score_identical_is_high                  (score(&img,&img) > 90.0)
  - score_degraded_is_lower                  (JPEG q8 round-trip scores < identity AND < 90.0)
  - search_finds_lowest_meeting_target       (|q| Ok(q as f64), target 50 → quality 50, met_target true, ≤ max_iters calls)
  - search_unreachable_target_is_best_effort (|_| Ok(10.0), target 90 → quality 100, met_target false)
  - search_propagates_scorer_error           (scorer Err → search returns that Err)
  - auto_jpeg_quality_is_monotone_in_target  (target 50 quality ≤ target 90 quality on a detailed image)
  - search_config_defaults_match_dec019      (for_target(90.0) → min 1, max 100, iters 8, target 90.0)
INTEGRATION (tests/cli.rs — use common::detailed_jpeg/detailed_png; ≥ 128×128;
drive the real binary; decode with image::load_from_memory):
  - shrink_target_visually_lossless_produces_valid_jpeg  (detailed_jpeg(160,160) → out is JPEG, dims 160×160)
  - shrink_lower_ssim_target_is_smaller_file             (--ssim 50 file bytes < --ssim 95 file bytes)
  - shrink_target_and_ssim_conflict_exits_2
  - shrink_ssim_out_of_range_exits_2                     (--ssim 150 → exit 2)
  - shrink_quality_and_target_conflict_exits_2           (-q 80 --target high → exit 2)
  - shrink_target_non_jpeg_is_ignored                    (detailed_png → out is PNG, exit 0)
  - shrink_target_multi_input_fan_out                    (two detailed jpgs --target high --out-dir D → both JPEG, exit 0)

The existing shrink/resize/thumbnail/convert/auto-orient + all unit/sink tests
MUST stay green (run the FULL suite). `shrink` with no `--target`/`--ssim` is
unchanged — do NOT alter its existing tests.

═══════════════════════════════════════════════════════════════════════════
LICENSE GATE (mandatory — DEC-018)
═══════════════════════════════════════════════════════════════════════════
After adding `ssimulacra2` and `cargo build`, run:
  just deny        # == cargo deny check licenses (the CI `licenses` job)
- If it PASSES: good, continue.
- If it FAILS ONLY because a transitive dep carries a PERMISSIVE license not yet in
  deny.toml's `allow` list (most likely candidate: `ISC`; also possibly `MIT-0`):
  add that EXACT SPDX id to the `allow` array in deny.toml, re-run `just deny`, and
  NOTE the addition in the PR body. DEC-018 explicitly anticipates this one-line fix.
- If it FAILS on ANY GPL / AGPL / LGPL (or any copyleft) license: STOP. Do NOT add
  an exception. Write a question to guidance/questions.yaml and halt — that would
  contradict the design's license analysis and needs the architect.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════
- `--max-size` byte budget (SPEC-017). - AVIF/WebP output or auto-quality on any
  non-JPEG encoder (SPEC-018/019). - Auto-quality on `convert` or any command but
  `shrink`. - A second image/pixel library (ssimulacra2 is a METRIC, not a pixel
  lib — fine). - rayon/parallel search or batch (STAGE-005). - Caching the winning
  candidate bytes. - A `--strict` or `--json` mode. - Any new Operation, any
  src/image or src/sink public-API change, any change to a non-shrink command's
  behavior, or any new DEC. If you think one is needed, STOP and add a question to
  guidance/questions.yaml.

═══════════════════════════════════════════════════════════════════════════
THE GATES (run from repo root; ALL must pass before the PR)
═══════════════════════════════════════════════════════════════════════════
  cargo build
  cargo test
  cargo clippy --all-targets -- -D warnings     # --all-targets is the CI gate
  cargo fmt --check                              # run `cargo fmt` first to fix
  just deny                                      # cargo deny check licenses (DEC-018)

RUN GATES AND COMMIT INCREMENTALLY — commit once `src/quality` + the dep compile
and clippy/fmt + `just deny` are clean; again once the unit tests pass; again once
the CLI wiring + integration tests pass. Do NOT leave all work uncommitted to the
end; a green committed checkpoint must survive an interruption. (Hard lesson from
SPEC-011.)

BEFORE YOU FINISH: re-read the spec's ## Failing Tests and CONFIRM EACH NAMED TEST
EXISTS in the code and runs — list them and check each off. A passing test COUNT
does not prove the prescribed tests were written. In particular confirm
`search_finds_lowest_meeting_target` asserts BOTH `quality == 50` AND the scorer
was called ≤ `max_iters` times, and `shrink_lower_ssim_target_is_smaller_file`
asserts the `--ssim 50` file is strictly SMALLER than the `--ssim 95` file.

Also: derive `Debug` on every new public type (`QualityError`, `SearchConfig`,
`QualityChoice`, `QualityTarget`); do not `{:?}`-format a non-Debug type.

═══════════════════════════════════════════════════════════════════════════
WHEN DONE
═══════════════════════════════════════════════════════════════════════════
1. Fill ONLY the spec's `## Build Completion` (branch, PR, criteria, deviations
   incl. any deny.toml license addition, follow-ups, 3-question reflection). Edit
   nothing else in the spec body.
2. Append a build cost session to the spec front-matter `cost.sessions` (cycle:
   build, agent: claude-sonnet-4-6, interface: claude-code, null numerics,
   recorded_at: 2026-06-16, a one-line note).
3. Hand-edit the spec front-matter `task.cycle` from `design` to `verify`. DO NOT
   run `just advance-cycle` or `just archive-spec`.
4. Mark the build line `[x]` in the timeline
   (projects/PROJ-001-crustyimg-mvp/specs/SPEC-016-...-timeline.md) with ACCURATE
   wording — "PR #N opened" (real number). Never "merged"/"approved".
5. Commit on the branch (Conventional Commits, e.g.
   `feat(quality): perceptual auto-quality shrink via SSIMULACRA2 (SPEC-016)`),
   end EACH commit with: `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`.
6. Push and open a PR on `jysf/crustyimg` (§13 template): Summary; Spec metadata
   PROJ-001/STAGE-008/SPEC-016; Decisions referenced [DEC-019 (ssimulacra2 +
   metric/threshold/search policy), DEC-016 (JPEG quality encode), DEC-018
   (permissive license gate — `just deny` green), DEC-015 (fan-out / format
   precedence), DEC-004 (pure-Rust default), DEC-002 (decode-once), DEC-012/007
   (clap/typed errors)]; Constraints checked (one-line evidence each, incl.
   `no-agpl-default-deps` ✅ — `just deny` green with ssimulacra2 BSD-2); New
   decisions: "No new DEC during build — DEC-019 already governs". End with the
   Claude Code generated-with footer.
```
