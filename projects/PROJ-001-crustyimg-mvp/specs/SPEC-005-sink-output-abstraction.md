---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-005
  type: story                      # epic | story | task | bug | chore
  cycle: verify                    # frame | design | build | verify | ship
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
  implementer: claude-sonnet-4-6   # build runs on Sonnet 4.6, separate session
  created_at: 2026-06-14

references:
  decisions:
    - DEC-002   # canonical Image (Sink encodes Image::pixels())
    - DEC-007   # typed thiserror in lib; anyhow + exit codes at the binary
    - DEC-004   # pure-Rust codecs by default; encode via `image` only
    - DEC-011   # viuer for the terminal-display sink (new top-level dep)
    # DEC-003 — note only: the default-preserve/drop-GPS metadata policy
    # happens at encode, which is here, BUT the metadata re-attach lane is
    # STAGE-004. SPEC-005 only encodes pixels via `image` and must not
    # actively destroy the Image's captured metadata bundle.
  constraints:
    - untrusted-input-hardening
    - ergonomic-defaults
    - no-unwrap-on-recoverable-paths
    - no-new-top-level-deps-without-decision
    - clippy-fmt-clean
    - every-public-fn-tested
    - test-before-implementation
  related_specs:
    - SPEC-002   # Image type + DynamicImage::write_to encode path + ImageError
    - SPEC-004   # Source::Input — yields the stem the name-template expands
    - SPEC-003   # Pipeline returns the final Image a Sink writes

# One sentence on what this spec contributes to its stage's
# value_contribution.
value_link: >
  Infrastructure enabling STAGE-001's "decode once -> ops -> emit to a sink"
  keystone — the output half of the pipeline (file / dir+template / stdout /
  terminal display), with the output path-traversal + overwrite hardening the
  project thesis depends on.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 45
      recorded_at: 2026-06-14
      notes: "subagent; cost not separately reported"
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 35
      recorded_at: 2026-06-14
      notes: "subagent; cost not separately reported"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-005: Sink output abstraction

## Context

This spec builds the **output half** of the STAGE-001 pixel-lane keystone:
`decode once -> ordered ops -> emit to a sink`. SPEC-002 gave us the canonical
[`Image`]; SPEC-003 gave us the `Pipeline` that returns a final `Image`;
SPEC-004 gave us the `Source` that yields ordered [`Input`]s (each carrying a
`stem`). The missing piece is the `Sink`: take a final `Image` and write it
somewhere safe.

It is the largest item in the STAGE-001 backlog because it folds together four
output shapes named in `docs/architecture.md` (§ Components -> Sink) and
`docs/api-contract.md` (Global Options + `stdin/stdout`):

- **file** — a specific output path (`-o <PATH>`);
- **dir + name-template** — write into `--out-dir` using a template
  (`{stem}`, `{ext}`, `{name}`, `{parent}`, e.g. `{stem}_web.{ext}`) derived
  from the input's stem (pairs with SPEC-004's `Input::stem()`);
- **stdout** (`-o -`) — write the encoded bytes to stdout, keeping all
  diagnostics on stderr so a pipe stays clean;
- **terminal display** (viuer) — render the image in the terminal; refuse on a
  non-tty.

It is also where the output-side **untrusted-input-hardening** lives: a
name-template/path that resolves outside `--out-dir` (path traversal) is
rejected with a typed error, and an existing file is **not overwritten without
`--yes`**. Failures surface as a typed `SinkError` the binary maps to **exit
code 5** (per `docs/api-contract.md`).

Parent stage: `STAGE-001` (foundation and pipeline core), backlog item
SPEC-005. Project: `PROJ-001` (crustyimg MVP).

## Goal

Implement the `Sink` output abstraction in `src/sink/`: encode a final `Image`
(format inferred from the output extension, or an explicit format) via the
existing `image` crate and write it to one of file / dir+name-template /
stdout / terminal display — rejecting path-traversal and refusing to overwrite
without `--yes`, all as typed errors and with zero panics on recoverable paths.

## Inputs

