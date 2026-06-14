//! Source input abstraction (SPEC-004).
//!
//! Resolves a single CLI argument into an ordered, deterministic list of
//! [`Input`]s the pipeline will process. Four dispatch shapes (decided before
//! touching the filesystem):
//!
//! - `"-"`                       → read stdin via injected `reader`
//! - contains `*`, `?`, or `[`  → glob-pattern expansion
//! - an existing directory       → non-recursive image listing
//! - anything else               → single-file path (existence checked)
//!
//! Layering (see `docs/architecture.md`): this module depends only on
//! `std`, `glob`, and [`crate::error`]. It must NOT touch `clap`, `image`
//! (the pixel crate), recipes, sinks, or terminals.
//!
//! ## Security (untrusted-input-hardening)
//!
//! Directory and glob enumeration skip any entry whose canonicalized real path
//! escapes the canonicalized root — dangling symlinks are also skipped
//! (canonicalize errors → skip, never propagate). See the symlink-escape check
//! in [`resolve`].

use std::io::Read;
use std::path::{Path, PathBuf};

// ── Types ─────────────────────────────────────────────────────────────────────

/// One resolved input the pipeline will process.
///
/// Carries enough to (a) load it (SPEC-002's `Image::load` / `from_bytes`)
/// and (b) name its output later (SPEC-005's name templates, via [`stem()`]).
///
/// Source does NOT decode — it describes what to load and what to call it.
///
/// [`stem()`]: Input::stem
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    /// A file on disk. The pipeline loads it with `Image::load(path)`.
    Path(PathBuf),
    /// Bytes read from stdin (the `-` argument). The pipeline loads them with
    /// `Image::from_bytes(&bytes)`. `stem` is the synthetic output name
    /// (always `"stdin"` unless a future spec adds `--name`).
    Stdin { bytes: Vec<u8>, stem: String },
}

impl Input {
    /// The output stem for name templates (`{stem}`): a file input's filename
    /// without its extension, or the stdin input's synthetic stem.
    ///
    /// Never contains a path separator. Returns `""` for a path whose stem
    /// is not valid UTF-8 (rare; treated as a safe default, not a panic).
    pub fn stem(&self) -> &str {
        match self {
            Input::Path(p) => p.file_stem().and_then(|s| s.to_str()).unwrap_or(""),
            Input::Stdin { stem, .. } => stem.as_str(),
        }
    }

    /// The path for `Input::Path`; `None` for stdin.
    ///
    /// Convenience for callers that want to log or load by path.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Input::Path(p) => Some(p.as_path()),
            Input::Stdin { .. } => None,
        }
    }
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

// ── Image extension allow-list ─────────────────────────────────────────────

/// Case-insensitive image-extension allow-list applied to directory listings
/// and broad glob matches (NOT to directly-named single files).
///
/// Decision: image-ness by extension, never by decoding (DEC-002 layering;
/// `single-image-library` constraint: Source must not call the pixel decoder).
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "ico"];

fn has_image_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            IMAGE_EXTENSIONS
                .iter()
                .any(|&allowed| e.eq_ignore_ascii_case(allowed))
        })
        .unwrap_or(false)
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Whether a string looks like a glob pattern (contains `*`, `?`, or `[`).
///
/// Pulled out as a free function so it is directly unit-testable and so the
/// dispatch in [`resolve`] is explicit and readable.
pub fn looks_like_glob(arg: &str) -> bool {
    arg.contains('*') || arg.contains('?') || arg.contains('[')
}

