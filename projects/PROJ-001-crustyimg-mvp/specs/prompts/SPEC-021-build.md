# SPEC-021 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect. The spec file is your only context. This
> prompt is deliberately prescriptive — follow it literally. Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-021 ("--max-size dimension-reduction fallback"). You
are NOT the architect; the spec is your source of truth. This completes `--max-size`
(SPEC-017): when lowering encoder quality can't hit the byte budget, DOWNSCALE the
output dimensions until it fits — and for LOSSLESS formats (no quality knob),
`--max-size` becomes a pure scale search. NO new dependency, NO new Operation, NO
change to the sink encode arms or to perceptual (`--target`/`--ssim`) / fixed-`-q`
behavior. Use ABSOLUTE paths.

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST
═══════════════════════════════════════════════════════════════════════════
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-021-max-size-dimension-reduction-fallback
Confirm `git branch --show-current` shows that branch before committing.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════
1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — §5/§6 gates, §11 conventions, §12 testing, §13 git/PR, §15 build-cycle rules.
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-021-max-size-dimension-reduction-fallback.md
   — THE SPEC. Implement its "## Outputs", "## Failing Tests", "## Notes" EXACTLY.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-023-max-size-dimension-reduction-fallback.md
   — the governing decision. Already on main — no new DEC.
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-019-*.md
   (the search core/policy) + DEC-016 (-q semantics).
5. The SHIPPED code you change (read the real signatures):
   src/quality/mod.rs — `auto_under_size`, `search_under_size`, `SearchConfig::
                        for_size_budget`, `encode_candidate_bytes`, `QualityChoice`,
                        `LossyFormat::supports_lossy_quality`, the `detailed_rgb` test fixture.
   src/cli/mod.rs     — `resolve_effective_quality` (+ its TWO call sites in
                        `run_pixel_op`: single + multi, where `sink.write(&out_img,
                        …, effective_quality, …)`).
   src/image/mod.rs   — `Image::from_parts(pixels, source_format, metadata)`,
                        `Image::pixels`, `Image::source_format`.

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact — follow the spec's ## Outputs)
═══════════════════════════════════════════════════════════════════════════
A. src/quality/mod.rs:
   - Add `pub struct SizeFit { pub quality: Option<u8>, pub image: Option<DynamicImage>,
     pub bytes: u64, pub scale_percent: u8, pub met_budget: bool }` (derive Debug).
   - Add `pub fn fit_under_size(reference: &DynamicImage, fmt: ImageFormat,
     budget_bytes: u64) -> Result<SizeFit, QualityError>`:
       * If `fmt.supports_lossy_quality()`: `let q = auto_under_size(reference, fmt,
         budget_bytes)?;` if `q.met_target` → return `SizeFit { quality: Some(q.quality),
         image: None, bytes: q.score as u64, scale_percent: 100, met_budget: true }`.
         Else fall to the scale search with `quality = Some(MIN_SEARCH_QUALITY)`.
       * Lossless (no lossy quality): scale search with `quality = None`.
       * Scale search: `let choice = search_under_size(|pct| size_at_scale(reference,
         fmt, pct, q_opt), budget_bytes, &SearchConfig::for_size_budget())?;` — here
         `choice.quality` is the chosen SCALE PERCENT (1..=100), `choice.score` the
         bytes. If `pct == 100` → `image: None`; else resize and `image: Some(..)`.
         Return `SizeFit { quality: q_opt, image, bytes: choice.score as u64,
         scale_percent: pct, met_budget: choice.met_target }`.
   - Add private `fn size_at_scale(reference: &DynamicImage, fmt: ImageFormat, pct: u8,
     quality: Option<u8>) -> Result<u64, QualityError>`: resize `reference` to `pct%`
     (`nw = (w as f64 * pct as f64 / 100.0).round().max(1.0) as u32`, same for h;
     `reference.resize_exact(nw, nh, ::image::imageops::FilterType::Lanczos3)`); then
     if `let Some(q) = quality` → `encode_candidate_bytes(&resized, fmt, q)?.len()`;
     else → `resized.write_to(&mut Cursor::new(Vec::new()), fmt)` and take the buffer
     length (map errors to `QualityError::Encode`). Add a private `fn resize_to_pct(..)`
     helper if it keeps `fit_under_size` clean (reused for the final resize).
   NOTE: `encode_candidate_bytes` is the EXISTING lossy probe (do not change it). The
   lossless probe MUST use `write_to(fmt)` — the same call the sink's default path makes.