- **Files to read:**
  - `src/image/mod.rs` — the canonical [`Image`]; you encode `img.pixels()`
    (a `&DynamicImage`) via `DynamicImage::write_to(&mut writer, format)`. There
    is **no** `Image::save`/`Image::encode` method yet — Sink adds the encode
    call. Note `Image::source_format()` and `Image::metadata()`.
  - `src/source/mod.rs` — `Input::stem()` (the `{stem}` token; never contains a
    separator) and `Input::path()` (`Some(&Path)` for files, used to derive
    `{name}`/`{parent}`; `None` for stdin).
  - `src/error.rs` — the `ImageError` + crate `Result` pattern to **mirror**
    for a module-local `SinkError` (do NOT widen the crate `Error`; follow how
    SPEC-004 kept `SourceError` local to `src/source/`).
  - `docs/data-model.md` § "Name templates (Sink)" — token table + default
    template `{stem}.{ext}`.
  - `docs/api-contract.md` § Global Options + "stdin / stdout (`-`)" + Exit
    Codes (5 = output write failed / refused / traversal).
  - `SECURITY.md` § "Path traversal on output".
- **External crates:** `image` 0.25 (already a dep — encode path);
  `viuer` (NEW top-level dep — see DEC-011 — terminal display, feature-gated);
  `tempfile` (existing dev-dep — test fixtures). `std::io::{Write, IsTerminal}`
  for the writer abstraction and tty detection (no extra crate for tty).
- **Related code paths:** `src/sink/` (new), `src/lib.rs`, `Cargo.toml`,
  `tests/sink.rs` (new).

## Outputs

- **Files created:**
  - `src/sink/mod.rs` — the `Sink` types, `SinkError`, the encode + write +
    name-template + traversal/overwrite logic, and `#[cfg(test)]` unit tests.
  - `tests/sink.rs` — integration tests through the public crate API
    (`crustyimg::sink::...`) with `tempfile` fixtures.
  - `decisions/DEC-011-viuer-terminal-display.md` — **already written at
    design time** (this cycle); the build only references it.
- **Files modified:**
  - `Cargo.toml` — add `viuer` as an **optional** dep + a `display` feature
    (pin the exact latest version; justified by DEC-011).
  - `src/lib.rs` — add `pub mod sink;`.
