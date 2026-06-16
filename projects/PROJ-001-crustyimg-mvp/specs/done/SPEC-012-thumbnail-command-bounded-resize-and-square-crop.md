---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-012
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
  implementer: claude-sonnet-4-6   # usually same Claude, different session
  created_at: 2026-06-15

references:
  decisions: [DEC-015, DEC-014, DEC-012, DEC-008, DEC-010, DEC-007, DEC-003]
  constraints:
    - ergonomic-defaults
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - test-before-implementation
    - untrusted-input-hardening
    - no-async-runtime
  related_specs: [SPEC-010, SPEC-011]

# One sentence on what this spec contributes to its stage's
# value_contribution.
value_link: "Adds the `thumbnail` convenience command — a bounded small resize (default long-edge cap) with `--square` center-crop — by mapping `(size, square)` onto the SHIPPED `Resize` op (`--square` ≡ resize `fill` NxN; plain ≡ resize `max` N) and reusing SPEC-011's CLI fan-out + DEC-015 format/exit-6 machinery via a shared `run_pixel_op` helper, delivering STAGE-003's `thumbnail` deliverable as one short command with zero new pixel code."

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
      notes: "design cycle, Opus subagent. Confirmed thumbnail needs NO new Operation — `--square` maps to resize `fill` (cover+center-crop, shipped SPEC-010), plain maps to resize `max`. Specified a `run_pixel_op` shared fan-out helper extracted from SPEC-011's `run_resize` (both call it), the `(size,square)`→OperationParams mapper, default --size 256. No new DEC (reuses DEC-015/014/012/008/010/007/003). Complexity S."
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 25
      recorded_at: 2026-06-15
      notes: "subagent; cost not separately reported. Clean first-pass build — incremental commits + an explicit test-existence checklist (SPEC-011 lessons) applied; --all-targets gate green."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "verify cycle, Opus read-only subagent. ✅ APPROVED, no punch list. Confirmed run_pixel_op refactor byte-for-byte faithful (all resize_* tests green), thumbnail semantics hands-on (256×128 default, 64×64 square, JPEG preserved, --size 0→exit 2, no upscale), DEC-015 inherited, all 14 tests present, --all-targets + CI 6/6 green."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-15
      notes: "orchestrator ship bookkeeping on main (merged PR #13 squash 95e664f; clean merge, no branch-protection issue; archive by hand)."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 4
---

# SPEC-012: thumbnail command bounded resize and square crop

## Context

`thumbnail` is the third STAGE-003 command, after `resize` shipped as
**SPEC-010** (the `Resize` `Operation` + the `OperationParams` mechanism, DEC-014)
and **SPEC-011** (the `resize` CLI command + multi-input `--out-dir` fan-out +
DEC-015 format/exit-6 policy). It is a **convenience** command: a bounded small
resize, with `--square` center-cropping to an exact square.

The central design insight — verified against the shipped `Resize` op
(`src/operation/mod.rs`, registered in `src/operation/registry.rs`) — is that
**`thumbnail` needs NO new `Operation`.** It reuses `Resize`'s already-shipped
modes:

- **`thumbnail --size N --square`** ≡ **`resize --fill NxN`.** SPEC-010's `fill`
  mode is *cover then center-crop to exactly W×H* (the PINNED six-mode math:
  `s = max(W/w, H/h)`, resize, then center-crop to `(W, H)`). With `W = H = N`
  that is exactly "produce an N×N square by covering then centre-cropping" — the
  thumbnail-square semantics.
- **`thumbnail --size N`** (non-square) ≡ **`resize --max N`.** SPEC-010's `max`
  mode bounds the **longest edge to N**, aspect preserved, **never upscaling**
  (`s = min(N/max(w,h), 1.0)`) — exactly "a bounded small image".

So `run_thumbnail` maps `(size, square)` → a `Resize` `OperationParams` (`mode=fill`
+ `width=height=N` for square; `mode=max` + `width=N` otherwise), builds the op via
`OperationRegistry::with_builtins().build("resize", &params)` (the **same**
construction path `run_resize` and recipes use, DEC-014), and runs the **same**
multi-input fan-out SPEC-011 already wrote.

- **Parent stage:** `STAGE-003` (transform and output). The stage scope line
  defines `thumbnail` as "a convenience bounded resize with `--square`
  center-crop"; success criterion: "`thumbnail` produces a bounded small image;
  `--square` center-crops."
