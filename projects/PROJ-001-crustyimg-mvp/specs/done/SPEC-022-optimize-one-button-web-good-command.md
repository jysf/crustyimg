---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-022
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-009
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build cycle (or orchestrator-direct main loop)
  created_at: 2026-06-17

references:
  decisions: [DEC-024, DEC-019, DEC-017, DEC-016, DEC-015, DEC-003, DEC-002, DEC-023]
  constraints:
    - ergonomic-defaults
    - clippy-fmt-clean
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - pure-rust-codecs-default
    - decode-once-no-per-op-disk
  related_specs: [SPEC-015, SPEC-016, SPEC-017, SPEC-021]

# One sentence on what this spec contributes to its stage's
# value_contribution.
value_link: "Delivers STAGE-009's headline one-button web-good command — the user-facing surface of the STAGE-008 perceptual + modern-format engine."

# Self-reported AI cost per cycle. Each cycle appends one entry to
# sessions[]; totals computed at ship. design/ship are main-loop
# (null-with-note); build/verify carry a real tokens_total.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-17
      notes: "main-loop, not separately metered (orchestrator design session)"
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 220000
      estimated_usd: 2.00
      duration_minutes: null
      recorded_at: 2026-06-17
      notes: "ORDER-OF-MAGNITUDE estimate — main-loop build (not separately metered): read the cli/quality/sink core + wrote run_optimize/optimize_auto_config + 16 tests + ran gates. Opus 4.8 list rate ($5/$25 per MTok), ~80/20 in/out, no cache discount."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 60000
      estimated_usd: 0.55
      duration_minutes: null
      recorded_at: 2026-06-17
      notes: "ORDER-OF-MAGNITUDE estimate — independent read-only Explore verify subagent (spec/DEC/code review + gate runs). Opus list rate, ~80/20 in/out."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-17
      notes: "main-loop, not separately metered (merge + ship bookkeeping)"
  totals:
    tokens_total: 280000
    estimated_usd: 2.55
    session_count: 4
---

# SPEC-022: optimize — one-button web-good command

## Context

STAGE-008 built crustyimg's outcome-driven engine: perceptual auto-quality
(`shrink --target`/`--ssim` via SSIMULACRA2, SPEC-016), the `--max-size` byte
budget with a dimension-reduction fallback (SPEC-017/021), and modern output
formats (AVIF/WebP, SPEC-018/019/020). That engine is powerful but it is spread
across flags on `shrink`/`convert`. The differentiator only becomes *legible* when
a user can run one short command and get the right result.

This spec is the **headline of STAGE-009** (see the stage's
`## Spec Backlog`): the `optimize` command — the "just make this web-good" button.
It bundles the three things a web/content developer almost always wants and almost
always forgets:

1. **Correctness** — bake EXIF orientation into pixels so a phone photo isn't
   served sideways (the `auto-orient` op, SPEC-015/DEC-017).
2. **Privacy** — strip all metadata (including GPS) — which the pixel-lane
   re-encode does inherently (DEC-003).
3. **Quality** — re-encode to a **perceptual visually-lossless target** by default
   (the smallest file with no visible loss), via the SSIMULACRA2 search
   (SPEC-016/DEC-019).

`optimize` is **pure composition** of already-shipped, already-tested primitives
(`src/quality`, the `auto-orient` registry op, `run_pixel_op`,
`resolve_effective_quality`). It adds **no new dependency** and **no new pixel or
search machinery** — only a clap subcommand, a thin handler, a default policy, and
one command-shape decision (**DEC-024**).

## Goal

Add a `crustyimg optimize <inputs…>` command that, with no flags, auto-orients each
input, strips its metadata, and re-encodes it to a perceptual *visually-lossless*
target in its original format — and that accepts `--target`/`--ssim`/`--max-size`
to override the outcome, `--max N` to optionally bound the long edge, and
`-o`/`--out-dir`/`--format` for output, reusing the shipped fan-out unchanged.

## Inputs

- **Files to read:** `src/cli/mod.rs` — the whole command surface; in particular
  `run_shrink` (the closest sibling), `run_pixel_op`, `resolve_effective_quality`,
  `shrink_auto_config`, `reject_quality_with_auto`, `parse_size`, `shrink_params`,
  the `Commands` enum, `dispatch`, `QualityTarget`, `AutoQuality`.
