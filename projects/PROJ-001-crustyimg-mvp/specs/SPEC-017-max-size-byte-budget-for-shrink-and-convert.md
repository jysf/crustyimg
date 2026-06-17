---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-017
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
  decisions: [DEC-019, DEC-016, DEC-015, DEC-004, DEC-018, DEC-007, DEC-012]
  constraints:
    - ergonomic-defaults
    - single-image-library
    - pure-rust-codecs-default
    - decode-once-no-per-op-disk
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - test-before-implementation
    - untrusted-input-hardening
  related_specs: [SPEC-016, SPEC-013, SPEC-014, SPEC-005]

value_link: "Delivers the other half of STAGE-008's 'tell me the outcome, not the knob' thesis — `shrink`/`convert --max-size 200KB` hits a FILE SIZE instead of a quality number, the top-ranked user pain for web image prep. Reuses the SPEC-016 perceptual search machinery (inverted) at near-zero marginal cost."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-16
      notes: "Design authored by the ORCHESTRATOR (Opus) directly (SPEC-013/014/015/016 pattern). Scope confirmed with the user: quality-only v1 (dimension-reduction fallback deferred to a follow-up). Reuses the shipped SPEC-016 `src/quality` search via a generic monotone-threshold core (also addresses the SPEC-016 review's 'generalize the search' altitude note). No new DEC — the byte-budget policy is an extension of DEC-019, recorded here."
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-16
      notes: "Build executed by the ORCHESTRATOR (Opus) directly in the main loop (the background-subagent-Bash limitation from SPEC-016 still applies). null numerics here — the orchestrator records a real-ish figure at ship (main-loop work has no clean per-cycle metering; /cost). Implemented `prompts/SPEC-017-build.md`: the `search_threshold` refactor (SPEC-016 tests preserved) + `search_jpeg_under_size`/`auto_jpeg_under_size`/`jpeg_size_at` + `AutoQuality` enum + `parse_size`/`fmt_bytes` + `--max-size` on shrink/convert. 17 new tests; full suite 238 green; all 5 gates pass. PR #20."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-017: `--max-size <SIZE>` byte budget for `shrink` and `convert`

## Context

SPEC-016 shipped the *quality-target* half of "set the look, not the number"
(`shrink --target`/`--ssim`). SPEC-017 ships the *size-target* half: **`--max-size
200KB`** — produce an output **≤ a byte budget**, no quality guessing. Hitting a
file-size cap is the top-ranked web-prep pain (CDN limits, email attachments,
"keep it under 100 KB").

- **Parent stage:** `STAGE-008` (modern formats & quality), spec #2 of 4 — the
  sibling of the flagship.
- **What's new:** `shrink` and `convert` gain `--max-size <SIZE>`. For a **JPEG**
  output, the command **binary-searches the encoder quality for the HIGHEST
  quality whose encoded size ≤ the budget** — the exact inverse of SPEC-016's
  "lowest quality whose score ≥ target." Encoded size is monotonically
  non-decreasing in quality, so the same capped binary search applies; this search
  needs only the **encoded byte length** (no decode, no SSIMULACRA2).
- **Reuse:** the SPEC-016 `src/quality` module already has the search skeleton.
  This spec **generalizes it** into one private monotone-threshold core that both
  the perceptual search (SPEC-016) and the size search (this) call — which also
  resolves the SPEC-016 review's altitude note that the search was JPEG/score-
  specific. The CLI's `run_pixel_op` `auto` hook (added in SPEC-016) is generalized
  from `Option<SearchConfig>` to an `Option<AutoQuality>` enum carrying either auto
  mode.
- **Scope is QUALITY-ONLY for v1** (user-confirmed 2026-06-16). When the budget
  cannot be met by quality alone — a **lossless output** (PNG/GIF/…), or even
  minimum quality still exceeds the budget — the command emits a **best-effort
  smallest encode + a clear stderr warning** (mirroring SPEC-016's unmet-target
  handling). **Dimension-reduction fallback** (downscale until it fits) is
  explicitly **deferred to a follow-up spec** (added to the STAGE-008 backlog).