/// Resolve a single CLI input argument into an ordered list of inputs.
///
/// Dispatch order (decided before touching the filesystem):
///
/// 1. `arg == "-"` → read `reader` to end; yield one `Stdin`.
/// 2. `looks_like_glob(arg)` → glob expansion (sorted, extension-filtered,
///    symlink-escape-checked).
/// 3. `Path::new(arg).is_dir()` → non-recursive directory listing (sorted,
///    extension-filtered, symlink-escape-checked).
/// 4. else → single-file branch (existence checked; NOT extension-filtered —
///    the user named it directly).
///
/// `reader` is injected (`&mut impl Read`) so tests can feed bytes without a
/// real stdin. Production passes `std::io::stdin().lock()`. Every non-stdin
/// branch ignores `reader`.
///
/// Results are sorted lexicographically by path (deterministic across runs).
/// Unreadable / non-image entries inside a glob or directory are skipped
/// silently (keeping `-o -` stdout pipes clean; see SPEC-007 for verbosity).
/// A missing single file, an empty glob match, or an invalid pattern is a
/// typed [`SourceError`].
pub fn resolve(arg: &str, reader: &mut impl Read) -> Result<Vec<Input>, SourceError> {
    if arg == "-" {
        return resolve_stdin(reader);
    }
    if looks_like_glob(arg) {
        return resolve_glob(arg);
    }
    if std::path::Path::new(arg).is_dir() {
        return resolve_directory(arg);
    }
    resolve_single_file(arg)
}

// ── Private dispatch functions ────────────────────────────────────────────────

/// Stdin branch: drain `reader` into a `Vec<u8>`, yield one `Input::Stdin`.
///
/// The `?` maps `io::Error` → `SourceError::Stdin` via `#[from]`.
fn resolve_stdin(reader: &mut impl Read) -> Result<Vec<Input>, SourceError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(vec![Input::Stdin {
        bytes,
        stem: "stdin".into(),
    }])
}

/// Glob branch: expand the pattern, filter to image extensions + symlink-
/// escape check, sort, and return. Empty match → `NotFound`.
fn resolve_glob(pattern: &str) -> Result<Vec<Input>, SourceError> {
    let paths_iter = glob::glob(pattern).map_err(|e| SourceError::InvalidPattern {
        pattern: pattern.into(),
        reason: e.to_string(),
    })?;

    // Determine the "root" for the symlink-escape check: the non-glob prefix
    // of the pattern. For patterns like `photos/*.jpg` the root is `photos/`;
    // for `*.jpg` it is `.`. We canonicalize this once.
    //
    // We walk the pattern from the left up to the first glob metachar to find
    // the base dir, then take its parent directory as the anchor.
    let base = glob_base_dir(pattern);
    // If canonicalize fails (e.g. the base doesn't exist yet) we fall through
    // without a root guard — the entries themselves will be checked.
    let root_opt = std::fs::canonicalize(&base).ok();

    let mut results: Vec<PathBuf> = Vec::new();
    for entry in paths_iter {
        let entry_path = match entry {
            Ok(p) => p,
            // GlobError on a specific entry → skip, do not abort the batch.
            Err(_) => continue,
        };

        // Extension filter: a broad glob like `*` should behave consistently
        // with a directory listing (both filter to image extensions).
        if !has_image_extension(&entry_path) {
            continue;
        }

        // Symlink-escape check against the glob base root.
        if let Some(ref root) = root_opt {
            let Ok(real) = std::fs::canonicalize(&entry_path) else {
                // Dangling symlink or unreadable entry → skip.
                continue;
            };
            if !real.starts_with(root) {
                continue;
            }
        }

        results.push(entry_path);
    }

    results.sort();

    if results.is_empty() {
        return Err(SourceError::NotFound(pattern.into()));
    }

    Ok(results.into_iter().map(Input::Path).collect())
}

