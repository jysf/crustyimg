# SPEC-012 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. Do not rely on any prior conversation. This prompt is
> deliberately prescriptive — follow it literally. Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-012 ("thumbnail command bounded resize and
square crop"). You are NOT the architect; the spec file is your source of truth.
`thumbnail` is a CONVENIENCE command over the already-shipped `resize`: it needs
NO new Operation. `--square --size N` ≡ `resize --fill NxN` (cover + center-crop
to exactly N×N); plain `--size N` ≡ `resize --max N` (bound the long edge to N,
aspect preserved, no upscale); `--size` defaults to 256. So `run_thumbnail` maps
`(size, square)` → a `Resize` OperationParams, builds the op THROUGH THE REGISTRY
(the same path `run_resize` and recipes use), and runs the SAME multi-input
fan-out SPEC-011 already wrote — which you EXTRACT into a shared `run_pixel_op`
helper that BOTH `run_resize` and `run_thumbnail` call. ALL changes are confined
to `src/cli/mod.rs` + `tests/cli.rs` (and a tiny `docs/api-contract.md` edit
ALREADY made by the architect — leave it). Do NOT modify any library module
(`src/image/`, `src/operation/`, `src/pipeline/`, `src/sink/`, `src/source/`).
Use ABSOLUTE paths for every file you read or write.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════

1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — conventions: §5 stack, §6 the EXACT commands (the gates below — note the
   clippy gate is now `cargo clippy --all-targets -- -D warnings`), §11 coding
   conventions (typed errors; NO unwrap/expect/panic! on recoverable paths;
   diagnostics → STDERR so machine/`-o -` output stays clean on STDOUT; thin
   main; the pixel core must NOT depend on clap — all CLI logic stays in
   `src/cli/`), §12 testing (unit in `#[cfg(test)]`, integration under tests/,
   NATIVE in-memory fixtures via the `image` crate — NO ImageMagick, NO committed
   binary fixtures; integration tests drive the REAL binary via
   `env!("CARGO_BIN_EXE_crustyimg")` + std::process::Command), §13 git/PR (branch
   naming, conventional commits + Co-Authored-By trailer, PR body template), §15
   build-cycle rules (spec edits LIMITED to `## Build Completion`; append a build
   cost session entry; create DEC-* only for NON-trivial NEW decisions — NONE
   expected here; the spec states why no DEC-016 is needed).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-012-thumbnail-command-bounded-resize-and-square-crop.md
   — THE SPEC. Implement its "## Failing Tests", "## Outputs" (the `run_thumbnail`
   handler, the `thumbnail_params` mapper + `DEFAULT_THUMBNAIL_SIZE`, the shared
   `run_pixel_op` helper extracted from `run_resize`, and the `run_resize` refactor
   to call it), and "## Acceptance Criteria" exactly. Read "## Implementation
   Context" and "## Notes for the Implementer" in FULL — they carry the op-build
   path (build a RESIZE op THROUGH the registry), the square/non-square mapping,
   the default-size choice, the DRY refactor instructions, and the inherited
   DEC-015 format/exit-6 behavior.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-015-resize-output-format-and-partial-batch.md
   — the output-format-preservation default (preserve source_format unless
   `--format` or `-o` ext dictates) + the partial-batch exit-6 semantics. It
   EXPLICITLY governs every later STAGE-003 fan-out command — thumbnail included.
   Already written — do NOT re-decide it; thumbnail inherits it via `run_pixel_op`.
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-014-operation-params-mechanism.md ,
   .../DEC-012-clap-cli-framework.md ,
   .../DEC-008-resize-backend-fast-image-resize.md ,
   .../DEC-010-source-crate-glob.md ,
   .../DEC-007-error-handling-thiserror-anyhow.md ,
   .../DEC-003-metadata-dual-lane.md
   — DEC-014: build the resize op via OperationParams + the registry (the recipe
   path); thumbnail has NO own operation. DEC-012: clap; `Commands::Thumbnail`
   already exists; pixel core must NOT depend on clap. DEC-008: the resize
   backend + `fill`/`max` modes are internal to the shipped op (SPEC-010) — the
   CLI just runs it. DEC-010: source::resolve is the source seam. DEC-007: typed
   errors → exit codes in the ONE `code()` mapping (no new variant). DEC-003:
   metadata preservation is the STAGE-004 container lane — `thumbnail` DROPS
   container metadata on re-encode; do NOT pull in img-parts/little_exif.
5. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/api-contract.md
   — the `thumbnail` entry (the architect just pinned the default size 256 +
   `--square` = cover+center-crop + format/exit-6 behavior) and the Exit Codes
   table (2 usage, 3 not found, 4 unsupported format, 5 write refused, 6 partial
   batch). Do NOT edit it.
6. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/constraints.yaml
   — ergonomic-defaults (one short command; default --size 256; output keeps
   source format), no-unwrap-on-recoverable-paths (no unwrap/expect/panic! in
   src/cli/; the fan-out catches per-input errors and aggregates),
   every-public-fn-tested (the `thumbnail_params` mapper gets unit tests),
   clippy-fmt-clean (now `--all-targets`), test-before-implementation,
   untrusted-input-hardening (--size 0 → typed op error → exit 2; Sink guards
   reused via run_pixel_op), no-async-runtime (sequential for loop, NO rayon).
7. The SHIPPED code you wire against (read the real signatures — do NOT modify
   any of these except `src/cli/mod.rs`):
   src/cli/mod.rs        — `Commands::Thumbnail { inputs, size, square }` (ALREADY
                           declared, dispatched to NotImplemented("thumbnail")).
                           READ `run_resize` IN FULL — it is BOTH the structural
                           template AND the refactor source: its op-build →
                           `Pipeline::new().push(op)` → `source::resolve` flatten →
                           single-vs-multi → per-input `output_format_for` → Sink →
                           partial-batch exit-6 loop is EXACTLY what thumbnail
                           needs. `resize_params`, `output_format_for`,
                           `ResizeModes`, `GlobalArgs`, `Overwrite`, `CliError` +
                           `code()` (PartialBatch→6, Usage→2 ALREADY exist) +
                           `exit_code_mapping_is_total`. THIS is the only src file
                           you modify.
   src/operation/mod.rs + registry.rs
                         — `Resize::from_params`; `OperationParams::{empty,
                           from_map, get_str, get_u32, get_f32}`;
                           `OperationRegistry::with_builtins().build("resize",
                           &params)`. There is NO "thumbnail" registry key — you
                           build a "resize" op with `mode:"fill"` or `mode:"max"`.
   src/pipeline/mod.rs   — `Pipeline::new()` + `push(op) -> Self` (BUILDER-style,
                           consumes self) + `run(&self, Image) -> Result<Image,
                           OperationError>`.
   src/source/mod.rs     — `source::resolve(arg, &mut reader) -> Result<Vec<Input>,
                           SourceError>`; `Input::{stem, path}`.
   src/sink/mod.rs       — `Sink::{File, Dir, Stdout}`, `Sink::write`, `SinkInput`,
                           `Overwrite`, `extension_for_format`,
                           `format_from_extension`. (Format threaded via
                           `format: Some(fmt)` per input — already how run_resize
                           does it; you don't touch the Sink.)
   src/image/mod.rs      — `Image::{load, from_bytes, source_format}`.
   tests/cli.rs          — integration conventions: `BIN`, `write_test_png`,
                           `write_test_jpeg` (added in SPEC-011), `stdout_str`/
                           `stderr_str`, tempfile; the `resize_*` tests are your
                           template. The `stub_command_returns_not_implemented`
                           test currently drives `thumbnail` — you REPOINT it to a
                           still-stubbed command (`shrink`).

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST (before editing ANY file)
═══════════════════════════════════════════════════════════════════════════

Do this BEFORE touching code so nothing ever lands on `main`:

  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-012-thumbnail-command-bounded-resize-and-square-crop

ALL code, test, and spec edits below happen ON THIS BRANCH. Never commit to
`main` (and never to a `chore/*` branch a background task may have left checked
out — if `git branch --show-current` prints anything other than the spec branch,
STOP and fix it). Confirm `git branch --show-current` prints
`feat/spec-012-thumbnail-command-bounded-resize-and-square-crop`, NOT `main` and
NOT a chore branch, before committing.

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact) — all in src/cli/mod.rs
═══════════════════════════════════════════════════════════════════════════

A. The DRY refactor — extract `run_pixel_op` from `run_resize`.
   Add a private helper holding the fan-out tail of `run_resize` (everything
   AFTER `let pipeline = Pipeline::new().push(op);`):
     /// Run a built single-op `Pipeline` over one-or-many resolved inputs and
     /// write the outputs — the shared CLI fan-out for pixel commands (DEC-015).
     fn run_pixel_op(
         pipeline: Pipeline,
         inputs: &[String],
         global: &GlobalArgs,
     ) -> Result<(), CliError>
   Move into it, BYTE-FOR-BYTE (behavior UNCHANGED): the `source::resolve`
   flatten into `let mut all: Vec<Input>`, the `all.is_empty()` → `NotFound`
   (exit 3), the `Overwrite` computation from `global.yes`, the `all.len() == 1`
   single-sink branch (per-input format via `output_format_for(global,
   output_path, img.source_format())`, build File/Stdout/Dir sink with
   `format: Some(fmt)`, load → run → write; failure keeps its natural code), and
   the `else` multi-input branch (REQUIRE `--out-dir` else `CliError::Usage` exit
   2; SEQUENTIAL for loop; per-input `output_format_for(global, None, …)` +
   `Sink::Dir { format: Some(fmt) }`; catch per-input Err → `eprintln!("error:
   {label}: {e}")` + `failed += 1`; after loop `failed > 0` → `CliError::PartialBatch
   { failed, total }`, exit 6). `pipeline.run` takes `&self`, so the one Pipeline
   value serves every input.

B. Refactor `run_resize` to call `run_pixel_op`.
   `run_resize` keeps its signature (`inputs: &[String]`, `modes: &ResizeModes<'_>`,
   `global: &GlobalArgs`) and its op-construction (`resize_params(...)` + the
   `OperationRegistry::with_builtins().build("resize", &params)` + the
   `RegistryError` → `CliError::Usage` map_err + `let pipeline =
   Pipeline::new().push(op);`). Its body then ENDS with:
     run_pixel_op(pipeline, inputs, global)
   Nothing else changes in run_resize. The existing `resize_*` integration tests
   and the `output_format_for_*`/`resize_params_*` unit tests MUST stay green —
   they prove the refactor is behavior-preserving.

C. thumbnail_params — (size, square)→OperationParams mapper (pure; INFALLIBLE).
     const DEFAULT_THUMBNAIL_SIZE: u32 = 256;
     fn thumbnail_params(size: Option<u32>, square: bool) -> OperationParams
   `let n = size.unwrap_or(DEFAULT_THUMBNAIL_SIZE);` then build a
   `BTreeMap<String, toml::Value>`:
     square == true  → {mode:"fill", width:n, height:n}
     square == false → {mode:"max",  width:n}
   Dims are `toml::Value::Integer(n as i64)`. Wrap via `OperationParams::from_map`.
   Return the `OperationParams` directly (NOT a Result) — the mapping is total;
   the only invalid value (`--size 0` → width 0) is rejected at the op-build step
   in run_thumbnail, NOT here (keeps dim validity in one place, DEC-014).

D. run_thumbnail handler — wire the dispatch arm.
   D1. In `dispatch`, replace
         Commands::Thumbnail { .. } => Err(CliError::NotImplemented("thumbnail")),
       with:
         Commands::Thumbnail { inputs, size, square } =>
             run_thumbnail(inputs, *size, *square, &cli.global),
   D2. Signature + flow:
         fn run_thumbnail(
             inputs: &[String], size: Option<u32>, square: bool,
             global: &GlobalArgs,
         ) -> Result<(), CliError> {
             let params = thumbnail_params(size, square);
             let op = OperationRegistry::with_builtins()
                 .build("resize", &params)
                 .map_err(|e| match e {
                     RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
                     RegistryError::Unknown { name } =>
                         CliError::Usage(format!("unknown operation '{name}'")),
                 })?;
             let pipeline = Pipeline::new().push(op);
             run_pixel_op(pipeline, inputs, global)
         }
       (The map_err handles `--size 0`: the op rejects width 0 → InvalidParams →
       CliError::Usage → exit 2. NO unwrap.)

E. DO NOT modify any library module. Only `src/cli/mod.rs` (code) +
   `tests/cli.rs` (tests). `docs/api-contract.md` is ALREADY edited by the
   architect — do NOT touch it. NO new Operation/registry key (thumbnail builds a
   "resize" op). NO new CliError variant or code() change (PartialBatch→6,
   Usage→2 already exist; do NOT touch `exit_code_mapping_is_total`). If you think
   a Sink/op/source change is needed to compile or to preserve format, STOP — it
   shouldn't be. Flag it in `## Build Completion` → Deviations and add a question
   to
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml
   rather than editing a library module.

   STANDING NOTE (lesson from SPEC-010): derive `Debug` on any new public type,
   and do NOT `{:?}`-format types that don't impl `Debug` (e.g. `Box<dyn
   Operation>`, `Pipeline`). You should NOT need any new type here; if you add a
   helper struct, derive `Debug`.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (make them pass)
═══════════════════════════════════════════════════════════════════════════

Implement EVERY test named in the spec's "## Failing Tests" — do not stop at a
green test COUNT; confirm each NAMED test exists and runs (SPEC-011 lesson: the
integration suite was missed despite green gates). Native in-memory fixtures only;
integration tests drive the real binary.

In src/cli/mod.rs `#[cfg(test)] mod tests` (use `super::*`; mirror the existing
`resize_params_*` tests — assert via `p.get_str("mode")`, `p.get_u32("width")`,
`p.get_u32("height")`):
  - thumbnail_params_max_default     (None,false → {mode:"max",width:256}; NO height)
  - thumbnail_params_max_sized       (Some(64),false → {mode:"max",width:64}; NO height)
  - thumbnail_params_square_default  (None,true → {mode:"fill",width:256,height:256})
  - thumbnail_params_square_sized    (Some(64),true → {mode:"fill",width:64,height:64})

