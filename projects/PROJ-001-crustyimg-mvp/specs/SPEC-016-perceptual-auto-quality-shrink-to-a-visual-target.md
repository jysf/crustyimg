---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-016
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
  created_at: 2026-06-16

references:
  decisions: [DEC-019, DEC-016, DEC-015, DEC-018, DEC-004, DEC-002, DEC-007, DEC-012]
  constraints:
    - ergonomic-defaults
    - single-image-library
    - pure-rust-codecs-default
    - no-agpl-default-deps
    - no-new-top-level-deps-without-decision
    - decode-once-no-per-op-disk
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - test-before-implementation
    - untrusted-input-hardening
  related_specs: [SPEC-013, SPEC-014, SPEC-005, SPEC-011]

value_link: "Delivers STAGE-008's flagship differentiator — `shrink --target visually-lossless`/`--ssim N` lets a user ask for a *look* (a perceptual quality) instead of guessing a JPEG quality number; the SSIMULACRA2 metric it adds is the foundation for the byte-budget (SPEC-017), AVIF/WebP quality (SPEC-018/019), and the future benchmark + diff work."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-16
      notes: "Design authored by the ORCHESTRATOR (Opus) directly (established pattern from SPEC-013/014/015 — full repo context already loaded; de-risk the upstream crate API in design). Verified the exact `ssimulacra2` 0.5.1 API (compute_frame_ssimulacra2 + Rgb::new + TransferCharacteristic::SRGB / ColorPrimaries::BT709) against docs.rs. Emitted DEC-019 (ssimulacra2 + metric/threshold/search policy). Complexity M (new `src/quality` module + first permissive metric dep + CLI surface on `shrink`)."
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-16
      notes: "Build executed by the ORCHESTRATOR (Opus) as the sanctioned fallback after the dispatched Sonnet 4.6 background subagent could not obtain Bash permission in its non-interactive context (did zero work). Implemented `prompts/SPEC-016-build.md` literally: new src/quality module (SSIMULACRA2 metric + generic quality search) + ssimulacra2 0.5.1 default dep + shrink --target/--ssim wiring + 14 tests. All 5 gates green (build · test 220 · clippy --all-targets · fmt · just deny); no deny.toml change needed. PR #18."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-016: perceptual auto-quality — shrink to a visual target

## Context

This is **STAGE-008's flagship** and the project's headline differentiator:
"**set the look, not the quality number.**" Today `shrink photo.jpg` re-encodes
JPEG at a fixed default quality (80, DEC-016) — the user still has to *guess* a
number and eyeball the result. The frontier paradigm (oavif, jpegli, squoosh's
auto modes) is to encode to a **perceptual target** and let the tool find the
encoder setting. SPEC-016 brings that to crustyimg on the already-shipped default
JPEG path.

- **Parent stage:** `STAGE-008` (modern formats & quality), spec #1 of 4 — the
  flagship, chosen first because it is the differentiator AND the most
  self-contained (default JPEG path, one permissive pure-Rust dependency).
- **What's new:** `shrink` gains `--target <PRESET>` and `--ssim <N>`. With
  either set, instead of a fixed quality, `shrink` **binary-searches the JPEG
  encoder quality**: for each candidate quality it encodes the (already-resized)
  image to JPEG in memory, decodes it back, and scores the decoded result against
  the resized original with **SSIMULACRA2** (the `ssimulacra2` crate, BSD-2,
  pure-Rust). It stops at the **lowest quality whose score ≥ the target** — the
  smallest file that still looks good enough. The metric + search live in a new
  `src/quality/` module; `DEC-019` records the metric, the threshold presets, and
  the search policy.
- **What stays the same:** `shrink` without `--target`/`--ssim` behaves **exactly
  as today** (resize to `--max` bound + fixed default quality 80 + metadata drop).
  Auto-quality is strictly **opt-in**. The resize step, source-format preservation
  (DEC-015), multi-input `--out-dir` fan-out + partial-batch exit 6 all come for
  free via the shared `run_pixel_op` helper — auto-quality plugs into it as a
  per-image quality resolution.
- **Scope is JPEG for v1** (the common web case). The search loop is deliberately
  encoder-agnostic so SPEC-017 (`--max-size`) reuses it and SPEC-018/019 generalize
  it onto AVIF/WebP. For a non-JPEG output format, `--target`/`--ssim` is a
  documented **no-op** (the encoder default is used), exactly mirroring how `-q`
  is ignored for lossless formats (DEC-016).

The api-contract addition: `shrink <INPUT...> [--max N] [--target PRESET | --ssim N]`.

## Goal

Wire perceptual auto-quality into `shrink`: with `--target <preset>` or
`--ssim <score>`, binary-search the JPEG encode quality against an SSIMULACRA2
score and emit the lowest-quality (smallest) JPEG whose score clears the target,
in a capped, in-memory, sub-second search — preserving `shrink`'s existing
behavior when neither flag is given. Add the metric + search as a self-contained,
unit-testable `src/quality/` module backed by the permissive `ssimulacra2` crate
(DEC-019); change no other command's behavior.

