---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-011
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
  implementer: claude-sonnet-4-6   # usually same Claude, different session
  created_at: 2026-06-15

references:
  decisions: [DEC-008, DEC-014, DEC-012, DEC-010, DEC-007, DEC-003, DEC-015]
  constraints:
    - ergonomic-defaults
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - test-before-implementation
    - untrusted-input-hardening
    - no-async-runtime
  related_specs: [SPEC-010, SPEC-004, SPEC-005, SPEC-007]

# One sentence on what this spec contributes to its stage's
# value_contribution.
value_link: "Wires the user-facing `resize` command on top of SPEC-010's Resize Operation — six mode flags, single-input `-o`/`-o -`, and SEQUENTIAL multi-input `--out-dir` fan-out with source-format preservation and partial-batch (exit 6) semantics — making `resize` the first STAGE-003 transform usable as one short command, and establishing the CLI fan-out + exit-6 pattern every later STAGE-003 command (thumbnail/shrink/convert/auto-orient) reuses."

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
      notes: "design cycle, Opus subagent; SPEC-011 = CLI half of split resize (SPEC-010 shipped the Operation). Emitted DEC-015 (output-format-preservation + partial-batch exit-6 policy)."
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "Sonnet build subagent wrote the code (src/cli + tests/cli) but its session dropped on an API error before running gates/committing; orchestrator (Opus) finished: fixed a clippy too_many_arguments (bundled the 6 mode flags into a ResizeModes struct), ran cargo fmt, verified all 4 gates green (146 tests), and completed bookkeeping + PR. The Failing-Tests integration suite was missed in recovery and added during the verify punch list (see verify session)."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "verify cycle, Opus read-only subagent. Proved all 11 ACs by hand; ⚠ PUNCH LIST — the spec's integration tests were never written (the dropped build + recovery only added unit tests). Sonnet follow-up added the 11 resize_* integration tests (157 tests total); re-confirmed green + CI 6/6. Then ✅."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "orchestrator ship bookkeeping on main (merged PR #12 squash 183c0c2; branch updated with main + CI re-validated rather than admin-bypassing branch protection; archive by hand)."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 4
---

# SPEC-011: resize cli command and multi input fan-out

## Context

This spec lands the **CLI half** of `resize`: the user-facing `resize`
command that parses the six mode flags, builds a `Resize` operation, runs it
through the pipeline, and writes outputs — including SEQUENTIAL multi-input
`--out-dir` fan-out.

- **Parent stage:** `STAGE-003` (transform and output). `resize` was assessed
  complexity **L** and **split** (approved): **SPEC-010** delivered the library
  (the `Resize` `Operation`, all six modes on the `fast_image_resize` SIMD
  backend, the `OperationParams` mechanism per DEC-014, registry registration,
  parity tests) — recipe-usable with zero CLI. **SPEC-011** (this spec) is the
  CLI command + multi-input fan-out. The split falls on the library↔CLI layering
  boundary. See the STAGE-003 backlog note (2026-06-15).
