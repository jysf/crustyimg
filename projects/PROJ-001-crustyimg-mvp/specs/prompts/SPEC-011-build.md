# SPEC-011 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. Do not rely on any prior conversation. This prompt is
> deliberately prescriptive — follow it literally. Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-011 ("resize cli command and multi input
fan-out"). You are NOT the architect; the spec file is your source of truth.
This spec is the CLI HALF of a split `resize` feature: SPEC-010 already shipped
the `Resize` Operation + the OperationParams mechanism + registry registration
(recipe-usable, on `main`). SPEC-011 wires the user-facing `resize` command on
top: parse the six mode flags, build the resize op THROUGH THE REGISTRY (the same
path recipes use), run it through the Pipeline, and write outputs — including
SEQUENTIAL multi-input `--out-dir` fan-out (NO rayon — parallel batch is
STAGE-005). ALL changes are confined to `src/cli/mod.rs` + `docs/api-contract.md`
(already edited by the architect — leave it). Do NOT modify any library module
(`src/image/`, `src/operation/`, `src/pipeline/`, `src/sink/`, `src/source/`).
Use ABSOLUTE paths for every file you read or write.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════

1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — conventions: §5 stack, §6 the EXACT commands (the gates below), §11 coding
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
   expected here, DEC-015 is already written by the architect).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-011-resize-cli-command-and-multi-input-fan-out.md
   — THE SPEC. Implement its "## Failing Tests", "## Outputs" (the `ArgGroup` on
   `Commands::Resize`, the `run_resize` handler, `parse_wxh`, `resize_params`,
   `output_format_for`, the new `CliError::PartialBatch`→6 and `CliError::Usage`
   →2 variants + their `code()` arms, the extended `exit_code_mapping_is_total`),
   and "## Acceptance Criteria" exactly. Read "## Implementation Context" and "##
   Notes for the Implementer" in FULL — they carry the op-construction path (build
   THROUGH the registry), the RegistryError→CliError::Usage mapping decision, the
   single-vs-multi fan-out logic, the resolution-error (exit 3) vs partial-batch
   (exit 6) boundary, and the per-input format→Sink construction (DEC-015).
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-015-resize-output-format-and-partial-batch.md
   — the output-format-preservation default (preserve source_format unless
   `--format` or `-o` ext dictates) + the partial-batch exit-6 semantics (any
   per-input failure → exit 6, all-fail included; single-input failures keep their
   natural code). Already written — do NOT re-decide it; implement it.
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-014-operation-params-mechanism.md ,
   .../DEC-012-clap-cli-framework.md ,
   .../DEC-010-source-crate-glob.md ,
   .../DEC-007-error-handling-thiserror-anyhow.md ,
   .../DEC-003-metadata-dual-lane.md
   — DEC-014: build the resize op via OperationParams + the registry (the recipe
   path); WxH-string parsing is the CLI's job (translate to flat width/height
   keys). DEC-012: clap; use an ArgGroup for mode exclusivity (clap owns exit 2);
   pixel core must NOT depend on clap. DEC-010: source::resolve is the source
   seam — each `inputs` arg resolves and flattens; don't re-implement globbing.
   DEC-007: typed errors → exit codes in the ONE `code()` mapping. DEC-003:
   metadata preservation is the STAGE-004 container lane — `resize` DROPS
   container metadata on re-encode; do NOT pull in img-parts/little_exif.
5. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/api-contract.md
   — the `resize` entry (the architect just clarified the format-preservation +
   exit-6 + metadata-dropped behavior) and the Exit Codes table (2 usage, 3 not
   found, 4 unsupported format, 5 write refused, 6 partial batch). Do NOT edit it.
6. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/constraints.yaml
   — ergonomic-defaults (one short command; output keeps source format),
   no-unwrap-on-recoverable-paths (no unwrap/expect/panic! in src/cli/; the
   fan-out catches per-input errors and aggregates), every-public-fn-tested (the
   pure helpers get unit tests; the two new CliError variants are covered by the
   extended exit_code_mapping_is_total), clippy-fmt-clean, test-before-
   implementation, untrusted-input-hardening (the Sink already guards traversal/
   overwrite — reuse it, don't bypass), no-async-runtime (sequential for loop, NO
   rayon).
