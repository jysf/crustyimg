---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-004
  type: story                      # epic | story | task | bug | chore
  cycle: verify                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet 4.6 (separate session)
  created_at: 2026-06-14

references:
  decisions:
    - DEC-002                       # canonical Image the inputs load into (decode-once)
    - DEC-007                       # thiserror in lib; typed errors; no panic on recoverable paths
    - DEC-004                       # pure-Rust default; `glob` is pure-Rust, no system deps
    - DEC-010                       # NEW: `glob` for patterns + std::fs for non-recursive dirs (no walkdir)
  constraints:
    - untrusted-input-hardening     # dir/glob must not escape via symlinks; skip bad entries; typed errors
    - no-unwrap-on-recoverable-paths
    - no-new-top-level-deps-without-decision   # `glob` is justified by DEC-010 (this spec emits it)
    - single-image-library
    - clippy-fmt-clean
    - every-public-fn-tested
    - test-before-implementation
  related_specs:
    - SPEC-002                      # Image::from_bytes / from_reader — Source yields bytes/paths; Image decodes
    - SPEC-005                      # Sink consumes the ordered inputs + their stems (out of scope here)
    - SPEC-003                      # Pipeline runs per input (out of scope here; Source just enumerates)

# One sentence on what this spec contributes to its stage's
# value_contribution.
value_link: "infrastructure: the Source abstraction that turns one CLI argument into the ordered, deterministic, safe list of inputs every later command and batch run iterates over"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Null numeric fields are fine (e.g. claude.ai web sessions); reports
# skip them in sums but count them in session_count. Examples of
# interface: claude-code | claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_input: null
      tokens_output: null
      estimated_usd: null
      duration_minutes: 45
      recorded_at: 2026-06-14
      notes: "subagent; cost not separately reported"
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_input: null
      tokens_output: null
      estimated_usd: null
      duration_minutes: 25
      recorded_at: 2026-06-14
      notes: "subagent; cost not separately reported"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 2
---

# SPEC-004: Source input abstraction

## Context

This is backlog item #4 of `STAGE-001` (foundation and pipeline core) in
`PROJ-001` (crustyimg MVP). Three specs have shipped: SPEC-001 (Cargo project +
multi-OS CI), SPEC-002 (the canonical `Image` type + `load`/`from_bytes`/
`from_reader` + `MetadataBundle`), and SPEC-003 (the `Operation` trait +
decode-once `Pipeline`). The pipeline can transform *an* image; nothing yet
turns a command-line argument into the *set* of images to run it over.

`Source` is that front door. Per `docs/architecture.md` § "Components", a
`Source` "resolves a CLI argument into an ordered list of inputs — a single
path yields one; a glob or directory yields many; `-` yields one stdin stream."
This is where batch fan-out *originates*: STAGE-005's `apply` command will drive
the resulting list in parallel with `rayon`, but the abstraction — enumerating
inputs safely and deterministically — lands here in STAGE-001.

Two properties matter for the rest of the project:

- **Deterministic order.** Batch output must be reproducible run-to-run, so a
  glob/directory must yield its matches in a stable sorted order, not the OS's
  arbitrary directory order.
- **Safe enumeration of untrusted paths.** Directory and glob resolution walk
  the filesystem. The `untrusted-input-hardening` constraint (added 2026-06-14)
  requires that **directory sources do not follow symlinks out of the tree** —
  a symlink inside a target directory that points outside it must not silently
  pull an external file into the batch. The deeper traversal audit + decode
  limits are STAGE-006; the *enumeration* basics are in scope here.

Each yielded input must carry enough to (a) **load** it — a path for files, or
the captured bytes for stdin — and (b) **name its output later** — a `stem` for
SPEC-005's name templates (`{stem}_web.{ext}`). Source does **not** decode:
decoding is SPEC-002's job (`Image::load` / `Image::from_bytes`). Source yields
a description of *what to load* and *what to call its output*.

## Goal

Implement `src/source/` so a single CLI input argument resolves into an ordered,
deterministic `Vec<Input>`: a single file → one input; a glob pattern → its
sorted matches; a directory → its top-level (non-recursive) image files, sorted;
and `-` → one stdin input carrying the bytes read from stdin plus a synthetic
stem. Resolution must skip unreadable/non-image entries gracefully, must not
follow symlinks that escape the resolved root, and must surface a missing or
invalid argument as a typed error — never a panic.

