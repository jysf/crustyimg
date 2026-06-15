# SPEC-009 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. Do not rely on any prior conversation. This prompt is
> deliberately prescriptive — follow it literally. Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-009 ("info command image inspection").
You are NOT the architect; the spec file is your source of truth. This spec
makes the `info` subcommand REAL — it replaces the NotImplemented("info") stub
with a real handler that prints an image's dimensions, format, file size on
disk, color type, bit depth, alpha, and ICC/EXIF presence; `--exif` dumps EXIF
tags read-only via kamadak-exif; `--json` emits machine-readable JSON to stdout.
Use ABSOLUTE paths for every file you read or write.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════

1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — conventions: §5 stack (kamadak-exif is the read-only EXIF crate; this spec
   ADDS it as a new top-level dep — DEC-013 justifies it), §6 the EXACT commands
   (the gates below), §11 coding conventions (library-first; `main.rs`/`cli` is
   the binary boundary; typed errors; NO unwrap/expect/panic! on recoverable
   paths; MACHINE/JSON OUTPUT TO STDOUT, DIAGNOSTICS TO STDERR; the pixel core
   `image/` MUST NOT depend on clap or grow serde derives; group imports
   std/external/local; comments explain WHY not WHAT; no dead code), §12 testing
   (integration under tests/, NATIVE in-memory fixtures via the `image` crate —
   NO ImageMagick, NO committed binary fixtures; trim stdout for Windows; REUSE
   the existing `jpeg_with_exif(w,h)` fixture in tests/common/mod.rs), §13 git/PR
   (branch naming, conventional commits + Co-Authored-By trailer, PR body
   template), §15 build-cycle rules (spec edits LIMITED to `## Build Completion`;
   append a build cost session entry; create DEC-* only for NON-trivial NEW
   decisions — NONE expected here, DEC-013 is already written).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-009-info-command-image-inspection.md
   — THE SPEC. Implement its "## Failing Tests", "## Outputs" (incl. the EXACT
   `InfoReport`/`ExifTag` DTOs, the locked JSON schema table, the `run_info`/
   `read_exif_tags`/`format_label`/`color_type_label`/`write_json` signatures),
   and "## Acceptance Criteria" exactly. Read "## Implementation Context" and
   "## Notes for the Implementer" in FULL — they carry the four locked design
   calls (file-size = on-disk; EXIF via read_from_container on full bytes;
   no-EXIF = success; serde_json is DEV-DEP ONLY + hand-rolled JSON emitter).
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/api-contract.md
   — the `info` contract: prints dimensions/format/FILE-SIZE/color-type/bit-
   depth/alpha/ICC+EXIF presence; `--exif` read-only (no-EXIF reports "no EXIF",
   exit 0); `--json` to stdout, diagnostics to stderr; single-image (first input
   on dir/glob). (The architect already clarified this entry; do NOT edit it.)
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-013-kamadak-exif-read-only-metadata.md
   — kamadak-exif added ALWAYS-ON (NOT feature-gated; it is pure-Rust, single
   transitive dep `mutate_once`, no build script). Read via
   `exif::Reader::new().read_from_container(&mut Cursor::new(&bytes))` over the
   FULL input bytes. `exif::Error::NotFound` (and any other Err) → graceful empty
   tag list, exit 0. Read-only — do NOT pull in img-parts/little_exif (STAGE-004
   write lane). kamadak-exif is imported as `exif` (`use exif;`).
5. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-003-metadata-dual-lane.md ,
   .../DEC-012-clap-cli-framework.md , and .../DEC-007-error-handling-thiserror-anyhow.md
   — DEC-003: `info` is on the READ path only; ICC/EXIF *presence* comes from the
   already-captured `ImageInfo.has_icc`/`has_exif`; the `--exif` tag DUMP is the
   only new read. DEC-012: clap derive; the `Info` variant + `input`/`--exif`/
   `--json` args are ALREADY declared — do NOT change the arg surface. DEC-007:
   typed errors → exit codes at the binary boundary; reuse `CliError` + `code()`
   unchanged (missing input → 3; decode → 1; unsupported format → 4; io → 3); add
   NO error variant — EXIF errors are swallowed inside `read_exif_tags`.
6. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/constraints.yaml
   — no-new-top-level-deps-without-decision (DEC-013 covers kamadak-exif;
   serde_json is dev-dep only, no DEC needed but call it out),
   pure-rust-codecs-default (kamadak-exif is pure-Rust), no-unwrap-on-recoverable-
   paths, every-public-fn-tested, clippy-fmt-clean, test-before-implementation,
   untrusted-input-hardening (malformed EXIF degrades, never panics),
   ergonomic-defaults (`info <path>` Just Works).
