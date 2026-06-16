# SPEC-013 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. This prompt is deliberately prescriptive — follow it literally.
> Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-013 ("shrink command — web-prep resize + quality
encode + strip"). You are NOT the architect; the spec is your source of truth.
This is the FIRST STAGE-003 command that touches the LIBRARY (a quality-aware
encode in src/sink), plus the CLI. Use ABSOLUTE paths for every file.

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST (before editing ANY file)
═══════════════════════════════════════════════════════════════════════════
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-013-shrink-command-web-prep-resize-quality-encode-strip
Confirm `git branch --show-current` shows that branch — NOT `main`, NOT any
`chore/*` branch — before you commit anything. ALL edits happen on this branch.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════
1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — §6 the EXACT gate commands (NOTE: the clippy gate is now
   `cargo clippy --all-targets -- -D warnings`), §11 conventions (typed errors;
   NO unwrap/expect/panic on recoverable paths; DIAGNOSTICS TO STDERR NEVER
   STDOUT; one image library only), §12 testing (native in-memory fixtures via
   the `image` crate), §13 git/PR, §15 build-cycle rules (spec edits limited to
   ## Build Completion; append a build cost session; create DEC only for NEW
   non-trivial decisions — NONE expected, DEC-016 already exists).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-013-shrink-command-web-prep-resize-quality-encode-strip.md
   — THE SPEC. Implement its "## Outputs", "## Failing Tests" and "## Notes for
   the Implementer" EXACTLY. They carry the precise signatures and the JPEG
   encoder API guidance.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-016-encode-quality-policy.md
   — the governing decision: `-q` → JPEG quality via JpegEncoder::new_with_quality;
   IGNORED for lossless formats (PNG/GIF/BMP/TIFF/ICO); `shrink` defaults to
   quality 80. Already on main — do NOT create a new DEC.
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-015-resize-output-format-and-partial-batch.md
   — format preservation + partial-batch exit 6 (inherited via run_pixel_op).
5. The SHIPPED code you change/reuse (read the real signatures):
   src/sink/mod.rs    — `encode_to_bytes`, `Sink::write`, `SinkError::Encode`,
                        sink unit tests (you add the `quality` param to write +
                        encode_to_bytes; the JPEG-quality branch is the new code).
   src/cli/mod.rs     — `run_pixel_op` (add a `quality` param; pass it to BOTH
                        sink.write calls), `run_resize`/`run_thumbnail` (callers;
                        mirror run_thumbnail for run_shrink), `run_apply` (also
                        calls sink.write), `thumbnail_params`/DEFAULT_THUMBNAIL_SIZE
                        (the pattern for shrink_params/DEFAULT_SHRINK_*),
                        `Commands::Shrink`, `CliError`/`code()`/exit_code_mapping_is_total
                        (DO NOT change these — no new variant).
   src/operation/registry.rs — `OperationRegistry::with_builtins().build("resize", &params)`.
   tests/cli.rs + tests/sink.rs + tests/common/mod.rs — conventions +
                        write_test_png / write_test_jpeg / gradient_jpeg.
   Cargo.toml         — `image` has the `jpeg` feature; JpegEncoder is available.
                        NO new dependency.

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact — follow the spec's ## Outputs)
═══════════════════════════════════════════════════════════════════════════
A. src/sink/mod.rs — quality-aware encode.
   A1. `encode_to_bytes(img: &Image, format: ImageFormat, quality: Option<u8>)
       -> Result<Vec<u8>, SinkError>`. When `format == ImageFormat::Jpeg` AND
       `quality == Some(q)`: clamp `let q = q.clamp(1, 100);` then encode via
       `::image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, q)` —
       prefer `img.pixels().write_with_encoder(encoder)`; if that method/path
       isn't right for image 0.25.10, use
       `encoder.encode_image(img.pixels())`. Map errors → `SinkError::Encode(e.to_string())`.
       ALL other (format, quality) cases: keep the existing
       `img.pixels().write_to(&mut cursor, format)` path (quality ignored).
   A2. `Sink::write(&self, img, input, overwrite, quality: Option<u8>, out)` —
       add the `quality` param; pass it to `encode_to_bytes` in the File/Dir/
       Stdout arms. The Display arm ignores quality.

B. src/cli/mod.rs — shrink command + thread quality.
   B1. `run_pixel_op(pipeline, inputs, global, quality: Option<u8>)` — add the
       param; pass `quality` to BOTH `sink.write(...)` calls (single + multi).
   B2. `run_resize` and `run_thumbnail`: pass `global.quality` as the new arg.
   B3. `run_apply`: pass `global.quality` to its `sink.write(...)` call.
   B4. `const DEFAULT_SHRINK_MAX: u32 = 1600;` `const DEFAULT_SHRINK_QUALITY: u8 = 80;`
   B5. `fn shrink_params(max: u32) -> OperationParams` → `{mode:"max", width:max}`
       (mirror thumbnail_params; derive nothing special; it's infallible).
   B6. `fn run_shrink(inputs: &[String], max: Option<u32>, global: &GlobalArgs)
       -> Result<(), CliError>` per the spec's Notes sketch: shrink_params(
       max.unwrap_or(DEFAULT_SHRINK_MAX)) → registry build("resize", &params)
       (map RegistryError::InvalidParams → CliError::Usage, Unknown → Usage) →
       Pipeline::new().push(op) → run_pixel_op(pipeline, inputs, global,
       Some(global.quality.unwrap_or(DEFAULT_SHRINK_QUALITY))).
   B7. Dispatch: replace `Commands::Shrink { .. } => Err(NotImplemented("shrink"))`
       with `Commands::Shrink { inputs, max } => run_shrink(inputs, *max, &cli.global)`.
   B8. DO NOT add a CliError variant; DO NOT touch code()/exit_code_mapping_is_total.

C. tests/sink.rs — update existing `sink.write(...)` calls to pass `None` for the
   new quality param (compile fix; assertions unchanged), and add the new encode
   unit tests (below).

GREP to be exhaustive: `grep -n "\.write(" src/cli/mod.rs src/sink/mod.rs` and
`grep -n "encode_to_bytes(" src/sink/mod.rs` — update EVERY call site.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (make them pass) — per the spec's ## Failing Tests
═══════════════════════════════════════════════════════════════════════════
UNIT (tests/sink.rs):
  - encode_jpeg_quality_lower_is_smaller — JPEG at Some(20) byte-len < Some(90);
    both decode to the same dims.
  - encode_png_ignores_quality — PNG at Some(10) byte-identical to None.
  - (update existing sink-write tests to pass None.)
INTEGRATION (tests/cli.rs — reuse write_test_png/write_test_jpeg):
  - shrink_defaults_bound_long_edge_and_shrink_file (2000×1000 jpg → long edge 1600, smaller file)
  - shrink_max_bounds_long_edge (200×100 --max 100 → 100×50)
  - shrink_quality_lower_is_smaller (-q 20 output smaller than -q 90, same dims)
  - shrink_png_preserves_format_quality_ignored (png --max 100 -q 10 → PNG, exit 0)
  - shrink_multi_input_fan_out_preserves_format (a.png→PNG, b.jpg→JPEG, --out-dir)
  - shrink_missing_input_exits_3
  - shrink_multi_without_out_dir_is_usage_error (exit 2, stderr mentions out-dir)
  - shrink_stdout_keeps_stdout_clean (-o -, stdout decodes, stderr empty)
  - REPOINT stub_command_returns_not_implemented from `shrink` → `convert`
    (e.g. `convert <png> --format png`), keep the exit-1 + "not yet implemented" asserts.

The existing resize_*/thumbnail_* integration tests + the other sink tests MUST
stay green (run the FULL suite). They don't pass -q, so output is unchanged.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════
- `convert`/`auto-orient` (own specs). - Selective metadata preserve / `--keep-gps`
  / container lane (STAGE-004) — write NO metadata-strip code (the re-encode
  already drops it). - rayon/parallel (STAGE-005). - WebP/AVIF, PNG compression
  control. - Any new dependency, Operation, CliError variant, or DEC. If you
  think one is needed, STOP and add a question to guidance/questions.yaml.

═══════════════════════════════════════════════════════════════════════════
THE GATES (run from repo root; ALL must pass before the PR)
═══════════════════════════════════════════════════════════════════════════
  cargo build
  cargo test
  cargo clippy --all-targets -- -D warnings     # NOTE: --all-targets is the CI gate now
  cargo fmt --check                              # run `cargo fmt` first to fix

RUN GATES AND COMMIT INCREMENTALLY — e.g. commit once the sink+cli compile and
clippy/fmt are clean, then again after the tests are green. Do NOT leave all
work uncommitted to the end; if your session is interrupted, a green committed
checkpoint must survive. (Hard lesson from SPEC-011: a dropped build left work
uncommitted and the prescribed tests unwritten.)

BEFORE YOU FINISH: re-read the spec's ## Failing Tests and CONFIRM EACH NAMED
TEST EXISTS in the code and runs — list them and check each off. A passing test
COUNT does not prove the prescribed tests were written.

Also: derive `Debug` on any new public type; do not `{:?}`-format a type that
doesn't impl Debug.

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
   (projects/PROJ-001-crustyimg-mvp/specs/SPEC-013-...-timeline.md) with ACCURATE
   wording — "PR #N opened" (real number). Never "merged"/"approved".
5. Commit on the branch (Conventional Commits, e.g.
   `feat(cli): shrink command with quality-aware encode (SPEC-013)`), end EACH
   commit with: `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`.
6. Push and open a PR on `jysf/crustyimg` (§13 template): Summary; Spec metadata
   PROJ-001/STAGE-003/SPEC-013; Decisions referenced [DEC-016 (encode quality),
   DEC-015 (format/exit-6), DEC-008/014 (resize op/params), DEC-004 (codec
   policy), DEC-003 (metadata drop)]; Constraints checked (one-line evidence
   each); New decisions: "No new DEC during build — DEC-016 already governs".
   End with the Claude Code generated-with footer.
```