7. The SHIPPED code you wire against (read the real signatures — do NOT modify
   any of these except `src/cli/mod.rs`):
   src/cli/mod.rs        — `Commands::Resize { inputs, max, exact, percent, fit,
                           fill, cover }` (ALREADY declared, dispatched to
                           NotImplemented("resize")); `run_apply`/`run_view`/
                           `run_info` (STRUCTURAL TEMPLATES); `build_sink` /
                           `resolve_format` helpers; `GlobalArgs`; `CliError` +
                           `code()` + `exit_code_mapping_is_total` (maps 1/2/3/4/5
                           — NO exit-6 path yet; you add it). THIS is the only
                           src file you modify.
   src/operation/mod.rs + registry.rs
                         — `Resize::from_params(&OperationParams) -> Result<Resize,
                           RegistryError>`; `OperationParams::{empty, from_map,
                           get_str, get_u32, get_f32}`; `OperationRegistry::
                           with_builtins().build(name, &params) -> Result<Box<dyn
                           Operation>, RegistryError>`. Build the resize op the
                           SAME way recipes do (registry.build("resize",&params)).
   src/pipeline/mod.rs   — `Pipeline` + push + `run(Image) -> Result<Image,
                           OperationError>`.
   src/source/mod.rs     — `source::resolve(arg, &mut reader) -> Result<Vec<Input>,
                           SourceError>` (one arg → many via glob/dir); `Input::
                           {stem, path}`.
   src/sink/mod.rs       — `Sink::{File, Dir, Stdout}`, `Sink::write`, `SinkInput`,
                           `Overwrite`, `extension_for_format`,
                           `format_from_extension`. NOTE: `Sink::Dir` defaults to
                           PNG when `format: None` — you pass `format: Some(fmt)`
                           per input so it never falls back to PNG (DEC-015).
   src/image/mod.rs      — `Image::{load, from_bytes, source_format}`.
                           `source_format()` → `image::ImageFormat` (drives
                           preservation).
   tests/cli.rs, tests/common/mod.rs
                         — integration conventions: `write_test_png`,
                           `gradient_jpeg`, `stdout_str`/`stderr_str`, tempfile,
                           drive the real binary. (The `stub_command_returns_not_
                           implemented` test currently drives `resize` — you UPDATE
                           it to drive a still-stubbed command, e.g. `thumbnail`.)

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST (before editing ANY file)
═══════════════════════════════════════════════════════════════════════════

Do this BEFORE touching code so nothing ever lands on `main`:

  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-011-resize-cli-command-and-multi-input-fan-out

ALL code, test, and spec edits below happen ON THIS BRANCH. Never commit to
`main` (and never to a `chore/*` branch a background task may have left checked
out — if `git branch --show-current` prints anything other than the spec branch,
STOP and fix it). Confirm `git branch --show-current` prints
`feat/spec-011-resize-cli-command-and-multi-input-fan-out`, NOT `main` and NOT a
chore branch, before committing.

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact) — all in src/cli/mod.rs
═══════════════════════════════════════════════════════════════════════════

A. ArgGroup on Commands::Resize (mode mutual-exclusivity → exit 2).
   Add the clap group attribute on the `Resize` variant (spec "## Outputs" (a)):
     #[command(group = clap::ArgGroup::new("mode")
         .required(true)
         .args(["max", "exact", "percent", "fit", "fill", "cover"]))]
   Zero modes → clap exit 2; two modes → clap exit 2. `resize --help` still exits
   0 (help short-circuits group validation) — the existing
   `each_subcommand_help_parses` test must stay green.

B. Two new CliError variants + code() arms.
   B1. Add to the `CliError` enum (thiserror):
         /// One or more inputs in a multi-input batch failed (others may have
         /// succeeded). A per-failure summary is printed to stderr first.
         #[error("{failed} of {total} inputs failed")]
         PartialBatch { failed: usize, total: usize },
         /// A usage error detected at runtime (malformed WxH; multi-input
         /// without --out-dir). Mirrors clap's exit 2.
         #[error("{0}")]
         Usage(String),
   B2. Extend `code()`: `CliError::PartialBatch { .. } => 6` and
       `CliError::Usage(_) => 2`. (Keep the mapping TOTAL — every variant has an
       arm.)

C. parse_wxh — WxH string parser (pure; spec "## Outputs" (c)).
     fn parse_wxh(s: &str) -> Result<(u32, u32), CliError>
   Split on a single ASCII 'x'/'X'; both parts must be positive integers (> 0).
   Anything malformed (no/extra separator, empty part, non-integer, 0, negative,
   overflow) → `CliError::Usage(<message>)` (so `.code()` == 2). NO panic.

