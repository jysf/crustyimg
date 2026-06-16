---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-013
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
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
  decisions: [DEC-016, DEC-015, DEC-008, DEC-014, DEC-004, DEC-003, DEC-012, DEC-007]
  constraints:
    - ergonomic-defaults
    - single-image-library
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - test-before-implementation
    - untrusted-input-hardening
  related_specs: [SPEC-010, SPEC-011, SPEC-012, SPEC-005]

value_link: "Delivers STAGE-003's headline web-prep command — `shrink` makes 'optimize this image for the web' one short command (resize to a sane bound + real quality-aware JPEG encode + metadata drop), and wires the `-q/--quality` knob (DEC-016) that `convert` reuses."

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
      notes: "design cycle authored by the ORCHESTRATOR (Opus) directly, after two consecutive design-subagent sessions dropped on API socket errors. Emitted DEC-016 (encode quality policy). Complexity M (first library/Sink change in STAGE-003 — quality-aware encode)."
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "Build cycle: quality-aware encode in sink, run_shrink, 8 integration tests + 2 unit tests. All gates green (181 tests pass). encode_to_bytes made pub for integration test access."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "verify cycle, Opus read-only subagent. ✅ APPROVED, no punch list. Proved the quality knob hands-on (JPEG -q 20 → 1507 bytes vs -q 90 → 3611 bytes; PNG ignores quality; default bounds 1600 + smaller file); confirmed resize/thumbnail unchanged (21 tests), quality threaded through all sink.write callers, DEC-016 conformance, --all-targets + CI 6/6. Ruled the encode_to_bytes-pub deviation a non-issue (consistent with the module's 4 existing pub helpers)."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "orchestrator ship bookkeeping on main (merged PR #14 squash 48bc5fc; clean merge; archive by hand)."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 4
---

# SPEC-013: shrink command — web-prep resize + quality encode + strip

## Context

`shrink` is the headline "optimize for web" command (STAGE-003): one short
command that takes a photo and produces a smaller, web-ready file — **resize to
a sensible long-edge bound + a real quality-aware encode + drop metadata**.

- **Parent stage:** `STAGE-003` (transform & output). The fourth command, after
  `resize` op (SPEC-010), `resize` CLI (SPEC-011), and `thumbnail` (SPEC-012) —
  all shipped on `main`.
- **Why now / what's new:** `shrink` is the FIRST command that needs
  **quality-aware encoding**. The current Sink (`src/sink/mod.rs`
  `encode_to_bytes`) only calls `img.pixels().write_to(&mut cursor, format)`,
  which uses the `image` crate's *default* JPEG quality and offers no knob.
  `-q/--quality` already exists as a global CLI arg but is unwired (SPEC-011
  deferred it to "the shrink/convert story"). So unlike `thumbnail`, this spec
  touches the **library** (`src/sink`) to add a quality-aware encode path, plus
  the CLI. **DEC-016** records the durable encode-quality policy (`-q` → JPEG
  quality, ignored for lossless formats; `shrink` defaults to quality 80);
  `convert` (SPEC-014) will reuse the same plumbing.
- **What stays the same:** `shrink` reuses the shipped `Resize` op (`mode=max`)
  and the shared `run_pixel_op` fan-out (DEC-015: per-input source-format
  preservation, multi-input `--out-dir`, partial-batch exit 6). It does NOT
  introduce a new `Operation`.

The api-contract pins the surface: `shrink <INPUT...> [--max N] [-q Q]`.

## Goal

Wire the `shrink` command: resize each input to a default long-edge bound
(`--max` overrides), re-encode with a default quality (`-q` overrides; applies
to JPEG, ignored for lossless formats), preserving each input's source format
(DEC-015) and dropping container metadata (inherent to the pixel-lane
re-encode). Add the quality-aware encode path to the Sink and thread quality
through `run_pixel_op`, leaving `resize`/`thumbnail` behavior unchanged. No
rayon (STAGE-005); no selective metadata preserve / `--keep-gps` (STAGE-004).

## Inputs