In tests/cli.rs (reuse write_test_png + write_test_jpeg; drive the real binary;
the `resize_*` tests are the template):
  - thumbnail_default_size_bounds_long_edge      (1000x500 png, no --size, -o out →
                                                  exit0, decoded 256x128)
  - thumbnail_size_bounds_long_edge              (100x50, --size 64 -o out → 64x32)
  - thumbnail_square_is_exact_square             (100x50, --size 64 --square -o out →
                                                  exactly 64x64)
  - thumbnail_does_not_upscale                   (40x30, --size 64 -o out → 40x30)
  - thumbnail_multi_input_fan_out_preserves_format (a.png+b.jpg in a dir, --size 64
                                                  --out-dir D → exit0; D/a.png PNG 64x32,
                                                  D/b.jpg JPEG 64x32 — format preserved)
  - thumbnail_missing_input_exits_3              (missing.png --size 64 -o out → exit3)
  - thumbnail_multi_without_out_dir_is_usage_error (two pngs, no --out-dir → exit2;
                                                  stderr mentions out-dir)
  - thumbnail_stdout_keeps_stdout_clean          (--size 64 -o - → exit0; stdout decodes)
  - thumbnail_partial_batch_exits_6              (valid png + garbage-bytes ".png" →
                                                  --out-dir → exit6; valid output written;
                                                  stderr names the fail)
  - thumbnail_size_zero_is_usage_error           (100x50, --size 0 -o out → exit2; no output)
  - REPOINT stub_command_returns_not_implemented (drive `shrink <in> --max 64 -o <out>`
                                                  instead of thumbnail; keep exit-1 +
                                                  "not yet implemented" assertions)

