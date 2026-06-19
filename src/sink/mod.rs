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

// ── AVIF encode constants (SPEC-018, DEC-020) ─────────────────────────────────

/// The fixed `rav1e` encode speed for AVIF output (1=slowest/best … 10=fastest).
/// 6 is a balanced default that keeps encodes (and tests) reasonably fast; a
/// per-invocation `--speed` knob is deferred (DEC-020). It MUST match the speed
/// used by `crate::quality::encode_candidate_bytes`'s AVIF arm so a probed
/// candidate's byte length equals the bytes this sink writes (the cross-sync
/// contract, now covering AVIF as well as JPEG — DEC-016/DEC-019).
#[cfg(feature = "avif")]
pub const AVIF_SPEED: u8 = 6;

/// The default AVIF quality (1–100, 100=best) when `-q` is omitted (DEC-020).
/// AVIF quality numbers are NOT comparable to JPEG's; use `--target`/`--ssim`/
/// `--max-size` to ask for an outcome rather than a raw number.
#[cfg(feature = "avif")]
pub const AVIF_DEFAULT_QUALITY: u8 = 80;

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

    /// The requested output format's codec is recognized but was not compiled
    /// into this build (a feature-gated codec, e.g. AVIF without `--features
    /// avif`). Maps to exit 4 (DEC-004). The message names the codec and the
    /// feature to rebuild with.
    #[error("{codec} support is not built; rebuild with --features {feature}")]
    CodecNotBuilt {
        codec: &'static str,
        feature: &'static str,
    },
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
        // WebP is a pure-Rust DEFAULT format (SPEC-019, DEC-021): decode (input)
        // + lossless encode. The lossless encode is reached via the default
        // `write_to` path in `encode_to_bytes` (no special arm). Lossy WebP
        // encode is the off-by-default `webp-lossy` feature (SPEC-020).
        "webp" => Ok(ImageFormat::WebP),
        // AVIF is recognized as a format REGARDLESS of the `avif` feature, so an
        // `.avif`/`--format avif` request resolves to a clear "codec not built"
        // error (exit 4) when the feature is off — not a vague "unsupported
        // extension" (SPEC-018, DEC-004/DEC-020). The actual encode is gated in
        // `encode_to_bytes` / `ensure_codec_built`.
        "avif" => Ok(ImageFormat::Avif),
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

/// Verify the encoder for `format` is compiled into this build (DEC-004).
///
/// A format whose extension is recognized but whose codec is feature-gated and
/// OFF (today only AVIF, behind `--features avif`) returns
/// [`SinkError::CodecNotBuilt`]; every always-built core format returns
/// `Ok(())`. Callers that resolve the output format UP FRONT — `convert`, which
/// must fail with a SINGLE exit 4 before any per-input fan-out rather than a
/// partial-batch exit 6 (DEC-004 / DEC-015) — use this to surface the unbuilt
/// codec before loading inputs. The per-write `encode_to_bytes` guard is the
/// belt-and-suspenders backstop for callers that do not pre-check.
pub fn ensure_codec_built(format: ImageFormat) -> Result<(), SinkError> {
    match format {
        #[cfg(not(feature = "avif"))]
        ImageFormat::Avif => Err(SinkError::CodecNotBuilt {
            codec: "avif",
            feature: "avif",
        }),
        _ => Ok(()),
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
    /// - `quality` is an optional encode quality (0–100). Applied to JPEG
    ///   output only; ignored for lossless formats (DEC-016).
    /// - `out` receives the encoded bytes for [`Sink::Stdout`] ONLY. The
    ///   injected writer exists so tests can capture bytes without touching
    ///   real stdout. Diagnostics never go to `out`.
    pub fn write(
        &self,
        img: &Image,
        input: &SinkInput<'_>,
        overwrite: Overwrite,
        quality: Option<u8>,
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
                let bytes = encode_to_bytes(img, fmt, quality)?;
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
                let bytes = encode_to_bytes(img, fmt, quality)?;
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
                let bytes = encode_to_bytes(img, fmt, quality)?;
                out.write_all(&bytes)?;
                Ok(())
            }

            Sink::Display { width, height } => {
                // quality is intentionally ignored for terminal display (DEC-016).
                let _ = quality;
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
                    // than silently succeeding. `display` is ON by default
                    // (DEC-027); this branch is only reached on a deliberate
                    // `--no-default-features` build, so point at the fix.
                    Err(SinkError::Display(
                        "this binary was built --no-default-features, so terminal \
                         display is not compiled in; rebuild with the `display` feature \
                         (e.g. `cargo build --features display` or `just view <image>`)"
                            .into(),
                    ))
                }
            }
        }
    }
}

// ── Raw-bytes write path (container lane, SPEC-026) ───────────────────────────