- **Project:** `PROJ-001` (crustyimg MVP).
- **Why now:** SPEC-011 shipped (PR #12, on `main`). `Commands::Thumbnail {
  inputs: Vec<String>, size: Option<u32>, square: bool }` is ALREADY declared in
  `src/cli/mod.rs` and dispatches to `CliError::NotImplemented("thumbnail")`. The
  `Resize` op (all six modes), the registry/params construction path, the
  `output_format_for` per-input format resolution, the `CliError::PartialBatch`
  (→6) / `CliError::Usage` (→2) variants, and the entire resolve→fan-out→exit-6
  loop in `run_resize` all exist on `main`. This spec is **wiring on existing
  surfaces** plus one small DRY refactor: add a `run_thumbnail` handler + a
  `(size, square)`→params mapper, and **extract** the fan-out body of `run_resize`
  into a shared `run_pixel_op` helper that both handlers call.

The api-contract (`docs/api-contract.md`) pins the surface: `thumbnail
<INPUT...> [--size N] [--square]` (S3).

## Goal

Wire the user-facing `thumbnail` command: map `(--size N, --square)` to a `Resize`
`OperationParams` (`fill` NxN when `--square`, else `max` N; `--size` defaults to
**256** when omitted), build the op through the registry (the recipe/resize path,
DEC-014), and run it through the **same** multi-input fan-out as `resize` —
single `-o`/`-o -`, sequential `--out-dir` fan-out for many, per-input
source-format preservation, partial-batch exit 6 (DEC-015) — by extracting that
fan-out into a shared `run_pixel_op` helper. No new `Operation`, no rayon
(STAGE-005), no metadata preservation (STAGE-004), no quality-aware encode.

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — the binary boundary. `Commands::Thumbnail { inputs:
    Vec<String>, size: Option<u32>, square: bool }` is ALREADY declared
    (dispatched to `NotImplemented("thumbnail")` in `dispatch`). **`run_resize`
    is the structural template AND the refactor target** — read it in full: its
    op-build → `Pipeline::new().push(op)` → `source::resolve` flatten →
    single-vs-multi → per-input `output_format_for` → `Sink` → partial-batch
    exit-6 loop is EXACTLY what `thumbnail` needs (it differs from `resize` only
    in how the op's params are produced). `resize_params`, `output_format_for`,
    `ResizeModes`, the `CliError` enum + `code()` + `exit_code_mapping_is_total`,
    `GlobalArgs`, `Overwrite`. The `CliError::PartialBatch`/`Usage` variants
    already exist.
  - `src/operation/mod.rs` + `src/operation/registry.rs` — the `Resize` op + its
    `mode`/`width`/`height` param schema; `OperationParams::{empty, from_map,
    get_str, get_u32, get_f32}`; `OperationRegistry::with_builtins().build(name,
    &params) -> Result<Box<dyn Operation>, RegistryError>`. Confirm `"resize"`'s
    `fill` and `max` modes (the registry registers `resize`; `Resize::from_params`
    validates). The thumbnail builds a **resize** op — there is no `"thumbnail"`
    registry key.
  - `src/pipeline/mod.rs` — `Pipeline::new()` + `push(Box<dyn Operation>) -> Self`
    (BUILDER-style; consumes self) + `run(Image) -> Result<Image, OperationError>`.
  - `src/source/mod.rs` — `source::resolve(arg, &mut reader) -> Result<Vec<Input>,
    SourceError>`; `Input::{stem, path}` + the `Input::Path`/`Input::Stdin` arms.
  - `src/sink/mod.rs` — `Sink::{File, Dir, Stdout}`, `Sink::write`, `SinkInput`,
    `Overwrite`, `format_from_extension`, `extension_for_format`. `Sink::Dir`
    DEFAULTS to PNG when `format` is `None` — pass `format: Some(resolved)`
    per-input (DEC-015), as `run_resize` already does.
  - `src/image/mod.rs` — `Image::{load, from_bytes, source_format}`;
    `source_format()` → `image::ImageFormat` (drives format preservation).
  - `docs/api-contract.md` — the `thumbnail <INPUT...> [--size N] [--square]`
    entry + the Exit Codes table (2 usage, 3 not found, 4 unsupported format, 5
    write refused, 6 partial batch).
  - `tests/cli.rs` — integration conventions: `BIN`
    (`env!("CARGO_BIN_EXE_crustyimg")`), `write_test_png`, `write_test_jpeg`
    (added in SPEC-011), `stdout_str`/`stderr_str`, `tempfile`; the `resize_*`
    tests are the template; `stub_command_returns_not_implemented` currently
    drives `thumbnail` and MUST be repointed (see Failing Tests).
- **External APIs:** none new (clap already a dep, DEC-012; the resize backend is
  internal to the op, DEC-008; no new crate).
- **Related code paths:** `src/cli/` ONLY. The pixel core (`src/image/`,
  `src/operation/`, `src/pipeline/`, `src/sink/`, `src/source/`) is READ-ONLY here
  — do NOT modify any library module.

## Outputs

- **Files modified:**
  - `src/cli/mod.rs` — the only source file modified. Specifically:

    **(a) `run_thumbnail` handler** + the dispatch arm. Replace
    `Commands::Thumbnail { .. } => Err(CliError::NotImplemented("thumbnail"))`
    with a destructuring arm calling `run_thumbnail`. Signature:
    ```rust
    fn run_thumbnail(
        inputs: &[String],
        size: Option<u32>,
        square: bool,
        global: &GlobalArgs,
    ) -> Result<(), CliError>
    ```
    Flow:
    1. `let params = thumbnail_params(size, square);` (helper (b); infallible —
       `size` defaults to `DEFAULT_THUMBNAIL_SIZE` and the mapping is total).
    2. Build the op via `OperationRegistry::with_builtins().build("resize",
       &params)`, mapping `RegistryError` → `CliError::Usage` exactly as
       `run_resize` does (`InvalidParams { reason, .. } => CliError::Usage(reason)`,
       `Unknown { name } => CliError::Usage(format!("unknown operation '{name}'"))`).
       (Defensive: with the default size and the pinned schema this never fails,
       but keep the typed mapping — no `unwrap`.)
    3. `let pipeline = Pipeline::new().push(op);`
    4. Call `run_pixel_op(pipeline, inputs, global)` (helper (c)) and return its
       `Result`.

    **(b) `thumbnail_params` — `(size, square)`→`OperationParams` mapper (pure,
    unit-tested).**
    ```rust
    /// The default long-edge bound when `--size` is omitted (DEFAULT below).
    const DEFAULT_THUMBNAIL_SIZE: u32 = 256;

    /// Map thumbnail args to the `Resize` OperationParams the registry expects
    /// (SPEC-010's PINNED schema). `thumbnail` is a convenience over `resize`:
    ///   - `--square` → resize `fill` N×N  (cover + center-crop to exactly N×N)
    ///   - else       → resize `max`  N     (bound the long edge to N, no upscale)
    /// `size` defaults to `DEFAULT_THUMBNAIL_SIZE` (256). Infallible: the mapping
    /// is total and the op validates the dims (N >= 1 always holds here).
    fn thumbnail_params(size: Option<u32>, square: bool) -> OperationParams
    ```
    Mapping (mirrors SPEC-010's PINNED param-key schema; values are
    `toml::Value::Integer(n as i64)`, collected into a `BTreeMap<String,
    toml::Value>` → `OperationParams::from_map`):
    - `square == true`  → `{ mode: "fill", width: N, height: N }`
    - `square == false` → `{ mode: "max",  width: N }`

    where `N = size.unwrap_or(DEFAULT_THUMBNAIL_SIZE)`. (Note: `--size 0` parses
    at the clap layer as a `u32` `0`; mapping it produces `width: 0`, which
    `Resize::from_params` rejects as `RegistryError::InvalidParams` → surfaced as
    `CliError::Usage` exit 2 in step (a)/(2). So `thumbnail_params` stays
    infallible and the op layer guards the degenerate value —
    `untrusted-input-hardening`.)

    **(c) `run_pixel_op` — the shared fan-out helper (the DRY refactor).**
    Extract the body of `run_resize` *after* the op is built into a reusable
    private helper, and have BOTH `run_resize` and `run_thumbnail` build their op
    (a `Pipeline`) then call it:
    ```rust
    /// Run a built single-op `Pipeline` over one-or-many resolved inputs and
    /// write the outputs — the shared CLI fan-out for pixel commands (DEC-015).
    ///
    /// - Resolves every `inputs` arg via `source::resolve`, flattening to one
    ///   `Vec<Input>`; a resolution error (missing path / empty glob) is a HARD
    ///   error (exit 3/2), NOT partial-batch; an empty result → `NotFound`
    ///   (exit 3).
    /// - 1 input  → single `-o`/`-o -`/`--out-dir` sink, per-input format via
    ///   `output_format_for`; a failure keeps its natural code (3/1/4/5).
    /// - >1 inputs → REQUIRE `--out-dir` (else `CliError::Usage`, exit 2);
    ///   SEQUENTIAL fan-out (no rayon); per-input failures are collected, a
    ///   summary printed to stderr, and exit 6 returned if any failed (DEC-015).
    fn run_pixel_op(
        pipeline: Pipeline,
        inputs: &[String],
        global: &GlobalArgs,
    ) -> Result<(), CliError>
    ```
    The helper contains EXACTLY the logic currently in `run_resize` from Step 3
    onward (the `let mut all: Vec<Input>` resolve/flatten, the `all.is_empty()`
    → `NotFound`, the `Overwrite` computation, the `all.len() == 1` single-sink
    branch with `output_format_for(global, output_path, img.source_format())`,
    and the `else` multi-input `--out-dir` sequential loop with per-input
    `output_format_for(global, None, …)` + `Sink::Dir { format: Some(fmt) }` +
    the `eprintln!` + `failed`/`PartialBatch` accounting). It takes the built
    `Pipeline` by value (consumed by `pipeline.run(img.clone())` per input, as
    today). NOTE: `pipeline.run` takes `img.clone()` per input (already the case
    in `run_resize`); since `Pipeline` is used by reference inside the loop via
    `pipeline.run`, confirm `Pipeline::run(&self, …)` borrows (it does — `run`
    takes `&self`), so one `Pipeline` value serves all inputs.

    **(d) `run_resize` is refactored to call `run_pixel_op`.** After building its
    op + `let pipeline = Pipeline::new().push(op);`, `run_resize`'s body becomes:
    ```rust
    run_pixel_op(pipeline, inputs, global)
    ```
    `run_resize` keeps its signature (`inputs`, `&ResizeModes<'_>`, `global`) and
    its op-construction (the `resize_params` + `registry.build` + the
    `RegistryError` → `CliError::Usage` mapping). Only the fan-out tail moves into
    `run_pixel_op`. **All existing `resize_*` integration tests + the
    `resize_params`/`output_format_for` unit tests MUST stay green** — they guard
    this refactor (behavior is unchanged; the code is relocated).

