# SPEC-014 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. This prompt is deliberately prescriptive — follow it literally.
> Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-014 ("convert command — re-encode across core
formats"). You are NOT the architect; the spec is your source of truth. This is a
CLI-ONLY command: it adds `run_convert` and threads a `forced_format` option
through the shared `run_pixel_op` fan-out. There is NO library/Sink change, NO new
Operation, NO new dependency, NO new DEC. Use ABSOLUTE paths for every file.

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST (before editing ANY file)
═══════════════════════════════════════════════════════════════════════════
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-014-convert-command-re-encode-across-core-formats
Confirm `git branch --show-current` shows that branch — NOT `main`, NOT any
`chore/*` branch — before you commit anything. ALL edits happen on this branch.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════
1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — §6 the EXACT gate commands (the clippy gate is
   `cargo clippy --all-targets -- -D warnings`), §11 conventions (typed errors;
   NO unwrap/expect/panic on recoverable paths; DIAGNOSTICS TO STDERR NEVER
   STDOUT; one image library only), §12 testing (native in-memory fixtures via
   the `image` crate), §13 git/PR, §15 build-cycle rules (spec edits limited to
   ## Build Completion; append a build cost session; create DEC only for NEW
   non-trivial decisions — NONE expected for convert).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-014-convert-command-re-encode-across-core-formats.md
   — THE SPEC. Implement its "## Outputs", "## Failing Tests" and "## Notes for
   the Implementer" EXACTLY. The `run_convert` sketch and the `run_pixel_op`
   forced-format threading are spelled out there.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-004-codec-policy-pure-rust-default.md
   — core format set; an unsupported/unbuilt codec (AVIF; WebP fast-follow) →
   exit 4. Already on main — do NOT create a new DEC.
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-015-resize-output-format-and-partial-batch.md
   — format precedence (`--format` > `-o` ext > source) + partial-batch exit 6.
5. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-016-encode-quality-policy.md
   — `-q` → JPEG quality, ignored for lossless; convert threads global.quality
   with NO forced default (only shrink defaults quality).
6. The SHIPPED code you change/reuse (read the real signatures):
   src/cli/mod.rs   — `run_pixel_op` (you ADD a `forced_format:
                      Option<::image::ImageFormat>` param at the END), its single-
                      and multi-input arms (use forced_format if Some, else
                      `output_format_for`), `output_format_for` (LEAVE UNCHANGED,
                      including its 3 unit tests), `resolve_format` (reuse to
                      resolve the target → exit 4), `run_resize`/`run_thumbnail`/
                      `run_shrink` (callers — pass `None`), `run_apply` (does NOT
                      call run_pixel_op — leave it), `Commands::Convert { inputs,
                      format }`, the dispatch arm, `CliError`/`code()`/
                      `exit_code_mapping_is_total` (DO NOT change these).
   src/pipeline/mod.rs — `Pipeline::new()` (empty pipeline = no-op re-encode;
                      `run` folds zero ops and returns pixels unchanged).
   src/sink/mod.rs  — `resolve_format` calls `format_from_extension`;
                      `extension_for_format` gives `{ext}` for --out-dir names;
                      `encode_to_bytes(img, format, quality)` already quality-
                      aware. SinkError::UnsupportedExtension/UnknownFormat → exit
                      4. NO change in this file.
   tests/cli.rs + tests/common/mod.rs — conventions + `solid_png`/`gradient_jpeg`
                      fixtures; the `stub_command_returns_not_implemented` test
                      (currently `convert` — REPOINT to `auto-orient`).

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact — follow the spec's ## Outputs)
═══════════════════════════════════════════════════════════════════════════
A. src/cli/mod.rs — thread a forced output format through the fan-out.
   A1. `run_pixel_op(pipeline, inputs, global, quality,
        forced_format: Option<::image::ImageFormat>)` — add the param at the END.
        In BOTH the single-input and multi-input arms, replace the
        `output_format_for(...)` call with:
            let fmt = match forced_format {
                Some(f) => f,
                None => output_format_for(global, <output_path | None>, img.source_format())?,
            };
        (single arm passes the `-o` output_path as today; multi arm passes None as
        today). Do NOT change output_format_for itself or its unit tests.
   A2. `run_resize`, `run_thumbnail`, `run_shrink`: add `None` as the new final
        arg to their `run_pixel_op(...)` calls. No other change.
   A3. NEW `fn run_convert(inputs: &[String], format: &str, global: &GlobalArgs)
        -> Result<(), CliError>`:
            // Resolve the REQUIRED target format ONCE, up front → exit 4 for an
            // unsupported/unbuilt codec (DEC-004), BEFORE any input is loaded.
            let fmt = resolve_format(Some(format))?
                .ok_or_else(|| CliError::Usage("convert requires a target --format".into()))?;
            let pipeline = Pipeline::new();          // pure re-encode (no op)
            run_pixel_op(pipeline, inputs, global, global.quality, Some(fmt))
        Read the target from the `format: &str` arg — NOT `global.format` (the
        convert-local `--format` shadows the global one, so `global.format` is
        None inside convert).
   A4. Dispatch: replace
        `Commands::Convert { .. } => Err(CliError::NotImplemented("convert"))`
        with
        `Commands::Convert { inputs, format } => run_convert(inputs, format, &cli.global)`.
   A5. DO NOT add a CliError variant; DO NOT touch code()/exit_code_mapping_is_total;
        DO NOT alter the `Commands::Convert` clap variant (its required `--format`
        is intentional); DO NOT change output_format_for or src/sink/*.

GREP to be exhaustive: `grep -n "run_pixel_op(" src/cli/mod.rs` — update EVERY
call site (run_resize, run_thumbnail, run_shrink get `None`; run_convert gets
`Some(fmt)`).

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (make them pass) — per the spec's ## Failing Tests
═══════════════════════════════════════════════════════════════════════════
INTEGRATION (tests/cli.rs — reuse common::solid_png / common::gradient_jpeg;
detect format via magic bytes: PNG `\x89PNG\r\n\x1a\n`, JPEG `\xFF\xD8`, GIF
`GIF8`; decode with image::load_from_memory / image::open):
  - convert_png_to_jpeg_changes_format           (png → --format jpg -o out.jpg → JPEG, dims kept)
  - convert_jpeg_to_png_changes_format           (jpg → --format png -o out.png → PNG)
  - convert_format_overrides_output_extension    (png --format gif -o out.png → output is GIF)
  - convert_unbuilt_codec_exits_4                 (--format avif → exit 4; --format webp → exit 4)
  - convert_unbuilt_codec_multi_input_exits_4_not_6  (two pngs --format avif --out-dir D → exit == Some(4), NOT 6)
  - convert_multi_input_fan_out                  (a.png b.png --format jpg --out-dir D → D/a.jpg, D/b.jpg both JPEG)
  - convert_quality_lower_is_smaller             (--format jpg -q 20 smaller than -q 90, same dims; USE gradient_jpeg/multi-color source, not flat solid)
  - convert_missing_input_exits_3                (missing input --format png → exit 3)
  - convert_multi_without_out_dir_is_usage_error (two inputs, no --out-dir → exit 2; stderr mentions out-dir)
  - convert_requires_format_flag                 (no --format → exit 2; stderr mentions --format)
  - convert_stdout_keeps_stdout_clean            (--format jpg -o - → stdout decodes as JPEG, stderr empty)
  - REPOINT stub_command_returns_not_implemented from `convert <png> --format png`
    → `auto-orient <png>` (still a stub); keep exit-1 + "not yet implemented" asserts.

The existing resize_*/thumbnail_*/shrink_* integration tests + the sink/unit
tests MUST stay green (run the FULL suite). They pass `None` for forced_format so
output is unchanged. NO new unit test is required (run_convert is private and
fully covered by the integration tests; encode + output_format_for are already
tested).

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════
- `auto-orient` (its own spec, SPEC-015 — it stays a stub; you only REPOINT the
  stub test to it). - Selective metadata preserve / `--keep-gps` / container lane
  (STAGE-004) — write NO metadata code (the re-encode already drops it).
- rayon/parallel (STAGE-005). - WebP/AVIF encoders or features (they exit 4 — do
  NOT wire them). - Any new dependency, Operation, CliError variant, Sink/
  encode_to_bytes change, output_format_for change, or DEC. If you think one is
  needed, STOP and add a question to guidance/questions.yaml.

═══════════════════════════════════════════════════════════════════════════
THE GATES (run from repo root; ALL must pass before the PR)
═══════════════════════════════════════════════════════════════════════════
  cargo build
  cargo test
  cargo clippy --all-targets -- -D warnings     # --all-targets is the CI gate
  cargo fmt --check                              # run `cargo fmt` first to fix

RUN GATES AND COMMIT INCREMENTALLY — commit once the cli compiles and clippy/fmt
are clean, then again after the tests are green. Do NOT leave all work
uncommitted to the end; a green committed checkpoint must survive an interruption.
(Hard lesson from SPEC-011: a dropped build left work uncommitted and the
prescribed tests unwritten.)

BEFORE YOU FINISH: re-read the spec's ## Failing Tests and CONFIRM EACH NAMED
TEST EXISTS in the code and runs — list them and check each off. A passing test
COUNT does not prove the prescribed tests were written. In particular confirm
`convert_unbuilt_codec_multi_input_exits_4_not_6` asserts exit code == Some(4)
(the up-front-resolution correctness point).

Also: do not `{:?}`-format a type that doesn't impl Debug (no new public types
are added here anyway).

