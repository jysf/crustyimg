---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-026
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
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
  decisions: [DEC-003, DEC-029, DEC-004, DEC-018, DEC-015, DEC-007]
  constraints:
    - metadata-not-via-pixel-encode
    - no-new-top-level-deps-without-decision
    - pure-rust-codecs-default
    - no-agpl-default-deps
    - clippy-fmt-clean
    - every-public-fn-tested
    - no-unwrap-on-recoverable-paths
  related_specs: [SPEC-002, SPEC-013, SPEC-014]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-004's <capability>". Optional; null is acceptable.
value_link: >
  Opens the container-level metadata lane (DEC-003) with `strip` + `clean
  --gps` — the privacy half of STAGE-004 and the start of the project's
  verifiable-privacy axis.

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
        (Failing Tests + Implementation Context), emitted DEC-029, added the
        deps + a design-time probe (img-parts/little_exif on real JPEG+PNG),
        fleshed out the api-contract strip/clean entries.
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-18
      notes: "metadata lane v1: src/metadata strip_all/clean_gps + Sink::write_bytes + run_metadata_lane fan-out + CliError::Metadata + tests; container lane, no pixel re-encode"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-026: metadata lane v1 — `strip` + `clean --gps`

## Context

This is the **first spec of STAGE-004** and the first code in the **container
lane** (DEC-003). Until now every command runs through the pixel lane
(decode → ops → encode), which inherently *drops* all metadata. The container
lane is the opposite: it edits container-level metadata (EXIF/ICC/XMP/IPTC)
**without ever re-decoding pixels**. This spec lands the two privacy-first
operations:

- **`strip`** — remove **all** container metadata.
- **`clean --gps`** — remove **only** GPS/location metadata, preserving the rest.

It is the start of the project's **verifiable-privacy axis** (`docs/moat.md`)
and the prerequisite for later upgrading `optimize` to *selectively preserve*
metadata (a DEC-024 follow-up). The `src/metadata/` module does not exist yet —
it is net-new here. `strip`/`clean` currently exist only as clap stubs that
return `CliError::NotImplemented` (`src/cli/mod.rs`).

