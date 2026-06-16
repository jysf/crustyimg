---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-015
  type: story                      # epic | story | task | bug | chore
  cycle: verify                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet 4.6, fresh session
  created_at: 2026-06-15

references:
  decisions: [DEC-017, DEC-003, DEC-002, DEC-013, DEC-015, DEC-016, DEC-014, DEC-008, DEC-012, DEC-007]
  constraints:
    - ergonomic-defaults
    - single-image-library
    - decode-once-no-per-op-disk
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - test-before-implementation
    - untrusted-input-hardening
  related_specs: [SPEC-010, SPEC-011, SPEC-012, SPEC-013, SPEC-014, SPEC-002]

value_link: "Delivers STAGE-003's `auto-orient` — bakes a photo's EXIF orientation into pixels and clears the tag, fixing the most common silent-rotation bug, and is the first recipe-usable Operation that reads container metadata to drive a pixel transform."

# Self-reported AI cost per cycle.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "Design authored by the ORCHESTRATOR (Opus) directly (proven path from SPEC-013/014). Verified the image 0.25.10 orientation API (Orientation::from_exif_chunk parses a raw TIFF chunk; DynamicImage::apply_orientation; decode() does NOT auto-apply) and the exact 1-entry-IFD fixture byte layout. Emitted DEC-017 (operations may READ the captured MetadataBundle to drive a pixel transform; auto-orient uses image's native Orientation, no kamadak-exif). Complexity M (first metadata-reading Operation + a new registry entry + a new EXIF fixture)."
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "AutoOrient op + orientation_from_exif_segment helper + registry entry + run_auto_orient CLI + jpeg_with_orientation fixture + 7 unit + 9 integration tests (5 new auto-orient + repoint stub + 2 registry). All 4 gates pass; 206/206 tests green."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-015: auto-orient command and operation — bake EXIF orientation into pixels

## Context

`auto-orient` is the LAST STAGE-003 command. Cameras record how a photo was
held as an EXIF **Orientation** tag (1–8) instead of rotating the pixels, and
the `image` crate's decoder does **not** apply it on decode — so a portrait
photo often appears sideways until software honors the tag. `auto-orient` fixes
this silent-rotation bug: it **bakes the orientation into the pixels** and clears
the tag.

- **Parent stage:** `STAGE-003` (transform & output). The SIXTH and final
  command, after `resize` (SPEC-010/011), `thumbnail` (SPEC-012), `shrink`
  (SPEC-013) and `convert` (SPEC-014) — all shipped on `main`. Shipping this
  completes STAGE-003.
- **Why now / what's new:** Unlike the other STAGE-003 commands, `auto-orient`
  is a **new `Operation`** (`AutoOrient`) registered in the registry — so it is
  **recipe-usable** (the `op = "auto-orient"` step in `docs/data-model.md`'s
  worked recipe). It is the FIRST op that **reads container metadata** (the raw
  EXIF captured at load, DEC-003) to drive a pixel transform. **DEC-017** records
  that this is allowed (ops may READ, never edit, the captured `MetadataBundle`)
  and that the op uses the `image` crate's **native** orientation handling —
  `image::metadata::Orientation::from_exif_chunk` + `DynamicImage::apply_orientation`
  — so it stays within the operation module's existing `::image` dependency
  surface (NO `kamadak-exif`).
- **What stays the same:** The CLI side is a thin `run_auto_orient` that builds
  the op via the registry and reuses the shared `run_pixel_op` fan-out
  (DEC-015 per-input source-format preservation, multi-input `--out-dir`,
  partial-batch exit 6; DEC-016 quality threaded — no default).

The api-contract pins the surface: `auto-orient <INPUT...>`.

## Goal