- **Files to read:**
  - `src/sink/mod.rs` — `Sink::write`, `encode_to_bytes` (the encode path you
    extend), `SinkError::Encode`, the sink unit tests.
  - `src/cli/mod.rs` — `run_pixel_op` (the shared fan-out — you add a `quality`
    param), `run_resize`/`run_thumbnail` (callers; mirror `run_thumbnail` for
    `run_shrink`), `run_apply` (also calls `sink.write`), `build_sink`,
    `output_format_for`, `GlobalArgs` (the `quality: Option<u8>` field),
    `Commands::Shrink { inputs, max }`, `CliError` + `code()` +
    `exit_code_mapping_is_total`.
  - `src/operation/registry.rs` / `mod.rs` — `Resize` op + the
    `mode`/`width` param schema (`run_shrink` builds a resize op via the
    registry, like `run_thumbnail`).
  - `Cargo.toml` — `image` is pinned with the `jpeg` feature;
    `image::codecs::jpeg::JpegEncoder` is available. NO new dependency.
  - `tests/cli.rs`, `tests/sink.rs`, `tests/common/mod.rs` — conventions +
    `write_test_png`/`write_test_jpeg`/`gradient_jpeg`.
- **External APIs:** none new.
- **Related code paths:** `src/sink/` (encode change) + `src/cli/` (command).
  Do NOT modify other library modules.

## Outputs

- **Files modified:**
  - **`src/sink/mod.rs`** — quality-aware encode:
    - `encode_to_bytes(img: &Image, format: ImageFormat, quality: Option<u8>)
      -> Result<Vec<u8>, SinkError>`: when `format == ImageFormat::Jpeg` and
      `quality == Some(q)`, encode via
      `::image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, q)` —
      e.g. `img.pixels().write_with_encoder(encoder)` (verify the exact `image`
      0.25.10 method: `DynamicImage::write_with_encoder`, else
      `encoder.encode_image(img.pixels())`). Clamp `q` to `1..=100` before use
      (JPEG quality range; avoids surprising values). All other `(format,
      quality)` cases keep the existing `write_to(format)` path (quality
      ignored for lossless formats — DEC-016).
    - `Sink::write(&self, img, input, overwrite, quality: Option<u8>, out: &mut
      dyn Write)`: add the `quality` param; pass it to `encode_to_bytes` in the
      `File`/`Dir`/`Stdout` arms. The `Display` arm ignores `quality`.
  - **`src/cli/mod.rs`**:
    - `run_pixel_op(pipeline, inputs, global, quality: Option<u8>)`: add the
      `quality` param; pass it to BOTH `sink.write(...)` calls (single + multi).
    - `run_resize` / `run_thumbnail`: pass `global.quality` as the new arg
      (honor `-q` where supported; behavior unchanged when `-q` absent).
    - `run_apply`: pass `global.quality` to its `sink.write(...)` call.
    - NEW `fn run_shrink(inputs: &[String], max: Option<u32>, global:
      &GlobalArgs) -> Result<(), CliError>`: resolve `effective_max =
      max.unwrap_or(DEFAULT_SHRINK_MAX)`; build a `Resize` op via the registry
      with `mode="max"`, `width=effective_max` (a small helper
      `shrink_params(max) -> OperationParams` mirroring `thumbnail_params`);
      `pipeline = Pipeline::new().push(op)`; resolve `effective_quality =
      Some(global.quality.unwrap_or(DEFAULT_SHRINK_QUALITY))`; call
      `run_pixel_op(pipeline, inputs, global, effective_quality)`.
    - `const DEFAULT_SHRINK_MAX: u32 = 1600;` and `const DEFAULT_SHRINK_QUALITY:
      u8 = 80;` (web-prep defaults — see Notes).
    - Dispatch: `Commands::Shrink { inputs, max } => run_shrink(inputs, *max,
      &cli.global)`.
    - NO new `CliError` variant; NO `code()`/`exit_code_mapping_is_total` change.
  - **`tests/sink.rs`** — update existing `sink.write(...)` calls for the new
    `quality` param (pass `None`); add the quality encode unit tests (below).
  - **`docs/api-contract.md`** — pin shrink's defaults (done in DESIGN if needed;
    otherwise the build leaves it).
- **New decisions:** `DEC-016` (emitted in design) — encode quality policy.
- **No new dependency. No new `Operation`. No library module beyond `src/sink`.**

## Acceptance Criteria

Each maps to a test.