impl Sink {
    /// Write already-encoded container `bytes` verbatim — the **container-lane**
    /// write path (SPEC-026, DEC-003). Unlike [`Sink::write`], this NEVER
    /// re-encodes pixels: the bytes are the output of `metadata::strip_all` /
    /// `clean_gps`, where the format is preserved and the compressed image data
    /// is carried through untouched (`metadata-not-via-pixel-encode`).
    ///
    /// Output shapes (only the file-producing variants are valid here):
    /// - [`Sink::File`] → write to the explicit path (extension already correct).
    /// - [`Sink::Dir`] → expand `template` over `input`, using `ext` (the input's
    ///   own extension, since the format is preserved) for `{ext}`; traversal- and
    ///   overwrite-guarded exactly like [`Sink::write`].
    /// - [`Sink::Stdout`] → write the raw bytes to `out` (pipe-friendly default).
    /// - [`Sink::Display`] → not a byte sink; rejected with [`SinkError::Display`].
    ///
    /// `ext` is the lowercase extension string for `{ext}` expansion in a `Dir`
    /// template (the `format` field of the sink is ignored — the container lane
    /// preserves the input format). Overwrite is guarded by `overwrite`.
    pub fn write_bytes(
        &self,
        bytes: &[u8],
        input: &SinkInput<'_>,
        ext: &str,
        overwrite: Overwrite,
        out: &mut dyn Write,
    ) -> Result<(), SinkError> {
        match self {
            Sink::File { path, .. } => {
                guard_overwrite(path, overwrite)?;
                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)?;
                let mut writer = BufWriter::new(file);
                writer.write_all(bytes)?;
                Ok(())
            }
            Sink::Dir { dir, template, .. } => {
                let file_name = expand_template(template, input.stem, ext, input.path);
                let full_path = safe_join(dir, &file_name)?;
                guard_overwrite(&full_path, overwrite)?;
                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&full_path)?;
                let mut writer = BufWriter::new(file);
                writer.write_all(bytes)?;
                Ok(())
            }
            Sink::Stdout { .. } => {
                out.write_all(bytes)?;
                Ok(())
            }
            Sink::Display { .. } => Err(SinkError::Display(
                "the metadata lane writes container bytes; terminal display is not a byte sink"
                    .into(),
            )),
        }
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Encode `img` pixels to a `Vec<u8>` in `format`, optionally at a specific
/// quality level.
///
/// Uses an in-memory `Cursor<Vec<u8>>` as the writer so all encoders have the
/// `Write + Seek` bound they may require. The resulting bytes are then written
/// to the actual destination (file or `out`) by the caller.
///
/// When `format == ImageFormat::Jpeg` and `quality == Some(q)`, the JPEG is
/// encoded via `JpegEncoder::new_with_quality` with `q` clamped to `1..=100`
/// (DEC-016). For all other `(format, quality)` combinations, `quality` is
/// ignored and the default `write_to` path is used (lossless formats have no
/// 0–100 quality knob).
pub fn encode_to_bytes(
    img: &Image,
    format: ImageFormat,
    quality: Option<u8>,
) -> Result<Vec<u8>, SinkError> {
    let mut cursor = Cursor::new(Vec::new());

    if format == ImageFormat::Jpeg {
        if let Some(q) = quality {
            // Clamp to 1..=100 (JPEG quality range; avoids surprising values).
            // NOTE: `crate::quality::encode_candidate_bytes` (the auto-quality /
            // byte-budget search, SPEC-016/017) re-implements this exact JPEG encode
            // to probe candidates. Keep the two in sync — see the contract comment
            // there (DEC-016).
            let q = q.clamp(1, 100);
            let encoder = ::image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, q);
            img.pixels()
                .write_with_encoder(encoder)
                .map_err(|e| SinkError::Encode(e.to_string()))?;
            return Ok(cursor.into_inner());
        }
    }

    if format == ImageFormat::Avif {
        // AVIF output is feature-gated (SPEC-018, DEC-020). With the feature on,
        // encode via `ravif` at the fixed AVIF_SPEED and the requested quality
        // (default 80). This encode MUST stay identical to
        // `crate::quality::encode_candidate_bytes`'s AVIF arm so the auto-quality
        // / byte-budget search probes match the bytes written here (DEC-019).
        #[cfg(feature = "avif")]
        {
            let q = quality.unwrap_or(AVIF_DEFAULT_QUALITY).clamp(1, 100);
            let encoder = ::image::codecs::avif::AvifEncoder::new_with_speed_quality(
                &mut cursor,
                AVIF_SPEED,
                q,
            );
            img.pixels()
                .write_with_encoder(encoder)
                .map_err(|e| SinkError::Encode(e.to_string()))?;
            return Ok(cursor.into_inner());
        }
        // Without the feature, AVIF output is a clear "codec not built" → exit 4
        // (DEC-004). `run_convert` resolves the format up front via
        // `ensure_codec_built`, so this is the belt-and-suspenders path for any
        // other caller (e.g. `shrink -o x.avif`).
        #[cfg(not(feature = "avif"))]
        {
            return Err(SinkError::CodecNotBuilt {
                codec: "avif",
                feature: "avif",
            });
        }
    }

    // WebP: LOSSY when a quality is set AND the `webp-lossy` feature is built
    // (SPEC-020, DEC-022) — via libwebp. Otherwise (no feature, or no quality)
    // fall through to the default `write_to` path, which writes LOSSLESS WebP
    // (SPEC-019). This encode MUST stay identical to
    // `crate::quality::encode_candidate_bytes`'s WebP arm so the auto-quality /
    // byte-budget search probes match the bytes written here (DEC-019/DEC-020).
    #[cfg(feature = "webp-lossy")]
    if format == ImageFormat::WebP {
        if let Some(q) = quality {
            let rgba = img.pixels().to_rgba8();
            let (w, h) = rgba.dimensions();
            let encoder = ::webp::Encoder::from_rgba(rgba.as_raw(), w, h);
            let memory = encoder.encode(q.clamp(1, 100) as f32);
            return Ok(memory.to_vec());
        }
        // quality == None → lossless (fall through to write_to below).
    }

    // All other (format, quality) cases: use the default write_to path.
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

    // ── AVIF (SPEC-018) ─────────────────────────────────────────────────────

    /// AVIF is recognized as an output format regardless of the feature, so the
    /// error surfaces as a clear "codec not built" rather than "unsupported
    /// extension" (DEC-004/DEC-020). Runs in the DEFAULT build.
    #[test]
    fn format_from_extension_recognizes_avif() {
        assert_eq!(
            format_from_extension(Path::new("x.avif")).unwrap(),
            ImageFormat::Avif
        );
    }

    /// WebP is a recognized output format (SPEC-019). It is built by default, so
    /// the lossless encode path (`write_to`) handles it without a `CodecNotBuilt`.
    #[test]
    fn format_from_extension_recognizes_webp() {
        assert_eq!(
            format_from_extension(Path::new("x.webp")).unwrap(),
            ImageFormat::WebP
        );
    }

    /// Build a structured image whose lossy size responds to the quality knob.
    #[cfg(any(feature = "avif", feature = "webp-lossy"))]
    fn detailed_image(w: u32, h: u32) -> Image {
        use ::image::{DynamicImage, RgbImage};
        let mut img = RgbImage::new(w, h);
        for (x, y, px) in img.enumerate_pixels_mut() {
            let gx = (x * 255 / w.max(1)) as i32;
            let gy = (y * 255 / h.max(1)) as i32;
            let tex = if ((x / 8) + (y / 8)) % 2 == 0 { 30 } else { 0 };
            let r = (gx + tex).clamp(0, 255) as u8;
            let g = (gy + tex).clamp(0, 255) as u8;
            let b = ((gx + gy) / 2).clamp(0, 255) as u8;
            *px = ::image::Rgb([r, g, b]);
        }
        let mut buf = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, ImageFormat::Png)
            .unwrap();
        Image::from_bytes(&buf.into_inner()).unwrap()
    }

    /// With the feature on, AVIF encodes valid AVIF bytes and the quality knob
    /// works: q30 is smaller than q90, and both magic-detect as AVIF.
    #[cfg(feature = "avif")]
    #[test]
    fn encode_avif_respects_quality() {
        let img = detailed_image(96, 96);
        let low = encode_to_bytes(&img, ImageFormat::Avif, Some(30)).expect("encode q30");
        let high = encode_to_bytes(&img, ImageFormat::Avif, Some(90)).expect("encode q90");

        assert_eq!(
            ::image::guess_format(&low).unwrap(),
            ImageFormat::Avif,
            "q30 output should be AVIF"
        );
        assert_eq!(
            ::image::guess_format(&high).unwrap(),
            ImageFormat::Avif,
            "q90 output should be AVIF"
        );
        assert!(
            low.len() < high.len(),
            "lower quality should produce fewer bytes: q30={} q90={}",
            low.len(),
            high.len()
        );
    }

    /// With the feature on, a WebP encode WITH a quality is lossy and the knob
    /// works: q30 is smaller than q90, and both magic-detect as WebP.
    #[cfg(feature = "webp-lossy")]
    #[test]
    fn encode_webp_lossy_respects_quality() {
        let img = detailed_image(96, 96);
        let low = encode_to_bytes(&img, ImageFormat::WebP, Some(30)).expect("encode q30");
        let high = encode_to_bytes(&img, ImageFormat::WebP, Some(90)).expect("encode q90");

        assert_eq!(
            ::image::guess_format(&low).unwrap(),
            ImageFormat::WebP,
            "q30 output should be WebP"
        );
        assert_eq!(
            ::image::guess_format(&high).unwrap(),
            ImageFormat::WebP,
            "q90 output should be WebP"
        );
        assert!(
            low.len() < high.len(),
            "lower quality should produce fewer bytes: q30={} q90={}",
            low.len(),
            high.len()
        );
    }
}