- **New exports (exact API the build must produce):**

  ```rust
  // src/sink/mod.rs

  /// Where a final Image is written. Constructed by the (future) CLI; here it
  /// is the public output contract the pipeline hands a final Image to.
  pub enum Sink {
      /// Write to one explicit output path (`-o <PATH>`). `None` extension on
      /// the path with no explicit `format` is a SinkError::UnknownFormat.
      File { path: PathBuf, format: Option<ImageFormat> },
      /// Write into `dir` using `template` over the input's stem
      /// (`{stem}_web.{ext}`). `format` (if Some) overrides extension inference.
      Dir { dir: PathBuf, template: String, format: Option<ImageFormat> },
      /// Write encoded bytes to stdout (`-o -`). `format` must be Some (there
      /// is no path to infer from) — None is SinkError::UnknownFormat.
      Stdout { format: Option<ImageFormat> },
      /// Render in the terminal via viuer. Refuses on a non-tty.
      Display,
  }

  /// Whether overwriting an existing destination file is permitted (`--yes`).
  /// (A small explicit enum reads better than a bare bool at call sites.)
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum Overwrite { Forbid, Allow }

  #[derive(Debug, thiserror::Error)]
  pub enum SinkError {
      #[error("could not write output")]
      Io(#[from] std::io::Error),
      #[error("could not encode image: {0}")]
      Encode(String),
      #[error("could not determine output format (no extension and no --format)")]
      UnknownFormat,
      #[error("unsupported output extension: {0}")]
      UnsupportedExtension(String),
      #[error("output path escapes the target directory: {0}")]
      Traversal(String),
      #[error("output file already exists (use --yes to overwrite): {0}")]
      AlreadyExists(String),
      #[error("terminal display requires a tty")]
      NotATty,
      #[error("terminal display failed: {0}")]
      Display(String),
  }

  impl Sink {
      /// Encode `img` and write it according to this sink, against the input
      /// naming context (for templating) and the overwrite policy. The `out`
      /// writer is used ONLY for the Stdout variant — injected so tests
      /// capture bytes without touching real stdout. Diagnostics never go to
      /// `out`.
      pub fn write(
          &self,
          img: &Image,
          input: &SinkInput<'_>,
          overwrite: Overwrite,
          out: &mut dyn std::io::Write,
      ) -> Result<(), SinkError>;
  }

  /// The naming context a Dir sink needs from the originating input.
  pub struct SinkInput<'a> {
      pub stem: &'a str,
      pub path: Option<&'a std::path::Path>,
  }

  // Free, directly-testable helpers (each needs at least one test):

  /// Infer an `image::ImageFormat` from a path's extension (case-insensitive),
  /// over the DEC-004 core set (png/jpg/jpeg/gif/bmp/tif/tiff/ico). An unknown
  /// extension is SinkError::UnsupportedExtension; no extension is
  /// SinkError::UnknownFormat.
  pub fn format_from_extension(path: &Path) -> Result<ImageFormat, SinkError>;

  /// The conventional extension string for a format (for `{ext}` expansion):
  /// Png->"png", Jpeg->"jpg", Gif->"gif", Bmp->"bmp", Tiff->"tiff", Ico->"ico".
  pub fn extension_for_format(format: ImageFormat) -> &'static str;

  /// Expand a name template over an input. Tokens: {stem} {ext} {name}
  /// {parent}. `ext` is the chosen output extension. Returns the final file
  /// NAME only (no directory) as a String.
  pub fn expand_template(
      template: &str,
      stem: &str,
      ext: &str,
      path: Option<&Path>,
  ) -> String;

  /// Join `dir` + an expanded file name and reject any result that escapes
  /// `dir` (path traversal). Canonicalize `dir`; ensure the resolved parent of
  /// the candidate stays within it; a `..`/absolute/separator escape is
  /// SinkError::Traversal.
  pub fn safe_join(dir: &Path, file_name: &str) -> Result<PathBuf, SinkError>;
  ```
- **Database changes:** none.

## Acceptance Criteria

Testable outcomes. Each maps to a test in `## Failing Tests`.

- [ ] **File sink writes a re-readable image.** `Sink::File { path, None }`
      with `path = out.png` writes bytes to `out.png`; the file exists and
      `Image::load(out.png)` decodes back to the same dimensions.
- [ ] **Format inferred from extension.** Writing to `out.jpg` produces a JPEG
      (decodes back as `ImageFormat::Jpeg`); writing to `out.png` produces a
      PNG. `format_from_extension` is case-insensitive (`OUT.PNG` -> Png).
- [ ] **Explicit format overrides extensionless path.** `Sink::File { path:
      "out", format: Some(Png) }` writes a PNG; `Sink::File { path: "out",
      format: None }` is `SinkError::UnknownFormat` (not a panic).
- [ ] **Unsupported extension is typed.** `out.xyz` with no explicit format ->
      `SinkError::UnsupportedExtension`.
- [ ] **Dir + template expansion.** `Sink::Dir { dir, "{stem}_web.{ext}",
      Some(Png) }` and an input whose `stem == "photo"` writes
      `dir/photo_web.png`, re-readable as an image. `expand_template` covers
      `{stem}`, `{ext}`, `{name}`, `{parent}`.
- [ ] **stdout writes only the encoded image.** `Sink::Stdout { Some(Png) }`
      with an injected `Vec<u8>` writer: after `write`, the captured bytes
      decode as a PNG image and **nothing else** is written to the writer
      (no diagnostics). `Sink::Stdout { None }` -> `SinkError::UnknownFormat`.
- [ ] **Overwrite guard.** Writing a File/Dir sink to an **existing** path with
      `Overwrite::Forbid` -> `SinkError::AlreadyExists`; with `Overwrite::Allow`
      it overwrites successfully. (stdout/display never trigger the guard.)
