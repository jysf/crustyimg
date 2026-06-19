# SPEC-029 build prompt — `watermark` (image overlay at a gravity anchor)

Start a **fresh session**. You are the IMPLEMENTER for SPEC-029 in the `crustyimg`
repo. The architect (Opus) wrote the spec + failing tests + DEC-031. This is a
pixel-lane `Operation` — the FIRST that composes a second image. Make the spec's
`## Failing Tests` pass with the smallest correct change, then open a PR and STOP.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-029-watermark-image-overlay-at-gravity-anchor.md`
   — `## Command surface (PINNED)`, `## Operation mechanics (PINNED)`, `## Failing Tests`,
   `## Notes for the Implementer`.
2. `decisions/DEC-031-multi-image-operation-overlay-loaded-at-io-boundary.md` — its
   `## Context` has the verified `image::imageops` probe + the overlay-loading rule.
3. `src/operation/mod.rs` — the `Operation` trait, `OperationParams`, the `Resize` op
   (closest model for a parameterized op using `image::imageops`), `OperationError`.
   You ADD `Gravity` + `Watermark` here.
4. `src/cli/mod.rs` — `run_thumbnail`/`run_pixel_op` (build a `Pipeline`, run fan-out),
   `Commands::Watermark` + its `NotImplemented` arm, `CliError::Usage`, `GlobalArgs`.
5. `src/image/mod.rs` — `Image::{load, width, height, pixels, with_pixels}`.

## What to build (no new dependency)
- `src/operation/mod.rs`:
  - `pub enum Gravity { Center, North, South, East, West, NorthEast, NorthWest, SouthEast, SouthWest }`
    + `FromStr` (parse the 9 lowercase names; junk → an error) + `Display`.
  - `pub struct Watermark { overlay: DynamicImage, overlay_path: String, gravity: Gravity, opacity: f32, scale: Option<f32>, margin: u32, tile: bool }`
    + a `pub fn new(..)` constructor + `impl Operation` (`name`="watermark";
    `params()` emits `image`(path)/`gravity`/`opacity`/`scale`/`margin`/`tile`;
    `apply()` per the spec's `## Operation mechanics`).
  - `apply`: work in RGBA8 (`base.pixels().to_rgba8()`); scale overlay via
    `imageops::resize(.., FilterType::Lanczos3)` so width = `scale × base_width`;
    if opacity<1, multiply overlay alpha; `tile` → step-tile with `imageops::overlay`;
    else compute `(x,y)` from `gravity`+`margin` and `imageops::overlay`. Return
    `base.with_pixels(DynamicImage::ImageRgba8(canvas))`.
- `src/cli/mod.rs`:
  - `run_watermark(inputs, image, gravity, opacity, scale, margin, tile, global)` — the
    IO boundary: `Image::load(image)` (→ exit 3); validate `opacity ∈ [0,1]`,
    `scale > 0`, gravity parse (→ `CliError::Usage`, exit 2) BEFORE constructing the op;
    `Watermark::new(...)`; `Pipeline::new().push(Box::new(op))`;
    `run_pixel_op(pipeline, inputs, global, global.quality, None, None)`.
  - Wire the `Commands::Watermark` dispatch arm. Reuse GLOBAL `-o`/`--out-dir`/`-q`/`-y`.
- Do NOT register `watermark` in `with_builtins()` (DEC-031 — recipe round-trip is
  STAGE-005). Do NOT load any file inside `src/operation/**`.

## Tests — every named test in the spec's `## Failing Tests` must exist + pass
- 8 unit tests in `src/operation/mod.rs`; 5 integration in a new `tests/watermark.rs`.
- Native fixtures (solid `RgbaImage`s); unit tests build `Watermark` with an in-memory
  overlay. **Confirm each named test exists** before claiming green.

## Gates (all must pass — INCLUDING the lean build)
```
cargo fmt && git add -u
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --no-default-features    # the CI 'lean build' gate — run it locally too
cargo deny check licenses            # no new dep — must stay green
```

## Git / PR
- Branch `feat/spec-029-watermark` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked
  `reports/daily|weekly/*.md`.
- PR title: `feat(operation): watermark image overlay at gravity anchor (SPEC-029)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-031, DEC-002, DEC-008,
  DEC-015, DEC-007 / Constraints checked / New decisions — "No new DEC" — DEC-031 was
  emitted at design).
- Fill the spec's `## Build Completion` + 3 reflection answers; append the build cost
  session (numerics null; orchestrator fills at ship).

## Cost
```
- cycle: build
  agent: claude-opus-4-8
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-18
  notes: "watermark: Gravity enum + Watermark Operation (image::imageops overlay/opacity/scale/tile) + run_watermark IO boundary over run_pixel_op; DEC-031; no new dep"
```
(Use the agent id of the session that actually runs the build.)

## When done
`just advance-cycle SPEC-029 verify`, open the PR, and **stop** — the orchestrator
pauses for the user before any merge.
