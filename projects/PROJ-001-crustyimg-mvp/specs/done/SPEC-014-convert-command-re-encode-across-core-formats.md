---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-014
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: S                    # S | M | L  (L means split it)

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
  decisions: [DEC-004, DEC-015, DEC-016, DEC-014, DEC-008, DEC-003, DEC-012, DEC-007]
  constraints:
    - ergonomic-defaults
    - single-image-library
    - pure-rust-codecs-default
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - test-before-implementation
    - untrusted-input-hardening
  related_specs: [SPEC-011, SPEC-012, SPEC-013, SPEC-005, SPEC-010]

value_link: "Delivers STAGE-003's `convert` command — re-encode an image (or a batch) from one core format to another in one short command, the everyday 'I need this PNG as a JPEG' task, reusing the fan-out + the DEC-016 quality knob."

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
      notes: "Design authored by the ORCHESTRATOR (Opus) directly (the proven path from SPEC-013). Verified the convert clap surface (the convert-local required --format shadows the global --format), the image 0.25.10 / kamadak-exif APIs, and the exit-4-up-front semantics. No new DEC — reuses DEC-004/015/016. Complexity S (CLI-only; no new Operation, no library/Sink change beyond threading a forced-format option through run_pixel_op)."
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "Fresh session build. Added forced_format param to run_pixel_op, wired run_convert, dispatched Commands::Convert. All 11 spec integration tests + repointed stub test pass (192 total). No new DEC — reuses DEC-004/015/016. Zero deviations from spec."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "Verify cycle, Opus read-only subagent. ✅ APPROVED, no punch list. Independently grepped + read all 12 named tests; confirmed convert_unbuilt_codec_multi_input_exits_4_not_6 pins exactly Some(4) (up-front-resolution correctness), the format-override test asserts GIF8 magic bytes, the quality test uses a gradient source. Ran the full suite on the branch (192 pass), clippy --all-targets + fmt clean, CI 6/6 (3-OS). Confirmed run_pixel_op's forced_format wired in both arms, output_format_for + tests unchanged, no new dep/op/CliError/Sink/DEC."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "Orchestrator ship bookkeeping on main (merged PR #15 squash bdd89f5; MERGEABLE/CLEAN, no update-branch needed; archive by hand per the just-glob caveat)."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 4
---

# SPEC-014: convert command — re-encode across core formats

## Context

`convert` is the STAGE-003 command for **changing an image's container
format** — "I have a PNG, I need a JPEG" — on a single image or a whole
batch. It is a **pure re-encode**: decode once, run no pixel transform, encode
to the requested target format.

- **Parent stage:** `STAGE-003` (transform & output). The FIFTH command, after
  `resize` op (SPEC-010), `resize` CLI (SPEC-011), `thumbnail` (SPEC-012) and
  `shrink` (SPEC-013) — all shipped on `main`. `auto-orient` (SPEC-015) is the
  last STAGE-003 spec.
