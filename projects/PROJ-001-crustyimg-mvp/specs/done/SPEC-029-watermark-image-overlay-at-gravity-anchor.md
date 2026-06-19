---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-029
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-004
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # usually same Claude, different session
  created_at: 2026-06-18

references:
  decisions: [DEC-031, DEC-002, DEC-008, DEC-015, DEC-007]
  constraints:
    - clippy-fmt-clean
    - every-public-fn-tested
    - no-unwrap-on-recoverable-paths
    - no-new-top-level-deps-without-decision
  related_specs: [SPEC-010, SPEC-013]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-004's <capability>". Optional; null is acceptable.
value_link: >
  Adds `watermark` — the image-overlay compositing command and the first
  multi-image Operation — the compositing half of STAGE-004.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-18
      notes: >
        Main-loop orchestrator work, not separately metered. Authored the spec
        (Failing Tests + Implementation Context); emitted DEC-031 (multi-image
        Operation overlay loaded at the IO boundary); ran a design-time probe
        confirming image::imageops overlay/alpha-opacity/resize/clip primitives
        (no new dep). pixel-lane compositing op.
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 149046
      estimated_usd: 1.34
      duration_minutes: 12
      recorded_at: 2026-06-18
      notes: >
        Real metered subagent (foreground Agent; subagent_tokens=149046,
        duration_ms=746884). estimated_usd at Opus 4.8 list (~80/20 in/out) —
        order-of-magnitude. watermark: Gravity enum (FromStr/Display + placement
        math) + Watermark Operation (image::imageops overlay/opacity/scale/tile,
        RGBA8) + run_watermark IO boundary over run_pixel_op; DEC-031; no new dep.
        13 named tests (8 unit + 5 integration) green; clippy/fmt/lean-build/deny
        clean. Repointed a stale NotImplemented stub test from watermark→edit.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 50000
      estimated_usd: 0.45
      duration_minutes: null
      recorded_at: 2026-06-18
      notes: >
        ORDER-OF-MAGNITUDE ESTIMATE (~50k) — read-only Explore subagent (no
        metered usage block) + orchestrator main-loop gate re-runs (cargo test
        343 ok / clippy / fmt / deny / cargo build --no-default-features lean).
        Explore verdict: APPROVED, no concerns; confirmed the DEC-031 file-IO
        boundary (no file IO in operation/, not in with_builtins) + compositing.
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-18
      notes: "Main-loop ship bookkeeping (merge dance + cost totals + reflection + archive); not separately metered."
  totals:
    tokens_total: 199046
    estimated_usd: 1.79
    session_count: 4
---

# SPEC-029: `watermark` — image overlay at a gravity anchor

## Context

The **compositing** half of STAGE-004 and the last spec before the stage's
metadata lane is joined by its pixel-lane sibling. `watermark <inputs…> --image
LOGO` overlays a logo image onto each base at a compass **gravity** anchor, with
`--opacity`/`--scale`/`--margin`/`--tile` controls. It is a **pixel-lane
`Operation`** (DEC-002) — and the **first one that composes a second image**,
which is why it needs **DEC-031**: the overlay is loaded at the IO boundary
(`run_watermark` in `src/cli/`) and handed to the op as in-memory pixels, so
`apply()` never touches files (AGENTS.md §11 — `src/operation/**` is file-free).

A design-time probe confirmed the compositing needs **no new dependency**:
`image::imageops::overlay` (source-over alpha), alpha-scaling for `--opacity`, and
`image::imageops::resize` for `--scale` cover everything (probe results in DEC-031).

`watermark` exists today only as a clap stub (`Commands::Watermark { inputs, image,
gravity, opacity, scale, margin, tile }`) returning `CliError::NotImplemented`.

Parent: `STAGE-004-compose-and-metadata`. Governing: **DEC-002** (`Operation`
extension point), **DEC-031** (multi-image overlay boundary). Text watermark
(`ab_glyph`/`imageproc`) is a **separate** later spec.

## Goal

Wire `watermark <inputs…> --image LOGO [--gravity G] [--opacity O] [--scale S]
[--margin M] [--tile]` as a pixel-lane `Operation` that composites the overlay onto
each base at the gravity anchor, using `image::imageops` only (no new dep), through
the standard `run_pixel_op` fan-out.

## Inputs

- **Files to read:**
  - `src/operation/mod.rs` — the `Operation` trait, `OperationParams`
    (`get_str`/`get_u32`/`get_f32`), the `Resize` op (model for a parameterized op +
    `apply` using `image::imageops`), `OperationError::Apply`. ADD `Gravity` +
    `Watermark` here.
  - `src/cli/mod.rs` — `Commands::Watermark` (declared), its `NotImplemented` arm,
    `run_thumbnail`/`run_pixel_op` (build a `Pipeline`, run the fan-out), `GlobalArgs`,
    `CliError::Usage`.
  - `src/image/mod.rs` — `Image::{load, width, height, pixels, with_pixels}`.
  - `decisions/DEC-031-*.md` (the probe + the overlay-loading boundary).