## Inputs

- **Files to read:**
  - `src/image/mod.rs` — the `Image` type. Note `Image::load(impl AsRef<Path>)`,
    `Image::from_bytes(&[u8])`, and `Image::from_reader<R: Read + Seek>` already
    exist. Source produces the *inputs* these consume; it does not call them.
  - `src/error.rs` — the existing `ImageError` + `pub type Result<T>` pattern to
    mirror when defining `SourceError`.
  - `src/lib.rs` — module declarations; you add `pub mod source;`.
  - `docs/architecture.md` § "Components" (Source) and § "Module / Layer
    Structure" (layering: `source → image`; Source must not depend on clap,
    recipes, sinks, or terminals).
  - `docs/data-model.md` § "Name templates (Sink)" — why each input carries a
    `stem` (`{stem}` token) and the `{name}`/`{parent}` tokens Sink will read.
  - `docs/api-contract.md` § "stdin / stdout (`-`)" — `-` reads an encoded image
    from stdin; the synthetic stem convention for stdin output naming.
- **External APIs:** none at runtime (no network). One new crate: `glob` 0.3
  (justified by DEC-010, written as part of this design — see Outputs).
- **Related code paths:** `src/image/`, `src/error.rs`, `src/lib.rs`.

## Outputs

- **Files created:**
  - `src/source/mod.rs` — the `Input` enum, the `Source` resolver, `SourceError`
    (thiserror), and a `#[cfg(test)] mod tests` with the unit tests below.
  - `tests/source.rs` — integration tests exercising the public Source API with
    `tempfile`-built fixtures (temp files/dirs/symlinks created in the test; **no
    committed fixtures**).
- **Files modified:**
  - `src/lib.rs` — add `pub mod source;`.
  - `Cargo.toml` — add `glob = "=0.3.x"` to `[dependencies]` (pin the exact patch
    available; see the existing `=`-pinned style) and add `tempfile` to
    `[dev-dependencies]` (test-only; dev-deps do **not** require a DEC under
    `no-new-top-level-deps-without-decision`, which scopes to top-level runtime
    deps — note this explicitly in Build Completion).
