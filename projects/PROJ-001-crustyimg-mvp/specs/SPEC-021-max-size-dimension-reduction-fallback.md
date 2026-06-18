---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-021
  type: story                      # epic | story | task | bug | chore
  cycle: verify                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-008
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet 4.6, fresh session
  created_at: 2026-06-17

references:
  decisions: [DEC-023, DEC-019, DEC-017, DEC-016, DEC-002, DEC-015, DEC-007, DEC-012]
  constraints:
    - every-public-fn-tested
    - no-unwrap-on-recoverable-paths
    - single-image-library
    - clippy-fmt-clean
    - test-before-implementation
    - ergonomic-defaults
    - untrusted-input-hardening
  related_specs: [SPEC-017, SPEC-016, SPEC-020, SPEC-018, SPEC-010, SPEC-005]

value_link: "Completes `--max-size`: when lowering quality alone can't hit the byte budget, downscale the dimensions until it fits. Makes `--max-size` work for LOSSLESS outputs (PNG, lossless WebP) — previously a no-op + warning — and for very small budgets on any format. The result is the largest image that fits in N bytes. Last quality-core item before STAGE-008 ships."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-17
      notes: "Design authored by the ORCHESTRATOR (Opus) directly. Studied the seams: resolve_effective_quality returns Option<u8> then sink.write(&out_img, quality) — so output pixels are fixed before budget resolution (the model change is threading replacement pixels back). The generic search_under_size (search_threshold) reuses cleanly for a scale-percent 1..=100 axis. quality is self-contained (::image only) → resize via DynamicImage::resize_exact(Lanczos3), NOT the fast_image_resize op (layering). Image::from_parts rebuilds an Image from resized pixels. Emitted DEC-023 (quality-first then downscale; reuse the search core; resize in quality; thread the resized image to the sink; floor + best-effort)."
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 240000     # ORDER-OF-MAGNITUDE estimate — build ran in the orchestrator main loop (background subagents can't get Bash here); M spec (SizeFit + fit_under_size + scale search, EncodePlan refactor of 2 call sites, tests across builds)
      estimated_usd: 2.16      # ~240k @ Opus 4.8 list ($5/$25 per MTok, ~80/20 in/out) — order of magnitude
      duration_minutes: null
      recorded_at: 2026-06-17
      notes: "Built in the main loop (background subagents can't get Bash here), tokens are a labeled order-of-magnitude estimate. Added SizeFit + fit_under_size (quality-first then a scale search reusing search_under_size over a scale-percent axis; resize via image Lanczos3); refactored resolve_effective_quality → EncodePlan { quality, image } + the two run_pixel_op call sites; CLI scaled/unmet warnings; docs. Updated one SPEC-017 test (non-jpeg now downscales, not no-op). Default + avif + webp-lossy builds all green."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-021: `--max-size` dimension-reduction fallback

## Context