7. The SHIPPED code you wire together (read the real signatures):
   src/cli/mod.rs    — the clap `Commands::Info` variant (input + --exif + --json,
                       ALREADY declared), `dispatch()` (the
                       `Info { .. } => Err(NotImplemented("info"))` line you
                       replace), `run_view` (the STRUCTURAL TEMPLATE for run_info:
                       resolve → first input → load), `run_apply`, the `CliError`
                       enum + `code()` (do NOT edit), the
                       `exit_code_mapping_is_total` unit test (must stay green
                       unchanged), the existing `#[cfg(test)] mod tests`.
   src/image/mod.rs  — `Image::load`, `Image::from_bytes`, `Image::info()`,
                       the `ImageInfo` fields (NOT Serialize; holds
                       image::ImageFormat/ColorType), `Image::metadata()`,
                       `img.info().byte_len` (the DECODED buffer length →
                       `decoded_bytes`). Do NOT add serde to this module.
   src/source/mod.rs — `source::resolve`, `Input::{Path, Stdin}`,
                       `Input::stem()`, `Input::path()`, `SourceError::NotFound`.
   tests/cli.rs      — integration conventions + the `write_test_png` helper you
                       REUSE for the info tests (this file does NOT `mod common;`).
   tests/common/mod.rs — `jpeg_with_exif(w,h)` (valid APP1 EXIF, zero-entry IFD)
                       you REUSE in the NEW tests/info_exif.rs file.

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST (before editing ANY file)
═══════════════════════════════════════════════════════════════════════════

Do this BEFORE touching code so nothing ever lands on `main`:

  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-009-info-command-image-inspection

ALL code, test, and spec edits below happen ON THIS BRANCH. Never commit to
`main`. Confirm `git branch --show-current` prints
`feat/spec-009-info-command-image-inspection`, NOT `main`, before committing.

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact)
═══════════════════════════════════════════════════════════════════════════

A. Cargo.toml — add the deps (DEC-013).

   A1. Under `[dependencies]`, add (always-on, NOT optional, NOT a feature):
         kamadak-exif = "=0.6.1"
       (The crate publishes as `kamadak-exif`; you import it as `exif`.)
   A2. Under `[dev-dependencies]`, add (TEST-ONLY JSON parse/validate):
         serde_json = "=1.0.150"
       Do NOT add serde_json to `[dependencies]`. Do NOT add img-parts or
       little_exif. Do NOT add a `[features]` entry for exif.