## Inputs

- **Files to read:**
  - `src/sink/mod.rs` — `encode_to_bytes(img: &Image, format, quality: Option<u8>)`
    (the DEC-016 JPEG quality encode this search drives candidate-by-candidate),
    `Sink::write` (the final write path, unchanged), `SinkError::Encode`.
  - `src/cli/mod.rs` — `run_pixel_op` (the shared fan-out — you ADD one trailing
    `auto: Option<quality::SearchConfig>` param and resolve it per-image), the
    existing `quality` threading, `run_shrink`/`shrink_params`/`DEFAULT_SHRINK_MAX`
    /`DEFAULT_SHRINK_QUALITY`, the OTHER callers of `run_pixel_op`
    (`run_resize`/`run_thumbnail`/`run_convert`/`run_auto_orient` — they each get
    one new `None` arg), `run_apply` (unaffected — it calls `sink.write` directly,
    not `run_pixel_op`), `Commands::Shrink { inputs, max }`, the dispatch arm,
    `CliError` + `code()` + `exit_code_mapping_is_total`.
  - `src/image/mod.rs` — `Image::pixels() -> &DynamicImage` (the resized pixels the
    search scores), `Image::from_bytes` (READ-ONLY — do not edit this module).
  - `src/lib.rs` — the module list (you add `pub mod quality;`).
  - `Cargo.toml` — deps + the `[features]` block (you add `ssimulacra2 = "=0.5.1"`
    as a DEFAULT dep — pure-Rust, BSD-2, permissive; no feature gate).
  - `deny.toml` — the cargo-deny allowlist (DEC-018). After adding the dep, run
    `just deny`; if it fails ONLY on an unlisted *permissive* license (e.g. `ISC`),
    add that exact SPDX id to the `allow` list and note it; if it fails on
    GPL/AGPL/LGPL, STOP (see Notes).
  - `tests/cli.rs`, `tests/common/mod.rs` — integration conventions + fixtures
    (`gradient_jpeg`, `solid_png`); you add a `detailed_jpeg`/`detailed_png`
    high-frequency fixture (below).
  - `decisions/DEC-019-*.md` (this spec emits it), `DEC-016`, `DEC-015`, `DEC-018`.
- **External APIs:** the `ssimulacra2` crate 0.5.1 (BSD-2-Clause). Exact API
  verified against docs.rs (pin these — do not improvise):
  - `pub fn compute_frame_ssimulacra2<T, U>(source: T, distorted: U) -> Result<f64, ssimulacra2::Ssimulacra2Error> where ssimulacra2::LinearRgb: TryFrom<T> + TryFrom<U>` — higher score = more similar; ~100 = identical; pass two `ssimulacra2::Rgb` values (`LinearRgb: TryFrom<Rgb>` holds).
  - `ssimulacra2::Rgb::new(data: Vec<[f32; 3]>, width: usize, height: usize, transfer: ssimulacra2::TransferCharacteristic, primaries: ssimulacra2::ColorPrimaries) -> Result<ssimulacra2::Rgb, _>` — `data` is row-major `width*height` RGB pixels as `[f32;3]`, **normalized to 0.0..=1.0** (i.e. each 8-bit channel `/ 255.0`). Use `ssimulacra2::TransferCharacteristic::SRGB` and `ssimulacra2::ColorPrimaries::BT709` (8-bit sRGB image data).
  - docs: https://docs.rs/ssimulacra2/0.5.1/ssimulacra2/
- **Related code paths:** new `src/quality/` (metric + search) + `src/cli/`
  (the `shrink` surface + the per-image resolution in `run_pixel_op`). Do NOT
  modify `src/image`, `src/operation`, `src/pipeline`, `src/recipe`, `src/source`.

## Outputs

