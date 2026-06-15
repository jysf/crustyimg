//! Sink output abstraction (SPEC-005).
//!
//! Takes a final [`Image`] from the pipeline and writes it to one of four
//! output shapes: a specific file, a directory with a name-template, stdout,
//! or terminal display via viuer (behind the `display` cargo feature, DEC-011).
//!
//! ## Security (untrusted-input-hardening)
//!
//! - **Path traversal:** [`safe_join`] canonicalizes the output dir and
//!   rejects any expanded file name containing `..`, a path separator, or an
//!   absolute component. The check is on the EXPANDED name, not the raw stem,
//!   because a user-supplied `--name-template` can inject separators.
//! - **Overwrite guard:** [`Sink::write`] checks `Path::exists()` BEFORE
//!   opening for write; returns [`SinkError::AlreadyExists`] when
//!   [`Overwrite::Forbid`] (the default). Only [`Overwrite::Allow`] (`--yes`)
//!   proceeds.
//!
//! ## Encoding
//!
//! Format is inferred from the output extension via [`format_from_extension`],
//! or supplied explicitly. Pixels are encoded via
//! `img.pixels().write_to(&mut Cursor<Vec<u8>>, format)` (the Cursor gives the
//! `Write + Seek` bound some encoders require) and the resulting bytes are then
//! written to the final destination — a `BufWriter<File>` or the injected `out`
//! writer. This uniform buffer-then-write path keeps `out: &mut dyn Write`
//! simple and keeps all diagnostics off `out` (DEC-002, DEC-004).
//!
//! Layering: this module depends on `::image` and [`crate::image`] only. It
//! must NOT touch `clap`, recipes, source internals, or terminals beyond viuer.

use std::fs::OpenOptions;
use std::io::{BufWriter, Cursor, IsTerminal, Write};
use std::path::{Path, PathBuf};

use ::image::ImageFormat;

use crate::image::Image;

// ── Public types ──────────────────────────────────────────────────────────────

/// Where a final [`Image`] is written. Constructed by the (future) CLI
/// (SPEC-007); here it is the public output contract the pipeline hands a
/// final [`Image`] to.
pub enum Sink {
    /// Write to one explicit output path (`-o <PATH>`). A `None` extension on
    /// the path with no explicit `format` is a [`SinkError::UnknownFormat`].
    File {
        path: PathBuf,
        format: Option<ImageFormat>,
    },
    /// Write into `dir` using `template` over the input's stem
    /// (`{stem}_web.{ext}`). `format` (if `Some`) overrides extension
    /// inference; the default format is PNG when neither is provided.
    Dir {
        dir: PathBuf,
        template: String,
        format: Option<ImageFormat>,
    },
    /// Write encoded bytes to stdout (`-o -`). `format` must be `Some` (there
    /// is no path to infer from) — `None` is [`SinkError::UnknownFormat`].
    Stdout { format: Option<ImageFormat> },
    /// Render in the terminal via viuer (behind the `display` cargo
    /// feature, DEC-011). Refuses with [`SinkError::NotATty`] on a
    /// non-tty regardless of whether the feature is enabled.
    /// `width`/`height` are optional sizing hints (`None`/`None` =
    /// fit to terminal, viuer's default).
    Display {
        width: Option<u32>,
        height: Option<u32>,
    },
}

/// Whether overwriting an existing destination file is permitted (`--yes`).
///
/// A small explicit enum reads better than a bare `bool` at call sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overwrite {
    /// Do not overwrite an existing file — return [`SinkError::AlreadyExists`].
    Forbid,
    /// Overwrite an existing file (the `--yes` flag).
    Allow,
}