Add an `AutoOrient` `Operation` that reads the orientation from the `Image`'s
captured EXIF segment, applies the corresponding rotation/flip to the pixels via
`image`'s native `Orientation`/`apply_orientation`, and drops the carried
metadata bundle (so the stale tag does not propagate; the pixel-lane re-encode
clears it inherently). Register it as `"auto-orient"` and wire the
`auto-orient` CLI command on top of the shared `run_pixel_op` fan-out. An image
with no EXIF, no orientation tag, or orientation 1 is a **no-op** (exit 0, not an
error).

## Inputs

- **Files to read:**
  - `src/operation/mod.rs` — the `Operation` trait, `OperationError`, the
    `Resize`/`Invert` impls (the template `AutoOrient` mirrors), the
    `#[cfg(test)] mod tests` (where `AutoOrient` unit tests go).
  - `src/operation/registry.rs` — `OperationRegistry::with_builtins` (register
    `"auto-orient"` alongside `identity`/`invert`/`resize`); the registry tests.
  - `src/image/mod.rs` — `Image::metadata()` → `Option<&MetadataBundle>`;
    `MetadataBundle { exif: Option<Vec<u8>>, icc: Option<Vec<u8>> }` (pub
    fields); `Image::from_parts(pixels, source_format, metadata)`;
    `Image::source_format()`; `Image::pixels()`. (Read-only — do NOT change this
    module.)
  - `src/cli/mod.rs` — `run_thumbnail`/`run_shrink` (the caller shape
    `run_auto_orient` mirrors), `run_pixel_op` (pass `global.quality` and `None`
    for `forced_format`), `Commands::AutoOrient { inputs }`, the dispatch arm,
    `RegistryError` mapping (→ `CliError::Usage`).
  - `decisions/DEC-017-operations-may-read-metadata-for-pixel-transforms.md` —
    the governing decision (already on `main` via the design commit).
  - `docs/api-contract.md` — the `auto-orient` entry (pinned during design).
  - `tests/common/mod.rs` — `jpeg_with_exif` (the pattern for the NEW
    `jpeg_with_orientation` fixture); `solid_png`/`gradient_jpeg`.
  - `tests/cli.rs` — conventions; `stub_command_returns_not_implemented`
    (currently points at `auto-orient` — REPOINT to `watermark`).
- **External APIs:** `image::metadata::Orientation` (`from_exif`,
  `from_exif_chunk`, variants), `DynamicImage::apply_orientation` — all in the
  already-pinned `image` 0.25.10. NO new dependency.
- **Related code paths:** `src/operation/` (op + registry) + `src/cli/` (command)
  + `tests/`. Do NOT modify `src/image/` or `src/sink/`.

## Outputs

