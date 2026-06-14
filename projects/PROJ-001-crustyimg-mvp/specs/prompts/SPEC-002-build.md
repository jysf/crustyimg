# SPEC-002 — BUILD prompt

> Paste this into a **fresh Claude session**. You are NOT the architect who
> wrote the spec. The spec file is your only context. Do not rely on any
> prior conversation.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-002 ("canonical Image type and load"). You
are NOT the architect; the spec file is your source of truth.

Read these files in order before writing any code:

1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — conventions: §5 stack (`image` 0.25, `thiserror` 2 in lib, no async),
   §6 EXACT commands, §11 coding conventions (library-first; the pixel core
   `src/image/**` must NOT depend on clap/files/terminals; typed errors; no
   unwrap/expect/panic! on recoverable paths; diagnostics to stderr), §12
   testing (native-generated fixtures — NO ImageMagick, NO committed binary
   fixtures; integration tests under tests/), §13 git/PR conventions, §15
   build-cycle rules.
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-002-canonical-image-type-and-load.md
   — the spec. Read the ENTIRE "## Implementation Context" section carefully:
   it lists the decisions, constraints, exact commands, the dependency note
   (why `image`/`thiserror` need NO new DEC), out-of-scope items, the
   recommended `image` feature set, and the metadata-capture approach.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/stages/STAGE-001-foundation-and-pipeline-core.md
   — the parent stage (this is backlog item #2).
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/brief.md
   — the project.
5. The decisions referenced by the spec:
   - /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-002-single-image-model-and-operation-trait.md
   - /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-003-metadata-dual-lane.md
   - /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-004-codec-policy-pure-rust-default.md
   - /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-007-error-handling-thiserror-anyhow.md
   (filenames may differ slightly — list the decisions dir to confirm exact names.)
6. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/constraints.yaml
   — full text of the constraints that apply to paths you'll touch.
7. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/data-model.md
   and /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/architecture.md
   — the Image/ImageInfo/MetadataBundle field tables and the `image/` module
   layering rules (the pixel core has no internal deps and no clap/I/O policy).
8. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/done/SPEC-001-cargo-project-and-multi-os-ci.md
   — what already exists (empty-deps scaffold, `version()` lib fn, smoke test,
   CI). `[dependencies]` is currently EMPTY; this spec adds the first two.

Before coding, mark the build cycle `[~]` in:
  /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-002-canonical-image-type-and-load-timeline.md

If you hit something needing architect judgment or an external unblock
(constraint unclear, a dependency beyond `image`/`thiserror` genuinely
required, scope drift into ops/source/sink/metadata-editing), change the
marker to `[?]` with a one-line reason and STOP. `[?]` is not a "don't know
what to do" dumping ground — ask if unsure.

If anything in the spec is ambiguous, add it to
  /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml
and stop (AGENTS.md §15).

Create the branch off main:
  git checkout main && git pull
  git checkout -b feat/spec-002-canonical-image-type-and-load

Implement to make the spec's "## Failing Tests" pass. The work (see the spec's
## Outputs for the precise contract):
- Cargo.toml          — add `image` and `thiserror` to [dependencies], pinned
                        to exact patch versions (AGENTS.md §5: image 0.25,
                        thiserror 2). Configure `image` to the pure-Rust
                        default feature set (DEC-004) — see the spec's "Notes
                        for the Implementer" for the recommended feature list.
                        Do NOT enable avif/mozjpeg/native features. Do NOT add
                        any OTHER crate (no tempfile, no separate EXIF parser).
- src/error.rs        — first typed library error: a thiserror `ImageError`
                        enum (Io(#[from] io::Error), Decode(...),
                        UnsupportedFormat(...)) + `pub type Result<T> =
                        std::result::Result<T, ImageError>;`. Clear #[error]
                        messages. No unwrap/expect/panic! (DEC-007).
- src/image/mod.rs    — canonical `Image` (wraps ::image::DynamicImage +
                        source_format + Option<MetadataBundle>), `ImageInfo`,
                        `MetadataBundle`, with the load/from_bytes/from_reader
                        entries, accessors, and `info()`. Capture the raw
                        EXIF/ICC bundle at load WITHOUT interpreting it
                        (DEC-003) — see the spec's metadata-capture note (scan
                        the container for the EXIF segment yourself; no new
                        dep; raw capture, not parsing). Refer to the crate as
                        `::image` to avoid the module-name collision. This
                        module must NOT touch clap/files-policy/terminals.
- src/lib.rs          — add `pub mod error;` and `pub mod image;` (keep the
                        existing version() fn + its test).
- tests/image_load.rs — the integration tests named in the spec's ## Failing
                        Tests, plus a native fixture helper (solid_png /
                        gradient_jpeg / rgba_png / an EXIF-bearing fixture)
                        that generates images in memory with the `image`
                        crate. NO ImageMagick, NO committed binary fixtures.
                        Use std::env::temp_dir() for the path-load test (no
                        tempfile crate). Match on ImageError VARIANTS, not
                        error strings.
- src/image/mod.rs unit tests — a #[cfg(test)] mod tests covering the pure
                        helpers (bit_depth/has_alpha derivation; MetadataBundle
                        predicates) so every-public-fn-tested holds.

Honor every constraint in the spec's Implementation Context. Key ones:
- single-image-library — `image` only; no second pixel crate.
- pure-rust-codecs-default — pure-Rust `image` features only; default
  `cargo build`/`cargo test` must need NO system libraries.
- no-unwrap-on-recoverable-paths — typed errors in src/image/** and
  src/error.rs; .unwrap() in tests/ and #[cfg(test)] is fine.
- no-new-top-level-deps-without-decision — `image`/`thiserror` are
  PRE-JUSTIFIED by DEC-002/DEC-004/DEC-007, so NO new DEC for them. If you
  reach for ANY other crate, STOP and write a /decisions/DEC-NNN-<slug>.md
  first (honest confidence), then add it.
- clippy-fmt-clean, every-public-fn-tested, test-before-implementation.

Run the gates locally until all green (from the repo root):
  cargo build
  cargo test
  cargo clippy -- -D warnings
  cargo fmt --check

When done:
1. Fill in the spec's "## Build Completion" section INCLUDING the three
   build-phase reflection questions (not optional, answer honestly). Note
   which `image` feature configuration you chose and how you captured the
   EXIF segment.
   NOTE (AGENTS.md §13): build-cycle edits to the spec stay limited to the
   `## Build Completion` section. Do NOT touch verify/ship bookkeeping
   (timeline marks beyond your build line, ship prompt, ship reflection, cost
   totals, stage backlog, archiving) — those land on `main` later.
2. Append a build cost session entry to the spec's `cost.sessions`:
     - cycle: build
       agent: <your model>
       interface: claude-code
       tokens_input: <best available>
       tokens_output: <best available>
       estimated_usd: <best available>
       duration_minutes: <estimate>
       recorded_at: <YYYY-MM-DD>
       notes: <one line if rework/unusual, else null>
   In Claude Code, run /cost and use its numbers; if unavailable, use null
   numeric fields with a note.
3. Run from the repo root:
     just advance-cycle SPEC-002 verify
4. Commit with a Conventional Commit (one spec per PR — constraint
   one-spec-per-pr), e.g.:
     feat(image): canonical Image type + load/decode + metadata capture (SPEC-002)
   End the commit message with:
     Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
5. Push the branch and open a GitHub PR on jysf/crustyimg (per AGENTS.md §13)
   using the gh CLI. PR title must carry the spec id, e.g.
   `feat(image): canonical Image type + load/decode (SPEC-002)`. PR body must
   follow the AGENTS.md §13 template:
     ## Summary
     - <bullets per structural change: Image type, load entries, ImageInfo,
       MetadataBundle capture, ImageError, image+thiserror deps, fixtures>
     ## Spec metadata
     - **Project:** PROJ-001
     - **Stage:** STAGE-001
     - **Spec:** SPEC-002
     ## Decisions referenced
     DEC-002 (single canonical model over image::DynamicImage), DEC-003
     (capture raw EXIF/ICC bundle at load, uninterpreted), DEC-004 (pure-Rust
     `image` codecs by default), DEC-007 (typed ImageError via thiserror)
     ## Constraints checked
     - `single-image-library` ✅ — <evidence: only `image` in deps>
     - `pure-rust-codecs-default` ✅ — <evidence: feature set / no system libs>
     - `no-unwrap-on-recoverable-paths` ✅ — <evidence>
     - `no-new-top-level-deps-without-decision` ✅ — image/thiserror
       pre-justified by DEC-002/004/007; no other dep added
     - `clippy-fmt-clean` ✅ — <evidence>
     - `every-public-fn-tested` ✅ — <evidence>
     - `test-before-implementation` ✅ — failing tests from spec made to pass
     - `one-spec-per-pr` ✅ — SPEC-002 only
     ## New decisions
     - No new DEC — `image`/`thiserror` pre-justified by DEC-002/DEC-004/DEC-007.
       (If you genuinely had to add another crate, list its DEC here instead.)
   End the PR body with the Claude Code generated-with footer.
6. Mark build `[x]` in the timeline with the PR number, cost, and date:
     - [x] **build** — prompt: `prompts/SPEC-002-build.md`
            PR #NNN, $X.XX, completed <YYYY-MM-DD>

Watch for:
- The `image`/module name collision — refer to the crate as `::image`.
- `pure-rust-codecs-default`: do NOT enable `avif` (pulls native libaom) or any
  native/mozjpeg feature; keep default build system-lib-free.
- EXIF capture is raw byte-scanning, NOT parsing (DEC-003) — and needs no new
  crate. The JPEG APP1 (`0xFF 0xE1` + `Exif\0\0`) or PNG `eXIf` chunk is enough
  to satisfy the "metadata captured when present" test; keep the fixture and
  the capture path in the same format.
- Resist scope creep: no Operation/Pipeline (SPEC-003), no Source/Sink
  (SPEC-004/005), no metadata EDITING (STAGE-004). Capture only.
- Match on `ImageError` variants in tests, not on error message strings.
- Keep `src/image/**` free of clap/process/terminal/recipe deps (layering).
```