- **Why now / what's new:** `convert` is the first command whose output format
  is **forced** by a required `--format` rather than preserved from the source
  (DEC-015 precedence #1, applied as a forced override). It re-uses, with **no
  library change**, the shipped pieces: the shared `run_pixel_op` fan-out
  (DEC-015 per-input write, multi-input `--out-dir`, partial-batch exit 6), the
  DEC-016 quality-aware encode (`-q` → JPEG quality, ignored for lossless
  formats), and the existing `resolve_format` → `SinkError` path that already
  maps an unsupported/unbuilt target codec to **exit 4** (DEC-004).
- **What stays the same:** No new `Operation` (the pipeline is empty — a pure
  re-encode; `Pipeline::new().run(img)` returns the image's pixels unchanged).
  No new dependency. No new `CliError` variant. No `Sink`/`encode_to_bytes`
  change.

The api-contract pins the surface: `convert <INPUT...> --format FMT [-q Q]`.
WebP output is fast-follow; AVIF is feature-gated and exits 4 when not built
(DEC-004).

## Goal

Wire the `convert` command: re-encode each input to the **required** `--format`
target (overriding source-format preservation), threading `-q/--quality`
(DEC-016, no forced default — `convert` is not `shrink`), via the shared
`run_pixel_op` fan-out. An unsupported or unbuilt target codec is resolved
**once up front** and returns **exit 4** (DEC-004) — a single capability error,
NOT a per-input partial-batch exit 6.

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — `run_pixel_op` (the shared fan-out; you add a
    `forced_format: Option<::image::ImageFormat>` param), `output_format_for`
    (the DEC-015 precedence helper — leave it and its tests UNCHANGED; the
    forced path is handled in `run_pixel_op`), `resolve_format` (string →
    `Option<ImageFormat>`, errors → exit 4; you reuse it to resolve the target),
    `run_resize`/`run_thumbnail`/`run_shrink` (callers; pass `None` for the new
    arg), `Commands::Convert { inputs, format }`, dispatch, `CliError`/`code()`.
  - `src/pipeline/mod.rs` — `Pipeline::new()` (empty pipeline = no-op re-encode).
  - `src/sink/mod.rs` — `format_from_extension`, `extension_for_format`,
    `encode_to_bytes(img, format, quality)` (already quality-aware, SPEC-013),
    `SinkError::{UnsupportedExtension, UnknownFormat}` (→ exit 4). NO change here.
  - `docs/api-contract.md` — the `convert` entry (you tighten it; see Outputs).
  - `tests/cli.rs`, `tests/common/mod.rs` — conventions + `solid_png` /
    `gradient_jpeg` fixtures; the `stub_command_returns_not_implemented` test
    (currently points at `convert` — REPOINT to `auto-orient`).
- **External APIs:** none new.
- **Related code paths:** `src/cli/` only. Do NOT modify other library modules.

## Outputs

- **Files modified:**
  - **`src/cli/mod.rs`**:
    - `run_pixel_op(pipeline, inputs, global, quality, forced_format:
      Option<::image::ImageFormat>)`: add the `forced_format` param. In BOTH the
      single-input and multi-input arms, resolve the per-write format as:
      ```text
      let fmt = match forced_format {
          Some(f) => f,
          None => output_format_for(global, output_path /* or None in multi */, img.source_format())?,
      };
      ```
      Everything else (sink construction, fan-out, partial-batch exit 6) is
      unchanged. Leave `output_format_for` and its three unit tests untouched.
    - `run_resize`, `run_thumbnail`, `run_shrink`: pass `None` as the new
      `forced_format` arg (behavior unchanged).
    - NEW `fn run_convert(inputs: &[String], format: &str, global: &GlobalArgs)
      -> Result<(), CliError>`:
      ```text
      // 1. Resolve the REQUIRED target format ONCE, up front.
      //    An unsupported/unbuilt codec → SinkError (UnsupportedExtension/
      //    UnknownFormat) → exit 4 (DEC-004), BEFORE any input is loaded — so a
      //    multi-input convert to an unbuilt codec is a single exit 4, not 6.
      let fmt = resolve_format(Some(format))?
          .ok_or_else(|| CliError::Usage("convert requires a target --format".into()))?;
      // 2. Pure re-encode: an empty pipeline returns pixels unchanged.
      let pipeline = Pipeline::new();
      // 3. Force `fmt` for every input; thread global.quality (no forced default).
      run_pixel_op(pipeline, inputs, global, global.quality, Some(fmt))
      ```
    - Dispatch: replace `Commands::Convert { .. } => Err(NotImplemented("convert"))`
      with `Commands::Convert { inputs, format } => run_convert(inputs, format, &cli.global)`.
    - NO new `CliError` variant; NO `code()`/`exit_code_mapping_is_total` change.
    - DO NOT remove or alter the `Commands::Convert { inputs, format }` clap
      variant — its **convert-local required `--format`** is intentional (clap
      enforces the required flag → exit 2 automatically; it shadows the global
      `--format` within `convert`, so read the value from the variant, NOT
      `global.format`).
  - **`docs/api-contract.md`** — the `convert` entry is **already pinned during
    design** (it documents: `--format` required → exit 2 if omitted; forced
    target format; `-q` threaded to JPEG / no forced default; per-input
    load/write failure → exit 6; unbuilt codec → exit 4 even multi-input; WebP
    fast-follow / AVIF feature-gated). Do NOT edit it during build unless the
    code deviates.
  - **`tests/cli.rs`** — add the integration tests below; REPOINT
    `stub_command_returns_not_implemented` from `convert` to `auto-orient`.
- **New decisions:** none. Reuses DEC-004 (codec policy / exit 4), DEC-015
  (format precedence + partial-batch exit 6), DEC-016 (quality knob).
- **No new dependency, no new `Operation`, no library/Sink change.**

## Acceptance Criteria

Each maps to a test.

- [ ] `convert <png> --format jpg -o out.jpg` → exit 0; output is JPEG, decodes,
  dimensions preserved. → `convert_png_to_jpeg_changes_format`
- [ ] `convert <jpg> --format png -o out.png` → exit 0; output is PNG, decodes.
  → `convert_jpeg_to_png_changes_format`
- [ ] `--format` overrides the `-o` extension (forced target wins). →
  `convert_format_overrides_output_extension`
- [ ] `convert <png> --format avif` → exit 4 (codec not built, DEC-004);
  `--format webp` → exit 4 (fast-follow). → `convert_unbuilt_codec_exits_4`
- [ ] Multi-input convert to an unbuilt codec → exit 4 (NOT 6) — resolved up
  front. → `convert_unbuilt_codec_multi_input_exits_4_not_6`
- [ ] `convert a.png b.png --format jpg --out-dir D` → exit 0; `D/a.jpg`,
  `D/b.jpg` both JPEG. → `convert_multi_input_fan_out`
- [ ] A lower `-q` yields a smaller JPEG than a higher `-q` (same image/format).
  → `convert_quality_lower_is_smaller`
- [ ] `convert <missing> --format png` → exit 3. → `convert_missing_input_exits_3`
- [ ] Multi-input without `--out-dir` → exit 2; stderr mentions `--out-dir`. →
  `convert_multi_without_out_dir_is_usage_error`
- [ ] `--format` omitted → exit 2 (clap required). → `convert_requires_format_flag`
- [ ] `convert <png> --format jpg -o -` → exit 0; stdout decodes as JPEG; stderr
  empty. → `convert_stdout_keeps_stdout_clean`
- [ ] `resize`/`thumbnail`/`shrink` outputs unchanged (their existing tests stay
  green; `run_pixel_op`'s new param defaults to `None` for them).

## Failing Tests

Written during **design**, made to pass during **build**. Mirror `tests/cli.rs`
conventions: drive the real binary via `assert_cmd`/`Command`; native in-memory
fixtures (`common::solid_png`, `common::gradient_jpeg`); `tempfile`; assert exit
codes via `output.status.code()`; decode outputs with
`image::load_from_memory` / `image::open`; detect format via magic bytes
(`\x89PNG`, `\xFF\xD8` JPEG, `GIF8`) or `image::ImageReader::...with_guessed_format`.

- **`tests/cli.rs`** (INTEGRATION — add to the existing file)
  - `convert_png_to_jpeg_changes_format` — write a `solid_png(40, 30, ..)` to a
    temp file; run `convert <png> --format jpg -o out.jpg`; assert exit 0, the
    output file's first two bytes are `0xFF 0xD8` (JPEG), it decodes, and decoded
    dims are 40×30 (re-encode preserves pixels' dimensions).
  - `convert_jpeg_to_png_changes_format` — `gradient_jpeg(32, 16)` → `convert
    <jpg> --format png -o out.png`; assert exit 0, output begins with the PNG
    signature `\x89PNG\r\n\x1a\n`, decodes at 32×16.
  - `convert_format_overrides_output_extension` — input PNG; run `convert <png>
    --format gif -o out.png` (note the `.png` extension); assert exit 0 and the
    output is actually GIF (`GIF8` magic) — proving the forced `--format` wins
    over the `-o` extension.
  - `convert_unbuilt_codec_exits_4` — `convert <png> --format avif` → exit 4;
    `convert <png> --format webp` → exit 4 (both not built — DEC-004).
  - `convert_unbuilt_codec_multi_input_exits_4_not_6` — two real PNGs +
    `--format avif --out-dir D` → exit **4** (assert `== Some(4)`, explicitly NOT
    6) — proves the target codec is resolved up front, before the fan-out.
  - `convert_multi_input_fan_out` — `solid_png` `a.png` + `solid_png` `b.png` →
    `convert a.png b.png --format jpg --out-dir D` → exit 0; `D/a.jpg` and
    `D/b.jpg` exist and are JPEG (`0xFF 0xD8`).
  - `convert_quality_lower_is_smaller` — a `gradient_jpeg`-sourced or
    `solid_png`-with-detail image; `convert <in> --format jpg -q 20 -o a.jpg` vs
    `-q 90 -o b.jpg`; assert `len(a.jpg) < len(b.jpg)`, both decode to the same
    dims. (Use an image with gradient/detail so quality affects size — a flat
    solid color compresses near-identically; prefer `gradient_jpeg` re-encoded or
    a multi-color fixture.)
  - `convert_missing_input_exits_3` — `convert no_such.png --format png` → exit 3.
  - `convert_multi_without_out_dir_is_usage_error` — two inputs, no `--out-dir`,
    `--format png` → exit 2; stderr contains `out-dir`.
  - `convert_requires_format_flag` — `convert <png>` with NO `--format` → exit 2
    (clap required-arg error); stderr mentions `--format`.
  - `convert_stdout_keeps_stdout_clean` — `convert <png> --format jpg -o -` →
    exit 0; stdout bytes decode as a JPEG image; stderr is empty.
  - REPOINT `stub_command_returns_not_implemented`: change its invocation from
    `convert <png> --format png` to `auto-orient <png>` (still a stub in this
    spec); keep the exit-1 + "not yet implemented" assertions. (`auto-orient`
    lands in SPEC-015; until then it is the remaining stub.)

> No new UNIT tests are required: `convert` is CLI-only and `run_convert` is a
> private fn fully exercised by the integration tests above; the encode path is
> already unit-tested (SPEC-013) and `output_format_for` is unchanged.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply
- `DEC-004` — pure-Rust codec policy / core format set (JPEG/PNG/GIF/BMP/TIFF/ICO);
  a native/unbuilt codec (AVIF) — and WebP (fast-follow, not yet wired) — returns
  **exit 4**. `convert`'s exit-4 path is the existing `resolve_format` →
  `SinkError::UnsupportedExtension`/`UnknownFormat` → `code() == 4`. **Resolve the
  target format ONCE up front** so an unbuilt codec is a single exit 4, never a
  per-input exit 6.
- `DEC-015` — output-format precedence (`--format` > `-o` ext > source) +
  partial-batch exit 6. `convert` forces `--format` for every input (precedence
  #1 as a forced override); genuine per-input **load/write** failures in a
  multi-input batch still go through `run_pixel_op` → exit 6.
- `DEC-016` — `-q` → JPEG quality (ignored for lossless formats). `convert`
  threads `global.quality` straight through (NO forced default — only `shrink`
  defaults quality). Reuses the SPEC-013 `encode_to_bytes` path verbatim.
- `DEC-014` — op-params/registry construction. NOT used here: `convert` runs an
  empty pipeline (no op), so there is nothing to build through the registry.
- `DEC-008` — resize backend. NOT used (no resize in `convert`).
- `DEC-003` — metadata dual-lane: the re-encode inherently drops container
  metadata (the `image` crate discards it on encode). `convert` does NOT touch
  the container lane; `--keep-gps`/selective preserve is STAGE-004.
- `DEC-012` / `DEC-007` — clap surface; typed errors → exit codes.

### Constraints that apply
- `ergonomic-defaults` — `convert photo.png --format jpg` is one short command;
  the only required flag is the inherent `--format` (you cannot convert without a
  target). No other boilerplate.
- `single-image-library` — re-encode uses the `image` crate only; no second lib.
- `pure-rust-codecs-default` — no native codec is pulled in; AVIF/WebP stay out
  (exit 4), consistent with DEC-004.
- `no-unwrap-on-recoverable-paths` — all failures are typed `CliError`/`SinkError`;
  no `unwrap`/`expect`/`panic!` in `src`.
- `every-public-fn-tested` — the changed `run_pixel_op` is covered by the existing
  resize/thumbnail/shrink tests (still green with `None`) plus the new convert
  tests (with `Some(fmt)`).
- `clippy-fmt-clean` — `cargo clippy --all-targets -- -D warnings` (the CURRENT
  CI gate) + `cargo fmt --check`.
- `untrusted-input-hardening` — the Sink's existing traversal/overwrite guards
  apply per input unchanged; no new untrusted surface.

### Prior related work
- `SPEC-011` (shipped, PR #12) — `run_pixel_op` + the DEC-015 fan-out you extend.
- `SPEC-012` (shipped, PR #13) — `run_thumbnail` (the caller shape you mirror).
- `SPEC-013` (shipped, PR #14) — the DEC-016 quality-aware `encode_to_bytes` /
  `Sink::write(quality)` `convert` reuses; `run_pixel_op` already carries
  `quality`. SPEC-013's Reflection explicitly named `convert` as the next reuse.

### Out of scope (create a new spec rather than expand)
- `auto-orient` (SPEC-015) — its own spec; the last STAGE-003 command.
- WebP output (fast-follow) and AVIF (feature-gated) — both exit 4 here; wiring a
  WebP/AVIF encoder is a later, separate change (DEC-004). Do NOT add a feature
  or dependency.
- Selective metadata preservation / `--keep-gps` (STAGE-004 container lane). The
  re-encode drops metadata inherently; write NO metadata code.
- rayon / parallel batch (STAGE-005). Fan-out stays sequential.
- Any new `Operation`, `CliError` variant, dependency, `Sink`/`encode_to_bytes`
  change, or `exit_code_mapping_is_total` change.

## Notes for the Implementer

- **`run_convert` is short** — resolve the target format up front, build an empty
  pipeline, call `run_pixel_op` with `Some(fmt)`. The sketch in Outputs is the
  whole function. Do NOT read the target format from `global.format` — the
  convert-local `--format` shadows the global one inside the `convert` subcommand,
  so `global.format` is `None` here; use the `format: &str` from
  `Commands::Convert`.
- **The empty pipeline is correct** — `Pipeline::new().run(img)` returns the
  image with pixels unchanged (the fold loop runs zero ops). `convert` is a pure
  re-encode; do NOT push `Identity` (unnecessary) and do NOT add an op.
- **Up-front format resolution is the key correctness point.** `resolve_format`
  is the single call that turns `--format avif` into `SinkError::UnsupportedExtension`
  (exit 4). Call it ONCE in `run_convert` before the fan-out so an unbuilt codec
  fails as a single exit 4 — never let it reach the per-input loop where it would
  be miscounted as a partial-batch exit 6.
- **Threading `forced_format` through `run_pixel_op`** is the only signature
  change. Add the param at the END (after `quality`). Update the three existing
  callers (`run_resize`, `run_thumbnail`, `run_shrink`) to pass `None`. Inside
  `run_pixel_op`, use `match forced_format { Some(f) => f, None =>
  output_format_for(...)? }` in BOTH the single and multi arms. Leave
  `output_format_for` (and `output_format_for_*` unit tests) untouched. `run_apply`
  does NOT call `run_pixel_op` — leave it alone.
- **`-q` has no default for `convert`.** Pass `global.quality` (an `Option<u8>`)
  directly; a JPEG target with no `-q` uses the encoder default (DEC-016). Only
  `shrink` forces quality 80.
- **`{ext}` in `--out-dir` names derives from the chosen format** —
  `extension_for_format(fmt)` already yields `jpg`/`png`/`gif`/… so `D/a.jpg`
  falls out automatically when converting to JPEG. No template work needed.
- **Quality-affects-size test:** a flat solid-color image compresses to nearly
  the same size at `-q 20` and `-q 90`; use `gradient_jpeg` (or a multi-color /
  noisy fixture) for `convert_quality_lower_is_smaller` so the assertion is
  robust. (SPEC-013's `shrink_quality_lower_is_smaller` used a gradient JPEG —
  mirror it.)
- **`Debug` on new public types:** none are added here, but do not `{:?}`-format
  a non-`Debug` type (a Sonnet build once hit two compile cycles on this).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-014-convert-command-re-encode-across-core-formats`
- **PR (if applicable):** PR #15 opened — https://github.com/jysf/crustyimg/pull/15
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - No new DEC during build — convert reuses DEC-004/015/016
- **Deviations from spec:**
  - None. All outputs implemented exactly as specified.
- **Follow-up work identified:**
  - SPEC-015 (`auto-orient`) is the remaining STAGE-003 stub; `stub_command_returns_not_implemented` now points to it.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing was unclear. The spec's implementation sketch (`run_convert` body, `forced_format` threading in both `run_pixel_op` arms) was precise and complete. The "GREP to be exhaustive" reminder in the build prompt was useful to ensure all four call sites were updated.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-004 (exit-4 up-front), DEC-015 (fan-out), and DEC-016 (quality threading) covered every code path. The spec's note that `global.format` is `None` inside the `convert` subcommand was the one sharp edge — without it a reader might try to read from `global.format`.

3. **If you did this task again, what would you do differently?**
   — Nothing material. The incremental commit discipline (implementation first, then tests) allowed a clean recovery point. The spec's fixture note ("use gradient_jpeg, not solid_png" for the quality test) was correct and worth following literally.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Little. The "scout the exact APIs in design" approach paid off twice: verifying
   that the convert-local `--format` shadows the global one (so the build reads from
   the variant, not `global.format`) and that an unbuilt codec must be resolved
   up front (single exit 4, not a per-input exit 6) turned the build into a clean,
   zero-deviation pass. The one durable lesson: when a command's output format is
   *forced and global to the invocation*, resolve it before the fan-out so a
   capability error doesn't masquerade as a partial-batch failure.

2. **Does any template, constraint, or decision need updating?**
   — No. `convert` reused DEC-004 (exit 4), DEC-015 (format precedence + exit 6),
   and DEC-016 (quality) cleanly with no new DEC. The `run_pixel_op` fan-out now
   carries both `quality` and `forced_format`, so `auto-orient` (SPEC-015)
   inherits both.

3. **Is there a follow-up spec I should write now before I forget?**
   — Only the already-planned SPEC-015 (`auto-orient`), the last STAGE-003 command —
   a new `AutoOrient` Operation that bakes EXIF orientation into pixels. (Design
   note for it: `image` 0.25.10 has `Orientation::from_exif_chunk` +
   `DynamicImage::apply_orientation`, so it can stay within the operation module's
   `::image` surface and needs no kamadak-exif.) Beyond that, `--keep-gps` /
   selective metadata preserve for `shrink`/`convert` remains a tracked STAGE-004
   container-lane item.
</content>
</invoke>