B. src/cli/mod.rs:
   - Introduce `struct EncodePlan { quality: Option<u8>, image: Option<Image> }`.
   - Change `resolve_effective_quality` to return `Result<EncodePlan, CliError>`.
     * `Some(AutoQuality::SizeBudget(budget))` (for ANY format — drop the old
       lossy-only guard / "only JPEG" warning): `let fit = quality::fit_under_size(
       out_img.pixels(), fmt, *budget)?;` Build `image = fit.image.map(|d|
       Image::from_parts(d, out_img.source_format(), None))`. Warn unless `global.quiet`:
       if `!fit.met_budget` → "could not meet the {fmt_bytes(budget)} budget even at the
       smallest size (best effort {fmt_bytes(fit.bytes)})"; else if `image.is_some()` →
       "scaled to {w}x{h} to fit the {fmt_bytes(budget)} budget" (w/h from the resized
       image). Return `EncodePlan { quality: fit.quality, image }`.
     * Perceptual arm and the `None` arm: return `EncodePlan { quality: <as today>,
       image: None }`. (Perceptual unchanged; AVIF perceptual fallback unchanged.)
   - Update BOTH call sites in `run_pixel_op`:
       let plan = resolve_effective_quality(quality, &auto, fmt, &out_img, global, &label)?;
       let write_img = plan.image.as_ref().unwrap_or(&out_img);
       sink.write(write_img, &sink_input, overwrite, plan.quality, &mut std::io::stdout().lock())?;

C. docs/api-contract.md: `shrink`/`convert` `--max-size` now also DOWNSCALES when
   quality can't hit the budget, and works for LOSSLESS outputs (PNG, lossless WebP);
   a downscale warns (unless --quiet). Update the relevant sentences.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (per the spec's ## Failing Tests)
═══════════════════════════════════════════════════════════════════════════
src/quality (unit; reuse `detailed_rgb`):
  - fit_under_size_lossless_downscales (Png, budget below full-size → image Some,
    scale_percent<100, bytes ≤ budget, met_budget)
  - fit_under_size_met_at_full_no_resize (huge budget → image None, scale_percent 100)
  - fit_under_size_lossy_scales (Jpeg, budget below min-quality full size → image Some,
    quality Some, bytes ≤ budget)
  - fit_under_size_scale_is_monotone (Png small vs large budget → scale_percent monotone)
  - fit_under_size_unfittable_best_effort (budget 1 → !met_budget, image Some, no panic)
tests/cli.rs (integration; common::detailed_png):
  - convert_png_max_size_downscales (convert detailed_png(256,256) --format png
    --max-size 8KB → exit 0, Png, file len ≤ 8000, decoded dims < 256, stderr has "scal")
  - max_size_keeps_dims_when_it_fits (small solid png --format png --max-size 1MB →
    exit 0, decoded dims unchanged)
All 5 gates MUST stay green; perceptual / fixed-`-q` / non-budget behavior unchanged.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════
- The 2D refinement (re-optimizing quality at the chosen scale). - A crop mode or a
  min-dimension knob. - Reusing the fast_image_resize `Resize` operation (resize via
  `image` in `quality`, per DEC-023). - Changing the sink encode arms or perceptual
  behavior. - A new dep / Operation / DEC. If you think you need one, STOP and add a
  question to guidance/questions.yaml.

═══════════════════════════════════════════════════════════════════════════
THE GATES
═══════════════════════════════════════════════════════════════════════════
  cargo build
  cargo test
  cargo clippy --all-targets -- -D warnings
  cargo fmt --check
  just deny
Also confirm the feature builds still compile/clippy clean (the change is feature-
independent but touches shared code): `cargo clippy --all-targets --features avif -- -D
warnings` and `cargo clippy --all-targets --features webp-lossy -- -D warnings`.

Commit INCREMENTALLY (SizeFit + fit_under_size + unit tests → EncodePlan + the two
call sites + CLI warnings → integration tests → docs). A green committed checkpoint
must survive an interruption.

BEFORE YOU FINISH: re-read the spec's ## Failing Tests and confirm EACH named test
exists and runs — list them and check each off. Confirm a budget met at full size
does NOT resize (image None), and that a downscaled output's file length ≤ budget
(cross-sync: the sink writes the exact resized pixels the search chose).

═══════════════════════════════════════════════════════════════════════════
WHEN DONE
═══════════════════════════════════════════════════════════════════════════
1. Fill ONLY the spec's `## Build Completion`.
2. Append a build cost session to `cost.sessions` (cycle: build; agent:
   claude-sonnet-4-6; interface: claude-code; tokens_total/estimated_usd/
   duration_minutes: null; recorded_at: 2026-06-17; notes: "<one line>"). Do NOT
   invent token numbers (the orchestrator fills real values at ship; if interactive, `/cost`).
3. Hand-edit the spec front-matter `task.cycle` from `design` to `verify`.
4. Mark the timeline build line `[x]` — "PR #N opened" (real number), never "merged".
5. Commit (Conventional Commits, e.g. `feat(quality): --max-size dimension fallback
   (SPEC-021)`); end EACH commit with
   `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`.
6. Push + open a PR on `jysf/crustyimg` (§13 template): Summary; Spec metadata
   PROJ-001/STAGE-008/SPEC-021; Decisions referenced [DEC-023, DEC-019, DEC-016,
   DEC-002, DEC-015, DEC-012/007]; Constraints checked (one-line evidence each, incl.
   `single-image-library` ✅ — resize via `image`, no new lib; `every-public-fn-tested`
   ✅); New decisions: "No new DEC during build — DEC-023 already governs". End with
   the Claude Code footer.
```