- **New exports (exact signatures the build must produce):**

  ```rust
  // src/source/mod.rs

  use std::path::{Path, PathBuf};

  /// One resolved input the pipeline will process. Carries enough to (a) load
  /// it (SPEC-002's Image::load / from_bytes) and (b) name its output later
  /// (SPEC-005's name templates, via `stem()`).
  ///
  /// Source does NOT decode — it describes what to load and what to call it.
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum Input {
      /// A file on disk. The pipeline loads it with `Image::load(path)`.
      Path(PathBuf),
      /// Bytes read from stdin (the `-` argument). The pipeline loads them with
      /// `Image::from_bytes(&bytes)`. `stem` is the synthetic output name
      /// (default "stdin").
      Stdin { bytes: Vec<u8>, stem: String },
  }

  impl Input {
      /// The output stem for name templates (`{stem}`): a file input's file
      /// stem (filename without extension), or the stdin input's synthetic
      /// stem. Never contains a path separator.
      pub fn stem(&self) -> &str;

      /// The path, for `Input::Path`; `None` for stdin. Convenience for callers
      /// that want to log or load by path.
      pub fn path(&self) -> Option<&Path>;
  }

  /// Errors resolving a CLI input argument into inputs (DEC-007).
  #[derive(Debug, thiserror::Error)]
  pub enum SourceError {
      /// The argument named a path that does not exist / is unreadable, or a
      /// glob that matched nothing.
      #[error("input not found or unreadable: {0}")]
      NotFound(String),

      /// The glob pattern itself was syntactically invalid.
      #[error("invalid glob pattern '{pattern}': {reason}")]
      InvalidPattern { pattern: String, reason: String },

      /// Reading stdin failed.
      #[error("could not read image from stdin")]
      Stdin(#[from] std::io::Error),
  }

  /// Resolve a single CLI input argument into an ordered list of inputs.
  ///
  /// Dispatch (decided BEFORE touching the filesystem, to keep behavior
  /// predictable):
  ///   - `"-"`                  -> read stdin via `reader`, yield one `Stdin`.
  ///   - contains a glob metachar (`*`, `?`, `[`) -> treat as a glob pattern.
  ///   - an existing directory  -> non-recursive listing of image files.
  ///   - any other string       -> a single file path (existence checked).
  ///
  /// `reader` is injected (`&mut impl Read`) so tests can feed bytes without a
  /// real stdin; production passes `std::io::stdin().lock()`.
  ///
  /// Order is deterministic: glob and directory results are sorted by path
  /// (lexicographic on the full path). Unreadable / non-image entries inside a
  /// glob or directory are SKIPPED (a `--verbose` warning to stderr is allowed
  /// but not required by tests); a missing single file, an empty glob match, or
  /// an invalid pattern is a typed `SourceError`.
  pub fn resolve(arg: &str, reader: &mut impl std::io::Read) -> Result<Vec<Input>, SourceError>;

  /// Whether a string looks like a glob pattern (contains `*`, `?`, or `[`).
  /// Pulled out as a free fn so it is directly unit-testable.
  pub fn looks_like_glob(arg: &str) -> bool;
  ```

  > The `resolve` signature takes a `reader` so stdin is testable without a real
  > tty/pipe. `resolve("-", &mut some_bytes_slice)` reads from the slice. For
  > the `-` case only the `reader` is consumed; for every other case `reader`
  > is ignored.

- **Database changes:** none.

## Acceptance Criteria

Each maps to at least one failing test below.

- [ ] **Single file → one input.** `resolve("<existing-file>")` returns exactly
  one `Input::Path` whose path is that file and whose `stem()` is the filename
  without extension.
- [ ] **Glob → sorted matches.** `resolve("<dir>/*.png")` over a directory
  containing several `.png` files (created out of alphabetical order) returns
  one `Input::Path` per match, **sorted lexicographically by path**, and
  excludes files that don't match the pattern (e.g. a sibling `.txt`).
- [ ] **Directory → sorted image files, non-recursive.** `resolve("<dir>")` over
  a directory containing image files plus a non-image file plus a nested
  subdirectory (with its own image) returns one `Input::Path` per **top-level
  image file only**, sorted by path: the non-image file is skipped and the
  nested subdirectory's image is **not** included (non-recursive).
- [ ] **`-` → one stdin input.** `resolve("-", &mut <bytes>)` returns exactly one
  `Input::Stdin` whose `bytes` equal the bytes read from the reader and whose
  `stem()` is the synthetic `"stdin"`.
- [ ] **Symlink escaping the root is not followed.** Given a directory `root/`
  containing a symlink that points to a file *outside* `root/`,
  `resolve("root")` does **not** include that external file. (On platforms
  where symlink creation in the test is unavailable — e.g. Windows without
  privilege — the test is skipped, documented in Build Completion.)
- [ ] **Non-image / unreadable entries handled gracefully.** A directory or glob
  containing a `.txt` (and/or a file with an image-ish extension but garbage
  bytes) yields the valid image inputs and silently skips the rest; resolution
  does **not** error and does **not** panic. (Source decides image-ness by
  extension allow-list, NOT by decoding — decoding is SPEC-002's job; a
  truly-corrupt image with a valid extension is still yielded as an `Input` and
  fails later at load.)
- [ ] **Deterministic order across runs.** Resolving the same glob/directory
  twice yields byte-identical ordered `Vec<Input>` (sorted, stable).
- [ ] **Missing path → typed error, not panic.** `resolve("<does-not-exist>")`
  (a plain path, no glob metachars) returns `Err(SourceError::NotFound(_))`.
  An empty glob match returns `Err(SourceError::NotFound(_))`. An invalid glob
  pattern returns `Err(SourceError::InvalidPattern { .. })`. None of these
  panic.
- [ ] **`Input::stem()` / `path()` behave.** `stem()` never contains a path
  separator; `path()` is `Some` for `Path`, `None` for `Stdin`.
- [ ] **`looks_like_glob` classifies.** `"a/*.jpg"` and `"a/file?.png"` and
  `"set[12].png"` are globs; `"a/file.jpg"`, `"-"`, and `"dir"` are not.
- [ ] **Gates green:** `cargo build`, `cargo test`, `cargo clippy -- -D warnings`,
  `cargo fmt --check` all pass.

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in **build** is
to make these pass. Build fixtures with `tempfile` (a dev-dependency) — create
temp dirs/files/symlinks *inside the test*; do **not** commit binary fixtures.
`.unwrap()` is fine inside `#[cfg(test)]` and `tests/`. To produce a real (tiny)
image file, encode a solid image in-memory and write the bytes to a temp path
(mirror the `solid_png` helper already in `src/image/mod.rs` tests).

- **`src/source/mod.rs` → `#[cfg(test)] mod tests`** (unit):
  - `"looks_like_glob_classifies_patterns"` — `looks_like_glob("a/*.jpg")`,
    `looks_like_glob("f?.png")`, `looks_like_glob("s[12].png")` are `true`;
    `looks_like_glob("a/file.jpg")`, `looks_like_glob("-")`,
    `looks_like_glob("dir")` are `false`.
  - `"input_stem_for_path"` — `Input::Path("/a/b/photo.JPG".into()).stem()` is
    `"photo"` and contains no `/`.
  - `"input_stem_and_path_for_stdin"` — an `Input::Stdin { bytes, stem:
    "stdin".into() }` has `stem() == "stdin"` and `path() == None`.
  - `"resolve_stdin_yields_one_input_with_bytes"` — `resolve("-", &mut
    &b"\x89PNG\r\n"[..])` (any byte slice) returns one `Input::Stdin` whose
    `bytes` equal the slice and whose `stem()` is `"stdin"`.

- **`tests/source.rs`** (integration, public API, `tempfile` fixtures):
  - `"single_file_resolves_to_one_input"` — write one PNG to a `TempDir`;
    `resolve(path, &mut std::io::empty())` returns one `Input::Path` at that
    path with the expected stem.
  - `"glob_returns_sorted_matches_excluding_nonmatches"` — write `c.png`,
    `a.png`, `b.png`, and `note.txt` into a `TempDir`; `resolve("<dir>/*.png")`
    returns exactly `[a.png, b.png, c.png]` as `Input::Path` in that sorted
    order; `note.txt` is absent.
  - `"directory_lists_top_level_images_sorted_non_recursive"` — write `2.png`,
    `1.png`, a `readme.txt`, and a subdirectory `sub/` containing `deep.png`;
    `resolve("<dir>")` returns exactly `[1.png, 2.png]` (sorted), excludes
    `readme.txt`, and excludes `sub/deep.png` (non-recursive).
  - `"directory_skips_symlink_escaping_root"` — create `outside/secret.png`
    OUTSIDE the target dir, create `root/` with one real `inside.png` and a
    symlink `root/link.png -> ../outside/secret.png`; `resolve("root")` returns
    only `inside.png` (the escaping symlink target is NOT yielded). Gate the
    symlink creation on `#[cfg(unix)]` (or detect Windows symlink-privilege
    failure and skip with an explanatory `eprintln!` + early return); document
    the platform handling in Build Completion.
  - `"nonimage_entries_are_skipped_not_errored"` — a directory with one valid
    `ok.png` and one `data.txt` and one `weird.bin`; `resolve("<dir>")` returns
    only `ok.png` and does NOT return an error.
  - `"resolution_order_is_deterministic"` — resolve the same glob twice; assert
    the two `Vec<Input>` are equal.
  - `"missing_single_file_is_not_found_error"` — `resolve("<tmp>/nope.png", &mut
    std::io::empty())` returns `Err(SourceError::NotFound(_))` (match the
    variant), no panic.
  - `"empty_glob_match_is_not_found_error"` — `resolve("<tmp>/*.png")` over an
    empty dir returns `Err(SourceError::NotFound(_))`.
  - `"invalid_glob_pattern_is_typed_error"` — a pattern `glob` rejects (e.g.
    `"a/[".to_string()` — note: it must still pass `looks_like_glob`, which `[`
    does) returns `Err(SourceError::InvalidPattern { .. })`, no panic.

  > For the integration tests, pass `&mut std::io::empty()` as the `reader` for
  > every non-stdin case; only the `"-"` cases consume it. Compare ordered
  > results by collecting the resolved paths into a `Vec<PathBuf>` (via
  > `Input::path()`) and asserting against the expected sorted vector, OR
  > compare file stems — whichever is clearer. Because temp dirs are
  > themselves under the OS temp root, comparisons should be on the file
  > names/stems, not absolute paths.

## Implementation Context

*Read this section (and the files it points to) before starting the build
cycle. It is the equivalent of a handoff document, folded into the spec since
there is no separate receiving agent. This build runs on **Sonnet 4.6** — the
section is deliberately prescriptive.*

### Decisions that apply

- `DEC-010` (**emitted with this spec** — read it at
  `decisions/DEC-010-source-crate-glob.md`) — Add exactly ONE new top-level
  dep: **`glob` 0.3** for glob-pattern expansion. Directories use
  `std::fs::read_dir`, **non-recursive**. Do **NOT** add `walkdir` (it is
  pre-named in architecture but reserved for a future `--recursive` flag). This
  is the dependency decision; you do not need to write a DEC — it already
  exists.
- `DEC-002` — The inputs Source yields are loaded into the one canonical
  `Image` (decode-once). Source must NOT decode; it yields paths/bytes that
  `Image::load`/`from_bytes` consume later. Keep `source/` out of the pixel core.
- `DEC-007` — Library returns typed `thiserror` enums; no
  `unwrap`/`expect`/`panic!` on recoverable paths. `SourceError` mirrors the
  existing `ImageError` style in `src/error.rs`. No `anyhow` in the library.
- `DEC-004` — Pure-Rust default, no system deps. `glob` is pure-Rust, so this
  holds. Do not add anything native.

### Constraints that apply

(See `/guidance/constraints.yaml` for full text.)

- `untrusted-input-hardening` (**warning, but central here**) — "directory
  sources do not follow symlinks out of the tree." Concretely: when listing a
  directory (or expanding a glob whose matches live under a root), for each
  candidate entry, resolve it and confirm it stays within the resolved root;
  **skip any entry whose real target escapes the root.** The recommended
  mechanism is below ("Symlink rule"). Surface failures as typed errors, never
  panics. The deeper audit + decode limits are STAGE-006 — do NOT set
  `image::Limits` here (that is the load path, SPEC-002/STAGE-006).
- `no-unwrap-on-recoverable-paths` (**blocking**) — Typed errors in library
  code; `.unwrap()`/`.expect()` only inside `#[cfg(test)]` / `tests/`. Every
  `fs` call returns a `Result` you must handle (map I/O errors on a *single
  named path* to `SourceError::NotFound`; map I/O errors while *iterating* a
  directory to a skip, not a hard error — see "Skip vs error" below).
- `no-new-top-level-deps-without-decision` (**warning**) — Satisfied by DEC-010
  for `glob`. `tempfile` is a **dev-dependency** (test-only) and is not a
  top-level runtime dep, so it does not require a DEC; still, call it out in
  Build Completion. Add NO other crate. If you think you need one, STOP, add to
  `questions.yaml`, mark the timeline `[?]`.
- `single-image-library` (**blocking**) — Source touches no pixel library at
  all; it deals in paths and bytes. (The image-ness check is by file extension,
  not by decoding — see below.)
- `clippy-fmt-clean` (**blocking**) — `cargo clippy -- -D warnings` and
  `cargo fmt --check` pass; no dead code. Group imports std / external (`glob`)
  / local (`crate::`).
- `every-public-fn-tested` (**warning**) — `resolve`, `looks_like_glob`,
  `Input::stem`, `Input::path` each have a test; the Failing Tests cover them.
- `test-before-implementation` (**blocking**) — The Failing Tests above are the
  contract; make them pass. Do not weaken or delete them to go green.

### Symlink rule (the security-critical bit — implement carefully)

For **directory** resolution (and for glob matches), do NOT follow a symlink
that points outside the resolved root:

1. Canonicalize the root once: `let root = std::fs::canonicalize(dir)?;` (maps a
   missing dir to `SourceError::NotFound` — map the `io::Error` accordingly,
   do NOT use the `#[from]` Stdin variant for this).
2. For each entry from `read_dir`, resolve its **real** location:
   `let Ok(real) = std::fs::canonicalize(&entry_path) else { continue; };` —
   if canonicalize fails (dangling symlink, permission), **skip** the entry
   rather than erroring. `canonicalize` follows symlinks, so `real` is the true
   target.
3. Keep the entry only if `real.starts_with(&root)`. If `real` escapes `root`,
   **skip it** (this is the "don't follow symlinks out of the tree" rule). A
   regular file inside the dir canonicalizes to under `root` and passes; a
   symlink to `../outside/x` canonicalizes outside `root` and is dropped.
4. Yield the **original** entry path (the one under the directory the user
   named), not the canonicalized one, so output naming/stems stay intuitive —
   *unless you prefer to yield canonical paths; either is acceptable as long as
   the escaping entry is excluded and the stem is the on-disk filename*.
   Document the choice in Build Completion.

> Windows note: `std::fs::canonicalize` returns a `\\?\` verbatim prefix on
> Windows; `starts_with` still works because both sides are canonicalized the
> same way. Always do the `starts_with` comparison on **canonicalized** paths on
> both sides — never compare a raw user path against a canonical one.

### Skip vs error (graceful degradation)

- A **named single path** that doesn't exist → `SourceError::NotFound` (hard
  error; the user asked for that one file).
- A **glob that matches nothing** → `SourceError::NotFound` (the user's pattern
  produced no inputs; better to tell them than to silently do nothing).
- An **invalid glob pattern** (syntax) → `SourceError::InvalidPattern`.
- An entry **inside** a directory/glob result that is unreadable, a non-image
  extension, or an escaping symlink → **skip it** (do not fail the whole batch
  for one bad entry). Do not print to stdout (keep `-o -` pipes clean,
  AGENTS.md §11). Note: SPEC-004 has no access to the verbosity flag yet (that
  is SPEC-007/clap), so skip silently for now — do NOT invent a logging
  dependency or an `eprintln!` the tests would have to tolerate.

### Image-ness by extension (NOT by decoding)

Source must not decode (that is SPEC-002's `Image`, and would violate the
layering `source → image` if Source called the decoder to filter). Decide
image-ness by a small **case-insensitive extension allow-list** matching the
core formats (DEC-004): `jpg`, `jpeg`, `png`, `gif`, `bmp`, `tif`, `tiff`,
`ico`. A file with one of these extensions is an image candidate; everything
else in a *directory listing* is skipped. (A glob pattern like `*.png` already
filters by extension, but still run candidates through the symlink check; a
broad glob like `*` or `<dir>/*` should also be extension-filtered so a glob
and a directory behave consistently.) A file the user names *directly* as a
single path is NOT extension-filtered — if they point at `weird` with no
extension, yield it and let `Image::load` decide.

### What prior specs already provide (your foundation)

- `crustyimg::image::Image` — `load(impl AsRef<Path>) -> Result<Image>`,
  `from_bytes(&[u8]) -> Result<Image>`, `from_reader<R: Read + Seek>`. **You do
  not call these** — Source yields `Input`s that a later caller (CLI/pipeline)
  loads. They exist so you know the shape: `Input::Path` pairs with
  `Image::load(path)`; `Input::Stdin { bytes, .. }` pairs with
  `Image::from_bytes(&bytes)`. (Stdin is not seekable, so bytes are read into a
  `Vec` and `from_bytes` is the right consumer — not `from_reader`, whose
  `Seek` bound stdin can't satisfy. Source does the read; it owns the `Vec`.)
- `crustyimg::error::{ImageError, Result}` — the typed-error pattern to mirror
  for `SourceError`. Keep `SourceError` local to `src/source/mod.rs` (like
  `OperationError` lives in `src/operation/`); do NOT widen the crate `Error`.
- `glob` 0.3 — `glob::glob(pattern) -> Result<Paths, PatternError>`; `Paths`
  yields `Result<PathBuf, GlobError>`. Map `PatternError` →
  `SourceError::InvalidPattern { pattern, reason }`; on each `GlobError` (an
  entry that couldn't be read), **skip** that entry. Apply the extension filter
  + symlink-escape check to the surviving paths, then sort before returning.

### Out of scope (for this spec specifically)

If any of these feels necessary during build, STOP and raise it
(`questions.yaml` or `[?]` in the timeline) rather than expanding this spec:

- **Sink / output** (SPEC-005) — no writing files, no name-template *expansion*
  (Source only provides the `stem`; Sink expands `{stem}_web.{ext}`), no
  `--out-dir`, no stdout, no viuer.
- **Recipes** (SPEC-006) and **CLI wiring / clap** (SPEC-007) — Source takes a
  plain `&str` argument and a `reader`; it does NOT parse args, read the
  verbosity flag, or know about subcommands. The CLI will call `resolve` later.
- **Parallel batch execution** (STAGE-005) — Source yields the *list*; `rayon`
  consumes it later. Do not spawn threads or add `rayon`.
- **Actual decoding** (SPEC-002, done) — Source never decodes; image-ness is by
  extension. Do not call `Image::load`/`from_bytes` from `source/`.
- **Decode limits / the full security assessment** (STAGE-006) — do NOT set
  `image::Limits`; do NOT add `cargo audit`. The symlink-escape skip is the only
  hardening in scope here.
- **Recursive directory walking** — non-recursive only (DEC-010). No `walkdir`,
  no `--recursive` flag.

### Exact commands (AGENTS.md §6)

```bash
cargo build                  # debug build
cargo test                   # all tests (unit + integration)
cargo clippy -- -D warnings  # lint; warnings are errors
cargo fmt --check            # formatting gate (use `cargo fmt` to fix)
```

All four must pass before opening the PR (the four gates).

### Dependency note (read before reaching for a crate)

- **`glob` 0.3** is the ONE new runtime dependency, justified by **DEC-010**
  (already written). Pin it `=0.3.x` to the latest available patch, matching the
  existing `=`-pinned style in `Cargo.toml`.
- **`tempfile`** is a **dev-dependency** (under `[dev-dependencies]`), used only
  by tests. It is not a top-level runtime dep; `no-new-top-level-deps-without-
  decision` targets runtime manifests. Add it, note it in Build Completion, no
  DEC needed.
- Add **NOTHING ELSE** — not `walkdir`, not `rayon`, not `serde`, not `clap`. If
  you believe you need another crate, STOP, add to `questions.yaml`, mark `[?]`.

## Notes for the Implementer

- **Dispatch order in `resolve`.** Check `arg == "-"` first (stdin). Then
  `looks_like_glob(arg)` → glob branch. Then `Path::new(arg).is_dir()` →
  directory branch. Else → single-file branch (check existence; map a missing
  file to `NotFound`). Deciding the branch *before* heavy filesystem work keeps
  behavior predictable and the code easy to follow.
- **`looks_like_glob`** is just "contains any of `*`, `?`, `[`". Keep it a tiny
  free fn (testable). `-` and plain paths return `false`.
- **Sorting.** After collecting `PathBuf`s (glob or directory), `paths.sort()` —
  `PathBuf`'s `Ord` is lexicographic over the OS string, which is stable and
  deterministic. Do the sort *after* the symlink-escape + extension filters so
  the order is over the surviving set.
- **The `reader` parameter.** Generic `&mut impl std::io::Read`. In the stdin
  branch: `let mut bytes = Vec::new(); reader.read_to_end(&mut bytes)?;` (the
  `?` maps an io error to `SourceError::Stdin` via `#[from]`). Production code
  (SPEC-007) will pass `std::io::stdin().lock()`; tests pass a byte slice or
  `std::io::empty()`. Every non-stdin branch ignores `reader`.
- **Stdin stem.** Use the constant `"stdin"`. (A future spec may let `--name`
  override it; not here.)
- **`Input::stem()` for paths.** `Path::file_stem()` → `OsStr` → `.to_str()`.
  Because `stem()` returns `&str` borrowed from the `Input`, store the `PathBuf`
  in the `Path` variant and compute `self.path().and_then(|p|
  p.file_stem()).and_then(|s| s.to_str()).unwrap_or("")` on demand (the
  `unwrap_or` here is a default, NOT a panic). If lifetimes fight you, it is
  acceptable to make `stem()` return an owned `String` instead of `&str` —
  document the change in Build Completion (a minor, sanctioned signature
  relaxation; update the unit test accordingly). For `Stdin`, `stem()` returns
  the stored `stem` field.
- **Canonicalization gotcha.** `std::fs::canonicalize` requires the path to
  exist; a dangling symlink errors — that is the case you want to **skip**
  (don't propagate). Wrap it: `let Ok(real) = std::fs::canonicalize(&p) else {
  continue; };`.
- **No panics.** Every `?` in library code returns a `SourceError`. The only
  `unwrap`s allowed are in `#[cfg(test)]` / `tests/`.
- **Module-name collision.** `image` is both a crate and the `src/image/`
  module — but Source does not use the `image` crate at all, so no `::image`
  aliasing is needed here. (Source deals in `std::path` + `glob` only.) If a
  *test* needs to write a real PNG fixture, that test may `use image::...` —
  the collision only bites inside library modules that name `mod image`, which
  `source/` does not.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.
AGENTS.md §13/§15: build-cycle edits to this spec stay LIMITED to this
`## Build Completion` section.*

- **Branch:** `feat/spec-004-source-input-abstraction`
- **PR (if applicable):** see timeline for PR number once opened
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - No new DEC — DEC-010 (glob) was written at design time; `tempfile` is a
    dev-dependency (test-only) and the extension allow-list is spec-mandated.
    Note: `tempfile = "=3.27.0"` was added under `[dev-dependencies]`; it is
    not a top-level runtime dep so the `no-new-top-level-deps-without-decision`
    constraint does not require a DEC — recorded here for completeness.
- **Deviations from spec:**
  - `stem()` returns `&str` (spec's preferred form) — no relaxation to `String`
    needed; the lifetime works because `Input::Path` stores a `PathBuf` and
    `Input::Stdin` stores a `String`, so `&str` borrows are valid for the
    lifetime of the `Input`.
  - Yielded **original** entry paths (not canonical paths) from directory and
    glob branches, so output stems/names stay intuitive and match what the user
    typed or saw on disk. The canonical path is only used internally for the
    symlink-escape `starts_with` check, never stored.
  - Stdin reader wired as `&mut impl Read` injected parameter; production will
    pass `std::io::stdin().lock()` (SPEC-007). Tests pass `&mut &b"..."[..]` or
    `std::io::empty()`. The `?` on `read_to_end` maps `io::Error` to
    `SourceError::Stdin` via `#[from]`.
  - **Symlink test platform handling:** the `directory_skips_symlink_escaping_root`
    test is split into two `#[cfg(unix)]` / `#[cfg(windows)]` functions. The Unix
    variant creates a real symlink via `std::os::unix::fs::symlink` and asserts
    the escaping entry is excluded. The Windows variant is a no-op stub with an
    explanatory `eprintln!` because symlink creation requires the
    `SeCreateSymbolicLink` privilege unavailable in standard CI. The guard code
    in `resolve_directory` is present on all platforms.
  - One clippy lint (`doc_overindented_list_items`) was tripped by the aligned
    continuation lines in the `resolve()` doc comment; fixed by de-indenting to
    3-space continuation indent.
- **Follow-up work identified:**
  - `--recursive` flag + `walkdir` (DEC-010 reserves this): will need its own
    spec + DEC when recursive directory walking is required.
  - `--verbose` warning for skipped entries: source currently skips silently
    (no `eprintln!` to keep stdout pipes clean, per §11). Once SPEC-007 lands
    the verbosity flag, a follow-up can wire `--verbose` skipped-entry warnings
    through the CLI.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The spec was well-structured and comprehensive. The one moment of friction
   was the glob-branch symlink-escape check: the spec says to check "relative to
   the pattern's base dir," but doesn't spell out how to extract that base dir
   from an arbitrary pattern string. The `Notes for the Implementer` section on
   the directory branch was clear; the glob branch equivalent required a small
   design decision (the `glob_base_dir` helper). This could be a one-liner note
   in a future spec revision.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No missing constraints. The `untrusted-input-hardening` rule was
   well-scoped to symlink-escape only, with an explicit note that decode limits
   land in STAGE-006. The only friction was Clippy's `doc_overindented_list_items`
   lint (relatively new in Rust 1.94); it wasn't anticipated in the spec's "Watch
   for" section but was trivially fixed and doesn't warrant a constraint addition.

3. **If you did this task again, what would you do differently?**
   — Run `cargo clippy` after writing the doc comments (before writing tests) to
   catch formatting issues early. The gates are cheap to run incrementally and
   surface issues one at a time rather than all at the end.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