/// Errors that can occur while writing an [`Image`] to a sink (DEC-007).
///
/// Kept module-local (not merged into the crate `Error`) to mirror
/// `SourceError` in `src/source/`. The binary maps these to exit code 5
/// (`docs/api-contract.md`).
#[derive(Debug, thiserror::Error)]
pub enum SinkError {
    /// An I/O error while opening, creating, or writing the output.
    #[error("could not write output")]
    Io(#[from] std::io::Error),

    /// The `image` crate could not encode the pixels to the chosen format.
    #[error("could not encode image: {0}")]
    Encode(String),

    /// No output extension and no explicit `--format`; cannot infer the
    /// output format. Includes the `Stdout { format: None }` case.
    #[error("could not determine output format (no extension and no --format)")]
    UnknownFormat,

    /// The output extension was present but is not in the DEC-004 core set.
    #[error("unsupported output extension: {0}")]
    UnsupportedExtension(String),

    /// The expanded output path would escape the target directory.
    #[error("output path escapes the target directory: {0}")]
    Traversal(String),

    /// The destination already exists and `--yes` was not passed.
    #[error("output file already exists (use --yes to overwrite): {0}")]
    AlreadyExists(String),

    /// `Sink::Display` was invoked on a non-tty stdout.
    #[error("terminal display requires a tty")]
    NotATty,

    /// The viuer render call failed (or the `display` feature is not built).
    #[error("terminal display failed: {0}")]
    Display(String),
}

/// The naming context a [`Sink::Dir`] needs from the originating input.
///
/// Constructed from [`source::Input`] by the future CLI (SPEC-007). Carries
/// just the fields the sink needs; the full `Input` stays in `src/source/`.
pub struct SinkInput<'a> {
    /// The output stem for name templates (`{stem}`): the source file's
    /// basename without extension, or the stdin synthetic stem. Never contains
    /// a path separator (SPEC-004 guarantee).
    pub stem: &'a str,
    /// The source path, if available (used to derive `{name}` and `{parent}`).
    /// `None` for stdin inputs.
    pub path: Option<&'a Path>,
}

// ── Free helpers (each exported and unit-tested) ──────────────────────────────

/// Infer an [`ImageFormat`] from a path's extension (case-insensitive), over
/// the DEC-004 core set: png / jpg / jpeg / gif / bmp / tif / tiff / ico.
///
/// - No extension → [`SinkError::UnknownFormat`].
/// - Extension not in the core set → [`SinkError::UnsupportedExtension`].
pub fn format_from_extension(path: &Path) -> Result<ImageFormat, SinkError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or(SinkError::UnknownFormat)?;

    match ext.to_ascii_lowercase().as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "gif" => Ok(ImageFormat::Gif),
        "bmp" => Ok(ImageFormat::Bmp),
        "tif" | "tiff" => Ok(ImageFormat::Tiff),
        "ico" => Ok(ImageFormat::Ico),
        other => Err(SinkError::UnsupportedExtension(other.to_owned())),
    }
}

/// The conventional lowercase extension string for a format (for `{ext}`
/// expansion): Png→"png", Jpeg→"jpg", Gif→"gif", Bmp→"bmp", Tiff→"tiff",
/// Ico→"ico". Any other variant returns a sensible lowercase default.
pub fn extension_for_format(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Gif => "gif",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Tiff => "tiff",
        ImageFormat::Ico => "ico",
        // Sensible fallbacks for any other variant (not reached for the core
        // set, but required for exhaustiveness since ImageFormat is non-exhaustive).
        ImageFormat::WebP => "webp",
        ImageFormat::Pnm => "pnm",
        ImageFormat::Tga => "tga",
        ImageFormat::Dds => "dds",
        ImageFormat::Hdr => "hdr",
        ImageFormat::OpenExr => "exr",
        ImageFormat::Farbfeld => "ff",
        ImageFormat::Avif => "avif",
        ImageFormat::Qoi => "qoi",
        _ => "bin",
    }
}