═══════════════════════════════════════════════════════════════════════════
WHEN DONE
═══════════════════════════════════════════════════════════════════════════
1. Fill ONLY the spec's `## Build Completion` (branch, PR, criteria, deviations,
   follow-ups, 3-question reflection). Edit nothing else in the spec body.
2. Append a build cost session to the spec front-matter `cost.sessions`
   (cycle: build, agent: claude-sonnet-4-6, interface: claude-code, null
   numerics, recorded_at: 2026-06-15, a one-line note).
3. Hand-edit the spec front-matter `task.cycle` from `build` to `verify`. DO NOT
   run `just advance-cycle` or `just archive-spec`.
4. Mark the build line `[x]` in the timeline
   (projects/PROJ-001-crustyimg-mvp/specs/SPEC-014-...-timeline.md) with ACCURATE
   wording — "PR #N opened" (real number). Never "merged"/"approved".
5. Commit on the branch (Conventional Commits, e.g.
   `feat(cli): convert command — forced-format re-encode (SPEC-014)`), end EACH
   commit with: `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`.
6. Push and open a PR on `jysf/crustyimg` (§13 template): Summary; Spec metadata
   PROJ-001/STAGE-003/SPEC-014; Decisions referenced [DEC-004 (codec/exit-4),
   DEC-015 (format precedence/exit-6), DEC-016 (quality knob), DEC-003 (metadata
   drop), DEC-012/007 (clap/typed errors)]; Constraints checked (one-line
   evidence each); New decisions: "No new DEC during build — convert reuses
   DEC-004/015/016". End with the Claude Code generated-with footer.
```
</content>