- **Related code paths:** `src/quality/mod.rs` (`SearchConfig::for_target`,
  `AutoQuality` is consumed by `resolve_effective_quality`); the `auto-orient` op
  in `src/operation/` (registered as `"auto-orient"`); `src/sink/mod.rs`
  (`encode_to_bytes` — unchanged).
- **Decisions:** DEC-024 (this spec's command-shape DEC — author it during build if
  not already present), DEC-019, DEC-017, DEC-016, DEC-015, DEC-003.
- **Tests to mirror:** `tests/cli.rs` — `auto_orient_cli_rotates_and_clears_tag`
  (orientation→dims+tag), `shrink_target_visually_lossless_produces_valid_jpeg`,
  `shrink_max_bounds_long_edge`, `shrink_quality_and_target_conflict_exits_2`,
  `convert_png_max_size_downscales`, `shrink_multi_input_fan_out_preserves_format`.
  Fixtures in `tests/common/mod.rs`: `jpeg_with_orientation`, `detailed_jpeg`,
  `gradient_jpeg`, `solid_png`.

## Outputs

- **Files modified:**
  - `src/cli/mod.rs` — add a `Commands::Optimize { … }` variant, a `dispatch` arm,
    the `run_optimize` handler, the `optimize_auto_config` helper, and unit tests.
  - `decisions/DEC-024-optimize-command-shape.md` — the new decision (see below).
  - `tests/cli.rs` — `optimize_*` integration tests.
  - `projects/PROJ-001-crustyimg-mvp/stages/STAGE-009-…md` — flip the SPEC-022
    backlog line on build completion (the orchestrator/ship handles status marks).
- **New exports:** none outside `src/cli` (the handler + helper are module-private,
  consistent with `run_shrink`/`shrink_auto_config`).
- **No new dependency. No database changes.**

## Command surface (PINNED)

```
crustyimg optimize <inputs…>
    [--max <N>]                       # optional long-edge bound (no resize by default)
    [--target <preset> | --ssim <score> | --max-size <SIZE>]   # outcome override
    # global: -o <PATH> | --out-dir <DIR> | --format <FMT> | --quiet | --yes | -j
```

- `--target <preset>` reuses the existing `QualityTarget` value-enum
  (`visually-lossless` | `high` | `medium`).
- `--ssim <score>` is an `f64` in `0..=100`.
- `--max-size <SIZE>` reuses `parse_size` (`200KB`, `1.5MB`, `512B`, `KiB`/`MiB`…).
- `--target`/`--ssim`/`--max-size` are **mutually exclusive** (clap
  `conflicts_with`/`conflicts_with_all`, mirroring `Shrink`).
- `--max <N>` is independent and may combine with any outcome mode.

### Default behavior (no outcome flag)

`optimize` is **always** in an auto-quality mode — the default is
`AutoQuality::Perceptual(SearchConfig::for_target(90.0))`, i.e.
`QualityTarget::VisuallyLossless`. Because auto is always on, a fixed `-q/--quality`
is a usage error (reuse `reject_quality_with_auto`, exit 2).

### Pipeline (PINNED order)

1. `auto-orient` op (always) — bake orientation, then it drops the metadata bundle
   (DEC-017).
2. **iff `--max N`:** a `resize` op in `mode=max` (long-edge bound N, no upscale) —
   built exactly like `shrink_params(N)`. Placed AFTER auto-orient so the bound
   applies to the visually-correct dimensions.

Then delegate to `run_pixel_op(pipeline, inputs, global, /*fixed_quality*/ None,
/*forced_format*/ None, /*auto*/ Some(mode))`. `forced_format = None` means the
output format is resolved per-input by `output_format_for` (DEC-015): `--format`
wins, else the `-o` extension, else the **input's source format is preserved**.

## Acceptance Criteria

- [ ] `crustyimg optimize photo.jpg -o out.jpg` (no other flags) produces a **valid
  JPEG**, **smaller** than a `-q 100` encode of the same input, with **dimensions
  preserved** (no resize) and **no EXIF** in the output.