The existing tests/cli.rs suite (help_lists_all_subcommands,
each_subcommand_help_parses, version_prints_semver, apply_*, view_*, info_*, and
ALL resize_*) and all unit tests (incl. resize_params_*, output_format_for_*,
parse_wxh_*, exit_code_mapping_is_total — they guard the run_pixel_op refactor)
MUST stay green. Run the FULL `cargo test`.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════

- Any LIBRARY change — src/image/, src/operation/, src/pipeline/, src/sink/,
  src/source/ are READ-ONLY. NO new `Thumbnail` Operation (it maps onto the
  shipped `Resize`'s `fill`/`max`). Format preservation is achieved by passing
  format: Some(_) to the EXISTING Sink (already in the relocated run_resize body).
- A new CliError variant or code() / exit_code_mapping_is_total change — none
  needed (PartialBatch→6, Usage→2 already exist).
- rayon / ANY parallelism / progress bars — STAGE-005 (DEC-006). Sequential loop.
- Metadata preservation (default-preserve / drop-GPS carry-over) — STAGE-004
  container lane. thumbnail DROPS container metadata. Do NOT add img-parts/little_exif.
- shrink / convert / auto-orient — later STAGE-003 specs (they will also reuse
  run_pixel_op).
- Quality-aware encode (-q/--quality) — thumbnail re-encodes at the encoder
  default; --quality is the shrink/convert story.
- WebP / AVIF output — DEC-004.
- A new DEC (the spec explains why no DEC-016) or a new top-level crate (clap
  exists). If you think a new RUNTIME crate, a library-module change, or a new DEC
  is needed, STOP and add a question to
  /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml
  instead of inventing it.

═══════════════════════════════════════════════════════════════════════════
THE GATES (run from the repo root; ALL must pass before the PR)
═══════════════════════════════════════════════════════════════════════════

  cargo build
  cargo test
  cargo clippy --all-targets -- -D warnings      # NOTE: --all-targets (CI changed, PR #10)
  cargo fmt --check                              # `cargo fmt` to fix, then re-check

NOTE: there is NO `--features display` gate for SPEC-012 — thumbnail does not
touch viuer. Run exactly these four. The clippy gate MUST be `--all-targets`
(plain `cargo clippy -- -D warnings` is NOT sufficient; CI gates --all-targets).

RUN THE GATES + COMMIT INCREMENTALLY — do NOT leave all work uncommitted to the
end. After each coherent chunk (e.g. the run_pixel_op refactor + run_resize still
green; then the thumbnail handler + unit tests; then the integration tests),
run the gates and make a commit on the branch. If the session drops, a green
committed checkpoint survives. (SPEC-011 lesson: a dropped build left work
uncommitted and the integration suite was missed.)

═══════════════════════════════════════════════════════════════════════════
WHEN DONE
═══════════════════════════════════════════════════════════════════════════

0. CONFIRM EVERY test named in the spec's `## Failing Tests` actually EXISTS and
   RUNS (grep the test names in src/cli/mod.rs + tests/cli.rs; run `cargo test
   thumbnail` and eyeball the list). A passing test count alone does NOT prove the
   prescribed tests were written (SPEC-011 lesson).
1. Fill in ONLY the spec's `## Build Completion` section (branch, PR, criteria
   met, deviations — INCLUDING the §E library-untouched confirmation and the
   run_pixel_op refactor note, follow-ups, and the 3-question build reflection).
   Do NOT edit any other part of the spec body.
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
   `feat(cli): thumbnail command (bounded resize + --square crop) (SPEC-012)`
   — end EACH commit message with:
       Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
   (Confirm `git branch --show-current` prints
   `feat/spec-012-thumbnail-command-bounded-resize-and-square-crop`, NOT `main`
   and NOT a chore branch, before committing.)