- **New exports / signatures:** all new items are PRIVATE to `src/cli/mod.rs`
  (the binary boundary). NO public-surface change: no new `CliError` variant
  (`PartialBatch`/`Usage` already exist), no `code()` change, no
  `exit_code_mapping_is_total` change. Private fns/consts added:
  `run_thumbnail`, `thumbnail_params`, `run_pixel_op`, `DEFAULT_THUMBNAIL_SIZE`.
- **Database changes:** none.

### Square / non-square semantics — PINNED

| invocation | resize equivalent | output |
|---|---|---|
| `thumbnail in --size N` | `resize --max N` | longest edge = N, aspect preserved, **never upscaled** |
| `thumbnail in --size N --square` | `resize --fill NxN` | **exactly** N×N (cover, then **center-crop**) |
| `thumbnail in` (no `--size`) | `resize --max 256` | longest edge ≤ 256 (DEFAULT) |
| `thumbnail in --square` (no `--size`) | `resize --fill 256x256` | exactly 256×256 |

`--size 0` → `width: 0` → op rejects → `CliError::Usage` (exit 2).

### Default size — PINNED (`ergonomic-defaults`)

`--size` defaults to **256**. Rationale: 256 is the conventional upper bound for
a "thumbnail" (gallery tiles, avatars, previews are typically 128–256 on the long
edge); it is small enough to be unambiguously a thumbnail yet large enough to stay
useful on modern/retina displays. This makes `thumbnail photo.jpg -o thumb.jpg`
work as one short command with no required flag — the `ergonomic-defaults` case.
Documented in the spec and `docs/api-contract.md`.