B. src/cli/mod.rs — replace the info stub with a real handler + helpers.

   B1. In `dispatch()`, replace:
         Commands::Info { .. } => Err(CliError::NotImplemented("info")),
       with:
         Commands::Info { input, exif, json } => {
             run_info(input, *exif, *json, &cli.global)
         }

   B2. Add the two serde-serializable DTOs (private to this module), EXACTLY as
       the spec's "## Outputs" block specifies: `InfoReport` (with
       `#[serde(skip_serializing_if = "Option::is_none")] exif: Option<Vec<ExifTag>>`)
       and `ExifTag { tag, ifd, value }`. Both derive
       `#[derive(Debug, Clone, serde::Serialize)]`. The `serde::Serialize` derive
       is FREE (serde is already a dep) and is used by the
       `info_report_serializes_fields` unit test; it is NOT how the production
       path emits JSON (see B6).

   B3. Add `fn format_label(fmt: ::image::ImageFormat) -> String` — map the core
       formats to stable lowercase labels: Png→"png", Jpeg→"jpeg", Gif→"gif",
       Bmp→"bmp", Tiff→"tiff", Ico→"ico"; for any other variant return a lowercase
       fallback (e.g. `format!("{fmt:?}").to_ascii_lowercase()`). No panic;
       `ImageFormat` is `#[non_exhaustive]` so include a catch-all arm.

   B4. Add `fn color_type_label(ct: ::image::ColorType) -> String` — Rgb8→"rgb8",
       Rgba8→"rgba8", L8→"l8", La8→"la8", Rgb16→"rgb16", Rgba16→"rgba16",
       L16→"l16", La16→"la16", Rgb32F→"rgb32f", Rgba32F→"rgba32f"; catch-all
       lowercase fallback. No panic; `ColorType` is `#[non_exhaustive]`.

   B5. Add `fn read_exif_tags(bytes: &[u8]) -> Vec<ExifTag>` (read-only, DEC-013):
         use std::io::Cursor;   // group with std imports at top of file
         // ... in the fn:
         match exif::Reader::new().read_from_container(&mut Cursor::new(bytes)) {
             Ok(exif) => exif
                 .fields()
                 .map(|f| ExifTag {
                     tag: f.tag.to_string(),
                     ifd: f.ifd_num.to_string(),
                     value: f.display_value().with_unit(&exif).to_string(),
                 })
                 .collect(),
             Err(_) => Vec::new(), // NotFound OR malformed → "no EXIF", not an error
         }
       NO unwrap/expect. Every Err (including NotFound) collapses to an empty Vec.

   B6. Add `fn write_json(report: &InfoReport, out: &mut impl std::io::Write)
       -> std::io::Result<()>` — the production `--json` emitter. Hand-roll a
       single-line JSON object in the EXACT field order of the spec's schema
       table (input, width, height, format, file_size_bytes, decoded_bytes,
       color_type, bit_depth, has_alpha, has_icc, has_exif, then `exif` ONLY when
       `report.exif.is_some()`). Escape string values: `"`→`\"`, `\`→`\\`, and
       control chars `< 0x20` as `\u00XX` (a small `escape_json(&str) -> String`
       helper is fine). Booleans/numbers print bare. End with `writeln!` (trailing
       newline). No panic; propagate io errors via `?`. The `exif` array elements
       are `{"tag":"..","ifd":"..","value":".."}` with the same escaping.

   B7. Add the handler (mirror `run_view`'s resolve→first→load structure; print
       instead of sinking). NO unwrap/expect/panic; `?` throughout:

         /// The `info` path: resolve the first input, load the image and its raw
         /// bytes (one read), build the report, and print human text or JSON to
         /// stdout. Single-image: resolves the FIRST input on a directory/glob.
         fn run_info(
             input: &str,
             exif: bool,
             json: bool,
             _global: &GlobalArgs,
         ) -> Result<(), CliError> {
             let resolved = source::resolve(input, &mut std::io::stdin().lock())?;
             let first = resolved
                 .into_iter()
                 .next()
                 .ok_or(CliError::Source(SourceError::NotFound(input.to_owned())))?;

             // Read the raw bytes ONCE: they give the file size, the decoded
             // image, and the EXIF source. (For a path, std::fs::read io-error
             // maps to ImageError::Io → exit 3, consistent with Image::load.)
             let (raw, label): (Vec<u8>, String) = match &first {
                 crate::source::Input::Path(p) => {
                     let bytes = std::fs::read(p).map_err(ImageError::Io)?;
                     (bytes, p.display().to_string())
                 }
                 crate::source::Input::Stdin { bytes, .. } => (bytes.clone(), "-".to_owned()),
             };
             let img = Image::from_bytes(&raw)?;
             let info = img.info();

             let exif_tags = if exif { Some(read_exif_tags(&raw)) } else { None };

             let report = InfoReport {
                 input: label,
                 width: info.width,
                 height: info.height,
                 format: format_label(info.format),
                 file_size_bytes: raw.len() as u64,
                 decoded_bytes: info.byte_len,
                 color_type: color_type_label(info.color_type),
                 bit_depth: info.bit_depth,
                 has_alpha: info.has_alpha,
                 has_icc: info.has_icc,
                 has_exif: info.has_exif,
                 exif: exif_tags,
             };

             let mut out = std::io::stdout().lock();
             if json {
                 write_json(&report, &mut out).map_err(crate::sink::SinkError::Io)?;
             } else {
                 print_human(&report, &mut out).map_err(crate::sink::SinkError::Io)?;
             }
             Ok(())
         }

       A genuine stdout-write failure maps through the EXISTING
       `SinkError::Io(#[from] std::io::Error)` variant (confirmed present in
       src/sink/mod.rs at the time of writing) → `CliError::Sink(_) => 5`. This
       reuses the existing mapping and `exit_code_mapping_is_total` stays
       UNCHANGED (its catch-all `Sink(_)` arm already covers `Io`). `?` on the
       `SinkError` works because `CliError::Sink(#[from] SinkError)` exists. Verify
       `SinkError::Io` is still the variant name before wiring; if it ever differs,
       do NOT invent a new variant — bubble through whatever existing I/O variant
       `SinkError` exposes and note it in Build Completion → Deviations.

   B8. Add `fn print_human(report: &InfoReport, out: &mut impl std::io::Write)
       -> std::io::Result<()>` — labeled lines to stdout per the spec's "Human
       output shape": an `input:` line, `dimensions: {w}x{h}`, `format:`,
       `file size: {n} bytes`, `color type:`, `bit depth:`, `alpha: yes|no`,
       `icc: yes|no`, `exif: yes|no` (from `report.has_exif`). When
       `report.exif` is `Some(tags)`: if empty print `exif tags: (none)`, else
       print `exif tags:` then one `  {tag}: {value}` line per tag. Must contain
       the assertable substrings: the `{w}x{h}` form, the lowercase format label,
       the color-type label, and the icc/exif presence words.

   B9. Do NOT touch `CliError`, `code()`, the clap `Commands` enum, or
       `exit_code_mapping_is_total`.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (make them pass)