5. Mark build `[x]` in the timeline
   (/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-012-thumbnail-command-bounded-resize-and-square-crop-timeline.md).
   ACCURATE BOOKKEEPING: when you mark build `[x]`, write ONLY what is true at
   build time — say "PR #N opened" (with the real number). Do NOT write "merged",
   do NOT claim the PR is approved, and do NOT assert any post-merge fact. Verify
   and ship record those later.
6. Push the branch and open a PR on the `jysf/crustyimg` remote per AGENTS.md §13
   (one spec per branch / per PR):
   - PR title carries the spec id, e.g.
     `feat(cli): thumbnail command — bounded resize + square crop (SPEC-012)`.
   - PR body uses the §13 template — Summary; Spec metadata PROJ-001/STAGE-003/
     SPEC-012; Decisions referenced [DEC-015 (format-preservation + partial-batch
     exit-6, inherited via the shared run_pixel_op), DEC-014 (resize op built via
     the registry/params path), DEC-012 (clap; Thumbnail variant already declared),
     DEC-008 (resize backend + fill/max modes internal to the shipped op),
     DEC-010 (source::resolve fan-out), DEC-007 (typed errors → exit codes; reused
     PartialBatch→6, Usage→2), DEC-003 (metadata is the container lane — thumbnail
     drops it)]; Constraints checked with one-line evidence each
     (ergonomic-defaults [default --size 256, format preserved],
     no-unwrap-on-recoverable-paths, every-public-fn-tested [thumbnail_params unit
     tests], clippy-fmt-clean [--all-targets], test-before-implementation,
     untrusted-input-hardening [--size 0 → typed Usage exit 2; Sink guards reused],
     no-async-runtime [sequential loop]); New decisions: "No new DEC — thumbnail
     reuses DEC-015/014/012/008/010/007/003 (see spec 'Why no new DEC')".
   - End the PR body with the Claude Code generated-with footer.

Remember: build edits to the spec are LIMITED to `## Build Completion` (plus the
front-matter cost session + task.cycle). Verify/ship bookkeeping lands on main
later, not on this branch.
```
</content>