/// Expand a name template over the naming context of an input.
///
/// Recognised tokens:
/// - `{stem}` — the input's stem (no extension, no separator).
/// - `{ext}` — the chosen output extension string (e.g. `"png"`).
/// - `{name}` — the input's file name (stem + original extension), derived from
///   `path.file_name()`; falls back to `"{stem}.{ext}"` when `path` is `None`.
/// - `{parent}` — the last component of `path.parent()`, or `""` when absent.
///
/// Any `{token}` not in the list above is left **literal** in the output.
/// Returns the final file NAME only (no directory component).
pub fn expand_template(template: &str, stem: &str, ext: &str, path: Option<&Path>) -> String {
    // Derive {name}: file_name from path, or synthesise stem.ext.
    let name: String = path
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| format!("{stem}.{ext}"));

    // Derive {parent}: last component of parent dir, or "".
    let parent: String = path
        .and_then(|p| p.parent())
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_owned();

    template
        .replace("{stem}", stem)
        .replace("{ext}", ext)
        .replace("{name}", &name)
        .replace("{parent}", &parent)
}

/// Join `dir` and an expanded `file_name`, rejecting any result that would
/// escape `dir` (path traversal).
///
/// Rejection rules (checked before joining):
/// - `file_name` is empty.
/// - `file_name` is absolute ([`Path::is_absolute`]).
/// - `file_name` contains a `..` component.
/// - `file_name` contains a path separator (`/` or `\`).
///
/// After the pre-join checks, `dir` is canonicalized (it must already exist;
/// a missing dir is a typed [`SinkError::Io`] from `fs::canonicalize`). The
/// candidate path is then verified to [`starts_with`] the canonicalized dir
/// (belt-and-suspenders confirmation). Returns the safe joined [`PathBuf`] or
/// [`SinkError::Traversal`].
///
/// [`starts_with`]: PathBuf::starts_with
pub fn safe_join(dir: &Path, file_name: &str) -> Result<PathBuf, SinkError> {
    // Pre-join rejection: empty name.
    if file_name.is_empty() {
        return Err(SinkError::Traversal(file_name.to_owned()));
    }

    // Pre-join rejection: absolute path.
    if Path::new(file_name).is_absolute() {
        return Err(SinkError::Traversal(file_name.to_owned()));
    }

    // Pre-join rejection: contains a path separator.
    if file_name.contains('/') || file_name.contains('\\') {
        return Err(SinkError::Traversal(file_name.to_owned()));
    }

    // Pre-join rejection: contains a `..` component.
    // After ruling out separators above a single-component name can't have
    // `..` as a path component, but we check the component iterator anyway
    // to be belt-and-suspenders safe.
    for component in Path::new(file_name).components() {
        use std::path::Component;
        if matches!(component, Component::ParentDir) {
            return Err(SinkError::Traversal(file_name.to_owned()));
        }
    }

    // Canonicalize dir — a missing or unreadable dir is a typed Io error
    // (via #[from] std::io::Error). Do NOT create the dir.
    let canonical_dir = std::fs::canonicalize(dir)?;

    // Build candidate and verify containment.
    let candidate = canonical_dir.join(file_name);

    if !candidate.starts_with(&canonical_dir) {
        return Err(SinkError::Traversal(file_name.to_owned()));
    }

    Ok(candidate)
}

// ── Sink::write ───────────────────────────────────────────────────────────────