### Format preservation + fan-out + exit 6 — INHERITED (DEC-015)

`thumbnail` inherits, FOR FREE via `run_pixel_op`, every behavior SPEC-011 pinned
for `resize`:
- single `-o <file>` / `-o -`; multi `--out-dir` (REQUIRED for >1 input, else
  `CliError::Usage` → exit 2);
- per-input source-format preservation (a `.jpg` thumbnail stays `.jpg`, a `.png`
  stays `.png`; `--format` overrides; `-o <path>` extension decides for the
  single-file case) — DEC-015;
- partial-batch exit 6 (any per-input failure → exit 6 after writing the
  successes + a stderr summary; all-fail included); single-input failures keep
  their natural code (3/1/4/5).

No new `CliError` variant; no `exit_code_mapping_is_total` change (confirmed —
both codes already exist and are tested).

### Out-of-scope confirmation — metadata + quality (STAGE-004 / later S3)

Like `resize`, `thumbnail` outputs **resized pixels in the target format;
container metadata (EXIF/ICC/orientation/copyright/GPS) is DROPPED** because the
`Sink` re-encodes via the `image` crate (DEC-003; the metadata-write crates are
not dependencies — that is the STAGE-004 container lane). `-q/--quality` is NOT
honored (encoder default), same as `resize`; quality-aware encode is the
`shrink`/`convert` story.

## Acceptance Criteria

Each criterion maps to a test in **Failing Tests** (integration tests drive the
real binary; unit tests cover the pure mapper).

- [ ] AC1 — `thumbnail <png>` (no `--size`) exits 0; the output's **long edge ==
  256** (the default), aspect preserved, no upscale. →
  `thumbnail_default_size_bounds_long_edge`
- [ ] AC2 — `thumbnail <png> --size 64` exits 0; long edge == 64, aspect
  preserved (a 100×50 source → 64×32). → `thumbnail_size_bounds_long_edge`
- [ ] AC3 — `thumbnail <png> --size 64 --square` exits 0; output is **exactly
  64×64** (cover + center-crop). → `thumbnail_square_is_exact_square`
- [ ] AC4 — `--size 64` does NOT upscale a smaller source (a 40×30 source stays
  40×30). → `thumbnail_does_not_upscale`