- **Files created:**
  - **`src/quality/mod.rs`** — the perceptual metric + the quality search. Public
    API (every public fn/type gets a test):
    - `#[derive(Debug, thiserror::Error)] pub enum QualityError` with arms:
      `Score(String)` (wraps a `ssimulacra2::Ssimulacra2Error` via `.to_string()`),
      `Convert(String)` (wraps the `Rgb::new` error via `.to_string()`),
      `Encode(String)` (a candidate JPEG encode failed). All `#[error("...")]`.
    - `pub fn score(reference: &::image::DynamicImage, candidate: &::image::DynamicImage) -> Result<f64, QualityError>`
      — convert BOTH via the private `to_ss_rgb` helper (`.to_rgb8()` → `Vec<[f32;3]>`
      normalized by `/255.0` → `Rgb::new(.., SRGB, BT709)`), then
      `compute_frame_ssimulacra2(ref_rgb, cand_rgb)`; map both error types into
      `QualityError`. (Reference and candidate MUST be the same dimensions — they
      are, since the candidate is `reference` round-tripped through JPEG.)
    - `#[derive(Debug, Clone)] pub struct SearchConfig { pub target: f64, pub min_quality: u8, pub max_quality: u8, pub max_iters: u8 }`
      with `pub fn for_target(target: f64) -> Self` defaulting `min_quality = MIN_SEARCH_QUALITY (1)`, `max_quality = MAX_SEARCH_QUALITY (100)`, `max_iters = MAX_SEARCH_ITERS (8)` (the DEC-019 constants).
    - `#[derive(Debug, Clone, Copy)] pub struct QualityChoice { pub quality: u8, pub score: f64, pub iterations: u8, pub met_target: bool }`
      — the search result.
    - `pub fn search_jpeg_quality<F>(score_at: F, cfg: &SearchConfig) -> Result<QualityChoice, QualityError> where F: FnMut(u8) -> Result<f64, QualityError>`
      — **generic over the per-quality scorer** so it is deterministically
      unit-testable with a synthetic monotonic closure. Binary-search the integer
      quality range for the LOWEST quality whose score ≥ `cfg.target`; memoize
      scores; cap at `cfg.max_iters` distinct `score_at` calls. If no quality in
      range meets the target, return the best-effort `max_quality` result with
      `met_target = false` (NEVER error for "unreachable target"). Propagate a
      `score_at` `Err` (a real scoring/encode failure) unchanged.
    - `pub fn auto_jpeg_quality(reference: &::image::DynamicImage, cfg: &SearchConfig) -> Result<QualityChoice, QualityError>`
      — the production wiring: `search_jpeg_quality(|q| { encode `reference` to JPEG
      bytes at quality `q` via `::image::codecs::jpeg::JpegEncoder::new_with_quality`
      (clamp `q` to 1..=100, identical to DEC-016/`encode_to_bytes`); decode the
      bytes via `::image::load_from_memory`; `score(reference, &decoded)` }, cfg)`.
      Encode failures → `QualityError::Encode`; decode failures → `QualityError::Encode`
      (a corrupt round-trip is an encode-path failure).
    - `pub const MIN_SEARCH_QUALITY: u8 = 1; pub const MAX_SEARCH_QUALITY: u8 = 100; pub const MAX_SEARCH_ITERS: u8 = 8;` (DEC-019).
    - The module depends ONLY on `::image`, `ssimulacra2`, `thiserror`, and `std`.
      It must NOT depend on `clap`, `crate::cli`, `crate::sink`, files, or terminals
      (it is a pure pixel+metric module, like `src/operation`).
  - **`decisions/DEC-019-perceptual-auto-quality-ssimulacra2.md`** — emitted in
    THIS design cycle (see below); the metric + threshold + search policy.