═══════════════════════════════════════════════════════════════════════════

Integration tests are binary-driven via env!("CARGO_BIN_EXE_crustyimg") +
std::process::Command; `tempfile::tempdir()`; trim stdout; assert exit codes via
`output.status.code()`. `info` has NO tty requirement, so a piped stdout under
`cargo test` is fine.

Add to the EXISTING /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/tests/cli.rs
(REUSE its local `write_test_png`; do NOT add `mod common;` here):

  - info_human_output_reports_core_facts
      Write an 8x8 PNG; run `info <png>`. Assert: code()==Some(0); trimmed,
      lowercased stdout contains "8x8", "png", "rgb8", and both an "icc" and an
      "exif" line; stderr is empty.
  - info_json_is_parseable_and_complete
      Write an 8x8 PNG; run `info --json <png>`. Assert: code()==Some(0);
      serde_json::from_slice::<serde_json::Value>(&output.stdout) succeeds and is
      an object; width==8, height==8, format=="png", color_type=="rgb8",
      bit_depth==8, has_alpha==false, has_icc==false, has_exif==false,
      file_size_bytes>0, decoded_bytes>0; obj.get("exif").is_none(); stderr empty.
  - info_json_exif_empty_array_on_plain_png
      Write an 8x8 PNG; run `info --json --exif <png>`. Assert: code()==Some(0);
      obj["exif"].as_array().unwrap().is_empty(); obj["has_exif"]==false.
  - info_exif_on_plain_png_reports_none
      Write an 8x8 PNG; run `info --exif <png>` (no --json). Assert:
      code()==Some(0); lowercased stdout contains "exif" and indicates absence
      ("no" on the exif line OR "(none)").
  - info_missing_input_exits_3
      Run `info <tempdir>/nope.png` (non-existent). Assert: code()==Some(3).

Create a NEW file /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/tests/info_exif.rs
(first line `mod common;`, then `use common::jpeg_with_exif;`):

  - info_exif_reports_present_on_jpeg_with_exif
      Write jpeg_with_exif(8,8) bytes to a tempfile via std::fs::write; run
      `info --exif <jpeg>`. Assert: code()==Some(0); lowercased stdout contains
      "exif" and "yes" (EXIF present). Do NOT assert any specific tag exists
      (the fixture's IFD is zero-entry). This pins "report EXIF-present, succeed
      even with no readable tags."

Add to the EXISTING `#[cfg(test)] mod tests` in src/cli/mod.rs (use `super::*`):

  - format_label_maps_core_formats
      Assert format_label maps Png→"png", Jpeg→"jpeg", Gif→"gif", Bmp→"bmp",
      Tiff→"tiff", Ico→"ico".
  - color_type_label_maps_color_types
      Assert color_type_label maps Rgb8→"rgb8", Rgba8→"rgba8", L8→"l8",
      Rgb16→"rgb16".
  - read_exif_tags_graceful_on_no_exif
      read_exif_tags(&[]) is empty; read_exif_tags(b"not an image") is empty;
      read_exif_tags(<plain png bytes>) is empty; read_exif_tags(<jpeg_with_exif
      bytes>) returns a Vec (len >= 0) without panicking. Generate the PNG/JPEG
      bytes INLINE in the unit test (the unit test cannot reach tests/common) —
      e.g. encode a tiny solid RGB PNG with the `image` crate; for the EXIF case
      either inline a minimal jpeg-with-exif byte builder or accept the plain-png
      assertions plus the &[]/garbage assertions (the integration test
      info_exif_reports_present_on_jpeg_with_exif already exercises the real
      fixture path). Keep it total, no unwrap on a recoverable path is required in
      test code but avoid panics that mask the assertion.
  - info_report_serializes_fields
      Construct an InfoReport with known field values and exif: None; assert
      serde_json::to_value(&report) yields the documented fields with expected
      values and has NO "exif" key. Then construct one with exif: Some(vec![]) and
      assert the "exif" key IS present and is an empty array. (This uses the
      serde_json dev-dep and the Serialize derive; it validates the field
      contract the hand-rolled write_json must match.)

The existing exit_code_mapping_is_total, each_subcommand_help_parses, and
help_lists_all_subcommands MUST still pass (no arg-surface / error-variant
change). Run the FULL `cargo test`.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════

- Any metadata WRITE (strip/clean/set/copy-metadata) — STAGE-004.
- img-parts / little_exif (the write lane) — do NOT add them.
- serde_json as a RUNTIME dependency — it is dev-dep ONLY; the production --json
  path is the hand-rolled write_json emitter.
- info on multiple images / batch / fan-out — single-image; first input only.
- ICC profile PARSING — report presence only (info.has_icc).
- A new CliError variant, exit-code change, or clap arg-surface change.
- Faking a tty / rendering — info has no tty requirement; it always prints.
- A feature gate for kamadak-exif — it is ALWAYS-ON (DEC-013).
If you think a new RUNTIME crate or a new DEC is needed, STOP and add a question
to /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml
instead of inventing it.

═══════════════════════════════════════════════════════════════════════════
THE GATES (run from the repo root; ALL must pass before the PR)
═══════════════════════════════════════════════════════════════════════════

  cargo build
  cargo test
  cargo clippy -- -D warnings
  cargo fmt --check                              # `cargo fmt` to fix, then re-check

NOTE: there is NO `--features display` gate for SPEC-009 — `info` does not touch
viuer. kamadak-exif is always-on, so the four standard gates above fully cover it
(serde_json being a dev-dep is also covered by `cargo test`). Run exactly these
four.

═══════════════════════════════════════════════════════════════════════════
WHEN DONE
═══════════════════════════════════════════════════════════════════════════

1. Fill in ONLY the spec's `## Build Completion` section (branch, PR, criteria
   met, deviations — INCLUDING your B7 stdout-write-error choice, follow-ups, and
   the 3-question build reflection). Do NOT edit any other part of the spec body.