- **Files modified:**
  - **`src/operation/mod.rs`** — NEW public `AutoOrient` op + a free helper:
    - `#[derive(Debug)] pub struct AutoOrient;`
    - `impl Operation for AutoOrient`:
      - `name()` → `"auto-orient"`; `params()` → `OperationParams::empty()`.
      - `apply(&self, img) -> Result<Image, OperationError>`:
        ```text
        let orientation = img.metadata()
            .and_then(|m| m.exif.as_deref())
            .and_then(orientation_from_exif_segment);
        match orientation {
            None | Some(::image::metadata::Orientation::NoTransforms) => Ok(img), // no-op
            Some(o) => {
                let mut pixels = img.pixels().clone();
                pixels.apply_orientation(o);
                let fmt = img.source_format();
                // Drop the carried bundle: the tag is now stale (DEC-017).
                Ok(Image::from_parts(pixels, fmt, None))
            }
        }
        ```
    - `fn orientation_from_exif_segment(exif: &[u8]) -> Option<::image::metadata::Orientation>`:
      strip a leading `b"Exif\0\0"` signature if present (JPEG APP1 payload),
      then `::image::metadata::Orientation::from_exif_chunk(tiff)` (PNG eXIf
      chunks are already bare TIFF). Returns `None` on no-tag / unparseable —
      never panics. Make it a free fn so it is directly unit-testable.
  - **`src/operation/registry.rs`** — in `with_builtins`, add
    `reg.register("auto-orient", |_p| Ok(Box::new(AutoOrient)));` and
    `use super::{..., AutoOrient};` (import the new type). No other change.
  - **`src/cli/mod.rs`** — NEW `fn run_auto_orient(inputs: &[String], global:
    &GlobalArgs) -> Result<(), CliError>`:
    ```text
    let op = OperationRegistry::with_builtins()
        .build("auto-orient", &OperationParams::empty())
        .map_err(|e| match e {
            RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
            RegistryError::Unknown { name } => CliError::Usage(format!("unknown operation '{name}'")),
        })?;
    let pipeline = Pipeline::new().push(op);
    run_pixel_op(pipeline, inputs, global, global.quality, None)
    ```
    Dispatch: replace `Commands::AutoOrient { .. } => Err(NotImplemented("auto-orient"))`
    with `Commands::AutoOrient { inputs } => run_auto_orient(inputs, &cli.global)`.
    NO new `CliError` variant; NO `code()`/`exit_code_mapping_is_total` change.
  - **`tests/common/mod.rs`** — NEW fixture
    `jpeg_with_orientation(w: u32, h: u32, orientation: u8) -> Vec<u8>`: mirror
    `jpeg_with_exif` but with a ONE-entry IFD for the Orientation tag (see Notes
    for the exact bytes).
  - **`tests/cli.rs`** — add the integration tests below; REPOINT
    `stub_command_returns_not_implemented` from `auto-orient` to `watermark`.
  - **`docs/api-contract.md`** — the `auto-orient` entry is **pinned during
    design**; do NOT edit it during build unless the code deviates.
- **New decisions:** `DEC-017` (emitted in design) — operations may read the
  captured `MetadataBundle` to drive a pixel transform; `auto-orient` uses
  `image`'s native `Orientation`.
- **No new dependency. No new `CliError` variant. No `src/image`/`src/sink` change.**

## Acceptance Criteria

Each maps to a test.

- [ ] `AutoOrient` on an image whose captured EXIF Orientation is 6 (rotate 90)
  swaps width/height and rotates the pixels. → `auto_orient_rotate90_swaps_dims`
- [ ] `AutoOrient` reads a PNG-style bare-TIFF EXIF chunk (no `Exif\0\0` prefix)
  as well as a JPEG-style prefixed one. → `auto_orient_reads_png_style_exif_chunk`
- [ ] `AutoOrient` applies a non-rotating transform correctly (orientation 2 =
  horizontal flip moves pixels). → `auto_orient_flip_horizontal_moves_pixels`
- [ ] `AutoOrient` is a no-op when there is no metadata / no orientation tag /
  orientation 1, returning the image unchanged. → `auto_orient_noop_*`
- [ ] After `AutoOrient` applies a transform, the returned image carries no
  metadata bundle. → asserted within `auto_orient_rotate90_swaps_dims`
- [ ] `orientation_from_exif_segment` extracts the orientation (prefixed and
  bare) and returns `None` on garbage. → `orientation_from_exif_segment_*`
- [ ] The registry resolves `"auto-orient"`. → `with_builtins_contains_auto_orient`,
  `build_auto_orient`
- [ ] CLI: `auto-orient <jpg-with-orientation-6>` rotates the file (dims swap)
  and the output carries NO EXIF (tag cleared). →
  `auto_orient_cli_rotates_and_clears_tag`
- [ ] CLI: `auto-orient <plain png>` (no EXIF) → exit 0, dimensions unchanged. →
  `auto_orient_cli_noop_without_exif`
- [ ] CLI: multi-input `--out-dir` fan-out preserves source format. →
  `auto_orient_cli_multi_input_fan_out`
- [ ] CLI: missing input → exit 3; multi-input without `--out-dir` → exit 2. →
  `auto_orient_cli_missing_input_exits_3`, `auto_orient_cli_multi_without_out_dir_is_usage_error`

## Failing Tests