- [ ] An input JPEG with EXIF orientation 6 is **reoriented** (output dimensions
  swapped W↔H, matching `auto_orient_cli_rotates_and_clears_tag`) AND its metadata
  is **stripped** (`info --exif` on the output reports `exif: no`).
- [ ] `optimize` defaults to the **visually-lossless** perceptual target: the
  resolved `AutoQuality` with no flags is `Perceptual` with `target == 90.0`.
- [ ] `--target high` → `Perceptual(70.0)`; `--target medium` → `Perceptual(50.0)`;
  `--ssim 85` → `Perceptual(85.0)`; `--ssim 150` → usage error (exit 2).
- [ ] `--max-size 8KB` → `SizeBudget(8000)`; the output is **≤ the budget** (or a
  reported best-effort), reusing the SPEC-017/021 fit.
- [ ] `--max 50` on a 100×60 input bounds the long edge to ≤ 50 (50×30).
- [ ] `--format png` (or `-o out.png`) writes **PNG**; the perceptual target is
  silently ignored for the lossless format (no error), mirroring `-q` on PNG.
- [ ] A fixed `-q 80` with `optimize` is a **usage error (exit 2)**.
- [ ] Multi-input `optimize a.jpg b.jpg --out-dir d/` writes both; a missing input
  exits 3; multi-input without `--out-dir` is a usage error (exit 2) — all inherited
  via `run_pixel_op` (DEC-015).
- [ ] `--target`/`--ssim`/`--max-size` are mutually exclusive (exit 2 when combined).
- [ ] `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and
  `cargo test` pass; `cargo deny check licenses` stays green (no new dep).

## Failing Tests

Written during **design**, BEFORE build. The implementer makes these pass.

### Unit tests — `src/cli/mod.rs` (`#[cfg(test)] mod tests`)

- **`optimize_parses_args`** — `Cli::try_parse_from(["crustyimg","optimize",
  "a.jpg","--max","800","-o","out.jpg"])` parses to `Commands::Optimize` with
  `inputs == ["a.jpg"]`, `max == Some(800)`, and `cli.global.output == Some("out.jpg")`.
- **`optimize_default_auto_is_visually_lossless`** — `optimize_auto_config(None,
  None, None)` returns `AutoQuality::Perceptual(cfg)` with `cfg.target == 90.0`.
- **`optimize_target_preset_sets_score`** — `optimize_auto_config(Some(QualityTarget::High),
  None, None)` → `Perceptual` with `target == 70.0`; `QualityTarget::Medium` →
  `50.0`; `QualityTarget::VisuallyLossless` → `90.0`.
- **`optimize_ssim_sets_and_validates`** — `optimize_auto_config(None, Some(85.0),
  None)` → `Perceptual(85.0)`; `optimize_auto_config(None, Some(150.0), None)` and
  `Some(-1.0)` each return `Err` with `code() == 2`.
- **`optimize_max_size_is_size_budget`** — `optimize_auto_config(None, None,
  Some("200KB"))` → `AutoQuality::SizeBudget(200_000)`.
