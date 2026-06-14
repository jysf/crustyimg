---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-002
  type: story                      # epic | story | task | bug | chore
  cycle: build                     # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # usually same Claude, different session
  created_at: 2026-06-13

references:
  decisions:
    - DEC-002                       # single canonical Image over image::DynamicImage + Operation trait
    - DEC-003                       # metadata dual-lane: capture raw EXIF/ICC bundle at load (no interpretation)
    - DEC-004                       # pure-Rust codecs by default; `image` crate, native codecs feature-gated
    - DEC-007                       # thiserror in lib; typed ImageError + crate Result alias; no panic on recoverable paths
  constraints:
    - single-image-library
    - pure-rust-codecs-default
    - no-unwrap-on-recoverable-paths
    - no-new-top-level-deps-without-decision
    - clippy-fmt-clean
    - every-public-fn-tested
    - test-before-implementation
  related_specs:
    - SPEC-001                      # scaffold this builds on (empty-deps std-only project + CI)
    - SPEC-003                      # next: Operation trait + Pipeline consume this Image

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-XXX's <capability>". Optional; null is acceptable.
value_link: "infrastructure enabling the canonical image model the whole pipeline and metadata lane build on"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Null numeric fields are fine (e.g. claude.ai web sessions); reports
# skip them in sums but count them in session_count. Examples of
# interface: claude-code | claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_input: null
      tokens_output: null
      estimated_usd: null
      duration_minutes: 35
      recorded_at: 2026-06-13
      notes: "subagent; cost not separately reported"
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_input: null
      tokens_output: null
      estimated_usd: null
      duration_minutes: 20
      recorded_at: 2026-06-13
      notes: "subagent; cost not separately reported"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-002: canonical Image type and load

## Context

This is the second spec of PROJ-001 (the crustyimg clean rebuild) and the
keystone of STAGE-001's backlog after the project scaffold (SPEC-001). It
introduces the **canonical in-memory image model** the entire pipeline rests
on: one `Image` type wrapping `image::DynamicImage`, a robust load/decode
entry point with format detection, an `ImageInfo` inspection struct, and the
first typed `thiserror` error enum for the library.

SPEC-001 shipped an empty-deps, std-only scaffold (a `crustyimg` library +
thin binary + green three-OS CI). SPEC-002 therefore introduces the **first
real runtime dependency**, the `image` crate — the single pixel library the
whole rebuild is built on (DEC-002). Adding a top-level dep normally requires
a new DEC (constraint `no-new-top-level-deps-without-decision`), but `image`
is already pre-justified by DEC-002 (single canonical model) and DEC-004
(pure-Rust codecs default; `image` named as the default decode/encode stack).
**No new DEC is required for `image`** — reference DEC-002/DEC-004 in the PR's
"New decisions" section as the justification. If the build needs ANY other
crate, it MUST emit a DEC first.

This spec also captures the **raw metadata bundle at load** without
interpreting it (DEC-003). The `image` crate discards metadata on encode, so
the later default-preserve policy + metadata lane (STAGE-004) need the raw
EXIF/ICC bytes to have been carried alongside the decoded pixels from the
moment of load. Here we only **capture and carry** that bundle; the
container-level *editing* of it is STAGE-004 and explicitly out of scope.

Everything downstream plugs into this: `Operation`/`Pipeline` (SPEC-003) fold
over an `Image`; `Source`/`Sink` (SPEC-004/005) read into and write out of it;
`info` (STAGE-002) reports an `ImageInfo`. Building any of those before this
contract exists would mean rework, so the canonical model is first.

