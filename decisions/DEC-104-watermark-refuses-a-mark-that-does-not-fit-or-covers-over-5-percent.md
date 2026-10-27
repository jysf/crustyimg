---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-104
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-10-26
supersedes: null
superseded_by: null

affected_scope:
  - src/operation/mod.rs
  - src/pipeline/mod.rs
  - src/cli/mod.rs
  - src/cli/ops.rs
  - src/cli/optimize.rs
  - src/cli/common.rs
  - src/cli/build.rs
  - docs/api-contract.md
  - docs/data-model.md
  - recipes/README.md

tags:
  - watermark
  - operation-trait
  - pipeline
  - batch
  - exit-codes
  - dec-002
  - dec-015
  - dec-085
---

# DEC-104: A watermark refuses a mark that does not fit or covers over 5 % of the image; a batch skips it and writes the image without it

## Decision

**The rule.** Before anything is drawn, `Watermark` checks each image in pure geometry, from
three numbers: the image's size, the mark's size **as drawn** (after `scale`, computed by the
one helper the composite also uses), and the mark's count of visible pixels (alpha > 0,
counted once at construction on the overlay's 16-bit view, before `opacity`).

1. **Fit, always.** The mark plus `margin` on the axes the gravity insets must fit on both
   axes: corners inset both, `north`/`south` only y, `east`/`west` only x, `center` neither,
   and `tile` neither (one tile must fit). Equality fits.
2. **Coverage, for a fixed-size mark** (no `scale`, no `tile`): refused when
   `visible × 100 > width × height × 5`, integer arithmetic. The 5 is
   `WATERMARK_COVERAGE_CAP_PERCENT`, a constant, not a setting.
3. Fit is reported first.

**The seam.** `Operation` gains one **defaulted** method, `refusal(&self, &Image) ->
Option<Refusal>` (default `None`) — DEC-002's trait grows by one method that no existing
implementor has to write. `Watermark::apply` calls it first and returns
`OperationError::Refused { op, refusal }`, so a direct caller of `apply` is refused too.
`Pipeline::run_with(img, OnRefusal)` asks each step before applying it: `Fail` returns the
error (`Pipeline::run` is exactly this), `Skip` leaves the step out, records a
`SkippedStep { step, op, refusal }`, and runs the remaining steps on the unchanged image.

**The behaviour split.**
- **One explicit input** (`watermark`, `apply --recipe` plain or ending in `optimize`, wasm
  `transform`): `Fail` → `error: watermark refused: …`, **exit 2**, nothing written. Exit 2
  because the user's input asked for something the image cannot take, like a bad flag value;
  `CliError::code()` maps `Operation(Refused { .. })` to 2 above the generic `Operation(_) => 1`.
- **A batch or `build`**: `Skip` → the image is written **without the mark**, its other steps
  applied; one unconditional `warning: <input>: watermark skipped, image written without it:
  <reason>` line per image (DEC-085's precedent: an incomplete output with no other signal
  warns regardless of `--quiet`); a `--quiet`-gated count (`watermark skipped on K of N
  images`; on `build`, `, watermark skipped on K` in the summary).
- **A skip is not a failure.** This amends DEC-015's partial-batch rule for this one outcome:
  a skip never counts toward exit 6, a batch whose only problem is skips exits 0, and `build`
  still writes (or checks) its lockfile, which records the unmarked bytes.
- A recipe `text` step refuses `scale` and `tile = true` at load (DEC-102's unused-by-mode
  check), matching the CLI's `--text` conflicts; otherwise a text mark could bypass the cap
  through `tile`.

## Context

`watermark --text` clipped a run wider than the image (the saturated placement anchored it at
x = 0 and the overlay was cut), silently ignored a margin larger than the room left, and drew
a few stray pixels when a large mark met a small image — all at exit 0, on every surface. An
image overlay larger than the base had the same failure. The maintainer ruled (2026-10-24,
2026-10-25): refuse on fit and on a fixed 5 % cap, measured geometrically; one input is an
error; a batch skips the image, writes it unmarked, reports it, and does not fail.

## Alternatives Considered

- **Count changed pixels.** Rejected: measured on the same white mark, 1661 px change on a
  black image and 1647 on a photo — the rule would refuse a mark on a dark image and pass it
  on a light one. The mark's own visible pixels bound every pixel it can change and depend
  only on the mark and the image's size.
- **SSIMULACRA2 or any perceptual measure.** Ruled out by the maintainer.
- **Cap scaled and tiled marks too.** Rejected: a scaled mark is proportionate by
  construction (capping it would refuse every image at `scale ≥ 0.23` regardless of size),
  and tiling covers the image by design.
- **A `Refused` error that carries the image back, or cloning the image before every step.**
  Rejected: an error holding a pixel buffer, or a full copy per step per image for every op to
  serve one. Asking `refusal` first costs nothing.
- **A check per CLI surface.** Rejected: seven copies of one rule.
- **A distinct non-zero exit for skips.** Rejected by ruling: exit 6 would fail CI on the
  normal mixed-directory case and block `build`'s lockfile.

## Consequences

- Every surface shares `Watermark::refusal`, so a surface test proves only its lane (exit 2 vs
  skip); the rule is tested once on the op.
- A mark that passes draws byte-identically to before (goldens captured on the code before
  the rule, `tests/watermark_damage.rs`).
- ⚠ **A cached `build` re-run does not repeat the skip line or the summary suffix.** A cache
  hit writes the stored bytes without running the pipeline, so it cannot know a step was
  skipped. This is the repo's existing contract for per-input warnings (a hit replays neither
  the truncated-JPEG nor the animated-input warning); carrying it needs a cache-entry format
  change, which is the right fix for all three warnings together. Filed on STAGE-050: "a
  cached `build` replays no per-input warnings". The output is still correct and locked, and
  `--no-cache` lists every skip again. No cache-schema or lockfile change here.
- `--json` / `--explain json` reports are unchanged; the warning lines go to stderr.
- Not covered by this rule: a non-finite CLI `--size` (`nan`, `inf`) or `--scale nan` still
  renders a tiny mark that fits, at exit 0 (a separate STAGE-050 item).

## Revisit if

- The maintainer wants the 5 % to be a setting, or a relative (percent-of-image) `--size`.
- `--min-size` (the opt-in "do not mark small images" rule) lands: it should reuse
  `OnRefusal::Skip` rather than add a second skip mechanism.
- The cache gains per-entry warnings: the skip line should replay from it.