- **`optimize_conflicting_modes_are_usage`** — `optimize_auto_config(Some(QualityTarget::High),
  None, Some("8KB"))` (and other multi-Some combos) → `Err` with `code() == 2`
  (defensive arm behind clap's `conflicts_with`).

### Integration tests — `tests/cli.rs`

- **`optimize_reorients_and_strips_metadata`** — write `jpeg_with_orientation(40,
  20, 6)` to a temp input; run `optimize <in> -o <out.jpg> --yes`; assert exit 0,
  the output decodes as JPEG with **swapped** dimensions (20×40, per
  `auto_orient_cli_rotates_and_clears_tag`), and `info --exif <out.jpg>` reports
  `exif: no` (metadata stripped).
- **`optimize_default_is_smaller_valid_jpeg`** — input `detailed_jpeg(96, 96)`; run
  `optimize <in> -o <out.jpg>`; assert exit 0, the output is a valid JPEG, its byte
  length is **less than** a `convert <in> --format jpeg -q 100` (or `shrink … -q
  100`) baseline of the same input, and dimensions are unchanged (96×96).
- **`optimize_preserves_format_and_dims_by_default`** — input `gradient_jpeg(100,
  60)`; `optimize <in> -o <out.jpg>` → output is JPEG and **100×60** (no resize, no
  format change).
- **`optimize_max_bounds_long_edge`** — input 100×60; `optimize --max 50 <in> -o
  <out.jpg>` → long edge ≤ 50 (50×30).
- **`optimize_format_change_to_png`** — `optimize <in.jpg> -o <out.png>` →
  output is PNG, exit 0 (perceptual target ignored for lossless, no error).
- **`optimize_max_size_fits_budget`** — input `detailed_jpeg(128,128)`; pick a
  budget below the full-size `-q 100` encode; `optimize <in> --max-size <budget>
  -o <out.jpg>` → output byte length ≤ budget (mirror `convert_png_max_size_downscales`
  / `max_size_keeps_dims_when_it_fits`).
- **`optimize_quality_flag_is_usage_error`** — `optimize <in> -q 80 -o <out.jpg>`
  exits **2**.
- **`optimize_multi_input_fan_out`** — two inputs + `--out-dir d/` → both outputs
  written, exit 0.
- **`optimize_missing_input_exits_3`** — `optimize does_not_exist.jpg -o <out>` →
  exit 3.
- **`optimize_multi_without_out_dir_is_usage_error`** — two inputs, no `--out-dir`
  → exit 2.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- **DEC-024 (NEW — author it)** — the `optimize` command shape: default perceptual
  visually-lossless, auto-orient + metadata-strip folded in, format/size-preserving,
  and the **explicit deferral of cross-format auto-negotiation**. Use the
  `decisions/_template.md`; set `affected_scope` to `src/cli/mod.rs`; confidence
  ~0.85; since < 0.9 is fine, no question entry required, but if you lower it below
  0.7 add a `guidance/questions.yaml` entry (AGENTS.md §17).
- **DEC-019** — perceptual auto-quality (SSIMULACRA2): `SearchConfig::for_target`,
  `AutoQuality::Perceptual`, the `met_target` best-effort warning. `optimize` reuses
  this unchanged; the visually-lossless score is 90.0 (the `QualityTarget::VisuallyLossless`
  anchor).
- **DEC-017** — the `auto-orient` op bakes orientation then drops the metadata
  bundle. This is what gives `optimize` both its correctness AND its metadata strip.
- **DEC-016** — quality policy: `-q` ignored for lossless formats; the perceptual
  search and `-q` are mutually exclusive (`reject_quality_with_auto`).
- **DEC-015** — output-format precedence (`--format` > `-o` ext > preserve source)
  and partial-batch exit 6; both come for free via `run_pixel_op` with
  `forced_format = None`.
- **DEC-003** — the metadata dual-lane. IMPORTANT distinction: `optimize`'s metadata
  removal is the **pixel-lane re-encode dropping everything**, NOT the
  selective-preserve container lane (keep orientation/ICC/copyright, drop only GPS),
  which is unbuilt (STAGE-004). Do not claim selective preservation. The global
  `--keep-gps` flag is a no-op today (container lane not built) — do not wire new
  behavior to it in this spec.
- **DEC-002** — decode-once: the pipeline decodes once; the perceptual search
  re-encodes/decodes candidates in memory only.
- **DEC-023** — `--max-size` quality-then-dimension fallback (reused via
  `resolve_effective_quality`'s `SizeBudget` arm → `fit_under_size`).

### Constraints that apply

- `ergonomic-defaults` — the no-flag case must be the right one (this command's
  entire reason to exist).
- `clippy-fmt-clean` — `--all-targets`, warnings as errors; **re-add every file
  `cargo fmt` touches before committing** (the known CI trap — run `cargo fmt`
  then `git add -u` before the final commit).
- `no-unwrap-on-recoverable-paths` — typed errors only in library/CLI paths;
  `unwrap` only in `#[cfg(test)]`.
- `every-public-fn-tested` — covered by the unit + integration tests above.
- `pure-rust-codecs-default`, `decode-once-no-per-op-disk` — unchanged; no new dep.

### Prior related work

- `SPEC-015` (shipped) — the `auto-orient` op + `run_auto_orient` (the orientation
  bake + metadata drop `optimize` folds in).
- `SPEC-016` (shipped, PR #18) — perceptual auto-quality + `run_shrink`'s
  `--target`/`--ssim` wiring (the closest analog to copy).
- `SPEC-017`/`SPEC-021` (shipped, PRs #20/#24) — `--max-size` + the dimension
  fallback (reused unchanged via `resolve_effective_quality`).

### Out of scope (for this spec specifically)

- **Cross-format auto-negotiation** (try JPEG/WebP/AVIF, pick the smallest). v1
  `optimize` preserves the input format unless `--format`/`-o` picks one. Recorded
  as deferred in DEC-024; it needs AVIF decode (to perceptually score AVIF) and is
  its own spec.
- **Selective metadata preservation** (DEC-003 container lane). `optimize` strips
  everything via the pixel re-encode. STAGE-004 work.
- **A default resize.** `optimize` does NOT resize unless `--max` is given.
- **`--json`/output reporting** of what optimize did (a nice follow-up; not here).
- **Animation, responsive sets, `diff`** — sibling STAGE-009 specs, not this one.

## Notes for the Implementer

- **This is composition, not invention.** `run_optimize` should be a near-twin of
  `run_shrink`, with three differences: (1) the default auto mode is
  `Perceptual(visually-lossless)` instead of `None`; (2) the pipeline leads with the
  `auto-orient` op; (3) there is no default resize (only an optional `--max`). Do
  NOT add new functions to `src/quality` or `src/sink`.
- **`optimize_auto_config` shape** (mirror `shrink_auto_config`, but it always
  returns a mode):
  ```rust
  fn optimize_auto_config(
      target: Option<QualityTarget>,
      ssim: Option<f64>,
      max_size: Option<&str>,
  ) -> Result<AutoQuality, CliError> {
      match (target, ssim, max_size) {
          (None, None, None) =>
              Ok(AutoQuality::Perceptual(SearchConfig::for_target(
                  QualityTarget::VisuallyLossless.target_score()))),
          (Some(t), None, None) =>
              Ok(AutoQuality::Perceptual(SearchConfig::for_target(t.target_score()))),
          (None, Some(s), None) => {
              if !(0.0..=100.0).contains(&s) {
                  return Err(CliError::Usage(format!(
                      "--ssim must be a score in 0..=100, got {s}")));
              }
              Ok(AutoQuality::Perceptual(SearchConfig::for_target(s)))
          }
          (None, None, Some(sz)) => Ok(AutoQuality::SizeBudget(parse_size(sz)?)),
          _ => Err(CliError::Usage(
              "--target/--ssim/--max-size are mutually exclusive".into())),
      }
  }
  ```
  `QualityTarget::target_score` is currently private; `optimize` is in the same
  module, so this is fine. (Optionally bump it to `pub(crate)` if cleaner.)
- **`run_optimize` shape:**
  ```rust
  fn run_optimize(inputs, max, target, ssim, max_size, global) -> Result<(), CliError> {
      let auto = Some(optimize_auto_config(target, ssim, max_size)?);
      reject_quality_with_auto(&auto, global)?;          // -q + optimize → exit 2
      let orient = OperationRegistry::with_builtins()
          .build("auto-orient", &OperationParams::empty())
          .map_err(/* same RegistryError→Usage map as run_shrink */)?;
      let mut pipeline = Pipeline::new().push(orient);
      if let Some(n) = max {
          let resize = OperationRegistry::with_builtins()
              .build("resize", &shrink_params(n))   // reuse: mode=max, width=n
              .map_err(/* … */)?;
          pipeline = pipeline.push(resize);
      }
      run_pixel_op(pipeline, inputs, global, None, None, auto)  // fixed_quality None, no forced fmt
  }
  ```
- **clap variant** (mirror `Shrink`'s attributes for the conflicts):
  ```rust
  /// One-button web-good: auto-orient + strip metadata + perceptual re-encode
  /// (visually-lossless by default), format/size-preserving (STAGE-009, DEC-024).
  Optimize {
      inputs: Vec<String>,
      #[arg(long)]
      max: Option<u32>,
      #[arg(long, value_enum)]
      target: Option<QualityTarget>,
      #[arg(long, conflicts_with = "target")]
      ssim: Option<f64>,
      #[arg(long, value_name = "SIZE", conflicts_with_all = ["target", "ssim"])]
      max_size: Option<String>,
  },
  ```
  Add the matching `dispatch` arm calling `run_optimize(inputs, *max, *target,
  *ssim, max_size.as_deref(), &cli.global)`.
- **Reorientation test dims:** confirm against `auto_orient_cli_rotates_and_clears_tag`
  — orientation 6 is a 90° rotation, so a 40×20 input becomes 20×40. Match its
  exact expectation rather than re-deriving.
- **Confirm every named failing test exists before claiming green** (the standing
  build discipline: diff this `## Failing Tests` list against the test files).
- **Cost:** append a build session entry to `cost.sessions` with a real
  `tokens_total` (or a labeled order-of-magnitude estimate if run in the main loop),
  per AGENTS.md §4.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-022-optimize`
- **PR (if applicable):** (opened during build; number recorded in the timeline)
- **All acceptance criteria met?** yes — all 10 acceptance criteria covered by the
  16 new tests (6 unit + 10 integration); `cargo build`, `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test` (0 failed), and
  `cargo deny check licenses` all green.
- **New decisions emitted:**
  - `DEC-024` — optimize command shape (default perceptual visually-lossless +
    auto-orient + strip; format/size-preserving; auto-negotiation deferred)
    *(authored during design, on `main`)*
- **Deviations from spec:** none. Built exactly to the pinned command surface and
  pipeline order; pure composition (`auto-orient` op + optional `resize max` +
  `run_pixel_op` with `auto = Some(Perceptual(visually-lossless))` by default). No
  new dependency, no change to `src/quality` or `src/sink`.
- **Follow-up work identified:**
  - Cross-format auto-negotiation (deferred per DEC-024; needs AVIF decode) — a
    future STAGE-009 spec.
  - `optimize` could gain `--json` reporting (what it did: in/out bytes, chosen
    quality, score) — fits a broader `--json` everywhere effort.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing material. The spec pinned the `optimize_auto_config`/`run_optimize`
   shapes, the pipeline order, and the exact reuse points, so the build was
   mechanical. The one judgement call made explicit by the design — keeping every
   perceptually-scored fixture ≥ 96px min dimension (SSIMULACRA2's proven-safe
   floor in the existing tests) — was already flagged and saved a guessing round.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-019/017/016/015 plus the cross-sync note covered the reuse surface.
   Worth recording for the next command: the perceptual default means any fixture
   that the pipeline downscales (`--max`) must still land ≥ ~96px or the metric can
   error — the spec anticipated this but it's a standing gotcha for `diff`/responsive.
3. **If you did this task again, what would you do differently?**
   — Nothing of substance. Possibly compute the q100 byte baselines once in a shared
   test helper rather than inline in two tests, but inlining kept each test
   self-contained, which matches the file's style.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused.*

1. **What would I do differently next time?**
   — Very little. This validated the STAGE-009 thesis that the differentiator
   surface is mostly composition: a new headline command landed as a clap variant +
   one thin handler + one config helper, reusing the entire STAGE-008 engine with no
   new dependency and no change to `src/quality`/`src/sink`. The only judgement that
   needed pinning up front was keeping every perceptually-scored image ≥ ~96px (the
   SSIMULACRA2 floor) — worth front-loading again for `diff`/responsive.
2. **Does any template, constraint, or decision need updating?**
   — No template/constraint change. DEC-024 captures the command shape and the
   deferral of cross-format auto-negotiation. One standing note for the stage: the
   "perceptual default" means any command that downscales before scoring inherits
   the ≥96px metric floor — fold this into the `diff`/responsive specs rather than
   rediscovering it.
3. **Is there a follow-up spec I should write now before I forget?**
   — Two, already in the STAGE-009 backlog as future specs: (a) cross-format
   auto-negotiation for `optimize` (blocked on AVIF decode, per DEC-024); (b) `--json`
   reporting (in/out bytes, chosen quality, score) — a natural `optimize` add and a
   broader "machine-readable everywhere" thread. Neither blocks the next STAGE-009
   spec (`diff`).
   — <answer>