- [ ] AC5 — Multi-input `thumbnail a.png b.jpg --size 64 --out-dir D` exits 0;
  each output exists in D, scaled, with **preserved format** (`a.png` stays PNG,
  `b.jpg` stays JPEG). → `thumbnail_multi_input_fan_out_preserves_format`
- [ ] AC6 — Missing input file → exit 3. → `thumbnail_missing_input_exits_3`
- [ ] AC7 — Multi-input WITHOUT `--out-dir` → exit 2 (usage; stderr mentions
  `--out-dir`). → `thumbnail_multi_without_out_dir_is_usage_error`
- [ ] AC8 — `thumbnail <png> --size 64 -o -` (single input, stdout) exits 0;
  stdout is ONLY the encoded image bytes (decodes), stderr carries diagnostics. →
  `thumbnail_stdout_keeps_stdout_clean`
- [ ] AC9 — Partial batch (one good PNG + one undecodable `.png` to `--out-dir`)
  exits **6**; the good input's output IS still written; stderr names the
  failure. → `thumbnail_partial_batch_exits_6`
- [ ] AC10 — `thumbnail <png> --size 0` → exit 2 (op rejects width 0 →
  `CliError::Usage`). → `thumbnail_size_zero_is_usage_error`
- [ ] AC11 — `thumbnail_params` unit coverage: `(Some(64), false)` →
  `{mode:"max", width:64}` (NO height); `(Some(64), true)` → `{mode:"fill",
  width:64, height:64}`; `(None, false)` → `{mode:"max", width:256}`; `(None,
  true)` → `{mode:"fill", width:256, height:256}`. → `thumbnail_params_*` unit
  tests.
- [ ] AC12 — the existing CLI suite stays green: the `resize_*` integration tests
  + `resize_params_*`/`output_format_for_*`/`parse_wxh_*`/`exit_code_mapping_is_total`
  unit tests (they guard the `run_pixel_op` refactor of `run_resize`), plus
  `help_lists_all_subcommands` / `each_subcommand_help_parses`. The
  `stub_command_returns_not_implemented` test (which currently drives
  `thumbnail`) MUST be REPOINTED to a still-stubbed command — `shrink` — keeping
  the exit-1 + "not yet implemented" assertions. → asserted by the full suite +
  the updated stub test.

## Failing Tests

Written during **design**, BEFORE build. The implementer makes these pass.
Native in-memory fixtures only; drive the real binary for integration tests.

- **`src/cli/mod.rs`** unit tests (in the existing `#[cfg(test)] mod tests`;
  `use super::*`):
  - `thumbnail_params_max_default` — `thumbnail_params(None, false)` → a params
    map with `mode == "max"`, `width == 256`, and **no** `height` key
    (`get_u32("height").is_none()`). (AC11)
  - `thumbnail_params_max_sized` — `thumbnail_params(Some(64), false)` →
    `{mode:"max", width:64}`, no `height`. (AC11)
  - `thumbnail_params_square_default` — `thumbnail_params(None, true)` →
    `{mode:"fill", width:256, height:256}`. (AC11)
  - `thumbnail_params_square_sized` — `thumbnail_params(Some(64), true)` →
    `{mode:"fill", width:64, height:64}`. (AC11)

  (Assert via `p.get_str("mode")`, `p.get_u32("width")`, `p.get_u32("height")` on
  the returned `OperationParams`, mirroring the existing `resize_params_*` tests.)

- **`tests/cli.rs`** integration tests (reuse `write_test_png`,
  `write_test_jpeg`; drive the real binary; `tempfile`; the `resize_*` tests are
  the template):
  - `thumbnail_default_size_bounds_long_edge` — `write_test_png(dir,"in.png",
    1000,500)`; run `thumbnail <in.png> -o <out.png>` (no `--size`); exit 0;
    `image::open` → width 256, height 128 (long edge bounded to the 256 default,
    aspect preserved). (AC1)
  - `thumbnail_size_bounds_long_edge` — 100×50 PNG; `thumbnail <in> --size 64 -o
    <out.png>`; exit 0; decoded 64×32. (AC2)
  - `thumbnail_square_is_exact_square` — 100×50 PNG; `thumbnail <in> --size 64
    --square -o <out.png>`; exit 0; decoded **exactly 64×64**. (AC3)
  - `thumbnail_does_not_upscale` — 40×30 PNG; `thumbnail <in> --size 64 -o
    <out.png>`; exit 0; decoded 40×30 (unchanged — `max` never upscales). (AC4)
  - `thumbnail_multi_input_fan_out_preserves_format` — write `a.png` (100×50) and
    `b.jpg` (100×50 via `write_test_jpeg`) into a dir; run `thumbnail <a.png>
    <b.jpg> --size 64 --out-dir <outdir>`; exit 0; `outdir/a.png` decodes to
    64×32 AND is PNG; `outdir/b.jpg` decodes to 64×32 AND is JPEG (assert via the
    preserved extension decoding successfully + `image::ImageReader::open(..)
    .with_guessed_format()?.format()`, mirroring
    `resize_multi_input_fan_out_preserves_format`). (AC5)
  - `thumbnail_missing_input_exits_3` — `thumbnail <missing.png> --size 64 -o
    <out>` → exit 3; no output created. (AC6)
  - `thumbnail_multi_without_out_dir_is_usage_error` — two PNGs as inputs, NO
    `--out-dir` (and no `-o`) → exit 2; stderr mentions `--out-dir`. (AC7)
  - `thumbnail_stdout_keeps_stdout_clean` — single PNG; `thumbnail <in.png>
    --size 64 -o -`; exit 0; `image::load_from_memory(&output.stdout)` decodes
    (64 on the long edge); stdout is ONLY image bytes. (AC8)
  - `thumbnail_partial_batch_exits_6` — `--out-dir` batch of one valid PNG + one
    file with a `.png` name but garbage bytes (undecodable); run `thumbnail
    <both> --size 64 --out-dir <outdir>`; exit **6**; the VALID input's output
    exists in `outdir` and decodes; stderr mentions the failing file. (AC9)
  - `thumbnail_size_zero_is_usage_error` — 100×50 PNG; `thumbnail <in> --size 0
    -o <out>` → exit 2 (the op rejects width 0 as `InvalidParams` → `Usage`); no
    output created. (AC10)
  - REPOINT `stub_command_returns_not_implemented` — change the driven command
    from `thumbnail` to a STILL-stubbed command (`shrink <in> --max 64 -o
    <out>`), keeping the exit-1 + "not yet implemented" assertions. (AC12)

