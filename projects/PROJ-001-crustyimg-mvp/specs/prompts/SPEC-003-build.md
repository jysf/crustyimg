# SPEC-003 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. Do not rely on any prior conversation. This prompt is
> deliberately prescriptive — follow it literally.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-003 ("Operation trait and Pipeline"). You are
NOT the architect; the spec file is your source of truth. Use ABSOLUTE paths.

Read these files in order before writing any code:

1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — conventions: §5 stack (`image` 0.25, `thiserror` 2 in lib, NO async, NO
   new top-level dep without a DEC), §6 EXACT commands (the four gates), §11
   coding conventions (library-first; the pixel core `src/operation/**` and
   `src/pipeline/**` must NOT depend on clap/files/terminals; typed errors; NO
   unwrap/expect/panic! on recoverable paths; group imports std/external/local),
   §12 testing (unit tests in `#[cfg(test)] mod tests`; integration tests under
   tests/; native in-memory fixtures — NO ImageMagick, NO committed binary
   fixtures), §13 git/PR conventions, §15 build-cycle rules (build edits to the
   spec are limited to `## Build Completion`; verify is read-only; ship
   bookkeeping lands on main).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-003-operation-trait-and-pipeline.md
   — THE SPEC. Read it ENTIRELY, especially "## Outputs" (exact signatures),
   "## Acceptance Criteria", "## Failing Tests" (exact test names + assertions),
   and the whole "## Implementation Context" + "## Notes for the Implementer"
   (the OperationParams dependency-free note and the constructor decision are
   load-bearing — do not skip them).
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/stages/STAGE-001-foundation-and-pipeline-core.md
   — the parent stage (this is backlog item #3). Note the Design Notes: keep
   the Operation trait SMALL; metadata is a SEPARATE lane, do NOT route it
   through Operation.
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/brief.md
   — the project.
5. The decisions referenced by the spec:
   - /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-002-single-image-model-and-operation-trait.md
   - /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-007-error-handling-thiserror-anyhow.md
6. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/constraints.yaml
   — full text of the constraints that apply: decode-once-no-per-op-disk,
   no-unwrap-on-recoverable-paths, single-image-library, clippy-fmt-clean,
   every-public-fn-tested, test-before-implementation.
7. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/data-model.md
   (§ Operation / Pipeline) and
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/architecture.md
   (§ Module/Layer Structure — pipeline → operation → image; operation → image;
   the pixel core has no clap/file/terminal deps).
8. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/image/mod.rs
   and /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/error.rs
   and /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/lib.rs
   — what SPEC-002 already shipped: the `Image` type + `pixels()`/`width()`/
   `height()`/`info()`/`metadata()`, the `ImageError` + `Result` pattern to
   mirror, and the current module declarations. NOTE: `Image` has NO public
   DynamicImage constructor yet — you will add a minimal one (see the spec's
   Notes → "the constructor question").

Before coding, mark the build cycle `[~]` in:
  /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-003-operation-trait-and-pipeline-timeline.md

If you hit something needing architect judgment or an external unblock
(constraint unclear; a dependency beyond `image`/`thiserror` genuinely seems
required; scope drift into Source/Sink/Recipe/registry/CLI/metadata or any
real transform like resize), change the marker to `[?]` with a one-line reason
and STOP. `[?]` is not a "don't know what to do" dumping ground — ask if unsure
by adding to:
  /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml

Create the branch off the LATEST main (sync first):
  git checkout main && git pull
  git checkout -b feat/spec-003-operation-trait-and-pipeline

Implement to make the spec's "## Failing Tests" pass. The exact work:

A. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/image/mod.rs
   — add a MINIMAL public constructor so Operations can return a transformed
   Image and tests can build one from a DynamicImage. Add `with_pixels(self,
   DynamicImage) -> Image` (preferred) and/or `from_parts(DynamicImage,
   ImageFormat, Option<MetadataBundle>) -> Image`. Carry source_format +
   metadata through UNCHANGED (operations never touch metadata — DEC-003).
   Add a unit test for each new public fn. This additive accessor needs NO DEC.

B. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/operation/mod.rs
   (NEW) — define, EXACTLY per the spec's Outputs + Notes:
     - `pub enum OperationParams { None }` (DEPENDENCY-FREE placeholder — do NOT
       import toml/serde; SPEC-006 widens it later). derive Debug/Clone/
       PartialEq/Eq.
     - `pub enum OperationError { Apply { op: &'static str, reason: String } }`
       via `thiserror::Error` with a clear `#[error(...)]` message (mirror
       src/error.rs ImageError style). This is a NEW typed error; you may put
       it here in operation/ (do NOT need to widen the crate error).
     - `pub trait Operation { fn name(&self) -> &'static str; fn params(&self)
       -> OperationParams; fn apply(&self, img: Image) -> Result<Image,
       OperationError>; }`
     - `pub struct Identity;` impl Operation: name = "identity", params =
       OperationParams::None, apply returns `Ok(img)` unchanged.
     - `pub struct Invert;` impl Operation: name = "invert", params =
       OperationParams::None, apply = HAND-ROLLED per-channel inversion. Convert
       `img.pixels().to_rgba8()`, map `[r,g,b,a] -> [255-r,255-g,255-b,a]`
       (alpha preserved), wrap `DynamicImage::ImageRgba8(buf)`, return
       `Ok(img.with_pixels(...))`. NO imageproc, NO photon-rs, NO new dep.
     - `#[cfg(test)] mod tests` with the operation unit tests named in the spec.
   Refer to the crate as `::image` to avoid the module-name collision.

C. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/pipeline/mod.rs
   (NEW) — define, EXACTLY per the spec's Outputs:
     - `pub struct Pipeline { ops: Vec<Box<dyn Operation>> }`
     - `pub fn new() -> Self`, `pub fn push(self, Box<dyn Operation>) -> Self`
       (builder), `pub fn len(&self) -> usize`, `pub fn is_empty(&self) -> bool`,
       `pub fn run(&self, img: Image) -> Result<Image, OperationError>` (fold:
       thread the Image through `op.apply(img)?` in order — `?` gives
       halt-on-first-error; empty pipeline returns img unchanged; NO disk I/O,
       NO clone between ops).
     - `impl Default for Pipeline` (= new()) — required so clippy
       `new_without_default` stays clean.
     - `#[cfg(test)] mod tests` with the pipeline unit tests named in the spec,
       including the test-only helper ops `RecordOrder` (records order via
       Rc<RefCell<Vec<&'static str>>> or Arc<Mutex<…>>) and `AlwaysFails`
       (returns OperationError::Apply). Match on the ERROR VARIANT, not strings.

D. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/lib.rs
   — add `pub mod operation;` and `pub mod pipeline;` (keep existing modules +
   version() + its test).

E. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/tests/pipeline.rs
   (NEW) — the integration tests named in the spec, through the crate's PUBLIC
   exports only (`crustyimg::pipeline::Pipeline`, `crustyimg::operation::{
   Identity, Invert}`, `crustyimg::image::Image`):
     - `public_pipeline_inverts_via_crate_api`
     - `empty_pipeline_is_identity_via_crate_api`
     - `operation_and_pipeline_sources_do_no_disk_io` — the STRUCTURAL guard:
       read src/operation/mod.rs and src/pipeline/mod.rs as text (this test
       reading source is allowed) and assert the NON-TEST code references none
       of: `std::fs`, `std::io::`, `File`, `OpenOptions`, `read_to_string`,
       `std::path`, `Path`. Exclude the `#[cfg(test)] mod tests` region (split
       on the `mod tests` marker). Document the exact heuristic in Build
       Completion.
   Build fixtures natively: `image::RgbaImage::from_fn(...)` →
   `DynamicImage::ImageRgba8(buf)` → your Image constructor. Compare pixels via
   `img.pixels().to_rgba8().into_raw()`. `.unwrap()` is fine in tests.

HARD RULES (do not violate):
- decode-once-no-per-op-disk: NO std::fs / std::io file ops / std::path in the
  LIBRARY code of operation/ and pipeline/. Ops are pure in-memory.
- single-image-library: ONLY the `image` crate. Invert is hand-rolled. NO
  imageproc/photon-rs.
- NO NEW DEPENDENCY. `image`/`thiserror` already exist and are pre-justified by
  DEC-002/DEC-007 — NO new DEC for them. If you think you need ANY other crate
  (including `toml`/`serde`), STOP, add to questions.yaml, mark `[?]`. Do NOT
  add it. (OperationParams is the dependency-free enum in the spec's Notes.)
- no-unwrap-on-recoverable-paths: typed errors in library code; .unwrap()/.
  expect() only in #[cfg(test)] and tests/. Pipeline::run returns the Err on op
  failure — it must NOT panic.
- Keep the Operation trait SMALL (name/params/apply). Do NOT add registry,
  serde, recipe, source, sink, clap, or metadata-as-operation. Those are
  SPEC-004/005/006/007 and STAGE-004.

Run the four gates locally until ALL green (from the repo root, absolute):
  cargo build
  cargo test
  cargo clippy -- -D warnings
  cargo fmt --check

When done:
1. Fill in the spec's "## Build Completion" section INCLUDING: the branch, PR,
   "all acceptance criteria met?", the constructor choice (with_pixels vs
   from_parts) and why, how you implemented the no-disk-IO guard, deviations,
   follow-ups, and the THREE build-phase reflection questions (answer honestly).
   AGENTS.md §13/§15: build-cycle edits to the spec stay LIMITED to
   `## Build Completion`. Do NOT touch verify/ship bookkeeping (timeline marks
   beyond your build line, ship prompt, ship reflection, cost totals, stage
   backlog, archiving) — those land on `main` later by the orchestrator.
2. Append a build cost session entry to the spec's `cost.sessions`:
     - cycle: build
       agent: claude-sonnet-4-6
       interface: claude-code
       tokens_input: <best available or null>
       tokens_output: <best available or null>
       estimated_usd: <best available or null>
       duration_minutes: <estimate>
       recorded_at: <YYYY-MM-DD>
       notes: "subagent; cost not separately reported"
   In Claude Code, run /cost and use its numbers; if unavailable, use null
   numeric fields with the note above.
3. Run from the repo root:
     just advance-cycle SPEC-003 verify
   NOTE: if `just advance-cycle` mis-globs (it can when multiple SPEC-003*
   files exist), do NOT fight it — instead set `task.cycle: verify` by hand in
   the front-matter of:
     /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-003-operation-trait-and-pipeline.md
4. Create DEC-* files only for NON-TRIVIAL build decisions (the additive Image
   constructor does NOT warrant a DEC; the OperationParams placeholder does NOT
   warrant a DEC — it is explicitly mandated by this spec). Most likely: "No
   new DEC".
5. Commit with a Conventional Commit (one spec per PR — constraint
   one-spec-per-pr), e.g.:
     feat(pipeline): Operation trait + decode-once Pipeline (SPEC-003)
   End the commit message with:
     Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
6. Push the branch and open a GitHub PR on jysf/crustyimg (per AGENTS.md §13)
   using the gh CLI. PR title must carry the spec id, e.g.
   `feat(pipeline): Operation trait + decode-once Pipeline (SPEC-003)`. PR body
   must follow the AGENTS.md §13 template:
     ## Summary
     - <bullets: Operation trait, OperationError, OperationParams placeholder,
       Identity + Invert ops, Pipeline fold executor, Image::with_pixels/
       from_parts, module wiring, tests>
     ## Spec metadata
     - **Project:** PROJ-001
     - **Stage:** STAGE-001
     - **Spec:** SPEC-003
     ## Decisions referenced
     DEC-002 (Operation trait + decode-once Pipeline over the canonical Image),
     DEC-007 (typed OperationError via thiserror; no panic on recoverable paths)
     ## Constraints checked
     - `decode-once-no-per-op-disk` ✅ — <evidence: no fs/io/path in op/pipeline
       library code; structural guard test>
     - `single-image-library` ✅ — <evidence: only `image`; Invert hand-rolled>
     - `no-unwrap-on-recoverable-paths` ✅ — <evidence: run() returns Err>
     - `clippy-fmt-clean` ✅ — <evidence>
     - `every-public-fn-tested` ✅ — <evidence>
     - `test-before-implementation` ✅ — failing tests from spec made to pass
     - `one-spec-per-pr` ✅ — SPEC-003 only
     ## New decisions
     - No new DEC — `image`/`thiserror` pre-justified by DEC-002/DEC-007; the
       Image constructor + OperationParams placeholder are spec-mandated
       additive changes.
   End the PR body with the Claude Code generated-with footer.
7. Mark build `[x]` in the timeline with the PR number, cost, and date:
     - [x] **build** — prompt: `prompts/SPEC-003-build.md`
            PR #NNN, $X.XX, completed <YYYY-MM-DD>

Watch for (Sonnet-specific reminders):
- The `image`/module name collision — refer to the crate as `::image` inside
  src/operation and src/pipeline.
- Do NOT import `toml` or `serde` for OperationParams — use the bare
  `enum OperationParams { None }` the spec mandates. Adding either is a
  constraint violation (no-new-top-level-deps-without-decision).
- Do NOT add `imageproc` for Invert — hand-roll the 255-v loop over to_rgba8().
- clippy will FAIL the build if you define `len()` without `is_empty()` and if
  you define `new()` without `impl Default` — both are in the contract, so
  implement all of them.
- Pipeline::run must HALT on the first error (the `?` operator does this) and
  must NOT panic — return the Err.
- Resist scope creep: NO registry, recipe, TOML, Source, Sink, clap, or any
  real transform (resize/crop/filter/watermark). Identity + Invert ONLY, and
  Invert exists only to make the fold testable.
- Operations must NOT express the metadata lane; just carry the input's
  metadata bundle through unchanged.
- The four gates must ALL pass before you open the PR.
```