The api-contract additions: `shrink <INPUT...> … [--max-size SIZE]` and
`convert <INPUT...> --format FMT [--max-size SIZE]`.

## Goal

Wire `--max-size <SIZE>` into `shrink` and `convert`: for a JPEG output,
binary-search the encoder quality for the largest output that fits the byte
budget; for a budget that quality alone cannot meet (lossless format or
min-quality-too-big), emit a best-effort smallest encode and warn. Generalize the
SPEC-016 `src/quality` search into one monotone-threshold core shared by the
perceptual and size searches, and the CLI `auto` hook into an `AutoQuality` enum.
Change no behavior of `shrink`/`convert` when `--max-size` is absent.

## Inputs

- **Files to read:**
  - `src/quality/mod.rs` (shipped, SPEC-016) — `search_jpeg_quality` (the binary
    search you refactor into a shared core), `SearchConfig`, `QualityChoice`,
    `auto_jpeg_quality`, `score_jpeg_at` (the JPEG encode the size probe mirrors),
    the `MIN_/MAX_SEARCH_QUALITY` + `MAX_SEARCH_ITERS` consts, `QualityError`.
  - `src/cli/mod.rs` (shipped, SPEC-016) — the `auto: Option<SearchConfig>` param
    on `run_pixel_op`, `resolve_effective_quality` (the per-output quality resolver
    + the unmet-target warning), `run_shrink` + `shrink_auto_config`, `run_convert`,
    `QualityTarget`, `Commands::Shrink`/`Commands::Convert`, `CliError` (the
    `Quality` arm + `code()`), `parse_wxh` (the pattern your `parse_size` mirrors).
  - `src/sink/mod.rs` — `encode_to_bytes` (the production JPEG encode the size
    probe MUST match byte-for-byte — see the SPEC-016 cross-ref comment).
  - `docs/api-contract.md` — the `shrink` (SPEC-016-extended) and `convert`
    entries you extend.
  - `tests/common/mod.rs` — `detailed_jpeg`/`detailed_png`/`solid_png` fixtures.
- **External APIs:** none new (no new dependency).
- **Related code paths:** `src/quality/` (the search generalization + size search)
  and `src/cli/` (the `--max-size` surface + `AutoQuality`). Do NOT modify
  `src/image`, `src/operation`, `src/pipeline`, `src/recipe`, `src/source`.

## Outputs