- [ ] `shrink <jpg>` (no flags) resizes to ≤ `DEFAULT_SHRINK_MAX` on the long
  edge and writes a JPEG smaller than the input. → `shrink_defaults_*`
- [ ] `shrink <jpg> --max 100` bounds the long edge to 100. → `shrink_max_*`
- [ ] A lower `-q` yields a SMALLER JPEG than a higher `-q` on the same image
  and resize. → `shrink_quality_lower_is_smaller`
- [ ] `shrink <png>` resizes a PNG to the default bound, output is PNG, `-q`
  ignored, no error. → `shrink_png_preserves_format_quality_ignored`
- [ ] Multi-input `shrink a.png b.jpg --out-dir D` → exit 0, each scaled, each
  keeps its source format. → `shrink_multi_input_fan_out_preserves_format`
- [ ] `shrink <missing>` → exit 3. → `shrink_missing_input_exits_3`
- [ ] Multi-input without `--out-dir` → exit 2. → `shrink_multi_without_out_dir_is_usage_error`
- [ ] `shrink <jpg> -o -` → stdout is the encoded image, stderr empty. → `shrink_stdout_keeps_stdout_clean`
- [ ] Sink unit: a JPEG encoded at low quality is smaller than at high quality
  (same image); a PNG encode is identical regardless of `quality`. →
  `encode_jpeg_quality_*`
- [ ] `resize`/`thumbnail` outputs are unchanged when no `-q` is passed (their
  existing tests stay green); the existing sink tests stay green.

## Failing Tests

Written during **design**, made to pass during **build**. Mirror `tests/cli.rs`
+ `tests/sink.rs` conventions (drive the real binary; native in-memory
fixtures; `tempfile`; trim stdout; assert exit codes via `output.status.code()`;
decode outputs with `image::open`/`image::load_from_memory`).

- **`tests/sink.rs`** (UNIT — the library encode change)
  - `encode_jpeg_quality_lower_is_smaller` — encode the same `DynamicImage` to
    JPEG via the sink at `Some(20)` vs `Some(90)`; assert the low-quality byte
    length < high-quality byte length, and both decode to the same dimensions.
  - `encode_png_ignores_quality` — encode a PNG at `Some(10)` and `None`; assert
    byte-identical output (quality ignored for lossless formats, DEC-016).
  - Update existing sink-write tests to pass `None` for the new `quality` param
    (compile fix; assertions unchanged).
- **`tests/cli.rs`** (INTEGRATION — add to the existing file; reuse
  `write_test_png`/`write_test_jpeg`)
  - `shrink_defaults_bound_long_edge_and_shrink_file` — a JPEG larger than
    `DEFAULT_SHRINK_MAX` on the long edge (e.g. 2000×1000) → `shrink <jpg>` →
    exit 0; output long edge == 1600 (== DEFAULT_SHRINK_MAX), output file size <
    input file size.
  - `shrink_max_bounds_long_edge` — `shrink <jpg> --max 100` on 200×100 → output
    100×50.
  - `shrink_quality_lower_is_smaller` — same JPEG, `shrink --max 200 -q 20 -o a.jpg`
    vs `-q 90 -o b.jpg`; assert `a.jpg` smaller than `b.jpg`, both decode at the
    same dims.
  - `shrink_png_preserves_format_quality_ignored` — `shrink <png> --max 100 -q 10`
    → output is PNG (magic bytes), 100×... , exit 0 (quality ignored, no error).
  - `shrink_multi_input_fan_out_preserves_format` — a PNG + a JPEG →
    `shrink a.png b.jpg --max 64 --out-dir D` → exit 0; `D/a.png` PNG, `D/b.jpg`
    JPEG, each long edge ≤ 64.
  - `shrink_missing_input_exits_3` — missing file → exit 3.
  - `shrink_multi_without_out_dir_is_usage_error` — two inputs, no `--out-dir` →
    exit 2; stderr mentions `--out-dir`.
  - `shrink_stdout_keeps_stdout_clean` — `shrink <jpg> --max 64 -o -` → exit 0;
    stdout decodes as an image; stderr empty.
  - REPOINT `stub_command_returns_not_implemented` from `shrink` → `convert`
    (e.g. `convert <png> --format png` — still a stub) so it keeps asserting the
    NotImplemented path on a still-stubbed command.

## Implementation Context

