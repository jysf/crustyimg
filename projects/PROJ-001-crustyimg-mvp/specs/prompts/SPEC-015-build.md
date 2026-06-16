# SPEC-015 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. This prompt is deliberately prescriptive — follow it literally.
> Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-015 ("auto-orient command and operation — bake
EXIF orientation into pixels"). You are NOT the architect; the spec is your
source of truth. This adds a NEW `AutoOrient` Operation (registered in the
registry, recipe-usable) + the `auto-orient` CLI command + a new test fixture.
It uses the `image` crate's NATIVE orientation handling — NO kamadak-exif, NO new
dependency, NO new CliError variant, NO src/image or src/sink change. Use ABSOLUTE
paths for every file.

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST (before editing ANY file)
═══════════════════════════════════════════════════════════════════════════
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-015-auto-orient-command-and-operation-bake-exif-orientation-into-pixels
Confirm `git branch --show-current` shows that branch — NOT `main`, NOT any
`chore/*` branch — before you commit anything. ALL edits happen on this branch.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════
1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — §6 the EXACT gate commands (the clippy gate is
   `cargo clippy --all-targets -- -D warnings`), §11 conventions (typed errors;
   NO unwrap/expect/panic on recoverable paths; one image library only; the
   operation module must NOT depend on clap/recipe/source/sink/fs/path), §12
   testing (native in-memory fixtures via the `image` crate), §13 git/PR, §15
   build-cycle rules (spec edits limited to ## Build Completion; append a build
   cost session; DEC-017 already exists — do NOT create a new DEC).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-015-auto-orient-command-and-operation-bake-exif-orientation-into-pixels.md
   — THE SPEC. Implement its "## Outputs", "## Failing Tests" and "## Notes for
   the Implementer" EXACTLY. The `AutoOrient::apply` body, the
   `orientation_from_exif_segment` helper, `run_auto_orient`, and the EXACT
   orientation-TIFF fixture bytes are all spelled out there.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-017-operations-may-read-metadata-for-pixel-transforms.md
   — the governing decision: ops may READ (never edit) the captured
   MetadataBundle; auto-orient uses image's native Orientation::from_exif_chunk +
   DynamicImage::apply_orientation; drop the carried bundle after baking. Already
   on main — do NOT create a new DEC.
4. The SHIPPED code you change/reuse (read the real signatures):
   src/operation/mod.rs      — the `Operation` trait, `OperationError`, the
                               `Resize`/`Invert` impls (template), the
                               `#[cfg(test)] mod tests` block. You ADD the
                               `AutoOrient` struct + impl + the free
                               `orientation_from_exif_segment` helper + unit tests.
   src/operation/registry.rs — `with_builtins` (add the "auto-orient" register
                               line + import `AutoOrient`); the registry tests
                               (add the two new ones).
   src/image/mod.rs          — `Image::metadata()` (→ Option<&MetadataBundle>),
                               `MetadataBundle { exif: Option<Vec<u8>>, icc: ... }`
                               (pub fields), `Image::from_parts`,
                               `source_format()`, `pixels()`. READ-ONLY — do NOT
                               edit this module.
   src/cli/mod.rs            — `run_thumbnail`/`run_shrink` (the shape
                               `run_auto_orient` mirrors), `run_pixel_op`
                               (signature: pipeline, inputs, global, quality,
                               forced_format — pass `global.quality` and `None`),
                               `Commands::AutoOrient { inputs }`, the dispatch arm,
                               the `RegistryError`→`CliError::Usage` mapping. DO
                               NOT change CliError/code()/exit_code_mapping_is_total.
   tests/common/mod.rs       — `jpeg_with_exif` (the splice pattern for the NEW
                               `jpeg_with_orientation` fixture).
   tests/cli.rs              — conventions; `stub_command_returns_not_implemented`
                               (currently `auto-orient` — REPOINT to `watermark`).

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact — follow the spec's ## Outputs)
═══════════════════════════════════════════════════════════════════════════
A. src/operation/mod.rs — the AutoOrient op + helper.
   A1. `#[derive(Debug)] pub struct AutoOrient;`
   A2. `impl Operation for AutoOrient`: `name()`→"auto-orient"; `params()`→
       `OperationParams::empty()`; `apply` per the spec sketch — read the
       orientation from `img.metadata().and_then(|m| m.exif.as_deref())
       .and_then(orientation_from_exif_segment)`; on `None` or
       `Orientation::NoTransforms` return `img` unchanged (no-op); otherwise
       clone the pixels, `apply_orientation(o)`, and return
       `Image::from_parts(pixels, img.source_format(), None)` (DROP the bundle —
       NOT `with_pixels`).
   A3. `fn orientation_from_exif_segment(exif: &[u8]) -> Option<::image::metadata::Orientation>`
       — strip a leading `b"Exif\0\0"` if present (use
       `exif.strip_prefix(b"Exif\0\0").unwrap_or(exif)`), then
       `::image::metadata::Orientation::from_exif_chunk(tiff)`. Free fn (directly
       unit-testable). Never panics.
   NOTE: `image::metadata::Orientation` and `DynamicImage::apply_orientation` are
   in the already-pinned `image` 0.25.10 — confirm with
   `cargo doc`/source if unsure. The operation module already uses `::image`
   (e.g. `::image::DynamicImage`), so NO new import path beyond `::image::metadata`.

B. src/operation/registry.rs — register the op.
   B1. Extend the `use super::{...}` line to import `AutoOrient`.
   B2. In `with_builtins`, add:
       `reg.register("auto-orient", |_params| Ok(Box::new(AutoOrient)));`

C. src/cli/mod.rs — the command.
   C1. NEW `fn run_auto_orient(inputs: &[String], global: &GlobalArgs)
       -> Result<(), CliError>` per the spec: build the "auto-orient" op via
       `OperationRegistry::with_builtins()` (map RegistryError→CliError::Usage
       exactly like run_thumbnail/run_shrink), `Pipeline::new().push(op)`, then
       `run_pixel_op(pipeline, inputs, global, global.quality, None)`.
   C2. Dispatch: replace
       `Commands::AutoOrient { .. } => Err(CliError::NotImplemented("auto-orient"))`
       with
       `Commands::AutoOrient { inputs } => run_auto_orient(inputs, &cli.global)`.

D. tests/common/mod.rs — NEW fixture
   `pub fn jpeg_with_orientation(w: u32, h: u32, orientation: u8) -> Vec<u8>`:
   mirror `jpeg_with_exif` but the APP1 payload is `b"Exif\0\0"` followed by a
   ONE-entry IFD for the Orientation tag. The exact TIFF bytes (little-endian)
   are in the spec's Notes:
     49 49 2A 00 / 08 00 00 00 / 01 00 / 12 01 / 03 00 / 01 00 00 00 /
     <orientation> 00 / 00 00 / 00 00 00 00
   (the `<orientation>` byte is the function arg). Splice as an APP1 segment after
   SOI exactly like `jpeg_with_exif`.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (make them pass) — per the spec's ## Failing Tests
═══════════════════════════════════════════════════════════════════════════
UNIT (src/operation/mod.rs `#[cfg(test)] mod tests`; import
`crate::image::MetadataBundle`; build images via `Image::from_parts(DynamicImage,
ImageFormat, Some(MetadataBundle{exif: Some(bytes), icc: None}))`):
  - auto_orient_name_and_params_stable
  - auto_orient_noop_without_metadata            (4×2, metadata None → 4×2)
  - auto_orient_noop_on_orientation_1            (orientation-1 bundle → dims unchanged)
  - auto_orient_rotate90_swaps_dims              (4×2, JPEG-style Exif\0\0+TIFF orient 6 → 2×4 AND result.metadata().is_none())
  - auto_orient_reads_png_style_exif_chunk       (4×2, BARE TIFF orient 6, no prefix → 2×4)
  - auto_orient_flip_horizontal_moves_pixels     (2×1 left/right distinct, orient 2 → cols swapped, dims 2×1)
  - orientation_from_exif_segment_prefixed_and_bare (prefixed & bare orient-6 → Some(Rotate90); garbage/empty → None)
UNIT (src/operation/registry.rs tests):
  - with_builtins_contains_auto_orient
  - build_auto_orient                            (build succeeds; op.name()=="auto-orient")
INTEGRATION (tests/cli.rs — use common::jpeg_with_orientation / common::solid_png;
drive the real binary; decode with image::load_from_memory; for "tag cleared"
run a SECOND `info <out> --json` invocation and assert the stdout contains
`"has_exif":false`):
  - auto_orient_cli_rotates_and_clears_tag       (jpeg_with_orientation(4,2,6) → out 2×4; info --json has_exif false)
  - auto_orient_cli_noop_without_exif            (solid_png(8,4) → out 8×4, exit 0)
  - auto_orient_cli_multi_input_fan_out          (two oriented jpgs --out-dir D → D/a.jpg,D/b.jpg both 2×4 JPEG)
  - auto_orient_cli_missing_input_exits_3
  - auto_orient_cli_multi_without_out_dir_is_usage_error (exit 2, stderr mentions out-dir)
  - REPOINT stub_command_returns_not_implemented from `auto-orient` → `watermark`
    (e.g. `watermark <png> --image <png>`); keep exit-1 + "not yet implemented" asserts.

The existing resize/thumbnail/shrink/convert + all unit/sink tests MUST stay
green (run the FULL suite).

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════
- kamadak-exif anywhere in the operation (use image's native Orientation). - Any
  edit/preserve of NON-orientation metadata, --keep-gps, strip (STAGE-004
  container lane). - EXIF capture for new formats (src/image is read-only here).
- A standalone rotate/flip op. - rayon/parallel (STAGE-005). - Any new
  dependency, CliError variant, src/image or src/sink change,
  exit_code_mapping_is_total change, or DEC. If you think one is needed, STOP and
  add a question to guidance/questions.yaml.

═══════════════════════════════════════════════════════════════════════════
THE GATES (run from repo root; ALL must pass before the PR)
═══════════════════════════════════════════════════════════════════════════
  cargo build
  cargo test
  cargo clippy --all-targets -- -D warnings     # --all-targets is the CI gate
  cargo fmt --check                              # run `cargo fmt` first to fix

RUN GATES AND COMMIT INCREMENTALLY — commit once the op+registry compile and
clippy/fmt are clean, then again after the tests are green. Do NOT leave all work
uncommitted to the end; a green committed checkpoint must survive an interruption.
(Hard lesson from SPEC-011.)

BEFORE YOU FINISH: re-read the spec's ## Failing Tests and CONFIRM EACH NAMED
TEST EXISTS in the code and runs — list them and check each off. A passing test
COUNT does not prove the prescribed tests were written. In particular confirm
`auto_orient_rotate90_swaps_dims` asserts BOTH dims 2×4 AND
`result.metadata().is_none()`, and `auto_orient_cli_rotates_and_clears_tag`
asserts the rotated dims AND `"has_exif":false` on the output.

Also: derive `Debug` on `AutoOrient`; do not `{:?}`-format a non-Debug type.

═══════════════════════════════════════════════════════════════════════════
WHEN DONE
═══════════════════════════════════════════════════════════════════════════
1. Fill ONLY the spec's `## Build Completion` (branch, PR, criteria, deviations,
   follow-ups, 3-question reflection). Edit nothing else in the spec body.
2. Append a build cost session to the spec front-matter `cost.sessions`
   (cycle: build, agent: claude-sonnet-4-6, interface: claude-code, null
   numerics, recorded_at: 2026-06-15, a one-line note).
3. Hand-edit the spec front-matter `task.cycle` from `build` to `verify`. DO NOT
   run `just advance-cycle` or `just archive-spec`.
4. Mark the build line `[x]` in the timeline
   (projects/PROJ-001-crustyimg-mvp/specs/SPEC-015-...-timeline.md) with ACCURATE
   wording — "PR #N opened" (real number). Never "merged"/"approved".
5. Commit on the branch (Conventional Commits, e.g.
   `feat(operation): auto-orient op + command — bake EXIF orientation (SPEC-015)`),
   end EACH commit with: `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`.
6. Push and open a PR on `jysf/crustyimg` (§13 template): Summary; Spec metadata
   PROJ-001/STAGE-003/SPEC-015; Decisions referenced [DEC-017 (ops read metadata),
   DEC-003 (metadata dual-lane), DEC-002 (Operation boundary), DEC-013 (kamadak
   stays in info lane), DEC-015 (fan-out), DEC-016 (quality), DEC-014 (registry),
   DEC-012/007 (clap/typed errors)]; Constraints checked (one-line evidence
   each); New decisions: "No new DEC during build — DEC-017 already governs". End
   with the Claude Code generated-with footer.
```
</content>