- **Files modified:**
  - **`src/lib.rs`** — add `pub mod quality;` (alongside the existing modules).
  - **`Cargo.toml`** — add `ssimulacra2 = "=0.5.1"` to `[dependencies]` (DEFAULT,
    no feature gate; pure-Rust, BSD-2 — DEC-019/DEC-018). Pin the exact patch.
  - **`deny.toml`** — ONLY if `just deny` flags an unlisted *permissive* transitive
    license (e.g. `ISC`): add that exact SPDX id to `allow`. Do not add anything
    copyleft (see Notes).
  - **`src/cli/mod.rs`**:
    - Add a clap value-enum:
      `#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)] pub enum QualityTarget { VisuallyLossless, High, Medium }`
      (clap renders these as `visually-lossless`, `high`, `medium`). Add
      `fn target_score(self) -> f64` mapping `VisuallyLossless => 90.0`,
      `High => 70.0`, `Medium => 50.0` (the DEC-019 presets).
    - Extend `Commands::Shrink` with two fields:
      `#[arg(long, value_enum)] target: Option<QualityTarget>` and
      `#[arg(long, conflicts_with = "target")] ssim: Option<f64>` (clap's
      `conflicts_with` makes `--target` + `--ssim` together a usage error → exit 2).
    - Update the dispatch arm to
      `Commands::Shrink { inputs, max, target, ssim } => run_shrink(inputs, *max, *target, *ssim, &cli.global)`.
    - Rewrite `run_shrink(inputs, max, target: Option<QualityTarget>, ssim: Option<f64>, global)`:
      resolve `auto: Option<quality::SearchConfig>` from `(target, ssim)` —
      `target` → `SearchConfig::for_target(t.target_score())`; `ssim` → validate
      `(0.0..=100.0).contains(&s)` (else `CliError::Usage`, exit 2) then
      `SearchConfig::for_target(s)`; neither → `None`. If `auto.is_some()` AND
      `global.quality.is_some()` → `CliError::Usage` ("`-q/--quality` cannot be
      combined with `--target`/`--ssim`"; exit 2). Build the resize op as today.
      The fixed quality passed to the fan-out is `None` when `auto.is_some()`, else
      `Some(global.quality.unwrap_or(DEFAULT_SHRINK_QUALITY))` (today's behavior).
      Call `run_pixel_op(pipeline, inputs, global, fixed_quality, None, auto)`.
    - `run_pixel_op`: add a trailing param `auto: Option<quality::SearchConfig>`
      (mirroring how `forced_format` was appended in SPEC-014). In BOTH the single
      and the multi arms, AFTER `out_img` and `fmt` are resolved and BEFORE
      `sink.write`, compute the EFFECTIVE quality:
      ```text
      let effective_quality: Option<u8> = match &auto {
          Some(cfg) if fmt == ImageFormat::Jpeg => {
              Some(quality::auto_jpeg_quality(out_img.pixels(), cfg)?.quality)
          }
          Some(_) => None,          // non-JPEG output: target ignored (encoder default)
          None     => quality,      // existing fixed-quality behavior
      };
      ```
      then `sink.write(&out_img, &sink_input, overwrite, effective_quality, out)`.
      In the multi arm this `?` is inside the per-input closure, so a scoring
      failure on one input becomes a partial-batch failure (exit 6), not a whole-run
      abort — the desired behavior.
    - Update the OTHER `run_pixel_op` callers — `run_resize`, `run_thumbnail`,
      `run_convert`, `run_auto_orient` — to pass `None` as the new trailing `auto`
      arg (one-token change each; behavior unchanged).
    - `CliError`: add `#[error(transparent)] Quality(#[from] quality::QualityError)`
      and map it to exit code **1** (generic runtime error) in `code()`; extend
      `exit_code_mapping_is_total` to assert the new arm.
  - **`tests/common/mod.rs`** — add `pub fn detailed_jpeg(w: u32, h: u32) -> Vec<u8>`
    and `pub fn detailed_png(w: u32, h: u32) -> Vec<u8>`: a DETERMINISTIC
    **structured** RGB pattern — a smooth gradient PLUS a mild mid-frequency texture.
    The structure is deliberate (see Notes): NOT a flat gradient/solid (those
    compress near-losslessly and score ~100 at every quality, so the search would
    always pick `min_quality`), and NOT pure high-frequency noise (that is so
    adversarial for JPEG that even quality 100 may not clear a high target AND a low
    target may also fail below 100, collapsing the monotonicity test). A
    gradient-dominated image with an 8px checker texture degrades cleanly at low
    quality yet reaches a high score at high quality. Exact pattern per channel at
    pixel (x, y) over `w×h`:
    ```text
    let gx = (x * 255 / w.max(1)) as i32;                 // smooth horizontal gradient
    let gy = (y * 255 / h.max(1)) as i32;                 // smooth vertical gradient
    let tex = if ((x / 8) + (y / 8)) % 2 == 0 { 30 } else { 0 };  // 8px mid-freq texture
    r = (gx + tex).clamp(0, 255) as u8
    g = (gy + tex).clamp(0, 255) as u8
    b = ((gx + gy) / 2).clamp(0, 255) as u8
    ```
    `detailed_jpeg` encodes this to JPEG (default quality), `detailed_png` to PNG.
  - **`docs/api-contract.md`** — extend the `shrink` entry with `--target`/`--ssim`
    (see Notes for the exact wording to add).
- **New decisions:** `DEC-019` (emitted in this design cycle).
- **No new `Operation`. No change to the `Sink` public API. No second image
  library. The only new dependency is `ssimulacra2` (permissive, default).**

## Acceptance Criteria

Each maps to a test (names in `## Failing Tests`).

- [ ] `shrink <jpg> --target visually-lossless -o out.jpg` → exit 0; `out.jpg`
  decodes as a JPEG at the resized dimensions. → `shrink_target_visually_lossless_produces_valid_jpeg`
- [ ] On the same detailed input, a lower target yields a smaller-or-equal file:
  `--ssim 50` output bytes `<` `--ssim 95` output bytes. → `shrink_lower_ssim_target_is_smaller_file`
- [ ] `--target` and `--ssim` together → exit 2 (clap conflict). → `shrink_target_and_ssim_conflict_exits_2`
- [ ] `--ssim 150` (out of 0..=100) → exit 2. → `shrink_ssim_out_of_range_exits_2`
- [ ] `-q 80 --target high` → exit 2 (quality + auto are incompatible). → `shrink_quality_and_target_conflict_exits_2`
- [ ] `shrink <png> --target visually-lossless -o out.png` → exit 0; output is PNG,
  target ignored, no error. → `shrink_target_non_jpeg_is_ignored`
- [ ] Multi-input `shrink a.jpg b.jpg --target high --out-dir D` → exit 0; both
  outputs are valid JPEGs (per-image search). → `shrink_target_multi_input_fan_out`
- [ ] `shrink <jpg>` with NO `--target`/`--ssim` is byte-identical to today
  (existing shrink tests stay green). → existing suite
- [ ] `quality::score(img, img)` on a detailed image is high (≥ 90). → `score_identical_is_high`
- [ ] `quality::score(img, degraded)` < `score(img, img)`, and a heavily-degraded
  candidate scores below 90. → `score_degraded_is_lower`
- [ ] `search_jpeg_quality` with a synthetic monotonic scorer returns the LOWEST
  quality meeting the target, in ≤ `max_iters` scorer calls. → `search_finds_lowest_meeting_target`
- [ ] `search_jpeg_quality` with an always-below scorer returns `max_quality`,
  `met_target=false`. → `search_unreachable_target_is_best_effort`
- [ ] `auto_jpeg_quality` on a detailed image: a lower target picks a
  lower-or-equal quality than a higher target. → `auto_jpeg_quality_is_monotone_in_target`

## Failing Tests

Written during **design**, made to pass during **build**. Unit tests live in
`src/quality/mod.rs`'s `#[cfg(test)] mod tests`; integration tests drive the real
binary in `tests/cli.rs` (native in-memory fixtures; `tempfile`; assert exit codes
via `output.status.code()`; decode outputs with `image::load_from_memory`).

- **`src/quality/mod.rs`** (UNIT — build images in-module via a local
  `fn detailed_rgb(w, h) -> ::image::DynamicImage` mirroring the `detailed_jpeg`
  pattern; sizes ≥ 64×64 so SSIMULACRA2's internal downscaling is well-defined):
  - `score_identical_is_high` — `score(&img, &img)` for a 96×96 detailed image
    returns a value `> 90.0` (identical frames score ~100).
  - `score_degraded_is_lower` — encode the detailed image to JPEG at quality 8 via
    `JpegEncoder::new_with_quality`, decode it, `let s = score(&img, &decoded)?`;
    assert `s < score(&img, &img)?` AND `s < 90.0` (degradation is detected).
  - `search_finds_lowest_meeting_target` — `search_jpeg_quality(|q| Ok(q as f64),
    &SearchConfig::for_target(50.0))` returns `quality == 50`, `met_target == true`
    (lowest q with `q as f64 >= 50.0`). Assert the scorer was called ≤ `max_iters`
    times (use a `Cell`/counter in the closure).
  - `search_unreachable_target_is_best_effort` — `search_jpeg_quality(|_| Ok(10.0),
    &SearchConfig::for_target(90.0))` returns `quality == MAX_SEARCH_QUALITY`,
    `met_target == false`.
  - `search_propagates_scorer_error` — a scorer returning
    `Err(QualityError::Encode("boom".into()))` makes `search_jpeg_quality` return
    that `Err` (not swallow it).
  - `auto_jpeg_quality_is_monotone_in_target` — on a 96×96 detailed image,
    `auto_jpeg_quality(.., for_target(50.0))?.quality <= auto_jpeg_quality(.., for_target(90.0))?.quality`.
  - `search_config_defaults_match_dec019` — `SearchConfig::for_target(90.0)` has
    `min_quality == 1`, `max_quality == 100`, `max_iters == 8`, `target == 90.0`.
- **`tests/cli.rs`** (INTEGRATION — add `mod common;` fixtures; use
  `common::detailed_jpeg`/`common::detailed_png`; sizes ≥ 128×128 so the search has
  real signal):
  - `shrink_target_visually_lossless_produces_valid_jpeg` — `detailed_jpeg(160,160)`
    → `shrink <jpg> --target visually-lossless -o out.jpg` → exit 0; `out.jpg`
    decodes as a JPEG (`image::load_from_memory` → `ImageFormat::Jpeg` /
    `guess_format`), dimensions == input dimensions (no `--max` downscale below
    160, default 1600 bound is not hit).
  - `shrink_lower_ssim_target_is_smaller_file` — same `detailed_jpeg(160,160)`;
    `shrink in.jpg --ssim 50 -o lo.jpg` and `shrink in.jpg --ssim 95 -o hi.jpg`;
    assert `fs::metadata("lo.jpg").len() < fs::metadata("hi.jpg").len()`.
  - `shrink_target_and_ssim_conflict_exits_2` — `shrink in.jpg --target high
    --ssim 80 -o o.jpg` → exit 2.
  - `shrink_ssim_out_of_range_exits_2` — `shrink in.jpg --ssim 150 -o o.jpg` →
    exit 2; stderr mentions ssim/range.
  - `shrink_quality_and_target_conflict_exits_2` — `shrink in.jpg -q 80
    --target high -o o.jpg` → exit 2; stderr mentions quality/target.
  - `shrink_target_non_jpeg_is_ignored` — `detailed_png(120,120)` → `shrink in.png
    --target visually-lossless -o out.png` → exit 0; `out.png` decodes as PNG
    (magic bytes), no error (target ignored for the lossless format).
  - `shrink_target_multi_input_fan_out` — two `detailed_jpeg` inputs → `shrink
    a.jpg b.jpg --target high --out-dir D` → exit 0; `D/a.jpg` and `D/b.jpg` both
    decode as JPEG.
  - (existing `shrink_*` tests remain unchanged and MUST stay green — `shrink`
    with no auto flags is unaffected.)

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply
- **`DEC-019`** (emitted with this spec) — THE governing decision: adopt
  `ssimulacra2` (BSD-2, permissive, default dep); SSIMULACRA2 as the perceptual
  metric (higher = better, ~100 identical); the threshold presets
  (visually-lossless 90 / high 70 / medium 50, tunable constants); the search
  policy (binary-search integer JPEG quality 1..=100 for the lowest score ≥ target,
  cap 8 iters, memoize, best-effort `max_quality` on unreachable target, JPEG-only
  v1, target ignored for non-JPEG output).
- `DEC-016` — encode-quality policy: `-q` → `JpegEncoder::new_with_quality`,
  ignored for lossless. The search drives this exact JPEG encode per candidate; the
  "target ignored for non-JPEG output" rule mirrors DEC-016's "`-q` ignored for
  lossless." `shrink`'s fixed default (80) still applies when NO auto flag is set.
- `DEC-015` — output-format precedence (`--format` > `-o` ext > preserve source) +
  partial-batch exit 6: inherited via `run_pixel_op`. The auto search runs ONLY
  when the resolved per-input format is JPEG.
- `DEC-018` / `no-agpl-default-deps` — the new dep MUST be permissive; `just deny`
  must stay green. `ssimulacra2` is BSD-2 (permissive). Run `just deny` after adding.
- `DEC-004` / `pure-rust-codecs-default` — `ssimulacra2` is pure-Rust, no system
  deps, default-buildable; no feature gate needed (unlike AVIF/native codecs).
- `DEC-002` / `decode-once-no-per-op-disk` — the original is decoded ONCE by the
  pipeline; the search re-encodes/decodes CANDIDATES in memory only (≤ 8, no disk).
- `DEC-007` / `DEC-012` — typed errors → exit codes; clap surface isolated in cli.

### Constraints that apply
- `ergonomic-defaults` — `shrink photo.jpg --target visually-lossless` is one
  short command; the preset names read as outcomes ("visually-lossless"), not knobs.
- `single-image-library` — scoring converts via the `image` crate's `.to_rgb8()`;
  `ssimulacra2` is a METRIC crate, not a second pixel/image-processing library
  (it does not decode/encode/resize images) — it does not violate the
  single-pixel-library rule. No `imageproc`, no photon.
- `pure-rust-codecs-default` / `no-agpl-default-deps` / `no-new-top-level-deps-without-decision`
  — `ssimulacra2` is pure-Rust + BSD-2 + carries DEC-019; `just deny` confirms.
- `decode-once-no-per-op-disk` — candidates never touch disk.
- `no-unwrap-on-recoverable-paths` — all scoring/encode failures are typed
  `QualityError`/`CliError`; no `unwrap`/`expect`/`panic!` in `src/`.
- `every-public-fn-tested` — `score`, `search_jpeg_quality`, `auto_jpeg_quality`,
  `SearchConfig::for_target`, `QualityTarget::target_score` all get tests.
- `clippy-fmt-clean` — `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check`.
- `untrusted-input-hardening` — candidate quality clamped 1..=100; a scoring/encode
  failure (incl. a pathologically tiny or malformed image) is a typed error, not a
  panic; the iteration cap bounds work on any input.

### Prior related work
- `SPEC-013` (shipped, PR #14) — `shrink` + the DEC-016 quality encode +
  `encode_to_bytes(quality)` this search drives; `run_shrink`/`shrink_params` you
  extend.
- `SPEC-014` (shipped, PR #15) — added the trailing `forced_format` param to
  `run_pixel_op`; the `auto` param is added the SAME way.
- `SPEC-011` (shipped) — `run_pixel_op` fan-out + DEC-015 partial-batch exit 6.
- `SPEC-005` (shipped) — the `Sink`/`encode_to_bytes` the final write uses.

### Out of scope (create a new spec rather than expand)
- `--max-size <KB>` byte budget (**SPEC-017** — reuses this search machinery).
- AVIF / WebP output and applying auto-quality to non-JPEG encoders
  (**SPEC-018/019** — the search is already encoder-agnostic, but the encoders
  are not built yet).
- Auto-quality on `convert` or any command other than `shrink`.
- Parallelizing the search or the batch (rayon is STAGE-005); caching the winning
  candidate bytes to skip the final re-encode (a minor optimization — not now).
- A `--strict` mode that errors when the target is unreachable (v1 is best-effort
  with `met_target=false`; could log under `-v`).
- Tuning the exact preset scores beyond DEC-019's anchors, or a `--json` report of
  the chosen quality/score (nice, but its own small spec).

## Notes for the Implementer

- **The search must be generic over the scorer.** `search_jpeg_quality<F: FnMut(u8)
  -> Result<f64, QualityError>>` is what makes the binary-search logic
  deterministically unit-testable (synthetic `|q| Ok(q as f64)`) AND reusable by
  `auto_jpeg_quality` (real encode→decode→score) and later by SPEC-017's size
  budget. Do NOT bake the encoding into the search.
- **Binary-search shape** (lowest quality with score ≥ target; memoize; cap iters):
  ```text
  fn search_jpeg_quality<F: FnMut(u8) -> Result<f64, QualityError>>(mut score_at, cfg) {
      let (mut lo, mut hi) = (cfg.min_quality, cfg.max_quality);
      let mut best: Option<(u8, f64)> = None;   // lowest q meeting target
      let mut last: (u8, f64) = (cfg.max_quality, f64::NAN); // fallback
      let mut cache: BTreeMap<u8, f64> = BTreeMap::new();
      let mut iters: u8 = 0;
      while lo <= hi && iters < cfg.max_iters {
          let mid = lo + (hi - lo) / 2;
          let s = match cache.get(&mid) { Some(&s) => s,
              None => { iters += 1; let s = score_at(mid)?; cache.insert(mid, s); s } };
          last = (mid, s);
          if s >= cfg.target { best = Some((mid, s)); if mid == cfg.min_quality { break } hi = mid - 1; }
          else { if mid == cfg.max_quality { break } lo = mid + 1; }
      }
      Ok(match best {
          Some((q, s)) => QualityChoice { quality: q, score: s, iterations: iters, met_target: true },
          None => {
              // No tested quality met the target. Best effort = the highest quality.
              // If max wasn't scored, its score is unknown; report `last` for context.
              QualityChoice { quality: cfg.max_quality, score: last.1, iterations: iters, met_target: false }
          }
      })
  }
  ```
  (Adjust freely as long as the tests pass; the contract is "lowest q with score ≥
  target, else max_quality best-effort, ≤ max_iters distinct scorer calls,
  propagate scorer Err". Watch the `u8` underflow at `mid == 0`/`min_quality` — the
  guards above avoid it.)
- **`auto_jpeg_quality` encode/decode** mirrors `encode_to_bytes`'s JPEG path
  EXACTLY (DEC-016): `let mut cur = Cursor::new(Vec::new()); let enc =
  ::image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cur, q.clamp(1,100));
  reference.write_with_encoder(enc).map_err(|e| QualityError::Encode(e.to_string()))?;
  let decoded = ::image::load_from_memory(&cur.into_inner()).map_err(|e|
  QualityError::Encode(e.to_string()))?;` then `score(reference, &decoded)`.
- **`to_ss_rgb` conversion** (pin this): `let rgb = img.to_rgb8(); let (w,h) =
  rgb.dimensions(); let data: Vec<[f32;3]> = rgb.pixels().map(|p| [p[0] as f32/255.0,
  p[1] as f32/255.0, p[2] as f32/255.0]).collect(); ssimulacra2::Rgb::new(data, w as
  usize, h as usize, ssimulacra2::TransferCharacteristic::SRGB,
  ssimulacra2::ColorPrimaries::BT709).map_err(|e| QualityError::Convert(e.to_string()))`.
  Then `ssimulacra2::compute_frame_ssimulacra2(ref_rgb, cand_rgb).map_err(|e|
  QualityError::Score(e.to_string()))`. The exact import path for `Rgb`/the enums is
  the crate root (`ssimulacra2::Rgb`, `ssimulacra2::TransferCharacteristic`,
  `ssimulacra2::ColorPrimaries`); confirm with `cargo doc -p ssimulacra2` if a path
  differs in 0.5.1, but do NOT change the enum *variants* (`SRGB`, `BT709`).
- **Fixtures must be STRUCTURED, not flat and not pure noise.** A smooth
  `gradient_jpeg`/`solid_png` compresses near-losslessly → scores ~100 at every
  quality → the search returns `min_quality` and the monotonicity tests are vacuous.
  Pure high-frequency noise is the opposite failure: JPEG can't reach a high score
  even at quality 100, so `--ssim 50` and `--ssim 95` can BOTH hit the q=100 ceiling
  and produce identical files (the strict `<` size test fails). The gradient+8px-checker
  pattern (Outputs) threads the needle: `--ssim 50` is met at a LOW quality (small
  file) while `--ssim 95` pushes quality high (large file or best-effort q=100) — so
  `lo < hi` is robust even if the high target is only met best-effort. Use this exact
  pattern in `detailed_jpeg`/`detailed_png` and the in-module `detailed_rgb`.
- **Score is content-dependent — assert RELATIONSHIPS, not absolute scores** in
  tests (identical > degraded; lower target ≤ higher target's quality/size). The
  one absolute is `score(img,img) > 90` (identical ≈ 100, safe across OSes) and
  "degraded < 90" at quality 8 on a detailed image (safe). Keep fixtures ≥ 64×64
  (unit) / ≥ 128×128 (integration) so SSIMULACRA2's 6× internal downscale is valid.
- **Adding the dep:** put `ssimulacra2 = "=0.5.1"` in `[dependencies]` (NOT behind a
  feature). Then `cargo build` and `just deny`. If `just deny` fails ONLY because a
  transitive dep carries a permissive license not yet in `deny.toml`'s `allow` list
  (the most likely candidate is `ISC`), add that exact SPDX id to `allow` and note
  it in the PR (DEC-018 explicitly anticipates this one-line fix). If it fails on
  ANY GPL/AGPL/LGPL license, STOP, do NOT add an exception, and write a question to
  `guidance/questions.yaml` — that would contradict the handoff's license analysis
  and needs the architect.
- **`run_pixel_op` change is surgical** — append `auto: Option<quality::SearchConfig>`,
  add the `effective_quality` match in the two `sink.write` sites, pass `None` from
  the four non-shrink callers. Do not refactor the fan-out otherwise.
- **api-contract wording to add** under the `shrink` entry: "`--target
  <visually-lossless|high|medium>` / `--ssim <0-100>` auto-tune the **JPEG** encode
  quality to a perceptual **SSIMULACRA2** target (binary-search the lowest quality
  whose score clears the target; capped, in-memory; DEC-019). Mutually exclusive
  with each other and with `-q`. Opt-in: without them `shrink` uses the fixed
  default quality (80). For a non-JPEG output format the target is **ignored**
  (encoder default), mirroring `-q` on lossless formats (DEC-016). If the target is
  unreachable even at quality 100, `shrink` emits the highest-quality encode
  (best-effort)."
- **Derive `Debug`** on `QualityError`, `SearchConfig`, `QualityChoice`,
  `QualityTarget`. Do not `{:?}`-format a non-Debug type (Sonnet has hit this).
- **Commit incrementally:** commit once `src/quality` + the dep compile and
  clippy/fmt are clean, again once the unit tests pass, again once the CLI wiring +
  integration tests pass. A green committed checkpoint must survive an interruption.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-016-perceptual-auto-quality-shrink-to-a-visual-target`
- **PR (if applicable):** #18 https://github.com/jysf/crustyimg/pull/18
- **All acceptance criteria met?** yes — all 14 named tests (7 unit in
  `src/quality`, 7 integration in `tests/cli.rs`) pass; full suite 220 tests green.
- **New decisions emitted:**
  - No new DEC during build — DEC-019 (authored in design) governs.
- **Deviations from spec:**
  - None functional. The dep tree did NOT require any `deny.toml` change —
    `ssimulacra2` 0.5.1 and its full transitive tree (yuvxyb, moxcms, av-data,
    v_frame, rayon, …) are all already-allowlisted permissive licenses; `just deny`
    is green untouched.
  - Process note (not a spec deviation): this build cycle was executed by the
    ORCHESTRATOR (Opus) directly, NOT a fresh Sonnet 4.6 session — the dispatched
    Sonnet background subagent could not obtain Bash permission in its
    non-interactive context and did zero work, so the orchestrator ran the build as
    the sanctioned fallback (AGENTS gotcha: finish the build yourself on a subagent
    drop). Implementation followed `prompts/SPEC-016-build.md` literally.
- **Follow-up work identified:**
  - SPEC-017 (`--max-size <KB>` byte budget) reuses `search_jpeg_quality` directly.
  - SPEC-018/019 (AVIF/WebP) generalize the encoder-agnostic search onto the new
    formats; `resolve_effective_quality` will extend past the JPEG-only guard then.
  - Possible niceties (own small specs): a `--strict` mode that errors on an
    unreachable target (today: best-effort `met_target=false`), and a `--json`
    report of the chosen quality/score/iterations.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing material. The spec pinned the exact `ssimulacra2` API
   (`compute_frame_ssimulacra2` + `Rgb::new` + `TransferCharacteristic::SRGB` /
   `ColorPrimaries::BT709`) and the binary-search sketch, so the module compiled
   and its 7 unit tests passed on the first real run. The structured-fixture
   guidance (gradient + 8px checker, neither flat nor pure noise) was the one
   subtle thing the spec rightly called out — it is exactly what makes the
   monotonicity assertions hold.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The relevant set (`no-agpl-default-deps` + the `just deny` gate,
   `single-image-library` with the explicit "a metric crate is not a second pixel
   library" carve-out, `decode-once`) was all listed and load-bearing.

3. **If you did this task again, what would you do differently?**
   — Validate the new dependency + `just deny` BEFORE writing any module code (I
   did this here — add dep → `cargo build` → `just deny` → then write the module),
   which de-risks the whole spec in the first two minutes. Worth making the default
   order for any spec that adds a dep.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
