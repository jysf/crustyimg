# SPEC-005 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. Do not rely on any prior conversation. This prompt is
> deliberately prescriptive — follow it literally.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-005 ("Sink output abstraction"). You are NOT
the architect; the spec file is your source of truth. Use ABSOLUTE paths.

Read these files in order before writing any code:

1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — conventions: §5 stack (`image` 0.25, `thiserror` 2 in lib, `viuer` for
   display, NO async, NO new top-level dep without a DEC), §6 EXACT commands
   (the four gates), §11 coding conventions (library-first; `src/sink/**` may
   depend on `image` but NOT on clap/recipes/source-internals/terminals beyond
   viuer; typed errors; NO unwrap/expect/panic! on recoverable paths;
   DIAGNOSTICS TO STDERR NEVER STDOUT; group imports std/external/local), §12
   testing (unit in #[cfg(test)], integration under tests/, NATIVE fixtures —
   NO ImageMagick, NO committed binary fixtures), §13 git/PR (branch,
   conventional commits, PR body template), §15 build-cycle rules (spec edits
   LIMITED to ## Build Completion).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-005-sink-output-abstraction.md
   — THE SPEC. Implement its "## Failing Tests" and "## Outputs" exactly. Read
   "## Implementation Context" and "## Notes for the Implementer" in full;
   they are written for you.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-011-viuer-terminal-display.md
   — the dependency decision: add `viuer` ONLY, as an OPTIONAL dep behind an
   off-by-default `display` cargo feature. The viuer RENDER CALL is
   #[cfg(feature = "display")]; the Sink::Display variant + NotATty refusal are
   ALWAYS compiled. Do NOT add any other crate.
   Also read:
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-002-single-image-model-and-operation-trait.md
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-004-codec-policy-pure-rust-default.md
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-007-error-handling-thiserror-anyhow.md
   (and SKIM /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-003-metadata-dual-lane.md
    — the preserve/drop-GPS policy is STAGE-004, OUT OF SCOPE here; you encode
    pixels only and must NOT destroy the Image's metadata bundle.)
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/constraints.yaml
   — full text of the constraints that apply: untrusted-input-hardening,
   ergonomic-defaults, no-unwrap-on-recoverable-paths,
   no-new-top-level-deps-without-decision, single-image-library,
   clippy-fmt-clean, every-public-fn-tested, test-before-implementation.
5. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/architecture.md
   (§ Components -> Sink; § Module/Layer Structure — layering `sink -> image`)
   and /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/data-model.md
   (§ "Name templates (Sink)" — the {stem}/{ext}/{name}/{parent} token table,
   default template `{stem}.{ext}`, and the traversal guard) and
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/api-contract.md
   (§ Global Options, § "stdin / stdout (`-`)", § Exit Codes — exit 5 = output
   write failed / refused / traversal) and
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/SECURITY.md
   (§ "Path traversal on output").
6. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/image/mod.rs
   (the canonical Image; you call `img.pixels()` -> `&DynamicImage` then
   `pixels.write_to(&mut writer, ImageFormat)`; note `solid_png` test helper to
   mirror; there is NO Image::save) and
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/source/mod.rs
   (Input::stem()/path() — the naming context; and the MODULE-LOCAL SourceError
   pattern to MIRROR for SinkError) and
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/error.rs
   (the ImageError + Result style) and
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/lib.rs
   (current module declarations).
7. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/Cargo.toml
   — the existing `=`-pinned dependency style to match; tempfile is already a
   dev-dep.

Before coding, mark the build cycle `[~]` in:
  /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-005-sink-output-abstraction-timeline.md

If you hit something needing architect judgment or an external unblock
(constraint unclear; a dependency beyond `viuer` + the existing `tempfile`
dev-dep genuinely seems required; scope drift into the metadata preserve policy,
Recipe/registry, CLI/clap parsing, rayon batch, decode limits, or encoder
quality plumbing), change the marker to `[?]` with a one-line reason and STOP.
`[?]` is not a "don't know what to do" dumping ground — ask if unsure by adding
to:
  /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml

Create the branch off the LATEST main (sync first):
  git checkout main && git pull
  git checkout -b feat/spec-005-sink-output-abstraction

Implement to make the spec's "## Failing Tests" pass. The exact work:

A. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/Cargo.toml
   — add to [dependencies]:
       viuer = { version = "=X.Y.Z", optional = true }
     Pin the EXACT latest published version (check crates.io; architecture.md
     pre-named 0.9 but current is ~0.11 — pin the REAL latest, `=`-pinned to
     match existing style).
   — add a [features] table:
       [features]
       display = ["dep:viuer"]
     (default features stay empty — display is OFF by default.)
   — add NOTHING ELSE. No is-terminal/atty crate (use std::io::IsTerminal). No
     new dev-dep (tempfile is already present). No walkdir/rayon/serde/clap.
   This is justified by DEC-011 — NO new DEC needed.

B. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/sink/mod.rs
   (NEW) — define, EXACTLY per the spec's "## Outputs" New exports:
     use std::io::{Write, IsTerminal};
     use std::path::{Path, PathBuf};
     use ::image::{DynamicImage, ImageFormat};   // alias `::image` to avoid the
                                                  // module-name collision, as
                                                  // src/image/mod.rs does.
     use crate::image::Image;

     - `pub enum Sink { File { path: PathBuf, format: Option<ImageFormat> },
        Dir { dir: PathBuf, template: String, format: Option<ImageFormat> },
        Stdout { format: Option<ImageFormat> }, Display }`
     - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum Overwrite {
        Forbid, Allow }`
     - `#[derive(Debug, thiserror::Error)] pub enum SinkError` with EXACTLY the
       variants in the spec (Io(#[from] std::io::Error), Encode(String),
       UnknownFormat, UnsupportedExtension(String), Traversal(String),
       AlreadyExists(String), NotATty, Display(String)) and the #[error(...)]
       messages shown there. Keep SinkError LOCAL to this module (mirror
       SourceError in src/source/); do NOT widen the crate Error.
     - `pub struct SinkInput<'a> { pub stem: &'a str, pub path: Option<&'a Path> }`
     - the four FREE helpers, each pub and each with a unit test:
         `pub fn format_from_extension(path: &Path) -> Result<ImageFormat, SinkError>`
            — lowercase the extension; map png->Png, jpg|jpeg->Jpeg, gif->Gif,
              bmp->Bmp, tif|tiff->Tiff, ico->Ico; NO extension -> UnknownFormat;
              unknown extension -> UnsupportedExtension(ext.into()).
         `pub fn extension_for_format(format: ImageFormat) -> &'static str`
            — Png->"png", Jpeg->"jpg", Gif->"gif", Bmp->"bmp", Tiff->"tiff",
              Ico->"ico"; for any other ImageFormat variant return a sensible
              lowercase default (it will not be reached for the core set).
         `pub fn expand_template(template: &str, stem: &str, ext: &str,
              path: Option<&Path>) -> String`
            — replace {stem}, {ext}, {name} (file name w/ extension, from
              path.file_name(); fall back to format!("{stem}.{ext}") if path is
              None), {parent} (path.parent().file_name(), else ""). Leave any
              other {token} LITERAL. Returns the file NAME only (no directory).
         `pub fn safe_join(dir: &Path, file_name: &str) -> Result<PathBuf, SinkError>`
            — see the TRAVERSAL RULE below; returns the safe joined PathBuf or
              SinkError::Traversal.
     - `impl Sink { pub fn write(&self, img: &Image, input: &SinkInput<'_>,
          overwrite: Overwrite, out: &mut dyn Write) -> Result<(), SinkError> }`
       Dispatch by variant:
         * File { path, format }: choose format = format.or_else(|| infer from
           path extension); if neither -> the appropriate UnknownFormat /
           UnsupportedExtension from format_from_extension. Run the OVERWRITE
           guard on `path`. Open BufWriter<File> (mapping io error via #[from]),
           encode pixels, done.
         * Dir { dir, template, format }: choose format = format OR the default
           `ImageFormat::Png` (the dir-sink default ext is png); compute
           ext = extension_for_format(format); file_name = expand_template(
           template, input.stem, ext, input.path); full = safe_join(dir,
           &file_name)?; OVERWRITE guard on `full`; encode to BufWriter<File>.
         * Stdout { format }: format MUST be Some -> else UnknownFormat. Encode
           pixels straight to `out` (the injected writer). NO overwrite guard,
           NO file. Write ONLY the encoded bytes to `out`.
         * Display: if !std::io::stdout().is_terminal() -> Err(NotATty). Else,
           under #[cfg(feature = "display")] call viuer to print img.pixels()
           (map any viuer error to SinkError::Display(e.to_string())); under
           #[cfg(not(feature = "display"))] return Err(SinkError::Display(
           "built without the `display` feature".into())). The NotATty check
           comes FIRST and is feature-independent.
     - a private `encode_to(writer: &mut dyn Write, img: &Image, format:
       ImageFormat) -> Result<(), SinkError>` helper: `img.pixels().write_to(
       &mut <writer>, format).map_err(|e| SinkError::Encode(e.to_string()))`.
       NOTE: DynamicImage::write_to needs `W: Write + Seek` for SOME formats but
       the core set used here (png/jpeg/gif/bmp/tiff/ico) works with a plain
       Write target via a buffer; if a format needs Seek, encode into an
       in-memory Vec<u8> (Cursor) first, then write the Vec to `out`/file. Pick
       whichever compiles cleanly for the core set; the Vec-then-write approach
       is the safe default and keeps `out: &mut dyn Write` simple.
     - `#[cfg(test)] mod tests` with the UNIT tests named in the spec.

   OVERWRITE guard (File + Dir only): if the destination path `exists()` AND
   `overwrite == Overwrite::Forbid` -> return SinkError::AlreadyExists(
   path.display().to_string()) BEFORE opening for write (do not truncate).
   Overwrite::Allow proceeds.

   TRAVERSAL RULE (safe_join — the trickiest bit):
     - Reject immediately if `file_name` is empty, is absolute
       (Path::new(file_name).is_absolute()), contains a `..` component, or
       contains a path separator ('/' or '\\'). On any of these ->
       SinkError::Traversal(file_name.into()).
     - Canonicalize `dir` (std::fs::canonicalize) — a MISSING dir maps to a
       typed SinkError (Io via #[from], or Traversal; either is acceptable per
       the spec's missing_out_dir test which only asserts "is a SinkError").
       Do NOT create the dir.
     - Build candidate = canonical_dir.join(file_name). Verify
       candidate.starts_with(&canonical_dir). (Because file_name has no
       separators/.. and dir is canonical, the parent IS canonical_dir; the
       starts_with check is the belt-and-suspenders confirmation.) If it does
       not start_with -> Traversal. Return the candidate.
     - Compare CANONICALIZED paths on BOTH sides. On Windows canonicalize adds
       a \\?\ verbatim prefix to both — never compare a raw arg to a canonical
       path.

C. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/lib.rs
   — add `pub mod sink;` (keep existing modules + version() + its test). Add a
   one-line doc comment mentioning SPEC-005 like the other modules.

D. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/tests/sink.rs
   (NEW) — the integration tests named in the spec, through the crate's PUBLIC
   exports only (`crustyimg::sink::{Sink, Overwrite, SinkError, SinkInput,
   format_from_extension, extension_for_format, expand_template, safe_join}` and
   `crustyimg::image::Image`). Use `tempfile::tempdir()` for fixtures; create
   the output dir inside the test. Produce a real tiny image in-memory by
   encoding a solid RgbImage to PNG bytes (mirror the `solid_png` helper in
   src/image/mod.rs tests: `image::RgbImage::from_pixel(w,h,Rgb([..]))` ->
   `DynamicImage::write_to(&mut Cursor, ImageFormat::Png)`), then
   `Image::from_bytes(&png)` to get a real `Image` to write. `.unwrap()` is fine
   in tests. For Stdout, pass `out = &mut Vec::<u8>::new()`. For File/Dir/Display
   calls that don't use stdout, still pass a `&mut Vec::<u8>::new()` (or
   `&mut std::io::sink()`) as `out`. The EXACT test names:
     - file_sink_writes_readable_image
     - format_inferred_from_extension_jpeg_and_png
     - explicit_format_overrides_missing_extension
     - unsupported_extension_is_typed_error
     - dir_sink_expands_name_template
     - stdout_sink_writes_only_encoded_bytes
     - overwrite_guard_forbids_then_allows
     - dir_sink_rejects_traversal_template
     - missing_out_dir_is_typed_not_panic
     - display_sink_refuses_non_tty
   Match error VARIANTS (e.g. `matches!(err, SinkError::Traversal(_))`), not
   strings. Assert re-readability by `Image::load(path)` / `Image::from_bytes`
   and comparing dimensions / `source_format()`.

HARD RULES (do not violate):
- untrusted-input-hardening: the Dir sink MUST reject a name-template/expanded
  name that escapes the canonicalized out-dir (.., separators, absolute) ->
  SinkError::Traversal; and File/Dir MUST NOT overwrite an existing file when
  Overwrite::Forbid -> SinkError::AlreadyExists. Both are typed errors, NEVER
  panics. The guard is on the EXPANDED name (a template can carry separators);
  Input::stem() itself never does.
- no-new-top-level-deps-without-decision: `viuer` is the ONLY new dep, OPTIONAL,
  behind `display`, pre-justified by DEC-011 — NO new DEC. If you think you need
  ANY other crate (is-terminal/atty, walkdir, rayon, serde, clap), STOP, add to
  questions.yaml, mark `[?]`. Use std::io::IsTerminal for tty detection.
- single-image-library: encode via the ONE `image` crate only (img.pixels()
  .write_to). viuer reuses `image` so it does not add a second pixel lib — do
  not pull any other image crate.
- no-unwrap-on-recoverable-paths: typed SinkError in library code;
  .unwrap()/.expect() ONLY in #[cfg(test)] and tests/. write()/safe_join()/
  format_from_extension() must NEVER panic — a missing dir / bad extension /
  traversal / existing file is a typed Err.
- diagnostics OFF stdout: the `out` writer carries ONLY encoded image bytes.
  Do NOT println!/write any log/diagnostic to `out`. (There is no logging in
  this spec anyway — just don't introduce any to `out`.)
- METADATA preserve policy is OUT OF SCOPE (STAGE-004). Encode pixels only; do
  NOT re-attach or strip the Image's metadata bundle, and do NOT implement
  drop-GPS/preserve here.
- Resist scope creep: NO clap/CLI arg parsing, NO Recipe/registry, NO rayon
  batch loop, NO decode limits, NO quality flag. Those are SPEC-006/007,
  STAGE-005, STAGE-006.

Run the FOUR GATES from the repo root and make ALL pass before opening the PR:
  cargo test
  cargo test --features display      # the display path must also compile/pass
  cargo clippy -- -D warnings
  cargo clippy --features display -- -D warnings
  cargo fmt --check
(If `cargo test --features display` cannot run viuer in the sandbox, at minimum
`cargo build --features display` MUST compile cleanly — note in Build
Completion which display-feature gate you were able to run.)

When the gates pass:

1. Fill in the spec's `## Build Completion` section ONLY (the build cycle may
   edit nothing else in the spec). Answer the three reflection questions
   honestly. Set Branch, PR, "All acceptance criteria met?", new decisions
   ("No new DEC — DEC-011 written at design time"), deviations, follow-ups.
2. Append a BUILD cost session entry to the spec's front-matter `cost.sessions`
   list (after the existing design entry):
       - cycle: build
         agent: claude-sonnet-4-6
         interface: claude-code
         tokens_total: null
         estimated_usd: null
         duration_minutes: <estimate>
         recorded_at: <YYYY-MM-DD>
         notes: "subagent; cost not separately reported"
   In Claude Code, run /cost and use its numbers; if unavailable, use null
   numeric fields with the note above.
3. Advance the cycle to verify. Run from the repo root:
     just advance-cycle SPEC-005 verify
   NOTE: `just advance-cycle` MIS-GLOBS when multiple SPEC-005* files exist
   (the spec + its timeline). Do NOT fight it — instead set
   `task.cycle: verify` BY HAND in the front-matter of:
     /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-005-sink-output-abstraction.md
   and verify with a quick grep that ONLY the spec (not the timeline) changed.
4. Create DEC-* files only for NON-TRIVIAL build decisions. DEC-011 already
   exists (the viuer dependency + feature gate) — do NOT duplicate it. The
   default-format-png choice for the dir sink and the extension map do NOT
   warrant a DEC. Most likely outcome: "No new DEC".
5. Commit with a Conventional Commit (one spec per PR — constraint
   one-spec-per-pr), e.g.:
     feat(sink): file/dir-template/stdout/display output with traversal + overwrite guards (SPEC-005)
   End the commit message with:
     Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
6. Push the branch and open a GitHub PR on jysf/crustyimg (per AGENTS.md §13)
   using the gh CLI. PR title must carry the spec id, e.g.
   `feat(sink): file/dir/stdout/display output abstraction (SPEC-005)`. PR body
   must follow the AGENTS.md §13 template:
     ## Summary
     - <bullets: Sink enum (File/Dir/Stdout/Display) + Overwrite + SinkInput +
       SinkError; Sink::write to an injected `out` writer; format_from_extension
       + extension_for_format + expand_template + safe_join; format inference;
       name-template expansion; path-traversal rejection; overwrite guard;
       stdout-only-encoded-bytes; viuer display behind off-by-default `display`
       feature with non-tty refusal; module wiring; unit + integration tests>
     ## Spec metadata
     - **Project:** PROJ-001
     - **Stage:** STAGE-001
     - **Spec:** SPEC-005
     ## Decisions referenced
     DEC-011 (viuer optional dep behind `display` feature),
     DEC-002 (encode the one canonical Image via the single pixel lib),
     DEC-004 (pure-Rust core-format encoders only),
     DEC-007 (typed SinkError via thiserror; exit 5 mapping at the binary later),
     DEC-003 (NOTE: metadata preserve policy deferred to STAGE-004)
     ## Constraints checked
     - `untrusted-input-hardening` ✅ — <evidence: safe_join rejects ../abs/sep
       escapes via canonicalize + starts_with; overwrite guard returns
       AlreadyExists without --yes; tests dir_sink_rejects_traversal_template,
       overwrite_guard_forbids_then_allows>
     - `ergonomic-defaults` ✅ — <evidence: `-o out.png` single path needs no
       extra flags; default dir template {stem}.{ext}>
     - `no-unwrap-on-recoverable-paths` ✅ — <evidence: typed SinkError; no
       panic on missing dir / bad ext / traversal / existing file>
     - `no-new-top-level-deps-without-decision` ✅ — <evidence: viuer only,
       optional, justified by DEC-011; tty via std::io::IsTerminal>
     - `single-image-library` ✅ — <evidence: encode via `image` only; viuer
       reuses `image`, no second pixel lib>
     - `clippy-fmt-clean` ✅ — <evidence: clippy + fmt clean, default AND
       --features display>
     - `every-public-fn-tested` ✅ — <evidence: Sink::write, the four helpers,
       each covered>
     - `test-before-implementation` ✅ — failing tests from spec made to pass
     - `one-spec-per-pr` ✅ — SPEC-005 only
     ## New decisions
     - No new DEC — DEC-011 (viuer + `display` feature) written at design time.
   End the PR body with the Claude Code generated-with footer.
7. Mark build `[x]` in the timeline with the PR number, cost, and date:
     - [x] **build** — prompt: `prompts/SPEC-005-build.md`
            PR #NNN, $X.XX, completed <YYYY-MM-DD>

Watch for (Sonnet-specific reminders — the trickiest bits):
- FORMAT INFERENCE: `format_from_extension` lowercases the extension. NO
  extension -> UnknownFormat (NOT UnsupportedExtension). An extension not in the
  core set -> UnsupportedExtension. Stdout with format=None -> UnknownFormat
  (there is no path to infer from). Use ::image::ImageFormat directly; do not
  invent your own format enum.
- ENCODING via the `image` crate: there is NO Image::save. Call
  `img.pixels().write_to(&mut target, format)`. Some `image` encoders require
  `Seek` on the writer; since `out` is a plain `&mut dyn Write` (no Seek), the
  SAFE pattern is: encode into an in-memory `Vec<u8>` via
  `std::io::Cursor::new(Vec::new())` (Cursor is Write+Seek), then write that Vec
  to the file BufWriter or to `out`. Use that buffer-then-write approach
  uniformly so File/Dir/Stdout share one `encode_to_bytes` path.
- TRAVERSAL CANONICALIZATION: canonicalize the out-dir AND compare with
  starts_with on the canonicalized candidate. canonicalize ERRORS on a missing
  dir — that error is a typed SinkError (do NOT create the dir, do NOT panic).
  Reject any expanded name containing `..`, a path separator, or an absolute
  component BEFORE joining. On Windows canonicalize prefixes BOTH sides with
  \\?\ so starts_with still holds — never compare raw vs canonical.
- TTY DETECTION: `use std::io::IsTerminal;` then
  `std::io::stdout().is_terminal()`. Under `cargo test` stdout is piped
  (non-tty), so Sink::Display returns NotATty — that is the ONLY display
  behavior the test pins. Do the NotATty check FIRST, before any
  #[cfg(feature)] viuer code, so the refusal test runs whether or not the
  feature is enabled.
- FEATURE GATING: viuer is `optional = true`; `[features] display =
  ["dep:viuer"]`; default features OFF. The viuer RENDER CALL is
  #[cfg(feature = "display")]; the Sink::Display variant + the NotATty refusal
  are ALWAYS compiled. The default `cargo build`/`cargo test`/clippy/fmt must be
  green with viuer NOT in the graph; ALSO confirm `cargo build --features
  display` (or test) compiles.
- KEEP DIAGNOSTICS OFF `out`: `out` gets encoded image bytes and nothing else.
  The stdout_sink_writes_only_encoded_bytes test asserts the captured Vec length
  equals the encoded image length — any stray byte fails it.
- OVERWRITE before truncate: check Path::exists() BEFORE opening the file for
  write; never truncate then error.
- METADATA: do NOT touch the preserve/drop-GPS policy (STAGE-004). Encode pixels
  only; leave Image::metadata() intact.
- The gates (cargo test, clippy -D warnings, fmt --check — default AND
  --features display) must ALL pass before you open the PR.
```