Written during **design**, made to pass during **build**.

- **`src/operation/mod.rs`** (UNIT — in the `#[cfg(test)] mod tests` block;
  build an `Image` via `Image::from_parts(DynamicImage, ImageFormat, Some(MetadataBundle{exif: Some(bytes), icc: None}))`,
  importing `crate::image::MetadataBundle`)
  - `auto_orient_name_and_params_stable` — `AutoOrient.name() == "auto-orient"`,
    `params()` empty.
  - `auto_orient_noop_without_metadata` — a 4×2 image with `metadata: None` →
    `apply` returns dims 4×2 unchanged.
  - `auto_orient_noop_on_orientation_1` — image with an orientation-1 EXIF bundle
    → dims unchanged.
  - `auto_orient_rotate90_swaps_dims` — a 4×2 image with a JPEG-style
    (`Exif\0\0`-prefixed) orientation-6 bundle → output dims 2×4 AND
    `result.metadata().is_none()` (bundle dropped, DEC-017).
  - `auto_orient_reads_png_style_exif_chunk` — same as above but a **bare TIFF**
    (no `Exif\0\0` prefix) orientation-6 bundle → output dims 2×4 (proves the
    prefix-strip handles both shapes).
  - `auto_orient_flip_horizontal_moves_pixels` — a 2×1 image with two distinct
    columns (e.g. left red, right blue) and an orientation-2 bundle → dims
    unchanged 2×1, but column 0 and column 1 are swapped (proves
    `apply_orientation` actually ran and is correct).
  - `orientation_from_exif_segment_prefixed_and_bare` — feed the helper a
    `Exif\0\0`-prefixed orientation-6 TIFF and a bare orientation-6 TIFF; both →
    `Some(Orientation::Rotate90)`. Garbage bytes and an empty slice → `None`.
- **`src/operation/registry.rs`** (UNIT)
  - `with_builtins_contains_auto_orient` — `reg.contains("auto-orient")`.
  - `build_auto_orient` — `reg.build("auto-orient", &OperationParams::empty())`
    succeeds; `op.name() == "auto-orient"`.
- **`tests/cli.rs`** (INTEGRATION — use the NEW `common::jpeg_with_orientation`
  and `common::solid_png`; drive the real binary; decode with
  `image::load_from_memory`; for "tag cleared", run a second `info --json`
  invocation on the output and assert `"has_exif":false`)
  - `auto_orient_cli_rotates_and_clears_tag` — write `jpeg_with_orientation(4, 2, 6)`;
    `auto-orient <jpg> -o out.jpg` → exit 0; `out.jpg` decodes at **2×4**
    (rotated); then `info out.jpg --json` reports `"has_exif":false` (the tag is
    gone after the re-encode).
  - `auto_orient_cli_noop_without_exif` — `solid_png(8, 4, ..)` (no EXIF) →
    `auto-orient <png> -o out.png` → exit 0; output decodes at 8×4 (unchanged).
  - `auto_orient_cli_multi_input_fan_out` — two `jpeg_with_orientation(4, 2, 6)`
    inputs → `auto-orient a.jpg b.jpg --out-dir D` → exit 0; `D/a.jpg`, `D/b.jpg`
    both JPEG, both decode at 2×4.
  - `auto_orient_cli_missing_input_exits_3` — missing file → exit 3.
  - `auto_orient_cli_multi_without_out_dir_is_usage_error` — two inputs, no
    `--out-dir` → exit 2; stderr mentions `out-dir`.
  - REPOINT `stub_command_returns_not_implemented` from `auto-orient` to
    `watermark` (a still-stubbed STAGE-004 command, e.g.
    `watermark <png> --image logo.png`); keep the exit-1 + "not yet implemented"
    assertions.