### Decisions that apply
- `DEC-016` — encode quality policy: `-q` → JPEG quality via
  `JpegEncoder::new_with_quality`; ignored for lossless formats; `shrink`
  defaults to quality 80. THE governing decision for this spec.
- `DEC-015` — output-format preservation (`--format` > `-o` ext > source) +
  partial-batch exit 6: inherited free via `run_pixel_op`. Do not convert
  PNG→JPEG silently.
- `DEC-008` — resize backend; `shrink`'s resize step is the shipped `Resize`
  `mode=max`.
- `DEC-014` — op-params construction path (`shrink` builds the resize op via the
  registry, like `resize`/`thumbnail`).
- `DEC-004` — pure-Rust codec policy; `JpegEncoder` is in the existing `image`
  dep, no new crate. The lossless core formats take no `-q`.
- `DEC-003` — metadata dual-lane: `shrink`'s "strip metadata" is the INHERENT
  drop on the pixel-lane re-encode (the `image` crate discards container
  metadata). Selective default-preserve / `--keep-gps` is the STAGE-004
  container lane and is OUT OF SCOPE here.
- `DEC-012` / `DEC-007` — clap surface; typed errors → exit codes.

### Constraints that apply
- `ergonomic-defaults` — `shrink photo.jpg` must "just work": default max 1600 +
  default quality 80, no required flags.
- `single-image-library` — quality JPEG encode uses the `image` crate's
  `JpegEncoder`; NO second image library, no `imageproc`.
- `no-unwrap-on-recoverable-paths` — encode/IO failures are typed
  `SinkError`/`CliError`; no `unwrap`/`expect`/`panic!` in `src`.
- `every-public-fn-tested` — the changed `encode_to_bytes` path + `run_shrink`'s
  helpers get tests (unit for encode quality; integration for the command).
- `clippy-fmt-clean` — `cargo clippy --all-targets -- -D warnings` (the CURRENT
  CI gate) + `cargo fmt --check`.
- `untrusted-input-hardening` — `q` clamped to `1..=100`; the resize op already
  caps oversize dims.

### Prior related work
- `SPEC-005` (shipped) — the `Sink` + `encode_to_bytes` this extends.
- `SPEC-011` (shipped) — `run_pixel_op` (you thread `quality` through it) + the
  DEC-015 fan-out; SPEC-011 explicitly DEFERRED `-q` to here.
- `SPEC-012` (shipped) — `run_thumbnail`/`thumbnail_params` (the shape
  `run_shrink`/`shrink_params` mirror).

### Out of scope (create a new spec rather than expand)
- `convert` (SPEC-014) and `auto-orient` — own specs. `convert` will REUSE this
  spec's quality plumbing (DEC-016).
- Selective metadata preservation / `--keep-gps` (STAGE-004 container lane).
  `shrink` drops ALL metadata in STAGE-003; `--keep-gps` is a no-op for `shrink`
  until STAGE-004. State this in the api-contract; do NOT pull in
  `img-parts`/`little_exif`.
- rayon / parallel batch (STAGE-005). Fan-out stays sequential.
- WebP/AVIF output (DEC-004), PNG compression-level control (a future explicit
  flag, not `-q`).
- Any new `Operation`, `CliError` variant, dependency, or `exit_code_mapping_is_total`
  change.

## Notes for the Implementer

- **`run_shrink` mirrors `run_thumbnail`** (SPEC-012): build a resize
  `OperationParams` → registry → pipeline → `run_pixel_op`, but pass the
  defaulted quality. Sketch:
  ```text
  fn run_shrink(inputs, max: Option<u32>, global) -> Result<(), CliError> {
      let params = shrink_params(max.unwrap_or(DEFAULT_SHRINK_MAX));   // {mode:"max", width:N}
      let op = OperationRegistry::with_builtins().build("resize", &params)
          .map_err(|e| match e { RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
                                 RegistryError::Unknown { name } => CliError::Usage(format!("unknown operation '{name}'")) })?;
      let pipeline = Pipeline::new().push(op);
      let quality = Some(global.quality.unwrap_or(DEFAULT_SHRINK_QUALITY));
      run_pixel_op(pipeline, inputs, global, quality)
  }
  ```