- **External crate (already a dep):** `image` 0.25 — `imageops::{overlay, resize}`,
  `RgbaImage`, `DynamicImage`, `FilterType`. No new dependency (DEC-031).
- **Related code paths:** `src/operation/mod.rs`, `src/cli/mod.rs`, `tests/`.

## Outputs

- **Files modified:**
  - `src/operation/mod.rs` — add:
    - `pub enum Gravity { Center, North, South, East, West, NorthEast, NorthWest, SouthEast, SouthWest }`
      with `FromStr`/parse + `Display` (string round-trip for params).
    - `pub struct Watermark { overlay: DynamicImage, overlay_path: String, gravity: Gravity, opacity: f32, scale: Option<f32>, margin: u32, tile: bool }`
      + `impl Operation` (`name`="watermark"; `params()` serializes path+placement;
      `apply()` composites).
    - A `pub fn` constructor (e.g. `Watermark::new(...)`) so the CLI builds it.
  - `src/cli/mod.rs` — `run_watermark(inputs, image, gravity, opacity, scale, margin,
    tile, global)`; wire the `Commands::Watermark` dispatch arm.
  - `docs/api-contract.md` — flesh out the `watermark` entry (done at design).
- **New exports:** `crate::operation::{Gravity, Watermark}`.
- **Database changes:** none.

## Command surface (PINNED)

```
crustyimg watermark <INPUTS...> --image LOGO [--gravity G] [--opacity O]
                                [--scale S] [--margin M] [--tile]
```

- **`--image LOGO`** (required) — the overlay file. Loaded once by `run_watermark`
  via `Image::load`; a missing/unreadable/undecodable logo → exit **3** (load error).
- **`--gravity G`** (default **`southeast`**) — compass anchor; one of
  `center north south east west northeast northwest southeast southwest`. Unknown →
  `CliError::Usage` (exit **2**).
- **`--opacity O`** (default **1.0**) — overlay alpha multiplier in **[0.0, 1.0]**;
  outside → exit **2**. Implemented by multiplying the overlay's alpha channel by O.
- **`--scale S`** (optional; default = overlay native size) — resize the overlay so
  its **width = S × base width** (aspect preserved). `S ≤ 0` → exit **2**.
- **`--margin M`** (u32, default **0**) — inset in pixels from the anchored edges
  (ignored for `center` and when `--tile`).
- **`--tile`** — repeat the (scaled, opacity-adjusted) overlay to cover the whole
  base; **ignores `--gravity` and `--margin`**.
- **Compositing:** convert base to RGBA, alpha-composite the overlay with
  `image::imageops::overlay` (source-over); out-of-bounds placement clips cleanly
  (probe-verified, no panic). It is a normal pixel-lane op — output format/quality and
  the fan-out are handled by `run_pixel_op` (DEC-015): single → stdout/`-o`/`--out-dir`,
  multi → `--out-dir`, per-input failure → exit 6.

## Operation mechanics (PINNED — probe-verified)

`Watermark::apply(base)`:
1. `let mut canvas = base.pixels().to_rgba8();` (work in RGBA8).
2. Prepare the overlay RGBA: `let mut ov = self.overlay.to_rgba8();`
   - if `Some(s) = self.scale`: target width `w = (canvas.width() as f32 * s).round()`,
     height scaled to preserve the overlay aspect; `ov = imageops::resize(&ov, w, h,
     FilterType::Lanczos3)` (`w`/`h` ≥ 1).
   - if `self.opacity < 1.0`: for each pixel `p.0[3] = (p.0[3] as f32 *
     self.opacity).round() as u8`.
3. Placement:
   - `tile`: for `y in (0..canvas.height()).step_by(ov.height())` and `x` similarly,
     `imageops::overlay(&mut canvas, &ov, x as i64, y as i64)` (edge tiles clip).
   - else: compute `(x, y)` from `gravity` over `(canvas.w/h, ov.w/h, margin)` — e.g.
     SE = `(W-ow-m, H-oh-m)`, center = `((W-ow)/2, (H-oh)/2)`, N = `((W-ow)/2, m)`,
     etc. (saturating so anchors stay ≥ 0); `imageops::overlay(&mut canvas, &ov, x, y)`.
4. `Ok(base.with_pixels(DynamicImage::ImageRgba8(canvas)))`. Validation failures (bad
   opacity/scale) are caught in the CLI before constructing the op, so `apply` only
   returns `OperationError::Apply` on genuinely unexpected failures.

## Acceptance Criteria

- [ ] `watermark base --image logo --gravity southeast` places the overlay in the
  bottom-right; far corner pixels match the overlay, opposite corner unchanged.