Run the FULL `cargo test`. The existing `tests/cli.rs` suite (help/version,
`apply_*`, `view_*`, `info_*`, **all `resize_*`**) and all unit tests (incl. the
`resize_params_*` / `output_format_for_*` that guard the `run_pixel_op` refactor)
must stay green.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-015` — output-format-preservation default (preserve `source_format()`
  unless `--format` or a `-o` extension dictates) + partial-batch exit-6
  semantics. DEC-015 explicitly governs **every later STAGE-003 fan-out command**
  — thumbnail included. `thumbnail` inherits this verbatim through the shared
  `run_pixel_op` helper; do NOT re-decide it.
- `DEC-014` — the operation-params mechanism. `thumbnail` builds a **resize** op
  the SAME way recipes and `run_resize` do: a `BTreeMap<String, toml::Value>` of
  `mode` + dims → `OperationParams::from_map` →
  `OperationRegistry::build("resize", &params)`. There is no `"thumbnail"`
  operation; the convenience lives entirely in the CLI mapping.
- `DEC-012` — clap is the CLI framework; `Commands::Thumbnail { inputs, size,
  square }` already exists. The pixel core must NOT depend on clap — all CLI logic
  stays in `src/cli/`. `--size`/`--square` need no `ArgGroup` (they are not
  mutually exclusive; `--square` is an orthogonal boolean).
- `DEC-008` — the resize backend (`fast_image_resize`) is internal to the op. The
  CLI does NOT touch it; thumbnail just runs the resize op through the pipeline.
  `fill`'s cover+center-crop is implemented in `Resize::apply` (SPEC-010).
- `DEC-010` — `source::resolve` (glob/dir/stdin/single-file) is the source seam;
  `run_pixel_op` resolves each `inputs` arg through it and flattens. No
  re-implemented globbing.
- `DEC-007` — typed errors → exit codes at the binary boundary. No new variant:
  `thumbnail` reuses `CliError::{PartialBatch (→6), Usage (→2), Source, Image,
  Sink, Operation}` and their existing `code()` arms.
- `DEC-003` — metadata dual-lane: metadata preservation is the CONTAINER lane
  (STAGE-004), not the pixel-encode lane this command uses; `thumbnail` drops
  container metadata on re-encode (see Out-of-scope confirmation).

### Constraints that apply

These apply to the paths this task touches (`src/cli/**`, `docs/api-contract.md`;
see `/guidance/constraints.yaml`):

- `ergonomic-defaults` — `thumbnail photo.jpg -o thumb.jpg` is one short command
  with a sensible DEFAULT size (256) and source-format preserved (DEC-015). No
  required flag.
- `no-unwrap-on-recoverable-paths` — NO `unwrap`/`expect`/`panic!` in
  `src/cli/`. `thumbnail_params` is infallible (total mapping); the op build,
  resolve, load, run, write all return typed `CliError`; the fan-out loop
  (in `run_pixel_op`) catches per-input errors and aggregates, never panics.
- `every-public-fn-tested` — the new pure helper `thumbnail_params` gets unit
  tests; `run_thumbnail`/`run_pixel_op` are exercised by the integration suite
  (and `run_pixel_op` additionally by the inherited `resize_*` tests). (All new
  items are private; the constraint targets public fns, but these get coverage
  regardless per the spec.)
- `clippy-fmt-clean` — `cargo clippy --all-targets -- -D warnings` + `cargo fmt
  --check` clean. Watch the `n as i64` casts in `thumbnail_params` (same pattern
  as `resize_params`).
- `test-before-implementation` — the failing tests above are the contract.
- `untrusted-input-hardening` — the `Sink`'s traversal/overwrite guards are
  inherited via `run_pixel_op` (do NOT bypass). `--size 0` is rejected by the op
  as a typed error (→ exit 2), never a panic/degenerate alloc; the op's oversize
  cap (SPEC-010: ≤ 50_000 px/edge, area ≤ 268M) still bounds large `--size`.
- `no-async-runtime` — the fan-out is the same sequential `for` loop. NO rayon,
  NO async (DEC-006; parallelism is STAGE-005).

### Prior related work

- `SPEC-010` (shipped, PR #11) — the `Resize` `Operation` + the `OperationParams`
  mechanism. Provides the `fill` (cover+center-crop) and `max` (long-edge bound,
  no upscale) modes thumbnail maps onto, and the registry `build("resize",
  &params)` construction path.
- `SPEC-011` (shipped, PR #12) — the `resize` CLI command + multi-input
  `--out-dir` fan-out + DEC-015 (format preservation + exit-6) + the
  `output_format_for`/`CliError::PartialBatch`/`CliError::Usage` surfaces +
  `write_test_jpeg`. **`run_resize` is the template and the `run_pixel_op`
  refactor source.**
- `PR #10` — the `clippy --all-targets` cleanup (CI now gates with
  `--all-targets`; build must use it).

### Out of scope (for this spec specifically)

If any of these feel necessary during build, write a new spec — do not expand
this one. If anything forces a change OUTSIDE `src/cli/` + `tests/cli.rs` +
`docs/api-contract.md`, STOP and flag it (add a question to
`guidance/questions.yaml`).

- **Any library change** — `src/image/`, `src/operation/`, `src/pipeline/`,
  `src/sink/`, `src/source/` are READ-ONLY. NO new `Thumbnail` `Operation` (it
  maps onto the shipped `Resize`); NO Sink/op/source edit. Format preservation is
  threaded via the existing `format: Some(_)` field, exactly as `run_resize`
  does.
- **A new `Operation` or registry key** — thumbnail is CLI-only; recipes can
  already express it via `resize` (`fill`/`max`), so no `"thumbnail"` op is
  warranted (it would duplicate `resize` semantics for no gain).
- **A new `CliError` variant / `code()` change** — none needed (`PartialBatch`,
  `Usage` already exist).
- **`rayon` / ANY parallelism / progress bars** — STAGE-005 (DEC-006). Sequential.
- **Metadata preservation (default-preserve / drop-GPS carry-over)** — STAGE-004
  container lane. `thumbnail` drops container metadata. Do NOT add
  `img-parts`/`little_exif`.
- **Quality-aware encode (`-q/--quality`)** — `thumbnail` re-encodes at the
  encoder default (same as `resize`); `--quality` is the `shrink`/`convert` story.
- **`shrink` / `convert` / `auto-orient`** — later STAGE-003 specs (they too will
  reuse `run_pixel_op`).
- **A new top-level dependency** — none needed.

## Notes for the Implementer

### The map, in one paragraph

`thumbnail` is `resize` with two pre-baked mode choices. `--square` → resize
`fill NxN`; plain → resize `max N`; `--size` defaults to 256. Build the resize
`OperationParams`, build the op via the registry (same as `run_resize`), push it
into a `Pipeline`, and hand the `Pipeline` to the shared `run_pixel_op` helper.
Everything else (resolve, single-vs-multi, per-input format, exit 6) is the SAME
code `resize` already runs.

### The DRY refactor (do this cleanly)

Extract `run_resize`'s tail (everything after `let pipeline =
Pipeline::new().push(op);`) into `run_pixel_op(pipeline: Pipeline, inputs:
&[String], global: &GlobalArgs) -> Result<(), CliError>`. Then both handlers are
thin: build params → build op (with the `RegistryError` → `CliError::Usage`
mapping) → `Pipeline::new().push(op)` → `run_pixel_op(pipeline, inputs, global)`.
The relocated logic is BYTE-FOR-BYTE the existing fan-out — do not change its
behavior; the `resize_*` integration tests + the `output_format_for_*` unit tests
must stay green to prove it. (Future `shrink`/`convert`/`auto-orient` reuse
`run_pixel_op` too — this is the payoff.)

### Op build + Pipeline

```rust
let params = thumbnail_params(size, square);
let op = OperationRegistry::with_builtins()
    .build("resize", &params)
    .map_err(|e| match e {
        RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
        RegistryError::Unknown { name } => CliError::Usage(format!("unknown operation '{name}'")),
    })?;
let pipeline = Pipeline::new().push(op);
run_pixel_op(pipeline, inputs, global)
```

`thumbnail_params` is INFALLIBLE (returns `OperationParams`, not `Result`) — the
default makes `size` always present and the schema is fixed; the only invalid
value (`--size 0`) is caught at the op-build step (`InvalidParams` → `Usage` →
exit 2), which is why `run_thumbnail` still does the typed `map_err`. Don't make
`thumbnail_params` return a `Result` — keep the degenerate-value guard in the op
(single source of truth for dim validity, DEC-014).

### Dispatch arm

In `dispatch`, replace
`Commands::Thumbnail { .. } => Err(CliError::NotImplemented("thumbnail")),` with:
```rust
Commands::Thumbnail { inputs, size, square } =>
    run_thumbnail(inputs, *size, *square, &cli.global),
```

### Don't forget

- Diagnostics → STDERR (`eprintln!`); stdout stays clean for `-o -` (AC8) — this
  is already true inside `run_pixel_op` (the relocated `run_resize` body).
- `DEFAULT_THUMBNAIL_SIZE` is a `const u32 = 256` near the resize helpers; cite it
  in the doc-comment so the default is discoverable.
- No new public type, so no `Debug`-derive concern (the SPEC-010 lesson) arises
  here — but keep the standing rule in mind for any helper struct you might add
  (you should NOT need one).
- `docs/api-contract.md`'s `thumbnail` line should be expanded to pin the default
  size (256) and the `--square` = cover+center-crop semantics (see Outputs;
  minimal edit).

### Why no new DEC

No genuinely new, durable decision arises. The two policies a STAGE-003 fan-out
command needs — output-format-preservation default + partial-batch exit-6 — are
already DEC-015, which explicitly governs thumbnail. The op-construction path is
DEC-014; clap is DEC-012; the resize backend/modes are DEC-008/SPEC-010; the
source seam is DEC-010; typed errors→codes is DEC-007; metadata-lane separation
is DEC-003. The default-size choice (256) and the `(size,square)`→`(fill|max)`
mapping are spec-local ergonomics, not a cross-cutting architectural decision —
they live in this spec + the api-contract, not a DEC. **No DEC-016 emitted.**

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-012-thumbnail-command-bounded-resize-and-square-crop`
- **PR (if applicable):** PR #13 opened — https://github.com/jysf/crustyimg/pull/13
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - None — thumbnail reuses DEC-015/014/012/008/010/007/003 (see "Why no new DEC")
- **Deviations from spec:**
  - None. Library modules (`src/operation/`, `src/sink/`, `src/source/`, `src/image/`,
    `src/pipeline/`) were NOT modified (confirmed). `exit_code_mapping_is_total` was NOT
    changed. No new `CliError` variant added. No new dependency. The `run_pixel_op`
    refactor is behavior-preserving: all existing `resize_*` integration tests and
    `resize_params_*`/`output_format_for_*` unit tests stay green.
- **Follow-up work identified:**
  - None beyond the already-planned STAGE-003 specs (shrink, convert, auto-orient)
    which will also reuse `run_pixel_op`.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing materially unclear. The spec was prescriptive about `run_pixel_op`'s
   exact body (byte-for-byte relocation from `run_resize`). The only friction was
   clippy doc-comment lint on the `/// - >1 inputs →` bullet (the leading `>` was
   interpreted as a blockquote continuation); fixed by rephrasing the multi-input
   bullet without the `>` character.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No missing constraint. The `clippy::doc_lazy_continuation` lint for doc comments
   with `>` is caught by `--all-targets` and worth a standing note, but it's covered
   by the existing `clippy-fmt-clean` constraint. No new decision warranted.

3. **If you did this task again, what would you do differently?**
   — Run `cargo fmt` immediately after each edit rather than discovering fmt failures
   at the gate step. The logic was straightforward (the spec was correctly prescriptive);
   the only rework was formatting. Incremental commits after each gate would catch this
   earlier.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Nothing — this was the smoothest cycle of STAGE-003. The build-prompt
   lessons banked from SPEC-010/011 (commit incrementally, confirm every named
   test exists, `--all-targets` gate, derive Debug) produced a clean first-pass
   build and a no-punch-list verify. The "thumbnail = thin wrapper over resize"
   insight kept it complexity S and avoided a redundant Operation.

2. **Does any template, constraint, or decision need updating?**
   — No. DEC-015 already declared it governs every STAGE-003 fan-out command,
   and thumbnail inherited it for free via the new shared `run_pixel_op` helper —
   exactly as intended. No new DEC, no constraint change.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec. The `run_pixel_op` shared helper is now the reuse seam for the
   remaining STAGE-003 commands: `shrink` (resize + quality encode + strip),
   `convert` (re-encode), `auto-orient` (EXIF orientation → pixels). `shrink`
   and `convert` will extend it with quality-aware / format-targeted encoding;
   they need fresh specs but no new framing.
</content>
</invoke>