- **Threading `quality` is the bulk of the change.** Add the param to
  `run_pixel_op` and to `Sink::write`/`encode_to_bytes`; update EVERY caller:
  `run_resize`/`run_thumbnail` pass `global.quality`; `run_shrink` passes the
  defaulted quality; `run_apply` passes `global.quality`; the `tests/sink.rs`
  `write(...)` calls pass `None`. Grep for `.write(` and `encode_to_bytes(` to
  find them all.
- **JPEG encoder API (verify against `image` 0.25.10):** prefer
  `img.pixels().write_with_encoder(::image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, q))`.
  If `write_with_encoder` isn't the right method, use
  `JpegEncoder::new_with_quality(&mut cursor, q).encode_image(img.pixels())`.
  Map any error to `SinkError::Encode(e.to_string())`. Clamp `q.clamp(1, 100)`.
- **Defaults rationale:** `DEFAULT_SHRINK_MAX = 1600` (a common "large web image"
  long edge — fits most layouts without being huge); `DEFAULT_SHRINK_QUALITY =
  80` (the standard web JPEG quality sweet spot — visually clean, meaningfully
  smaller). Both overridable (`--max`, `-q`).
- **`-q` now also reaches `resize`/`thumbnail`** (they pass `global.quality`).
  This is intentional and harmless — their existing tests don't pass `-q`, so
  output is unchanged; a user who does `resize p.jpg -q 50` now gets quality 50.
  Document only in passing; the headline quality command is `shrink`.
- **Metadata:** do NOT write any metadata-strip code — the re-encode already
  drops it. Just don't claim selective preservation.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-013-shrink-command-web-prep-resize-quality-encode-strip`
- **PR (if applicable):** #14 https://github.com/jysf/crustyimg/pull/14
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - No new DEC during build — DEC-016 already governs.
- **Deviations from spec:**
  - `encode_to_bytes` made `pub` (not `pub(crate)`) to allow access from the integration test crate (`tests/sink.rs`). The spec describes it as a "private helper" but the prescribed tests call it directly. Making it `pub` is the minimal change that satisfies both the test requirement and the `every-public-fn-tested` constraint.
- **Follow-up work identified:**
  - SPEC-014 (`convert`) will reuse the quality plumbing from this spec per DEC-016.
  - STAGE-004: `--keep-gps` wiring for `shrink`.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — The visibility of `encode_to_bytes` was ambiguous: the spec described it as a private helper but the test section requires calling it directly from an integration test file (which is a separate crate). The spec's `## Notes for the Implementer` section says to verify the `write_with_encoder` API path — that was clear but required checking that `DynamicImage::write_with_encoder` exists in image 0.25.10 (it does).

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The `encode_to_bytes` visibility gap: the constraints list `every-public-fn-tested` but the function was originally private. The spec should have noted that the prescribed unit tests in `tests/sink.rs` require it to be `pub` (or suggested an alternative like testing through `Sink::Stdout`).

3. **If you did this task again, what would you do differently?**
   — Run `cargo fmt` before each commit rather than after clippy (it saves one round-trip). Also would read `tests/sink.rs` more carefully first to understand the full list of `sink.write` call sites before starting edits, rather than grepping after.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — The design was authored by the orchestrator directly after two design
   subagents dropped on API socket errors. That worked (clean build + clean
   verify), but it's a fallback — when the API is flaky, a fresh-agent design
   still produces better-isolated rigor. One small spec wart: I labeled the
   `tests/sink.rs` tests "UNIT", which pushed `encode_to_bytes` to `pub`; either
   call them integration tests (they are — separate crate) or place them inline.
   Verify ruled it a non-issue (the module's sibling helpers are already pub),
   so no harm, but the spec wording could be tighter.

2. **Does any template, constraint, or decision need updating?**
   — No. DEC-016 cleanly governs the quality policy and `convert` (SPEC-014)
   will reuse the exact `quality` plumbing. The `run_pixel_op` shared helper now
   carries `quality` too, so convert/auto-orient inherit it.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec beyond the planned `convert` (SPEC-014) and `auto-orient`
   (SPEC-015). `convert` is the natural next: it reuses this spec's quality
   encode + DEC-016 + DEC-015 format handling, adding a required `--format`
   target (and exit 4 for unbuilt codecs, DEC-004). `--keep-gps`/selective
   metadata preserve remains a tracked STAGE-004 item (the container lane).