- [ ] **Path traversal rejected.** A Dir sink whose expanded name escapes
      `dir` — e.g. template `../{stem}.{ext}`, or `safe_join(dir,
      "../../etc/x.png")` — returns `SinkError::Traversal`; no file is written
      outside `dir`.
- [ ] **Missing/unwritable output dir is typed, not a panic.** A Dir sink whose
      `dir` does not exist (and is not created) -> a typed `SinkError`
      (`Io` or `Traversal`), never a panic/unwrap.
- [ ] **Display sink refuses on a non-tty.** Under `cargo test` (stdout is not
      a tty), calling `Sink::Display.write(...)` returns `SinkError::NotATty`
      rather than attempting to render.
- [ ] **Every new public fn is tested** and `cargo clippy -- -D warnings` +
      `cargo fmt --check` are clean; no `unwrap`/`expect`/`panic!` outside
      `#[cfg(test)]`.

## Failing Tests

Written during **design**, BEFORE build. The implementer makes these pass.
Unit tests live in a `#[cfg(test)] mod tests` in `src/sink/mod.rs`; integration
tests live in `tests/sink.rs` and go through the public crate API only. Use
`tempfile::tempdir()` for filesystem fixtures; produce real images in-memory by
encoding a solid `RgbImage` to PNG bytes (mirror `solid_png` in
`src/image/mod.rs` tests). `.unwrap()` is fine in tests.

- **`src/sink/mod.rs` (`#[cfg(test)] mod tests`)** — unit tests of the free
  helpers:
  - `format_from_extension_is_case_insensitive` — asserts: `out.png`,
    `out.PNG`, `out.JpG` map to `Png`/`Png`/`Jpeg`; `out` (no ext) ->
    `Err(UnknownFormat)`; `out.xyz` -> `Err(UnsupportedExtension)`.
  - `extension_for_format_round_trips` — asserts: `extension_for_format(Jpeg)
    == "jpg"`, `...(Png) == "png"`, and feeding each back through
    `format_from_extension` of `foo.{ext}` yields the same format.
  - `expand_template_expands_all_tokens` — asserts: with `stem="photo"`,
    `ext="png"`, `path=Some("/a/b/photo.jpg")`, the template
    `"{stem}_web.{ext}"` -> `"photo_web.png"`; `"{name}"` -> `"photo.jpg"`;
    `"{parent}"` -> `"b"`; an unknown `{token}` is left literal (documented).
  - `safe_join_accepts_in_dir_name` — asserts: `safe_join(dir, "photo.png")`
    yields `dir/photo.png` and `starts_with(canonicalized dir)`.
  - `safe_join_rejects_parent_escape` — asserts: `safe_join(dir, "../x.png")`,
    `safe_join(dir, "../../etc/passwd")`, and an absolute `safe_join(dir,
    "/etc/x.png")` each -> `Err(Traversal)`.

- **`tests/sink.rs` (integration, public API)**:
  - `file_sink_writes_readable_image` — write `Sink::File { tmp/out.png, None
    }`; assert the file exists and `Image::load` decodes to the source
    dimensions.
  - `format_inferred_from_extension_jpeg_and_png` — write the same `Image` to
    `out.jpg` and `out.png`; assert each re-loads with the matching
    `source_format()`.
  - `explicit_format_overrides_missing_extension` — `Sink::File { tmp/out,
    Some(Png) }` writes a PNG (re-loads as Png); `Sink::File { tmp/out, None }`
    -> `matches!(err, SinkError::UnknownFormat)`.
  - `unsupported_extension_is_typed_error` — `Sink::File { tmp/out.xyz, None }`
    -> `matches!(err, SinkError::UnsupportedExtension(_))`.
  - `dir_sink_expands_name_template` — `Sink::Dir { tmp, "{stem}_web.{ext}",
    Some(Png) }` with `SinkInput { stem: "photo", path:
    Some(Path::new("in/photo.jpg")) }`; assert `tmp/photo_web.png` exists and
    re-loads.
  - `stdout_sink_writes_only_encoded_bytes` — `Sink::Stdout { Some(Png) }` with
    `out = &mut Vec::<u8>::new()`; after `write`, assert the captured Vec
    decodes via `Image::from_bytes` as a Png AND the Vec length equals the
    encoded image length (no trailing diagnostic bytes). `Sink::Stdout { None
    }` -> `UnknownFormat`.
  - `overwrite_guard_forbids_then_allows` — pre-create `tmp/out.png`; writing
    `Sink::File { tmp/out.png, None }` with `Overwrite::Forbid` ->
    `matches!(err, SinkError::AlreadyExists(_))`; the SAME write with
    `Overwrite::Allow` returns `Ok` and changes the file.
  - `dir_sink_rejects_traversal_template` — `Sink::Dir { tmp,
    "../{stem}.{ext}", Some(Png) }` -> `matches!(err, SinkError::Traversal(_))`;
    assert no file was created in `tmp`'s parent.
  - `missing_out_dir_is_typed_not_panic` — `Sink::Dir { tmp/does_not_exist,
    "{stem}.{ext}", Some(Png) }` -> an `Err(SinkError::_)` (do not assert which
    variant beyond "is a SinkError"); the test must not panic.
  - `display_sink_refuses_non_tty` — `Sink::Display.write(...)` under
    `cargo test` (piped, non-tty) -> `matches!(err, SinkError::NotATty)`. This
    is the only display assertion (rendering itself is not unit-testable).