- Parent stage: `projects/PROJ-001-crustyimg-mvp/stages/STAGE-001-foundation-and-pipeline-core.md` (backlog item #2)
- Project: `projects/PROJ-001-crustyimg-mvp/brief.md`
- Architecture: `docs/architecture.md` — the `image/` module + `Image` wrapper + layering rules
- Data model: `docs/data-model.md` — the in-memory `Image`/`ImageInfo` model + `MetadataBundle`
- CLI contract: `docs/api-contract.md` — how `info` (STAGE-002) will later consume `ImageInfo`

## Goal

Implement the `crustyimg` `image/` module: a canonical `Image` type wrapping
`image::DynamicImage` (carrying the source format and an optional raw metadata
bundle captured at load), an `ImageInfo` inspection struct, and load entry
points (from a path, from bytes, and from a reader) with format detection and
typed `ImageError` failures. Add the `image` crate as the first runtime
dependency (pre-justified by DEC-002/DEC-004) and the first typed `thiserror`
error surface (`src/error.rs`).

## Inputs

- **Files to read:**
  - `AGENTS.md` — §5 stack (`image` 0.25; `thiserror` 2 in lib), §6 EXACT
    commands, §11 conventions (library-first; pixel core must not depend on
    clap/files/terminals; typed errors, no panic on recoverable paths),
    §12 testing (native-generated fixtures, no ImageMagick), §13/§15 git/PR.
  - `projects/PROJ-001-crustyimg-mvp/stages/STAGE-001-foundation-and-pipeline-core.md`
    — parent stage (backlog item #2); the "load/decode with format detection
    and clear errors" line and the metadata-lane separation note.
  - `projects/PROJ-001-crustyimg-mvp/brief.md` — project thesis + scope.
  - `docs/architecture.md` — the `image/` module is the "stable core"; it has
    no internal deps and no I/O policy / clap knowledge.
  - `docs/data-model.md` — the `Image` / `ImageInfo` / `MetadataBundle`
    conceptual signatures (field tables) this spec pins.
  - `decisions/DEC-002-single-image-model-and-operation-trait.md`
  - `decisions/DEC-003-metadata-dual-lane.md`
  - `decisions/DEC-004-codec-policy-pure-rust-default.md`
  - `decisions/DEC-007-error-handling-thiserror-anyhow.md`
  - `guidance/constraints.yaml` — full text of the constraints below.
  - `projects/PROJ-001-crustyimg-mvp/specs/done/SPEC-001-cargo-project-and-multi-os-ci.md`
    — what already exists (empty-deps scaffold, `version()` lib fn, smoke test).
- **External APIs:** `image` crate 0.25 — <https://docs.rs/image/0.25> — no
  auth. Key items: `image::DynamicImage`, `image::ImageFormat`,
  `image::ColorType`, `image::ImageReader` (format-guessing reader),
  `image::guess_format(&[u8])`, `image::load_from_memory[_with_format]`,
  `image::ImageError` (decode error type to wrap).
- **Related code paths:** `src/lib.rs` (existing root; add `pub mod image;`
  and `pub mod error;`), `src/` (new `image/` module, `error.rs`).

## Outputs

- **Files created (at build):**
  - `src/error.rs` — the first typed library error. A `thiserror`-derived
    `ImageError` enum and a crate-level `Result<T>` alias. Minimum variants:
    - `Io(#[from] std::io::Error)` — file open/read failure (path entry only).
    - `Decode(...)` — wraps/maps `image::ImageError`; the pixels could not be
      decoded (corrupt/invalid data).
    - `UnsupportedFormat(...)` — the byte stream's format could not be
      detected / is not a format `image` can decode.
    Each variant carries a clear `#[error("...")]` message (diagnostics go to
    stderr at the binary boundary; messages should name the failure, not the
    path — the path is added as `anyhow` context in `main`, a later spec).
    Export `pub type Result<T> = std::result::Result<T, ImageError>;` (the
    crate `Result` alias; later specs may widen this to a unified `Error` —
    DEC-007 — but SPEC-002 only needs the image surface).
  - `src/image/mod.rs` — the `image/` module root. Defines and re-exports the
    public types below. (If the implementer prefers `src/image.rs` as a single
    file that is acceptable; the module name is what matters. Note the module
    is named `image`, shadowing the `image` crate inside the module — refer to
    the crate as `::image` or import specific items at the top.)
  - The module must provide:
    - `pub struct Image` — wraps the decoded pixels + source format + optional
      raw metadata bundle. Conceptual fields (per `docs/data-model.md`):
      - `pixels: ::image::DynamicImage`
      - `source_format: ::image::ImageFormat`
      - `metadata: Option<MetadataBundle>`
      Constructors/accessors (signatures are the contract — names may be
      lightly adjusted only if a test is updated to match):
      - `pub fn load(path: impl AsRef<std::path::Path>) -> Result<Image>` —
        open the file, detect format, decode, capture the raw metadata bundle.
      - `pub fn from_bytes(bytes: &[u8]) -> Result<Image>` — detect format
        from the byte slice, decode, capture metadata.
      - `pub fn from_reader<R: std::io::Read + std::io::Seek>(reader: R) -> Result<Image>`
        — for the stdin path later (SPEC-004); a seekable reader so format
        detection can peek then rewind.
      - `pub fn width(&self) -> u32`, `pub fn height(&self) -> u32`
      - `pub fn source_format(&self) -> ::image::ImageFormat`
      - `pub fn metadata(&self) -> Option<&MetadataBundle>`
      - `pub fn info(&self) -> ImageInfo`
    - `pub struct ImageInfo` — read-only inspection (what `info` reports
      later). Public fields per `docs/data-model.md`:
      - `width: u32`, `height: u32`
      - `format: ::image::ImageFormat`
      - `color_type: ::image::ColorType`
      - `bit_depth: u8` (bits per channel, derived from `ColorType`)
      - `has_alpha: bool` (derived from `ColorType`)
      - `byte_len: u64` (decoded in-memory byte length of the pixel buffer)
      - `has_icc: bool`, `has_exif: bool` (true iff the captured
        `MetadataBundle` carried that segment)
    - `pub struct MetadataBundle` — raw, **uninterpreted** container segments
      captured at load (DEC-003). Holds optional raw bytes:
      - `exif: Option<Vec<u8>>` — raw EXIF segment bytes, not parsed.
      - `icc: Option<Vec<u8>>` — raw ICC profile bytes, not parsed.
      Plus simple predicates `pub fn has_exif(&self) -> bool` /
      `pub fn has_icc(&self) -> bool`. **Do not parse, validate, or interpret
      the bytes here** — that is the metadata lane (STAGE-004). Capture is
      best-effort: use what `image`/the decoder exposes (e.g. ICC via the
      decoder's `icc_profile()`); if the chosen path cannot extract a segment
      for a given format, leaving it `None` is acceptable for this spec as long
      as the JPEG-with-EXIF acceptance test passes (see Notes for the
      Implementer for the recommended extraction approach).
  - `tests/image_load.rs` — integration tests (see `## Failing Tests`).
  - A test-only fixture helper for synthesizing tiny images in memory
    (solid/gradient) and encoding them to PNG/JPEG byte vectors. Place it
    where the failing tests can use it — either a `#[cfg(test)]` helper inside
    the integration test file, or `tests/common/mod.rs` (Cargo treats files
    under a `tests/<dir>/` as a shared module, not a separate test binary).
    **Generate fixtures natively** with the `image` crate's encoders — do NOT
    shell out to ImageMagick and do NOT commit binary fixtures.
- **Files modified:**
  - `Cargo.toml` — add `image` to `[dependencies]` and `thiserror` to
    `[dependencies]`. Pin exact patch versions (AGENTS.md §5: `image` 0.25,
    `thiserror` 2). Configure `image` features for the pure-Rust default set
    (see Notes for the Implementer for the recommended feature list).
  - `src/lib.rs` — add `pub mod error;` and `pub mod image;` (keep the
    existing `version()` fn + its test).
- **New exports:**
  - `crustyimg::error::{ImageError, Result}`
  - `crustyimg::image::{Image, ImageInfo, MetadataBundle}`
- **Database changes:** none.

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] `Image::load` and `Image::from_bytes` decode a valid **PNG** and report
      the correct `width`/`height` and `source_format == ImageFormat::Png`.
- [ ] `Image::from_bytes` decodes a valid **JPEG** and reports the correct
      `width`/`height` and `source_format == ImageFormat::Jpeg`.
- [ ] Loading a **bogus / non-image byte slice** returns
      `Err(ImageError::Decode | ImageError::UnsupportedFormat)` — a typed
      error, **not** a panic — and decoding truncated/corrupt data of a known
      format returns `Err(ImageError::Decode)` (no panic).
- [ ] `Image::load` on a **path that does not exist** returns
      `Err(ImageError::Io(_))` (not a panic).
- [ ] When loading a **JPEG that carries an EXIF segment**, the resulting
      `Image::metadata()` is `Some(_)` and `metadata().unwrap().has_exif()`
      is `true`; `ImageInfo.has_exif` is `true`.
- [ ] When loading an image with **no metadata** (e.g. a freshly-generated
      PNG with no ICC/EXIF), `Image::metadata()` is `None` **or** a bundle
      whose `has_exif()`/`has_icc()` are both `false`, and `ImageInfo.has_exif`
      / `has_icc` are `false`. (The spec accepts either representation of
      "absent"; the test asserts the predicates, not `Option` shape.)
- [ ] `ImageInfo` fields are correct for a known generated image:
      `width`/`height` match the source, `color_type` matches the encoded
      color type (e.g. `Rgb8` for a solid RGB image, `Rgba8` for one with
      alpha), `bit_depth == 8`, `has_alpha` matches the color type, and
      `byte_len` equals the decoded pixel-buffer length.
- [ ] No `unwrap()`/`expect()`/`panic!()` on recoverable paths in
      `src/image/**` or `src/error.rs` (constraint
      `no-unwrap-on-recoverable-paths`).
- [ ] Exactly one pixel library is present: `image` (constraint
      `single-image-library`); no second pixel/image crate added.
- [ ] Default `cargo build`/`cargo test` succeed with no system libraries
      (pure-Rust feature set only — constraint `pure-rust-codecs-default`).
- [ ] `cargo clippy -- -D warnings` and `cargo fmt --check` are clean
      (constraint `clippy-fmt-clean`).
- [ ] Every new public fn has at least one test (constraint
      `every-public-fn-tested`).

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in **build** is
to make these pass. Fixtures are generated natively at test time (no
ImageMagick, no committed binaries). The recommended fixture helpers produce
in-memory byte vectors:

- `fn solid_png(w: u32, h: u32, rgb: [u8;3]) -> Vec<u8>` — encode a solid
  `RgbImage` to PNG bytes.
- `fn gradient_jpeg(w: u32, h: u32) -> Vec<u8>` — encode a gradient `RgbImage`
  to JPEG bytes.
- `fn rgba_png(w: u32, h: u32) -> Vec<u8>` — encode an `RgbaImage` (alpha) to
  PNG bytes.
- A helper that produces **JPEG bytes carrying a minimal EXIF segment** (see
  Notes for the Implementer — the simplest reliable route is to build the
  JFIF/EXIF APP1 segment by hand or splice a tiny known-good EXIF block into a
  generated JPEG; assert only that the bundle captured it, not its contents).

Test list:

- **`tests/image_load.rs`**
  - `"load_png_from_bytes_reports_dimensions_and_format"` — generate a
    `solid_png(7, 5, [10,20,30])`; `Image::from_bytes(&png)` is `Ok`; asserts
    `width()==7`, `height()==5`, `source_format()==ImageFormat::Png`.
  - `"load_png_from_path_reports_dimensions_and_format"` — write the generated
    PNG to a `tempfile`/`std::env::temp_dir()` path, `Image::load(path)` is
    `Ok`, same dimension/format assertions; remove the temp file after.
    (Use `std::env::temp_dir()` + a unique name; no `tempfile` crate — that
    would need a DEC.)
  - `"load_jpeg_from_bytes_reports_dimensions_and_format"` — generate a
    `gradient_jpeg(16, 9)`; asserts `Ok`, `width()==16`, `height()==9`,
    `source_format()==ImageFormat::Jpeg`.
  - `"bogus_bytes_return_typed_error_not_panic"` — `Image::from_bytes(b"not an image at all")`
    returns `Err`, and the variant is `ImageError::UnsupportedFormat` or
    `ImageError::Decode` (match on the enum; assert it is one of those two,
    not a string compare). Must not panic.
  - `"truncated_png_returns_decode_error"` — take valid PNG bytes, truncate to
    the first ~20 bytes (header present, body cut), assert
    `Err(ImageError::Decode)`.
  - `"missing_file_returns_io_error"` — `Image::load("/no/such/crustyimg-test-file.png")`
    returns `Err(ImageError::Io(_))` (match the variant); must not panic.
  - `"jpeg_with_exif_captures_metadata_bundle"` — generate JPEG bytes carrying
    an EXIF APP1 segment; `Image::from_bytes(&jpeg)` is `Ok`;
    `img.metadata().is_some()`; `img.metadata().unwrap().has_exif()` is `true`;
    `img.info().has_exif` is `true`.
  - `"plain_png_has_no_metadata"` — generate a `solid_png` with no metadata;
    after load, `img.metadata()` is `None` OR
    `!img.metadata().unwrap().has_exif() && !img.metadata().unwrap().has_icc()`;
    and `img.info().has_exif == false && img.info().has_icc == false`.
  - `"info_fields_correct_for_rgb8"` — `solid_png(4,3,[1,2,3])` → load → `info()`:
    `width==4`, `height==3`, `format==ImageFormat::Png`,
    `color_type==ColorType::Rgb8`, `bit_depth==8`, `has_alpha==false`,
    `byte_len == 4*3*3` (w*h*channels).
  - `"info_fields_correct_for_rgba8"` — `rgba_png(2,2)` → load → `info()`:
    `color_type==ColorType::Rgba8`, `has_alpha==true`, `bit_depth==8`,
    `byte_len == 2*2*4`.

- **`src/image/mod.rs`** (unit tests in a `#[cfg(test)] mod tests` block) —
  cover any pure helper fns directly so `every-public-fn-tested` holds even
  for non-load entry points:
  - `"info_derives_bit_depth_and_alpha_from_color_type"` — construct an
    `Image` from in-memory generated bytes (or a small helper) and assert the
    `ColorType`→(`bit_depth`,`has_alpha`) derivation for at least `Rgb8`
    (8, false) and `Rgba8` (8, true). (If the derivation is a free fn, test it
    directly.)
  - `"metadata_bundle_predicates"` — a `MetadataBundle { exif: Some(vec![1]), icc: None }`
    reports `has_exif()==true`, `has_icc()==false`; an empty bundle reports
    both `false`.

## Implementation Context

*Read this section (and the files it points to) before starting the build
cycle. It is the equivalent of a handoff document, folded into the spec since
there is no separate receiving agent.*

### Decisions that apply

- `DEC-002` — **Single canonical image model + `Operation` trait.** `Image` is
  the ONE in-memory model and wraps `image::DynamicImage`; `image` is the ONE
  pixel library. This spec builds exactly that model (the `Operation`/`Pipeline`
  half is SPEC-003). Do not add a second pixel crate.
- `DEC-003` — **Metadata dual-lane; capture at load.** Capture the raw
  EXIF/ICC bundle alongside the decoded pixels at load WITHOUT interpreting it,
  so the later preserve policy + metadata lane have it. This spec is the
  **capture** half; container-level editing is STAGE-004 and out of scope.
- `DEC-004` — **Pure-Rust codecs by default; `image` crate.** Use `image`'s
  pure-Rust decoders/encoders only (JPEG/PNG/GIF/BMP/TIFF/ICO). Native codecs
  (mozjpeg/avif/rexiv2) are off-by-default cargo features and are NOT touched
  here. Configure `image` features to the pure-Rust set (see Notes).
- `DEC-007` — **`thiserror` in the library; typed errors, no panic on
  recoverable paths.** This spec introduces the FIRST typed error enum
  (`ImageError`) + crate `Result` alias. The library returns typed errors;
  `anyhow` and exit-code mapping stay at the binary boundary (a later spec).
  No `unwrap`/`expect`/`panic!` on recoverable paths.

> **Dependency note (read this):** Adding `image` is normally gated by
> `no-new-top-level-deps-without-decision`, but it is **already justified by
> DEC-002 (single canonical model) and DEC-004 (`image` is the named default
> stack)** — so **no new DEC is required** for `image`, nor for `thiserror`
> (named in DEC-007). State "No new DEC — `image`/`thiserror` pre-justified by
> DEC-002/DEC-004/DEC-007" in the PR's "New decisions" section. If the build
> reaches for ANY other crate (e.g. `tempfile`, a separate EXIF parser, a
> second image crate), it MUST stop and emit a `DEC-*` first.

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `single-image-library` — exactly one pixel library (`image`) wrapped by
  `Image`; no second image-processing crate.
- `pure-rust-codecs-default` — default build uses only pure-Rust `image`
  codecs; no native-dep codec (keep CI trivially green on three OSes).
- `no-unwrap-on-recoverable-paths` — `src/image/**` and `src/error.rs` return
  typed errors; no `unwrap`/`expect`/`panic!` on recoverable paths. (`.unwrap()`
  in `#[cfg(test)]`/`tests/` setup is idiomatic and allowed.)
- `no-new-top-level-deps-without-decision` — `image` + `thiserror` are
  pre-justified (DEC-002/004/007); any OTHER new dep needs a DEC first.
- `clippy-fmt-clean` — `cargo clippy -- -D warnings` and `cargo fmt --check`
  must be clean; no dead code.
- `every-public-fn-tested` — every new public fn gets at least one test.
- `test-before-implementation` — the `## Failing Tests` are the contract;
  make them pass, do not rewrite them to fit the code (minor signature
  adjustments require updating the corresponding test, not deleting it).

### Prior related work

- `SPEC-001` (shipped, PR #1) — the empty-deps std-only scaffold: a `crustyimg`
  library crate (`src/lib.rs` with `pub fn version()` + unit test), a thin
  `src/main.rs`, `tests/smoke.rs`, three-OS CI (`.github/workflows/ci.yml`),
  `.gitignore` (`/target`). `[dependencies]` is currently EMPTY — this spec
  adds the first two. `Cargo.lock` is committed (binary-crate convention).

### Out of scope (for this spec specifically)

If any of these feels necessary during build, create/await the owning spec
rather than expanding this one.

- The `Operation` trait + `Pipeline` executor — **SPEC-003**. (This spec
  defines the `Image` they consume; it does not define ops or the fold.)
- `Source` (file/glob/dir/stdin) — **SPEC-004**. `from_reader` exists here for
  the stdin path to call later, but no source resolution / globbing.
- `Sink` (file/dir+template/stdout/display) — **SPEC-005**. No encode-to-output
  here; SPEC-002 only decodes/loads.
- `Recipe` + operation registry — **SPEC-006**.
- clap subcommands / CLI surface / `info` command wiring — **SPEC-007** /
  STAGE-002. `ImageInfo` is the data the future `info` command will print;
  this spec does not add the command.
- **ANY metadata editing / encode-preserve policy** — **STAGE-004**. This spec
  only **captures/carries** the raw bundle; it does not parse, edit, strip,
  clean, set, or re-emit metadata, and adds none of `kamadak-exif`/`img-parts`/
  `little_exif` (those land with the metadata lane).
- WebP / AVIF / native codecs (`mozjpeg`/`ravif`/`rexiv2`) and their cargo
  features — feature-gated, later (DEC-004).
- A unified crate-wide `Error` enum spanning recipe/metadata/sink errors — a
  later spec widens `error.rs` (DEC-007); SPEC-002 only needs `ImageError` +
  `Result`.

### Exact commands (from AGENTS.md §6)

```bash
cargo build                         # debug build (pure-Rust default features)
cargo test                          # all tests (unit + integration)
cargo test image_load               # just this spec's integration file
cargo clippy -- -D warnings         # lint, warnings are errors
cargo fmt --check                   # formatting gate (CI); `cargo fmt` to fix
```

## Notes for the Implementer

- **Module name collision:** the module is `image`, the crate is also `image`.
  Inside `src/image/mod.rs`, refer to the crate as `::image` (leading colons)
  or `use ::image::{DynamicImage, ImageFormat, ColorType};` at the top.
  Alternatively rename the crate in `Cargo.toml` via `image = { package =
  "image", ... }` is NOT needed — `::image` is cleaner and avoids surprises.
- **Recommended `image` features (pure-Rust default set, DEC-004):** disable
  default features and opt into only the pure-Rust core formats so the build
  stays system-lib-free and the dependency tree is lean. A good starting point:
  `image = { version = "0.25", default-features = false, features = ["png",
  "jpeg", "gif", "bmp", "tiff", "ico"] }`. All of these are pure-Rust in
  `image` 0.25. (Note: `image`'s default features are already pure-Rust, so
  leaving defaults on is also acceptable and simpler — but trimming to the core
  MVP formats keeps build time down and makes the "pure-Rust only" intent
  explicit. Pick one and note it in Build Completion.) Do NOT enable `avif`
  (pulls native libaom via `ravif`) or any `mozjpeg`/native feature.
- **Format detection:** prefer `image::ImageReader::new(Cursor::new(bytes))
  .with_guessed_format()?` then `.format()` (gives you the
  `Option<ImageFormat>`) and `.decode()`. For a path, `ImageReader::open(path)?
  .with_guessed_format()?` does both the IO and the sniff. Map a `None` format
  to `ImageError::UnsupportedFormat` and a decode failure to
  `ImageError::Decode`. `ImageReader::open` returns `io::Error` for a missing
  file → maps to `ImageError::Io`.
- **ICC capture:** in `image` 0.25 the format-specific decoders expose
  `icc_profile() -> ImageResult<Option<Vec<u8>>>`. If you decode through a
  concrete decoder you can pull the ICC bytes; through the convenience
  `ImageReader::decode()` path it is harder. It is acceptable for this spec to
  capture ICC best-effort and leave it `None` when the convenience path can't
  surface it — the ICC capture is fully exercised in STAGE-004, and no
  acceptance test here requires ICC bytes (only EXIF is asserted present).
- **EXIF capture is the load-bearing metadata test.** The `image` crate does
  NOT generally surface EXIF on decode. The pragmatic capture approach for
  SPEC-002 is to **sniff the raw container for the EXIF segment yourself
  without a new dependency** — e.g. for JPEG, scan the APP1 marker
  (`0xFF 0xE1`) for the `Exif\0\0` signature and copy that segment's bytes into
  `MetadataBundle.exif`. This is byte-scanning, not EXIF *parsing* (DEC-003:
  capture raw, do not interpret), and needs no extra crate. Keep it small and
  format-scoped (JPEG APP1 is enough to satisfy the acceptance test; PNG `eXIf`
  chunk is a nice-to-have). Document whatever you implement in Build Completion.
- **Generating an EXIF-bearing JPEG fixture without a dependency:** build the
  JPEG bytes from the generated gradient JPEG and splice a minimal valid APP1
  EXIF segment (`0xFF 0xE1`, 2-byte big-endian length, `Exif\0\0`, then a
  tiny TIFF header `II*\0` + zero IFD) right after the SOI (`0xFF 0xD8`). The
  test only needs your capture to detect the `Exif\0\0` signature and record
  the segment — it does not assert the EXIF contents. Put this fixture builder
  in the shared test helper. (If splicing proves fiddly on JPEG, the PNG `eXIf`
  chunk is simpler to synthesize — generate a PNG and insert an `eXIf` chunk
  before `IEND`; then have your capture recognize the PNG `eXIf` chunk instead.
  Either format satisfies the "EXIF captured when present" criterion; pick the
  one your capture path reads and keep the fixture + capture in the same format.)
- **`bit_depth` / `has_alpha` from `ColorType`:** `image::ColorType` has
  `bytes_per_pixel()`, `channel_count()`, `has_alpha()`, and `bits_per_pixel()`.
  Derive `bit_depth` as bits-per-channel (e.g. `Rgb8`/`Rgba8` → 8; `Rgb16` →
  16). A small free fn `fn color_type_bit_depth(ct: ColorType) -> u8` is clean
  and unit-testable. `has_alpha` can use `ColorType::has_alpha()`.
- **`byte_len`:** the decoded pixel buffer length. `DynamicImage::as_bytes()`
  gives `&[u8]`; `byte_len = pixels.as_bytes().len() as u64`. This is the
  in-memory decoded size, not the encoded file size (file size is a Source/Sink
  concern).
- **Keep the pixel core clean (architecture layering):** `src/image/**` must
  not reference `clap`, `std::process`, terminals, or recipe/source/sink types.
  It depends only on `::image`, `std`, and `crate::error`.
- **No temp-file crate:** the path-load test should use `std::env::temp_dir()`
  + a unique filename and clean up; do not add `tempfile` (would need a DEC).
- **Test `.unwrap()` is fine.** The `no-unwrap-on-recoverable-paths` constraint
  is scoped to `src/**` library code; `tests/` and `#[cfg(test)]` setup may use
  `.unwrap()`/`.expect()` idiomatically.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-002-canonical-image-type-and-load`
- **PR (if applicable):** see timeline (opened on jysf/crustyimg)
- **All acceptance criteria met?** yes
- **`image` feature configuration chosen:** `image = { version = "=0.25.10",
  default-features = false, features = ["png", "jpeg", "gif", "bmp", "tiff",
  "ico"] }` — default features OFF, only the pure-Rust core MVP formats
  (DEC-004). No `avif`/`mozjpeg`/native features. `thiserror = "=2.0.18"`. Both
  pinned to exact patch versions. `cargo tree --depth 1` shows exactly two
  direct deps (`image`, `thiserror`) and no native codec crate
  (ravif/mozjpeg/libaom/dav1d absent).
- **EXIF/ICC capture approach (DEC-003, capture-only):** raw byte-scanning of
  the container, NOT parsing. JPEG: walk the marker segments and copy the
  payload of the first APP1 (`FF E1`) segment starting with `Exif\0\0` into
  `MetadataBundle.exif`; ICC is best-effort from the first APP2 (`FF E2`)
  `ICC_PROFILE\0` segment. PNG: copy the raw `eXIf` chunk data into `exif` and
  the `iCCP` chunk data into `icc`. Bytes are stored verbatim; nothing is
  parsed/validated/interpreted. When no segment is present the bundle is empty
  and `Image::metadata()` returns `None`. The EXIF-bearing fixture is a
  generated gradient JPEG with a hand-spliced minimal valid APP1/`Exif\0\0`
  segment (II*\0 + zero-entry IFD); the test asserts only that capture detected
  it, never its contents.
- **New decisions emitted:**
  - None. `image`/`thiserror` are pre-justified by DEC-002/DEC-004/DEC-007;
    no other crate was added (no `tempfile`, no separate EXIF parser).
- **Deviations from spec:**
  - None material. `Image::load` reads the file via `std::fs::read` (mapping
    `io::Error` → `ImageError::Io`) then routes through `from_bytes`, rather
    than `ImageReader::open(...).with_guessed_format()`. This is functionally
    equivalent for the acceptance tests, keeps a single decode+capture code
    path, and lets the metadata byte-scan see the full container for both the
    path and bytes entries. `from_reader` likewise reads to end then delegates.
  - Added two non-required-but-useful public accessors not enumerated in the
    spec's signature list: `Image::pixels(&self) -> &DynamicImage` (downstream
    operations in SPEC-003 will need pixel access) — covered by a unit test to
    keep `every-public-fn-tested` true.
- **Follow-up work identified:**
  - Multi-chunk JPEG ICC reassembly and broader-format metadata capture are
    deferred to the metadata lane (STAGE-004), as the spec scopes. No new
    backlog spec needed now.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Very little. The one genuine fork was the EXIF-capture route: the spec
   offered both JPEG-APP1-splice and PNG-eXIf fixtures and said "pick the one
   your capture reads." I implemented capture for both formats and chose the
   JPEG APP1 fixture for the acceptance test, which removed the ambiguity but
   took a moment to decide. The exact `ColorType` API surface for deriving
   `bit_depth` (`bits_per_pixel()`/`channel_count()`) was the only thing I had
   to confirm against the crate rather than the spec.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The constraint set was complete and accurate for the paths touched.
   The dependency note pre-clearing `image`/`thiserror` (so no DEC churn) was
   especially helpful and prevented a needless DEC.

3. **If you did this task again, what would you do differently?**
   — Write the JPEG/PNG segment-walking helpers test-first as pure unit tests
   in `src/image` before wiring them into `capture()`, rather than relying on
   the integration fixture to exercise them — the marker-walk logic (standalone
   vs length-bearing markers, SOS bail-out) is the fiddliest part and deserves
   isolated coverage. I added a plain-PNG `capture()` unit test but could have
   gone further on the JPEG walker.

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