impl Sink {
    /// Encode `img` and write it according to this sink.
    ///
    /// - `input` provides the naming context for [`Sink::Dir`] templates.
    /// - `overwrite` controls the overwrite guard for [`Sink::File`] and
    ///   [`Sink::Dir`] (stdout and display never check this).
    /// - `out` receives the encoded bytes for [`Sink::Stdout`] ONLY. The
    ///   injected writer exists so tests can capture bytes without touching
    ///   real stdout. Diagnostics never go to `out`.
    pub fn write(
        &self,
        img: &Image,
        input: &SinkInput<'_>,
        overwrite: Overwrite,
        out: &mut dyn Write,
    ) -> Result<(), SinkError> {
        match self {
            Sink::File { path, format } => {
                // Choose format: explicit override, or infer from extension.
                let fmt = match format {
                    Some(f) => *f,
                    None => format_from_extension(path)?,
                };
                // Overwrite guard: check before opening (never truncate then error).
                guard_overwrite(path, overwrite)?;
                // Encode to bytes, then write to a BufWriter<File>.
                let bytes = encode_to_bytes(img, fmt)?;
                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)?;
                let mut writer = BufWriter::new(file);
                writer.write_all(&bytes)?;
                Ok(())
            }

            Sink::Dir {
                dir,
                template,
                format,
            } => {
                // Choose format: explicit override, or default to PNG for dir sinks.
                let fmt = format.unwrap_or(ImageFormat::Png);
                let ext = extension_for_format(fmt);
                // Expand template and build the safe output path.
                let file_name = expand_template(template, input.stem, ext, input.path);
                let full_path = safe_join(dir, &file_name)?;
                // Overwrite guard.
                guard_overwrite(&full_path, overwrite)?;
                // Encode and write.
                let bytes = encode_to_bytes(img, fmt)?;
                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&full_path)?;
                let mut writer = BufWriter::new(file);
                writer.write_all(&bytes)?;
                Ok(())
            }

            Sink::Stdout { format } => {
                // format must be Some — there is no path to infer from.
                let fmt = format.ok_or(SinkError::UnknownFormat)?;
                // Encode to bytes and write ONLY the encoded bytes to `out`.
                // No diagnostic output goes here.
                let bytes = encode_to_bytes(img, fmt)?;
                out.write_all(&bytes)?;
                Ok(())
            }

            Sink::Display { width, height } => {
                // NotATty check is always first, feature-independent.
                if !std::io::stdout().is_terminal() {
                    return Err(SinkError::NotATty);
                }

                // Actual render behind the `display` feature gate (DEC-011).
                // The not(feature) branch is the tail expression when the
                // feature is off; when the feature is on, the cfg(feature)
                // block is the only live branch (the not-feature block is
                // compiled away), so we use a single cfg-selected expression.
                #[cfg(feature = "display")]
                {
                    // The match binds &Option<u32>, so deref to pass Option<u32>.
                    let conf = viuer::Config {
                        width: *width,
                        height: *height,
                        use_kitty: true,
                        use_iterm: true,
                        ..Default::default()
                    };
                    viuer::print(img.pixels(), &conf)
                        .map_err(|e| SinkError::Display(e.to_string()))
                        .map(|_| ())
                }
                #[cfg(not(feature = "display"))]
                {
                    // Silence unused-variable warning in the feature-off build;
                    // the feature-on build above actively uses both fields.
                    let _ = (width, height);
                    // When built without the feature, inform the caller rather
                    // than silently succeeding.
                    Err(SinkError::Display(
                        "built without the `display` feature".into(),
                    ))
                }
            }
        }
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Encode `img` pixels to a `Vec<u8>` in `format`.
///
/// Uses an in-memory `Cursor<Vec<u8>>` as the writer so all encoders have the
/// `Write + Seek` bound they may require. The resulting bytes are then written
/// to the actual destination (file or `out`) by the caller.
fn encode_to_bytes(img: &Image, format: ImageFormat) -> Result<Vec<u8>, SinkError> {
    let mut cursor = Cursor::new(Vec::new());
    img.pixels()
        .write_to(&mut cursor, format)
        .map_err(|e| SinkError::Encode(e.to_string()))?;
    Ok(cursor.into_inner())
}