- **Project:** `PROJ-001` (crustyimg MVP).
- **Why now:** SPEC-010 shipped (PR #11, on `main`). The `Resize` op +
  `OperationRegistry::with_builtins().build("resize", &params)` path exists and
  is parity-tested. The `Commands::Resize` clap variant is already declared in
  `src/cli/mod.rs` but dispatches to `CliError::NotImplemented("resize")`. The
  `Sink`, `source::resolve`, and the `run_apply`/`run_view` structural templates
  all exist. This spec is **wiring on existing surfaces**: it adds a `run_resize`
  handler, a `WxH` parser, a flags→`OperationParams` mapper, per-input
  output-format resolution, an `ArgGroup` on `Commands::Resize`, and a new
  exit-6 `CliError` variant. No pixel-core or library changes.

The api-contract (`docs/api-contract.md`) pins the surface: `resize <INPUT...>
--max N | --exact WxH | --percent P | --fit WxH | --fill WxH | --cover WxH`
(mutually exclusive), multi-input + `--out-dir`, exit codes 2 (usage), 3 (not
found), 5 (write refused), **6 (partial batch failure)**.

## Goal

Wire the user-facing `resize` command: enforce exactly-one-of the six mode
flags (clap `ArgGroup`, exit 2), parse `WxH` strings, build the `Resize` op
through the registry (the same construction path recipes use, DEC-014), run it
through the `Pipeline`, and write outputs — a single `-o <file>`/`-o -` for one
resolved input, or SEQUENTIAL `--out-dir` name-template fan-out for many,
preserving each input's source format by default and exiting **6** on partial
batch failure. No rayon (STAGE-005), no metadata preservation (STAGE-004).

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — the binary boundary. `Commands::Resize { inputs:
    Vec<String>, max: Option<u32>, exact/fit/fill/cover: Option<String>,
    percent: Option<f32> }` is ALREADY declared (dispatched to
    `NotImplemented("resize")`). `run_apply` (recipe→pipeline→source→image→sink)
    and `run_view`/`run_info` are the structural templates. `build_sink` /
    `resolve_format` helpers, `GlobalArgs` (note `output`, `out_dir`,
    `name_template`, `format`, `quality`, `yes`). `CliError` + `code()` + the
    `exit_code_mapping_is_total` test (currently maps 1/2/3/4/5 — there is NO
    exit-6 path yet).
  - `src/operation/mod.rs` + `registry.rs` — `Resize::from_params(&OperationParams)
    -> Result<Resize, RegistryError>`; `OperationParams::{empty, from_map,
    get_str, get_u32, get_f32}`; `OperationRegistry::with_builtins().build(name,
    &params) -> Result<Box<dyn Operation>, RegistryError>`. The CLI builds a
    resize op the SAME way recipes do.
  - `src/pipeline/mod.rs` — `Pipeline` + `Pipeline::push` + `run(Image) ->
    Result<Image, OperationError>` (the executor the op flows through).
  - `src/source/mod.rs` — `source::resolve(arg, &mut reader) -> Result<Vec<Input>,
    SourceError>` (one arg can expand to many via glob/dir); `Input::{stem, path}`.
  - `src/sink/mod.rs` — `Sink::{File, Dir, Stdout}`, `Sink::write`, `SinkInput`,
    `Overwrite`, `expand_template`, `format_from_extension`,
    `extension_for_format`. NOTE: `Sink::Dir` DEFAULTS to PNG when `format` is
    `None` — this spec must NOT silently turn `a.jpg` into `a.png` (see Outputs
    §per-input format resolution + DEC-015).
  - `src/image/mod.rs` — `Image::{load, from_bytes, source_format}`. `source_format()`
    returns `image::ImageFormat`, used to preserve the input's format on output.
  - `docs/api-contract.md` — the `resize` entry + the Exit Codes table (esp.
    code 6).
  - `tests/cli.rs` + `tests/common/mod.rs` — integration conventions: drive the
    real binary via `env!("CARGO_BIN_EXE_crustyimg")` + `std::process::Command`,
    `write_test_png`, `tempfile`, `stdout_str`/`stderr_str` helpers, native
    fixtures only.
- **External APIs:** none new (clap is already a dep, DEC-012; no new crate).
- **Related code paths:** `src/cli/` ONLY. The pixel core (`src/image/`,
  `src/operation/`, `src/pipeline/`, `src/sink/`, `src/source/`) is read-only
  here — do NOT modify any library module.

## Outputs

- **Files modified:**
  - `src/cli/mod.rs` — the only source file modified. Specifically:

    **(a) `ArgGroup` on `Commands::Resize` (mode mutual-exclusivity → exit 2).**
    Add a clap `#[group(...)]` so exactly one mode flag is required and at most
    one allowed. clap enforces it and emits exit 2 (no runtime check needed):
    ```rust
    /// Resize one or more images (STAGE-003).
    #[command(group = clap::ArgGroup::new("mode")
        .required(true)
        .args(["max", "exact", "percent", "fit", "fill", "cover"]))]
    Resize {
        inputs: Vec<String>,
        #[arg(long)]
        max: Option<u32>,
        #[arg(long)]
        exact: Option<String>,
        #[arg(long)]
        percent: Option<f32>,
        #[arg(long)]
        fit: Option<String>,
        #[arg(long)]
        fill: Option<String>,
        #[arg(long)]
        cover: Option<String>,
    },
    ```
    Zero modes → clap error exit 2; two modes → clap error exit 2. (The existing
    `each_subcommand_help_parses` test runs `resize --help` which does NOT
    trigger the group requirement — `--help` short-circuits clap before group
    validation — so that test stays green.)

    **(b) `run_resize` handler.** Wire the dispatch arm
    `Commands::Resize { .. } => run_resize(...)` (replacing the
    `NotImplemented("resize")` arm). Signature:
    ```rust
    fn run_resize(
        inputs: &[String],
        max: Option<u32>,
        exact: Option<&str>,
        percent: Option<f32>,
        fit: Option<&str>,
        fill: Option<&str>,
        cover: Option<&str>,
        global: &GlobalArgs,
    ) -> Result<(), CliError>
    ```
    Flow:
    1. Build the resize `OperationParams` from the active flag (helper (d)).
    2. Build the op once via `OperationRegistry::with_builtins().build("resize",
       &params)` → `Box<dyn Operation>` (param errors → `CliError::Recipe(
       RecipeError::InvalidOperation { .. })` OR a dedicated mapping — see Notes
       "param errors"; map cleanly to a runtime error). Build a `Pipeline` with
       this one op (`Pipeline::new().push(op)` or equivalent).
    3. Resolve EVERY arg in `inputs` via `source::resolve`, flattening into one
       `Vec<Input>` (a single arg may expand to many via glob/dir). Empty inputs
       (no positional) → resolve `""`? No — `clap` makes `inputs` possibly empty;
       treat empty / all-empty resolution as `SourceError::NotFound` (exit 3).
    4. Decide single vs. multi by the FLATTENED resolved count:
       - **1 resolved input:** write via the `-o <file>`/`-o -` sink (the
         `build_sink` path). If `--out-dir` is set instead of `-o`, use the Dir
         sink (still fine for one input).
       - **>1 resolved inputs:** REQUIRE `--out-dir` (if absent → a typed
         `CliError` mapping to exit 2, usage — e.g. reuse a clear message; see
         Notes). Fan out SEQUENTIALLY (plain `for` loop, NO rayon).
    5. For each resolved input: load (`Image::load`/`from_bytes`) → `pipeline.run`
       → resolve the per-input output format (helper (e)) → write via the sink.
    6. **Partial failure:** collect per-input errors; if ANY input fails, print a
       per-failure summary to STDERR (`eprintln!`) and continue the loop. After
       the loop: if `failed == 0` → `Ok(())`. If `failed > 0` → `Err(
       CliError::PartialBatch { failed, total })` (exit 6) — INCLUDING the
       all-fail case (every input failed still exits 6 with the summary).
       Rationale: a single uniform batch-failure code; documented in DEC-015.
       (For the single-input path, a failure surfaces as that input's own typed
       `CliError` with its natural code — 3/1/4/5 — NOT exit 6; exit 6 is the
       multi-input contract. See DEC-015.)

    **(c) `parse_wxh` — `WxH`-string parser (CLI's job, pure, unit-tested).**
    ```rust
    /// Parse a `WxH` dimension string (e.g. "800x600") into (width, height).
    ///
    /// Both parts must be positive integers separated by a single ASCII 'x'
    /// (case-insensitive: 'x' or 'X'). A malformed string (no separator,
    /// empty part, non-integer, zero, negative, overflow) is a typed usage
    /// error. The op layer takes separate width/height keys, so WxH-string
    /// parsing lives HERE (DEC-014: recipes use flat width/height; the CLI
    /// translates its WxH ergonomics into those keys).
    fn parse_wxh(s: &str) -> Result<(u32, u32), CliError>
    ```
    A malformed `WxH` → `CliError` mapping to **exit 2** (usage error,
    consistent with clap's mode-exclusivity exit 2). Add a `CliError` variant or
    reuse one that maps to 2 — see Notes "WxH error code". Recommended: a
    dedicated `CliError::Usage(String)` variant (`code()` → 2) so the message is
    clear; this also gives the `--out-dir`-required case a home.

    **(d) `resize_params` — flags→`OperationParams` mapper (pure, unit-tested).**
    ```rust
    /// Map the active resize mode flag to the OperationParams the registry's
    /// "resize" constructor expects (the SPEC-010 PINNED schema): a `mode`
    /// string plus the per-mode dimension keys. Exactly one flag is set
    /// (clap's ArgGroup guarantees it); WxH strings are parsed via parse_wxh.
    fn resize_params(
        max: Option<u32>,
        exact: Option<&str>,
        percent: Option<f32>,
        fit: Option<&str>,
        fill: Option<&str>,
        cover: Option<&str>,
    ) -> Result<OperationParams, CliError>
    ```
    Mapping (mirrors SPEC-010's PINNED param-key schema, `docs/data-model.md`):
    - `--max N`       → `{ mode: "max",     width: N }`
    - `--exact WxH`   → `{ mode: "exact",   width: W, height: H }`
    - `--percent P`   → `{ mode: "percent", percent: P }`
    - `--fit WxH`     → `{ mode: "fit",     width: W, height: H }`
    - `--fill WxH`    → `{ mode: "fill",    width: W, height: H }`
    - `--cover WxH`   → `{ mode: "cover",   width: W, height: H }`
    Values are `toml::Value::Integer(n as i64)` for dims and
    `toml::Value::Float(p as f64)` for percent; collect into a
    `BTreeMap<String, toml::Value>` → `OperationParams::from_map`. If somehow no
    flag is set (clap should prevent it), return a `CliError::Usage` (defensive,
    not a panic). Range/validity of the dims themselves is enforced by
    `Resize::from_params` (the registry build), which surfaces its own typed
    error — `resize_params` only does the WxH split + map build.

    **(e) `output_format_for` — per-input output-format resolution (pure-ish,
    unit-tested where pure; DEC-015).**
    ```rust
    /// Decide the output ImageFormat for one input (DEC-015):
    ///   1. `--format FMT`        → that format (force; FMT via resolve_format).
    ///   2. else `-o <path>` ext  → inferred from the path extension.
    ///   3. else                  → PRESERVE the input's source_format().
    /// Returns the resolved ImageFormat. An unrecognized --format is a typed
    /// SinkError (exit 4) surfaced via resolve_format.
    fn output_format_for(
        global: &GlobalArgs,
        output_path: Option<&Path>,
        source_format: ImageFormat,
    ) -> Result<ImageFormat, CliError>
    ```
    This is the ergonomic core of DEC-015: `resize a.jpg --max 800 --out-dir d`
    writes `d/a.jpg` (NOT `d/a.png`). Because each input may differ (mixed
    jpg/png batch), the format is resolved PER-INPUT inside the fan-out loop, and
    the `Sink::Dir`/`Sink::File` is constructed with an EXPLICIT
    `format: Some(resolved)` so the sink never falls back to its PNG default.

    **(f) `CliError::PartialBatch { failed: usize, total: usize }`** + a
    `code()` arm returning **6**:
    ```rust
    /// One or more inputs in a multi-input batch failed (others may have
    /// succeeded). A per-failure summary is printed to stderr before this is
    /// returned. (api-contract exit code 6.)
    #[error("{failed} of {total} inputs failed")]
    PartialBatch { failed: usize, total: usize },
    ```
    And **(g) `CliError::Usage(String)`** (for malformed WxH and the
    `--out-dir`-required case) → `code()` returns **2**:
    ```rust
    /// A usage error detected at runtime (malformed WxH, multi-input without
    /// --out-dir). Mirrors clap's exit 2. Diagnostics go to stderr.
    #[error("{0}")]
    Usage(String),
    ```
    Extend `code()` with the `PartialBatch => 6` and `Usage(_) => 2` arms, and
    **extend `exit_code_mapping_is_total`** to assert both (see Failing Tests).

- **New exports / signatures:** all new items are private to `src/cli/mod.rs`
  (the binary boundary); the only PUBLIC-surface change is the two new
  `CliError` variants (`CliError` is `pub`):
  - `CliError::PartialBatch { failed: usize, total: usize }` (`code()` → 6).
  - `CliError::Usage(String)` (`code()` → 2).
  - Private fns: `run_resize`, `parse_wxh`, `resize_params`, `output_format_for`.
- **Database changes:** none.

### Output-format-preservation policy — PINNED (DEC-015)

The default output format for `resize` is the **input's source format**, NOT
the `Sink::Dir` PNG default. Precedence, per input:

1. `--format FMT` (explicit force) — overrides everything.
2. else the `-o <path>` extension (single-input file sink) — the path decides.
3. else **preserve** `img.source_format()` (the default for `--out-dir` and for
   `-o -` when no `--format` is given — though `-o -` realistically needs
   `--format` if the source is undeterminable; preserve covers the normal case).

This is a durable ergonomic policy (it governs every later STAGE-003 fan-out
command), so it is recorded as **DEC-015** rather than buried in the spec.

### Out-of-scope confirmation — metadata preservation (STAGE-004)

`resize` outputs **resized pixels in the target format; container metadata
(EXIF/ICC/orientation/copyright/GPS) is DROPPED**, because the current `Sink`
re-encodes via the `image` crate (`img.pixels().write_to(..)` in
`src/sink/mod.rs`), which discards all container metadata by nature, and the
metadata-WRITE crates (`img-parts`/`little_exif`) are NOT dependencies (that is
the STAGE-004 container lane). STAGE-003's scope line mentions "default-preserve
/ drop-GPS on pixel-lane encodes (DEC-003) via the existing Sink" — but genuine
default-preserve is **NOT achievable with today's Sink**; it requires the
container lane (STAGE-004) or a dedicated follow-up. This spec does NOT pull in
`img-parts`/`little_exif` and does NOT attempt metadata carry-over. (Verified
against `src/sink/mod.rs::encode_to_bytes` and DEC-003.)

## Acceptance Criteria

Each criterion maps to a test in **Failing Tests** (integration tests drive the
real binary; unit tests cover the pure helpers).

- [ ] AC1 — `resize <png> --max N -o out.png` exits 0; `out.png` exists and is
  decodable with N on its long edge (decode + assert dims). →
  `resize_max_single_input_writes_scaled`
- [ ] AC2 — `resize <png> --exact WxH -o out.png` exits 0; output is exactly
  W×H. → `resize_exact_single_input_exact_dims`
- [ ] AC3 — Multi-input `resize <two-files-or-dir> --max N --out-dir D` exits 0;
  each output exists in D with the expected scaled dims AND **preserved format**
  (a `.jpg` input stays `.jpg`, a `.png` input stays `.png`). →
  `resize_multi_input_fan_out_preserves_format`
- [ ] AC4 — Zero mode flags → exit 2 (clap ArgGroup). →
  `resize_no_mode_is_usage_error`
- [ ] AC5 — Two mode flags (`--max .. --exact ..`) → exit 2 (clap ArgGroup). →
  `resize_two_modes_is_usage_error`
- [ ] AC6 — Malformed `WxH` (`--exact abc`, `--exact 800x`, `--exact 0x10`) →
  exit 2. → `resize_bad_wxh_is_usage_error`
- [ ] AC7 — Missing input file → exit 3. → `resize_missing_input_exits_3`
- [ ] AC8 — Partial batch (one good + one undecodable input to `--out-dir`)
  exits **6**; the good input's output IS still written; stderr names the
  failure. → `resize_partial_batch_exits_6`
- [ ] AC9 — `resize <png> --max N -o -` (single input, stdout) exits 0; stdout
  is ONLY the encoded image bytes (decodes), stderr carries any diagnostics. →
  `resize_stdout_keeps_stdout_clean`
- [ ] AC10 — Multi-input WITHOUT `--out-dir` → exit 2 (usage). →
  `resize_multi_without_out_dir_is_usage_error`
- [ ] AC11 — `--format png` on a `.jpg` input forces PNG output (overrides
  preserve). → `resize_format_override_changes_format` (may be folded into a
  fan-out test asserting the output extension/decoded format)
- [ ] AC12 — `parse_wxh` unit coverage: `"800x600"` → `(800,600)`; case-`X`
  accepted; `"abc"`/`"800x"`/`"x600"`/`"0x10"`/`"800x0"`/`"-1x10"` → error. →
  `parse_wxh_*` unit tests
- [ ] AC13 — `resize_params` unit coverage: each mode flag → the correct
  `OperationParams` map (`mode` + minimal keys; `--max` has no `height`). →
  `resize_params_*` unit tests
- [ ] AC14 — `output_format_for` unit coverage: `--format` wins; else `-o` ext;
  else source format preserved. → `output_format_for_*` unit tests
- [ ] AC15 — `exit_code_mapping_is_total` is EXTENDED and green: `PartialBatch`
  → 6, `Usage` → 2 (plus all prior arms). → `exit_code_mapping_is_total`
- [ ] AC16 — the existing CLI integration suite stays green — in particular
  `each_subcommand_help_parses` (`resize --help` still exits 0 despite the
  required ArgGroup) and `help_lists_all_subcommands`. The
  `stub_command_returns_not_implemented` test (which currently asserts `resize
  … --max 800` exits 1 "not yet implemented") MUST be UPDATED: `resize` is no
  longer a stub. Replace it with a different still-stubbed command (e.g.
  `thumbnail`) so the stub-path coverage is retained. → asserted by the full
  suite + the updated stub test.

## Failing Tests

Written during **design**, BEFORE build. The implementer makes these pass.
Native in-memory fixtures only; drive the real binary for integration tests.

- **`src/cli/mod.rs`** unit tests (in the existing `#[cfg(test)] mod tests`;
  `use super::*`):
  - `parse_wxh_parses_valid` — `parse_wxh("800x600")` → `Ok((800, 600))`;
    `parse_wxh("1920X1080")` → `Ok((1920, 1080))` (uppercase `X` accepted).
    (AC12)
  - `parse_wxh_rejects_malformed` — each of `"abc"`, `"800x"`, `"x600"`,
    `"800"`, `"800x600x1"`, `"0x10"`, `"800x0"`, `"-1x10"`, `""` → `Err(_)`
    whose `.code()` is 2. (AC12, AC6)
  - `resize_params_max_minimal` — `resize_params(Some(20), None, None, None,
    None, None)` → params map `{mode:"max", width:20}` with NO `height` and NO
    `percent` key. (AC13)
  - `resize_params_exact_has_both_dims` — `resize_params(None, Some("33x77"),
    None, None, None, None)` → `{mode:"exact", width:33, height:77}`. (AC13)
  - `resize_params_percent` — `resize_params(None, None, Some(50.0), None, None,
    None)` → `{mode:"percent", percent:50.0}` (Float). (AC13)
  - `resize_params_fit_fill_cover` — each of `--fit`/`--fill`/`--cover` with
    `"40x40"` → `{mode:"<m>", width:40, height:40}` with the right mode literal.
    (AC13)
  - `resize_params_bad_wxh_is_usage` — `resize_params(None, Some("nope"), None,
    None, None, None)` → `Err` with `.code() == 2`. (AC6/AC13)
  - `output_format_for_format_flag_wins` — with `global.format = Some("png")`,
    `output_path = Some("/x/a.jpg")`, `source = Jpeg` → `Png`. (AC14)
  - `output_format_for_path_ext` — with `global.format = None`, `output_path =
    Some("/x/a.png")`, `source = Jpeg` → `Png`. (AC14)
  - `output_format_for_preserves_source` — with `global.format = None`,
    `output_path = None`, `source = Jpeg` → `Jpeg`. (AC14)
  - `exit_code_mapping_is_total` — EXTEND the existing test (do not duplicate)
    with: `CliError::PartialBatch { failed: 1, total: 3 }.code() == 6` and
    `CliError::Usage("bad".into()).code() == 2`. Keep all prior assertions.
    (AC15)

- **`tests/cli.rs`** integration tests (reuse `write_test_png`, add a JPEG
  fixture helper writing `tests/common`-style gradient JPEG bytes to a path, or
  inline it; drive the real binary; `tempfile`):
  - `resize_max_single_input_writes_scaled` — `write_test_png(dir,"in.png",100,
    50)`; run `resize <in.png> --max 20 -o <out.png>`; exit 0; `out.png` exists;
    `image::open` → 20×10 (long edge 20). (AC1)
  - `resize_exact_single_input_exact_dims` — 100×50 PNG; `resize <in> --exact
    33x77 -o <out.png>`; exit 0; decoded 33×77. (AC2)
  - `resize_multi_input_fan_out_preserves_format` — write a `a.png` (100×50) and
    a `b.jpg` (gradient JPEG, 100×50) into a dir; run `resize <dir-or-both>
    --max 20 --out-dir <outdir>`; exit 0; `outdir/a.png` decodes to 20×10 AND is
    PNG; `outdir/b.jpg` decodes to 20×10 AND is JPEG (assert via
    `image::ImageReader::open(..).with_guessed_format()?.format()` or by the file
    extension being preserved + decoding). (AC3, AC11-adjacent: preservation)
  - `resize_format_override_changes_format` — single `.jpg` input; `resize <in.jpg>
    --max 20 --format png -o <out.png>`; exit 0; output decodes as PNG. (AC11)
    (May be merged with the fan-out test if cleaner; keep at least one explicit
    `--format` override assertion.)
  - `resize_no_mode_is_usage_error` — `resize <in.png> -o <out>` (NO mode flag)
    → exit 2; output NOT created. (AC4)
  - `resize_two_modes_is_usage_error` — `resize <in.png> --max 20 --exact 10x10
    -o <out>` → exit 2. (AC5)
  - `resize_bad_wxh_is_usage_error` — `resize <in.png> --exact abc -o <out>` →
    exit 2; also assert `--exact 800x` → exit 2 (one test with two invocations
    is fine). (AC6)
  - `resize_missing_input_exits_3` — `resize <missing.png> --max 20 -o <out>` →
    exit 3. (AC7)
  - `resize_partial_batch_exits_6` — `--out-dir` batch of one valid PNG + one
    file with `.png` name but garbage bytes (undecodable); run `resize <both>
    --max 20 --out-dir <outdir>`; exit **6**; the VALID input's output exists in
    `outdir` and decodes; stderr mentions the failing file. (AC8)
  - `resize_stdout_keeps_stdout_clean` — single PNG; `resize <in.png> --max 20
    -o -`; exit 0; `image::load_from_memory(&output.stdout)` decodes to 20×10;
    (a known-format source preserves PNG on `-o -`). (AC9)
  - `resize_multi_without_out_dir_is_usage_error` — two PNGs as inputs, NO
    `--out-dir` and NO `-o` (or `-o <file>` for a multi-input) → exit 2; assert
    stderr mentions `--out-dir`. (AC10)
  - UPDATE `stub_command_returns_not_implemented` — change the driven command
    from `resize` to a STILL-stubbed command (`thumbnail <in> --size 64 -o
    <out>`), keeping the exit-1 + "not yet implemented" assertions. (AC16)

Run the FULL `cargo test`. The existing `tests/cli.rs` suite (help/version,
`apply_*`, `view_*`, `info_*`) and all unit tests must stay green.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-012` — clap is the CLI framework; the `Commands::Resize` variant + global
  args already exist. Use the `ArgGroup` derive attribute for mode
  mutual-exclusivity (clap owns exit 2). The pixel core must NOT depend on clap —
  all CLI logic stays in `src/cli/`.
- `DEC-014` — the operation-params mechanism. The CLI builds a resize op the
  SAME way recipes do: a `BTreeMap<String, toml::Value>` of `mode` + dims →
  `OperationParams::from_map` → `OperationRegistry::build("resize", &params)`.
  WxH-string parsing is the CLI's ergonomic translation INTO the flat width/
  height keys the op takes (the op never sees a `"WxH"` string).
- `DEC-008` — the resize backend (`fast_image_resize`) is internal to the op
  (SPEC-010). The CLI does NOT touch it; it just runs the op through the pipeline.
- `DEC-010` — `source::resolve` (glob/dir/stdin/single-file) is the source seam;
  each `inputs` arg resolves through it and flattens. The CLI does not
  re-implement globbing.
- `DEC-007` — typed errors → exit codes at the binary boundary. New typed
  variants `CliError::PartialBatch` (→ 6) and `CliError::Usage` (→ 2) live in the
  ONE `code()` mapping; the `exit_code_mapping_is_total` test guards it.
- `DEC-003` — metadata dual-lane. Confirms metadata preservation is the CONTAINER
  lane (STAGE-004), not the pixel-encode lane this command uses; `resize` drops
  container metadata (the Sink re-encodes). See Out-of-scope confirmation.
- `DEC-015` (emitted with this spec) — output-format-preservation default
  (preserve `source_format()` unless `--format` or a `-o` extension dictates
  otherwise) + partial-batch exit-6 semantics (any failure in a multi-input
  batch → exit 6, all-fail included; single-input failures keep their natural
  code).

### Constraints that apply

These apply to the paths this task touches (`src/cli/**`, `docs/api-contract.md`;
see `/guidance/constraints.yaml`):

- `ergonomic-defaults` — `resize a.jpg --max 800` is one short command; the
  output stays `a.jpg` (format preserved, DEC-015). No required boilerplate flag
  beyond the one mode flag the operation inherently needs.
- `no-unwrap-on-recoverable-paths` — NO `unwrap`/`expect`/`panic!` in
  `src/cli/`. Every fallible step (`parse_wxh`, `source::resolve`, `Image::load`,
  `pipeline.run`, `sink.write`, `resolve_format`) returns a typed `CliError`.
  The fan-out loop catches per-input errors and aggregates, never panics.
- `every-public-fn-tested` — the new pure helpers (`parse_wxh`, `resize_params`,
  `output_format_for`) get unit tests; `run_resize` is exercised by the
  integration suite. The two new `CliError` variants are covered by the extended
  `exit_code_mapping_is_total`.
- `clippy-fmt-clean` — `cargo clippy -- -D warnings` + `cargo fmt --check` clean.
  Watch integer casts in `parse_wxh`/dim mapping (`as i64`/`as f64`) and the
  `Option<&str>` borrowing in dispatch.
- `test-before-implementation` — the failing tests above are the contract.
- `untrusted-input-hardening` — the `Sink` already guards output traversal
  (`safe_join`) and overwrite (`guard_overwrite` / `--yes`); the fan-out reuses
  the Sink, so those guards are inherited. Do NOT bypass them. A traversal/
  overwrite refusal on one input is just that input's failure (counts toward the
  exit-6 tally).
- `no-async-runtime` — the fan-out is a plain sequential `for` loop. NO rayon, NO
  async. (DEC-006; parallelism is STAGE-005.)

### Prior related work

- `SPEC-010` (shipped, PR #11) — the `Resize` `Operation` + `OperationParams`
  mechanism + registry registration. `Resize::from_params` + `build("resize",
  &params)` are the construction path this CLI reuses. The PINNED param-key
  schema (`mode` + `width`/`height`/`percent`) is the mapping target for
  `resize_params`.
- `SPEC-004` (shipped) — `source::resolve` (the multi-input expansion seam).
- `SPEC-005` (shipped) — the `Sink` (File/Dir/Stdout), name-template expansion,
  traversal/overwrite guards, `format_from_extension`/`extension_for_format`.
- `SPEC-007`/`SPEC-008`/`SPEC-009` (shipped) — the clap CLI skeleton + dispatch
  + exit-code mapping (`run_apply`/`run_view`/`run_info` are the structural
  templates; `build_sink`/`resolve_format` are the helpers to reuse/extend).

### Out of scope (for this spec specifically)

If any of these feel necessary during build, write a new spec — do not expand
this one. If anything forces a change OUTSIDE `src/cli/` + `docs/api-contract.md`,
STOP and flag it.

- **Any library change** — `src/image/`, `src/operation/`, `src/pipeline/`,
  `src/sink/`, `src/source/` are READ-ONLY here. The resize op already exists
  (SPEC-010). If you think a Sink/op change is needed, STOP. (The format-
  preservation is achieved by passing `format: Some(resolved)` to the EXISTING
  `Sink` variants per input — no Sink edit.)
- **`rayon` / ANY parallelism / progress bars** — STAGE-005 (DEC-006). The
  fan-out is sequential.
- **Metadata preservation (default-preserve / drop-GPS carry-over)** — STAGE-004
  container lane. `resize` drops container metadata (see Out-of-scope
  confirmation). Do NOT add `img-parts`/`little_exif`.
- **`thumbnail` / `shrink` / `convert` / `auto-orient`** — later STAGE-003 specs
  (they reuse this command's fan-out + format-preservation + exit-6 pattern).
- **Quality-aware encode (`-q/--quality`)** — `resize` re-encodes at the
  encoder default; `--quality` is the `shrink`/`convert` story (later specs).
  `--format` IS honored (it overrides the preserved format). Document that
  `resize` ignores `--quality` for now (or note it as a no-op) — do NOT wire a
  quality-aware encoder here. (If `-q` must be threaded into the Sink, that's a
  Sink change → out of scope; defer.)
- **WebP / AVIF output** — DEC-004 (fast-follow / feature-gated).
- **A new top-level dependency** — none is needed (clap exists). If you think one
  is, STOP and add a question to `guidance/questions.yaml`.

## Notes for the Implementer

### Op construction (share the recipe path)

Build the params map then go through the registry — do NOT construct `Resize`
directly even though `Resize::from_params` is public; using
`OperationRegistry::with_builtins().build("resize", &params)` keeps the CLI and
recipes on ONE construction path (DEC-014/DEC-005) and means dim-range
validation (the op's job) is shared. Example:
```rust
let params = resize_params(max, exact, percent, fit, fill, cover)?;
let registry = OperationRegistry::with_builtins();
let op = registry
    .build("resize", &params)
    .map_err(/* RegistryError → a runtime CliError, see below */)?;
let mut pipeline = Pipeline::new();
pipeline.push(op);
```

### Param errors (RegistryError → CliError)

`registry.build("resize", &params)` returns `Result<_, RegistryError>`.
`CliError` has no direct `From<RegistryError>`. Two clean options — pick one and
be consistent:
- (preferred) Map `RegistryError::InvalidParams { reason, .. }` →
  `CliError::Usage(reason)` (exit 2) — a bad dim from the CLI IS a usage error,
  consistent with the WxH/mode exit-2 story; map `RegistryError::Unknown` →
  `CliError::Usage` too (defensive; "resize" is always registered). This keeps
  resize's bad-input failures all at exit 2.
- (alternative) Wrap via `RecipeError::InvalidOperation` and let the existing
  `CliError::Recipe(_) => 1` arm handle it (exit 1). Less ergonomic (a bad
  `--exact 0x10` becoming a generic runtime error reads worse than a usage
  error), so prefer the first. Whichever you choose, DOCUMENT it in Build
  Completion and make the WxH/dim error codes CONSISTENT across `parse_wxh`
  (exit 2) and the op's range rejection.
  > Decision for the build: use `CliError::Usage` (exit 2) for resize param
  > rejections so all malformed-resize-input paths are uniformly exit 2.

### WxH error code + the --out-dir-required code

`parse_wxh` failures and the "multi-input requires --out-dir" failure both map
to **exit 2** (usage). Use `CliError::Usage(String)` (`code()` → 2). This
matches clap's own exit-2 for the mode-exclusivity group, so ALL `resize` usage
errors are uniformly exit 2.

### Single vs. multi + the fan-out loop

- Flatten: `let mut all: Vec<Input> = Vec::new(); for arg in inputs { all.extend(
  source::resolve(arg, &mut stdin_lock)?); }` — but note `source::resolve`
  consumes stdin only for `"-"`; lock stdin once outside the loop, or pass a
  fresh lock per arg (only one `"-"` arg is meaningful). A missing single arg →
  its `SourceError::NotFound` propagates (exit 3) BEFORE the loop's
  partial-batch logic — i.e. resolution errors on the FIRST pass are hard
  errors (exit 3), not partial-batch (exit 6); partial-batch is for per-input
  LOAD/RUN/WRITE failures AFTER successful resolution. (Document this boundary;
  it matches "missing input → exit 3" AC7 vs. "undecodable input in batch →
  exit 6" AC8.)
- `all.is_empty()` → `CliError::Source(SourceError::NotFound(<joined args>))`
  (exit 3).
- `all.len() == 1` → build ONE sink from `build_sink`-style logic but with the
  per-input resolved format; load → run → write; a failure is that input's
  natural typed error (exit 3/1/4/5), NOT exit 6.
- `all.len() > 1` → require `global.out_dir.is_some()` else
  `CliError::Usage("multiple inputs require --out-dir".into())` (exit 2). Then a
  sequential loop; per input: load → run → resolve format → build a
  `Sink::Dir { dir, template, format: Some(resolved) }` (template from
  `--name-template` or the `"{stem}.{ext}"` default) → `write`. Catch each
  input's `Result`; on `Err(e)`, `eprintln!("error: {}: {e}", input_label)` and
  bump `failed`. After the loop, `failed > 0` → `CliError::PartialBatch {
  failed, total: all.len() }`.

### Per-input format → Sink construction (DEC-015)

For the Dir fan-out, construct the sink INSIDE the loop with the per-input
format so each output keeps its own format:
```rust
let fmt = output_format_for(global, None, img.source_format())?; // --out-dir: no -o path
let ext = crate::sink::extension_for_format(fmt);
let template = global.name_template.clone().unwrap_or_else(|| "{stem}.{ext}".to_owned());
let sink = Sink::Dir { dir: PathBuf::from(out_dir), template, format: Some(fmt) };
```
The `{ext}` token in the template expands to `extension_for_format(fmt)`
(handled inside `Sink::Dir::write` already — it calls `extension_for_format` on
the chosen format), so passing `format: Some(fmt)` makes BOTH the encoded bytes
AND the `{ext}` filename match the preserved format. (Confirm in
`src/sink/mod.rs::Sink::write` Dir arm: it does `let ext =
extension_for_format(fmt)` where `fmt = format.unwrap_or(Png)` — so
`Some(fmt)` overrides the PNG default for both. Good — no Sink change needed.)

For the single-input `-o <path>` case, the path extension already drives the
format in `Sink::File` (when `format: None`); but to honor DEC-015's `--format`
override and to keep behavior uniform, resolve via `output_format_for(global,
Some(path), img.source_format())` and pass `format: Some(fmt)`.

### Don't forget

- `Path` is already imported in `src/cli/mod.rs` (`use std::path::{Path,
  PathBuf}`). `ImageFormat` is reachable as `::image::ImageFormat`.
- `OperationRegistry` / `Pipeline` / `OperationParams` need `use` lines — check
  what's already imported at the top of `src/cli/mod.rs` and add only what's
  missing (`crate::operation::{OperationRegistry, OperationParams}`,
  `crate::pipeline::Pipeline`). `OperationRegistry` is already imported.
- Diagnostics go to STDERR (`eprintln!`); stdout stays clean for `-o -` (AC9).
- Derive `Debug` on any new public type, and don't `{:?}`-format types that
  don't impl `Debug` (a SPEC-010 lesson). The two new `CliError` variants are
  fields-only on an already-`Debug` enum — fine.

### Why no new DEC beyond DEC-015

DEC-015 captures the only genuinely new, durable policy (format-preservation
default + partial-batch exit-6 semantics) — it governs every later STAGE-003
fan-out command, so it earns a decision record. Everything else is wiring on
existing decisions (DEC-007/008/010/012/014/003). No second pixel library, no
new crate, no new architectural seam.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-011-resize-cli-command-and-multi-input-fan-out`
- **PR (if applicable):** #12
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - None during build — DEC-015 (emitted at design) governs; no new DEC.
- **Deviations from spec:**
  - Bundled the six mode flags into a private `ResizeModes<'a>` struct so
    `run_resize` takes 3 args instead of 8 (clippy `too_many_arguments`). Pure
    refactor; `resize_params` and its unit tests are unchanged.
  - The original build session dropped (API socket error) before adding the
    integration tests to `tests/cli.rs`. All 11 mandated integration tests were
    added in a verify-punch-list follow-up session (Sonnet, 2026-06-15); PR #12
    was updated. All 4 gates green at 157 tests (146 → +11).
- **Follow-up work identified:**
  - None new. Remaining STAGE-003 ops (thumbnail/shrink/convert/auto-orient)
    reuse this CLI + the SPEC-010 op/params mechanism + DEC-015 format policy.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing material; the spec named the registry/params construction path,
   the `ArgGroup`, the new CliError variants, and the per-input format
   resolution precisely. The only build incident was an infrastructure one: the
   Sonnet build session dropped on an API socket error after writing the code
   but before running gates/committing. The orchestrator recovered it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The `too_many_arguments` clippy lint on an 8-arg handler wasn't
   pre-empted by the spec; a standing build-prompt note to "bundle >7 args into
   a struct" would have avoided the fix (added as a lesson, alongside the
   SPEC-010 derive-Debug note).

3. **If you did this task again, what would you do differently?**
   — Run the four gates incrementally during the build (not just at the end) so
   a dropped session leaves a green, committed checkpoint rather than
   uncommitted work needing recovery.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Have build agents run gates and commit INCREMENTALLY, not just at the end.
   This build's Sonnet session dropped on an API error after writing code but
   before gates/commit; the orchestrator recovered it but missed that the
   spec's integration-test suite was never written (the unit tests passed, so
   "gates green" was true but incomplete). A green committed checkpoint after
   each chunk would have made the gap obvious. The cold verify caught it — which
   is exactly why the read-only verify cycle exists; it paid for itself here.

2. **Does any template, constraint, or decision need updating?**
   — Worth adding to the build-prompt boilerplate: an explicit end-of-build
   checklist item "confirm EVERY test named in the spec's `## Failing Tests`
   exists and runs" — a test COUNT alone (146 passing) doesn't prove the
   prescribed tests are present. No constraint/DEC change; DEC-015 holds.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec. SPEC-011 completes the `resize` feature (op in SPEC-010 +
   CLI here). The remaining STAGE-003 commands — `thumbnail`, `shrink`,
   `convert`, `auto-orient` — reuse this exact CLI fan-out + the DEC-015 format
   policy + the SPEC-010 op/params mechanism, so they need no new framing. The
   pre-existing `clippy --all-targets` dead-code debt (flagged by verify) is
   addressed by the open chore PR #10.