D. resize_params — flags→OperationParams mapper (pure; spec "## Outputs" (d)).
     fn resize_params(
         max: Option<u32>, exact: Option<&str>, percent: Option<f32>,
         fit: Option<&str>, fill: Option<&str>, cover: Option<&str>,
     ) -> Result<OperationParams, CliError>
   Exactly one flag is set (clap's ArgGroup guarantees it). Build a
   `BTreeMap<String, toml::Value>`:
     --max N     → {mode:"max",     width:N}
     --exact WxH → {mode:"exact",   width:W, height:H}   (W,H via parse_wxh)
     --percent P → {mode:"percent", percent:P}           (toml::Value::Float(P as f64))
     --fit WxH   → {mode:"fit",     width:W, height:H}
     --fill WxH  → {mode:"fill",    width:W, height:H}
     --cover WxH → {mode:"cover",   width:W, height:H}
   Dims are `toml::Value::Integer(n as i64)`. Wrap via `OperationParams::from_map`.
   No flag set (shouldn't happen) → `CliError::Usage(...)` (defensive, not panic).
   Do NOT validate dim ranges here — that's the op's job (the registry build).

E. output_format_for — per-input output-format resolution (DEC-015; spec (e)).
     fn output_format_for(
         global: &GlobalArgs, output_path: Option<&Path>,
         source_format: ImageFormat,
     ) -> Result<ImageFormat, CliError>
   Precedence: (1) `--format FMT` via `resolve_format(global.format.as_deref())?`
   (unrecognized → its SinkError, exit 4); (2) else `output_path` extension via
   `crate::sink::format_from_extension(path)`; (3) else PRESERVE `source_format`.

F. run_resize handler — wire the dispatch arm.
   F1. In `dispatch`, replace
         Commands::Resize { .. } => Err(CliError::NotImplemented("resize")),
       with a destructuring arm calling:
         run_resize(inputs, *max, exact.as_deref(), *percent, fit.as_deref(),
                    fill.as_deref(), cover.as_deref(), &cli.global)
   F2. Signature (spec "## Outputs" (b)):
         fn run_resize(inputs: &[String], max: Option<u32>, exact: Option<&str>,
             percent: Option<f32>, fit: Option<&str>, fill: Option<&str>,
             cover: Option<&str>, global: &GlobalArgs) -> Result<(), CliError>
   F3. Flow:
       1. params = resize_params(max, exact, percent, fit, fill, cover)?
       2. op = OperationRegistry::with_builtins().build("resize", &params)
              .map_err(|e| match e {
                  RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
                  RegistryError::Unknown { name } =>
                      CliError::Usage(format!("unknown operation '{name}'")),
              })?;
          (Per spec Notes "param errors": resize param rejections → exit 2 Usage,
          so ALL malformed-resize-input paths are uniformly exit 2.)
          Build a Pipeline with this one op: `let pipeline = Pipeline::new().push(op);`
          (push is BUILDER-style — it consumes self and returns Self, not a
          mutating method).
       3. Resolve EVERY arg in `inputs` via source::resolve, flattening into one
          Vec<Input>. Lock stdin ONCE outside the loop (only a "-" arg consumes
          it). A resolution error (missing path / empty glob / invalid pattern)
          PROPAGATES here (`?`) — this is a hard error (exit 3 / 2), NOT
          partial-batch. If the flattened Vec is empty → CliError::Source(
          SourceError::NotFound(<joined inputs args>)) (exit 3).
       4. Single vs multi by the FLATTENED count:
          - len == 1: resolve the per-input format via output_format_for(global,
            <the -o path if any>, img.source_format()), build the sink (File/
            Stdout/Dir per the -o/-o -/out-dir flags, with format: Some(fmt)),
            load → pipeline.run → write. A failure here is that input's natural
            typed CliError (exit 3/1/4/5) — NOT exit 6.
          - len > 1: REQUIRE global.out_dir.is_some(); else return
            CliError::Usage("multiple inputs require --out-dir".into()) (exit 2).
            Then a SEQUENTIAL `for` loop (NO rayon). Per input: load → run →
            output_format_for(global, None, img.source_format()) → build
            Sink::Dir { dir, template, format: Some(fmt) } (template from
            --name-template or "{stem}.{ext}") → write. Catch each Result; on Err,
            `eprintln!("error: {}: {e}", <input label>)` and bump `failed`. After
            the loop: failed == 0 → Ok(()); failed > 0 → Err(CliError::PartialBatch
            { failed, total: all.len() }) (exit 6, all-fail included).
       5. Diagnostics → STDERR (eprintln!); for -o - the encoded bytes go to
          stdout via the Sink and STDOUT STAYS CLEAN (no eprintln to stdout).

   Per-input format → Sink (DEC-015): pass `format: Some(fmt)` to the Sink
   variant so the Dir sink never falls back to its PNG default and the `{ext}`
   template token derives from the preserved format (confirm in
   src/sink/mod.rs::Sink::write Dir arm — it computes ext from the chosen format).
   Reuse the existing Overwrite::{Forbid, Allow} from `global.yes`, like run_apply.

G. DO NOT modify any library module. Only `src/cli/mod.rs` (code) +
   `tests/cli.rs` (tests). `docs/api-contract.md` is ALREADY edited by the
   architect — do NOT touch it. If you think a Sink/op/source change is needed to
   compile or to preserve format, STOP — it shouldn't be (format is threaded via
   the existing `format: Some(_)` field). Flag it in `## Build Completion` →
   Deviations and add a question to
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml
   rather than editing a library module.

   STANDING NOTE (lesson from SPEC-010): derive `Debug` on any new public type,
   and do NOT `{:?}`-format types that don't impl `Debug` (e.g. `Box<dyn
   Operation>`, `Pipeline`). The two new CliError variants are fields-only on an
   already-`Debug` enum — fine; write any test format strings accordingly.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (make them pass)
═══════════════════════════════════════════════════════════════════════════

Implement EVERY test named in the spec's "## Failing Tests". Native in-memory
fixtures only; integration tests drive the real binary.

In src/cli/mod.rs `#[cfg(test)] mod tests` (use `super::*`):
  - parse_wxh_parses_valid            ("800x600"→(800,600); "1920X1080" ok)
  - parse_wxh_rejects_malformed       ("abc","800x","x600","800","800x600x1",
                                       "0x10","800x0","-1x10","" → Err, code()==2)
  - resize_params_max_minimal         (Some(20),.. → {mode:"max",width:20}; NO height/percent)
  - resize_params_exact_has_both_dims ("33x77" → {mode:"exact",width:33,height:77})
  - resize_params_percent             (Some(50.0) → {mode:"percent",percent:50.0} Float)
  - resize_params_fit_fill_cover      (each "40x40" → right mode + width/height)
  - resize_params_bad_wxh_is_usage    (exact "nope" → Err, code()==2)
  - output_format_for_format_flag_wins  (format=Some("png"),path=Some(a.jpg),src=Jpeg → Png)
  - output_format_for_path_ext          (format=None,path=Some(a.png),src=Jpeg → Png)
  - output_format_for_preserves_source  (format=None,path=None,src=Jpeg → Jpeg)
  - exit_code_mapping_is_total        EXTEND the existing test: add
                                       PartialBatch{failed:1,total:3}.code()==6 and
                                       Usage("bad".into()).code()==2; keep ALL
                                       prior assertions. (Do not duplicate the test.)

In tests/cli.rs (reuse write_test_png; add a JPEG-on-disk helper using the
`image` crate's JPEG encoder, or inline gradient-JPEG bytes; drive the real binary):
  - resize_max_single_input_writes_scaled       (100x50 png, --max 20 -o out → exit0, decoded 20x10)
  - resize_exact_single_input_exact_dims         (100x50, --exact 33x77 -o → 33x77)
  - resize_multi_input_fan_out_preserves_format  (a.png + b.jpg in a dir, --max 20
                                                  --out-dir D → exit0; D/a.png is PNG 20x10,
                                                  D/b.jpg is JPEG 20x10 — assert format preserved)
  - resize_format_override_changes_format        (.jpg in, --max 20 --format png -o out.png →
                                                  exit0, output decodes as PNG)
  - resize_no_mode_is_usage_error                (no mode flag → exit2; no output file)
  - resize_two_modes_is_usage_error              (--max 20 --exact 10x10 → exit2)
  - resize_bad_wxh_is_usage_error                (--exact abc → exit2; --exact 800x → exit2)
  - resize_missing_input_exits_3                 (missing.png --max 20 → exit3)
  - resize_partial_batch_exits_6                 (valid png + garbage-bytes ".png" → --out-dir →
                                                  exit6; valid output written; stderr names the fail)
  - resize_stdout_keeps_stdout_clean             (--max 20 -o - → exit0; stdout decodes 20x10)
  - resize_multi_without_out_dir_is_usage_error  (two pngs, no --out-dir → exit2; stderr mentions out-dir)
  - UPDATE stub_command_returns_not_implemented  (drive `thumbnail <in> --size 64 -o <out>`
                                                  instead of resize; keep exit-1 +
                                                  "not yet implemented" assertions)

The existing tests/cli.rs suite (help_lists_all_subcommands,
each_subcommand_help_parses, version_prints_semver, apply_*, view_*, info_*) and
all unit tests MUST stay green. Run the FULL `cargo test`.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════

- Any LIBRARY change — src/image/, src/operation/, src/pipeline/, src/sink/,
  src/source/ are READ-ONLY. The resize op already exists (SPEC-010). Format
  preservation is achieved by passing format: Some(_) to the EXISTING Sink.
- rayon / ANY parallelism / progress bars — STAGE-005 (DEC-006). Sequential loop.
- Metadata preservation (default-preserve / drop-GPS carry-over) — STAGE-004
  container lane. resize DROPS container metadata. Do NOT add img-parts/little_exif.
- thumbnail / shrink / convert / auto-orient — later STAGE-003 specs.
- Quality-aware encode (-q/--quality) — resize re-encodes at the encoder default;
  --quality is the shrink/convert story. --format IS honored. If -q must be
  threaded into the Sink, that's a Sink change → out of scope; defer.
- WebP / AVIF output — DEC-004.
- A new DEC (DEC-015 is already written) or a new top-level crate (clap exists).
If you think a new RUNTIME crate, a library-module change, or a new DEC is needed,
STOP and add a question to
/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml
instead of inventing it.

═══════════════════════════════════════════════════════════════════════════
THE GATES (run from the repo root; ALL must pass before the PR)
═══════════════════════════════════════════════════════════════════════════

  cargo build
  cargo test
  cargo clippy -- -D warnings
  cargo fmt --check                              # `cargo fmt` to fix, then re-check

NOTE: there is NO `--features display` gate for SPEC-011 — resize does not touch
viuer. Run exactly these four.

═══════════════════════════════════════════════════════════════════════════
WHEN DONE
═══════════════════════════════════════════════════════════════════════════

1. Fill in ONLY the spec's `## Build Completion` section (branch, PR, criteria
   met, deviations — INCLUDING which RegistryError→CliError mapping you used
   (the spec recommends CliError::Usage/exit 2) and the §G library-untouched
   confirmation, follow-ups, and the 3-question build reflection). Do NOT edit any
   other part of the spec body.
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
   `feat(cli): resize command + multi-input out-dir fan-out (SPEC-011)`
   — a single commit covering src/cli/mod.rs + tests/cli.rs + the spec is fine;
   end EACH commit message with:
       Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
   (Confirm `git branch --show-current` prints
   `feat/spec-011-resize-cli-command-and-multi-input-fan-out`, NOT `main` and NOT
   a chore branch, before committing.)
5. Mark build `[x]` in the timeline
   (/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-011-resize-cli-command-and-multi-input-fan-out-timeline.md).
   ACCURATE BOOKKEEPING: when you mark build `[x]`, write ONLY what is true at
   build time — say "PR #N opened" (with the real number). Do NOT write "merged",
   do NOT claim the PR is approved, and do NOT assert any post-merge fact. Verify
   and ship record those later.
6. Push the branch and open a PR on the `jysf/crustyimg` remote per AGENTS.md §13
   (one spec per branch / per PR):
   - PR title carries the spec id, e.g.
     `feat(cli): resize command and multi-input fan-out (SPEC-011)`.
   - PR body uses the §13 template — Summary; Spec metadata PROJ-001/STAGE-003/
     SPEC-011; Decisions referenced [DEC-015 (output-format-preservation +
     partial-batch exit-6, first implemented here), DEC-014 (op built via the
     registry/params path), DEC-012 (clap ArgGroup for mode exclusivity), DEC-010
     (source::resolve fan-out), DEC-007 (typed errors → exit codes; new
     PartialBatch→6, Usage→2), DEC-003 (metadata is the container lane — resize
     drops it)]; Constraints checked with one-line evidence each
     (ergonomic-defaults [format preserved], no-unwrap-on-recoverable-paths,
     every-public-fn-tested, clippy-fmt-clean, test-before-implementation,
     untrusted-input-hardening [Sink guards reused], no-async-runtime [sequential
     loop]); New decisions: list "DEC-015 — resize output format + partial-batch
     (emitted during design; first implemented here)".
   - End the PR body with the Claude Code generated-with footer.

Remember: build edits to the spec are LIMITED to `## Build Completion` (plus the
front-matter cost session + task.cycle). Verify/ship bookkeeping lands on main
later, not on this branch.
```