/// Check the overwrite guard for a File or Dir sink.
///
/// If the path already exists and `overwrite == Overwrite::Forbid`, return
/// [`SinkError::AlreadyExists`] WITHOUT truncating the file.
fn guard_overwrite(path: &Path, overwrite: Overwrite) -> Result<(), SinkError> {
    if path.exists() && overwrite == Overwrite::Forbid {
        return Err(SinkError::AlreadyExists(path.display().to_string()));
    }
    Ok(())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── format_from_extension ──────────────────────────────────────────────

    #[test]
    fn format_from_extension_is_case_insensitive() {
        assert!(matches!(
            format_from_extension(Path::new("out.png")),
            Ok(ImageFormat::Png)
        ));
        assert!(matches!(
            format_from_extension(Path::new("out.PNG")),
            Ok(ImageFormat::Png)
        ));
        assert!(matches!(
            format_from_extension(Path::new("out.JpG")),
            Ok(ImageFormat::Jpeg)
        ));
        // No extension → UnknownFormat.
        assert!(matches!(
            format_from_extension(Path::new("out")),
            Err(SinkError::UnknownFormat)
        ));
        // Unknown extension → UnsupportedExtension.
        assert!(matches!(
            format_from_extension(Path::new("out.xyz")),
            Err(SinkError::UnsupportedExtension(_))
        ));
    }

    // ── extension_for_format ───────────────────────────────────────────────

    #[test]
    fn extension_for_format_round_trips() {
        assert_eq!(extension_for_format(ImageFormat::Jpeg), "jpg");
        assert_eq!(extension_for_format(ImageFormat::Png), "png");
        assert_eq!(extension_for_format(ImageFormat::Gif), "gif");
        assert_eq!(extension_for_format(ImageFormat::Bmp), "bmp");
        assert_eq!(extension_for_format(ImageFormat::Tiff), "tiff");
        assert_eq!(extension_for_format(ImageFormat::Ico), "ico");

        // Feed each back through format_from_extension of "foo.{ext}" and
        // verify we get the same format back.
        for &fmt in &[
            ImageFormat::Png,
            ImageFormat::Jpeg,
            ImageFormat::Gif,
            ImageFormat::Bmp,
            ImageFormat::Tiff,
            ImageFormat::Ico,
        ] {
            let ext = extension_for_format(fmt);
            let path_str = format!("foo.{ext}");
            let inferred = format_from_extension(Path::new(&path_str)).unwrap();
            assert_eq!(
                inferred, fmt,
                "round-trip failed for {fmt:?}: ext={ext}, inferred={inferred:?}"
            );
        }
    }

    // ── expand_template ────────────────────────────────────────────────────

    #[test]
    fn expand_template_expands_all_tokens() {
        let stem = "photo";
        let ext = "png";
        let path = Some(Path::new("/a/b/photo.jpg"));

        // {stem}_web.{ext}
        assert_eq!(
            expand_template("{stem}_web.{ext}", stem, ext, path),
            "photo_web.png"
        );
        // {name} → file name from path
        assert_eq!(expand_template("{name}", stem, ext, path), "photo.jpg");
        // {parent} → last component of parent
        assert_eq!(expand_template("{parent}", stem, ext, path), "b");
        // Unknown {token} left literal
        assert_eq!(expand_template("{unknown}", stem, ext, path), "{unknown}");
        // path = None → {name} falls back to "stem.ext"
        assert_eq!(expand_template("{name}", stem, ext, None), "photo.png");
        // path = None → {parent} falls back to ""
        assert_eq!(expand_template("{parent}", stem, ext, None), "");
    }

    // ── safe_join ──────────────────────────────────────────────────────────

    #[test]
    fn safe_join_accepts_in_dir_name() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let result = safe_join(dir, "photo.png").unwrap();
        assert!(result.starts_with(std::fs::canonicalize(dir).unwrap()));
        assert_eq!(result.file_name().unwrap(), "photo.png");
    }

    #[test]
    fn safe_join_rejects_parent_escape() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        // ../x.png
        assert!(matches!(
            safe_join(dir, "../x.png"),
            Err(SinkError::Traversal(_))
        ));
        // ../../etc/passwd
        assert!(matches!(
            safe_join(dir, "../../etc/passwd"),
            Err(SinkError::Traversal(_))
        ));
        // Absolute path
        assert!(matches!(
            safe_join(dir, "/etc/x.png"),
            Err(SinkError::Traversal(_))
        ));
    }
}