- [ ] `--opacity 0.5` blends (the overlaid region is a base/overlay mix, not the pure
  overlay color).
- [ ] `--scale 0.5` resizes the overlay to ~half the base width before compositing.
- [ ] `--margin M` offsets the anchor inward by M px.
- [ ] `--tile` repeats the overlay across the whole base (overlay color appears in
  multiple, far-apart regions); `--gravity`/`--margin` ignored.
- [ ] `--gravity center` centers the overlay.
- [ ] Missing/unreadable `--image` → exit **3**; bad `--opacity` (e.g. 2.0) or
  `--scale 0` or unknown `--gravity` → exit **2**.
- [ ] Multi-input fan-out: two bases + `--out-dir` → two composited outputs; overlay
  loaded once.
- [ ] No new dependency (`cargo deny` green); op lives in `src/operation/`, the file
  load only in `src/cli/` (DEC-031).

## Failing Tests

Written during **design**, BEFORE build. Native fixtures (solid-color `RgbaImage`s
via the `image` crate); no ImageMagick. Unit tests construct `Watermark` directly with
an in-memory overlay `DynamicImage`.

- **`src/operation/mod.rs` (unit, `#[cfg(test)] mod tests`)**
  - `"gravity_parse_round_trips"` — `Gravity::from_str` accepts the 9 names (+ rejects
    junk → error); `to_string` round-trips.
  - `"watermark_southeast_places_overlay"` — base 20×20 red, overlay 4×4 blue, SE,
    margin 0 → pixel (18,18) is blue, (0,0) is red.
  - `"watermark_center_places_overlay"` — overlay centered; middle pixel is overlay,
    corners are base.
  - `"watermark_opacity_blends"` — opacity 0.5 → overlaid pixel is a red/blue blend
    (neither pure base nor pure overlay).
  - `"watermark_scale_resizes_overlay"` — scale 0.5 on a 20-wide base → the overlaid
    region spans ~10 px wide (overlay color present across ~half-width, not 4 px).
  - `"watermark_margin_offsets_anchor"` — SE margin 2 → overlay shifted 2 px inward
    (corner pixel reverts to base; the inset block is overlay).
  - `"watermark_tile_covers_base"` — tile → overlay color appears in both the
    top-left and bottom-right regions.
  - `"watermark_params_includes_path_and_placement"` — `params()` emits `image`
    (path), `gravity`, `opacity`, `scale`, `margin`, `tile`.
- **`tests/watermark.rs` (integration, drives the binary)**
  - `"watermark_writes_composited_output"` — `watermark base.png --image logo.png -o
    out.png` → exit 0; out decodes and differs from base.
  - `"watermark_missing_image_exits_3"` — `--image nonexistent.png` → exit 3.
  - `"watermark_bad_opacity_exits_2"` — `--opacity 2.0` → exit 2.
  - `"watermark_unknown_gravity_exits_2"` — `--gravity sideways` → exit 2.
  - `"watermark_multi_input_fanout"` — two bases + `--out-dir` → two outputs; exit 0.