Parent: `STAGE-004-compose-and-metadata` (backlog items "`strip`" and "`clean
--gps`"). Governing decision: **DEC-003** (dual-lane). Dependency decision:
**DEC-029** (pins `img-parts` + `little_exif`; pure-Rust + permissive; includes
a design-time probe that verified the approach on real JPEG + PNG).

## Goal

Wire `strip` (remove all metadata via `img-parts` segment/chunk removal) and
`clean --gps` (remove only GPS via `little_exif` tag removal) for **JPEG and
PNG**, operating purely on container bytes so decoded pixels are byte-identical
and no pixel re-encode occurs. Unsupported formats exit cleanly (code 4).

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — `Commands::Strip`/`Commands::Clean` clap variants (already
    declared), `dispatch` arms (currently `NotImplemented`), `run_pixel_op`
    (the fan-out shape to mirror), `CliError` + `code()`, `build_sink`, `GlobalArgs`.
  - `src/image/mod.rs` — `MetadataBundle` + the byte-scanning `scan_jpeg_*` /
    `scan_png_chunk` helpers (context for what "EXIF/ICC segment" means; the lane
    does NOT reuse the load-time bundle — see Notes).
  - `src/sink/mod.rs` — `Sink`, `SinkInput`, `Overwrite`, `safe_join`,
    `extension_for_format`, the overwrite guard — to add a raw-bytes write path.
  - `decisions/DEC-003-metadata-dual-lane.md`, `decisions/DEC-029-*.md`.
- **External crates (added by DEC-029):** `img-parts` `=0.4.0`
  (https://docs.rs/img-parts/0.4.0), `little_exif` `=0.6.23`
  (https://docs.rs/little_exif/0.6.23). `image::guess_format` for decode-free
  format sniffing.
- **Related code paths:** `src/metadata/` (new), `src/cli/mod.rs`,
  `src/sink/mod.rs`, `tests/`.

## Outputs

- **Files created:**
  - `src/metadata/mod.rs` — the container-lane module. Public byte→byte API:
    - `pub fn strip_all(bytes: &[u8]) -> Result<Vec<u8>, MetadataError>`
    - `pub fn clean_gps(bytes: &[u8]) -> Result<Vec<u8>, MetadataError>`
    - `pub enum MetadataError { UnsupportedFormat(String), Container(String), Exif(String) }`
      (`thiserror`, `#[derive(Debug, Error)]`).
  - `tests/metadata.rs` — integration tests over the binary (strip/clean
    end-to-end, fan-out, exit codes).
- **Files modified:**
  - `src/lib.rs` — add `pub mod metadata;`.
  - `src/cli/mod.rs` — add `CliError::Metadata(#[from] MetadataError)` with
    `code()` mapping (UnsupportedFormat → 4; Container/Exif → 1); wire
    `Commands::Strip`/`Commands::Clean` dispatch arms to `run_strip`/`run_clean`
    delegating to a shared `run_metadata_lane` fan-out helper.
  - `src/sink/mod.rs` — add a raw-bytes write path (e.g.
    `Sink::write_bytes(&[u8], &SinkInput, Overwrite, &mut impl Write)`) that writes
    container bytes verbatim (format preserved; no re-encode).
  - `Cargo.toml` — `img-parts = "=0.4.0"`, `little_exif = "=0.6.23"` (DEC-029).
  - `docs/api-contract.md` — flesh out the `strip` / `clean --gps` entries (done
    at design).
- **New exports:** `crate::metadata::{strip_all, clean_gps, MetadataError}`.
- **Database changes:** none.

## Command surface (PINNED)

```
crustyimg strip <INPUTS...>
crustyimg clean <INPUTS...> --gps
```

- **Format coverage v1:** JPEG + PNG. Any other detected format → exit **4**
  ("metadata lane does not support <fmt> yet") — a single, clear error, mirroring
  the codec-not-built pattern. (For a multi-input batch, a per-input unsupported
  format is a per-input failure → counts toward partial-batch exit **6**.)
- **Format is always preserved** — `strip`/`clean` never transcode. `-q/--quality`
  and `--format` are irrelevant and ignored (the lane writes container bytes
  verbatim); do not declare local copies of the global flags (the SPEC-024 lesson).
- **`clean` requires `--gps` in v1.** `clean` without `--gps` → `CliError::Usage`
  (exit **2**) with "clean requires --gps". (Leaves room for future selective
  flags / `--all`.)
- **Fan-out (reuse the `run_pixel_op` shape, DEC-015):**
  - Resolve every input via `source::resolve` (missing path / empty glob → exit
    3/2, a hard error, not partial-batch).
  - **Single** resolved input: write to `-o PATH`, else `--out-dir` (templated),
    else **stdout** (raw bytes — pipe-friendly, matches the pixel ops' default).
  - **Multiple** resolved inputs: require `--out-dir` (else exit 2); per-input
    load/transform/write failures print to stderr and yield exit **6**
    (partial batch); a single-input failure keeps its natural code.
  - Overwrite guarded by `-y/--yes` (reuse `Overwrite`).
- **No-op semantics:** `strip`/`clean` on a file that already has no relevant
  metadata succeed (exit 0) and emit a byte-faithful copy. In particular
  `little_exif` returns `Err("No EXIF data found!")` for a JPEG with no EXIF —
  `clean_gps` MUST catch that and return `Ok(bytes.to_vec())`.

## Lane mechanics (PINNED — probe-verified, DEC-029)

`strip_all(bytes)` and `clean_gps(bytes)` each sniff the format with
`image::guess_format(bytes)` (no decode) and branch:

- **`strip_all` — JPEG:** `Jpeg::from_bytes(Bytes::from(bytes.to_vec()))`, then
  `remove_segments_by_marker(m)` for `m in 0xE1..=0xEF` (APP1 EXIF/XMP … APP15)
  **and** `0xFE` (COM); serialize via `jpeg.encoder().write_to(&mut out)`. (Keep
  APP0/JFIF — it is structural, not user metadata.)
- **`strip_all` — PNG:** `Png::from_bytes(..)`, then `remove_chunks_by_type(k)` for
  `k in [b"eXIf", b"iCCP", b"tEXt", b"zTXt", b"iTXt", b"tIME"]`; serialize via
  `png.encoder().write_to(&mut out)`. (Keep critical/render chunks: IHDR, PLTE,
  IDAT, IEND, tRNS, gAMA, cHRM, sRGB, bKGD, pHYs.)
- **`clean_gps` — JPEG & PNG:** `Metadata::new_from_vec(&bytes.to_vec(), ext)`
  where `ext` is `FileExtension::JPEG` / `FileExtension::PNG { as_zTXt_chunk: false }`;
  on `Err` whose message contains "No EXIF" → return the bytes unchanged
  (no-op). Otherwise `md.get_ifd_mut(ExifTagGroup::GPS, 0)`, collect
  `get_tags().iter().map(|t| t.as_u16())`, `remove_tag(id)` each; write back into a
  clone of the input bytes via `md.write_to_vec(&mut out, ext)`.
- **Pixel invariant:** in all four paths the compressed scan / IDAT is carried
  verbatim — decoding the output yields pixels byte-identical to decoding the
  input. This is the constraint `metadata-not-via-pixel-encode` made concrete.

## Acceptance Criteria

- [ ] `strip` on a JPEG carrying EXIF (incl. GPS) + ICC produces output with **no
  APP1/APP2/COM** segments and **no parseable EXIF**; decoded pixels are identical
  to the input's decoded pixels.
- [ ] `strip` on a PNG carrying an `eXIf`/`tEXt` chunk produces output with those
  chunks removed; decoded pixels identical.
- [ ] `clean --gps` on a JPEG with GPS + Orientation + Copyright removes **only**
  GPS (Orientation + Copyright survive); decoded pixels identical.
- [ ] `clean --gps` / `strip` on a file with no relevant metadata exits 0 and emits
  a decode-identical copy (no-op).
- [ ] `strip`/`clean` on an unsupported format (e.g. BMP/GIF/TIFF/WebP) → exit **4**
  with a clear message (single input); per-input in a batch → counts toward exit 6.
- [ ] `clean` without `--gps` → exit **2**.
- [ ] Fan-out: single input → stdout by default / `-o` / `--out-dir`; multiple
  inputs require `--out-dir`; a failing input in a batch → exit **6**; overwrite of
  an existing output is refused without `-y`.
- [ ] No pixel re-encode anywhere in the lane (asserted via decode-equality, and by
  the code path never constructing an `Image`/`DynamicImage` for the metadata op).
- [ ] `cargo deny check licenses` green with the two new crates; build stays
  pure-Rust (no `--features` needed).

## Failing Tests

Written during **design**, BEFORE build. Build makes them pass. Fixtures are
generated **natively** (no ImageMagick): a small image via the `image` crate, EXIF
seeded with `little_exif` (a test helper `jpeg_with_exif()` / `png_with_exif()` that
sets `Orientation`, `Copyright`, and `GPSLatitudeRef`/`GPSLongitudeRef`).

- **`src/metadata/mod.rs` (unit, `#[cfg(test)] mod tests`)**
  - `"strip_all_jpeg_removes_all_metadata"` — seed EXIF+GPS, `strip_all` → re-parse
    with `little_exif` yields the "No EXIF" error (or empty); `img_parts::Jpeg` shows
    no APP1/APP2/COM. asserts: all metadata gone.
  - `"strip_all_jpeg_preserves_pixels"` — decode input vs `strip_all` output with the
    `image` crate → `to_rgba8()` buffers equal. asserts: no pixel change.
  - `"strip_all_png_removes_metadata_chunks"` — seed a PNG with an injected `tEXt`
    (or `eXIf`) chunk via `img-parts`, `strip_all` → that chunk absent; pixels equal.
  - `"clean_gps_removes_only_gps"` — seed Orientation+Copyright+GPS, `clean_gps` →
    re-parse: GPS tags absent, Orientation + Copyright present. asserts: selective.
  - `"clean_gps_preserves_pixels"` — decode-equality input vs output.
  - `"clean_gps_no_exif_is_noop_ok"` — JPEG with no EXIF → `clean_gps` returns `Ok`
    with decode-identical bytes (the "No EXIF data found" path).
  - `"strip_all_unsupported_format_errors"` — a BMP (or GIF) byte blob → `strip_all`
    returns `MetadataError::UnsupportedFormat`.
  - `"clean_gps_unsupported_format_errors"` — same for `clean_gps`.
- **`tests/metadata.rs` (integration, drives the binary via `assert`-on-exit)**
  - `"strip_jpeg_to_stdout_has_no_exif"` — `strip fixture.jpg` to stdout (`-o -` or
    default) → captured bytes have no EXIF; exit 0.
  - `"clean_gps_jpeg_removes_location_keeps_orientation"` — `clean fixture.jpg --gps
    -o out.jpg` → out has no GPS, keeps Orientation; exit 0.
  - `"clean_without_gps_flag_exits_2"` — `clean fixture.jpg` → exit 2.
  - `"strip_unsupported_format_exits_4"` — `strip fixture.bmp` → exit 4.
  - `"strip_multi_input_requires_out_dir"` — two inputs, no `--out-dir` → exit 2.
  - `"strip_multi_input_fanout_writes_all"` — two JPEGs + `--out-dir` → two stripped
    outputs; exit 0.
  - `"strip_batch_partial_failure_exits_6"` — one good JPEG + one unsupported input
    + `--out-dir` → exit 6, the good one still written.
  - `"strip_refuses_overwrite_without_yes"` — existing output, no `-y` → exit 5;
    with `-y` → exit 0.
- **`tests/cli.rs`** — add `"strip"` and `"clean"` to BOTH subcommand-list tests if
  those exhaustive lists exist (the SPEC-024 lesson).

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-003` — **governing.** Two-lane model; metadata-only commands never touch the
  pixel encode path; read=kamadak / segment=img-parts / tag=little_exif.
- `DEC-029` — pins `img-parts` `=0.4.0` + `little_exif` `=0.6.23` (pure-Rust,
  permissive); contains the probe with the exact verified API calls — **mirror them**.
- `DEC-004` / `pure-rust-codecs-default` — both crates are pure-Rust; no feature gate
  needed; default build stays zero-system-deps.
- `DEC-018` — `cargo deny check licenses` must stay green (it does; no new exception).
- `DEC-015` — partial-batch fan-out + exit-6 semantics; reuse the `run_pixel_op` shape.
- `DEC-007` — typed `thiserror` errors in the library; only `cli`/`main` map to exit
  codes; **no `unwrap`/`expect`/`panic!` on recoverable paths** (tests may unwrap).

### Constraints that apply

- `metadata-not-via-pixel-encode` (blocking) — the whole point: no decode/encode of
  pixels in the lane. Enforced by the decode-equality tests + never building an
  `Image` for the op.
- `no-new-top-level-deps-without-decision` — satisfied by DEC-029.
- `pure-rust-codecs-default`, `no-agpl-default-deps` — satisfied (both MIT/Apache,
  pure-Rust).
- `clippy-fmt-clean`, `every-public-fn-tested`, `no-unwrap-on-recoverable-paths`.

### Prior related work

- `SPEC-002` (shipped) — the canonical `Image` + load-time `MetadataBundle`
  byte-scanning (`scan_jpeg_exif`/`scan_jpeg_icc`/`scan_png_chunk`). Useful as a
  reference for segment/chunk layout; the lane does NOT depend on it.
- `SPEC-013`/`SPEC-014` (shipped) — `auto-orient` (reads EXIF orientation) and
  `convert` (forced-format fan-out + exit-4 codec errors) — the closest existing
  patterns for "EXIF-aware command" and "fan-out + exit 4".

### Out of scope (for this spec specifically)

- `set` (write tags), `copy-metadata`, `watermark` (own STAGE-004 specs).
- The EXIF **audit-linter** (reuses exit 7; later spec).
- Wiring **selective-preserve** into `optimize`/the pixel-lane encode path — that is
  the DEC-024 follow-up and depends on encode-time ICC/orientation preserve, which is
  the *still-open* `metadata-icc-coverage` question (this spec does NOT resolve it).
- **WebP/TIFF** strip/clean — `little_exif` nominally supports them but v1 is JPEG +
  PNG only; extend in a follow-up once probed.
- IPTC/XMP-specific selective edits (strip removes them wholesale; no targeted XMP).

## Notes for the Implementer

- **Mirror the DEC-029 probe exactly** — the API calls there compile and behave as
  asserted (`Jpeg::from_bytes` / `remove_segments_by_marker` / `encoder().write_to`;
  `Metadata::new_from_vec` / `get_ifd_mut(ExifTagGroup::GPS, 0)` / `remove_tag` /
  `write_to_vec`). `img-parts` types: `img_parts::jpeg::Jpeg`,
  `img_parts::png::Png`, `img_parts::Bytes`. `little_exif`:
  `little_exif::metadata::Metadata`, `::exif_tag::ExifTag`,
  `::filetype::FileExtension`, `::ifd::ExifTagGroup`.
- **The lane re-reads raw file bytes** (`std::fs::read` for `Input::Path`; the
  in-memory bytes for `Input::Stdin`). It does NOT use the load-time
  `MetadataBundle` (which only captured EXIF/ICC, not XMP/IPTC, and only for
  JPEG/PNG). Reading raw bytes is what lets `strip` remove *everything*.
- **Format sniff without decode:** `image::guess_format(&bytes)` → `ImageFormat`;
  match `Jpeg`/`Png`, else `UnsupportedFormat`. Do not call `Image::load`.
- **Raw-bytes sink:** keep the new write path in `src/sink` (the
  `metadata-not-via-pixel-encode` constraint scopes `src/sink/**`). Reuse
  `safe_join` + the overwrite guard + the `{stem}.{ext}` template; `ext` = the input
  file's own extension (format is preserved). A write failure → `SinkError` → exit 5.
- **A file-read I/O error** in the handler (after `source::resolve` already
  confirmed the path) → map to exit **3** (input unreadable) — reuse the existing
  `ImageError::Io`→3 mapping or add a small arm; do not invent a new top-level code.
- `clean`'s clap variant already has `gps: bool`; gate on it (`if !gps { Usage }`).
- `little_exif` emits `log` records and uses the `FileExtension::PNG { as_zTXt_chunk }`
  variant — pass `false`. Keep all diagnostics on **stderr**, stdout clean for `-o -`.
- The design-time **probe project** (`/tmp/mdprobe_ver`) is throwaway; the verified
  snippets live in DEC-029's Context — that is the source of truth.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-026-metadata-lane`
- **PR (if applicable):** see PR opened at build end (title
  `feat(metadata): strip + clean --gps container lane (SPEC-026)`).
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - None — DEC-029's probe matched the real API exactly; no build-forced decision.
- **Deviations from spec:**
  - The build-cost `cost.sessions` entry records `agent: claude-opus-4-8`
    (the session that actually ran the build) rather than the
    `claude-sonnet-4-6` placeholder in the build prompt's cost snippet, to keep
    the ledger honest. Structure + null numerics + notes are otherwise as
    specified.
  - The new container-lane write path is `Sink::write_bytes` (the exact name the
    spec suggested as an example) — a method on the existing `Sink` enum that
    handles File / Dir (templated, `{ext}` = preserved input extension) / Stdout
    and rejects `Sink::Display` (not a byte sink).
- **Follow-up work identified:**
  - WebP/TIFF strip/clean (deferred in this spec; `little_exif` nominally
    supports them — extend the lane once probed).
  - Selective-preserve wiring into `optimize`/the pixel-lane encode path
    (DEC-024 follow-up; depends on the still-open `metadata-icc-coverage`).
  - `set` / `copy-metadata` (own STAGE-004 specs) reuse `run_metadata_lane`.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Almost nothing. The PINNED Lane mechanics + DEC-029's verified API snippets
   meant the lane code wrote itself. The only real-API surprises were
   `img-parts`'s `remove_chunks_by_type([u8;4])` (needs `*b"eXIf"`, not a slice)
   and `clean_gps`'s GPS-IFD borrow needing two `get_ifd_mut` calls (collect ids
   immutably-ish, then a second mutable borrow to remove) — both minor.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The spec already named the file-read-after-resolve → exit 3 mapping, the
   stdout-clean rule, and the global-flag-reuse (SPEC-024) lesson — exactly the
   places I'd otherwise have tripped.

3. **If you did this task again, what would you do differently?**
   — Inspect the vendored crate sources for the exact `[u8;4]` vs `&[u8]` chunk
   signature and the `get_ifd_mut` create-on-miss behavior up front (I confirmed
   both before writing, which paid off). No process change needed.

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