2. Append a build cost session entry to the spec front-matter `cost.sessions`
   (cycle: build, agent: claude-sonnet-4-6, interface: claude-code,
   tokens_total: null, estimated_usd: null, duration_minutes: <est>,
   recorded_at: 2026-06-15, notes: "subagent; cost not separately reported").
   Do NOT recompute cost.totals (ship does that).
3. Advance the cycle to verify by HAND-EDITING the spec front-matter `task.cycle`
   from `build` to `verify`. DO NOT run `just advance-cycle` or `just
   archive-spec` — they MIS-GLOB in this repo; the orchestrator does all other
   bookkeeping by hand. Only edit the spec's Build Completion section + the cost
   session + task.cycle.
4. Commit ON THE BRANCH (created in Step 0) with Conventional Commits, e.g.
   `feat(cli): real info command with --exif and --json (SPEC-009)`
   — a single commit covering Cargo.toml + cli + tests + spec is fine; end EACH
   commit message with:
       Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
   (Confirm `git branch --show-current` prints
   `feat/spec-009-info-command-image-inspection`, NOT `main`, before committing.)
5. Mark build `[x]` in the timeline
   (/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-009-info-command-image-inspection-timeline.md).
   ACCURATE BOOKKEEPING: when you mark build `[x]`, write ONLY what is true at
   build time — say "PR #N opened" (with the real number). Do NOT write "merged",
   do NOT claim the PR is approved, and do NOT assert any post-merge fact. Verify
   and ship record those later.
6. Push the branch and open a PR on the `jysf/crustyimg` remote per AGENTS.md §13
   (one spec per branch / per PR):
   - PR title carries the spec id, e.g.
     `feat(cli): info command image inspection (SPEC-009)`.
   - PR body uses the §13 template — Summary; Spec metadata PROJ-001/STAGE-002/
     SPEC-009; Decisions referenced [DEC-013 (kamadak-exif always-on read-only
     EXIF), DEC-003 (read-lane only), DEC-012 (clap surface unchanged), DEC-007
     (reuse CliError/code, no new variant)]; Constraints checked with one-line
     evidence each (no-new-top-level-deps-without-decision [DEC-013 covers
     kamadak-exif; serde_json is dev-dep only], pure-rust-codecs-default,
     no-unwrap-on-recoverable-paths, every-public-fn-tested, clippy-fmt-clean,
     test-before-implementation, untrusted-input-hardening, ergonomic-defaults);
     New decisions: list "DEC-013 — kamadak-exif read-only metadata (emitted
     during design; added to Cargo.toml here)".
   - End the PR body with the Claude Code generated-with footer.

Remember: build edits to the spec are LIMITED to `## Build Completion` (plus the
front-matter cost session + task.cycle). Verify/ship bookkeeping lands on main
later, not on this branch.
```