- **Files modified:**
  - **`src/quality/mod.rs`**:
    - **Refactor** the binary search in `search_jpeg_quality` into a private
      generic core:
      `fn search_threshold<F>(probe: F, cfg: &SearchConfig, accept: impl Fn(f64) -> bool, prefer_lower: bool) -> Result<QualityChoice, QualityError> where F: FnMut(u8) -> Result<f64, QualityError>`
      — binary-search the quality range; `accept(metric)` decides if a quality
      satisfies the constraint; `prefer_lower` picks which satisfying quality wins
      (`true` → lowest, for "smallest file ≥ score"; `false` → highest, for
      "largest quality ≤ budget"). On NO satisfying quality, best-effort =
      `cfg.max_quality` when `prefer_lower` else `cfg.min_quality`, with
      `met_target = false`. ≤ `cfg.max_iters` probe calls; propagate a probe `Err`.
    - `search_jpeg_quality` becomes a thin wrapper: `search_threshold(score_at,
      cfg, |m| m >= cfg.target, true)`. (Public signature + behavior UNCHANGED —
      its SPEC-016 tests must stay green.)
    - NEW `pub fn search_jpeg_under_size<F>(size_at: F, budget_bytes: u64, cfg: &SearchConfig) -> Result<QualityChoice, QualityError> where F: FnMut(u8) -> Result<u64, QualityError>`
      — wrapper: `search_threshold(|q| Ok(size_at(q)? as f64), cfg, |m| m <= budget_bytes as f64, false)`. The returned `QualityChoice.score` carries the
      achieved encoded size in bytes (as f64).
    - NEW `pub fn auto_jpeg_under_size(reference: &::image::DynamicImage, budget_bytes: u64) -> Result<QualityChoice, QualityError>`
      — production wiring: `search_jpeg_under_size(|q| jpeg_size_at(reference, q),
      budget_bytes, &SearchConfig::for_size_budget())`.
    - NEW private `fn jpeg_size_at(reference: &DynamicImage, quality: u8) -> Result<u64, QualityError>`
      — encode `reference` to JPEG at `quality.clamp(1,100)` via the SAME
      `JpegEncoder::new_with_quality` path as `score_jpeg_at`/`encode_to_bytes`
      (keep the cross-ref comment), return the byte length. **No decode, no
      scoring** — the size search only needs the encoded length.
    - NEW `pub fn SearchConfig::for_size_budget() -> Self` — same bounds as
      `for_target` (min 1, max 100, iters 8) with `target: f64::NAN` (unused for a
      size search; documented).
    - Generalize the `QualityChoice.score` doc: "the search metric at the chosen
      quality — an SSIMULACRA2 score for a perceptual search, or the encoded byte
      size for a size-budget search."
    - Module dependencies UNCHANGED (`::image`, `ssimulacra2`, `thiserror`, `std`).
  - **`src/cli/mod.rs`**:
    - NEW `pub enum AutoQuality { Perceptual(SearchConfig), SizeBudget(u64) }`
      (`#[derive(Debug, Clone)]`). Replace the `run_pixel_op` param
      `auto: Option<SearchConfig>` with `auto: Option<AutoQuality>`.
    - `resolve_effective_quality`: match the new enum:
      - `Some(AutoQuality::Perceptual(cfg))` + JPEG → `auto_jpeg_quality(..)` (the
        SPEC-016 path; unmet → the existing perceptual warning).
      - `Some(AutoQuality::SizeBudget(budget))` + JPEG → `auto_jpeg_under_size(out_img.pixels(), budget)`; on `!met_target` warn (unless `--quiet`): the budget could not be met by quality (smallest achievable size is `choice.score`), "dimension reduction not yet supported".
      - `Some(_)` + NON-JPEG → `None` (target/budget ignored, encoder default); for
        a `SizeBudget` on a lossless format, warn (unless `--quiet`) that
        `--max-size` needs a lossy format / dimension reduction is not yet
        supported.
      - `None` → fixed `quality`.
    - NEW `fn parse_size(s: &str) -> Result<u64, CliError>` — parse a size string to
      bytes: optional decimal number + optional unit suffix (case-insensitive):
      none/`B` = bytes, `KB`/`K` = ×1000, `MB`/`M` = ×1_000_000, `KiB` = ×1024,
      `MiB` = ×1_048_576. Reject empty / non-numeric / zero / negative / overflow →
      `CliError::Usage` (exit 2). (Mirror `parse_wxh`'s typed-usage-error style.)
    - `Commands::Shrink`: add `#[arg(long, value_name = "SIZE", conflicts_with_all = ["target", "ssim"])] max_size: Option<String>`.
    - `Commands::Convert`: add `#[arg(long, value_name = "SIZE")] max_size: Option<String>`.
    - `run_shrink(inputs, max, target, ssim, max_size, global)`: resolve
      `auto: Option<AutoQuality>` — at most one of `{target, ssim, max_size}` (clap
      `conflicts_with_all` enforces target/ssim/max_size exclusivity); `target`/`ssim`
      → `AutoQuality::Perceptual(..)` (existing `shrink_auto_config`), `max_size`
      → `AutoQuality::SizeBudget(parse_size(..)?)`. Reject `auto.is_some() &&
      global.quality.is_some()` → `CliError::Usage` (exit 2; `-q` pins quality,
      `--max-size`/`--target`/`--ssim` search for it).
    - `run_convert(inputs, format, max_size, global)`: resolve
      `auto = max_size.map(|s| parse_size(s).map(AutoQuality::SizeBudget)).transpose()?`;
      reject `auto.is_some() && global.quality.is_some()` → Usage (exit 2). Pass the
      `auto` through `run_pixel_op` (convert keeps its `forced_format`).
    - Update the dispatch arms for `Shrink`/`Convert` to pass the new fields.
    - Update the `run_pixel_op` callers for the param type change: `run_resize`,
      `run_thumbnail`, `run_auto_orient` pass `None`; `run_shrink`/`run_convert`
      pass the resolved `Option<AutoQuality>`.
    - NO new `CliError` variant (the existing `Quality`/`Usage` arms cover it).
  - **`docs/api-contract.md`** — add `--max-size` to the `shrink` and `convert`
    entries (the wording is in Notes).
- **New decisions:** none. The byte-budget search policy is an extension of
  **DEC-019** (the perceptual-search policy) and is recorded in this spec's
  Implementation Context; DEC-019 already covers the metric/search/best-effort
  approach. (The dimension-fallback deferral is a scope decision, noted in the
  STAGE-008 backlog.)
- **No new dependency. No new `Operation`. No change to the `Sink` public API.**

## Acceptance Criteria

Each maps to a test.

- [ ] `shrink <jpg> --max-size <feasible B> -o out.jpg` → exit 0; `out.jpg` is a
  JPEG whose on-disk size ≤ B. → `shrink_max_size_fits_budget`
- [ ] A larger budget yields a larger-or-equal file than a smaller budget, same
  input. → `shrink_max_size_larger_budget_not_smaller`
- [ ] `--max-size` + `--target` / `--ssim` / `-q` → exit 2. →
  `shrink_max_size_conflicts_*`
- [ ] An infeasible tiny budget (smaller than even quality-1) → exit 0
  (best-effort smallest) + a stderr warning. → `shrink_max_size_infeasible_warns`
- [ ] `shrink <png> --max-size <B>` → exit 0, output is PNG, a stderr warning that
  the budget needs a lossy format. → `shrink_max_size_non_jpeg_warns`
- [ ] `convert <png> --format jpeg --max-size <B>` → exit 0; output JPEG ≤ B. →
  `convert_max_size_to_jpeg_fits`
- [ ] `convert … --max-size … -q N` → exit 2. → `convert_max_size_conflicts_with_quality`
- [ ] `shrink`/`convert` with NO `--max-size` are byte-identical to today
  (existing suites stay green). → existing tests
- [ ] `search_jpeg_under_size` with a synthetic monotone size fn returns the
  HIGHEST quality whose size ≤ budget, ≤ `max_iters` probe calls. →
  `search_under_size_finds_highest_fitting`
- [ ] `search_jpeg_under_size` with an all-too-big size fn returns `min_quality`,
  `met_target=false`. → `search_under_size_unfittable_is_best_effort`
- [ ] `auto_jpeg_under_size`: a smaller budget picks a lower-or-equal quality than
  a larger budget. → `auto_under_size_is_monotone_in_budget`
- [ ] `parse_size` parses units and rejects junk. → `parse_size_*`
- [ ] `search_jpeg_quality` (SPEC-016) is unchanged after the refactor (its tests
  stay green). → existing `quality::tests`

## Failing Tests

Written during **design**, made to pass during **build**. Unit tests in
`src/quality/mod.rs` and `src/cli/mod.rs` `#[cfg(test)]`; integration in
`tests/cli.rs` (drive the real binary; `tempfile`; `image::guess_format`/
`load_from_memory`; assert file sizes via `std::fs::metadata(..).len()`).

- **`src/quality/mod.rs`** (UNIT — reuse the in-module `detailed_rgb` fixture):
  - `search_under_size_finds_highest_fitting` — `search_jpeg_under_size(|q|
    Ok(q as u64 * 10), 500, &SearchConfig::for_size_budget())` → `quality == 50`
    (50×10 = 500 ≤ 500), `met_target == true`, probe calls ≤ `MAX_SEARCH_ITERS`
    (count via a `Cell`).
  - `search_under_size_unfittable_is_best_effort` — `search_jpeg_under_size(|_|
    Ok(10_000), 100, ..)` → `quality == MIN_SEARCH_QUALITY`, `met_target == false`.
  - `search_under_size_propagates_error` — a probe returning `Err(QualityError::Encode(..))` propagates.
  - `auto_under_size_is_monotone_in_budget` — on a 96×96 detailed image,
    `auto_jpeg_under_size(.., small)?.quality <= auto_jpeg_under_size(.., large)?.quality`
    for `small < large` (pick budgets straddling the q-range).
  - `search_config_for_size_budget_bounds` — `for_size_budget()` has `min_quality
    == 1`, `max_quality == 100`, `max_iters == 8`.
  - (the existing SPEC-016 `quality::tests` — `score_*`, `search_*`,
    `auto_jpeg_quality_*`, `search_config_defaults_match_dec019` — MUST stay green
    through the `search_threshold` refactor.)
- **`src/cli/mod.rs`** (UNIT):
  - `parse_size_units` — `"200000" → 200000`, `"200KB" → 200000`, `"200k" →
    200000`, `"1.5MB" → 1_500_000`, `"1KiB" → 1024`, `"2MiB" → 2_097_152`.
  - `parse_size_rejects_junk` — `""`, `"abc"`, `"0"`, `"0KB"`, `"-5KB"`, `"12GB"`
    (unsupported unit) each `Err` with `code() == 2`.
- **`tests/cli.rs`** (INTEGRATION — `detailed_jpeg(160,160)` etc.; for budgets,
  pick values relative to the input's encoded size so they are robust):
  - `shrink_max_size_fits_budget` — `shrink in.jpg --max-size 6KB -o out.jpg` →
    exit 0; `out.jpg` is JPEG and `metadata(out).len() <= 6000`. (6 KB is
    comfortably above a 160×160 quality-1 JPEG, so feasible.)
  - `shrink_max_size_larger_budget_not_smaller` — `--max-size 4KB` vs `--max-size
    12KB`; assert `len(4KB output) <= len(12KB output)`.
  - `shrink_max_size_conflicts_with_target_exits_2` — `--max-size 5KB --target high`
    → exit 2.
  - `shrink_max_size_conflicts_with_ssim_exits_2` — `--max-size 5KB --ssim 90` → exit 2.
  - `shrink_max_size_conflicts_with_quality_exits_2` — `--max-size 5KB -q 80` → exit 2.
  - `shrink_max_size_infeasible_warns` — `shrink in.jpg --max-size 200 -o out.jpg`
    (200 bytes — below quality-1) → exit 0; `out.jpg` exists; stderr warns (contains
    "budget" / "could not"). (Best-effort smallest.)
  - `shrink_max_size_non_jpeg_warns` — `shrink in.png --max-size 1KB -o out.png` →
    exit 0; `out.png` is PNG; stderr warns the budget needs a lossy format.
  - `convert_max_size_to_jpeg_fits` — `convert in.png --format jpeg --max-size 6KB
    -o out.jpg` → exit 0; JPEG, `len <= 6000`.
  - `convert_max_size_conflicts_with_quality_exits_2` — `convert in.png --format
    jpeg --max-size 6KB -q 80` → exit 2.

## Implementation Context

### Decisions that apply
- **`DEC-019`** — the perceptual auto-quality policy. The byte-budget search is its
  dual: same capped binary search over JPEG quality, same best-effort-on-unmet +
  warning, same decode-once discipline — but the metric is **encoded size** (no
  decode, no SSIMULACRA2) and the goal is **highest quality ≤ budget** (not lowest
  ≥ target). This spec generalizes DEC-019's search into the shared
  `search_threshold` core. **No new DEC** — recorded here as a DEC-019 extension.
- `DEC-016` — the JPEG `JpegEncoder::new_with_quality` encode the size probe and
  the sink share; the probe size MUST equal the written size (keep the cross-ref
  comment). `--max-size` is opt-in like `-q`; when absent, behavior is unchanged.
- `DEC-015` — output-format precedence + partial-batch exit 6: inherited via
  `run_pixel_op`. The size search runs only for a JPEG output; `convert`'s forced
  format decides whether the budget is achievable by quality.
- `DEC-004` — JPEG is the only lossy core codec today, so v1 is JPEG-only; AVIF/WebP
  budgets follow when those land (SPEC-018/019), reusing this search.
- `DEC-018` — no new dependency, so the license gate is untouched (still run
  `just deny`).
- `DEC-007` / `DEC-012` — typed errors → exit codes; clap surface in cli.

### Constraints that apply
- `ergonomic-defaults` — `shrink photo.jpg --max-size 200KB` is one short command;
  the budget reads as the outcome.
- `single-image-library` — JPEG encode via the `image` crate; no new lib.
- `decode-once-no-per-op-disk` — the size search re-encodes candidates **in
  memory** (≤ 8); it does not even decode them (cheaper than the perceptual
  search). The original is decoded once by the pipeline.
- `no-unwrap-on-recoverable-paths` — `parse_size` and the search return typed
  errors; no panics. `untrusted-input-hardening` — quality clamped 1..=100; size
  parsing rejects overflow/zero/negative; the iteration cap bounds work.
- `every-public-fn-tested` — `search_jpeg_under_size`, `auto_jpeg_under_size`,
  `SearchConfig::for_size_budget`, `parse_size` all covered.
- `clippy-fmt-clean`, `test-before-implementation`.

### Prior related work
- `SPEC-016` (shipped, PR #18) — the `src/quality` search + the `auto` hook +
  `resolve_effective_quality` + the unmet-target warning this extends. The review
  of SPEC-016 flagged that the search was JPEG/score-specific; the
  `search_threshold` refactor here is the intended generalization.
- `SPEC-013`/`SPEC-014` — `run_shrink`/`run_convert` + the `run_pixel_op` fan-out.
- `SPEC-005` — `encode_to_bytes` (the encode the probe mirrors).

### Out of scope (create a new spec rather than expand)
- **Dimension-reduction fallback** (downscale until the budget fits) — the deferred
  half, a new STAGE-008 backlog item. v1 is quality-only; lossless/infeasible →
  best-effort + warning.
- `--max-size` on AVIF/WebP (SPEC-018/019 — the search already generalizes).
- A `--json` report of the achieved size/quality; a `--strict` error-on-unmet mode.
- Binary (KiB/MiB) as the DEFAULT interpretation — v1 treats `KB`/`MB` as decimal
  (1000-based, matching how people say "200 KB"); `KiB`/`MiB` are the binary opt-in.

## Notes for the Implementer

- **The size search does NOT need SSIMULACRA2 or a decode.** `jpeg_size_at` encodes
  at `q` and returns `bytes.len() as u64`. That is the whole probe. Do not score.
- **Reuse, don't duplicate, the binary search.** Refactor the SPEC-016 loop into
  `search_threshold(probe, cfg, accept, prefer_lower)` and make BOTH
  `search_jpeg_quality` and `search_jpeg_under_size` thin wrappers. The only
  differences are `accept` (`>= target` vs `<= budget`) and `prefer_lower`
  (true vs false) and the best-effort fallback quality (`max_quality` vs
  `min_quality`). Confirm the SPEC-016 `quality::tests` still pass unchanged.
- **`prefer_lower` semantics in the loop:** on `accept(metric)` true, record best
  and move toward the preferred side (`prefer_lower` → `hi = mid - 1`; else `lo =
  mid + 1`); on false, move the other way. Mind the u8 underflow guards at
  `min_quality`/`max_quality` exactly as the shipped loop does.
- **`AutoQuality` is the SPEC-016 `auto` param generalized.** It carries either
  search mode; `resolve_effective_quality` is the one place that dispatches. Keep
  the JPEG-only guard there (non-JPEG → `None` + the appropriate warning). The
  perceptual path and its warning are unchanged; add the size-budget path + its two
  warnings (infeasible budget; lossless format).
- **Warning wording (stderr, unless `--quiet`; name the input):** infeasible →
  `"warning: {label}: could not meet the {budget} budget (smallest is {achieved} at quality {q}); dimension reduction not yet supported"`; lossless format →
  `"warning: {label}: --max-size needs a lossy output format (JPEG); {fmt} is lossless and was left at encoder default"`. A small `fn fmt_bytes(u64) -> String` (e.g. `"6.0 KB"`) makes these readable — unit-test it.
- **Budget feasibility in tests:** pick budgets relative to the fixture's encoded
  size so assertions are robust — a 160×160 `detailed_jpeg` at quality 1 is a few
  KB, so `6KB` is feasible and `200` bytes is infeasible. Assert `metadata().len()
  <= budget` for the feasible case (true by construction when met), and the
  presence of the warning for the infeasible/lossless cases.
- **clap conflicts:** `--target`/`--ssim`/`--max-size` are pairwise exclusive via
  `conflicts_with_all` on `max_size` plus the existing `ssim conflicts_with
  target`. `-q` is a GLOBAL arg (can't use clap `conflicts_with` against a
  subcommand arg) → keep the runtime `auto.is_some() && global.quality.is_some()`
  check (the SPEC-016 pattern), now also in `run_convert`.
- **Derive `Debug`** on `AutoQuality`. Commit incrementally (search refactor +
  size search green → CLI wiring green → integration green). Run `just deny`
  (no new dep, should stay green) as part of the gate set.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-017-max-size-byte-budget-for-shrink-and-convert`
- **PR (if applicable):** #20 https://github.com/jysf/crustyimg/pull/20
- **All acceptance criteria met?** yes — all 17 named tests (5 quality unit, 3 cli
  unit, 9 integration) pass; full suite 238 green; the SPEC-016 `quality::tests`
  stayed green through the `search_threshold` refactor.
- **New decisions emitted:**
  - No new DEC — DEC-019 governs (the byte-budget search is its dual).
- **Deviations from spec:**
  - None functional. The `search_threshold` refactor preserved `search_jpeg_quality`'s
    public signature and all SPEC-016 behavior (verified by the unchanged tests).
  - Process note (not a spec deviation): the build cycle was run by the
    ORCHESTRATOR (Opus) directly, not a fresh Sonnet subagent — the same
    background-subagent-Bash limitation seen on SPEC-016.
- **Follow-up work identified:**
  - The deferred **`--max-size` dimension-reduction fallback** is already a tracked
    STAGE-008 backlog item (makes `--max-size` work for PNG and very-small budgets).
  - When AVIF/WebP land (SPEC-018/019), `resolve_effective_quality`'s JPEG-only
    guard and the `jpeg`-named search entry-points generalize onto them.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing material. The `search_threshold(probe, cfg, accept, prefer_lower)`
   contract and the `prefer_lower` loop semantics were spelled out, so the
   refactor landed with the SPEC-016 tests green on the first run, and the size
   search (which needs no decode/score) fell straight out of it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. Reusing the shipped search + the `auto` hook meant the relevant decisions
   (DEC-019/016/015) were already the right set; no new dependency, so `just deny`
   stayed green untouched.

3. **If you did this task again, what would you do differently?**
   — Nothing notable. Refactoring the shared core FIRST (and re-running the
   SPEC-016 tests before adding the size search) made the "don't regress the
   flagship" risk a non-issue — worth keeping as the order for any
   reuse-by-generalization spec.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
