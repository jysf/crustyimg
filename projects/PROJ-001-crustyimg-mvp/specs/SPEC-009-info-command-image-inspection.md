---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-009
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # usually same Claude, different session
  created_at: 2026-06-14

references:
  decisions: [DEC-013, DEC-003, DEC-012, DEC-007]
  constraints:
    - no-new-top-level-deps-without-decision
    - pure-rust-codecs-default
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - test-before-implementation
    - untrusted-input-hardening
    - ergonomic-defaults
  related_specs: [SPEC-002, SPEC-007, SPEC-008]

# One sentence on what this spec contributes to its stage's
# value_contribution.
value_link: "Delivers STAGE-002's `info` capability — structured, read-only image inspection (dimensions/format/file-size/color-type/bit-depth/alpha/ICC+EXIF presence), EXIF tag dump, and the machine-readable `--json` output convention later commands reuse."

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
      recorded_at: 2026-06-15
      notes: "design cycle, Opus subagent"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-009: info command image inspection

## Context

This spec makes the `info` subcommand **real**. It is the **second and final**
STAGE-002 backlog item (after SPEC-008 `view`, shipped 2026-06-15, PR #8). It
replaces the `Commands::Info { .. } => Err(CliError::NotImplemented("info"))`
stub in `src/cli/mod.rs`.

- **Parent stage:** `STAGE-002` (view and info) — the first real, read-only
  commands. `info` is the structured-inspection half. See the stage's Design
  Notes: EXIF is **read-only** here via `kamadak-exif`; `--json` sets the
  structured-output convention (machine to stdout, diagnostics to stderr).
- **Project:** `PROJ-001` (crustyimg MVP).
- **Prior decisions:** DEC-013 (adds `kamadak-exif` always-on, read-only — emitted
  during *this* spec's design), DEC-003 (metadata dual-lane — `info` is on the
  read path only; the write lane is STAGE-004), DEC-012 (clap surface — the
  `Info` variant + its `--exif`/`--json` flags are already declared), DEC-007
  (typed errors → exit codes at the binary boundary).

The core facts are **already computed**: `Image::info()` (in `src/image/mod.rs`,
SPEC-002) returns an `ImageInfo { width, height, format, color_type, bit_depth,
has_alpha, byte_len, has_icc, has_exif }`. This spec mostly **formats** those
facts for human and JSON output, adds the **file-size-on-disk** headline, and
adds the **`--exif` tag dump** via `kamadak-exif`.

## Goal

Implement `crustyimg info <INPUT> [--exif] [--json]`: print an image's
dimensions, format, file size on disk, color type, bit depth, alpha, and
ICC/EXIF presence as human-readable text on stdout; `--json` emits the same
facts as machine-readable JSON on stdout (diagnostics on stderr); `--exif`
dumps EXIF tags read-only via `kamadak-exif`, treating "no EXIF" as success.

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — the `Commands::Info` variant, `dispatch()` (the stub line
    to replace), `run_view`/`run_apply` (structural templates: resolve → load →
    act), `CliError` + `code()` (reuse; do **not** add variants), the
    `exit_code_mapping_is_total` unit test (must stay green unchanged).
  - `src/image/mod.rs` — `Image::load` / `Image::from_bytes` (load entries),
    `Image::info() -> ImageInfo`, `ImageInfo` fields, `Image::metadata()`.
    Note: `ImageInfo` derives `Debug, Clone, PartialEq, Eq` but **not**
    `serde::Serialize`, and holds `image::ImageFormat` / `image::ColorType`
    (which are not `Serialize`). **Do not add serde derives to the pixel core.**
  - `src/source/mod.rs` — `source::resolve`, `Input::{Path, Stdin}`,
    `Input::stem()`, `Input::path()`, `SourceError::NotFound`.
  - `tests/cli.rs` — integration conventions (drive the real binary, trim
    stdout, assert `status.code()`); has a local `write_test_png`.
  - `tests/common/mod.rs` — native in-memory fixtures: `solid_png`,
    `gradient_jpeg`, `rgba_png`, and **`jpeg_with_exif(w,h)`** (valid APP1 EXIF;
    REUSE for the `--exif` test). NOTE: `tests/cli.rs` does **not** currently
    `mod common;` — the EXIF-fixture test must go in a test file that uses
    `common` (see Failing Tests).
- **External APIs:** `kamadak-exif` 0.6.1 (imported as `exif`) — read-only EXIF
  reader. Docs: https://docs.rs/kamadak-exif/0.6.1 . Entry point:
  `exif::Reader::new().read_from_container(&mut reader)`.
- **Related code paths:** `src/cli/` (the binary boundary — new stdout text and
  the serde DTO live here, NOT the pixel core).

## Outputs

- **Files modified:**
  - `Cargo.toml` — add the dependency line (DEC-013):
    `kamadak-exif = "=0.6.1"` under `[dependencies]` (always-on, NOT optional,
    NOT behind a feature). Add `serde_json = "=1.0.150"` under
    `[dev-dependencies]` (test-only JSON parsing/validation — see Notes; it is
    NOT a runtime dep, so no product DEC is required — call it out in the PR).
  - `src/cli/mod.rs` — replace the `info` stub with a real `run_info` handler;
    add the serde-serializable `InfoReport` DTO, the `ExifTag` DTO, the
    `format_label` / `color_type_label` mapping helpers, and the
    `read_exif_tags` helper. New unit tests in the existing `#[cfg(test)] mod
    tests`.
  - `tests/cli.rs` — add the `info` human-output, `--json`, `--exif <plain png>`,
    and `info <missing>` integration tests.
  - `tests/info_exif.rs` (**new** integration file) — the `--exif <jpeg_with_exif>`
    test that needs the shared `jpeg_with_exif` fixture; declares `mod common;`.
- **New exports / signatures** (all in `src/cli/mod.rs`; keep `run_info`,
  `read_exif_tags`, the DTOs, and the label helpers **private** — `fn`, not
  `pub fn` — they are binary-boundary internals; unit-test them via `super::`):

  ```rust
  /// CLI-local, serde-serializable inspection report (NOT the pixel-core
  /// ImageInfo, which is not Serialize and holds non-Serialize image:: types).
  /// Built from ImageInfo + the file-size-on-disk + the optional EXIF dump.
  #[derive(Debug, Clone, serde::Serialize)]
  struct InfoReport {
      /// Input path as given (or "-" for stdin).
      input: String,
      width: u32,
      height: u32,
      /// Stable lowercase format label, e.g. "png", "jpeg".
      format: String,
      /// Encoded file size on disk in bytes (NOT the decoded buffer length).
      file_size_bytes: u64,
      /// Decoded in-memory pixel-buffer length in bytes (distinct from file size).
      decoded_bytes: u64,
      /// Stable lowercase color-type label, e.g. "rgb8", "rgba8", "l8".
      color_type: String,
      /// Bits per channel (8, 16, ...).
      bit_depth: u8,
      has_alpha: bool,
      has_icc: bool,
      has_exif: bool,
      /// Present only when --exif is passed: the read EXIF tags (possibly empty).
      /// Omitted entirely (serde `skip_serializing_if`) when --exif is absent.
      #[serde(skip_serializing_if = "Option::is_none")]
      exif: Option<Vec<ExifTag>>,
  }

  /// One EXIF tag rendered for output (read-only; kamadak-exif, DEC-013).
  #[derive(Debug, Clone, serde::Serialize)]
  struct ExifTag {
      /// Tag name, e.g. "Make", "Orientation" (kamadak-exif's Tag Display).
      tag: String,
      /// Which IFD the tag came from, e.g. "primary", "thumbnail" (In Display).
      ifd: String,
      /// Human-readable value via Field::display_value().with_unit(&exif).
      value: String,
  }

  /// Map an `image::ImageFormat` to a stable lowercase label for output.
  /// Free fn so it is directly unit-testable; no panic on any variant.
  fn format_label(fmt: ::image::ImageFormat) -> String;

  /// Map an `image::ColorType` to a stable lowercase label, e.g. "rgb8".
  /// Free fn; unit-testable; no panic on any variant.
  fn color_type_label(ct: ::image::ColorType) -> String;

  /// Read EXIF tags from full container bytes (read-only, DEC-013). Returns an
  /// empty Vec when there is NO EXIF (`exif::Error::NotFound`) or the EXIF is
  /// malformed/unreadable — "no EXIF" is NOT an error. Never panics.
  fn read_exif_tags(bytes: &[u8]) -> Vec<ExifTag>;

  /// Emit the report as a single-line JSON object to `out` (the production
  /// `--json` path). Hand-rolled so the runtime crate set needs no `serde_json`
  /// (see Notes). Prints the documented keys in order, escaping string values.
  /// Total: no panic.
  fn write_json(report: &InfoReport, out: &mut impl std::io::Write) -> std::io::Result<()>;

  /// The `info` path: resolve the first input, load the image + raw bytes,
  /// build the report, and print human text or JSON to stdout.
  fn run_info(
      input: &str,
      exif: bool,
      json: bool,
      _global: &GlobalArgs,
  ) -> Result<(), CliError>;
  ```

- **Database changes:** none.

### Locked JSON schema (`info --json`)

`--json` prints **exactly one** JSON object to stdout, then a newline. Field
names and types are **stable** (this sets the structured-output convention later
`--json` commands reuse — per the stage notes). Diagnostics go to stderr; on
success stderr is empty.

| Field | Type | Notes |
|---|---|---|
| `input` | string | the input arg as given, or `"-"` for stdin |
| `width` | number (u32) | pixel width |
| `height` | number (u32) | pixel height |
| `format` | string | lowercase label, e.g. `"png"`, `"jpeg"` |
| `file_size_bytes` | number (u64) | **encoded file size on disk** (or stdin byte length) |
| `decoded_bytes` | number (u64) | decoded in-memory pixel-buffer length (distinct) |
| `color_type` | string | lowercase label, e.g. `"rgb8"`, `"rgba8"`, `"l8"` |
| `bit_depth` | number (u8) | bits per channel |
| `has_alpha` | bool | |
| `has_icc` | bool | ICC profile present (captured at load) |
| `has_exif` | bool | EXIF segment present (captured at load) |
| `exif` | array of `{tag, ifd, value}` (all strings) | **present only when `--exif`** is passed; an array (possibly empty); the key is **omitted** when `--exif` is absent |

Example (`info --json --exif` on a PNG with no EXIF):

```json
{"input":"photo.png","width":8,"height":8,"format":"png","file_size_bytes":91,"decoded_bytes":192,"color_type":"rgb8","bit_depth":8,"has_alpha":false,"has_icc":false,"has_exif":false,"exif":[]}
```

### Human output shape (no `--json`)

Plain `info` prints labeled lines to **stdout** (it is the requested output),
one fact per line, e.g.:

```
input:      photo.png
dimensions: 8x8
format:     png
file size:  91 bytes
color type: rgb8
bit depth:  8
alpha:      no
icc:        no
exif:       no
```

With `--exif`, append either `exif tags: (none)` or one `Tag: Value` line per
tag (using `Field::display_value().with_unit(&exif)`), e.g.:

```
exif:       yes
exif tags:
  Make: Canon
  Orientation: row 0 at top and column 0 at left
```

The exact label wording is the implementer's call **as long as** the
acceptance-criteria substrings below appear: the dimensions (`8x8` form), the
format label, a color-type label, and the EXIF/ICC presence are all assertable.

## Acceptance Criteria

Each criterion maps to a test in **Failing Tests**.

- [ ] AC1 — `info <png>` exits 0 and prints, on **stdout**, the dimensions
  (`8x8`), the format label (`png`), a color-type label, and ICC + EXIF presence
  lines. (human output) → `info_human_output_reports_core_facts`
- [ ] AC2 — `info --json <png>` exits 0; **stdout parses as a single JSON object**
  with all documented fields present and correct (`width`/`height`/`format`/
  `file_size_bytes`/`color_type`/`bit_depth`/`has_alpha`/`has_icc`/`has_exif`);
  **stderr is empty**; the `exif` key is **absent** (no `--exif`). →
  `info_json_is_parseable_and_complete`
- [ ] AC3 — `info --json --exif <plain png>` exits 0; stdout JSON includes an
  `exif` key that is an **empty array** (`[]`); `has_exif` is `false`; exit 0
  (no EXIF is not an error). → `info_json_exif_empty_array_on_plain_png`
- [ ] AC4 — `info --exif <jpeg_with_exif>` exits 0 and reports **EXIF present**
  (human output contains `exif:` + `yes`). The shared `jpeg_with_exif` fixture's
  IFD has **zero entries**, so the test asserts the command **succeeds and
  reports EXIF-present**, NOT that any specific tag exists. →
  `info_exif_reports_present_on_jpeg_with_exif` (in `tests/info_exif.rs`)
- [ ] AC5 — `info --exif <plain png>` exits 0 and reports **no EXIF** gracefully
  (human output contains `exif:` + `no`, or `(none)`); not an error. →
  `info_exif_on_plain_png_reports_none`
- [ ] AC6 — `info <missing>` exits **3** (input not found). →
  `info_missing_input_exits_3`
- [ ] AC7 — unit: `format_label` maps `Png`→`"png"`, `Jpeg`→`"jpeg"`,
  `Gif`→`"gif"`, `Bmp`→`"bmp"`, `Tiff`→`"tiff"`, `Ico`→`"ico"`; no panic. →
  `format_label_maps_core_formats`
- [ ] AC8 — unit: `color_type_label` maps `Rgb8`→`"rgb8"`, `Rgba8`→`"rgba8"`,
  `L8`→`"l8"`, `Rgb16`→`"rgb16"`; no panic. → `color_type_label_maps_color_types`
- [ ] AC9 — unit: `read_exif_tags` returns an **empty Vec** for plain PNG bytes
  and for malformed/truncated bytes (graceful, no panic); returns a Vec (length
  ≥ 0) for `jpeg_with_exif` bytes. → `read_exif_tags_graceful_on_no_exif`
- [ ] AC10 — unit: building an `InfoReport` from a known `ImageInfo` and
  serializing it with `serde_json` yields the documented fields with expected
  values (the `exif` key absent when `None`). → `info_report_serializes_fields`
- [ ] AC11 — the existing `exit_code_mapping_is_total`, `each_subcommand_help_parses`,
  and `help_lists_all_subcommands` tests stay **green** (no error-variant or
  arg-surface changes). → asserted by re-running the existing suite.

## Failing Tests

Written during **design**, BEFORE build. The implementer makes these pass.
Drive the real binary via `env!("CARGO_BIN_EXE_crustyimg")` + `std::process::Command`;
trim stdout (Windows `\r\n`); assert via `output.status.code()`. Under `cargo test`
the child's stdout is a pipe (non-tty) — fine for `info` (no tty requirement).

- **`tests/cli.rs`** (reuse the local `write_test_png` helper; `tempfile::tempdir()`):
  - `info_human_output_reports_core_facts` — write an 8x8 PNG; run `info <png>`.
    Asserts: `status.code() == Some(0)`; trimmed stdout (lowercased) contains
    `"8x8"`, `"png"`, a color-type label substring (`"rgb8"`), and both an `icc`
    and an `exif` line; `stderr` is empty.
  - `info_json_is_parseable_and_complete` — write an 8x8 PNG; run
    `info --json <png>`. Asserts: exit 0; `serde_json::from_slice::<serde_json::Value>(&output.stdout)`
    **succeeds** and is an object; `width == 8`, `height == 8`, `format == "png"`,
    `color_type == "rgb8"`, `bit_depth == 8`, `has_alpha == false`,
    `has_icc == false`, `has_exif == false`, `file_size_bytes` > 0,
    `decoded_bytes` > 0; the `"exif"` key is **absent** (`obj.get("exif").is_none()`);
    `output.stderr.is_empty()`.
  - `info_json_exif_empty_array_on_plain_png` — write an 8x8 PNG; run
    `info --json --exif <png>`. Asserts: exit 0; parsed JSON object has
    `exif` == an empty array (`obj["exif"].as_array().unwrap().is_empty()`);
    `has_exif == false`.
  - `info_exif_on_plain_png_reports_none` — write an 8x8 PNG; run `info --exif <png>`
    (no `--json`). Asserts: exit 0; trimmed stdout (lowercased) contains
    `"exif"` and indicates absence (`"no"` on the exif line, or `"(none)"`).
  - `info_missing_input_exits_3` — run `info <tempdir>/nope.png` (non-existent).
    Asserts: `status.code() == Some(3)`.

- **`tests/info_exif.rs`** (NEW file; first line `mod common;`, then
  `use common::jpeg_with_exif;` — this is the file that uses the shared fixture):
  - `info_exif_reports_present_on_jpeg_with_exif` — write `jpeg_with_exif(8,8)`
    bytes to a tempfile via `std::fs::write`; run `info --exif <jpeg>`. Asserts:
    `status.code() == Some(0)`; trimmed stdout (lowercased) contains `"exif"` and
    `"yes"` (EXIF-present). Does **NOT** assert any specific tag exists (the
    fixture's IFD is zero-entry). This pins "detect-and-report EXIF presence,
    succeed even with no readable tags."

- **`src/cli/mod.rs`** unit tests (in the existing `#[cfg(test)] mod tests`):
  - `format_label_maps_core_formats` — AC7 assertions.
  - `color_type_label_maps_color_types` — AC8 assertions.
  - `read_exif_tags_graceful_on_no_exif` — `read_exif_tags(&[])` is empty;
    `read_exif_tags(b"not an image")` is empty; `read_exif_tags(<plain png bytes>)`
    is empty; `read_exif_tags(<jpeg_with_exif bytes>)` returns a Vec (len ≥ 0),
    no panic. (Generate the PNG/JPEG bytes inline in the unit test as the
    pixel-core tests do, or via a small local helper — the unit test cannot reach
    `tests/common`.)
  - `info_report_serializes_fields` — construct an `InfoReport` with known field
    values and `exif: None`; `serde_json::to_value(&report)` (dev-dep) yields the
    documented fields with expected values and **no** `"exif"` key. Then construct
    one with `exif: Some(vec![])` and assert the `"exif"` key is present and an
    empty array.

The existing `exit_code_mapping_is_total`, `each_subcommand_help_parses`, and
`help_lists_all_subcommands` must STILL pass (no arg-surface / error-variant
change). Run the FULL `cargo test`.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-013` — adds `kamadak-exif` **always-on** (NOT feature-gated), read-only,
  via `exif::Reader::new().read_from_container(&mut Cursor::new(&bytes))` over
  the **full input bytes**; `exif::Error::NotFound` → graceful "no EXIF" (exit 0).
  Tag values via `Field::display_value().with_unit(&exif)`. Do NOT pull in the
  STAGE-004 write crates (`img-parts` / `little_exif`).
- `DEC-003` — metadata dual-lane: `info` is on the **read** path only. ICC/EXIF
  *presence* comes from the already-captured `ImageInfo.has_icc` / `has_exif`
  (no re-scan needed for presence). The `--exif` tag *dump* is the only new read.
- `DEC-012` — clap is the CLI framework; the `Info` variant and its
  `input` / `--exif` / `--json` args are **already declared** in `Commands` —
  do NOT change the arg surface.
- `DEC-007` — typed errors → exit codes at the binary boundary. Reuse the
  existing `CliError` + `code()` mapping (missing input → 3 via
  `SourceError::NotFound`; decode → 1; unsupported format → 4; io → 3). Do **not**
  add a `CliError` variant: EXIF read failures are handled gracefully inside
  `read_exif_tags` (return empty), never surfaced as an error, so no new mapping
  and `exit_code_mapping_is_total` stays unchanged.

### Constraints that apply

- `no-new-top-level-deps-without-decision` — DEC-013 satisfies it for
  `kamadak-exif`. `serde_json` is a **dev-dependency** (test-only) — a dev-dep
  does not need a product DEC, but it is called out here and in the PR body.
- `pure-rust-codecs-default` — `kamadak-exif` is pure-Rust (verified: only
  transitive dep is the pure-Rust `mutate_once`; no build script, no native libs)
  — safe to add always-on without a feature gate.
- `no-unwrap-on-recoverable-paths` — no `unwrap`/`expect`/`panic!` anywhere in
  `src/`. Use `?` for the resolve/load path; `read_exif_tags` swallows EXIF
  errors into an empty Vec (no panic, no propagation).
- `every-public-fn-tested` — the new fns (`format_label`, `color_type_label`,
  `read_exif_tags`, `InfoReport` build/serialize) each get a unit test (they are
  private, tested via `super::` in the module's `#[cfg(test)]`).
- `clippy-fmt-clean` — `cargo clippy -- -D warnings` and `cargo fmt --check` must
  be clean. The `_global` param is unused by `info` (no `-o`/`--format` for an
  inspection command) — keep the leading underscore as `run_view` does.
- `test-before-implementation` — these failing tests are the contract.
- `untrusted-input-hardening` — malformed/partial EXIF must degrade to an empty
  tag list, never panic; the image decode already sets limits in `Image::load`.
- `ergonomic-defaults` — `info <path>` with no flags is the common case and must
  Just Work (human output to stdout, exit 0).

### Prior related work

- `SPEC-008` (shipped 2026-06-15, PR #8) — `view` command. `run_view` in
  `src/cli/mod.rs` is the **structural template** for `run_info`: resolve the
  first input → load via `Image::load` / `Image::from_bytes` → act. `run_info`
  drops the Sink and prints to stdout instead.
- `SPEC-002` — `Image::info()` + `ImageInfo` + the load entries already exist;
  this spec consumes them.
- `SPEC-007` — the clap surface, `dispatch()`, `CliError` + `code()`, and the
  `exit_code_mapping_is_total` test.

### Out of scope (for this spec specifically)

If any of these feel necessary during build, write a new spec — do not expand
this one:

- **Any metadata WRITE** (strip/clean/set/copy-metadata) — STAGE-004,
  `img-parts` / `little_exif`. This spec is read-only.
- **`info` on multiple images / batch / fan-out** — `info` is single-image;
  resolve the **first** input only (mirror `run_view`).
- **ICC profile parsing** — report **presence only** (`has_icc` from the captured
  bundle); do NOT parse/inspect the ICC profile contents.
- **A new error variant, exit-code change, or arg-surface change.**
- **Faking a tty / rendering** — `info` has no tty requirement; it always prints.

### Notes for the Implementer

- **Gotcha — `ImageInfo` is not `Serialize`** and holds `image::ImageFormat` /
  `image::ColorType` (not Serialize). Do **not** derive serde on the pixel core.
  Build the CLI-local `InfoReport` DTO and map fields explicitly via
  `format_label` / `color_type_label`. The DTO lives in `src/cli/mod.rs`.
- **File size vs decoded buffer (resolved #3):** the **headline** size is the
  **encoded file size on disk** — `std::fs::metadata(path).len()` for an
  `Input::Path`, or `bytes.len() as u64` for `Input::Stdin`. The decoded
  in-memory pixel-buffer length (`ImageInfo.byte_len`) is surfaced as a
  **distinct** `decoded_bytes` field so the two are never confused. Do NOT change
  `ImageInfo`.
- **EXIF read (resolved #4):** call `exif::Reader::new().read_from_container(&mut
  std::io::Cursor::new(bytes))` on the **full input bytes** (file contents for a
  path, captured bytes for stdin). On `Ok(exif)`, map `exif.fields()` to
  `ExifTag { tag: f.tag.to_string(), ifd: f.ifd_num.to_string(),
  value: f.display_value().with_unit(&exif).to_string() }`. On
  `Err(exif::Error::NotFound(_))` **or any other Err**, return an empty Vec —
  "no EXIF" and "malformed EXIF" are both non-fatal. `read_from_container` wants
  the **whole container** (not the bare APP1 payload), which is why we pass the
  full input bytes rather than the captured `MetadataBundle.exif` (that payload
  carries the `Exif\0\0` prefix `read_raw` would reject).
- **`has_exif` vs the tag dump:** `has_exif` (presence) comes from the captured
  bundle and may be `true` even when `read_exif_tags` returns an empty Vec (e.g.
  the zero-entry-IFD `jpeg_with_exif` fixture). That's expected and is exactly
  what AC4 pins. Report presence from `ImageInfo.has_exif`; report tags from
  `read_exif_tags`.
- **Output streams (resolved #5):** human report **and** `--json` both go to
  **stdout** (mutually-exclusive shapes; the requested output). Diagnostics /
  errors go to **stderr**. On `--json` success, stderr must be empty (AC2).
- **JSON emission — `serde_json` is dev-dep ONLY (locked decision):** the runtime
  crate set has `serde` (+derive) but **not** `serde_json`, and we are NOT adding
  `serde_json` to `[dependencies]` (it would be a new runtime top-level dep
  needing its own DEC, for a single small JSON object). Instead, the production
  `--json` path emits the object with a **small, explicit hand-rolled writer**
  over the `InfoReport` fields — a private `fn write_json(report: &InfoReport,
  out: &mut impl std::io::Write) -> std::io::Result<()>` that prints the documented
  keys in order, escaping string values (`"`, `\`, and control chars `< 0x20`).
  Keep it tiny and total (no panic; map any write io-error to a `CliError` only if
  stdout write genuinely fails — otherwise return `Ok`). The
  `#[derive(serde::Serialize)]` on the DTOs is retained because it is **free**
  (serde is already a dep) and lets the `info_report_serializes_fields` unit test
  validate field mapping via `serde_json::to_value` under dev-deps; the
  production emitter and the derive must agree on field names/types (the
  integration `--json` parse tests in `tests/cli.rs` enforce this). `serde_json`
  appears **only** in `[dev-dependencies]` and is used **only** in test code.
- **Resolve → load mirror `run_view`:** `source::resolve(input, &mut
  std::io::stdin().lock())?` → `.into_iter().next()` →
  `ok_or(CliError::Source(SourceError::NotFound(input.to_owned())))?` → match
  `Input::Path(p)` (read file bytes once with `std::fs::read(p)?` mapped to
  `CliError::Image(ImageError::Io(_))` OR load via `Image::load`; you need the
  raw bytes for both the file size and the EXIF read, so prefer reading bytes
  once then `Image::from_bytes(&bytes)`) / `Input::Stdin { bytes, .. }`
  (`Image::from_bytes(bytes)`; file size = `bytes.len()`).
  - For a path: read bytes with `std::fs::read(p)` (io error →
    `CliError::Image(ImageError::Io(e))` → exit 3, consistent with
    `Image::load`); file size = `bytes.len() as u64` (equivalently
    `std::fs::metadata(p).len()`); decode via `Image::from_bytes(&bytes)`.
  - This single-read approach gives the file size, the decoded image, and the
    EXIF source bytes from one read — clean and avoids a metadata/`fs::read`
    race.
- **dispatch wiring:** replace
  `Commands::Info { .. } => Err(CliError::NotImplemented("info")),`
  with
  `Commands::Info { input, exif, json } => run_info(input, *exif, *json, &cli.global),`
- **`tests/info_exif.rs` is a new file** because `tests/cli.rs` does not
  `mod common;` and adding it there risks unused-import churn across the existing
  tests. A dedicated file with `mod common;` keeps the shared `jpeg_with_exif`
  fixture usage isolated and the existing `tests/cli.rs` untouched beyond the new
  `info` tests that use only its local `write_test_png`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

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