- **`tests/cli.rs`** — `"watermark"` is already in the subcommand-list tests
  (SPEC-007); confirm, add only if missing.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-031` — **the key one.** Overlay loaded at the IO boundary (`run_watermark`),
  op holds in-memory pixels, `apply()` is file-free, watermark NOT in
  `with_builtins()` (recipe round-trip is STAGE-005). Mirror its probe.
- `DEC-002` — `watermark` is a new `Operation` impl (the pixel extension point); do
  not route it through the metadata lane.
- `DEC-008` — the SIMD resize backend; `--scale` may reuse it or `image::imageops::
  resize` (Lanczos3) — overlay scaling is not perf-critical, either is fine.
- `DEC-015` — fan-out/exit semantics come free from `run_pixel_op`.
- `DEC-007` — typed errors; CLI validates opacity/scale/gravity (→ `Usage`, exit 2);
  no `unwrap`/`expect`/`panic!` off test paths.

### Constraints that apply

- `src/operation/**` must not depend on files/clap/terminals (AGENTS.md §11) —
  enforced by DEC-031 (load in CLI, not in the op).
- `no-new-top-level-deps-without-decision` — none added (compositing via `image`).
- `clippy-fmt-clean`, `every-public-fn-tested`, `no-unwrap-on-recoverable-paths`.

### Prior related work

- `SPEC-010` (resize) — the closest model: a parameterized `Operation` whose `apply`
  uses `image::imageops`; `run_thumbnail` builds a `Pipeline` + calls `run_pixel_op`.
- `SPEC-013` (auto-orient) — another op + its CLI handler.

### Out of scope (for this spec specifically)

- **Text watermark** (`ab_glyph` + `imageproc::drawing`) — a separate STAGE-004 spec
  with its own font-dependency DEC.
- **Recipe round-trip** of `watermark` (registering it in `with_builtins`, resolving
  the overlay path in the recipe loader) — STAGE-005 (DEC-031 defers this).
- Blend modes beyond source-over, rotation of the overlay, caption/shapes/borders,
  montage/append — post-MVP (docs/backlog.md).

## Notes for the Implementer

- **Mirror DEC-031's probe** — `imageops::overlay(&mut base_rgba, &overlay_rgba, x:
  i64, y: i64)` is source-over alpha; out-of-bounds clips (no panic). Opacity =
  multiply overlay alpha; scale = `imageops::resize`.
- **`run_watermark` is the IO boundary** — `Image::load(image)` (→ exit 3 on
  failure), validate `opacity ∈ [0,1]` / `scale > 0` / gravity parse (→ `Usage`, exit
  2) BEFORE constructing the op, then `Watermark::new(overlay_dynimg, path, gravity,
  opacity, scale, margin, tile)`, `Pipeline::new().push(op)`,
  `run_pixel_op(pipeline, inputs, global, global.quality, None, None)`.
- Reuse the **global** `--out-dir`/`-q`/`-y`/`-o` flags — do NOT declare locals (the
  SPEC-024 collision lesson).
- Put `watermark` integration tests in a new `tests/watermark.rs`; keep diagnostics on
  stderr.
- `Gravity` belongs in `src/operation/` (placement math + future `crop` reuse, AGENTS.md
  §14); give it `FromStr` (parse errors map to `CliError::Usage` at the CLI).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-029-watermark`
- **PR (if applicable):** `feat(operation): watermark image overlay at gravity anchor (SPEC-029)`
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - None — DEC-031 was emitted at design and covers the multi-image boundary.
- **Deviations from spec:**
  - None functional. One adjustment to a pre-existing test: `tests/cli.rs`'s
    `stub_command_returns_not_implemented` was pointed at `watermark` as its
    sample stub; since `watermark` is now real, it was repointed to `edit`
    (still a STAGE-005 stub). `tests/cli.rs` subcommand-list tests already
    listed `watermark` and stayed valid.
- **Follow-up work identified:**
  - STAGE-005: wire the recipe loader to resolve `watermark`'s overlay path so
    it round-trips (the deferred half of DEC-031) and register it in
    `with_builtins()`.
  - Text watermark (`ab_glyph` + `imageproc::drawing`) — a separate STAGE-004
    spec with its own font-dependency DEC.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing material. The PINNED `## Operation mechanics` gave the exact RGBA8
   flow, gravity math, and `imageops` calls; DEC-031's probe removed all
   ambiguity about where the overlay loads. The one thing the spec didn't flag
   was that the existing `stub_command_returns_not_implemented` test used
   `watermark` as its stub sample and needed repointing.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-031 (boundary), DEC-002 (Operation), DEC-015 (fan-out), DEC-007
   (typed errors) were all the relevant ones and were referenced. The "don't
   register in `with_builtins()`" rule was explicit, which prevented the obvious
   wrong turn.

3. **If you did this task again, what would you do differently?**
   — Nothing significant. The implementation mirrored `Resize` and
   `run_auto_orient` closely, so the smallest-correct-change path was clear from
   the start. I'd grep for the spec's command name across `tests/` first to
   surface stub-sample tests like the one above before building.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Little. The interesting work was up front: `watermark` is the first
   Operation to compose a *second* image, which collided with the "pixel core is
   file-free" rule (AGENTS.md §11). Resolving that as a design decision (DEC-031 —
   overlay loaded at the IO boundary, op holds in-memory pixels, registry deferred)
   *before* the build meant the implementer never had to improvise the architecture.
   The probe (image::imageops overlay/opacity/scale/tile) was lighter-weight than
   the metadata-crate probes since `image` is well-known — right-sized to the risk.

2. **Does any template, constraint, or decision need updating?**
   — No template/constraint change. DEC-031 captured the multi-image boundary and
   the STAGE-005 deferral (registry round-trip). This session also added the
   lean-build gate (`cargo build --no-default-features`) to verify after the
   SPEC-028 lean flake — and SPEC-029's build prompt + my verify both ran it,
   catching nothing (good) but closing the gap. Worth folding that gate into the
   standard build/verify checklist going forward.

3. **Is there a follow-up spec I should write now before I forget?**
   — Two tracked on the stage backlog, neither blocking: (a) **text watermark**
   (`ab_glyph` + `imageproc::drawing`) — the last STAGE-004 item; it needs a
   font-dependency DEC (likely a feature-gated or default permissive font crate) and
   reuses `Gravity` + the placement math just built; (b) wiring `watermark` into
   **recipes** in STAGE-005 (register in `with_builtins`, resolve the overlay path in
   the recipe loader — the deferred half of DEC-031). No new spec file yet.