/// Directory branch (non-recursive): list top-level entries, apply symlink-
/// escape + extension filters, sort, return. A missing/unreadable dir is
/// `NotFound`; an empty-but-valid dir is an empty `Vec` (not an error).
fn resolve_directory(dir: &str) -> Result<Vec<Input>, SourceError> {
    // Canonicalize the root once — this is the anchor for the escape check.
    // A missing directory is NotFound (do NOT use the #[from] Stdin variant).
    let root = std::fs::canonicalize(dir).map_err(|_| SourceError::NotFound(dir.into()))?;

    let read_dir = std::fs::read_dir(&root).map_err(|_| SourceError::NotFound(dir.into()))?;

    let mut results: Vec<PathBuf> = Vec::new();
    for entry_result in read_dir {
        // A single unreadable entry → skip, not a hard error.
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };
        let entry_path = entry.path();

        // Symlink-escape check: canonicalize the entry so symlinks are
        // resolved. A dangling symlink errors from canonicalize → skip.
        let Ok(real) = std::fs::canonicalize(&entry_path) else {
            continue;
        };

        // A symlink pointing outside root (or to root itself as a dir) must
        // not pull external files into the batch.
        if !real.starts_with(&root) {
            continue;
        }

        // Skip subdirectories (non-recursive; top-level files only).
        // We check `real` (the canonicalized path) so a symlink-to-dir is
        // also skipped.
        if real.is_dir() {
            continue;
        }

        // Extension filter.
        if !has_image_extension(&entry_path) {
            continue;
        }

        // Yield the original entry path (intuitive stem/naming), not real.
        results.push(entry_path);
    }

    results.sort();
    Ok(results.into_iter().map(Input::Path).collect())
}

/// Single-file branch: check existence; yield one `Input::Path` or `NotFound`.
///
/// NOT extension-filtered — if the user named the file directly, we yield it
/// and let `Image::load` decide whether it is a valid image.
fn resolve_single_file(arg: &str) -> Result<Vec<Input>, SourceError> {
    let path = PathBuf::from(arg);
    if path.exists() {
        Ok(vec![Input::Path(path)])
    } else {
        Err(SourceError::NotFound(arg.into()))
    }
}

/// Compute the base directory for the symlink-escape check in the glob branch.
///
/// Walk the pattern left-to-right to find the first glob metachar (`*`, `?`,
/// `[`); everything before it is the "literal prefix". The base directory is
/// the parent of that prefix, or `.` if there is no parent (e.g. `*.jpg`).
fn glob_base_dir(pattern: &str) -> PathBuf {
    // Find the byte offset of the first metachar.
    let metachar_pos = pattern
        .char_indices()
        .find(|(_, c)| matches!(c, '*' | '?' | '['))
        .map(|(i, _)| i);

    let prefix = match metachar_pos {
        Some(pos) => &pattern[..pos],
        None => pattern, // No metachar → treat the whole thing as prefix.
    };

    let prefix_path = PathBuf::from(prefix);
    // The prefix usually ends with a separator (e.g. `photos/`). Take the
    // parent directory of that prefix.
    if prefix_path.is_dir() {
        prefix_path
    } else {
        prefix_path
            .parent()
            .map(|p| {
                if p.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    p.to_path_buf()
                }
            })
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_like_glob_classifies_patterns() {
        // Positive cases — at least one metachar.
        assert!(looks_like_glob("a/*.jpg"));
        assert!(looks_like_glob("f?.png"));
        assert!(looks_like_glob("s[12].png"));
        // Negative cases — no metachar.
        assert!(!looks_like_glob("a/file.jpg"));
        assert!(!looks_like_glob("-"));
        assert!(!looks_like_glob("dir"));
    }

    #[test]
    fn input_stem_for_path() {
        let input = Input::Path(PathBuf::from("/a/b/photo.JPG"));
        let s = input.stem();
        assert_eq!(s, "photo");
        // Must not contain a path separator.
        assert!(!s.contains('/'));
        assert!(!s.contains('\\'));
    }

    #[test]
    fn input_stem_and_path_for_stdin() {
        let input = Input::Stdin {
            bytes: vec![1, 2, 3],
            stem: "stdin".into(),
        };
        assert_eq!(input.stem(), "stdin");
        assert_eq!(input.path(), None);
    }

    #[test]
    fn resolve_stdin_yields_one_input_with_bytes() {
        let data: &[u8] = b"\x89PNG\r\n";
        let mut reader: &[u8] = data;
        let result = resolve("-", &mut reader).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Input::Stdin { bytes, stem } => {
                assert_eq!(bytes.as_slice(), data);
                assert_eq!(stem, "stdin");
            }
            Input::Path(_) => panic!("expected Stdin variant"),
        }
    }
}