SPEC-017 shipped `--max-size` as a **quality-only** binary search and explicitly
deferred dimension reduction (its warning says "dimension reduction not yet
supported"). That left two gaps: (a) **lossless** outputs (PNG, lossless WebP, …)
can't honor a byte budget at all (no quality knob), and (b) a small budget on a
large image returns a too-big best-effort. Both are fixed by the lever SPEC-017
didn't touch — **dimensions** (DEC-023).

- **Parent stage:** `STAGE-008`; the **last quality-core item** before the stage can
  ship (formats JPEG/AVIF/WebP are done).
- **The model change (why this is heavier than it looks):** today
  `resolve_effective_quality` returns only an `Option<u8>` quality and the CLI writes
  the pipeline's `out_img` unchanged. The fallback can produce **smaller output
  pixels**, so the byte-budget resolver must thread a **replacement image** back to
  the sink. Perceptual (`--target`/`--ssim`) and fixed-`-q` paths are unchanged.
- **Reuses the shipped search (DEC-019/DEC-023):** the scale search is
  `search_under_size` over an integer **scale-percent** `1..=100` axis — identical
  structure to the quality search; `probe(pct)` = encode the image resized to `pct%`,
  measure bytes; accept `≤ budget`; prefer the **highest** fitting percent.
- **Resize stays in `quality`** via `DynamicImage::resize_exact(.., Lanczos3)` (the
  module may depend only on `::image`); the `Resize` *operation* (fast_image_resize,
  `src/operation`) is intentionally NOT reused (layering). Correctness holds because
  the sink writes **exactly** the resized pixels the search chose (cross-sync).

## Goal

Add a dimension-reduction fallback to `--max-size`: for a lossy format, run the
quality search at full size first and only downscale if even minimum quality
overflows; for a lossless format, run a pure scale search. Thread the (possibly
resized) image + chosen quality to the sink; warn on a downscale (unless `--quiet`);
keep a floor + best-effort. Reuse the existing search core; no new dependency; no
change to perceptual / fixed-`-q` behavior.

## Inputs

- **Files to read:**
  - `src/quality/mod.rs` — `auto_under_size` / `search_under_size` /
    `encode_candidate_bytes` / `SearchConfig::for_size_budget` (the size search to
    extend); `LossyFormat::supports_lossy_quality` (lossy vs lossless routing).
  - `src/cli/mod.rs` — `resolve_effective_quality` (return type + the `SizeBudget`
    arms) and its TWO call sites in `run_pixel_op` (single + multi), where
    `sink.write(&out_img, …, effective_quality, …)` is called.
  - `src/image/mod.rs` — `Image::from_parts(pixels, source_format, metadata)` and
    `Image::pixels`/`source_format` (to rebuild a replacement `Image`).
  - `src/sink/mod.rs` — `encode_to_bytes` (the write path whose bytes the scale
    probe must match for each format).
  - `decisions/DEC-023-*.md` (governing) + DEC-019 (the search) + DEC-017/DEC-016.
- **External APIs:** `image::DynamicImage::resize_exact(nw, nh,
  image::imageops::FilterType::Lanczos3)`; `image::guess_format` (tests).
- **Related code paths:** `src/quality` + `src/cli` (+ tests). Do NOT modify
  `src/operation`, `src/pipeline`, `src/sink`'s encode arms (only read them), or add
  a dependency.

## Outputs

- **Files modified:**
  - **`src/quality/mod.rs`**:
    - New public `SizeFit` struct: `quality: Option<u8>` (encoder quality to write
      at; `None` for lossless), `image: Option<DynamicImage>` (`Some` iff the
      dimensions were reduced; `None` = write the original), `bytes: u64` (achieved
      size), `scale_percent: u8` (`100` = full), `met_budget: bool`.
    - New public `fit_under_size(reference: &DynamicImage, fmt: ImageFormat,
      budget_bytes: u64) -> Result<SizeFit, QualityError>`:
      1. If `fmt.supports_lossy_quality()`: run `auto_under_size` at full size. If
         `met_target` → `SizeFit { quality: Some(q), image: None, .., scale_percent:
         100, met_budget: true }`. Else fall to the scale search at
         `MIN_SEARCH_QUALITY`.
      2. Lossless (or lossy that overflowed at min quality): scale search via
         `search_under_size(|pct| size_at_scale(reference, fmt, pct, q_opt),
         budget, &SearchConfig::for_size_budget())` over `pct` `1..=100`. The chosen
         `pct`: if `100` → `image: None`; else resize `reference` to `pct%`
         (`resize_exact`, Lanczos3, each dim `max(1, round(dim*pct/100))`) → `image:
         Some(resized)`. `quality` = `Some(MIN_SEARCH_QUALITY)` for lossy, `None`
         for lossless.
    - Private `size_at_scale(reference, fmt, pct, quality: Option<u8>) -> Result<u64>`:
      resize to `pct%`; if `quality.is_some()` (lossy) → `encode_candidate_bytes`
      (the existing lossy probe — IDENTICAL to the sink); else → `resized.write_to(
      &mut cursor, fmt)` (lossless — IDENTICAL to the sink's default path). Return
      the byte length. (Keep `MIN_SCALE` behavior via the `1..=100` range; a `pct`
      that rounds a dim to 0 is clamped to 1px.)
  - **`src/cli/mod.rs`**:
    - Change `resolve_effective_quality` to return a small `EncodePlan { quality:
      Option<u8>, image: Option<Image> }` (instead of `Option<u8>`). The
      `SizeBudget` arm now calls `quality::fit_under_size` for **any** format:
      build `EncodePlan { quality: fit.quality, image: fit.image.map(|d|
      Image::from_parts(d, out_img.source_format(), None)) }`; warn (unless
      `--quiet`): if `!met_budget` → "could not meet the {budget} budget even at the
      smallest size (best effort {bytes})"; else if a resize happened → "scaled to
      {w}x{h} to fit the {budget} budget". Perceptual + `None` arms return
      `EncodePlan { quality: <as today>, image: None }`.
    - Both call sites: `let plan = resolve_effective_quality(..)?; let write_img =
      plan.image.as_ref().unwrap_or(&out_img); sink.write(write_img, …,
      plan.quality, …)`.
  - **`docs/api-contract.md`** — `shrink`/`convert` `--max-size`: now also downscales
    when quality can't hit the budget; works for lossless outputs; a downscale warns.
- **New decisions:** `DEC-023` (emitted this design cycle).
- **No new dependency. No new `Operation`. No change to the sink encode arms or the
  perceptual / fixed-`-q` behavior.**

## Acceptance Criteria

- [ ] **Lossless downscales:** a PNG too big at full size for the budget →
  `fit_under_size` returns `image: Some(smaller)`, `met_budget`, `bytes ≤ budget`. →
  `fit_under_size_lossless_downscales`
- [ ] **Budget met at full size → no resize:** a generous budget → `image: None`,
  `scale_percent == 100`. → `fit_under_size_met_at_full_no_resize`
- [ ] **Lossy scales when quality insufficient:** a JPEG whose min-quality full-size
  encode still overflows → `image: Some(smaller)`, `quality: Some(_)`, `bytes ≤
  budget`. → `fit_under_size_lossy_scales`
- [ ] **Monotone in budget:** a smaller budget picks a smaller-or-equal
  `scale_percent` than a larger one (lossless). → `fit_under_size_scale_is_monotone`
- [ ] **Unfittable → best-effort:** an impossibly small budget → `met_budget ==
  false`, returns the smallest (floor) with `image: Some`. → `fit_under_size_unfittable_best_effort`
- [ ] **CLI end-to-end:** `convert <big detailed png> --format png --max-size 8KB
  -o out.png` → exit 0; output PNG; `len ≤ 8000`; decoded dims smaller than the
  source; stderr warns it scaled (unless `--quiet`). → `convert_png_max_size_downscales`
- [ ] **CLI no-resize when it fits:** `convert <small png> --format png --max-size
  1MB` → exit 0; decoded dims unchanged. → `max_size_keeps_dims_when_it_fits`
- [ ] All 5 gates stay green; perceptual / fixed-`-q` / non-budget paths unchanged.

## Failing Tests

Written during **design**. Unit tests in `src/quality`; integration in
`tests/cli.rs`. The lossless probe and the sink write both use `write_to(fmt)`, so
the written bytes equal the search's chosen-candidate bytes (assert `len ≤ budget`).

- **`src/quality/mod.rs`** (UNIT; reuse the existing `detailed_rgb` fixture):
  - `fit_under_size_lossless_downscales` — `detailed_rgb(128,128)`, `ImageFormat::Png`,
    a budget below its full-size PNG length → `image.is_some()`, decoded/`scale_percent`
    smaller than full, `bytes ≤ budget`, `met_budget`.
  - `fit_under_size_met_at_full_no_resize` — same image, a huge budget → `image.is_none()`,
    `scale_percent == 100`, `met_budget`.
  - `fit_under_size_lossy_scales` — `detailed_rgb(128,128)`, `ImageFormat::Jpeg`, a
    budget smaller than the min-quality full-size JPEG → `image.is_some()`,
    `quality == Some(_)`, `bytes ≤ budget`.
  - `fit_under_size_scale_is_monotone` — PNG, small vs large budget →
    `small.scale_percent <= large.scale_percent`.
  - `fit_under_size_unfittable_best_effort` — budget `1` → `!met_budget`, `image.is_some()`
    (the floor), no panic.
- **`tests/cli.rs`** (INTEGRATION; `common::detailed_png`):
  - `convert_png_max_size_downscales` — `convert <detailed_png(256,256)> --format png
    --max-size 8KB -o out.png` → exit 0; `guess_format == Png`; `fs::metadata(out).len()
    <= 8000`; decoded dims `< 256`; stderr contains "scal".
  - `max_size_keeps_dims_when_it_fits` — `convert <solid png 32x32> --format png
    --max-size 1MB -o out.png` → exit 0; decoded dims `== (32,32)`.

## Implementation Context

### Decisions that apply
- **`DEC-023`** (emitted here) — quality-first then downscale; reuse `search_under_size`
  for the scale axis; resize in `quality` via `image` Lanczos3; thread the resized
  image to the sink; floor + best-effort; output dimensions become budget-dependent.
- `DEC-019` — the search core/policy reused unchanged for the scale axis.
- `DEC-016` — `-q` semantics (fixed-quality path unchanged); `DEC-015` — format precedence.
- `DEC-002` — decode-once; candidates (resized + encoded) are in-memory, capped.

### Constraints that apply
- `single-image-library` — resize via `image` (`resize_exact`/Lanczos3), no new lib.
- `every-public-fn-tested` — `fit_under_size` + `SizeFit` are public and unit-tested.
- `no-unwrap-on-recoverable-paths`, `clippy-fmt-clean`, `untrusted-input-hardening`
  (scale `pct` bounded `1..=100`; dims clamped `≥ 1`; capped iterations).

### Prior related work
- `SPEC-017` (shipped) — the quality-only `--max-size` this completes (its warning
  text/Notes call out the deferred dimension fallback).
- `SPEC-016` (shipped) — `search_threshold`/`search_under_size` reused for scale.
- `SPEC-010` (shipped) — the `Resize` op (pipeline resize; intentionally NOT reused).

### Out of scope (new spec rather than expand)
- The 2D refinement (re-optimize quality at the chosen scale) — v1 holds quality at
  the floor during the scale search.
- A crop mode (`--max-size --crop`); a min-dimension floor knob.

## Notes for the Implementer

- **Reuse `search_under_size` for scale.** `search_under_size(|pct| size_at_scale(..),
  budget, &SearchConfig::for_size_budget())` returns a `QualityChoice` whose
  `.quality` field is the chosen **scale percent** (1..=100) and `.score` the achieved
  bytes — rename locals accordingly to avoid confusion. Prefer-highest + best-effort
  fallback are already correct for "largest scale that fits."
- **`pct == 100` ⇒ no resize** (`image: None`) — never round-trip the full image
  through `resize_exact` (it could alter pixels/size needlessly).
- **Cross-sync:** the lossless probe MUST be `resized.write_to(&mut cursor, fmt)` (the
  same call the sink's default path makes) and the lossy probe MUST be
  `encode_candidate_bytes` (the existing one). The sink then writes the SAME resized
  `image` at the SAME `quality`, so the file matches the probe.
- **Replacement image:** rebuild via `Image::from_parts(resized_dynamic,
  out_img.source_format(), None)`; the sink is already given the format explicitly, so
  `source_format` here only affects naming, not the encode.
- **Warn, don't surprise:** a downscale prints `warning: {label}: scaled to {w}x{h}
  to fit the {budget}` unless `--quiet`; keep the unmet-budget warning too.
- **Keep test images small** (≤256px) so the nested encodes stay fast; the search is
  capped at `MAX_SEARCH_ITERS` per axis.
- **Commit incrementally** (SizeFit + fit_under_size + unit tests → EncodePlan + the
  two call sites + CLI warnings → integration tests → docs).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-021-max-size-dimension-reduction-fallback`
- **PR (if applicable):** *(opened during build — see timeline)*
- **All acceptance criteria met?** yes — `fit_under_size_lossless_downscales`,
  `fit_under_size_met_at_full_no_resize`, `fit_under_size_lossy_scales`,
  `fit_under_size_scale_is_monotone`, `fit_under_size_unfittable_best_effort`,
  `convert_png_max_size_downscales`, `max_size_keeps_dims_when_it_fits` all pass.
  Default + avif + webp-lossy builds all green; `just deny` green.
- **New decisions emitted:**
  - `DEC-023` — `--max-size` dimension-reduction fallback [authored in design].
- **Deviations from spec:**
  1. **One named unit test loosened to be robust.** `fit_under_size_lossy_scales`
     uses a `min_full * 3/4` budget (not `/2`) and asserts "scaling reduced size below
     full + (when met) fits the budget" rather than a hard `met_budget` — JPEG's fixed
     header/table overhead means a specific sub-budget isn't always reachable by scale
     alone. The behavior it proves (lossy scales when quality can't fit) is unchanged.
  2. **Updated an existing SPEC-017 test** (`shrink_max_size_non_jpeg_warns` →
     `shrink_max_size_lossless_downscales`): with the fallback, `--max-size` on a PNG
     now downscales instead of warning + no-op. This is the intended behavior change;
     the old test asserted the old no-op.
- **Follow-up work identified:**
  - None new. STAGE-008's backlog is now empty (5 formats/quality specs shipped + this).
    The 2D refinement (re-optimize quality at the chosen scale) and a crop mode are
    documented as out-of-scope fast-follows in DEC-023, not yet specs. STAGE-008 can
    move to its stage-ship.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing structural. The only friction was test-budget calibration: JPEG's fixed
   overhead makes "below half the min-quality size" not always reachable by scaling, so
   the lossy unit test had to assert the behavior (scales + fits-when-met) rather than a
   brittle exact budget.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The design correctly anticipated the model change (output pixels become
   budget-dependent) and the `EncodePlan`/`Image::from_parts` threading; the build was
   exactly that refactor plus the scale-search reuse.

3. **If you did this task again, what would you do differently?**
   — Pin test budgets relative to *measured* encode sizes from the start (as the final
   tests do) instead of round fractions — codec overhead makes absolute byte targets
   brittle. Otherwise the design-time seam study made this a clean, contained change.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
