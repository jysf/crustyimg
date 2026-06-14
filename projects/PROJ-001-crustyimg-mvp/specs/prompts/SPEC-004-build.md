# SPEC-004 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. Do not rely on any prior conversation. This prompt is
> deliberately prescriptive — follow it literally.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-004 ("Source input abstraction"). You are NOT
the architect; the spec file is your source of truth. Use ABSOLUTE paths.

Read these files in order before writing any code:

1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — conventions: §5 stack (`image` 0.25, `thiserror` 2 in lib, NO async, NO
   new top-level runtime dep without a DEC), §6 EXACT commands (the four
   gates), §11 coding conventions (library-first; `src/source/**` must NOT
   depend on clap/recipes/sinks/terminals; typed errors; NO unwrap/expect/
   panic! on recoverable paths; diagnostics to stderr never stdout; group
   imports std/external/local), §12 testing (unit in #[cfg(test)], integration
   under tests/, native fixtures — NO ImageMagick, NO committed binary
   fixtures), §13 git/PR (branch, conventional commits, PR body template),
   §15 build-cycle rules (spec edits LIMITED to ## Build Completion).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-004-source-input-abstraction.md
   — THE SPEC. Implement its "## Failing Tests" and "## Outputs" exactly. Read
   "## Implementation Context" and "## Notes for the Implementer" in full;
   they are written for you.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-010-source-crate-glob.md
   — the dependency decision: add `glob` 0.3 ONLY; directories use
   std::fs::read_dir, NON-RECURSIVE; do NOT add walkdir.
   Also read:
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-002-single-image-model-and-operation-trait.md
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-007-error-handling-thiserror-anyhow.md
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-004-codec-policy-pure-rust-default.md
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/constraints.yaml
   — full text of the constraints that apply: untrusted-input-hardening,
   no-unwrap-on-recoverable-paths, no-new-top-level-deps-without-decision,
   single-image-library, clippy-fmt-clean, every-public-fn-tested,
   test-before-implementation.
5. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/architecture.md
   (§ Components → Source; § Module/Layer Structure — layering `source → image`;
   Source has no clap/recipe/sink/terminal deps) and
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/data-model.md
   (§ "Name templates (Sink)" — why each Input carries a `stem`) and
   /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/api-contract.md
   (§ "stdin / stdout (`-`)").
6. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/image/mod.rs
   and /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/error.rs
   and /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/lib.rs
   — what prior specs shipped: `Image::load/from_bytes/from_reader` (you do NOT
   call these — Source only yields the inputs they consume), the `ImageError` +
   `Result` typed-error pattern to MIRROR for `SourceError`, and the current
   module declarations.
7. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/Cargo.toml
   — the existing `=`-pinned dependency style to match.

Before coding, mark the build cycle `[~]` in:
  /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-004-source-input-abstraction-timeline.md

If you hit something needing architect judgment or an external unblock
(constraint unclear; a dependency beyond `glob` + the `tempfile` dev-dep
genuinely seems required; scope drift into Sink/Recipe/registry/CLI/clap/rayon,
decode limits, or recursive directory walking), change the marker to `[?]` with
a one-line reason and STOP. `[?]` is not a "don't know what to do" dumping
ground — ask if unsure by adding to:
  /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml

Create the branch off the LATEST main (sync first):
  git checkout main && git pull
  git checkout -b feat/spec-004-source-input-abstraction

Implement to make the spec's "## Failing Tests" pass. The exact work:

A. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/Cargo.toml
   — add to [dependencies]:  glob = "=0.3.x"  (pin the EXACT latest 0.3 patch;
     match the existing `=`-pin style). This is justified by DEC-010 — NO new
     DEC needed.
   — add a [dev-dependencies] section with:  tempfile = "<latest 3.x>"  (pin a
     version). tempfile is TEST-ONLY; it is NOT a top-level runtime dep, so the
     no-new-top-level-deps-without-decision constraint does not require a DEC —
     but NOTE it in Build Completion. Add NOTHING ELSE (no walkdir, rayon,
     serde, clap).

B. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/source/mod.rs
   (NEW) — define, EXACTLY per the spec's Outputs:
     - `pub enum Input { Path(PathBuf), Stdin { bytes: Vec<u8>, stem: String } }`
       deriving Debug/Clone/PartialEq/Eq, with:
         `pub fn stem(&self) -> &str`  (Path: file_stem→to_str, default "" via
            unwrap_or — NOT a panic; Stdin: the stored stem field)
         `pub fn path(&self) -> Option<&Path>`  (Some for Path, None for Stdin)
     - `pub enum SourceError` via thiserror::Error with variants:
         `NotFound(String)` — missing single path / empty glob match
         `InvalidPattern { pattern: String, reason: String }` — bad glob syntax
         `Stdin(#[from] std::io::Error)` — stdin read failed
       Mirror src/error.rs ImageError style. Keep SourceError LOCAL to this
       module (like OperationError lives in src/operation/); do NOT widen the
       crate Error.
     - `pub fn looks_like_glob(arg: &str) -> bool` — true iff `arg` contains
       any of `*`, `?`, `[`.
     - `pub fn resolve(arg: &str, reader: &mut impl std::io::Read)
          -> Result<Vec<Input>, SourceError>` — dispatch IN THIS ORDER:
         1. arg == "-"  -> read reader to_end into a Vec<u8> (the `?` maps io
            error to SourceError::Stdin via #[from]); return one
            Input::Stdin { bytes, stem: "stdin".into() }.
         2. looks_like_glob(arg)  -> glob branch (below).
         3. Path::new(arg).is_dir()  -> directory branch (below).
         4. else single-file branch: if Path::new(arg) exists, return one
            Input::Path; else Err(SourceError::NotFound(arg.into())). A directly
            named single file is NOT extension-filtered.
     - `#[cfg(test)] mod tests` with the UNIT tests named in the spec.

   GLOB branch:
     - `glob::glob(arg)` -> map the PatternError to
       SourceError::InvalidPattern { pattern: arg.into(), reason: e.to_string() }.
     - Iterate the Paths; for each `Ok(path)` keep it, for each `Err(_)`
       (GlobError) SKIP it. Then filter to image extensions (allow-list below)
       and run the SYMLINK-ESCAPE check relative to the pattern's base dir —
       see the symlink rule. Sort the survivors. If EMPTY ->
       Err(SourceError::NotFound(arg.into())).

   DIRECTORY branch (NON-RECURSIVE):
     - `let root = std::fs::canonicalize(arg)` — map an io error to
       SourceError::NotFound(arg.into()) (do NOT use the #[from] Stdin variant).
     - `std::fs::read_dir(&root)` — same NotFound mapping on the open error.
     - For each entry (each read_dir item is a Result — on an Err entry, SKIP):
         * compute entry_path
         * if entry_path.is_dir() (after the symlink check below resolves to a
           dir) -> SKIP (non-recursive; top-level files only)
         * SYMLINK-ESCAPE: `let Ok(real) = std::fs::canonicalize(&entry_path)
           else { continue; };` then keep ONLY IF `real.starts_with(&root)`;
           otherwise SKIP. (canonicalize follows symlinks, so a symlink that
           points outside `root` resolves outside and is dropped.)
         * EXTENSION filter: keep only files whose extension (case-insensitive)
           is in the allow-list { jpg, jpeg, png, gif, bmp, tif, tiff, ico };
           SKIP everything else.
     - Collect surviving paths, `paths.sort()`, return as Vec<Input::Path>. An
       empty result is OK (returns an empty Vec) — do NOT error on an empty
       directory; only a MISSING directory errors (handled by canonicalize).

   IMAGE EXTENSION allow-list (case-insensitive): jpg, jpeg, png, gif, bmp,
   tif, tiff, ico. Apply it to directory listings AND to glob matches (so a
   `*` glob and a directory behave consistently). Do NOT apply it to a directly
   named single file.

   Refer ONLY to std::path + the `glob` crate here. Source does NOT use the
   `image` crate at all (no decoding) — so no `::image` aliasing is needed in
   library code.

C. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/src/lib.rs
   — add `pub mod source;` (keep existing modules + version() + its test).

D. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/tests/source.rs
   (NEW) — the integration tests named in the spec, through the crate's PUBLIC
   exports only (`crustyimg::source::{resolve, looks_like_glob, Input,
   SourceError}`). Use `tempfile::TempDir`/`tempfile::tempdir()` for fixtures;
   create files/dirs/symlinks INSIDE the test. Produce a real tiny PNG by
   encoding a solid image in-memory (mirror the `solid_png` helper in
   src/image/mod.rs tests: `image::RgbImage::from_pixel(w,h,Rgb([..]))` ->
   `DynamicImage::write_to(&mut Cursor, ImageFormat::Png)` -> write bytes to a
   temp path). `.unwrap()` is fine in tests. The exact test names:
     - single_file_resolves_to_one_input
     - glob_returns_sorted_matches_excluding_nonmatches
     - directory_lists_top_level_images_sorted_non_recursive
     - directory_skips_symlink_escaping_root  (gate symlink creation on
       #[cfg(unix)] via std::os::unix::fs::symlink; on Windows either use
       std::os::windows::fs::symlink_file and skip-with-eprintln if it fails
       for lack of privilege, OR cfg it out entirely — document which in Build
       Completion)
     - nonimage_entries_are_skipped_not_errored
     - resolution_order_is_deterministic
     - missing_single_file_is_not_found_error
     - empty_glob_match_is_not_found_error
     - invalid_glob_pattern_is_typed_error
   For every NON-stdin call, pass `&mut std::io::empty()` as the reader. Compare
   ordered results by file STEM or file NAME (not absolute temp paths). Match
   error VARIANTS (e.g. `matches!(err, SourceError::NotFound(_))`), not strings.

HARD RULES (do not violate):
- untrusted-input-hardening: directory/glob enumeration MUST NOT follow a
  symlink whose canonicalized target escapes the canonicalized root — skip it.
  Compare `starts_with` on CANONICALIZED paths on BOTH sides.
- no-new-top-level-deps-without-decision: `glob` is the ONLY new RUNTIME dep and
  is pre-justified by DEC-010 — NO new DEC. `tempfile` is a dev-dep (test-only),
  no DEC, but NOTE it. If you think you need ANY other crate (walkdir, rayon,
  serde, clap, a logging crate), STOP, add to questions.yaml, mark `[?]`. Do NOT
  add it.
- single-image-library: Source uses NO pixel library — paths + bytes + glob
  only. Image-ness is decided by file EXTENSION, NEVER by decoding. Do NOT call
  Image::load/from_bytes from src/source/.
- no-unwrap-on-recoverable-paths: typed SourceError in library code;
  .unwrap()/.expect() ONLY in #[cfg(test)] and tests/. resolve() must NEVER
  panic — a dangling symlink / unreadable entry is a SKIP, a missing named path
  / empty glob is a typed Err.
- Non-recursive directories ONLY. Top-level files; SKIP subdirectories. No
  walkdir, no recursion, no --recursive flag.
- Resist scope creep: NO Sink, name-template expansion, --out-dir, stdout,
  viuer, Recipe, registry, clap, rayon, decode limits (image::Limits), or
  cargo-audit. Those are SPEC-005/006/007, STAGE-005, STAGE-006.

Run the four gates locally until ALL green (from the repo root, absolute):
  cargo build
  cargo test
  cargo clippy -- -D warnings
  cargo fmt --check

When done:
1. Fill in the spec's "## Build Completion" section INCLUDING: the branch, PR,
   "all acceptance criteria met?", whether `stem()` stayed `&str` or relaxed to
   `String` (and why), whether you yielded original or canonical entry paths,
   how you wired the stdin reader, the platform handling of the symlink test
   (unix-gated / windows-skipped), the tempfile dev-dep note, deviations,
   follow-ups (e.g. a future --recursive + walkdir spec under a new DEC), and
   the THREE build-phase reflection questions (answer honestly). AGENTS.md
   §13/§15: build-cycle edits to the spec stay LIMITED to `## Build Completion`.
   Do NOT touch verify/ship bookkeeping (timeline marks beyond your build line,
   ship prompt, ship reflection, cost totals, stage backlog, archiving) —
   those land on `main` later by the orchestrator.
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
3. Advance the cycle to verify. Run from the repo root:
     just advance-cycle SPEC-004 verify
   NOTE: `just advance-cycle` MIS-GLOBS (it can when multiple SPEC-004* files
   exist — the spec + its timeline). Do NOT fight it — instead set
   `task.cycle: verify` BY HAND in the front-matter of:
     /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-004-source-input-abstraction.md
   and verify with a quick grep that only the spec (not the timeline) changed.
4. Create DEC-* files only for NON-TRIVIAL build decisions. DEC-010 already
   exists (the glob dependency) — do NOT duplicate it. The tempfile dev-dep and
   the extension allow-list do NOT warrant a DEC. Most likely outcome: "No new
   DEC".
5. Commit with a Conventional Commit (one spec per PR — constraint
   one-spec-per-pr), e.g.:
     feat(source): resolve file/glob/dir/stdin into ordered inputs (SPEC-004)
   End the commit message with:
     Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
6. Push the branch and open a GitHub PR on jysf/crustyimg (per AGENTS.md §13)
   using the gh CLI. PR title must carry the spec id, e.g.
   `feat(source): file/glob/dir/stdin input resolution (SPEC-004)`. PR body
   must follow the AGENTS.md §13 template:
     ## Summary
     - <bullets: Input enum (Path/Stdin) + stem()/path(); SourceError;
       looks_like_glob; resolve() dispatch; glob expansion via `glob`;
       non-recursive directory listing via std::fs; symlink-escape skip;
       extension allow-list; module wiring; unit + integration tests>
     ## Spec metadata
     - **Project:** PROJ-001
     - **Stage:** STAGE-001
     - **Spec:** SPEC-004
     ## Decisions referenced
     DEC-010 (glob for patterns + std::fs non-recursive dirs, no walkdir),
     DEC-002 (inputs load into the canonical Image; Source does not decode),
     DEC-007 (typed SourceError via thiserror; no panic on recoverable paths),
     DEC-004 (glob is pure-Rust; no system deps)
     ## Constraints checked
     - `untrusted-input-hardening` ✅ — <evidence: symlink-escape entries
       skipped via canonicalize + starts_with(root); test
       directory_skips_symlink_escaping_root>
     - `no-unwrap-on-recoverable-paths` ✅ — <evidence: resolve returns typed
       SourceError; bad entries skipped, never panic>
     - `no-new-top-level-deps-without-decision` ✅ — <evidence: glob justified
       by DEC-010; tempfile is a dev-dep>
     - `single-image-library` ✅ — <evidence: no pixel lib in source/;
       image-ness by extension, not decode>
     - `clippy-fmt-clean` ✅ — <evidence>
     - `every-public-fn-tested` ✅ — <evidence: resolve/looks_like_glob/
       Input::stem/Input::path each tested>
     - `test-before-implementation` ✅ — failing tests from spec made to pass
     - `one-spec-per-pr` ✅ — SPEC-004 only
     ## New decisions
     - No new DEC — DEC-010 (glob) written at design time; tempfile is a dev-dep
       and the extension allow-list is spec-mandated.
   End the PR body with the Claude Code generated-with footer.
7. Mark build `[x]` in the timeline with the PR number, cost, and date:
     - [x] **build** — prompt: `prompts/SPEC-004-build.md`
            PR #NNN, $X.XX, completed <YYYY-MM-DD>

Watch for (Sonnet-specific reminders):
- SYMLINK CANONICALIZATION is the trickiest bit. canonicalize() ERRORS on a
  dangling symlink or a path that doesn't exist — that error is a SKIP
  (`let Ok(real) = canonicalize(&p) else { continue; };`), NOT a propagate.
  Canonicalize BOTH the root and each entry, then `real.starts_with(&root)`.
  On Windows, canonicalize adds a `\\?\` verbatim prefix to BOTH sides, so
  starts_with still works — but NEVER compare a raw arg against a canonical
  path.
- DETERMINISTIC SORT: read_dir order is OS-arbitrary. Collect into a Vec and
  `.sort()` (PathBuf Ord is lexicographic) AFTER the symlink + extension
  filters, so order is over the surviving set. Tests assert sorted order.
- STDIN: stdin is NOT seekable, so do NOT route it through Image::from_reader
  (its Seek bound). Source reads the `reader` to a Vec<u8> and yields
  Input::Stdin { bytes, .. }; a later caller uses Image::from_bytes(&bytes).
  In tests pass a byte slice (`&mut &b"..."[..]`) or `std::io::empty()`.
- WINDOWS PATH QUIRKS: use std::path APIs (file_stem, extension, starts_with) —
  do NOT hand-split on `/`. Extension comparison must be case-insensitive
  (`eq_ignore_ascii_case`). The symlink integration test must be cfg-gated for
  Unix (std::os::unix::fs::symlink) and skipped/handled on Windows where
  symlink creation needs privilege.
- Do NOT add walkdir (DEC-010 reserves it for a future recursive flag), and do
  NOT decode images to test image-ness — extension allow-list only.
- A glob/directory that yields ZERO valid entries: glob -> NotFound (the user's
  pattern produced nothing); an existing-but-empty directory -> empty Vec (NOT
  an error). A MISSING named single file -> NotFound. Get these three cases
  right; the tests pin them.
- The four gates must ALL pass before you open the PR.
```