## Implementation Context

*Read this section (and the files it points to) before starting the build
cycle. It is the handoff document, folded into the spec.*

### Decisions that apply

- `DEC-002` — one canonical `Image` over `image::DynamicImage`; Sink encodes
  `img.pixels()` via the single pixel library — **no second image crate**.
- `DEC-004` — pure-Rust codecs by default; encode only via `image`'s pure-Rust
  encoders over the core set (png/jpeg/gif/bmp/tiff/ico). No mozjpeg/avif here;
  AVIF/native are feature-gated elsewhere and out of scope.
- `DEC-007` — typed `thiserror` errors in the library (`SinkError`); the binary
  (a later spec) maps them to exit codes (5 for output write/refused/traversal).
  No `unwrap`/`expect`/`panic!` on recoverable paths.
- `DEC-011` — **new top-level dep `viuer`** for the terminal-display sink,
  written this design cycle; see `decisions/DEC-011-viuer-terminal-display.md`.
  The display sink is **feature-gated behind an off-by-default `display` cargo
  feature** so viuer's transitive deps never burden the default build/CI
  (rationale in DEC-011). Build it accordingly (see Notes).
- `DEC-003` — **note only.** The default-preserve/drop-GPS policy happens at
  encode time, and encode happens here — BUT the metadata re-attach lane is
  **STAGE-004**. SPEC-005 encodes pixels only (the `image` crate drops metadata
  on encode by nature). Do **not** implement preserve/drop-GPS here; do **not**
  actively destroy `Image::metadata()` (leave the bundle on the `Image`; you
  just don't re-attach it). The preserve policy is a STAGE-004 concern that
  will hook the encode path later.

### Constraints that apply

(see `/guidance/constraints.yaml` for full text)

- `untrusted-input-hardening` — **the headline of this spec.** Output sinks
  must (1) reject any name-template/path value that resolves outside the target
  `--out-dir` (path traversal) and (2) not overwrite an existing file without
  `--yes`. Both are typed `SinkError`s, never panics. **Traversal rule:**
  canonicalize the target `dir`; resolve the candidate file's parent and assert
  it `starts_with` the canonicalized `dir`; reject `..` segments, embedded path
  separators from a template/stem, and absolute components -> `SinkError::
  Traversal`. **Overwrite rule:** for File/Dir, if the destination already
  exists and `Overwrite::Forbid`, return `SinkError::AlreadyExists`; only
  `Overwrite::Allow` (CLI `--yes`) overwrites.
- `ergonomic-defaults` — the default template is `{stem}.{ext}` and the simple
  `-o out.png` path needs no extra flags; complexity (templates, out-dir) is
  opt-in. Don't require boilerplate for the single-file case.
- `no-unwrap-on-recoverable-paths` — typed `SinkError` everywhere in library
  code; `.unwrap()`/`.expect()` only inside `#[cfg(test)]` and `tests/`.
- `no-new-top-level-deps-without-decision` — `viuer` is the ONLY new top-level
  dep and is justified by **DEC-011** (already written). Add nothing else
  (no `atty`/`is-terminal` crate — use `std::io::IsTerminal`, stable since Rust
  1.70; no new dev-dep beyond the existing `tempfile`).
- `clippy-fmt-clean`, `every-public-fn-tested`, `test-before-implementation` —
  standard STAGE-001 gates.

### Prior related work

- `SPEC-002` (shipped, PR #2) — `Image`, `Image::load`/`from_bytes`,
  `Image::pixels()` (`&DynamicImage`), `source_format()`, `metadata()`. The
  encode primitive you call is `image::DynamicImage::write_to(&mut writer,
  ImageFormat)` (used in SPEC-002's own tests). There is **no** `Image::save`
  yet — Sink owns the encode call.
- `SPEC-004` (shipped, PR #4) — `source::Input` with `stem()` (never contains a
  separator — but a malicious `--name-template` still can, hence the traversal
  guard) and `path()`. `SourceError` is module-local — mirror that pattern for
  `SinkError`.
- `SPEC-003` (shipped, PR #3) — `Pipeline` returns the final `Image` that a
  `Sink` writes.
- `DEC-010` — set the precedent (this design cycle's DEC-011 mirrors it) for a
  one-crate dependency add with a written DEC.

### Out of scope (for this spec specifically)

If any of these feel necessary during build, STOP and add a question rather
than expanding this spec:

- **The metadata preserve policy / container-lane re-attach on encode**
  (orientation/ICC/copyright preserve, drop-GPS) — **STAGE-004**. Encode pixels
  only here.
- Recipes / operation registry — SPEC-006.
- CLI wiring (`clap`, parsing `-o`/`--out-dir`/`--name-template`/`--yes`/
  `--format`/`--quality` into a `Sink`/`Overwrite`) — SPEC-007. SPEC-005
  defines the `Sink`/`Overwrite`/`SinkInput` types the CLI will construct.
- Parallel batch writing (rayon, progress) — STAGE-005. Sink writes **one**
  image; the batch loop calls `Sink::write` per input later.
- Encoder **quality** (`-q`/JPEG quality) plumbing — keep encode at the
  `image` default for the chosen format. Do NOT add a quality flag/param
  surface — that is CLI/STAGE-003 territory.
- WebP/AVIF/native codecs (DEC-004 feature gates) — out.
- Actual transforms / operations — other stages.

## Notes for the Implementer

- **Encoding an `Image`:** there is no `Image::save`. Get the pixels with
  `img.pixels()` (`&DynamicImage`) and call
  `pixels.write_to(&mut writer, format)` — map its `image::ImageError` to
  `SinkError::Encode(e.to_string())`. For File/Dir, the writer is a
  `BufWriter<File>`; for Stdout, it is the injected `&mut dyn Write`. **Keep all
  diagnostics off the `out` writer** — `out` carries ONLY encoded image bytes
  so `-o -` pipes stay clean (AGENTS.md §11: diagnostics to stderr).
- **Why `out: &mut dyn Write` for stdout (testability):** do NOT write to
  `std::io::stdout()` directly inside `Sink::write`. Take the writer as a
  parameter so the test passes a `&mut Vec<u8>` and asserts on the captured
  bytes. The real CLI (SPEC-007) will pass `std::io::stdout().lock()`.
- **Format inference:** `format_from_extension` lowercases the extension and
  matches the DEC-004 core set; `jpg`/`jpeg`->`Jpeg`, `tif`/`tiff`->`Tiff`. No
  extension -> `UnknownFormat`; an extension not in the set ->
  `UnsupportedExtension`. `Stdout` with `format: None` is `UnknownFormat` (no
  path to infer from). Use `image::ImageFormat` directly — do not invent a
  format enum.
- **Traversal canonicalization (the trickiest bit):** `safe_join` should
  canonicalize the `dir` (it must exist; a missing dir is a typed error — do
  NOT create it). Reject the candidate if the expanded file name contains a
  path separator, a `..` component, or is absolute, AND verify the final joined
  path's parent canonicalizes to within the canonicalized `dir`. Compare
  CANONICALIZED paths on both sides (`starts_with`). On Windows, canonicalize
  adds a `\\?\` verbatim prefix to BOTH sides, so `starts_with` still holds —
  never compare a raw arg to a canonical path. `Input::stem()` never contains a
  separator, but a user-supplied `--name-template` can, so the guard is on the
  EXPANDED name, not the stem.
- **TTY detection:** use `std::io::stdout().is_terminal()` (`use
  std::io::IsTerminal;`). Under `cargo test` stdout is piped (non-tty), so
  `Sink::Display.write(...)` returns `SinkError::NotATty` — that is the only
  display behavior the tests pin. The actual viuer render call lives behind the
  `display` cargo feature; when the feature is OFF, `Sink::Display` still
  compiles and still returns `NotATty` on a non-tty (the refusal path is
  feature-independent so the test always runs). See DEC-011 for the exact
  feature shape.
- **Feature gating (`display`):** add `[features] display = ["dep:viuer"]` to
  `Cargo.toml` and make `viuer` an OPTIONAL dependency
  (`viuer = { version = "=X.Y.Z", optional = true }`). The viuer **call** is
  `#[cfg(feature = "display")]`; the `NotATty` refusal and the `Sink::Display`
  variant are always compiled. This keeps default CI free of viuer's transitive
  (and possibly native) deps. See DEC-011 for the rationale and the
  transitive-dep note.
- **Overwrite check:** test existence with `Path::exists()` BEFORE opening for
  write; if it exists and `Overwrite::Forbid`, return `AlreadyExists` without
  truncating. Only File and Dir sinks check this.
- **`{ext}` in templates:** the expanded extension is the chosen output
  extension string (`extension_for_format(format)` when a format is set, else
  the dir-sink default `png`). Decide the format BEFORE expanding the template
  so `{ext}` is correct.
- **Reuse:** mirror the `solid_png` test helper from `src/image/mod.rs` for
  fixtures; mirror SPEC-004's module-local error enum style.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-005-sink-output-abstraction`
- **PR (if applicable):** see PR opened after this commit
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - No new DEC — DEC-011 (viuer + `display` feature) was written at design time.
- **Deviations from spec:**
  - `viuer::print` returns `Result<(u32, u32), _>` not `Result<(), _>`; added
    `.map(|_| ())` to discard the terminal-dimensions return value and match the
    `-> Result<(), SinkError>` signature. Not a spec deviation — the spec said
    "map any viuer error to SinkError::Display"; the return-type adaptation is
    an obvious implementation detail.
  - `ImageFormat::Pcx` is deprecated in `image` 0.25; removed it from the
    `extension_for_format` match arms (folded into the `_ => "bin"` fallback).
    No functional impact on the DEC-004 core set.
- **Follow-up work identified:**
  - CI job that builds `--features display` on all three OSes (per DEC-011
    "Consequences — Negative"; flagged there as a STAGE-001/006 follow-up).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — The `viuer::print` return type (`Result<(u32, u32), _>`) is not documented
   in the spec's notes; the build prompt says "map any viuer error" but the
   success-value discard needed one extra `.map(|_| ())` that clippy flagged
   only under `--features display`. Minor but worth noting for future display
   integrations: document the return type of the render function.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The `ImageFormat` enum in `image` 0.25 is `non_exhaustive`, which forces
   a `_ => ...` wildcard arm in `extension_for_format`. The spec lists the six
   core variants but doesn't mention that exhaustive matching is impossible;
   a brief note in the Implementation Context would have saved a clippy
   investigation about why the `Pcx` deprecation warning appeared.

3. **If you did this task again, what would you do differently?**
   — Write the `#[cfg(feature = "display")]` block as a single expression from
   the start (avoiding the `return Ok(());` clippy lint), and run
   `cargo clippy --features display` as the very first clippy invocation since
   the feature-gated code path isn't exercised by the default build. Both are
   easy to miss in a first pass.

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