The existing resize/thumbnail/shrink/convert tests + all unit/sink tests MUST
stay green (run the FULL suite).

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply
- `DEC-017` — operations may READ the captured `MetadataBundle` to drive a pixel
  transform; `auto-orient` uses `image`'s native `Orientation::from_exif_chunk`
  + `DynamicImage::apply_orientation` (NO `kamadak-exif`); drop the carried
  bundle after baking. THE governing decision.
- `DEC-003` — metadata dual-lane: `auto-orient` only **reads** the captured EXIF
  to choose the transform; it does NOT edit container metadata (that is the
  STAGE-004 container lane). The output tag is cleared by the inherent pixel-lane
  drop + the bundle drop.
- `DEC-002` — the `Operation` boundary: `AutoOrient` is a pure in-memory pixel
  transform; reading the already-captured bundle requires no disk I/O
  (`decode-once-no-per-op-disk` holds).
- `DEC-013` — `kamadak-exif` is the read-only `info` lane; keep it OUT of the
  operation module (use `image`'s native orientation parser instead).
- `DEC-015` / `DEC-016` — the `run_pixel_op` fan-out (source-format preservation,
  exit 6) and quality threading (`global.quality`, no default) — inherited via
  `run_auto_orient` → `run_pixel_op(..., None)`.
- `DEC-014` — op-params construction: `auto-orient` is parameterless
  (`OperationParams::empty()`); it builds through the registry like the others.
- `DEC-008` — resize backend (not used here; the `Resize` impl is only the
  structural template).
- `DEC-012` / `DEC-007` — clap surface; typed errors → exit codes.

### Constraints that apply
- `single-image-library` — orientation parsing + rotation use the `image` crate
  only; NO `kamadak-exif`, NO second image lib.
- `decode-once-no-per-op-disk` — `AutoOrient::apply` reads the already-captured
  EXIF bytes from the in-memory `Image`; it does NOT open files or re-decode.
- `no-unwrap-on-recoverable-paths` — `orientation_from_exif_segment` returns
  `Option` (no panic on bad EXIF); `apply` never unwraps.
- `every-public-fn-tested` — `AutoOrient` (name/params/apply) and the registry
  entry get unit tests; the helper is unit-tested; the command is
  integration-tested.
- `ergonomic-defaults` — `auto-orient photo.jpg` is one short command, no flags.
- `clippy-fmt-clean` — `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check`.
- `untrusted-input-hardening` — malformed/short EXIF is handled gracefully
  (`from_exif_chunk` returns `None`); no panic on adversarial bytes.

### Prior related work
- `SPEC-010` (shipped, PR #11) — the `Resize` op + `from_params` constructor;
  the structural template for a registry-registered op (minus params).
- `SPEC-012`/`SPEC-013`/`SPEC-014` (shipped) — `run_thumbnail`/`run_shrink`/
  `run_convert` (the `run_auto_orient` caller shape) + `run_pixel_op` (now
  carries `quality` + `forced_format`).
- `SPEC-002` (shipped) — `MetadataBundle::capture` (JPEG APP1 `Exif\0\0`, PNG
  `eXIf`); `auto-orient` consumes exactly what capture records.

### Out of scope (create a new spec rather than expand)
- Editing/preserving NON-orientation metadata across `auto-orient` (selective
  preserve / `--keep-gps` / `strip`) — STAGE-004 container lane. `auto-orient`
  drops the whole bundle after baking; richer preservation is later.
- Extending EXIF capture to more source formats (TIFF/BMP/GIF/ICO) — STAGE-004.
  For those, `auto-orient` is a safe no-op today (no orientation captured).
- A standalone `rotate`/`flip` geometry op (post-MVP geometry wave).
- rayon / parallel batch (STAGE-005). Fan-out stays sequential.
- Any new dependency, `CliError` variant, `src/image`/`src/sink` change, or
  `exit_code_mapping_is_total` change.

## Notes for the Implementer

- **Exact orientation TIFF fixture (little-endian, Orientation = N):** a valid
  raw EXIF/TIFF chunk that `image::metadata::Orientation::from_exif_chunk` will
  parse is the 22 bytes below (then a 4-byte next-IFD offset of 0 for validity):
  ```text
  49 49 2A 00            // "II", 42  (little-endian TIFF magic)
  08 00 00 00            // IFD offset = 8
  01 00                  // entry count = 1
  12 01                  // tag 0x0112 (Orientation)
  03 00                  // type 3 (SHORT)
  01 00 00 00            // count = 1
  <N> 00                 // value = N (the orientation, e.g. 06 00)
  00 00                  // value padding
  00 00 00 00            // next-IFD offset = 0
  ```
  `from_exif_chunk` requires `tag == 0x0112 && type == 3 && count == 1` and reads
  the value as a `u16`. For a JPEG container, prepend `b"Exif\0\0"` and splice it
  as an APP1 segment after SOI — `jpeg_with_orientation` should mirror
  `jpeg_with_exif`'s splice but use this 1-entry IFD payload (payload =
  `b"Exif\0\0"` + the bytes above). For the UNIT tests, you can pass the chunk
  (with or without the `Exif\0\0` prefix) straight into a `MetadataBundle`.
- **`apply_orientation` semantics (image 0.25.10):** EXIF 6 → `Rotate90` (a W×H
  image becomes H×W); 8 → `Rotate270`; 3 → `Rotate180`; 2 → `FlipHorizontal`;
  4 → `FlipVertical`; 1 → `NoTransforms`. Use `Orientation::from_exif(u8)`
  indirectly via `from_exif_chunk` — do NOT hand-roll the rotate/flip matrix.
- **Drop the bundle, don't carry it.** Return
  `Image::from_parts(rotated_pixels, img.source_format(), None)` — NOT
  `img.with_pixels(..)` (which would carry the stale orientation tag forward).
  This is the DEC-017 correctness point; the no-op branch returns `img` unchanged
  (bundle intact, harmless).
- **No-op is success, not error.** No EXIF, no orientation tag, or orientation 1
  → return the image unchanged and exit 0. Never error on a missing tag.
- **`run_auto_orient` mirrors `run_shrink`/`run_thumbnail`** but with an empty
  params map and `global.quality` (no forced default) + `None` forced_format.
- **Decoder does NOT auto-orient.** `Image::from_bytes` (via `ImageReader::decode`)
  decodes raw pixels and does NOT apply orientation, so `AutoOrient` has real
  work to do; do not assume the pixels are already oriented.
- **`Debug` on `AutoOrient`:** derive it (the spec sketch does). Do not
  `{:?}`-format a non-`Debug` type.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-015-auto-orient-command-and-operation-bake-exif-orientation-into-pixels`
- **PR (if applicable):** #15 (see timeline for exact URL)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - No new DEC during build — DEC-017 already governs.
- **Deviations from spec:**
  - None. The `orientation_from_exif_segment` helper, the `AutoOrient::apply` body, `run_auto_orient`, the fixture byte layout, and all named tests follow the spec exactly.
- **Follow-up work identified:**
  - Selective metadata preserve across `auto-orient` (keep non-orientation tags): STAGE-004 container lane.
  - Extend EXIF capture to TIFF/BMP/GIF/ICO: STAGE-004.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing significant. The spec was unusually precise: exact TIFF bytes, the `from_parts(…, None)` vs `with_pixels` distinction, the `strip_prefix` helper shape, and the two-step `info --json` verification for the integration test were all spelled out. Rustfmt's trailing-comment alignment (aligning a standalone comment to the inline comments in the block above) caused a minor delay — removed the offending comment to keep fmt happy.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The `mod common;` pattern for `tests/cli.rs` (vs other integration test files that already had it) wasn't mentioned, but was trivial to sort out by reading the other test files.

3. **If you did this task again, what would you do differently?**
   — Run `cargo fmt` immediately after each file edit rather than after all edits; the formatter's comment-alignment behavior surfaced mid-gate and required an extra round-trip.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
</content>
